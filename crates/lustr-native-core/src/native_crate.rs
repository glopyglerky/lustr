//! Experimental typed native artifact and early resolver import.
//! Format sixteen carries symbolic type/const nominal templates and typed
//! concrete applications without inventing an uninstantiated layout. General
//! predicates and bodies still need their own records.
//! with explicit original runtime dependencies and locations. It is not rustc metadata, a Rust
//! ABI promise. Imported calls use the explicit native signature and
//! linkage interface. No source or AST declarations are synthesized.
use crate::{ast, check, def, diag, intern, resolve, ty, unit_object};
use std::{collections::BTreeSet, io::Write, path::Path, process::{Command, Stdio}};
use ty::TyIntern;

const MAGIC: &[u8] = b"LUSTR-NATIVE-CRATE\0\x10";
#[path = "native_namespace.rs"] mod namespace;
#[path = "native_types.rs"] mod types;
#[path = "native_regions.rs"] mod regions;
#[path = "native_nominals.rs"] mod nominals;
#[path = "native_generic_nominals.rs"] mod generic_nominals;
#[path = "native_generic_applications.rs"] mod generic_applications;
#[path = "native_consts.rs"] mod consts;
#[path = "native_runtime.rs"] pub(crate) mod runtime;
const MAX_IMAGE: usize = 72 * 1024 * 1024;
const MAX_DEFS: usize = 4096;
const MAX_PARAMS: usize = 64;

pub(crate) fn names(p:&check::collect::Program)->std::sync::Arc<crate::cgen::names::Names>{
    std::sync::Arc::new(crate::cgen::names::Names{fns:p.fns.iter().enumerate().map(|(n,_)|Some(format!("{:016x}",p.disk_id(def::LocalDefId(n as u32)).hash.0))).collect(),adts:(0..p.adts.len()).map(|n|{let Some(d)=p.defs.def_of_adt(ty::AdtId(n as u32))else{return format!("inert{n}");};if let Some((def,args))=p.gx.adt_instance_of.get(n).copied().flatten(){format!("c{}i{:016x}",d.krate.0,p.instance_key_hash(&crate::driver::graph::InstanceKey{def,args}).0)}else{format!("c{}d{:016x}",d.krate.0,d.hash.0)}}).collect(),by_id:Default::default()})
}
pub(crate) fn signature_type(p:&check::collect::Program,t:ty::Ty)->bool {types::supported(p,&p.tys,t,0)}
pub(crate) fn function_declarations(p:&check::collect::Program,id:def::LocalDefId)->bool {regions::declarations(p,id)}
pub(crate) fn check_initializers(p:&mut check::collect::Program,errors:&mut Vec<diag::Diag>){consts::check_initializers(p,errors)}
pub(crate) fn prepare_declared_types(p:&mut check::collect::Program)->Result<(),String>{
    for(n,g)in p.gx.adt_generics.iter().enumerate(){if !g.params.is_empty()&&p.gx.adt_instance_of[n].is_none()&&p.defs.def_of_adt(ty::AdtId(n as u32)).is_some_and(|d|d.krate==p.krate){generic_nominals::Template::capture(p,ty::AdtId(n as u32))?;}}
    let types=p.fns.iter().flat_map(|f|f.params.iter().copied().chain([f.ret])).chain(p.consts.iter().map(|c|c.ty)).collect::<Vec<_>>();
    check::generics::materialize_native_adt_types(p,&types);Ok(())
}
pub(crate) fn consumer_type(p:&check::collect::Program,tys:&impl TyIntern,t:ty::Ty,depth:usize)->bool {
    if depth>128{return false;}
    match tys.kind(t) {
        // These are original checked consumer-local types, not decoded image
        // types. Their complete native binder/region facts stay in BodyTys and
        // the original pointer proof. Image types validate their own records.
        ty::TyKind::FnPtr(id)=>{let s=tys.sig(id);let v=s.value;s.count<=64&&!v.is_unsafe&&!v.extern_c&&v.free_slots==0&&v.free_region==ty::Region::Erased
            &&tys.tys_of(v.inputs).len()<=MAX_PARAMS&&tys.tys_of(v.inputs).iter().chain([&v.output]).all(|&t|consumer_type(p,tys,t,depth+1))},
        ty::TyKind::FnDef(def,args)=>if tys.args(args).is_empty(){p.native_fn(def).is_some()||p.adt_by_def(def).is_some()||p.local_def(def.hash).is_some_and(|f|p.def_id(f)==def)}else{
            def.krate!=p.krate&&p.defs.local_adt(def).is_some_and(|id|!p.gx.adt_generics[id.0 as usize].params.is_empty())&&tys.args(args).iter().all(|g|match g{ty::GenericArg::Ty(t)=>consumer_type(p,tys,*t,depth+1),ty::GenericArg::Const(c)=>matches!(tys.const_kind(*c),ty::ConstKind::Int(_)|ty::ConstKind::Bool(_)|ty::ConstKind::Char(_)),_=>false})
        },
        ty::TyKind::Ref(_,_,t)|ty::TyKind::Slice(t)|ty::TyKind::Array(t,_)=>consumer_type(p,tys,t,depth+1),
        ty::TyKind::Tuple(ts)=>tys.tys_of(ts).iter().all(|&t|consumer_type(p,tys,t,depth+1)),
        ty::TyKind::Str=>true,
        ty::TyKind::Adt(d,a)if d.krate!=p.krate&&p.defs.local_adt(d).is_some_and(|id|!p.gx.adt_generics[id.0 as usize].params.is_empty())=>{
            // These are freshly checked original consumer applications of an
            // imported template; image signatures still require concrete
            // native layout/ABI records through `signature_type`.
            tys.local_adt_instance(d,a).is_some()&&tys.args(a).iter().all(|g|match g{ty::GenericArg::Ty(t)=>consumer_type(p,tys,*t,depth+1),ty::GenericArg::Const(c)=>matches!(tys.const_kind(*c),ty::ConstKind::Int(_)|ty::ConstKind::Bool(_)|ty::ConstKind::Char(_)),_=>false})
        },
        _=>types::supported(p,tys,t,depth),
    }
}

/// The image fixes one native C ABI. Unknown upstream release/
/// LLVM generation overrides are not represented in that image. Keep the
/// process environment fixed; effective cached ABI is checked independently.
pub(crate) fn fixed_abi()->Result<(),String>{
    use std::os::unix::ffi::OsStrExt;
    if std::env::vars_os().any(|(name,_)|name.as_bytes().starts_with(b"LUST_RELEASE_")||name.as_bytes().starts_with(b"LUST_LL_")) {
        return Err("native type image does not admit release/LLVM ABI overrides".into());
    }Ok(())
}

/// Actual compiler artifact and C toolchain identities are supplied by the
/// bootstrapping caller, which must hash those files. This internal component
/// has no production/Cargo admission path yet. The native runtime build hash
/// is checked independently as well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Generation { pub compiler: [u8;32], pub backend: [u8;32] }

pub(crate) struct Backend { executable: std::path::PathBuf, executable_digest:[u8;32], linker:std::path::PathBuf, linker_digest:[u8;32], objcopy:std::path::PathBuf,objcopy_digest:[u8;32], digest: [u8;32] }
impl Backend {
    pub(crate) fn select(path:&Path)->Result<Self,String>{
        if !path.is_absolute(){return Err("native artifact backend must be an explicit absolute executable".into());}
        let executable=path.canonicalize().map_err(|e|e.to_string())?;
        let executable_digest=file_digest(&executable)?;
        // Preserve the driver's original invocation name: a canonical ld
        // symlink can name generic lld, whose argv[0] selects another mode.
        // Hash the same followed file before/after every tool invocation.
        let linker=std::path::PathBuf::from("/usr/bin/ld");let linker_digest=file_digest(&linker)?;
        let objcopy=std::path::PathBuf::from("/usr/bin/objcopy");let objcopy_digest=file_digest(&objcopy)?;
        let digest=digest(&[executable_digest,linker_digest,objcopy_digest].concat())?;
        Ok(Self{executable,executable_digest,linker,linker_digest,objcopy,objcopy_digest,digest})
    }
    pub(crate) fn digest(&self)->[u8;32]{self.digest}
    pub(crate) fn executable(&self)->&Path{&self.executable}
    pub(crate) fn linker(&self)->&Path{&self.linker}
    pub(crate) fn objcopy(&self)->&Path{&self.objcopy}
    pub(crate) fn verify(&self,generation:&Generation)->Result<(),String>{
        if generation.backend!=self.digest||file_digest(&self.executable)?!=self.executable_digest||file_digest(&self.linker)?!=self.linker_digest||file_digest(&self.objcopy)?!=self.objcopy_digest{return Err("native artifact selected backend changed or differs from its recorded generation".into());}Ok(())
    }
}

// Upstream sha256sum is an explicit host dependency of this experimental
// format, not owned shell logic or a claim of a native cryptography provider.
pub(crate) fn digest(bytes: &[u8]) -> Result<[u8;32],String> {
    let mut child = Command::new("/usr/bin/sha256sum").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().map_err(|e|format!("cannot start artifact digest: {e}"))?;
    let written = child.stdin.take().unwrap().write_all(bytes);
    let out = child.wait_with_output().map_err(|e|e.to_string())?;
    written.map_err(|e|e.to_string())?;
    if !out.status.success() || out.stdout.len()!=68 { return Err("artifact digest failed".into()); }
    let mut hash=[0;32];
    for (slot,pair) in hash.iter_mut().zip(out.stdout[..64].chunks_exact(2)) {
        let text=std::str::from_utf8(pair).map_err(|e|e.to_string())?;
        *slot=u8::from_str_radix(text,16).map_err(|e|e.to_string())?;
    }
    Ok(hash)
}
pub(crate) fn file_digest(path: &Path) -> Result<[u8;32],String> {
    // File size is bounded before allocation. Callers retain the file through
    // compilation; this is an identity observation, not a filesystem lease.
    let file=std::fs::File::open(path).map_err(|e|e.to_string())?;
    if file.metadata().map_err(|e|e.to_string())?.len()>512*1024*1024 {return Err("compiler identity exceeds bound".into());}
    use std::io::Read;
    let mut bytes=Vec::new();file.take(512*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len()>512*1024*1024 {return Err("compiler identity grew beyond bound".into());}
    digest(&bytes)
}

/// Stable on-disk type tags. Dense native Ty/ArgsId/Sym values never enter
/// this format. Lifetimes, ADTs, traits, generics and layouts need their own
/// semantic records; they are refused rather than erased into this schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scalar { Unit, Bool, Char, Never, Int(ty::IntTy) }
impl Scalar {
    fn from_ty(p: &check::collect::Program, t: ty::Ty) -> Result<Self,String> {
        match p.tys.kind(t) {
            ty::TyKind::Unit=>Ok(Self::Unit), ty::TyKind::Bool=>Ok(Self::Bool),
            ty::TyKind::Char=>Ok(Self::Char), ty::TyKind::Never=>Ok(Self::Never),
            ty::TyKind::Int(t)=>Ok(Self::Int(t)),
            _=>Err("native crate signature needs an unimplemented type/lifetime/layout record".into()),
        }
    }
    fn tag(self)->u8 {use ty::IntTy::*; match self {
        Self::Unit=>0,Self::Bool=>1,Self::Char=>2,Self::Never=>3,
        Self::Int(t)=>match t {I8=>4,I16=>5,I32=>6,I64=>7,Isize=>8,U8=>9,U16=>10,U32=>11,U64=>12,Usize=>13,I128=>14,U128=>15},
    }}
    fn read(tag:u8)->Result<Self,String>{use ty::IntTy::*;Ok(match tag {
        0=>Self::Unit,1=>Self::Bool,2=>Self::Char,3=>Self::Never,
        4=>Self::Int(I8),5=>Self::Int(I16),6=>Self::Int(I32),7=>Self::Int(I64),8=>Self::Int(Isize),
        9=>Self::Int(U8),10=>Self::Int(U16),11=>Self::Int(U32),12=>Self::Int(U64),13=>Self::Int(Usize),
        14=>Self::Int(I128),15=>Self::Int(U128),_=>return Err("unknown native type tag".into()),
    })}
    fn intern(self,tys:&mut ty::GlobalTys)->ty::Ty {tys.intern(match self {
        Self::Unit=>ty::TyKind::Unit,Self::Bool=>ty::TyKind::Bool,Self::Char=>ty::TyKind::Char,
        Self::Never=>ty::TyKind::Never,Self::Int(t)=>ty::TyKind::Int(t),
    })}
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Function {
    pub id: def::DiskDefId, pub name: String, pub public: bool,
    pub params: Vec<u32>, pub ret: u32, pub symbol: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Artifact {
    name: String, metadata: String, stable: def::StableCrateHash,
    generation: Generation, source: [u8;32], overflow: bool, debug: bool,
    functions: Vec<Function>, types: types::Graph, constants:Vec<consts::Record>, namespace: namespace::Image, regions:Vec<regions::Signature>, runtime:runtime::Requirements, object: Vec<u8>,
}
fn ident(name:&str)->bool { !name.is_empty() && name.len()<=128 && name.bytes().enumerate().all(|(n,b)| b==b'_'||b.is_ascii_alphabetic()||(n>0&&b.is_ascii_digit())) }
fn metadata(value:&str)->bool{!value.is_empty()&&value.len()<=128&&value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')}
fn text(out:&mut Vec<u8>,s:&str){out.extend_from_slice(&(s.len() as u32).to_le_bytes());out.extend_from_slice(s.as_bytes());}
fn blob(out:&mut Vec<u8>,s:&[u8]){out.extend_from_slice(&(s.len() as u32).to_le_bytes());out.extend_from_slice(s);}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a>{
    fn bytes(&mut self,n:usize)->Result<&'a [u8],String>{let (a,b)=self.0.split_at_checked(n).ok_or("truncated native crate")?;self.0=b;Ok(a)}
    fn byte(&mut self)->Result<u8,String>{Ok(self.bytes(1)?[0])}
    fn len(&mut self,max:usize)->Result<usize,String>{let n=u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()) as usize;if n>max {return Err("native crate field exceeds bound".into());}Ok(n)}
    fn text(&mut self,max:usize)->Result<String,String>{let n=self.len(max)?;String::from_utf8(self.bytes(n)?.to_vec()).map_err(|e|e.to_string())}
    fn boolean(&mut self)->Result<bool,String>{match self.byte()?{0=>Ok(false),1=>Ok(true),_=>Err("invalid native boolean".into())}}
    fn hash(&mut self)->Result<[u8;32],String>{Ok(self.bytes(32)?.try_into().unwrap())}
}
impl Artifact {
    /// Called only after the actual object's complete native checks succeed.
    pub(crate) fn produce(p:&check::collect::Program,name:&str,metadata:&str,source:&str,object:Vec<u8>,generation:Generation,runtime:runtime::Requirements)->Result<Self,String>{
        if !ident(name)||!self::metadata(metadata)||p.krate!=def::LOCAL_CRATE||p.fns.is_empty()&&p.consts.is_empty()||p.fns.len()>MAX_DEFS {return Err("native crate identity/definition bound".into());}
        let mut functions=Vec::new();let mut types=types::Graph::default();let mut regions=Vec::new();let names=names(p);
        crate::cgen::names::with(Some(&names),||types.capture_definitions(p))?;
        for (n,f) in p.fns.iter().enumerate(){
            let check::collect::FnLoc::Item(index)=f.loc else{return Err("native crate needs nested/generic definition records".into())};
            let item=&p.ast.items[index as usize];let written=f.def(&p.ast);
            if written.qual!=ast::FnQual::default()||!function_declarations(p,def::LocalDefId(n as u32))||f.owner.is_some()||f.params.len()>MAX_PARAMS {return Err("native crate needs qualifier/generic/body records".into());}
            let public=matches!(item.vis,ast::Vis::Public);
            let id=p.disk_id(def::LocalDefId(n as u32));
            let (params,ret)=crate::cgen::names::with(Some(&names),||Ok::<_,String>((f.params.iter().map(|&t|types.capture(p,t)).collect::<Result<_,_>>()?,types.capture(p,f.ret)?)))?;
            functions.push(Function{id,name:p.name(f.name).into(),public,params,ret,symbol:format!("f_{:016x}",id.hash.0)});
            regions.push(crate::cgen::names::with(Some(&names),||regions::Signature::capture(p,def::LocalDefId(n as u32),&mut types))?);
        }
        let constants=crate::cgen::names::with(Some(&names),||consts::capture(p,&mut types))?;
        if p.ast.items.iter().any(|item|!matches!(item.kind,ast::ItemKind::Fn(_)|ast::ItemKind::Struct(_)|ast::ItemKind::Enum(_)|ast::ItemKind::Const(_))&&matches!(item.vis,ast::Vis::Public)) {return Err("native crate needs exported non-function records".into());}
        let namespace=namespace::Image::capture(p)?;
        let artifact=Self{name:name.into(),metadata:metadata.into(),stable:p.stable_crate,generation,source:digest(source.as_bytes())?,overflow:p.overflow_checks,debug:p.debug_assertions,functions,types,constants,namespace,regions,runtime,object};
        artifact.validate()?;Ok(artifact)
    }
    fn validate(&self)->Result<(),String>{
        self.types.validate()?;
        if self.types.nominals().iter().any(|r|r.id.krate!=self.stable){return Err("native nominal crate identity mismatch".into());}
        let mut tys=ty::GlobalTys::new();let types=self.types.intern(&mut tys)?;
        if self.regions.len()!=self.functions.len(){return Err("native region/function count mismatch".into());}
        let identity=format!("lustr-native-object-v1/{}/{}/x86_64-unknown-linux-gnu/abort/opt0/debug0/overflow{}/assert{}",self.name,self.metadata,self.overflow,self.debug);
        if !ident(&self.name)||!metadata(&self.metadata)||self.functions.is_empty()&&self.constants.is_empty()||self.functions.len()>MAX_DEFS||self.stable!=def::stable_crate_hash(&identity,"lib",&self.name,false){return Err("native crate identity mismatch".into());}
        let symbols=unit_object::symbols(&self.object)?;
        self.runtime.validate(&self.functions,&self.object)?;
        let mut ids=BTreeSet::new();let mut names=BTreeSet::new();
        for f in &self.functions {
            if f.name.len()>16384||f.params.len()>MAX_PARAMS||!f.params.iter().chain([&f.ret]).all(|&id|self.types.contains(id))||f.id.krate!=self.stable||f.id.hash!=def::free_fn_hash(self.stable,&[],&f.name)
                ||!ids.insert(f.id)||!names.insert(&f.name)||f.symbol!=format!("f_{:016x}",f.id.hash.0)||!symbols.functions.contains(f.symbol.as_bytes())
            {return Err("native crate definition/type/code identity mismatch".into());}
        }
        for(f,r)in self.functions.iter().zip(&self.regions){r.install(&mut tys,&self.types,&types,&f.params,f.ret)?;}
        consts::validate(&self.constants,self.stable,&self.types)?;
        self.namespace.validate(self.stable,&self.functions,&self.types.nominals(),&self.constants)?;
        if !symbols.functions.contains(format!("lustr_unit_register_{:016x}",self.stable.0).as_bytes()){return Err("native crate lacks its real unit registration".into());}
        Ok(())
    }
    pub(crate) fn encode(&self)->Result<Vec<u8>,String>{
        self.validate()?;let mut out=MAGIC.to_vec();
        out.extend_from_slice(&self.generation.compiler);out.extend_from_slice(&self.generation.backend);text(&mut out,env!("LUST_BUILD_HASH"));
        // Exact supported target/profile/ABI/pipeline; unknown generations and
        // extensions refuse. Empty deps/features/cfg/build reads are factual
        // only for the existing closed primitive producer, not general Cargo.
        text(&mut out,"x86_64-unknown-linux-gnu/edition2024/lib/abort/opt0/debug0/native-adts15-runtime-namespace/deps0/features0/cfg0/buildreads0");
        text(&mut out,&self.name);text(&mut out,&self.metadata);out.extend_from_slice(&self.stable.0.to_le_bytes());out.extend_from_slice(&self.source);
        out.extend_from_slice(&[self.overflow as u8,self.debug as u8]);out.extend_from_slice(&(self.functions.len() as u32).to_le_bytes());
        for f in &self.functions {out.extend_from_slice(&f.id.to_bytes());text(&mut out,&f.name);out.push(f.public as u8);out.extend_from_slice(&(f.params.len() as u32).to_le_bytes());for id in &f.params{out.extend_from_slice(&id.to_le_bytes());}out.extend_from_slice(&f.ret.to_le_bytes());text(&mut out,&f.symbol);}
        blob(&mut out,&self.types.encode());
        let namespace=self.namespace.encode();if namespace.len()>8*1024*1024{return Err("native namespace image exceeds bound".into());}blob(&mut out,&namespace);
        let mut regions=Vec::new();regions.extend_from_slice(&(self.regions.len() as u32).to_le_bytes());for r in &self.regions{r.encode(&mut regions);}if regions.len()>4*1024*1024{return Err("native region image bound".into());}blob(&mut out,&regions);
        let runtime=self.runtime.encode();if runtime.len()>8*1024*1024{return Err("native runtime image bound".into());}blob(&mut out,&runtime);
        blob(&mut out,&self.object);
        let constants=consts::encode(&self.constants);if constants.len()>4*1024*1024{return Err("native constants image bound".into());}blob(&mut out,&constants);
        if out.len()>MAX_IMAGE-32{return Err("native crate image exceeds bound".into());}
        out.extend_from_slice(&digest(&out)?);Ok(out)
    }
    pub(crate) fn decode(bytes:&[u8],expected:&Generation)->Result<Self,String>{
        if bytes.len()<MAGIC.len()+32||bytes.len()>MAX_IMAGE{return Err("native crate image bound".into());}
        let (payload,checksum)=bytes.split_at(bytes.len()-32);if digest(payload)?.as_slice()!=checksum{return Err("native crate checksum mismatch".into());}
        let mut r=Reader(payload);if r.bytes(MAGIC.len())?!=MAGIC{return Err("native crate format generation mismatch".into());}
        let generation=Generation{compiler:r.hash()?,backend:r.hash()?};if &generation!=expected||r.text(128)?!=env!("LUST_BUILD_HASH"){return Err("native crate compiler/backend/runtime generation mismatch".into());}
        if r.text(256)?!="x86_64-unknown-linux-gnu/edition2024/lib/abort/opt0/debug0/native-adts15-runtime-namespace/deps0/features0/cfg0/buildreads0"{return Err("native crate target/profile/pipeline mismatch".into());}
        let name=r.text(128)?;let metadata=r.text(128)?;let stable=def::StableCrateHash(u64::from_le_bytes(r.bytes(8)?.try_into().unwrap()));let source=r.hash()?;
        let overflow=r.boolean()?;let debug=r.boolean()?;let count=r.len(MAX_DEFS)?;let mut functions=Vec::with_capacity(count);
        for _ in 0..count {let id=def::DiskDefId::from_bytes(r.bytes(16)?).unwrap();let name=r.text(16384)?;let public=r.boolean()?;let count=r.len(MAX_PARAMS)?;let mut number=||Ok::<_,String>(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()));let params=(0..count).map(|_|number()).collect::<Result<_,_>>()?;let ret=number()?;let symbol=r.text(128)?;functions.push(Function{id,name,public,params,ret,symbol});}
        let size=r.len(4*1024*1024)?;let types=types::Graph::decode(r.bytes(size)?)?;
        let size=r.len(8*1024*1024)?;let namespace=namespace::Image::decode(r.bytes(size)?)?;
        let size=r.len(4*1024*1024)?;let mut rr=Reader(r.bytes(size)?);let count=rr.len(MAX_DEFS)?;let regions=(0..count).map(|_|regions::Signature::decode(&mut rr)).collect::<Result<_,_>>()?;if !rr.0.is_empty(){return Err("native region trailing data".into());}
        let size=r.len(8*1024*1024)?;let runtime=runtime::Requirements::decode(r.bytes(size)?)?;
        let size=r.len(64*1024*1024)?;let object=r.bytes(size)?.to_vec();let size=r.len(4*1024*1024)?;let constants=consts::decode(r.bytes(size)?)?;if !r.0.is_empty(){return Err("native crate trailing semantic data".into());}
        let artifact=Self{name,metadata,stable,generation,source,overflow,debug,functions,types,constants,namespace,regions,runtime,object};artifact.validate()?;Ok(artifact)
    }
    pub(crate) fn functions(&self)->&[Function]{&self.functions}
    pub(crate) fn object(&self)->&[u8]{&self.object}
    pub(crate) fn runtime_requirements(&self)->&runtime::Requirements{&self.runtime}
}

/// This store owns decoded semantic data. All dense IDs are assigned afresh
/// here; neither producer dense indices nor the consumer bin hash are reused.
pub(crate) struct CrateStore {
    crates: Vec<Artifact>, aliases: Vec<(String,usize)>, local: def::CrateIds,
}
impl CrateStore {
    pub(crate) fn new(mut inputs:Vec<(String,Artifact)>,local:def::CrateIds)->Result<Self,String>{
        if local.local!=def::LOCAL_CRATE||inputs.len()>256{return Err("native crate-store bound/local identity".into());}
        inputs.sort_by_key(|(_,a)|a.stable);let mut crates:Vec<Artifact>=Vec::new();let mut aliases=Vec::new();let mut names=BTreeSet::new();
        for (alias,artifact) in inputs {
            if !ident(&alias)||!names.insert(alias.clone())||artifact.stable==local.stable{return Err("native crate alias/identity collision".into());}
            artifact.validate()?;
            let index=match crates.iter().position(|a|a.stable==artifact.stable){Some(n) if crates[n]==artifact=>n,Some(_)=>return Err("conflicting native builds share one crate identity".into()),None=>{crates.push(artifact);crates.len()-1}};
            aliases.push((alias,index));
        }
        Ok(Self{crates,aliases,local})
    }
    fn def(&self,n:usize,f:&Function)->def::DefId{def::DefId{krate:def::CrateId(n as u32+1),hash:f.id.hash}}
    pub(crate) fn disk_id(&self,id:def::DefId)->Option<def::DiskDefId>{let a=self.crates.get(id.krate.0.checked_sub(1)? as usize)?;a.functions.iter().any(|f|f.id.hash==id.hash).then_some(def::DiskDefId{krate:a.stable,hash:id.hash})}
    pub(crate) fn function(&self,id:def::DefId)->Option<&Function>{self.crates.get(id.krate.0.checked_sub(1)? as usize)?.functions.iter().find(|f|f.id.hash==id.hash)}
    fn mount(&self,t:&mut resolve::imports::ResolvedTables,i:&intern::Interner)->Result<(),String>{
        use resolve::imports::ExternInfo;
        let mut roots=Vec::new();
        for (n,a) in self.crates.iter().enumerate(){
            roots.push(a.namespace.mount(&a.name,a.stable,n,t,i,&a.types.nominals())?);
        }
        for (alias,n) in &self.aliases {
            if t.externs.iter().any(|e|e.owner==ty::defs::ROOT_MODULE&&e.name==*alias){return Err("native crate alias conflicts with an existing extern".into());}
            t.externs.push(ExternInfo{name:alias.clone(),def:t.module_defs[roots[*n].0 as usize],is_std_lib:false,owner:ty::defs::ROOT_MODULE,root_module:Some(roots[*n]),unsupported:None});
        }
        Ok(())
    }
    pub(crate) fn resolve(&self,input:&resolve::worklist::CrateInput,i:&intern::Interner,k:&intern::Known)->Result<resolve::worklist::ResolvedCrate,resolve::worklist::ResolveError>{
        let mut mount=|t:&mut resolve::imports::ResolvedTables|self.mount(t,i).map_err(|e|diag::Diag::new("L0508",diag::Span::default(),e,"Native semantic imports were not admitted."));
        if input.crate_ids.local!=self.local.local||input.crate_ids.stable!=self.local.stable {
            let mut map=diag::SourceMap::default();map.add_file(&input.root_path.to_string_lossy(),&input.root_text);
            return Err(resolve::worklist::ResolveError{map,all_text:input.root_text.clone(),std_asks:Default::default(),platform_asks:None,diags:vec![diag::Diag::new("L0508",diag::Span::default(),"native crate consumer identity changed","Create a fresh store for the actual consumer.")]});
        }
        resolve::worklist::resolve_crate_with_imports(input,i,k,false,&mut mount)
    }
    /// Materialize actual signature types in the consumer's existing native
    /// type store. No FnInfo/local function or AST node is invented. Native
    /// body-call checking/lowering must explicitly learn these foreign defs.
    pub(crate) fn install_signatures(&self,p:&mut check::collect::Program)->Result<(),String>{
        self.install_signatures_with_runtime(p,false)
    }
    fn install_signatures_with_runtime(&self,p:&mut check::collect::Program,runtime_checked:bool)->Result<(),String>{
        if !runtime_checked&&self.crates.iter().any(|a|!a.runtime.imports.is_empty()){return Err("native runtime dependencies require actual providers before imported definitions".into());}
        if p.krate!=self.local.local||p.stable_crate!=self.local.stable{return Err("native signature consumer identity mismatch".into());}
        // Stage dense shapes only. No source-visible DefData, type name or
        // signature is published until ALL images, nodes and regions pass.
        let all_types=self.crates.iter().enumerate().map(|(n,a)|a.types.stage(p,def::CrateId(n as u32+1))).collect::<Result<Vec<_>,_>>()?;
        // Establish the complete template boundary across every image before
        // the original allocator creates any concrete instance.
        let instance_types=all_types.iter().flatten().copied().collect::<Vec<_>>();
        check::generics::materialize_native_adt_types(p,&instance_types);
        let names=names(p);
        for (a,types) in self.crates.iter().zip(&all_types){crate::cgen::names::with(Some(&names),||a.types.check_native(p,types))?;}
        let all_regions=self.crates.iter().enumerate().map(|(n,a)|a.functions.iter().zip(&a.regions).map(|(f,r)|r.install(&mut p.tys,&a.types,&all_types[n],&f.params,f.ret)).collect::<Result<Vec<_>,_>>()).collect::<Result<Vec<_>,_>>()?;
        let nominal_defs=self.crates.iter().enumerate().flat_map(|(n,a)|a.types.nominals().into_iter().map(move|r|def::DefId{krate:def::CrateId(n as u32+1),hash:r.id.hash})).collect::<Vec<_>>();
        let mut errors=Vec::new();check::outlives::check_native_adts(p,&nominal_defs,&mut errors);
        if !errors.is_empty(){return Err(format!("native nominal outlives obligations: {}",errors.iter().map(|e|e.message.as_str()).collect::<Vec<_>>().join("; ")));}
        for (n,a) in self.crates.iter().enumerate(){a.types.publish(p,def::CrateId(n as u32+1),&all_types[n])?;}
        for (n,a) in self.crates.iter().enumerate(){consts::publish(&a.constants,p,def::CrateId(n as u32+1),&all_types[n])?;}
        for (n,a) in self.crates.iter().enumerate(){let types=&all_types[n];for f in &a.functions {let id=self.def(n,f);let tables=p.resolved.as_ref().ok_or("native signatures require the actual resolved imports")?;let info=tables.def_info.get(&id).ok_or("native definition is absent from current resolver")?;
            let mut data=ty::defs::DefData::skeleton(ty::defs::DefKind::ForeignFn,None,p.interner.intern(&f.name),a.namespace.function_visibility(f.id),diag::Span::default());
            data.module=info.module;data.sig=ty::SigData::Fn{params:f.params.iter().map(|&id|types[id as usize]).collect(),ret:types[f.ret as usize]};
            data.native_linkage=Some(Box::new(ty::defs::NativeLinkage{id:f.id,symbol:f.symbol.clone()}));p.defs.insert(id,data);
        }}
        for(n,a)in self.crates.iter().enumerate(){for(f,r)in a.functions.iter().zip(&all_regions[n]){p.regions.native_fns.insert(self.def(n,f),r.clone());}}Ok(())
    }

    /// Closed component execution gate. This uses the real resolver, checker,
    /// borrow checker and C units. No imported source is parsed or replayed.
    /// Fixed structural ABI and abort/overflow-off/assert-off profile only;
    /// this is neither Cargo admission nor general core/std or Rust ABI support.
    pub(crate) fn executable(&self,input:&resolve::worklist::CrateInput,generation:&Generation,backend:&Backend,out:&Path)->Result<(),String>{
        use crate::{borrowck,cgen,lower};use std::{fs,sync::Arc};
        fixed_abi()?;
        backend.verify(generation)?;
        let profile=crate::protocol::ProfileFlags{panic_abort:true,overflow_checks:false,debug_assertions:false};
        if input.env!=resolve::cfg::CfgEnv::new("x86_64-unknown-linux-gnu",&[],false,profile)
            ||!input.deps.is_empty()||!input.cargo_env.is_empty()||input.std_splice_lo.is_some()
            || self.crates.iter().any(|a|a.generation!=*generation||a.overflow||a.debug)
        {return Err("native consumer requires the exact closed primitive generation/profile".into());}
        // Runtime-bearing images cannot install semantic payloads until this
        // operation's actual original providers close every dependency edge.
        // The provider object is used only by this invocation, never replayed
        // from a previous consumer request.
        let needs_runtime=self.crates.iter().any(|a|!a.runtime.imports.is_empty());
        let runtime_object=if needs_runtime {
            fs::create_dir(out).map_err(|e|e.to_string())?;
            let provider=runtime::Provider::compile(out,generation,backend)?;
            let locations:Vec<_>=self.crates.iter().flat_map(|a|a.runtime.locations.iter().cloned()).collect();
            let object=runtime::compile(&runtime::location_unit(&locations)?,"import-locations",out,generation,backend)?;
            let symbols=unit_object::symbols(&fs::read(object).map_err(|e|e.to_string())?)?;
            for a in &self.crates{a.runtime.validate(&a.functions,&a.object)?;provider.check(&a.runtime,&symbols,&a.object)?;}
            Some(provider.object)
        }else{None};
        let i=Arc::new(intern::Interner::default());let k=Arc::new(intern::Known::new(&i));
        let mut resolved=self.resolve(input,&i,&k).map_err(|mut e|diag::render(&mut e.diags,&e.map,&e.all_text,false,20))?;
        if let Some(ast)=crate::native_assertion_text::proposal(&resolved.ast,&resolved.map,&input.root_text,&i,&k){resolved.ast=ast;}
        let report=|mut errors:Vec<diag::Diag>|diag::render(&mut errors,&resolved.map,&resolved.all_text,false,20);
        let mut order=check::collect::DefOrder::default();
        let mut definitions=|p:&mut check::collect::Program|self.install_signatures_with_runtime(p,runtime_object.is_some()).map_err(|e|vec![diag::Diag::new("L0508",diag::Span::default(),e,"Independent native signatures were not installed.")]);
        let mut p=check::collect::collect_with_definitions(Arc::new(resolved.ast),i,k,input.crate_ids,false,&mut order,Some(Arc::new(resolved.tables)),&mut definitions).map_err(report)?;
        p.unwind=false;p.overflow_checks=false;p.debug_assertions=false;p.unwind_tables=false;p.record_reads=false;
        if self.crates.iter().any(|a|a.types.has_templates()){
            let errors=check::generics::materialize(&mut p);if !errors.is_empty(){return Err(report(errors));}
        }
        let mut errors=Vec::new();check_initializers(&mut p,&mut errors);if !errors.is_empty(){return Err(report(errors));}
        if p.adts.iter().enumerate().any(|(n,_)|p.defs.def_of_adt(ty::AdtId(n as u32)).is_some_and(|d|d.krate==p.krate))||!p.statics.is_empty(){return Err("native consumer needs type/static semantic images".into());}
        let main=p.main.ok_or("native consumer needs a real main")?;
        let slots:Vec<u32>=(0..p.fns.len()).map(|n|n as u32).collect();
        let names=names(&p);
        let mut functions=Vec::new();
        let mut slot_lines=Vec::new();let mut slot_files=Vec::new();
        for (n,f) in p.fns.iter().enumerate(){
            let id=def::LocalDefId(n as u32);let written=f.def(&p.ast);
            if p.gx.fn_class.get(n)!=Some(&check::generics::FnClass::Plain)
                || !function_declarations(&p,id)||written.body.is_none()
                || !f.params.iter().chain([&f.ret]).all(|&t|signature_type(&p,t))
            {return Err("native consumer needs qualifier/generic/type/body images".into());}
            let (checked,errors)=check::check_fn(&p,id);if !errors.is_empty(){return Err(report(errors));}
            // Preserve native semantic errors before an unrepresented local
            // qualifier refusal (notably nonconst calls inside const fns).
            if written.qual!=ast::FnQual::default(){return Err("native consumer needs qualifier/generic/type/body images".into());}
            if !cgen::names::with(Some(&names),||f.params.iter().chain([&f.ret]).all(|&t|cgen::native_type_abi(&p,t).is_some())) {return Err("native consumer effective C ABI differs from the image".into());}
            let body=lower::lower(&p,id,checked);let errors=borrowck::borrowck(&p,&body);if !errors.is_empty(){return Err(report(errors));}
            if !body.locals.iter().all(|l|consumer_type(&p,&body.tys,l.ty,0)){return Err("native consumer needs additional body type images".into());}
            let at=resolved.map.call_site(written.name_span);let line=resolved.map.line_display_col(at).0;
            slot_lines.push(line);slot_files.push(at.file.0);
            let generated=cgen::names::with(Some(&names),||cgen::gen_fn(&p,&body,&resolved.map,line,&slots));
            if let Some(error)=&generated.error{return Err(error.clone());}functions.push(generated);
        }
        // Own a fresh operation directory. Failure retains these generated
        // inputs/diagnostics for the operation owner; it never retries an app.
        if !needs_runtime{fs::create_dir(out).map_err(|e|e.to_string())?;}
        fs::write(out.join("consumer.source.rs"),&input.root_text).map_err(|e|e.to_string())?;
        let mut glue=cgen::GlueSet::default();glue.add(&p,&slots,functions.iter().flat_map(|f|f.glue.iter()));
        let indexes:Vec<usize>=(0..functions.len()).collect();
        let paths:Vec<&str>=(0..resolved.map.file_count()).map(|n|resolved.map.path(diag::FileId(n as u32))).collect();
        let mut locations:Vec<_>=self.crates.iter().flat_map(|a|a.runtime.locations.iter().cloned()).collect();
        locations.extend(p.fns.iter().enumerate().map(|(n,_)|runtime::Location{id:p.disk_id(def::LocalDefId(n as u32)),path:paths[slot_files[n]as usize].into(),line:slot_lines[n]}));
        let location_unit=runtime::location_unit(&locations)?;
        let units=cgen::names::with(Some(&names),||vec![
            ("consumer",cgen::fn_unit(&p,&functions,&indexes,&glue,"lust_register_u0",Some(main.0))),
            ("glue",cgen::glue_unit(&p,&glue)),("table",cgen::table_unit(slots.len(),&[0])),
            ("runtime",cgen::runtime_unit()),
            ("locations",location_unit),
            ("main",cgen::main_unit(main.0,&[],cgen::unwind_init(&p,|f|f.0))),
        ]);
        let mut objects=Vec::new();
        for (label,text) in units {
            if label=="runtime"{if let Some(object)=&runtime_object{objects.push(object.clone());continue;}}
            let source=out.join(format!("{label}.c"));let object=out.join(format!("{label}.o"));fs::write(&source,text).map_err(|e|e.to_string())?;
            backend.verify(generation)?;
            let command=Command::new(backend.executable()).args(["--target=x86_64-unknown-linux-gnu","-std=gnu11","-fno-strict-aliasing","-fwrapv","-O0","-c"]).arg(&source).arg("-o").arg(&object).output().map_err(|e|e.to_string())?;
            fs::write(out.join(format!("{label}.stdout")),&command.stdout).map_err(|e|e.to_string())?;fs::write(out.join(format!("{label}.stderr")),&command.stderr).map_err(|e|e.to_string())?;
            if !command.status.success(){return Err(format!("native consumer C backend {label}: {}",String::from_utf8_lossy(&command.stderr)));}
            backend.verify(generation)?;objects.push(object);
        }
        // Independent producer objects enter exactly once, irrespective of
        // import aliases; the same checked image owns both types and symbols.
        for (n,a) in self.crates.iter().enumerate(){let object=out.join(format!("producer-{n}.o"));fs::write(&object,&a.object).map_err(|e|e.to_string())?;objects.push(object);}
        backend.verify(generation)?;
        let command=Command::new(backend.executable()).args(&objects).args(["-pthread","-lm","-ldl","-o"]).arg(out.join("app")).output().map_err(|e|e.to_string())?;
        fs::write(out.join("link.stdout"),&command.stdout).map_err(|e|e.to_string())?;fs::write(out.join("link.stderr"),&command.stderr).map_err(|e|e.to_string())?;
        if !command.status.success(){return Err(format!("native consumer link: {}",String::from_utf8_lossy(&command.stderr)));}
        backend.verify(generation)?;Ok(())
    }
}
