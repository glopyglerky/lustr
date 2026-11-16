use std::ffi::{CString, c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn getrlimit(resource: i32, limit: *mut Limit) -> i32;
    fn setrlimit(resource: i32, limit: *const Limit) -> i32;
    fn prctl(option: i32, ...) -> i32;
}
#[repr(C)]
struct Limit {
    current: u64,
    maximum: u64,
}
type Sink = unsafe extern "C" fn(*mut c_void, i32, *const u8, usize);
type Compile = unsafe extern "C" fn(usize, *const *const c_char, *const c_char, Sink, *mut c_void) -> i32;
type RetainedCompile =
    unsafe extern "C" fn(usize, *const *const c_char, usize, *const *const c_char, *const c_char, Sink, *mut c_void) -> i32;

unsafe extern "C" fn receive(context: *mut c_void, stream: i32, bytes: *const u8, len: usize) {
    // The matched native entry borrows these bytes only for this callback.
    let output = unsafe { &mut *context.cast::<(Vec<u8>, Vec<u8>)>() };
    let bytes = unsafe { std::slice::from_raw_parts(bytes, len) };
    match stream {
        1 => output.0.extend_from_slice(bytes),
        2 => output.1.extend_from_slice(bytes),
        _ => {}
    }
}

pub fn process_compatible() -> bool {
    // PR_GET_THP_DISABLE reads the current policy without changing it.
    unsafe { prctl(42, 0u64, 0u64, 0u64, 0u64) == 0 }
}

pub fn compile(library: &Path, args: &[String]) -> Option<Result<(i32, Vec<u8>, Vec<u8>), &'static str>> {
    compile_entry(library, args, None, false)
}

/// Explicit experiment; the ordinary compiler entry never selects this ABI.
#[allow(dead_code)]
pub fn compile_retained(
    library: &Path,
    args: &[String],
    env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Option<Result<(i32, Vec<u8>, Vec<u8>), &'static str>> {
    compile_entry(library, args, Some(env), false)
}

/// Explicit shared semantic-session experiment; not selected by product routing.
#[allow(dead_code)]
pub fn compile_session(
    library: &Path,
    args: &[String],
    env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Option<Result<(i32, Vec<u8>, Vec<u8>), &'static str>> {
    compile_entry(library, args, Some(env), true)
}

fn compile_entry(
    library: &Path,
    args: &[String],
    client_env: Option<&[(std::ffi::OsString, std::ffi::OsString)]>,
    shared: bool,
) -> Option<Result<(i32, Vec<u8>, Vec<u8>), &'static str>> {
    if args.len() > 4096 || !library.is_file() {
        return None;
    }
    // The upstream one-shot pipeline returns THP to its normal enabled state.
    // Keep nondefault caller policies in a child, where they cannot leak into
    // a subsequent full-core exec.
    if !process_compatible() {
        return None;
    }
    Some((|| {
        let path = CString::new(library.as_os_str().as_bytes()).map_err(|_| "invalid native library path")?;
        let args: Vec<_> =
            args.iter().map(|a| CString::new(a.as_bytes())).collect::<Result<_, _>>().map_err(|_| "invalid native argument")?;
        let pointers: Vec<_> = args.iter().map(|a| a.as_ptr()).collect();
        let env: Vec<CString> = client_env
            .into_iter()
            .flatten()
            .flat_map(|(k, v)| [k, v])
            .map(|v| {
                if v.to_str().is_none() {
                    return Err("native retained environment requires UTF-8");
                }
                CString::new(v.as_os_str().as_bytes()).map_err(|_| "invalid native environment")
            })
            .collect::<Result<_, _>>()?;
        let env_pointers: Vec<_> = env.iter().map(|v| v.as_ptr()).collect();
        let namespace =
            std::fs::read_to_string(library.with_file_name("lustr-native.id")).map_err(|_| "native compiler identity is missing")?;
        let namespace = namespace.trim();
        if namespace.len() != 64 || !namespace.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid native compiler identity");
        }
        let namespace = CString::new(namespace).unwrap();
        let mut output = (Vec::new(), Vec::new());
        // Linux x86_64 is a build constraint. RTLD_NOW|RTLD_LOCAL keeps the
        // unchanged compiler's Rust allocator and internals out of the client ABI.
        let handle = unsafe { dlopen(path.as_ptr(), 2) };
        if handle.is_null() {
            return Err("cannot load the matched native compiler library");
        }
        let entry_name = if shared {
            c"lustr_native_session_v1"
        } else if client_env.is_some() {
            c"lustr_native_retained_v1"
        } else {
            c"lustr_native_v2"
        };
        let entry = unsafe { dlsym(handle, entry_name.as_ptr()) };
        if entry.is_null() {
            return Err("native compiler library has no matching entry point");
        }
        let mut stack = Limit { current: 0, maximum: 0 };
        let restore_stack = unsafe { getrlimit(3, &mut stack) } == 0;
        // The library remains loaded until exit/exec. Its leaked one-shot
        // session follows upstream's process-lifetime allocation strategy.
        let code = unsafe {
            let context = (&mut output as *mut (Vec<u8>, Vec<u8>)).cast();
            if client_env.is_some() {
                let compile: RetainedCompile = std::mem::transmute(entry);
                compile(pointers.len(), pointers.as_ptr(), env_pointers.len(), env_pointers.as_ptr(), namespace.as_ptr(), receive, context)
            } else {
                let compile: Compile = std::mem::transmute(entry);
                compile(pointers.len(), pointers.as_ptr(), namespace.as_ptr(), receive, context)
            }
        };
        if restore_stack {
            let mut current = Limit { current: 0, maximum: 0 };
            if unsafe { getrlimit(3, &mut current) } != 0 {
                return Err("cannot inspect the caller's stack limit after compilation");
            }
            if (current.current != stack.current || current.maximum != stack.maximum) && unsafe { setrlimit(3, &stack) } != 0 {
                return Err("cannot restore the caller's stack limit");
            }
        }
        Ok((code, output.0, output.1))
    })())
}
