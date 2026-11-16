//! Actual TinyCC ELF input observations, including failed search candidates.
//! The Linux archive's libc `open` calls are wrapped at our own link boundary.
//! Parallel child C units are replayed with the identical native compiler and
//! options plus a Rust observer; an identical object binds its observations to the
//! object actually linked. No source/frontend/compiler fallback is involved.
//! Files must be stable during a request. Unknown observation disables reuse.
use std::os::unix::{
    ffi::{OsStrExt, OsStringExt},
    fs::{FileExt, MetadataExt},
    io::FromRawFd,
};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const BYTES: usize = 64 * 1024 * 1024;
const NODES: usize = 8192;

#[derive(PartialEq, Eq)]
enum Input {
    Missing(i32),
    File { real: PathBuf, mode: u32, bytes: Vec<u8> },
}

impl Input {
    fn descriptor(path: &Path, file: &fs::File, limit: usize) -> Option<Self> {
        let m = file.metadata().ok()?;
        if !m.is_file() || m.len() > limit as u64 {
            return None;
        }
        let real = path.canonicalize().ok()?;
        let bound = fs::metadata(path).ok()?;
        if (m.dev(), m.ino()) != (bound.dev(), bound.ino()) {
            return None;
        }
        let mut bytes = vec![0; usize::try_from(m.len()).ok()?];
        let mut n = 0;
        while n < bytes.len() {
            match file.read_at(&mut bytes[n..], n as u64) {
                Ok(0) => return None,
                Ok(read) => n += read,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (),
                Err(_) => return None,
            }
        }
        let mut extra = [0];
        if file.read_at(&mut extra, n as u64).ok()? != 0 {
            return None;
        }
        Some(Self::File { real, mode: m.mode(), bytes })
    }
    fn read(path: &Path, limit: usize) -> Option<Self> {
        match fs::File::open(path) {
            Ok(f) => Self::descriptor(path, &f, limit),
            Err(e) => match e.raw_os_error()? {
                code @ (2 | 20) => Some(Self::Missing(code)),
                _ => None,
            },
        }
    }
    fn size(&self) -> usize {
        match self {
            Self::Missing(_) => 0,
            Self::File { real, bytes, .. } => real.as_os_str().as_bytes().len() + bytes.len(),
        }
    }
    fn volatile(&self) -> bool {
        match self {
            Self::Missing(_) => false,
            Self::File { bytes, .. } => {
                [b"__TIME__".as_slice(), b"__DATE__", b"__TIMESTAMP__"].iter().any(|token| bytes.windows(token.len()).any(|w| w == *token))
            }
        }
    }
}

#[derive(Default)]
struct Observations {
    inputs: BTreeMap<PathBuf, Input>,
    bytes: usize,
    incomplete: bool,
}
impl Observations {
    fn add(&mut self, path: PathBuf, input: Option<Input>) -> Option<()> {
        let input = input?;
        if let Some(old) = self.inputs.get(&path) {
            return (old == &input).then_some(());
        }
        self.bytes = self.bytes.checked_add(path.as_os_str().as_bytes().len())?.checked_add(input.size())?;
        if self.bytes > BYTES || self.inputs.len() >= NODES {
            return None;
        }
        self.inputs.insert(path, input);
        Some(())
    }
}

pub(super) struct Proof(Observations);
impl Proof {
    pub(super) fn stands(&self) -> bool {
        self.0.inputs.iter().all(|(p, old)| Input::read(p, BYTES).as_ref() == Some(old))
    }
}

static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static OBSERVED: std::sync::Mutex<Option<Observations>> = std::sync::Mutex::new(None);
thread_local! { static INSIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

pub(super) struct Observer;

// Only two C-unit children run at once. Their outputs are files, so waiting in
// unit order cannot block on a sibling's pipe. All children in a batch are
// reaped before any fallible proof merge or return.
struct Replay {
    object: PathBuf,
    c: PathBuf,
    source: Input,
    trace: PathBuf,
    replay: PathBuf,
    errors: PathBuf,
    child: Option<std::process::Child>,
}

impl Replay {
    fn start(object: &Path, seen: &Observations, dir: &Path, lib: &Path, bin: &Path, observer: &Path, debug: bool)
        -> Result<Option<Self>, crate::driver::Response>
    {
        let prepare = || {
            let original = Input::read(object, BYTES)?;
            if seen.inputs.get(object) != Some(&original) {
                return None;
            }
            let c = object.with_extension("c");
            let source = Input::read(&c, BYTES)?;
            if !matches!(source, Input::File { .. }) || source.volatile() {
                return None;
            }
            let trace = object.with_extension("lustr-input-trace");
            let replay = object.with_extension("lustr-input-replay.o");
            let errors = object.with_extension("lustr-input-errors");
            debug_assert!(trace.starts_with(dir) && replay.starts_with(dir) && errors.starts_with(dir));
            let mut b = std::ffi::OsString::from("-B");
            b.push(lib);
            let mut command = std::process::Command::new(bin);
            command.arg(b).args(["-c", "-w", "-fno-asynchronous-unwind-tables"]).env("LD_PRELOAD", observer);
            if debug {
                command.arg("-g");
            }
            command.arg("-o").arg(&replay).arg(&c)
                .stdout(fs::File::create(&trace).ok()?)
                .stderr(fs::File::create(&errors).ok()?);
            Some((Self { object: object.into(), c, source, trace, replay, errors, child: None }, command))
        };
        let Some((mut unit, mut command)) = prepare() else { return Ok(None) };
        unit.child = Some(command.spawn().map_err(|error| crate::driver::Response {
            code: 3,
            stderr: format!("error: native input-proof compiler could not start: {error}\n"),
            ..Default::default()
        })?);
        Ok(Some(unit))
    }

    fn wait(&mut self) -> Result<(), crate::driver::Response> {
        let status = self.child.as_mut().unwrap().wait().map_err(|error| crate::driver::Response {
            code: 3,
            stderr: format!("error: native input-proof compiler could not wait: {error}\n"),
            ..Default::default()
        })?;
        self.child.take();
        if status.success() {
            return Ok(());
        }
        let mut diagnostic = Vec::new();
        let _ = fs::File::open(&self.errors).map(|f| f.take(65536).read_to_end(&mut diagnostic));
        Err(crate::driver::Response {
            code: 3,
            stderr: format!("error: native input-proof compiler failed: {status}\n{}", String::from_utf8_lossy(&diagnostic)),
            ..Default::default()
        })
    }

    fn observe(&self, seen: &mut Observations, dir: &Path) -> Option<()> {
        let Input::File { bytes: actual, .. } = seen.inputs.get(&self.object)? else { return None };
        let Input::File { bytes: audited, .. } = Input::read(&self.replay, BYTES)? else { return None };
        if actual != &audited {
            return None;
        }
        if fs::metadata(&self.trace).ok()?.len() > BYTES as u64 {
            return None;
        }
        let mut bytes = Vec::new();
        fs::File::open(&self.trace).ok()?.take((BYTES + 1) as u64).read_to_end(&mut bytes).ok()?;
        if bytes.len() > BYTES || bytes.is_empty() {
            return None;
        }
        let mut frame = std::io::Cursor::new(bytes);
        let mut records = 0usize;
        while frame.position() < frame.get_ref().len() as u64 {
            let mut four = [0; 4];
            frame.read_exact(&mut four).ok()?;
            let length = u32::from_le_bytes(four) as usize;
            records = records.checked_add(1)?;
            if length == 0 || length > 16384 || records > NODES {
                return None;
            }
            let mut path = vec![0; length];
            frame.read_exact(&mut path).ok()?;
            if path.contains(&0) {
                return None;
            }
            let path = PathBuf::from(std::ffi::OsString::from_vec(path));
            let path = if path.is_absolute() { path } else { std::env::current_dir().ok()?.join(path) };
            frame.read_exact(&mut four).ok()?;
            let result = i32::from_le_bytes(four);
            let input = Input::read(&path, BYTES)?;
            match (&input, result) {
                (Input::File { .. }, 0) => (),
                (Input::Missing(code), reported) if reported.checked_neg() == Some(*code) => (),
                _ => return None,
            }
            if path == self.c {
                if input != self.source {
                    return None;
                }
            } else {
                if path.starts_with(dir) {
                    return None;
                }
                // One shared map retains the original global byte/node bounds.
                seen.add(path, Some(input))?;
            }
        }
        seen.inputs.remove(&self.object);
        Some(())
    }
}

impl Drop for Replay {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            // Covers unwinding and unexpected wait errors as well as setup
            // refusals. A successful wait already removed the child above.
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn replay_objects(objects: &[PathBuf], seen: &mut Observations, dir: &Path, lib: &Path, bin: &Path, observer: &Path, debug: bool,
    failure: &mut Option<crate::driver::Response>) -> Option<()>
{
    for pair in objects.chunks(2) {
        let mut batch = Vec::with_capacity(2);
        let mut ready = true;
        let mut start_failure = None;
        for p in pair {
            // Every ignored transient needs the observed original object,
            // exact C source, and byte-identical audited replay.
            match Replay::start(p, seen, dir, lib, bin, observer, debug) {
                Ok(Some(unit)) => batch.push(unit),
                Ok(None) => { ready = false; break; }
                Err(error) => { start_failure = Some(error); break; }
            }
        }
        // Reap ALL started siblings before a proof refusal or error return.
        // Report the first failed unit in deterministic sorted order, without
        // retrying compilation. Drop also covers panic/unexpected wait errors.
        for unit in &mut batch {
            if let Err(error) = unit.wait() {
                failure.get_or_insert(error);
            }
        }
        if failure.is_none() {
            *failure = start_failure;
        }
        if failure.is_some() || !ready {
            return None;
        }
        for unit in batch {
            unit.observe(seen, dir)?;
        }
    }
    Some(())
}

#[cfg(lustr_tcc_input_observer)]
fn child_observer() -> Option<PathBuf> {
    if std::env::var_os("LD_PRELOAD").is_some() {
        return None;
    }
    let path = std::env::var_os("LUSTR_TCC_INPUT_OBSERVER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("LUSTR_BUILT_TCC_INPUT_OBSERVER")));
    // The dynamic loader splits LD_PRELOAD on whitespace and colons, without
    // an escaping mechanism. Never execute a differently parsed library list.
    if !path.is_absolute() || path.as_os_str().as_bytes().iter().any(|b| b.is_ascii_whitespace() || *b == b':') {
        return None;
    }
    let Input::File { bytes, .. } = Input::read(&path, BYTES)? else {
        return None;
    };
    (bytes.as_slice() == include_bytes!(env!("LUSTR_BUILT_TCC_INPUT_OBSERVER"))).then_some(path)
}
#[cfg(not(lustr_tcc_input_observer))]
fn child_observer() -> Option<PathBuf> {
    None
}

impl Observer {
    pub(super) fn begin() -> Option<Self> {
        if !cfg!(lustr_tcc_input_observer) {
            return None;
        }
        *OBSERVED.lock().ok()? = Some(Observations::default());
        ACTIVE.store(true, std::sync::atomic::Ordering::Release);
        Some(Self)
    }
    pub(super) fn finish(
        self,
        session: &crate::driver::Session,
        req: &crate::driver::Request,
    ) -> Result<Option<Proof>, crate::driver::Response> {
        let mut failure = None;
        let proof = self.complete(session, req, &mut failure);
        match failure {
            Some(failure) => Err(failure),
            None => Ok(proof),
        }
    }
    fn complete(
        self,
        session: &crate::driver::Session,
        req: &crate::driver::Request,
        failure: &mut Option<crate::driver::Response>,
    ) -> Option<Proof> {
        ACTIVE.store(false, std::sync::atomic::Ordering::Release);
        let mut seen = OBSERVED.lock().ok()?.take()?;
        if seen.incomplete || seen.inputs.is_empty() {
            return None;
        }
        let dir = session.obj_dir().canonicalize().ok()?;
        let (lib, bin) = crate::tcc::tcc_paths();
        let observer = child_observer()?;
        seen.add(observer.clone(), Input::read(&observer, BYTES))?;
        // The embedded bytes are the actual child bound at this DSO's build.
        if fs::read(&bin).ok()?.as_slice() != include_bytes!(env!("LUST_TCC_BIN")) {
            return None;
        }
        let mut objects = Vec::new();
        let mut entries = 0usize;
        let mut names = 0usize;
        for entry in fs::read_dir(&dir).ok()? {
            let entry = entry.ok()?;
            let p = entry.path();
            entries = entries.checked_add(1)?;
            names = names.checked_add(p.as_os_str().as_bytes().len())?;
            if entries > NODES || names > BYTES {
                return None;
            }
            let name = p.file_name()?.to_str()?;
            if !name.starts_with('u') || !name.ends_with(".o") {
                continue;
            }
            if name.len() > 64 || !name[1..name.len() - 2].bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            objects.push(p);
        }
        if objects.is_empty() {
            return None;
        }
        objects.sort();
        replay_objects(&objects, &mut seen, &dir, &lib, &bin, &observer, req.debug, failure)?;
        if seen.inputs.iter().any(|(p, i)| p.starts_with(&dir) || i.volatile()) {
            return None;
        }
        let proof = Proof(seen);
        proof.stands().then_some(proof)
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        ACTIVE.store(false, std::sync::atomic::Ordering::Release);
        if let Ok(mut state) = OBSERVED.lock() {
            state.take();
        }
    }
}

#[cfg(all(test, lustr_tcc_input_observer))]
mod replay_tests {
    use super::*;

    fn fixture(texts: &[&str]) -> (super::super::scratch::Scratch, PathBuf, Vec<PathBuf>, Observations) {
        let root = super::super::scratch::Scratch::new(&std::env::temp_dir()).unwrap();
        let dir = root.path().join("objects");
        fs::create_dir(&dir).unwrap();
        let header = root.path().join("header.h");
        fs::write(&header, "#define VALUE 7\n").unwrap();
        let (lib, bin) = crate::tcc::tcc_paths();
        let mut seen = Observations::default();
        let mut objects = Vec::new();
        for (n, text) in texts.iter().enumerate() {
            let object = dir.join(format!("u{n}.o"));
            let c = object.with_extension("c");
            fs::write(&c, format!("#include \"{}\"\n{text}\n", header.display())).unwrap();
            let mut b = std::ffi::OsString::from("-B");
            b.push(&lib);
            let o = std::process::Command::new(&bin).arg(b).args(["-c", "-w", "-fno-asynchronous-unwind-tables"])
                .arg("-o").arg(&object).arg(&c).output().unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            seen.add(object.clone(), Input::read(&object, BYTES)).unwrap();
            objects.push(object);
        }
        (root, dir, objects, seen)
    }

    #[test]
    fn paired_actual_replays_merge_all_units_and_keep_header_bindings() {
        let (root, dir, objects, mut seen) = fixture(&["int a(void) { return VALUE; }", "int b(void) { return VALUE + 1; }", "int c(void) { return VALUE + 2; }"]);
        let (lib, bin) = crate::tcc::tcc_paths();
        let mut failure = None;
        assert!(replay_objects(&objects, &mut seen, &dir, &lib, &bin, &child_observer().unwrap(), false, &mut failure).is_some());
        assert!(failure.is_none());
        assert!(objects.iter().all(|p| !seen.inputs.contains_key(p) && p.with_extension("lustr-input-replay.o").is_file()));
        let header = root.path().join("header.h");
        assert!(seen.inputs.contains_key(&header));
        let proof = Proof(seen);
        assert!(proof.stands());
        fs::write(header, "#define VALUE 8\n").unwrap();
        assert!(!proof.stands());
    }

    #[test]
    fn paired_clock_refusal_does_not_hide_a_started_sibling_failure() {
        let (_root, dir, objects, mut seen) = fixture(&["#define JOIN(a,b) a ## b\nconst char *clock = JOIN(__TI,ME__);", "int b(void) { return VALUE; }"]);
        fs::write(objects[1].with_extension("c"), "#error real replay failure\n").unwrap();
        let (lib, bin) = crate::tcc::tcc_paths();
        let mut failure = None;
        assert!(replay_objects(&objects, &mut seen, &dir, &lib, &bin, &child_observer().unwrap(), false, &mut failure).is_none());
        let failure = failure.unwrap();
        assert_eq!(failure.code, 3);
        assert!(failure.stderr.contains("real replay failure"));
        assert!(objects[0].with_extension("lustr-input-replay.o").is_file(), "successful sibling was not waited");
    }

    #[test]
    fn actual_replay_spawn_and_signal_failures_stay_failure3_and_are_reaped() {
        let (_root, dir, objects, seen) = fixture(&["int a(void) { return VALUE; }"]);
        let (lib, bin) = crate::tcc::tcc_paths();
        let observer = child_observer().unwrap();
        let failure = Replay::start(&objects[0], &seen, &dir, &lib, &dir.join("missing-compiler"), &observer, false).err().unwrap();
        assert_eq!(failure.code, 3);
        assert!(failure.stderr.contains("could not start"));
        // Keep a real TinyCC child blocked on its open stdin pipe so the
        // signal assertion does not race an already completed compilation.
        let mut b = std::ffi::OsString::from("-B");
        b.push(&lib);
        let errors = dir.join("signalled.errors");
        let child = std::process::Command::new(&bin).arg(b).args(["-c", "-o"])
            .arg(dir.join("signalled.o")).arg("-")
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null())
            .stderr(fs::File::create(&errors).unwrap()).spawn().unwrap();
        let mut replay = Replay { object: objects[0].clone(), c: objects[0].with_extension("c"),
            source: Input::read(&objects[0].with_extension("c"), BYTES).unwrap(),
            trace: dir.join("signalled.trace"), replay: dir.join("signalled.o"), errors, child: Some(child) };
        replay.child.as_mut().unwrap().kill().unwrap();
        let failure = replay.wait().unwrap_err();
        assert_eq!(failure.code, 3);
        assert!(failure.stderr.contains("signal"));
        assert!(replay.child.is_none(), "signalled child was not reaped");
    }
}

#[cfg(lustr_tcc_input_observer)]
mod wrapper {
    use super::*;
    use std::ffi::{CStr, c_char, c_int, c_void};
    unsafe extern "C" {
        fn __real_open(path: *const c_char, flags: c_int, ...) -> c_int;
        fn __real_dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
        fn __real_time(output: *mut std::ffi::c_long) -> std::ffi::c_long;
        fn __errno_location() -> *mut c_int;
    }
    // Only the inspected x86_64 Linux C ABI is enabled. Read the optional
    // third argument only when the caller supplied O_CREAT/O_TMPFILE.
    #[unsafe(no_mangle)]
    unsafe extern "C" fn __wrap_open(path: *const c_char, flags: c_int, mut args: ...) -> c_int {
        let mode = if flags & 64 != 0 || flags & 0x410000 == 0x410000 { unsafe { args.next_arg::<std::ffi::c_uint>() } } else { 0 };
        let fd = unsafe { __real_open(path, flags, mode) };
        let errno = unsafe { *__errno_location() };
        if ACTIVE.load(std::sync::atomic::Ordering::Acquire) && flags & 3 == 0 {
            let result = std::panic::catch_unwind(|| {
                INSIDE.with(|inside| {
                    if inside.replace(true) {
                        return;
                    }
                    struct Reset<'a>(&'a std::cell::Cell<bool>);
                    impl Drop for Reset<'_> {
                        fn drop(&mut self) {
                            self.0.set(false);
                        }
                    }
                    let _reset = Reset(inside);
                    let mut state = OBSERVED.lock().unwrap_or_else(|e| e.into_inner());
                    let Some(s) = state.as_mut() else {
                        return;
                    };
                    let observed = (|| {
                        if flags != 0 {
                            return None;
                        }
                        let path = PathBuf::from(std::ffi::OsStr::from_bytes(unsafe { CStr::from_ptr(path) }.to_bytes()));
                        if path.as_os_str().as_bytes().len() > 16384 {
                            return None;
                        }
                        let path = if path.is_absolute() { path } else { std::env::current_dir().ok()?.join(path) };
                        let input = if fd < 0 {
                            matches!(errno, 2 | 20).then_some(Input::Missing(errno))
                        } else {
                            // Borrow the fd; pread does not alter TinyCC's offset.
                            let file = std::mem::ManuallyDrop::new(unsafe { fs::File::from_raw_fd(fd) });
                            Input::descriptor(&path, &file, BYTES.saturating_sub(s.bytes))
                        };
                        s.add(path, input)
                    })();
                    if observed.is_none() {
                        s.incomplete = true;
                    }
                })
            });
            if result.is_err() {
                if let Ok(mut state) = OBSERVED.lock() {
                    if let Some(s) = state.as_mut() {
                        s.incomplete = true;
                    }
                }
            }
        }
        unsafe {
            *__errno_location() = errno;
        }
        fd
    }
    #[unsafe(no_mangle)]
    unsafe extern "C" fn __wrap_dlopen(path: *const c_char, flags: c_int) -> *mut c_void {
        if ACTIVE.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok(mut s) = OBSERVED.lock() {
                if let Some(s) = s.as_mut() {
                    s.incomplete = true;
                }
            }
        }
        unsafe { __real_dlopen(path, flags) }
    }
    #[unsafe(no_mangle)]
    unsafe extern "C" fn __wrap_time(output: *mut std::ffi::c_long) -> std::ffi::c_long {
        // In the qualified TinyCC archive, this is called when __DATE__ or
        // __TIME__ actually expands, including names formed by token pasting.
        if ACTIVE.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok(mut state) = OBSERVED.lock() {
                if let Some(s) = state.as_mut() {
                    s.incomplete = true;
                }
            }
        }
        unsafe { __real_time(output) }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn actual_wrapped_opens_preserve_offset_errno_and_negative_bindings() {
            let root = crate::inference::scratch::Scratch::new(&std::env::temp_dir()).unwrap();
            let file = root.path().join("input");
            let missing = root.path().join("missing");
            fs::write(&file, b"abc").unwrap();
            let observer = Observer::begin().unwrap();
            let c = std::ffi::CString::new(file.as_os_str().as_bytes()).unwrap();
            unsafe {
                *__errno_location() = 87;
            }
            let fd = unsafe { __wrap_open(c.as_ptr(), 0, 0) };
            assert!(fd >= 0);
            assert_eq!(unsafe { *__errno_location() }, 87);
            let mut opened = unsafe { fs::File::from_raw_fd(fd) };
            let mut first = [0];
            opened.read_exact(&mut first).unwrap();
            assert_eq!(&first, b"a", "observer advanced the native reader offset");
            let c = std::ffi::CString::new(missing.as_os_str().as_bytes()).unwrap();
            assert_eq!(unsafe { __wrap_open(c.as_ptr(), 0, 0) }, -1);
            assert_eq!(unsafe { *__errno_location() }, 2);
            let observed = OBSERVED.lock().unwrap().take().unwrap();
            assert!(!observed.incomplete);
            assert_eq!(observed.inputs.len(), 2);
            drop(observer);
            let proof = Proof(observed);
            assert!(proof.stands());
            fs::write(&missing, b"new candidate").unwrap();
            assert!(!proof.stands());
            fs::remove_file(&missing).unwrap();
            assert!(proof.stands());
            fs::write(&file, b"changed").unwrap();
            assert!(!proof.stands());
        }

        #[test]
        fn unsupported_files_and_bounds_leave_native_open_working_without_a_proof() {
            let root = crate::inference::scratch::Scratch::new(&std::env::temp_dir()).unwrap();
            let huge = root.path().join("huge");
            fs::File::create(&huge).unwrap().set_len(BYTES as u64 + 1).unwrap();
            for path in [huge.as_path(), Path::new("/dev/null")] {
                let observer = Observer::begin().unwrap();
                let c = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
                let fd = unsafe { __wrap_open(c.as_ptr(), 0, 0) };
                assert!(fd >= 0);
                drop(unsafe { fs::File::from_raw_fd(fd) });
                assert!(OBSERVED.lock().unwrap().as_ref().unwrap().incomplete);
                drop(observer);
            }
            let observer = Observer::begin().unwrap();
            let c = std::ffi::CString::new(root.path().join("created").as_os_str().as_bytes()).unwrap();
            let fd = unsafe { __wrap_open(c.as_ptr(), 1 | 64 | 512, 0o640) };
            assert!(fd >= 0, "mode argument was not forwarded");
            drop(unsafe { fs::File::from_raw_fd(fd) });
            assert!(root.path().join("created").is_file());
            assert!(OBSERVED.lock().unwrap().as_ref().unwrap().inputs.is_empty());
            drop(observer);
        }

        #[test]
        fn actual_child_observer_records_token_pasted_clock_expansion() {
            let root = crate::inference::scratch::Scratch::new(&std::env::temp_dir()).unwrap();
            let source = root.path().join("clock.c");
            fs::write(&source, "#define JOIN_RAW(a,b) a ## b\n#define JOIN(a,b) JOIN_RAW(a,b)\nconst char *value = JOIN(__TI,ME__);\n")
                .unwrap();
            let (lib, bin) = crate::tcc::tcc_paths();
            let mut b = std::ffi::OsString::from("-B");
            b.push(lib);
            let output = std::process::Command::new(bin)
                .arg(b)
                .arg("-c")
                .arg(&source)
                .arg("-o")
                .arg(root.path().join("clock.o"))
                .env("LD_PRELOAD", child_observer().unwrap())
                .output()
                .unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            let mut stream = std::io::Cursor::new(output.stdout);
            let mut clock = false;
            while stream.position() < stream.get_ref().len() as u64 {
                let mut len = [0; 4];
                stream.read_exact(&mut len).unwrap();
                let length = u32::from_le_bytes(len) as usize;
                if length == 0 {
                    clock = true;
                    continue;
                }
                assert!(length <= 16384);
                let mut path_and_result = vec![0; length + 4];
                stream.read_exact(&mut path_and_result).unwrap();
            }
            assert!(clock, "actual TinyCC clock expansion escaped the Rust observer");
        }
    }
}
