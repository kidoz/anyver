//! Debian version parsing and the dpkg `verrevcmp` comparison algorithm.
//!
//! Also hosts [`extract_epoch_colon`], the shared `EPOCH:rest` splitter that
//! RPM reuses (both ecosystems use the same colon-epoch syntax).

use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub(crate) struct DebianVersion {
    epoch: u64,
    upstream: String,
    revision: String,
}

pub(crate) fn extract_epoch_colon(s: &str) -> (u64, &str) {
    for (i, ch) in s.char_indices() {
        if ch == ':' {
            if let Ok(e) = s[..i].parse::<u64>() {
                return (e, &s[i + 1..]);
            }
            break;
        }
        if !ch.is_ascii_digit() {
            break;
        }
    }
    (0, s)
}

pub(crate) fn parse_debian(input: &str) -> Result<DebianVersion, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Debian version".to_string());
    }
    let (epoch, rest) = extract_epoch_colon(s);
    // Split upstream and revision at last '-'
    let (upstream, revision) = match rest.rfind('-') {
        Some(pos) => (rest[..pos].to_string(), rest[pos + 1..].to_string()),
        None => (rest.to_string(), String::new()),
    };
    Ok(DebianVersion { epoch, upstream, revision })
}

pub(crate) fn dpkg_order(c: Option<u8>) -> i32 {
    match c {
        None => 0,
        Some(b'~') => -1,
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}

pub(crate) fn verrevcmp(a: &[u8], b: &[u8]) -> Ordering {
    let mut ia = 0usize;
    let mut ib = 0usize;
    while ia < a.len() || ib < b.len() {
        // Compare non-digit parts
        loop {
            let a_nondigit = ia < a.len() && !a[ia].is_ascii_digit();
            let b_nondigit = ib < b.len() && !b[ib].is_ascii_digit();
            if !a_nondigit && !b_nondigit {
                break;
            }
            let ac = dpkg_order(if ia < a.len() { Some(a[ia]) } else { None });
            let bc = dpkg_order(if ib < b.len() { Some(b[ib]) } else { None });
            if ac != bc {
                return ac.cmp(&bc);
            }
            if ia < a.len() {
                ia += 1;
            }
            if ib < b.len() {
                ib += 1;
            }
            if ia >= a.len() && ib >= b.len() {
                break;
            }
        }
        // Skip leading zeros
        while ia < a.len() && a[ia] == b'0' {
            ia += 1;
        }
        while ib < b.len() && b[ib] == b'0' {
            ib += 1;
        }
        // Compare digit runs
        let mut first_diff = 0i32;
        while ia < a.len() && a[ia].is_ascii_digit() && ib < b.len() && b[ib].is_ascii_digit() {
            if first_diff == 0 {
                first_diff = i32::from(a[ia]) - i32::from(b[ib]);
            }
            ia += 1;
            ib += 1;
        }
        if ia < a.len() && a[ia].is_ascii_digit() {
            return Ordering::Greater;
        }
        if ib < b.len() && b[ib].is_ascii_digit() {
            return Ordering::Less;
        }
        if first_diff != 0 {
            return if first_diff > 0 { Ordering::Greater } else { Ordering::Less };
        }
    }
    Ordering::Equal
}

pub(crate) fn cmp_debian(a: &DebianVersion, b: &DebianVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| verrevcmp(a.upstream.as_bytes(), b.upstream.as_bytes()))
        .then_with(|| verrevcmp(a.revision.as_bytes(), b.revision.as_bytes()))
}
