//! Emit closed native items from the original compiler's source snapshot.
#![allow(dead_code, unused_imports)]

include!("../crates/lustr-native-core/src/lib.rs");

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

fn run() -> Result<(), String> {
    use std::io::Read;

    let mut args = std::env::args().skip(1);
    let link = match args.next().as_deref() {
        Some("check") => false,
        Some("link") => true,
        _ => return Err("missing native item operation".into()),
    };
    let mut extents = Vec::new();
    loop {
        let start = args.next().ok_or("missing native item offset or argument separator")?;
        if start == "--" {
            break;
        }
        let start = start.parse::<u32>().map_err(|_| "invalid native item start")?;
        let end = args.next().ok_or("missing native item end")?.parse::<u32>()
            .map_err(|_| "invalid native item end")?;
        extents.push(start..end);
    }
    let mut snapshot = String::new();
    std::io::stdin().take(native_object::SOURCE_LIMIT as u64 + 1).read_to_string(&mut snapshot)
        .map_err(|error| format!("cannot read original source snapshot: {error}"))?;
    if snapshot.len() > native_object::SOURCE_LIMIT {
        return Err("native object source exceeds current bound".into());
    }
    let bindings = native_object::complete_items(&args.collect::<Vec<_>>(), &snapshot, &extents, link)?;
    for (name, symbol) in bindings {
        println!("{name} {symbol}");
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
