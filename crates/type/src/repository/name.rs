// SPDX-FileCopyrightText: 2022 Profian Inc. <opensource@profian.com>
// SPDX-License-Identifier: AGPL-3.0-only

use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::bail;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

/// A repository name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Name(String);

impl Name {
    #[inline]
    fn validate(s: impl AsRef<str>) -> anyhow::Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            bail!("empty repository name")
        } else if s
            .find(|c| !matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '-'))
            .is_some()
        {
            bail!("invalid characters in repository name")
        } else {
            Ok(())
        }
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<String> for Name {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Deref for Name {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        name.try_into().map_err(D::Error::custom)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Name {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s).map(|()| Self(s.into()))
    }
}

impl TryFrom<String> for Name {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::validate(&s).map(|()| Self(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        assert!("".parse::<Name>().is_err());
        assert!(" ".parse::<Name>().is_err());
        assert!("/".parse::<Name>().is_err());
        assert!("/name".parse::<Name>().is_err());
        assert!("name/".parse::<Name>().is_err());
        assert!("/name/".parse::<Name>().is_err());
        assert!("group//name".parse::<Name>().is_err());
        assert!("group/subgroup///name".parse::<Name>().is_err());
        assert!("group/subg%roup/name".parse::<Name>().is_err());
        assert!("group/subgяoup/name".parse::<Name>().is_err());
        assert!("group /subgroup/name".parse::<Name>().is_err());
        assert!("group/subgr☣up/name".parse::<Name>().is_err());
        assert!("gr.oup/subgroup/name".parse::<Name>().is_err());
        assert!("group/name".parse::<Name>().is_err());
        assert!("group/subgroup/name".parse::<Name>().is_err());
        assert!("gr0uP/subgr0up/-n4mE".parse::<Name>().is_err());

        assert_eq!("name".parse::<Name>().unwrap(), Name("name".into()));
        assert_eq!("-n4M3".parse::<Name>().unwrap(), Name("-n4M3".into()));
    }
}
