//! Version constraint parsing and evaluation: bare operators (`>=1.0,<2.0`),
//! OR groups (`^1.0 || ^2.0`), and npm/cargo-style shorthand ranges
//! (`^1.2.3`, `~1.2`, `1.2.x`, `*`), plus the `satisfies` entry point.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::ranges::RangeSet;

use super::conv::eco_from_str;

#[cfg(test)]
pub(crate) fn parse_constraint(spec: &str) -> Result<(&str, &str), String> {
    let s = spec.trim();
    if let Some(v) = s.strip_prefix(">=") {
        Ok((">=", v.trim()))
    } else if let Some(v) = s.strip_prefix("<=") {
        Ok(("<=", v.trim()))
    } else if let Some(v) = s.strip_prefix("!=") {
        Ok(("!=", v.trim()))
    } else if let Some(v) = s.strip_prefix("==") {
        Ok(("==", v.trim()))
    } else if let Some(v) = s.strip_prefix('>') {
        Ok((">", v.trim()))
    } else if let Some(v) = s.strip_prefix('<') {
        Ok(("<", v.trim()))
    } else {
        Err(format!(
            "invalid constraint: '{s}'; expected operator (>=, <=, >, <, ==, !=) followed by version"
        ))
    }
}

/// Check whether `version` satisfies `constraint`.
///
/// Supported syntax:
///   - Operators: `>=`, `<=`, `>`, `<`, `==`, `!=`
///   - AND via comma: `>=1.0,<2.0`
///   - OR via `||`: `^1.0 || ^2.0`
///   - Shorthand: `^1.2.3`, `~1.2`, `1.2.x`, `*`
#[pyfunction(signature = (version, constraint, ecosystem = "generic"))]
pub(crate) fn satisfies(version: &str, constraint: &str, ecosystem: &str) -> PyResult<bool> {
    range_contains(version, constraint, ecosystem)
}

#[pyfunction(signature = (version, range, ecosystem = "generic"))]
pub(crate) fn range_contains(version: &str, range: &str, ecosystem: &str) -> PyResult<bool> {
    let eco = eco_from_str(ecosystem)?;
    let parsed = RangeSet::parse(range).map_err(PyValueError::new_err)?;
    parsed.contains(eco, version).map_err(PyValueError::new_err)
}

#[pyfunction(signature = (left, right, ecosystem = "generic"))]
pub(crate) fn ranges_intersect(left: &str, right: &str, ecosystem: &str) -> PyResult<bool> {
    let eco = eco_from_str(ecosystem)?;
    let left = RangeSet::parse(left).map_err(PyValueError::new_err)?;
    let right = RangeSet::parse(right).map_err(PyValueError::new_err)?;
    left.intersects(eco, &right).map_err(PyValueError::new_err)
}
