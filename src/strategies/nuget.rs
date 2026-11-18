//! `NuGet` version-release precedence, as used by `VersionComparer.VersionRelease`.

use super::SortKey;
use super::semver::{validate_build, validate_prerelease};
use crate::parser::{ParsedVersion, Seg};

#[derive(Debug, Clone)]
pub(crate) struct NugetVersion {
    release: Vec<u32>,
    prerelease: Option<String>,
    key: SortKey,
}

impl NugetVersion {
    pub(crate) fn key(&self) -> &SortKey {
        &self.key
    }

    pub(crate) fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    pub(crate) fn version_view(&self, input: &str) -> ParsedVersion {
        let mut segments: Vec<_> =
            self.release.iter().map(|&part| Seg::Num(u64::from(part))).collect();
        if let Some(pre) = &self.prerelease {
            segments.extend(pre.split('.').map(|label| Seg::Text(label.to_string())));
        }
        ParsedVersion {
            raw: input.to_string(),
            epoch: 0,
            segments,
            build: input.trim().split_once('+').map_or_else(String::new, |(_, label)| label.into()),
            is_prerelease: self.is_prerelease(),
            is_postrelease: false,
        }
    }
}

pub(super) fn parse_nuget(input: &str) -> Result<NugetVersion, String> {
    let mut rest = input.trim();
    if let Some((before_build, build)) = rest.split_once('+') {
        validate_build(build)?;
        rest = before_build;
    }
    let (core, prerelease) =
        rest.split_once('-').map_or((rest, None), |(core, pre)| (core, Some(pre)));
    let mut release = Vec::with_capacity(4);
    for part in core.split('.') {
        let part = part.trim();
        let number = part.parse::<u32>().ok().filter(|&n| i32::try_from(n).is_ok());
        if let Some(number) = number.filter(|_| part.bytes().all(|b| b.is_ascii_digit()))
            && release.len() < 4
        {
            release.push(number);
        } else {
            return Err(format!("invalid NuGet version: '{input}'"));
        }
    }
    let mut key: SortKey = (0..4)
        .map(|i| (2, u64::from(release.get(i).copied().unwrap_or_default()), String::new()))
        .collect();
    key.push((i32::from(prerelease.is_none()), 0, String::new()));
    let prerelease = if let Some(pre) = prerelease {
        validate_prerelease(pre)?;
        let pre = pre.to_ascii_lowercase();
        key.extend(pre.split('.').map(|label| {
            // NuGet treats numeric labels beyond Int32.MaxValue as text.
            match label.parse::<i32>().ok().and_then(|n| u64::try_from(n).ok()) {
                Some(n) => (0, n, String::new()),
                None => (1, 0, label.to_string()),
            }
        }));
        Some(pre)
    } else {
        None
    };
    Ok(NugetVersion { release, prerelease, key })
}
