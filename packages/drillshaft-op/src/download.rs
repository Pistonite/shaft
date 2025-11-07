use std::io::Write;
use std::path::Path;
use std::sync::{LazyLock, OnceLock};

use cu::pre::*;

use reqwest::Client;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init_http_client() -> cu::Result<()> {
    let client = Client::builder()
        .gzip(true)
        .https_only(true)
        .build()?;
    if CLIENT.set(client).is_err() {
        cu::bail!("http client already initialized");
    }
    Ok(())
}

fn client() -> cu::Result<&'static Client> {
    cu::check!(CLIENT.get(), "http client not intialized")
}

#[inline(always)]
pub async fn co_download_to_file(url: impl AsRef<str>, path: impl AsRef<Path>) -> cu::Result<()> {
    co_download_to_file_impl(url.as_ref(), path.as_ref()).await
}

async fn co_download_to_file_impl(url: &str, path: &Path) -> cu::Result<()> {
    let bar = cu::progress_unbounded_lowp(format!("downloading {url}"));
    let response = cu::co::spawn(client()?
    .get(url)
        .send());
    let mut writer = cu::fs::buf_writer(path)?;
    let mut response = cu::check!(response.co_join().await?, "GET {url} failed")?;

    while let Some(chunk) = cu::check!(response.chunk().await, "GET {url} failed to receive chunk")? {
        cu::check!(writer.write_all(&chunk), "GET {url} failed to save file to '{}'", path.display())?;
    }
    drop(bar);

    cu::check!(writer.flush(), "GET {url} failed to flush to file '{}'", path.display())?;
    Ok(())
}

// async fn co_download_to_memory_impl(url: &str) -> cu::Result<Vec<u8>> {
// }
