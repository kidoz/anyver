//! The `Version` Python class (`PyVersion`) and its methods: construction,
//! rich comparison, accessors, segment/release views, dict and pickle
//! round-tripping, and the `sort_key` encoding.

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::cmp::Ordering;

use crate::parser::{Ecosystem, ParsedVersion, Seg, autodetect_ecosystem, parse, tag_weight};
use crate::strategies::{
    ParsedRepr, canonical_segments, compare_for_ecosystem, parse_for_ecosystem,
};

use super::conv::{SegToPy, eco_from_str, extract_parsed_for_ecosystem};

/// Tuple encoding shared by the portable sort key and Python hashing.
fn version_key(version: &PyVersion) -> Vec<(i32, u64, String)> {
    match &version.repr {
        ParsedRepr::Pep440(p) => p.key().to_vec(),
        ParsedRepr::Semver(v) => {
            let mut key = vec![
                (2, v.major, String::new()),
                (2, v.minor, String::new()),
                (2, v.patch, String::new()),
            ];
            key.push((i32::from(v.pre.is_none()), 0, String::new()));
            if let Some(pre) = &v.pre {
                for part in pre.split('.') {
                    key.push(match part.parse::<u64>() {
                        Ok(n) => (0, n, String::new()),
                        Err(_) => (1, 0, part.to_string()),
                    });
                }
            }
            key
        }
        ParsedRepr::Generic(p) => {
            let mut key = vec![(2, p.epoch, String::new())];
            for seg in canonical_segments(&p.segments).iter() {
                key.push(match seg {
                    Seg::Num(n) => (100, *n, String::new()),
                    Seg::Text(s) => (
                        tag_weight(s).unwrap_or(29),
                        0,
                        if tag_weight(s).is_some() { String::new() } else { s.clone() },
                    ),
                });
            }
            key.push((30, 0, String::new()));
            key
        }
        ParsedRepr::Debian(v) => v.sort_key(),
        ParsedRepr::Rpm(v) => v.sort_key(),
        ParsedRepr::Maven(v) => v.sort_key(),
    }
}

#[pyclass(name = "Version", module = "anyver", from_py_object)]
#[derive(Debug, Clone)]
pub(crate) struct PyVersion {
    pub(crate) inner: ParsedVersion,
    pub(crate) eco: Ecosystem,
    pub(crate) repr: ParsedRepr,
}

#[pymethods]
impl PyVersion {
    #[new]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    pub(crate) fn new(version: &str, ecosystem: &str) -> PyResult<Self> {
        let inner = parse(version);
        let (eco, repr) = if ecosystem.eq_ignore_ascii_case("auto") {
            let detected = autodetect_ecosystem(version);
            // Auto-detection: fall back to generic if ecosystem parse fails
            match parse_for_ecosystem(detected, version) {
                Ok(r) => (detected, r),
                Err(_) => (Ecosystem::Generic, ParsedRepr::Generic(inner.clone())),
            }
        } else {
            let eco = eco_from_str(ecosystem)?;
            let repr = parse_for_ecosystem(eco, version).map_err(PyValueError::new_err)?;
            (eco, repr)
        };
        Ok(PyVersion { inner, eco, repr })
    }

    fn __richcmp__<'py>(
        &self,
        py: Python<'py>,
        other: &Bound<'py, PyAny>,
        op: CompareOp,
    ) -> PyResult<Bound<'py, PyAny>> {
        let equality = matches!(op, CompareOp::Eq | CompareOp::Ne);
        let ord = if let Ok(v) = other.extract::<PyRef<PyVersion>>() {
            if self.eco != v.eco {
                if equality {
                    return Ok(matches!(op, CompareOp::Ne)
                        .into_pyobject(py)?
                        .to_owned()
                        .into_any());
                }
                return Err(PyTypeError::new_err(
                    "ordering requires matching ecosystems; use compare()",
                ));
            }
            compare_for_ecosystem(self.eco, &self.repr, &v.repr)
        } else if equality {
            return Ok(py.NotImplemented().into_bound(py));
        } else if let Ok(s) = other.extract::<String>() {
            let parsed = parse_for_ecosystem(self.eco, &s).map_err(PyValueError::new_err)?;
            compare_for_ecosystem(self.eco, &self.repr, &parsed)
        } else {
            return Ok(py.NotImplemented().into_bound(py));
        };
        let result = match op {
            CompareOp::Lt => ord == Ordering::Less,
            CompareOp::Le => ord != Ordering::Greater,
            CompareOp::Eq => ord == Ordering::Equal,
            CompareOp::Ne => ord != Ordering::Equal,
            CompareOp::Gt => ord == Ordering::Greater,
            CompareOp::Ge => ord != Ordering::Less,
        };
        Ok(result.into_pyobject(py)?.to_owned().into_any())
    }

    fn __str__(&self) -> &str {
        &self.inner.raw
    }

    fn __repr__(&self) -> String {
        format!("Version('{}')", self.inner.raw)
    }

    fn __len__(&self) -> usize {
        self.inner.segments.len()
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn __getitem__<'py>(&self, py: Python<'py>, idx: isize) -> PyResult<Bound<'py, PyAny>> {
        let len = self.inner.segments.len() as isize;
        let real_idx = if idx < 0 { len + idx } else { idx };
        if real_idx < 0 || real_idx >= len {
            return Err(PyIndexError::new_err(format!(
                "segment index {idx} out of range (version has {len} segments)"
            )));
        }
        Ok(self.inner.segments[real_idx as usize].to_py(py))
    }

    fn __bool__(&self) -> bool {
        !self.inner.segments.is_empty()
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.eco.as_str().hash(&mut hasher);
        match &self.repr {
            ParsedRepr::Rpm(v) => v.hash_key().hash(&mut hasher),
            ParsedRepr::Debian(v) => v.hash_key().hash(&mut hasher),
            ParsedRepr::Maven(v) => v.canonical().hash(&mut hasher),
            _ => version_key(self).hash(&mut hasher),
        }
        hasher.finish()
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.segments(py)?.call_method0("__iter__")
    }

    #[getter]
    fn raw(&self) -> &str {
        &self.inner.raw
    }

    #[getter]
    fn ecosystem(&self) -> &'static str {
        self.eco.as_str()
    }

    #[getter]
    fn epoch(&self) -> u64 {
        self.inner.epoch
    }

    #[getter]
    fn build(&self) -> &str {
        &self.inner.build
    }

    #[getter]
    fn is_prerelease(&self) -> bool {
        self.inner.is_prerelease
    }

    #[getter]
    fn is_postrelease(&self) -> bool {
        self.inner.is_postrelease
    }

    #[getter]
    fn is_stable(&self) -> bool {
        !self.inner.is_prerelease
    }

    #[getter]
    fn count(&self) -> usize {
        self.inner.segments.len()
    }

    #[getter]
    fn major(&self) -> u64 {
        match self.inner.segments.first() {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    #[getter]
    fn minor(&self) -> u64 {
        match self.inner.segments.get(1) {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    #[getter]
    fn patch(&self) -> u64 {
        match self.inner.segments.get(2) {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    fn segments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items: Vec<Bound<'py, PyAny>> =
            self.inner.segments.iter().map(|s| s.to_py(py)).collect();
        PyTuple::new(py, items)
    }

    fn release<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let nums: Vec<Bound<'py, PyAny>> = self
            .inner
            .segments
            .iter()
            .take_while(|s| matches!(s, Seg::Num(_)))
            .map(|s| s.to_py(py))
            .collect();
        PyTuple::new(py, nums)
    }

    #[pyo3(signature = (other, ecosystem = None))]
    fn compare(&self, other: &Bound<'_, PyAny>, ecosystem: Option<&str>) -> PyResult<i32> {
        let eco = match ecosystem {
            None => self.eco,
            Some(s) if s.eq_ignore_ascii_case("auto") => self.eco,
            Some(s) => eco_from_str(s)?,
        };
        let left = if eco == self.eco {
            self.repr.clone()
        } else {
            parse_for_ecosystem(eco, &self.inner.raw).map_err(PyValueError::new_err)?
        };
        let right = extract_parsed_for_ecosystem(other, eco)?;
        Ok(match compare_for_ecosystem(eco, &left, &right) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("raw", &self.inner.raw)?;
        dict.set_item("ecosystem", self.eco.as_str())?;
        dict.set_item("epoch", self.inner.epoch)?;
        dict.set_item("major", self.major())?;
        dict.set_item("minor", self.minor())?;
        dict.set_item("patch", self.patch())?;
        dict.set_item("build", &self.inner.build)?;
        dict.set_item("is_prerelease", self.inner.is_prerelease)?;
        dict.set_item("is_postrelease", self.inner.is_postrelease)?;
        Ok(dict)
    }

    /// Reconstruct a Version from the dict produced by `to_dict()`.
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, data: &Bound<'_, PyDict>) -> PyResult<Self> {
        let raw: String = match data.get_item("raw")? {
            Some(v) => v.extract()?,
            None => return Err(PyValueError::new_err("missing 'raw' key")),
        };
        let eco: String = match data.get_item("ecosystem")? {
            Some(v) => v.extract()?,
            None => "auto".to_string(),
        };
        PyVersion::new(&raw, &eco)
    }

    /// Classmethod alternative to the constructor; raises `ValueError` on invalid input.
    #[classmethod]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    fn parse(_cls: &Bound<'_, PyType>, version: &str, ecosystem: &str) -> PyResult<Self> {
        PyVersion::new(version, ecosystem)
    }

    /// Fallible parse; returns None instead of raising on invalid input.
    #[staticmethod]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    fn try_parse(version: &str, ecosystem: &str) -> Option<Self> {
        PyVersion::new(version, ecosystem).ok()
    }

    /// Pickle support. Restores via `Version(raw, ecosystem)` so the parsed
    /// state is rebuilt identically.
    fn __reduce__<'py>(slf: &Bound<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let inner = slf.borrow();
        let args = PyTuple::new(
            py,
            vec![
                inner.inner.raw.clone().into_pyobject(py)?.into_any(),
                inner.eco.as_str().into_pyobject(py)?.into_any(),
            ],
        )?;
        let type_obj = slf.get_type();
        PyTuple::new(py, vec![type_obj.into_any(), args.into_any()])
    }

    fn sort_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, version_key(self))
    }
}
