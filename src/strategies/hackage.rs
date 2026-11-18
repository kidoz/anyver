//! Hackage/Cabal package release versions.

pub(super) fn parse_hackage(input: &str) -> Result<Vec<u64>, String> {
    input
        .trim()
        .split('.')
        .map(|part| {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!("invalid Hackage version: '{input}'"));
            }
            super::semver::parse_strict_u64(part, "Hackage version component")
        })
        .collect()
}
