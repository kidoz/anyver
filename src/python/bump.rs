//! Version-bumping helpers exposed to Python: `bump_major`/`bump_minor`/
//! `bump_patch`, `bump_prerelease`, and `next_stable`.

use pyo3::prelude::*;

use crate::parser::{ParsedVersion, Seg, parse};

use super::conv::value_error;

fn increment(value: u64) -> PyResult<u64> {
    value.checked_add(1).ok_or_else(|| value_error("version increment exceeds u64"))
}

/// The numeric release component at `index`; missing or text → 0. A component
/// wider than `u64` has no exact `u64` form to print or increment.
fn component(v: &ParsedVersion, index: usize) -> PyResult<u64> {
    match v.segments.get(index) {
        Some(Seg::Num(n)) => Ok(*n),
        Some(Seg::Wide(digits)) => {
            Err(value_error(format!("version component '{digits}' exceeds u64")))
        }
        _ => Ok(0),
    }
}

#[pyfunction]
pub(crate) fn bump_major(version: &str) -> PyResult<String> {
    let v = parse(version);
    Ok(format!("{}.0.0", increment(component(&v, 0)?)?))
}

#[pyfunction]
pub(crate) fn bump_minor(version: &str) -> PyResult<String> {
    let v = parse(version);
    Ok(format!("{}.{}.0", component(&v, 0)?, increment(component(&v, 1)?)?))
}

#[pyfunction]
pub(crate) fn bump_patch(version: &str) -> PyResult<String> {
    let v = parse(version);
    Ok(format!("{}.{}.{}", component(&v, 0)?, component(&v, 1)?, increment(component(&v, 2)?)?))
}

fn release_triple(v: &ParsedVersion) -> PyResult<(u64, u64, u64)> {
    Ok((component(v, 0)?, component(v, 1)?, component(v, 2)?))
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
pub(crate) fn bump_prerelease(version: &str, tag: &str) -> PyResult<String> {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v)?;
    let tag_lower = tag.to_ascii_lowercase();

    // Find existing prerelease tag index
    let first_text = v.segments.iter().position(|s| matches!(s, Seg::Text(_)));
    if let Some(pos) = first_text
        && let Seg::Text(existing) = &v.segments[pos]
        && *existing == tag_lower
    {
        let next_num = match v.segments.get(pos + 1) {
            Some(Seg::Num(n)) => Some(*n),
            Some(Seg::Wide(digits)) => {
                return Err(value_error(format!("version component '{digits}' exceeds u64")));
            }
            _ => None,
        };
        let n = next_num.map(increment).transpose()?.unwrap_or(0);
        return Ok(format!("{major}.{minor}.{patch}-{tag_lower}.{n}"));
    }
    Ok(format!("{major}.{minor}.{patch}-{tag_lower}.0"))
}

/// Return the release form of a version, stripping any prerelease/postrelease
/// tags and build metadata. `1.2.3-alpha.1+build` → `1.2.3`.
#[pyfunction]
pub(crate) fn next_stable(version: &str) -> PyResult<String> {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v)?;
    Ok(format!("{major}.{minor}.{patch}"))
}
