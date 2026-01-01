use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use sha2::{Digest, Sha256};

pub async fn co_sha256(path: &Path) -> cu::Result<String> {
    let mut reader = cu::fs::co_reader(path).await?;
    let mut hasher = AsyncSha256(Sha256::new());
    cu::co_copy(&mut reader, &mut hasher).await?;
    let hash = hasher.0.finalize();
    Ok(format!("{hash:x}"))
}

/// Async adapter for hasher
struct AsyncSha256(Sha256);
impl tokio::io::AsyncWrite for AsyncSha256 {
    #[inline(always)]
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        use std::io::Write;
        Poll::Ready(self.get_mut().0.write(buf))
    }
    #[inline(always)]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        use std::io::Write;
        Poll::Ready(self.get_mut().0.flush())
    }
    #[inline(always)]
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }
}
