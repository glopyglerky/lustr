//! Transport-independent artifact work after one native completion/emission.
//! Shared by the public retained owner and the bounded research caller. Source
//! is consumed only as the already checked snapshot; no frontend enters here.
use crate::{def,diag};
use crate::driver::library::{Object as NativeObject,Snapshot,history};
use crate::driver::library::completion::Function;
use lustr_metadata_encoder as metadata;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{collections::{BTreeMap,BTreeSet},io::Write,path::{Path,PathBuf},sync::Arc};
#[path="unit_object.rs"]
pub(crate) mod unit_object;
#[path="metadata_archive.rs"]
mod archive;

pub(crate) struct Context {
    pub crate_name:String, pub metadata:String, pub extra_filename:String,
    pub source:PathBuf, pub cwd:String, pub profile:metadata::Profile,
}
impl Context {
    pub fn archive_name(&self)->String {format!("lib{}{}.rlib",self.crate_name,self.extra_filename)}
}
pub(crate) struct Service {
    sdk:Arc<metadata::Sdk>, sysroot:PathBuf, context:Context,
    originals:BTreeMap<u64,Original>, renamed:BTreeMap<(u64,String),Renamed>,
    pub artifact:Option<Arc<Artifact>>,
    check_metadata: Option<(u64, Arc<Vec<u8>>)>,
    // At most three embedding choices for the current native snapshot.
    variants:BTreeMap<Option<bool>,Arc<Artifact>>,
    published:Option<Arc<Artifact>>, policy:history::Policy,
    original_used:BTreeMap<u64,u64>, renamed_used:BTreeMap<(u64,String),u64>,
    evicted_originals:Vec<u64>, evicted_renamed:Vec<(u64,String)>,
    active_entries:usize, inactive_entries:usize, inactive_bytes:usize,

}
impl Service {
    pub fn new(sdk:Arc<metadata::Sdk>,sysroot:PathBuf,context:Context)->Result<Self,String> {
        Ok(Self{sdk,sysroot,context,originals:BTreeMap::new(),renamed:BTreeMap::new(),artifact:None,check_metadata:None,variants:BTreeMap::new(),published:None,
            policy:history::policy()?,original_used:BTreeMap::new(),renamed_used:BTreeMap::new(),
            evicted_originals:Vec::new(),evicted_renamed:Vec::new(),active_entries:0,inactive_entries:0,inactive_bytes:0})
    }
    pub fn policy(&self)->history::Policy{self.policy}
    pub fn begin_request(&mut self){self.evicted_originals.clear();self.evicted_renamed.clear();}
    pub fn published(&mut self,artifact:Arc<Artifact>){
        if self.published.as_ref().is_none_or(|old|!Arc::ptr_eq(old,&artifact)) {
            self.published=Some(artifact);self.prune();
        }
    }
    pub fn history(&self)->Value {
        json!({"active_entries":self.active_entries,"inactive_entries":self.inactive_entries,"inactive_bytes":self.inactive_bytes,
            "metadata_variants":self.variants.len(),"metadata_variant_limit":3,
            "evicted_original_serials":self.evicted_originals,"evicted_renamed":self.evicted_renamed.iter().map(|(serial,mapping)|
                json!({"serial":serial,"mapping_sha256":hash(mapping.as_bytes())})).collect::<Vec<_>>()})
    }
    fn prune(&mut self){
        #[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
        enum Key{Original(u64),Renamed((u64,String))}
        let mut protected=BTreeSet::new();
        let generation=self.artifact.as_ref().map_or(0,|a|a.generation);
        for artifact in self.variants.values().chain(self.published.iter()) {
            protected.extend(artifact.native_serials.iter().copied().map(Key::Original));
            protected.extend(artifact.rename_keys.iter().cloned().map(Key::Renamed));
        }
        for key in &protected {match key {
            Key::Original(serial)=>{self.original_used.insert(*serial,generation);},
            Key::Renamed(key)=>{self.renamed_used.insert(key.clone(),generation);},
        }}
        let entries=|this:&Self| {
            this.originals.iter().map(|(serial,original)|(Key::Original(*serial),*this.original_used.get(serial).unwrap_or(&0),original.bytes()))
                .chain(this.renamed.iter().map(|(key,renamed)|(Key::Renamed(key.clone()),*this.renamed_used.get(key).unwrap_or(&0),
                    key.1.capacity()+renamed.bytes.capacity()+renamed.hash.capacity()+std::mem::size_of::<Renamed>()))).collect::<Vec<_>>()
        };
        let inactive=entries(self).into_iter().filter(|(key,_,_)|!protected.contains(key)).collect();
        for key in history::victims(inactive,self.policy){match key {
            Key::Original(serial)=>{self.originals.remove(&serial);self.original_used.remove(&serial);self.evicted_originals.push(serial);},
            Key::Renamed(key)=>{self.renamed.remove(&key);self.renamed_used.remove(&key);self.evicted_renamed.push(key);},
        }}
        self.active_entries=0;self.inactive_entries=0;self.inactive_bytes=0;
        for (key,_,bytes) in entries(self){
            if protected.contains(&key){self.active_entries+=1;}
            else{self.inactive_entries+=1;self.inactive_bytes+=bytes;}
        }
    }
}
struct Original {
    hash: String,
    digest: [u8;32],
    helpers: Vec<String>,
    symbols: BTreeSet<String>,
}
impl Original {
    fn bytes(&self)->usize {std::mem::size_of::<Self>()+self.hash.capacity()+self.helpers.capacity()*std::mem::size_of::<String>()+self.helpers.iter().map(String::capacity).sum::<usize>()+self.symbols.iter().map(String::capacity).sum::<usize>()}
}
#[derive(Clone)]
struct Renamed { bytes: Arc<Vec<u8>>, hash: String }
pub(crate) struct Artifact {
    pub generation: u64,
    pub files: BTreeMap<String,Arc<Vec<u8>>>,
    pub fields: Value,
    pub archive: Arc<Vec<u8>>,
    native_serials:BTreeSet<u64>, rename_keys:BTreeSet<(u64,String)>,
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}
fn put(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("write {}: {error}", path.display()))
}
fn native_id(id: def::DiskDefId) -> metadata::NativeId {
    metadata::NativeId { krate: id.krate.0, path: id.hash.0 }
}
fn span(value: diag::Span) -> metadata::Span {
    metadata::Span { lo: value.lo, hi: value.hi }
}
fn command(root: &Path, label: &str, command: &mut std::process::Command, report: &mut Value) -> Result<Vec<u8>, String> {
    let record = format!("{command:?}");
    let argv:Vec<_>=std::iter::once(command.get_program()).chain(command.get_args()).map(|a|a.to_string_lossy().into_owned()).collect();
    let cwd=command.get_current_dir().map(Path::to_path_buf);
    let started=std::time::Instant::now();
    let result = command.output().map_err(|error| format!("{label}: {error}"))?;
    let duration_ns=started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    put(&root.join(format!("{label}.stdout")), &result.stdout)?;
    put(&root.join(format!("{label}.stderr")), &result.stderr)?;
    report["commands"].as_array_mut().unwrap().push(json!({"name":label,"command":record,"argv":argv,"cwd":cwd,"status":result.status.code(),"duration_ns":duration_ns,
        "stdout_sha256":hash(&result.stdout),"stderr_sha256":hash(&result.stderr)}));
    if !result.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&result.stderr)));
    }
    if !result.stderr.is_empty() {
        return Err(format!("{label}: unexpected successful stderr was preserved"));
    }
    Ok(result.stdout)
}

/// Compare allocated bytes and actual relocation targets across symbol
/// renaming. Numeric symbol indexes may change; their semantic targets may not.
pub(crate) fn object_contents(bytes: &[u8], mapping: &BTreeMap<String, String>) -> Result<Value, String> {
    let file = object::File::parse(bytes).map_err(|error| error.to_string())?;
    let mut sections = Vec::new();
    for section in file.sections() {
        let object::SectionFlags::Elf { sh_flags } = section.flags() else {
            return Err("non-ELF emitted section".into());
        };
        if sh_flags & u64::from(object::elf::SHF_ALLOC) == 0 {
            continue;
        }
        let mut relocations = Vec::new();
        for (offset, relocation) in section.relocations() {
            let target = match relocation.target() {
                object::RelocationTarget::Symbol(index) => {
                    let symbol = file.symbol_by_index(index).map_err(|error| error.to_string())?;
                    if symbol.kind() == object::SymbolKind::Section {
                        let index = symbol.section_index().ok_or("section symbol has no section")?;
                        format!(
                            "section:{}",
                            file.section_by_index(index).map_err(|error| error.to_string())?.name().map_err(|error| error.to_string())?
                        )
                    } else {
                        let name = symbol.name().map_err(|error| error.to_string())?;
                        format!("symbol:{}", mapping.get(name).map_or(name, String::as_str))
                    }
                }
                object::RelocationTarget::Section(index) => format!(
                    "section:{}",
                    file.section_by_index(index).map_err(|error| error.to_string())?.name().map_err(|error| error.to_string())?
                ),
                object::RelocationTarget::Absolute => "absolute".into(),
                _ => return Err("unrepresented emitted relocation target".into()),
            };
            relocations.push(json!([
                offset,
                format!("{:?}", relocation.kind()),
                format!("{:?}", relocation.encoding()),
                relocation.size(),
                relocation.addend(),
                target
            ]));
        }
        sections.push(
            json!({"name":section.name().map_err(|error| error.to_string())?,"size":section.size(),"align":section.align(),"flags":sh_flags,
            "data_sha256":hash(section.data().map_err(|error| error.to_string())?),"relocations":relocations}),
        );
    }
    let mut symbols = Vec::new();
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|error| error.to_string())?;
        let section = match symbol.section() {
            object::SymbolSection::Section(index) => format!(
                "section:{}",
                file.section_by_index(index).map_err(|error| error.to_string())?.name().map_err(|error| error.to_string())?
            ),
            other => format!("{other:?}"),
        };
        symbols.push(json!([mapping.get(name).map_or(name, String::as_str), section,
            symbol.address(), symbol.size(), format!("{:?}", symbol.kind()),
            format!("{:?}", symbol.scope()), symbol.is_weak(), format!("{:?}", symbol.flags())]));
    }
    symbols.sort_by_cached_key(Value::to_string);
    Ok(json!({"sections": sections, "symbols": symbols}))
}

fn id_value(id:def::DiskDefId)->Value { json!([id.krate.0,id.hash.0]) }
fn definition_name(snapshot:&Snapshot,id:def::DiskDefId)->Value {
    let name=snapshot.functions.iter().find(|f|f.definition()==id).map(|f|f.name()).unwrap_or("<absent>");
    json!({"name":name,"native_id":id_value(id)})
}
fn mapped_functions(functions:&[Function])->Vec<metadata::Function> {
    functions.iter().map(|f|metadata::Function {definition:native_id(f.definition()),name:f.name().into(),
        name_span:span(f.name_span()),span:span(f.span()),public:f.visibility()==crate::driver::library::completion::Visibility::Public,
        parameters:f.parameters().iter().map(|p|metadata::Parameter{name:p.name().into(),span:span(p.span())}).collect(),
        calls:f.calls().iter().copied().map(native_id).collect() }).collect()
}
fn check_object(object:&NativeObject, snapshot:&Snapshot)->Result<Original,String> {
    let symbols=unit_object::symbols(&object.bytes)?;
    let function=snapshot.functions.iter().find(|f|f.definition()==object.definition).ok_or("object has no completed definition")?;
    let trampoline=object.symbol.replacen("f_","tramp_",1);
    let definitions=BTreeSet::from([object.symbol.as_bytes().to_vec(),trampoline.as_bytes().to_vec()]);
    let expected_calls:BTreeSet<Vec<u8>>=function.calls().iter().map(|id| {
        snapshot.extern_symbols.get(id).map(String::as_str)
            .or_else(|| snapshot.objects.iter().find(|(o,_)|o.definition==*id).map(|(o,_)|o.symbol.as_str()))
            .map(|symbol| symbol.as_bytes().to_vec()).ok_or("completed call has no actual object or imported linkage")
    }).collect::<Result<_,_>>()?;
    if symbols.defined!=definitions || symbols.functions!=definitions || symbols.undefined!=expected_calls {
        return Err("per-function native ELF definition/call ownership differs from the completed graph".into());
    }
    // TinyCC's actual x64 call relocations must name exactly the completed
    // source callees. Section/local relocations are not external dependencies.
    let file=object::File::parse(object.bytes.as_slice()).map_err(|e|e.to_string())?;
    let mut used=BTreeSet::new();
    for section in file.sections() {
        for (offset,relocation) in section.relocations() {
            let object::RelocationTarget::Symbol(index)=relocation.target() else {continue};
            let symbol=file.symbol_by_index(index).map_err(|e|e.to_string())?;
            if !symbol.is_undefined() {continue;}
            let name=symbol.name_bytes().map_err(|e|e.to_string())?;
            if name.is_empty() {return Err("unnamed undefined relocation".into());}
            let flags=relocation.flags();
            let object::RelocationFlags::Elf{r_type}=flags else {return Err("non-ELF call relocation".into())};
            let data=section.data().map_err(|e|e.to_string())?;
            let offset:usize=offset.try_into().map_err(|_|"relocation offset overflow")?;
            if !matches!(r_type,object::elf::R_X86_64_PC32|object::elf::R_X86_64_PLT32)
                || relocation.addend()!=-4 || offset==0 || !matches!(data.get(offset-1),Some(0xe8|0xe9))
                || offset.checked_add(4).is_none_or(|end|end>data.len()) {
                return Err("native undefined reference is not a represented direct scalar call".into());
            }
            used.insert(name.to_vec());
        }
    }
    if used!=expected_calls {return Err("completed native call lacks actual relocation evidence".into());}
    Ok(Original{hash:hash(&object.bytes),digest:Sha256::digest(object.bytes.as_slice()).into(),helpers:vec![trampoline],symbols:symbols.defined.iter().chain(&symbols.undefined).map(|s|String::from_utf8(s.clone()).map_err(|e|e.to_string())).collect::<Result<_,_>>()?})
}
fn verify_members(bytes:&[u8],files:&BTreeMap<String,Arc<Vec<u8>>>,members:&[String])->Result<(),String> {
    let archive=object::read::archive::ArchiveFile::parse(bytes).map_err(|e|e.to_string())?;
    let mut actual=Vec::new();
    for member in archive.members() {
        let member=member.map_err(|e|e.to_string())?;
        let name=std::str::from_utf8(member.name()).map_err(|e|e.to_string())?;
        let expected=files.get(name).ok_or("archive contains an unowned member")?;
        if member.data(bytes).map_err(|e|e.to_string())?!=expected.as_slice() {return Err("archive member changed actual bytes".into());}
        actual.push(name.to_owned());
    }
    if actual!=members {return Err("archive inventory is missing, duplicated or reordered".into());}
    Ok(())
}
impl Service {
    pub(crate) fn check_metadata(
        &mut self, generation: u64, source: &str, functions: &[Function],
        dependency_context: Option<&metadata::Sdk>, report: &mut Value,
    ) -> Result<Arc<Vec<u8>>, String> {
        if let Some((cached, bytes)) = &self.check_metadata
            && *cached == generation
        {
            report["artifact_reused"] = json!(true);
            return Ok(bytes.clone());
        }
        report["phase"] = json!("source-owned-encode");
        report["counts"]["metadata_encodes"] = json!(1);
        let bytes = Arc::new(metadata::encode_check(&metadata::Input {
            crate_name: &self.context.crate_name,
            metadata: &self.context.metadata,
            extra_filename: &self.context.extra_filename,
            embed_metadata: None,
            source_path: self.context.source.to_str().ok_or("source path is not Unicode")?,
            working_directory: &self.context.cwd,
            source,
            root_span: metadata::Span { lo: 0, hi: source.len() as u32 },
            profile: self.context.profile,
            functions: &mapped_functions(functions),
            emitted: &[],
            backend_helpers: &[],
            objects: &[],
        }, dependency_context.unwrap_or(&self.sdk))?);
        self.check_metadata = Some((generation, bytes.clone()));
        Ok(bytes)
    }

    pub(crate) fn artifact(&mut self,snapshot:&Snapshot,step:&Path,report:&mut Value,materialize:bool)->Result<Arc<Artifact>,String> {
        self.artifact_with_options(snapshot,step,report,materialize,None,None)
    }
    pub(crate) fn artifact_with_options(&mut self,snapshot:&Snapshot,step:&Path,report:&mut Value,materialize:bool,
        embed_metadata:Option<bool>, dependency_context:Option<&metadata::Sdk>)->Result<Arc<Artifact>,String> {
        let stage=step.join("staging");
        report["phase"]=json!("staging");
        let cached=self.variants.get(&embed_metadata).filter(|a|a.generation==snapshot.generation).cloned();
        let reused=cached.is_some();
        if !reused || materialize { std::fs::create_dir(&stage).map_err(|e|format!("create owned staging: {e}"))?; }
        if !reused {
            let functions=mapped_functions(&snapshot.functions);
            let instances:Vec<_>=snapshot.objects.iter().map(|(o,_)|metadata::EmittedFunction{definition:native_id(o.definition),symbol:o.symbol.clone()}).collect();
            for (object,_) in &snapshot.objects {
                if !self.originals.contains_key(&object.serial) {
                    self.originals.insert(object.serial,check_object(object,snapshot)?);
                }
            }
            let mut objects:Vec<_>=snapshot.objects.iter().map(|(o,_)|metadata::ObjectMember{
                member:format!("{}_{:016x}.o",self.context.crate_name,o.definition.hash.0),producer_object_sha256:self.originals[&o.serial].digest,
                definitions:vec![native_id(o.definition)]}).collect();
            objects.sort_by(|a,b|a.member.cmp(&b.member));
            let helpers:Vec<_>=snapshot.objects.iter().flat_map(|(o,_)|self.originals[&o.serial].helpers.clone()).collect();
            report["phase"]=json!("source-owned-encode");report["counts"]["metadata_encodes"]=json!(1);
            let encoded=metadata::encode(&metadata::Input {crate_name:&self.context.crate_name,metadata:&self.context.metadata,extra_filename:&self.context.extra_filename,embed_metadata,
                source_path:self.context.source.to_str().ok_or("source path is not Unicode")?,working_directory:&self.context.cwd,
                source:&snapshot.source,root_span:metadata::Span{lo:0,hi:snapshot.source.len() as u32},
                profile:self.context.profile,functions:&functions,emitted:&instances,backend_helpers:&helpers,objects:&objects},
                dependency_context.unwrap_or(&self.sdk))?;
            let mapping:BTreeMap<_,_>=encoded.symbols.iter().map(|s|(s.native_symbol.clone(),s.rust_symbol.clone())).collect();
            let rename_text:String=mapping.iter().map(|(native,rust)|format!("{native} {rust}\n")).collect();
            let mut files:BTreeMap<String,Arc<Vec<u8>>>=BTreeMap::new();
            files.insert("symbols.map".into(),Arc::new(rename_text.as_bytes().to_vec()));
            files.insert(format!("{}.rmeta",self.context.crate_name),Arc::new(encoded.metadata.clone()));
            files.insert("lib.rmeta".into(),Arc::new(archive::wrapper(b".rmeta",encoded.metadata_stub.as_deref().unwrap_or(&encoded.metadata))?));
            let member_names:Vec<_>=objects.iter().map(|o|o.member.as_str()).collect();
            let link=metadata::encode_rlib_link(&member_names)?;
            files.insert("lib.rmeta-link".into(),Arc::new(archive::wrapper(b".rmeta-link",&link)?));
            put(&stage.join("symbols.map"),rename_text.as_bytes())?;
            let mut object_records=Vec::new();
            let mut rename_keys=BTreeSet::new();
            let llvm=self.sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/bin");
            for (object,object_reused) in &snapshot.objects {
                let member=format!("{}_{:016x}.o",self.context.crate_name,object.definition.hash.0);
                let original_name=format!("native_{:016x}.o",object.definition.hash.0);
                let c_name=format!("native_{:016x}.c",object.definition.hash.0);
                files.insert(original_name.clone(),object.bytes.clone());
                files.insert(c_name.clone(),Arc::new(object.c_source.as_bytes().to_vec()));
                put(&stage.join(&original_name),&object.bytes)?;
                // Only names actually present in this object can change it.
                // Adding an unrelated declaration must not rename every object.
                let relevant_mapping:String=mapping.iter().filter(|(native,_)|self.originals[&object.serial].symbols.contains(*native))
                    .map(|(native,rust)|format!("{native} {rust}\n")).collect();
                let key=(object.serial,relevant_mapping);
                rename_keys.insert(key.clone());
                report["phase"]=json!("native-symbol-map");
                let renamed=if let Some(renamed)=self.renamed.get(&key) {renamed.clone()} else {
                    command(step,&format!("objcopy-{:016x}",object.definition.hash.0),std::process::Command::new(llvm.join("llvm-objcopy"))
                        .current_dir(&stage).args(["--redefine-syms=symbols.map",&original_name,&member]),report)?;
                    let bytes=read(&stage.join(&member))?;
                    let old=unit_object::symbols(&object.bytes)?;let new=unit_object::symbols(&bytes)?;
                    let expected=|names:&BTreeSet<Vec<u8>>|names.iter().map(|name|mapping.get(String::from_utf8_lossy(name).as_ref())
                        .map_or_else(||name.clone(),|name|name.as_bytes().to_vec())).collect::<BTreeSet<_>>();
                    if new.defined!=expected(&old.defined) || new.functions!=expected(&old.functions) || new.undefined!=expected(&old.undefined)
                        || object_contents(&object.bytes,&mapping)?!=object_contents(&bytes,&BTreeMap::new())? {
                        return Err("renamed per-function object changed native bytes, relocations or ownership".into());
                    }
                    let renamed=Renamed{hash:hash(&bytes),bytes:Arc::new(bytes)};
                    self.renamed.insert(key,renamed.clone());renamed
                };
                if !stage.join(&member).exists() {put(&stage.join(&member),&renamed.bytes)?;}
                files.insert(member.clone(),renamed.bytes.clone());
                object_records.push(json!({"name":snapshot.functions.iter().find(|f|f.definition()==object.definition).unwrap().name(),
                    "native_id":id_value(object.definition),"native_symbol":object.symbol,"rust_symbol":mapping[&object.symbol],"member":member,
                    "original_sha256":self.originals[&object.serial].hash,"renamed_sha256":renamed.hash,"reused":object_reused,
                    "native_object_serial":object.serial,"original_path":format!("staging/{original_name}"),"renamed_path":format!("staging/{member}"),
                    "c_path":format!("staging/{c_name}")}));
            }
            for (name,bytes) in &files { if !stage.join(name).exists(){put(&stage.join(name),bytes)?;} }
            let mut members=vec!["lib.rmeta".to_owned(),"lib.rmeta-link".to_owned()];
            members.extend(objects.iter().map(|o|o.member.clone()));
            report["phase"]=json!("original-archive");report["counts"]["archive_builds"]=json!(1);
            let mut cmd=std::process::Command::new(llvm.join("llvm-ar"));
            cmd.current_dir(&stage).args(["crsD","--format=gnu"]).arg(self.context.archive_name()).args(&members);
            command(step,"archive",&mut cmd,report)?;
            let bytes=read(&stage.join(self.context.archive_name()))?;verify_members(&bytes,&files,&members)?;
            let fields=json!({"objects":object_records,"archive_members":members,"rlib_sha256":hash(&bytes),
                "source_sha256":hash(snapshot.source.as_bytes()),"metadata_sha256":hash(&encoded.metadata),"backend_helpers":helpers,
                "embed_metadata":embed_metadata,"metadata_is_stub":encoded.metadata_stub.is_some(),
                "stable_crate_id":format!("{:016x}",encoded.stable_crate_id),
                "crate_hash_le_hex":encoded.crate_hash.iter().map(|b|format!("{b:02x}")).collect::<String>(),
                "symbol_mapping":encoded.symbols.iter().map(|s|json!({"native_id":[s.definition.krate,s.definition.path],"rust_def_index":s.rust_def_index,
                    "rust_def_path_hash":format!("{:016x}",s.rust_def_path_hash),"native":s.native_symbol,"rust":s.rust_symbol,"public":s.public})).collect::<Vec<_>>(),
                "functions":functions.iter().map(|f|json!({"name":f.name,"native_id":[f.definition.krate,f.definition.path],"public":f.public,
                    "name_span":[f.name_span.lo,f.name_span.hi],"span":[f.span.lo,f.span.hi],"parameters":f.parameters.iter().map(|p|json!({"name":p.name,"span":[p.span.lo,p.span.hi]})).collect::<Vec<_>>(),
                    "calls":f.calls.iter().map(|id|json!([id.krate,id.path])).collect::<Vec<_>>()})).collect::<Vec<_>>()});
            let artifact=Arc::new(Artifact{generation:snapshot.generation,files,fields,archive:Arc::new(bytes),
                native_serials:snapshot.objects.iter().map(|(object,_)|object.serial).collect(),rename_keys});
            // Replace variant history only after construction succeeds. The last
            // durable publication remains protected even across generations.
            self.variants.retain(|_,a|a.generation==snapshot.generation);
            self.variants.insert(embed_metadata,artifact.clone());
            self.artifact=Some(artifact);
            self.prune();
        } else {
            self.artifact=cached;
        }
        let artifact=self.artifact.as_ref().unwrap();
        if reused && materialize {
            for (name,bytes) in &artifact.files {put(&stage.join(name),bytes)?;}
            put(&stage.join(self.context.archive_name()),&artifact.archive)?;
        }
        if materialize {
            for (key,value) in artifact.fields.as_object().unwrap(){report[key]=value.clone();}
            // Observation follows actual per-request native object reuse.
            for record in report["objects"].as_array_mut().unwrap() {
                let serial=record["native_object_serial"].as_u64().unwrap();
                record["reused"]=json!(snapshot.objects.iter().find(|(o,_)|o.serial==serial).ok_or("artifact/native object generations differ")?.1);
            }
        }
        report["artifact_reused"]=json!(reused);
        Ok(artifact.clone())
    }
}
