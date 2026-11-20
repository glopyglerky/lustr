//! Exact upstream musl fill compatibility dependency. This is C computation,
//! not owned Rust computation or whole-toolchain/no_std qualification.
use crate::{unit_object,native_memory,native_crate::{Backend,Generation}};
use std::{fs,path::{Path,PathBuf},process::Command};

pub(crate) const SOURCE:&str=include_str!("../../../upstream/runtime/musl-1.2.5/memset.c");
pub(crate) fn provider(dir:&Path,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    let input=dir.join("upstream-fill.c");let object=dir.join("upstream-fill.o");fs::write(&input,SOURCE).map_err(|e|e.to_string())?;
    backend.verify(generation)?;
    let command=Command::new(backend.executable()).args(["--target=x86_64-unknown-linux-gnu","-std=gnu11","-ffreestanding","-fno-builtin","-fno-strict-aliasing","-fwrapv","-O0","-c"]).arg(&input).arg("-o").arg(&object).output().map_err(|e|e.to_string())?;
    fs::write(dir.join("upstream-fill.stderr"),&command.stderr).map_err(|e|e.to_string())?;
    if !command.status.success(){return Err(format!("upstream fill compilation: {}",String::from_utf8_lossy(&command.stderr)));}backend.verify(generation)?;
    let symbols=unit_object::symbols(&fs::read(&object).map_err(|e|e.to_string())?)?;
    if !symbols.undefined.is_empty()||symbols.defined!=[b"memset".to_vec()].into_iter().collect()||symbols.functions!=symbols.defined{return Err("upstream fill provider has imports or unexpected exports".into());}Ok(object)
}
pub(crate) fn close(object:&Path,dir:&Path,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    let symbols=unit_object::symbols(&fs::read(object).map_err(|e|e.to_string())?)?;
    if !symbols.undefined.contains(b"memset".as_slice()){return Ok(object.into());}
    let fill=provider(dir,generation,backend)?;let provided=unit_object::symbols(&fs::read(&fill).map_err(|e|e.to_string())?)?;
    native_memory::link_private(object,&fill,&provided,dir,"fill",generation,backend)
}
