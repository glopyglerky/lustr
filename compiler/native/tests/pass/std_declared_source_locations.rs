// Hook locations are stdout, so the ordinary differential gate compares the
// complete declared Rust source identity as well as its file, line and column.
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let at = info.location().unwrap();
        println!("{}:{}:{}", at.file(), at.line(), at.column());
    }));
    println!("{}", std::panic::catch_unwind(|| {
        let _ = i32::from_str_radix("10", 1);
    }).is_err());
    println!("{}", std::panic::catch_unwind(|| {
        let _ = std::time::Duration::new(u64::MAX, 1_000_000_000);
    }).is_err());
    println!("{}", std::panic::catch_unwind(|| {
        let mut values = vec![1, 2];
        let _ = values.drain(3..);
    }).is_err());
}
