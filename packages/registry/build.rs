use std::path::Path;

fn main() -> cu::Result<()> {
    let crate_path = cu::env_var("CARGO_MANIFEST_DIR")?;
    shaft_registry_build::run_build(Path::new(&crate_path));
    Ok(())
}
