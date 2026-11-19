#![allow(uncommon_codepoints)]
struct S { café: u32 }
impl S { fn résumé(&self) -> bool { false } }
macro_rules! has { ($token:tt) => { false }; }

fn local() { let r#café = 1u32; assert!(r#café == 2); }
fn keyword() { let r#type = 1u32; assert!(r#type == 2); }
fn field() { let value = S { café: 1 }; assert!(value.r#café == 2); }
fn method() { let value = S { café: 1 }; assert!(value.r#résumé()); }
fn constructor() { assert!((S { r#café: 1 }).r#café == 2); }
fn token() { assert!(has!(r#café)); }
fn literal() { assert!(has!("é")); }
fn vector_token() { let r#café = 1u32; assert!(vec![r#café] == vec![2]); }
fn vector_literal() { assert!(vec!["é"] == vec!["é"]); }
fn label() { assert!('r#é: loop { break 'é false; }); }
fn keyword_label() { assert!('r#fn: loop { break 'r#fn false; }); }

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for f in [local as fn(), keyword, field, method, constructor, token, literal, vector_token, vector_literal, label, keyword_label] {
        let payload = std::panic::catch_unwind(f).unwrap_err();
        if let Some(s) = payload.downcast_ref::<&str>() { println!("{}", s); }
        if let Some(s) = payload.downcast_ref::<String>() { println!("{}", s); }
    }
}
