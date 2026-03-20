use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cu::pre::*;
use itertools::Itertools as _;
use pm::pre::*;

use super::kebab;
use super::platform::{Target, TargetSet};

pub fn parse_module_file_structure(top_path: &Path) -> cu::Result<Option<ModuleFileStructure>> {
    // allowed structures: (assuming foo-bar is the name of the package)
    // /foo-bar.rs
    // /foo-bar_target.rs
    // /foo-bar/mod.rs
    // /foo-bar/mod_target.rs

    let top_path_str = top_path.as_utf8()?;
    let file_name = cu::check!(
        top_path.file_name_str(),
        "unable to determine file name for module file structure: '{top_path_str}'"
    )?;

    if let Some(file_name_stripped) = file_name.strip_suffix(".rs") {
        let (package_name, targets) = 
        cu::check!(Target::parse(file_name_stripped), "failed to parse target from filep path: '{top_path_str}'")?;
        cu::ensure!(kebab::is_kebab(package_name), "in file: '{top_path_str}'")?;
        let structure = ModuleFileStructure::single_file(package_name.to_string(), targets, top_path.to_path_buf());
        return Ok(Some(structure));
    }

    if !top_path.is_dir() {
        cu::warn!(
            "ignoring unknown file in packages dir: '{}'",
            top_path.display()
        );
        return Ok(None);
    }

    // /foo-bar/mod.rs
    // /foo-bar/mod_target.rs
    let package_name = file_name;
    cu::ensure!(!package_name.is_empty(), "in path: '{top_path_str}'")?;
    cu::ensure!(kebab::is_kebab(package_name), "in path: '{top_path_str}'")?;
    let mut structure = ModuleFileStructure::new(package_name.to_string());
    for entry in cu::fs::read_dir(top_path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.as_utf8()?;
        let file_name = cu::check!(
            path.file_name_str(),
            "unable to determine file name for module file structure: '{path_str}'"
        )?;
        let Some(file_name_stripped) = file_name.strip_suffix(".rs") else {
            continue;
        };
        let Ok((package_name, targets)) = Target::parse(file_name_stripped) else {
            continue;
        };
        cu::ensure!(package_name == "mod", "in file: '{path_str}'");
        structure.add(targets, path)?;
    }

    if structure.target_structure.is_empty() {
        cu::warn!("empty package file structure for package '{package_name}'");
        return Ok(None);
    }

    Ok(Some(structure))
}

pub struct ModuleFileStructure {
    pub package_name: String,
    pub target_structure: Vec<ModuleFileTarget>,
}
pub struct ModuleFileTarget {
    pub file: PathBuf,
    pub targets: TargetSet
}
impl ModuleFileStructure {
    pub fn single_file(package_name: String, targets: TargetSet, file: PathBuf) -> Self {
        Self {
            package_name: package_name.to_string(),
            target_structure: vec![
                ModuleFileTarget {
                    file,
                    targets
                }
            ]
        }
    }
    pub fn new(package_name: String) -> Self {
        Self {
            package_name,
            target_structure: vec![],
        }
    }
    pub fn add(&mut self, targets: TargetSet, file: PathBuf) -> cu::Result<()> {
        for s in &self.target_structure {
            let conflict = s.targets.intersection(targets);
            if !conflict.is_empty() {
                cu::bail!("conflicting targets '{:?}' in files '{}' and '{}'",
                    conflict,
                    s.file.display(),
                    file.display()
            );
            }
        }
        self.target_structure.push(ModuleFileTarget{file,targets});
        Ok(())
    }
}

pub struct ParsedModule {
    pub data: Vec<ModuleData>,
}
impl ParsedModule {
    pub fn parse(structure: &ModuleFileStructure) -> cu::Result<Self> {
        let mut data = Vec::with_capacity(structure.target_structure.len());
        for s in &structure.target_structure {
            let content = cu::fs::read_string(&s.file)?;
            let mut module_data = cu::check!(
                cu::parse::<ModuleData>(&content),
                "failed to parse file: '{}'",
                s.file.display()
            )?;
            module_data.targets = s.targets;
            data.push(module_data);
        }
        let mut need_fill_doc_targets = TargetSet::new();
        for d in &data {
            if d.short_desc().is_empty() {
                need_fill_doc_targets.extend(d.targets);
            }
        }
        let mut fill_doc = Vec::with_capacity(need_fill_doc_targets.len());
        for t in need_fill_doc_targets {
            match t {
                Target::LinuxAptX64 | Target::LinuxPacmanX64 => {
                    // try different architecture - no solution
                    // try different flavor
                    let mut targets_to_try = Target::linux_x64();
                    targets_to_try.remove(t);
                    'outer: for t2 in targets_to_try {
                        for d in &data {
                            if !d.targets.contains(t2) || d.short_desc().is_empty() {
                                continue;
                            }
                            fill_doc.push((t, d.doc.clone()));
                            break 'outer;
                        }
                    }
                }
                Target::WindowsX64 | Target::WindowsArm => {
                    // try different architecture
                    let mut targets_to_try = Target::win();
                    targets_to_try.remove(t);
                    'outer: for t2 in targets_to_try {
                        for d in &data {
                            if !d.targets.contains(t2) || d.short_desc().is_empty() {
                                continue;
                            }
                            fill_doc.push((t, d.doc.clone()));
                            break 'outer;
                        }
                    }
                }
                Target::MacosArm => {
                    // try different architecture - no solution
                }
            }
        }
        for (t, fill_doc) in fill_doc {
            for d in &mut data {
                if d.targets.contains(t) && d.short_desc().is_empty() {
                    d.doc = fill_doc;
                    break;
                }
            }
        }
        Ok(Self { data })
    }
    pub fn collect_binaries(&self, out: &mut BTreeSet<String>) {
        for d in &self.data {
            out.extend(d.kebab_binaries.iter().cloned());
        }
    }
}
// #[derive(Default)]
pub struct ModuleData {
    pub targets: TargetSet,
    /// doc paragraphs
    ///
    /// If not specified, it will try to find same OS but different architecture.
    /// Then on linux it will try other flavors. However it will not try other OS
    pub doc: Vec<String>,
    pub kebab_binaries: BTreeSet<String>,
    pub has_binary_dependencies: bool,
    pub has_config_dependencies: bool,
    pub has_download: bool,
    pub has_configure: bool,
    pub has_clean: bool,
    pub has_config_location: bool,
    pub has_backup_restore: bool,
    pub has_pre_uninstall: bool,
}
impl ModuleData {
    pub fn short_desc(&self) -> &str {
        match self.doc.first() {
            Some(x) => x,
            None => "",
        }
    }

    pub fn long_desc(&self) -> String {
        self.doc.iter().skip(1).join("\n\n")
    }
}

impl cu::Parse for ModuleData {
    type Output = Self;
    fn parse_borrowed(x: &str) -> cu::Result<Self::Output> {
        let file_syntax = syn::parse_file(x)?;

        let doc = parse_file_attributes_doc(&file_syntax.attrs);
        let mut binaries = BTreeSet::default();
        let mut export_idents = vec![];

        for item in file_syntax.items {
            match item {
                syn::Item::Macro(item) => {
                    let Some(ident) = item.mac.path.get_ident() else {
                        continue;
                    };
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "register_binaries" => {
                            let body = item.mac.parse_body::<MacroBody>()?;
                            for lit in body.items {
                                binaries.insert(lit.value());
                            }
                        }
                        "binary_dependencies" => {
                            export_idents.push("binary_dependencies".to_string());
                        }
                        "config_dependencies" => {
                            export_idents.push("config_dependencies".to_string());
                        }
                        "config_file" => {
                            export_idents.push("config_location".to_string());
                        }
                        _ => {}
                    }
                }

                syn::Item::Fn(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        export_idents.push(item.sig.ident.to_string());
                    }
                }

                syn::Item::Use(item) => {
                    if let syn::Visibility::Public(_) = item.vis {
                        extract_idents_from_use_tree(&item.tree, &mut export_idents);
                    }
                }

                _ => {}
            }
        }

        let mut has_binary_dependencies = false;
        let mut has_config_dependencies = false;
        let mut has_download = false;
        let mut has_configure = false;
        let mut has_clean = false;
        let mut has_config_location = false;
        let mut has_backup = false;
        let mut has_restore = false;
        let mut has_pre_uninstall = false;
        for ident in export_idents {
            match ident.as_str() {
                "binary_dependencies" => has_binary_dependencies = true,
                "config_dependencies" => has_config_dependencies = true,
                "download" => has_download = true,
                "configure" => has_configure = true,
                "clean" => has_clean = true,
                "config_location" => has_config_location = true,
                "backup" => has_backup = true,
                "restore" => has_restore = true,
                "pre_uninstall" => has_pre_uninstall = true,
                _ => {}
            }
        }
        if has_backup != has_restore {
            cu::bail!(
                "a package must have both `backup` and `restore` functions, or have neither."
            );
        }

        Ok(Self {
            targets: Default::default(),
            doc,
            kebab_binaries: binaries,
            has_binary_dependencies,
            has_config_dependencies,
            has_download,
            has_configure,
            has_clean,
            has_config_location,
            has_backup_restore: has_backup,
            has_pre_uninstall,
        })
    }
}

struct MacroBody {
    items: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]>,
}
impl syn::parse::Parse for MacroBody {
    fn parse(input: syn::parse::ParseStream) -> pm::Result<Self> {
        let items = syn::punctuated::Punctuated::parse_terminated(input)?;
        Ok(Self { items })
    }
}

fn parse_file_attributes_doc(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut doc = vec!["".to_string()];

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let Ok(lit) = attr.meta.require_name_value() else {
            continue;
        };
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(ref lit),
            ..
        }) = lit.value
        else {
            continue;
        };
        let doc_lit = lit.value();
        let doc_lit = doc_lit.trim();
        if doc_lit.is_empty() {
            if !doc.last().unwrap().is_empty() {
                doc.push("".to_string());
            }
        } else {
            let last = doc.last_mut().unwrap();
            if !last.is_empty() && !last.ends_with(' ') {
                last.push(' ');
            }
            last.push_str(doc_lit);
        }
    }

    doc
}

fn extract_idents_from_use_tree(tree: &syn::UseTree, out: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(tree) => {
            extract_idents_from_use_tree(&tree.tree, out);
        }
        syn::UseTree::Name(tree) => {
            out.push(tree.ident.to_string());
        }
        syn::UseTree::Rename(tree) => {
            out.push(tree.rename.to_string());
        }
        syn::UseTree::Glob(_) => {}
        syn::UseTree::Group(tree) => {
            for item in &tree.items {
                extract_idents_from_use_tree(item, out);
            }
        }
    }
}
