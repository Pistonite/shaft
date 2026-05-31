#[cfg(windows)]
pub fn set_clipboard(content: &str) -> cu::Result<()> {
    if let Err(ec) = clipboard_win::set_clipboard(clipboard_win::formats::Unicode, content) {
        cu::bail!("error code: {ec}");
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn set_clipboard(_: &str) -> cu::Result<()> {
    cu::bail!("clipboard is currently not implemented on non-windows");
}
