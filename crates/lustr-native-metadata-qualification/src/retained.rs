//! Socket-free host adapter for the actual retained native library handoff.
//! Only original metadata/archive work follows native checking and TinyCC.
use super::*;
use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use crate::driver::library::{Object as NativeObject, Snapshot};

use crate::library_artifact::{Artifact,Service,Context};
struct Host {
    sysroot: PathBuf,
    source: PathBuf,
    root: PathBuf,
    cwd: String,
    session: driver::Session,
    key: driver::SessionKey,
    artifacts: Service,
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
impl Host {
    fn new(source:PathBuf,root:PathBuf,receipt:PathBuf,sysroot:PathBuf)->Result<Self,String> {
        if std::env::var_os("RUSTC_FORCE_RUSTC_VERSION").is_some_and(|value|value!=CFG_VERSION) {
            return Err("RUSTC_FORCE_RUSTC_VERSION differs from pinned metadata context".into());
        }
        let sdk=Arc::new(lustr_metadata_sdk::open_bound(&receipt,&sysroot,SDK_RECEIPT)?);
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
        let artifacts=Service::new(sdk,sysroot.clone(),Context{crate_name:"consumer".into(),metadata:"native_consumer_v1".into(),extra_filename:String::new(),
            source:source.clone(),cwd:cwd.clone(),profile:metadata::Profile{overflow_checks:true,debug_assertions:true}})?;
        Ok(Self{sysroot,source,root,cwd,session,key,artifacts,artifact:None,last_published:None,last_success:None,index:0,tool_identities})
    }
    fn artifact(&mut self,snapshot:&Snapshot,step:&Path,report:&mut Value)->Result<(),String> {
        self.artifact=Some(self.artifacts.artifact(snapshot,step,report,true)?);
        report["phase"]=json!("publication");
        match unit_publication::publish(&step.join("staging/libconsumer.rlib"),&step.join("libconsumer.rlib"))
            .map_err(|error|format!("publish retained native archive: {error}"))? {
            unit_publication::Publication::Durable=>{
                self.last_published=self.artifact.clone();
                self.artifacts.published(self.artifact.as_ref().unwrap().clone());self.session.library_published();
            },
            unit_publication::Publication::DurabilityUnknown(error)=>return Err(format!("published archive durability unconfirmed: {error}")),
        }
        Ok(())
    }
    fn request(&mut self,id:&str,action:&str,output:&str)->Result<Value,String> {
        if id.is_empty() || !["build","check","prepare-drop"].contains(&action) || output.is_empty() || output=="." || output==".."
            || !output.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b)) {return Err("invalid retained request".into());}
        self.index+=1;let step=self.root.join(output);
        self.artifacts.begin_request();
        if !step.is_dir(){return Err("retained output must be an existing owned child directory".into());}
        let mut report=json!({"schema":"retained-native-metadata-report-v1","id":id,"session_id":std::process::id(),"request_index":self.index,
            "status":3,"phase":"native-prepare","source_path":self.source,"sdk_receipt_sha256":SDK_RECEIPT,
            "native_context":self.key.describe(),"artifact_tools":self.tool_identities,"commands":[],"objects":[],"checked_definitions":[],"generated_definitions":[],
            "counts":{"checked_bodies":0,"generated_bodies":0,"compiled_objects":0,"reused_objects":0,"metadata_encodes":0,"archive_builds":0,"sdk_loads":1},
            "artifact_reused":false,"last_successful_request":self.last_success});
        let request=driver::Request{cmd:if action=="check"{driver::Cmd::Check}else{driver::Cmd::Build},session:self.key.clone(),file:self.source.clone(),
            out:step.join("libconsumer.rlib"),json:false,timings:false,debug:false,emit_c:None,max_errors:20,run_args:None,harness:false,cc:false,library_options:Default::default(),client:Default::default()};
        let result=match driver::prepare(&request,&mut self.session) {
            driver::Preparation::Ready(prepared)=>if action=="prepare-drop"{drop(prepared);driver::Response::default()}else{prepared.emit()},
            driver::Preparation::Complete(r)|driver::Preparation::Rejected(r)|driver::Preparation::Failed(r)|driver::Preparation::IntegrationIncomplete(r)=>r,
        };
        self.session.prune_library_history();
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
