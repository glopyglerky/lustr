#![allow(uncommon_codepoints)]
include!("identifier_nfc_included.rs.inc");

fn main() {
    println!("{} | {} | {}", from_included!(caféé), from_included!(r#caféé), résultat());
}
