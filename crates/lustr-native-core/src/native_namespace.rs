//! Stable projection of the actual producer resolver. This carries bindings,
//! including re-exports, separately from definition identity and code symbols.
//! No producer ModuleId, Sym or dense crate index crosses the image boundary.
use super::{Artifact, Function, Reader, MAX_DEFS, ident, text};
use crate::{check, def, diag, intern, resolve, ty};
use resolve::imports::{Bind, BindState, DefInfo, InsertOut, ResKind, ResolvedTables, Target, VisKind};
use crate::driver::graph::Namespace;
use std::collections::BTreeSet;
type Path = Vec<String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Visibility { Private, Public, Crate, Super, In(Path) }
impl Visibility {
    pub(super) fn capture(v:&VisKind,i:&intern::Interner)->Self { match v {
        VisKind::Priv=>Self::Private,VisKind::Pub=>Self::Public,VisKind::Crate=>Self::Crate,VisKind::Super=>Self::Super,
        VisKind::In(s)=>Self::In(i.get(*s).split("::").map(str::to_owned).collect()),
    }}
    pub(super) fn mount(&self,i:&intern::Interner)->VisKind {match self {
        Self::Private=>VisKind::Priv,Self::Public=>VisKind::Pub,Self::Crate=>VisKind::Crate,Self::Super=>VisKind::Super,
        Self::In(p)=>VisKind::In(i.intern(&p.join("::"))),
    }}
    pub(super) fn def_vis(&self)->ty::defs::Vis {match self {Self::Public=>ty::defs::Vis::Pub,Self::Private=>ty::defs::Vis::Priv,_=>ty::defs::Vis::Restricted}}
    pub(super) fn encode(&self,out:&mut Vec<u8>){out.push(match self{Self::Private=>0,Self::Public=>1,Self::Crate=>2,Self::Super=>3,Self::In(_)=>4});if let Self::In(p)=self{path(out,p);}}
    pub(super) fn decode(r:&mut Reader<'_>)->Result<Self,String>{Ok(match r.byte()?{0=>Self::Private,1=>Self::Public,2=>Self::Crate,3=>Self::Super,4=>Self::In(read_path(r)?),_=>return Err("native namespace visibility tag".into())})}
    pub(super) fn validate(&self)->Result<(),String>{if let Self::In(p)=self {if p.is_empty()||p.iter().any(|s|!ident(s)){return Err("native namespace visibility path".into());}}Ok(())}
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Module { path:Path, id:Option<def::DiskDefId>, vis:Visibility }
#[derive(Clone, Debug, PartialEq, Eq)]
struct Definition { id:def::DiskDefId, name:String, module:Path, vis:Visibility, kind:u8 }
#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination { Definition(def::DiskDefId), Module(Path), Variant(def::DiskDefId,String) }
#[derive(Clone, Debug, PartialEq, Eq)]
struct Binding { module:Path, namespace:u8, name:String, target:Destination, vis:Visibility, defining_module:Path }
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Image { modules:Vec<Module>, definitions:Vec<Definition>, bindings:Vec<Binding> }
fn path(out:&mut Vec<u8>,p:&Path){out.extend_from_slice(&(p.len() as u32).to_le_bytes());for s in p{text(out,s);}}
fn read_path(r:&mut Reader<'_>)->Result<Path,String>{(0..r.len(128)?).map(|_|r.text(128)).collect()}
fn id(r:&mut Reader<'_>)->Result<def::DiskDefId,String>{Ok(def::DiskDefId::from_bytes(r.bytes(16)?).unwrap())}
fn flat(p:&Path,name:&str)->String {
    if p.is_empty()&&(resolve::worklist::PRELUDE_TYPES.contains(&name)||ty::lang::is_primitive_type_name(name)){format!("crate${name}")}
    else {p.iter().map(String::as_str).chain([name]).collect::<Vec<_>>().join("$")}
}
impl Image {
    pub(super) fn capture(p:&check::collect::Program)->Result<Self,String>{
        let t=p.resolved.as_ref().ok_or("native producer requires its actual resolver tables")?;
        if t.modules.len()>MAX_DEFS||t.def_info.len()>MAX_DEFS{return Err("native namespace bound".into());}
        let stable=|d:def::DefId|->Result<def::DiskDefId,String>{if d.krate!=p.krate{return Err("native producer dependency semantic records required".into());}Ok(def::DiskDefId{krate:p.stable_crate,hash:d.hash})};
        let module=|m:ty::defs::ModuleId|->Result<Path,String>{let info=t.modules.get(m.0 as usize).ok_or("native producer module identity")?;if info.krate_root!=ty::defs::ROOT_MODULE{return Err("native producer dependency module records required".into());}Ok(info.path.iter().map(|&s|p.name(s).to_owned()).collect())};
        let mut modules=Vec::new();
        for (n,m) in t.modules.iter().enumerate(){let path=module(ty::defs::ModuleId(n as u32))?;let id=if path.is_empty(){None}else{Some(stable(t.module_defs[n])?)};modules.push(Module{path,id,vis:Visibility::capture(&m.vis,&p.interner)});}
        modules.sort_by(|a,b|a.path.len().cmp(&b.path.len()).then(a.path.cmp(&b.path)));
        let mut definitions=Vec::new();
        for (&d,info) in &t.def_info {
            let kind=match info.kind {ResKind::Fn=>0,ResKind::Const=>1,ResKind::Adt=>2,_=>return Err("native namespace needs additional typed definition records".into())};
            definitions.push(Definition{id:stable(d)?,name:intern::source_name(p.name(info.flat)).into(),module:module(info.module)?,vis:Visibility::capture(&info.vis,&p.interner),kind});
        }
        definitions.sort_by_key(|d|d.id);
        let mut bindings=Vec::new();
        for (n,maps) in t.maps.iter().enumerate(){for (namespace,map) in maps.iter().enumerate(){for (&name,state) in map {
            let BindState::One(b)=state else{return Err("native namespace needs ambiguity records".into())};
            let target=match b.target {Target::Def(d)=>Destination::Definition(stable(d)?),Target::Module(m)=>Destination::Module(module(m)?),Target::Variant(d,v)=>Destination::Variant(stable(d)?,p.name(v).into()),_=>return Err("native namespace needs external records".into())};
            bindings.push(Binding{module:module(ty::defs::ModuleId(n as u32))?,namespace:namespace as u8,name:p.name(name).into(),target,vis:Visibility::capture(&b.vis,&p.interner),defining_module:module(b.def_module)?});
        }}}
        bindings.sort_by(|a,b|(&a.module,a.namespace,&a.name).cmp(&(&b.module,b.namespace,&b.name)));
        if !t.std_aliases.is_empty(){return Err("native namespace needs genuine core/alloc/std records".into());}
        Ok(Self{modules,definitions,bindings})
    }
    pub(super) fn validate(&self,stable:def::StableCrateHash,functions:&[Function],nominals:&[&super::nominals::Record],constants:&[super::consts::Record])->Result<(),String>{
        if self.modules.is_empty()||self.modules.len()>MAX_DEFS||self.definitions.len()>MAX_DEFS||self.bindings.len()>MAX_DEFS*4{return Err("native namespace bound".into());}
        let mut paths=BTreeSet::new();let mut ids=BTreeSet::new();
        for m in &self.modules {
            m.vis.validate()?;
            if m.path.len()>128||m.path.iter().any(|s|!ident(s))||(!m.path.is_empty()&&!paths.contains(&m.path[..m.path.len()-1].to_vec()))||!paths.insert(m.path.clone()) {return Err("native namespace module tree".into());}
            let expected=m.path.split_last().map(|(name,parent)|def::DiskDefId{krate:stable,hash:def::item_hash(stable,&[],def::DefKind::Module,&flat(&parent.to_vec(),name),0)});
            if m.id!=expected||m.id.is_some_and(|d|!ids.insert(d)){return Err("native namespace module identity".into());}
            if m.path.is_empty()&&m.vis!=Visibility::Public{return Err("native namespace root visibility".into());}
        }
        for d in &self.definitions {
            d.vis.validate()?;
            let name=flat(&d.module,&d.name);let hash=match d.kind{0=>def::free_fn_hash(stable,&[],&name),1=>def::item_hash(stable,&[],def::DefKind::Const,&name,0),2=>def::item_hash(stable,&[],def::DefKind::Adt,&name,0),_=>return Err("native namespace definition kind".into())};
            if !ident(&d.name)||!paths.contains(&d.module)||d.id!= (def::DiskDefId{krate:stable,hash})||!ids.insert(d.id){return Err("native namespace definition identity".into());}
            if d.kind==0&&!functions.iter().any(|f|f.id==d.id&&f.name==name&&f.public==(d.vis==Visibility::Public)){return Err("native namespace definition/signature mismatch".into());}
            if d.kind==2&&!nominals.iter().any(|r|r.id==d.id&&r.name==name){return Err("native namespace nominal definition/shape mismatch".into());}
            if d.kind==1&&d.vis==Visibility::Public&&!constants.iter().any(|r|r.id==d.id&&r.name==name){return Err("native namespace constant definition/value mismatch".into());}
        }
        if functions.iter().any(|f|!self.definitions.iter().any(|d|d.id==f.id&&d.kind==0)){return Err("native namespace missing function definition".into());}
        if nominals.iter().any(|r|!self.definitions.iter().any(|d|d.id==r.id&&d.kind==2)){return Err("native namespace missing nominal definition".into());}
        if constants.iter().any(|r|!self.definitions.iter().any(|d|d.id==r.id&&d.kind==1)){return Err("native namespace missing constant definition".into());}
        let mut keys=BTreeSet::new();
        for b in &self.bindings {
            b.vis.validate()?;
            if !ident(&b.name)||!paths.contains(&b.module)||!paths.contains(&b.defining_module)||!keys.insert((&b.module,b.namespace,&b.name)){return Err("native namespace binding identity".into());}
            match &b.target {
                Destination::Module(p) if b.namespace==0&&paths.contains(p)&&!p.is_empty()=>{},
                Destination::Definition(id) if b.namespace<=1=>{let d=self.definitions.iter().find(|d|d.id==*id).ok_or("native namespace binding definition missing")?;let ctor=d.kind==2&&nominals.iter().any(|r|r.id==*id&&r.has_constructor());if b.namespace==0&&d.kind!=2||b.namespace==1&&d.kind==2&&!ctor{return Err("native namespace nominal target/namespace mismatch".into());}},
                Destination::Variant(id,name) if b.namespace<=1=>{if !ident(name)||!nominals.iter().any(|r|r.id==*id&&r.is_enum&&r.variants.iter().any(|v|v.name==*name)){return Err("native namespace variant target identity".into());}},
                _=>return Err("native namespace target/namespace mismatch".into()),
            }
        }
        // Reuse the real resolver's restrictions and visibility regions. A
        // valid checksum cannot authorize a malformed scope or wider export.
        let i=intern::Interner::default();let mut tables=ResolvedTables::new(i.intern("consumer"));
        let root=self.mount("producer",stable,0,&mut tables,&i,nominals)?;
        let module=|p:&Path|tables.module_by_path_in(root,&p.iter().map(|s|i.intern(s)).collect::<Vec<_>>()).unwrap();
        for m in &self.modules {let at=module(&m.path);let scope=tables.modules[at.0 as usize].parent.unwrap_or(root);if !tables.restriction_ok(&m.vis.mount(&i),scope,&i){return Err("native namespace visibility restriction".into());}}
        for d in &self.definitions {if !tables.restriction_ok(&d.vis.mount(&i),module(&d.module),&i){return Err("native namespace visibility restriction".into());}}
        for r in nominals {let d=self.definitions.iter().find(|d|d.id==r.id).unwrap();for f in r.fields(){if !tables.restriction_ok(&f.vis.mount(&i),module(&d.module),&i)||r.is_enum&&f.vis!=d.vis{return Err("native nominal field visibility restriction".into());}}}
        for b in &self.bindings {
            let (target_vis,target_module)=match &b.target {
                Destination::Definition(id)|Destination::Variant(id,_)=>{let d=self.definitions.iter().find(|d|d.id==*id).unwrap();(d.vis.mount(&i),module(&d.module))},
                Destination::Module(p)=>{let m=self.modules.iter().find(|m|m.path==*p).unwrap();(m.vis.mount(&i),module(&p[..p.len()-1].to_vec()))},
            };
            let vis=b.vis.mount(&i);let scope=module(&b.defining_module);
            if !tables.restriction_ok(&vis,scope,&i)||!tables.visible(&target_vis,target_module,module(&b.module),&i)||!tables.vis_within(&vis,scope,&target_vis,target_module,&i){return Err("native namespace visibility/re-export mismatch".into());}
        }
        Ok(())
    }
    pub(super) fn encode(&self)->Vec<u8>{
        let mut out=Vec::new();out.extend_from_slice(&(self.modules.len() as u32).to_le_bytes());
        for m in &self.modules{path(&mut out,&m.path);out.push(m.id.is_some() as u8);if let Some(id)=m.id{out.extend_from_slice(&id.to_bytes());}m.vis.encode(&mut out);}
        out.extend_from_slice(&(self.definitions.len() as u32).to_le_bytes());
        for d in &self.definitions{out.extend_from_slice(&d.id.to_bytes());text(&mut out,&d.name);path(&mut out,&d.module);d.vis.encode(&mut out);out.push(d.kind);}
        out.extend_from_slice(&(self.bindings.len() as u32).to_le_bytes());
        for b in &self.bindings{path(&mut out,&b.module);out.push(b.namespace);text(&mut out,&b.name);match &b.target{Destination::Definition(id)=>{out.push(0);out.extend_from_slice(&id.to_bytes());},Destination::Module(p)=>{out.push(1);path(&mut out,p);},Destination::Variant(id,name)=>{out.push(2);out.extend_from_slice(&id.to_bytes());text(&mut out,name);}}b.vis.encode(&mut out);path(&mut out,&b.defining_module);}
        out
    }
    pub(super) fn decode(bytes:&[u8])->Result<Self,String>{
        let mut r=Reader(bytes);let mut modules=Vec::new();for _ in 0..r.len(MAX_DEFS)?{modules.push(Module{path:read_path(&mut r)?,id:if r.boolean()?{Some(id(&mut r)?)}else{None},vis:Visibility::decode(&mut r)?});}
        let mut definitions=Vec::new();for _ in 0..r.len(MAX_DEFS)?{definitions.push(Definition{id:id(&mut r)?,name:r.text(128)?,module:read_path(&mut r)?,vis:Visibility::decode(&mut r)?,kind:r.byte()?});}
        let mut bindings=Vec::new();for _ in 0..r.len(MAX_DEFS*4)?{bindings.push(Binding{module:read_path(&mut r)?,namespace:r.byte()?,name:r.text(128)?,target:match r.byte()?{0=>Destination::Definition(id(&mut r)?),1=>Destination::Module(read_path(&mut r)?),2=>Destination::Variant(id(&mut r)?,r.text(128)?),_=>return Err("native namespace target tag".into())},vis:Visibility::decode(&mut r)?,defining_module:read_path(&mut r)?});}
        if !r.0.is_empty(){return Err("native namespace trailing data".into());}Ok(Self{modules,definitions,bindings})
    }
    pub(super) fn mount(&self,name:&str,stable:def::StableCrateHash,n:usize,t:&mut ResolvedTables,i:&intern::Interner,nominals:&[&super::nominals::Record])->Result<ty::defs::ModuleId,String>{
        let root=t.add_crate_root(i.intern(name));
        t.crate_magic.insert(i.intern(&format!("$lust$crate${:016x}",stable.0)),root);
        let dense=|d:def::DiskDefId|def::DefId{krate:def::CrateId(n as u32+1),hash:d.hash};
        t.module_defs[root.0 as usize]=dense(def::DiskDefId{krate:stable,hash:def::item_hash(stable,&[],def::DefKind::Module,name,0)});
        let mut modules=std::collections::BTreeMap::new();modules.insert(Vec::<String>::new(),root);
        for m in self.modules.iter().filter(|m|!m.path.is_empty()) {let (name,parent)=m.path.split_last().unwrap();let at=t.add_module(modules[parent],i.intern(name),m.vis.mount(i),diag::Span::default());t.module_defs[at.0 as usize]=dense(m.id.unwrap());modules.insert(m.path.clone(),at);}
        for d in &self.definitions {let id=dense(d.id);let flat=i.intern(&format!("$lustr$crate${:016x}${:016x}",stable.0,d.id.hash.0));let module=modules[&d.module];
            t.def_info.insert(id,DefInfo{kind:match d.kind{0=>ResKind::Fn,1=>ResKind::Const,2=>ResKind::Adt,_=>return Err("native namespace definition kind".into())},flat,module,vis:d.vis.mount(i)});
            if t.flat[if d.kind==2{0}else{1}].insert(flat,id).is_some(){return Err("native crate flat identity collision".into());}
            t.display_names.insert(flat,i.intern(&std::iter::once(name).chain(d.module.iter().map(String::as_str)).chain([d.name.as_str()]).collect::<Vec<_>>().join("::")));
        }
        for r in nominals {let def=dense(r.id);let info=t.def_info.get(&def).ok_or("native nominal mounted definition missing")?;
            if r.is_enum{if t.variants.insert(def,r.variants.iter().map(|v|i.intern(&v.name)).collect()).is_some(){return Err("native enum duplicate variant table".into());}}
            if r.has_constructor()&&t.flat[1].insert(info.flat,def).is_some(){return Err("native nominal constructor identity collision".into());}
            let fields=r.variants.iter().filter(|v|v.shape==check::collect::VariantShape::Named).flat_map(|v|&v.fields).map(|f|(i.intern(&f.name),f.vis.mount(i))).collect();
            if (r.is_enum||r.legacy_named())&&t.field_vis.insert(def,(info.module,fields)).is_some(){return Err("native nominal duplicate field visibility".into());}
            if !r.is_enum&&r.variants[0].shape==check::collect::VariantShape::Tuple&&t.tuple_field_vis.insert(def,(info.module,r.variants[0].fields.iter().map(|f|f.vis.mount(i)).collect())).is_some(){return Err("native nominal duplicate tuple visibility".into());}
        }
        for b in &self.bindings {let target=match &b.target{Destination::Definition(id)=>Target::Def(dense(*id)),Destination::Module(p)=>Target::Module(modules[p]),Destination::Variant(id,name)=>Target::Variant(dense(*id),i.intern(name))};
            let ns=match b.namespace{0=>Namespace::Type,1=>Namespace::Value,_=>return Err("native namespace unsupported namespace".into())};
            if !matches!(t.insert(modules[&b.module],ns,i.intern(&b.name),Bind{target,vis:b.vis.mount(i),rank:2,def_module:modules[&b.defining_module]}),InsertOut::Changed){return Err("native namespace duplicate mounted binding".into());}
        }
        Ok(root)
    }
    pub(super) fn function_visibility(&self,id:def::DiskDefId)->ty::defs::Vis{self.definitions.iter().find(|d|d.id==id).unwrap().vis.def_vis()}
}
