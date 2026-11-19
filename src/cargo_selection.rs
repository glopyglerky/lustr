// Cargo selects the unchanged package and target. This module cannot compile
// a rejected native request or publish a Rust artifact.
use super::protocol::Request;
use serde_json::Value;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

mod selection_cache {
    include!("selection_cache.rs");
}

pub fn prime_selection_tools(parent: &Path) -> io::Result<()> {
    selection_cache::prime_tools(parent).ok_or_else(|| error("selection tools are not a stable admitted source package"))
}

fn error(message: &str) -> io::Error {
    io::Error::other(message)
}

fn cargo(parent: &Path) -> Command {
    let mut command = Command::new(parent.join("cargo"));
    command.env("RUSTC", parent.join("rustc")).env("RUSTC_WRAPPER", "").env("RUSTC_WORKSPACE_WRAPPER", "");
    command
}

pub struct Selection {
    package: Value,
    target: Value,
    pub manifest: PathBuf,
    pub name: String,
}

impl Selection {
    fn retained(&self) -> Value {
        serde_json::json!({"manifest":self.manifest, "name":self.name, "package":self.package, "target":self.target})
    }

    fn from_retained(mut value: Value) -> Option<Self> {
        if value.as_object()?.len() != 4 {
            return None;
        }
        let manifest = PathBuf::from(value["manifest"].as_str()?);
        let name = value["name"].as_str()?.to_owned();
        let package = value.get_mut("package")?.take();
        let target = value.get_mut("target")?.take();
        if !package.is_object()
            || !target.is_object()
            || package["manifest_path"].as_str()? != manifest.to_str()?
            || target["name"].as_str()? != name
            || package["id"].as_str()?.is_empty()
            || target["src_path"].as_str()?.is_empty()
            || target["kind"].as_array()?.is_empty()
            || target["kind"].as_array()?.iter().any(|kind| !kind.is_string())
            || !package["targets"].as_array()?.contains(&target)
        {
            return None;
        }
        Some(Self { package, target, manifest, name })
    }
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
    if let Some(selected) = before.as_ref().and_then(selection_cache::Snapshot::load) {
        // Only the successful Cargo admission and unique selection below can
        // create this record. Exact input and every target binding still agree.
        selection_cache::diagnostic("hit");
        return Ok(selected);
    }
    selection_cache::diagnostic(if before.is_some() { "miss" } else { "ineligible" });
    let metadata = output(
        cargo(parent)
            .args(["metadata", "--no-deps", "--format-version=1", "--manifest-path"])
            .arg(manifest)
            .args(request.session.cargo_net.args()),
    )?;
    let metadata: Value = serde_json::from_slice(&metadata.stdout)?;
    // The retained native Cargo probe compiles host/build-script/proc-macro
    // units with a complete compiler. Until those obligations have a selective
    // complementary owner, only its proven no-probe graph may enter Lustr.
    let members = metadata["workspace_members"].as_array().ok_or_else(|| error("Cargo metadata has no workspace members"))?;
    if members.len() != 1 {
        return Err(error("native Cargo integration is incomplete for a multi-member Cargo graph"));
    }
    let packages = metadata["packages"].as_array().ok_or_else(|| error("Cargo metadata has no packages"))?;
    if packages.iter().any(|package| {
        package["dependencies"].as_array().is_none_or(|dependencies| !dependencies.is_empty())
            || package["targets"].as_array().is_none_or(|targets| {
                targets.iter().any(|target| {
                    target["kind"].as_array().is_none_or(|kinds| kinds.iter().any(|kind| kind == "custom-build" || kind == "proc-macro"))
                })
            })
    }) {
        return Err(error("native Cargo integration is incomplete for dependencies, build scripts or proc macros"));
    }
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
                after.store(&metadata, &selected);
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
