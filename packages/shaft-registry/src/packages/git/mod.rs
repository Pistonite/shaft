
//! Git version control System

use cu::pre::*;
use op::installer::pacman;

use crate::pre::*;

metadata_binaries!("git");

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    if cu::which("git").is_err() {
        return Ok(Verified::NotInstalled);
    }

    match ctx.platform {
        Platform::Arch => {
            // the git package
            let mut pacman = pacman::lock()?;
            if !pacman.is_installed("git")? {
                cu::bail!("current 'git' is not installed with pacman; please uninstall it or use the 'system-git' package");
            }
        }
        Platform::Windows => {
            // portable git
            let expected = op::home::bin("git.exe");
            cu::check!(ctx.check_bin_location("git", &expected),
                "current 'git' is not installed with shaft; please uninstall it or use the 'system-git' package")?;
        }
    }

    version::verify(ctx)
}
pub async fn download(ctx: &Context) -> cu::Result<()> {
    match ctx.platform {
        Platform::Windows => {
            let temp_dir = ctx.temp_dir();
            let download_path = temp_dir.join("git.7z.exe");
            let extract_path = temp_dir.join("extracted");
            let url = version::windows_download_url();
            // download
            op::co_download_to_file(url, &download_path).await?;
            // extract
            download_path.command()
            .add(cu::args!["-o", extract_path, "-y"])
                .all_null()
                .co_wait_nz().await?;
            Ok(())
        }
        Platform::Arch => {
            return Ok(())
        }
    }
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    todo!()
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    todo!()
}

pub mod version;
