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
    let _ = writeln!(
        out,
        r###"#![allow(unused)]
use corelib::opfs;"###
    );

    let mut path = PathStack::new();
    for (meta_pkg, value) in table {
        path.push(&meta_pkg);
        build_metadata_item(&mut out, 0, &meta_pkg, &mut path, "", &value)?;
        path.pop();
    }

    util::write_str_if_modified("registry metadata", &metadata_output_path, &out)?;
    Ok(())
}

fn build_metadata_module_or_function(
    out: &mut String,
    depth: usize,
    name: &str,
    path: &mut PathStack,
    cfg_attr: &str,
    table: &toml::Table,
) -> cu::Result<()> {
    if let Some(match_expr) = table.get("__match__") {
        let match_expr = cu::check!(
            match_expr.as_str(),
            "__match__ expression must be a string, at: {path}"
        )?;
        let mut out2 = String::new();
        let mut ty = None;
        for (key, value) in table {
            if key == "__match__" {
                continue;
            }
            path.push(key);
            if key.starts_with("cfg(") {
                let toml::Value::Table(value) = value else {
                    cu::bail!("cfg attribute must be a table, at: {path}.{key}");
                };
                for (cfg_key, cfg_value) in value {
                    if cfg_key.starts_with("cfg(") {
                        cu::bail!("consecutive cfg not allowed, at: {path}.{key}.{cfg_key}");
                    }
                    let cfg_attr = format!("#[{key}]");
                    path.push(cfg_key);
                    let next_ty = build_metadata_match_arm(
                        &mut out2,
                        depth + 1,
                        cfg_key,
                        path,
                        &cfg_attr,
                        cfg_value,
                    )?;
                    match &mut ty {
                        None => ty = Some(next_ty),
                        Some(old) => {
                            if *old != next_ty {
                                cu::bail!("type mismatch in match: {old} != {next_ty}, at: {path}");
                            }
                        }
                    }
                    path.pop();
                }
            } else {
                let next_ty = build_metadata_match_arm(&mut out2, depth + 1, key, path, "", value)?;
                match &mut ty {
                    None => ty = Some(next_ty),
                    Some(old) => {
                        if *old != next_ty {
                            cu::bail!("type mismatch in match: {old} != {next_ty}, at: {path}");
                        }
                    }
                }
            }
            path.pop();
        }
        if !cfg_attr.is_empty() {
            let _ = writeln!(out, "{:>width$}{cfg_attr}", "", width = depth * 4);
        }
        let _ = writeln!(
            out,
            "{:>width$}#[allow(non_snake_case)]",
            "",
            width = depth * 4
        );

        let ty = cu::check!(ty, "empty match expression, at: {path}")?;
        let ty = if ty == "&str" { "&'static str" } else { ty };
        let _ = writeln!(
            out,
            "{:>width$}pub fn {name}() -> {ty} {{ match {match_expr} {{",
            "",
            width = depth * 4
        );
        out.push_str(&out2);
        let _ = writeln!(out, "{:>width$}}} }}", "", width = depth * 4);

        return Ok(());
    }

    build_metadata_module(out, depth, name, path, cfg_attr, table)
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
        let _ = writeln!(out, "{:>width$}{cfg_attr}", "", width = depth * 4);
    }
    let _ = writeln!(out, "{:>width$}pub mod {name} {{", "", width = depth * 4);
    let _ = writeln!(out, "{:>width$}    use super::*;", "", width = depth * 4);

    for (key, value) in table {
        path.push(key);
        if key.starts_with("cfg(") {
            let toml::Value::Table(value) = value else {
                cu::bail!("cfg attribute must be a table, at: {path}.{key}");
            };
            for (cfg_key, cfg_value) in value {
                if cfg_key.starts_with("cfg(") {
                    cu::bail!("consecutive cfg not allowed, at: {path}.{key}.{cfg_key}");
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
        toml::Value::String(s) => parse_toml_value(s),
        toml::Value::Integer(x) => {
            cu::bail!("use a literal to specify numbers (e.g \"{x}i64\"), at: {path}");
        }
        toml::Value::Float(x) => {
            cu::bail!("use a literal to specify numbers (e.g. \"{x}f64\"), at: {path}");
        }
        toml::Value::Boolean(s) => ("bool", format!("{s}")),
        toml::Value::Datetime(_) => {
            cu::bail!("datetime is not supported, at: {path}");
        }
        toml::Value::Array(_) => {
            cu::bail!("array is not supported, at: {path}");
        }
        toml::Value::Table(table) => {
            return build_metadata_module_or_function(out, depth, name, path, cfg_attr, table);
        }
    };

    if !cfg_attr.is_empty() {
        let _ = writeln!(out, "{:>width$}{cfg_attr}", "", width = depth * 4);
    }
    let _ = writeln!(
        out,
        "{:>width$}pub static {name}: {ty} = {value};",
        "",
        width = depth * 4
    );

    Ok(())
}

fn build_metadata_match_arm(
    out: &mut String,
    depth: usize,
    match_arm: &str,
    path: &mut PathStack,
    cfg_attr: &str,
    value: &toml::Value,
) -> cu::Result<&'static str> {
    let s = cu::check!(value.as_str(), "match arm must be string, at: {path}")?;
    let (ty, value) = parse_toml_value(s);
    if cfg_attr.is_empty() {
        let _ = writeln!(
            out,
            "{:>width$}{match_arm} => {{ {value} }}",
            "",
            width = depth * 4
        );
    } else {
        let _ = writeln!(
            out,
            "{0:>width$}{cfg_attr}\n{0:>width$}{match_arm} => {{ {value} }}",
            "",
            width = depth * 4
        );
    }
    Ok(ty)
}

fn parse_toml_value(s: &str) -> (&'static str, String) {
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
