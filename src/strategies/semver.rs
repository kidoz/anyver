//! Strict `SemVer` parsing and comparison (also used by Go, crates.io, Hex,
//! Swift, and npm, which share `SemVer`'s grammar and ordering rules).

use super::SortKey;
use crate::parser::strip_v;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SemVer {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
    pub(crate) pre: Option<String>,
}

impl SemVer {
    pub(crate) fn release(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch, pre: None }
    }

    pub(crate) fn core(&self) -> (u64, u64, u64) {
        (self.major, self.minor, self.patch)
    }

    pub(crate) fn next_release(&self) -> Option<Self> {
        if let Some(patch) = self.patch.checked_add(1) {
            return Some(Self::release(self.major, self.minor, patch));
        }
        if let Some(minor) = self.minor.checked_add(1) {
            return Some(Self::release(self.major, minor, 0));
        }
        Some(Self::release(self.major.checked_add(1)?, 0, 0))
    }

    pub(crate) fn sort_key(&self) -> SortKey {
        let mut key = vec![
            (2, self.major, String::new()),
            (2, self.minor, String::new()),
            (2, self.patch, String::new()),
            (i32::from(self.pre.is_none()), 0, String::new()),
        ];
        if let Some(pre) = &self.pre {
            for id in pre.split('.') {
                if let Ok(number) = id.parse::<u64>() {
                    key.push((0, number, String::new()));
                } else if is_numeric_identifier(id) {
                    // A marker above the largest u64 preserves the existing
                    // key encoding for versions that already fit in u64.
                    key.push((0, u64::MAX, "+".to_string()));
                    key.push((0, id.len() as u64, id.to_string()));
                } else {
                    key.push((1, 0, id.to_string()));
                }
            }
        }
        key
    }
}

fn is_numeric_identifier(s: &str) -> bool {
    s.bytes().all(|b| b.is_ascii_digit())
}

pub(crate) fn parse_strict_u64(s: &str, label: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err(format!("empty {label}"));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(format!("leading zero in {label}: '{s}'"));
    }
    s.parse::<u64>().map_err(|_| format!("invalid {label}: '{s}'"))
}

pub(crate) fn validate_prerelease(pre: &str) -> Result<(), String> {
    if pre.is_empty() {
        return Err("empty prerelease".to_string());
    }
    for ident in pre.split('.') {
        if ident.is_empty() {
            return Err("empty identifier in prerelease".to_string());
        }
        if !ident.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(format!("invalid character in prerelease identifier: '{ident}'"));
        }
        let is_numeric = ident.bytes().all(|b| b.is_ascii_digit());
        if is_numeric && ident.len() > 1 && ident.starts_with('0') {
            return Err(format!("leading zero in numeric prerelease identifier: '{ident}'"));
        }
        if is_numeric && ident.parse::<u64>().is_err() {
            return Err(format!("numeric prerelease identifier out of range for u64: '{ident}'"));
        }
    }
    Ok(())
}

pub(super) fn validate_build(build: &str) -> Result<(), String> {
    if build.split('.').any(|ident| {
        ident.is_empty() || !ident.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    }) {
        return Err(format!("invalid build metadata: '{build}'"));
    }
    Ok(())
}

pub(crate) fn parse_semver_strict(input: &str) -> Result<SemVer, String> {
    let s = strip_v(input.trim());
    let s = match s.find('+') {
        Some(p) => {
            validate_build(&s[p + 1..])?;
            &s[..p]
        }
        None => s,
    };
    let (core, pre) = match s.find('-') {
        Some(p) => (&s[..p], Some(s[p + 1..].to_string())),
        None => (s, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Expected 3 parts, got {}", parts.len()));
    }
    let major = parse_strict_u64(parts[0], "major")?;
    let minor = parse_strict_u64(parts[1], "minor")?;
    let patch = parse_strict_u64(parts[2], "patch")?;
    if let Some(ref p) = pre {
        validate_prerelease(p)?;
    }
    Ok(SemVer { major, minor, patch, pre })
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn cmp_semver_strict(left: &SemVer, right: &SemVer) -> Ordering {
    let core = left
        .major
        .cmp(&right.major)
        .then(left.minor.cmp(&right.minor))
        .then(left.patch.cmp(&right.patch));
    if core != Ordering::Equal {
        return core;
    }
    match (&left.pre, &right.pre) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(lp), Some(rp)) => {
            let left_parts: Vec<&str> = lp.split('.').collect();
            let right_parts: Vec<&str> = rp.split('.').collect();
            let count = left_parts.len().max(right_parts.len());
            for i in 0..count {
                match (left_parts.get(i), right_parts.get(i)) {
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (None, None) => return Ordering::Equal,
                    (Some(lv), Some(rv)) => {
                        let ord = match (lv.parse::<u64>(), rv.parse::<u64>()) {
                            (Ok(ln), Ok(rn)) => ln.cmp(&rn),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => lv.cmp(rv),
                        };
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    }
                }
            }
            Ordering::Equal
        }
    }
}
