fn trace(value: &str) {
    if let Some(path) = std::env::var_os("LUSTR_MIXED_TRACE") {
        use std::io::Write;
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{value}");
        }
    }
}

