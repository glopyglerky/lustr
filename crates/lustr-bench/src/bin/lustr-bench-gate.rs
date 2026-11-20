//! Normalize successful libtest summaries while retaining unmodified output.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    io::Write,
    path::Path,
    process::{Command, ExitCode},
};

fn test_summary(stdout: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(stdout).map_err(|_| "test output is not UTF-8")?;
    let mut groups = Vec::new();
    let mut names = Vec::new();
    let mut ignored_names = Vec::new();
    let mut group_names = Vec::new();
    let mut group_ignored_names = Vec::new();
    let mut observed = std::collections::BTreeMap::new();
    for line in text.lines() {
        if let Some(result) = line.strip_prefix("test result: ok. ") {
            let counts = result.split("; finished in ").next().ok_or("missing test counts")?;
            let fields: Vec<_> = counts.split("; ").collect();
            if fields.len() != 5 {
                return Err("incomplete test summary".into());
            }
            let mut parsed = Vec::new();
            for (field, label) in fields.iter().zip(["passed", "failed", "ignored", "measured", "filtered out"]) {
                let number = field.strip_suffix(&format!(" {label}")).ok_or("unrecognized test counts")?;
                if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("test counts must be nonnegative integers".into());
                }
                parsed.push(number.parse::<usize>().map_err(|_| "test count overflow")?);
            }
            if parsed[1] != 0 || parsed[3] != 0 || parsed[4] != 0 {
                return Err("test summary reports failed, measured or filtered tests".into());
            }
            if parsed[0] != observed.values().filter(|&&ignored| !ignored).count()
                || parsed[2] != observed.values().filter(|&&ignored| ignored).count()
            {
                return Err("test counts differ from observed individual outcomes".into());
            }
            observed.clear();
            group_names.sort();
            group_ignored_names.sort();
            groups.push(json!({
                "counts": counts,
                "passed_test_names": std::mem::take(&mut group_names),
                "ignored_test_names": std::mem::take(&mut group_ignored_names),
            }));
        } else if line.starts_with("test result:") {
            return Err("failed or unrecognized test summary".into());
        } else if let Some((name, status)) = line.strip_prefix("test ").and_then(|line| line.rsplit_once(" ... ")) {
            let ignored = status == "ignored" || status.starts_with("ignored, ");
            if name.is_empty() || !(ignored || status == "ok") {
                return Err("failed or unrecognized individual test result".into());
            }
            if observed.insert(name.to_owned(), ignored).is_some() {
                return Err("duplicated individual test result".into());
            }
            if ignored {
                ignored_names.push(line.to_owned());
                group_ignored_names.push(line.to_owned());
            } else {
                names.push(line.to_owned());
                group_names.push(line.to_owned());
            }
        }
    }
    if groups.is_empty() || names.is_empty() || !observed.is_empty() {
        return Err("no completed, nonempty test harness observed".into());
    }
    groups.sort_by_cached_key(Value::to_string);
    names.sort();
    ignored_names.sort();
    Ok(json!({"kind":"tests","groups":groups,"passed_test_names":names,"ignored_test_names":ignored_names}))
}

fn has_error_code(stderr: &[u8], code: &str) -> bool {
    stderr.split(|byte| *byte == b'\n').any(|line| {
        serde_json::from_slice::<Value>(line).is_ok_and(|diagnostic| {
            diagnostic["$message_type"] == "diagnostic" && diagnostic["level"] == "error" && diagnostic["code"]["code"] == code
        })
    })
}

fn validate_tests(args: &[OsString]) -> Result<(), &'static str> {
    if args.first().is_none_or(|arg| arg != "test") || args.iter().any(|arg| arg == "--no-run") {
        return Err("tests gate requires an executing Cargo test command");
    }
    Ok(())
}

fn execute() -> Result<u8, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let mode = args.next().ok_or("usage: lustr-bench-gate tests <Cargo args...> | reject <source>")?;
    if mode == "--version" && args.next().is_none() {
        println!("lustr-bench-gate {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    let rejection = mode == "reject";
    let mut command = if mode == "tests-via" {
        return Err("Retired whole-rustc tests-via acquisition is disabled; use an explicit direct Cargo oracle".into());
    } else {
        let variable = if rejection {
            "RUSTC"
        } else if mode == "tests" {
            "CARGO"
        } else {
            return Err("unknown gate".into());
        };
        let program = std::env::var_os(variable).ok_or("explicit tool selection is required")?;
        if !Path::new(&program).is_absolute() {
            return Err("gate tool must be absolute".into());
        }
        let mut command = Command::new(program);
        if rejection {
            let source = args.next().ok_or("missing rejection source")?;
            if args.next().is_some() {
                return Err("unexpected rejection argument".into());
            }
            let target = std::env::var_os("CARGO_TARGET_DIR").ok_or("runner-owned output required")?;
            command.args([
                OsString::from("--edition=2024"),
                OsString::from("--error-format=json"),
                source,
                OsString::from("--emit=metadata"),
                OsString::from("--out-dir"),
                target,
            ]);
        } else {
            let args: Vec<_> = args.collect();
            validate_tests(&args)?;
            command.args(args);
        }
        command
    };
    let output = command.output()?;
    // Raw streams remain in the runner's evidence; only comparison output is normalized.
    let mut diagnostic = std::io::stderr().lock();
    writeln!(diagnostic, "raw status: {}", output.status)?;
    writeln!(diagnostic, "raw stdout: {} bytes", output.stdout.len())?;
    diagnostic.write_all(&output.stdout)?;
    writeln!(diagnostic, "\nraw stderr: {} bytes", output.stderr.len())?;
    diagnostic.write_all(&output.stderr)?;
    diagnostic.flush()?;
    if rejection {
        if output.status.code() != Some(1) || !has_error_code(&output.stderr, "E0502") {
            return Err("expected invalid-borrow diagnostic E0502 was not observed".into());
        }
        println!("{}", json!({"kind":"rejection","diagnostic":"E0502"}));
        Ok(1)
    } else {
        if !output.status.success() {
            return Err("project tests failed".into());
        }
        println!("{}", test_summary(&output.stdout)?);
        Ok(0)
    }
}

fn main() -> ExitCode {
    match execute() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("gate validation failed: {error}");
            ExitCode::from(42)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durations_are_ignored_but_counts_and_test_names_are_preserved() {
        let a = b"test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let b = b"test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.10s\n";
        assert_eq!(test_summary(a).unwrap(), test_summary(b).unwrap());
        let changed = b"test x ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        assert!(test_summary(changed).is_err());
        assert!(test_summary(b"build succeeded").is_err());
    }

    #[test]
    fn success_label_cannot_hide_missing_filtered_failed_or_duplicate_tests() {
        for output in [
            "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
            "test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s\n",
            "test x ... ok\ntest y ... FAILED\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
            "test x ... ok\ntest x ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
            "test x ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\ntest y ... ok\n",
        ] {
            assert!(test_summary(output.as_bytes()).is_err(), "{output}");
        }
    }

    #[test]
    fn ignored_names_and_every_harness_remain_visible() {
        let output = b"test x ... ok\ntest y ... ignored, explicit upstream reason\ntest result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let result = test_summary(output).unwrap();
        assert_eq!(result["groups"].as_array().unwrap().len(), 2);
        assert_eq!(result["ignored_test_names"], json!(["test y ... ignored, explicit upstream reason"]));
    }

    #[test]
    fn names_cannot_move_between_differently_sized_harnesses() {
        let summary = |names: &[&str]| {
            format!(
                "{}test result: ok. {} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n",
                names.iter().map(|name| format!("test {name} ... ok\n")).collect::<String>(),
                names.len()
            )
        };
        let a = summary(&["x"]) + &summary(&["y", "z"]);
        let b = summary(&["y"]) + &summary(&["x", "z"]);
        assert_ne!(test_summary(a.as_bytes()).unwrap(), test_summary(b.as_bytes()).unwrap());
    }
}
