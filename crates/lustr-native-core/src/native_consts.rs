//! Evaluated closed constants. No AST indexes, allocation IDs or local fn
//! slots cross the image. Original const-body type/borrow checks run first.
use super::{Reader,Scalar,types,text,MAX_DEFS};
use crate::{check,def,diag,ty};
use ty::{TyIntern,consteval::ConstValue as CV,constqual::Qualif};
const MAX_VALUES:usize=32768;
const MAX_DEPTH:usize=128;

pub(super) fn check_initializers(p:&mut check::collect::Program,errors:&mut Vec<diag::Diag>){
    check::const_body::check_all_with(p,errors,&|p,f,checked|{
        // Reuse the native static walk on its actual transient initializer
        // row. The evaluator's selected path cannot erase a non-const call.
        check::constfn::check_const_fn_with_inline(p,checked,p.fn_info(f),false,true)
    });
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) enum Value{Unit,Bool(bool),Char(char),Int(i128),Float(u64,ty::FloatTy),Aggregate(Vec<Value>),Variant(u32,Vec<Value>),Repeat(Box<Value>,u64)}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Record{pub id:def::DiskDefId,pub name:String,pub ty:u32,value:Value,qualif:Qualif}

fn step(depth:usize,budget:&mut usize)->Result<(),String>{if depth>MAX_DEPTH||*budget==0{return Err("native constant value depth/work bound".into());}*budget-=1;Ok(())}
impl Value{
    fn capture(v:&CV,depth:usize,budget:&mut usize)->Result<Self,String>{step(depth,budget)?;Ok(match v{
        CV::Unit=>Self::Unit,CV::Bool(v)=>Self::Bool(*v),CV::Char(v)=>Self::Char(*v),CV::Int(v)=>Self::Int(*v),CV::Float(v,f)=>Self::Float(*v,*f),
        CV::Aggregate(v)=>Self::Aggregate(v.iter().map(|v|Self::capture(v,depth+1,budget)).collect::<Result<_,_>>()?),
        CV::Variant(n,v)=>Self::Variant(*n,v.iter().map(|v|Self::capture(v,depth+1,budget)).collect::<Result<_,_>>()?),
        CV::Repeat(v,n)=>Self::Repeat(Box::new(Self::capture(v,depth+1,budget)?),*n),
        _=>return Err("native constant needs reference/allocation/function identity records".into()),
    })}
    fn materialize(&self)->CV{match self{Self::Unit=>CV::Unit,Self::Bool(v)=>CV::Bool(*v),Self::Char(v)=>CV::Char(*v),Self::Int(v)=>CV::Int(*v),Self::Float(v,f)=>CV::Float(*v,*f),Self::Aggregate(v)=>CV::Aggregate(v.iter().map(Self::materialize).collect()),Self::Variant(n,v)=>CV::Variant(*n,v.iter().map(Self::materialize).collect()),Self::Repeat(v,n)=>CV::Repeat(Box::new(v.materialize()),*n)}}
    fn validate(&self,g:&types::Graph,t:u32,depth:usize,budget:&mut usize)->Result<(),String>{
        use types::Kind;step(depth,budget)?;
        let fields=|vs:&[Value],ids:&[u32],budget:&mut usize|->Result<(),String>{if vs.len()!=ids.len(){return Err("native constant aggregate arity/type mismatch".into());}for(v,&t)in vs.iter().zip(ids){v.validate(g,t,depth+1,budget)?;}Ok(())};
        match(self,g.kind(t)?){
            (Self::Unit,Kind::Scalar(Scalar::Unit))|(Self::Bool(_),Kind::Scalar(Scalar::Bool))|(Self::Char(_),Kind::Scalar(Scalar::Char))=>Ok(()),
            (Self::Int(v),Kind::Scalar(Scalar::Int(it)))=>{let b=it.bits();let fits=if b==128{true}else if it.signed(){let hi=1i128<<(b-1);*v>=-hi&&*v<hi}else{*v>=0&&(*v as u128)<1u128<<b};if fits{Ok(())}else{Err("native constant integer range/type mismatch".into())}},
            (Self::Float(bits,f),Kind::Float(want)) if f==want&&(*f==ty::FloatTy::F64||*bits<=u32::MAX as u64)=>Ok(()),
            (Self::Aggregate(v),Kind::Tuple(ids))=>fields(v,ids,budget),
            (Self::Aggregate(v),Kind::Array(t,n)) if v.len()as u64==*n=>{for v in v{v.validate(g,*t,depth+1,budget)?;}Ok(())},
            (Self::Repeat(v,n),Kind::Array(t,want)) if n==want=>{let before=*budget;v.validate(g,*t,depth+1,budget)?;let cost=before-*budget;let expanded=cost.checked_mul(usize::try_from(*n).map_err(|_|"native constant work bound")?).ok_or("native constant work bound")?;let extra=expanded.saturating_sub(cost);*budget=budget.checked_sub(extra).ok_or("native constant work bound")?;Ok(())},
            (Self::Variant(n,v),Kind::Nominal(r))=>{let variant=r.variants.get(*n as usize).ok_or("native constant variant index")?;if !r.is_enum&&*n!=0{return Err("native constant struct variant index".into());}fields(v,&variant.fields.iter().map(|f|f.ty).collect::<Vec<_>>(),budget)},
            // Original evaluator stores named/tuple structs as aggregates.
            (Self::Aggregate(v),Kind::Nominal(r)) if !r.is_enum=>fields(v,&r.variants[0].fields.iter().map(|f|f.ty).collect::<Vec<_>>(),budget),
            (Self::Variant(n,v),Kind::Instance(a))=>{if !a.is_enum&&*n!=0{return Err("native constant struct variant index".into());}fields(v,a.fields.get(*n as usize).ok_or("native constant variant index")?,budget)},
            (Self::Aggregate(v),Kind::Instance(a))if !a.is_enum=>fields(v,a.fields.first().ok_or("native constant application shape")?,budget),
            _=>Err("native constant value/type shape mismatch".into()),
        }
    }
    fn encode(&self,out:&mut Vec<u8>){let list=|out:&mut Vec<u8>,v:&[Value]|{out.extend_from_slice(&(v.len()as u32).to_le_bytes());for v in v{v.encode(out);}};match self{
        Self::Unit=>out.push(0),Self::Bool(v)=>out.extend_from_slice(&[1,*v as u8]),Self::Char(v)=>{out.push(2);out.extend_from_slice(&(*v as u32).to_le_bytes());},Self::Int(v)=>{out.push(3);out.extend_from_slice(&v.to_le_bytes());},Self::Float(v,f)=>{out.extend_from_slice(&[4,if *f==ty::FloatTy::F32{0}else{1}]);out.extend_from_slice(&v.to_le_bytes());},Self::Aggregate(v)=>{out.push(5);list(out,v);},Self::Variant(n,v)=>{out.push(6);out.extend_from_slice(&n.to_le_bytes());list(out,v);},Self::Repeat(v,n)=>{out.push(7);out.extend_from_slice(&n.to_le_bytes());v.encode(out);},
    }}
    fn decode(r:&mut Reader<'_>,depth:usize,budget:&mut usize)->Result<Self,String>{step(depth,budget)?;Ok(match r.byte()?{
        0=>Self::Unit,1=>Self::Bool(r.boolean()?),2=>Self::Char(char::from_u32(number(r)?).ok_or("native constant invalid char")?),3=>Self::Int(i128::from_le_bytes(r.bytes(16)?.try_into().unwrap())),
        4=>{let f=match r.byte()?{0=>ty::FloatTy::F32,1=>ty::FloatTy::F64,_=>return Err("native constant float tag".into())};Self::Float(u64::from_le_bytes(r.bytes(8)?.try_into().unwrap()),f)},
        tag@5..=6=>{let variant=if tag==6{Some(number(r)?)}else{None};let n=r.len(MAX_VALUES.min(*budget))?;let v=(0..n).map(|_|Self::decode(r,depth+1,budget)).collect::<Result<_,_>>()?;if let Some(n)=variant{Self::Variant(n,v)}else{Self::Aggregate(v)}},
        7=>{let n=u64::from_le_bytes(r.bytes(8)?.try_into().unwrap());Self::Repeat(Box::new(Self::decode(r,depth+1,budget)?),n)},
        _=>return Err("native constant value tag".into()),
    })}
}
fn number(r:&mut Reader<'_>)->Result<u32,String>{Ok(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()))}
pub(super) fn capture(p:&check::collect::Program,g:&mut types::Graph)->Result<Vec<Record>,String>{
    if p.consts.len()!=p.const_defs.len()||p.consts.len()>MAX_DEFS{return Err("native constant definition bound".into());}let mut budget=MAX_VALUES;let mut records=Vec::new();
    for(c,d)in p.consts.iter().zip(&p.const_defs){if d.krate!=p.krate{return Err("native constant dependency records required".into());}
        let info=p.resolved.as_ref().and_then(|t|t.def_info.get(d)).ok_or("native constant resolver identity absent")?;
        let mut graph=g.clone();let mut remaining=budget;
        let represented=(||Ok::<_,String>(Record{id:def::DiskDefId{krate:p.stable_crate,hash:d.hash},name:p.name(c.name).into(),ty:graph.capture(p,c.ty)?,value:Value::capture(&c.value,0,&mut remaining)?,qualif:c.qualif}))();
        match represented{Ok(r)=>{*g=graph;budget=remaining;records.push(r);},Err(e)if info.vis!=crate::resolve::imports::VisKind::Pub=>{
            // Preserve inaccessible private constant bindings accepted by
            // earlier images. No imported value/type is granted for them.
            let _=e;
        },Err(e)=>return Err(e)}
    }Ok(records)
}
pub(super) fn validate(records:&[Record],stable:def::StableCrateHash,g:&types::Graph)->Result<(),String>{
    if records.len()>MAX_DEFS{return Err("native constant definition bound".into());}let mut ids=std::collections::BTreeSet::new();let mut budget=MAX_VALUES;
    for r in records{if r.name.is_empty()||r.name.len()>16384||r.id.krate!=stable||r.id.hash!=def::item_hash(stable,&[],def::DefKind::Const,&r.name,0)||!ids.insert(r.id){return Err("native constant definition identity".into());}if r.qualif!=Qualif::NONE{return Err("native constant promotion/drop qualifiers unsupported".into());}r.value.validate(g,r.ty,0,&mut budget)?;}Ok(())
}
pub(super) fn encode(records:&[Record])->Vec<u8>{let mut out=Vec::new();out.extend_from_slice(&(records.len()as u32).to_le_bytes());for r in records{out.extend_from_slice(&r.id.to_bytes());text(&mut out,&r.name);out.extend_from_slice(&r.ty.to_le_bytes());out.extend_from_slice(&[r.qualif.drop as u8,r.qualif.cell as u8]);r.value.encode(&mut out);}out}
pub(super) fn decode(bytes:&[u8])->Result<Vec<Record>,String>{let mut r=Reader(bytes);let n=r.len(MAX_DEFS)?;let mut records=Vec::new();let mut budget=MAX_VALUES;for _ in 0..n{records.push(Record{id:def::DiskDefId::from_bytes(r.bytes(16)?).unwrap(),name:r.text(16384)?,ty:number(&mut r)?,qualif:Qualif{drop:r.boolean()?,cell:r.boolean()?},value:Value::decode(&mut r,0,&mut budget)?});}if !r.0.is_empty(){return Err("native constant trailing data".into());}Ok(records)}
pub(super) fn publish(records:&[Record],p:&mut check::collect::Program,krate:def::CrateId,types:&[ty::Ty])->Result<(),String>{
    for r in records{let id=def::DefId{krate,hash:r.id.hash};let tables=p.resolved.as_ref().ok_or("native constants require resolved imports")?;let info=tables.def_info.get(&id).ok_or("native constant missing resolver identity")?;
        if p.defs.get(id).is_some()||p.const_names.contains_key(&info.flat)||p.const_by_def(id).is_some(){return Err("native constant existing definition conflict".into());}
        let mut data=ty::defs::DefData::skeleton(ty::defs::DefKind::Const,None,info.flat,match info.vis{crate::resolve::imports::VisKind::Pub=>ty::defs::Vis::Pub,crate::resolve::imports::VisKind::Priv=>ty::defs::Vis::Priv,_=>ty::defs::Vis::Restricted},diag::Span::default());data.module=info.module;
        let ty=*types.get(r.ty as usize).ok_or("native constant type index")?;let index=p.consts.len()as u32;p.defs.insert(id,data);p.const_defs.push(id);p.const_names.insert(info.flat,index);p.consts.push(check::collect::ConstItem{name:info.flat,ty,value:r.value.materialize(),qualif:r.qualif,value_fp:Default::default()});
    }Ok(())
}
