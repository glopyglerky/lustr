//! Run the pinned Rust implementation for host procedural macro crates.
#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_structures;

use rustc_session::{EarlyDiagCtxt, config};
use rustc_structures::CrateType;
use std::process::ExitCode;

rustc_driver::override_c_allocator_in_binary!();

struct HostProcMacro;

impl rustc_driver::Callbacks for HostProcMacro {
    fn config(&mut self, compiler: &mut rustc_interface::interface::Config) {
        if compiler.opts.crate_types != [CrateType::ProcMacro]
            || compiler.opts.target_triple.tuple() != config::host_tuple()
        {
            EarlyDiagCtxt::new(config::ErrorOutputType::default())
                .early_fatal("the host procedural macro compiler requires a host proc-macro crate");
        }
    }
}

fn main() -> ExitCode {
    let diagnostics = EarlyDiagCtxt::new(config::ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&diagnostics);
    rustc_driver::install_ice_hook(rustc_driver::DEFAULT_BUG_REPORT_URL, |_| ());
    rustc_driver::install_ctrlc_handler();
    rustc_driver::catch_with_exit_code(|| {
        rustc_driver::compiler_entrypoint(&rustc_driver::args::raw_args(&diagnostics), &mut HostProcMacro);
    })
}
