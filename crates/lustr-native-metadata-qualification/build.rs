// Reuse the existing native qualification construction. No upstream-semantic
// feature exists in this standalone crate, and no Rust driver is linked.
include!("../lustr-native-core/build.rs");
