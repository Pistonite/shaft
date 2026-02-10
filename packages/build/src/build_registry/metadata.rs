use std::fmt::Write as _;

use cu::pre::*;

use crate::util;

/// Build registry metadata file from metadata.toml
pub fn build_metadata() -> cu::Result<()> {
    let registry_path = util::registry_dir()?;
    let metadata_toml_path = registry_path.join("metadata.toml");
    let metadata_output_path = registry_path.join("src").join("metadata.gen.rs");

    cu::info!("saving metadata to {}", metadata_output_path.display());

    let table = toml::parse::<toml::Table>(&cu::fs::read_string(metadata_toml_path)?)?;

    let mut out = String::new();
    let _ = writeln!(out, "#![allow(unused)]");

    let mut path = PathStack::new();
    for (meta_pkg, value) in table {
        path.push(&meta_pkg);
        build_metadata_item(&mut out, 0, &meta_pkg, &mut path, "", &value)?;
        path.pop();
    }

    util::write_str_if_modified("registry metadata", &metadata_output_path, &out)?;
    Ok(())
}

fn build_metadata_module(
    out: &mut String,
    depth: usize,
    name: &str,
    path: &mut PathStack,
    cfg_attr: &str,
    table: &toml::Table,
) -> cu::Result<()> {
    use std::fmt::Write as _;

    if !cfg_attr.is_empty() {
        let _ = writeln!(out, "{:width$}pub mod {name} {{", "", width = depth * 4);
    } else {
        let _ = writeln!(
            out,
            "{:width$}{cfg_attr}\npub mod {name} {{",
            "",
            width = depth * 4
        );
    }

    for (key, value) in table {
        path.push(key);
        if key.starts_with("cfg(") {
            let toml::Value::Table(value) = value else {
                cu::bail!("cargo:error=cfg attribute must be a table, at: {path}.{key}");
            };
            for (cfg_key, cfg_value) in value {
                if cfg_key.starts_with("cfg(") {
                    cu::bail!(
                        "cargo:error=consecutive cfg not allowed, at: {path}.{key}.{cfg_key}"
                    );
                }
                let cfg_attr = format!("#[{key}]");
                path.push(cfg_key);
                build_metadata_item(out, depth + 1, cfg_key, path, &cfg_attr, cfg_value)?;
                path.pop();
            }
        } else {
            build_metadata_item(out, depth + 1, key, path, "", value)?;
        }
        path.pop();
    }
    let _ = writeln!(out, "{:width$}}}", "", width = depth * 4);
    Ok(())
}

fn build_metadata_item(
    out: &mut String,
    depth: usize,
    name: &str,
    path: &mut PathStack,
    cfg_attr: &str,
    value: &toml::Value,
) -> cu::Result<()> {
    use std::fmt::Write as _;

    let (ty, value) = match value {
        toml::Value::String(s) => {
            if s.starts_with('\'') && s.ends_with('\'') {
                ("&str", format!("{:?}", &s[1..s.len() - 1]))
            } else if s.starts_with('"') && s.ends_with('"') {
                ("&str", format!("{s:?}"))
            } else if s.strip_suffix("u8").is_some() {
                ("u8", s.to_string())
            } else if s.strip_suffix("u16").is_some() {
                ("u16", s.to_string())
            } else if s.strip_suffix("u32").is_some() {
                ("u32", s.to_string())
            } else if s.strip_suffix("u64").is_some() {
                ("u64", s.to_string())
            } else if s.strip_suffix("u128").is_some() {
                ("u128", s.to_string())
            } else if s.strip_suffix("usize").is_some() {
                ("usize", s.to_string())
            } else if s.strip_suffix("i8").is_some() {
                ("i8", s.to_string())
            } else if s.strip_suffix("i16").is_some() {
                ("i16", s.to_string())
            } else if s.strip_suffix("i32").is_some() {
                ("i32", s.to_string())
            } else if s.strip_suffix("i64").is_some() {
                ("i64", s.to_string())
            } else if s.strip_suffix("i128").is_some() {
                ("i128", s.to_string())
            } else if s.strip_suffix("isize").is_some() {
                ("isize", s.to_string())
            } else if s.strip_suffix("f32").is_some() {
                ("f32", s.to_string())
            } else if s.strip_suffix("f64").is_some() {
                ("f64", s.to_string())
            } else {
                ("&str", format!("{s:?}"))
            }
        }
        toml::Value::Integer(x) => {
            cu::bail!("cargo:error=use a literal to specify numbers (e.g \"{x}i64\"), at: {path}");
        }
        toml::Value::Float(x) => {
            cu::bail!("cargo:error=use a literal to specify numbers (e.g. \"{x}f64\"), at: {path}");
        }
        toml::Value::Boolean(s) => ("bool", format!("{s}")),
        toml::Value::Datetime(_) => {
            cu::bail!("cargo:error=datetime is not supported, at: {path}");
        }
        toml::Value::Array(_) => {
            cu::bail!("cargo:error=array is not supported, at: {path}");
        }
        toml::Value::Table(table) => {
            return build_metadata_module(out, depth, name, path, cfg_attr, table);
        }
    };

    let _ = writeln!(
        out,
        "{:>width$}{cfg_attr}pub static {name}: {ty} = {value};",
        "",
        width = depth * 4
    );

    Ok(())
}

struct PathStack(Vec<String>);

impl PathStack {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, segment: &str) {
        self.0.push(segment.to_string());
    }

    fn pop(&mut self) {
        self.0.pop();
    }
}

impl std::fmt::Display for PathStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for segment in &self.0 {
            if !first {
                f.write_str(".")?;
            }
            f.write_str(segment)?;
            first = false;
        }
        Ok(())
    }
}
