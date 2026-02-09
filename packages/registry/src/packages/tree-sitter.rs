//! Tree-sitter CLI for building and compiling tree-sitter parsers
use crate::pre::*;
register_binaries!("tree-sitter");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("tree-sitter");
    let v = command_output!("tree-sitter", ["--version"]);
    let v = v.strip_prefix("tree-sitter ").unwrap_or(&v);
    check_outdated!(v, metadata[tree_sitter]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    let file_name = tree_sitter_base_name()?;
    hmgr::download_file(
        format!("{file_name}.gz"),
        tree_sitter_url()?,
        metadata::tree_sitter::SHA,
        ctx.bar(),
    )?;
    Ok(())
}

fn tree_sitter_url() -> cu::Result<String> {
    let repo = metadata::tree_sitter::REPO;
    let version = metadata::tree_sitter::VERSION;
    let file_name = tree_sitter_base_name()?;
    Ok(format!(
        "{repo}/releases/download/v{version}/{file_name}.gz"
    ))
}

fn tree_sitter_base_name() -> cu::Result<&'static str> {
    if cfg!(windows) {
        Ok(if_arm!("tree-sitter-windows-arm64", else "tree-sitter-windows-x64"))
    } else if cfg!(target_os = "linux") {
        Ok("tree-sitter-linux-x64")
    } else {
        cu::bail!("macos unsupported yet")
    }
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    let file_name = tree_sitter_base_name()?;
    let tree_sitter_download = hmgr::paths::download(format!("{file_name}.gz"), tree_sitter_url()?);
    let tree_sitter_bytes = cu::fs::read(tree_sitter_download)?;
    let tree_sitter_binary_path = ctx.install_dir().join(bin_name!("tree-sitter"));
    opfs::ungz_bytes(&tree_sitter_bytes, &tree_sitter_binary_path)?;
    opfs::set_executable(&tree_sitter_binary_path)?;
    Ok(())
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    let tree_sitter_binary_path = ctx.install_dir().join(bin_name!("tree-sitter"));
    ctx.add_item(Item::link_bin(
        bin_name!("tree-sitter"),
        tree_sitter_binary_path.into_utf8()?,
    ))?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
