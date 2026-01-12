use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use cu::pre::*;
use reqwest::Client;

use crate::{hmgr, opfs};

static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    Client::builder()
        .gzip(true)
        .https_only(true)
        .build()
        .map_err(|x| format!("{x}"))
});

fn client() -> cu::Result<&'static Client> {
    let client: &Result<Client, String> = &CLIENT;
    match client {
        Ok(c) => return Ok(c),
        Err(e) => {
            cu::bail!("error initializing https client: {e}")
        }
    }
}

/// Download URL to a temporary location, return the path to the downloaded file.
///
/// The SHA256 checksum is used to verify integrity.
///
/// The result is cached across multiple runs
#[inline(always)]
pub fn download_file(
    identifier: impl AsRef<Path>,
    url: impl AsRef<str>,
    sha256_checksum: impl AsRef<str>,
) -> cu::Result<PathBuf> {
    download_file_impl(identifier.as_ref(), url.as_ref(), sha256_checksum.as_ref())
}
#[cu::context("failed to download {} from {}", identifier.display(), url)]
fn download_file_impl(identifier: &Path, url: &str, sha256_checksum: &str) -> cu::Result<PathBuf> {
    cu::debug!(
        "looking for download: {} from {}",
        identifier.display(),
        url
    );
    let target_path = hmgr::paths::download(identifier, url);
    let sha256_checksum = sha256_checksum.to_ascii_lowercase();
    if target_path.exists() {
        let actual_checksum = opfs::file_sha256(&target_path)?;
        if sha256_checksum == actual_checksum {
            cu::info!(
                "got file from cache: {} ({})",
                identifier.display(),
                sha256_checksum
            );
            return Ok(target_path);
        }
    }

    for i in 0..5 {
        match i {
            0 => cu::info!("downloading {} from {}", identifier.display(), url),
            x => {
                cu::warn!("waiting for 5s before retrying...");
                std::thread::sleep(Duration::from_secs(5));
                cu::info!(
                    "downloading {} from {} (retry #{})",
                    identifier.display(),
                    url,
                    x
                )
            }
        }
        let path = target_path.clone();
        let url = url.to_string();
        let result = cu::co::run(async move { do_download_file(path, url).await });
        if let Err(e) = result {
            cu::warn!("failed to download {}: {:?}", identifier.display(), e);
            continue;
        }
        let actual_checksum = match opfs::file_sha256(&target_path) {
            Err(e) => {
                cu::warn!("failed to hash {}: {:?}", identifier.display(), e);
                continue;
            }
            Ok(x) => x,
        };
        if sha256_checksum == actual_checksum {
            cu::info!("downloaded {} ({})", identifier.display(), sha256_checksum);
            return Ok(target_path);
        }
    }
    cu::bail!(
        "failed to download {}, see error messages above",
        identifier.display()
    );
}

async fn do_download_file(path: PathBuf, url: String) -> cu::Result<()> {
    let mut response = cu::check!(client()?.get(url).send().await, "failed to send request")?;
    let mut writer = cu::fs::buf_writer(path)?;

    while let Some(chunk) = cu::check!(response.chunk().await, "failed to read response chunk")? {
        writer.write_all(&chunk)?;
    }
    writer.flush()?;
    Ok(())
}
