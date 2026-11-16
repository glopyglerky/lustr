//! Native region-image composition and rejection through original borrow rules.
use super::*;
const ORIGINAL:&str="#![no_std]\npub fn borrowed(x:&i64)->&i64{x}\n";
const OTHER:&str="#![no_std]\npub fn bump(x:&mut i64)->&mut i64{*x+=2;x}\npub fn array(x:&[i64;5])->&[i64;5]{x}\npub fn pair(x:&(i64,bool))->&(i64,bool){x}\npub fn read(x:&i64,y:&i64)->i64{*x+*y}\npub fn static_id(x:&'static i64)->&'static i64{x}\npub fn nested_read(x:&&i64)->i64{**x}\npub fn static_slot(x:&mut &'static i64){}\npub fn touch(x:&mut i64,y:&mut i64){*x+=1;*y+=1;}\n";
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(name,text,metadata)in[("borrowed",ORIGINAL,"region_original"),("other",OTHER,"region_other")]{
        let source=root.join(format!("{name}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{name}.lustr-native"));
        let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{name}.native.log")),message.as_bytes());put(&root.join(format!("{name}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{message}");
        let bytes=std::fs::read(&image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);images.push((name.into(),artifact));
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{name}.rlib"))).output().unwrap();record(root,&format!("{name}.oracle-build"),&oracle);assert!(oracle.status.success());
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("region-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid)in[
        ("direct","use borrowed::borrowed;use other::{bump,array,pair,read};fn main(){let x=7i64;let r=borrowed(&x);assert!(*r==7);let f=borrowed;assert!(*f(&x)==7);let mut y=3i64;let m=bump(&mut y);assert!(*m==5);*m=9;assert!(y==9);let a=[1i64,2,3,4,5];let z=array(&a);assert!(z[4]==5);let p=(8i64,true);let q=pair(&p);assert!(q.0==8);assert!(q.1);assert!(read(&x,&y)==16);}",true),
        ("pointer","use borrowed::borrowed;use other::bump;fn main(){let x=7i64;let f:fn(&i64)->&i64=borrowed;let r=f(&x);assert!(*r==7);let mut y=2i64;let m:fn(&mut i64)->&mut i64=bump;*m(&mut y)=9;assert!(y==9);}",true),
        ("independent","use other::{read,nested_read,touch};fn main(){let x=4i64;{let y=5i64;assert!(read(&x,&y)==9);assert!(nested_read(&&y)==5);}let mut a=1i64;let mut b=2i64;touch(&mut a,&mut b);assert!(a==2);assert!(b==3);}",true),
        ("escape","use borrowed::borrowed;fn main(){let r;{let x=4i64;r=borrowed(&x);}assert!(*r==4);}",false),
        ("value-escape","use borrowed::borrowed;fn main(){let f=borrowed;let r;{let x=4i64;r=f(&x);}assert!(*r==4);}",false),
        ("pointer-escape","use borrowed::borrowed;fn main(){let f:fn(&i64)->&i64=borrowed;let r;{let x=4i64;r=f(&x);}assert!(*r==4);}",false),
        ("pointer-generality","use other::static_id;fn main(){let f:fn(&i64)->&i64=static_id;}",false),
        ("mutable-alias","use other::bump;fn main(){let mut x=1i64;let r=bump(&mut x);x=9;assert!(*r==3);}",false),
        ("mutable-move","use other::bump;fn main(){let mut x=1i64;let r=bump(&mut x);let s=r;*r=9;*s=7;}",false),
        ("wrong-mutability","use other::bump;fn main(){let x=1i64;bump(&x);}",false),
        ("static-input","use other::static_id;fn main(){let x=4i64;let r=static_id(&x);assert!(*r==4);}",false),
        ("array-escape","use other::array;fn main(){let r;{let a=[1i64,2,3,4,5];r=array(&a);}assert!(r[0]==1);}",false),
        ("nested-invariance","use other::static_slot;fn main(){let x=4i64;let mut r=&x;static_slot(&mut r);}",false),
        ("independent-alias","use other::touch;fn main(){let mut x=4i64;touch(&mut x,&mut x);}",false),
    ]{
        let file=format!("{label}.rs");put(&root.join(&file),text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());
        if !valid{let error=result.err().unwrap_or_else(||panic!("INVALID {label} produced native output; never execute"));assert!(!out.join("app").exists());let code=match label{"mutable-alias"=>"L0312","mutable-move"=>"L0301","wrong-mutability"|"pointer-generality"=>"L0110","independent-alias"=>"L0304",_=>"L0306"};assert!(error.contains(code),"{label} must reach actual type/borrow rule {code}: {error}");}else{result.unwrap_or_else(|e|panic!("{label}: {e}"));}
        let mut command=std::process::Command::new("/core/bin/rustc");command.arg(root.join(&file)).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in["borrowed","other"]{command.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let oracle_app=root.join(format!("{label}-oracle"));let oracle=command.arg("-o").arg(&oracle_app).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert_eq!(oracle.status.success(),valid);
        if valid{let native=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&native);assert!(native.status.success());let oracle=std::process::Command::new(oracle_app).output().unwrap();record(root,&format!("{label}.oracle-app"),&oracle);assert!(oracle.status.success());assert_eq!(native.stdout,oracle.stdout);assert_eq!(native.stderr,oracle.stderr);}
    }
    // Perturb both the interner and native type tables before installing into a
    // genuinely collected Program with its own original local region records.
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("region_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"fn local(x:&i64)->&i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("region remap collect errors {}",e.len()));let fns=p.fns.len();let locals=p.regions.fns.len();for n in 100..132{use ty::TyIntern;let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),locals);assert_eq!(p.regions.native_fns.len(),9);assert!(p.regions.native_fns.values().any(|s|s.n_univ==2&&s.params.len()==2));
    malformed(root,generation);
    println!("native original borrowed signature, independent shared/mutable/array/tuple/static region records, runtime/oracle parity and borrow refusals PASS; full Rust/Cargo/no_std/SAME-work speed OPEN");
}
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation){
    fn n(b:&[u8],p:&mut usize)->usize{let n=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;n}
    fn text(b:&[u8],p:&mut usize){let c=n(b,p);*p+=c;}
    let bytes=std::fs::read(root.join("borrowed.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];let mut at=b"LUSTR-NATIVE-CRATE\0\x05".len()+64;for _ in 0..4{text(original,&mut at);}at+=8+32+2;
    for _ in 0..n(original,&mut at){at+=16;text(original,&mut at);at+=1;let count=n(original,&mut at);at+=4*(count+1);text(original,&mut at);}
    for _ in 0..2{let size=n(original,&mut at);at+=size;}
    let size=n(original,&mut at);let end=at+size;let count=at;assert_eq!(n(original,&mut at),1);let universal=at;assert_eq!(n(original,&mut at),1);let param_count=n(original,&mut at);assert_eq!(param_count,1);let param=at;at+=4;let ret=at;assert!(at+4<end);
    for(label,pos,value,expected)in[("region-count",count,0,"region trailing data"),("universal",universal,0,"region signature bounds"),("region-param",param,0,"region/check type shape mismatch"),("region-return",ret,u32::MAX,"region type index")]{
        let mut payload=original.to_vec();payload[pos..pos+4].copy_from_slice(&value.to_le_bytes());payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&payload);let error=native_crate::Artifact::decode(&payload,generation).err().expect("malformed regions accepted");put(&root.join(format!("malformed-{label}.log")),error.as_bytes());assert!(error.contains(expected),"{label}: {error}");
    }
}
