//! Named late-bound function lifetimes use the original native region engine.
use super::*;
const ORIGINAL:&str="#![no_std]\npub fn choose<'a,'b>(x:&'a i64,y:&'b i64)->&'a i64{let _=y;x}\n";
const PROVIDER:&str="#![no_std]\npub fn id<'a>(x:&'a i64)->&'a i64{x}\npub fn mutate<'a>(x:&'a mut i64)->&'a mut i64{*x+=3;x}\npub fn swap<'a,'b>(x:&'a i64,y:&'b i64)->(&'b i64,&'a i64){(y,x)}\npub fn tied<'a>(x:&'a i64,y:&'a i64)->&'a i64{let _=y;x}\npub fn adapt<'a>(f:fn(&i64)->&i64,x:&'a i64)->&'a i64{f(x)}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),o.status.to_string().as_bytes());}
fn oracle(root:&std::path::Path,source:&std::path::Path,out:&std::path::Path,name:Option<&str>)->std::process::Output{
    let mut c=std::process::Command::new("/core/bin/rustc");c.arg(source).args(["--edition=2024","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);
    if let Some(name)=name{c.args(["--crate-name","same","--crate-type=rlib"]).arg(format!("-Cmetadata={name}"));}else{for name in["original","provider"]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}}
    if name.is_none(){c.arg("--extern").arg(format!("again={}",root.join("liboriginal.rlib").display()));}
    c.arg("-o").arg(out).output().unwrap()
}
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,first:bool){
    let source=root.join("original.rs");put(&source,ORIGINAL.as_bytes());let image=root.join("original.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&source,&image,"named_original"),generation,backend);
    put(&root.join("original.native.log"),message.as_bytes());put(&root.join("original.native.status"),code.to_string().as_bytes());
    if first{assert_eq!(code,1,"{message}");assert!(!image.exists());println!("original named late-bound producer native refusal retained; no image/app");return;}
    assert_eq!(code,0,"{message}");
    let bytes=std::fs::read(&image).unwrap();let original=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(original.encode().unwrap(),bytes);
    let o=oracle(root,&source,&root.join("liboriginal.rlib"),Some("original"));record(root,"original.oracle-build",&o);assert!(o.status.success());
    let provider_source=root.join("provider.rs");let provider_text=format!("{PROVIDER}pub fn mix<'a>(x:&'a i64,y:&i64)->&'a i64{{let _=y;x}}\npub fn requires_static<'a>(x:&'a i64,y:&'static i64){{let _=x;let _=y;}}\n");put(&provider_source,provider_text.as_bytes());let image=root.join("provider.lustr-native");
    let(code,message)=native_object::run_artifact(&args(&provider_source,&image,"named_provider"),generation,backend);put(&root.join("provider.native.log"),message.as_bytes());assert_eq!(code,0,"{message}");
    let bytes=std::fs::read(&image).unwrap();let provider=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(provider.encode().unwrap(),bytes);
    let o=oracle(root,&provider_source,&root.join("libprovider.rlib"),Some("provider"));record(root,"provider.oracle-build",&o);assert!(o.status.success());
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("named-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(vec![("original".into(),original.clone()),("again".into(),original),("provider".into(),provider)],ids).unwrap();
    for(label,text,valid,diagnosis)in[
        ("independent","fn main(){let x=10i64;let r;{let y=20i64;r=original::choose(&x,&y);}assert!(*r==10);let z=7i64;let p=provider::swap(&x,&z);assert!(*p.0==7);assert!(*p.1==10);}",true,""),
        ("pointer","fn main(){let p:fn(&i64)->&i64=provider::id;let q=p;let x=8i64;assert!(*q(&x)==8);assert!(*provider::adapt(q,&x)==8);}",true,""),
        ("mutable","fn main(){let mut x=2i64;let r=provider::mutate(&mut x);*r+=4;assert!(x==9);}",true,""),
        ("local-named","fn local<'a>(x:&'a i64)->&'a i64{provider::id(x)}fn main(){let x=6i64;assert!(*local(&x)==6);assert!(*again::choose(&x,&x)==6);}",true,""),
        ("mixed-named-elided","fn main(){let x=13i64;let r;{let y=2i64;r=provider::mix(&x,&y);}assert!(*r==13);}",true,""),
        ("escape","fn main(){let r;let x=10i64;{let y=20i64;r=original::choose(&y,&x);}assert!(*r==20);}",false,"L0306"),
        ("alias","fn main(){let mut x=2i64;let a=provider::mutate(&mut x);let b=provider::mutate(&mut x);*a+=*b;}",false,"L0304"),
        ("static","fn main(){let x=7i64;let r:&'static i64=provider::id(&x);assert!(*r==7);}",false,"L0306"),
        ("generality","fn main(){let p:fn(&i64,&i64)=provider::requires_static;let _=p;}",false,"L0110"),
        // This independently invalid coercion reaches an earlier unsupported
        // pointer grammar. Preserve it as a frontier, not a generality proof.
        ("unsupported-output-binder","fn main(){let p:for<'a,'b> fn(&'a i64,&'b i64)->&'a i64=provider::tied;let _=p;}",false,"L0203"),
        ("nonconst","use provider::id;const fn bad<'a>(x:&'a i64)->&'a i64{id(x)}fn main(){}",false,"non-const"),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let native=root.join(format!("native-{label}"));let result=store.executable(&request,generation,backend,&native);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());
        if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.err().unwrap_or_else(||panic!("INVALID {label} emitted artifact; NEVER execute"));assert!(!native.join("app").exists());assert!(e.contains(diagnosis),"{label}: {e}");}
        let o=oracle(root,&source,&root.join(format!("oracle-{label}")),None);record(root,&format!("{label}.oracle-build"),&o);assert_eq!(o.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&o.stderr));
        if valid{let n=std::process::Command::new(native.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&n);assert!(n.status.success());let o=std::process::Command::new(root.join(format!("oracle-{label}"))).output().unwrap();record(root,&format!("{label}.oracle-app"),&o);assert!(o.status.success());assert_eq!(n.stdout,o.stdout);assert_eq!(n.stderr,o.stderr);}
    }
    for(label,text,valid)in[
        ("early-unused","#![no_std]\npub fn f<'a>(x:i64)->i64{x}\n",true),
        ("bounded","#![no_std]\npub fn f<'a:'b,'b>(x:&'a i64)->&'b i64{x}\n",true),
        ("type-generic","#![no_std]\npub fn f<T>(x:T)->T{x}\n",true),
        ("invalid-return","#![no_std]\npub fn f<'a,'b>(x:&'a i64,y:&'b i64)->&'a i64{let _=x;y}\n",false),
    ]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));let(code,message)=native_object::run_artifact(&args(&source,&image,label.replace('-',"_").as_str()),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());let represented=matches!(label,"early-unused"|"bounded");assert_eq!(code,if represented{0}else{1},"{message}");assert_eq!(image.exists(),represented);if !valid{assert!(message.contains("L0733"),"{message}");}
        let o=oracle(root,&source,&root.join(format!("lib{label}.rlib")),Some(&label.replace('-',"_")));record(root,&format!("{label}.oracle-build"),&o);assert_eq!(o.status.success(),valid);
    }
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("named_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));
    let resolved=resolve_ok(&store,&input(root,"fn local<'a>(x:&'a i64)->&'a i64{x}fn main(){}",ids),&i,&k);let mut order=check::collect::DefOrder::default();
    let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("named remap errors {}",e.len()));let fns=p.fns.len();let local=p.regions.fns.len();
    store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),local);
    use ty::TyIntern;assert!(p.regions.native_fns.values().any(|s|s.n_univ==2&&s.early==[false,false]&&s.params.len()==2&&matches!(p.tys.kind(s.params[0]),ty::TyKind::Ref(ty::Region::Param(0),_,_))&&matches!(p.tys.kind(s.params[1]),ty::TyKind::Ref(ty::Region::Param(1),_,_))&&matches!(p.tys.kind(s.ret),ty::TyKind::Ref(ty::Region::Param(0),_,_))));
    assert!(p.regions.native_fns.values().any(|s|s.n_univ==2&&s.early==[false]&&s.params.len()==2&&matches!(p.tys.kind(s.params[1]),ty::TyKind::Ref(ty::Region::Param(1),_,_))));
    println!("named late-bound region images: unchanged original producer, native/oracle runtime parity and actual escape/alias/static/generality/const rejection; exact remap without AST/FnInfo; early/bounded declarations have separate qualification, type-generic frontier remains OPEN");
}
