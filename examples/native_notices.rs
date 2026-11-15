//! Collect original notice files from checksum-pinned registry archives.
//! The input inventory is conservative source reachability, not runtime linkage.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};
use std::process::{Command, Output};

fn checked(command: &mut Command) -> Output {
    let output = command.output().unwrap();
    assert!(output.status.success(), "command failed: {command:?}: {}", String::from_utf8_lossy(&output.stderr));
    output
}

fn digest(path: &Path) -> String {
    String::from_utf8(checked(Command::new("sha256sum").arg(path)).stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}

fn download(url: &str, path: &Path) {
    if path.is_file() {
        return;
    }
    let temporary = path.with_extension("download");
    checked(
        Command::new("wget")
            .args(["--no-config", "--https-only", "--timeout=45", "--tries=1", "--no-hsts", "--output-document"])
            .arg(&temporary)
            .arg(url),
    );
    assert!(temporary.is_file(), "download returned no file: {url}");
    fs::rename(temporary, path).unwrap();
}

fn vcs_notices(registry: &Path, output: &Path, overrides: &Value) {
    fs::create_dir_all(output.join("trees")).unwrap();
    let inventory: Value = serde_json::from_slice(&fs::read(registry.join("inventory.json")).unwrap()).unwrap();
    let mut packages = Vec::new();
    for package in inventory["packages"].as_array().unwrap() {
        let prefix = format!("{}-{}", package["name"].as_str().unwrap(), package["version"].as_str().unwrap());
        let archive = registry.join(format!("{prefix}.crate"));
        assert_eq!(digest(&archive), package["archive_sha256"].as_str().unwrap());
        let member = |name: &str| checked(Command::new("tar").arg("-xOzf").arg(&archive).arg(format!("{prefix}/{name}"))).stdout;
        let listed = String::from_utf8(checked(Command::new("tar").arg("-tzf").arg(&archive)).stdout).unwrap();
        if !listed.lines().any(|p| p == format!("{prefix}/.cargo_vcs_info.json")) {
            let row = json!({"package":package,"status":"unresolved: registry archive has no VCS commit metadata","notices":[]});
            fs::write(output.join(format!("{prefix}.json")), serde_json::to_vec_pretty(&row).unwrap()).unwrap();
            println!("{prefix}: unresolved VCS provenance");
            packages.push(row);
            continue;
        }
        let vcs_bytes = member(".cargo_vcs_info.json");
        let vcs: Value = serde_json::from_slice(&vcs_bytes).unwrap();
        let commit = vcs["git"]["sha1"].as_str().unwrap();
        assert!(commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()));
        let vcs_dirty = vcs["git"]["dirty"].as_bool().unwrap_or(false);
        let declared_crate_path = vcs["path_in_vcs"].as_str().unwrap_or("");
        assert!(Path::new(declared_crate_path).components().all(|c| matches!(c, Component::Normal(_))));
        let manifest_bytes = member("Cargo.toml");
        let manifest = std::str::from_utf8(&manifest_bytes).unwrap();
        let package_section = manifest.split("[package]").nth(1).unwrap().split("\n[").next().unwrap();
        let Some(repository) = package_section.lines().find_map(|line| line.strip_prefix("repository = \"")?.strip_suffix('"')) else {
            let row = json!({"package":package,"status":"unresolved: registry manifest names no VCS repository","vcs":vcs,"notices":[]});
            fs::write(output.join(format!("{prefix}.json")), serde_json::to_vec_pretty(&row).unwrap()).unwrap();
            println!("{prefix}: unresolved repository (dirty={vcs_dirty})");
            packages.push(row);
            continue;
        };
        let declared_repository_path = repository
            .strip_prefix("https://github.com/")
            .expect("review non-GitHub VCS provenance explicitly")
            .trim_end_matches('/')
            .trim_end_matches(".git");
        let repository_path = if let Some(mapping) = overrides.get(&prefix) {
            assert_eq!(mapping["declared_repository"].as_str(), Some(repository));
            assert_eq!(mapping["commit"].as_str(), Some(commit));
            mapping["repository_path"].as_str().unwrap()
        } else {
            declared_repository_path
        };
        assert!(
            repository_path.split('/').count() == 2
                && repository_path.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.'))
        );
        let tree_url = format!("https://api.github.com/repos/{repository_path}/git/trees/{commit}?recursive=1");
        let tree_file = output.join("trees").join(format!("{}-{commit}.json", repository_path.replace('/', "--")));
        download(&tree_url, &tree_file);
        let tree: Value = serde_json::from_slice(&fs::read(&tree_file).unwrap()).unwrap();
        assert_eq!(tree["truncated"].as_bool(), Some(false), "incomplete VCS tree");
        let destination = output.join("notices").join(&prefix);
        fs::create_dir_all(&destination).unwrap();
        let original_manifest = destination.join("registry-Cargo.toml.orig");
        fs::write(&original_manifest, member("Cargo.toml.orig")).unwrap();
        let manifest_blob =
            String::from_utf8(checked(Command::new("git").arg("hash-object").arg("--no-filters").arg(&original_manifest)).stdout).unwrap();
        let matching_manifests: Vec<_> = tree["tree"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["type"] == "blob"
                    && e["sha"] == manifest_blob.trim()
                    && Path::new(e["path"].as_str().unwrap()).file_name().is_some_and(|n| n == "Cargo.toml")
            })
            .collect();
        assert_eq!(matching_manifests.len(), 1, "exact registry manifest must identify one VCS package");
        let crate_path = Path::new(matching_manifests[0]["path"].as_str().unwrap()).parent().unwrap().to_str().unwrap();
        if !declared_crate_path.is_empty() {
            assert_eq!(crate_path, declared_crate_path, "recorded VCS subdirectory differs");
        }
        let mut notices = Vec::new();
        for entry in tree["tree"].as_array().unwrap() {
            if entry["type"] != "blob" || !matches!(entry["mode"].as_str(), Some("100644" | "100755")) {
                continue;
            }
            let relative = entry["path"].as_str().unwrap();
            let path = Path::new(relative);
            assert!(path.components().all(|c| matches!(c, Component::Normal(_))));
            let leaf = path.file_name().unwrap().to_str().unwrap().to_ascii_lowercase();
            if !["license", "licence", "copying", "copyright", "notice", "authors"].iter().any(|word| leaf.contains(word)) {
                continue;
            }
            let parent = path.parent().unwrap();
            // Include original notice files at the crate and every ancestor,
            // plus notice directories immediately under those locations.
            let legal_directory = parent
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| ["licenses", "licences", "notices"].contains(&n.to_ascii_lowercase().as_str()));
            let scope = if legal_directory { parent.parent().unwrap() } else { parent };
            if !Path::new(crate_path).starts_with(scope) {
                continue;
            }
            let url = format!("https://raw.githubusercontent.com/{repository_path}/{commit}/{relative}");
            let file = destination.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            download(&url, &file);
            let blob = String::from_utf8(checked(Command::new("git").arg("hash-object").arg("--no-filters").arg(&file)).stdout).unwrap();
            assert_eq!(blob.trim(), entry["sha"].as_str().unwrap(), "VCS blob identity mismatch");
            notices.push(json!({"path":relative,"source":url,"git_blob":blob.trim(),"sha256":digest(&file),"bytes":fs::metadata(&file).unwrap().len()}));
        }
        fs::write(destination.join("registry-Cargo.toml"), &manifest_bytes).unwrap();
        fs::write(destination.join("registry-vcs-info.json"), &vcs_bytes).unwrap();
        let row = json!({"package":package,"declared_repository":repository,"resolved_repository_path":repository_path,"commit":commit,"vcs_dirty":vcs_dirty,"declared_path_in_vcs":declared_crate_path,"resolved_path_in_vcs":crate_path,"crate_manifest_git_blob":manifest_blob.trim(),"tree_source":tree_url,"tree_sha256":digest(&tree_file),"notices":notices});
        fs::write(output.join(format!("{prefix}.json")), serde_json::to_vec_pretty(&row).unwrap()).unwrap();
        println!("{prefix}: {} exact-commit original notices", notices.len());
        packages.push(row);
        fs::write(output.join("inventory.json"), serde_json::to_vec_pretty(&json!({"scope":"Original notice text from exact VCS base commits recorded in checksum-verified registry archives. Dirty or absent VCS metadata and absent original notice files remain explicit limitations. Conservative source-lock scope, not an exact linked-runtime or complete legal-compliance claim.","registry_inventory_sha256":digest(&registry.join("inventory.json")),"packages":packages})).unwrap()).unwrap();
    }
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|a| a == "vcs") {
        assert!((3..=4).contains(&args.len()), "native_notices vcs REGISTRY_COLLECTION OUTPUT_DIRECTORY [REVIEWED_REPOSITORY_MAP_JSON]");
        let overrides = args.get(3).map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap()).unwrap_or(Value::Null);
        vcs_notices(Path::new(&args[1]), Path::new(&args[2]), &overrides);
        return;
    }
    assert_eq!(args.len(), 3, "native_notices COVERAGE_JSON RUST_CARGO_LOCK NEW_OUTPUT_DIRECTORY");
    let coverage: Value = serde_json::from_slice(&fs::read(&args[0]).unwrap()).unwrap();
    let lock = fs::read_to_string(&args[1]).unwrap();
    let output = Path::new(&args[2]);
    fs::create_dir(output).unwrap();
    fs::create_dir(output.join("notices")).unwrap();
    let mut checksums = BTreeMap::new();
    for package in lock.split("[[package]]").skip(1) {
        let field = |name: &str| package.lines().find_map(|line| line.strip_prefix(&format!("{name} = \""))?.strip_suffix('"'));
        if field("source").is_some_and(|s| s.starts_with("registry+")) {
            checksums.insert((field("name").unwrap(), field("version").unwrap()), field("checksum").unwrap());
        }
    }
    let mut inventory = Vec::new();
    for package in coverage["declarations_without_text_blocks"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap();
        let version = package["version"].as_str().unwrap();
        assert!(name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        assert!(version.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+')));
        let checksum = checksums[&(name, version)];
        let prefix = format!("{name}-{version}");
        let archive = output.join(format!("{prefix}.crate"));
        let url = format!("https://static.crates.io/crates/{name}/{name}-{version}.crate");
        checked(
            Command::new("wget")
                .args(["--no-config", "--https-only", "--timeout=45", "--tries=1", "--no-hsts", "--output-document"])
                .arg(&archive)
                .arg(&url),
        );
        assert_eq!(digest(&archive), checksum, "registry archive checksum mismatch: {prefix}");
        let listed = checked(Command::new("tar").arg("-tzf").arg(&archive));
        let members = String::from_utf8(listed.stdout).unwrap();
        let mut notices = Vec::new();
        for member in members.lines() {
            let path = Path::new(member);
            assert!(path.components().all(|c| matches!(c, Component::Normal(_))), "unsafe archive member");
            assert_eq!(path.components().next().unwrap().as_os_str(), prefix.as_str());
            let leaf = path.file_name().unwrap().to_str().unwrap().to_ascii_lowercase();
            if !["license", "licence", "copying", "copyright", "notice", "authors"].iter().any(|word| leaf.contains(word)) {
                continue;
            }
            let mode = checked(Command::new("tar").arg("-tvzf").arg(&archive).arg("--").arg(member));
            if mode.stdout.first() != Some(&b'-') {
                continue;
            }
            let bytes = checked(Command::new("tar").arg("-xOzf").arg(&archive).arg("--").arg(member)).stdout;
            let relative = path.strip_prefix(&prefix).unwrap();
            let destination = output.join("notices").join(&prefix).join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(&destination, &bytes).unwrap();
            notices.push(json!({"path":relative,"bytes":bytes.len(),"sha256":digest(&destination)}));
        }
        let row = json!({"name":name,"version":version,"declared_license":package["license"],"source":url,"archive_sha256":checksum,"notices":notices});
        fs::write(output.join(format!("{prefix}.json")), serde_json::to_vec_pretty(&row).unwrap()).unwrap();
        println!("{prefix}: {} original notice files", notices.len());
        inventory.push(row);
    }
    fs::write(output.join("inventory.json"), serde_json::to_vec_pretty(&json!({"rust_source":coverage["rust_source"],"cargo_lock_blob":coverage["cargo_lock_blob"],"coverage_input_sha256":digest(Path::new(&args[0])),"cargo_lock_sha256":digest(Path::new(&args[1])),"scope":"Checksum-verified original registry notice files for declaration-only sections in the conservative source-lock closure. Includes optional/dev/other-target packages; not an exact runtime-linkage or universal legal-compliance claim.","packages":inventory})).unwrap()).unwrap();
}
