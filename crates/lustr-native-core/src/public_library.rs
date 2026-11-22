//! Public retained library owner. Transport is outside this module. One native
//! preparation supplies both emitted code and completed metadata facts.
use crate::{diag,driver,protocol};
use crate::library_artifact::{Context,Service};
use lustr_metadata_encoder as metadata;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{fs,io::Write,path::{Path,PathBuf},sync::{Arc,OnceLock,atomic::{AtomicU64,Ordering}}};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
#[path="unit_publication.rs"]
mod publication;
#[path="public_library_import.rs"]
mod imports;
#[path="public_executable_import.rs"]
mod executable;
static REQUESTS:AtomicU64=AtomicU64::new(0);
static SDK_LOADS:AtomicU64=AtomicU64::new(0);
static OWNER_START:OnceLock<Option<String>>=OnceLock::new();
static INSTALLED:OnceLock<Result<Arc<Installed>,String>>=OnceLock::new();

fn hash(bytes:&[u8])->String {Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()}
fn write_new(path:&Path,bytes:&[u8])->Result<(),String>{
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("create {}: {e}",path.display()))?;
    file.write_all(bytes).map_err(|e|format!("write {}: {e}",path.display()))
}
#[derive(PartialEq,Eq)]
struct Stamp(u64,u64,u64,i64,i64,i64,i64);
fn stamp(path:&Path)->Result<Stamp,String>{
    let m=fs::symlink_metadata(path).map_err(|e|format!("metadata SDK input {}: {e}",path.display()))?;
    if !m.file_type().is_file(){return Err("metadata SDK input is not a regular file".into());}
    Ok(Stamp(m.dev(),m.ino(),m.len(),m.mtime(),m.mtime_nsec(),m.ctime(),m.ctime_nsec()))
}
struct Installed {
    root:PathBuf, sdk:Arc<metadata::Sdk>, inputs:Vec<(PathBuf,Stamp)>, tools:Value,
}
impl Installed {
    fn load()->Result<Arc<Self>,String>{
        let root=std::env::current_exe().map_err(|e|e.to_string())?.parent().and_then(Path::parent)
            .ok_or("installed compiler has no package root")?.canonicalize().map_err(|e|e.to_string())?;
        let receipt=root.join(protocol::library_contract::SDK_RECEIPT_PATH);
        let receipt_bytes=fs::read(&receipt).map_err(|e|e.to_string())?;
        if hash(&receipt_bytes)!=protocol::library_contract::SDK_RECEIPT {return Err("installed SDK receipt changed".into());}
        let facts:Value=serde_json::from_slice(&receipt_bytes).map_err(|e|e.to_string())?;
        let mut paths=vec![receipt];
        for dep in facts["dependencies"].as_array().ok_or("SDK dependencies missing")? {
            for field in ["artifact","link_artifact"] {
                paths.push(root.join(dep[field]["path"].as_str().ok_or("SDK artifact path missing")?));
            }
        }
        let tool_path=root.join("lib/lustr-metadata/artifact-tools.json");
        let bytes=fs::read(&tool_path).map_err(|e|e.to_string())?;
        if bytes!=include_bytes!("../../../upstream/metadata-artifact-tools.json") {
            return Err("installed artifact tool declaration differs from compiler construction".into());
        }
        paths.push(tool_path);
        let manifest:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
        for field in ["tools","allowed_sdk_libraries"] {
            for record in manifest[field].as_array().ok_or("artifact tool closure absent")? {
                paths.push(root.join(record["path"].as_str().ok_or("artifact tool path absent")?));
            }
        }
        let inputs=paths.into_iter().map(|p|Ok((p.clone(),stamp(&p)?))).collect::<Result<Vec<_>,String>>()?;
        let sdk=Arc::new(lustr_metadata_sdk::open_bound(&inputs[0].0,&root,protocol::library_contract::SDK_RECEIPT)?);
        let mut tools=Vec::new();
        for field in ["tools","allowed_sdk_libraries"] {
            for record in manifest[field].as_array().ok_or("artifact tool closure absent")? {
                let path=root.join(record["path"].as_str().ok_or("artifact tool path absent")?);
                let data=fs::read(&path).map_err(|e|format!("artifact tool closure {}: {e}",path.display()))?;
                if Some(data.len() as u64)!=record["size"].as_u64() || Some(hash(&data).as_str())!=record["sha256"].as_str() {
                    return Err("artifact tool closure differs from original exact bytes".into());
                }
                if field=="tools" {tools.push(json!({"path":path,"sha256":record["sha256"],"size":record["size"]}));}
            }
        }
        for (path,before) in &inputs {
            if stamp(path)?!=*before {return Err("metadata SDK changed during owner initialization".into());}
        }
        SDK_LOADS.fetch_add(1,Ordering::Relaxed);
        Ok(Arc::new(Self{root,sdk,inputs,tools:json!(tools)}))
    }
    fn unchanged(&self)->Result<(),String>{
        for (path,loaded) in &self.inputs {
            if stamp(path)?!=*loaded {return Err(format!("loaded metadata SDK generation changed: {}",path.display()));}
        }
        Ok(())
    }
}
pub(crate) struct Owner {
    installed:Arc<Installed>, artifacts:Service, identity:String,
    imported: Option<imports::Import>,
    executable: Option<executable::ExecutableImports>,
    binary_output: Option<BinaryArtifact>,
}

struct BinaryArtifact {
    source: Arc<str>,
    shown: Option<PathBuf>,
    mode: u32,
    bytes: Arc<[u8]>,
}
impl Owner {
    pub(crate) fn new(req:&driver::Request)->Result<Self,String>{
        let key=req.session.library.as_ref().ok_or("library identity absent")?;
        if !driver::library::request_supported(req){return Err("public library request context is not represented".into());}
        let installed=INSTALLED.get_or_init(Installed::load).clone()?;
        installed.unchanged()?;
        let imported = if let Some(external) = &key.external {
            if req.session.krate.manifest_path.is_none() { return Err("extern metadata requires the actual Cargo unit".into()); }
            Some(imports::Import::open(external, &installed.sdk)?)
        } else { None };
        let context=Context{crate_name:key.crate_name.clone(),metadata:key.metadata.clone(),extra_filename:key.extra_filename.clone(),
            source:key.source_path.clone(),cwd:key.working_directory.to_str().ok_or("library cwd is not Unicode")?.into(),
            profile:metadata::Profile{overflow_checks:req.session.profile.overflow_checks,debug_assertions:req.session.profile.debug_assertions}};
        let artifacts=Service::new(installed.sdk.clone(),installed.root.clone(),context)?;
        let executable = if let Some(dependencies) = &key.executable_dependencies {
            if req.session.krate.manifest_path.is_none() { return Err("executable artifacts require the actual Cargo unit".into()); }
            Some(executable::ExecutableImports::open(dependencies, &installed.sdk)?)
        } else { None };
        Ok(Self{installed,artifacts,identity:hash(req.session.describe().as_bytes()),imported,executable,binary_output:None})
    }

    pub(crate) fn import_is_current(&self, req: &driver::Request) -> bool {
        if let Some(executable) = &self.executable {
            return req.session.library.as_ref().and_then(|key| key.executable_dependencies.as_ref())
                .is_some_and(|inputs| executable.unchanged(inputs));
        }
        match (&self.imported, req.session.library.as_ref().and_then(|key| key.external.as_ref())) {
            (Some(imported), Some(external)) => imported.unchanged(&external.artifact),
            (None, None) => true,
            _ => false,
        }
    }
    pub(crate) fn compile(&mut self,req:&driver::Request,session:&mut driver::Session)->driver::Response {
        let serial=REQUESTS.fetch_add(1,Ordering::Relaxed)+1;
        if self.executable.is_some() {
            return self.compile_executable(req, session, serial);
        }
        self.artifacts.begin_request();
        session.configure_library_history(self.artifacts.policy());
        let report_root=req.client.env.as_ref().and_then(|env|env.iter().find(|(name,_)|name=="LUSTR_LIBRARY_REPORT_DIR").map(|(_,value)|PathBuf::from(value)));
        let key=req.session.library.as_ref().expect("library entry is typed");
        let observed=report_root.is_some();
        let step=report_root.as_ref().map_or_else(||session.obj_dir().join(format!("artifact-request-{serial}")),
            |root|root.join(format!("request-{}-{serial}",std::process::id())));
        let owner_start=observed.then(||OWNER_START.get_or_init(||fs::read_to_string("/proc/self/stat").ok().and_then(|s|s.rsplit_once(") ")
            .and_then(|(_,tail)|tail.split_whitespace().nth(19).map(str::to_owned)))).clone()).flatten();
        let profile=req.session.profile;
        let check = req.cmd == driver::Cmd::Check;
        let emits: &[&str] = if check { &["dep-info", "metadata"] } else { &["link", "metadata", "dep-info"] };
        let mut report=if observed {json!({"schema":"lustr-public-scalar-library-v1","owner_pid":std::process::id(),"owner_start":owner_start,
            "request_index":serial,"session_identity":self.identity,"native_context":req.session.describe(),
            "library_options":{"embed_metadata":req.library_options.embed_metadata,
                "force_warn_unused_crate_dependencies":req.library_options.force_warn_unused_crate_dependencies},
            "context":{"crate_name":key.crate_name,"metadata":key.metadata,"extra_filename":key.extra_filename,
                "source_path":key.source_path,"cwd":key.working_directory,"target":req.session.target_triple,"edition":"2024",
                "emits":if check { &["dep-info", "metadata"][..] } else { &["dep-info", "metadata", "link"][..] },"sdk_receipt_sha256":key.sdk_receipt,
                "profile":{"panic_abort":profile.panic_abort,"overflow_checks":profile.overflow_checks,"debug_assertions":profile.debug_assertions},
                "check_cfg":key.check_cfg,"search_dirs":key.search_paths},
            "source_path":req.file,"sdk_receipt_sha256":key.sdk_receipt,"status":3,"phase":"native-prepare",
            "artifact_tools":self.installed.tools,"commands":[],"objects":[],"outputs":[],"checked_definitions":[],"generated_definitions":[],
            "artifact_reused":false,"counts":{"checked_bodies":0,"generated_bodies":0,"compiled_objects":0,"reused_objects":0,
                "metadata_encodes":0,"archive_builds":0,"sdk_loads":SDK_LOADS.load(Ordering::Relaxed)},"timings_ns":{}})}
            else {json!({"phase":"native-prepare","commands":[],"objects":[],"counts":{"metadata_encodes":0,"archive_builds":0}})};
        let mut response=driver::Response::default();
        let mut dependency_lint_ready=false;
        let mut import_used = false;
        let outcome=(||->Result<(),String>{
            if let Some(root)=&report_root {
                if !root.is_absolute() || !root.is_dir(){return Err("library report directory must be an existing absolute directory".into());}
            } else {fs::create_dir_all(session.obj_dir()).map_err(|e|e.to_string())?;}
            fs::create_dir(&step).map_err(|e|format!("create library request scope: {e}"))?;
            // Rust's stable crate identity consumes this version. The actual
            // client environment is revalidated even for a retained no-op.
            if !version_supported(req) {
                response.code=4;
                response.stderr="error: RUSTC_FORCE_RUSTC_VERSION differs from pinned metadata context\n".into();
                report["phase"]=json!("request-context");return Ok(());
            }
            self.installed.unchanged()?;
            // Exactly one continuation; metadata consumes its completed snapshot.
            let native_start=observed.then(std::time::Instant::now);
            response = {
                let prepared = if let Some(imported) = &self.imported {
                    driver::prepare_library_import(req, session, imported)
                } else { driver::prepare(req, session) };
                match prepared {
                    driver::Preparation::Ready(prepared)=>prepared.emit(),
                    driver::Preparation::Complete(r)|driver::Preparation::Rejected(r)|driver::Preparation::Failed(r)|driver::Preparation::IntegrationIncomplete(r)=>r,
                }
            };
            if let Some(start)=native_start{report["timings_ns"]["native"]=json!(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);}
            session.prune_library_history();
            let state=session.library_state().ok_or("library native state disappeared")?;
            dependency_lint_ready=state.dependency_lint_ready;
            import_used = state.extern_used_on_error;
            if !self.import_is_current(req) { return Err("extern metadata changed during library checking".into()); }
            if observed {
            report["generation"]=json!(state.generation);
            report["source_sha256"]=json!(hash(state.source.as_bytes()));
            report["counts"]["checked_bodies"]=json!(state.checked.len());
            report["counts"]["generated_bodies"]=json!(state.generated.len());
            report["counts"]["compiled_objects"]=json!(state.compiled_objects);
            report["counts"]["reused_objects"]=json!(state.reused_objects);
            let names=|ids:&Vec<crate::def::DiskDefId>|ids.iter().map(|id|json!({"name":state.function_name(*id),"native_id":[id.krate.0,id.hash.0]})).collect::<Vec<_>>();
            report["checked_definitions"]=json!(names(&state.checked));
            report["generated_definitions"]=json!(names(&state.generated));
            }
            if response.code!=0 {
                report["phase"]=json!(if response.code==4{"native-completion"}else if !state.generated.is_empty(){"native-emission"}else{"native-prepare"});
                return Ok(());
            }
            let artifact_start=observed.then(std::time::Instant::now);
            let functions = state.completed_functions().ok_or("native library returned no completed functions")?;
            let used = self.imported.as_ref().filter(|imported| functions.iter().flat_map(|function| function.calls())
                .any(|id| imported.owns(*id)));
            import_used = used.is_some();
            let metadata_context = used.map(|imported| &imported.sdk);
            let (linked, checked_metadata) = if check {
                (None, Some(self.artifacts.check_metadata(
                    state.generation, &state.source, functions, metadata_context,
                    &mut report,
                )?))
            } else {
                let snapshot = state.snapshot.clone().ok_or("native library returned no actual emission snapshot")?;
                (Some(self.artifacts.artifact_with_options(
                    &snapshot, &step, &mut report, observed, req.library_options.embed_metadata, metadata_context,
                )?), None)
            };
            let check_paths;
            let build_paths;
            let paths: &[PathBuf] = if check {
                check_paths = key.check_outputs(&req.out).ok_or("requested metadata output plan changed")?;
                &check_paths
            } else {
                build_paths = key.outputs(&req.out).ok_or("requested library output plan changed")?;
                &build_paths
            };
            if let Some(start)=artifact_start{report["timings_ns"]["artifact"]=json!(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);}
            report["phase"]=json!("publication");
            let publication_start=observed.then(std::time::Instant::now);
            // The admitted completion proves exactly one original source file,
            // no imported source and no env!/include! reads.
            let input=req.client.shown.as_deref().unwrap_or(&req.file);
            let dep=dep_info(paths,input)?;
            let check_bytes;
            let build_bytes;
            let expected: &[&[u8]] = if let Some(metadata) = &checked_metadata {
                check_bytes = [dep.as_slice(), metadata.as_slice()];
                &check_bytes
            } else {
                let artifact = linked.as_ref().expect("successful library build has linked artifacts");
                let raw = artifact.files.get(&format!("{}.rmeta", key.crate_name)).ok_or("raw metadata is absent")?;
                build_bytes = [artifact.archive.as_slice(), raw.as_slice(), dep.as_slice()];
                &build_bytes
            };
            let matches=paths.iter().zip(expected).all(|(path,&bytes)|fs::read(path).is_ok_and(|old|old==bytes));
            report["output_reused"]=json!(matches);
            report["publication_count"]=json!(0);report["publication_state"]=json!("unchanged");
            if !matches {
                let parent=req.out.parent().ok_or("output parent absent")?;
                let staging=parent.join(format!(".lustr-library-{}-{serial}",std::process::id()));
                fs::create_dir(&staging).map_err(|e|format!("stage output set: {e}"))?;
                let publication=(||->Result<(),String>{
                    let staged=paths.iter().map(|p|staging.join(p.file_name().unwrap())).collect::<Vec<_>>();
                    for (path,bytes) in staged.iter().zip(expected){write_new(path,bytes)?;}
                    // The artifact Cargo consumes is installed last.
                    let order: &[usize] = if check { &[0, 1] } else { &[1, 2, 0] };
                    let pairs: Vec<_> = order.iter().map(|&i| (staged[i].as_path(), paths[i].as_path())).collect();
                    match publication::replace_set(&pairs).map_err(|e|format!("replace library output set: {e}"))? {
                        publication::Replacement::Durable=>{
                            report["publication_count"]=json!(paths.len());report["publication_state"]=json!("durable");
                            if let Some(artifact) = &linked {
                                self.artifacts.published(artifact.clone());session.library_published();
                            }
                            // Retain the committed receipt before any fallible
                            // staging cleanup or observation can follow it.
                            if observed {report["committed_outputs"]=output_records(paths,expected,emits);}
                            Ok(())
                        },
                        publication::Replacement::Incomplete{replaced,error}=>{report["publication_state"]=json!("incomplete");report["publication_count"]=json!(replaced);Err(format!("library output replacement incomplete after {replaced} files: {error}"))},
                        publication::Replacement::DurabilityUnknown(error)=>{report["publication_state"]=json!("durability-unknown");report["publication_count"]=json!(paths.len());Err(format!("library output durability unconfirmed: {error}"))},
                    }
                })();
                let cleanup=fs::remove_dir_all(&staging).map_err(|e|format!("clean owned output staging: {e}"));
                publication?;cleanup?;
            }
            if matches && let Some(artifact) = &linked {
                self.artifacts.published(artifact.clone());session.library_published();
            }
            if observed {report["outputs"]=output_records(paths,expected,emits);}
            if let Some(start)=publication_start{report["timings_ns"]["publication"]=json!(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);}
            if req.json {
                for (path,emit) in paths.iter().zip(emits) {
                    response.stderr.push_str(&json!({"$message_type":"artifact","artifact":path,"emit":emit}).to_string());
                    response.stderr.push('\n');
                }
            }
            report["phase"]=json!("complete");
            Ok(())
        })();
        if let Err(error)=outcome {
            response.code=3;
            response.stderr.push_str(&format!("error: {error}\n"));
            report["error"]=json!(error);
        }
        normalize_failure(req,&mut response);
        if dependency_lint_ready && req.json && req.library_options.force_warn_unused_crate_dependencies
            && (self.imported.is_none() || response.code == 0 || import_used) {
            // Completed scalar bodies expose every reference to an admitted
            // dependency as an actual checked call edge.
            let unused = self.imported.as_ref().filter(|_| !import_used)
                .map(|imported| imported.alias.as_str()).into_iter().collect::<Vec<_>>();
            let lint=json!({"$message_type":"unused_extern","lint_level":"force-warn","unused_extern_names":unused});
            response.stderr.insert_str(0,&format!("{lint}\n"));
        }
        if observed {
            let policy=self.artifacts.policy();
            let native=session.library_state().map(|state|{
                let h=&state.history;
                json!({"active_objects":h.active_objects,"inactive_objects":h.inactive_objects,"inactive_object_bytes":h.inactive_object_bytes,
                    "active_definitions":h.active_definitions,"inactive_definitions":h.inactive_definitions,"inactive_definition_bytes":h.inactive_definition_bytes,
                    "evicted_object_serials":h.evicted_object_serials,"retired_definition_ids":h.retired_definition_ids.iter().map(|id|[id.krate.0,id.hash.0]).collect::<Vec<_>>()})
            });
            report["cache_history"]=json!({"policy":{"inactive_entries":policy.inactive_entries,"inactive_bytes":policy.inactive_bytes,
                "owner_start_bound":true,"scope":"each native-object, native-definition, and artifact cache"},"native":native,"artifacts":self.artifacts.history()});
        }
        report["status"]=json!(response.code);
        report["outcome"]=json!(if response.code==0{"emitted"}else if response.code==1{"rejected"}else if response.code==4{"integration-incomplete"}else{"failed"});
        report["native_diagnostics"]=json!(response.stderr);
        if observed && step.is_dir() {
            let evidence=(||->Result<(),String>{
                write_new(&step.join("native.stdout"),response.stdout.as_bytes())?;
                write_new(&step.join("native.stderr"),response.stderr.as_bytes())?;
                write_new(&step.join("report.json"),&serde_json::to_vec_pretty(&report).map_err(|e|e.to_string())?)
            })();
            if let Err(error)=evidence {response.code=3;response.stderr.push_str(&format!("error: public library observation failed: {error}\n"));normalize_failure(req,&mut response);}
        } else if step.is_dir() {
            if let Err(error)=fs::remove_dir_all(&step) {response.code=3;response.stderr.push_str(&format!("error: public library cleanup failed: {error}\n"));normalize_failure(req,&mut response);}
        }
        response
    }

    fn compile_executable(&mut self, req: &driver::Request, session: &mut driver::Session, serial: u64) -> driver::Response {
        let result = (|| -> Result<driver::Response, String> {
            let executable = self.executable.as_ref().expect("the executable request has bound inputs");
            self.installed.unchanged()?;
            if !version_supported(req) { return Err("executable SDK version differs from the pinned context".into()); }
            let key = req.session.library.as_ref().ok_or("executable context absent")?;
            let check = req.cmd == driver::Cmd::Check;
            let emit = if check { "metadata" } else { "link" };
            let [dependencies, output] = key.executable_outputs(&req.out, check).ok_or("executable output plan changed")?;
            let filename = output.file_name().ok_or("executable output filename absent")?;
            let parent = output.parent().ok_or("executable output parent absent")?;
            let stage = parent.join(format!(".lustr-executable-{}-{serial}", std::process::id()));
            let staged_output = stage.join(filename);
            fs::create_dir(&stage).map_err(|error| format!("stage native executable: {error}"))?;
            let result = (|| -> Result<driver::Response, String> {
                let mut staged = req.clone();
                staged.out = staged_output;
                let mut checked = 0;
                let mut reused = false;
                let mut source = None;
                let mut unused = Vec::new();
                let mut response = match driver::prepare_library_import(&staged, session, executable) {
                    driver::Preparation::Ready(prepared) => {
                        if !prepared.uses_only_root_source() {
                            return Err("Cargo executable dependencies are outside the represented source-file set".into());
                        }
                        if req.json && req.library_options.force_warn_unused_crate_dependencies {
                            let used = prepared.executable_extern_uses().ok_or("executable extern lint form is not represented")?;
                            unused = executable.unused_externs(&used);
                        }
                        checked = prepared.checked_functions().len();
                        if check {
                            prepared.emit()
                        } else if let Some(cached) = &self.binary_output
                            && cached.source.as_ref() == prepared.source() && cached.shown == req.client.shown {
                            write_new(&staged.out, &cached.bytes)?;
                            fs::set_permissions(&staged.out, fs::Permissions::from_mode(cached.mode)).map_err(|error|error.to_string())?;
                            reused = true;
                            driver::Response::default()
                        } else {
                            source = Some(Arc::<str>::from(prepared.source()));
                            prepared.emit()
                        }
                    }
                    driver::Preparation::Complete(response) | driver::Preparation::Rejected(response)
                        | driver::Preparation::Failed(response) | driver::Preparation::IntegrationIncomplete(response) => response,
                };
                if response.code != 0 { return Ok(response); }
                if !key.executable_dependencies.as_ref().is_some_and(|inputs| executable.unchanged(inputs)) {
                    return Err("Cargo executable dependency changed during checking or linking".into());
                }
                // rustc emits a zero-byte metadata output for an executable.
                // Its dependencies carry the genuine library metadata.
                if check {
                    write_new(&staged.out, &[])?;
                }
                let completed = if let Some(source) = source {
                    let mode = fs::metadata(&staged.out).map_err(|error|error.to_string())?.mode() & 0o777;
                    let bytes = fs::read(&staged.out).map_err(|error|error.to_string())?.into();
                    Some(BinaryArtifact { source, shown:req.client.shown.clone(), mode, bytes })
                } else { None };
                let input = req.client.shown.as_deref().unwrap_or(&req.file);
                let paths = [dependencies, output];
                let dep = stage.join(key.dep_info_name());
                write_new(&dep, &dep_info(&paths, input)?)?;
                match publication::replace_set(&[(&dep, &paths[0]), (&staged.out, &paths[1])])
                    .map_err(|error| format!("replace Cargo executable: {error}"))? {
                    publication::Replacement::Durable => {}
                    publication::Replacement::Incomplete { replaced, error } => return Err(format!("executable publication stopped after {replaced} files: {error}")),
                    publication::Replacement::DurabilityUnknown(error) => return Err(format!("executable publication durability unknown: {error}")),
                }
                if let Some(completed) = completed { self.binary_output = Some(completed); }
                if req.json {
                    if req.library_options.force_warn_unused_crate_dependencies {
                        response.stderr.push_str(&format!("{}\n", json!({"$message_type":"unused_extern",
                            "lint_level":"force-warn", "unused_extern_names":unused})));
                    }
                    for (artifact, emit) in paths.iter().zip(["dep-info", emit]) {
                        response.stderr.push_str(&format!("{}\n", json!({"$message_type":"artifact","artifact":artifact,"emit":emit})));
                    }
                }
                if let Some(root) = req.client.env.as_ref().and_then(|env|env.iter().find(|(name,_)|name=="LUSTR_LIBRARY_REPORT_DIR")) {
                    let root = Path::new(&root.1);
                    if !root.is_absolute() || !root.is_dir() { return Err("executable report directory must be an existing absolute directory".into()); }
                    let report = root.join(format!("executable-{}-{serial}", std::process::id()));
                    fs::create_dir(&report).map_err(|error| error.to_string())?;
                    write_new(&report.join("report.json"), &serde_json::to_vec_pretty(&json!({
                        "schema":"lustr-public-executable-v1", "owner_pid":std::process::id(), "status":0,
                        "source_path":req.file, "checked_bodies":checked, "artifact_reused":reused,
                        "outputs":paths, "emit":emit, "unused_extern_names":unused,
                    })).map_err(|error| error.to_string())?)?;
                }
                Ok(response)
            })();
            let cleanup = fs::remove_dir_all(&stage).map_err(|error| format!("clean executable staging: {error}"));
            let response = result?;
            cleanup?;
            Ok(response)
        })();
        let mut response = result.unwrap_or_else(|error| failed(req, &error));
        normalize_failure(req, &mut response);
        response
    }
}
fn version_supported(req:&driver::Request)->bool {
    const VERSION:&str="1.102.0-nightly (5b7b37aef 2026-11-08)";
    match &req.client.env {
        Some(env)=>env.iter().filter(|(name,_)|name=="RUSTC_FORCE_RUSTC_VERSION").all(|(_,value)|value==VERSION),
        None=>std::env::var_os("RUSTC_FORCE_RUSTC_VERSION").is_none_or(|value|value==VERSION),
    }
}
fn output_records(paths:&[PathBuf],expected:&[&[u8]],emits:&[&str])->Value{
    json!(paths.iter().zip(expected).zip(emits).map(|((path,bytes),emit)|
        json!({"path":path,"sha256":hash(bytes),"size":bytes.len(),"emit":emit})).collect::<Vec<_>>())
}
/// Original Rust 5b7b37 rustc_interface::passes::escape_dep_filename and
/// write_out_deps layout, applied to the actual completed source/output list.
fn dep_info(outputs:&[PathBuf],source:&Path)->Result<Vec<u8>,String>{
    let source=source.to_str().ok_or("dep-info source is not Unicode")?;
    if source.contains(['\n','\r','\0']){return Err("dep-info source normalization is unrepresented".into());}
    let escaped=source.replace(' ',"\\ ");
    let mut text=String::new();
    for output in outputs {
        let output=output.to_str().ok_or("dep-info output is not Unicode")?;
        if output.contains(['\n','\r','\0']){return Err("dep-info output normalization is unrepresented".into());}
        text.push_str(&format!("{output}: {escaped}\n\n"));
    }
    text.push_str(&format!("{escaped}:\n"));
    Ok(text.into_bytes())
}
fn normalize_failure(req:&driver::Request,response:&mut driver::Response){
    if !req.json || response.code==0{return;}
    let mut diagnostics=String::new();
    for stream in [&response.stdout,&response.stderr] {
        for line in stream.lines() {
            if serde_json::from_str::<Value>(line).is_ok_and(|v|v["$message_type"]=="diagnostic") {
                diagnostics.push_str(line);diagnostics.push('\n');
            } else if !line.is_empty() {diagnostics.push_str(&diag::cargo_message(line));}
        }
    }
    response.stdout.clear();response.stderr=diagnostics;
}
pub(crate) fn failed(req:&driver::Request,error:&str)->driver::Response {
    let mut response=driver::Response{code:3,stderr:format!("error: {error}\n"),..Default::default()};
    normalize_failure(req,&mut response);response
}
