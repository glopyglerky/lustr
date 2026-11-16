//! Wide references at the independent native crate boundary.
use super::*;
pub(super) const ORIGINAL:&str="#![no_std]\npub fn view(x:&[i64])->&[i64]{x}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),o.status.to_string().as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"wide_original"),generation,backend);put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());
    let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-o"]).arg(root.join("liboriginal.rlib")).output().unwrap();record(root,"original.oracle-build",&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));println!("unchanged slice-reference producer native refusal/no image/no apps; separate explicit SDK oracle rlib0");
}
const PROVIDER:&str=r#"#![no_std]
pub fn text(x:&str)->&str{x}
pub fn first(x:&[i64])->i64{x[0]}
pub fn change(x:&mut [i64])->&mut [i64]{x[0]+=3;x}
pub fn choose<'a:'b,'b>(x:&'a [i64],y:&'b [i64])->&'b [i64]{let _=y;x}
pub fn refs<'a,'b>(x:&'a [&'b i64])->&'b i64{x[0]}
pub struct View<'a>{pub values:&'a [i64],pub label:&'a str}
pub fn packet<'a>(v:View<'a>)->View<'a>{v}
pub fn callback(f:fn(&[i64])->&[i64],x:&[i64])->i64{f(x)[0]}
"#;
fn oracle(root:&std::path::Path,source:&std::path::Path,out:&std::path::Path,producer:Option<&str>)->std::process::Output{
    let mut c=std::process::Command::new("/core/bin/rustc");c.arg(source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    if let Some(metadata)=producer{c.args(["--crate-name","same","--crate-type=rlib"]).arg(format!("-Cmetadata={metadata}"));}else{for name in["base","other","provider"]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}}
    c.arg("-o").arg(out).output().unwrap()
}
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("base",ORIGINAL,"wide_original"),("other",ORIGINAL,"wide_other"),("provider",PROVIDER,"wide_provider")]{
        let source=root.join(format!("producer-{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");let b=std::fs::read(image).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();assert_eq!(a.encode().unwrap(),b);images.push((label.into(),a));let o=oracle(root,&source,&root.join(format!("lib{label}.rlib")),Some(metadata));record(root,&format!("{label}.oracle-producer"),&o);assert!(o.status.success(),"{label}: {}",String::from_utf8_lossy(&o.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("wide-reference-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid,diagnosis)in[
        ("direct","fn main(){let a=[7i64,11,13];assert!(base::view(&a)[1]==11);assert!(other::view(&a)[2]==13);assert!(provider::first(&a)==7);let e:[i64;0]=[];assert!(base::view(&e).len()==0);}",true,""),
        ("mutable","fn main(){let mut a=[4i64,5];{let r=provider::change(&mut a);r[1]=9;assert!(r[0]==7);}assert!(a[0]==7);assert!(a[1]==9);}",true,""),
        ("strings","fn main(){let s=provider::text(\"héllø世界\");assert!(s==\"héllø世界\");assert!(provider::text(\"\")==\"\");}",true,""),
        ("nested","fn main(){let x=17i64;let a=[&x];assert!(*provider::refs(&a)==17);let b=[19i64];let p=provider::View{values:&b,label:\"view\"};let p=provider::packet(p);assert!(p.values[0]==19);assert!(p.label==\"view\");}",true,""),
        ("pointer","fn main(){let a=[23i64];let f:fn(&[i64])->&[i64]=base::view;assert!(f(&a)[0]==23);assert!(provider::callback(other::view,&a)==23);let g:fn(&str)->&str=provider::text;assert!(g(\"wide\")==\"wide\");}",true,""),
        ("bounded","fn main(){let a=[29i64];let b=[31i64];assert!(provider::choose(&a,&b)[0]==29);}",true,""),
        ("inner-outlives-slice","fn main(){let x=37i64;let r;{let a=[&x];r=provider::refs(&a);}assert!(*r==37);}",true,""),
        ("inner-escape","fn main(){let r;{let x=37i64;let a=[&x];r=provider::refs(&a);}assert!(*r==37);}",false,"L0306"),
        ("escape","fn main(){let r;{let a=[7i64];r=base::view(&a);}assert!(r[0]==7);}",false,"L0306"),
        ("alias","fn main(){let mut a=[7i64];let r=provider::change(&mut a);let s=&mut a;s[0]=2;assert!(r[0]==10);}",false,"L0304"),
        ("static","fn main(){let a=[7i64];let r:&'static [i64]=base::view(&a);assert!(r[0]==7);}",false,"L0306"),
        ("type","fn main(){let a=[7u8];let _=base::view(&a);}",false,"L0110"),
        ("move","fn main(){let a=[7i64];let p=provider::View{values:&a,label:\"v\"};provider::packet(p);provider::packet(p);}",false,"L0301"),
    ]{
        let source=root.join(format!("consumer-{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.expect_err("INVALID native output; NEVER EXECUTE");assert!(e.contains(diagnosis),"{label}: {e}");assert!(!out.join("app").exists());}let app=root.join(format!("{label}-oracle"));let o=oracle(root,&source,&app,None);record(root,&format!("{label}.oracle-consumer"),&o);assert_eq!(o.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&o.stderr));if valid{let n=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success(),"{label}: {n:?}");let o=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    for(label,text)in[("producer-escape","#![no_std]\npub fn bad<'a>()->&'a [i64]{let a=[1i64];&a}\n"),("producer-shared-write","#![no_std]\npub fn bad(x:&[i64]){x[0]=1;}\n")]{let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,"invalid_wide"),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());assert_eq!(code,1,"INVALID producer accepted; NEVER CONSUME: {message}");assert!(!image.exists());assert!(message.contains(if label=="producer-escape"{"L0307"}else{"L0308"}),"{label}: {message}");let o=oracle(root,&source,&root.join(format!("{label}.rlib")),Some("invalid"));record(root,&format!("{label}.oracle"),&o);assert!(!o.status.success());}
    malformed(root,generation,backend);remap(root,generation);
    println!("native wide references: original/slice/str/mutable/nested/callback/bounded runtime and semantic rejection PASS; fullRust/no_std/nativeCargo/SAMEcompletework OPEN");
}
#[derive(Clone,Copy)]
struct Wire{tag:usize,region:usize,mutable:usize,element:usize,layout:usize,copy:usize,abi:usize}
// The original/ref-only images have no nominal or pointer records. Navigate
// their wire independently of the decoder so malformed inputs keep checksums.
fn wires(b:&[u8])->Vec<Wire>{
    fn n(b:&[u8],p:&mut usize)->usize{let v=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;v}
    fn text(b:&[u8],p:&mut usize){let v=n(b,p);*p+=v;}
    let mut p=b"LUSTR-NATIVE-CRATE\0".len()+1+64;for _ in 0..4{text(b,&mut p);}p+=8+32+2;let count=n(b,&mut p);for _ in 0..count{p+=16;text(b,&mut p);p+=1;let count=n(b,&mut p);p+=4*(count+1);text(b,&mut p);}let size=n(b,&mut p);let end=p+size;let count=n(b,&mut p);let mut result=Vec::new();
    for _ in 0..count{let tag=p;p+=1;let wide=if b[tag]==27{let region=p;p+=1;if b[region]==2{p+=4;}let mutable=p;p+=1;let element=p;p+=4;Some((region,mutable,element))}else{assert!(b[tag]<=17);None};let layout=p;p+=16;let fields=n(b,&mut p);p+=8*fields;let niche=b[p];p+=1;if niche==1{p+=48;}p+=1;let copy=p;p+=2;let abi=p;p+=19;if let Some((region,mutable,element))=wide{result.push(Wire{tag,region,mutable,element,layout,copy,abi});}}
    assert_eq!(p,end);assert!(!result.is_empty());result
}
fn checksum(mut b:Vec<u8>)->Vec<u8>{b.extend_from_slice(&native_crate::digest(&b).unwrap());b}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let raw=std::fs::read(root.join("base.lustr-native")).unwrap();let original=&raw[..raw.len()-32];let rows=wires(original);let w=rows[0];let named=rows.iter().find(|w|original[w.region]==2).unwrap();
    let reject=|label:&str,b:Vec<u8>,expected:&str|{let b=checksum(b);put(&root.join(format!("malformed-{label}.lustr-native")),&b);let e=native_crate::Artifact::decode(&b,generation).err().expect("malformed wide reference accepted");assert!(e.contains(expected),"{label}: {e}");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());};
    let mut b=original.to_vec();b[w.element..w.element+4].copy_from_slice(&u32::MAX.to_le_bytes());reject("forward-element",b,"forward/cyclic");
    let mut b=original.to_vec();b[w.mutable]=2;reject("mutability-tag",b,"boolean");
    let mut b=original.to_vec();b[w.region]=3;reject("region-tag",b,"region tag");
    let mut b=original.to_vec();b[named.region+1..named.region+5].copy_from_slice(&63u32.to_le_bytes());reject("unbound-region",b,"universal index");
    let mut b=original.to_vec();b[w.layout..w.layout+8].copy_from_slice(&8u64.to_le_bytes());reject("thin-layout",b,"layout/Copy/drop");
    let mut b=original.to_vec();b[w.copy]=0;reject("shared-copy",b,"layout/Copy/drop");
    let mut b=original.to_vec();b[w.tag]=255;reject("type-tag",b,"type tag");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=13;reject("old-format13",b,"format generation");
    let mut images:Vec<_>=["base","other"].iter().map(|name|{let b=std::fs::read(root.join(format!("{name}.lustr-native"))).unwrap();let a=native_crate::Artifact::decode(&b,generation).unwrap();(a.functions()[0].id.krate,b)}).collect();images.sort_by_key(|x|x.0);let healthy=native_crate::Artifact::decode(&images[0].1,generation).unwrap();let raw=&images[1].1;let w=wires(raw)[0];let mut bad=raw[..raw.len()-32].to_vec();bad[w.abi+16]^=1;let bad=checksum(bad);put(&root.join("later-bad-wide-abi.lustr-native"),&bad);let bad=native_crate::Artifact::decode(&bad,generation).unwrap();
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("wide-late-abi","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("good".into(),healthy),("bad".into(),bad)],ids).unwrap();let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().filter(|d|d.krate!=def::LOCAL_CRATE).copied().collect();let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let e=store.install_signatures(p).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");for d in &foreign{assert!(p.defs.get(*d).is_none());assert!(!p.type_names.contains_key(&tables.def_info[d].flat));}assert!(p.regions.native_fns.is_empty());assert!(p.consts.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),e,"Expected wide ABI refusal")])};let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("later wide ABI must stop collection");let rendered=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!rendered.contains("division by zero"));put(&root.join("later-abi-before-definitions.log"),rendered.as_bytes());
    let mut request=request;request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("later-bad-wide-abi-native");let e=store.executable(&request,generation,backend,&out).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");assert!(!out.join("app").exists());put(&root.join("later-abi-executable.log"),e.as_bytes());
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;let a=native_crate::Artifact::decode(&std::fs::read(root.join("base.lustr-native")).unwrap(),generation).unwrap();let stable=a.functions()[0].id;let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("wide-remap","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("base".into(),a)],ids).unwrap();let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("wide_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"fn local<'x>(x:&'x i64)->&'x i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect {}",e.len()));let fns=p.fns.len();let local=p.regions.fns.len();store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),local);let(d,s)=p.regions.native_fns.iter().next().unwrap();assert_eq!(d.krate,def::CrateId(1));assert_eq!(d.hash,stable.hash);let ty::TyKind::Ref(ty::Region::Param(0),ty::Mut::Shared,e)=p.tys.kind(s.ret)else{panic!("wide result region");};assert!(matches!(p.tys.kind(e),ty::TyKind::Slice(_)));put(&root.join("remap-install-only.txt"),b"Fresh interner/local fn+region rows preserved; full foreign identity and original slice output region checked. INSTALL ONLY; no execution from perturbed Program.\n");
}
