//! Component-only matched experiment. It does not compile a Rust application.
//! Retains every sample; full compiler qualification and timing remain separate.
#[path = "../crates/lustr-semantic/src/mixed_link_input.rs"]
mod link_input;

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

// Exact predecessor copy policy from mixed_objects::retain_file at 048d824.
fn retained_copy(path: &mut Option<PathBuf>, directory: &Path, n: &mut usize) -> io::Result<()> {
    let Some(source) = path else { return Ok(()) };
    let target = directory.join(format!("object-{}-{}", *n, source.file_name().unwrap().to_string_lossy()));
    *n += 1;
    fs::copy(&*source, &target)?;
    fs::File::open(&target)?.sync_all()?;
    *source = target;
    Ok(())
}

fn measure(root: &Path, source: &Path, data: &[u8], sample: usize, durable: bool) -> io::Result<u128> {
    let directory = root.join(format!("sample-{sample}-{}", if durable { "predecessor" } else { "candidate" }));
    fs::create_dir(&directory)?;
    let mut paths: Vec<_> = (0..16).map(|_| Some(source.to_owned())).collect();
    let mut n = 0;
    let start = Instant::now();
    for path in &mut paths {
        if durable {
            retained_copy(path, &directory, &mut n)?;
        } else {
            link_input::stage(path, &directory, &mut n)?;
        }
    }
    let ns = start.elapsed().as_nanos();
    // Correctness work and cleanup are outside both timed intervals.
    for path in paths {
        if fs::read(path.unwrap())? != data {
            return Err(io::Error::other("staged input differs"));
        }
    }
    if fs::read(source)? != data {
        return Err(io::Error::other("retained input changed"));
    }
    fs::remove_dir_all(directory)?;
    Ok(ns)
}

fn main() -> io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or_else(|| io::Error::other("expected a new output directory"))?);
    if args.next().is_some() {
        return Err(io::Error::other("expected exactly one argument"));
    }
    fs::create_dir(&root)?;
    let data: Vec<u8> = (0..=255).cycle().take(65536).collect();
    let source = root.join("retained.o");
    fs::write(&source, &data)?;
    fs::File::open(&source)?.sync_all()?;
    fs::File::open(&root)?.sync_all()?;
    // Balanced randomized order, no discarded warmup or selected winner.
    let seed = 0x6c757374725f696fu64;
    let mut state = seed;
    let mut order: Vec<_> = (0..24).map(|i| i % 2 == 0).collect();
    for i in (1..order.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        order.swap(i, state as usize % (i + 1));
    }
    println!("{{\"kind\":\"scope\",\"scope\":\"temporary-link-copy-only-not-compiler-speed\",\"pairs\":24,\"files_per_lane\":16,\"bytes_per_file\":65536,\"seed\":{seed},\"page_caches_flushed\":false}}");
    for (sample, predecessor_first) in order.into_iter().enumerate() {
        let (predecessor, candidate) = if predecessor_first {
            let a = measure(&root, &source, &data, sample, true)?;
            (a, measure(&root, &source, &data, sample, false)?)
        } else {
            let b = measure(&root, &source, &data, sample, false)?;
            (measure(&root, &source, &data, sample, true)?, b)
        };
        println!("{{\"kind\":\"pair\",\"sample\":{sample},\"predecessor_first\":{predecessor_first},\"predecessor_ns\":{predecessor},\"candidate_ns\":{candidate}}}");
    }
    println!("{{\"kind\":\"complete\",\"pairs\":24,\"compiler_performance_admission\":false}}");
    Ok(())
}
