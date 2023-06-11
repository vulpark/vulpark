// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use regex::Regex;

/// A string that only contains a subset of characters and <= 32 chars.
///
/// Illegal characters are replaced with '-' or ' '.
///
/// Inputs > 32 are trimmed to be 32 in length.
#[derive(Debug, Clone, Copy)]
pub struct RestrictedString;

impl RestrictedString {
    pub fn space(value: &str) -> String {
        Self::trim(Self::filter(value, " "))
    }

    pub fn no_space(value: &str) -> String {
        Self::trim(Self::filter(value, "-"))
    }

    fn filter(value: &str, repl: &str) -> String {
        Regex::new("[\\s\u{200b}-\u{200f}\u{2060}]")
            .unwrap()
            .replace_all(value, repl)
            .to_string()
    }

    fn trim(value: String) -> String {
        if value.len() <= 32 {
            return value;
        }
        value.split_at(32).0.to_string()
    }
}
