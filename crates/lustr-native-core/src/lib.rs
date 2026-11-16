// The native compiler uses the pinned Lust reference and explicit integration
// policy. The standalone-output proposal is unadmitted during qualification.
// This crate supplies the owned bounded C ABI entries and native complement.
#[cfg(not(feature = "native-inference"))]
include!("../../../compiler/native/src/main.rs");
#[cfg(feature = "native-inference")]
include!("native_modules.rs");

#[cfg(feature = "native-inference")]
mod inference;

#[cfg(feature = "native-inference")]
mod native_object;
#[cfg(feature = "native-inference")]
mod native_memory;
#[cfg(feature = "native-inference")]
mod native_fill;
#[cfg(feature = "native-inference")]
mod unit_object;
#[cfg(feature = "native-inference")]
mod unit_publication;
#[cfg(feature = "native-inference")]
mod native_crate;

use std::ffi::{CStr, c_char, c_void};
type Sink = unsafe extern "C" fn(*mut c_void, i32, *const u8, usize);
// Native setup and TinyCC have process-global state. Serialize every entry
// into that pipeline, including calls through different experimental ABIs.
static NATIVE_ENTRY: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Experimental native object phase interface. No compiler fallback, Cargo
/// metadata fiction, std runtime or retained-result reuse is involved.
#[cfg(feature = "native-inference")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_object_v1(argc: usize, argv: *const *const c_char, sink: Sink, context: *mut c_void) -> i32 {
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
    unsafe { native_entry(argc, argv, namespace, sink, context, false, false, None) }
}

/// Experimental bounded sequential ABI. It reuses only validated successful
/// artifacts in this loaded compiler instance; it retains no transformed AST.
#[cfg(feature = "native-inference")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_retained_v1(
    argc: usize,
    argv: *const *const c_char,
    envc: usize,
    envp: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
) -> i32 {
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    if CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 64 {
        return 3;
    }
    unsafe { sequential_entry(argc, argv, envc, envp, namespace, sink, context, false) }
}

/// Experimental shared native Session. Same-thread sequential calls only, at
/// most 64 per process. No executable reuse, daemon, or compiler fallback.
#[cfg(feature = "native-inference")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_native_session_v1(
    argc: usize,
    argv: *const *const c_char,
    envc: usize,
    envp: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
) -> i32 {
    static OWNER: std::sync::Mutex<Option<std::thread::ThreadId>> = std::sync::Mutex::new(None);
    static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let Ok(mut owner) = OWNER.try_lock() else { return 3; };
    let current = std::thread::current().id();
    if owner.as_ref().is_some_and(|owner| *owner != current) { return 2; }
    if CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 64 { return 3; }
    *owner = Some(current);
    unsafe { sequential_entry(argc, argv, envc, envp, namespace, sink, context, true) }
}

#[cfg(feature = "native-inference")]
unsafe fn sequential_entry(
    argc: usize,
    argv: *const *const c_char,
    envc: usize,
    envp: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
    shared: bool,
) -> i32 {
    if envc > 8192 || (envc != 0 && envp.is_null()) || envc % 2 != 0 {
        return 2;
    }
    let raw = if envc == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(envp, envc) } };
    let env: Option<Vec<String>> =
        raw.iter().map(|p| if p.is_null() { None } else { unsafe { CStr::from_ptr(*p) }.to_str().ok().map(str::to_owned) }).collect();
    let Some(env) = env else {
        return 1;
    };
    let env: Vec<_> = env.chunks_exact(2).map(|p| (p[0].clone(), p[1].clone())).collect();
    if env.iter().any(|(k, _)| k.is_empty() || k.contains('=')) {
        return 2;
    }
    let mut names = std::collections::BTreeSet::new();
    if env.iter().any(|(k, _)| !names.insert(k)) {
        return 2;
    }
    unsafe { native_entry(argc, argv, namespace, sink, context, true, shared, Some(env)) }
}

unsafe fn native_entry(
    argc: usize,
    argv: *const *const c_char,
    namespace: *const c_char,
    sink: Sink,
    context: *mut c_void,
    retained: bool,
    shared: bool,
    client_env: Option<Vec<(String, String)>>,
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
    if (req.session.krate.manifest_path.is_some() && !cfg!(feature = "native-inference"))
        || req.cc
        || req.run_args.is_some()
        || image::state_dir().is_some()
    {
        return 2;
    }
    #[cfg(feature = "native-inference")]
    if std::env::vars_os().any(|(k, v)| k.to_str().is_none() || v.to_str().is_none()) {
        // Native graph setup has Unicode-only environment observations and
        // its macro API otherwise treats a non-Unicode value as absent.
        // Refuse the unsupported input before either can lose that fact.
        let message = b"error: native inference does not support a non-Unicode caller environment\n";
        unsafe { sink(context, 2, message.as_ptr(), message.len()) };
        return 1;
    }
    // Cache identity includes the installed entry library's hash and runtime
    // family, without changing the caller's environment or source bytes.
    req.session.krate.package_id.push_str("#lustr-native-v2:");
    req.session.krate.package_id.push_str(namespace);
    own_env(&mut req);
    if let Some(env) = client_env {
        req.client.env = Some(env);
    }
    raise_stack_limit();
    tcc::use_mimalloc();
    let start = std::time::Instant::now();
    let cache = if !retained && req.emit_c.is_none() { driver::cache::Cache::open(&req.file) } else { None };
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
        #[cfg(not(feature = "native-inference"))]
        let dir = std::env::temp_dir().join(format!("lust-{}", std::process::id()));
        #[cfg(not(feature = "native-inference"))]
        let mut session = {
            let mut session = driver::Session::new(dir, false, req.session.clone());
            session.caches.leak_at_exit = true;
            session
        };
        let compiled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            #[cfg(feature = "native-inference")]
            {
                if shared { inference::session_compile(&req) }
                else if retained { inference::retained_compile(&req) }
                else { inference::compile(&req) }
            }
            #[cfg(not(feature = "native-inference"))]
            {
                driver::compile(&req, &mut session)
            }
        }));
        #[cfg(not(feature = "native-inference"))]
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
        #[cfg(not(feature = "native-inference"))]
        std::mem::forget(session);
        response
    };
    unsafe {
        sink(context, 1, response.stdout.as_ptr(), response.stdout.len());
        sink(context, 2, response.stderr.as_ptr(), response.stderr.len());
    }
    response.code
}
