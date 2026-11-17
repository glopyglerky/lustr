//! Optional wall-time attribution for a separate, untimed qualification run.
//! Spans can nest; their durations must not be added as independent costs.
//! Disabled runs do not read a clock or write profiling output.
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

struct Event {
    phase: &'static str,
    start_ns: u128,
    end_ns: u128,
}
struct Recorder {
    directory: PathBuf,
    start: Instant,
    stamp: u128,
    events: Mutex<Vec<Event>>,
}
static RECORDER: OnceLock<Option<Recorder>> = OnceLock::new();

fn recorder() -> Option<&'static Recorder> {
    RECORDER
        .get_or_init(|| {
            let directory = PathBuf::from(std::env::var_os("LUSTR_MIXED_PROFILE_DIR")?);
            Some(Recorder {
                directory,
                start: Instant::now(),
                stamp: SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_nanos(),
                events: Mutex::new(Vec::new()),
            })
        })
        .as_ref()
}

pub struct Span(Option<(&'static Recorder, &'static str, u128)>);

pub fn span(phase: &'static str) -> Span {
    Span(recorder().map(|recorder| (recorder, phase, recorder.start.elapsed().as_nanos())))
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((recorder, phase, start_ns)) = self.0 {
            let end_ns = recorder.start.elapsed().as_nanos();
            if let Ok(mut events) = recorder.events.lock() {
                events.push(Event { phase, start_ns, end_ns });
            }
        }
    }
}

/// Reporting errors do not change compiler diagnostics or its exit status.
/// The external profiling qualifier requires a complete, valid record for
/// every observed compiler invocation, so absent/partial output fails that run.
pub fn finish(success: bool, reused: bool, check_only: bool, request_key: &str) {
    let Some(recorder) = recorder() else { return };
    let write = || -> std::io::Result<()> {
        let events = recorder.events.lock().map_err(|_| std::io::Error::other("profile recorder poisoned"))?;
        let path = recorder.directory.join(format!("{}-{}.jsonl", std::process::id(), recorder.stamp));
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(path)?;
        writeln!(
            file,
            "{}",
            serde_json::json!({"schema":"lustr.mixed-phase-profile","version":1,
                "pid":std::process::id(),"started_unix_ns":recorder.stamp,
                "request_key":request_key,
                "rustc":env!("LUSTR_BUILD_RUSTC_COMMIT"),"native":env!("LUST_BUILD_HASH"),
                "instrumented":true,"nested_spans":true})
        )?;
        for event in events.iter() {
            writeln!(
                file,
                "{}",
                serde_json::json!({"phase":event.phase,"start_ns":event.start_ns,"end_ns":event.end_ns,
                    "elapsed_ns":event.end_ns-event.start_ns})
            )?;
        }
        writeln!(file, "{}", serde_json::json!({"completed":true,"success":success,"reused":reused,"check_only":check_only}))
    };
    let _ = write();
}
