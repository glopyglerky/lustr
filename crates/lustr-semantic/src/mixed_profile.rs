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

enum Event {
    Span {
        phase: &'static str,
        start_ns: u128,
        end_ns: u128,
    },
    NativeResponse {
        parent_start_ns: u128,
        roundtrip: Option<(u128, u128)>,
        unit_identity: String,
        response_code: i32,
        emitter_records: Vec<String>,
    },
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

impl Span {
    /// Close the client interval before inspecting its response's diagnostic
    /// profile. The returned values are only coordinates on this client clock.
    pub fn close(mut self) -> Option<(u128, u128)> {
        self.record()
    }

    fn record(&mut self) -> Option<(u128, u128)> {
        if let Some((recorder, phase, start_ns)) = self.0.take() {
            let end_ns = recorder.start.elapsed().as_nanos();
            if let Ok(mut events) = recorder.events.lock() {
                events.push(Event::Span { phase, start_ns, end_ns });
            }
            Some((start_ns, end_ns))
        } else {
            None
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        self.record();
    }
}

/// Keep the exact remote record with the request that returned it. Server
/// durations have their own clock and are never converted into client spans.
/// Missing or malformed records remain evidence for the external validator;
/// profiling cannot change the compiler response or turn a failure into zero.
pub fn native_response(parent: &Span, roundtrip: Option<(u128, u128)>, unit_identity: &str, response_code: i32, stderr: &str) {
    let Some((recorder, "native-compile", parent_start_ns)) = parent.0 else { return };
    let emitter_records = stderr.lines().filter_map(|line| line.strip_prefix("lustr mixed native profile: ").map(str::to_owned)).collect();
    if let Ok(mut events) = recorder.events.lock() {
        events.push(Event::NativeResponse {
            parent_start_ns,
            roundtrip,
            unit_identity: unit_identity.to_owned(),
            response_code,
            emitter_records,
        });
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
            serde_json::json!({"schema":"lustr.mixed-phase-profile","version":2,
                "pid":std::process::id(),"started_unix_ns":recorder.stamp,
                "request_key":request_key,
                "rustc":env!("LUSTR_BUILD_RUSTC_COMMIT"),"native":env!("LUST_BUILD_HASH"),
                "instrumented":true,"nested_spans":true})
        )?;
        for event in events.iter() {
            let record = match event {
                Event::Span { phase, start_ns, end_ns } => {
                    serde_json::json!({"phase":phase,"start_ns":start_ns,"end_ns":end_ns,"elapsed_ns":end_ns-start_ns})
                }
                Event::NativeResponse { parent_start_ns, roundtrip, unit_identity, response_code, emitter_records } => {
                    serde_json::json!({"kind":"native-response","parent_start_ns":parent_start_ns,
                        "roundtrip":roundtrip,"unit_identity":unit_identity,"response_code":response_code,
                        "emitter_status":if *response_code == 0 { "reported" } else { "unavailable-failed-response" },
                        "emitter_records":emitter_records})
                }
            };
            writeln!(file, "{record}")?;
        }
        writeln!(file, "{}", serde_json::json!({"completed":true,"success":success,"reused":reused,"check_only":check_only}))
    };
    let _ = write();
}
