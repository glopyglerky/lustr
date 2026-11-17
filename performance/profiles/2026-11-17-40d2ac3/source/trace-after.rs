fn trace(value: &str) {
    if let Some(path) = std::env::var_os("LUSTR_MIXED_TRACE") {
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
            // Cargo can run several compiler processes at once. Keep the
            // complete diagnostic and its final newline together, including
            // multi-line emitter details and any write_all short-write retry.
            let record = format!("{value}\n");
            if f.lock().is_ok() {
                let _ = f.write_all(record.as_bytes());
            }
        }
    }
}

