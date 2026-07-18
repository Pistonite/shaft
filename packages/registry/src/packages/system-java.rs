//! Use Java Toolchain in PATH
use crate::pre::*;

register_binaries!(
    "java",
    "javac"
);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_path!("java");
    check_in_path!("javac");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    verify(ctx)?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
