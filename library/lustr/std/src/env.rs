#[path = "../../../lust/library/std/src/env.rs"]
mod baseline;
pub use baseline::*;

/// Native executable and shared-library naming on the qualified Linux host.
pub mod consts {
    pub use super::baseline::consts::*;

    #[cfg(target_os = "linux")]
    pub const EXE_SUFFIX: &str = "";
    #[cfg(target_os = "linux")]
    pub const EXE_EXTENSION: &str = "";
    #[cfg(target_os = "linux")]
    pub const DLL_PREFIX: &str = "lib";
    #[cfg(target_os = "linux")]
    pub const DLL_SUFFIX: &str = ".so";
    #[cfg(target_os = "linux")]
    pub const DLL_EXTENSION: &str = "so";
}
