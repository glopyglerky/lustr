//! Real nominal definition frontier in the native crate image.
use super::*;
pub(super) const ORIGINAL:&str="#![no_std]\npub struct Packet {pub value:i64, hidden:bool}\npub fn make(x:i64)->Packet{Packet{value:x,hidden:true}}\npub fn consume(x:Packet)->i64{if x.hidden{x.value+1}else{0}}\n";
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"nominal_original"),generation,backend);
    put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());
    assert_eq!(code,1,"{message}");assert!(!image.exists());
    println!("original unchanged nominal producer native refusal retained; no image/app");
}

const WIDE:&str="#![no_std]\npub struct Packet {pub data:[i64;5]}\npub struct Holder {pub packet:Packet,pub ready:bool}\npub fn make(x:i64)->Packet{Packet{data:[x,2,3,4,5]}}\npub fn sum(x:Packet)->i64{x.data[0]+x.data[1]+x.data[2]+x.data[3]+x.data[4]}\npub fn wrap(x:Packet)->Holder{Holder{packet:x,ready:true}}\npub fn unwrap(x:Holder)->Packet{x.packet}\npub fn borrow(x:&Packet)->&Packet{x}\n";
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    qualify(root,generation,backend,true)
}
// A separate partial gate for the original Packet and an independently
// identified identical-layout producer. This partial gate cannot qualify the
// unchanged WIDE case; its retained first failure needs the complete run gate.
pub(super) fn original_gate(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    qualify(root,generation,backend,false)
}
fn qualify(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,wide:bool){
    let mut images=Vec::new();
    for(name,text,metadata)in[("base",ORIGINAL,"nominal_original"),("other",ORIGINAL,"nominal_independent"),("wide",WIDE,"nominal_wide")]{
        if name=="wide"&&!wide{continue;}
        let source=root.join(format!("{name}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{name}.lustr-native"));
        let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);record_text(root,&format!("{name}.native"),code,&message);assert_eq!(code,0,"{name}: {message}");
        let bytes=std::fs::read(&image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);images.push((name.into(),artifact));
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{name}.rlib"))).output().unwrap();record(root,&format!("{name}.oracle-build"),&oracle);assert!(oracle.status.success());
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("nominal-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid,diagnosis)in[
        ("direct","use base::{make,consume};fn main(){let p=make(7);assert!(p.value==7);assert!(consume(p)==8);}",true,""),
        ("pointer","use base::{Packet,make,consume};fn main(){let f:fn(Packet)->i64=consume;let p=make(9);assert!(f(p)==10);}",true,""),
        ("independent","use base::{make,consume};use other::{make as make_other,consume as consume_other};fn main(){let p=make(3);let q=make_other(7);assert!(consume(p)==4);assert!(consume_other(q)==8);}",true,""),
        ("large-nested","use wide::{Packet,make,sum,wrap,unwrap};fn local(x:Packet)->Packet{x}fn main(){let p=local(make(8));let h=wrap(p);assert!(h.ready);let q=unwrap(h);let f:fn(Packet)->i64=sum;assert!(f(q)==22);}",true,""),
        ("borrow","use wide::{make,borrow,sum};fn main(){let p=make(6);let r=borrow(&p);assert!(r.data[0]==6);assert!(sum(p)==20);}",true,""),
        ("moved","use base::{make,consume};fn main(){let p=make(3);consume(p);consume(p);}",false,"L0301"),
        ("private","use base::make;fn main(){let p=make(3);let b=p.hidden;}",false,"private"),
        ("private-construct","use base::Packet;fn main(){let p=Packet{value:3,hidden:true};}",false,"private"),
        ("distinct","use base::consume;use other::make;fn main(){consume(make(3));}",false,"L0110"),
        ("escape","use wide::{make,borrow};fn main(){let r;{let p=make(3);r=borrow(&p);}let x=r.data[0];}",false,"L0306"),
    ]{
        if !wide&&matches!(label,"large-nested"|"borrow"|"escape"){continue;}
        let file=root.join(format!("{label}.rs"));put(&file,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());
        if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let error=result.err().unwrap_or_else(||panic!("invalid {label} produced native output; NEVER execute"));assert!(!out.join("app").exists());assert!(error.contains(diagnosis),"{label}: expected {diagnosis}: {error}");}
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(&file).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in ["base","other","wide"]{if name=="wide"&&!wide{continue;}command.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let app=root.join(format!("{label}-oracle"));let oracle=command.arg("-o").arg(&app).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert_eq!(oracle.status.success(),valid);
        if valid{let native=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&native);assert!(native.status.success());let oracle=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&oracle);assert!(oracle.status.success());assert_eq!(native.stdout,oracle.stdout);assert_eq!(native.stderr,oracle.stderr);}
    }
    malformed(root,generation,backend,ids);remap(root,generation);namespace_malformed(root,generation,backend);
    println!("native nominal gate PASS (wide={wide}); full Rust/Cargo/no_std/SAME-work speed remain OPEN; host runtime composition is not no_std runtime acceptance");
}
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
fn record_text(root:&std::path::Path,label:&str,code:i32,text:&str){put(&root.join(format!("{label}.log")),text.as_bytes());put(&root.join(format!("{label}.status")),code.to_string().as_bytes());}
fn namespace_malformed(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    fn n(b:&[u8],p:&mut usize)->usize{let v=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;v}
    fn text(b:&[u8],p:&mut usize){let v=n(b,p);*p+=v;}
    fn path(b:&[u8],p:&mut usize){for _ in 0..n(b,p){text(b,p);}}
    fn vis(b:&[u8],p:&mut usize){let v=b[*p];*p+=1;if v==4{path(b,p);}}
    fn put_text(b:&mut Vec<u8>,s:&str){b.extend_from_slice(&(s.len()as u32).to_le_bytes());b.extend_from_slice(s.as_bytes());}
    fn namespace(b:&[u8])->(usize,usize,usize,Vec<(std::ops::Range<usize>,Vec<u8>,usize,u8)>,usize,Vec<(std::ops::Range<usize>,Vec<u8>,usize)>){
        let mut p=b"LUSTR-NATIVE-CRATE\0\x06".len()+64;for _ in 0..4{text(b,&mut p);}p+=8+32+2;for _ in 0..n(b,&mut p){p+=16;text(b,&mut p);p+=1;let len=n(b,&mut p);p+=4*(len+1);text(b,&mut p);}let len=n(b,&mut p);p+=len;let size_at=p;let size=n(b,&mut p);let end=p+size;
        for _ in 0..n(b,&mut p){path(b,&mut p);let some=b[p];p+=1;if some==1{p+=16;}vis(b,&mut p);}
        let count_at=p;let mut defs=Vec::new();for _ in 0..n(b,&mut p){let start=p;let id=b[p..p+16].to_vec();p+=16;text(b,&mut p);path(b,&mut p);let visibility=p;vis(b,&mut p);let kind=b[p];p+=1;defs.push((start..p,id,visibility,kind));}
        let binds_at=p;let mut binds=Vec::new();for _ in 0..n(b,&mut p){let start=p;path(b,&mut p);p+=1;text(b,&mut p);assert_eq!(b[p],0,"test fixture has definition bindings only");p+=1;let id=b[p..p+16].to_vec();p+=16;let visibility=p;vis(b,&mut p);path(b,&mut p);binds.push((start..p,id,visibility));}assert_eq!(p,end);(size_at,end,count_at,defs,binds_at,binds)
    }
    fn reject(root:&std::path::Path,label:&str,mut payload:Vec<u8>,generation:&native_crate::Generation,expected:&str){payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&payload);let e=native_crate::Artifact::decode(&payload,generation).err().expect("malformed namespace accepted");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());assert!(e.contains(expected),"{label}: {e}");}
    let source=root.join("namespace-private.rs");put(&source,b"#![no_std]\nfn private(x:i64)->i64{x+1}\npub fn value(x:i64)->i64{private(x)}\n");let image=root.join("namespace-private.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"nominal_namespace"),generation,backend);record_text(root,"namespace-private.native",code,&message);assert_eq!(code,0,"{message}");let bytes=std::fs::read(image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();let private=artifact.functions().iter().find(|f|!f.public).unwrap().id.to_bytes().to_vec();let original=&bytes[..bytes.len()-32];let(size_at,end,count_at,defs,binds_at,binds)=namespace(original);
    let mut payload=original.to_vec();let d=defs.iter().find(|d|d.1==private).unwrap();assert_eq!(payload[d.2],0);payload[d.2]=1;let mut changed=0;for b in &binds{if b.1==private{assert_eq!(payload[b.2],0);payload[b.2]=1;changed+=1;}}assert!(changed>0);reject(root,"private-function-public-namespace",payload,generation,"definition/signature mismatch");
    let stable=artifact.functions()[0].id.krate;let ghost=def::DiskDefId{krate:stable,hash:def::item_hash(stable,&[],def::DefKind::Adt,"Ghost",0)};let mut definition=ghost.to_bytes().to_vec();put_text(&mut definition,"Ghost");definition.extend_from_slice(&0u32.to_le_bytes());definition.extend_from_slice(&[1,2]);let mut binding=0u32.to_le_bytes().to_vec();binding.push(0);put_text(&mut binding,"Ghost");binding.push(0);binding.extend_from_slice(&ghost.to_bytes());binding.push(1);binding.extend_from_slice(&0u32.to_le_bytes());
    let mut new_ns=original[size_at+4..count_at].to_vec();new_ns.extend_from_slice(&((defs.len()+1)as u32).to_le_bytes());new_ns.extend_from_slice(&original[count_at+4..binds_at]);new_ns.extend_from_slice(&definition);new_ns.extend_from_slice(&((binds.len()+1)as u32).to_le_bytes());new_ns.extend_from_slice(&original[binds_at+4..end]);new_ns.extend_from_slice(&binding);let mut payload=original[..size_at].to_vec();payload.extend_from_slice(&(new_ns.len()as u32).to_le_bytes());payload.extend_from_slice(&new_ns);payload.extend_from_slice(&original[end..]);reject(root,"ghost-nominal-without-shape",payload,generation,"nominal definition/shape mismatch");
    let bytes=std::fs::read(root.join("base.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];let(size_at,end,count_at,defs,binds_at,binds)=namespace(original);let missing=&defs.iter().find(|d|d.3==2).unwrap().1;let kept_defs:Vec<_>=defs.iter().filter(|d|&d.1!=missing).collect();let kept_binds:Vec<_>=binds.iter().filter(|b|&b.1!=missing).collect();let mut new_ns=original[size_at+4..count_at].to_vec();new_ns.extend_from_slice(&(kept_defs.len()as u32).to_le_bytes());for d in kept_defs{new_ns.extend_from_slice(&original[d.0.clone()]);}new_ns.extend_from_slice(&(kept_binds.len()as u32).to_le_bytes());for b in kept_binds{new_ns.extend_from_slice(&original[b.0.clone()]);}let mut payload=original[..size_at].to_vec();payload.extend_from_slice(&(new_ns.len()as u32).to_le_bytes());payload.extend_from_slice(&new_ns);payload.extend_from_slice(&original[end..]);reject(root,"nominal-without-namespace",payload,generation,"missing nominal definition");let _=binds_at;
}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    fn number(b:&[u8],p:&mut usize)->u32{let n=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap());*p+=4;n}
    fn text(b:&[u8],p:&mut usize)->(usize,usize){let n=number(b,p)as usize;let start=*p;*p+=n;(start,n)}
    fn region(b:&[u8],p:&mut usize){let tag=b[*p];*p+=1;if tag==2{*p+=4;}}
    fn visibility(b:&[u8],p:&mut usize){let tag=b[*p];*p+=1;if tag==4{for _ in 0..number(b,p){text(b,p);}}}
    let bytes=std::fs::read(root.join("base.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];
    let mut at=b"LUSTR-NATIVE-CRATE\0\x06".len()+64;for _ in 0..4{text(original,&mut at);}at+=8+32+2;
    for _ in 0..number(original,&mut at){at+=16;text(original,&mut at);at+=1;let n=number(original,&mut at);at+=4*(n as usize+1);text(original,&mut at);}
    let type_size_at=at;let size=number(original,&mut at)as usize;let end=at+size;let nodes=number(original,&mut at);
    let mut nominal=None;
    for node in 0..nodes{
        let tag=original[at];at+=1;let mut record=None;
        match tag{
            0..=17=>{},18=>{let n=number(original,&mut at);at+=4*n as usize;},19=>at+=12,
            20=>{let n=number(original,&mut at);at+=4*n as usize+9;},21=>{region(original,&mut at);at+=5;},
            22=>{let identity=at;at+=16;text(original,&mut at);let n=number(original,&mut at);let mut fields=Vec::new();for _ in 0..n{let name=text(original,&mut at);let ty=at;at+=4;let vis=at;visibility(original,&mut at);fields.push((name,ty,vis));}record=Some((node,identity,fields));},
            _=>panic!("unexpected tested native graph tag"),
        }
        let layout=at;at+=16;let n=number(original,&mut at);at+=8*n as usize;let niche=original[at];at+=1;if niche==1{at+=48;}
        at+=1;let copy=at;let drop=at+1;at+=2;let abi=at;at+=19;
        if let Some((node,identity,fields))=record{nominal=Some((node,identity,fields,layout,copy,drop,abi));}
    }
    assert_eq!(at,end);let(node,identity,fields,layout,copy,drop,abi)=nominal.unwrap();assert_eq!(fields.len(),2);
    let mut mutations=Vec::<(&str,Vec<u8>,&str)>::new();
    let mut b=original.to_vec();b[identity]^=1;mutations.push(("nominal-identity",b,"nominal"));
    let mut b=original.to_vec();b[fields[0].1..fields[0].1+4].copy_from_slice(&node.to_le_bytes());mutations.push(("nominal-cycle",b,"forward/cyclic"));
    let n=fields[0].0.1;let old=fields[1].0.1;let mut b=original[..fields[1].0.0-4].to_vec();b.extend_from_slice(&(n as u32).to_le_bytes());b.extend_from_slice(&original[fields[0].0.0..fields[0].0.0+n]);b.extend_from_slice(&original[fields[1].0.0+old..]);b[type_size_at..type_size_at+4].copy_from_slice(&((size-old+n)as u32).to_le_bytes());mutations.push(("duplicate-field",b,"field identity"));
    let mut b=original.to_vec();b[fields[0].2]=255;mutations.push(("field-visibility",b,"visibility tag"));
    let mut b=original.to_vec();b[layout]^=1;mutations.push(("nominal-layout",b,"layout/Copy/drop"));
    let mut b=original.to_vec();b[copy]=1;mutations.push(("nominal-copy",b,"layout/Copy/drop"));
    let mut b=original.to_vec();b[drop]=1;mutations.push(("nominal-drop",b,"layout/Copy/drop"));
    for(label,mut payload,diagnosis)in mutations{payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&payload);let error=native_crate::Artifact::decode(&payload,generation).err().expect("malformed nominal image accepted");put(&root.join(format!("malformed-{label}.log")),error.as_bytes());assert!(error.contains(diagnosis),"{label}: {error}");}
    // Effective ABI is checked in the actual C naming context, after all
    // shapes exist but before publishing even an earlier valid image's defs.
    let other=std::fs::read(root.join("other.lustr-native")).unwrap();let base_artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();let other_artifact=native_crate::Artifact::decode(&other,generation).unwrap();
    fn metadata_len(b:&[u8])->usize{let mut p=b"LUSTR-NATIVE-CRATE\0\x06".len()+64;for _ in 0..3{text(b,&mut p);}number(b,&mut p)as usize}
    let (target,good,bad_stable)=if base_artifact.functions()[0].id.krate<other_artifact.functions()[0].id.krate{(&other[..other.len()-32],base_artifact,other_artifact.functions()[0].id.krate)}else{(original,other_artifact,base_artifact.functions()[0].id.krate)};
    assert!(good.functions()[0].id.krate<bad_stable,"bad image must sort AFTER a valid image");let offset=abi.checked_add_signed(metadata_len(target)as isize-metadata_len(original)as isize).unwrap();assert_eq!(target[offset+17],original[abi+17]);let mut payload=target.to_vec();payload[offset+17]^=1;payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join("malformed-nominal-abi.lustr-native"),&payload);
    let bad=native_crate::Artifact::decode(&payload,generation).unwrap();
    let store=native_crate::CrateStore::new(vec![("bad".into(),bad),("good".into(),good)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().filter(|d|d.krate!=def::LOCAL_CRATE).copied().collect();assert!(!foreign.is_empty());let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let error=store.install_signatures(p).expect_err("malformed ABI installed");assert!(error.contains("C ABI mismatch"),"{error}");for &d in &foreign{assert!(p.defs.get(d).is_none(),"foreign DefData before ALLimage ABI validation");let flat=tables.def_info[&d].flat;assert!(!p.type_names.contains_key(&flat));}assert!(p.regions.native_fns.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),error,"Expected pre-definition ABI refusal")])};
    let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("ABI refusal must stop collection");let rendered=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!rendered.contains("division by zero"));put(&root.join("malformed-nominal-abi-before-definitions.log"),rendered.as_bytes());
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("bad_abi_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local{v:i64}fn local(x:&i64)->&i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("bad ABI perturbed collect {}",e.len()));let fns=p.fns.len();let locals=p.regions.fns.len();for n in 101..133{use ty::TyIntern;let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}let error=store.install_signatures(&mut p).unwrap_err();assert!(error.contains("C ABI mismatch"));assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),locals);assert!(p.regions.native_fns.is_empty());for &d in &foreign{assert!(p.defs.get(d).is_none());assert!(!p.type_names.contains_key(&p.resolved.as_ref().unwrap().def_info[&d].flat));}put(&root.join("bad-later-abi-perturbed-install-only.txt"),b"Actual later-image ABI refusal with existing local Adt/type/region rows leaves all foreign definitions/type names/signatures unpublished. INSTALL ONLY; no application execution from this Program.\n");
    let mut request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("malformed-abi-native");let error=store.executable(&request,generation,backend,&out).unwrap_err();assert!(error.contains("C ABI mismatch"));assert!(!out.exists());put(&root.join("malformed-nominal-abi-executable.log"),error.as_bytes());
}

fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;
    let artifact=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("perturbed-nominal-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("base".into(),artifact)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("nominal_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local{v:i64}fn local(x:&i64)->&i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect errors {}",e.len()));let fns=p.fns.len();let regions=p.regions.fns.len();let adts=p.adts.len();for n in 101..133{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}
    store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),regions);assert_eq!(p.adts.len(),adts+1);let foreign=p.adts.iter().enumerate().find(|(n,_)|p.def_of_adt(ty::AdtId(*n as u32)).krate!=def::LOCAL_CRATE&&p.def_of_adt(ty::AdtId(*n as u32)).krate!=ty::BUILTIN_CRATE).unwrap();assert_eq!(foreign.0,adts);let def=p.def_of_adt(ty::AdtId(adts as u32));let t=p.tys.get_adt_ty(ty::AdtId(adts as u32)).unwrap();assert_eq!(p.tys.adt_parts(t).unwrap().0,def);assert_eq!(p.defs.get(def).unwrap().kind,ty::defs::DefKind::Struct);assert!(!check::collect::is_copy(&p,&p.tys,t));assert!(!check::collect::needs_drop(&p,&p.tys,t));put(&root.join("remap-install-only.txt"),b"Fresh interner/prepopulated native types/local Adt/local region rows preserved; no FnInfo/local region fabricated. INSTALL ONLY; no execution from this Program.\n");
}
