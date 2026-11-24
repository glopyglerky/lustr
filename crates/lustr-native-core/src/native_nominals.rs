//! Native ADT definitions with separate kept item-region fields.
//! Every image's layout, Copy, drop and ABI validate before `publish`.
use super::{Reader,namespace,MAX_PARAMS};
use crate::{ast,check,def,diag,resolve,ty};
use ty::TyIntern;
use check::collect::VariantShape;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Field {pub name:String,pub ty:u32,pub vis:namespace::Visibility}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Variant {pub name:String,pub shape:VariantShape,pub disc:i128,pub fields:Vec<Field>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Record {pub id:def::DiskDefId,pub name:String,pub is_enum:bool,pub variants:Vec<Variant>,pub regions:Option<ItemRegions>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct ItemRegions {pub names:Vec<String>,pub fields:Vec<Vec<u32>>,pub facts:Vec<(u32,u32)>}

pub(super) fn eligible(p:&check::collect::Program,tys:&impl TyIntern,t:ty::Ty)->bool{
    let ty::TyKind::Adt(d,args)=tys.kind(t)else{return false;};
    let Some(a)=p.adt_by_def(d)else{return false;};let adt=p.adt(a);
    let lifetimes=p.gx.adt_lifetimes.get(&d).map_or(0,Vec::len);let args=tys.args(args);
    if lifetimes>64||!args.is_empty()&&(args.len()!=lifetimes||args.iter().any(|g|!matches!(g,ty::GenericArg::Lifetime(ty::Region::Erased|ty::Region::Static|ty::Region::Param(0..64))))){return false;}
    let Some(g)=p.gx.adt_generics.get(a.0 as usize)else{return false;};let derives=adt.derives_unlogged();
    !adt.non_exhaustive&&adt.repr==ty::layout::Repr::default()
        &&adt.variants.len()<=MAX_PARAMS&&(adt.is_enum||adt.variants.len()==1)
        &&adt.variants.iter().all(|v|!v.non_exhaustive&&v.fields.len()<=MAX_PARAMS)
        &&!derives.copy&&!derives.clone&&!derives.debug&&!derives.partial_eq&&!derives.eq
        &&!derives.hash&&!derives.partial_ord&&!derives.ord&&!derives.default
        &&g.params.is_empty()&&g.predicates.is_empty()
        &&p.regions.adt_facts.get(&d).is_none_or(|v|v.len()<=4096)
        &&p.gx.adt_scopes.get(a.0 as usize).is_some_and(|s|s.region_facts.len()<=4096)
        &&p.gx.adt_instance_of.get(a.0 as usize).is_some_and(|x|x.is_none())
        &&!p.drop_adts.contains(&a)&&!check::collect::needs_drop(p,tys,t)
        &&!check::collect::is_copy(p,tys,t)
}
impl Record{
    pub(super) fn fields(&self)->impl Iterator<Item=&Field>{self.variants.iter().flat_map(|v|&v.fields)}
    pub(super) fn has_constructor(&self)->bool{!self.is_enum&&self.variants.len()==1&&self.variants[0].shape!=VariantShape::Named}
    pub(super) fn legacy_named(&self)->bool{!self.is_enum&&self.variants.len()==1&&self.variants[0].shape==VariantShape::Named}
    pub(super) fn wire_tag(&self)->u8{if self.regions.is_some(){24}else if self.legacy_named(){22}else{23}}
    pub(super) fn capture(p:&check::collect::Program,t:ty::Ty,mut child:impl FnMut(ty::Ty)->Result<u32,String>)->Result<Self,String>{
        Self::capture_with(p,t,&mut child,false)
    }
    pub(super) fn capture_template(p:&check::collect::Program,t:ty::Ty,mut child:impl FnMut(ty::Ty)->Result<u32,String>)->Result<Self,String>{
        Self::capture_with(p,t,&mut child,true)
    }
    fn capture_with(p:&check::collect::Program,t:ty::Ty,mut child:impl FnMut(ty::Ty)->Result<u32,String>,template:bool)->Result<Self,String>{
        if !template&&!eligible(p,&p.tys,t){return Err("native nominal needs generic/derive/trait/drop/shape records".into());}
        let(d,_)=p.tys.adt_parts(t).unwrap();if d.krate!=p.krate{return Err("native nominal producer dependency records required".into());}
        let a=p.adt(p.adt_by_def(d).unwrap());let tables=p.resolved.as_ref().ok_or("native nominal needs original resolver fields")?;
        let written=p.ast.items.iter().find_map(|item|match &item.kind{ast::ItemKind::Struct(s) if s.name==a.name=>Some((&s.generics,&s.where_clause)),ast::ItemKind::Enum(e) if e.name==a.name=>Some((&e.generics,&e.where_clause)),_=>None}).ok_or("native nominal requires its original item header")?;
        let names=p.gx.adt_lifetimes.get(&d).map_or(&[][..],Vec::as_slice);
        if !template&&(written.1.preds.iter().any(|p|!matches!(p,ast::WherePred::Lifetime{..}))||written.0.params.len()!=names.len()||written.0.params.iter().zip(names).any(|(g,name)|!matches!(g,ast::GenericParam::Lifetime{lt,attrs,..}if lt.name==*name&&attrs.is_empty()))){return Err("native nominal needs written generic/region/obligation records".into());}
        // Actual checked declaration facts, independent of erased layout.
        // Redundant written bounds retain the same canonical relation.
        let mut facts=p.gx.adt_scopes[p.adt_by_def(d).unwrap().0 as usize].region_facts.clone();facts.sort_unstable();facts.dedup();
        let mut recorded=p.regions.adt_facts.get(&d).cloned().unwrap_or_default();recorded.sort_unstable();recorded.dedup();
        if facts!=recorded{return Err("native nominal original declaration facts disagree".into());}
        let info=tables.def_info.get(&d).ok_or("native nominal definition is missing")?;
        if a.is_enum&&tables.variants.get(&d).is_none_or(|vs|vs.iter().copied().ne(a.variants.iter().map(|v|v.name))){return Err("native enum resolver variant identity mismatch".into());}
        let mut variants=Vec::new();
        for v in &a.variants{
            let mut fields=Vec::new();for(n,f)in v.fields.iter().enumerate(){
                let vis=if a.is_enum{&info.vis}else{match v.shape{
                    VariantShape::Named=>{let(module,vs)=tables.field_vis.get(&d).ok_or("native nominal named field visibility is missing")?;if *module!=info.module||vs.len()!=v.fields.len(){return Err("native nominal field visibility count/module mismatch".into());}vs.get(&f.name).ok_or("native nominal field visibility missing")?},
                    VariantShape::Tuple=>{let(module,vs)=tables.tuple_field_vis.get(&d).ok_or("native nominal tuple field visibility is missing")?;if *module!=info.module||vs.len()!=v.fields.len(){return Err("native nominal tuple visibility count/module mismatch".into());}&vs[n]},
                    VariantShape::Unit=>return Err("native unit nominal has fields".into()),
                }};
                fields.push(Field{name:p.name(f.name).into(),ty:child(f.ty)?,vis:namespace::Visibility::capture(vis,&p.interner)});
            }
            variants.push(Variant{name:p.name(v.name).into(),shape:v.shape,disc:v.disc,fields});
        }
        let kept=p.regions.adt_fields.get(&d).ok_or("native nominal lacks original region fields")?;
        if kept.len()!=variants.len()||kept.iter().zip(&variants).any(|(k,v)|k.len()!=v.fields.len()){return Err("native nominal original region field shape mismatch".into());}
        let mut used=Vec::new();for &t in kept.iter().flatten(){crate::borrowck::regions_of(&p.tys,t,&mut used);}
        let regions=if names.is_empty()&&used.is_empty()&&facts.is_empty(){None}else{Some(ItemRegions{names:names.iter().map(|&s|p.name(s).to_owned()).collect(),fields:kept.iter().map(|v|v.iter().map(|&t|child(t)).collect::<Result<_,_>>()).collect::<Result<_,_>>()?,facts})};
        let record=Self{id:def::DiskDefId{krate:p.stable_crate,hash:d.hash},name:p.name(a.name).into(),is_enum:a.is_enum,variants,regions};record.validate()?;Ok(record)
    }
    pub(super) fn validate(&self)->Result<(),String>{
        if self.name.is_empty()||self.name.len()>16384||self.id.hash!=def::item_hash(self.id.krate,&[],def::DefKind::Adt,&self.name,0)||self.variants.len()>MAX_PARAMS||!self.is_enum&&self.variants.len()!=1{return Err("native nominal definition identity/bound".into());}
        if let Some(r)=&self.regions{let mut names=std::collections::BTreeSet::new();if r.names.len()>64||r.names.iter().any(|s|!super::ident(s)||s=="static"||s=="_"||!names.insert(s))||r.fields.len()!=self.variants.len()||r.fields.iter().zip(&self.variants).any(|(a,b)|a.len()!=b.fields.len()){return Err("native nominal item region declaration/field shape".into());}}
        if let Some(r)=&self.regions{if r.facts.len()>4096||r.facts.windows(2).any(|w|w[0]>=w[1])||r.facts.iter().any(|&(a,b)|a as usize>=r.names.len()||b!=check::generics::STATIC_FACT&&b as usize>=r.names.len()){return Err("native nominal outlives fact index/order/bound".into());}}
        let mut names=std::collections::BTreeSet::new();let mut discs=std::collections::BTreeSet::new();
        for v in &self.variants{
            if v.fields.len()>MAX_PARAMS||v.shape==VariantShape::Unit&&!v.fields.is_empty(){return Err("native nominal variant shape/bound".into());}
            if self.is_enum{if !super::ident(&v.name)||!names.insert(&v.name)||!discs.insert(v.disc){return Err("native enum variant identity/discriminant".into());}}
            else if v.name!=self.name||v.disc!=0{return Err("native struct variant identity/discriminant".into());}
            let mut fields=std::collections::BTreeSet::new();for(n,f)in v.fields.iter().enumerate(){if !fields.insert(&f.name)||match v.shape{VariantShape::Named=>!super::ident(&f.name),VariantShape::Tuple=>f.name!=n.to_string(),VariantShape::Unit=>true}{return Err("native nominal field identity".into());}f.vis.validate()?;}
        }Ok(())
    }
    fn fields_out(fields:&[Field],out:&mut Vec<u8>){out.extend_from_slice(&(fields.len()as u32).to_le_bytes());for f in fields{super::text(out,&f.name);out.extend_from_slice(&f.ty.to_le_bytes());f.vis.encode(out);}}
    fn fields_in(r:&mut Reader<'_>)->Result<Vec<Field>,String>{let n=r.len(MAX_PARAMS)?;(0..n).map(|_|Ok(Field{name:r.text(128)?,ty:u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()),vis:namespace::Visibility::decode(r)?})).collect()}
    // Existing closed records remain compact. Tag24 adds the actual kept
    // fields, lifetime declarations and declared outlives facts in format11;
    // tag25 in the graph applies them.
    pub(super) fn encode(&self,out:&mut Vec<u8>){
        out.extend_from_slice(&self.id.to_bytes());super::text(out,&self.name);
        if self.wire_tag()==22{Self::fields_out(&self.variants[0].fields,out);}else{
            out.push(self.is_enum as u8);out.extend_from_slice(&(self.variants.len()as u32).to_le_bytes());
            for v in &self.variants{super::text(out,&v.name);out.push(match v.shape{VariantShape::Unit=>0,VariantShape::Tuple=>1,VariantShape::Named=>2});out.extend_from_slice(&v.disc.to_le_bytes());Self::fields_out(&v.fields,out);}
        }
        if let Some(r)=&self.regions{out.extend_from_slice(&(r.names.len()as u32).to_le_bytes());for name in &r.names{super::text(out,name);}for fields in &r.fields{for &t in fields{out.extend_from_slice(&t.to_le_bytes());}}out.extend_from_slice(&(r.facts.len()as u32).to_le_bytes());for &(a,b)in &r.facts{out.extend_from_slice(&a.to_le_bytes());out.extend_from_slice(&b.to_le_bytes());}}
    }
    pub(super) fn decode(r:&mut Reader<'_>,tag:u8)->Result<Self,String>{
        let id=def::DiskDefId::from_bytes(r.bytes(16)?).unwrap();let name=r.text(16384)?;
        let(is_enum,variants)=if tag!=22{let is_enum=r.boolean()?;let mut variants=Vec::new();for _ in 0..r.len(MAX_PARAMS)?{let name=r.text(16384)?;let shape=match r.byte()?{0=>VariantShape::Unit,1=>VariantShape::Tuple,2=>VariantShape::Named,_=>return Err("native nominal variant shape tag".into())};let disc=i128::from_le_bytes(r.bytes(16)?.try_into().unwrap());let fields=Self::fields_in(r)?;variants.push(Variant{name,shape,disc,fields});}(is_enum,variants)}else{(false,vec![Variant{name:name.clone(),shape:VariantShape::Named,disc:0,fields:Self::fields_in(r)?}])};
        let regions=if tag==24{let names=(0..r.len(64)?).map(|_|r.text(128)).collect::<Result<_,_>>()?;let fields=variants.iter().map(|v|(0..v.fields.len()).map(|_|Ok(u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()))).collect::<Result<_,String>>()).collect::<Result<_,_>>()?;let facts=(0..r.len(4096)?).map(|_|Ok((u32::from_le_bytes(r.bytes(4)?.try_into().unwrap()),u32::from_le_bytes(r.bytes(4)?.try_into().unwrap())))).collect::<Result<_,String>>()?;Some(ItemRegions{names,fields,facts})}else{None};
        let record=Self{id,name,is_enum,variants,regions};record.validate()?;if record.wire_tag()!=tag{return Err("noncanonical native nominal shape tag".into());}Ok(record)
    }
    pub(super) fn shape(&self,types:&[ty::Ty])->ty::layout::AdtShape{ty::layout::AdtShape{repr:Default::default(),is_enum:self.is_enum,variants:self.variants.iter().map(|v|v.fields.iter().map(|f|types[f.ty as usize]).collect()).collect(),discs:self.variants.iter().map(|v|v.disc).collect(),hide_niche:false}}
    pub(super) fn stage(&self,p:&mut check::collect::Program,def:def::DefId,types:&[ty::Ty])->Result<(),String>{
        let info=p.resolved.as_ref().and_then(|t|t.def_info.get(&def)).ok_or("native nominal not in current resolver")?;
        if info.kind!=resolve::imports::ResKind::Adt||p.defs.get(def).is_some()||p.defs.local_adt(def).is_some(){return Err("native nominal staging conflicts with existing definitions".into());}
        let id=ty::AdtId(p.adts.len()as u32);if p.tys.local_adt_of(def)!=Some(id){return Err("native nominal fresh shape index mismatch".into());}
        let variants=self.variants.iter().map(|v|check::collect::Variant{name:if self.is_enum{p.interner.intern(&v.name)}else{info.flat},shape:v.shape,fields:v.fields.iter().map(|f|check::collect::Field{name:p.interner.intern(&f.name),ty:types[f.ty as usize],span:diag::Span::default(),derive_bounds:Vec::new()}).collect(),disc:v.disc,non_exhaustive:false}).collect();
        p.adts.push(check::collect::AdtDef::new(info.flat,self.is_enum,variants,Default::default(),diag::Span::default(),Default::default(),false));
        let mut scope=check::generics::Scope::default();
        if let Some(r)=&self.regions{let names:Vec<_>=r.names.iter().map(|s|p.interner.intern(s)).collect();scope.lifetimes=names.iter().enumerate().map(|(n,&s)|(s,n as u32)).collect();scope.region_facts=r.facts.clone();if !names.is_empty(){p.gx.adt_lifetimes.insert(def,names);}if !r.facts.is_empty(){p.regions.adt_facts.insert(def,r.facts.clone());}p.regions.adt_fields.insert(def,r.fields.iter().map(|v|v.iter().map(|&t|types[t as usize]).collect()).collect());}
        if self.regions.is_none(){p.regions.adt_fields.insert(def,self.variants.iter().map(|v|v.fields.iter().map(|f|types[f.ty as usize]).collect()).collect());}
        p.gx.adt_info.push((def,0));p.gx.adt_generics.push(Default::default());p.gx.adt_envs.push(Vec::new());p.gx.adt_scopes.push(scope);p.gx.adt_instance_of.push(None);p.defs.register_adt_shape(def,id);Ok(())
    }
    pub(super) fn publish(&self,p:&mut check::collect::Program,def:def::DefId,types:&[ty::Ty])->Result<(),String>{
        let info=p.resolved.as_ref().unwrap().def_info.get(&def).ok_or("native nominal resolver disappeared")?;
        let mut data=ty::defs::DefData::skeleton(if self.is_enum{ty::defs::DefKind::Enum}else{ty::defs::DefKind::Struct},None,info.flat,namespace::Visibility::capture(&info.vis,&p.interner).def_vis(),diag::Span::default());data.module=info.module;
        let a=p.defs.local_adt(def).ok_or("native nominal shape was not staged")?;if p.type_names.insert(info.flat,a).is_some(){return Err("native nominal name conflicts with current type".into());}
        p.defs.insert(def,data);Ok(())
    }
}
