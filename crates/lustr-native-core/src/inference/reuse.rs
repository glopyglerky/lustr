//! Bounded, process-local reuse of a successful artifact, never an edited AST.
//! Unknown reads, a changing input, or a missing/tampered output force a fresh
//! native compile. Runtime code is fixed by this loaded DSO's lifetime.
use crate::{driver, expand, protocol, resolve};
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_NODES: usize = 8192;

#[derive(PartialEq, Eq)]
enum Node {
    File(u32, Vec<u8>),
    Dir(u32, Vec<OsString>),
    Link(PathBuf),
}

#[derive(PartialEq, Eq)]
struct Tree {
    nodes: BTreeMap<PathBuf, Node>,
    files: BTreeSet<PathBuf>,
    roots: Vec<PathBuf>,
}

impl Tree {
    fn read(roots: &[PathBuf], output: &Path) -> Option<Self> {
        let mut tree = Self { nodes: BTreeMap::new(), files: BTreeSet::new(), roots: roots.to_vec() };
        let mut bytes = 0;
        for root in roots {
            tree.walk(root, output, &mut bytes, &mut BTreeSet::new(), 0)?;
        }
        Some(tree)
    }

    fn walk(&mut self, path: &Path, output: &Path, bytes: &mut usize, active: &mut BTreeSet<PathBuf>, depth: usize) -> Option<()> {
        if path == output {
            return Some(());
        }
        if depth > 64 || self.nodes.len() >= MAX_NODES {
            return None;
        }
        if self.nodes.contains_key(path) {
            return Some(());
        }
        *bytes = bytes.checked_add(path.as_os_str().as_encoded_bytes().len())?;
        if *bytes > MAX_BYTES {
            return None;
        }
        let meta = fs::symlink_metadata(path).ok()?;
        if meta.file_type().is_symlink() {
            let target = fs::read_link(path).ok()?;
            *bytes = bytes.checked_add(target.as_os_str().as_encoded_bytes().len())?;
            if *bytes > MAX_BYTES {
                return None;
            }
            self.nodes.insert(path.to_owned(), Node::Link(target));
            let real = fs::canonicalize(path).ok()?;
            if !active.insert(real.clone()) {
                return None;
            }
            self.walk(&real, output, bytes, active, depth + 1)?;
            active.remove(&real);
        } else if meta.is_file() {
            let len = usize::try_from(meta.len()).ok()?;
            if len > MAX_BYTES.checked_sub(*bytes)? {
                return None;
            }
            let mut data = Vec::new();
            fs::File::open(path).ok()?.take((MAX_BYTES - *bytes + 1) as u64).read_to_end(&mut data).ok()?;
            *bytes = bytes.checked_add(data.len())?;
            if *bytes > MAX_BYTES {
                return None;
            }
            self.files.insert(fs::canonicalize(path).ok()?);
            self.nodes.insert(path.to_owned(), Node::File(meta.mode(), data));
        } else if meta.is_dir() {
            let mut names = Vec::new();
            for entry in fs::read_dir(path).ok()? {
                let name = entry.ok()?.file_name();
                if path.join(&name) == output {
                    continue;
                }
                if names.len() + self.nodes.len() + 1 >= MAX_NODES {
                    return None;
                }
                *bytes = bytes.checked_add(name.as_encoded_bytes().len().checked_mul(2)?)?;
                if *bytes > MAX_BYTES {
                    return None;
                }
                names.push(name);
            }
            names.sort();
            self.nodes.insert(path.to_owned(), Node::Dir(meta.mode(), names.clone()));
            for name in names {
                self.walk(&path.join(name), output, bytes, active, depth + 1)?;
            }
        } else {
            return None;
        }
        Some(())
    }

    fn covers(&self, path: &Path, output: &Path) -> bool {
        path != output && fs::canonicalize(path).ok().is_some_and(|p| self.files.contains(&p))
    }
}

#[derive(PartialEq, Eq)]
struct Identity {
    request: Vec<u8>,
    env: Vec<(OsString, OsString)>,
    cwd: PathBuf,
    graph: Option<String>,
    bindings: Vec<PathBuf>,
}

fn identity(req: &driver::Request) -> Option<Identity> {
    if req.cmd != driver::Cmd::Build || req.emit_c.is_some() || req.cc || req.harness || req.run_args.is_some() || req.timings {
        return None;
    }
    let graph = if let Some(manifest) = &req.session.krate.manifest_path {
        let key = &req.session;
        let greq = resolve::cargo::GraphRequest {
            manifest_path: manifest.clone(),
            package: None,
            features: key.features.clone(),
            default_features: key.default_features,
            all_features: key.all_features,
            target_triple: key.target_triple.clone(),
            cargo_net: key.cargo_net,
            env: req.client.env.as_deref().map(resolve::cargo::cargo_env_of),
        };
        Some(format!("{:?}", resolve::cargo::load_graph(&greq).ok()?))
    } else {
        None
    };
    let mut env: Vec<_> = std::env::vars_os().collect();
    env.sort();
    let mut bindings =
        vec![req.file.canonicalize().ok()?, req.session.krate.root.canonicalize().ok()?, req.out.parent()?.canonicalize().ok()?];
    if let Some(manifest) = &req.session.krate.manifest_path {
        bindings.push(manifest.canonicalize().ok()?);
    }
    Some(Identity { request: protocol::encode(req), env, cwd: std::env::current_dir().ok()?, graph, bindings })
}

pub(super) struct Before {
    identity: Identity,
    tree: Tree,
    output: PathBuf,
    bindings: Vec<(PathBuf, PathBuf)>,
}

fn runtime_roots() -> Option<Vec<PathBuf>> {
    let mut roots = vec![resolve::adapters::library_root()];
    roots.extend(std::env::var_os("LUST_STD_SRC").filter(|s| !s.is_empty()).map(PathBuf::from));
    roots.extend(std::env::var_os("LUSTR_INFERENCE_REUSE_ROOTS").into_iter().flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>()));
    let (lib, bin) = crate::tcc::tcc_paths();
    roots.extend([lib, bin]);
    for root in &mut roots {
        *root = root.canonicalize().ok()?;
    }
    roots.sort();
    roots.dedup();
    Some(roots)
}

impl Before {
    pub(super) fn read(req: &driver::Request) -> Option<Self> {
        let identity = identity(req)?;
        let output = req.out.parent()?.canonicalize().ok()?.join(req.out.file_name()?);
        // Excluding an output cannot exclude source or module discovery.
        if output.extension().is_some_and(|x| x == "rs") || fs::symlink_metadata(&output).is_ok_and(|m| m.file_type().is_symlink()) {
            return None;
        }
        let source = req.session.krate.manifest_path.as_deref().unwrap_or(&req.file).parent()?.canonicalize().ok()?;
        let mut roots = runtime_roots()?;
        roots.push(source);
        roots.sort();
        roots.dedup();
        if roots.iter().any(|p| output == *p) {
            return None;
        }
        let tree = Tree::read(&roots, &output)?;
        Some(Self { identity, tree, output, bindings: Vec::new() })
    }

    pub(super) fn observed(&mut self, session: &driver::Session, retry: &driver::Request) -> bool {
        let front = &session.front;
        if !front.resolve_cache.source_paths().all(|p| self.tree.covers(p, &self.output)) {
            return false;
        }
        for path in front.resolve_cache.source_paths() {
            let Ok(real) = path.canonicalize() else {
                return false;
            };
            self.bindings.push((path.to_owned(), real));
        }
        let deps: Vec<_> = front.chunks.iter().flat_map(|c| c.expand_deps.iter().cloned()).collect();
        let client =
            expand::ClientView { env: retry.client.env.clone().map(std::sync::Arc::new), shown_root: front.client.shown_root.clone() };
        if expand::deps_stale(&deps, &client) {
            return false;
        }
        for dep in &deps {
            match dep {
                expand::Dep::File { path, .. } => {
                    if !self.tree.covers(path, &self.output) {
                        return false;
                    }
                    let Ok(real) = path.canonicalize() else {
                        return false;
                    };
                    self.bindings.push((path.to_owned(), real));
                }
                expand::Dep::Position => return false,
                _ => (),
            }
        }
        let Some(kept) = front.kept.iter().flatten().next() else {
            return false;
        };
        let Some(tables) = &kept.program.resolved else {
            return false;
        };
        // The complete trees cover both existing and newly appearing modules.
        // An alternative outside these roots is an unknown negative read.
        tables.module_alts.iter().all(|p| {
            if p == &self.output {
                return false;
            }
            let mut parent = p.parent();
            while let Some(dir) = parent {
                match dir.canonicalize() {
                    Ok(real) => {
                        if !self.tree.nodes.contains_key(&real) {
                            return false;
                        }
                        self.bindings.push((dir.to_owned(), real));
                        return true;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => parent = dir.parent(),
                    Err(_) => return false,
                }
            }
            false
        })
    }

    fn bindings_stand(&self) -> bool {
        self.bindings.iter().all(|(path, real)| path.canonicalize().ok().as_ref() == Some(real))
    }
}

struct Saved {
    before: Before,
    response: driver::Response,
    artifact: Vec<u8>,
    mode: u32,
    host: super::host_inputs::Proof,
}

#[derive(Default)]
pub(super) struct Cache {
    saved: Option<Saved>,
    runtime: Option<Tree>,
    retired: bool,
}

impl Cache {
    pub(super) const fn new() -> Self {
        Self { saved: None, runtime: None, retired: false }
    }

    pub(super) fn runtime_valid(&mut self) -> bool {
        // Upstream has process-global std location/runtime caches. Changing
        // those inputs requires a fresh process, even on an artifact miss.
        let current = runtime_roots().and_then(|r| Tree::read(&r, Path::new("")));
        if self.retired || current.is_none() || self.runtime.as_ref().is_some_and(|old| Some(old) != current.as_ref()) {
            self.retired = true;
            self.saved = None;
            return false;
        }
        self.runtime = current;
        true
    }
    pub(super) fn lookup(&mut self, req: &driver::Request) -> Option<driver::Response> {
        let saved = self.saved.take()?;
        if identity(req)? != saved.before.identity {
            return None;
        }
        if !saved.before.bindings_stand() {
            return None;
        }
        if !saved.host.stands() {
            return None;
        }
        if Tree::read(&saved.before.tree.roots, &saved.before.output)? != saved.before.tree {
            return None;
        }
        let meta = fs::symlink_metadata(&saved.before.output).ok()?;
        if !meta.is_file() || meta.mode() != saved.mode || fs::read(&saved.before.output).ok()? != saved.artifact {
            return None;
        }
        let mut response = saved.response.clone();
        self.saved = Some(saved);
        response.stderr.push_str("native inference: verified no-op artifact reuse\n");
        Some(response)
    }

    pub(super) fn store(
        &mut self,
        req: &driver::Request,
        before: Before,
        response: &driver::Response,
        host: Option<super::host_inputs::Proof>,
    ) {
        self.saved = None;
        let Some(host) = host else {
            return;
        };
        if !host.stands() {
            return;
        }
        if response.code != 0 || response.ran || identity(req).as_ref() != Some(&before.identity) {
            return;
        }
        if !before.bindings_stand() {
            return;
        }
        if Tree::read(&before.tree.roots, &before.output).as_ref() != Some(&before.tree) {
            return;
        }
        let Ok(meta) = fs::symlink_metadata(&before.output) else {
            return;
        };
        if !meta.is_file() || meta.len() > MAX_BYTES as u64 {
            return;
        }
        let Ok(artifact) = fs::read(&before.output) else {
            return;
        };
        if !artifact.starts_with(b"\x7fELF") {
            return;
        }
        self.saved = Some(Saved { before, response: response.clone(), artifact, mode: meta.mode(), host });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_tree_refuses_large_directory_and_oversized_file() {
        let scratch = super::super::scratch::Scratch::new(&std::env::temp_dir()).unwrap();
        let many = scratch.path().join("many");
        fs::create_dir(&many).unwrap();
        for i in 0..=MAX_NODES {
            fs::File::create(many.join(i.to_string())).unwrap();
        }
        assert!(Tree::read(&[many], Path::new("")).is_none());
        let file = scratch.path().join("oversized");
        fs::File::create(&file).unwrap().set_len((MAX_BYTES + 1) as u64).unwrap();
        assert!(Tree::read(&[file], Path::new("")).is_none());
    }
}
