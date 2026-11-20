//! Socket-free host adapter for the actual retained native library handoff.
//! Only original metadata/archive work follows native checking and TinyCC.
use super::*;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use crate::driver::library::{Object as NativeObject, Snapshot};

struct Original {
    hash: String,
    digest: [u8;32],
    helpers: Vec<String>,
    symbols: BTreeSet<String>,
}
#[derive(Clone)]
struct Renamed { bytes: Arc<Vec<u8>>, hash: String }
struct Artifact {
    generation: u64,
    files: BTreeMap<String,Arc<Vec<u8>>>,
    fields: Value,
    archive: Arc<Vec<u8>>,
}
struct Host {
    sdk: metadata::Sdk,
    sysroot: PathBuf,
    source: PathBuf,
    root: PathBuf,
    cwd: String,
    session: driver::Session,
    key: driver::SessionKey,
    originals: BTreeMap<u64,Original>,
    renamed: BTreeMap<(u64,String),Renamed>,
    artifact: Option<Arc<Artifact>>,
    last_published: Option<Arc<Artifact>>,
    last_success: Option<String>,
    index: u64,
    tool_identities: Value,
}
fn id_value(id:def::DiskDefId)->Value { json!([id.krate.0,id.hash.0]) }
fn definition_name(snapshot:&Snapshot,id:def::DiskDefId)->Value {
    let name=snapshot.functions.iter().find(|f|f.definition()==id).map(|f|f.name()).unwrap_or("<absent>");
    json!({"name":name,"native_id":id_value(id)})
}
fn mapped_functions(snapshot:&Snapshot)->Vec<metadata::Function> {
    snapshot.functions.iter().map(|f|metadata::Function {definition:native_id(f.definition()),name:f.name().into(),
        name_span:span(f.name_span()),span:span(f.span()),public:f.visibility()==native_consumer::Visibility::Public,
        parameters:f.parameters().iter().map(|p|metadata::Parameter{name:p.name().into(),span:span(p.span())}).collect(),
        calls:f.calls().iter().copied().map(native_id).collect() }).collect()
}
fn check_object(object:&NativeObject, snapshot:&Snapshot)->Result<Original,String> {
    let symbols=unit_object::symbols(&object.bytes)?;
    let function=snapshot.functions.iter().find(|f|f.definition()==object.definition).ok_or("object has no completed definition")?;
    let trampoline=object.symbol.replacen("f_","tramp_",1);
    let definitions=BTreeSet::from([object.symbol.as_bytes().to_vec(),trampoline.as_bytes().to_vec()]);
    let expected_calls:BTreeSet<Vec<u8>>=function.calls().iter().map(|id| {
        snapshot.objects.iter().find(|(o,_)|o.definition==*id).map(|(o,_)|o.symbol.as_bytes().to_vec()).ok_or("completed call has no actual object")
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
impl Host {
    fn new(source:PathBuf,root:PathBuf,receipt:PathBuf,sysroot:PathBuf)->Result<Self,String> {
        if std::env::var_os("RUSTC_FORCE_RUSTC_VERSION").is_some_and(|value|value!=CFG_VERSION) {
            return Err("RUSTC_FORCE_RUSTC_VERSION differs from pinned metadata context".into());
        }
        let sdk=lustr_metadata_sdk::open_bound(&receipt,&sysroot,SDK_RECEIPT)?;
        let mut key=driver::SessionKey::synthetic(&source,false);
        key.krate.target_kind="lib".into();key.krate.target_name="consumer".into();
        key.krate.package_id.push_str("#retained-native-metadata-v1");
        key.target_triple="x86_64-unknown-linux-gnu".into();
        key.profile=protocol::ProfileFlags{panic_abort:true,overflow_checks:true,debug_assertions:true};
        let session=driver::Session::scalar_library(root.join("native-objects"),key.clone())?;
        let cwd=std::env::current_dir().map_err(|e|e.to_string())?.to_str().ok_or("working directory is not Unicode")?.to_owned();
        let llvm=sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/bin");
        let objcopy=llvm.join("llvm-objcopy");let ar=llvm.join("llvm-ar");
        let tool_identities=json!([{"path":objcopy,"sha256":hash(&read(&objcopy)?)},{"path":ar,"sha256":hash(&read(&ar)?)}]);
        Ok(Self{sdk,sysroot,source,root,cwd,session,key,originals:BTreeMap::new(),renamed:BTreeMap::new(),artifact:None,last_published:None,last_success:None,index:0,tool_identities})
    }
    fn artifact(&mut self,snapshot:&Snapshot,step:&Path,report:&mut Value)->Result<(),String> {
        let stage=step.join("staging");
        report["phase"]=json!("staging");
        std::fs::create_dir(&stage).map_err(|e|format!("create owned staging: {e}"))?;
        let reused=self.artifact.as_ref().is_some_and(|a|a.generation==snapshot.generation);
        if !reused {
            let functions=mapped_functions(snapshot);
            let instances:Vec<_>=snapshot.objects.iter().map(|(o,_)|metadata::EmittedFunction{definition:native_id(o.definition),symbol:o.symbol.clone()}).collect();
            for (object,_) in &snapshot.objects {
                if !self.originals.contains_key(&object.serial) {
                    self.originals.insert(object.serial,check_object(object,snapshot)?);
                }
            }
            let mut objects:Vec<_>=snapshot.objects.iter().map(|(o,_)|metadata::ObjectMember{
                member:format!("consumer_{:016x}.o",o.definition.hash.0),producer_object_sha256:self.originals[&o.serial].digest,
                definitions:vec![native_id(o.definition)]}).collect();
            objects.sort_by(|a,b|a.member.cmp(&b.member));
            let helpers:Vec<_>=snapshot.objects.iter().flat_map(|(o,_)|self.originals[&o.serial].helpers.clone()).collect();
            report["phase"]=json!("source-owned-encode");report["counts"]["metadata_encodes"]=json!(1);
            let encoded=metadata::encode(&metadata::Input {crate_name:"consumer",metadata:"native_consumer_v1",
                source_path:self.source.to_str().ok_or("source path is not Unicode")?,working_directory:&self.cwd,
                source:&snapshot.source,root_span:metadata::Span{lo:0,hi:snapshot.source.len() as u32},
                profile:metadata::Profile{overflow_checks:true,debug_assertions:true},functions:&functions,emitted:&instances,backend_helpers:&helpers,objects:&objects},&self.sdk)?;
            let mapping:BTreeMap<_,_>=encoded.symbols.iter().map(|s|(s.native_symbol.clone(),s.rust_symbol.clone())).collect();
            let rename_text:String=mapping.iter().map(|(native,rust)|format!("{native} {rust}\n")).collect();
            let mut files:BTreeMap<String,Arc<Vec<u8>>>=BTreeMap::new();
            files.insert("symbols.map".into(),Arc::new(rename_text.as_bytes().to_vec()));
            files.insert("consumer.rmeta".into(),Arc::new(encoded.metadata.clone()));
            files.insert("lib.rmeta".into(),Arc::new(archive::wrapper(b".rmeta",&encoded.metadata)?));
            let member_names:Vec<_>=objects.iter().map(|o|o.member.as_str()).collect();
            let link=metadata::encode_rlib_link(&member_names)?;
            files.insert("lib.rmeta-link".into(),Arc::new(archive::wrapper(b".rmeta-link",&link)?));
            put(&stage.join("symbols.map"),rename_text.as_bytes())?;
            let mut object_records=Vec::new();
            let llvm=self.sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/bin");
            for (object,object_reused) in &snapshot.objects {
                let member=format!("consumer_{:016x}.o",object.definition.hash.0);
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
            cmd.current_dir(&stage).args(["crsD","--format=gnu","libconsumer.rlib"]).args(&members);
            command(step,"archive",&mut cmd,report)?;
            let bytes=read(&stage.join("libconsumer.rlib"))?;verify_members(&bytes,&files,&members)?;
            let fields=json!({"objects":object_records,"archive_members":members,"rlib_sha256":hash(&bytes),
                "source_sha256":hash(snapshot.source.as_bytes()),"metadata_sha256":hash(&encoded.metadata),"backend_helpers":helpers,
                "stable_crate_id":format!("{:016x}",encoded.stable_crate_id),
                "crate_hash_le_hex":encoded.crate_hash.iter().map(|b|format!("{b:02x}")).collect::<String>(),
                "symbol_mapping":encoded.symbols.iter().map(|s|json!({"native_id":[s.definition.krate,s.definition.path],"rust_def_index":s.rust_def_index,
                    "rust_def_path_hash":format!("{:016x}",s.rust_def_path_hash),"native":s.native_symbol,"rust":s.rust_symbol,"public":s.public})).collect::<Vec<_>>(),
                "functions":functions.iter().map(|f|json!({"name":f.name,"native_id":[f.definition.krate,f.definition.path],"public":f.public,
                    "name_span":[f.name_span.lo,f.name_span.hi],"span":[f.span.lo,f.span.hi],"parameters":f.parameters.iter().map(|p|json!({"name":p.name,"span":[p.span.lo,p.span.hi]})).collect::<Vec<_>>(),
                    "calls":f.calls.iter().map(|id|json!([id.krate,id.path])).collect::<Vec<_>>()})).collect::<Vec<_>>()});
            self.artifact=Some(Arc::new(Artifact{generation:snapshot.generation,files,fields,archive:Arc::new(bytes)}));
        }
        let artifact=self.artifact.as_ref().unwrap();
        if reused {
            for (name,bytes) in &artifact.files {put(&stage.join(name),bytes)?;}
            put(&stage.join("libconsumer.rlib"),&artifact.archive)?;
        }
        for (key,value) in artifact.fields.as_object().unwrap(){report[key]=value.clone();}
        // Object reuse is per native request, independent of metadata/archive reuse.
        for record in report["objects"].as_array_mut().unwrap() {
            let serial=record["native_object_serial"].as_u64().unwrap();
            record["reused"]=json!(snapshot.objects.iter().find(|(o,_)|o.serial==serial).ok_or("artifact/native object generations differ")?.1);
        }
        report["artifact_reused"]=json!(reused);report["phase"]=json!("publication");
        match unit_publication::publish(&stage.join("libconsumer.rlib"),&step.join("libconsumer.rlib"))
            .map_err(|error| format!("publish retained native archive: {error}"))? {
            unit_publication::Publication::Durable=>{self.last_published=self.artifact.clone();},
            unit_publication::Publication::DurabilityUnknown(error)=>return Err(format!("published archive durability unconfirmed: {error}")),
        }
        Ok(())
    }
    fn request(&mut self,id:&str,action:&str,output:&str)->Result<Value,String> {
        if id.is_empty() || !["build","check","prepare-drop"].contains(&action) || output.is_empty() || output=="." || output==".."
            || !output.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b)) {return Err("invalid retained request".into());}
        self.index+=1;let step=self.root.join(output);
        if !step.is_dir(){return Err("retained output must be an existing owned child directory".into());}
        let mut report=json!({"schema":"retained-native-metadata-report-v1","id":id,"session_id":std::process::id(),"request_index":self.index,
            "status":3,"phase":"native-prepare","source_path":self.source,"sdk_receipt_sha256":SDK_RECEIPT,
            "native_context":self.key.describe(),"artifact_tools":self.tool_identities,"commands":[],"objects":[],"checked_definitions":[],"generated_definitions":[],
            "counts":{"checked_bodies":0,"generated_bodies":0,"compiled_objects":0,"reused_objects":0,"metadata_encodes":0,"archive_builds":0,"sdk_loads":1},
            "artifact_reused":false,"last_successful_request":self.last_success});
        let request=driver::Request{cmd:if action=="check"{driver::Cmd::Check}else{driver::Cmd::Build},session:self.key.clone(),file:self.source.clone(),
            out:step.join("libconsumer.rlib"),json:false,timings:false,debug:false,emit_c:None,max_errors:20,run_args:None,harness:false,cc:false,client:Default::default()};
        let result=match driver::prepare(&request,&mut self.session) {
            driver::Preparation::Ready(prepared)=>if action=="prepare-drop"{drop(prepared);driver::Response::default()}else{prepared.emit()},
            driver::Preparation::Complete(r)|driver::Preparation::Rejected(r)|driver::Preparation::Failed(r)|driver::Preparation::IntegrationIncomplete(r)=>r,
        };
        put(&step.join("native.stdout"),result.stdout.as_bytes())?;put(&step.join("native.stderr"),result.stderr.as_bytes())?;
        let state=self.session.library_state().ok_or("native library state disappeared")?;
        report["source_sha256"]=if result.code==0 {
            self.artifact.as_ref().filter(|a|a.generation==state.generation).map_or_else(||json!(hash(state.source.as_bytes())),|a|a.fields["source_sha256"].clone())
        } else {json!(hash(state.source.as_bytes()))};
        report["native_diagnostics"]=json!(result.stderr);
        report["counts"]["checked_bodies"]=json!(state.checked.len());
        report["counts"]["generated_bodies"]=json!(state.generated.len());
        report["counts"]["compiled_objects"]=json!(state.compiled_objects);
        report["counts"]["reused_objects"]=json!(state.reused_objects);
        report["native_next_object_serial"]=json!(state.next_object);
        report["checked_definitions"]=json!(state.checked.iter().map(|id|json!({"name":state.function_name(*id),"native_id":id_value(*id)})).collect::<Vec<_>>());
        report["generated_definitions"]=json!(state.generated.iter().map(|id|json!({"name":state.function_name(*id),"native_id":id_value(*id)})).collect::<Vec<_>>());
        let mut status=result.code;
        if status==0 && action=="build" {
            let snapshot=state.snapshot.clone().ok_or("native library returned no emission snapshot")?;
            report["checked_definitions"]=json!(snapshot.checked.iter().map(|&id|definition_name(&snapshot,id)).collect::<Vec<_>>());
            report["generated_definitions"]=json!(snapshot.generated.iter().map(|&id|definition_name(&snapshot,id)).collect::<Vec<_>>());
            report["counts"]["generated_bodies"]=json!(snapshot.generated.len());
            report["counts"]["compiled_objects"]=json!(snapshot.objects.iter().filter(|(_,reused)|!reused).count());
            report["counts"]["reused_objects"]=json!(snapshot.objects.iter().filter(|(_,reused)|*reused).count());
            if let Err(error)=self.artifact(&snapshot,&step,&mut report){status=3;report["error"]=json!(error);}
            else{self.last_success=Some(id.to_owned());report["phase"]=json!("complete");}
        } else if status==0 {report["phase"]=json!(if action=="prepare-drop"{"prepared-dropped"}else{"checked"});}
        else{report["error"]=json!(result.stderr);
            if status==4{report["phase"]=json!("native-completion");}
            else if status==3 && (!state.generated.is_empty() || state.compiled_objects!=0){report["phase"]=json!("native-emission");}
        }
        report["status"]=json!(status);report["last_successful_request"]=json!(self.last_success);
        put(&step.join("report.json"),&serde_json::to_vec_pretty(&report).map_err(|e|e.to_string())?)?;
        Ok(json!({"id":id,"status":status,"phase":report["phase"],"report":format!("{output}/report.json")}))
    }
}
pub(super) fn run(args:&[std::ffi::OsString])->Result<(),String> {
    if args.len()!=4{return Err("--retained SOURCE EXISTING_ROOT EXACT_SDK_RECEIPT PINNED_SDK".into());}
    crate::tcc::use_mimalloc();
    let mut host=Host::new(PathBuf::from(&args[0]),PathBuf::from(&args[1]),PathBuf::from(&args[2]),PathBuf::from(&args[3]))?;
    println!("{}",json!({"event":"ready","schema":"retained-native-metadata-v1","session_id":std::process::id(),"sdk_receipt_sha256":SDK_RECEIPT}));
    std::io::stdout().flush().map_err(|e|e.to_string())?;
    for line in BufReader::new(std::io::stdin().lock()).lines() {
        let line=line.map_err(|e|e.to_string())?;
        let request:Value=serde_json::from_str(&line).map_err(|e|e.to_string())?;
        let response=host.request(request["id"].as_str().ok_or("missing request id")?,request["action"].as_str().ok_or("missing action")?,request["output"].as_str().ok_or("missing output")?)?;
        println!("{response}");std::io::stdout().flush().map_err(|e|e.to_string())?;
    }
    Ok(())
}
