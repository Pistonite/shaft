mod build_registry;
mod util;

use cu::pre::*;

#[derive(clap::Parser, AsRef)]
struct Args {
    /// Always write the output
    #[clap(short, long)]
    clean: bool,
    #[clap(flatten)]
    #[as_ref]
    flags: cu::cli::Flags,
}

#[cu::cli]
fn main(args: Args) -> cu::Result<()> {
    cu::lv::disable_print_time();
    if args.clean {
        util::set_clean();
    }
    build_registry::metadata::build_metadata()?;
    build_registry::packages::build_packages()?;
    Ok(())
}
