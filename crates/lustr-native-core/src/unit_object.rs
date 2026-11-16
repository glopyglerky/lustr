//! Bounded ELF object validation for the first native freestanding unit seam.
//! This validates actual emitted symbols, never inferred library names.
use std::collections::{BTreeSet,BTreeMap};

fn part(b: &[u8], at: usize, n: usize) -> Result<&[u8], String> {
    b.get(at..at.checked_add(n).ok_or("ELF range overflow")?).ok_or("truncated ELF".into())
}
fn word(b: &[u8], at: usize, n: usize) -> Result<u64, String> {
    let mut out = [0; 8]; out[..n].copy_from_slice(part(b, at, n)?); Ok(u64::from_le_bytes(out))
}
fn number(b: &[u8], at: usize, n: usize) -> Result<usize, String> {
    word(b, at, n)?.try_into().map_err(|_| "ELF offset exceeds host address space".into())
}

pub(crate) struct Symbols {
    pub defined: BTreeSet<Vec<u8>>,
    pub functions: BTreeSet<Vec<u8>>,
    pub undefined: BTreeSet<Vec<u8>>,
    pub kinds:BTreeMap<Vec<u8>,SymbolKind>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum SymbolKind { Unknown, Function, Data, ThreadLocal }

pub(crate) fn symbols(bytes: &[u8]) -> Result<Symbols, String> {
    if bytes.len() > 64 * 1024 * 1024 || part(bytes, 0, 7)? != b"\x7fELF\x02\x01\x01"
        || word(bytes, 16, 2)? != 1 || word(bytes, 18, 2)? != 62
        || word(bytes, 20, 4)? != 1 || word(bytes, 52, 2)? != 64
    { return Err("expected a bounded x86_64 little-endian ELF relocatable object".into()); }
    let base = number(bytes, 40, 8)?;
    let stride = number(bytes, 58, 2)?;
    let count = number(bytes, 60, 2)?;
    if stride != 64 || count == 0 { return Err("unsupported ELF section table".into()); }
    part(bytes, base, count.checked_mul(stride).ok_or("ELF section count overflow")?)?;
    let mut out = Symbols { defined: BTreeSet::new(), functions: BTreeSet::new(), undefined: BTreeSet::new(),kinds:BTreeMap::new() };
    let mut tables = 0;
    for n in 0..count {
        let sh = base + n * stride;
        if word(bytes, sh + 4, 4)? != 2 { continue; }
        tables += 1;
        let at = number(bytes, sh + 24, 8)?;
        let size = number(bytes, sh + 32, 8)?;
        let link = number(bytes, sh + 40, 4)?;
        if link >= count || number(bytes, sh + 56, 8)? != 24 || size % 24 != 0 {
            return Err("invalid ELF symbol table".into());
        }
        part(bytes, at, size)?;
        let strings = base + link * stride;
        if word(bytes, strings + 4, 4)? != 3 { return Err("ELF symbols lack a string table".into()); }
        let names = part(bytes, number(bytes, strings + 24, 8)?, number(bytes, strings + 32, 8)?)?;
        for sym in (at..at + size).step_by(24).skip(1) {
            let name = number(bytes, sym, 4)?;
            let info = part(bytes, sym + 4, 1)?[0];
            let binding = info >> 4;
            if binding == 0 { continue; }
            if !matches!(binding, 1 | 2) { return Err("unqualified ELF symbol binding".into()); }
            let tail = names.get(name..).ok_or("invalid ELF symbol name offset")?;
            let end = tail.iter().position(|b| *b == 0).ok_or("unterminated ELF symbol name")?;
            if end == 0 { return Err("unnamed nonlocal ELF symbol".into()); }
            let kind=match info&15{0=>SymbolKind::Unknown,1=>SymbolKind::Data,2=>SymbolKind::Function,6=>SymbolKind::ThreadLocal,_=>return Err("unqualified ELF symbol kind".into())};
            let key=tail[..end].to_vec();
            if let Some(old)=out.kinds.get_mut(&key){if *old==SymbolKind::Unknown{*old=kind;}else if kind!=SymbolKind::Unknown&&*old!=kind{return Err("contradictory ELF symbol kinds".into());}}
            else{out.kinds.insert(key,kind);}
            let section=number(bytes,sym+6,2)?;
            if kind==SymbolKind::ThreadLocal&&section!=0&&(section>=count||word(bytes,base+section*stride+8,8)?&0x400==0){return Err("ELF TLS definition is outside a TLS section".into());}
            if section == 0 { out.undefined.insert(tail[..end].to_vec()); }
            else {
                out.defined.insert(tail[..end].to_vec());
                if info & 15 == 2 {
                    if section>=count || word(bytes,base+section*stride+8,8)? & 4 == 0 {
                        return Err("ELF function is outside an executable section".into());
                    }
                    let value=number(bytes,sym+8,8)?;let len=number(bytes,sym+16,8)?;
                    let section_len=number(bytes,base+section*stride+32,8)?;
                    if len==0 || value.checked_add(len).is_none_or(|end|end>section_len){return Err("ELF function is outside its section".into());}
                    out.functions.insert(tail[..end].to_vec());
                }
            }
        }
    }
    if tables != 1 { return Err("expected exactly one ELF symbol table".into()); }
    Ok(out)
}

/// Evidence for each actual undefined use in the fixed x86_64/O0 C ABI.
/// STT_NOTYPE alone proves nothing. Admit direct call/jump relocations and
/// original 64-bit location loads; unrepresented address-taking/use refuses.
pub(crate) fn dependency_uses(bytes:&[u8])->Result<BTreeMap<Vec<u8>,SymbolKind>,String>{
    let symbols=symbols(bytes)?;let base=number(bytes,40,8)?;let count=number(bytes,60,2)?;
    let table=(0..count).find(|&n|word(bytes,base+n*64+4,4)==Ok(2)).ok_or("ELF missing symbol table")?;
    let sh=base+table*64;let at=number(bytes,sh+24,8)?;let size=number(bytes,sh+32,8)?;let link=number(bytes,sh+40,4)?;
    let strsh=base+link*64;let names=part(bytes,number(bytes,strsh+24,8)?,number(bytes,strsh+32,8)?)?;
    let mut uses=BTreeMap::new();
    for n in 0..count{
        let sh=base+n*64;if word(bytes,sh+4,4)?!=4{continue;}
        if number(bytes,sh+40,4)?!=table||number(bytes,sh+56,8)?!=24{return Err("unqualified ELF relocation table".into());}
        let target=number(bytes,sh+44,4)?;if target>=count{return Err("ELF relocation target".into());}
        let target=base+target*64;let executable=word(bytes,target+8,8)?&4!=0;
        let code=part(bytes,number(bytes,target+24,8)?,number(bytes,target+32,8)?)?;
        let start=number(bytes,sh+24,8)?;let len=number(bytes,sh+32,8)?;if len%24!=0{return Err("ELF relocation size".into());}part(bytes,start,len)?;
        for r in(start..start+len).step_by(24){
            let info=word(bytes,r+8,8)?;let index=(info>>32)as usize;if index>=size/24{return Err("ELF relocation symbol index".into());}let sym=at+24*index;
            if number(bytes,sym+6,2)?!=0||part(bytes,sym+4,1)?[0]>>4==0{continue;}
            let name=number(bytes,sym,4)?;let tail=names.get(name..).ok_or("ELF relocation symbol name")?;let end=tail.iter().position(|&b|b==0).ok_or("ELF relocation unterminated name")?;let name=tail[..end].to_vec();
            if name.is_empty(){return Err("unnamed nonlocal ELF dependency".into());}
            let offset=number(bytes,r,8)?;let addend=word(bytes,r+16,8)?as i64;part(code,offset,4)?;
            if !executable||addend!=-4{return Err("native dependency use lacks qualified instruction evidence".into());}
            let kind=match info as u32{
                4 if offset>0&&matches!(code[offset-1],0xe8|0xe9)=>SymbolKind::Function,
                // RIP-relative qword load, or the original GOT address load
                // immediately followed by a qword read through that address.
                2 if offset>=3&&code[offset-3]==0x48&&code[offset-2]==0x8b&&code[offset-1]&0xc7==5=>SymbolKind::Data,
                9|42 if offset>=3&&part(code,offset-3,3)?==[0x48,0x8b,0x05]&&part(code,offset+4,2)?==[0x48,0x8b]&&part(code,offset+6,1)?[0]&0xc7==0=>SymbolKind::Data,
                _=>return Err("native dependency use has unrepresented relocation/instruction evidence".into()),
            };
            let declared=*symbols.kinds.get(&name).ok_or("native dependency has no symbol kind record")?;if declared!=SymbolKind::Unknown&&declared!=kind{return Err("native dependency declaration/use kind mismatch".into());}
            if uses.insert(name,kind).is_some_and(|old|old!=kind){return Err("native dependency has contradictory function/data uses".into());}
        }
    }
    if uses.keys().cloned().collect::<BTreeSet<_>>()!=symbols.undefined{return Err("native dependency lacks actual use evidence".into());}Ok(uses)
}
