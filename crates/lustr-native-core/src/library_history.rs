//! Owner-lifetime resource policy for inactive library evidence. Live source,
//! in-flight snapshots and the last durable artifact are never budget victims.
use std::sync::OnceLock;
#[derive(Clone,Copy,Debug)]
pub(crate) struct Policy { pub inactive_entries:usize, pub inactive_bytes:usize }
impl Default for Policy {
    fn default()->Self {Self{inactive_entries:128,inactive_bytes:16*1024*1024}}
}
static POLICY:OnceLock<Result<Policy,String>>=OnceLock::new();
pub(crate) fn policy()->Result<Policy,String>{
    POLICY.get_or_init(||{
        let mut policy=Policy::default();
        if let Some(value)=std::env::var_os("LUSTR_LIBRARY_HISTORY_ENTRIES") {
            let value=value.to_str().ok_or("library history budget is not Unicode")?;
            let entries=value.parse::<usize>().map_err(|_|"library history budget must be an integer from 0 through 128")?;
            if entries>policy.inactive_entries{return Err("library history budget may only lower the 128-entry default".into());}
            policy.inactive_entries=entries;
        }
        Ok(policy)
    }).clone()
}
/// Both limits use this same LRU cut. Byte accounting covers retained payloads;
/// allocator/map overhead and the active source are observed in owner RSS.
pub(crate) fn victims<K:Ord>(mut inactive:Vec<(K,u64,usize)>,policy:Policy)->Vec<K>{
    inactive.sort_by(|a,b|a.1.cmp(&b.1).then_with(||a.0.cmp(&b.0)));
    let mut count=inactive.len();
    let mut bytes=inactive.iter().fold(0usize,|n,(_,_,size)|n.saturating_add(*size));
    let mut result=Vec::new();
    for (key,_,size) in inactive {
        if count<=policy.inactive_entries && bytes<=policy.inactive_bytes{break;}
        count-=1;bytes=bytes.saturating_sub(size);result.push(key);
    }
    result
}
