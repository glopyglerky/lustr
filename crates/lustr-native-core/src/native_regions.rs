//! Original native region signatures, separate from region-erased check types.
//! Wire references are image type indices and local universal-region indices.
use super::{Reader,types,MAX_PARAMS};
use crate::{ast,borrowck,check,def,ty};

/// Lifetime-only declarations need no monomorphized body. Keep their names
/// independently of diagnostic strings, and retain the original checker's
/// declared facts and early/late classification. Type/const parameters and
/// predicates still need their own native records.
pub(super) fn declarations(p:&check::collect::Program,id:def::LocalDefId)->bool{
    declared(p,id).is_ok()
}
fn declared(p:&check::collect::Program,id:def::LocalDefId)->Result<Vec<String>,String>{
    let f=p.fn_info(id);let written=f.def(&p.ast);
    if written.where_clause.preds.iter().any(|b|!matches!(b,ast::WherePred::Lifetime{..}))
        ||written.generics.params.len()>64||f.owner.is_some(){return Err("native function needs type/const/trait declaration records".into());}
    let names=written.generics.params.iter().map(|g|match g{
        ast::GenericParam::Lifetime{lt,attrs,..}if attrs.is_empty()=>Ok(p.name(lt.name).to_owned()),
        _=>Err("native function needs type/const/attribute declaration records".into()),
    }).collect::<Result<Vec<_>,String>>()?;
    if names.is_empty()&&written.where_clause.preds.is_empty(){return Ok(names);}
    let sig=p.regions.body_sig(p,id).ok_or("native function lacks original declaration region signature")?;
    let scope=p.gx.fn_scopes.get(id.0 as usize).ok_or("native function lacks original declaration scope")?;
    let mut facts=scope.scope.region_facts.iter().map(|&(a,b)|(ty::Region::Param(a),if b==check::generics::STATIC_FACT{ty::Region::Static}else{ty::Region::Param(b)})).collect::<Vec<_>>();canonical(&mut facts);
    let mut actual=sig.facts.clone();canonical(&mut actual);
    if names.len()!=sig.early.len()||names.iter().enumerate().any(|(n,s)|sig.names.get(n)!=Some(&format!("lifetime `'{s}`")))||facts!=actual{return Err("native function original declaration/scope/region mismatch".into());}Ok(names)
}
fn fact_key(&(a,b):&(ty::Region,ty::Region))->(u32,u32){let index=|r|match r{ty::Region::Param(n)=>n,ty::Region::Static=>u32::MAX,_=>u32::MAX-1};(index(a),index(b))}
fn canonical(facts:&mut Vec<(ty::Region,ty::Region)>){facts.sort_by_key(fact_key);facts.dedup();}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Signature {
    n:u32,params:Vec<u32>,ret:u32,facts:Vec<(ty::Region,ty::Region)>,
    names:Vec<String>,early:Vec<bool>,declared:Vec<String>,
}
impl Signature {
    pub(super) fn capture(p:&check::collect::Program,id:def::LocalDefId,graph:&mut types::Graph)->Result<Self,String>{
        let f=p.fn_info(id);
        let fallback=borrowck::RegionSig{params:f.params.clone(),..borrowck::RegionSig::const_value(f.ret)};
        let sig=p.regions.body_sig(p,id).unwrap_or(&fallback);
        if !sig.fn_bounds.is_empty()||!sig.trait_bounds.is_empty()||!sig.item_bounds.is_empty()||sig.opaque_caps.is_some()||!sig.header.is_empty()||!sig.bounds.is_empty(){return Err("native region signature needs generic/trait/opaque records".into());}
        let declared=declared(p,id)?;let mut facts=sig.facts.clone();canonical(&mut facts);
        Ok(Self{n:sig.n_univ,params:sig.params.iter().map(|&t|graph.capture(p,t)).collect::<Result<_,_>>()?,ret:graph.capture(p,sig.ret)?,facts,names:sig.names.clone(),early:sig.early.clone(),declared})
    }
    pub(super) fn install(&self,tys:&mut ty::GlobalTys,graph:&types::Graph,types:&[ty::Ty],params:&[u32],ret:u32)->Result<borrowck::RegionSig,String>{
        if self.n>64||self.names.len()!=self.n as usize||self.names.iter().any(|s|s.len()>1024)||self.early.len()>self.n as usize||self.facts.len()>4096||self.params.len()!=params.len()||params.len()>MAX_PARAMS{return Err("native region signature bounds".into());}
        let mut declared=std::collections::BTreeSet::new();
        if self.declared.len()!=self.early.len()||self.declared.iter().enumerate().any(|(n,s)|!crate::lexer::lifetime_name(s)||s=="static"||s=="_"||!declared.insert(s)||self.names.get(n)!=Some(&format!("lifetime `'{s}`"))){return Err("native function region declaration identity/count".into());}
        let ty=|id:u32|types.get(id as usize).copied().ok_or_else(||"native region type index".to_owned());
        for &id in self.params.iter().chain([&self.ret]){graph.kept(id)?;}
        let ps=self.params.iter().map(|&id|ty(id)).collect::<Result<Vec<_>,_>>()?;let out=ty(self.ret)?;
        let mut named=Vec::new();for &t in ps.iter().chain([&out]){borrowck::regions_of(tys,t,&mut named);}
        let valid=|r:ty::Region|matches!(r,ty::Region::Static)||matches!(r,ty::Region::Param(n) if n<self.n);
        if named.iter().any(|&r|!valid(r))||self.facts.iter().any(|&(a,b)|!valid(a)||!valid(b)){return Err("native region universal index/tag".into());}
        let declared=|r|matches!(r,ty::Region::Param(n) if (n as usize)<self.declared.len());
        if self.facts.windows(2).any(|w|fact_key(&w[0])>=fact_key(&w[1]))||self.facts.iter().any(|&(a,b)|!declared(a)||!(declared(b)||b==ty::Region::Static)){return Err("native function declared fact index/order".into());}
        // The original native generic collector classifies a named lifetime
        // as early when a written fact mentions it or no input names it.
        // Check that structural invariant before passing records to the
        // original call-instantiation and pointer-generality engines.
        let mut inputs=Vec::new();for &t in &ps{borrowck::regions_of(tys,t,&mut inputs);}
        if self.early.iter().enumerate().any(|(n,&early)|{let r=ty::Region::Param(n as u32);early!=(self.facts.iter().any(|&(a,b)|a==r||b==r)||!inputs.contains(&r))}){return Err("native function early/late region classification".into());}
        for (&kept,&erased)in ps.iter().zip(params){let erased=ty(erased)?;if borrowck::erase(tys,kept)!=erased||borrowck::erase(tys,erased)!=erased{return Err("native region/check type shape mismatch".into());}}
        let erased=ty(ret)?;if borrowck::erase(tys,out)!=erased||borrowck::erase(tys,erased)!=erased{return Err("native region/check type shape mismatch".into());}
        Ok(borrowck::RegionSig{n_univ:self.n,params:ps,ret:out,facts:self.facts.clone(),names:self.names.clone(),early:self.early.clone(),..borrowck::RegionSig::const_value(out)})
    }
    pub(super) fn encode(&self,out:&mut Vec<u8>){
        nout(out,self.n);nout(out,self.params.len() as u32);for &p in &self.params{nout(out,p);}nout(out,self.ret);
        nout(out,self.facts.len() as u32);for &(a,b)in &self.facts{types::region_out(out,a);types::region_out(out,b);}
        nout(out,self.names.len() as u32);for s in &self.names{super::text(out,s);}
        nout(out,self.early.len() as u32);out.extend(self.early.iter().map(|b|*b as u8));
        nout(out,self.declared.len()as u32);for s in &self.declared{super::text(out,s);}
    }
    pub(super) fn decode(r:&mut Reader<'_>)->Result<Self,String>{
        let n=r.len(64)? as u32;let c=r.len(MAX_PARAMS)?;let params=(0..c).map(|_|number(r)).collect::<Result<_,_>>()?;let ret=number(r)?;
        let c=r.len(4096)?;let facts=(0..c).map(|_|Ok::<_,String>((types::region_in(r)?,types::region_in(r)?))).collect::<Result<_,_>>()?;
        let c=r.len(64)?;let names=(0..c).map(|_|r.text(1024)).collect::<Result<_,_>>()?;let c=r.len(64)?;let early=(0..c).map(|_|r.boolean()).collect::<Result<_,_>>()?;
        let c=r.len(64)?;let declared=(0..c).map(|_|r.text(1024)).collect::<Result<_,_>>()?;
        Ok(Self{n,params,ret,facts,names,early,declared})
    }
}
fn nout(out:&mut Vec<u8>,n:u32){out.extend_from_slice(&n.to_le_bytes());}
fn number(r:&mut Reader<'_>)->Result<u32,String>{Ok(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()))}
