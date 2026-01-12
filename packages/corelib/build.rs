use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cu::pre::*;
use flate2::Compression;
use flate2::write::GzEncoder;
use ignore::Walk;
use tar::{Builder as TarBuilder, HeaderMode};

fn main() -> cu::Result<()> {
    make_tools_targz()
}

/// Pack the tools directory into a .tar.gz at hmgr/tools.tar.gz
fn make_tools_targz() -> cu::Result<()> {
    let crate_path = PathBuf::from(cu::env_var("CARGO_MANIFEST_DIR")?);
    let mut tar_builder = {
        let mut path = crate_path.clone();
        path.extend(["src", "hmgr", "tools.tar.gz"]);
        let file = cu::fs::writer(path)?;
        let gz_encoder = GzEncoder::new(file, Compression::default());
        let mut builder = TarBuilder::new(gz_encoder);
        builder.mode(HeaderMode::Deterministic);
        builder.follow_symlinks(false);
        builder
    };

    let tools_path = {
        let mut path = crate_path.parent_abs()?;
        path.push("tools");
        path
    };

    for entry in Walk::new(&tools_path) {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let rel_path = entry_path.try_to_rel_from(&tools_path);
        cu::ensure!(rel_path.is_relative(), "'{}'", rel_path.display())?;
        let mut file = cu::check!(
            File::open(&entry_path),
            "failed to open '{}'",
            entry_path.display()
        )?;
        println!("cargo::rerun-if-changed={}", entry_path.as_utf8()?);
        tar_builder.append_file(&rel_path, &mut file)?;
    }

    tar_builder.into_inner()?.finish()?.flush()?;
    Ok(())
}
