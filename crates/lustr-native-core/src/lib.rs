// The product ABI uses the pinned Lust pipeline and explicit integration policy.
// Separate, bounded qualifiers inspect native phase/object/import mechanisms;
// they do not export the general product compiler ABI.
#[cfg(not(any(feature = "exclusive-native", feature = "native-qualification")))]
compile_error!("select exclusive-native for the product or native-qualification for bounded research");
#[cfg(all(feature = "exclusive-native", any(feature = "native-qualification", feature = "upstream-semantic")))]
compile_error!("exclusive-native cannot be combined with native-qualification or upstream-semantic");
#[cfg(feature = "exclusive-native")]
include!("../../../compiler/native/src/main.rs");
#[cfg(feature = "native-qualification")]
include!("native_modules.rs");

#[cfg(feature = "native-qualification")]
mod native_assertion_text;

#[cfg(feature = "native-qualification")]
mod native_object;
#[cfg(feature = "native-qualification")]
mod native_memory;
#[cfg(feature = "native-qualification")]
mod native_fill;
#[cfg(feature = "native-qualification")]
mod unit_object;
#[cfg(feature = "native-qualification")]
mod unit_publication;
#[cfg(feature = "native-qualification")]
mod native_crate;

use std::ffi::{CStr, c_char, c_void};
type Sink = unsafe extern "C" fn(*mut c_void, i32, *const u8, usize);
// Native setup and TinyCC have process-global state. Serialize every entry
// into that pipeline, including calls through different experimental ABIs.
static NATIVE_ENTRY: std::sync::Mutex<()> = std::sync::Mutex::new(());

// The imported-Rust embedding uses its shared System allocation contract.
// No exported native entry may start a front image with that allocation mode.
fn embedding_environment_supported() -> bool {
    !cfg!(feature = "upstream-semantic")
        || (image::state_dir().is_none() && std::env::var_os("LUST_FRONT_ARENA").is_none())
}

/// Experimental native object phase interface. No compiler fallback, Cargo
/// metadata fiction, std runtime or retained-result reuse is involved.
#[cfg(feature = "native-qualification")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_object_v1(argc: usize, argv: *const *const c_char, sink: Sink, context: *mut c_void) -> i32 {
    if !embedding_environment_supported() { return 3; }
    let Ok(_entry) = NATIVE_ENTRY.try_lock() else { return 3; };
    static USED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if USED.swap(true,std::sync::atomic::Ordering::SeqCst) || argc>4096 || argv.is_null() { return 3; }
    let pointers=unsafe{std::slice::from_raw_parts(argv,argc)};
    let args:Option<Vec<String>>=pointers.iter().map(|p|if p.is_null(){None}else{unsafe{CStr::from_ptr(*p)}.to_str().ok().map(str::to_owned)}).collect();
    let Some(args)=args else{return 2};
    raise_stack_limit();
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||native_object::run(&args)));
    let (code,text)=match result{Ok(v)=>v,Err(_)=>(3,"error: native object compiler panicked\n".into())};
    unsafe{sink(context,2,text.as_ptr(),text.len())};code
}

/// One single-file build/check per process. Callers keep argument strings and
/// the sink/context alive until return. No Rust-owned allocation crosses the ABI.
#[cfg(feature = "exclusive-native")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_v2(
    argc: usize,
    argv: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
) -> i32 {
    if !embedding_environment_supported() { return 3; }
    static USED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if USED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return 3;
    }
    unsafe { native_entry(argc, argv, namespace, sink, context) }
}

#[cfg(feature = "exclusive-native")]
unsafe fn native_entry(
    argc: usize,
    argv: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
) -> i32 {
    let Ok(_entry) = NATIVE_ENTRY.try_lock() else { return 3; };
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
    if req.session.krate.manifest_path.is_some()
        || req.cc
        || req.run_args.is_some()
        || image::state_dir().is_some()
    {
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
        let mut session = {
            let mut session = driver::Session::new(dir, false, req.session.clone());
            session.caches.leak_at_exit = true;
            session
        };
        let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            driver::compile(&req, &mut session)
        }));
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
