// Exercise argv-to-owned-String conversion, including mixed iterator ends.
use std::env;
use std::process::Command;

fn child() {
    let mut args = env::args();
    assert!(args.next().is_some());
    assert_eq!(args.next().as_deref(), Some("--child"));
    let values: Vec<String> = args.collect();
    println!("values={values:?}");

    let mut ends = env::args().skip(2);
    println!("front={:?} back={:?}", ends.next(), ends.next_back());
    println!("middle={:?}", ends.collect::<Vec<_>>());

    let reverse: Vec<String> = env::args().skip(2).rev().collect();
    println!("reverse={reverse:?}");
}

fn main() {
    if env::args().nth(1).as_deref() == Some("--child") {
        child();
        return;
    }
    let executable = env::current_exe().unwrap();
    let cases: &[&[&str]] = &[
        &[],
        &[""],
        &["one", "two", "three"],
        &["café", "e\u{301}", "中", "🙂"],
        &["", "é", " ", "--flag", ""],
    ];
    for (index, args) in cases.iter().enumerate() {
        let output = Command::new(&executable).arg("--child").args(*args).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        println!("case={index}");
        print!("{}", String::from_utf8(output.stdout).unwrap());
    }
}
