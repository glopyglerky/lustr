use std::ffi::{CString, c_char, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[path = "host_runtime.rs"]
mod host_runtime;
#[allow(unused_imports)]
pub use host_runtime::NATIVE_LIBRARY;

unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn getrlimit(resource: i32, limit: *mut Limit) -> i32;
    fn setrlimit(resource: i32, limit: *const Limit) -> i32;
    #[cfg(target_os = "linux")]
    fn prctl(option: i32, ...) -> i32;
}
#[repr(C)]
struct Limit {
    current: u64,
    maximum: u64,
}
type Sink = unsafe extern "C" fn(*mut c_void, i32, *const u8, usize);
type Compile = unsafe extern "C" fn(usize, *const *const c_char, *const c_char, Sink, *mut c_void) -> i32;

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
    #[cfg(target_os = "linux")]
    {
        unsafe { prctl(42, 0u64, 0u64, 0u64, 0u64) == 0 }
    }
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

pub fn compile(library: &Path, args: &[String]) -> Option<Result<(i32, Vec<u8>, Vec<u8>), &'static str>> {
    if args.len() > 4096 || !library.is_file() {
        return None;
    }
    // The upstream one-shot pipeline returns THP to its normal enabled state.
    // Keep nondefault caller policies in a child so native setup cannot change
    // the calling process's policy.
    if !process_compatible() {
        return None;
    }
    Some((|| {
        let path = CString::new(library.as_os_str().as_bytes()).map_err(|_| "invalid native library path")?;
        let args: Vec<_> =
            args.iter().map(|a| CString::new(a.as_bytes())).collect::<Result<_, _>>().map_err(|_| "invalid native argument")?;
        let pointers: Vec<_> = args.iter().map(|a| a.as_ptr()).collect();
        let namespace =
            std::fs::read_to_string(library.with_file_name("lustr-native.id")).map_err(|_| "native compiler identity is missing")?;
        let namespace = namespace.trim();
        if namespace.len() != 64 || !namespace.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid native compiler identity");
        }
        let namespace = CString::new(namespace).unwrap();
        let mut output = (Vec::new(), Vec::new());
        // RTLD_NOW|RTLD_LOCAL keeps the compiler's Rust allocator and internals
        // out of the client ABI. Darwin's RTLD_LOCAL has a distinct value.
        let handle = unsafe { dlopen(path.as_ptr(), host_runtime::RTLD_NOW_LOCAL) };
        if handle.is_null() {
            return Err("cannot load the matched native compiler library");
        }
        let entry = unsafe { dlsym(handle, c"lustr_native_v2".as_ptr()) };
        if entry.is_null() {
            return Err("native compiler library has no matching entry point");
        }
        let mut stack = Limit { current: 0, maximum: 0 };
        let restore_stack = unsafe { getrlimit(3, &mut stack) } == 0;
        // The library remains loaded until exit/exec. Its leaked one-shot
        // session follows upstream's process-lifetime allocation strategy.
        let code = unsafe {
            let context = (&mut output as *mut (Vec<u8>, Vec<u8>)).cast();
            let compile: Compile = std::mem::transmute(entry);
            compile(pointers.len(), pointers.as_ptr(), namespace.as_ptr(), receive, context)
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
