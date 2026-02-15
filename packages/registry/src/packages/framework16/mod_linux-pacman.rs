//! Configuration for Framework Laptop 16

use crate::pre::*;

pub fn verify(_: &Context) -> cu::Result<Verified> {
    Ok(Verified::NotInstalled)
    // TODO: https://github.com/paco3346/fw16-kbd-uleds
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    Ok(())
}
