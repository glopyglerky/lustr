//! Emit one closed native item from the original compiler's source snapshot.
#![allow(dead_code, unused_imports)]

include!("../crates/lustr-native-core/src/lib.rs");

#[link(name = "tcc", kind = "static")]
unsafe extern "C" {}

fn run() -> Result<(), String> {
    use std::io::Read;

    let mut args = std::env::args().skip(1);
    let mut position = || {
        args.next().ok_or("missing native byte offset")?.parse::<u32>()
            .map_err(|_| "invalid native byte offset")
    };
    let extent = position()?..position()?;
    let mut snapshot = String::new();
    std::io::stdin().take(native_object::SOURCE_LIMIT as u64 + 1).read_to_string(&mut snapshot)
        .map_err(|error| format!("cannot read original source snapshot: {error}"))?;
    if snapshot.len() > native_object::SOURCE_LIMIT {
        return Err("native object source exceeds current bound".into());
    }
    let symbol = native_object::emit_item(&args.collect::<Vec<_>>(), &snapshot, extent)?;
    println!("{symbol}");
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
