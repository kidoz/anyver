//! Stability-aware selectors: filtering out pre-releases (`stable_versions`)
//! and picking the newest stable release (`latest_stable`).

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::parser::parse;
use crate::strategies::{ParsedRepr, compare_for_ecosystem, parse_for_ecosystem};

use super::conv::{extract_parsed_for_ecosystem, extract_raw, resolve_eco};
use super::pyversion::PyVersion;

fn is_prerelease_obj(obj: &Bound<'_, PyAny>) -> PyResult<bool> {
    if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
        Ok(v.inner.is_prerelease)
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(parse(&s).is_prerelease)
    } else {
        Err(PyTypeError::new_err("expected Version or str"))
    }
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn stable_versions<'py>(
    _py: Python<'py>,
    versions: &Bound<'py, PyAny>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let versions = versions.try_iter()?.collect::<PyResult<Vec<_>>>()?;
    let eco = resolve_eco(ecosystem, versions.first())?;
    versions
        .into_iter()
        .filter_map(|obj| {
            match is_prerelease_obj(&obj) {
                Err(e) => Some(Err(e)),
                Ok(true) => None, // skip pre-releases
                Ok(false) => {
                    // validate against ecosystem
                    let raw = extract_raw(&obj).ok()?;
                    match parse_for_ecosystem(eco, &raw) {
                        Ok(_) => Some(Ok(obj)),
                        Err(e) => Some(Err(PyValueError::new_err(e))),
                    }
                }
            }
        })
        .collect()
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn latest_stable<'py>(
    versions: &Bound<'py, PyAny>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let versions = versions.try_iter()?.collect::<PyResult<Vec<_>>>()?;
    if versions.is_empty() {
        return Err(PyValueError::new_err("empty sequence"));
    }
    let eco = resolve_eco(ecosystem, versions.first())?;
    let mut stable: Vec<(ParsedRepr, Bound<'py, PyAny>)> = Vec::new();
    for obj in versions {
        if is_prerelease_obj(&obj)? {
            continue;
        }
        let repr = extract_parsed_for_ecosystem(&obj, eco)?;
        stable.push((repr, obj));
    }
    if stable.is_empty() {
        return Err(PyValueError::new_err("no stable versions found"));
    }
    stable.sort_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0));
    Ok(stable.pop().unwrap().1)
}
