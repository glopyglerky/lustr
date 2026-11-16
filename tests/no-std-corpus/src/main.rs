#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
mod memory;

// Align Linux x86_64's initial stack before entering the Rust C-ABI body.
// No host C startup, allocator, libc, pthread or std dependency is supplied.
global_asm!(".global _start", "_start:", "and rsp, -16", "call frame_entry", "ud2");

fn exit(code: usize) -> ! {
    // SAFETY: Linux x86_64 exit consumes only the integer status and cannot
    // return. No output/clobber values survive a noreturn asm expression.
    unsafe { asm!("syscall", in("rax") 60usize, in("rdi") code, options(noreturn, nostack)); }
}

#[unsafe(no_mangle)]
extern "C" fn frame_entry() -> ! {
    exit(if frame_core::exercise() { 0 } else { 41 })
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! { exit(42) }

// The installed GNU core image retains a DWARF personality reference even
// with an aborting consumer. No unwinding is implemented by this corpus:
// an attempted personality call terminates with its own observable status.
#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() -> ! { exit(43) }
