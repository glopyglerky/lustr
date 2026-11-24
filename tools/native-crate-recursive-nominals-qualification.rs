//! Recursive nominal identities through the original native type/borrow engine.
use super::*;
pub(super) const ORIGINAL:&str=r#"#![no_std]
pub enum Chain<'a>{End(i64),Link(&'a Chain<'a>,i64)}
pub fn sum(p:&Chain<'_>)->i64{match p{Chain::End(v)=>*v,Chain::Link(next,v)=>sum(next)+*v}}
"#;
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"recursive_original"),generation,backend);put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());
    let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("liboriginal.rlib")).output().unwrap();record(root,"original.oracle-build",&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));
    println!("original recursive nominal native refusal/no image/no apps; separate explicit SDK oracle rlib build passes");
}
const MUTUAL:&str=r#"#![no_std]
pub enum Left<'a>{End(i64),Next(&'a Right<'a>)}
pub struct Right<'a>{pub left:&'a Left<'a>,pub value:i64}
pub fn left(p:&Left<'_>)->i64{match p{Left::End(v)=>*v,Left::Next(r)=>right(r)}}
pub fn right(p:&Right<'_>)->i64{left(p.left)+p.value}
pub enum Mutable<'a>{End(i64),Next(&'a mut Mutable<'a>)}
pub fn bump(p:&mut Mutable<'_>){match p{Mutable::End(v)=>*v+=1,Mutable::Next(next)=>bump(next)}}
pub fn value(p:&Mutable<'_>)->i64{match p{Mutable::End(v)=>*v,Mutable::Next(next)=>value(next)}}
"#;
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("base",ORIGINAL,"recursive_original"),("other",ORIGINAL,"recursive_independent"),("mutual",MUTUAL,"recursive_mutual")]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");let b=std::fs::read(image).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();assert_eq!(a.encode().unwrap(),b);images.push((label.into(),a));
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-producer"),&oracle);assert!(oracle.status.success(),"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("recursive-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid)in[
        ("chain","use base::{Chain,sum};fn main(){let a=Chain::End(3);let b=Chain::Link(&a,7);let c=Chain::Link(&b,11);assert!(sum(&c)==21);}",true),
        ("mutual-cycle","use mutual::{Left,Right,left,right};fn main(){let a=Left::End(5);let b=Right{left:&a,value:13};let c=Left::Next(&b);assert!(left(&c)==18);assert!(right(&b)==18);}",true),
        ("mutable-valid","use mutual::{Mutable,bump,value};fn main(){let mut a=Mutable::End(7);let mut b=Mutable::Next(&mut a);bump(&mut b);assert!(value(&b)==8);}",true),
        ("mutable","use mutual::{Mutable,bump};fn main(){let mut a=Mutable::End(7);{let mut b=Mutable::Next(&mut a);bump(&mut b);}match a{Mutable::End(v)=>assert!(v==8),Mutable::Next(_)=>assert!(false)}}",false),
        ("independent","use base::{Chain,sum};use other::{Chain as Other,sum as other};fn main(){let a=Chain::End(3);let b=Other::End(7);assert!(sum(&a)==3);assert!(other(&b)==7);}",true),
        ("escape","use base::{Chain,sum};fn main(){let p;{let a=Chain::End(3);p=Chain::Link(&a,7);}sum(&p);}",false),
        ("move","use base::{Chain,sum};fn main(){let a=Chain::End(3);let b=a;sum(&a);}",false),
        ("identity","use base::sum;use other::Chain;fn main(){let a=Chain::End(3);sum(&a);}",false),
        ("wrong-field","use base::Chain;fn main(){let a=Chain::End(3);let b=Chain::Link(&a,true);}",false),
        ("static","use base::Chain;fn fail<'a>(p:&'a Chain<'a>)->&'static Chain<'static>{p}fn main(){}",false),
        ("mutable-alias","use mutual::{Mutable,bump};fn main(){let mut a=Mutable::End(7);let mut b=Mutable::Next(&mut a);bump(&mut a);bump(&mut b);}",false),
        ("mutable-invariance","use mutual::Mutable;fn shorten<'a>(p:Mutable<'static>,r:&'a i64)->Mutable<'a>{p}fn main(){}",false),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.expect_err("invalid emitted output; NEVER EXECUTE");let code=match label{"mutable"=>"L0303","escape"=>"L0306","move"=>"L0301","identity"|"wrong-field"=>"L0110","static"|"mutable-invariance"=>"L0733","mutable-alias"=>"L0304",_=>unreachable!()};assert!(e.contains(code),"{label}: {e}");assert!(!out.join("app").exists());}
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in["base","other","mutual"]{command.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let app=root.join(format!("{label}-oracle"));let oracle=command.arg("-o").arg(&app).output().unwrap();record(root,&format!("{label}.oracle-consumer"),&oracle);assert_eq!(oracle.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
        if valid{let n=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success());let o=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    malformed(root,generation);late_abi(root,generation,backend,ids);remap(root,generation);
    println!("native recursive borrowed nominal gate PASS; full Rust/no_std/native Cargo/SAME complete-work speed remain OPEN");
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;
    let a=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let stable=a.functions()[0].id.krate;let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("recursive-remap","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("base".into(),a)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("recursive_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local<'x>{v:&'x i64}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect {}",e.len()));let adts=p.adts.len();let fns=p.fns.len();let region_fns=p.regions.fns.len();
    let local=p.adts.iter().enumerate().find_map(|(n,a)|(p.name(a.name)=="Local").then(||p.def_of_adt(ty::AdtId(n as u32)))).unwrap();let kept=p.regions.adt_fields[&local][0][0];p.regions.outlives_items.push(check::outlives::Item{implied:vec![kept],required:vec![kept],facts:Vec::new(),preds:Vec::new(),span:diag::Span::default()});let queued=p.regions.outlives_items[0].clone();for n in 17..49{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}
    store.install_signatures(&mut p).unwrap();assert_eq!(p.adts.len(),adts+1);assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),region_fns);assert_eq!(p.regions.outlives_items.len(),1);let current=&p.regions.outlives_items[0];assert_eq!(current.implied,queued.implied);assert_eq!(current.required,queued.required);assert_eq!(current.facts,queued.facts);assert_eq!(current.preds,queued.preds);assert_eq!(current.span,queued.span);
    let d=p.def_of_adt(ty::AdtId(adts as u32));assert_eq!(d.hash,def::item_hash(stable,&[],def::DefKind::Adt,"Chain",0));assert_eq!(p.gx.adt_lifetimes[&d].iter().map(|&s|p.name(s)).collect::<Vec<_>>(),["a"]);let ref_ty=p.regions.adt_fields[&d][1][0];let ty::TyKind::Ref(r,m,inner)=p.tys.kind(ref_ty)else{panic!("recursive reference missing")};assert_eq!(r,ty::Region::Param(0));assert_eq!(m,ty::Mut::Shared);let ty::TyKind::Adt(target,args)=p.tys.kind(inner)else{panic!("recursive nominal missing")};assert_eq!(target,d);assert_eq!(p.tys.args(args),[ty::GenericArg::Lifetime(ty::Region::Param(0))]);let erased=p.adts[adts].variants[1].fields[0].ty;assert_eq!(borrowck::erase(&mut p.tys,ref_ty),erased);
    put(&root.join("remap-install-only.txt"),b"Fresh interner/local ADT/type/region perturbation retains exact stable recursive target, declaration, kept Param0 and erased field equality; all queued consumer Item fields unchanged; no extra FnInfo/local region rows. INSTALL ONLY, no runtime from perturbed Program.\n");
}
fn late_abi(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    let mut images:Vec<_>=["base","other"].iter().map(|n|{let b=std::fs::read(root.join(format!("{n}.lustr-native"))).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();(a.functions()[0].id.krate,b,a)}).collect();images.sort_by_key(|v|v.0);let good=images[0].2.clone();let raw=&images[1].1;let wire=borrowed_nominal_qualification::reference_wires(raw);let mut b=raw[..raw.len()-32].to_vec();b[wire[0].5]^=1;b.extend_from_slice(&native_crate::digest(&b).unwrap());put(&root.join("later-reference-bad-abi.lustr-native"),&b);let bad=native_crate::Artifact::decode(&b,generation).unwrap();let store=native_crate::CrateStore::new(vec![("good".into(),good),("bad".into(),bad)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().copied().filter(|d|d.krate!=def::LOCAL_CRATE).collect();let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let e=store.install_signatures(p).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");for d in &foreign{assert!(p.defs.get(*d).is_none());assert!(!p.type_names.contains_key(&tables.def_info[d].flat));}assert!(p.consts.is_empty()&&p.const_defs.is_empty()&&p.const_names.is_empty());assert!(p.regions.native_fns.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),e,"Expected later recursive-reference ABI refusal")])};let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("late ABI accepted");let text=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!text.contains("division by zero"));put(&root.join("later-reference-before-publication.log"),text.as_bytes());let mut request=request;request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("later-reference-native");let e=store.executable(&request,generation,backend,&out).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");assert!(!out.join("app").exists());
}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation){
    let raw=std::fs::read(root.join("base.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let refs=borrowed_nominal_qualification::reference_wires(original);let kept=refs.iter().find(|r|!r.4.is_empty()).expect("actual kept recursive reference");
    let reject=|name:&str,mut b:Vec<u8>,expected:&str|{b.extend_from_slice(&native_crate::digest(&b).unwrap());put(&root.join(format!("malformed-{name}.lustr-native")),&b);let e=native_crate::Artifact::decode(&b,generation).err().expect("malformed recursive reference accepted");assert!(e.contains(expected),"{name}: {e}");put(&root.join(format!("malformed-{name}.log")),e.as_bytes());};
    let mut b=original.to_vec();b[kept.2]^=1;reject("foreign-target",b,"reference target is absent");
    let mut b=original.to_vec();b[kept.2+8]^=1;reject("unknown-target",b,"reference target is absent");
    let mut b=original.to_vec();b[kept.1]=2;reject("mutable-tag",b,"invalid native boolean");
    let mut b=original.to_vec();assert_eq!(b[kept.4[0]],2);b[kept.4[0]+1..kept.4[0]+5].copy_from_slice(&63u32.to_le_bytes());reject("unbound-application",b,"field region declaration");
    let mut b=original.to_vec();b[kept.3..kept.3+4].copy_from_slice(&65u32.to_le_bytes());reject("argument-bound",b,"field exceeds bound");
    let mut b=original.to_vec();b[kept.3..kept.3+4].copy_from_slice(&0u32.to_le_bytes());let at=kept.4[0];b.drain(at..at+5);let size=u32::from_le_bytes(b[kept.6..kept.6+4].try_into().unwrap());b[kept.6..kept.6+4].copy_from_slice(&(size-5).to_le_bytes());reject("missing-application",b,"kept nominal reference requires");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=11;reject("old-format11",b,"format generation");
}
