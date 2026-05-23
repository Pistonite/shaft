use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cu::pre::*;

fn main() -> ExitCode {
    if let Err(e) = main_internal() {
        println!("cargo::error={e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
fn main_internal() -> cu::Result<()> {
    let config_path = cu::env_var("SHAFT_SHIM_BUILD_CONFIG")?;
    if config_path.is_empty() {
        cu::bail!("SHAFT_SHIM_BUILD_CONFIG not set!");
    }
    let main_rs = shaftim_build::build(Path::new(&config_path))?;

    let mut main_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    main_path.push("main.rs");
    println!("cargo::rerun-if-changed={}", main_path.as_utf8()?);
    cu::fs::write(main_path, main_rs)?;
    Ok(())
}
