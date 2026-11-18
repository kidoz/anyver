//! Perl decimal and dotted versions.

use super::SortKey;

#[derive(Debug, Clone)]
pub(crate) struct CpanVersion {
    components: Vec<u64>,
    pub(crate) is_alpha: bool,
}

impl CpanVersion {
    pub(crate) fn components(&self) -> &[u64] {
        &self.components
    }

    pub(crate) fn sort_key(&self) -> SortKey {
        self.components.iter().map(|&n| (0, n, String::new())).collect()
    }
}

pub(super) fn parse_cpan(input: &str) -> Result<CpanVersion, String> {
    let text = input.trim();
    if text == "undef" {
        return Ok(CpanVersion { components: Vec::new(), is_alpha: false });
    }
    let has_v = text.starts_with('v');
    let text = text.strip_prefix('v').unwrap_or(text);
    let error = || format!("invalid CPAN version: '{input}'");

    let (text, is_alpha) = if let Some((prefix, tail)) = text.split_once('_') {
        if !prefix.contains('.')
            || !prefix.as_bytes().last().is_some_and(u8::is_ascii_digit)
            || (tail.is_empty() && !has_v && prefix.bytes().filter(|&b| b == b'.').count() < 2)
            || !tail.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(error());
        }
        (format!("{prefix}{tail}"), true)
    } else {
        (text.to_string(), false)
    };
    let mut components = Vec::new();
    let dots = text.bytes().filter(|&b| b == b'.').count();
    let is_dotted = has_v || dots > 1;
    let component = |part: &str| {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(error());
        }
        let number: i32 = part.parse().map_err(|_| error())?;
        u64::try_from(number).map_err(|_| error())
    };
    if is_dotted {
        let text = if dots == 1 { text.strip_suffix('.').unwrap_or(&text) } else { &text };
        let mut parts = text.split('.');
        let first = parts.next().ok_or_else(error)?;
        components.push(if first.is_empty() && !has_v { 0 } else { component(first)? });
        for part in parts {
            components.push(component(part)?);
        }
    } else {
        let (whole, fraction) =
            text.split_once('.').map_or((text.as_str(), None), |(a, b)| (a, Some(b)));
        if whole.is_empty() && fraction.is_none() {
            return Err(error());
        }
        components.push(if whole.is_empty() { 0 } else { component(whole)? });
        if let Some(fraction) = fraction {
            if !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return Err(error());
            }
            // Perl groups decimal fractions in sets of three digits and
            // fills a short final group with zeros on the right.
            for digits in fraction.as_bytes().chunks(3) {
                let value = digits.iter().fold(0, |n, &digit| n * 10 + u64::from(digit - b'0'));
                components.push(match digits.len() {
                    1 => value * 100,
                    2 => value * 10,
                    _ => value,
                });
            }
        }
    }
    while components.last() == Some(&0) {
        components.pop();
    }
    Ok(CpanVersion { components, is_alpha })
}
