use crate::util;

use super::{emit, parse};

pub fn build_packages() -> cu::Result<()> {
    let registry_path = util::registry_dir()?;
    let packages_path = registry_path.join("src").join("packages");
    let output_path = registry_path.join("src").join("packages.gen.rs");

    cu::info!("saving packages to {}", output_path.display());

    let mut builder = emit::RegistryBuilder::new(registry_path.join("src"));

    for entry in cu::fs::read_dir(&packages_path)? {
        let entry = entry?;
        let path = entry.path();
        let Some(structure) = parse::parse_module_file_structure(&path)? else {
            continue;
        };
        builder.add(structure)?;
    }

    let output = builder.build()?;
    util::write_str_if_modified("registry packages", &output_path, &output)?;

    Ok(())
}
