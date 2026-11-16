use std::ffi::OsString;
use std::cell::Cell; use std::rc::Rc; use std::panic::{catch_unwind, AssertUnwindSafe};
macro_rules! amount { () => { 41u64 }; }
struct Salt { value:u64 }
struct Guard(Rc<Cell<usize>>); impl Drop for Guard { fn drop(&mut self) { self.0.set(self.0.get()+1); } }
fn arguments(n:usize)->Vec<OsString> { let source=OsString::from("roundtrip");
 let inferred=source.into(); let path:std::path::PathBuf=inferred;
 let inferred_back=path.into(); let restored:OsString=inferred_back;
 assert_eq!(restored,OsString::from("roundtrip"));
 let mut xs=Vec::new(); xs.extend([std::path::PathBuf::from("flag").into(),std::path::PathBuf::from(format!("value={n}")).into()]); xs }
fn step(value:u64,salt:u64)->u64 { value.wrapping_mul(17).wrapping_add(salt as u64).wrapping_add(amount!()) }
fn main() { std::panic::set_hook(Box::new(|_|{})); let drops=Rc::new(Cell::new(0));
 let salt=Salt{value:7}; let mut total=salt.value as u64;
 for i in 0..100 { total=step(total,i); }
 let args=arguments(3); assert_eq!(args.len(),2);
 assert!(catch_unwind(AssertUnwindSafe(||{let _g=Guard(drops.clone()); panic!("witness");})).is_err(),"must unwind");
 assert_eq!(drops.get(),1);  println!("{args:?}:{total}:{}",drops.get());
}
