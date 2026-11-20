use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Component, Path};
use std::process::{Command, Stdio};

#[derive(Clone)]
struct Pin {
    revision: String,
    tree: String,
    path: String,
}

#[derive(Clone)]
struct Integration {
    declaration: BTreeMap<String, String>,
    patch: Vec<u8>,
    qualification: Vec<u8>,
}

fn fields(text: &str, keys: &[&str]) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or("invalid source declaration")?;
        let key = key.trim();
        let value = value.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')).ok_or("invalid declaration value")?;
        if !keys.contains(&key) || value.contains(['"', '\\', '\0']) || values.insert(key.into(), value.into()).is_some() {
            return Err("unknown or duplicate declaration field".into());
        }
    }
    if values.len() != keys.len() {
        return Err("missing declaration field".into());
    }
    Ok(values)
}

fn oid(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn pin() -> Result<Pin, String> {
    let values = fields(include_str!("../upstream/lust.pin"), &["repository", "revision", "tree", "path"])?;
    if values["repository"] != "https://github.com/t3dotgg/lust-lang"
        || !oid(&values["revision"])
        || !oid(&values["tree"])
        || values["path"].is_empty()
        || !Path::new(&values["path"]).components().all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("invalid pristine upstream reference".into());
    }
    Ok(Pin { revision: values["revision"].clone(), tree: values["tree"].clone(), path: values["path"].clone() })
}

fn integration(text: &str, patch: &[u8], qualification: &[u8]) -> Result<Integration, String> {
    let declaration = fields(text, &["reference_revision", "reference_tree", "patch_blob", "integrated_tree", "qualification_blob"])?;
    if !declaration.values().all(|s| oid(s)) {
        return Err("invalid integration object identity".into());
    }
    Ok(Integration { declaration, patch: patch.into(), qualification: qualification.into() })
}

fn git(repo: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git").arg("-C").arg(repo).args(args).output().map_err(|e| format!("cannot execute Git: {e}"))?;
    if !output.status.success() {
        return Err("source check failed; retain unique work and inspect the scoped Git diff".into());
    }
    Ok(output.stdout)
}

fn object(repo: &Path, spec: &str) -> Result<String, String> {
    let out = git(repo, &["rev-parse", "--verify", spec])?;
    Ok(std::str::from_utf8(&out).map_err(|_| "invalid Git identity")?.trim_end_matches('\n').into())
}

fn blob(repo: &Path, bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child.stdin.take().unwrap().write_all(bytes).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("cannot identify integration bytes".into());
    }
    Ok(std::str::from_utf8(&out.stdout).map_err(|_| "invalid Git blob identity")?.trim_end_matches('\n').into())
}

// This is the complete binary diff, including modes, symlinks and gitlinks.
// Explicit options keep local display/diff preferences out of the declaration.
fn patch(repo: &Path, reference: &str, integrated: &str) -> Result<Vec<u8>, String> {
    git(
        repo,
        &[
            "-c",
            "core.quotePath=true",
            "diff",
            "--binary",
            "--full-index",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--no-color",
            "--ignore-submodules=none",
            "--submodule=short",
            "--diff-algorithm=myers",
            "--no-indent-heuristic",
            "--unified=3",
            "--inter-hunk-context=0",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            reference,
            integrated,
            "--",
        ],
    )
}

fn check(repo: &Path, pin: &Pin, integration: &Integration, index: bool) -> Result<(), String> {
    check_mode(repo, pin, integration, index, false)
}

fn check_mode(repo: &Path, pin: &Pin, integration: &Integration, index: bool, candidate: bool) -> Result<(), String> {
    let d = &integration.declaration;
    if d["reference_revision"] != pin.revision
        || d["reference_tree"] != pin.tree
        || object(repo, &format!("{}^{{tree}}", pin.revision))? != pin.tree
    {
        return Err("pristine reference identity mismatch".into());
    }
    if blob(repo, &integration.patch)? != d["patch_blob"] || blob(repo, &integration.qualification)? != d["qualification_blob"] {
        return Err("integration patch or qualification identity mismatch".into());
    }
    let q = fields(
        std::str::from_utf8(&integration.qualification).map_err(|_| "invalid qualification")?,
        &[
            "reference_revision",
            "reference_tree",
            "patch_blob",
            "integrated_tree",
            "status",
            "correctness_evidence",
            "review_evidence",
            "performance_evidence",
        ],
    )?;
    for key in ["reference_revision", "reference_tree", "patch_blob", "integrated_tree"] {
        if q[key] != d[key] {
            return Err("qualification does not bind this exact integration".into());
        }
    }
    let evidence = ["correctness_evidence", "review_evidence", "performance_evidence"];
    if integration.patch.is_empty() {
        if d["integrated_tree"] != pin.tree || q["status"] != "empty" || evidence.iter().any(|k| q[*k] != "unchanged-reference") {
            return Err("empty integration must preserve the pristine tree".into());
        }
    } else if !(q["status"] == "reviewed" || candidate && q["status"] == "candidate")
        || evidence.iter().any(|k| q[*k].trim().is_empty() || q[*k] == "unchanged-reference")
    {
        return Err("nonempty integration requires exact reviewed qualification and evidence references".into());
    }
    if patch(repo, &pin.tree, &d["integrated_tree"])? != integration.patch {
        return Err("declared patch differs from the complete reference-to-integrated diff".into());
    }
    let tree = if index {
        let out = git(repo, &["write-tree", &format!("--prefix={}/", pin.path)])?;
        std::str::from_utf8(&out).map_err(|_| "invalid index tree")?.trim_end_matches('\n').into()
    } else {
        object(repo, &format!("HEAD:{}", pin.path))?
    };
    if tree != d["integrated_tree"] {
        return Err("Lust tree differs from the exact declared integration".into());
    }
    let entries = git(repo, &["ls-files", "-v", "-z", "--", &pin.path])?;
    if entries.split(|b| *b == 0).filter(|s| !s.is_empty()).any(|s| s[0] == b'S' || s[0].is_ascii_lowercase()) {
        return Err("Lust source has assume-unchanged or skip-worktree flags; hidden drift is not admitted".into());
    }
    git(
        repo,
        &["-c", "core.filemode=true", "diff", "--no-ext-diff", "--ignore-submodules=none", "--quiet", "--exit-code", "--", &pin.path],
    )?;
    if !index {
        git(repo, &["diff", "--cached", "--no-ext-diff", "--ignore-submodules=none", "--quiet", "--exit-code", "HEAD", "--", &pin.path])?;
    }
    if !git(repo, &["ls-files", "--others", "--exclude-standard", "-z", "--", &pin.path])?.is_empty() {
        return Err("unexplained untracked Lust source; preserve it outside the admitted integration".into());
    }
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let index = args.iter().any(|arg| arg == "--index");
    let candidate = args.iter().any(|arg| arg == "--candidate");
    if args.len() != usize::from(index) + usize::from(candidate) || args.iter().any(|arg| arg != "--index" && arg != "--candidate") {
        eprintln!("usage: lust-upstream-check [--index] [--candidate]");
        std::process::exit(2);
    }
    let result = pin().and_then(|pin| {
        let i = integration(
            include_str!("../upstream/lust.integration"),
            include_bytes!("../upstream/lust-integration.patch"),
            include_bytes!("../upstream/lust-integration.qualification"),
        )?;
        check_mode(Path::new("."), &pin, &i, index, candidate)?;
        println!(
            "Lust reference {}; exact declared integration tree {}; patch {}; {}",
            pin.revision,
            i.declaration["integrated_tree"],
            i.declaration["patch_blob"],
            if i.patch.is_empty() {
                "empty integration"
            } else if candidate {
                "candidate source identity only; release admission remains required"
            } else {
                "qualification record bound; evidence admission remains required"
            }
        );
        Ok(())
    });
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    static SERIAL: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: std::path::PathBuf,
        pin: Pin,
    }

    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("lust-upstream-check-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
            fs::create_dir(&root).unwrap();
            git(&root, &["init", "-q"]).unwrap();
            fs::create_dir(root.join("vendor")).unwrap();
            fs::write(root.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
            fs::write(root.join("owned.rs"), "fn owned() {}\n").unwrap();
            git(&root, &["add", "--", "vendor", "owned.rs"]).unwrap();
            let mut f = Self { root, pin: Pin { revision: String::new(), tree: String::new(), path: "vendor".into() } };
            f.commit();
            f.pin.tree = object(&f.root, "HEAD:vendor").unwrap();
            // A real pristine upstream commit whose complete tree is the reference.
            let out = f.git_auth(&["commit-tree", &f.pin.tree, "-m", "pristine reference fixture"]);
            f.pin.revision = String::from_utf8(out).unwrap().trim().into();
            f
        }
        fn git_auth(&self, args: &[&str]) -> Vec<u8> {
            let out = Command::new("git")
                .arg("-C")
                .arg(&self.root)
                .args(["-c", "commit.gpgsign=false"])
                .args(args)
                .env("GIT_AUTHOR_NAME", "glopyglerky")
                .env("GIT_COMMITTER_NAME", "glopyglerky")
                .env("GIT_AUTHOR_EMAIL", "189872235+glopyglerky@users.noreply.github.com")
                .env("GIT_COMMITTER_EMAIL", "189872235+glopyglerky@users.noreply.github.com")
                .output()
                .unwrap();
            assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
            out.stdout
        }
        fn commit(&self) {
            self.git_auth(&["commit", "-q", "-m", "source guard fixture"]);
        }
        fn declaration(&self, tree: &str, status: &str) -> Integration {
            let bytes = patch(&self.root, &self.pin.tree, tree).unwrap();
            let hash = blob(&self.root, &bytes).unwrap();
            // These synthetic records exercise identity checks, not compiler admission.
            let e = if bytes.is_empty() { "unchanged-reference" } else { "synthetic-guard-fixture-only" };
            let q = format!(
                "reference_revision = \"{}\"\nreference_tree = \"{}\"\npatch_blob = \"{hash}\"\nintegrated_tree = \"{tree}\"\nstatus = \"{status}\"\ncorrectness_evidence = \"{e}\"\nreview_evidence = \"{e}\"\nperformance_evidence = \"{e}\"\n",
                self.pin.revision, self.pin.tree
            );
            let qh = blob(&self.root, q.as_bytes()).unwrap();
            integration(&format!("reference_revision = \"{}\"\nreference_tree = \"{}\"\npatch_blob = \"{hash}\"\nintegrated_tree = \"{tree}\"\nqualification_blob = \"{qh}\"\n", self.pin.revision, self.pin.tree), &bytes, q.as_bytes()).unwrap()
        }
        fn empty(&self) -> Integration {
            self.declaration(&self.pin.tree, "empty")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn detects_source_edits_and_additions_while_preserving_owned_work() {
        let f = Fixture::new();
        let i = f.empty();
        check(&f.root, &f.pin, &i, false).unwrap();
        fs::write(f.root.join("owned.rs"), "fn independently_owned() {}\n").unwrap();
        check(&f.root, &f.pin, &i, false).unwrap();
        fs::write(f.root.join("vendor/source.rs"), "fn changed() {}\n").unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        assert!(check(&f.root, &f.pin, &i, true).is_err());
        fs::write(f.root.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        fs::write(f.root.join("vendor/extra.rs"), "fn extra() {}\n").unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        fs::remove_file(f.root.join("vendor/extra.rs")).unwrap();
        check(&f.root, &f.pin, &i, false).unwrap();
        assert_eq!(fs::read_to_string(f.root.join("owned.rs")).unwrap(), "fn independently_owned() {}\n");
    }

    #[test]
    fn checks_a_staged_restoration_before_commit() {
        let f = Fixture::new();
        let i = f.empty();
        fs::write(f.root.join("vendor/source.rs"), "fn changed() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        f.commit();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        fs::write(f.root.join("vendor/source.rs"), "fn upstream() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        check(&f.root, &f.pin, &i, true).unwrap();
    }

    #[test]
    fn admits_exact_declared_diff_and_rejects_unexplained_drift() {
        let f = Fixture::new();
        fs::write(f.root.join("vendor/source.rs"), "fn upstream() {}\nfn shared_interface() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        let tree = String::from_utf8(git(&f.root, &["write-tree", "--prefix=vendor/"]).unwrap()).unwrap().trim().to_owned();
        let i = f.declaration(&tree, "reviewed");
        check(&f.root, &f.pin, &i, true).unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        f.commit();
        check(&f.root, &f.pin, &i, false).unwrap();
        fs::write(f.root.join("vendor/source.rs"), "fn unexpected() {}\n").unwrap();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        assert!(check(&f.root, &f.pin, &i, true).is_err());
        f.commit();
        assert!(check(&f.root, &f.pin, &i, false).is_err());
    }

    #[test]
    fn candidate_identity_check_preserves_the_release_hold() {
        let f = Fixture::new();
        fs::write(f.root.join("vendor/source.rs"), "fn upstream() {}\nfn candidate_interface() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        let tree = String::from_utf8(git(&f.root, &["write-tree", "--prefix=vendor/"]).unwrap()).unwrap().trim().to_owned();
        let i = f.declaration(&tree, "candidate");
        assert!(check(&f.root, &f.pin, &i, true).is_err());
        check_mode(&f.root, &f.pin, &i, true, true).unwrap();
        fs::write(f.root.join("vendor/source.rs"), "fn unexplained() {}\n").unwrap();
        assert!(check_mode(&f.root, &f.pin, &i, true, true).is_err());
    }

    #[test]
    fn rejects_patch_record_reference_and_expected_result_tampering() {
        let f = Fixture::new();
        fs::write(f.root.join("vendor/source.rs"), "fn interface() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor"]).unwrap();
        f.commit();
        let tree = object(&f.root, "HEAD:vendor").unwrap();
        let i = f.declaration(&tree, "reviewed");
        check(&f.root, &f.pin, &i, false).unwrap();
        let mut bad = i.clone();
        bad.patch.push(b'\n');
        assert!(check(&f.root, &f.pin, &bad, false).is_err());
        let mut bad = i.clone();
        bad.qualification.push(b'\n');
        assert!(check(&f.root, &f.pin, &bad, false).is_err());
        let mut bad = i.clone();
        bad.declaration.insert("reference_tree".into(), tree.clone());
        assert!(check(&f.root, &f.pin, &bad, false).is_err());
        let mut bad = i.clone();
        bad.declaration.insert("integrated_tree".into(), f.pin.tree.clone());
        assert!(check(&f.root, &f.pin, &bad, false).is_err());
        let pending = f.declaration(&tree, "pending");
        assert!(check(&f.root, &f.pin, &pending, false).is_err());
        let mut bad = i.clone();
        bad.patch = b"unexplained patch".to_vec();
        bad.declaration.insert("patch_blob".into(), blob(&f.root, &bad.patch).unwrap());
        // Even after rewriting the record to match its new patch identity, the
        // independently regenerated complete tree diff must reject these bytes.
        let old_hash = &i.declaration["patch_blob"];
        let q = String::from_utf8(bad.qualification).unwrap().replace(old_hash, &bad.declaration["patch_blob"]);
        bad.qualification = q.into_bytes();
        bad.declaration.insert("qualification_blob".into(), blob(&f.root, &bad.qualification).unwrap());
        assert!(check(&f.root, &f.pin, &bad, false).is_err());
    }

    #[test]
    fn refuses_index_flags_that_hide_worktree_drift() {
        for flag in ["--assume-unchanged", "--skip-worktree"] {
            let f = Fixture::new();
            let i = f.empty();
            git(&f.root, &["update-index", flag, "--", "vendor/source.rs"]).unwrap();
            fs::write(f.root.join("vendor/source.rs"), "fn hidden_drift() {}\n").unwrap();
            assert!(check(&f.root, &f.pin, &i, false).is_err(), "{flag} concealed drift");
            assert!(check(&f.root, &f.pin, &i, true).is_err(), "{flag} concealed staged-mode drift");
        }
    }

    #[test]
    fn complete_patch_includes_gitlinks_despite_local_diff_configuration() {
        let mut f = Fixture::new();
        let old_dep = f.pin.revision.clone();
        fs::create_dir(f.root.join("vendor/dependency")).unwrap();
        git(&f.root, &["update-index", "--add", "--cacheinfo", &format!("160000,{old_dep},vendor/dependency")]).unwrap();
        f.commit();
        f.pin.tree = object(&f.root, "HEAD:vendor").unwrap();
        f.pin.revision =
            String::from_utf8(f.git_auth(&["commit-tree", &f.pin.tree, "-m", "reference with gitlink"])).unwrap().trim().into();
        let new_dep =
            String::from_utf8(f.git_auth(&["commit-tree", &f.pin.tree, "-m", "second dependency fixture"])).unwrap().trim().to_owned();
        git(&f.root, &["update-index", "--cacheinfo", &format!("160000,{new_dep},vendor/dependency")]).unwrap();
        fs::write(f.root.join("vendor/source.rs"), "fn shared_interface() {}\n").unwrap();
        git(&f.root, &["add", "--", "vendor/source.rs"]).unwrap();
        f.commit();
        git(&f.root, &["config", "diff.ignoreSubmodules", "all"]).unwrap();
        git(&f.root, &["config", "diff.submodule", "log"]).unwrap();
        let tree = object(&f.root, "HEAD:vendor").unwrap();
        let i = f.declaration(&tree, "reviewed");
        assert!(i.patch.windows(b"dependency".len()).any(|w| w == b"dependency"), "gitlink omitted");
        assert!(i.patch.windows(old_dep.len()).any(|w| w == old_dep.as_bytes()), "old gitlink identity omitted");
        assert!(i.patch.windows(new_dep.len()).any(|w| w == new_dep.as_bytes()), "new gitlink identity omitted");
        check(&f.root, &f.pin, &i, false).unwrap();
    }
}
