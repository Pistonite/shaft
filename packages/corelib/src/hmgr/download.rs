use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use cu::pre::*;
use reqwest::Client;
use reqwest::header::CONTENT_LENGTH;

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
    bar: Option<Arc<cu::ProgressBar>>,
) -> cu::Result<PathBuf> {
    download_file_impl(
        identifier.as_ref(),
        url.as_ref(),
        sha256_checksum.as_ref(),
        bar,
    )
}
#[cu::context("failed to download {} from {}", identifier.display(), url)]
fn download_file_impl(
    identifier: &Path,
    url: &str,
    sha256_checksum: &str,
    bar: Option<Arc<cu::ProgressBar>>,
) -> cu::Result<PathBuf> {
    cu::debug!(
        "looking for download: {} from {}",
        identifier.display(),
        url
    );
    let target_path = hmgr::paths::download(identifier, url);
    let sha256_checksum = sha256_checksum.to_ascii_lowercase();
    if target_path.exists() {
        let bar = cu::progress(format!("checking cached {}", identifier.display()))
            .parent(bar.clone())
            .spawn();
        let actual_checksum = opfs::file_sha256(&target_path, Some(bar.clone()))?;
        if sha256_checksum == actual_checksum {
            cu::progress!(bar, "disk cache");
            bar.done();
            cu::debug!(
                "got file from cache: {} ({})",
                identifier.display(),
                sha256_checksum
            );
            return Ok(target_path);
        }
        bar.done();
    }
    let bar = cu::progress(format!("{}", identifier.display()))
        .parent(bar)
        .spawn();

    for i in 0..5 {
        match i {
            0 => {
                cu::info!("downloading {} from {}", identifier.display(), url);
            }
            x => {
                cu::progress!(bar, "waiting for 5s before retrying...");
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
        let bar_ = bar.clone();
        let result = cu::co::run(async move { do_download_file(path, url, Some(bar_)).await });
        if let Err(e) = result {
            cu::warn!("failed to download {}: {:?}", identifier.display(), e);
            continue;
        }
        let actual_checksum = match opfs::file_sha256(&target_path, Some(bar.clone())) {
            Err(e) => {
                cu::warn!("failed to hash {}: {:?}", identifier.display(), e);
                continue;
            }
            Ok(x) => x,
        };
        if sha256_checksum == actual_checksum {
            cu::info!("downloaded {} ({})", identifier.display(), sha256_checksum);
            bar.done();
            return Ok(target_path);
        }
    }
    cu::bail!(
        "failed to download {}, see error messages above",
        identifier.display()
    );
}

async fn do_download_file(
    path: PathBuf,
    url: String,
    bar: Option<Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    let bar = cu::progress("downloaded")
        .total_bytes(0)
        .eta(true)
        .percentage(true)
        .keep(true)
        .parent(bar)
        .spawn();
    cu::progress!(bar, "{url}");
    let mut response = cu::check!(client()?.get(url).send().await, "failed to send request")?;
    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|x| x.to_str().ok())
        .and_then(|x| x.parse::<u64>().ok());
    if let Some(l) = length {
        bar.set_total(l);
    }
    let mut writer = cu::fs::buf_writer(path)?;

    while let Some(chunk) = cu::check!(response.chunk().await, "failed to read response chunk")? {
        writer.write_all(&chunk)?;
        cu::progress!(bar += chunk.len());
    }
    writer.flush()?;
    Ok(())
}
