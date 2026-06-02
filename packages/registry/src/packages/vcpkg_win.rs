//! Vcpkg C/C++ dependency management tool

use crate::pre::*;

use itertools::Itertools as _;

register_binaries!("vcpkg");
binary_dependencies!(Git);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("vcpkg");
    let version = command_output!("vcpkg", ["version"]);
    let version_parsed = version.lines().next().unwrap_or_default();
    let version_parsed = version_parsed.split(' ').next_back().unwrap_or_default();
    let version_parsed = version_parsed.split('-').take(3).join("-");
    if version_parsed.is_empty() {
        cu::warn!("failed to parse vcpkg version, raw output:\n{version}");
        return Ok(Verified::NotUpToDate);
    }

    check_outdated!(&version_parsed, metadata[vcpkg]::VERSION);
    // we don't check the version of the repo as it could be on a commit that's intentionally
    // overriden by the current user
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "vcpkg.exe",
        download_url(),
        metadata::vcpkg::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let vcpkg_exe = hmgr::paths::download("vcpkg.exe", download_url());
    let repo = ctx.install_dir();
    cu::check!(ensure_repo(&repo, ctx), "failed to checkout vcpkg repo")?;
    // see scripts/bootstrap.ps1
    let vcpkg_target = cu::path!(repo / "vcpkg.exe");
    cu::fs::copy(&vcpkg_exe, &vcpkg_target)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let repo = ctx.install_dir();
    let vcpkg_target = repo.join("vcpkg.exe");
    ctx.add_item(Item::shim_bin(
        bin_name!("vcpkg"),
        ShimCommand::target(vcpkg_target.into_utf8()?),
    ))?;
    ctx.add_item(Item::user_env_var("VCPKG_ROOT", repo.into_utf8()?))?;
    Ok(())
}

fn ensure_repo(repo: &Path, ctx: &Context) -> cu::Result<()> {
    let mut should_clone = true;
    if repo.exists() {
        should_clone = false;
        let fetch_checkout_result = (|| {
            cu::which("git")?
                .command()
                .stdoe(cu::lv::P)
                .stdin_inherit()
                .add(cu::args!["-C", repo, "fetch"])
                .wait_nz()?;
            cu::which("git")?
                .command()
                .stdoe(cu::lv::P)
                .stdin_inherit()
                .add(cu::args![
                    "-C",
                    repo,
                    "reset",
                    "--hard",
                    metadata::vcpkg::source::TAG
                ])
                .wait_nz()?;
            cu::Ok(())
        })();
        if let Err(e) = fetch_checkout_result {
            cu::error!("failed to update vcpkg repo: {e:?}");
            cu::hint!("will try clean clone");
            should_clone = true;
        }
    }
    if should_clone {
        cu::fs::make_dir_empty(repo)?;
        {
            let (child, bar, _) = cu::which("git")?
                .command()
                .stdoe(cu::pio::spinner("cloning vcpkg").configure_spinner(|s| s.parent(ctx.bar())))
                .stdin_inherit()
                .add(cu::args!["clone", metadata::vcpkg::source::REPO, repo])
                .spawn()?;
            child.wait_nz()?;
            bar.done();
        }
        cu::which("git")?
            .command()
            .stdoe(cu::lv::P)
            .stdin_inherit()
            .add(cu::args![
                "-C",
                repo,
                "config",
                "advice.detachedHead",
                "false"
            ])
            .wait_nz()?;
        cu::which("git")?
            .command()
            .stdoe(cu::lv::P)
            .stdin_inherit()
            .add(cu::args![
                "-C",
                repo,
                "checkout",
                metadata::vcpkg::source::TAG
            ])
            .wait_nz()?;
    }
    Ok(())
}

fn download_url() -> String {
    let repo = metadata::vcpkg::REPO;
    let arch = if opfs::is_arm() { "-arm64" } else { "" };
    let version = metadata::vcpkg::VERSION;
    format!("{repo}/releases/download/{version}/vcpkg{arch}.exe")
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}
