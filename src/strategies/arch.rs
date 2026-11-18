//! Arch Linux package version comparison, as implemented by pacman's vercmp.

use std::cmp::Ordering;

use super::SortKey;

type ComponentValue = (i32, u64, String);

#[derive(Debug, Clone)]
struct Component {
    separators: u64,
    value: ComponentValue,
}

#[derive(Debug, Clone)]
struct VersionPart {
    components: Vec<Component>,
    trailing_separator: bool,
}

impl VersionPart {
    fn parse(mut rest: &str) -> Self {
        let mut components = Vec::new();
        while !rest.is_empty() {
            let separators = rest.bytes().take_while(|b| !b.is_ascii_alphanumeric()).count();
            rest = &rest[separators..];
            let Some(first) = rest.bytes().next() else {
                return Self { components, trailing_separator: true };
            };
            let len = if first.is_ascii_digit() {
                rest.bytes().take_while(u8::is_ascii_digit).count()
            } else {
                rest.bytes().take_while(u8::is_ascii_alphabetic).count()
            };
            let (value, tail) = rest.split_at(len);
            let value = if first.is_ascii_digit() { numeric(value) } else { (0, 0, value.into()) };
            components.push(Component { separators: separators as u64, value });
            rest = tail;
        }
        Self { components, trailing_separator: false }
    }

    fn compare(&self, other: &Self) -> Ordering {
        let mut left = self.components.iter();
        let mut right = other.components.iter();
        loop {
            let order = match (left.next(), right.next()) {
                (Some(a), Some(b)) => {
                    a.separators.cmp(&b.separators).then_with(|| a.value.cmp(&b.value))
                }
                (Some(a), None) => return remaining_order(a, other.trailing_separator),
                (None, Some(b)) => return remaining_order(b, self.trailing_separator).reverse(),
                (None, None) => return self.trailing_separator.cmp(&other.trailing_separator),
            };
            if order != Ordering::Equal {
                return order;
            }
        }
    }

    fn append_key(&self, key: &mut SortKey) {
        for component in &self.components {
            let bare_text = component.separators == 0 && component.value.0 == 0;
            key.push((if bare_text { 0 } else { 2 }, component.separators, String::new()));
            key.push(component.value.clone());
        }
        key.push((if self.trailing_separator { 3 } else { 1 }, 0, String::new()));
    }
}

fn numeric(text: &str) -> ComponentValue {
    let digits = text.trim_start_matches('0');
    (1, digits.len() as u64, digits.into())
}

fn remaining_order(component: &Component, other_has_separator: bool) -> Ordering {
    if component.value.0 == 0 && (component.separators == 0 || other_has_separator) {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArchVersion {
    epoch: ComponentValue,
    upstream: VersionPart,
    release: Option<VersionPart>,
}

impl ArchVersion {
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        self.epoch.cmp(&other.epoch).then_with(|| self.upstream.compare(&other.upstream)).then_with(
            || match (&self.release, &other.release) {
                (Some(left), Some(right)) => left.compare(right),
                // A missing package release matches all releases of an upstream version.
                _ => Ordering::Equal,
            },
        )
    }

    pub(crate) fn hash_key(&self) -> SortKey {
        let mut key = vec![self.epoch.clone()];
        self.upstream.append_key(&mut key);
        key
    }

    pub(crate) fn sort_key(&self) -> SortKey {
        let mut key = self.hash_key();
        if let Some(release) = &self.release {
            release.append_key(&mut key);
        }
        key
    }
}

pub(super) fn parse_arch(input: &str) -> Result<ArchVersion, String> {
    let text = input.trim();
    let epoch_len = text.bytes().take_while(u8::is_ascii_digit).count();
    let (epoch, rest) =
        text[epoch_len..].strip_prefix(':').map_or(("", text), |rest| (&text[..epoch_len], rest));
    let (upstream, release) =
        rest.rsplit_once('-').map_or((rest, None), |(upstream, release)| (upstream, Some(release)));
    if !upstream.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        || text.bytes().any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
    {
        return Err(format!("invalid Arch version: '{input}'"));
    }
    Ok(ArchVersion {
        epoch: numeric(epoch),
        upstream: VersionPart::parse(upstream),
        release: release.map(VersionPart::parse),
    })
}
