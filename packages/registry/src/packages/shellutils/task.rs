use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    check_in_shaft!("task");
    check_in_shaft!("x");
    let v = command_output!("task", ["--version"]);
    check_outdated!(v.trim(), metadata[task]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        task_filename(),
        task_url()?,
        metadata::task::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;

    let task_archive = hmgr::paths::download(task_filename(), task_url()?);
    let task_temp = hmgr::paths::temp_dir("task-unarchive");
    opfs::unarchive(task_archive, &task_temp, true)?;
    cu::fs::copy(
        task_temp.join(bin_name!("task")),
        install_dir.join(bin_name!("task")),
    )?;

    Ok(())
}

fn task_filename() -> &'static str {
    if cfg!(windows) {
        "task.zip"
    } else {
        "task.tgz"
    }
}

fn task_url() -> cu::Result<String> {
    let artifact = if cfg!(windows) {
        let arch = if opfs::is_arm() { "arm64" } else { "amd64" };
        format!("windows_{arch}.zip")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux_amd64.tar.gz".to_string()
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "darwin_arm64.tar.gz".to_string()
    } else {
        cu::bail!("task is not supported on current OS/Architecture");
    };
    let repo = metadata::task::REPO;
    let ver = metadata::task::VERSION;
    Ok(format!("{repo}/releases/download/v{ver}/task_{artifact}"))
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let task_exe = ctx.install_dir().join(bin_name!("task")).into_utf8()?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("task")).into_utf8()?,
        task_exe.clone(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("x")).into_utf8()?,
        task_exe.clone(),
    ))?;
    let mut script = command_output!(&task_exe, ["--completion", "bash"]);
    script.push_str("\ncomplete -F _task x");
    ctx.add_item(Item::bash(script))?;
    let mut script = "compdef _task x\n".to_string();
    script.push_str(&command_output!(&task_exe, ["--completion", "zsh"]));
    ctx.add_item(Item::zsh(script))?;
    let script = r#"Invoke-Expression (& {((task --completion powershell).replace("-CommandName task","-CommandName task,x") | Out-String)})"#;
    ctx.add_item(Item::pwsh(script))?;
    Ok(())
}
