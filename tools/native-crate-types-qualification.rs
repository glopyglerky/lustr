//! Compound signature regression for the actual native semantic image.
use super::*;
use ty::TyIntern;
const PAIR:&str="#![no_std]\npub fn pair(x:(i64,bool))->(i64,bool){(x.0+1,!x.1)}\n";
const COMPOUND:&str="#![no_std]\npub mod api {\npub fn array(x:[i64;5])->[i64;5]{x}\npub fn splat(x:i64)->[i64;5]{[x,x+1,x+2,x+3,x+4]}\npub fn zero(x:[u8;0])->[u8;0]{x}\npub fn tuple_zero(x:([u8;0],u64))->([u8;0],u64){(x.0,x.1+2)}\npub fn apply(f:fn((i64,bool))->(i64,bool),x:(i64,bool))->(i64,bool){f(x)}\npub fn apply_array(f:fn([i64;5])->[i64;5],x:[i64;5])->[i64;5]{f(x)}\npub fn select()->fn(i64)->i64{step}\nfn step(x:i64)->i64{x*3}\npub fn nested(x:((u8,u64),[i32;2]))->((u8,u64),[i32;2]){x}\npub fn float(x:f64)->f64{x+0.5}\npub fn twin(x:(i64,bool))->(i64,bool){(x.0+2,x.1)}\n}\npub use api::splat;\n";
const CONSUMER:&str="use pair::pair; use compound::api::{array,zero,tuple_zero,apply,apply_array,select,nested,float}; use compound::splat; use mirror::api::array as mirror_array;\nfn local(x:i64)->i64{x+7}\nfn main(){let p=pair((4,true));assert!(p.0==5);assert!(!p.1);let a=splat(10);let b=array(a);assert!(b[0]==10);assert!(b[4]==14);assert!(a[3]==13);let z=zero([]);assert!(z.len()==0);let zp:fn([u8;0])->[u8;0]=zero;let zr=zp([]);assert!(zr.len()==0);let tz=tuple_zero(([],8));assert!(tz.1==10);let q:fn((i64,bool))->(i64,bool)=pair;let r=apply(q,(9,false));assert!(r.0==10);assert!(r.1);let ap:fn([i64;5])->[i64;5]=array;let c=apply_array(ap,b);assert!(c[2]==12);let m=mirror_array(c);assert!(m[4]==14);let s=select();assert!(s(4)==12);assert!(local(s(2))==13);let n=nested(((2u8,19u64),[3i32,4i32]));assert!(n.0.0==2);assert!(n.0.1==19);assert!(n.1[1]==4);assert!(float(1.5)==2.0);}\n";
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,first:bool){
    let source=root.join("pair.rs");put(&source,PAIR.as_bytes());
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-Cmetadata=typed_pair"]).arg("-o").arg(root.join("libpair.rlib")).output().unwrap();
    put(&root.join("pair.oracle.stdout"),&oracle.stdout);put(&root.join("pair.oracle.stderr"),&oracle.stderr);put(&root.join("pair.oracle.status"),format!("{}",oracle.status).as_bytes());assert!(oracle.status.success());
    let image=root.join("pair.lustr-native");let(code,message)=native_object::run_artifact(&args(&source,&image,"typed_pair"),generation,backend);
    put(&root.join("pair.native.log"),message.as_bytes());put(&root.join("pair.native.status"),code.to_string().as_bytes());
    if first {assert_eq!(code,1,"{message}");assert!(!image.exists());assert!(message.contains("type/lifetime/layout record"),"{message}");println!("FIRST original tuple signature: explicit oracle accepted, native refused without image; compiler {}",std::env::current_exe().unwrap().display());return;}
    assert_eq!(code,0,"{message}");let bytes=std::fs::read(&image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);
    let other_source=root.join("compound.rs");put(&other_source,COMPOUND.as_bytes());let other_image=root.join("compound.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&other_source,&other_image,"typed_compound"),generation,backend);put(&root.join("compound.native.log"),message.as_bytes());put(&root.join("compound.native.status"),code.to_string().as_bytes());assert_eq!(code,0,"{message}");
    let other_bytes=std::fs::read(&other_image).unwrap();let other=native_crate::Artifact::decode(&other_bytes,generation).unwrap();assert_eq!(other.encode().unwrap(),other_bytes);
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("typed-consumer","bin","consumer",false)};
    let mirror_image=root.join("mirror.lustr-native");let(code,message)=native_object::run_artifact(&args(&other_source,&mirror_image,"typed_mirror"),generation,backend);put(&root.join("mirror.native.log"),message.as_bytes());assert_eq!(code,0,"{message}");let mirror=native_crate::Artifact::decode(&std::fs::read(mirror_image).unwrap(),generation).unwrap();
    let store=native_crate::CrateStore::new(vec![("pair".into(),artifact.clone()),("compound".into(),other.clone()),("again".into(),artifact.clone()),("mirror".into(),mirror)],ids).unwrap();
    let mut request=input(root,CONSUMER,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
    put(&root.join("consumer.rs"),CONSUMER.as_bytes());let native=root.join("native");let result=store.executable(&request,generation,backend,&native);put(&root.join("native.result"),format!("{result:?}").as_bytes());result.unwrap_or_else(|e|panic!("compound consumer: {e}"));
    let run=std::process::Command::new(native.join("app")).output().unwrap();record(root,"native-app",&run);assert!(run.status.success());
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&other_source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-Cmetadata=typed_compound"]).arg("-o").arg(root.join("libcompound.rlib")).output().unwrap();record(root,"compound-oracle-build",&oracle);assert!(oracle.status.success());
    let oracle=std::process::Command::new("/core/bin/rustc").arg(&other_source).args(["--edition=2024","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off","-Cmetadata=typed_mirror"]).arg("-o").arg(root.join("libmirror.rlib")).output().unwrap();record(root,"mirror-oracle-build",&oracle);assert!(oracle.status.success());
    let oracle=oracle_consumer(root,"consumer.rs","oracle-app");record(root,"oracle-build",&oracle);assert!(oracle.status.success());let oracle=std::process::Command::new(root.join("oracle-app")).output().unwrap();record(root,"oracle-app",&oracle);assert!(oracle.status.success());assert_eq!(run.stdout,oracle.stdout);assert_eq!(run.stderr,oracle.stderr);
    // Same native type graph, independently remapped into fresh consumer tables.
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("unrelated_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect errors {}",e.len()));let before=p.fns.len();for n in 100..132{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),before);
    for(label,text)in[
        ("tuple-type","use pair::pair; fn main(){pair((true,4i64));}"),
        ("tuple-arity","use pair::pair; fn main(){pair((4i64,true,1u8));}"),
        ("array-length","use compound::api::array; fn main(){array([1i64,2,3,4]);}"),
        ("callback-type","use pair::pair; use compound::api::apply; fn wrong(x:(i64,bool))->bool{x.1} fn main(){apply(wrong,(1,true));}"),
        ("independent-item","use pair::pair; use compound::api::twin; fn main(){let mut f=pair;f=twin;}"),
        ("const-call","use pair::pair; const X:(i64,bool)=if false {pair((4,true))}else{(0,false)}; fn main(){}"),
    ]{
        let file=format!("reject-{label}.rs");put(&root.join(&file),text.as_bytes());let mut r=request.clone();r.root_text=text.into();let result=store.executable(&r,generation,backend,&root.join(format!("reject-{label}")));let error=result.err().unwrap_or_else(||panic!("INVALID {label} emitted artifact; never execute"));put(&root.join(format!("reject-{label}.native.log")),error.as_bytes());assert!(!root.join(format!("reject-{label}/app")).exists());
        if label=="const-call"{assert!(error.contains("non-const"),"{error}");}let oracle=oracle_consumer(root,&file,&format!("oracle-{label}"));record(root,&format!("reject-{label}.oracle"),&oracle);assert!(!oracle.status.success());
    }
    for(label,text)in[
        ("pointer-unsafe","#![no_std]\npub fn qualifier(f:unsafe fn(i64)->i64)->unsafe fn(i64)->i64{f}\n"),
        ("pointer-extern","#![no_std]\npub fn qualifier(f:extern \"C\" fn(i64)->i64)->extern \"C\" fn(i64)->i64{f}\n"),
    ]{let source=root.join(format!("unsupported-{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("unsupported-{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,"unsupported"),generation,backend);put(&root.join(format!("unsupported-{label}.log")),message.as_bytes());assert_eq!(code,1,"{message}");assert!(!image.exists());}
    malformed_types(root,&bytes,generation,&request,backend);
    malformed_types(root,&other_bytes,generation,&request,backend);
    println!("native typed image: original tuple regression, arrays/large and ZST returns, callbacks/function values, modules/reexports/floats, native runtime + independent oracle parity, fresh remap, invalid-program and unrepresented-region/qualifier refusals PASS; full image/Cargo/no_std/SAME-work speed remain OPEN");
}

fn record(root:&std::path::Path,label:&str,output:&std::process::Output){put(&root.join(format!("{label}.stdout")),&output.stdout);put(&root.join(format!("{label}.stderr")),&output.stderr);put(&root.join(format!("{label}.status")),format!("{}",output.status).as_bytes());}
fn oracle_consumer(root:&std::path::Path,file:&str,out:&str)->std::process::Output{let mut c=std::process::Command::new("/core/bin/rustc");c.arg(root.join(file)).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for(name,lib)in[("pair","pair"),("compound","compound"),("again","pair"),("mirror","mirror")]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{lib}.rlib")).display()));}c.arg("-o").arg(root.join(out)).output().unwrap()}

fn malformed_types(root:&std::path::Path,bytes:&[u8],generation:&native_crate::Generation,request:&resolve::worklist::CrateInput,backend:&native_crate::Backend){
    fn n(b:&[u8],p:&mut usize)->usize{let v=u32::from_le_bytes(b[*p..*p+4].try_into().unwrap())as usize;*p+=4;v}
    fn text(b:&[u8],p:&mut usize){let len=n(b,p);*p+=len;}
    fn seal(mut b:Vec<u8>)->Vec<u8>{b.extend_from_slice(&native_crate::digest(&b).unwrap());b}
    let original=&bytes[..bytes.len()-32];let mut at=b"LUSTR-NATIVE-CRATE\0\x05".len()+64;for _ in 0..4{text(original,&mut at);}at+=8+32+2;
    let functions=n(original,&mut at);let mut first_id=0;
    for f in 0..functions{at+=16;text(original,&mut at);at+=1;let params=n(original,&mut at);if f==0{first_id=at;}at+=4*(params+1);text(original,&mut at);}
    let graph_size=n(original,&mut at);let graph_end=at+graph_size;let count_at=at;let count=n(original,&mut at);let label_prefix=if functions==1{"pair"}else{"compound"};
    let mut changes=vec![("function-id",first_id,u32::MAX.to_le_bytes().to_vec(),"definition/type/code identity"),("node-bound",count_at,u32::MAX.to_le_bytes().to_vec(),"field exceeds bound")];let mut abi=None;let mut array=None;let mut pointer=None;
    for index in 0..count{
        let tag=original[at];at+=1;
        match tag{0..=17=>{},18=>{let c=n(original,&mut at);if c>0{changes.push(("cyclic",at,(index as u32).to_le_bytes().to_vec(),"forward/cyclic"));}at+=4*c;},19=>{at+=4;array=Some(at);at+=8;},20=>{let c=n(original,&mut at);at+=4*c+8;pointer=Some(at);at+=1;},_=>panic!("unknown original node")}
        let layout=at;at+=16;let fields=n(original,&mut at);at+=8*fields;let niche=original[at];at+=1+48*niche as usize;let uninhabited=at;at+=1;let copy=at;at+=1;let drop=at;at+=1;let c_size=at;at+=16;let by_ptr=at+1;at+=3;
        if index==0{changes.push(("layout-size",layout,123456u64.to_le_bytes().to_vec(),"layout/Copy/drop mismatch"));changes.push(("copy",copy,vec![0],"layout/Copy/drop mismatch"));changes.push(("drop",drop,vec![1],"layout/Copy/drop mismatch"));changes.push(("inhabited",uninhabited,vec![1-original[uninhabited]],"layout/Copy/drop mismatch"));}
        if tag==19&&original[by_ptr]==1{abi=Some((c_size,by_ptr));}
    }assert_eq!(at,graph_end);
    if let Some(pos)=array{changes.push(("array-bound",pos,u64::MAX.to_le_bytes().to_vec(),"array length bound"));}
    if let Some(pos)=pointer{changes.push(("pointer-slots",pos,vec![1],"function pointer signature bound"));}
    if functions>1{assert!(abi.is_some()&&array.is_some()&&pointer.is_some(),"compound graph must actually exercise passing rules and pointer records");}
    // Duplicate labels from multiple tuple nodes remain distinct operations.
    for(index,(label,pos,replacement,expected))in changes.into_iter().enumerate(){let mut payload=original.to_vec();payload[pos..pos+replacement.len()].copy_from_slice(&replacement);let payload=seal(payload);let label=format!("malformed-{label_prefix}-{index}-{label}");put(&root.join(format!("{label}.lustr-native")),&payload);let error=native_crate::Artifact::decode(&payload,generation).err().expect("malformed graph accepted");put(&root.join(format!("{label}.log")),error.as_bytes());assert!(error.contains(expected),"{label}: {error}");}
    if let Some((c_size,by_ptr))=abi{
        for(label,pos,replacement)in[("abi-size",c_size,123456u64.to_le_bytes().to_vec()),("abi-pointer",by_ptr,vec![0])]{
            let mut payload=original.to_vec();payload[pos..pos+replacement.len()].copy_from_slice(&replacement);let payload=seal(payload);let label=format!("malformed-{label_prefix}-{label}");put(&root.join(format!("{label}.lustr-native")),&payload);
            let artifact=native_crate::Artifact::decode(&payload,generation).unwrap();let store=native_crate::CrateStore::new(vec![("compound".into(),artifact)],request.crate_ids).unwrap();let mut r=request.clone();r.root_text="use compound::api::array; const BAD:i64=1/0; fn main(){}".into();
            let result=store.executable(&r,generation,backend,&root.join(&label));let error=result.err().expect("mismatched effective ABI accepted");put(&root.join(format!("{label}.log")),error.as_bytes());assert!(error.contains("consumer layout/drop/C ABI mismatch"),"{error}");assert!(!root.join(&label).exists(),"ABI check must precede initializer/body/output effects");
        }
    }
}
