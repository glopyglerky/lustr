#![allow(non_snake_case, non_upper_case_globals, uncommon_codepoints)]
const BASE_café: u32 = 11;
const fn résumé(é: u32) -> u32 { é + BASE_café }
const VALUE: u32 = résumé(7);
mod résumé {
    pub const BASE_café: u32 = 19;
    pub fn r#é(r#café: u32) -> u32 { r#café + BASE_café }
}
fn borrow<'é>(value: &'é u32) -> &'é u32 { value }
fn raw_borrow<'r#é>(value: &'r#é u32) -> &'r#é u32 { value }
fn main() {
    let café = 23u32;
    let à̕ = 5u32;
    let 가 = 3u32;
    let Å = 2u32;
    let label = 'é: loop { break 'é 29u32; };
    let value = résumé::r#é(café) + VALUE + à̕ + 가 + Å + label;
    println!("{} {} {} {}", VALUE, value, borrow(&value), raw_borrow(&value));
}
