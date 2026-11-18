//! `RubyGems` version grammar and `Gem::Version#<=>` ordering.

use std::cmp::Ordering;

use super::SortKey;

const ZERO: (i32, u64, String) = (1, 0, String::new());

#[derive(Debug, Clone)]
pub(crate) struct RubyVersion {
    raw: SortKey,
    canonical: SortKey,
    pub(crate) is_prerelease: bool,
}

impl RubyVersion {
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        // RubyGems checks canonical equality before comparing raw components.
        if self.canonical == other.canonical {
            return Ordering::Equal;
        }
        (0..self.raw.len().max(other.raw.len()))
            .map(|i| self.raw.get(i).unwrap_or(&ZERO).cmp(other.raw.get(i).unwrap_or(&ZERO)))
            .find(|&order| order != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    }

    pub(crate) fn sort_key(&self) -> SortKey {
        // Encode runs of zeros relative to the next nonzero component. This
        // gives a total order while treating omitted components as zeros.
        let mut key = Vec::new();
        let mut zeros = 0;
        for token in &self.canonical {
            if *token == ZERO {
                zeros += 1;
                continue;
            }
            let run = if token.0 == 0 { (0, zeros) } else { (2, u64::MAX - zeros) };
            key.push((run.0, run.1, String::new()));
            key.push(token.clone());
            zeros = 0;
        }
        key.push((1, 0, String::new()));
        key
    }
}

pub(super) fn parse_ruby(input: &str) -> Result<RubyVersion, String> {
    let input = input.trim();
    let (release, pre) = input.split_once('-').map_or((input, None), |(a, b)| (a, Some(b)));
    let valid = |s: &str, allowed: fn(u8) -> bool| {
        s.split('.').all(|part| !part.is_empty() && part.bytes().all(allowed))
    };
    if !valid(release, |b| b.is_ascii_alphanumeric())
        || !release.split('.').next().is_some_and(|part| part.bytes().all(|b| b.is_ascii_digit()))
        || pre.is_some_and(|s| !valid(s, |b| b.is_ascii_alphanumeric() || b == b'-'))
    {
        return Err(format!("invalid RubyGems version: '{input}'"));
    }

    let mut raw = Vec::new();
    let mut rest = input;
    while let Some(first) = rest.bytes().next() {
        if first == b'-' {
            raw.push((0, 0, "pre".to_string()));
            rest = &rest[1..];
        } else if first == b'.' {
            rest = &rest[1..];
        } else {
            let is_digit = first.is_ascii_digit();
            let len = rest
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() && b.is_ascii_digit() == is_digit)
                .count();
            let part = &rest[..len];
            if is_digit {
                let digits = part.trim_start_matches('0');
                raw.push((1, digits.len() as u64, digits.to_string()));
            } else {
                raw.push((0, 0, part.to_string()));
            }
            rest = &rest[len..];
        }
    }

    let prerelease_start = raw.iter().position(|part| part.0 == 0);
    let mut canonical = raw.clone();
    if let Some(start) = prerelease_start {
        let suffix = canonical.split_off(start);
        while canonical.last() == Some(&ZERO) {
            canonical.pop();
        }
        canonical.extend(suffix);
    }
    while canonical.last() == Some(&ZERO) {
        canonical.pop();
    }
    Ok(RubyVersion { raw, canonical, is_prerelease: prerelease_start.is_some() })
}
