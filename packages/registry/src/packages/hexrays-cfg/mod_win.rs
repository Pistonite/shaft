//! Configuration for Hex-Rays IDA Pro

use crate::pre::*;

register_binaries!("hexrays", "hexrays-python", "vihexrays"); // uv creates both python/python3
binary_dependencies!(Uv, Viopen);
version_cache!(pub static VERSION = metadata::hexrays::CFG_VERSION);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    todo!()
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    todo!()
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    todo!()
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
