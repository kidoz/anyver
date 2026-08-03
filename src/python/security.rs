//! Python bindings for PURL, VERS, and OSV affected-range helpers.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::parser::Ecosystem;
use crate::ranges::RangeSet;
use crate::security::{Purl, Vers};

use super::conv::eco_from_str;

fn purl_to_dict<'py>(py: Python<'py>, purl: &Purl) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("type", &purl.package_type)?;
    dict.set_item("namespace", &purl.namespace)?;
    dict.set_item("name", &purl.name)?;
    dict.set_item("version", &purl.version)?;
    dict.set_item("qualifiers", &purl.qualifiers)?;
    dict.set_item("subpath", &purl.subpath)?;
    dict.set_item("ecosystem", purl.ecosystem().map(Ecosystem::as_str))?;
    Ok(dict)
}

#[pyfunction]
pub(crate) fn parse_purl<'py>(py: Python<'py>, purl: &str) -> PyResult<Bound<'py, PyDict>> {
    let parsed = Purl::parse(purl).map_err(PyValueError::new_err)?;
    purl_to_dict(py, &parsed)
}

#[pyfunction(signature = (package_type, name, version = None, namespace = None, qualifiers = None, subpath = None))]
pub(crate) fn build_purl(
    package_type: &str,
    name: &str,
    version: Option<&str>,
    namespace: Option<&str>,
    qualifiers: Option<&str>,
    subpath: Option<&str>,
) -> PyResult<String> {
    if package_type.trim().is_empty() || name.trim().is_empty() {
        return Err(PyValueError::new_err("package_type and name are required"));
    }
    let purl = Purl {
        package_type: package_type.to_ascii_lowercase(),
        namespace: namespace.map(str::to_string),
        name: name.to_string(),
        version: version.map(str::to_string),
        qualifiers: qualifiers.map(str::to_string),
        subpath: subpath.map(str::to_string),
    };
    Ok(purl.to_purl())
}

#[pyfunction]
pub(crate) fn purl_ecosystem(purl: &str) -> PyResult<Option<&'static str>> {
    let parsed = Purl::parse(purl).map_err(PyValueError::new_err)?;
    Ok(parsed.ecosystem().map(Ecosystem::as_str))
}

#[pyfunction]
pub(crate) fn vers_contains(vers: &str, version: &str) -> PyResult<bool> {
    let parsed = Vers::parse(vers).map_err(PyValueError::new_err)?;
    parsed.contains(version).map_err(PyValueError::new_err)
}

#[pyfunction(signature = (ecosystem, range))]
pub(crate) fn build_vers(ecosystem: &str, range: &str) -> PyResult<String> {
    let eco = eco_from_str(ecosystem)?;
    let vers = Vers::new(eco, range).map_err(PyValueError::new_err)?;
    Ok(vers.to_vers())
}

#[pyfunction(signature = (version, affected, ecosystem = None))]
pub(crate) fn osv_affected(
    version: &str,
    affected: &Bound<'_, PyDict>,
    ecosystem: Option<&str>,
) -> PyResult<bool> {
    let eco = if let Some(eco) = ecosystem {
        eco_from_str(eco)?
    } else if let Some(package) = affected.get_item("package")? {
        let package = package.cast::<PyDict>()?;
        match package.get_item("ecosystem")? {
            Some(value) => eco_from_str(&value.extract::<String>()?)?,
            None => Ecosystem::Generic,
        }
    } else {
        Ecosystem::Generic
    };

    if let Some(ranges_obj) = affected.get_item("ranges")? {
        let ranges = ranges_obj.cast::<PyList>()?;
        for item in ranges {
            let range = item.cast::<PyDict>()?;
            if range_contains_osv_events(version, eco, range)? {
                return Ok(true);
            }
        }
    }

    if let Some(versions_obj) = affected.get_item("versions")? {
        let versions = versions_obj.cast::<PyList>()?;
        for item in versions {
            if item.extract::<String>()? == version {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn range_contains_osv_events(
    version: &str,
    ecosystem: Ecosystem,
    range: &Bound<'_, PyDict>,
) -> PyResult<bool> {
    let Some(events_obj) = range.get_item("events")? else {
        return Ok(false);
    };
    let events = events_obj.cast::<PyList>()?;
    let mut introduced: Option<String> = None;
    let mut fixed: Option<String> = None;
    let mut last_affected: Option<String> = None;
    for event_obj in events {
        let event = event_obj.cast::<PyDict>()?;
        if let Some(value) = event.get_item("introduced")? {
            introduced = Some(value.extract()?);
        }
        if let Some(value) = event.get_item("fixed")? {
            fixed = Some(value.extract()?);
        }
        if let Some(value) = event.get_item("last_affected")? {
            last_affected = Some(value.extract()?);
        }
    }

    let mut clauses = Vec::new();
    if let Some(start) = introduced
        && start != "0"
    {
        clauses.push(format!(">={start}"));
    }
    if let Some(end) = fixed {
        clauses.push(format!("<{end}"));
    }
    if let Some(end) = last_affected {
        clauses.push(format!("<={end}"));
    }
    if clauses.is_empty() {
        return Ok(false);
    }
    let range = RangeSet::parse(&clauses.join(",")).map_err(PyValueError::new_err)?;
    range.contains(ecosystem, version).map_err(PyValueError::new_err)
}
