use super::protocol::{Cmd, Request};
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

mod selection_cache {
    include!("selection_cache.rs");
}

pub fn prime_selection_tools(parent: &Path) -> io::Result<()> {
    selection_cache::prime_tools(parent).ok_or_else(|| error("selection tools are not a stable admitted source package"))
}

fn error(message: &str) -> io::Error {
    io::Error::other(message)
}

fn publish(artifact: &Path, destination: &Path) -> io::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temporary(PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let mut input = std::fs::File::open(artifact)?;
    let (temporary, mut output) = loop {
        let path = destination.with_file_name(format!(".lustr-artifact-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break (Temporary(path), file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    io::copy(&mut input, &mut output)?;
    output.set_permissions(input.metadata()?.permissions())?;
    drop(output);
    // A cancellation can leave a temporary sibling, but cannot publish a
    // partially copied executable or overwrite a caller artifact mid-copy.
    std::fs::rename(&temporary.0, destination)
}

fn cargo(parent: &Path) -> Command {
    let mut command = Command::new(parent.join("cargo"));
    command.env("RUSTC", parent.join("rustc-full")).env("RUSTDOC", parent.join("rustdoc"));
    if std::env::var_os("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT").is_none() {
        command.env("__CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT", "1");
    }
    command
}

fn settings(request: &Request, command: &mut Command) {
    let p = request.session.profile;
    command.args(["-C", if p.panic_abort { "panic=abort" } else { "panic=unwind" }]);
    command.args(["-C", if p.overflow_checks { "overflow-checks=on" } else { "overflow-checks=off" }]);
    command.args(["-C", if p.debug_assertions { "debug-assertions=on" } else { "debug-assertions=off" }]);
    if request.debug {
        command.arg("-g");
    }
}

pub struct Selection {
    package: Value,
    target: Value,
    pub manifest: PathBuf,
    pub name: String,
}

fn selector<'a>(args: &'a [String], names: &[&str]) -> Option<&'a str> {
    let mut selected = None;
    // prepare() already parsed this exact native CLI. Skip its command and
    // source, and consume values as the upstream parser does: an output path
    // such as '--bin=other' is a value, never another selector.
    let mut args = args.iter().skip(2);
    while let Some(arg) = args.next() {
        if arg == "--" {
            break;
        }
        if names.contains(&arg.as_str()) {
            selected = args.next().map(String::as_str);
        } else if matches!(
            arg.as_str(),
            "-o" | "--emit-c"
                | "--profile"
                | "--max-errors"
                | "--manifest-path"
                | "--package"
                | "-p"
                | "--bin"
                | "--features"
                | "-C"
                | "--target"
        ) {
            args.next();
        } else if let Some((key, value)) = arg.split_once('=') {
            if names.contains(&key) {
                selected = Some(value);
            }
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    #[test]
    fn selector_values_cannot_become_selectors() {
        let args = [
            "build",
            "file.rs",
            "--bin",
            "first",
            "-o",
            "--bin=wrong",
            "--features",
            "--package=wrong",
            "--bin=second",
            "--package",
            "real@0.2.0",
            "--",
            "--bin=ignored",
        ]
        .map(str::to_owned);
        assert_eq!(super::selector(&args, &["--bin"]), Some("second"));
        assert_eq!(super::selector(&args, &["--package", "-p"]), Some("real@0.2.0"));
    }
}

fn output(command: &mut Command) -> io::Result<std::process::Output> {
    let result = crate::owned_child::output(command)?;
    if !result.status.success() {
        io::stderr().write_all(&result.stderr)?;
        return Err(error("Cargo rejected the package or manifest selection"));
    }
    Ok(result)
}

pub fn select(request: &Request, parent: &Path, args: &[String]) -> io::Result<Selection> {
    let manifest = request.session.krate.manifest_path.as_ref().ok_or_else(|| error("missing Cargo manifest"))?;
    let before = selection_cache::Snapshot::capture(request, parent, args);
    if let Some((metadata, package_id)) = before.as_ref().and_then(selection_cache::Snapshot::load) {
        // Cached Cargo output passes the same identity and uniqueness filter as
        // fresh Cargo output. The retained native graph is not a substitute.
        if let Ok(selected) = select_metadata(request, args, &metadata, package_id.as_deref()) {
            selection_cache::diagnostic("hit");
            return Ok(selected);
        }
    }
    selection_cache::diagnostic(if before.is_some() { "miss" } else { "ineligible" });
    let metadata = output(
        cargo(parent).args(["metadata", "--format-version=1", "--manifest-path"]).arg(manifest).args(request.session.cargo_net.args()),
    )?;
    let metadata: Value = serde_json::from_slice(&metadata.stdout)?;
    let package_id = if let Some(spec) = selector(args, &["--package", "-p"]) {
        // Cargo owns package-spec syntax, including versions, URLs and
        // workspace-inherited identities. Do not split a synthetic Lust id.
        let result = output(
            cargo(parent).args(["pkgid", "--manifest-path"]).arg(manifest).args(["--package", spec]).args(request.session.cargo_net.args()),
        )?;
        Some(String::from_utf8(result.stdout).map_err(|_| error("non-UTF8 Cargo package id"))?.trim().to_owned())
    } else {
        None
    };
    let selected = select_metadata(request, args, &metadata, package_id.as_deref())?;
    if let Some(before) = before {
        if let Some(after) = selection_cache::Snapshot::capture(request, parent, args) {
            if before == after {
                after.store(&metadata, package_id.as_deref());
            }
        }
    }
    Ok(selected)
}

fn select_metadata(request: &Request, args: &[String], metadata: &Value, package_id: Option<&str>) -> io::Result<Selection> {
    let name = selector(args, &["--bin"]);
    let source = request.file.canonicalize()?;
    let selected: Vec<_> = metadata["packages"]
        .as_array()
        .ok_or_else(|| error("Cargo metadata has no packages"))?
        .iter()
        .filter(|p| package_id.is_none_or(|id| p["id"] == id))
        .flat_map(|package| package["targets"].as_array().into_iter().flatten().map(move |target| (package, target)))
        .filter(|(_, target)| name.is_none_or(|name| target["name"] == name))
        .filter(|(_, target)| target["src_path"].as_str().and_then(|p| Path::new(p).canonicalize().ok()).as_ref() == Some(&source))
        .filter(|(_, target)| {
            target["kind"].as_array().is_some_and(|kinds| {
                kinds.iter().any(|k| {
                    if request.session.krate.target_kind == "bin" {
                        k == "bin"
                    } else {
                        matches!(k.as_str(), Some("lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro"))
                    }
                })
            })
        })
        .collect();
    if selected.len() != 1 {
        return Err(error("package, target name, kind and source do not identify exactly one Cargo target"));
    }
    let (package, target) = selected[0];
    Ok(Selection {
        manifest: package["manifest_path"].as_str().map(PathBuf::from).ok_or_else(|| error("Cargo package has no manifest"))?,
        name: target["name"].as_str().ok_or_else(|| error("Cargo target has no name"))?.to_owned(),
        package: package.clone(),
        target: target.clone(),
    })
}

pub fn run(request: &Request, parent: &Path, selection: Option<&Selection>) -> io::Result<ExitStatus> {
    if request.emit_c.is_some() {
        return Err(error("this input needs the full Rust core, which cannot emit Lust C units"));
    }
    let Some(manifest) = request.session.krate.manifest_path.as_ref() else {
        let mut command = Command::new(parent.join("rustc-full"));
        command.arg(request.client.shown.as_ref().unwrap_or(&request.file)).arg("--edition=2024");
        command.arg("-o").arg(if matches!(request.cmd, Cmd::Check) { Path::new("/dev/null") } else { &request.out });
        if matches!(request.cmd, Cmd::Check) {
            command.arg("--emit=metadata");
        }
        if request.session.krate.target_kind == "lib" {
            command.arg("--crate-type=rlib");
        }
        if !request.session.target_triple.is_empty() {
            command.arg("--target").arg(&request.session.target_triple);
        }
        for feature in &request.session.features {
            command.arg("--cfg").arg(format!("feature={feature:?}"));
        }
        if request.json {
            command.arg("--error-format=json");
        }
        settings(request, &mut command);
        return Err(command.exec());
    };
    let Selection { package, target, .. } = selection.ok_or_else(|| error("Cargo target was not validated"))?;
    let mut command = cargo(parent);
    command
        .arg("rustc")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--package")
        .arg(package["id"].as_str().ok_or_else(|| error("Cargo package has no id"))?)
        .arg("--message-format=json-render-diagnostics")
        .args(request.session.cargo_net.args());
    if request.session.krate.target_kind == "bin" {
        command.arg("--bin").arg(target["name"].as_str().ok_or_else(|| error("Cargo target has no name"))?);
    } else {
        command.arg("--lib");
    }
    if !request.session.default_features {
        command.arg("--no-default-features");
    }
    if request.session.all_features {
        command.arg("--all-features");
    }
    if !request.session.features.is_empty() {
        command.arg("--features").arg(request.session.features.join(","));
    }
    if !request.session.target_triple.is_empty() {
        command.arg("--target").arg(&request.session.target_triple);
    }
    if matches!(request.cmd, Cmd::Check) {
        command.args(["--profile", "check"]);
    }
    command.arg("--");
    settings(request, &mut command);
    let mut child = crate::owned_child::OwnedChild::spawn(command.stdout(Stdio::piped()).stderr(Stdio::inherit()))?;
    let mut artifact = None;
    for line in BufReader::new(child.stdout().unwrap()).lines() {
        let line = line?;
        let message: Value = serde_json::from_str(&line)?;
        match message["reason"].as_str() {
            Some("compiler-message") => {
                if request.json {
                    println!("{}", message["message"]);
                } else if let Some(rendered) = message["message"]["rendered"].as_str() {
                    eprint!("{rendered}");
                }
            }
            Some("compiler-artifact")
                if message["package_id"] == package["id"]
                    && message["target"]["name"] == target["name"]
                    && message["target"]["kind"] == target["kind"]
                    && message["target"]["src_path"] == target["src_path"] =>
            {
                artifact = message["executable"].as_str().map(std::path::PathBuf::from).or_else(|| {
                    message["filenames"]
                        .as_array()?
                        .iter()
                        .filter_map(Value::as_str)
                        .find(|p| p.ends_with(".rlib"))
                        .map(std::path::PathBuf::from)
                });
            }
            _ => {}
        }
    }
    let status = child.wait()?;
    if status.success() && matches!(request.cmd, Cmd::Build) {
        let artifact = artifact.ok_or_else(|| error("Cargo succeeded without the requested executable or Rust library artifact"))?;
        if artifact != request.out {
            publish(&artifact, &request.out)?;
        }
    }
    Ok(status)
}
