//! Debian version parsing and the dpkg `verrevcmp` comparison algorithm.
//!
//! Also hosts [`extract_epoch_colon`], the shared `EPOCH:rest` splitter that
//! RPM reuses (both ecosystems use the same colon-epoch syntax).

use std::cmp::Ordering;

use super::SortKey;

#[derive(Debug, Clone)]
pub(crate) struct DebianVersion {
    epoch: u64,
    upstream: String,
    revision: String,
}

impl DebianVersion {
    /// Encode dpkg's alternating byte and arbitrary-precision digit ordering.
    pub(crate) fn sort_key(&self) -> SortKey {
        let mut key = vec![(0, self.epoch, String::new())];
        encode_part(&self.upstream, &mut key);
        encode_part(&self.revision, &mut key);
        key
    }

    pub(crate) fn hash_key(&self) -> (u64, String, String) {
        fn canonical(s: &str) -> String {
            let mut out = String::new();
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    let mut digits = String::from(c);
                    while chars.peek().is_some_and(char::is_ascii_digit) {
                        if let Some(c) = chars.next() {
                            digits.push(c);
                        }
                    }
                    let digits = digits.trim_start_matches('0');
                    if !digits.is_empty() {
                        out.push('#');
                        out.push_str(digits);
                        out.push('#');
                    }
                } else {
                    out.push(c);
                }
            }
            out
        }
        (self.epoch, canonical(&self.upstream), canonical(&self.revision))
    }
}

fn encode_part(part: &str, key: &mut SortKey) {
    let mut rest = part.as_bytes();
    while let Some(&first) = rest.first() {
        if first.is_ascii_digit() {
            let len = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            let digits = &part[part.len() - rest.len()..part.len() - rest.len() + len];
            let digits = digits.trim_start_matches('0');
            rest = &rest[len..];
            // A final zero run equals an absent run. Internal zero runs still
            // separate non-digit phases ("a0b" sorts below "ab").
            if !digits.is_empty() || !rest.is_empty() {
                key.push((0, digits.len() as u64, digits.to_string()));
            }
        } else {
            key.push((dpkg_order(Some(first)), 0, String::new()));
            rest = &rest[1..];
        }
    }
    // The first end token can tie an internal zero run; the second then
    // compares as dpkg's missing byte against the following non-digit byte.
    // This resolves the upstream comparison before revision tokens are read.
    key.extend([(0, 0, String::new()), (0, 0, String::new())]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_key_matches_comparator_all_pairs() {
        let mut inputs = vec!["0:1.0-00".to_string(), "1:0.1-1".to_string()];
        for prefix in ["1", "1.0", "1a", "1~", "1-0"] {
            for suffix in [
                "",
                "0",
                "00",
                "01",
                "2",
                "10",
                "a",
                "0a",
                "~",
                "0~",
                "~0",
                "+",
                ".",
                "0.0~",
                "-1",
                "é",
                "99999999999999999999999999",
            ] {
                inputs.push(format!("{prefix}{suffix}"));
            }
        }
        let parsed: Vec<_> = inputs.iter().map(|s| parse_debian(s).unwrap()).collect();
        for (i, a) in parsed.iter().enumerate() {
            for (j, b) in parsed.iter().enumerate() {
                assert_eq!(
                    a.sort_key().cmp(&b.sort_key()),
                    cmp_debian(a, b),
                    "{} versus {}",
                    inputs[i],
                    inputs[j]
                );
            }
        }
    }
}
