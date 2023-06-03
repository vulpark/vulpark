use std::{ops::Deref, str::FromStr};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A string that only contains a subset of characters and <= 32 chars.
///
/// Illegal characters are replaced with '-' at instantiation.
///
/// Inputs > 32 are trimmed to be 32 in length.
#[derive(Debug, Clone)]
pub struct RestrictedString {
    inner: String,
}

impl RestrictedString {
    const ILLEGAL_CHARS: Lazy<Regex> =
        Lazy::new(|| Regex::new("[\\s\u{200b}-\u{200f}\u{2060}]").unwrap());

    pub fn new(value: &str) -> Self {
        Self {
            inner: Self::trim(Self::filter(value)),
        }
    }

    fn filter(value: &str) -> String {
        Self::ILLEGAL_CHARS.replace_all(&value, "-").to_string()
    }

    fn trim(value: String) -> String {
        if value.len() <= 32 {
            return value;
        }
        value.split_at(32).0.to_string()
    }
}

impl From<String> for RestrictedString {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl From<&str> for RestrictedString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Into<String> for RestrictedString {
    fn into(self) -> String {
        self.inner
    }
}

impl Deref for RestrictedString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ToString for RestrictedString {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}

impl FromStr for RestrictedString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Serialize for RestrictedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self)
    }
}

impl<'de> Deserialize<'de> for RestrictedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(&String::deserialize(deserializer)?))
    }
}
