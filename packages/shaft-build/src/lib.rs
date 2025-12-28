use std::path::Path;

use cu::pre::*;

mod parse;
mod emit;
mod util;

pub fn run_build(crate_path: &Path) {
    if let Err(e) = run_build_impl(crate_path) {
        for line in format!("{e:?}").lines() {
            println!("cargo::error={line}");
        }
    }
}

fn run_build_impl(crate_path: &Path) -> cu::Result<()> {
    let packages_path = {
        let mut p = crate_path.normalize()?;
        p.extend(["src", "packages"]);
        p
    };
    println!("cargo::rerun-if-changed={}", packages_path.as_utf8()?);
    let registry_path = packages_path.parent_abs()?;
    let output_path = registry_path.join("packages.gen.rs");

    let mut builder = emit::RegistryBuilder::new(registry_path);

    for entry in cu::fs::read_dir(&packages_path)? {
        let entry = entry?;
        let path = entry.path();
        let Some(structure) = parse::parse_module_file_structure(&path)? else {
            continue;
        };
        builder.add(structure)?;
    }

    let output = builder.build()?;
    cu::fs::write(output_path, output)?;

    Ok(())
}
