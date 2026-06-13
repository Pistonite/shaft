//! PowerShell 7

use crate::pre::*;

register_binaries!("pwsh");
binary_dependencies!(_7z);
config_dependencies!(Shellutils); // for vipwsh

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    match cu::which("pwsh") {
        Err(_) => return Ok(Verified::NotInstalled),
        Ok(path) => {
            if path != hmgr::paths::binary(bin_name!("pwsh"))
            // sometimes inside the shell, the pwsh executable can point
            // to the real executable, so we need this extra check
            && path != ctx.install_dir().join(bin_name!("pwsh"))
            {
                cu::bail!(
                    "found existing '{}' installed outside of shaft, please uninstall it first (at '{}')",
                    "pwsh",
                    path.display()
                );
            }
        }
    }
    let version = command_output!(
        "pwsh",
        [
            "-NoLogo",
            "-NoProfile",
            "-c",
            "$PSVersionTable.PSVersion.ToString()"
        ]
    );
    check_outdated!(version.trim(), metadata[pwsh]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("pwsh.zip", download_url(), metadata::pwsh::SHA(), ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    let pwsh_dir = ctx.install_dir();
    let all_hosts_profile = pwsh_dir.join("profile.ps1");
    let curr_host_profile = pwsh_dir.join("Microsoft.PowerShell_profile.ps1");
    let all_hosts_profile_str = cu::fs::read_string(&all_hosts_profile).ok();
    let curr_host_profile_str = cu::fs::read_string(&curr_host_profile).ok();

    ctx.move_install_to_old_if_exists()?;

    struct WriteProfileGuard {
        all_hosts_profile: PathBuf,
        curr_host_profile: PathBuf,
        all_hosts_profile_str: Option<String>,
        curr_host_profile_str: Option<String>,
    }
    impl Drop for WriteProfileGuard {
        fn drop(&mut self) {
            if let Some(s) = &self.all_hosts_profile_str {
                if let Err(e) = cu::fs::write(&self.all_hosts_profile, s) {
                    cu::warn!("failed to write back old PowerShell profile: {e:?}");
                }
            }
            if let Some(s) = &self.curr_host_profile_str {
                if let Err(e) = cu::fs::write(&self.curr_host_profile, s) {
                    cu::warn!("failed to write back old PowerShell profile: {e:?}");
                }
            }
        }
    }
    let guard = WriteProfileGuard {
        all_hosts_profile,
        curr_host_profile,
        all_hosts_profile_str,
        curr_host_profile_str,
    };

    let pwsh_zip = hmgr::paths::download("pwsh.zip", download_url());
    opfs::unarchive(pwsh_zip, &pwsh_dir, true)?;

    drop(guard);

    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let pwsh_exe = install_dir.join("pwsh.exe");
    ctx.add_item(Item::shim_bin(
        bin_name!("pwsh"),
        ShimCommand::target(pwsh_exe.as_utf8()?),
    ))?;

    let ps7_profile_path = install_dir.join("profile.ps1");
    let mut edit_profile_path = ps7_profile_path.clone();
    let config = ctx.load_config(CONFIG)?;

    if let Some(ps5_profile) = config.use_ps5_profile {
        // get ps5 profile location
        let (child, stdout) = cu::which("powershell.exe")?
            .command()
            .args([
                "-NoLogo",
                "-NoProfile",
                "-c",
                &format!("$Profile.{ps5_profile}"),
            ])
            .stdout(cu::pio::string())
            .stderr(cu::lv::E)
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        let ps5_profile_path = Path::new(stdout.join()??.trim()).normalize()?;
        if let Ok(ps5_profile_content) = cu::fs::read_string(&ps5_profile_path) {
            cu::fs::write(ps7_profile_path, ps5_profile_content)?;
        }
        edit_profile_path = ps5_profile_path;
    }

    if ctx.is_installed(PkgId::Shellutils) {
        ctx.add_item(Item::shim_bin(
            bin_name!("vipwsh"),
            ShimCommand::target("viopen").args([edit_profile_path.into_utf8()?]),
        ))?;
    }
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn download_url() -> String {
    let repo = metadata::pwsh::REPO;
    let arch = if opfs::is_arm() { "arm64" } else { "x64" };
    let version = metadata::pwsh::VERSION;
    format!("{repo}/releases/download/v{version}/PowerShell-{version}-win-{arch}.zip")
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: []
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    pub use_ps5_profile: Option<ProfileName>,
}
#[derive(Deserialize, Debug, Display)]
#[display("{self:?}")]
enum ProfileName {
    AllUsersAllHosts,
    AllUsersCurrentHost,
    CurrentUserAllHosts,
    CurrentUserCurrentHost,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_profile_name() {
        assert_eq!(
            ProfileName::AllUsersCurrentHost.to_string(),
            "AllUsersCurrentHost"
        );
    }
}
