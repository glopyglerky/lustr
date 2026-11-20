//! Explicit native runtime dependencies and original cross-crate locations.
//! These records do not select a no_std panic handler or supply core/alloc.
//! Host execution uses the unchanged upstream runtime and its declared host
//! link. A bare freestanding object still requires an empty undefined set.
use crate::{cgen,def,diag,check,native_memory,unit_object};
use super::{Backend,Generation,digest};
use std::{collections::BTreeSet,fs,path::{Path,PathBuf},process::Command};

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Location { pub id:def::DiskDefId, pub path:String, pub line:u32 }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct Requirements { pub source:[u8;32], pub imports:BTreeSet<Vec<u8>>, pub locations:Vec<Location> }
pub(crate) fn source_digest()->Result<[u8;32],String>{let mut source=cgen::runtime_unit().into_bytes();source.extend_from_slice(crate::native_fill::SOURCE.as_bytes());digest(&source)}
impl Requirements {
    pub(crate) fn capture(p:&check::collect::Program,map:&diag::SourceMap)->Result<Self,String>{
        let locations=p.fns.iter().enumerate().map(|(n,f)|{let at=map.call_site(f.def(&p.ast).name_span);Location{id:p.disk_id(def::LocalDefId(n as u32)),path:map.path(at.file).into(),line:map.line_display_col(at).0}}).collect();
        Ok(Self{source:source_digest()?,imports:BTreeSet::new(),locations})
    }
    pub(crate) fn validate(&self,functions:&[super::Function],object:&[u8])->Result<(),String>{
        if self.source!=source_digest()?||self.locations.len()!=functions.len()||self.imports.len()>4096{return Err("native runtime source/location generation mismatch".into());}
        for(l,f)in self.locations.iter().zip(functions){if l.id!=f.id||l.line==0||l.path.is_empty()||l.path.len()>16384||l.path.contains('\0'){return Err("native runtime location/definition mismatch".into());}}
        if self.imports.iter().any(|s|s.is_empty()||s.len()>256||s.contains(&0))||unit_object::symbols(object)?.undefined!=self.imports{return Err("native runtime dependency edges differ from the actual object".into());}Ok(())
    }
    pub(crate) fn encode(&self)->Vec<u8>{let mut out=self.source.to_vec();out.extend_from_slice(&(self.imports.len()as u32).to_le_bytes());for s in &self.imports{super::blob(&mut out,s);}out.extend_from_slice(&(self.locations.len()as u32).to_le_bytes());for l in &self.locations{out.extend_from_slice(&l.id.to_bytes());super::text(&mut out,&l.path);out.extend_from_slice(&l.line.to_le_bytes());}out}
    pub(crate) fn decode(bytes:&[u8])->Result<Self,String>{let mut r=super::Reader(bytes);let source=r.hash()?;let count=r.len(4096)?;let mut imports=BTreeSet::new();for _ in 0..count{let n=r.len(256)?;if !imports.insert(r.bytes(n)?.to_vec()){return Err("duplicate native runtime dependency".into());}}let count=r.len(4096)?;let mut locations=Vec::new();for _ in 0..count{let id=def::DiskDefId::from_bytes(r.bytes(16)?).ok_or("native runtime definition identity")?;let path=r.text(16384)?;let line=u32::from_le_bytes(r.bytes(4)?.try_into().unwrap());locations.push(Location{id,path,line});}if !r.0.is_empty(){return Err("native runtime trailing data".into());}Ok(Self{source,imports,locations})}
}

pub(crate) fn location_unit(locations:&[Location])->Result<String,String>{
    let mut ids=BTreeSet::new();for l in locations{if !ids.insert(l.id.hash){return Err("native runtime location symbol collision".into());}}
    let names=cgen::names::Names{fns:locations.iter().map(|l|Some(format!("{:016x}",l.id.hash.0))).collect(),adts:Vec::new(),by_id:Default::default()};
    let paths:Vec<_>=locations.iter().map(|l|l.path.as_str()).collect();let files:Vec<_>=(0..locations.len()).map(|n|n as u32).collect();let lines:Vec<_>=locations.iter().map(|l|l.line).collect();let slots:Vec<_>=(0..locations.len()).map(|n|n as u32).collect();
    Ok(cgen::names::with(Some(&std::sync::Arc::new(names)),||cgen::names::loc_unit(&slots,&cgen::SlotFiles{paths:&paths,of_slot:&files},&lines,true)))
}
pub(crate) fn compile(source:&str,label:&str,dir:&Path,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    let input=dir.join(format!("{label}.c"));let object=dir.join(format!("{label}.o"));fs::write(&input,source).map_err(|e|e.to_string())?;
    backend.verify(generation)?;
    let command=Command::new(backend.executable()).args(["--target=x86_64-unknown-linux-gnu","-std=gnu11","-fno-strict-aliasing","-fwrapv","-O0","-c"]).arg(&input).arg("-o").arg(&object).output().map_err(|e|e.to_string())?;
    fs::write(dir.join(format!("{label}.stdout")),&command.stdout).map_err(|e|e.to_string())?;fs::write(dir.join(format!("{label}.stderr")),&command.stderr).map_err(|e|e.to_string())?;
    if !command.status.success(){return Err(format!("native runtime provider {label}: {}",String::from_utf8_lossy(&command.stderr)));}backend.verify(generation)?;Ok(object)
}
pub(crate) struct Provider { pub object:PathBuf, symbols:unit_object::Symbols }
impl Provider {
    pub(crate) fn compile(dir:&Path,generation:&Generation,backend:&Backend)->Result<Self,String>{let object=compile(&cgen::runtime_unit(),"runtime",dir,generation,backend)?;let symbols=unit_object::symbols(&fs::read(&object).map_err(|e|e.to_string())?)?;Ok(Self{object,symbols})}
    pub(crate) fn check(&self,requirements:&Requirements,location_symbols:&unit_object::Symbols,object:&[u8])->Result<(),String>{
        let actual=unit_object::symbols(object)?;
        if !self.symbols.defined.is_disjoint(&location_symbols.defined)||!location_symbols.undefined.is_subset(&self.symbols.defined){return Err("native runtime/location provider closure conflict".into());}
        if requirements.source!=source_digest()?||actual.undefined!=requirements.imports||!actual.defined.is_disjoint(&self.symbols.defined)||!actual.defined.is_disjoint(&location_symbols.defined){return Err("native runtime provider/definition conflict".into());}
        let definitions:BTreeSet<_>=self.symbols.defined.union(&location_symbols.defined).cloned().collect();
        if !actual.undefined.is_subset(&definitions){return Err(format!("native object has dependencies absent from the actual native runtime/location providers: {:?}",actual.undefined.difference(&definitions).collect::<Vec<_>>()));}
        for(name,used)in unit_object::dependency_uses(object)?{
            let provided=self.symbols.kinds.get(&name).or_else(||location_symbols.kinds.get(&name)).ok_or("native runtime provider symbol kind absent")?;
            if *provided==unit_object::SymbolKind::Unknown||*provided!=used{return Err("native runtime dependency function/data kind mismatch".into());}
        }Ok(())
    }
}

/// Close memory exactly, then prove every remaining edge against actual
/// original runtime and location objects. No platform/libc symbol is admitted
/// merely because the final host linker could find it.
pub(crate) fn prepare(object:&Path,dir:&Path,requirements:&mut Requirements,generation:&Generation,backend:&Backend)->Result<PathBuf,String>{
    let symbols=unit_object::symbols(&fs::read(object).map_err(|e|e.to_string())?)?;
    if symbols.undefined.is_empty(){return Ok(object.into());}
    let memory=native_memory::Provider::compile(dir,generation,backend)?;
    if symbols.undefined.is_subset(&memory.symbols.functions){return native_memory::close(object,dir,generation,backend);}
    let object=memory.link(object,dir,generation,backend)?;
    let object=crate::native_fill::close(&object,dir,generation,backend)?;
    let bytes=fs::read(&object).map_err(|e|e.to_string())?;requirements.imports=unit_object::symbols(&bytes)?.undefined;
    let runtime=Provider::compile(dir,generation,backend)?;
    let locations=compile(&location_unit(&requirements.locations)?,"provider-locations",dir,generation,backend)?;
    let locations=unit_object::symbols(&fs::read(locations).map_err(|e|e.to_string())?)?;
    runtime.check(requirements,&locations,&bytes)?;Ok(object)
}
