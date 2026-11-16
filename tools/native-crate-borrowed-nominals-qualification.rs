//! Lifetime-bearing nominal definitions through the existing native checker.
use super::*;
pub(super) const ORIGINAL:&str="#![no_std]\npub struct View<'a>{pub value:&'a i64}\npub fn make<'a>(x:&'a i64)->View<'a>{View{value:x}}\npub fn get<'a>(x:View<'a>)->&'a i64{x.value}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"borrowed_original"),generation,backend);put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());
    let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("liboriginal.rlib")).output().unwrap();record(root,"original.oracle-build",&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));
    println!("original lifetime-bearing nominal native refusal/no image/no apps; separate explicit SDK oracle rlib build passes");
}
#[derive(Default)]
struct Wire{tag:u8,tag_at:usize,type_size_at:usize,item_start:usize,item_end:usize,name:String,check:Vec<usize>,kept:Vec<usize>,names:Vec<(usize,usize)>,base:Option<usize>,regions:Vec<usize>,abi:usize,enum_offsets:Vec<usize>,is_enum:bool,fact_count:Option<usize>,facts:Vec<usize>,reference:Option<(usize,usize,usize,usize)>}
fn number(b:&[u8],p:&mut usize)->usize{let n=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;n}
fn text(b:&[u8],p:&mut usize)->(usize,usize){let n=number(b,p);let start=*p;*p+=n;(start,n)}
fn region(b:&[u8],p:&mut usize){let tag=b[*p];*p+=1;if tag==2{*p+=4;}}
fn vis(b:&[u8],p:&mut usize){let tag=b[*p];*p+=1;if tag==4{for _ in 0..number(b,p){text(b,p);}}}
fn wires(b:&[u8])->(Vec<Wire>,Vec<usize>){
    let mut p=b"LUSTR-NATIVE-CRATE\0\x0b".len()+64;for _ in 0..4{text(b,&mut p);}p+=8+32+2;
    for _ in 0..number(b,&mut p){p+=16;text(b,&mut p);p+=1;let n=number(b,&mut p);p+=4*(n+1);text(b,&mut p);}
    let type_size_at=p;let size=number(b,&mut p);let end=p+size;let count=number(b,&mut p);let mut nodes:Vec<Wire>=Vec::new();
    for _ in 0..count{let mut w=Wire{tag:b[p],tag_at:p,type_size_at,..Default::default()};p+=1;
        match w.tag{
            0..=17=>{},18=>{let n=number(b,&mut p);p+=4*n;},19=>p+=12,20=>{let n=number(b,&mut p);p+=4*n+9;},21=>{region(b,&mut p);p+=5;},
            22..=24=>{p+=16;let(s,n)=text(b,&mut p);w.name=String::from_utf8(b[s..s+n].to_vec()).unwrap();let variants=if w.tag==22{1}else{w.is_enum=b[p]==1;p+=1;number(b,&mut p)};
                for _ in 0..variants{if w.tag!=22{text(b,&mut p);p+=17;}for _ in 0..number(b,&mut p){text(b,&mut p);w.check.push(p);p+=4;vis(b,&mut p);}}
                if w.tag==24{w.item_start=p;for _ in 0..number(b,&mut p){w.names.push(text(b,&mut p));}for _ in 0..w.check.len(){w.kept.push(p);p+=4;}w.fact_count=Some(p);for _ in 0..number(b,&mut p){w.facts.push(p);p+=8;}w.item_end=p;}
            },
            25=>{w.base=Some(p);let base=number(b,&mut p);w.is_enum=nodes[base].is_enum;for _ in 0..number(b,&mut p){w.regions.push(p);region(b,&mut p);}},
            26=>{let r=p;region(b,&mut p);let m=p;p+=1;let target=p;p+=16;let count=p;for _ in 0..number(b,&mut p){w.regions.push(p);region(b,&mut p);}w.reference=Some((r,m,target,count));},
            _=>panic!("fixture type tag"),
        }
        p+=16;for _ in 0..number(b,&mut p){p+=8;}let niche=b[p];p+=1;if niche!=0{p+=48;}p+=3;w.abi=p+16;p+=19;
        if w.is_enum{for _ in 0..number(b,&mut p){for _ in 0..number(b,&mut p){w.enum_offsets.push(p);p+=8;}}let tag=b[p];p+=1;match tag{0=>{},1=>p+=4,2=>p+=9,3=>p+=76,_=>panic!("fixture enum tag")}}
        nodes.push(w);
    }assert_eq!(p,end);let size=number(b,&mut p);p+=size;let size=number(b,&mut p);let end=p+size;let mut sigs=Vec::new();
    for _ in 0..number(b,&mut p){p+=4;for _ in 0..number(b,&mut p){sigs.push(p);p+=4;}sigs.push(p);p+=4;for _ in 0..number(b,&mut p){region(b,&mut p);region(b,&mut p);}for _ in 0..number(b,&mut p){text(b,&mut p);}let n=number(b,&mut p);p+=n;for _ in 0..number(b,&mut p){text(b,&mut p);}}
    assert_eq!(p,end);(nodes,sigs)
}
fn at(b:&[u8],p:usize)->usize{u32::from_le_bytes(b[p..p+4].try_into().unwrap())as usize}
pub(super) fn fact_wires(b:&[u8])->Vec<(String,usize,Vec<usize>,usize,usize)>{wires(b).0.into_iter().filter_map(|w|Some((w.name,w.fact_count?,w.facts,w.abi,w.type_size_at))).collect()}
pub(super) fn reference_wires(b:&[u8])->Vec<(usize,usize,usize,usize,Vec<usize>,usize,usize)>{wires(b).0.into_iter().filter_map(|w|{let(r,m,t,c)=w.reference?;Some((r,m,t,c,w.regions,w.abi,w.type_size_at))}).collect()}
fn replace(b:&mut[u8],p:usize,n:usize){b[p..p+4].copy_from_slice(&(n as u32).to_le_bytes());}
fn checksum(mut b:Vec<u8>)->Vec<u8>{b.extend_from_slice(&native_crate::digest(&b).unwrap());b}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    let reject=|label:&str,b:Vec<u8>,expected:&str|{let b=checksum(b);put(&root.join(format!("malformed-{label}.lustr-native")),&b);let error=native_crate::Artifact::decode(&b,generation).err().expect("malformed borrowed metadata accepted");put(&root.join(format!("malformed-{label}.log")),error.as_bytes());assert!(error.contains(expected),"{label}: {error}");};
    let bytes=std::fs::read(root.join("shapes.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];let(nodes,sigs)=wires(original);let pair=nodes.iter().find(|w|w.name=="Pair").unwrap();let nested=nodes.iter().find(|w|w.name=="Nested").unwrap();let app=nodes.iter().find(|w|w.tag==25).unwrap();
    let mut b=original.to_vec();b[pair.names[0].0]=b'_';reject("reserved-name",b,"region declaration");
    let mut b=original.to_vec();b[pair.names[1].0]=b[pair.names[0].0];reject("duplicate-name",b,"region declaration");
    let mut b=original.to_vec();replace(&mut b,pair.kept[0],at(original,pair.check[0]));reject("erased-kept-field",b,"field region declaration");
    let mut b=original.to_vec();let kept=at(original,nested.kept[0]);let base=at(original,nodes[kept].base.unwrap());replace(&mut b,nested.kept[0],base);reject("missing-nested-application",b,"kept nominal requires");
    let mut b=original.to_vec();let outer=nodes.iter().find(|w|w.name=="StaticOuter").unwrap();assert!(outer.names.is_empty());b[outer.tag_at]=23;let removed=outer.item_end-outer.item_start;b.drain(outer.item_start..outer.item_end);replace(&mut b,outer.type_size_at,at(original,outer.type_size_at)-removed);reject("missing-complete-nested-kept-record",b,"kept nominal requires");
    let mut b=original.to_vec();let root_id=sigs.iter().copied().find(|&p|nodes[at(original,p)].tag==25).unwrap();let base=at(original,nodes[at(original,root_id)].base.unwrap());replace(&mut b,root_id,base);reject("missing-signature-application",b,"kept nominal requires");
    let mut b=original.to_vec();let index=nodes.iter().position(|w|std::ptr::eq(w,app)).unwrap();replace(&mut b,app.base.unwrap(),index);reject("forward-application",b,"earlier definition");
    let mut b=original.to_vec();let scalar=nodes.iter().position(|w|w.tag<=17).unwrap();replace(&mut b,app.base.unwrap(),scalar);reject("scalar-application",b,"earlier definition");
    let mut b=original.to_vec();b[app.regions[0]]=0;reject("erased-application-region",b,"");
    let mut b=original.to_vec();assert_eq!(b[app.regions[0]],2);replace(&mut b,app.regions[0]+1,63);reject("unbound-application-region",b,"");
    let mut b=original.to_vec();let enum_app=nodes.iter().find(|w|w.tag==25&&w.is_enum&&!w.enum_offsets.is_empty()).unwrap();let p=enum_app.enum_offsets[0];b[p]^=1;reject("applied-enum-layout",b,"layout/Copy/drop mismatch");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=10;reject("old-format10",b,"format generation");
    // Keep ABI corruption structurally valid and sort it after a healthy
    // image. No semantic payload, type name or initializer may be published.
    let mut images:Vec<_>=["base","other","shapes"].iter().map(|name|{let b=std::fs::read(root.join(format!("{name}.lustr-native"))).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();(a.functions()[0].id.krate,b)}).collect();images.sort_by_key(|x|x.0);let healthy=native_crate::Artifact::decode(&images[0].1,generation).unwrap();let raw=&images[2].1;let(w,_)=wires(raw);let mut bad=raw[..raw.len()-32].to_vec();bad[w[0].abi]^=1;let bad=checksum(bad);put(&root.join("later-bad-abi.lustr-native"),&bad);let bad=native_crate::Artifact::decode(&bad,generation).unwrap();let store=native_crate::CrateStore::new(vec![("good".into(),healthy),("bad".into(),bad)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().filter(|d|d.krate!=def::LOCAL_CRATE).copied().collect();let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let e=store.install_signatures(p).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");for d in &foreign{assert!(p.defs.get(*d).is_none());assert!(!p.type_names.contains_key(&tables.def_info[d].flat));}assert!(p.regions.native_fns.is_empty());assert!(p.consts.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),e,"Expected later-image ABI refusal")])};let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("later ABI must stop collection");let rendered=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!rendered.contains("division by zero"));put(&root.join("later-abi-before-definitions.log"),rendered.as_bytes());
    let mut request=request;request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("later-bad-abi-native");let e=store.executable(&request,generation,backend,&out).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");assert!(!out.join("app").exists());put(&root.join("later-abi-executable.log"),e.as_bytes());
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;
    let a=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let stable=a.functions()[0].id.krate;
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("perturbed-borrowed-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(vec![("base".into(),a)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("borrowed_noise_{n}"));}
    let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local<'x>{v:&'x i64}fn local(x:&i64)->&i64{x}fn main(){}",ids);
    let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect errors {}",e.len()));
    let fns=p.fns.len();let region_fns=p.regions.fns.len();let adts=p.adts.len();for n in 101..133{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}
    store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),region_fns);assert_eq!(p.adts.len(),adts+1);
    let def=p.def_of_adt(ty::AdtId(adts as u32));assert_eq!(def.hash,def::item_hash(stable,&[],def::DefKind::Adt,"View",0));
    assert_eq!(p.gx.adt_lifetimes[&def].iter().map(|&s|p.name(s)).collect::<Vec<_>>(),["a"]);
    let i64_ty=p.tys.intern(ty::TyKind::Int(ty::IntTy::I64));let kept=p.regions.adt_fields[&def][0][0];assert_eq!(p.tys.kind(kept),ty::TyKind::Ref(ty::Region::Param(0),ty::Mut::Shared,i64_ty));
    let erased=p.adts[adts].variants[0].fields[0].ty;assert_eq!(p.tys.kind(erased),ty::TyKind::Ref(ty::Region::Erased,ty::Mut::Shared,i64_ty));
    put(&root.join("remap-install-only.txt"),b"Actual lifetime declaration/kept field/erased shape and stable identity retain exact relationships after fresh interner and existing local ADT/type/region tables. No new FnInfo/local function-region rows. INSTALL ONLY, no app execution from perturbed Program.\n");
}
const SHAPES:&str=r#"#![no_std]
pub struct Pair<'a,'b>{pub left:&'a i64,pub right:&'b i64}
pub struct MutView<'a>{pub value:&'a mut i64}
pub struct Nested<'a>{pub pair:Pair<'a,'a>,pub anchor:&'a i64}
pub enum Entry<'a>{None,Some(&'a i64)}
pub enum Tagged<'a>{Left(&'a i64),Right(&'a i64)}
pub struct StaticOuter(pub [Pair<'static,'static>;1]);
pub fn left<'a,'b>(p:Pair<'a,'b>)->&'a i64{p.left}
pub fn right<'a,'b>(p:Pair<'a,'b>)->&'b i64{p.right}
pub fn bump<'a>(p:MutView<'a>)->&'a mut i64{*p.value+=1;p.value}
pub fn nested<'a>(p:Nested<'a>)->&'a i64{p.pair.left}
pub fn entry<'a>(e:Entry<'a>)->i64{match e{Entry::None=>0,Entry::Some(p)=>*p}}
pub fn tagged<'a>(e:Tagged<'a>)->i64{match e{Tagged::Left(p)=>*p,Tagged::Right(p)=>*p+1}}
"#;
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("base",ORIGINAL,"borrowed_original"),("other",ORIGINAL,"borrowed_independent"),("shapes",SHAPES,"borrowed_shapes")]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");
        let bytes=std::fs::read(image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);images.push((label.into(),artifact));
        let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-producer"),&o);assert!(o.status.success(),"{label}: {}",String::from_utf8_lossy(&o.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("borrowed-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid)in[
        ("direct","use base::{View,make,get};fn main(){let x=17;assert!(*get(make(&x))==17);let p=View{value:&x};assert!(*get(p)==17);}",true),
        ("named-local","use base::{View,make,get};fn keep<'a>(x:View<'a>)->View<'a>{x}fn main(){let x=23;assert!(*get(keep(make(&x)))==23);}",true),
        ("independent","use base::{make,get};use other::{make as om,get as og};fn main(){let x=9;assert!(*get(make(&x))==9);assert!(*og(om(&x))==9);}",true),
        ("two-regions","use shapes::{Pair,left,right,Nested,nested};fn main(){let a=31;let b=41;assert!(*left(Pair{left:&a,right:&b})==31);assert!(*right(Pair{left:&a,right:&b})==41);assert!(*nested(Nested{pair:Pair{left:&a,right:&a},anchor:&a})==31);}",true),
        ("mutable","use shapes::{MutView,bump};fn main(){let mut a=7;{let r=bump(MutView{value:&mut a});*r+=2;}assert!(a==10);}",true),
        ("enum","use shapes::{Entry,entry,Tagged,tagged};fn main(){let x=13;assert!(entry(Entry::Some(&x))==13);assert!(entry(Entry::None)==0);assert!(tagged(Tagged::Left(&x))==13);assert!(tagged(Tagged::Right(&x))==14);}",true),
        ("escape","use base::{make,get};fn main(){let r;{let x=3;r=get(make(&x));}let y=*r;}",false),
        ("static","use base::{View,get};fn leak<'a>(p:View<'a>)->&'static i64{get(p)}fn main(){}",false),
        ("two-escape","use shapes::{Pair,right};fn main(){let x=3;let r;{let y=7;r=right(Pair{left:&x,right:&y});}let z=*r;}",false),
        ("mut-alias","use shapes::{MutView,bump};fn main(){let mut x=1;let r=bump(MutView{value:&mut x});let s=bump(MutView{value:&mut x});*r+=*s;}",false),
        ("moved","use base::{make,get};fn main(){let x=3;let p=make(&x);get(p);get(p);}",false),
        ("identity","use base::get;use other::make;fn main(){let x=3;get(make(&x));}",false),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{assert!(result.is_err(),"invalid {label} emitted output; NEVER EXECUTE");assert!(!out.join("app").exists());}
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in["base","other","shapes"]{command.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let app=root.join(format!("{label}-oracle"));let oracle=command.arg("-o").arg(&app).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert_eq!(oracle.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
        if valid{let n=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success());let o=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    for(label,text)in[
        ("bad-producer-escape","#![no_std]\npub struct View<'a>{pub value:&'a i64}\npub fn fail<'a>()->View<'a>{let x=3;View{value:&x}}\n"),
        ("bad-producer-region","#![no_std]\npub struct Pair<'a,'b>{pub a:&'a i64,pub b:&'b i64}\npub fn fail<'a,'b>(p:Pair<'a,'b>)->&'a i64{p.b}\n"),
    ]{let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,label.replace('-',"_").as_str()),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());assert_eq!(code,1,"invalid producer accepted: {label}; NEVER CONSUME/EXECUTE");assert!(!image.exists());let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-o"]).arg(root.join(format!("{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-producer"),&oracle);assert!(!oracle.status.success());}
    malformed(root,generation,backend,ids);remap(root,generation);
    println!("native lifetime-bearing nominal gate PASS; fullRust/no_std/Cargo/SAMEcompletework remain OPEN");
}
