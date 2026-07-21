//! Gradle - Build system for Java
use crate::pre::*;

binary_dependencies!(Java);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_cargo!("gradlew" in crate "gradle-wrapper-cli");
    check_outdated!(&v.version, metadata[gradle]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::binstall_git("gradle-wrapper-cli", metadata::gradle::REPO, ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("gradle-wrapper-cli")?;
    Ok(())
}
