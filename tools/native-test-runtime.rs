//! Adapter between native test descriptors and the pinned Rust libtest.
#![feature(test)]

extern crate test;

use std::ffi::{CStr, c_char, c_int, c_void};
use std::process::ExitCode;
use std::sync::Arc;
use test::{ShouldPanic, TestDesc, TestDescAndFn, TestFn, TestName, TestType};

type CopyMessage = unsafe extern "C" fn(*mut c_void, *const u8, usize);
type Run = unsafe extern "C" fn(*mut c_void, CopyMessage) -> c_int;

#[repr(C)]
pub struct Test {
    pub name: *const c_char,
    pub file: *const c_char,
    pub ignore_reason: *const c_char,
    pub expected: *const c_char,
    pub line: u32,
    pub column: u32,
    pub flags: u32,
    pub run: Run,
}

unsafe extern "C" fn copy_message(output: *mut c_void, ptr: *const u8, len: usize) {
    // The call is synchronous. The emitter holds the original panic payload
    // until the copy completes, then owns its destruction.
    let output = unsafe { &mut *output.cast::<Option<String>>() };
    let bytes = unsafe { message_bytes(ptr, len) };
    *output = std::str::from_utf8(bytes).ok().map(str::to_owned);
}

unsafe fn message_bytes<'a>(ptr: *const u8, len: usize) -> &'a [u8] {
    if len == 0 {
        &[]
    } else {
        // Strings are borrowed from the invoking native frame for this call.
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// # Safety
/// The native runtime supplies UTF-8 print data borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_test_write(to_stderr: c_int, bytes: *const u8, len: usize) {
    let message = String::from_utf8_lossy(unsafe { message_bytes(bytes, len) });
    if to_stderr != 0 {
        eprint!("{message}");
    } else {
        print!("{message}");
    }
}

/// # Safety
/// Native source and panic text is borrowed through this default-hook call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_test_panic_message(
    bytes: *const u8,
    len: usize,
    file: *const c_char,
    file_len: usize,
    line: u32,
    column: u32,
    tid: u64,
    note: c_int,
) {
    let message = String::from_utf8_lossy(unsafe { message_bytes(bytes, len) });
    let file = String::from_utf8_lossy(unsafe { message_bytes(file.cast(), file_len) });
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("<unnamed>");
    eprintln!("\nthread '{name}' ({tid}) panicked at {file}:{line}:{column}:\n{message}");
    if note != 0 {
        eprintln!("note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace");
    }
}

unsafe fn text(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        return None;
    }
    // Test descriptor strings are immutable and part of the native executable.
    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

fn runnable(run: Run) -> TestFn {
    TestFn::DynTestFn(Arc::new(move || {
        let mut message: Option<String> = None;
        let context = (&mut message as *mut Option<String>).cast();
        // The native thunk catches and drops its original panic. It returns
        // a string payload as Err, which libtest folds into its own result.
        match unsafe { run(context, copy_message) } {
            0 => Ok(()),
            1 => Err(message.expect("native string panic has a valid UTF-8 message")),
            2 => std::panic::resume_unwind(Box::new(NativePanic)),
            status @ 257..=511 => test::assert_test_result(ExitCode::from((status - 256) as u8)),
            _ => unreachable!("invalid native test status"),
        }
    }))
}

#[derive(Debug)]
struct NativePanic;

unsafe fn descriptor(t: &Test) -> Option<TestDescAndFn> {
    let name = unsafe { text(t.name) }?;
    let source_file = unsafe { text(t.file) }?;
    let ignore_message = unsafe { text(t.ignore_reason) };
    let should_panic = if t.flags & 2 == 0 {
        ShouldPanic::No
    } else if t.expected.is_null() {
        ShouldPanic::Yes
    } else {
        ShouldPanic::YesWithMessage(unsafe { text(t.expected) }?)
    };
    let test_type = match t.flags >> 2 {
        0 => TestType::Unknown,
        1 => TestType::UnitTest,
        2 => TestType::IntegrationTest,
        _ => return None,
    };
    Some(TestDescAndFn {
        desc: TestDesc {
            name: TestName::StaticTestName(name),
            ignore: t.flags & 1 != 0,
            ignore_message,
            source_file,
            start_line: t.line as usize,
            start_col: t.column as usize,
            end_line: t.line as usize,
            end_col: t.column as usize,
            should_panic,
            compile_fail: false,
            no_run: false,
            test_type,
        },
        testfn: runnable(t.run),
    })
}

/// # Safety
/// argv is the host argc/argv array. The compiler owns immutable descriptors
/// and their strings for the life of this executable. Each callback follows
/// the synchronous native test ABI and never propagates a foreign exception.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lustr_test_main(argc: c_int, argv: *const *const c_char, count: usize, tests: *const Test) -> c_int {
    let mut args = Vec::new();
    for index in 0..argc as usize {
        let ptr = unsafe { *argv.add(index) };
        let Ok(arg) = unsafe { CStr::from_ptr(ptr) }.to_str() else {
            eprintln!("error: expected UTF-8 test arguments");
            return 101;
        };
        args.push(arg.to_owned());
    }
    let mut tests: Vec<_> = (0..count)
        .map(|index| unsafe { descriptor(&*tests.add(index)) })
        .collect::<Option<Vec<_>>>()
        .expect("valid native test descriptors");
    tests.sort_by(|a, b| a.desc.name.as_slice().cmp(b.desc.name.as_slice()));
    let refs: Vec<_> = tests.iter().collect();
    if test::test_main(&args, &refs) == ExitCode::SUCCESS { 0 } else { 101 }
}
