#![allow(uncommon_codepoints)]
macro_rules! canonical { (caféé) => { 1u32 }; (r#caféé) => { 2u32 }; }
macro_rules! raw_first { (r#caféé) => { 3u32 }; (caféé) => { 4u32 }; }
macro_rules! ident { ($name:ident) => { stringify!($name) }; }
macro_rules! token { ($name:tt) => { stringify!($name) }; }
macro_rules! lifetime { ($name:lifetime) => { stringify!($name) }; }
macro_rules! define { ($name:ident) => { fn $name() -> u32 { 31 } }; }
define!(résumé);
fn main() {
 println!("{} {} {} {}", canonical!(caféé), canonical!(r#caféé), raw_first!(caféé), raw_first!(r#caféé));
 println!("{} | {} | {} | {}", stringify!(café), stringify!(r#café), ident!(r#café), token!(r#café));
 println!("{} | {} | {} | {}", stringify!('é), stringify!('r#é), lifetime!('é), lifetime!('r#é));
 println!("{} | {} | {} | {}", stringify!("é"), concat!("é"), r"é", résumé());
}
