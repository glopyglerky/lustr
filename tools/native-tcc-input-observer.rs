//! Private Linux x86_64 preload for a native C-unit proof replay. Records
//! length-framed raw pathname bytes, never TinyCC's ambiguous human trace.
//! This is not a compiler, daemon, or general syscall sandbox.
use std::{
    cell::Cell,
    ffi::{CStr, c_char, c_int, c_long},
    io::Write,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

unsafe extern "C" {
    fn syscall(number: c_long, ...) -> c_long;
    fn __errno_location() -> *mut c_int;
}
thread_local! { static INSIDE: Cell<bool> = const { Cell::new(false) }; }
static COUNT: AtomicUsize = AtomicUsize::new(0);
static OUTPUT: Mutex<()> = Mutex::new(());

fn record(path: &[u8], result: i32) -> std::io::Result<()> {
    let _lock = OUTPUT.lock().map_err(|_| std::io::Error::other("observer poisoned"))?;
    let mut out = std::io::stdout().lock();
    if path.is_empty() || path.len() > 16384 || COUNT.fetch_add(1, Ordering::Relaxed) >= 8192 {
        out.write_all(&0u32.to_le_bytes())?;
        return out.flush();
    }
    out.write_all(&(path.len() as u32).to_le_bytes())?;
    out.write_all(path)?;
    out.write_all(&result.to_le_bytes())?;
    out.flush()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn open(path: *const c_char, flags: c_int, mut args: ...) -> c_int {
    let mode = if flags & 64 != 0 || flags & 0x410000 == 0x410000 { unsafe { args.next_arg::<std::ffi::c_uint>() } } else { 0 };
    // Linux x86_64 SYS_openat, AT_FDCWD. The third open argument is only
    // consumed by the kernel when the caller's flags require a mode.
    let fd = unsafe { syscall(257, -100i32, path, flags, mode) } as c_int;
    let errno = unsafe { *__errno_location() };
    if flags & 3 == 0 {
        let done = std::panic::catch_unwind(|| {
            INSIDE.with(|inside| {
                if inside.replace(true) {
                    return;
                }
                struct Reset<'a>(&'a Cell<bool>);
                impl Drop for Reset<'_> {
                    fn drop(&mut self) {
                        self.0.set(false);
                    }
                }
                let _reset = Reset(inside);
                let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();
                let result = if flags == 0 { if fd < 0 { -errno } else { 0 } } else { i32::MIN };
                if record(bytes, result).is_err() {
                    let _ = record(&[], i32::MIN);
                }
            })
        });
        if done.is_err() {
            let _ = record(&[], i32::MIN);
        }
    }
    unsafe {
        *__errno_location() = errno;
    }
    fd
}

#[unsafe(no_mangle)]
unsafe extern "C" fn time(output: *mut c_long) -> c_long {
    let result = unsafe { syscall(201, output) };
    let errno = unsafe { *__errno_location() };
    // A clock-dependent macro is an unknown semantic input for retained reuse.
    // The invalid frame rejects proof even when its name was token-pasted.
    let _ = std::panic::catch_unwind(|| {
        let _ = record(&[], i32::MIN);
    });
    unsafe {
        *__errno_location() = errno;
    }
    result
}
