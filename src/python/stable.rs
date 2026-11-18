//! Stability-aware selectors: filtering out pre-releases (`stable_versions`)
//! and picking the newest stable release (`latest_stable`).

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::parser::Ecosystem;
use crate::strategies::ParsedRepr;

use super::compare::collection_max;
use super::conv::{extract_raw, resolve_eco};
use super::pyversion::PyVersion;

fn stable_candidate(obj: &Bound<'_, PyAny>, eco: Ecosystem) -> PyResult<Option<ParsedRepr>> {
    if let Ok(version) = obj.extract::<PyRef<PyVersion>>()
        && version.eco == eco
    {
        return Ok((!version.is_prerelease()).then(|| version.repr.clone()));
    }
    let version = PyVersion::new(&extract_raw(obj)?, eco.as_str())?;
    Ok((!version.is_prerelease()).then_some(version.repr))
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
pub(crate) fn stable_versions<'py>(
    _py: Python<'py>,
    versions: &Bound<'py, PyAny>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let versions = versions.try_iter()?.collect::<PyResult<Vec<_>>>()?;
    let eco = resolve_eco(ecosystem, versions.first())?;
    let mut stable = Vec::new();
    for obj in versions {
        if stable_candidate(&obj, eco)?.is_some() {
            stable.push(obj);
        }
    }
    Ok(stable)
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
    let mut stable = Vec::new();
    for obj in versions {
        if let Some(repr) = stable_candidate(&obj, eco)? {
            stable.push((repr, obj));
        }
    }
    collection_max(eco, stable)
        .map(|(_, obj)| obj)
        .ok_or_else(|| PyValueError::new_err("no stable versions found"))
}
