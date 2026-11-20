//! Exercise the production mixed-object validator or measure that one operation.
//! Component timings do not establish whole-compiler acceleration.
#[path = "../crates/lustr-native-core/src/mixed_symbols.rs"]
mod symbols;
use std::{path::Path, process::Command, time::Instant};

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 3 && args[0] == "check" {
        if let Err(error) = symbols::check(Path::new(&args[1]), args[2].to_str().expect("expected symbol must be UTF-8")) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    } else if args.len() == 5 && args[0] == "bench" {
        let count: usize = args[4].to_str().unwrap().parse().unwrap();
        assert!((2..=10000).contains(&count));
        println!("pair,position,lane,elapsed_ns");
        for pair in 0..count {
            for position in 0..2 {
                let reference = (pair + position) % 2 == 0;
                let start = Instant::now();
                if reference {
                    let output = Command::new(&args[3]).args(["--undefined-only", "--format=posix"]).arg(&args[1]).output().unwrap();
                    assert!(output.status.success() && output.stdout.is_empty(), "reference did not establish symbol closure");
                } else {
                    symbols::check(Path::new(&args[1]), args[2].to_str().unwrap()).unwrap();
                }
                let elapsed = start.elapsed().as_nanos();
                println!("{pair},{position},{},{elapsed}", if reference { "nm" } else { "in-process" });
            }
        }
    } else {
        eprintln!("usage: check-mixed-symbols check OBJECT EXPORT | bench OBJECT EXPORT NM PAIRS");
        std::process::exit(2);
    }
}
