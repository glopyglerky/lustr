//! Compose a local development toolchain from the pinned complete package and
//! the matching mixed/native compiler pair. Existing packages are read only.
//! This uses explicit symlinks; it is not a standalone distribution archive.
use std::{
    fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};
fn link(source: &Path, target: &Path) {
    symlink(source, target).unwrap();
}
fn copy(source: &Path, target: &Path) {
    fs::copy(source, target).unwrap();
    fs::File::open(target).unwrap().sync_all().unwrap();
}
fn main() {
    let mut args = std::env::args_os().skip(1);
    let baseline = PathBuf::from(args.next().expect("BASELINE MIXED_DRIVER NATIVE_CORE DESTINATION")).canonicalize().unwrap();
    let driver = PathBuf::from(args.next().unwrap()).canonicalize().unwrap();
    let native = PathBuf::from(args.next().unwrap()).canonicalize().unwrap();
    let destination = PathBuf::from(args.next().unwrap());
    assert!(args.next().is_none());
    for file in ["bin/lustr", "bin/rustc-full", "bin/lust-core", "bin/cargo", "lib/lust/library/std/src/lib.rs"] {
        assert!(baseline.join(file).is_file(), "incomplete pinned baseline: {file}");
    }
    fs::create_dir(&destination).unwrap();
    let destination = destination.canonicalize().unwrap();
    fs::create_dir(destination.join("bin")).unwrap();
    fs::create_dir(destination.join("lib")).unwrap();
    let component = destination.join("lib/lustr-mixed");
    fs::create_dir(&component).unwrap();
    fs::create_dir(component.join("bin")).unwrap();
    fs::create_dir(component.join("lib")).unwrap();
    for entry in fs::read_dir(&baseline).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name != "bin" && name != "lib" {
            link(&entry.path(), &destination.join(name));
        }
    }
    for entry in fs::read_dir(baseline.join("lib")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name != "lustr-mixed" {
            link(&entry.path(), &destination.join("lib").join(&name));
            link(&entry.path(), &component.join("lib").join(name));
        }
    }
    for entry in fs::read_dir(baseline.join("bin")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if name != "lustr" && name != "rustc" && name != "rustc-full" && name != "rustc-cargo" && name != "lustr-cargo" && name != "rustc-reference" {
            link(&entry.path(), &destination.join("bin").join(name));
        }
    }
    copy(&baseline.join("bin/lustr"), &destination.join("bin/lustr"));
    link(Path::new("lustr"), &destination.join("bin/rustc"));
    copy(&driver, &component.join("bin/lustr-mixed"));
    copy(&native, &component.join("bin/lust-core"));
    link(Path::new("../lib/lustr-mixed/bin/lustr-mixed"), &destination.join("bin/rustc-full"));
    // Cargo sends complete rustc requests. Enter the same mixed driver
    // directly, avoiding an extra lustr process/exec for every Cargo unit.
    // The public bin/rustc still retains its direct-file Lust fast path.
    fs::create_dir(destination.join("bin/lustr-cargo")).unwrap();
    link(Path::new("../rustc-full"), &destination.join("bin/lustr-cargo/rustc"));
    assert_eq!(
        destination.join("bin/lustr-cargo/rustc").canonicalize().unwrap(),
        destination.join("bin/rustc-full").canonicalize().unwrap()
    );
    link(&baseline.join("bin/rustc-full"), &destination.join("bin/rustc-reference"));
    link(&baseline.join("bin/rustc-full"), &component.join("bin/rustc-reference"));
    let mut manifest = fs::OpenOptions::new().write(true).create_new(true).open(destination.join("MIXED-COMPONENTS.txt")).unwrap();
    writeln!(manifest,"Local development composition; retains the referenced baseline.\nRust implementation: upstream source 5b7b37aefcb5b2fba3c2392bc9065b9e4602d494.\nLust implementation: original retained pipeline plus the declared integration diff.\nBaseline={}\nMixed driver={}\nNative core={}",baseline.display(),driver.display(),native.display()).unwrap();
    manifest.sync_all().unwrap();
    for path in
        [component.join("bin"), component.join("lib"), component, destination.join("bin/lustr-cargo"), destination.join("bin"), destination.join("lib"), destination.clone()]
    {
        fs::File::open(path).unwrap().sync_all().unwrap();
    }
    println!("{}", destination.display());
}
