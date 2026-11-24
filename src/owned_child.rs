//! One owned foreground process group. Host signal/wait constants are explicit;
//! signal handlers call only async-signal-safe libc functions.
#[path = "host_runtime.rs"]
mod host_runtime;
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};

const SIGNALS: [i32; 4] = [1, 2, 3, 15];
const KILL: i32 = 9;
static GROUP: AtomicI32 = AtomicI32::new(0);
static PENDING: AtomicI32 = AtomicI32::new(0);
use host_runtime::{EMPTY_SIGSET, SIG_BLOCK, SIG_SETMASK, SigSet, WNOWAIT};
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn getpid() -> i32;
    #[link_name = "signal"]
    fn set_signal(signal: i32, handler: usize) -> usize;
    fn sigemptyset(set: *mut SigSet) -> i32;
    fn sigaddset(set: *mut SigSet, signal: i32) -> i32;
    fn pthread_sigmask(how: i32, set: *const SigSet, old: *mut SigSet) -> i32;
    fn waitid(kind: i32, id: u32, info: *mut [u64; 16], options: i32) -> i32;
}

extern "C" fn cancel(signal: i32) {
    // Native Lust may retain worker threads. A signal delivered to another
    // thread during fork must be deferred until the child's group is known.
    // Store-before-load and the spawn-side store-before-swap form an SC
    // handshake: either this handler sees the group, or spawn sees PENDING.
    PENDING.store(signal, Ordering::SeqCst);
    let group = GROUP.load(Ordering::SeqCst);
    if group == -1 {
        return;
    }
    unsafe {
        if group > 0 {
            kill(-group, KILL);
        }
        set_signal(signal, 0);
        kill(getpid(), signal);
    }
}

struct Blocked(SigSet);
impl Blocked {
    fn new() -> io::Result<Self> {
        let mut set = EMPTY_SIGSET;
        let mut old = EMPTY_SIGSET;
        unsafe {
            sigemptyset(&mut set);
            for signal in SIGNALS {
                sigaddset(&mut set, signal);
            }
            let result = pthread_sigmask(SIG_BLOCK, &set, &mut old);
            if result != 0 {
                return Err(io::Error::from_raw_os_error(result));
            }
        }
        Ok(Self(old))
    }
}
impl Drop for Blocked {
    fn drop(&mut self) {
        unsafe {
            pthread_sigmask(SIG_SETMASK, &self.0, std::ptr::null_mut());
        }
    }
}

pub struct OwnedChild {
    child: Option<Child>,
    observer: Option<std::thread::JoinHandle<io::Result<()>>>,
    handlers: Vec<(i32, usize)>,
}

fn observe_exit(pid: u32) -> io::Result<()> {
    let mut info = [0; 16];
    loop {
        // Retain the leader until the owner reaps it. Killing descendants
        // here also closes inherited pipes while the owner is draining them.
        if unsafe { waitid(1, pid, &mut info, 4 | WNOWAIT) } == 0 {
            unsafe {
                kill(-(pid as i32), KILL);
            }
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

impl OwnedChild {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        let blocked = Blocked::new()?;
        GROUP
            .compare_exchange(0, -1, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| io::Error::other("a foreground compiler group is already owned"))?;
        let mut owned = Self { child: None, observer: None, handlers: Vec::new() };
        for signal in SIGNALS {
            let old = unsafe { set_signal(signal, cancel as *const () as usize) };
            if old == usize::MAX {
                return Err(io::Error::last_os_error());
            }
            owned.handlers.push((signal, old));
            // Preserve a caller's explicitly ignored signals.
            if old == 1 {
                unsafe {
                    set_signal(signal, old);
                }
            }
        }
        let mask = blocked.0;
        let handlers = owned.handlers.clone();
        command.process_group(0);
        unsafe {
            command.pre_exec(move || {
                for &(signal, old) in &handlers {
                    set_signal(signal, if old == 1 { 1 } else { 0 });
                }
                let result = pthread_sigmask(SIG_SETMASK, &mask, std::ptr::null_mut());
                if result != 0 {
                    return Err(io::Error::from_raw_os_error(result));
                }
                Ok(())
            });
        }
        owned.child = Some(command.spawn()?);
        let pid = owned.child.as_ref().unwrap().id();
        GROUP.store(pid as i32, Ordering::SeqCst);
        owned.observer = Some(std::thread::Builder::new().name("lustr-child-exit".into()).spawn(move || observe_exit(pid))?);
        let pending = PENDING.swap(0, Ordering::SeqCst);
        if pending != 0 {
            cancel(pending);
        }
        drop(blocked);
        Ok(owned)
    }

    pub fn stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.as_mut()?.stdout.take()
    }

    fn kill_group(&self) {
        if let Some(child) = &self.child {
            unsafe {
                kill(-(child.id() as i32), KILL);
            }
        }
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        if self.child.is_none() {
            return Err(io::Error::other("compiler already reaped"));
        }
        self.observer
            .take()
            .ok_or_else(|| io::Error::other("missing compiler exit observer"))?
            .join()
            .map_err(|_| io::Error::other("compiler exit observer panicked"))??;
        self.kill_group();
        GROUP.store(0, Ordering::SeqCst);
        let status = self.child.as_mut().unwrap().wait()?;
        self.child = None;
        Ok(status)
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.kill_group();
        GROUP.store(0, Ordering::SeqCst);
        if let Some(observer) = self.observer.take() {
            let _ = observer.join();
        }
        if let Some(child) = &mut self.child {
            let _ = child.wait();
        }
        let pending = PENDING.swap(0, Ordering::SeqCst);
        if pending != 0 {
            cancel(pending);
        }
        for &(signal, old) in &self.handlers {
            unsafe {
                set_signal(signal, old);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::time::{Duration, Instant};

    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    #[test]
    fn cancellation_during_group_creation_is_retained() {
        let _lock = SERIAL.lock().unwrap();
        super::GROUP.store(-1, super::Ordering::SeqCst);
        super::cancel(15);
        assert_eq!(super::PENDING.swap(0, super::Ordering::SeqCst), 15);
        super::GROUP.store(0, super::Ordering::SeqCst);
    }

    #[test]
    #[ignore = "child fixture for exited_leader_cannot_leave_pipe_readers_blocked"]
    fn pipe_fixture() {
        let Some(stage) = std::env::var_os("LUSTR_PIPE_FIXTURE") else {
            return;
        };
        let marker = std::path::PathBuf::from(std::env::var_os("LUSTR_PIPE_MARKER").unwrap());
        if stage == "leaf" {
            std::fs::write(&marker, std::process::id().to_string()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(30));
        } else {
            fixture_command("leaf", &marker).spawn().unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while !marker.exists() && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert!(marker.exists());
            std::process::exit(17);
        }
    }

    fn fixture_command(stage: &str, marker: &std::path::Path) -> std::process::Command {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--ignored", "--exact", "owned_child::tests::pipe_fixture", "--nocapture"])
            .env("LUSTR_PIPE_FIXTURE", stage)
            .env("LUSTR_PIPE_MARKER", marker);
        command
    }

    struct ProcessObservation {
        terminal: bool,
        raw: String,
    }

    #[cfg(target_os = "linux")]
    fn observe_process(pid: i32) -> io::Result<ProcessObservation> {
        let raw = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(raw) => raw,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(ProcessObservation { terminal: true, raw: "absent (ENOENT)".into() });
            }
            Err(error) => return Err(io::Error::new(error.kind(), format!("cannot inspect pid {pid}: {error}"))),
        };
        let malformed = || io::Error::new(io::ErrorKind::InvalidData, format!("malformed /proc/{pid}/stat: {raw:?}"));
        if !raw.starts_with(&format!("{pid} (")) {
            return Err(malformed());
        }
        // comm may itself contain ')'; the state follows its final closing ')'.
        let (_, rest) = raw.rsplit_once(')').ok_or_else(malformed)?;
        let mut fields = rest.split_whitespace();
        let state = fields.next().ok_or_else(malformed)?;
        if state.len() != 1 || !b"RSDZTtXxKWPI".contains(&state.as_bytes()[0]) {
            return Err(malformed());
        }
        fields.next().and_then(|parent| parent.parse::<u32>().ok()).ok_or_else(malformed)?;
        Ok(ProcessObservation { terminal: state == "Z", raw })
    }

    #[cfg(target_os = "macos")]
    fn observe_process(pid: i32) -> io::Result<ProcessObservation> {
        let output = std::process::Command::new("/bin/ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .env("LC_ALL", "C")
            .output()?;
        let raw = String::from_utf8(output.stdout)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("invalid ps output for pid {pid}: {error}")))?;
        if !output.stderr.is_empty() || !matches!(output.status.code(), Some(0 | 1)) {
            return Err(io::Error::other(format!(
                "cannot inspect pid {pid}: ps status={}, stdout={raw:?}, stderr={:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let state = raw.trim().as_bytes();
        if output.status.code() == Some(1) && state.is_empty() {
            return Ok(ProcessObservation { terminal: true, raw: "absent (ps exit 1, empty output)".into() });
        }
        // Darwin ps(1) has one run-state letter followed only by state flags.
        if output.status.code() != Some(0)
            || state.is_empty()
            || !b"IRSTUZ".contains(&state[0])
            || !state[1..].iter().all(|flag| b"+<>AELNSsVWX".contains(flag))
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("malformed ps state for pid {pid}: {raw:?}")));
        }
        Ok(ProcessObservation { terminal: state[0] == b'Z', raw })
    }

    fn wait_for_terminal(pid: i32, deadline: Instant) -> io::Result<()> {
        loop {
            let observed = observe_process(pid)?;
            let now = Instant::now();
            if now >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("pid {pid} did not reach an observed terminal state before the deadline; last state: {:?}", observed.raw),
                ));
            }
            if observed.terminal {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1).min(deadline - now));
        }
    }

    #[test]
    fn terminal_observation_rejects_a_live_process() {
        struct CleanupChild(std::process::Child);
        impl Drop for CleanupChild {
            fn drop(&mut self) {
                // The observation helper never signals the process. This guard
                // owns cleanup even if an assertion or status read fails.
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }

        let _lock = SERIAL.lock().unwrap();
        let root = std::env::temp_dir().join(format!("lustr-live-observation-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let marker = root.join("leaf.pid");
        let mut child = CleanupChild(
            fixture_command("leaf", &marker)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
        let ready_deadline = Instant::now() + Duration::from_secs(3);
        while !marker.exists() && Instant::now() < ready_deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(marker.exists(), "negative-control process did not reach its leaf fixture");
        let pid = i32::try_from(child.0.id()).unwrap();
        assert!(child.0.try_wait().unwrap().is_none(), "negative-control process exited before observation");
        let error = wait_for_terminal(pid, Instant::now() + Duration::from_millis(50)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut, "live process was not observed to the deadline: {error}");
        assert!(child.0.try_wait().unwrap().is_none(), "observation must not kill the negative-control process");
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exited_leader_cannot_leave_pipe_readers_blocked() {
        use std::io::Read;
        let _lock = SERIAL.lock().unwrap();
        let root = std::env::temp_dir().join(format!("lustr-pipes-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        for capture in [true, false] {
            let marker = root.join(if capture { "capture.pid" } else { "stream.pid" });
            let mut command = fixture_command("leader", &marker);
            let start = Instant::now();
            let deadline = start + Duration::from_secs(3);
            let status = if capture {
                super::output(&mut command).unwrap().status
            } else {
                let mut child =
                    super::OwnedChild::spawn(command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())).unwrap();
                child.stdout().unwrap().read_to_end(&mut Vec::new()).unwrap();
                child.wait().unwrap()
            };
            assert_eq!(status.code(), Some(17));
            assert!(start.elapsed() < Duration::from_secs(3), "pipe drain waited for leaf timeout");
            let pid: i32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
            // Linux exit_files closes pipe writers before exit_notify publishes
            // EXIT_ZOMBIE. EOF and reaping the leader are not a leaf-exit barrier.
            // Observe actual termination within the original request deadline.
            wait_for_terminal(pid, deadline).unwrap_or_else(|error| {
                panic!("leaf survived leader cleanup: capture={capture}, elapsed={:?}: {error}", start.elapsed());
            });
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}

pub fn output(command: &mut Command) -> io::Result<Output> {
    let mut child = OwnedChild::spawn(command.stdout(Stdio::piped()).stderr(Stdio::piped()))?;
    let mut stdout = child.stdout().unwrap();
    let mut stderr = child.child.as_mut().unwrap().stderr.take().unwrap();
    let mut out = Vec::new();
    let mut err = Vec::new();
    std::thread::scope(|scope| -> io::Result<()> {
        let reader = scope.spawn(|| stderr.read_to_end(&mut err));
        let result = stdout.read_to_end(&mut out);
        if result.is_err() {
            child.kill_group();
        }
        let errors = reader.join().map_err(|_| io::Error::other("compiler stderr reader panicked"))?;
        result?;
        errors?;
        Ok(())
    })?;
    Ok(Output { status: child.wait()?, stdout: out, stderr: err })
}
