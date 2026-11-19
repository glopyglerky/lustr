// Unicode properties changed in the declared Rust SDK. The native evaluator
// and generated program must agree, even when Lust was built by another Rust.
const LOWER: bool = '\u{558}'.is_lowercase();
const UPPER: bool = '\u{a7dd}'.is_uppercase();
const VERSION: (u8, u8, u8) = char::UNICODE_VERSION;

fn main() {
    let mut args = std::env::args().skip(1);
    let lower = args.next().unwrap_or(String::from("\u{558}")).chars().next().unwrap();
    let upper = args.next().unwrap_or(String::from("\u{a7dd}")).chars().next().unwrap();
    assert_eq!(LOWER, lower.is_lowercase());
    assert_eq!(UPPER, upper.is_uppercase());
    println!("{:?} {} {} {} {}", VERSION, LOWER, UPPER, lower.is_lowercase(), upper.is_uppercase());
}
