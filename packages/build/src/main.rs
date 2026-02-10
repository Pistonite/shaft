mod build_corelib;
mod build_registry;
mod util;

#[cu::cli]
fn main(_: cu::cli::Flags) -> cu::Result<()> {
    build_corelib::build_tools()?;
    build_registry::metadata::build_metadata()?;
    build_registry::packages::build_packages()?;
    Ok(())
}
