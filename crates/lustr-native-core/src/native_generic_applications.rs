//! Concrete applications of the exact symbolic template in this image.
//! Field roots must equal original substitution; machine facts are checked
//! independently against the original native consumer materializer.
use super::{Reader,Scalar,MAX_PARAMS,generic_nominals::Template};
use crate::{check,def,ty};
use ty::TyIntern;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Argument{Type(u32),Const(Scalar,u128)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Application{pub target:def::DiskDefId,pub is_enum:bool,pub args:Vec<Argument>,pub fields:Vec<Vec<u32>>}
pub(super) fn template(p:&check::collect::Program,tys:&impl TyIntern,t:ty::Ty)->Option<ty::AdtId>{
    let ty::TyKind::Adt(d,args)=tys.kind(t)else{return None;};let a=p.defs.local_adt(d)?;
    (d.krate==p.krate&&!tys.args(args).is_empty()&&!p.gx.adt_generics[a.0 as usize].params.is_empty()).then_some(a)
}
pub(super) fn eligible(p:&check::collect::Program,tys:&impl TyIntern,t:ty::Ty)->bool{
    let ty::TyKind::Adt(d,args)=tys.kind(t)else{return false;};template(p,tys,t).is_some_and(|a|Template::capture(p,a).is_ok())&&tys.local_adt_instance(d,args).is_some()
}
fn constant(s:Scalar,bits:u128)->Result<ty::ConstKind,String>{
    Ok(match s{
        Scalar::Bool if bits<=1=>ty::ConstKind::Bool(bits!=0),
        Scalar::Char=>ty::ConstKind::Char(u32::try_from(bits).ok().and_then(char::from_u32).ok_or("native application char range")?),
        Scalar::Int(i)=>{let n=i.bits();if n<128&&if i.signed(){let v=bits as i128;let hi=1i128<<(n-1);v< -hi||v>=hi}else{bits>=1u128<<n}{return Err("native application integer range".into());}ty::ConstKind::Int(bits)},
        _=>return Err("native application const type/value".into()),
    })
}
impl Application{
    pub(super) fn capture(p:&check::collect::Program,t:ty::Ty,mut child:impl FnMut(ty::Ty)->Result<u32,String>)->Result<Self,String>{
        let a=template(p,&p.tys,t).ok_or("native application needs its original local template")?;let ty::TyKind::Adt(d,args)=p.tys.kind(t)else{unreachable!()};
        let raw=p.tys.args(args);let params=&p.gx.adt_generics[a.0 as usize].params;if raw.len()!=params.len(){return Err("native application argument arity".into());}
        let mut args=Vec::new();for(g,q)in raw.iter().zip(params){args.push(match(g,q.kind){
            (ty::GenericArg::Ty(t),ty::ParamKind::Type)=>Argument::Type(child(*t)?),
            (ty::GenericArg::Const(c),ty::ParamKind::Const(t))=>{let s=Scalar::from_ty(p,t)?;let bits=match p.tys.const_kind(*c){ty::ConstKind::Int(n)=>n,ty::ConstKind::Bool(b)=>b as u128,ty::ConstKind::Char(c)=>c as u128,_=>return Err("native application const is not concrete".into())};if constant(s,bits)?!=p.tys.const_kind(*c){return Err("native application const kind".into());}Argument::Const(s,bits)},
            _=>return Err("native application argument kind".into()),
        });}
        let adt=p.adt(p.tys.local_adt(t).ok_or("native application has no original materialized instance")?);
        let fields=adt.variants.iter().map(|v|v.fields.iter().map(|f|child(f.ty)).collect::<Result<_,_>>()).collect::<Result<_,_>>()?;
        Ok(Self{target:def::DiskDefId{krate:p.stable_crate,hash:d.hash},is_enum:adt.is_enum,args,fields})
    }
    pub(super) fn intern(&self,tys:&mut ty::GlobalTys,template:&Template,def:def::DefId,mut child:impl FnMut(u32)->Result<ty::Ty,String>)->Result<(ty::Ty,ty::layout::AdtShape),String>{
        use super::generic_nominals::Parameter;
        if self.target!=template.record.id||self.is_enum!=template.record.is_enum||self.args.len()!=template.parameters().len(){return Err("native application template identity/kind/arity".into());}
        let mut args=Vec::new();for(g,q)in self.args.iter().zip(template.parameters()){args.push(match(g,q){
            (Argument::Type(n),Parameter::Type(_))=>{let t=child(*n)?;if !check::collect::is_sized_ty(tys,t){return Err("native application original Sized obligation".into());}ty::GenericArg::Ty(t)},
            (Argument::Const(s,bits),Parameter::Const(_,want))if s==want=>ty::GenericArg::Const(tys.intern_const(constant(*s,*bits)?)),
            _=>return Err("native application argument kind/type".into()),
        });}
        let shape=template.substituted_shape(tys,&args);if self.fields.len()!=shape.variants.len(){return Err("native application variant arity".into());}
        for(ids,fields)in self.fields.iter().zip(&shape.variants){if ids.len()!=fields.len(){return Err("native application field arity".into());}for(&id,&t)in ids.iter().zip(fields){if child(id)?!=t{return Err("native application original substituted field mismatch".into());}}}
        let args=tys.intern_args(&args);Ok((tys.intern(ty::TyKind::Adt(def,args)),shape))
    }
    pub(super) fn encode(&self,out:&mut Vec<u8>){out.extend_from_slice(&self.target.to_bytes());out.push(self.is_enum as u8);super::types::u32le(out,self.args.len()as u32);for g in &self.args{match g{Argument::Type(n)=>{out.push(0);super::types::u32le(out,*n);},Argument::Const(s,bits)=>{out.extend_from_slice(&[1,s.tag()]);out.extend_from_slice(&bits.to_le_bytes());}}}super::types::u32le(out,self.fields.len()as u32);for f in &self.fields{super::types::ids_out(out,f);}}
    pub(super) fn decode(r:&mut Reader<'_>)->Result<Self,String>{let target=def::DiskDefId::from_bytes(r.bytes(16)?).unwrap();let is_enum=r.boolean()?;let args=(0..r.len(MAX_PARAMS)?).map(|_|Ok(match r.byte()?{0=>Argument::Type(super::types::number(r)?),1=>Argument::Const(Scalar::read(r.byte()?)?,u128::from_le_bytes(r.bytes(16)?.try_into().unwrap())),_=>return Err("native application argument tag".into())})).collect::<Result<_,String>>()?;let fields=(0..r.len(MAX_PARAMS)?).map(|_|super::types::ids_in(r)).collect::<Result<_,_>>()?;Ok(Self{target,is_enum,args,fields})}
}
