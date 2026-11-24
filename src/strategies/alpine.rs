//! Alpine apk-tools version grammar and token ordering.

use super::SortKey;

#[derive(Debug, Clone)]
pub(crate) struct AlpineVersion {
    pub(crate) key: SortKey,
    pub(crate) is_prerelease: bool,
    pub(crate) is_postrelease: bool,
}

fn take<'a>(rest: &mut &'a str, predicate: impl FnMut(&u8) -> bool) -> &'a str {
    let len = rest.bytes().take_while(predicate).count();
    let (token, tail) = rest.split_at(len);
    *rest = tail;
    token
}

/// A digit run as a sort token: canonical digits ordered by length, then
/// lexically. apk compares numeric tokens this way, so a component wider than
/// `u64` keeps its exact position instead of wrapping around.
fn number(digits: &str) -> (u64, String) {
    let digits = digits.trim_start_matches('0');
    (digits.len() as u64, digits.to_string())
}

pub(super) fn parse_alpine(input: &str) -> Result<AlpineVersion, String> {
    let mut rest = input.trim();
    let error = || format!("invalid Alpine version: '{input}'");
    let first = take(&mut rest, u8::is_ascii_digit);
    if first.is_empty() {
        return Err(error());
    }
    let (len, value) = number(first);
    let mut key = vec![(7, len, value)];
    while let Some(tail) = rest.strip_prefix('.') {
        rest = tail;
        let digits = take(&mut rest, u8::is_ascii_digit);
        if digits.is_empty() {
            return Err(error());
        }
        // apk compares dot components lexically if either starts with zero.
        // Such components always sort below one starting with 1–9.
        let token = if digits.starts_with('0') {
            (6, 0, digits.to_string())
        } else {
            let (len, value) = number(digits);
            (7, len, value)
        };
        key.push(token);
    }
    if let Some(&letter) = rest.as_bytes().first().filter(|b| b.is_ascii_lowercase()) {
        key.push((5, u64::from(letter), String::new()));
        rest = &rest[1..];
    }

    let mut is_prerelease = false;
    let mut is_postrelease = false;
    while let Some(tail) = rest.strip_prefix('_') {
        rest = tail;
        let (rank, suffix) = match take(&mut rest, u8::is_ascii_lowercase) {
            "alpha" => (-1, 0),
            "beta" => (-1, 1),
            "pre" => (-1, 2),
            "rc" => (-1, 3),
            "cvs" => (4, 0),
            "svn" => (4, 1),
            "git" => (4, 2),
            "hg" => (4, 3),
            "p" => (4, 4),
            _ => return Err(error()),
        };
        is_prerelease |= rank < 0;
        is_postrelease |= rank > 0;
        key.push((rank, suffix, String::new()));
        let digits = take(&mut rest, u8::is_ascii_digit);
        if !digits.is_empty() {
            let (len, value) = number(digits);
            key.push((3, len, value));
        }
    }
    if let Some(tail) = rest.strip_prefix('~') {
        rest = tail;
        let hash = take(&mut rest, |b| b.is_ascii_digit() || (b'a'..=b'f').contains(b));
        if hash.is_empty() {
            return Err(error());
        }
        is_postrelease = true;
        key.push((2, 0, hash.to_string()));
    }
    if let Some(tail) = rest.strip_prefix("-r") {
        rest = tail;
        let digits = take(&mut rest, u8::is_ascii_digit);
        if digits.is_empty() {
            return Err(error());
        }
        is_postrelease = true;
        let (len, value) = number(digits);
        key.push((1, len, value));
    }
    if !rest.is_empty() {
        return Err(error());
    }
    key.push((0, 0, String::new()));
    Ok(AlpineVersion { key, is_prerelease, is_postrelease })
}
