//! Stable structural type records for native images. Wire indices refer only
//! to earlier nodes in this image. Reference cycles name a nominal definition
//! by its stable identity, never by producer Ty/SigId/ConstId values.
//! Region signatures are separate records; nominal definitions and generic
//! obligations still require their native records.
use super::{Reader, Scalar, MAX_PARAMS};
use crate::{cgen, check, def, ty};
use super::{nominals,generic_applications};
use ty::TyIntern;
use ty::layout::AdtShapes;
const MAX_NODES:usize=4096;
const MAX_EDGES:usize=32768;
const MAX_DEPTH:usize=128;
const MAX_SIZE:u64=64*1024*1024;
const MAX_ARRAY:u64=1024*1024;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Kind {
    Scalar(Scalar), Float(ty::FloatTy), Tuple(Vec<u32>), Array(u32,u64),
    Ref(ty::Region,ty::Mut,u32), Nominal(Box<nominals::Record>),
    // These nodes describe the sized reference, including its actual wide
    // layout/ABI. No layout is invented for the unsized referent itself.
    SliceRef(ty::Region,ty::Mut,u32), StrRef(ty::Region,ty::Mut),
    NominalApply{base:u32,regions:Vec<ty::Region>},
    Instance(Box<generic_applications::Application>),
    // Only a reference may close a nominal cycle. The complete definition
    // must occur in this image and its lifetime application is kept separately.
    NominalRef{region:ty::Region,mutable:ty::Mut,target:def::DiskDefId,regions:Vec<ty::Region>},
    // Original safe Rust pointer signature, including its output lifetime
    // ownership mask. Free item regions and unrepresented binders refuse;
    // the original parser may canonicalize a binder equivalent to elision.
    FnPtr{params:Vec<u32>,ret:u32,own_out:u32,out_slots:u8},
}
#[derive(Clone,Debug,PartialEq,Eq)]
struct Node {kind:Kind,layout:ty::layout::Layout,copy:bool,drop:bool,abi:cgen::NativeTypeAbi}
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub(super) struct Graph {nodes:Vec<Node>,templates:Vec<super::generic_nominals::Template>}
struct Shapes {base:u32,values:Vec<Option<ty::layout::AdtShape>>}
impl ty::layout::AdtShapes for Shapes {fn adt_shape(&self,id:ty::AdtId)->Option<ty::layout::AdtShape>{let s=self.values.get(id.0.checked_sub(self.base)? as usize)?.as_ref()?;Some(ty::layout::AdtShape{repr:s.repr,is_enum:s.is_enum,variants:s.variants.clone(),discs:s.discs.clone(),hide_niche:s.hide_niche})}}

pub(super) fn supported(p:&check::collect::Program,tys:&impl TyIntern,t:ty::Ty,depth:usize)->bool{
    supported_at(Some(p),tys,t,depth,&mut MAX_EDGES)
}
fn supported_at(p:Option<&check::collect::Program>,tys:&impl TyIntern,t:ty::Ty,depth:usize,budget:&mut usize)->bool{
    if depth>MAX_DEPTH||*budget==0{return false;}*budget-=1;
    match tys.kind(t){
        ty::TyKind::Unit|ty::TyKind::Bool|ty::TyKind::Char|ty::TyKind::Never|ty::TyKind::Int(_)|ty::TyKind::Float(_)=>true,
        ty::TyKind::Tuple(ts)=>tys.tys_of(ts).len()<=MAX_PARAMS&&tys.tys_of(ts).iter().all(|&t|supported_at(p,tys,t,depth+1,budget)),
        ty::TyKind::Array(t,c)=>matches!(tys.const_kind(c),ty::ConstKind::Int(n) if n<=MAX_ARRAY as u128)&&supported_at(p,tys,t,depth+1,budget),
        ty::TyKind::Ref(r,_,t)=>matches!(r,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64))&&match tys.kind(t){
            ty::TyKind::Slice(e)=>supported_at(p,tys,e,depth+1,budget),
            ty::TyKind::Str=>true,
            // Referent fields are checked by their complete nominal record;
            // traversing them here would unfold recursive references forever.
            ty::TyKind::Adt(..)=>p.map_or_else(||tys.local_adt(t).is_some(),|p|nominals::eligible(p,tys,t)||generic_applications::eligible(p,tys,t)),
            _=>supported_at(p,tys,t,depth+1,budget),
        },
        ty::TyKind::Adt(..)=>p.map_or_else(||tys.local_adt(t).is_some(),|p|(nominals::eligible(p,tys,t)||generic_applications::eligible(p,tys,t))&&p.adt(tys.local_adt(t).unwrap()).variants.iter().flat_map(|v|&v.fields).all(|f|supported_at(Some(p),tys,f.ty,depth+1,budget))),
        ty::TyKind::FnPtr(id)=>{let s=tys.sig(id);let v=s.value;s.count==0&&!v.is_unsafe&&!v.extern_c&&v.free_slots==0&&v.free_region==ty::Region::Erased
            &&tys.tys_of(v.inputs).len()<=MAX_PARAMS
            &&tys.tys_of(v.inputs).iter().chain([&v.output]).all(|&t|pointer_part(p,tys,t,depth+1,budget))&&pointer_slots(tys,v)},
        _=>false,
    }
}
fn pointer_part(p:Option<&check::collect::Program>,tys:&impl TyIntern,t:ty::Ty,depth:usize,budget:&mut usize)->bool{
    if depth>MAX_DEPTH||*budget==0{return false;}*budget-=1;
    match tys.kind(t){
        ty::TyKind::Ref(r,_,t)=>matches!(r,ty::Region::Erased|ty::Region::Static)&&match tys.kind(t){
            ty::TyKind::Slice(e)=>pointer_part(p,tys,e,depth+1,budget),
            ty::TyKind::Str=>true,
            _=>pointer_part(p,tys,t,depth+1,budget),
        },
        ty::TyKind::Tuple(ts)=>tys.tys_of(ts).iter().all(|&t|pointer_part(p,tys,t,depth+1,budget)),
        ty::TyKind::Array(t,_)=>pointer_part(p,tys,t,depth+1,budget),
        ty::TyKind::Adt(d,args) if !tys.args(args).is_empty()||p.is_some_and(|p|p.gx.adt_lifetimes.get(&d).is_some_and(|v|!v.is_empty()))=>false,
        _=>supported_at(p,tys,t,depth,budget),
    }
}

// The native proof shifts a u32 by each input/output slot. Bound this before
// entering it, and reject masks that could treat a concrete borrowed output
// as a substituted type argument. Nested pointers bind their own slots.
fn pointer_slots(tys:&impl TyIntern,s:ty::FnSig)->bool{
    let mut inputs=Vec::new();for &t in tys.tys_of(s.inputs){crate::borrowck::regions_of(tys,t,&mut inputs);}
    let mut output=Vec::new();crate::borrowck::regions_of(tys,s.output,&mut output);
    if inputs.len()+output.len()>32{return false;}
    let mask=output.iter().enumerate().filter(|(_,r)|**r==ty::Region::Erased).fold(0u32,|m,(n,_)|m|1u32<<n);
    if mask!=0&&(inputs.len()!=1||inputs[0]!=ty::Region::Erased){return false;}
    if s.out_slots==u8::MAX{s.own_out==0}else{s.out_slots as usize==output.len()&&s.own_out==mask}
}

impl Graph{
    pub(super) fn has_templates(&self)->bool{!self.templates.is_empty()}
    pub(super) fn capture_definitions(&mut self,p:&check::collect::Program)->Result<(),String>{
        for(n,_)in p.adts.iter().enumerate(){let a=ty::AdtId(n as u32);let Some(d)=p.defs.def_of_adt(a)else{continue;};if d.krate!=p.krate||p.gx.adt_instance_of[n].is_some(){continue;}
            if p.gx.adt_generics[n].params.is_empty(){let t=p.tys.get_adt_ty(a).ok_or("native nominal has no actual type")?;self.capture(p,t)?;}
            else{self.capture_template(p,a)?;}
        }Ok(())
    }
    fn capture_template(&mut self,p:&check::collect::Program,a:ty::AdtId)->Result<(),String>{let t=super::generic_nominals::Template::capture(p,a)?;if let Some(old)=self.templates.iter().find(|old|old.record.id==t.record.id){if old!=&t{return Err("native symbolic nominal inconsistent capture".into());}}else{self.templates.push(t);}Ok(())}
    /// A kept context must represent every nominal lifetime application.
    /// A bare definition is the erased checker shape, never a kept use.
    pub(super) fn kept(&self,id:u32)->Result<(),String>{self.kept_at(id,false,0,&mut MAX_EDGES)}
    fn kept_at(&self,id:u32,pointer:bool,depth:usize,budget:&mut usize)->Result<(),String>{
        if depth>MAX_DEPTH||*budget==0{return Err("native kept type depth/work bound".into());}*budget-=1;
        match self.kind(id)?{
            Kind::Nominal(r)if r.regions.as_ref().is_some_and(|r|!r.names.is_empty())=>Err("native kept nominal requires its lifetime application".into()),
            Kind::NominalApply{..}if pointer=>Err("native borrowed nominal pointer needs binder/slot records".into()),
            Kind::Instance(a)=>{if pointer{return Err("native generic nominal pointer needs binder/slot records".into());}for arg in &a.args{if let generic_applications::Argument::Type(t)=arg{self.kept_at(*t,pointer,depth+1,budget)?;}}for &t in a.fields.iter().flatten(){self.kept_at(t,pointer,depth+1,budget)?;}Ok(())},
            Kind::NominalRef{target,regions,..}=>{
                let record=self.nominals().into_iter().find(|r|r.id==*target).ok_or("native nominal reference target is absent")?;
                let n=record.regions.as_ref().map_or(0,|r|r.names.len());
                if n!=regions.len(){return Err("native kept nominal reference requires its lifetime application".into());}
                if pointer&&n!=0{return Err("native borrowed nominal pointer needs binder/slot records".into());}Ok(())
            },
            Kind::Ref(_,_,t)|Kind::SliceRef(_,_,t)|Kind::Array(t,_)=>self.kept_at(*t,pointer,depth+1,budget),
            Kind::Tuple(ts)=>{for &t in ts{self.kept_at(t,pointer,depth+1,budget)?;}Ok(())},
            Kind::FnPtr{params,ret,..}=>{for &t in params.iter().chain([ret]){self.kept_at(t,true,depth+1,budget)?;}Ok(())},
            _=>Ok(()),
        }
    }
    pub(super) fn kind(&self,id:u32)->Result<&Kind,String>{self.nodes.get(id as usize).map(|n|&n.kind).ok_or_else(||"native constant type index".into())}
    pub(super) fn capture(&mut self,p:&check::collect::Program,t:ty::Ty)->Result<u32,String>{self.capture_at(p,t,0,&mut MAX_EDGES,&mut Default::default())}
    fn capture_at(&mut self,p:&check::collect::Program,t:ty::Ty,depth:usize,budget:&mut usize,active:&mut std::collections::BTreeSet<def::DefId>)->Result<u32,String>{
        if depth>MAX_DEPTH||*budget==0{return Err("native type graph depth/work bound".into());}*budget-=1;
        // A completed definition is shared by every application and traversal
        // root, including the other end of a mutually recursive component.
        if let ty::TyKind::Adt(d,args)=p.tys.kind(t){if p.tys.args(args).is_empty(){let id=def::DiskDefId{krate:p.stable_crate,hash:d.hash};if let Some(n)=self.nodes.iter().position(|node|matches!(&node.kind,Kind::Nominal(r)if r.id==id)){return Ok(n as u32);}}}
        let kind=match p.tys.kind(t){
            ty::TyKind::Ref(r,m,t)=>{if !matches!(r,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64)){return Err("native reference requires a represented region".into());}
                if let ty::TyKind::Adt(d,args)=p.tys.kind(t)&&active.contains(&d){
                    if d.krate!=p.krate||!nominals::eligible(p,&p.tys,t){return Err("native recursive reference needs its complete local nominal definition".into());}
                    let regions=p.tys.args(args).iter().map(|g|match g{ty::GenericArg::Lifetime(r)=>Ok(*r),_=>Err("native recursive reference needs type/const generic records")}).collect::<Result<_,_>>()?;
                    Kind::NominalRef{region:r,mutable:m,target:def::DiskDefId{krate:p.stable_crate,hash:d.hash},regions}
                }else{match p.tys.kind(t){
                    ty::TyKind::Slice(e)=>Kind::SliceRef(r,m,self.capture_at(p,e,depth+1,budget,active)?),
                    ty::TyKind::Str=>Kind::StrRef(r,m),
                    _=>Kind::Ref(r,m,self.capture_at(p,t,depth+1,budget,active)?),
                }}},
            ty::TyKind::Float(f)=>Kind::Float(f),
            ty::TyKind::Tuple(ts)=>{let ts=p.tys.tys_of(ts);if ts.len()>MAX_PARAMS{return Err("native type graph field bound".into());}Kind::Tuple(ts.iter().map(|&t|self.capture_at(p,t,depth+1,budget,active)).collect::<Result<_,_>>()?)},
            ty::TyKind::Array(t,c)=>{let ty::ConstKind::Int(n)=p.tys.const_kind(c) else{return Err("native array requires its resolved concrete const value".into())};if n>MAX_ARRAY as u128{return Err("native array length bound".into());}Kind::Array(self.capture_at(p,t,depth+1,budget,active)?,n as u64)},
            ty::TyKind::FnPtr(id)=>{let s=p.tys.sig(id);let v=s.value;if !supported(p,&p.tys,t,0){return Err("native function pointer needs qualifier/region records".into());}
                Kind::FnPtr{params:p.tys.tys_of(v.inputs).iter().map(|&t|self.capture_at(p,t,depth+1,budget,active)).collect::<Result<_,_>>()?,ret:self.capture_at(p,v.output,depth+1,budget,active)?,own_out:v.own_out,out_slots:v.out_slots}},
            ty::TyKind::Adt(..) if generic_applications::template(p,&p.tys,t).is_some()=>{
                let a=generic_applications::template(p,&p.tys,t).unwrap();self.capture_template(p,a)?;
                Kind::Instance(Box::new(generic_applications::Application::capture(p,t,|field|self.capture_at(p,field,depth+1,budget,active))?))
            },
            ty::TyKind::Adt(d,args) if !p.tys.args(args).is_empty()=>{
                if !nominals::eligible(p,&p.tys,t){return Err("native nominal application requires represented lifetime arguments".into());}
                let base=p.tys.get(ty::TyKind::Adt(d,ty::EMPTY_ARGS)).ok_or("native nominal application lacks its actual definition type")?;
                let base=self.capture_at(p,base,depth+1,budget,active)?;
                let regions=p.tys.args(args).iter().map(|g|match g{ty::GenericArg::Lifetime(r)=>Ok(*r),_=>Err("native nominal application needs type/const generic records")}).collect::<Result<_,_>>()?;
                Kind::NominalApply{base,regions}
            },
            ty::TyKind::Adt(d,_)=>{
                if !active.insert(d){return Err("native by-value nominal cycle".into());}
                let record=nominals::Record::capture(p,t,|field|self.capture_at(p,field,depth+1,budget,active));active.remove(&d);
                Kind::Nominal(Box::new(record?))
            },
            _=>Kind::Scalar(Scalar::from_ty(p,t)?),
        };
        let layout=ty::layout::layout_of_ty(&p.tys,p,t).map_err(|_|"native type has no qualified layout")?;
        if layout.size>MAX_SIZE{return Err("native type layout size bound".into());}
        let abi=cgen::native_type_abi(p,t).ok_or("native type image requires the recorded C ABI")?;
        let node=Node{kind,layout,copy:check::collect::is_copy(p,&p.tys,t),drop:check::collect::needs_drop(p,&p.tys,t),abi};
        if let Some(n)=self.nodes.iter().position(|x|*x==node){return Ok(n as u32);}
        if self.nodes.len()>=MAX_NODES{return Err("native type graph node bound".into());}
        self.nodes.push(node);Ok(self.nodes.len() as u32-1)
    }
    pub(super) fn contains(&self,id:u32)->bool{(id as usize)<self.nodes.len()}
    fn closed_nominals(&self)->Vec<&nominals::Record>{self.nodes.iter().filter_map(|n|if let Kind::Nominal(r)=&n.kind{Some(r.as_ref())}else{None}).collect()}
    pub(super) fn nominals(&self)->Vec<&nominals::Record>{self.closed_nominals().into_iter().chain(self.templates.iter().map(|t|&t.record)).collect()}
    pub(super) fn intern(&self,tys:&mut ty::GlobalTys)->Result<Vec<ty::Ty>,String>{self.intern_into(tys,0,&|d|def::DefId{krate:def::CrateId(1),hash:d.hash},false)}
    fn intern_into(&self,tys:&mut ty::GlobalTys,base:u32,map:&impl Fn(def::DiskDefId)->def::DefId,staging:bool)->Result<Vec<ty::Ty>,String>{
        let mut shapes=Shapes{base,values:Vec::new()};let mut definitions=std::collections::BTreeMap::new();
        if self.nodes.is_empty()||self.nodes.len()>MAX_NODES{return Err("native type graph node bound".into());}
        // Register identity/shape indexes only. Complete shapes and all native
        // checks still precede every imported DefData/name/signature/value.
        for r in self.closed_nominals(){r.validate()?;let id=ty::AdtId(base+definitions.len()as u32);if definitions.insert(r.id,r).is_some(){return Err("native nominal duplicate definition".into());}let def=map(r.id);if tys.local_adt_of(def).is_some(){return Err("native nominal type identity already registered".into());}tys.register_adt(def,id);}
        shapes.values.resize_with(definitions.len()+self.templates.len(),||None);
        for(n,t)in self.templates.iter().enumerate(){t.validate()?;let d=map(t.record.id);if definitions.contains_key(&t.record.id)||tys.local_adt_of(d).is_some(){return Err("native symbolic nominal duplicate definition".into());}let at=definitions.len()+n;tys.register_adt(d,ty::AdtId(base+at as u32));shapes.values[at]=Some(t.substituted_shape(tys,&[]));}
        let mut types=Vec::new();let mut depths:Vec<usize>=Vec::new();let mut costs:Vec<usize>=Vec::new();let mut total=0usize;let mut edges=0usize;let mut facts=0usize;
        for (n,node)in self.nodes.iter().enumerate(){
            let mut depth=0;let mut cost=1usize;
            let mut child=|id:u32|->Result<ty::Ty,String>{let id=id as usize;if id>=n{return Err("native type graph forward/cyclic reference".into());}edges+=1;cost=cost.checked_add(costs[id]).ok_or("native type graph work bound")?;if edges>MAX_EDGES||cost>MAX_EDGES{return Err("native type graph edge/work bound".into());}depth=depth.max(depths[id]+1);Ok(types[id])};
            let kind=match &node.kind{
                Kind::Scalar(s)=>{costs.push(1);total+=1;if total>MAX_EDGES{return Err("native type graph work bound".into());}types.push(s.intern(tys));depths.push(0);if !staging{self.check_layout(tys,&shapes,n,types[n])?;}continue;},
                Kind::Nominal(r)=>{
                    r.validate()?;
                    let fields=r.fields().map(|f|child(f.ty)).collect::<Result<Vec<_>,_>>()?;
                    for &t in &fields{if crate::borrowck::erase(tys,t)!=t{return Err("native nominal check fields are not region erased".into());}}
                    if let Some(item)=&r.regions{
                        facts+=item.facts.len();if facts>4096{return Err("native nominal cumulative outlives fact bound".into());}
                        for &t in item.fields.iter().flatten(){self.kept(t)?;}
                        let kept=item.fields.iter().flatten().map(|&t|child(t)).collect::<Result<Vec<_>,_>>()?;let mut used=std::collections::BTreeSet::new();let mut has_regions=false;
                        for(&kept,&erased)in kept.iter().zip(&fields){if crate::borrowck::erase(tys,kept)!=erased{return Err("native nominal kept/check field shape mismatch".into());}let mut regions=Vec::new();crate::borrowck::regions_of(tys,kept,&mut regions);has_regions|=!regions.is_empty();for region in regions{match region{ty::Region::Static=>{},ty::Region::Param(k)if(k as usize)<item.names.len()=>{used.insert(k);},_=>return Err("native nominal field region declaration mismatch".into())}}}
                        if used.len()!=item.names.len()||!has_regions{return Err("native nominal unused or empty item region declaration".into());}
                    }else{
                        for f in r.fields(){self.kept(f.ty)?;}
                        for &t in &fields{let mut regions=Vec::new();crate::borrowck::regions_of(tys,t,&mut regions);if !regions.is_empty(){return Err("native nominal fields need item region records".into());}}
                    }
                    let def=map(r.id);let at=tys.local_adt_of(def).unwrap().0-base;shapes.values[at as usize]=Some(r.shape(&types));ty::TyKind::Adt(def,ty::EMPTY_ARGS)
                },
                Kind::Instance(a)=>{
                    let template=self.templates.iter().find(|t|t.record.id==a.target).ok_or("native application template is absent from exact image")?;
                    let(t,shape)=a.intern(tys,template,map(a.target),&mut child)?;
                    if !staging{let ty::TyKind::Adt(d,args)=tys.kind(t)else{unreachable!()};if let Some(old)=tys.local_adt_instance(d,args){let old=shapes.adt_shape(old).ok_or("native application shape missing")?;if old.repr!=shape.repr||old.is_enum!=shape.is_enum||old.variants!=shape.variants||old.discs!=shape.discs||old.hide_niche!=shape.hide_niche{return Err("native application inconsistent duplicate shape".into());}}else{let id=ty::AdtId(base+shapes.values.len()as u32);shapes.values.push(Some(shape));tys.register_adt_instance(d,args,id);}}
                    tys.kind(t)
                },
                Kind::NominalRef{region,mutable,target,regions}=>{
                    let record=definitions.get(target).ok_or("native nominal reference target is absent")?;
                    let count=record.regions.as_ref().map_or(0,|r|r.names.len());
                    if !matches!(region,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64))||!regions.is_empty()&&regions.len()!=count||regions.iter().any(|r|!matches!(r,ty::Region::Static|ty::Region::Param(0..64))){return Err("native nominal reference region arity/tag".into());}
                    let args=tys.intern_args(&regions.iter().map(|&r|ty::GenericArg::Lifetime(r)).collect::<Vec<_>>());let referent=tys.intern(ty::TyKind::Adt(map(*target),args));ty::TyKind::Ref(*region,*mutable,referent)
                },
                Kind::NominalApply{base,regions}=>{
                    let t=child(*base)?;let Some(Node{kind:Kind::Nominal(record),..})=self.nodes.get(*base as usize)else{return Err("native nominal application must name an earlier definition".into());};
                    if regions.is_empty()||record.regions.as_ref().is_none_or(|r|r.names.len()!=regions.len())||regions.iter().any(|r|!matches!(r,ty::Region::Static|ty::Region::Param(0..64))){return Err("native nominal application region arity/tag".into());}
                    let ty::TyKind::Adt(def,_)=tys.kind(t)else{return Err("native nominal application definition type".into());};let args=tys.intern_args(&regions.iter().map(|&r|ty::GenericArg::Lifetime(r)).collect::<Vec<_>>());ty::TyKind::Adt(def,args)
                },
                Kind::Float(f)=>ty::TyKind::Float(*f),
                Kind::Tuple(ids)=>{if ids.len()>MAX_PARAMS{return Err("native type graph field bound".into());}let ts=ids.iter().map(|&id|child(id)).collect::<Result<Vec<_>,_>>()?;ty::TyKind::Tuple(tys.intern_tys(&ts))},
                Kind::Array(id,len)=>{if *len>MAX_ARRAY{return Err("native array length bound".into());}let t=child(*id)?;let c=tys.intern_const(ty::ConstKind::Int(*len as u128));ty::TyKind::Array(t,c)},
                Kind::Ref(r,m,id)=>{if !matches!(r,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64)){return Err("native reference region bound".into());}ty::TyKind::Ref(*r,*m,child(*id)?)},
                Kind::SliceRef(r,m,id)=>{if !matches!(r,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64)){return Err("native reference region bound".into());}let e=child(*id)?;if !check::collect::is_sized_ty(tys,e){return Err("native slice element must be sized".into());}let slice=tys.intern(ty::TyKind::Slice(e));ty::TyKind::Ref(*r,*m,slice)},
                Kind::StrRef(r,m)=>{if !matches!(r,ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64)){return Err("native reference region bound".into());}let text=tys.intern(ty::TyKind::Str);ty::TyKind::Ref(*r,*m,text)},
                Kind::FnPtr{params,ret,own_out,out_slots}=>{if params.len()>MAX_PARAMS{return Err("native function pointer signature bound".into());}let ps=params.iter().map(|&id|child(id)).collect::<Result<Vec<_>,_>>()?;let ret=child(*ret)?;let inputs=tys.intern_tys(&ps);let mut sig=ty::FnSig::plain(inputs,ret);sig.own_out=*own_out;sig.out_slots=*out_slots;ty::TyKind::FnPtr(tys.intern_sig(ty::Binder::plain(sig)))},
            };
            if depth>MAX_DEPTH{return Err("native type graph depth bound".into());}total=total.checked_add(cost).ok_or("native type graph work bound")?;if total>MAX_EDGES{return Err("native type graph work bound".into());}costs.push(cost);types.push(tys.intern(kind));depths.push(depth);if !staging{self.check_layout(tys,&shapes,n,types[n])?;}
            if matches!(node.kind,Kind::FnPtr{..}){self.kept(n as u32)?;if !supported_at(None,tys,types[n],0,&mut MAX_EDGES){return Err("native function pointer signature bound or qualifier/region records".into());}}
        }Ok(types)
    }
    fn check_layout(&self,tys:&ty::GlobalTys,shapes:&Shapes,n:usize,t:ty::Ty)->Result<(),String>{
        let node=&self.nodes[n];let actual=ty::layout::layout_of_ty(tys,shapes,t).map_err(|_|"native type graph layout is unsupported")?;
        fn copy(tys:&ty::GlobalTys,t:ty::Ty)->bool{match tys.kind(t){ty::TyKind::Adt(..)=>false,ty::TyKind::Ref(_,m,_)=>!m.is_mut(),ty::TyKind::Tuple(ts)=>tys.tys_of(ts).iter().all(|&t|copy(tys,t)),ty::TyKind::Array(t,_)=>copy(tys,t),_=>true}}
        if actual.size>MAX_SIZE||actual!=node.layout||node.copy!=copy(tys,t)||node.drop{return Err("native type graph layout/Copy/drop mismatch".into());}Ok(())
    }
    pub(super) fn validate(&self)->Result<(),String>{
        self.intern(&mut ty::GlobalTys::new())?;if self.templates.len()>MAX_PARAMS{return Err("native symbolic nominal count bound".into());}let mut ids=std::collections::BTreeSet::new();for r in self.nominals(){if !ids.insert(r.id){return Err("native nominal duplicate definition".into());}}for t in &self.templates{t.validate()?;}Ok(())
    }
    pub(super) fn stage(&self,p:&mut check::collect::Program,krate:def::CrateId)->Result<Vec<ty::Ty>,String>{
        let types=self.intern_into(&mut p.tys,p.adts.len() as u32,&|d|def::DefId{krate,hash:d.hash},true)?;
        for r in self.closed_nominals(){r.stage(p,def::DefId{krate,hash:r.id.hash},&types)?;}
        for t in &self.templates{t.stage(p,def::DefId{krate,hash:t.record.id.hash})?;}Ok(types)
    }
    pub(super) fn check_native(&self,p:&check::collect::Program,types:&[ty::Ty])->Result<(),String>{
        for (node,&t)in self.nodes.iter().zip(types){
            if let Kind::Instance(a)=&node.kind{let instance=p.tys.local_adt(t).ok_or("native application instance was not materialized")?;let actual=p.adt(instance);if actual.is_enum!=a.is_enum||actual.variants.len()!=a.fields.len()||actual.variants.iter().zip(&a.fields).any(|(v,ids)|v.fields.len()!=ids.len()||v.fields.iter().zip(ids).any(|(f,&id)|f.ty!=types[id as usize])){return Err("native application original materialized fields mismatch".into());}}
            if ty::layout::layout_of_ty(&p.tys,p,t).ok().as_ref()!=Some(&node.layout)||node.copy!=check::collect::is_copy(p,&p.tys,t)||node.drop!=check::collect::needs_drop(p,&p.tys,t)||Some(&node.abi)!=cgen::native_type_abi(p,t).as_ref(){return Err("native type graph consumer layout/drop/C ABI mismatch".into());}
        }Ok(())
    }
    pub(super) fn publish(&self,p:&mut check::collect::Program,krate:def::CrateId,types:&[ty::Ty])->Result<(),String>{
        for r in self.closed_nominals(){r.publish(p,def::DefId{krate,hash:r.id.hash},types)?;}
        for t in &self.templates{t.publish(p,def::DefId{krate,hash:t.record.id.hash})?;}Ok(())
    }
    pub(super) fn encode(&self)->Vec<u8>{
        let mut out=Vec::new();u32le(&mut out,self.nodes.len() as u32);
        for node in &self.nodes{
            match &node.kind{
                Kind::Nominal(r)=>{out.push(r.wire_tag());r.encode(&mut out);},
                Kind::Instance(a)=>{out.push(29);a.encode(&mut out);},
                Kind::NominalApply{base,regions}=>{out.push(25);u32le(&mut out,*base);u32le(&mut out,regions.len()as u32);for &r in regions{region_out(&mut out,r);}},
                Kind::NominalRef{region,mutable,target,regions}=>{out.push(26);region_out(&mut out,*region);out.push(mutable.is_mut()as u8);out.extend_from_slice(&target.to_bytes());u32le(&mut out,regions.len()as u32);for &r in regions{region_out(&mut out,r);}},
                Kind::Scalar(s)=>out.push(s.tag()),Kind::Float(ty::FloatTy::F32)=>out.push(16),Kind::Float(ty::FloatTy::F64)=>out.push(17),
                Kind::Tuple(ids)=>{out.push(18);ids_out(&mut out,ids);},
                Kind::Array(id,len)=>{out.push(19);u32le(&mut out,*id);u64le(&mut out,*len);},
                Kind::Ref(r,m,id)=>{out.push(21);region_out(&mut out,*r);out.push(m.is_mut() as u8);u32le(&mut out,*id);},
                Kind::SliceRef(r,m,id)=>{out.push(27);region_out(&mut out,*r);out.push(m.is_mut() as u8);u32le(&mut out,*id);},
                Kind::StrRef(r,m)=>{out.push(28);region_out(&mut out,*r);out.push(m.is_mut() as u8);},
                Kind::FnPtr{params,ret,own_out,out_slots}=>{out.push(20);ids_out(&mut out,params);u32le(&mut out,*ret);u32le(&mut out,*own_out);out.push(*out_slots);},
            }
            let l=&node.layout;u64le(&mut out,l.size);u64le(&mut out,l.align);u32le(&mut out,l.field_offsets.len() as u32);for &o in &l.field_offsets{u64le(&mut out,o);}
            out.push(l.niche.is_some() as u8);if let Some(n)=l.niche{u64le(&mut out,n.offset);u64le(&mut out,n.scalar_size);out.extend_from_slice(&n.free.to_le_bytes());out.extend_from_slice(&n.start.to_le_bytes());}
            out.extend_from_slice(&[l.uninhabited as u8,node.copy as u8,node.drop as u8]);u64le(&mut out,node.abi.size);u64le(&mut out,node.abi.align);out.extend_from_slice(&[node.abi.param_dropped as u8,node.abi.param_by_ptr as u8,node.abi.ret_by_out as u8]);
            if enum_node(&node.kind,&self.nodes).expect("validated native enum application"){enum_layout_out(&mut out,l);}
        }
        if !self.templates.is_empty(){out.extend_from_slice(b"NGEN");u32le(&mut out,self.templates.len()as u32);for t in &self.templates{t.encode(&mut out);}}
        out
    }
    pub(super) fn decode(bytes:&[u8])->Result<Self,String>{
        let mut r=Reader(bytes);let count=r.len(MAX_NODES)?;let mut nodes:Vec<Node>=Vec::new();
        for _ in 0..count{
            let kind=match r.byte()?{29=>Kind::Instance(Box::new(generic_applications::Application::decode(&mut r)?)),tag@22..=24=>Kind::Nominal(Box::new(nominals::Record::decode(&mut r,tag)?)),25=>{let base=number(&mut r)?;let regions=(0..r.len(64)?).map(|_|region_in(&mut r)).collect::<Result<_,_>>()?;Kind::NominalApply{base,regions}},26=>{let region=region_in(&mut r)?;let mutable=ty::Mut::from_bool(r.boolean()?);let target=def::DiskDefId::from_bytes(r.bytes(16)?).unwrap();let regions=(0..r.len(64)?).map(|_|region_in(&mut r)).collect::<Result<_,_>>()?;Kind::NominalRef{region,mutable,target,regions}},tag@0..=15=>Kind::Scalar(Scalar::read(tag)?),16=>Kind::Float(ty::FloatTy::F32),17=>Kind::Float(ty::FloatTy::F64),18=>Kind::Tuple(ids_in(&mut r)?),19=>Kind::Array(number(&mut r)?,wide(&mut r)?),20=>Kind::FnPtr{params:ids_in(&mut r)?,ret:number(&mut r)?,own_out:number(&mut r)?,out_slots:r.byte()?},21=>Kind::Ref(region_in(&mut r)?,ty::Mut::from_bool(r.boolean()?),number(&mut r)?),27=>Kind::SliceRef(region_in(&mut r)?,ty::Mut::from_bool(r.boolean()?),number(&mut r)?),28=>Kind::StrRef(region_in(&mut r)?,ty::Mut::from_bool(r.boolean()?)),_=>return Err("unknown native type tag".into())};
            let size=wide(&mut r)?;let align=wide(&mut r)?;let fields=r.len(MAX_PARAMS)?;let field_offsets=(0..fields).map(|_|wide(&mut r)).collect::<Result<_,_>>()?;
            let niche=if r.boolean()?{Some(ty::layout::Niche{offset:wide(&mut r)?,scalar_size:wide(&mut r)?,free:u128::from_le_bytes(r.bytes(16)?.try_into().unwrap()),start:u128::from_le_bytes(r.bytes(16)?.try_into().unwrap())})}else{None};
            let mut layout=ty::layout::Layout{size,align,field_offsets,niche,uninhabited:r.boolean()?,variant_offsets:Vec::new(),enum_tag:None};let copy=r.boolean()?;let drop=r.boolean()?;
            let abi=cgen::NativeTypeAbi{size:wide(&mut r)?,align:wide(&mut r)?,param_dropped:r.boolean()?,param_by_ptr:r.boolean()?,ret_by_out:r.boolean()?};
            if enum_node(&kind,&nodes)?{enum_layout_in(&mut r,&mut layout)?;}
            nodes.push(Node{kind,layout,copy,drop,abi});
        }
        let mut templates=Vec::new();if !r.0.is_empty(){if r.bytes(4)?!=b"NGEN"{return Err("native type graph trailing data".into());}let count=r.len(MAX_PARAMS)?;if count==0{return Err("native symbolic nominal noncanonical empty table".into());}for _ in 0..count{templates.push(super::generic_nominals::Template::decode(&mut r)?);}}
        if !r.0.is_empty(){return Err("native type graph trailing data".into());}let graph=Self{nodes,templates};graph.validate()?;Ok(graph)
    }
}
fn enum_node(kind:&Kind,nodes:&[Node])->Result<bool,String>{match kind{Kind::Instance(a)=>Ok(a.is_enum),Kind::Nominal(r)=>Ok(r.is_enum),Kind::NominalApply{base,..}=>match nodes.get(*base as usize){Some(Node{kind:Kind::Nominal(r),..})=>Ok(r.is_enum),_=>Err("native nominal application must name an earlier definition".into())},_=>Ok(false)}}
pub(super) fn u32le(out:&mut Vec<u8>,n:u32){out.extend_from_slice(&n.to_le_bytes());}
fn niche_out(out:&mut Vec<u8>,n:ty::layout::Niche){u64le(out,n.offset);u64le(out,n.scalar_size);out.extend_from_slice(&n.free.to_le_bytes());out.extend_from_slice(&n.start.to_le_bytes());}
fn niche_in(r:&mut Reader<'_>)->Result<ty::layout::Niche,String>{Ok(ty::layout::Niche{offset:wide(r)?,scalar_size:wide(r)?,free:u128::from_le_bytes(r.bytes(16)?.try_into().unwrap()),start:u128::from_le_bytes(r.bytes(16)?.try_into().unwrap())})}
fn enum_layout_out(out:&mut Vec<u8>,l:&ty::layout::Layout){
    u32le(out,l.variant_offsets.len()as u32);for fields in &l.variant_offsets{u32le(out,fields.len()as u32);for &offset in fields{u64le(out,offset);}}
    match l.enum_tag{None=>out.push(0),Some(ty::layout::EnumTag::None(v))=>{out.push(1);u32le(out,v as u32);},Some(ty::layout::EnumTag::Explicit{size,signed})=>{out.push(2);u64le(out,size);out.push(signed as u8);},Some(ty::layout::EnumTag::NicheFilled{variant,niche,first,last,start})=>{out.push(3);u32le(out,variant as u32);niche_out(out,niche);u32le(out,first as u32);u32le(out,last as u32);out.extend_from_slice(&start.to_le_bytes());}}
}
fn enum_layout_in(r:&mut Reader<'_>,l:&mut ty::layout::Layout)->Result<(),String>{
    for _ in 0..r.len(MAX_PARAMS)?{let fields=(0..r.len(MAX_PARAMS)?).map(|_|wide(r)).collect::<Result<_,_>>()?;l.variant_offsets.push(fields);}
    l.enum_tag=match r.byte()?{0=>None,1=>Some(ty::layout::EnumTag::None(number(r)? as usize)),2=>Some(ty::layout::EnumTag::Explicit{size:wide(r)?,signed:r.boolean()?}),3=>Some(ty::layout::EnumTag::NicheFilled{variant:number(r)? as usize,niche:niche_in(r)?,first:number(r)? as usize,last:number(r)? as usize,start:u128::from_le_bytes(r.bytes(16)?.try_into().unwrap())}),_=>return Err("native enum layout tag".into())};Ok(())
}
fn u64le(out:&mut Vec<u8>,n:u64){out.extend_from_slice(&n.to_le_bytes());}
pub(super) fn ids_out(out:&mut Vec<u8>,ids:&[u32]){u32le(out,ids.len() as u32);for &id in ids{u32le(out,id);}}
pub(super) fn ids_in(r:&mut Reader<'_>)->Result<Vec<u32>,String>{let n=r.len(MAX_PARAMS)?;(0..n).map(|_|number(r)).collect()}
pub(super) fn number(r:&mut Reader<'_>)->Result<u32,String>{Ok(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()))}
fn wide(r:&mut Reader<'_>)->Result<u64,String>{Ok(u64::from_le_bytes(r.bytes(8)?.try_into().unwrap()))}
pub(super) fn region_out(out:&mut Vec<u8>,r:ty::Region){match r{ty::Region::Erased=>out.push(0),ty::Region::Static=>out.push(1),ty::Region::Param(n)=>{out.push(2);u32le(out,n);},_=>unreachable!("validated native region")}}
pub(super) fn region_in(r:&mut Reader<'_>)->Result<ty::Region,String>{match r.byte()?{0=>Ok(ty::Region::Erased),1=>Ok(ty::Region::Static),2=>{let n=number(r)?;if n>=64{return Err("native reference region bound".into());}Ok(ty::Region::Param(n))},_=>Err("native reference region tag".into())}}
