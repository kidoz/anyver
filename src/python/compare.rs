//! Module-level comparison functions exposed to Python: the `compare`/`gt`/
//! `lt`/… predicates, `sort_versions`, `batch_compare`, and `min`/`max`.
//!
//! [`compare_str_with_ecosystem`] is the shared string-comparison primitive
//! reused by the constraint solver and the crate's unit tests.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::cmp::Ordering;

use crate::parser::{Ecosystem, autodetect_ecosystem};
use crate::strategies::{ParsedRepr, compare_for_ecosystem, parse_for_ecosystem};

use super::conv::{eco_from_str, extract_parsed_for_ecosystem, resolve_eco, resolve_eco_for_obj};
use super::pyversion::PyVersion;

pub(crate) fn compare_str_with_ecosystem(a: &str, b: &str, ecosystem: &str) -> PyResult<Ordering> {
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        autodetect_ecosystem(a)
    } else {
        eco_from_str(ecosystem)?
    };
    let left = parse_for_ecosystem(eco, a).map_err(PyValueError::new_err)?;
    let right = parse_for_ecosystem(eco, b).map_err(PyValueError::new_err)?;
    Ok(compare_for_ecosystem(eco, &left, &right))
}

#[pyfunction]
pub(crate) fn compare_semver_strict(a: &str, b: &str) -> PyResult<i32> {
    let pa = parse_for_ecosystem(Ecosystem::Semver, a).map_err(PyValueError::new_err)?;
    let pb = parse_for_ecosystem(Ecosystem::Semver, b).map_err(PyValueError::new_err)?;
    Ok(match compare_for_ecosystem(Ecosystem::Semver, &pa, &pb) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

#[pyfunction]
#[pyo3(signature = (s, ecosystem = "auto"))]
pub(crate) fn version(s: &str, ecosystem: &str) -> PyResult<PyVersion> {
    PyVersion::new(s, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn compare(
    a: &Bound<'_, PyAny>,
    b: &Bound<'_, PyAny>,
    ecosystem: &str,
) -> PyResult<i32> {
    let eco = resolve_eco_for_obj(ecosystem, a)?;
    let pa = extract_parsed_for_ecosystem(a, eco)?;
    let pb = extract_parsed_for_ecosystem(b, eco)?;
    Ok(match compare_for_ecosystem(eco, &pa, &pb) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn sort_versions<'py>(
    _py: Python<'py>,
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let eco = resolve_eco(ecosystem, versions.first())?;
    let mut pairs: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    pairs.sort_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0));
    Ok(pairs.into_iter().map(|(_, obj)| obj).collect())
}

#[pyfunction(signature = (pairs, ecosystem = "generic"))]
pub(crate) fn batch_compare(
    pairs: Vec<(Bound<'_, PyAny>, Bound<'_, PyAny>)>,
    ecosystem: &str,
) -> PyResult<Vec<i32>> {
    let is_auto = ecosystem.eq_ignore_ascii_case("auto");
    let base_eco = if is_auto { Ecosystem::Generic } else { eco_from_str(ecosystem)? };

    pairs
        .iter()
        .map(|(a, b)| {
            let eco = if is_auto { resolve_eco_for_obj("auto", a)? } else { base_eco };
            let pa = extract_parsed_for_ecosystem(a, eco)?;
            let pb = extract_parsed_for_ecosystem(b, eco)?;
            Ok(match compare_for_ecosystem(eco, &pa, &pb) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            })
        })
        .collect()
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn gt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn ge(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn lt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn le(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn eq(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Equal)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn ne(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Equal)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn gte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    ge(a, b, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
pub(crate) fn lte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    le(a, b, ecosystem)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn max_version<'py>(
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
    if versions.is_empty() {
        return Err(PyValueError::new_err("empty sequence"));
    }
    let eco = resolve_eco(ecosystem, versions.first())?;
    let mut parsed: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    parsed.sort_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0));
    Ok(parsed.pop().unwrap().1)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn min_version<'py>(
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
    if versions.is_empty() {
        return Err(PyValueError::new_err("empty sequence"));
    }
    let eco = resolve_eco(ecosystem, versions.first())?;
    let parsed: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let (_, min_obj) =
        parsed.into_iter().min_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0)).unwrap();
    Ok(min_obj)
}
