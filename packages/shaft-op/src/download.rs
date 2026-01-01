use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;

use cu::pre::*;
use reqwest::Client;

use crate::util;

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init_http_client() -> cu::Result<()> {
    let client = Client::builder().gzip(true).https_only(true).build()?;
    if CLIENT.set(client).is_err() {
        cu::bail!("http client already initialized");
    }
    Ok(())
}

fn client() -> cu::Result<&'static Client> {
    cu::check!(CLIENT.get(), "http client not intialized")
}

#[inline(always)]
pub async fn co_download_to_file(
    url: impl AsRef<str>,
    path: impl AsRef<Path>,
    sha256: &str,
) -> cu::Result<()> {
    co_download_to_file_impl(url.as_ref(), path.as_ref(), sha256).await
}
async fn co_download_to_file_impl(url: &str, path: &Path, sha256: &str) -> cu::Result<()> {
    cu::check!(
        co_download_to_file_inner(url, path, sha256).await,
        "failed to download '{url}' to {}",
        path.display()
    )
}

async fn co_download_to_file_inner(url: &str, path: &Path, sha256: &str) -> cu::Result<()> {
    // if the file already exists and hash matches, skip download
    if let Ok(hash) = util::co_sha256(path).await
        && hash == sha256
    {
        cu::debug!("hash matched for '{url}'");
        return Ok(());
    }

    let bar = cu::progress_unbounded_lowp(format!("downloading {url}"));

    let response = cu::co::spawn(client()?.get(url).send());
    let mut writer = cu::fs::buf_writer(path)?;
    let mut response = response.co_join().await??;

    while let Some(chunk) = response.chunk().await? {
        writer.write_all(&chunk)?;
    }
    drop(bar);

    writer.flush()?;
    let hash = util::co_sha256(path).await?;
    cu::ensure!(
        hash == sha256,
        "downloaded file did not match expected hash"
    );

    Ok(())
}

// async fn co_download_to_memory_impl(url: &str) -> cu::Result<Vec<u8>> {
// }
