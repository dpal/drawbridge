// SPDX-FileCopyrightText: 2022 Profian Inc. <opensource@profian.com>
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Algorithm, ContentDigest, Reader, Writer};

use std::collections::BTreeSet;
use std::ops::{Deref, DerefMut};

use futures::io::{self, copy, sink, AsyncRead};
use serde::{Deserialize, Serialize};

/// A set of hashing algorithms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Algorithms(BTreeSet<Algorithm>);

impl Default for Algorithms {
    fn default() -> Self {
        let mut set = BTreeSet::new();
        assert!(set.insert(Algorithm::Sha224));
        assert!(set.insert(Algorithm::Sha256));
        assert!(set.insert(Algorithm::Sha384));
        assert!(set.insert(Algorithm::Sha512));
        Self(set)
    }
}

impl From<BTreeSet<Algorithm>> for Algorithms {
    fn from(value: BTreeSet<Algorithm>) -> Self {
        Self(value)
    }
}

impl Deref for Algorithms {
    type Target = BTreeSet<Algorithm>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Algorithms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Algorithms {
    /// Creates a reader instance
    pub fn reader<T>(&self, reader: T) -> Reader<T> {
        Reader::new(reader, self.iter().cloned())
    }

    /// Creates a writer instance
    pub fn writer<T>(&self, writer: T) -> Writer<T> {
        Writer::new(writer, self.iter().cloned())
    }

    /// Calculates a digest from an async reader
    pub async fn read(&self, reader: impl Unpin + AsyncRead) -> io::Result<(u64, ContentDigest)> {
        let mut r = self.reader(reader);
        let n = copy(&mut r, &mut sink()).await?;
        Ok((n, r.digests()))
    }

    /// Calculates a digest from a sync reader
    pub fn read_sync(&self, reader: impl std::io::Read) -> io::Result<(u64, ContentDigest)> {
        let mut r = self.reader(reader);
        let n = std::io::copy(&mut r, &mut std::io::sink())?;
        Ok((n, r.digests()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn digest() {
        let algorithms = Algorithms::default();
        let rdr = &b"foo"[..];
        let content_digest = "sha-224=:CAj2TmDViXn8tnbJbsk4Jw3qQkRa7vzTpOb42w==:,sha-256=:LCa0a2j/xo/5m0U8HTBBNBNCLXBkg7+g+YpeiGJm564=:,sha-384=:mMEf/f3VQGdrGhN8saIrKnA1DJpEFx1rEYDGvly7LuP3nVMsih3Z7y6OCOdSo7q7:,sha-512=:9/u6bgY2+JDlb7vzKD5STG+jIErimDgtYkdB0NxmODJuKCxBvl5CVNiCB3LFUYosWowMf37aGVlKfrU5RT4e1w==:"
                .parse::<ContentDigest>()
                .unwrap();
        assert_eq!(
            algorithms.read(rdr).await.unwrap(),
            ("foo".len() as _, content_digest.clone())
        );
        assert_eq!(
            algorithms.read_sync(rdr).unwrap(),
            ("foo".len() as _, content_digest)
        );
    }
}
