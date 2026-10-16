//! RPM version parsing and the `rpmvercmp` comparison algorithm, including
//! the `~` (sorts before everything) and `^` (sorts after everything) markers.

use std::cmp::Ordering;

use super::debian::extract_epoch_colon;

#[derive(Debug, Clone)]
pub(crate) struct RpmVersion {
    epoch: u64,
    version: String,
    release: String,
}

impl RpmVersion {
    pub(crate) fn hash_key(&self) -> (u64, Vec<String>, Vec<String>) {
        fn tokens(s: &str) -> Vec<String> {
            let mut rest = s.as_bytes();
            let mut out = Vec::new();
            while let Some(&first) = rest.first() {
                if matches!(first, b'~' | b'^') {
                    out.push(char::from(first).to_string());
                    rest = &rest[1..];
                } else if first.is_ascii_alphanumeric() {
                    let numeric = first.is_ascii_digit();
                    let len = rest
                        .iter()
                        .take_while(|b| {
                            if numeric { b.is_ascii_digit() } else { b.is_ascii_alphabetic() }
                        })
                        .count();
                    let text = String::from_utf8_lossy(&rest[..len]);
                    out.push(if numeric {
                        format!("n{}", text.trim_start_matches('0'))
                    } else {
                        format!("s{text}")
                    });
                    rest = &rest[len..];
                } else {
                    rest = &rest[1..];
                }
            }
            out
        }
        (self.epoch, tokens(&self.version), tokens(&self.release))
    }
}

pub(crate) fn parse_rpm(input: &str) -> Result<RpmVersion, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty RPM version".to_string());
    }
    let (epoch, rest) = extract_epoch_colon(s);
    // Split version and release at last '-'
    let (version, release) = match rest.rfind('-') {
        Some(pos) => (rest[..pos].to_string(), rest[pos + 1..].to_string()),
        None => (rest.to_string(), String::new()),
    };
    Ok(RpmVersion { epoch, version, release })
}

pub(crate) fn rpmverscmp(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut ia = 0usize;
    let mut ib = 0usize;

    while ia < ab.len() || ib < bb.len() {
        // Skip leading non-alphanumeric (except ~ and ^)
        while ia < ab.len() && !ab[ia].is_ascii_alphanumeric() && ab[ia] != b'~' && ab[ia] != b'^' {
            ia += 1;
        }
        while ib < bb.len() && !bb[ib].is_ascii_alphanumeric() && bb[ib] != b'~' && bb[ib] != b'^' {
            ib += 1;
        }

        // Handle ~ (sorts before everything including end)
        if (ia < ab.len() && ab[ia] == b'~') || (ib < bb.len() && bb[ib] == b'~') {
            if ia >= ab.len() || ab[ia] != b'~' {
                return Ordering::Greater;
            }
            if ib >= bb.len() || bb[ib] != b'~' {
                return Ordering::Less;
            }
            ia += 1;
            ib += 1;
            continue;
        }

        // Handle ^ (sorts after everything else, opposite of ~)
        if (ia < ab.len() && ab[ia] == b'^') || (ib < bb.len() && bb[ib] == b'^') {
            if ia >= ab.len() {
                return Ordering::Less; // a at end, b has ^ → a < b
            }
            if ib >= bb.len() {
                return Ordering::Greater; // b at end, a has ^ → a > b
            }
            if ab[ia] != b'^' {
                return Ordering::Greater; // a not ^, b is ^ → a > b
            }
            if bb[ib] != b'^' {
                return Ordering::Less; // a is ^, b not ^ → a < b
            }
            ia += 1;
            ib += 1;
            continue;
        }

        // End of one string
        if ia >= ab.len() || ib >= bb.len() {
            break;
        }

        let is_num = ab[ia].is_ascii_digit();

        let sa = ia;
        let sb = ib;

        if is_num {
            while ia < ab.len() && ab[ia].is_ascii_digit() {
                ia += 1;
            }
            while ib < bb.len() && bb[ib].is_ascii_digit() {
                ib += 1;
            }
        } else {
            while ia < ab.len() && ab[ia].is_ascii_alphabetic() {
                ia += 1;
            }
            while ib < bb.len() && bb[ib].is_ascii_alphabetic() {
                ib += 1;
            }
        }

        // If b run is empty, type mismatch — digits always win
        if sb == ib {
            return if is_num { Ordering::Greater } else { Ordering::Less };
        }

        if is_num {
            // Skip leading zeros
            let mut za = sa;
            let mut zb = sb;
            while za < ia && ab[za] == b'0' {
                za += 1;
            }
            while zb < ib && bb[zb] == b'0' {
                zb += 1;
            }
            // Longer number is greater
            let len_a = ia - za;
            let len_b = ib - zb;
            if len_a != len_b {
                return len_a.cmp(&len_b);
            }
            // Same length — lexical comparison
            let cmp = ab[za..ia].cmp(&bb[zb..ib]);
            if cmp != Ordering::Equal {
                return cmp;
            }
        } else {
            // Alpha run — lexical comparison
            let cmp = ab[sa..ia].cmp(&bb[sb..ib]);
            if cmp != Ordering::Equal {
                return cmp;
            }
        }
    }

    // Whichever still has characters remaining is newer
    if ia >= ab.len() && ib >= bb.len() {
        Ordering::Equal
    } else if ia < ab.len() {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

pub(crate) fn cmp_rpm(a: &RpmVersion, b: &RpmVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| rpmverscmp(&a.version, &b.version))
        .then_with(|| rpmverscmp(&a.release, &b.release))
}
