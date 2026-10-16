//! PEP 440 grammar, normalization, and comparison keys.

use std::cmp::Ordering;

use crate::parser::strip_v;

/// A portable lexicographic key element. Numeric values retain all 64 bits.
pub(crate) type KeyPart = (i32, u64, String);

#[derive(Debug, Clone)]
pub(crate) struct Pep440Version {
    key: Vec<KeyPart>,
}

impl Pep440Version {
    pub(crate) fn key(&self) -> &[KeyPart] {
        &self.key
    }
}

fn number(input: &mut &str) -> Result<Option<u64>, String> {
    let len = input.bytes().take_while(u8::is_ascii_digit).count();
    if len == 0 {
        return Ok(None);
    }
    let value = input[..len].parse().map_err(|_| "PEP 440 number exceeds u64".to_string())?;
    *input = &input[len..];
    Ok(Some(value))
}

fn suffix(input: &mut &str, names: &[&str]) -> Result<Option<(usize, u64)>, String> {
    let rest = input.strip_prefix(['.', '-', '_']).unwrap_or(input);
    for (rank, name) in names.iter().enumerate() {
        if let Some(tail) = rest.strip_prefix(name) {
            let mut tail = tail.strip_prefix(['.', '-', '_']).unwrap_or(tail);
            let value = number(&mut tail)?.unwrap_or(0);
            *input = tail;
            return Ok(Some((rank, value)));
        }
    }
    Ok(None)
}

pub(crate) fn parse_pep440(input: &str) -> Result<Pep440Version, String> {
    let lower = input.trim().to_ascii_lowercase();
    let mut rest = strip_v(&lower);
    let first = number(&mut rest)?.ok_or_else(|| format!("invalid PEP 440 version: '{input}'"))?;
    let (epoch, first_release) = if let Some(tail) = rest.strip_prefix('!') {
        rest = tail;
        (first, number(&mut rest)?.ok_or("missing PEP 440 release")?)
    } else {
        (0, first)
    };
    let mut release = vec![first_release];
    while rest.starts_with('.') && rest.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
        rest = &rest[1..];
        release.push(number(&mut rest)?.ok_or("missing release number")?);
    }
    let pre = suffix(&mut rest, &["alpha", "beta", "preview", "pre", "rc", "a", "b", "c"])?.map(
        |(rank, n)| {
            (
                match rank {
                    0 | 5 => 0,
                    1 | 6 => 1,
                    _ => 2,
                },
                n,
            )
        },
    );
    let post = if rest.starts_with('-') && rest.as_bytes().get(1).is_some_and(u8::is_ascii_digit) {
        rest = &rest[1..];
        number(&mut rest)?
    } else {
        suffix(&mut rest, &["post", "rev", "r"])?.map(|(_, n)| n)
    };
    let dev = suffix(&mut rest, &["dev"])?.map(|(_, n)| n);
    let local = if let Some(label) = rest.strip_prefix('+') {
        rest = "";
        let mut parts = Vec::new();
        for part in label.split(['.', '-', '_']) {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(format!("invalid PEP 440 local label: '{label}'"));
            }
            parts.push(if part.bytes().all(|b| b.is_ascii_digit()) {
                (2, part.parse().map_err(|_| "PEP 440 local number exceeds u64")?, String::new())
            } else {
                (1, 0, part.to_string())
            });
        }
        parts
    } else {
        Vec::new()
    };
    if !rest.is_empty() {
        return Err(format!("invalid PEP 440 suffix: '{rest}'"));
    }
    while release.last() == Some(&0) {
        release.pop();
    }
    let mut key = vec![(0, epoch, String::new())];
    key.extend(release.into_iter().map(|n| (1, n, String::new())));
    key.push((0, 0, String::new())); // end of release tuple
    key.push(match pre {
        Some((rank, n)) => (rank + 1, n, String::new()),
        None if dev.is_some() && post.is_none() => (0, 0, String::new()),
        None => (4, 0, String::new()),
    });
    key.push((i32::from(post.is_some()), post.unwrap_or(0), String::new()));
    key.push((i32::from(dev.is_none()), dev.unwrap_or(0), String::new()));
    key.extend(local);
    Ok(Pep440Version { key })
}

pub(crate) fn cmp_pep440(a: &Pep440Version, b: &Pep440Version) -> Ordering {
    a.key.cmp(&b.key)
}
