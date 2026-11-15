// The native compiler implementation and its original executable entry point
// remain byte-for-byte upstream. This crate adds one bounded C ABI entry.
include!("../../../compiler/native/src/main.rs");

use std::ffi::{CStr, c_char, c_void};
type Sink = unsafe extern "C" fn(*mut c_void, i32, *const u8, usize);

/// One single-file build/check per process. Callers keep argument strings and
/// the sink/context alive until return. No Rust-owned allocation crosses the ABI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_v2(
    argc: usize,
    argv: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
) -> i32 {
    static USED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if USED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return 3;
    }
    if argc == 0 || argc > 4096 || argv.is_null() || namespace.is_null() {
        return 2;
    }
    let Ok(namespace) = unsafe { CStr::from_ptr(namespace) }.to_str() else {
        return 2;
    };
    if namespace.len() != 64 || !namespace.bytes().all(|b| b.is_ascii_hexdigit()) {
        return 2;
    }
    let args: Option<Vec<String>> = unsafe { std::slice::from_raw_parts(argv, argc) }
        .iter()
        .map(|p| if p.is_null() { None } else { unsafe { CStr::from_ptr(*p) }.to_str().ok().map(str::to_owned) })
        .collect();
    let Some(args) = args else {
        return 2;
    };
    let Some(Invocation::Compile { mut req, run: false, .. }) = protocol::parse_args(&args) else {
        return 2;
    };
    if req.session.krate.manifest_path.is_some() || req.cc || req.run_args.is_some() || image::state_dir().is_some() {
        return 2;
    }
    // Cache identity includes the installed entry library's hash and runtime
    // family, without changing the caller's environment or source bytes.
    req.session.krate.package_id.push_str("#lustr-native-v2:");
    req.session.krate.package_id.push_str(namespace);
    own_env(&mut req);
    raise_stack_limit();
    tcc::use_mimalloc();
    let start = std::time::Instant::now();
    let cache = if req.emit_c.is_none() { driver::cache::Cache::open(&req.file) } else { None };
    let source = cache.as_ref().and_then(|_| std::fs::read(&req.file).ok());
    let cached = match (&cache, &source) {
        (Some(cache), Some(source)) => cache.lookup(&req, source, start),
        _ => None,
    };
    let response = if let Some(response) = cached {
        response
    } else {
        let small = std::fs::metadata(&req.file).map_or(true, |m| m.len() < BIG_FILE);
        if small {
            tcc::set_huge_pages(false);
        } else {
            pool::small_pages_in_parallel();
        }
        let dir = std::env::temp_dir().join(format!("lust-{}", std::process::id()));
        let mut session = driver::Session::new(dir, false, req.session.clone());
        session.caches.leak_at_exit = true;
        let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| driver::compile(&req, &mut session)));
        remove_obj_dir(session.obj_dir());
        if small {
            tcc::set_huge_pages(true);
        }
        let response = match compiled {
            Ok(response) => response,
            Err(_) => return 3,
        };
        if let (Some(cache), Some(source)) = (&cache, &source) {
            cache.store(&req, source, &response);
        }
        std::mem::forget(session);
        response
    };
    unsafe {
        sink(context, 1, response.stdout.as_ptr(), response.stdout.len());
        sink(context, 2, response.stderr.as_ptr(), response.stderr.len());
    }
    response.code
}
