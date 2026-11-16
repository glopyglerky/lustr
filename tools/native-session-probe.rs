//! Explicit shared-native-session protocol probe. No daemon or fallback.
#[path = "../src/native_link.rs"]
mod native_link;
use std::io::{BufRead, Read, Write};

fn fields(input: &mut impl BufRead) -> Option<Vec<String>> {
    let mut line = String::new();
    if input.read_line(&mut line).unwrap() == 0 { return None; }
    let count: usize = line.trim().parse().unwrap();
    assert!(count <= 8192);
    let mut fields = Vec::new();
    for _ in 0..count {
        line.clear();
        input.read_line(&mut line).unwrap();
        let len: usize = line.trim().parse().unwrap();
        assert!(len <= 1024 * 1024);
        let mut data = vec![0; len];
        input.read_exact(&mut data).unwrap();
        fields.push(String::from_utf8(data).unwrap());
    }
    Some(fields)
}

fn main() {
    let library = std::path::PathBuf::from(std::env::args_os().nth(1).expect("LIBRARY"));
    let before: Vec<_> = std::env::vars_os().collect();
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    while let Some(args) = fields(&mut input) {
        let env = fields(&mut input).expect("complete environment frame");
        assert_eq!(env.len() % 2, 0);
        let env: Vec<_> = env.chunks_exact(2).map(|p| (p[0].clone().into(), p[1].clone().into())).collect();
        let (code, stdout, stderr) = native_link::compile_session(&library, &args, &env)
            .expect("native session ABI unavailable").expect("native session ABI failure");
        assert_eq!(std::env::vars_os().collect::<Vec<_>>(), before, "native session mutated process environment");
        writeln!(output, "{code} {} {}", stdout.len(), stderr.len()).unwrap();
        output.write_all(&stdout).unwrap();
        output.write_all(&stderr).unwrap();
        output.flush().unwrap();
    }
}
