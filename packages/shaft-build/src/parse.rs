use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cu::pre::*;
use itertools::Itertools as _;
use pm::pre::*;

use crate::util::{self, Platform};

pub fn parse_module_file_structure(top_path: &Path) -> cu::Result<Option<ModuleFileStructure>> {
    // allowed structures: (assuming foo-bar is the name of the package)
    // /foo-bar.rs
    // /foo-bar_platform.rs
    // /foo-bar/mod.rs
    // /foo-bar/mod_platform.rs

    let top_path_str = top_path.as_utf8()?;
    let file_name = cu::check!(top_path.file_name(), "unable to determine file name for module file structure: '{top_path_str}'")?;
    // unwrap: checked path is utf-8 above
    let file_name = file_name.to_str().unwrap();

    if let Some(file_name_stripped) = file_name.strip_suffix(".rs") {
        let mut parts = file_name_stripped.split("_");
        let package_name = cu::check!(parts.next(), "unable to determine package name from file: '{top_path_str}'")?;
        cu::ensure!(!package_name.is_empty(), "package name cannot be empty: in file: '{top_path_str}'");
        cu::ensure!(util::is_kebab(package_name), "package name must be kebab-case: in file: '{top_path_str}'");
        let platform = match parts.next() {
            None => Platform::Any,
            Some(x) => cu::check!(cu::parse::<Platform>(x), "unable to determine package platform: in file: '{top_path_str}'")?
        };
        cu::ensure!(parts.next().is_none(), "package file should contains at most one '_': in file: '{top_path_str}'");
        let files = std::iter::once((platform, top_path.to_path_buf())).collect();
        let structure = ModuleFileStructure {
            package_name: package_name.to_string(),
            files
        };
        return Ok(Some(structure));
    }

    if !top_path.is_dir() {
        cu::warn!("ignoring unknown file in packages dir: '{}'", top_path.display());
        return Ok(None);
    }
    
    let package_name = file_name;
    cu::ensure!(!package_name.is_empty(), "package name cannot be empty: in path: '{top_path_str}'");
    cu::ensure!(util::is_kebab(package_name), "package name must be kebab-case: in path: '{top_path_str}'");
    let mut structure = ModuleFileStructure::new(package_name.to_string());
    for entry in cu::fs::read_dir(top_path)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.as_utf8()?;
        let file_name = cu::check!(path.file_name(), "unable to determine file name for module file structure: '{path_str}'")?;
        // unwrap: checked path is utf-8 above
        let file_name = file_name.to_str().unwrap();
        if file_name == "mod.rs" {
            structure.add(Platform::Any, path)?;
            continue;
        }
        let Some(file_name_stripped) = file_name.strip_suffix(".rs") else {
            continue;
        };
        let Some(platform) = file_name_stripped.strip_prefix("mod_") else {
            continue;
        };
        let platform = cu::check!(cu::parse::<Platform>(platform), "unable to determine package platform: in file: '{path_str}'")?;
        structure.add(platform, path)?;
    }

    if structure.files.is_empty() {
        println!("cargo::warning=empty package file structure for package '{package_name}'");
        return Ok(None);
    }
    
    Ok(Some(structure))
}

pub struct ModuleFileStructure {
    pub package_name: String,
    pub files: BTreeMap<Platform, PathBuf>
}
impl ModuleFileStructure {
    pub fn new(package_name: String) -> Self {
        Self {
            package_name,
            files: Default::default()
        }
    }
    pub fn extend(&mut self, other: BTreeMap<Platform,PathBuf>) -> cu::Result<()> {
        for (platform, path) in other {
            self.add(platform, path)?;
        }
        Ok(())
    }
    pub fn add(&mut self, platform: Platform, path: PathBuf) -> cu::Result<()> {
            if let Some(existing_platform) = platform.find_conflict(self.files.keys().copied()) {
                let existing_path = self.files.get(&existing_platform).expect("should find existing platform");
                cu::bail!("conflicting platform '{}' (file: '{}') and '{}' (file: '{}')", 
                    existing_platform,
                    existing_path.display(),
                    platform,
                    path.display()
                );
            }
            self.files.insert(platform, path);
        Ok(())
    }
}

pub struct ParsedModule {
    pub platform_data: BTreeMap<Platform, ModuleData>,
}
impl ParsedModule {
    pub fn parse(structure: &ModuleFileStructure) -> cu::Result<Self> {
        let mut platform_data = BTreeMap::default();
        for (platform, path) in &structure.files {
            let content = cu::fs::read_string(path)?;
            let data = cu::check!(cu::parse::<ModuleData>(&content), "failed to parse file: '{}'", path.display())?;
            platform_data.insert(*platform, data);
        }
        Ok(Self{platform_data})
    }
    pub fn collect_binaries(&self, out: &mut BTreeSet<String>) {
        for data in self.platform_data.values() {
            out.extend(data.kebab_binaries.iter().cloned());
        } 
    }
}
pub struct ModuleData {
    pub doc: Vec<String>,
    pub kebab_binaries: BTreeSet<String>,
    pub has_binary_dependencies: bool,
    pub has_config_dependencies: bool,
    pub has_download: bool,
    pub has_build: bool,
    pub has_configure: bool,
    pub has_clean: bool,
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
                    if item.mac.path.is_ident("metadata_binaries") {
                        let body = item.mac.parse_body::<MacroBody>()?;
                        for lit in body.items {
                            binaries.insert(lit.value());
                        }
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
        let mut has_build = false;
        let mut has_configure = false;
        let mut has_clean = false;
        for ident in export_idents {
            match ident.as_str() {
                "binary_dependencies" => has_binary_dependencies = true,
                "config_dependencies" => has_config_dependencies = true,
                "download" => has_download = true,
                "build" => has_build = true,
                "configure" => has_configure = true,
                "clean" => has_clean = true,
                _ => {}
            }
        }

        Ok(Self {
            doc,
            kebab_binaries: binaries,
            has_binary_dependencies,
            has_config_dependencies,
            has_download,
            has_build,
            has_configure,
            has_clean,
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
