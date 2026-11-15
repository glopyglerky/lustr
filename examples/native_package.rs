//! Explicit source export, complete manifests, and raw package audits.
//! Reports are private until their paths and findings have been reviewed.
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

fn output(command: &mut Command) -> Vec<u8> {
    let result = command.output().unwrap();
    assert!(result.status.success(), "command failed: {command:?}: {}", String::from_utf8_lossy(&result.stderr));
    result.stdout
}

fn digest(path: &Path) -> String {
    String::from_utf8(output(Command::new("sha256sum").arg("--").arg(path))).unwrap()[..64].to_owned()
}

fn save(path: &Path, value: &Value) {
    let mut file = fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap();
    file.write_all(&serde_json::to_vec_pretty(value).unwrap()).unwrap();
    file.sync_all().unwrap();
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn safe_link(name: &Path, target: &Path, hard: bool) {
    assert!(!target.is_absolute(), "absolute archive link");
    let mut depth = if hard { 0 } else { name.parent().unwrap().components().count() };
    for part in target.components() {
        match part {
            Component::Normal(_) => depth += 1,
            Component::CurDir => (),
            Component::ParentDir => {
                assert!(depth > 0, "link leaves package");
                depth -= 1;
            }
            _ => panic!("invalid link"),
        }
    }
}

fn entries(root: &Path) -> Vec<Value> {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<Value>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let name = path.strip_prefix(root).unwrap();
        let name = if name.as_os_str().is_empty() { Path::new(".") } else { name };
        let mut row = json!({"path":name,"mode":format!("0o{:o}",metadata.mode() & 0o7777)});
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(path).unwrap();
            safe_link(name, &target, false);
            row["type"] = json!("symlink");
            row["target"] = json!(target);
        } else if metadata.is_file() {
            row["type"] = json!("file");
            row["bytes"] = json!(metadata.len());
            row["sha256"] = json!(digest(path));
        } else {
            assert!(metadata.is_dir(), "nonregular package member");
            row["type"] = json!("directory");
        }
        rows.push(row);
        if metadata.is_dir() {
            let mut children: Vec<_> = fs::read_dir(path).unwrap().map(|e| e.unwrap().path()).collect();
            children.sort();
            for child in children {
                visit(root, &child, rows);
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows
}

fn patterns(policy: &Path) -> Vec<Vec<u8>> {
    let policy: Value = serde_json::from_slice(&fs::read(policy).unwrap()).unwrap();
    let mut values: BTreeSet<String> =
        policy["observed_private_prefixes"].as_array().unwrap().iter().map(|p| p.as_str().unwrap().to_owned()).collect();
    for entry in policy["ordered_rustc_remaps"].as_array().unwrap() {
        values.insert(entry["source"].as_str().unwrap().to_owned());
    }
    if let Ok(home) = std::env::var("HOME") {
        if home.len() > 1 {
            values.insert(home);
        }
    }
    values.into_iter().map(String::into_bytes).collect()
}

struct Scan<R> {
    reader: R,
    patterns: Vec<Vec<u8>>,
    tail: Vec<u8>,
    position: u64,
    hits: Vec<Value>,
}
impl<R: Read> Read for Scan<R> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(bytes)?;
        if n == 0 {
            return Ok(0);
        }
        let mut joined = std::mem::take(&mut self.tail);
        let previous = joined.len();
        joined.extend_from_slice(&bytes[..n]);
        for (i, pattern) in self.patterns.iter().enumerate() {
            for (at, _) in joined.windows(pattern.len()).enumerate().filter(|(_, s)| *s == pattern.as_slice()) {
                if at + pattern.len() > previous && self.hits.len() < 100 {
                    self.hits.push(json!({"kind":"private-prefix","pattern_index":i,"offset":self.position - previous as u64 + at as u64}));
                }
            }
        }
        for marker in
            [b"-----BEGIN PRIVATE KEY-----".as_slice(), b"-----BEGIN RSA PRIVATE KEY-----", b"-----BEGIN OPENSSH PRIVATE KEY-----"]
        {
            if let Some(at) = joined.windows(marker.len()).position(|s| s == marker) {
                if at + marker.len() > previous && self.hits.len() < 100 {
                    self.hits.push(json!({"kind":"private-key-marker","offset":self.position - previous as u64 + at as u64}));
                }
            }
        }
        for (prefix, minimum) in
            [(b"ghp_".as_slice(), 36), (b"gho_", 36), (b"ghu_", 36), (b"ghs_", 36), (b"github_pat_", 40), (b"sk-", 32), (b"AKIA", 20)]
        {
            for (at, _) in joined.windows(prefix.len()).enumerate().filter(|(_, s)| *s == prefix) {
                let end = at + minimum;
                if end <= joined.len()
                    && end > previous
                    && joined[at + prefix.len()..end].iter().all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'-'))
                    && self.hits.len() < 100
                {
                    self.hits.push(
                        json!({"kind":"token-shape-heuristic","offset":self.position-previous as u64+at as u64,"family_index":prefix[0]}),
                    );
                }
            }
        }
        let keep = self.patterns.iter().map(Vec::len).max().unwrap_or(1).max(128);
        self.tail = joined[joined.len().saturating_sub(keep)..].to_vec();
        self.position += n as u64;
        Ok(n)
    }
}
fn scan<R: Read>(reader: R, patterns: Vec<Vec<u8>>) -> Scan<R> {
    Scan { reader, patterns, tail: Vec::new(), position: 0, hits: Vec::new() }
}

fn field(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes.split(|b| *b == 0).next().unwrap()).unwrap()
}
fn number(bytes: &[u8]) -> u64 {
    let text = field(bytes).trim();
    if text.is_empty() { 0 } else { u64::from_str_radix(text, 8).unwrap() }
}

fn audit_archive(archive: &Path, policy: &Path) -> Value {
    let mut compressed = fs::File::open(archive).unwrap();
    let mut gzip_header = [0; 10];
    compressed.read_exact(&mut gzip_header).unwrap();
    assert_eq!(&gzip_header[..4], &[31, 139, 8, 0], "gzip name/comment/extra header present");
    assert_eq!(&gzip_header[4..8], &[0; 4]);
    let mut child = Command::new("gzip").arg("-dc").arg("--").arg(archive).stdout(Stdio::piped()).spawn().unwrap();
    let mut reader = scan(std::io::BufReader::new(child.stdout.take().unwrap()), patterns(policy));
    let mut count = 0;
    let mut pax = serde_json::Map::new();
    loop {
        let mut header = [0u8; 512];
        reader.read_exact(&mut header).unwrap();
        if header.iter().all(|b| *b == 0) {
            let mut tail = [0; 8192];
            let mut zero_bytes = 512;
            loop {
                let n = reader.read(&mut tail).unwrap();
                if n == 0 {
                    break;
                }
                assert!(tail[..n].iter().all(|b| *b == 0), "nonzero tar trailer");
                zero_bytes += n;
            }
            assert!(zero_bytes >= 1024 && pax.is_empty());
            break;
        }
        let checksum: u64 = header.iter().enumerate().map(|(i, b)| if (148..156).contains(&i) { 32 } else { *b as u64 }).sum();
        assert_eq!(number(&header[148..156]), checksum, "tar checksum");
        assert_eq!(number(&header[108..116]), 0);
        assert_eq!(number(&header[116..124]), 0);
        assert_eq!(number(&header[136..148]), 0);
        assert_eq!(field(&header[265..297]), "");
        assert_eq!(field(&header[297..329]), "");
        let prefix = field(&header[345..500]);
        let leaf = field(&header[..100]);
        let name = if prefix.is_empty() { leaf.to_owned() } else { format!("{prefix}/{leaf}") };
        assert!(relative(Path::new(&name)), "unsafe tar header name");
        let size = number(&header[124..136]);
        let kind = header[156];
        if kind == b'x' {
            assert!(pax.is_empty() && size < 1024 * 1024);
            let mut data = vec![0; size as usize];
            reader.read_exact(&mut data).unwrap();
            let mut at = 0;
            while at < data.len() {
                let space = data[at..].iter().position(|b| *b == b' ').unwrap() + at;
                let n: usize = std::str::from_utf8(&data[at..space]).unwrap().parse().unwrap();
                assert!(n > space - at + 1 && at + n <= data.len());
                let line = std::str::from_utf8(&data[space + 1..at + n]).unwrap().strip_suffix('\n').unwrap();
                let (key, value) = line.split_once('=').unwrap();
                assert!(["path", "linkpath"].contains(&key), "unapproved PAX metadata: {key}");
                assert!(pax.insert(key.into(), json!(value)).is_none());
                at += n;
            }
        } else {
            assert!([0, b'0', b'1', b'2', b'5'].contains(&kind), "unsupported archive member");
            let real_name = pax.get("path").and_then(Value::as_str).unwrap_or(&name);
            assert!(relative(Path::new(real_name)));
            if [b'1', b'2'].contains(&kind) {
                let link = pax.get("linkpath").and_then(Value::as_str).unwrap_or_else(|| field(&header[157..257]));
                let package = Path::new(real_name).components().next().unwrap().as_os_str();
                if kind == b'1' {
                    assert!(Path::new(link).starts_with(package), "hard link leaves top-level package");
                    safe_link(Path::new(real_name).strip_prefix(package).unwrap(), Path::new(link).strip_prefix(package).unwrap(), true);
                } else {
                    safe_link(Path::new(real_name).strip_prefix(package).unwrap(), Path::new(link), false);
                }
            }
            let mut remaining = size;
            let mut buffer = [0; 65536];
            while remaining > 0 {
                let n = remaining.min(buffer.len() as u64) as usize;
                reader.read_exact(&mut buffer[..n]).unwrap();
                remaining -= n as u64;
            }
            pax.clear();
            count += 1;
        }
        let padding = (512 - size % 512) % 512;
        let mut bytes = [0; 512];
        reader.read_exact(&mut bytes[..padding as usize]).unwrap();
        assert!(bytes[..padding as usize].iter().all(|b| *b == 0));
    }
    assert!(child.wait().unwrap().success());
    json!({"archive_sha256":digest(archive),"archive_bytes":fs::metadata(archive).unwrap().len(),"members":count,"raw_bytes_scanned":reader.position,"header_acceptance":"raw UID/GID 0; empty uname/gname; zero mtime; relative names and contained links; path/linkpath-only PAX; checked header checksums and zero trailer; gzip no filename/comment/mtime","findings":reader.hits,"credential_scope":"Heuristic private-key markers and common GitHub/OpenAI/AWS token shapes; findings require review. Not a guarantee that arbitrary credentials are absent."})
}

// Inspect the delivered ELF bytes rather than inferring a host contract from
// build flags or from just the command entry. Keep readelf's original output
// alongside the extracted version requirements for review.
fn platform(root: &Path) -> Value {
    let mut objects = Vec::new();
    let mut required = BTreeSet::new();
    for row in entries(root) {
        if row["type"] != "file" {
            continue;
        }
        let path = root.join(row["path"].as_str().unwrap());
        let mut header = [0; 4];
        if fs::File::open(&path).unwrap().read_exact(&mut header).is_err() || header != *b"\x7fELF" {
            continue;
        }
        let raw = String::from_utf8(output(
            Command::new("readelf").args(["--wide", "--file-header", "--program-headers", "--dynamic", "--version-info"]).arg(&path),
        ))
        .unwrap();
        let mut versions = BTreeSet::new();
        for line in raw.lines() {
            if let Some((_, tail)) = line.split_once("Name: ") {
                let name = tail.split_whitespace().next().unwrap();
                if ["GLIBC_", "GLIBCXX_", "CXXABI_", "GCC_"].iter().any(|p| name.starts_with(p)) {
                    versions.insert(name.to_owned());
                }
            }
        }
        required.extend(versions.iter().cloned());
        objects.push(json!({"path":row["path"],"sha256":row["sha256"],"required_symbol_versions":versions,"readelf":raw}));
    }
    assert!(!objects.is_empty(), "runtime contains no ELF objects");
    json!({"scope":"Observed ELF headers, interpreter, dynamic dependencies and symbol versions in this payload. This is not execution qualification on another host or a claim that system build/link dependencies are bundled.","objects":objects,"required_symbol_versions":required})
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args[0].to_str().unwrap() {
        "owned" => {
            let root = Path::new(&args[1]);
            let input: Value = serde_json::from_slice(&fs::read(&args[2]).unwrap()).unwrap();
            let mut rows = Vec::new();
            for row in input.as_array().unwrap() {
                let name = row["path"].as_str().unwrap();
                assert!(relative(Path::new(name)));
                let path = root.join(name);
                let meta = fs::symlink_metadata(&path).unwrap();
                let value = if meta.file_type().is_symlink() {
                    json!({"path":name,"type":"symlink","target":fs::read_link(&path).unwrap()})
                } else {
                    assert!(meta.is_file());
                    json!({"path":name,"type":"file","bytes":meta.len(),"sha256":digest(&path)})
                };
                rows.push(value);
            }
            save(Path::new(&args[3]), &json!(rows));
        }
        "inventory" => {
            let rows = entries(Path::new(&args[1]));
            save(Path::new(&args[2]), &json!(rows));
        }
        "source" => {
            let source = Path::new(&args[1]);
            let owned: Value = serde_json::from_slice(&fs::read(&args[2]).unwrap()).unwrap();
            let destination = Path::new(&args[3]);
            fs::create_dir(destination).unwrap();
            let tracked = output(Command::new("git").arg("-C").arg(source).args(["ls-files", "--cached", "-z"]));
            let mut names: BTreeSet<PathBuf> =
                tracked.split(|b| *b == 0).filter(|p| !p.is_empty()).map(|p| PathBuf::from(std::str::from_utf8(p).unwrap())).collect();
            for row in owned.as_array().unwrap() {
                names.insert(PathBuf::from(row["path"].as_str().unwrap()));
            }
            let mut gitlinks = Vec::new();
            for name in names {
                assert!(relative(&name));
                assert!(name != Path::new("crates/lustr/bench/cache-study.py"));
                let from = source.join(&name);
                let to = destination.join(&name);
                fs::create_dir_all(to.parent().unwrap()).unwrap();
                let meta = fs::symlink_metadata(&from).unwrap();
                if meta.file_type().is_symlink() {
                    let target = fs::read_link(from).unwrap();
                    safe_link(&name, &target, false);
                    std::os::unix::fs::symlink(target, to).unwrap();
                } else if meta.is_dir() {
                    let record = output(Command::new("git").arg("-C").arg(source).args(["ls-files", "--stage", "--"]).arg(&name));
                    let record = String::from_utf8(record).unwrap();
                    let fields: Vec<_> = record.split_whitespace().collect();
                    assert!(fields.len() == 4 && fields[0] == "160000", "unlisted directory in source export");
                    fs::create_dir(&to).unwrap();
                    gitlinks.push(json!({"path":name,"revision":fields[1],"scope":"Original Git submodule pointer; empty directory as in git archive. Not a compiler build dependency. Contents are not included."}));
                } else {
                    assert!(meta.is_file());
                    fs::copy(from, &to).unwrap();
                    fs::set_permissions(to, fs::Permissions::from_mode(meta.mode() & 0o777)).unwrap();
                }
            }
            save(&destination.parent().unwrap().join("export-gitlinks.json"), &json!(gitlinks));
        }
        "audit-tree" => {
            let root = Path::new(&args[1]);
            let patterns = patterns(Path::new(&args[2]));
            let rows = entries(root);
            let mut findings = Vec::new();
            for row in &rows {
                let name = row["path"].as_str().unwrap();
                let mut path_scanner = scan(name.as_bytes(), patterns.clone());
                std::io::copy(&mut path_scanner, &mut std::io::sink()).unwrap();
                for hit in path_scanner.hits {
                    findings.push(json!({"path":name,"location":"member-name","finding":hit}));
                }
                if row["type"] == "file" {
                    let mut scanner = scan(fs::File::open(root.join(name)).unwrap(), patterns.clone());
                    std::io::copy(&mut scanner, &mut std::io::sink()).unwrap();
                    for hit in scanner.hits {
                        findings.push(json!({"path":name,"location":"content","finding":hit}));
                    }
                }
                if let Some(target) = row["target"].as_str() {
                    let mut scanner = scan(target.as_bytes(), patterns.clone());
                    std::io::copy(&mut scanner, &mut std::io::sink()).unwrap();
                    for hit in scanner.hits {
                        findings.push(json!({"path":name,"location":"link-target","finding":hit}));
                    }
                }
            }
            save(
                Path::new(&args[3]),
                &json!({"entries":rows.len(),"findings":findings,"credential_scope":"Heuristic private-key markers and common GitHub/OpenAI/AWS token shapes; findings require review. Not a guarantee that arbitrary credentials are absent."}),
            );
        }
        "archive" => {
            let root = Path::new(&args[1]).canonicalize().unwrap();
            let archive = Path::new(&args[2]);
            assert!(!archive.exists());
            output(
                Command::new("tar")
                    .current_dir(root.parent().unwrap())
                    .args([
                        "--sort=name",
                        "--mtime=@0",
                        "--owner=0",
                        "--group=0",
                        "--numeric-owner",
                        "--format=posix",
                        "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
                        "-I",
                        "gzip -1 -n",
                        "-cf",
                    ])
                    .arg(archive)
                    .arg("--")
                    .arg(root.file_name().unwrap()),
            );
            save(Path::new(&args[4]), &audit_archive(archive, Path::new(&args[3])));
        }
        "audit-archive" => {
            save(Path::new(&args[3]), &audit_archive(Path::new(&args[1]), Path::new(&args[2])));
        }
        "platform" => {
            save(Path::new(&args[2]), &platform(Path::new(&args[1])));
        }
        _ => panic!("native_package inventory|source|audit-tree|archive|platform ..."),
    }
}
