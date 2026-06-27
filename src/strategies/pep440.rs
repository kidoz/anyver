//! PEP 440 parsing and comparison.
//!
//! Builds on the generic comparator for the public release segments but
//! preserves and compares the local-version label (`+LOCAL`) so that
//! `1.0+abc` sorts strictly greater than `1.0` per PEP 440 §"Local version
//! identifiers" — a suffix the generic strategy silently drops.

use crate::parser::{ParsedVersion, Seg, parse_generic, strip_v};
use std::cmp::Ordering;

use super::generic::{cmp_parsed, cmp_segments};

#[derive(Debug, Clone)]
pub(crate) struct Pep440Version {
    public: ParsedVersion,
    /// Parsed local-version segments (PEP 440 `+LOCAL`). `None` when the
    /// version has no local label; this is strictly less than any `Some(_)`.
    local: Option<Vec<Seg>>,
}

impl Pep440Version {
    /// The parsed local-version segments, if the version carries a `+LOCAL`
    /// label. Used by `Version.sort_key()` so the key stays isomorphic with
    /// `cmp_pep440`, which orders `None < Some(_)` and then by `cmp_segments`.
    pub(crate) fn local(&self) -> Option<&[Seg]> {
        self.local.as_deref()
    }
}

/// Tokenize a PEP 440 local version label: digits → `Seg::Num`, letters →
/// lowercased `Seg::Text`, with `.`, `-`, `_` as separators per the spec.
fn tokenize_local(s: &str) -> Vec<Seg> {
    let lower = s.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let len = bytes.len();
    let mut out: Vec<Seg> = Vec::new();
    let mut i = 0;
    while i < len {
        let ch = bytes[i];
        if ch == b'.' || ch == b'-' || ch == b'_' {
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let n: u64 = lower[start..i].parse().unwrap_or(u64::MAX);
            out.push(Seg::Num(n));
            continue;
        }
        if ch.is_ascii_alphabetic() {
            let start = i;
            while i < len && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            out.push(Seg::Text(lower[start..i].to_string()));
            continue;
        }
        i += 1;
    }
    out
}

pub(crate) fn parse_pep440(input: &str) -> Result<Pep440Version, String> {
    validate_pep440(input)?;
    let public = parse_generic(input);
    let trimmed = strip_v(input.trim());
    let after_epoch = match trimmed.find('!') {
        Some(pos) if trimmed[..pos].bytes().all(|b| b.is_ascii_digit()) => &trimmed[pos + 1..],
        _ => trimmed,
    };
    let local = after_epoch.find('+').map(|pos| tokenize_local(&after_epoch[pos + 1..]));
    Ok(Pep440Version { public, local })
}

pub(crate) fn cmp_pep440(a: &Pep440Version, b: &Pep440Version) -> Ordering {
    let ord = cmp_parsed(&a.public, &b.public);
    if ord != Ordering::Equal {
        return ord;
    }
    match (&a.local, &b.local) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(la), Some(lb)) => cmp_segments(la, lb),
    }
}

pub(crate) fn validate_pep440(input: &str) -> Result<(), String> {
    let s = strip_v(input.trim());
    // Extract epoch (N!)
    let after_epoch = if let Some(pos) = s.find('!') {
        let epoch_str = &s[..pos];
        if epoch_str.is_empty() || !epoch_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("invalid PEP 440 epoch: '{epoch_str}'"));
        }
        &s[pos + 1..]
    } else {
        s
    };
    // Strip +local
    let ver_part = match after_epoch.find('+') {
        Some(pos) => &after_epoch[..pos],
        None => after_epoch,
    };
    if ver_part.is_empty() {
        return Err(format!("empty PEP 440 version: '{input}'"));
    }
    if !ver_part.as_bytes()[0].is_ascii_digit() {
        return Err(format!("PEP 440 version must start with a digit: '{input}'"));
    }
    Ok(())
}
