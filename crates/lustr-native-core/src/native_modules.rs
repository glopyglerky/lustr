// Explicit composition for the opt-in native extension. Every upstream module
// below uses the pinned reference and explicit integration policy. The current
// standalone-output proposal is unadmitted during qualification. The default
// ABI still includes main.rs.
// Only the parser and collection interfaces are supplied by owned facades.
// Ordinary module declarations preserve each file's native child-module
// directory. No generated source, copied implementation or patched overlay.
#[path = "../../../compiler/native/src"]
mod pinned {
    #[allow(dead_code)] pub(crate) mod ast;
    pub(crate) mod borrowck;
    pub(crate) mod capabilities;
    pub(crate) mod cc;
    pub(crate) mod cgen;
    pub(crate) mod check {
        include!("../../../compiler/native/src/check/mod.rs");
        // Additive access to the original private relation, required because
        // upstream generic checking names its checker through crate::check.
        pub(crate) fn lustr_numeric_from_ok(src: crate::ty::TyKind, dst: crate::ty::TyKind) -> bool {
            Ctx::numeric_from_ok(src, dst)
        }
    }
    pub(crate) mod colwidth;
    pub(crate) mod daemon;
    pub(crate) mod def;
    pub(crate) mod diag;
    pub(crate) mod driver;
    pub(crate) mod expand;
    pub(crate) mod explain;
    pub(crate) mod image;
    pub(crate) mod intern;
    pub(crate) mod ir;
    pub(crate) mod lexer;
    pub(crate) mod lower;
    #[allow(dead_code)] pub(crate) mod mono;
    pub(crate) mod parser;
    pub(crate) mod pool;
    pub(crate) mod protocol;
    #[allow(dead_code)] pub(crate) mod resolve;
    pub(crate) mod scan;
    pub(crate) mod stdload;
    pub(crate) mod stdloc;
    pub(crate) mod stdmods;
    pub(crate) mod tcc;
    pub(crate) mod testrun;
    #[allow(dead_code)] pub(crate) mod traits;
    pub(crate) mod ty;
    pub(crate) mod unicode;
}
use pinned::{ast, borrowck, capabilities, cc, cgen, colwidth, daemon, def, diag,
    driver, expand, explain, image, intern, ir, lexer, lower, mono, pool, protocol,
    resolve, scan, stdload, stdloc, stdmods, tcc, testrun, traits, ty, unicode};
#[path = "expanded_parser.rs"] mod parser;
#[path = "expanded_check.rs"] mod check;

use protocol::Invocation;
// rustc_driver's shared Rust runtime and this executable exchange owned Rust
// allocations. Its embedding uses System consistently; a private mimalloc
// global would free blocks allocated inside the DSO with another allocator.
// The ordinary native worker retains its original image allocator unchanged.
#[global_allocator]
#[cfg(not(feature = "upstream-semantic"))]
static GLOBAL: image::Alloc = image::Alloc;
const BIG_FILE: u64 = 256 << 10;
fn own_env(req: &mut driver::Request) { req.client.env = None; }

// Linux/Darwin host ABI, kept narrow. The opt-in component uses the same native stack
// budget; this is platform access, not an all-Rust runtime safety claim.
fn raise_stack_limit() {
    #[repr(C)] struct Limit { soft: u64, hard: u64 }
    unsafe extern "C" {
        fn getrlimit(resource: i32, limit: *mut Limit) -> i32;
        fn setrlimit(resource: i32, limit: *const Limit) -> i32;
    }
    let mut limit = Limit { soft: 0, hard: 0 };
    // SAFETY: both calls receive the two u64 fields of the native rlimit.
    if unsafe { getrlimit(3, &mut limit) } == 0 {
        let wanted = (pool::STACK_SIZE as u64).min(limit.hard);
        if wanted > limit.soft {
            limit.soft = wanted;
            unsafe { setrlimit(3, &limit) };
        }
    }
}
