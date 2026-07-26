//! The `Version` Python class (`PyVersion`) and its methods: construction,
//! rich comparison, accessors, segment/release views, dict and pickle
//! round-tripping, and the `sort_key` encoding.

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::cmp::Ordering;

use crate::parser::{Ecosystem, ParsedVersion, Seg, autodetect_ecosystem, parse, tag_weight};
use crate::strategies::{
    ParsedRepr, cmp_parsed, compare_for_ecosystem, normalized, parse_for_ecosystem,
};

use super::conv::{SegToPy, eco_from_str, extract_parsed_for_ecosystem};

/// Compare two `PyVersion` values using the shared ecosystem if both match,
/// otherwise fall back to generic comparison. Uses pre-parsed repr — no re-parsing.
fn richcmp_ord(a: &PyVersion, b: &PyVersion) -> Ordering {
    if a.eco == b.eco {
        compare_for_ecosystem(a.eco, &a.repr, &b.repr)
    } else {
        cmp_parsed(&a.inner, &b.inner)
    }
}

/// A single `(rank, value, text)` element of a `sort_key()` tuple.
fn key_tuple<'py>(py: Python<'py>, a: i64, b: i64, c: &str) -> PyResult<Bound<'py, PyAny>> {
    Ok(PyTuple::new(
        py,
        vec![
            a.into_pyobject(py)?.into_any(),
            b.into_pyobject(py)?.into_any(),
            c.into_pyobject(py)?.into_any(),
        ],
    )?
    .into_any())
}

/// Split a segment list into the numeric prefix and the tag suffix, dropping
/// the segments both `cmp_segments` and `__hash__` treat as transparent:
/// trailing `Num(0)` (via `normalized`) and the zeros sitting just before the
/// first tag. Shared by `sort_key()` and `__hash__` so the two never disagree.
fn canonical_parts(raw_segs: &[Seg]) -> (&[Seg], &[Seg]) {
    let segs = normalized(raw_segs);
    match segs.iter().position(|s| matches!(s, Seg::Text(_))) {
        Some(pos) => {
            let mut end = pos;
            while end > 0 {
                if let Seg::Num(0) = segs[end - 1] {
                    end -= 1;
                } else {
                    break;
                }
            }
            (&segs[..end], &segs[pos..])
        }
        None => (segs, &[][..]),
    }
}

/// Encode a segment list into `sort_key()` tuples isomorphic with
/// `cmp_segments`: emit `(2, n, "")` for numbers and `(1, weight, text)` for
/// tags (numbers outrank tags), then a trailing `(1, 30, "")` release-boundary
/// sentinel so a bare release sorts above any pre-release tag and below any
/// post-release tag.
fn encode_segments<'py>(py: Python<'py>, raw_segs: &[Seg]) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let sat = |n: u64| -> i64 { i64::try_from(n).unwrap_or(i64::MAX) };
    let (prefix, suffix) = canonical_parts(raw_segs);
    let mut out: Vec<Bound<'py, PyAny>> = Vec::with_capacity(prefix.len() + suffix.len() + 1);
    for seg in prefix.iter().chain(suffix.iter()) {
        out.push(match seg {
            Seg::Num(n) => key_tuple(py, 2, sat(*n), "")?,
            Seg::Text(s) => key_tuple(py, 1, i64::from(tag_weight(s).unwrap_or(29)), s)?,
        });
    }
    out.push(key_tuple(py, 1, 30, "")?);
    Ok(out)
}

fn encode_local_segments<'py>(
    py: Python<'py>,
    raw_segs: &[Seg],
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let sat = |n: u64| -> i64 { i64::try_from(n).unwrap_or(i64::MAX) };
    let segs = normalized(raw_segs);
    let mut out: Vec<Bound<'py, PyAny>> = Vec::with_capacity(segs.len() + 1);
    for seg in segs {
        out.push(match seg {
            Seg::Num(n) => key_tuple(py, 2, sat(*n), "")?,
            Seg::Text(s) => key_tuple(py, 1, 29, s)?,
        });
    }
    out.push(key_tuple(py, 1, 30, "")?);
    Ok(out)
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
        let ord = if let Ok(v) = other.extract::<PyRef<PyVersion>>() {
            richcmp_ord(self, &v)
        } else if let Ok(s) = other.extract::<String>() {
            // Parse the str with this Version's ecosystem, so the left-hand
            // side's declared ecosystem wins — matching Version.compare(str).
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
        // Hash the generic view, not the ecosystem one, because cross-ecosystem
        // `==` falls back to generic comparison — so `a == b → hash(a) ==
        // hash(b)` needs the generic segments on both sides.
        //
        // Maven is the exception: it equates versions the raw generic parse
        // keeps apart (`1.0-1 == 1.0.0-1`), so its *canonical* form is hashed
        // instead. That form re-parses to the same generic segments a plain
        // release would produce, so the cross-ecosystem case still agrees.
        let canonical;
        let parsed = match &self.repr {
            ParsedRepr::Maven(m) => {
                canonical = parse(&m.canonical());
                &canonical
            }
            _ => &self.inner,
        };
        parsed.epoch.hash(&mut hasher);
        let (prefix, suffix) = canonical_parts(&parsed.segments);
        // Release aliases (`final`/`ga`/`release`, weight 30) are transparent:
        // the comparator ignores them when trailing and Maven's canonical form
        // drops them outright, so `1.0-final7` must hash like `1.0.7`.
        let transparent = |s: &&Seg| !matches!(s, Seg::Text(t) if tag_weight(t) == Some(30));
        for s in prefix.iter().chain(suffix.iter()).filter(transparent) {
            match s {
                Seg::Num(n) => {
                    0u8.hash(&mut hasher);
                    n.hash(&mut hasher);
                }
                // Tags hash by weight, not spelling: `cmp_two` compares known
                // tags by weight, so `1.0-a` and `1.0-alpha` are equal and must
                // hash equal. Unknown tags keep comparing (and hashing) by text.
                Seg::Text(t) => {
                    if let Some(w) = tag_weight(t) {
                        1u8.hash(&mut hasher);
                        w.hash(&mut hasher);
                    } else {
                        2u8.hash(&mut hasher);
                        t.hash(&mut hasher);
                    }
                }
            }
        }
        hasher.finish()
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
        // The key encodes the generic segment ordering exactly (see
        // `encode_segments`), prefixed by the epoch. For PEP 440 the local
        // label (`+LOCAL`) is appended after the public encoding so the key
        // stays isomorphic with `cmp_pep440`: a label-bearing version yields a
        // strictly longer key (None < Some), and two labels then compare via
        // the same segment encoding (cmp_segments).
        let sat = |n: u64| -> i64 { i64::try_from(n).unwrap_or(i64::MAX) };
        let mut tuples: Vec<Bound<'py, PyAny>> = Vec::new();
        tuples.push(key_tuple(py, 2, sat(self.inner.epoch), "")?);
        tuples.extend(encode_segments(py, &self.inner.segments)?);
        if let ParsedRepr::Pep440(p) = &self.repr
            && let Some(local) = p.local()
        {
            tuples.extend(encode_local_segments(py, local)?);
        }
        PyTuple::new(py, tuples)
    }
}
