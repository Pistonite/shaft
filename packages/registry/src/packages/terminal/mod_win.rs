//! Configuration for Windows Terminal

use crate::pre::*;

static CFG_VERSION: VersionCache =
    VersionCache::new("terminal", metadata::terminal::CONFIG_VERSION);
static FONT_VERSION: VersionCache =
    VersionCache::new("hack-nerd-font", metadata::hack_font::VERSION);

pub fn config_dependencies() -> EnumSet<PkgId> {
    enum_set! { PkgId::Pwsh }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("wt");
    let is_config_uptodate = CFG_VERSION.is_uptodate()?;
    Ok(Verified::is_uptodate(is_config_uptodate))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "hack-nerd-font.zip",
        font_download_url(),
        metadata::hack_font::SHA,
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if cu::which("wt").is_err() {
        cu::info!("installing Microsoft.WindowsTerminal with winget");
        opfs::ensure_terminated("wt.exe")?;
        opfs::ensure_terminated("WindowsTerminal.exe")?;
        epkg::winget::install("Microsoft.WindowsTerminal", ctx.bar_ref())?;
    }
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    cu::info!("installing hack nerd font...");
    cu::check!(configure_font(), "failed to install hack nerd font")?;

    let setting_path = setting_json()?;
    let config = cu::check!(
        json::parse::<json::Value>(&cu::fs::read_string(&setting_path)?),
        "failed to parse config for windows terminal"
    )?;
    let input = json! {
        {
            "config": config,
            "meta": {
            "pwsh_installed": ctx.is_installed(PkgId::Pwsh),
            "install_dir": hmgr::paths::install_dir("pwsh").as_utf8()?,
        }
    }
        };
    let config = cu::check!(
        jsexe::run(&input, include_str!("./config.js")),
        "failed to configure windows terminal"
    )?;
    cu::fs::write_json_pretty(setting_path, &config)?;
    CFG_VERSION.update()?;

    Ok(())
}

fn configure_font() -> cu::Result<()> {
    let zip_path = hmgr::paths::download("hack-nerd-font.zip", font_download_url());
    let temp_dir = hmgr::paths::temp_dir("hack-nerd-font");
    opfs::unarchive(&zip_path, &temp_dir, true)?;

    // reset the font to Consolas
    let setting_path = setting_json()?;
    let config = cu::check!(
        json::parse::<json::Value>(&cu::fs::read_string(&setting_path)?),
        "failed to parse config for windows terminal"
    )?;
    let config = cu::check!(
        jsexe::run(&config, include_str!("./reset_font.js")),
        "failed to reset font for windows terminal"
    )?;
    cu::fs::write_json_pretty(setting_path, &config)?;

    // create fonts folder
    let fonts_folder = {
        let mut dir = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
        dir.extend(["Microsoft", "Windows", "Fonts"]);
        cu::fs::make_dir(&dir)?;
        dir
    };

    // collect font files first
    let mut font_files = Vec::new();
    for entry in cu::fs::read_dir(&temp_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("ttf")) {
            let file_name = path.file_name().unwrap();
            let dest = fonts_folder.join(file_name);
            font_files.push((
                path.file_stem().expect("no file stem").as_utf8()?.to_string(),
                path,
                dest,
            ));
        }
    }

    // delete existing font registry entries
    let del_commands: Vec<String> = font_files
        .iter()
        .map(|(name, _, _)| {
            format!(
                r#"Remove-ItemProperty -Path 'HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts' -Name '{name} (TrueType)' -ErrorAction SilentlyContinue"#
            )
        })
        .collect();
    let script = del_commands.join("\n");
    let status = cu::which("powershell")?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait()?;
    if !status.success() {
        cu::warn!("powershell returned {status}, when removing font entries from registry");
    }

    // copy all *.ttf files to fonts folder
    for (_, path, dest) in &font_files {
        if let Err(e) = cu::fs::copy(path, dest) {
            cu::hint!("failed to copy font file - if this is a permission error, close all terminal processes, and retry");
            cu::rethrow!(e);
        }
    }

    // register fonts in registry using powershell
    let reg_commands: Vec<String> = font_files
        .iter()
        .map(|(name, _, dest)| {
            let dest = dest.as_utf8().expect("invalid utf8 path");
            format!(
                r#"Set-ItemProperty -Path 'HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts' -Name '{name} (TrueType)' -Value '{dest}'"#
            )
        })
        .collect();
    let script = reg_commands.join("\n");
    cu::which("powershell")?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;

    FONT_VERSION.update()?;
    Ok(())
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("uninstalling windows terminal is not supported");
}
pub use pre_uninstall as uninstall;

fn setting_json() -> cu::Result<PathBuf> {
    let mut p = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
    p.extend([
        "Packages",
        "Microsoft.WindowsTerminal_8wekyb3d8bbwe",
        "LocalState",
        "settings.json",
    ]);
    Ok(p)
}

fn font_download_url() -> String {
    let repo = metadata::hack_font::REPO;
    let version = metadata::hack_font::VERSION;
    format!("{repo}/releases/download/v{version}/Hack.zip")
}
