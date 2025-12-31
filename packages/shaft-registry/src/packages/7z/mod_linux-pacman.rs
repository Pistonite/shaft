use cu::pre::*;
use enumset::EnumSet;

use crate::pre::*;

metadata_binaries!("7z");

static VERSION: &str = "25.01";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    todo!()
}
pub fn install(_: &Context) -> cu::Result<()> {
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
