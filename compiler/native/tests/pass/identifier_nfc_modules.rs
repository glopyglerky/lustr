#[path = "identifier_nfc_module.rs.inc"]
mod résumé;
use résumé::café as résultat;

fn main() {
    println!("{} {}", résumé::café(), résultat());
}
