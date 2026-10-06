//! Test the python standalone environment
//!
//! cargo run -p shaft-corelib --example pyse -- 3.13.16 ./pyse-test six "certifi<2025"
use std::path::PathBuf;

use cu::pre::*;
use shaft_corelib::pyse::PythonStandaloneEnvironment;

#[derive(clap::Parser, Clone)]
struct Args {
    /// The CPython version, for example 3.14.8
    version: String,
    /// The installation directory
    location: PathBuf,
    /// Managed requirements
    requirements: Vec<String>,
    #[clap(flatten)]
    inner: cu::cli::Flags,
}

#[cu::cli(flags = "inner")]
fn main(args: Args) -> cu::Result<()> {
    let env = PythonStandaloneEnvironment::new(
        &args.location,
        &args.version,
        args.requirements,
        "this is a test environment, use the pipi script to install packages".to_string(),
    )?;
    if let Ok(x) = env.python_interpreter() {
        cu::info!("already set-up: {}", x.try_to_rel().display());
        return Ok(());
    }
    let bar = cu::progress("setting up python standalone environment").spawn();
    env.ensure(Some(bar.clone()))?;
    bar.done();
    cu::info!(
        "python: {}",
        env.python_interpreter()?.try_to_rel().display()
    );
    Ok(())
}
