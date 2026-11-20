#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_metadata;
extern crate rustc_middle;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

struct Probe;
impl Callbacks for Probe {
    fn config(&mut self, config: &mut interface::Config) {
        config.override_queries = Some(|_, providers| {
            providers.queries.typeck_root = |_, _| panic!("PROBE: consumer body typechecking was demanded");
            providers.queries.analysis = |_, _| panic!("PROBE: complete analysis was demanded by metadata");
        });
    }
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        eprintln!("PROBE: entered genuine metadata encoder after expansion");
        let _ = rustc_metadata::fs::encode_and_write_metadata(tcx);
        eprintln!("PROBE: encoder unexpectedly completed without analysis");
        Compilation::Stop
    }
}
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    rustc_driver::catch_with_exit_code(|| rustc_driver::compiler_entrypoint(&args, &mut Probe))
}
