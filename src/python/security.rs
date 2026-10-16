//! Python bindings for PURL, VERS, and OSV affected-range helpers.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::parser::Ecosystem;
use crate::ranges::{Comparator, RangeOp, RangeSet};
use crate::security::{Purl, Vers, osv_ecosystem};

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
            Some(value) => osv_ecosystem(&value.extract::<String>()?),
            None => Ecosystem::Generic,
        }
    } else {
        Ecosystem::Generic
    };

    if let Some(ranges_obj) = affected.get_item("ranges")? {
        let ranges = ranges_obj.cast::<PyList>()?;
        for item in ranges {
            let range = item.cast::<PyDict>()?;
            let branches = osv_range_branches(range)?;
            if branches.is_empty() {
                continue;
            }
            let windows = RangeSet { branches };
            let range_eco = match range.get_item("type")? {
                Some(kind) if kind.extract::<String>()?.eq_ignore_ascii_case("semver") => {
                    Ecosystem::Semver
                }
                _ => eco,
            };
            if windows.contains_bounds(range_eco, version).map_err(PyValueError::new_err)? {
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

/// Turn one OSV `ranges[]` entry into a set of affected windows, combined with
/// a logical or.
///
/// An OSV range is an event *timeline*, not a single interval: `introduced`
/// opens a window and `fixed`/`last_affected` closes it, and a range commonly
/// carries several pairs (`0 → 1.2.0`, then `2.0.0 → 2.3.0`). Folding the
/// events into one interval keeps only the last window and silently reports
/// versions in every earlier one as unaffected.
fn osv_range_branches(range: &Bound<'_, PyDict>) -> PyResult<Vec<Vec<Comparator>>> {
    // `GIT` ranges carry commit hashes where versions would go; comparing them
    // as versions is meaningless (and errors out on strict ecosystems).
    if let Some(kind) = range.get_item("type")?
        && kind.extract::<String>()?.eq_ignore_ascii_case("git")
    {
        return Ok(Vec::new());
    }
    let Some(events_obj) = range.get_item("events")? else {
        return Ok(Vec::new());
    };
    let events = events_obj.cast::<PyList>()?;

    let mut branches: Vec<Vec<Comparator>> = Vec::new();
    let mut open: Option<Vec<Comparator>> = None;
    for event_obj in events {
        let event = event_obj.cast::<PyDict>()?;
        if let Some(value) = event.get_item("introduced")? {
            // A second `introduced` without an intervening close means the
            // previous window runs to infinity.
            if let Some(previous) = open.take() {
                branches.push(previous);
            }
            let start: String = value.extract()?;
            // `introduced: "0"` is OSV's "since the beginning" sentinel and is
            // not a real version — it must not be parsed as one.
            let clauses = if start == "0" {
                Vec::new()
            } else {
                vec![Comparator { op: RangeOp::Ge, version: start }]
            };
            open = Some(clauses);
        }
        if let Some(value) = event.get_item("fixed")? {
            let mut clauses = open.take().unwrap_or_default();
            clauses.push(Comparator { op: RangeOp::Lt, version: value.extract()? });
            branches.push(clauses);
        }
        if let Some(value) = event.get_item("last_affected")? {
            let mut clauses = open.take().unwrap_or_default();
            clauses.push(Comparator { op: RangeOp::Le, version: value.extract()? });
            branches.push(clauses);
        }
    }
    if let Some(previous) = open.take() {
        branches.push(previous);
    }
    Ok(branches)
}
