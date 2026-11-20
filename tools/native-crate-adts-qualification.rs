//! Native nominal constructors, variant identity and layout composition.
use super::*;
pub(super) const TUPLE:&str="#![no_std]\npub struct Pair(pub i64, bool);\npub fn make(x:i64)->Pair{Pair(x,true)}\npub fn take(x:Pair)->i64{if x.1{x.0+1}else{0}}\n";
pub(super) const UNIT:&str="#![no_std]\npub struct Token;\npub fn make()->Token{Token}\npub fn take(_x:Token)->i64{17}\n";
pub(super) const ENUM:&str="#![no_std]\npub enum Message{Empty,Number(i64),Record{value:i64,ready:bool}}\npub fn make(x:i64)->Message{Message::Number(x)}\npub fn take(x:Message)->i64{match x{Message::Empty=>0,Message::Number(v)=>v,Message::Record{value,ready}=>if ready{value+1}else{0}}}\n";
fn record(root:&std::path::Path,label:&str,o:&std::process::Output){put(&root.join(format!("{label}.stdout")),&o.stdout);put(&root.join(format!("{label}.stderr")),&o.stderr);put(&root.join(format!("{label}.status")),format!("{}",o.status).as_bytes());}
pub(super) fn first(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    for(label,text)in[("tuple",TUPLE),("unit",UNIT),("enum",ENUM)]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));
        let(code,message)=native_object::run_artifact(&args(&source,&image,label),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,1,"{label}: {message}");assert!(!image.exists());
        let o=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-build"),&o);assert!(o.status.success(),"{label}: {}",String::from_utf8_lossy(&o.stderr));
    }
    println!("three original nominal-shape firsts: native refusal/no image/no app; separate SDK oracle rlib builds pass");
}
const MIXED:&str="#![no_std]\npub struct PublicPair(pub i64,pub bool);\npub enum Envelope{Missing,Data([i64;5]),Nested{packet:PublicPair,ready:bool}}\npub enum MaybeFlag{None,Some(bool)}\npub enum Code{Minus=-5,Zero=0,Next}\npub fn wrap(x:i64)->Envelope{Envelope::Nested{packet:PublicPair(x,true),ready:true}}\npub fn sum(x:Envelope)->i64{match x{Envelope::Missing=>0,Envelope::Data(a)=>a[0]+a[4],Envelope::Nested{packet,ready}=>if ready{packet.0}else{0}}}\npub fn flag(x:MaybeFlag)->i64{match x{MaybeFlag::None=>3,MaybeFlag::Some(b)=>if b{5}else{7}}}\npub fn code(x:Code)->i64{x as i64}\npub use Envelope::Data as Payload;\n";
const EMPTY:&str="#![no_std]\npub enum Void{}\npub fn consume(x:Void)->i64{match x{}}\n";
pub(super) fn run(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend){
    let mut images=Vec::new();
    for(label,text,metadata)in[("tuple",TUPLE,"tuple"),("unit",UNIT,"unit"),("message",ENUM,"message"),("other",ENUM,"independent_message"),("mixed",MIXED,"mixed"),("empty",EMPTY,"empty")]{
        let source=root.join(format!("{label}.rs"));put(&source,text.as_bytes());let image=root.join(format!("{label}.lustr-native"));
        let(code,message)=native_object::run_artifact(&args(&source,&image,metadata),generation,backend);put(&root.join(format!("{label}.native.log")),message.as_bytes());put(&root.join(format!("{label}.native.status")),code.to_string().as_bytes());assert_eq!(code,0,"{label}: {message}");
        let bytes=std::fs::read(&image).unwrap();let artifact=native_crate::Artifact::decode(&bytes,generation).unwrap();assert_eq!(artifact.encode().unwrap(),bytes);images.push((label.into(),artifact));
        let oracle=std::process::Command::new("/core/bin/rustc").arg(&source).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","--crate-name","same","--crate-type=rlib","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]).arg(format!("-Cmetadata={metadata}")).arg("-o").arg(root.join(format!("lib{label}.rlib"))).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert!(oracle.status.success(),"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
    }
    let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("adt-consumer","bin","consumer",false)};
    let store=native_crate::CrateStore::new(images,ids).unwrap();
    for(label,text,valid,diagnosis)in[
        ("tuple-return","use tuple::{make,take};fn main(){let p=make(8);assert!(p.0==8);assert!(take(p)==9);}",true,""),
        ("unit-constructor","use unit::{Token,make,take};fn main(){assert!(take(Token)==17);assert!(take(Token{})==17);assert!(take(make())==17);}",true,""),
        ("enum-constructors","use message::{Message,take,make};fn main(){assert!(take(Message::Empty)==0);assert!(take(Message::Number(13))==13);assert!(take(Message::Record{value:7,ready:true})==8);let p:fn(Message)->i64=take;assert!(p(make(4))==4);}",true,""),
        ("variant-imports","use message::Message::*;use message::take;fn main(){assert!(take(Empty)==0);assert!(take(Number(11))==11);assert!(take(Record{value:3,ready:true})==4);}",true,""),
        ("mixed-layouts","use mixed::{PublicPair,Envelope,MaybeFlag,Code,wrap,sum,flag,code,Payload};fn main(){assert!(sum(wrap(13))==13);assert!(sum(Payload([1,2,3,4,7]))==8);assert!(sum(Envelope::Nested{packet:PublicPair(19,true),ready:true})==19);assert!(sum(Envelope::Missing)==0);assert!(flag(MaybeFlag::None)==3);assert!(flag(MaybeFlag::Some(true))==5);assert!(flag(MaybeFlag::Some(false))==7);assert!(code(Code::Minus)==-5);assert!(code(Code::Next)==1);}",true,""),
        ("cross-match","use message::{Message,make};fn read(x:Message)->i64{match x{Message::Empty=>0,Message::Number(v)=>v,Message::Record{value,ready}=>if ready{value}else{0}}}fn main(){assert!(read(make(21))==21);}",true,""),
        ("tuple-private","use tuple::make;fn main(){let p=make(3);let x=p.1;}",false,"private"),
        ("tuple-private-constructor","use tuple::Pair;fn main(){let p=Pair(3,true);}",false,"private"),
        ("tuple-move","use tuple::{make,take};fn main(){let p=make(3);take(p);take(p);}",false,"L0301"),
        ("enum-move","use message::{make,take};fn main(){let p=make(3);take(p);take(p);}",false,"L0301"),
        ("enum-distinct","use message::take;use other::make;fn main(){take(make(3));}",false,"L0110"),
        ("enum-argument","use message::Message;fn main(){let p=Message::Number(true);}",false,"L0110"),
        ("enum-nonexhaustive","use message::Message;fn read(x:Message)->i64{match x{Message::Empty=>0}}fn main(){}",false,"non-exhaustive"),
    ]{
        let file=root.join(format!("{label}.rs"));put(&file,text.as_bytes());let mut request=input(root,text,ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});
        let out=root.join(format!("{label}-native"));let result=store.executable(&request,generation,backend,&out);put(&root.join(format!("{label}.native.result")),format!("{result:?}").as_bytes());
        if valid{result.unwrap_or_else(|e|panic!("{label}: {e}"));}else{let e=result.err().unwrap_or_else(||panic!("invalid {label} emitted native output; NEVER execute"));assert!(!out.join("app").exists());assert!(e.contains(diagnosis),"{label}: expected {diagnosis}: {e}");}
        let mut c=std::process::Command::new("/core/bin/rustc");c.arg(&file).args(["--edition=2024","--target=x86_64-unknown-linux-gnu","-Cpanic=abort","-Copt-level=0","-Cdebuginfo=0","-Coverflow-checks=off","-Cdebug-assertions=off"]);for name in ["tuple","unit","message","other","mixed"]{c.arg("--extern").arg(format!("{name}={}",root.join(format!("lib{name}.rlib")).display()));}let app=root.join(format!("{label}-oracle"));let oracle=c.arg("-o").arg(&app).output().unwrap();record(root,&format!("{label}.oracle-build"),&oracle);assert_eq!(oracle.status.success(),valid,"{label}: {}",String::from_utf8_lossy(&oracle.stderr));
        if valid{let native=std::process::Command::new(out.join("app")).output().unwrap();record(root,&format!("{label}.native-app"),&native);assert!(native.status.success());let oracle=std::process::Command::new(app).output().unwrap();record(root,&format!("{label}.oracle-app"),&oracle);assert!(oracle.status.success());assert_eq!(native.stdout,oracle.stdout);assert_eq!(native.stderr,oracle.stderr);}
    }
    malformed(root,generation,backend,ids);remap(root,generation);
    println!("native ADT gate PASS; real core/alloc/no_std runtime/full Rust/Cargo/SAME complete-work speed remain OPEN");
}

#[derive(Clone)]
struct FieldWire{name:std::ops::Range<usize>,ty:usize,vis:usize}
#[derive(Clone)]
struct VariantWire{name:std::ops::Range<usize>,shape:usize,disc:usize,fields:Vec<FieldWire>}
struct NominalWire{name:String,variants:Vec<VariantWire>,count:usize,layout:usize,copy:usize,abi:usize,enum_tag:Option<usize>,offsets:Vec<usize>}
struct Cursor<'a>{b:&'a[u8],at:usize}
impl Cursor<'_>{
    fn n(&mut self)->usize{let n=u32::from_le_bytes(self.b[self.at..self.at+4].try_into().unwrap())as usize;self.at+=4;n}
    fn byte(&mut self)->u8{let n=self.b[self.at];self.at+=1;n}
    fn text(&mut self)->std::ops::Range<usize>{let n=self.n();let start=self.at;self.at+=n;start..self.at}
    fn vis(&mut self){if self.byte()==4{for _ in 0..self.n(){self.text();}}}
    fn fields(&mut self)->Vec<FieldWire>{(0..self.n()).map(|_|{let name=self.text();let ty=self.at;self.at+=4;let vis=self.at;self.vis();FieldWire{name,ty,vis}}).collect()}
}
fn graph_wire(b:&[u8])->(usize,usize,Vec<NominalWire>){
    let mut c=Cursor{b,at:b"LUSTR-NATIVE-CRATE\0\x08".len()+64};for _ in 0..4{c.text();}c.at+=8+32+2;
    for _ in 0..c.n(){c.at+=16;c.text();c.at+=1;let n=c.n();c.at+=4*(n+1);c.text();}
    let size_at=c.at;let size=c.n();let end=c.at+size;let mut records=Vec::new();
    for _ in 0..c.n(){
        let tag=c.byte();let mut record=None;let mut is_enum=false;
        match tag{
            0..=17=>{},18=>{let n=c.n();c.at+=4*n;},19=>c.at+=12,
            20=>{let n=c.n();c.at+=4*n+9;},21=>{if c.byte()==2{c.at+=4;}c.at+=5;},
            22|23=>{c.at+=16;let name=c.text();let name=String::from_utf8(b[name].to_vec()).unwrap();let mut variants=Vec::new();let count;
                if tag==23{is_enum=c.byte()==1;count=c.at;for _ in 0..c.n(){let name=c.text();let shape=c.at;c.at+=1;let disc=c.at;c.at+=16;let fields=c.fields();variants.push(VariantWire{name,shape,disc,fields});}}
                else{count=c.at;let fields=c.fields();variants.push(VariantWire{name:0..0,shape:0,disc:0,fields});}
                record=Some((name,variants,count));},
            _=>panic!("unexpected fixture graph tag {tag}"),
        }
        let layout=c.at;c.at+=16;let n=c.n();c.at+=8*n;if c.byte()==1{c.at+=48;}c.at+=1;let copy=c.at;c.at+=2;let abi=c.at;c.at+=19;
        let mut offsets=Vec::new();let enum_tag=if is_enum{for _ in 0..c.n(){for _ in 0..c.n(){offsets.push(c.at);c.at+=8;}}let pos=c.at;match c.byte(){0=>{},1=>c.at+=4,2=>c.at+=9,3=>c.at+=4+48+4+4+16,t=>panic!("fixture enum tag {t}")};Some(pos)}else{None};
        if let Some((name,variants,count))=record{records.push(NominalWire{name,variants,count,layout,copy,abi,enum_tag,offsets});}
    }assert_eq!(c.at,end);(size_at,end,records)
}
fn reject(root:&std::path::Path,label:&str,mut payload:Vec<u8>,generation:&native_crate::Generation,diagnosis:&str){
    payload.extend_from_slice(&native_crate::digest(&payload).unwrap());put(&root.join(format!("malformed-{label}.lustr-native")),&payload);
    let e=native_crate::Artifact::decode(&payload,generation).err().expect("malformed ADT image accepted");put(&root.join(format!("malformed-{label}.log")),e.as_bytes());assert!(e.contains(diagnosis),"{label}: expected {diagnosis}: {e}");
}
fn malformed(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    let bytes=std::fs::read(root.join("mixed.lustr-native")).unwrap();let original=&bytes[..bytes.len()-32];let(size_at,end,records)=graph_wire(original);
    let envelope=records.iter().find(|r|r.name=="Envelope").unwrap();let niche=records.iter().find(|r|r.name=="MaybeFlag").unwrap();let tuple=records.iter().find(|r|r.name=="PublicPair").unwrap();
    let mut b=original.to_vec();b[envelope.variants[1].shape]=0;reject(root,"unit-with-fields",b,generation,"variant shape");
    let mut b=original.to_vec();b[envelope.variants[1].disc..envelope.variants[1].disc+16].copy_from_slice(&original[envelope.variants[0].disc..envelope.variants[0].disc+16]);reject(root,"duplicate-discriminant",b,generation,"variant identity/discriminant");
    let old=envelope.variants[1].name.clone();let replacement=&original[envelope.variants[0].name.clone()];let mut b=original[..old.start-4].to_vec();b.extend_from_slice(&(replacement.len()as u32).to_le_bytes());b.extend_from_slice(replacement);b.extend_from_slice(&original[old.end..]);let new_size=end-size_at-4-old.len()+replacement.len();b[size_at..size_at+4].copy_from_slice(&(new_size as u32).to_le_bytes());reject(root,"duplicate-variant",b,generation,"variant identity/discriminant");
    let mut b=original.to_vec();b[envelope.count..envelope.count+4].copy_from_slice(&u32::MAX.to_le_bytes());reject(root,"variant-bound",b,generation,"exceeds bound");
    let mut b=original.to_vec();b[tuple.variants[0].fields[0].name.start]=b'1';reject(root,"tuple-field-order",b,generation,"field identity");
    let mut b=original.to_vec();b[envelope.variants[1].fields[0].ty..envelope.variants[1].fields[0].ty+4].copy_from_slice(&u32::MAX.to_le_bytes());reject(root,"variant-field-cycle",b,generation,"forward/cyclic");
    let mut b=original.to_vec();b[envelope.variants[2].fields[0].vis]=0;reject(root,"enum-field-visibility",b,generation,"field visibility restriction");
    let mut b=original.to_vec();b[envelope.copy]=1;reject(root,"enum-copy",b,generation,"layout/Copy/drop");
    let mut b=original.to_vec();b[envelope.offsets[0]]^=1;reject(root,"variant-layout",b,generation,"layout/Copy/drop");
    let tag=envelope.enum_tag.unwrap();assert_eq!(original[tag],2);let mut b=original.to_vec();b[tag+1]^=1;reject(root,"enum-tag-width",b,generation,"layout/Copy/drop");
    let tag=niche.enum_tag.unwrap();assert_eq!(original[tag],3);let mut b=original.to_vec();b[tag+1]^=1;reject(root,"enum-niche-variant",b,generation,"layout/Copy/drop");let mut b=original.to_vec();b[tag+1+4+16]^=1;reject(root,"enum-niche-free",b,generation,"layout/Copy/drop");
    let mut b=original.to_vec();b[b"LUSTR-NATIVE-CRATE\0".len()]=7;reject(root,"old-format7",b,generation,"format generation");
    // A genuine variant re-export remains a separate namespace binding.
    let ns_size=u32::from_le_bytes(original[end..end+4].try_into().unwrap())as usize;let ns=&original[end+4..end+4+ns_size];let needle=b"\x04\0\0\0Data";let positions:Vec<_>=ns.windows(needle.len()).enumerate().filter_map(|(n,s)|(s==needle).then_some(n)).collect();assert_eq!(positions.len(),2,"native variant binding in both original namespaces");let mut b=original.to_vec();for pos in positions{let pos=end+4+pos+4;b[pos..pos+4].copy_from_slice(b"Nope");}reject(root,"variant-binding-identity",b,generation,"variant target identity");
    late_abi(root,generation,backend,ids);
}
fn late_abi(root:&std::path::Path,generation:&native_crate::Generation,backend:&native_crate::Backend,ids:def::CrateIds){
    let left=std::fs::read(root.join("message.lustr-native")).unwrap();let right=std::fs::read(root.join("other.lustr-native")).unwrap();let a=native_crate::Artifact::decode(&left,generation).unwrap();let b=native_crate::Artifact::decode(&right,generation).unwrap();
    assert!(a.runtime_requirements().imports.is_empty()&&b.runtime_requirements().imports.is_empty());
    let(good,target)=if a.functions()[0].id.krate<b.functions()[0].id.krate{(a,right)}else{(b,left)};let payload=&target[..target.len()-32];let(_,_,records)=graph_wire(payload);let record=records.iter().find(|r|r.name=="Message").unwrap();let mut changed=payload.to_vec();changed[record.abi+17]^=1;changed.extend_from_slice(&native_crate::digest(&changed).unwrap());put(&root.join("malformed-later-enum-abi.lustr-native"),&changed);let bad=native_crate::Artifact::decode(&changed,generation).unwrap();assert!(good.functions()[0].id.krate<bad.functions()[0].id.krate);
    let store=native_crate::CrateStore::new(vec![("good".into(),good),("bad".into(),bad)],ids).unwrap();let i=std::sync::Arc::new(intern::Interner::default());let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let tables=std::sync::Arc::new(resolved.tables);let foreign:Vec<_>=tables.def_info.keys().filter(|d|d.krate!=def::LOCAL_CRATE).copied().collect();assert!(!foreign.is_empty());let mut order=check::collect::DefOrder::default();let mut called=false;
    let mut install=|p:&mut check::collect::Program|{called=true;let e=store.install_signatures(p).unwrap_err();assert!(e.contains("C ABI mismatch"),"{e}");for d in &foreign{assert!(p.defs.get(*d).is_none());assert!(!p.type_names.contains_key(&tables.def_info[d].flat));if let Some(a)=p.adt_by_def(*d){let fields:Vec<Vec<_>>=p.adt(a).variants.iter().map(|v|v.fields.iter().map(|f|f.ty).collect()).collect();assert_eq!(p.regions.adt_fields[d],fields);}}assert!(p.regions.native_fns.is_empty());assert!(p.consts.is_empty());Err(vec![diag::Diag::new("L0508",diag::Span::default(),e,"Expected later-image pre-definition ABI refusal; original fields stage only")])};
    let result=check::collect::collect_with_definitions(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(tables.clone()),&mut install);assert!(called);let mut errors=result.err().expect("bad ABI must stop collection");let rendered=diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);assert!(!rendered.contains("division by zero"));put(&root.join("later-enum-abi-before-definitions.log"),rendered.as_bytes());
    let mut request=input(root,"const FAIL:i64=1/0;fn main(){}",ids);request.env=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false});let out=root.join("malformed-later-enum-abi-native");let e=store.executable(&request,generation,backend,&out).unwrap_err();assert!(e.contains("C ABI mismatch"));assert!(!out.exists());put(&root.join("later-enum-abi-executable.log"),e.as_bytes());
}
fn remap(root:&std::path::Path,generation:&native_crate::Generation){
    use ty::TyIntern;let artifact=native_crate::Artifact::decode(&std::fs::read(root.join("message.lustr-native")).unwrap(),generation).unwrap();let ids=def::CrateIds{local:def::LOCAL_CRATE,stable:def::stable_crate_hash("perturbed-adt-consumer","bin","consumer",false)};let store=native_crate::CrateStore::new(vec![("message".into(),artifact)],ids).unwrap();
    let i=std::sync::Arc::new(intern::Interner::default());for n in 0..200{i.intern(&format!("adt_noise_{n}"));}let k=std::sync::Arc::new(intern::Known::new(&i));let request=input(root,"struct Local{v:i64}fn local(x:&i64)->&i64{x}fn main(){}",ids);let resolved=resolve_ok(&store,&request,&i,&k);let mut order=check::collect::DefOrder::default();let mut p=check::collect::collect_with_order_resolved(std::sync::Arc::new(resolved.ast),i,k,ids,false,&mut order,Some(std::sync::Arc::new(resolved.tables))).unwrap_or_else(|e|panic!("remap collect errors {}",e.len()));let fns=p.fns.len();let regions=p.regions.fns.len();let adts=p.adts.len();for n in 101..133{let c=p.tys.intern_const(ty::ConstKind::Int(n));p.tys.intern(ty::TyKind::Array(ty::I32,c));}store.install_signatures(&mut p).unwrap();assert_eq!(p.fns.len(),fns);assert_eq!(p.regions.fns.len(),regions);assert_eq!(p.adts.len(),adts+1);let id=ty::AdtId(adts as u32);let def=p.def_of_adt(id);assert_eq!(p.defs.get(def).unwrap().kind,ty::defs::DefKind::Enum);let a=p.adt(id);assert!(a.is_enum);assert_eq!(a.variants.len(),3);assert_eq!(p.name(a.variants[1].name),"Number");assert_eq!(a.variants[1].disc,1);let t=p.tys.get_adt_ty(id).unwrap();assert!(!check::collect::is_copy(&p,&p.tys,t));assert!(!check::collect::needs_drop(&p,&p.tys,t));assert_eq!(p.regions.adt_fields[&def].len(),3);put(&root.join("remap-install-only.txt"),b"Actual enum records retain variants/discriminants/fields after fresh interner and existing local Adt/type/region tables; no FnInfo/local region rows fabricated. INSTALL ONLY; no application executed from this Program.\n");
}
