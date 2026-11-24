//! Close compiler-generated memory calls with Lust's exact upstream memory
//! unit. This remains an upstream C compatibility dependency, not owned Rust
//! memory computation or libc/no_std-runtime qualification.
use crate::{cc, native_crate::{Backend,Generation},unit_object};
use std::{fs,path::{Path,PathBuf},process::Command};
pub(crate) fn close(object:&Path,scratch:&Path,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    let symbols=unit_object::symbols(&fs::read(object).map_err(|e|e.to_string())?)?;
    if symbols.undefined.is_empty(){return Ok(object.into());}
    let memory=Provider::compile(scratch,generation,backend)?;
    if !symbols.undefined.is_subset(&memory.symbols.functions){return Err(format!("native freestanding object has unprovided runtime symbols: {:?}",symbols.undefined));}
    let closed=memory.link(object,scratch,generation,backend)?;
    if !unit_object::symbols(&fs::read(&closed).map_err(|e|e.to_string())?)?.undefined.is_empty(){return Err("native memory closure remains incomplete".into());}Ok(closed)
}

/// The actual original memory object, with no imports. Linking it closes only
/// its real definitions; callers still owe every remaining runtime dependency.
pub(crate) struct Provider { object:PathBuf, pub(crate) symbols:unit_object::Symbols }
impl Provider {
pub(crate) fn compile(scratch:&Path,generation:&Generation,backend:&Backend)->Result<Self,String>{
    backend.verify(generation)?;
    let source=scratch.join("upstream-memory.c");fs::write(&source,cc::memfast_unit()).map_err(|e|e.to_string())?;
    let memory=scratch.join("upstream-memory.o");
    let output=Command::new(backend.executable()).args(["--target=x86_64-unknown-linux-gnu","-std=gnu11","-ffreestanding","-fno-builtin","-fno-strict-aliasing","-fwrapv","-O0","-c"]).arg(&source).arg("-o").arg(&memory).output().map_err(|e|e.to_string())?;
    if !output.status.success(){return Err(format!("upstream memory compilation: {}",String::from_utf8_lossy(&output.stderr)));}
    let provided=unit_object::symbols(&fs::read(&memory).map_err(|e|e.to_string())?)?;
    if !provided.undefined.is_empty(){return Err("upstream memory provider has unresolved dependencies".into());}
    backend.verify(generation)?;Ok(Self{object:memory,symbols:provided})
}
pub(crate) fn link(&self,object:&Path,scratch:&Path,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    link_private(object,&self.object,&self.symbols,scratch,"memory",generation,backend)
}
}
/// Relocate and localize a concrete import-free memory provider. The caller
/// owns its semantic provenance; this mechanical join grants no admission.
pub(crate) fn link_private(object:&Path,provider:&Path,provided:&unit_object::Symbols,scratch:&Path,label:&str,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    if !provided.undefined.is_empty(){return Err("private memory provider has imports".into());}
    let symbols=unit_object::symbols(&fs::read(object).map_err(|e|e.to_string())?)?;
    if !provided.defined.is_disjoint(&symbols.defined){return Err("native object overlaps the original memory provider".into());}
    for(name,kind)in unit_object::dependency_uses(&fs::read(object).map_err(|e|e.to_string())?)?{if provided.defined.contains(&name)&&provided.kinds.get(&name)!=Some(&kind){return Err("private memory dependency function/data kind mismatch".into());}}
    backend.verify(generation)?;
    let combined=scratch.join(format!("{label}-closed.o"));let output=Command::new(backend.linker()).arg("-r").arg(object).arg(provider).arg("-o").arg(&combined).output().map_err(|e|e.to_string())?;
    if !output.status.success(){return Err(format!("native memory closure link: {}",String::from_utf8_lossy(&output.stderr)));}
    // Each producer owns a private copy: runtime definitions cannot collide
    // across independent crates or replace the host process's libc symbols.
    use std::os::unix::ffi::OsStringExt;
    let localized=scratch.join(format!("{label}-closed-local.o"));let mut cmd=Command::new(backend.objcopy());
    for name in &provided.defined{let mut arg=b"--localize-symbol=".to_vec();arg.extend_from_slice(name);cmd.arg(std::ffi::OsString::from_vec(arg));}
    let output=cmd.arg(&combined).arg(&localized).output().map_err(|e|e.to_string())?;
    if !output.status.success(){return Err(format!("native memory closure localization: {}",String::from_utf8_lossy(&output.stderr)));}
    backend.verify(generation)?;
    let actual=unit_object::symbols(&fs::read(&localized).map_err(|e|e.to_string())?)?;
    let remaining=symbols.undefined.difference(&provided.defined).cloned().collect();
    if actual.undefined!=remaining||actual.defined!=symbols.defined||actual.functions!=symbols.functions{return Err("native memory closure altered exported symbols or dependency edges".into());}Ok(localized)
}
