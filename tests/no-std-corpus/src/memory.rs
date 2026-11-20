//! Explicit freestanding memory ABI for this corpus. Volatile byte operations
//! prevent these implementations from lowering recursively to themselves.
use core::ffi::c_void;

#[unsafe(no_mangle)]
unsafe extern "C" fn memset(dst: *mut c_void, byte: i32, count: usize) -> *mut c_void {
    for index in 0..count {
        // SAFETY: the caller supplies count writable bytes; zero counts never
        // evaluate an offset or dereference, including for null pointers.
        unsafe { dst.cast::<u8>().add(index).write_volatile(byte as u8); }
    }
    dst
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcmp(left: *const c_void, right: *const c_void, count: usize) -> i32 {
    for index in 0..count {
        // SAFETY: the caller supplies count readable bytes at both pointers.
        let (a, b) = unsafe { (left.cast::<u8>().add(index).read_volatile(), right.cast::<u8>().add(index).read_volatile()) };
        if a != b { return i32::from(a) - i32::from(b); }
    }
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn memcpy(dst: *mut c_void, source: *const c_void, count: usize) -> *mut c_void {
    for index in 0..count {
        // SAFETY: the caller supplies count readable/writable nonoverlapping
        // bytes. Zero counts never evaluate a pointer offset or dereference.
        unsafe { dst.cast::<u8>().add(index).write_volatile(source.cast::<u8>().add(index).read_volatile()); }
    }
    dst
}
