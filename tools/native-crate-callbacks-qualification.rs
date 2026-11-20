//! Borrowed callbacks cross real native artifact boundaries and retain the
//! original native pointer proof. The explicit SDK is an independent oracle.
use super::*;
const ORIGINAL:&str="#![no_std]\npub fn borrowed(f:fn(&i64)->&i64)->fn(&i64)->&i64{f}\n";
const PROVIDER:&str="#![no_std]\npub fn id(x:&i64)->&i64{x}\npub fn bump(x:&mut i64)->&mut i64{*x+=2;x}\npub fn selected()->fn(&i64)->&i64{id}\npub fn selected_mut()->fn(&mut i64)->&mut i64{bump}\npub fn static_id(x:&'static i64)->&'static i64{x}\npub fn selected_static()->fn(&'static i64)->&'static i64{static_id}\npub fn pair(x:&i64)->(&i64,i64){(x,*x+1)}\n";
const RELAY:&str="#![no_std]\npub fn apply(f:fn(&i64)->&i64,x:&i64)->&i64{f(x)}\npub fn apply_mut(f:fn(&mut i64)->&mut i64,x:&mut i64)->&mut i64{f(x)}\npub fn relay(f:fn(&i64)->&i64)->fn(&i64)->&i64{f}\npub fn pair(f:fn(&i64)->(&i64,i64),x:&i64)->(&i64,i64){f(x)}\npub fn count(f:fn(&i64,&i64)->i64,x:&i64,y:&i64)->i64{f(x,y)}\npub fn explicit(f:for<'a> fn(&'a i64)->&'a i64)->for<'a> fn(&'a i64)->&'a i64{f}\npub fn higher(f:fn(fn(&i64)->&i64)->fn(&i64)->&i64,x:fn(&i64)->&i64)->fn(&i64)->&i64{f(x)}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),o.status.to_string().as_bytes());}
fn oracle(root:&std::path::Path,source:&std::path::Path,out:&std::path::Path,producer:Option<&str>)->std::process::Output{
    let mut c=std::process::Command::new("/core/bin/rustc");c.arg(source).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    if let Some(name)=producer{c.args(["--crate-name","same","--crate-type=rlib"]).arg(format!("-Cmetadata={name}"));}else{for name in["original","provider","relay"]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}}
    c.arg("-o").arg(out).output().unwrap()
}
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(name,text)in[("original",ORIGINAL),("provider",PROVIDER),("relay",RELAY)]{
        let source=root.join(format!("{name}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{name}.lustr-native"));
        let(code,message)=native_object::run_artifact(&args(&source,&image,name),generation,backend);put(&root.join(format!("{name}.native.log")),message.as_bytes());put(&root.join(format!("{name}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{name}: {message}");
        let bytes=std::fs::read(&image).unwrap();let a=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(a.encode().unwrap(),bytes);images.push((name.to_owned(),a));
        let o=oracle(root,&source,&root.join(format!("lib{name}.rlib")),Some(name));record(root,&format!("{name}.oracle-build"),&o);assert!(o.status.success(),"{}",String::from_utf8_lossy(&o.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("callback-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid)in[
        ("direct","use original::borrowed;use provider::{id,bump,pair};use relay::{apply,apply_mut};fn main(){let x=7i64;let f=borrowed(id);assert!(*apply(f,&x)==7);let p=relay::pair(pair,&x);assert!(*p.0==7);assert!(p.1==8);let mut y=1i64;let r=apply_mut(bump,&mut y);*r=11;assert!(y==11);}",true),
        ("returned","use provider::{selected,selected_mut};use relay::{apply,apply_mut,relay};fn main(){let x=8i64;let f=relay(selected());let g=f;assert!(*apply(g,&x)==8);let mut y=3i64;assert!(*apply_mut(selected_mut(),&mut y)==5);}",true),
        ("pointer","use original::borrowed;use provider::id;use relay::apply;fn main(){let select:fn(fn(&i64)->&i64)->fn(&i64)->&i64=borrowed;let f=select(id);let x=9i64;assert!(*apply(f,&x)==9);}",true),
        ("independent","use relay::count;fn add(x:&i64,y:&i64)->i64{*x+*y}fn main(){let x=4i64;{let y=5i64;assert!(count(add,&x,&y)==9);}assert!(*relay::apply(provider::selected(),&x)==4);}",true),
        ("explicit-nested","fn main(){let x=6i64;let f=relay::higher(original::borrowed,relay::explicit(provider::id));assert!(*f(&x)==6);}",true),
        ("static-callback","fn main(){let x:&'static i64=&7;let f=provider::selected_static();assert!(*f(x)==7);}",true),
        ("escape","use provider::selected;use relay::apply;fn main(){let r;{let x=1i64;r=apply(selected(),&x);}assert!(*r==1);}",false),
        ("mutable-alias","use provider::selected_mut;use relay::apply_mut;fn main(){let mut x=1i64;let r=apply_mut(selected_mut(),&mut x);x=9;assert!(*r==3);}",false),
        ("mutable-move","use provider::selected_mut;use relay::apply_mut;fn main(){let mut x=1i64;let r=apply_mut(selected_mut(),&mut x);let s=r;*r=4;*s=5;}",false),
        ("generality","use provider::static_id;use relay::apply;fn main(){let x=2i64;apply(static_id,&x);}",false),
        ("mutability","use provider::bump;use relay::apply;fn main(){let x=2i64;apply(bump,&x);}",false),
        ("returned-generality","use relay::relay;fn narrow(x:&'static i64)->&'static i64{x}fn main(){let f=relay(narrow);}",false),
        ("returned-static","fn main(){let f=relay::relay(provider::selected_static());}",false),
        ("callback-type","use relay::apply;fn wrong(x:&i64)->i64{*x}fn main(){let x=2i64;apply(wrong,&x);}",false),
        ("const","use provider::selected;const X:bool=false&&{selected();true};fn main(){}",false),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let native=root.join(format!("native-{label}"));let result=store.executable(&request,generation,backend,&native);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());
        if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.err().unwrap_or_else(||panic!("INVALID {label} emitted artifact; NEVER execute"));assert!(!native.join("app").exists());let expected=match label{"escape"=>"L0306","mutable-alias"=>"L0312","mutable-move"=>"L0301","const"=>"non-const",_=>"L0110"};assert!(e.contains(expected),"{label}: {e}");}
        let o=oracle(root,&source,&root.join(format!("oracle-{label}")),None);record(root,&format!("{label}.oracle-build"),&o);assert_eq!(o.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&o.stderr));
        if valid{let n=std::process::Command::new(native.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success());let o=std::process::Command::new(root.join(format!("oracle-{label}"))).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    // Fresh allocation histories: installation only, not execution from this
    // deliberately perturbed Program. The executable cases above are separate.
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("callback_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));
    let request=input(root,"fn local(x:&i64)->&i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("callback remap errors {}",e.len()));let before=p.fns.len();let local=p.regions.fns.len();for n in 100..132{use ty::TyIntern;let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),before);assert_eq!(p.regions.fns.len(),local);
    {use ty::TyIntern;assert!(p.regions.native_fns.values().any(|s|s.params.iter().any(|&t|matches!(p.tys.kind(t),ty::TyKind::FnPtr(id) if p.tys.sig(id).value.own_out==1&&p.tys.sig(id).value.out_slots==1))));}
    // A resource frontier, not a Rust language rejection. The upstream proof
    // shifts a u32 per slot, so 33 real reference inputs must never enter it.
    let wide=format!("#![no_std]\npub fn wide(f:fn({}))->fn({}){{f}}\n",vec!["&i64";33].join(","),vec!["&i64";33].join(","));let source=root.join("slot-bound.rs");put(&source,wide.as_bytes());let image=root.join("slot-bound.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"slot_bound"),generation,backend);put(&root.join("slot-bound.native.log"),message.as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());let o=oracle(root,&source,&root.join("libslot-bound.rlib"),Some("slot_bound"));record(root,"slot-bound.oracle-build",&o);assert!(o.status.success());
    malformed(root,generation);
    println!("native callback images: unchanged original producer, independent returned/shared/mutable/tuple callbacks, native+oracle execution parity, actual escape/alias/move/generality/type/const rejections, valid-checksum slot/mask refusals PASS; full image/Rust/Cargo/no_std/SAME-work speed OPEN");
}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation){
    fn n(b:&[u8],p:&mut usize)->usize{let v=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;v}
    fn text(b:&[u8],p:&mut usize){let c=n(b,p);*p+=c;}
    let bytes=std::fs::read(root.join("original.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];let mut at=b"LUSTR-NATIVE-CRATE\0\x05".len()+64;for _ in 0..4{text(original,&mut at);}at+=8+32+2;
    let c=n(original,&mut at);for _ in 0..c{at+=16;text(original,&mut at);at+=1;let ps=n(original,&mut at);at+=4*(ps+1);text(original,&mut at);}let size=n(original,&mut at);let end=at+size;let c=n(original,&mut at);let mut pointer=None;
    for _ in 0..c{let tag=original[at];at+=1;match tag{0..=17=>{},18=>{let c=n(original,&mut at);at+=4*c;},19=>at+=12,20=>{let c=n(original,&mut at);at+=4*c+4;pointer=Some(at);at+=5;},21=>{let tag=original[at];at+=1;if tag==2{at+=4;}at+=5;},_=>panic!("unknown graph tag")};at+=16;let c=n(original,&mut at);at+=8*c;let niche=original[at];at+=1+48*niche as usize+3+16+3;}
    assert_eq!(at,end);let mask=pointer.expect("original borrowed pointer must be represented");assert_eq!(&original[mask..mask+5],&[1,0,0,0,1]);
    for(label,pos,replacement)in[("missing-mask",mask,0u32.to_le_bytes().to_vec()),("outside-mask",mask,2u32.to_le_bytes().to_vec()),("wrong-width",mask+4,vec![0]),("unknown-width-with-mask",mask+4,vec![u8::MAX])]{let mut b=original.to_vec();b[pos..pos+replacement.len()].copy_from_slice(&replacement);b.extend_from_slice(&native_crate::digest(&b).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&b);let e=native_crate::Artifact::decode(&b,generation).err().expect("malformed callback accepted");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());assert!(e.contains("function pointer signature bound"),"{e}");}
    let mut old=original.to_vec();old[b"LUSTR-NATIVE-CRATE\0".len()]=4;old.extend_from_slice(&native_crate::digest(&old).unwrap());put(&root.join("old-format.lustr-native"),&old);assert!(native_crate::Artifact::decode(&old,generation).err().unwrap().contains("format generation"));
}
