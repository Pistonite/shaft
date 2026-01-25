//! Configuration for Windows Terminal

use crate::pre::*;

static INTERNAL_VERSION: &str = "6";
static FONT_VERSION: &str = "v3.4.0";

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::_7z }
}

pub fn config_dependencies() -> EnumSet<PkgId> {
    enum_set! { PkgId::Pwsh }
}

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("wt");
    let id = ctx.pkg.to_str();
    let version = hmgr::get_cached_version(id)?;
    Ok(Verified::is_uptodate(version.as_deref() == Some(INTERNAL_VERSION)))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("hack-nerd-font.zip", font_download_url(), "8ca33a60c791392d872b80d26c42f2bfa914a480f9eb2d7516d9f84373c36897", ctx.bar())?;
    Ok(())
}

pub fn install(_: &Context) -> cu::Result<()> {
    if cu::which("wt").is_err() {
        cu::info!("installing Microsoft.WindowsTerminal with winget");
        opfs::ensure_terminated("wt.exe")?;
        opfs::ensure_terminated("WindowsTerminal.exe")?;
        epkg::winget::install("Microsoft.WindowsTerminal")?;
    }
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let font_version = hmgr::get_cached_version("hack-nerd-font")?;
    if font_version.as_deref() != Some(FONT_VERSION) {
        cu::info!("installing hack nerd font...");
        let zip_path = hmgr::paths::download("hack-nerd-font.zip", font_download_url());
        let temp_dir = hmgr::paths::temp_dir("hack-nerd-font");
        opfs::un7z(&zip_path, &temp_dir)?;

        // install all *.ttf files in temp_dir for current user
        let script = format!(
            r#"$fontsFolder = "$env:LOCALAPPDATA\Microsoft\Windows\Fonts"
if (-not (Test-Path $fontsFolder)) {{ New-Item -ItemType Directory -Path $fontsFolder -Force | Out-Null }}
$fontReg = "HKCU:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"
$files = Get-ChildItem {} -Filter "*.ttf"
foreach ($file in $files) {{
    $dest = Join-Path $fontsFolder $file.Name
    Copy-Item $file.FullName -Destination $dest -Force
    $fontName = [System.IO.Path]::GetFileNameWithoutExtension($file.Name) + " (TrueType)"
    Set-ItemProperty -Path $fontReg -Name $fontName -Value $dest
}}"#,
            opfs::quote_path(&temp_dir)?
        );
        cu::which("powershell")?
            .command()
            .args(["-NoLogo", "-NoProfile", "-c", &script])
            .stdout(cu::lv::D)
            .stderr(cu::lv::E)
            .stdin_null()
            .wait_nz()?;

        hmgr::set_cached_version("hack-nerd-font", FONT_VERSION)?;
    }

    let setting_path = setting_json()?;
    let config = cu::check!(json::parse::<json::Value>(&cu::fs::read_string(&setting_path)?), "failed to parse config for windows terminal")?;
    let input = json! {
        {
            "config": config,
            "meta": {
                "pwsh_installed": ctx.is_installed(PkgId::Pwsh),
                "install_dir": hmgr::paths::install_dir("pwsh").as_utf8()?,
            }
        }
    };
    let config = cu::check!(jsexe::run(&input, include_str!("./config.js")), "failed to configure windows terminal")?;
    cu::fs::write_json_pretty(setting_path, &config)?;
    hmgr::set_cached_version(ctx.pkg.to_str(), INTERNAL_VERSION)?;

    Ok(())
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("uninstalling windows terminal is not supported");
}
pub use pre_uninstall as uninstall;

fn setting_json() -> cu::Result<PathBuf> {
    let mut p = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
    p.extend(["Packages", "Microsoft.WindowsTerminal_8wekyb3d8bbwe", "LocalState","settings.json"]);
    Ok(p)
}

fn font_download_url() -> String {
    format!("https://github.com/ryanoasis/nerd-fonts/releases/download/{FONT_VERSION}/Hack.zip")
}
