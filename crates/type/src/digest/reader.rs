// SPDX-FileCopyrightText: 2022 Profian Inc. <opensource@profian.com>
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Algorithm, ContentDigest};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::AsyncRead;
use sha2::digest::DynDigest;

/// A hashing reader
///
/// This type wraps another reader and hashes the bytes as they are read.
#[allow(missing_debug_implementations)] // DynDigest does not implement Debug
pub struct Reader<T> {
    reader: T,
    digests: Vec<(Algorithm, Box<dyn DynDigest>)>,
}

impl<T> Reader<T> {
    pub(crate) fn new(reader: T, digests: impl IntoIterator<Item = Algorithm>) -> Self {
        let digests = digests.into_iter().map(|a| (a, a.hasher())).collect();
        Reader { reader, digests }
    }

    fn update(&mut self, buf: &[u8]) {
        for digest in &mut self.digests {
            digest.1.update(buf);
        }
    }

    /// Calculates the digests for all the bytes written so far.
    pub fn digests(&self) -> ContentDigest<Box<[u8]>> {
        let mut set = ContentDigest::default();

        for digest in &self.digests {
            let _ = set.insert(digest.0, digest.1.clone().finalize().into());
        }

        set
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Reader<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.reader).poll_read(cx, buf).map_ok(|n| {
            self.update(&buf[..n]);
            n
        })
    }
}

impl<T: io::Read> io::Read for Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        self.update(&buf[..n]);
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use futures::io::{copy, sink};

    use super::*;

    #[async_std::test]
    async fn success() {
        const HASH: &str = "sha-256=:LCa0a2j/xo/5m0U8HTBBNBNCLXBkg7+g+YpeiGJm564=:";
        let hash: ContentDigest = HASH.parse().unwrap();

        let mut reader = hash.reader(&b"foo"[..]);
        assert_eq!(copy(&mut reader, &mut sink()).await.unwrap(), 3);
        assert_eq!(reader.digests(), hash);
    }

    #[async_std::test]
    async fn failure() {
        const HASH: &str = "sha-256=:LCa0a2j/xo/5m0U8HTBBNBNCLXBkg7+g+YpeiGJm564=:";
        let hash: ContentDigest = HASH.parse().unwrap();

        let mut reader = hash.reader(&b"bar"[..]);
        assert_eq!(copy(&mut reader, &mut sink()).await.unwrap(), 3);
        assert_ne!(reader.digests(), hash);
    }
}
