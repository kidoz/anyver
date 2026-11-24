//! Python bindings for PURL, VERS, and OSV affected-range helpers.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::parser::Ecosystem;
use crate::ranges::RangeSet;
use crate::security::{
    OsvEvent, Purl, Vers, osv_ecosystem as resolve_osv_ecosystem, osv_range_windows,
    validate_purl_type,
};
use crate::strategies::{compare_for_ecosystem, parse_for_ecosystem};
use std::cmp::Ordering;

use super::conv::{eco_from_str, value_error};

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
    let parsed = Purl::parse(purl).map_err(value_error)?;
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
        return Err(value_error("package_type and name are required"));
    }
    let package_type = package_type.to_ascii_lowercase();
    validate_purl_type(&package_type).map_err(value_error)?;
    let purl = Purl {
        package_type,
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
    let parsed = Purl::parse(purl).map_err(value_error)?;
    Ok(parsed.ecosystem().map(Ecosystem::as_str))
}

#[pyfunction]
pub(crate) fn vers_contains(vers: &str, version: &str) -> PyResult<bool> {
    let parsed = Vers::parse(vers).map_err(value_error)?;
    parsed.contains(version).map_err(value_error)
}

#[pyfunction(signature = (ecosystem, range))]
pub(crate) fn build_vers(ecosystem: &str, range: &str) -> PyResult<String> {
    let eco = eco_from_str(ecosystem)?;
    let vers = Vers::new(eco, range).map_err(value_error)?;
    Ok(vers.to_vers())
}

/// Resolve the ecosystem name of an OSV record (`Debian:11`, `PyPI`, `Go`) to
/// the anyver mode that orders its versions. Unrecognised names are an error.
#[pyfunction]
pub(crate) fn osv_ecosystem(name: &str) -> PyResult<&'static str> {
    resolve_osv_ecosystem(name).map(Ecosystem::as_str).map_err(value_error)
}

#[pyfunction(signature = (version, affected, ecosystem = None))]
pub(crate) fn osv_affected(
    version: &str,
    affected: &Bound<'_, PyDict>,
    ecosystem: Option<&str>,
) -> PyResult<bool> {
    // The package ecosystem is only needed when something has to be compared
    // with it, so an unknown name does not fail a record whose ranges are all
    // `SEMVER` or `GIT`, or whose `versions` match exactly.
    let explicit = match ecosystem {
        Some(s) => Some(eco_from_str(s)?),
        None => None,
    };
    let name = if explicit.is_none() { package_ecosystem(affected)? } else { None };
    let resolve = || -> PyResult<Ecosystem> {
        match (explicit, name.as_deref()) {
            (Some(eco), _) => Ok(eco),
            (None, Some(name)) => resolve_osv_ecosystem(name).map_err(value_error),
            (None, None) => Ok(Ecosystem::Generic),
        }
    };

    if let Some(ranges_obj) = affected.get_item("ranges")? {
        let ranges = ranges_obj.cast::<PyList>()?;
        for item in ranges {
            let range = item.cast::<PyDict>()?;
            let range_type =
                range.get_item("type")?.map(|kind| kind.extract::<String>()).transpose()?;
            let is_type = |expected: &str| {
                range_type.as_deref().is_some_and(|kind| kind.eq_ignore_ascii_case(expected))
            };
            // `GIT` ranges carry commit hashes where versions would go;
            // comparing them as versions is meaningless (and errors out on
            // strict ecosystems).
            if is_type("git") {
                continue;
            }
            let range_eco = if is_type("semver") { Ecosystem::Semver } else { resolve()? };
            let windows = osv_range_windows(range_eco, &osv_events(range)?).map_err(value_error)?;
            let affected_windows = RangeSet { branches: windows };
            if affected_windows.contains_bounds(range_eco, version).map_err(value_error)? {
                return Ok(true);
            }
        }
    }

    if let Some(versions_obj) = affected.get_item("versions")? {
        let versions = versions_obj.cast::<PyList>()?;
        let mut listed = Vec::with_capacity(versions.len());
        for item in versions {
            let entry: String = item.extract()?;
            if entry == version {
                return Ok(true);
            }
            listed.push(entry);
        }
        // Another spelling of the same release (`1.2.3.0` for NuGet's `1.2.3`)
        // names the same version, so compare through the ecosystem as ranges
        // do. Without a known ecosystem, or for an entry it cannot parse, only
        // the exact spelling above applies.
        if let Some(eco) = resolve().ok()
            && let Ok(target) = parse_for_ecosystem(eco, version)
        {
            for entry in listed {
                if parse_for_ecosystem(eco, &entry).is_ok_and(|parsed| {
                    compare_for_ecosystem(eco, &target, &parsed) == Ordering::Equal
                }) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// The `affected.package.ecosystem` value of an OSV record, when it has one.
fn package_ecosystem(affected: &Bound<'_, PyDict>) -> PyResult<Option<String>> {
    let Some(package) = affected.get_item("package")? else {
        return Ok(None);
    };
    let package = package.cast::<PyDict>()?;
    match package.get_item("ecosystem")? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read the `events` list of one OSV range.
fn osv_events(range: &Bound<'_, PyDict>) -> PyResult<Vec<OsvEvent>> {
    let Some(events_obj) = range.get_item("events")? else {
        return Ok(Vec::new());
    };
    let events = events_obj.cast::<PyList>()?;
    let mut out = Vec::with_capacity(events.len());
    for item in events {
        let event = item.cast::<PyDict>()?;
        for (key, build) in [
            ("limit", OsvEvent::Limit as fn(String) -> OsvEvent),
            ("introduced", OsvEvent::Introduced),
            ("fixed", OsvEvent::Fixed),
            ("last_affected", OsvEvent::LastAffected),
        ] {
            if let Some(value) = event.get_item(key)? {
                out.push(build(value.extract()?));
            }
        }
    }
    Ok(out)
}
