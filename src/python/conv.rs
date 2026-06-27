//! Shared conversion and extraction helpers used across the Python binding
//! modules: ecosystem resolution, `Seg → Python` conversion, and pulling a
//! parsed representation out of a `Version` or `str` argument.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::parser::{Ecosystem, Seg, autodetect_ecosystem};
use crate::strategies::{ParsedRepr, parse_for_ecosystem};

use super::pyversion::PyVersion;

pub(crate) fn eco_from_str(input: &str) -> PyResult<Ecosystem> {
    Ecosystem::from_str(input).map_err(PyValueError::new_err)
}

pub(crate) trait SegToPy {
    fn to_py<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny>;
}

impl SegToPy for Seg {
    fn to_py<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        match self {
            Seg::Num(n) => (*n).into_pyobject(py).unwrap().into_any(),
            Seg::Text(s) => s.as_str().into_pyobject(py).unwrap().into_any(),
        }
    }
}

pub(crate) fn resolve_eco(
    ecosystem: &str,
    first: Option<&Bound<'_, PyAny>>,
) -> PyResult<Ecosystem> {
    if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(obj) = first {
            resolve_eco_for_obj("auto", obj)
        } else {
            Ok(Ecosystem::Generic)
        }
    } else {
        eco_from_str(ecosystem)
    }
}

pub(crate) fn extract_raw(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
        Ok(v.inner.raw.clone())
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(s)
    } else {
        Err(PyTypeError::new_err("expected Version or str"))
    }
}

pub(crate) fn resolve_eco_for_obj(ecosystem: &str, obj: &Bound<'_, PyAny>) -> PyResult<Ecosystem> {
    if ecosystem.eq_ignore_ascii_case("auto") {
        if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
            Ok(v.eco)
        } else if let Ok(s) = obj.extract::<String>() {
            Ok(autodetect_ecosystem(&s))
        } else {
            Err(PyTypeError::new_err("expected Version or str"))
        }
    } else {
        eco_from_str(ecosystem)
    }
}

pub(crate) fn extract_parsed_for_ecosystem(
    obj: &Bound<'_, PyAny>,
    ecosystem: Ecosystem,
) -> PyResult<ParsedRepr> {
    if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
        match ecosystem {
            Ecosystem::Generic => Ok(ParsedRepr::Generic(v.inner.clone())),
            _ => parse_for_ecosystem(ecosystem, &v.inner.raw).map_err(PyValueError::new_err),
        }
    } else if let Ok(s) = obj.extract::<String>() {
        parse_for_ecosystem(ecosystem, &s).map_err(PyValueError::new_err)
    } else {
        Err(PyTypeError::new_err("expected Version or str"))
    }
}
