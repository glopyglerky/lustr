//! Lifetime-bound declarations through the existing native region engine.
use super::*;
pub(super) const ORIGINAL:&str="#![no_std]\npub fn shorten<'a:'b,'b>(x:&'a i64,y:&'b i64)->&'b i64{let _=y;x}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),o.status.to_string().as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"bounded_fn_original"),generation,backend);put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());
    let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("liboriginal.rlib")).output().unwrap();record(root,"original.oracle-build",&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));println!("unchanged lifetime-bound producer native refusal/no image/no apps; separate explicit SDK oracle rlib0");
}
const PROVIDER:&str=r#"#![no_std]
pub fn where_bound<'a,'b>(x:&'a i64,y:&'b i64)->&'b i64 where 'a:'b {let _=y;x}
pub fn transitive<'a:'b,'b:'c,'c>(x:&'a i64,y:&'b i64,z:&'c i64)->&'c i64{let _=(y,z);x}
pub fn self_bound<'a:'a>(x:&'a i64)->&'a i64{x}
pub fn static_bound<'a:'static>(x:&'a i64)->&'static i64{x}
pub fn unused<'a>(x:i64)->i64{x+1}
pub fn unicode<'é>(x:&'é i64)->&'é i64{x}
pub fn mixed<'a:'a>(x:&'a i64,y:&i64)->&'a i64{let _=y;x}
pub fn cycle<'a:'b,'b:'a>(x:&'a i64,y:&'b i64)->&'b i64{let _=y;x}
pub struct View<'a>{pub value:&'a i64}
pub fn view<'a:'b,'b>(x:View<'a>,y:&'b i64)->&'b i64{let _=y;x.value}
"#;
fn oracle(root:&std::path::Path,source:&std::path::Path,out:&std::path::Path,producer:Option<&str>)->std::process::Output{
    let mut c=std::process::Command::new("/core/bin/rustc");c.arg(source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    if let Some(metadata)=producer{c.args(["--crate-name","same","--crate-type=rlib"]).arg(format!("-Cmetadata={metadata}"));}else{for name in["base","other","provider"]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}}
    c.arg("-o").arg(out).output().unwrap()
}
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("base",ORIGINAL,"bounded_fn_original"),("other",ORIGINAL,"bounded_fn_other"),("provider",PROVIDER,"bounded_fn_provider")]{let source=root.join(format!("producer-{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");let b=std::fs::read(image).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();assert_eq!(a.encode().unwrap(),b);images.push((label.into(),a));let o=oracle(root,&source,&root.join(format!("lib{label}.rlib")),Some(metadata));record(root,&format!("{label}.oracle-producer"),&o);assert!(o.status.success(),"{label}: {}",String::from_utf8_lossy(&o.stderr));}
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("bounded-function-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid,diagnosis)in[
        ("direct","fn main(){let x=7i64;let y=3i64;assert!(*base::shorten(&x,&y)==7);assert!(*other::shorten(&y,&x)==3);assert!(*provider::where_bound(&x,&y)==7);assert!(provider::unused(4)==5);}",true,""),
        ("transitive","fn main(){let x=9i64;let y=2i64;let z=1i64;assert!(*provider::transitive(&x,&y,&z)==9);assert!(*provider::cycle(&x,&y)==9);}",true,""),
        ("mixed","fn main(){let x=13i64;let r;{let y=2i64;r=provider::mixed(&x,&y);}assert!(*r==13);assert!(*provider::unicode(&x)==13);}",true,""),
        ("nominal","use provider::{View,view};fn main(){let x=17i64;let y=3i64;let p=View{value:&x};assert!(*view(p,&y)==17);}",true,""),
        ("static-pointer","fn main(){let f:fn(&'static i64)->&'static i64=provider::self_bound;let x:&'static i64=&21;assert!(*f(x)==21);assert!(*provider::static_bound(x)==21);}",true,""),
        ("escape","fn main(){let r;let y=3i64;{let x=7i64;r=base::shorten(&x,&y);}assert!(*r==7);}",false,"L0306"),
        ("static","fn main(){let x=7i64;let r=provider::static_bound(&x);assert!(*r==7);}",false,"L0306"),
        ("generality","fn main(){let f:fn(&i64)->&i64=provider::self_bound;let _=f;}",false,"L0110"),
        ("nominal-escape","use provider::{View,view};fn main(){let r;let y=2i64;{let x=7i64;r=view(View{value:&x},&y);}assert!(*r==7);}",false,"L0306"),
        ("nominal-move","use provider::{View,view};fn main(){let x=7i64;let y=2i64;let p=View{value:&x};view(p,&y);view(p,&y);}",false,"L0301"),
    ]{let source=root.join(format!("consumer-{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.expect_err("INVALID native output; NEVER EXECUTE");assert!(e.contains(diagnosis),"{label}: {e}");assert!(!out.join("app").exists());}let app=root.join(format!("{label}-oracle"));let o=oracle(root,&source,&app,None);record(root,&format!("{label}.oracle-consumer"),&o);assert_eq!(o.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&o.stderr));if valid{let n=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success(),"{label}: {n:?}");let o=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}}
    malformed(root,generation);remap(root,generation);
    println!("native lifetime-bound function images: original producer, declared/where/static/transitive/cyclic/unused/mixed/nominal runtime and rejection gate PASS; full Rust/no_std/native Cargo/SAME complete-work speed OPEN");
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;let a=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let stable=a.functions()[0].id;let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("bounded-function-remap","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("base".into(),a)],ids).unwrap();let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("bounded_fn_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"fn local<'x>(x:&'x i64)->&'x i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect {}",e.len()));let fns=p.fns.len();let local=p.regions.fns.len();store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),local);assert_eq!(p.regions.native_fns.len(),1);let(d,s)=p.regions.native_fns.iter().next().unwrap();assert_eq!(d.krate,def::CrateId(1));assert_eq!(d.hash,stable.hash);assert_eq!(s.early,[true,true]);assert_eq!(s.names,["lifetime `'a`","lifetime `'b`"]);assert_eq!(s.facts,[(ty::Region::Param(0),ty::Region::Param(1))]);assert!(matches!(p.tys.kind(s.ret),ty::TyKind::Ref(ty::Region::Param(1),_,_)));put(&root.join("remap-install-only.txt"),b"Fresh interner/local function/region rows preserved; full foreign identity, original early flags/names/fact/output checked. INSTALL ONLY, no execution from perturbed Program.\n");
}
// Independent byte navigation of format13's region record, separate from its decoder.
struct Wire{n:usize,facts:Vec<(usize,usize)>,names:Vec<(usize,usize)>,early:Vec<usize>,decl_count:usize,decl:Vec<(usize,usize)>}
fn wires(b:&[u8])->(usize,Vec<Wire>){
    fn number(b:&[u8],at:&mut usize)->usize{let n=u32::from_le_bytes(b[*at..*at+4].try_into().unwrap())as usize;*at+=4;n}fn text(b:&[u8],at:&mut usize)->(usize,usize){let n=number(b,at);let start=*at;*at+=n;(start,n)}fn region(b:&[u8],at:&mut usize)->usize{let start=*at;*at+=1;if b[start]==2{*at+=4;}start}
    let mut at=b"LUSTR-NATIVE-CRATE\0".len()+1+64;for _ in 0..4{text(b,&mut at);}at+=8+32+2;let n=number(b,&mut at);for _ in 0..n{at+=16;text(b,&mut at);at+=1;let n=number(b,&mut at);at+=4*n+4;text(b,&mut at);}for _ in 0..2{let n=number(b,&mut at);at+=n;}let region_size=at;let size=number(b,&mut at);let end=at+size;let n=number(b,&mut at);let mut rows=Vec::new();for _ in 0..n{let univ=at;at+=4;let n=number(b,&mut at);at+=4*n+4;let n=number(b,&mut at);let facts=(0..n).map(|_|(region(b,&mut at),region(b,&mut at))).collect();let n=number(b,&mut at);let names=(0..n).map(|_|text(b,&mut at)).collect();let n=number(b,&mut at);let early=(at..at+n).collect();at+=n;let decl_count=at;let n=number(b,&mut at);let decl=(0..n).map(|_|text(b,&mut at)).collect();rows.push(Wire{n:univ,facts,names,early,decl_count,decl});}assert_eq!(at,end);(region_size,rows)
}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation){
    let raw=std::fs::read(root.join("base.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let(_,rows)=wires(original);let w=&rows[0];assert_eq!(w.decl.len(),2);assert_eq!(w.facts.len(),1);
    let reject=|label:&str,mut b:Vec<u8>,expected:&str|{b.extend_from_slice(&native_crate::digest(&b).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&b);let e=native_crate::Artifact::decode(&b,generation).err().expect("malformed bound function accepted");assert!(e.contains(expected),"{label}: {e}");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());};
    let mut b=original.to_vec();b[w.early[0]]=0;reject("early-classification",b,"early/late region classification");
    let mut b=original.to_vec();b[w.early[0]]=2;reject("early-tag",b,"invalid native boolean");
    let mut b=original.to_vec();b[w.decl[1].0]=b[w.decl[0].0];reject("duplicate-declaration",b,"declaration identity/count");
    let mut b=original.to_vec();b[w.decl[0].0]=b'_';reject("reserved-declaration",b,"declaration identity/count");
    let mut b=original.to_vec();b[w.names[0].0]^=1;reject("diagnostic-declaration",b,"declaration identity/count");
    let mut b=original.to_vec();b[w.facts[0].0+1..w.facts[0].0+5].copy_from_slice(&63u32.to_le_bytes());reject("fact-index",b,"universal index");
    let mut b=original.to_vec();b[w.decl_count..w.decl_count+4].copy_from_slice(&65u32.to_le_bytes());reject("declaration-count",b,"field exceeds bound");
    let mut b=original.to_vec();b[w.n..w.n+4].copy_from_slice(&1u32.to_le_bytes());reject("universal-count",b,"signature bounds");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=12;reject("old-format12",b,"format generation");
    let raw=std::fs::read(root.join("provider.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let a=native_crate::Artifact::decode(&raw,generation).unwrap();let mixed=a.functions().iter().position(|f|f.name=="mixed").unwrap();let(_,rows)=wires(original);let w=&rows[mixed];let mut b=original.to_vec();assert_eq!(w.decl.len(),1);b[w.facts[0].1+1..w.facts[0].1+5].copy_from_slice(&1u32.to_le_bytes());reject("anonymous-fact",b,"declared fact index/order");
}
