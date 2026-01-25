//! GNU Coreutils, Diffutils, and other basic commands

use crate::pre::*;

mod eza;

register_binaries!("ls", "diff");

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::Git | BinId::CargoBinstall }
}

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    eza::verify()?;
    check_bin_in_path_and_shaft!("diff");
    check_bin_in_path_and_shaft!("diff3");
    check_bin_in_path_and_shaft!("cmp");
    check_bin_in_path_and_shaft!("gzip");
    check_bin_in_path_and_shaft!("sed");
    check_bin_in_path_and_shaft!("grep");
    let which_info = check_installed_with_cargo!("which");
    if Version(&which_info.version) < metadata::coreutils::which::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let coreutils_info = check_installed_with_cargo!("coreutils");
    if Version(&coreutils_info.version) < metadata::coreutils::uutils::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    eza::install()?;
    epkg::cargo::binstall("coreutils")?;
    epkg::cargo::install_git_commit("which", metadata::shellutils::REPO, metadata::shellutils::COMMIT)?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    eza::uninstall()?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    eza::configure(ctx)?;
    // configure coreutils
    // we need to copy installed coreutils to bin to ensure
    // it's on the same drive
    let coreutils_path = hmgr::paths::binary(bin_name!("coreutils"));
    cu::fs::remove(&coreutils_path)?;
    let coreutils_src = cu::which("coreutils")?;
    cu::fs::copy(&coreutils_src, &coreutils_path)?;
    let coreutils_path = coreutils_path.into_utf8()?;

    let list_output = command_output!("coreutils", ["--list"]);
    let utils: Vec<_> = list_output.lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric()))
        .collect();
    {
        let bar = cu::progress("configuring coreutils")
            .total(utils.len())
            .parent(ctx.bar())
            .spawn();
        let has_pwsh = ctx.is_installed(PkgId::Pwsh);

        // Run shell checks in parallel with pool of 4
        cu::co::run(async move {
            let pool = cu::co::pool(4);
            let mut handles = Vec::with_capacity(utils.len());
            for util in &utils {
                let util = util.to_string();
                let handle = pool.spawn(async move {
                    let is_alias = shell_has_alias("powershell", &util).await?
                    || (has_pwsh && shell_has_alias("pwsh", &util).await?);
                    let is_binary = shell_has_binary("powershell", &util).await?
                    || (has_pwsh && shell_has_binary("pwsh", &util).await?);
                    cu::Ok((util, is_alias, is_binary))
                });
                handles.push(handle);
            }

            let mut set = cu::co::set(handles);
            while let Some(result) = set.next().await {
                let (util, is_alias, is_binary) = result??;
                cu::progress!(bar += 1, "{util}");
                let link_path = hmgr::paths::binary(bin_name!(&util)).into_utf8()?;
                if is_alias {
                    cu::info!("removing powershell alias: {util}");
                    ctx.add_item(hmgr::Item::Pwsh(format!("Remove-Item Alias:{util} -Force")))?;
                }
                if is_binary {
                    cu::info!("overriding powershell command: {util}");
                    ctx.add_item(hmgr::Item::Pwsh(format!("Set-Alias -Name {util} -Value '{link_path}'")))?;
                }
                ctx.add_item(hmgr::Item::LinkBin(link_path, coreutils_path.clone()))?;
            }
            cu::Ok(())
        })?;
    }
    // configure utils from mingw
    const MINGW_UTILS: &[&str] = &["diff", "diff3", "cmp", "gzip", "sed", "grep"];
    for util in MINGW_UTILS {
        let exe_path = opfs::find_in_wingit(format!("usr/bin/{util}.exe"))?;
        ctx.add_item(hmgr::Item::ShimBin(
            bin_name!(util),
            vec![exe_path.into_utf8()?],
        ))?;
    }
    Ok(())
}

async fn shell_has_alias(shell: &str, util: &str) -> cu::Result<bool> {
    let script = format!(
        "if (Get-Alias {util} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"
    );
    cu::which(shell)?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .all_null()
        .co_wait().await
        .map(|s| s.success())
}

async fn shell_has_binary(shell: &str, util: &str) -> cu::Result<bool> {
    // Remove alias first (if any), then check if there's a binary
    let script = format!(
        "Remove-Item Alias:{util} -Force; if ((Get-Command {util} -ErrorAction SilentlyContinue).CommandType -eq 'Application') {{ exit 0 }} else {{ exit 1 }}"
    );
    cu::which(shell)?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .all_null()
        .co_wait().await
        .map(|s| s.success())
}
