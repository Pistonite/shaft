//! CMake makefile generator

use crate::pre::*;

register_binaries!("cmake");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("cmake");
    let v = command_output!("cmake", ["--version"]);
    let mut v = v.split_whitespace();
    if v.next() != Some("cmake") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    if v.next() != Some("version") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    let Some(v) = v.next() else {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(v, metadata[cmake]::VERSION);

    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "cmake.tar.gz",
        metadata::cmake::URL(),
        metadata::cmake::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

fn release_name() -> String {
    let version = metadata::cmake::VERSION;
    format!("cmake-{version}-linux-x86_64")
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    {
        let bar = cu::progress("unpacking cmake")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let temp_extract_dir = hmgr::paths::temp_dir("cmake-extract");
        let cmake_tar = hmgr::paths::download("cmake.tar.gz", metadata::cmake::URL());
        // the archive contains a top-level directory, with bin/, share/, doc/ and man/ inside
        opfs::unarchive_rename(
            &cmake_tar,
            &temp_extract_dir,
            temp_extract_dir.join(release_name()),
            ctx.install_dir(),
            true,
            None,
        )?;
        bar.done();
    }
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let cmake_bin_dir = ctx.install_dir().join("bin");
    // shim instead of link, since cmake finds its modules (share/cmake-x.y)
    // relative to the real executable path
    for file_name in ["cmake", "ccmake", "cpack", "ctest", "cmake-gui"] {
        let to = cmake_bin_dir.join(file_name).into_utf8()?;
        ctx.add_item(Item::shim_bin(file_name, ShimCommand::target(to)))?;
    }

    // The completion scripts support both new (_comp_initialize) and old (_init_completion)
    // bash-completion APIs, and only need them to exist when completing
    let completion_dir =
        cu::path!((ctx.install_dir()) / "share" / "bash-completion" / "completions");
    let mut script = String::new();
    for file_name in ["cmake", "cpack", "ctest"] {
        let path = completion_dir.join(file_name).into_utf8()?;
        script.push_str(&format!("source '{path}'\n"));
    }
    ctx.add_item(Item::bash(script))?;
    Ok(())
}
