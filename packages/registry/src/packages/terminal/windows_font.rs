use crate::pre::*;

version_cache!(static VERSION = metadata::hack_font::VERSION);

pub fn verify() -> cu::Result<Verified> {
    check_version_cache!(VERSION);
    Ok(Verified::UpToDate)
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

fn font_download_url() -> String {
    let repo = metadata::hack_font::REPO;
    let version = metadata::hack_font::VERSION;
    format!("{repo}/releases/download/v{version}/Hack.zip")
}

pub fn install(setting_path: &Path) -> cu::Result<()> {
    cu::info!("installing hack nerd font...");
    if opfs::ensure_terminated("notepad.exe").is_err() {
        let _ = cu::prompt!(
            "detected notepad.exe is currently open - please close it, then press ENTER to continue"
        );
        opfs::ensure_terminated("notepad.exe")?;
    }
    let zip_path = hmgr::paths::download("hack-nerd-font.zip", font_download_url());
    let temp_dir = hmgr::paths::temp_dir("hack-nerd-font");
    opfs::unarchive(&zip_path, &temp_dir, true)?;

    // reset the font to Consolas - if config exists
    if setting_path.exists() {
        let config = cu::check!(
            json::parse::<json::Value>(&cu::fs::read_string(setting_path)?),
            "failed to parse config for windows terminal"
        )?;
        let config = cu::check!(
            jsexe::run(&config, include_str!("./reset_font.js")),
            "failed to reset font for windows terminal"
        )?;
        cu::fs::write_json_pretty(setting_path, &config)?;
    }

    let fonts_folder = {
        let mut dir = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
        dir.extend(["Microsoft", "Windows", "Fonts"]);
        cu::fs::make_dir(&dir)?;
        dir
    };

    let mut font_files = Vec::new();
    for entry in cu::fs::read_dir(&temp_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ttf"))
        {
            let file_name = path.file_name().unwrap();
            let dest = fonts_folder.join(file_name);
            font_files.push((
                path.file_stem()
                    .expect("no file stem")
                    .as_utf8()?
                    .to_string(),
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
            cu::hint!(
                "failed to copy font file - if this is a permission error (file in use), close all terminal processes, and retry"
            );
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

    VERSION.update()?;
    Ok(())
}
