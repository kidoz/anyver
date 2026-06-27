//! Version-bumping helpers exposed to Python: `bump_major`/`bump_minor`/
//! `bump_patch`, `bump_prerelease`, and `next_stable`.

use pyo3::prelude::*;

use crate::parser::{ParsedVersion, Seg, parse};

#[pyfunction]
pub(crate) fn bump_major(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.0.0")
}

#[pyfunction]
pub(crate) fn bump_minor(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.{minor}.0")
}

#[pyfunction]
pub(crate) fn bump_patch(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let patch = match v.segments.get(2) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.{minor}.{patch}")
}

fn release_triple(v: &ParsedVersion) -> (u64, u64, u64) {
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let patch = match v.segments.get(2) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    (major, minor, patch)
}

/// Bump the prerelease counter on a version.
///
/// Semantics:
///   - If the version has a prerelease segment with the same `tag`, the
///     first numeric segment following that tag is incremented.
///   - Otherwise the existing prerelease is replaced with `-{tag}.0`.
///   - If no release triple is present, it's filled with `0.0.0`.
#[pyfunction]
#[pyo3(signature = (version, tag = "alpha"))]
pub(crate) fn bump_prerelease(version: &str, tag: &str) -> String {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v);
    let tag_lower = tag.to_ascii_lowercase();

    // Find existing prerelease tag index
    let first_text = v.segments.iter().position(|s| matches!(s, Seg::Text(_)));
    if let Some(pos) = first_text
        && let Seg::Text(existing) = &v.segments[pos]
        && *existing == tag_lower
    {
        let next_num = v.segments.get(pos + 1).and_then(|s| match s {
            Seg::Num(n) => Some(*n),
            Seg::Text(_) => None,
        });
        let n = next_num.map_or(0, |n| n + 1);
        return format!("{major}.{minor}.{patch}-{tag_lower}.{n}");
    }
    format!("{major}.{minor}.{patch}-{tag_lower}.0")
}

/// Return the release form of a version, stripping any prerelease/postrelease
/// tags and build metadata. `1.2.3-alpha.1+build` → `1.2.3`.
#[pyfunction]
pub(crate) fn next_stable(version: &str) -> String {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v);
    format!("{major}.{minor}.{patch}")
}
