//! Symbolic nominal templates. An uninstantiated template has no machine
//! layout or ABI. Original native generics/mono creates consumer instances.
use super::{Reader,Scalar,nominals,MAX_PARAMS};
use crate::{ast,check,def,diag,ty};
use ty::TyIntern;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Parameter {Type(String),Const(String,Scalar)}
#[derive(Clone,Debug,PartialEq,Eq)]
enum Length {Value(u64),Parameter(u32)}
#[derive(Clone,Debug,PartialEq,Eq)]
enum Symbolic {Scalar(Scalar),Parameter(u32),Tuple(Vec<Symbolic>),Array(Box<Symbolic>,Length)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Template {
    pub record:nominals::Record,
    parameters:Vec<Parameter>,
    // Exact original implicit Sized predicates, by combined type/const index.
    sized:Vec<u32>,
    fields:Vec<Symbolic>,
}
fn number(r:&mut Reader<'_>)->Result<u32,String>{Ok(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()))}
fn count(out:&mut Vec<u8>,n:u32){out.extend_from_slice(&n.to_le_bytes());}
fn name(p:&Parameter)->&str{match p{Parameter::Type(n)|Parameter::Const(n,_)=>n}}
impl Symbolic {
    fn capture(p:&check::collect::Program,t:ty::Ty,depth:usize,budget:&mut usize)->Result<Self,String>{
        if depth>128||*budget==0{return Err("native symbolic field depth/work bound".into());}*budget-=1;
        Ok(match p.tys.kind(t){
            ty::TyKind::Param(n)=>Self::Parameter(n.0),
            ty::TyKind::Tuple(ts)=>{let ts=p.tys.tys_of(ts);if ts.len()>MAX_PARAMS{return Err("native symbolic tuple bound".into());}Self::Tuple(ts.iter().map(|&t|Self::capture(p,t,depth+1,budget)).collect::<Result<_,_>>()?)},
            ty::TyKind::Array(t,c)=>{let n=match p.tys.const_kind(c){ty::ConstKind::Int(n)if n<=1024*1024=>Length::Value(n as u64),ty::ConstKind::Param(n)=>Length::Parameter(n),_=>return Err("native symbolic array needs a concrete length or declared const parameter".into())};Self::Array(Box::new(Self::capture(p,t,depth+1,budget)?),n)},
            _=>Self::Scalar(Scalar::from_ty(p,t)?),
        })
    }
    fn validate(&self,params:&[Parameter],depth:usize,budget:&mut usize)->Result<(),String>{
        if depth>128||*budget==0{return Err("native symbolic field depth/work bound".into());}*budget-=1;
        match self {
            Self::Scalar(Scalar::Never)=>Err("native symbolic never field remains unqualified".into()),
            Self::Scalar(_)=>Ok(()),
            Self::Parameter(n)=>if matches!(params.get(*n as usize),Some(Parameter::Type(_))){Ok(())}else{Err("native symbolic type parameter kind/index".into())},
            Self::Tuple(ts)=>{if ts.len()>MAX_PARAMS{return Err("native symbolic tuple bound".into());}for t in ts{t.validate(params,depth+1,budget)?;}Ok(())},
            Self::Array(t,n)=>{match n{Length::Value(n)if *n<=1024*1024=>{},Length::Parameter(n)if matches!(params.get(*n as usize),Some(Parameter::Const(_,Scalar::Int(ty::IntTy::Usize))))=>{},_=>return Err("native symbolic array const kind/index/bound".into())}t.validate(params,depth+1,budget)},
        }
    }
    fn intern(&self,tys:&mut ty::GlobalTys)->ty::Ty{match self{
        Self::Scalar(s)=>s.intern(tys),Self::Parameter(n)=>tys.intern(ty::TyKind::Param(ty::ParamId(*n))),
        Self::Tuple(ts)=>{let ts:Vec<_>=ts.iter().map(|t|t.intern(tys)).collect();let ts=tys.intern_tys(&ts);tys.intern(ty::TyKind::Tuple(ts))},
        Self::Array(t,n)=>{let t=t.intern(tys);let n=tys.intern_const(match n{Length::Value(n)=>ty::ConstKind::Int(*n as u128),Length::Parameter(n)=>ty::ConstKind::Param(*n)});tys.intern(ty::TyKind::Array(t,n))},
    }}
    fn encode(&self,out:&mut Vec<u8>){match self{
        Self::Scalar(s)=>{out.push(0);out.push(s.tag());},Self::Parameter(n)=>{out.push(1);count(out,*n);},
        Self::Tuple(ts)=>{out.push(2);count(out,ts.len()as u32);for t in ts{t.encode(out);}},
        Self::Array(t,n)=>{out.push(3);t.encode(out);match n{Length::Value(n)=>{out.push(0);out.extend_from_slice(&n.to_le_bytes());},Length::Parameter(n)=>{out.push(1);count(out,*n);}}},
    }}
    fn type_parameters(&self,out:&mut std::collections::BTreeSet<u32>){match self{Self::Parameter(n)=>{out.insert(*n);},Self::Tuple(ts)=>for t in ts{t.type_parameters(out)},Self::Array(t,_)=>t.type_parameters(out),Self::Scalar(_)=>{}}}
    fn decode(r:&mut Reader<'_>,depth:usize,budget:&mut usize)->Result<Self,String>{
        if depth>128||*budget==0{return Err("native symbolic field depth/work bound".into());}*budget-=1;
        Ok(match r.byte()?{
            0=>Self::Scalar(Scalar::read(r.byte()?)?),1=>Self::Parameter(number(r)?),
            2=>Self::Tuple((0..r.len(MAX_PARAMS)?).map(|_|Self::decode(r,depth+1,budget)).collect::<Result<_,_>>()?),
            3=>{let t=Box::new(Self::decode(r,depth+1,budget)?);let n=match r.byte()?{0=>Length::Value(u64::from_le_bytes(r.bytes(8)?.try_into().unwrap())),1=>Length::Parameter(number(r)?),_=>return Err("native symbolic const tag".into())};Self::Array(t,n)},
            _=>return Err("native symbolic type tag".into()),
        })
    }
}
impl Template {
    pub(super) fn parameters(&self)->&[Parameter]{&self.parameters}
    pub(super) fn substituted_shape(&self,tys:&mut ty::GlobalTys,args:&[ty::GenericArg])->ty::layout::AdtShape{
        let fields=self.fields.iter().map(|t|{let t=t.intern(tys);ty::subst(tys,t,args)}).collect::<Vec<_>>();self.record.shape(&fields)
    }
    pub(super) fn capture(p:&check::collect::Program,a:ty::AdtId)->Result<Self,String>{
        let d=p.def_of_adt(a);let adt=p.adt(a);let g=&p.gx.adt_generics[a.0 as usize];let derives=adt.derives_unlogged();
        if d.krate!=p.krate||g.parent.is_some()||g.params.is_empty()||g.params.len()>MAX_PARAMS
            ||p.gx.adt_instance_of[a.0 as usize].is_some()||p.gx.adt_lifetimes.get(&d).is_some_and(|v|!v.is_empty())
            ||p.gx.param_defaults.contains_key(&d)||!p.gx.adt_scopes[a.0 as usize].region_facts.is_empty()
            ||adt.non_exhaustive||adt.repr!=Default::default()||adt.variants.iter().any(|v|v.non_exhaustive)
            ||derives.copy||derives.clone||derives.debug||derives.partial_eq||derives.eq||derives.hash||derives.partial_ord||derives.ord||derives.default||p.drop_adts.contains(&a)
        {return Err("native symbolic nominal requires additional lifetime/default/derive/drop/representation records".into());}
        let mut parameters=Vec::new();for q in &g.params{parameters.push(match q.kind{ty::ParamKind::Type=>Parameter::Type(p.name(q.name).into()),ty::ParamKind::Const(t)=>Parameter::Const(p.name(q.name).into(),Scalar::from_ty(p,t)?),_=>return Err("native symbolic parameter kind".into())});}
        let mut sized=Vec::new();for pred in &g.predicates{let ty::Predicate::Trait(b)=pred else{return Err("native symbolic nominal needs general predicate records".into());};let tr=b.value;let args=p.tys.args(tr.args);if b.count!=0||tr.def!=p.gx.lang.sized||tr.bindings!=ty::EMPTY_BINDINGS||args.len()!=1{return Err("native symbolic nominal needs general predicate records".into());}let ty::GenericArg::Ty(t)=args[0]else{return Err("native symbolic Sized argument".into());};let ty::TyKind::Param(n)=p.tys.kind(t)else{return Err("native symbolic Sized parameter".into());};sized.push(n.0);}
        let mut fields=Vec::new();let mut budget=32768;let t=p.tys.get_adt_ty(a).ok_or("native symbolic nominal definition type missing")?;
        let record=nominals::Record::capture_template(p,t,|t|{let n=fields.len()as u32;fields.push(Symbolic::capture(p,t,0,&mut budget)?);Ok(n)})?;
        if record.regions.is_some(){return Err("native symbolic nominal needs kept lifetime fields".into());}
        let result=Self{record,parameters,sized,fields};result.validate()?;Ok(result)
    }
    pub(super) fn validate(&self)->Result<(),String>{
        self.record.validate()?;if self.record.regions.is_some()||self.parameters.is_empty()||self.parameters.len()>MAX_PARAMS{return Err("native symbolic nominal parameter/region bound".into());}
        let mut names=std::collections::BTreeSet::new();for p in &self.parameters{let n=name(p);if !crate::lexer::lifetime_name(n)||n=="_"||n=="Self"||!names.insert(n){return Err("native symbolic parameter name/identity".into());}if let Parameter::Const(_,s)=p{if !matches!(s,Scalar::Bool|Scalar::Char|Scalar::Int(_)){return Err("native symbolic const parameter type".into());}}}
        let expected:Vec<_>=self.parameters.iter().enumerate().filter_map(|(n,p)|matches!(p,Parameter::Type(_)).then_some(n as u32)).collect();if self.sized!=expected{return Err("native symbolic nominal exact Sized obligations".into());}
        if self.fields.len()!=self.record.fields().count()||self.record.fields().enumerate().any(|(n,f)|f.ty!=n as u32){return Err("native symbolic nominal field root identity".into());}
        let mut budget=32768;let mut used=std::collections::BTreeSet::new();for f in &self.fields{f.validate(&self.parameters,0,&mut budget)?;f.type_parameters(&mut used);}if self.parameters.iter().enumerate().any(|(n,p)|matches!(p,Parameter::Type(_))&&!used.contains(&(n as u32))){return Err("native symbolic nominal unused type parameter".into());}
        if self.record.is_enum&&self.record.variants.iter().any(|v|v.disc<i64::MIN as i128||v.disc>i64::MAX as i128){return Err("native symbolic enum default isize discriminant range".into());}Ok(())
    }
    pub(super) fn stage(&self,p:&mut check::collect::Program,d:def::DefId)->Result<(),String>{
        self.validate()?;let a=ty::AdtId(p.adts.len()as u32);match p.tys.local_adt_of(d){None=>{p.tys.register_adt(d,a);},Some(id)if id==a=>{},_=>return Err("native symbolic nominal identity already registered".into())}
        let fields:Vec<_>=self.fields.iter().map(|t|t.intern(&mut p.tys)).collect();self.record.stage(p,d,&fields)?;
        let mut scope=check::generics::Scope::default();let mut params=Vec::new();for(n,q)in self.parameters.iter().enumerate(){let name=p.interner.intern(name(q));let kind=match q{Parameter::Type(_)=>{let t=p.tys.intern(ty::TyKind::Param(ty::ParamId(n as u32)));scope.type_params.push((name,n as u32));scope.param_tys.push((name,t));ty::ParamKind::Type},Parameter::Const(_,s)=>{let t=s.intern(&mut p.tys);scope.const_params.push((name,n as u32,t));ty::ParamKind::Const(t)}};params.push(ty::ParamDef{name,kind,span:diag::Span::default()});}
        let predicates=self.sized.iter().map(|&n|{let t=p.tys.intern(ty::TyKind::Param(ty::ParamId(n)));let args=p.tys.intern_args(&[ty::GenericArg::Ty(t)]);ty::Predicate::Trait(ty::Binder::plain(ty::TraitRef{def:p.gx.lang.sized,args,bindings:ty::EMPTY_BINDINGS}))}).collect::<Vec<_>>();
        p.gx.adt_info[a.0 as usize]=(d,params.len());p.gx.adt_generics[a.0 as usize]=ty::Generics{parent:None,params,predicates:predicates.clone()};p.gx.adt_envs[a.0 as usize]=predicates;p.gx.adt_scopes[a.0 as usize]=scope;p.gx.has_generics=true;p.gx.user_generics=true;Ok(())
    }
    pub(super) fn publish(&self,p:&mut check::collect::Program,d:def::DefId)->Result<(),String>{
        self.record.publish(p,d,&[])?;let a=p.defs.local_adt(d).ok_or("native symbolic nominal shape missing")?;let g=p.gx.adt_generics[a.0 as usize].clone();p.defs.get_mut(d).ok_or("native symbolic nominal definition missing")?.generics=g;Ok(())
    }
    pub(super) fn encode(&self,out:&mut Vec<u8>){out.push(self.record.wire_tag());self.record.encode(out);count(out,self.parameters.len()as u32);for p in &self.parameters{super::text(out,name(p));match p{Parameter::Type(_)=>out.push(0),Parameter::Const(_,s)=>{out.push(1);out.push(s.tag());}}}count(out,self.sized.len()as u32);for &n in &self.sized{count(out,n);}count(out,self.fields.len()as u32);for t in &self.fields{t.encode(out);}}
    pub(super) fn decode(r:&mut Reader<'_>)->Result<Self,String>{let tag=r.byte()?;if !matches!(tag,22|23){return Err("native symbolic nominal shape tag".into());}let record=nominals::Record::decode(r,tag)?;let parameters=(0..r.len(MAX_PARAMS)?).map(|_|{let n=r.text(1024)?;Ok(match r.byte()?{0=>Parameter::Type(n),1=>Parameter::Const(n,Scalar::read(r.byte()?)?),_=>return Err("native symbolic parameter kind tag".into())})}).collect::<Result<_,String>>()?;let sized=(0..r.len(MAX_PARAMS)?).map(|_|number(r)).collect::<Result<_,_>>()?;let mut budget=32768;let fields=(0..r.len(MAX_PARAMS*MAX_PARAMS)?).map(|_|Symbolic::decode(r,0,&mut budget)).collect::<Result<_,_>>()?;let result=Self{record,parameters,sized,fields};result.validate()?;Ok(result)}
}
