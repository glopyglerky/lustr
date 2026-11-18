fn main() -> Result<std::process::ExitCode, &'static str> {
    println!("result exit code");
    Ok(std::process::ExitCode::from(7))
}
