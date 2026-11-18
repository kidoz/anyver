//! Java 9+ runtime versions, ordered like `java.lang.Runtime.Version.compareTo`.

use super::SortKey;
use crate::parser::{ParsedVersion, Seg};

// The variant order matches Java's numeric < text < release precedence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Prerelease {
    Numeric { digits: usize, value: String },
    Text(String),
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct JdkVersion {
    release: Vec<u32>,
    prerelease: Prerelease,
    build: Option<u32>,
    optional: Option<String>,
}

impl JdkVersion {
    pub(crate) fn is_prerelease(&self) -> bool {
        self.prerelease != Prerelease::Release
    }

    pub(crate) fn sort_key(&self) -> SortKey {
        let mut key: SortKey =
            self.release.iter().map(|&part| (3, u64::from(part), String::new())).collect();
        key.push(match &self.prerelease {
            Prerelease::Numeric { digits, value } => (0, *digits as u64, value.clone()),
            Prerelease::Text(value) => (1, 0, value.clone()),
            Prerelease::Release => (2, 0, String::new()),
        });
        key.push(match self.build {
            Some(number) => (1, u64::from(number), String::new()),
            None => (0, 0, String::new()),
        });
        if let Some(optional) = &self.optional {
            key.push((0, 0, optional.clone()));
        }
        key
    }

    pub(crate) fn version_view(&self, input: &str) -> ParsedVersion {
        let mut segments: Vec<_> =
            self.release.iter().map(|&part| Seg::Num(u64::from(part))).collect();
        match &self.prerelease {
            // Treat a prerelease as one identifier. A numeric prerelease
            // must stay separate from the numeric release components.
            Prerelease::Numeric { value, .. } | Prerelease::Text(value) => {
                segments.push(Seg::Text(value.clone()));
            }
            Prerelease::Release => {}
        }
        ParsedVersion {
            raw: input.to_string(),
            epoch: 0,
            segments,
            build: input.split_once('+').map_or_else(String::new, |(_, suffix)| suffix.to_string()),
            is_prerelease: self.is_prerelease(),
            is_postrelease: false,
        }
    }
}

fn parse_number(input: &str) -> Option<u32> {
    if input.is_empty()
        || (input.len() > 1 && input.starts_with('0'))
        || !input.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    // Runtime.Version uses Java's signed Integer for release and build.
    input.parse::<i32>().ok().and_then(|number| u32::try_from(number).ok())
}

fn parse_version(input: &str) -> Option<JdkVersion> {
    let (before_build, build_suffix) =
        input.split_once('+').map_or((input, None), |(left, right)| (left, Some(right)));
    let mut parts = before_build.splitn(3, '-');
    let release: Vec<_> = parts.next()?.split('.').map(parse_number).collect::<Option<_>>()?;
    if release.first() == Some(&0) || release.last() == Some(&0) {
        return None;
    }
    let prerelease = match parts.next() {
        None => Prerelease::Release,
        Some(pre) if pre.is_empty() || !pre.bytes().all(|b| b.is_ascii_alphanumeric()) => {
            return None;
        }
        Some(pre) if pre.bytes().all(|b| b.is_ascii_digit()) => {
            let trimmed = pre.trim_start_matches('0');
            let value = if trimmed.is_empty() { "0" } else { trimmed };
            Prerelease::Numeric { digits: value.len(), value: value.to_string() }
        }
        Some(pre) => Prerelease::Text(pre.to_string()),
    };
    let (build, optional) = match (build_suffix, parts.next()) {
        (None, optional) => (None, optional),
        (Some(_), Some(_)) => return None,
        (Some(suffix), None) => {
            let (build, optional) =
                suffix.split_once('-').map_or((suffix, None), |(left, right)| (left, Some(right)));
            if build.is_empty() && optional.is_some() && prerelease == Prerelease::Release {
                (None, optional)
            } else {
                (Some(parse_number(build)?), optional)
            }
        }
    };
    if optional.is_some_and(|text| {
        text.is_empty()
            || !text.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.'))
    }) {
        return None;
    }
    Some(JdkVersion { release, prerelease, build, optional: optional.map(str::to_string) })
}

pub(super) fn parse_jdk(input: &str) -> Result<JdkVersion, String> {
    parse_version(input).ok_or_else(|| format!("invalid JDK runtime version: '{input}'"))
}
