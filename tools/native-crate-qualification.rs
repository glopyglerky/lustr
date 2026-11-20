//! SDK bootstrap of the compiler qualifier. Producer objects use only native
//! Lust/Lustr phases and the explicit C backend. No rustc source acceptance.
#![allow(dead_code, unused_imports)]
include!("../crates/lustr-native-core/src/lib.rs");
#[link(name="tcc",kind="static")]
unsafe extern "C" {}

#[path = "native-crate-types-qualification.rs"] mod typed_image_qualification;
#[path = "native-crate-regions-qualification.rs"] mod region_image_qualification;
#[path = "native-crate-callbacks-qualification.rs"] mod callback_image_qualification;
#[path = "native-crate-nominals-qualification.rs"] mod nominal_image_qualification;
#[path = "native-crate-named-regions-qualification.rs"] mod named_region_qualification;
#[path = "native-crate-runtime-qualification.rs"] mod runtime_qualification;
#[path = "native-crate-adts-qualification.rs"] mod adt_qualification;
#[path = "native-crate-consts-qualification.rs"] mod const_qualification;
#[path = "native-crate-borrowed-nominals-qualification.rs"] mod borrowed_nominal_qualification;
#[path = "native-crate-bounded-nominals-qualification.rs"] mod bounded_nominal_qualification;
#[path = "native-crate-recursive-nominals-qualification.rs"] mod recursive_nominal_qualification;
#[path = "native-crate-bounded-functions-qualification.rs"] mod bounded_function_qualification;
#[path = "native-crate-wide-references-qualification.rs"] mod wide_reference_qualification;
#[path = "native-crate-generic-nominals-qualification.rs"] mod generic_nominal_qualification;
#[path = "native-crate-generic-applications-qualification.rs"] mod generic_application_qualification;

fn put(path:&std::path::Path,bytes:&[u8]){
    use std::io::Write;
    let mut f=std::fs::OpenOptions::new().create_new(true).write(true).open(path).unwrap();f.write_all(bytes).unwrap();f.sync_all().unwrap();
}
fn args(source:&std::path::Path,output:&std::path::Path,metadata:&str)->Vec<String>{vec![
    source.to_str().unwrap().into(),"--crate-name".into(),"same".into(),"--edition=2024".into(),"--crate-type=lib".into(),"--emit=lustr-native-crate".into(),
    "--target=x86_64-unknown-linux-gnu".into(),"-Copt-level=0".into(),"-Cdebuginfo=0".into(),"-Cpanic=abort".into(),"-Coverflow-checks=off".into(),"-Cdebug-assertions=off".into(),
    format!("-Cmetadata={metadata}"),"-o".into(),output.to_str().unwrap().into(),
]}
fn input(root:&std::path::Path,text:&str,ids:def::CrateIds)->resolve::worklist::CrateInput{
    resolve::worklist::CrateInput {root_path:root.join("consumer.rs"),root_text:text.into(),env:resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags::DEV),
        crate_ids:ids,crate_name:"consumer".into(),cargo_env:Vec::new(),deps:Vec::new(),client:Default::default(),std_splice_lo:None,display_root:None}
}
fn resolve_ok(store:&native_crate::CrateStore,input:&resolve::worklist::CrateInput,i:&intern::Interner,k:&intern::Known)->resolve::worklist::ResolvedCrate{
    store.resolve(input,i,k).unwrap_or_else(|mut e|panic!("{}",diag::render(&mut e.diags,&e.map,&e.all_text,false,20)))
}
fn main(){
    raise_stack_limit();
    let mut arguments=std::env::args_os().skip(1);let mode=arguments.next().expect("NEW_OUTPUT_DIRECTORY");
    let path_test=mode=="--backend-path-test";
    let callee_first=mode=="--callee-first";
    let indirect_first=mode=="--indirect-first";
    let abi_producer=mode=="--abi-producer";let abi_consumer=mode=="--abi-consumer";
    let abi_value=mode=="--abi-value-consumer";
    let namespace_test=mode=="--namespace";
    let types_first=mode=="--types-first";let types_test=mode=="--types";
    let regions_test=mode=="--regions";
    let callbacks_test=mode=="--callbacks";
    let nominal_first=mode=="--nominal-first";
    let nominals_test=mode=="--nominals";
    let nominal_original_test=mode=="--nominal-original";
    let named_first=mode=="--named-first";let named_test=mode=="--named";
    let runtime_test=mode=="--runtime";
    let adts_first=mode=="--adts-first";
    let adts_test=mode=="--adts";
    let consts_first=mode=="--consts-first";
    let consts_test=mode=="--consts";
    let borrowed_first=mode=="--borrowed-nominals-first";
    let borrowed_test=mode=="--borrowed-nominals";
    let bounded_first=mode=="--bounded-nominals-first";
    let bounded_test=mode=="--bounded-nominals";
    let recursive_first=mode=="--recursive-nominals-first";
    let recursive_test=mode=="--recursive-nominals";
    let bounded_fn_first=mode=="--bounded-functions-first";
    let bounded_fn_test=mode=="--bounded-functions";
    let wide_first=mode=="--wide-references-first";
    let wide_test=mode=="--wide-references";
    if mode=="--generic-applications"||mode=="--generic-applications-first"||mode=="--generic-nominals-first"||mode=="--generic-nominals"{
        let root=std::path::PathBuf::from(arguments.next().expect("CHILD_DIRECTORY"));std::fs::create_dir(&root).unwrap();
        let backend=native_crate::Backend::select(std::path::Path::new("/usr/bin/clang")).unwrap();
        let generation=native_crate::Generation{compiler:native_crate::file_digest(&std::env::current_exe().unwrap()).unwrap(),backend:backend.digest()};
        if mode=="--generic-applications"{generic_application_qualification::run(&root,&generation,&backend);}else if mode=="--generic-applications-first"{generic_application_qualification::first(&root,&generation,&backend);}else if mode=="--generic-nominals-first"{generic_nominal_qualification::first(&root,&generation,&backend);}else{generic_nominal_qualification::run(&root,&generation,&backend);}return;
    }
    let root=std::path::PathBuf::from(if wide_first||wide_test||bounded_fn_first||bounded_fn_test||recursive_first||recursive_test||path_test||abi_producer||abi_consumer||abi_value||callee_first||indirect_first||namespace_test||types_first||types_test||regions_test||callbacks_test||nominal_first||nominals_test||nominal_original_test||named_first||named_test||runtime_test||adts_first||adts_test||consts_first||consts_test||borrowed_first||borrowed_test||bounded_first||bounded_test{arguments.next().expect("CHILD_DIRECTORY")}else{mode});std::fs::create_dir(&root).unwrap();
    let backend=native_crate::Backend::select(std::path::Path::new("/usr/bin/clang")).unwrap();
    let generation=native_crate::Generation {compiler:native_crate::file_digest(&std::env::current_exe().unwrap()).unwrap(),backend:backend.digest()};
    if wide_first{wide_reference_qualification::first(&root,&generation,&backend);return;}
    if wide_test{wide_reference_qualification::run(&root,&generation,&backend);return;}
    if bounded_fn_first{bounded_function_qualification::first(&root,&generation,&backend);return;}
    if bounded_fn_test{bounded_function_qualification::run(&root,&generation,&backend);return;}
    if recursive_first{recursive_nominal_qualification::first(&root,&generation,&backend);return;}
    if recursive_test{recursive_nominal_qualification::run(&root,&generation,&backend);return;}
    if bounded_first{bounded_nominal_qualification::first(&root,&generation,&backend);return;}
    if bounded_test{bounded_nominal_qualification::run(&root,&generation,&backend);return;}
    if borrowed_first{borrowed_nominal_qualification::first(&root,&generation,&backend);return;}
    if borrowed_test{borrowed_nominal_qualification::run(&root,&generation,&backend);return;}
    if consts_first{const_qualification::first(&root,&generation,&backend);return;}
    if consts_test{const_qualification::run(&root,&generation,&backend);return;}
    if adts_first{adt_qualification::first(&root,&generation,&backend);return;}
    if adts_test{adt_qualification::run(&root,&generation,&backend);return;}
    if runtime_test{runtime_qualification::run(&root,&generation,&backend);return;}
    if nominal_first{nominal_image_qualification::first(&root,&generation,&backend);return;}
    if nominals_test{nominal_image_qualification::run(&root,&generation,&backend);return;}
    if nominal_original_test{nominal_image_qualification::original_gate(&root,&generation,&backend);return;}
    if named_first||named_test{named_region_qualification::run(&root,&generation,&backend,named_first);return;}
    if regions_test{region_image_qualification::run(&root,&generation,&backend);return;}
    if callbacks_test{callback_image_qualification::run(&root,&generation,&backend);return;}
    if types_first||types_test{typed_image_qualification::run(&root,&generation,&backend,types_first);return;}
    if namespace_test{namespace_probes(&root,&generation,&backend);return;}
    if callee_first||indirect_first {callee_probes(&root,&generation,&backend,indirect_first);return;}
    if abi_producer {
        let source=root.join("wide.rs");put(&source,b"#![no_std]\npub fn signed(x:i128)->i128{x+1}\npub fn unsigned(x:u128)->u128{x+2}\n");
        let(code,message)=native_object::run_artifact(&args(&source,&root.join("wide.lustr-native"),"wide"),&generation,&backend);print!("{message}");std::process::exit(code);
    }
    if abi_consumer||abi_value {
        let image=std::path::PathBuf::from(arguments.next().expect("PRODUCER_IMAGE"));
        let artifact=native_crate::Artifact::decode(&std::fs::read(image).unwrap(),&generation).unwrap();
        let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("wide-consumer","bin","consumer",false)};
        let store=native_crate::CrateStore::new(vec![("wide".into(),artifact)],ids).unwrap();
        let source=if abi_value {"use wide::{signed,unsigned}; fn main(){let s:fn(i128)->i128=signed;let u:fn(u128)->u128=unsigned;assert!(s(170141183460469231731687303715884105726i128)==170141183460469231731687303715884105727i128);assert!(u(340282366920938463463374607431768211453u128)==340282366920938463463374607431768211455u128);}"}else{"use wide::{signed,unsigned}; fn main(){assert!(signed(170141183460469231731687303715884105726i128)==170141183460469231731687303715884105727i128);assert!(unsigned(340282366920938463463374607431768211453u128)==340282366920938463463374607431768211455u128);}"};
        let mut request=input(&root,source,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        if let Err(error)=store.executable(&request,&generation,&backend,&root.join("native")){eprintln!("{error}");std::process::exit(1);}
        let run=std::process::Command::new(root.join("native/app")).output().unwrap();put(&root.join("app.stdout"),&run.stdout);put(&root.join("app.stderr"),&run.stderr);put(&root.join("app.status"),format!("{}",run.status).as_bytes());std::process::exit(run.status.code().unwrap_or(1));
    }
    if path_test {
        let source=root.join("source.rs");put(&source,b"#![no_std]\npub fn value(x:i64)->i64{x+1}\n");
        let (code,message)=native_object::run_artifact(&args(&source,&root.join("path-independent.lustr-native"),"pathcase"),&generation,&backend);
        print!("{message}");std::process::exit(code);
    }
    let first="#![no_std]\nfn private(x:i64)->i64{x+2}\npub fn value(x:i64)->i64{private(x)*3}\npub fn flag(x:bool)->bool{!x}\n";
    let second="#![no_std]\npub fn value(x:bool)->bool{!x}\nfn private(x:i64)->i64{x+9}\n";
    let mut images=Vec::new();
    for (label,source,metadata) in [("left",first,"version1"),("right",second,"version2")] {
        let src=root.join(format!("{label}.rs"));let out=root.join(format!("{label}.lustr-native"));put(&src,source.as_bytes());
        let (code,message)=native_object::run_artifact(&args(&src,&out,metadata),&generation,&backend);put(&root.join(format!("{label}.log")),message.as_bytes());assert_eq!(code,0,"{message}");
        let bytes=std::fs::read(&out).unwrap();let artifact=native_crate::Artifact::decode(&bytes,&generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);
        assert!(!artifact.functions().is_empty());assert!(unit_object::symbols(artifact.object()).unwrap().undefined.is_empty());
        // The payload checksum covers semantics and every actual object byte.
        let mut corrupt=bytes.clone();let at=corrupt.len()-40;corrupt[at]^=1;assert!(native_crate::Artifact::decode(&corrupt,&generation).is_err());
        let mut wrong=generation.clone();wrong.compiler[0]^=1;assert!(native_crate::Artifact::decode(&bytes,&wrong).is_err());
        wrong=generation.clone();wrong.backend[0]^=1;assert!(native_crate::Artifact::decode(&bytes,&wrong).is_err());
        assert!(native_crate::Artifact::decode(&bytes[..bytes.len()-1],&generation).is_err());
        images.push(artifact);
    }
    for (label,text) in [("lifetime","#![no_std]\npub fn value<'a>(x:i64)->i64{x}\n"),("lifetime-bound","#![no_std]\npub fn value<'a:'b,'b>(x:i64)->i64{x}\n"),("where-lifetime","#![no_std]\npub fn value(x:i64)->i64 where 'static:'static{x}\n")] {
        let source=root.join(format!("{label}.rs"));let output=root.join(format!("{label}.lustr-native"));put(&source,text.as_bytes());
        let (code,message)=native_object::run_artifact(&args(&source,&output,"lifetime_declaration"),&generation,&backend);put(&root.join(format!("{label}.log")),message.as_bytes());
        assert_eq!(code,0,"{label}: {message}");let bytes=std::fs::read(&output).unwrap();assert_eq!(native_crate::Artifact::decode(&bytes,&generation).unwrap().encode().unwrap(),bytes);
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join(format!("lib{label}.rlib"))).output().unwrap();put(&root.join(format!("{label}.oracle.stdout")),&oracle.stdout);put(&root.join(format!("{label}.oracle.stderr")),&oracle.stderr);put(&root.join(format!("{label}.oracle.status")),oracle.status.to_string().as_bytes());assert!(oracle.status.success());
    }
    assert!(native_crate::Backend::select(std::path::Path::new("clang")).is_err());
    let mut wrong=generation.clone();wrong.backend[0]^=1;let out=root.join("wrong-backend.lustr-native");
    let (code,message)=native_object::run_artifact(&args(&root.join("left.rs"),&out,"wrong_backend"),&wrong,&backend);assert_eq!(code,1);assert!(!out.exists());put(&root.join("wrong-backend.log"),message.as_bytes());
    let fake=root.join("fake-path");std::fs::create_dir(&fake).unwrap();std::os::unix::fs::symlink("/usr/bin/false",fake.join("clang")).unwrap();
    let child=std::process::Command::new(std::env::current_exe().unwrap()).arg("--backend-path-test").arg(root.join("explicit-backend"))
        .env("PATH",format!("{}:/usr/bin:/bin",fake.display())).output().unwrap();put(&root.join("explicit-backend.stdout"),&child.stdout);put(&root.join("explicit-backend.stderr"),&child.stderr);assert!(child.status.success());
    native_crate::Artifact::decode(&std::fs::read(root.join("explicit-backend/path-independent.lustr-native")).unwrap(),&generation).unwrap();
    malformed(&root,&images[0].encode().unwrap(),&generation);
    malformed_namespace(&root,&images[0].encode().unwrap(),&generation);
    assert_ne!(images[0].functions()[0].id.krate,images[1].functions()[0].id.krate);
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("independent-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(vec![("left".into(),images[0].clone()),("right".into(),images[1].clone()),("again".into(),images[0].clone())],ids).unwrap();
    let changed=root.join("changed.rs");let changed_image=root.join("changed.lustr-native");put(&changed,first.replace("x+2","x+3").as_bytes());
    let (code,message)=native_object::run_artifact(&args(&changed,&changed_image,"version1"),&generation,&backend);put(&root.join("changed.log"),message.as_bytes());assert_eq!(code,0,"{message}");
    let changed=native_crate::Artifact::decode(&std::fs::read(changed_image).unwrap(),&generation).unwrap();
    assert_eq!(changed.functions()[0].id,images[0].functions()[0].id);
    assert!(native_crate::CrateStore::new(vec![("old".into(),images[0].clone()),("new".into(),changed)],ids).is_err(),"conflicting builds cannot share one imported identity");
    let i=std::sync::Arc::new(intern::Interner::default());
    // Deliberately change the consumer's dense interner allocation history.
    for n in 0..200 {i.intern(&format!("unrelated_{n}"));}
    let k=std::sync::Arc::new(intern::Known::new(&i));
    let good="use left::value as integer;\nuse right::value as boolean;\nuse again::value as alias;\nfn main(){}\n";
    let resolved=resolve_ok(&store,&input(&root,good,ids),&i,&k);
    use driver::graph::{BindingRes,Namespace};use ty::defs::ROOT_MODULE;
    let bound=|name:&str|match resolved.tables.binding(ROOT_MODULE,Namespace::Value,i.intern(name)).unwrap(){BindingRes::Def(id)=>id,_=>panic!("not a real definition")};
    let left=bound("integer");let right=bound("boolean");assert_ne!(left.krate,right.krate);assert_eq!(left,bound("alias"));assert_ne!(left.krate,def::LOCAL_CRATE);
    let lf=store.function(left).unwrap();let rf=store.function(right).unwrap();assert_eq!(store.disk_id(left),Some(lf.id));assert_eq!(store.disk_id(right),Some(rf.id));
    assert_eq!(lf.params.len(),1);assert_eq!(rf.params.len(),1);
    let mut order=check::collect::DefOrder::default();
    let tables=std::sync::Arc::new(resolved.tables);
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i.clone(),k.clone(),ids,false,&mut order,Some(tables)).unwrap_or_else(|e|panic!("collect diagnostics {}",e.len()));
    let local_functions=p.fns.len();let original_items=p.ast.items.len();store.install_signatures(&mut p).unwrap();
    assert_eq!(p.fns.len(),local_functions);assert_eq!(p.ast.items.len(),original_items);assert_eq!(local_functions,1,"no synthetic import function AST");
    for (id,kind) in [(left,ty::TyKind::Int(ty::IntTy::I64)),(right,ty::TyKind::Bool)] {
        let data=p.defs.get(id).unwrap();assert_eq!(data.kind,ty::defs::DefKind::ForeignFn);
        let ty::SigData::Fn{params,ret}=&data.sig else{panic!("no actual typed signature")};assert_eq!(params.len(),1);
        assert_eq!(ty::TyIntern::kind(&p.tys,params[0]),kind);assert_eq!(ty::TyIntern::kind(&p.tys,*ret),kind);
    }
    for (label,text) in [("private","use left::private; fn main(){}"),("absent","use left::absent; fn main(){}"),("namespace","use left::value; fn main(){let _:value;}"),("ambiguous","use left::*; use right::*; fn main(){value(1);}")] {
        let result=store.resolve(&input(&root,text,ids),&i,&k);
        if label=="namespace" {let resolved=result.unwrap_or_else(|_|panic!("namespace should reach native type checking"));
            let mut order=check::collect::DefOrder::default();let program=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i.clone(),k.clone(),ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables)));
            match program{Err(e)=>assert!(!e.is_empty()),Ok(p)=>{let (_,errors)=check::check_fn(&p,p.main.unwrap());assert!(!errors.is_empty());}}
        } else {assert!(result.is_err(),"{label} unexpectedly resolved");}
        put(&root.join(format!("{label}.source.rs")),text.as_bytes());
    }
    // A real imported call must pass the existing native argument checker.
    let call="use left::value; fn main(){let x=value(4);}";
    let resolved=resolve_ok(&store,&input(&root,call,ids),&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i.clone(),k.clone(),ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("call collect diagnostics {}",e.len()));
    store.install_signatures(&mut p).unwrap();let (_,mut errors)=check::check_fn(&p,p.main.unwrap());
    put(&root.join("call-frontier.diagnostics"),diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20).as_bytes());
    assert!(errors.is_empty(),"real typed native imported call remains unsupported");
    put(&root.join("call-frontier.source.rs"),call.as_bytes());
    runtime_calls(&root,&store,ids,&generation,&backend);
    runtime_function_values(&root,&store,ids,&generation,&backend);
    let callee=root.join("callee-regression");std::fs::create_dir(&callee).unwrap();callee_probes(&callee,&generation,&backend,false);
    abi_processes(&root);
    // A fresh interner/consumer bin identity must not alter producer DiskDefId.
    let ids2=def::CrateIds{stable:def::stable_crate_hash("different-consumer","bin","other",false),..ids};
    let store2=native_crate::CrateStore::new(vec![("left".into(),images[0].clone())],ids2).unwrap();let j=intern::Interner::default();let known=intern::Known::new(&j);
    let resolved=resolve_ok(&store2,&input(&root,"use left::value; fn main(){}",ids2),&j,&known);
    let BindingRes::Def(remapped)=resolved.tables.binding(ROOT_MODULE,Namespace::Value,j.intern("value")).unwrap() else{panic!("missing remap")};
    assert_eq!(store2.disk_id(remapped),store.disk_id(left));assert!(store.resolve(&input(&root,good,ids2),&i,&k).is_err());
    assert!(native_crate::CrateStore::new(vec![("dup".into(),images[0].clone()),("dup".into(),images[1].clone())],ids).is_err());
    let reserved=native_crate::CrateStore::new(vec![("core".into(),images[0].clone())],ids).unwrap();assert!(reserved.resolve(&input(&root,"fn main(){}",ids),&i,&k).is_err());
    // Actual code closure remains a separately usable object. No application
    // runs here; runtime and consumer codegen are explicitly later gates.
    for (n,image) in images.iter().enumerate(){put(&root.join(format!("object-{n}.o")),image.object());}
    println!("native semantic artifact gate: independent real producers, checked imported calls and native consumer execution, alias/version/privacy/namespace/ambiguity checks, generation/corruption refusal and consumer remap PASS; no Cargo/speed/full-language/Rust-ABI admission");
}

fn callee_probes(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,indirect_only:bool){
    let source=root.join("producer.rs");put(&source,b"#![no_std]\npub fn value(x:i64)->i64{x+1}\n");let image=root.join("producer.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"callee_first"),generation,backend);assert_eq!(code,0,"{message}");
    let artifact=native_crate::Artifact::decode(&std::fs::read(image).unwrap(),generation).unwrap();let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("callee-probe","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("left".into(),artifact)],ids).unwrap();
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("libleft.rlib")).output().unwrap();put(&root.join("oracle-producer.stderr"),&oracle.stderr);assert!(oracle.status.success());
    let mut all=true;
    for (label,text,valid) in [
        ("reference-side-effect","use left::value; fn main(){let mut seen=false; ({seen=true; &value})(1); assert!(seen);}",true),
        ("item-side-effect","use left::value; fn main(){let mut seen=false; ({seen=true; value})(1); assert!(seen);}",true),
        ("reference-order","use left::value; fn main(){let mut n=0i64; let x=({n=n*10+1; &value})({n=n*10+2; 4}); assert!(n==12); assert!(x==5);}",true),
        ("item-order","use left::value; fn main(){let mut n=0i64; let x=({n=n*10+1; value})({n=n*10+2; 4}); assert!(n==12); assert!(x==5);}",true),
        ("pointer-argument-change","use left::value; fn double(x:i64)->i64{x*2} fn main(){let mut p:fn(i64)->i64=value; let result=p({p=double; 4}); assert!(result==5); assert!(p(4)==8);}",true),
        ("moved-reference","use left::value; fn main(){let mut f=value; let r=&mut f; let s=r; r(1);}",false),
        ("borrowed-item","use left::value; fn main(){let mut f=value; let r=&mut f; f(1); r(2);}",false),
    ] {
        if indirect_only&&label!="pointer-argument-change"{continue;}
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let out=root.join(format!("native-{label}"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("native-{label}.result")),format!("{result:?}").as_bytes());
        if valid {match result {Ok(())=>{let run=std::process::Command::new(out.join("app")).output().unwrap();put(&root.join(format!("native-{label}.stdout")),&run.stdout);put(&root.join(format!("native-{label}.stderr")),&run.stderr);put(&root.join(format!("native-{label}.status")),format!("{}",run.status).as_bytes());all &= run.status.success();},Err(_)=>all=false,}}else{all &= result.is_err();/* Invalid Rust artifacts MUST NEVER execute. */}
        let checked=std::process::Command::new("/core/bin/rustc").arg(source).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","--extern"]).arg(format!("left={}",root.join("libleft.rlib").display())).arg("-o").arg(root.join(format!("oracle-{label}"))).output().unwrap();put(&root.join(format!("oracle-{label}.stderr")),&checked.stderr);assert_eq!(checked.status.success(),valid);
        if valid {let run=std::process::Command::new(root.join(format!("oracle-{label}"))).output().unwrap();put(&root.join(format!("oracle-{label}.run.stderr")),&run.stderr);put(&root.join(format!("oracle-{label}.run.status")),format!("{}",run.status).as_bytes());assert!(run.status.success());}
    }
    assert!(all,"native callee evaluation/borrow firsts failed; preserve all artifacts, invalid app UNEXECUTED");
}

fn abi_processes(root:&std::path::Path){
    let exe=std::env::current_exe().unwrap();
    for (label,key) in [("ordinary",None),("return-override",Some("LUST_RELEASE_AGG_RET")),("parameter-override",Some("LUST_RELEASE_AGG_PARAM")),("backend-override",Some("LUST_RELEASE_BACKEND"))] {
        let producer=root.join(format!("abi-producer-{label}"));let mut command=std::process::Command::new(&exe);command.arg("--abi-producer").arg(&producer);if let Some(key)=key{command.env(key,if key=="LUST_RELEASE_BACKEND"{"llvm"}else{"all"});}
        let output=command.output().unwrap();put(&root.join(format!("abi-producer-{label}.stdout")),&output.stdout);put(&root.join(format!("abi-producer-{label}.stderr")),&output.stderr);put(&root.join(format!("abi-producer-{label}.status")),format!("{}",output.status).as_bytes());
        if key.is_some(){assert!(!output.status.success());assert!(!producer.join("wide.lustr-native").exists());assert!(String::from_utf8_lossy(&output.stdout).contains("does not admit release/LLVM ABI overrides"));}else{assert!(output.status.success());}
    }
    let image=root.join("abi-producer-ordinary/wide.lustr-native");
    for value in [false,true] {for (label,key) in [("ordinary",None),("return-override",Some("LUST_RELEASE_AGG_RET")),("parameter-override",Some("LUST_RELEASE_AGG_PARAM")),("backend-override",Some("LUST_RELEASE_BACKEND"))] {
        let label=format!("{}{label}",if value{"value-"}else{""});let consumer=root.join(format!("abi-consumer-{label}"));let mut command=std::process::Command::new(&exe);command.arg(if value{"--abi-value-consumer"}else{"--abi-consumer"}).arg(&consumer).arg(&image);if let Some(key)=key{command.env(key,if key=="LUST_RELEASE_BACKEND"{"llvm"}else{"all"});}
        let output=command.output().unwrap();put(&root.join(format!("abi-consumer-{label}.stdout")),&output.stdout);put(&root.join(format!("abi-consumer-{label}.stderr")),&output.stderr);put(&root.join(format!("abi-consumer-{label}.status")),format!("{}",output.status).as_bytes());
        if key.is_some(){assert!(!output.status.success());assert!(!consumer.join("native/app").exists());assert!(String::from_utf8_lossy(&output.stderr).contains("does not admit release/LLVM ABI overrides"));}else{assert!(output.status.success());}
    }}
}

fn runtime_calls(root:&std::path::Path,store:&native_crate::CrateStore,ids:def::CrateIds,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let text="use left::value as integer; use right::value as boolean; use again::value as alias;\nfn local(x:i64)->i64{x+1}\nfn main(){assert!(integer(4)==18); assert!(!boolean(true)); assert!(alias(local(4))==21); let x=9i64; assert!(integer(local(local(x)))==39);}\n";
    put(&root.join("runtime-consumer.rs"),text.as_bytes());
    let mut request=input(root,&text,ids);
    request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
    let mut dead=request.clone();dead.root_text="use left::value; const X:i64=if false {value(4)} else {0}; fn main(){}".into();
    put(&root.join("first-dead-const.source.rs"),dead.root_text.as_bytes());
    let result=store.executable(&dead,generation,backend,&root.join("first-dead-const"));
    put(&root.join("first-dead-const.result"),format!("{result:?}").as_bytes());
    assert!(result.is_err(),"invalid dead constant foreign call accepted; emitted artifact MUST NOT be executed");
    assert!(result.unwrap_err().contains("non-const"),"dead constant call must report its actual non-const identity");
    let native=root.join("runtime-native");store.executable(&request,generation,backend,&native).unwrap_or_else(|e|panic!("native consumer: {e}"));
    let run=std::process::Command::new(native.join("app")).output().unwrap();
    put(&root.join("runtime-native.stdout"),&run.stdout);put(&root.join("runtime-native.stderr"),&run.stderr);put(&root.join("runtime-native.status"),format!("{}",run.status).as_bytes());assert!(run.status.success(),"native consumer execution failed");
    // SDK use is explicit differential oracle only; it never provides code
    // to the native image, native consumer or their link.
    for (label,metadata) in [("left","version1"),("right","version2")] {
        let output=std::process::Command::new("/core/bin/rustc").arg(root.join(format!("{label}.rs"))).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();
        put(&root.join(format!("oracle-{label}.stdout")),&output.stdout);put(&root.join(format!("oracle-{label}.stderr")),&output.stderr);assert!(output.status.success());
    }
    let mut command=std::process::Command::new("/core/bin/rustc");command.arg(root.join("runtime-consumer.rs")).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    for (alias,label) in [("left","left"),("again","left"),("right","right")]{command.arg("--extern").arg(format!("{alias}={}",root.join(format!("lib{label}.rlib")).display()));}
    let output=command.arg("-o").arg(root.join("oracle-app")).output().unwrap();put(&root.join("oracle-consumer.stdout"),&output.stdout);put(&root.join("oracle-consumer.stderr"),&output.stderr);assert!(output.status.success());
    let oracle=std::process::Command::new(root.join("oracle-app")).output().unwrap();put(&root.join("oracle-runtime.stdout"),&oracle.stdout);put(&root.join("oracle-runtime.stderr"),&oracle.stderr);put(&root.join("oracle-runtime.status"),format!("{}",oracle.status).as_bytes());assert!(oracle.status.success());assert_eq!(run.stdout,oracle.stdout);assert_eq!(run.stderr,oracle.stderr);
    for (label,source) in [
        ("wrong-argument","use left::value; fn main(){value(true);}"),
        ("wrong-return","use left::value; fn main(){let _:bool=value(4);}"),
        ("arity","use left::value; fn main(){value(1,2);}"),
        ("generic-argument","use left::value; fn main(){value::<i64>(4);}"),
        ("const-call","use left::value; const X:i64=value(4); fn main(){}"),
        ("dead-const-call","use left::value; const X:i64=if false {value(4)} else {0}; fn main(){}"),
        ("short-circuit-call","use left::flag; const X:bool=false && flag(true); fn main(){}"),
        ("short-circuit-or","use left::flag; const X:bool=true || flag(true); fn main(){}"),
        ("dead-body-const","use left::value; fn main(){const X:i64=if false {value(4)} else {0};}"),
        ("short-circuit-body-const","use left::flag; fn main(){const X:bool=false && flag(true);}"),
        ("inline-const-call","use left::flag; fn main(){let _=const{false && flag(true)};}"),
        ("anonymous-const-call","use left::value; fn main(){let _=[0u8;if false {value(4) as usize} else {0}];}"),
    ] {
        put(&root.join(format!("reject-{label}.rs")),source.as_bytes());let mut request=request.clone();request.root_text=source.into();
        let result=store.executable(&request,generation,backend,&root.join(format!("reject-{label}")));let message=result.err().unwrap_or_else(||panic!("{label} incorrectly accepted"));
        put(&root.join(format!("reject-{label}.log")),message.as_bytes());
        assert!(!root.join(format!("reject-{label}/app")).exists());
        if label.contains("const")||label.starts_with("short-circuit"){assert!(message.contains("non-const"),"{label}: {message}");}
        let mut oracle=std::process::Command::new("/core/bin/rustc");oracle.arg(root.join(format!("reject-{label}.rs"))).args(["--edition=2024","-Cpanic=abort","--extern"]).arg(format!("left={}",root.join("libleft.rlib").display())).arg("-o").arg(root.join(format!("oracle-{label}")));
        let checked=oracle.output().unwrap();put(&root.join(format!("oracle-reject-{label}.stderr")),&checked.stderr);assert!(!checked.status.success(),"oracle {label}");
    }
}

fn runtime_function_values(root:&std::path::Path,store:&native_crate::CrateStore,ids:def::CrateIds,generation:&native_crate::Generation,backend:&native_crate::Backend){
    // The retained V63 first rejected `let f=value; f(4)`; keep that source
    // unchanged in this mixed witness while extending independent item use.
    let source="use left::value; use again::value as alias; use right::value as boolean;\nfn double(x:i64)->i64{x*2}\nfn apply(f:fn(i64)->i64,x:i64)->i64{f(x)}\nfn selected()->fn(i64)->i64{value}\nfn main(){let f=value; f(4); assert!(f(4)==18); let p:fn(i64)->i64=value; assert!(p(5)==21); let q=value as fn(i64)->i64; assert!(apply(q,6)==24); assert!(selected()(7)==27); let choose=boolean(false); let mixed=if choose {alias}else{double}; assert!(mixed(8)==30); let local=if !choose {value}else{double}; assert!(local(9)==18); let b:fn(bool)->bool=boolean; assert!(!b(true)); let copied=f; assert!(copied(10)==36);}\n";
    put(&root.join("function-values.rs"),source.as_bytes());
    let mut request=input(root,source,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
    let native=root.join("function-values-native");store.executable(&request,generation,backend,&native).unwrap_or_else(|e|panic!("native function values: {e}"));
    let run=std::process::Command::new(native.join("app")).output().unwrap();put(&root.join("function-values-native.stdout"),&run.stdout);put(&root.join("function-values-native.stderr"),&run.stderr);put(&root.join("function-values-native.status"),format!("{}",run.status).as_bytes());assert!(run.status.success(),"native function-value application failed");
    let mut command=std::process::Command::new("/core/bin/rustc");command.arg(root.join("function-values.rs")).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    for (alias,label) in [("left","left"),("again","left"),("right","right")]{command.arg("--extern").arg(format!("{alias}={}",root.join(format!("lib{label}.rlib")).display()));}
    let checked=command.arg("-o").arg(root.join("function-values-oracle")).output().unwrap();put(&root.join("function-values-oracle-build.stderr"),&checked.stderr);assert!(checked.status.success());
    let oracle=std::process::Command::new(root.join("function-values-oracle")).output().unwrap();put(&root.join("function-values-oracle.stdout"),&oracle.stdout);put(&root.join("function-values-oracle.stderr"),&oracle.stderr);put(&root.join("function-values-oracle.status"),format!("{}",oracle.status).as_bytes());assert!(oracle.status.success());assert_eq!(run.stdout,oracle.stdout);assert_eq!(run.stderr,oracle.stderr);
    for (label,text) in [
        ("item-identity","use left::flag; use right::value; fn main(){let mut f=flag; f=value;}"),
        ("pointer-argument","use left::value; fn main(){let p:fn(bool)->i64=value;}"),
        ("pointer-return","use left::value; fn main(){let p:fn(i64)->bool=value;}"),
        ("value-generics","use left::value; fn main(){let p=value::<i64>;}"),
        ("value-arity","use left::value; fn main(){let p=value; p(1,2);}"),
        ("value-wrong-arg","use left::value; fn main(){let p=value; p(true);}"),
        ("value-const-fn","use left::value; const fn bad()->i64{let p=value; p(1)} fn main(){}"),
        ("pointer-const-fn","use left::value; const fn bad()->i64{let p:fn(i64)->i64=value; p(1)} fn main(){}"),
        ("value-dead-const-fn","use left::value; const fn bad()->i64{let p=value; if false {p(1)}else{0}} fn main(){}"),
        ("value-short-const","use left::flag; fn main(){let _=const{let f=flag; false&&f(true)};}"),
        ("pointer-short-const","use left::flag; fn main(){let _=const{let f:fn(bool)->bool=flag; true||f(true)};}"),
    ]{
        let path=root.join(format!("values-reject-{label}.rs"));put(&path,text.as_bytes());let mut bad=request.clone();bad.root_text=text.into();
        let out=root.join(format!("values-reject-{label}"));let result=store.executable(&bad,generation,backend,&out);let message=result.err().unwrap_or_else(||panic!("invalid {label} emitted an artifact; MUST NOT execute it"));put(&root.join(format!("values-reject-{label}.log")),message.as_bytes());assert!(!out.join("app").exists());
        if label.contains("const"){
            let pointer_call=label=="pointer-const-fn"&&message.contains("error[L0631]")&&message.contains("function pointer");
            // A foreign item value in an inline constant still needs a real
            // const-value identity record. This is explicit unsupported scope,
            // not evidence of a nonconst-call diagnosis or constant support.
            let image_gap=matches!(label,"value-short-const"|"pointer-short-const")&&message.contains("error[L0630]")&&message.contains("bounded const evaluator");
            assert!(message.contains("non-const")||pointer_call||image_gap,"{label}: {message}");
        }
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(path).args(["--edition=2024","-Cpanic=abort"]);
        for (alias,label) in [("left","left"),("right","right")]{command.arg("--extern").arg(format!("{alias}={}",root.join(format!("lib{label}.rlib")).display()));}
        let checked=command.arg("-o").arg(root.join(format!("values-oracle-reject-{label}"))).output().unwrap();put(&root.join(format!("values-oracle-reject-{label}.stderr")),&checked.stderr);assert!(!checked.status.success(),"stock accepted invalid {label}");
    }
}

fn malformed(root:&std::path::Path,bytes:&[u8],generation:&native_crate::Generation){
    fn number(b:&[u8],n:usize)->usize{u32::from_le_bytes(b[n..n+4].try_into().unwrap()) as usize}
    fn text(b:&[u8],at:&mut usize){*at+=4+number(b,*at);}
    fn reject(root:&std::path::Path,label:&str,mut payload:Vec<u8>,generation:&native_crate::Generation){
        payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&payload);
        let error=native_crate::Artifact::decode(&payload,generation).err().expect(label);put(&root.join(format!("malformed-{label}.log")),error.as_bytes());
        let expected=match label{"type-tag"=>"unknown native type tag","string-bound"|"object-bound"=>"field exceeds bound","boolean"=>"invalid native boolean","profile"=>"target/profile/pipeline mismatch","duplicate-def"=>"definition/type/code identity mismatch","trailing"=>"trailing semantic data",_=>unreachable!()};
        assert!(error.contains(expected),"{label}: decoder rejected the wrong obligation: {error}");
    }
    let original=&bytes[..bytes.len()-32];let mut at=b"LUSTR-NATIVE-CRATE\0\x05".len()+64;text(original,&mut at);let profile=at+4;text(original,&mut at);let name=at;text(original,&mut at);text(original,&mut at);at+=8+32+2;
    let count=number(original,at);at+=4;let start=at;let mut records=Vec::new();let mut first_tag=0;let mut first_bool=0;
    for n in 0..count{let begin=at;at+=16;text(original,&mut at);if n==0{first_bool=at;}at+=1;let len=number(original,at);at+=4;at+=4*(len+1);text(original,&mut at);records.push(begin..at);}
    let type_graph=at;first_tag=at+8;at+=4+number(original,at);let namespace=at;at+=4+number(original,at);at+=4+number(original,at);let runtime_size=at;at+=4+number(original,at);let object_size=at;
    let mut payload=original.to_vec();payload[first_tag]=255;reject(root,"type-tag",payload,generation);
    let mut payload=original.to_vec();payload[name..name+4].copy_from_slice(&u32::MAX.to_le_bytes());reject(root,"string-bound",payload,generation);
    let mut payload=original.to_vec();payload[first_bool]=2;reject(root,"boolean",payload,generation);
    let mut payload=original.to_vec();payload[profile]=b'!';reject(root,"profile",payload,generation);
    assert!(records.len()>1);let mut payload=original[..start].to_vec();payload.extend_from_slice(&original[records[0].clone()]);payload.extend_from_slice(&original[records[0].clone()]);for r in records.iter().skip(2){payload.extend_from_slice(&original[r.clone()]);}payload.extend_from_slice(&original[type_graph..runtime_size]);
    // Keep the new runtime relation consistent so this still reaches the
    // independent duplicate-definition obligation rather than another guard.
    let mut runtime=native_crate::runtime::Requirements::decode(&original[runtime_size+4..object_size]).unwrap();runtime.locations[1].id=runtime.locations[0].id;let runtime=runtime.encode();payload.extend_from_slice(&(runtime.len()as u32).to_le_bytes());payload.extend_from_slice(&runtime);payload.extend_from_slice(&original[object_size..]);reject(root,"duplicate-def",payload,generation);
    let mut payload=original.to_vec();payload[object_size..object_size+4].copy_from_slice(&u32::MAX.to_le_bytes());reject(root,"object-bound",payload,generation);
    let mut payload=original.to_vec();payload.push(0);reject(root,"trailing",payload,generation);
}

fn namespace_probes(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let producer="#![no_std]\nmod implementation { pub mod api { pub fn value(x:i64)->i64{super::helper(x)+1} pub(crate) fn restricted(x:i64)->i64{x} fn private(x:i64)->i64{x} } pub(super) fn helper(x:i64)->i64{x*3} }\npub use implementation::api;\npub use implementation::api::value as renamed;\npub mod other { pub fn value(x:bool)->bool{!x} }\n";
    let source=root.join("producer.rs");put(&source,producer.as_bytes());let image=root.join("producer.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"namespace"),generation,backend);put(&root.join("producer.log"),message.as_bytes());assert_eq!(code,0,"{message}");
    let bytes=std::fs::read(&image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);
    malformed_namespace(root,&bytes,generation);
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("namespace-consumer","bin","consumer",false)};
    let second_source=root.join("second.rs");put(&second_source,producer.as_bytes());let second_image=root.join("second.lustr-native");let(code,message)=native_object::run_artifact(&args(&second_source,&second_image,"namespace_second"),generation,backend);assert_eq!(code,0,"{message}");let second=native_crate::Artifact::decode(&std::fs::read(second_image).unwrap(),generation).unwrap();
    let store=native_crate::CrateStore::new(vec![("left".into(),artifact.clone()),("again".into(),artifact),("right".into(),second)],ids).unwrap();
    let i=intern::Interner::default();let k=intern::Known::new(&i);let resolved=resolve_ok(&store,&input(root,"fn main(){}",ids),&i,&k);
    let roots:Vec<_>=["left","right","again"].iter().map(|name|resolved.tables.externs.iter().find(|e|e.name==*name).unwrap().root_module.unwrap()).collect();assert_ne!(roots[0],roots[1]);assert_eq!(roots[0],roots[2]);
    let mut keys=std::collections::BTreeSet::new();for root in &roots[..2]{for path in [vec![],vec![i.intern("implementation")],vec![i.intern("implementation"),i.intern("api")]]{let module=resolved.tables.module_by_path_in(*root,&path).unwrap();let key=resolved.tables.module_key(module);assert!(keys.insert(key));assert_eq!(resolved.tables.module_by_key(key),Some(module));}}
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("libsame.rlib")).output().unwrap();put(&root.join("oracle-producer.stderr"),&oracle.stderr);assert!(oracle.status.success());
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&second_source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cmetadata=namespace_second","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("libsame-second.rlib")).output().unwrap();put(&root.join("oracle-second.stderr"),&oracle.stderr);assert!(oracle.status.success());
    for (label,text,valid) in [
        ("paths","use left::api::value; use again::renamed; use left::other::value as boolean; fn main(){assert!(value(4)==13);let p:fn(i64)->i64=renamed;assert!(p(5)==16);assert!(!boolean(true));assert!(left::api::value(6)==19);}",true),
        ("glob","use left::api::*; fn main(){assert!(value(7)==22);}",true),
        ("hidden-module","fn main(){left::implementation::api::value(1);}",false),
        ("restricted","fn main(){left::api::restricted(1);}",false),
        ("private","use left::api::private; fn main(){private(1);}",false),
        ("same-short-name","use left::api::value; use left::other::value as boolean; fn main(){let mut f=value;f=boolean;}",false),
        ("equal-signature-items","use left::api::value; use right::api::value as other; fn main(){let mut f=value;f=other;}",false),
        ("wrong-type","fn main(){left::api::value(true);}",false),
    ] {
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.root_path=source.clone();request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let out=root.join(format!("native-{label}"));let native=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native-result")),format!("{native:?}").as_bytes());
        let oracle_out=root.join(format!("oracle-{label}"));let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","--extern"]).arg(format!("left={}",root.join("libsame.rlib").display())).arg("--extern").arg(format!("again={}",root.join("libsame.rlib").display())).arg("--extern").arg(format!("right={}",root.join("libsame-second.rlib").display())).arg("-o").arg(&oracle_out).output().unwrap();put(&root.join(format!("{label}.oracle-diagnostics")),&oracle.stderr);
        if !valid{assert!(native.is_err(),"invalid native acceptance: {label}; NEVER execute artifact");assert!(!out.join("app").exists());assert!(!oracle.status.success());continue;}
        native.unwrap();assert!(oracle.status.success(),"{}",String::from_utf8_lossy(&oracle.stderr));
        let native=std::process::Command::new(out.join("app")).output().unwrap();let oracle=std::process::Command::new(oracle_out).output().unwrap();
        for (which,result) in [("native",&native),("oracle",&oracle)] {put(&root.join(format!("{label}.{which}-stdout")),&result.stdout);put(&root.join(format!("{label}.{which}-stderr")),&result.stderr);put(&root.join(format!("{label}.{which}-status")),format!("{}",result.status).as_bytes());}
        assert!(native.status.success()&&oracle.status.success());assert_eq!(native.stdout,oracle.stdout);assert_eq!(native.stderr,oracle.stderr);
    }
    println!("native producer resolver namespace image: paths, module re-export, function alias, glob, independent item identities and native runtime/oracle parity; hidden/restricted/private/type refusals PASS");
}

// Mutate retained wire bytes independently of the candidate namespace types.
// Every case recomputes the checksum, so integrity failure cannot satisfy it.
fn malformed_namespace(root:&std::path::Path,bytes:&[u8],generation:&native_crate::Generation){
    fn number(b:&[u8],at:&mut usize)->usize{let n=u32::from_le_bytes(b[*at..*at+4].try_into().unwrap()) as usize;*at+=4;n}
    fn string(b:&[u8],at:&mut usize)->String{let n=number(b,at);let s=std::str::from_utf8(&b[*at..*at+n]).unwrap().to_owned();*at+=n;s}
    fn path(b:&[u8],at:&mut usize)->Vec<String>{(0..number(b,at)).map(|_|string(b,at)).collect()}
    fn visibility(b:&[u8],at:&mut usize){let tag=b[*at];*at+=1;if tag==4{path(b,at);}}
    let payload=&bytes[..bytes.len()-32];let mut at=b"LUSTR-NATIVE-CRATE\0\x05".len()+64;for _ in 0..4{string(payload,&mut at);}at+=8+32+2;
    for _ in 0..number(payload,&mut at){at+=16;string(payload,&mut at);at+=1;let count=number(payload,&mut at);at+=4*(count+1);string(payload,&mut at);}
    let type_size=number(payload,&mut at);at+=type_size;
    let namespace_length=at;let size=number(payload,&mut at);let namespace_start=at;let namespace_end=at+size;
    for _ in 0..number(payload,&mut at){path(payload,&mut at);let present=payload[at];at+=1+16*present as usize;visibility(payload,&mut at);}
    for _ in 0..number(payload,&mut at){at+=16;string(payload,&mut at);path(payload,&mut at);visibility(payload,&mut at);at+=1;}
    let mut private=None;
    for _ in 0..number(payload,&mut at){let module=path(payload,&mut at);at+=1;let name=string(payload,&mut at);let target=payload[at];at+=1;if target==0{at+=16;}else{path(payload,&mut at);}let begin=at;visibility(payload,&mut at);let end=at;path(payload,&mut at);if name=="private"{private=Some((begin..end,module.is_empty()));}}
    assert_eq!(at,namespace_end);let (range,root_private)=private.expect("private binding");
    let mut in_path=vec![4];in_path.extend_from_slice(&2u32.to_le_bytes());for s in ["crate","missing"]{in_path.extend_from_slice(&(s.len() as u32).to_le_bytes());in_path.extend_from_slice(s.as_bytes());}
    let mut cases=vec![("private-public",vec![1]),("missing-in",in_path)];
    if root_private{cases.push(("root-super",vec![3]));}else{let mut in_path=vec![4];in_path.extend_from_slice(&2u32.to_le_bytes());for s in ["crate","other"]{in_path.extend_from_slice(&(s.len() as u32).to_le_bytes());in_path.extend_from_slice(s.as_bytes());}cases.push(("nonancestor-in",in_path));}
    for (label,replacement) in cases {
        let mut altered=payload.to_vec();altered.splice(range.clone(),replacement);let newsize=altered.len()-(payload.len()-size);altered[namespace_length..namespace_length+4].copy_from_slice(&(newsize as u32).to_le_bytes());altered.extend_from_slice(&native_crate::digest(&altered).unwrap());
        put(&root.join(format!("namespace-malformed-{label}.lustr-native")),&altered);let error=native_crate::Artifact::decode(&altered,generation).err().expect(label);put(&root.join(format!("namespace-malformed-{label}.log")),error.as_bytes());assert!(error.contains("native namespace visibility"),"{label}: {error}");
    }
    assert!(namespace_start<namespace_end);
}
