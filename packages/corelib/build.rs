use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use cu::pre::*;
use flate2::Compression;
use flate2::write::GzEncoder;
use ignore::WalkBuilder;
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
    println!("cargo::rerun-if-changed={}", tools_path.as_utf8()?);

    // Create a Cargo.toml for tools that inherit dependencies versions
    // from shaft itself
    let workspace_cargo_toml = {
        let mut path = crate_path.parent_abs_times(2)?;
        path.push("Cargo.toml");
        cu::toml::parse::<cu::toml::Table>(&cu::fs::read_string(&path)?)?
    };

    let tools_cargo_content = {
        let mut out = r#"
[workspace]
resolver = "2"
members = [
    "shaftim",
    "shaftim-build",
]
"#
        .to_string();
        let mut new_table = cu::toml::Table::new();
        if let Some(workspace) = workspace_cargo_toml.get("workspace") {
            if let Some(deps) = workspace.get("dependencies") {
                let mut ws = cu::toml::Table::new();
                ws.insert("dependencies".to_string(), deps.clone());
                new_table.insert("workspace".to_string(), cu::toml::Value::Table(ws));
            }
        }
        out.push_str(&cu::toml::stringify_pretty(&new_table)?);
        out
    };

    cu::fs::write(tools_path.join("Cargo.toml"), &tools_cargo_content)?;
    {
        let bytes = tools_cargo_content.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("Cargo.toml")?;
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar_builder.append(&header, bytes)?;
    }

    let mut builder = WalkBuilder::new(&tools_path);
    builder.filter_entry(|entry| {
        if entry.file_type().is_none_or(|x| !x.is_dir()) {
            return true;
        }
        cfg!(windows) || entry.file_name() != "__windows__"
    });
    builder.add_custom_ignore_filename(".corelibignore");

    for entry in builder.build() {
        let entry = entry?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        let rel_path = entry_path.try_to_rel_from(&tools_path);
        cu::ensure!(rel_path.is_relative(), "'{}'", rel_path.display())?;
        let mut file = cu::check!(
            File::open(entry_path),
            "failed to open '{}'",
            entry_path.display()
        )?;
        tar_builder.append_file(&rel_path, &mut file)?;
    }

    tar_builder.into_inner()?.finish()?.flush()?;
    Ok(())
}
