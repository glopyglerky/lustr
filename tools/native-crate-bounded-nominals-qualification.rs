//! Declared nominal lifetime obligations through original native tables.
use super::*;
pub(super) const ORIGINAL:&str="#![no_std]\npub struct Bounded<'a,'b:'a>{pub short:&'a i64,pub long:&'b i64}\npub fn sum(p:Bounded<'_,'_>)->i64{*p.short+*p.long}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"bounded_original"),generation,backend);put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());
    let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("liboriginal.rlib")).output().unwrap();record(root,"original.oracle-build",&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));
    println!("original bounded lifetime nominal native refusal/no image/no apps; separate explicit SDK oracle rlib build passes");
}
const SHAPES:&str=r#"#![no_std]
pub struct Where<'a,'b> where 'b:'a {pub short:&'a i64,pub long:&'b i64}
pub enum Choice<'a,'b:'a>{Short(&'a i64),Long(&'b i64)}
pub struct Static<'a:'static>{pub value:&'a i64}
pub struct Chain<'a,'b:'a,'c:'b>{pub a:&'a i64,pub b:&'b i64,pub c:&'c i64}
pub struct Equivalent<'a:'b,'b:'a>{pub a:&'a i64,pub b:&'b i64}
pub struct Outer<'a,'b:'a>{pub pair:Where<'a,'b>}
pub struct StaticOuter<'a:'static>{pub p:Static<'a>}
pub struct Closed(pub i64);
pub fn where_sum(p:Where<'_,'_>)->i64{*p.short+*p.long}
pub fn choice(p:Choice<'_,'_>)->i64{match p{Choice::Short(x)=>*x,Choice::Long(x)=>*x}}
pub fn static_value(p:Static<'_>)->i64{*p.value}
pub fn chain(p:Chain<'_,'_,'_>)->i64{*p.a+*p.b+*p.c}
pub fn equal(p:Equivalent<'_,'_>)->i64{*p.a+*p.b}
pub fn outer(p:Outer<'_,'_>)->i64{where_sum(p.pair)}
pub fn static_outer(p:StaticOuter<'_>)->i64{*p.p.value}
pub fn closed(p:Closed)->i64{p.0}
"#;
fn replace(b:&mut[u8],p:usize,n:u32){b[p..p+4].copy_from_slice(&n.to_le_bytes());}
fn checksum(mut b:Vec<u8>)->Vec<u8>{b.extend_from_slice(&native_crate::digest(&b).unwrap());b}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation){
    let raw=std::fs::read(root.join("shapes.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let wires=borrowed_nominal_qualification::fact_wires(original);let chain=wires.iter().find(|w|w.0=="Chain").unwrap();assert_eq!(chain.2.len(),2);
    let reject=|label:&str,b:Vec<u8>,expected:&str|{let b=checksum(b);put(&root.join(format!("malformed-{label}.lustr-native")),&b);let e=native_crate::Artifact::decode(&b,generation).err().expect("malformed outlives image accepted");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());assert!(e.contains(expected),"{label}: {e}");};
    let mut b=original.to_vec();replace(&mut b,chain.2[0],3);reject("unbound-subject",b,"outlives fact index/order/bound");
    let mut b=original.to_vec();replace(&mut b,chain.2[0]+4,3);reject("unbound-target",b,"outlives fact index/order/bound");
    let mut b=original.to_vec();replace(&mut b,chain.2[0],u32::MAX);reject("static-subject",b,"outlives fact index/order/bound");
    let mut b=original.to_vec();b[chain.2[1]..chain.2[1]+8].copy_from_slice(&original[chain.2[0]..chain.2[0]+8]);reject("duplicate-fact",b,"outlives fact index/order/bound");
    let mut b=original.to_vec();b[chain.2[0]..chain.2[0]+8].copy_from_slice(&original[chain.2[1]..chain.2[1]+8]);b[chain.2[1]..chain.2[1]+8].copy_from_slice(&original[chain.2[0]..chain.2[0]+8]);reject("unsorted-facts",b,"outlives fact index/order/bound");
    let mut b=original.to_vec();replace(&mut b,chain.1,4097);reject("fact-count",b,"field exceeds bound");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=10;reject("old-format10",b,"format generation");
}
fn nested_static_before_publication(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    let raw=std::fs::read(root.join("shapes.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let wires=borrowed_nominal_qualification::fact_wires(original);let outer=wires.iter().find(|w|w.0=="StaticOuter").unwrap();assert_eq!(outer.2.len(),1);let mut b=original.to_vec();replace(&mut b,outer.1,0);let size=u32::from_le_bytes(original[outer.4..outer.4+4].try_into().unwrap());replace(&mut b,outer.4,size-8);b.drain(outer.2[0]..outer.2[0]+8);let bad=checksum(b);put(&root.join("malformed-nested-static.lustr-native"),&bad);
    // Complete applications, indexes, erasure and ABI still validate. The
    // original native obligation check must refuse before publication.
    let bad=native_crate::Artifact::decode(&bad,generation).unwrap();let healthy=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let store=native_crate::CrateStore::new(vec![("healthy".into(),healthy),("bad".into(),bad)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let mut request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().filter(|d|d.krate!=def::LOCAL_CRATE).copied().collect();let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let e=store.install_signatures(p).unwrap_err();assert!(e.contains("native nominal outlives obligations"),"{e}");for d in &foreign{assert!(p.defs.get(*d).is_none());assert!(!p.type_names.contains_key(&tables.def_info[d].flat));}assert!(p.consts.is_empty());assert!(p.regions.native_fns.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),e,"Expected original native outlives refusal before publication")])};
    let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("nested static must stop collection");let rendered=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!rendered.contains("division by zero"));put(&root.join("nested-static-before-definitions.log"),rendered.as_bytes());
    request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("nested-static-native");let e=store.executable(&request,generation,backend,&out).unwrap_err();assert!(e.contains("native nominal outlives obligations"),"{e}");assert!(!out.join("app").exists());put(&root.join("nested-static-executable.log"),e.as_bytes());
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;
    let a=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let stable=a.functions()[0].id.krate;
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("perturbed-bounded-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("base".into(),a)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("bounded_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local<'x>{v:&'x i64}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect errors {}",e.len()));let adts=p.adts.len();let fns=p.fns.len();let region_fns=p.regions.fns.len();
    let local=p.adts.iter().enumerate().find_map(|(n,a)|(p.name(a.name)=="Local").then(||p.def_of_adt(ty::AdtId(n as u32)))).expect("actual consumer Local nominal");let kept=p.regions.adt_fields[&local][0][0];p.regions.outlives_items.push(check::outlives::Item{implied:vec![kept],required:vec![kept],facts:Vec::new(),preds:Vec::new(),span:diag::Span::default()});let saved=p.regions.outlives_items[0].clone();
    for n in 101..133{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}store.install_signatures(&mut p).unwrap();assert_eq!(p.adts.len(),adts+1);assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),region_fns);assert_eq!(p.regions.outlives_items.len(),1);let current=&p.regions.outlives_items[0];assert_eq!(current.required,saved.required);assert_eq!(current.implied,saved.implied);assert_eq!(current.facts,saved.facts);assert_eq!(current.preds,saved.preds);assert_eq!(current.span,saved.span);
    let d=p.def_of_adt(ty::AdtId(adts as u32));assert_eq!(d.hash,def::item_hash(stable,&[],def::DefKind::Adt,"Bounded",0));assert_eq!(p.gx.adt_lifetimes[&d].iter().map(|&s|p.name(s)).collect::<Vec<_>>(),["a","b"]);assert_eq!(p.regions.adt_facts[&d],[(1,0)]);assert_eq!(p.gx.adt_scopes[adts].region_facts,[(1,0)]);
    let i64_ty=p.tys.intern(ty::TyKind::Int(ty::IntTy::I64));for n in 0..2{assert_eq!(p.tys.kind(p.regions.adt_fields[&d][0][n]),ty::TyKind::Ref(ty::Region::Param(n as u32),ty::Mut::Shared,i64_ty));}
    put(&root.join("remap-install-only.txt"),b"Actual stable definition, declaration names, both scope/table outlives facts and exact kept fields survive fresh interner/local ADT/type/region perturbation. No extra FnInfo/local function-region rows. INSTALL ONLY, no app execution from perturbed Program.\n");
}
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("base",ORIGINAL,"bounded_original"),("other",ORIGINAL,"bounded_independent"),("shapes",SHAPES,"bounded_shapes")]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");let b=std::fs::read(image).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();assert_eq!(a.encode().unwrap(),b);images.push((label.into(),a));
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-producer"),&oracle);assert!(oracle.status.success(),"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("bounded-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid)in[
        ("direct","use base::{Bounded,sum};fn main(){let a=11;let b=17;assert!(sum(Bounded{short:&a,long:&b})==28);}",true),
        ("implied-bound","use base::Bounded;fn weaken<'a,'b>(p:Bounded<'a,'b>)->&'a i64{p.long}fn main(){let a=11;let b=17;assert!(*weaken(Bounded{short:&a,long:&b})==17);}",true),
        ("where-enum-nested","use shapes::{Where,where_sum,Choice,choice,Outer,outer,Closed,closed};fn main(){let a=3;let b=7;assert!(where_sum(Where{short:&a,long:&b})==10);assert!(choice(Choice::Short(&a))==3);assert!(choice(Choice::Long(&b))==7);assert!(outer(Outer{pair:Where{short:&a,long:&b}})==10);assert!(closed(Closed(29))==29);}",true),
        ("transitive-equality","use shapes::{Chain,chain,Equivalent,equal};fn transitive<'a,'b,'c>(p:Chain<'a,'b,'c>)->&'a i64{p.c}fn main(){let a=3;let b=7;let c=11;assert!(*transitive(Chain{a:&a,b:&b,c:&c})==11);assert!(chain(Chain{a:&a,b:&b,c:&c})==21);assert!(equal(Equivalent{a:&a,b:&b})==10);}",true),
        ("static-bound","use shapes::{Static,static_value,StaticOuter,static_outer};fn static_get<'a>(p:Static<'a>)->&'static i64{p.value}fn main(){assert!(static_value(Static{value:&59})==59);assert!(*static_get(Static{value:&73})==73);assert!(static_outer(StaticOuter{p:Static{value:&83}})==83);}",true),
        ("independent","use base::{Bounded,sum};use other::{Bounded as Other,sum as other};fn main(){let a=11;let b=17;assert!(sum(Bounded{short:&a,long:&b})==28);assert!(other(Other{short:&a,long:&b})==28);}",true),
        ("unmet-annotation","use base::Bounded;fn fail<'a,'b>(a:&'a i64,b:&'b i64){let p:Bounded<'a,'b>=Bounded{short:a,long:b};}fn main(){}",false),
        ("static-local","use shapes::{Static,static_value};fn main(){let a=3;static_value(Static{value:&a});}",false),
        ("static-wrapper","use shapes::Static;struct Wrapper<'a>{p:Static<'a>}fn main(){}",false),
        ("escape","use base::Bounded;fn weaken<'a,'b>(p:Bounded<'a,'b>)->&'a i64{p.long}fn main(){let r;{let a=1;let b=3;r=weaken(Bounded{short:&a,long:&b});}let x=*r;}",false),
        ("invent-static","use base::Bounded;fn fail<'a,'b>(p:Bounded<'a,'b>)->&'static i64{p.long}fn main(){}",false),
        ("identity","use base::{Bounded,sum};use other::Bounded as Other;fn main(){let a=1;sum(Other{short:&a,long:&a});}",false),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.expect_err("invalid emitted output; NEVER EXECUTE");let code=match label{"unmet-annotation"|"static-wrapper"=>"L0617","static-local"|"escape"=>"L0306","invent-static"=>"L0733","identity"=>"L0110",_=>unreachable!()};assert!(e.contains(code),"{label}: {e}");assert!(!out.join("app").exists());}
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in["base","other","shapes"]{command.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let app=root.join(format!("{label}-oracle"));let oracle=command.arg("-o").arg(&app).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert_eq!(oracle.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
        if valid{let n=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success());let o=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    for(label,text)in[
        ("bad-producer-static","#![no_std]\npub struct Static<'a:'static>{pub v:&'a i64}\npub struct Wrap<'a>{pub p:Static<'a>}\npub fn f()->i64{0}\n"),
        ("bad-producer-bound","#![no_std]\npub struct Bounded<'a,'b:'a>{pub a:&'a i64,pub b:&'b i64}\npub fn fail<'a,'b>(p:Bounded<'a,'b>)->&'static i64{p.b}\n"),
    ]{let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,label.replace('-',"_").as_str()),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,1,"invalid producer accepted: {label}; NEVER CONSUME/EXECUTE");assert!(!image.exists());let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-o"]).arg(root.join(format!("{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-producer"),&oracle);assert!(!oracle.status.success());}
    malformed(root,generation);nested_static_before_publication(root,generation,backend,ids);remap(root,generation);
    println!("native declared nominal lifetime bounds PASS; fullRust/no_std/Cargo/SAMEcompletework remain OPEN");
}
