//! Configuration for Hex-Rays IDA Pro

use crate::pre::*;

register_binaries!("hexrays", "hexrays-python", "vihexrays"); // uv creates both python/python3
binary_dependencies!(Uv, Viopen);
version_cache!(pub static CFG_VERSION = metadata::hexrays::CFG_VERSION);

static IDA_PRO_VERSION: &str = "9.4";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    cu::check!(
        find_ida_pro(),
        "hexrays-cfg requires IDA Pro {IDA_PRO_VERSION} to be installed"
    )?;
    check_in_shaft!("hexrays");
    check_in_shaft!("hexrays-python");
    check_in_shaft!("vihexrays");
    check_config_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    todo!()
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    todo!()
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    todo!()
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn find_ida_pro() -> cu::Result<PathBuf> {
    let system_drive = opfs::system_drive()?;
    let ida_path = PathBuf::from(format!(
        "{system_drive}\\Program Files\\IDA Professional {IDA_PRO_VERSION}"
    ));
    if !ida_path.is_dir() {
        cu::bail!(
            "IDA Pro {IDA_PRO_VERSION} is not installed: expected installation at '{}'",
            ida_path.display()
        );
    }
    if !ida_path.join("ida.exe").is_file() {
        cu::bail!(
            "IDA Pro installation at '{}' is invalid: ida.exe not found",
            ida_path.display()
        );
    }
    Ok(ida_path)
}

fn ida_pro_user_dir() -> cu::Result<PathBuf> {
    // let mut appdata = cu::env_var("APPDATA").unwrap_or_default();
    // if appdata.is_empty() {
    //
    // }
    // let system_drive = system_drive.trim_end_matches(['\\', '/']);
    // let ida_path = PathBuf::from(format!(
    //     "{system_drive}\\Program Files\\IDA Professional {IDA_PRO_VERSION}"
    // ));
    //     let system_drive = opfs::system_drive()?;
    // }
    // let appdata =
    todo!()
}
