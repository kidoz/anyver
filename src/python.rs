use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};
use std::cmp::Ordering;

use crate::parser::{Ecosystem, ParsedVersion, Seg, autodetect_ecosystem, parse, tag_weight};
use crate::strategies::{
    ParsedRepr, cmp_parsed, compare_for_ecosystem, normalized, parse_for_ecosystem,
};

fn eco_from_str(input: &str) -> PyResult<Ecosystem> {
    Ecosystem::from_str(input).map_err(PyValueError::new_err)
}

trait SegToPy {
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

/// Compare two `PyVersion` values using the shared ecosystem if both match,
/// otherwise fall back to generic comparison. Uses pre-parsed repr — no re-parsing.
fn richcmp_ord(a: &PyVersion, b: &PyVersion) -> Ordering {
    if a.eco == b.eco {
        compare_for_ecosystem(a.eco, &a.repr, &b.repr)
    } else {
        cmp_parsed(&a.inner, &b.inner)
    }
}

#[pyclass(name = "Version", module = "anyver", from_py_object)]
#[derive(Debug, Clone)]
pub(crate) struct PyVersion {
    inner: ParsedVersion,
    eco: Ecosystem,
    repr: ParsedRepr,
}

#[pymethods]
impl PyVersion {
    #[new]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    fn new(version: &str, ecosystem: &str) -> PyResult<Self> {
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
        // Hash based on generic segments only (no ecosystem) because
        // cross-ecosystem == falls back to generic comparison.
        // This ensures a == b → hash(a) == hash(b) always holds.
        self.inner.epoch.hash(&mut hasher);
        let segs = normalized(&self.inner.segments);
        for s in segs {
            match s {
                Seg::Num(n) => {
                    0u8.hash(&mut hasher);
                    n.hash(&mut hasher);
                }
                Seg::Text(t) => {
                    1u8.hash(&mut hasher);
                    t.hash(&mut hasher);
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
        // Build a normalized segment list for sort_key encoding:
        // 1. Strip trailing Num(0) from the whole list (standard normalization)
        // 2. Find the first Text segment; strip trailing Num(0) from the
        //    numeric prefix before it. This ensures that Num(0) segments
        //    between the release part and the first tag are transparent,
        //    matching the comparison algorithm's Num(0)-vs-Missing skip.
        let segs = normalized(&self.inner.segments);
        let first_text = segs.iter().position(|s| matches!(s, Seg::Text(_)));
        let (prefix, suffix) = match first_text {
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
        };

        let cap = prefix.len() + suffix.len() + 2;
        let mut tuples: Vec<Bound<'py, PyAny>> = Vec::with_capacity(cap);

        let mk_tuple = |py: Python<'py>, a: i64, b: i64, c: &str| -> PyResult<Bound<'py, PyAny>> {
            Ok(PyTuple::new(
                py,
                vec![
                    a.into_pyobject(py).unwrap().into_any(),
                    b.into_pyobject(py).unwrap().into_any(),
                    c.into_pyobject(py).unwrap().into_any(),
                ],
            )?
            .into_any())
        };

        let sat = |n: u64| -> i64 { i64::try_from(n).unwrap_or(i64::MAX) };
        tuples.push(mk_tuple(py, 2, sat(self.inner.epoch), "")?);

        for seg in prefix.iter().chain(suffix.iter()) {
            let t = match seg {
                Seg::Num(n) => mk_tuple(py, 2, sat(*n), "")?,
                Seg::Text(s) => {
                    let w = i64::from(tag_weight(s).unwrap_or(29));
                    mk_tuple(py, 1, w, s)?
                }
            };
            tuples.push(t);
        }

        tuples.push(mk_tuple(py, 1, 30, "")?);

        PyTuple::new(py, tuples)
    }
}

fn resolve_eco(ecosystem: &str, first: Option<&Bound<'_, PyAny>>) -> PyResult<Ecosystem> {
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

fn extract_raw(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
        Ok(v.inner.raw.clone())
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(s)
    } else {
        Err(PyTypeError::new_err("expected Version or str"))
    }
}

fn resolve_eco_for_obj(ecosystem: &str, obj: &Bound<'_, PyAny>) -> PyResult<Ecosystem> {
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

fn extract_parsed_for_ecosystem(
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
fn version(s: &str, ecosystem: &str) -> PyResult<PyVersion> {
    PyVersion::new(s, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn compare(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, ecosystem: &str) -> PyResult<i32> {
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
fn sort_versions<'py>(
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
fn batch_compare(
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
fn gt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn ge(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn lt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn le(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn eq(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Equal)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn ne(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Equal)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn gte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    ge(a, b, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn lte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    le(a, b, ecosystem)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
fn max_version<'py>(
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
fn min_version<'py>(
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

/// Parse 1-3 numeric parts from a version prefix, treating `x`, `X`, and `*`
/// as wildcards. Returns `Some((parts, wildcard_depth))` where
/// `wildcard_depth` is 3 if no wildcard, else the index at which it appears.
fn parse_num_prefix(input: &str) -> Option<(Vec<u64>, usize)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(3);
    let mut wildcard_at: Option<usize> = None;
    for (i, raw) in s.split('.').enumerate() {
        if i >= 3 {
            break;
        }
        if raw == "x" || raw == "X" || raw == "*" {
            wildcard_at = Some(i);
            break;
        }
        let n: u64 = raw.parse().ok()?;
        parts.push(n);
    }
    if parts.is_empty() && wildcard_at != Some(0) {
        return None;
    }
    Some((parts, wildcard_at.unwrap_or(3)))
}

fn fmt_triple(major: u64, minor: u64, patch: u64) -> String {
    format!("{major}.{minor}.{patch}")
}

/// Expand a shorthand range token (`^1.2.3`, `~1.2`, `1.2.x`, `*`) into a list
/// of basic constraints. Bare constraints (`>=1.0.0`, `==1.0.0`) pass through.
fn expand_shorthand(token: &str) -> Result<Vec<(String, String)>, String> {
    let t = token.trim();
    if t.is_empty() {
        return Err("empty constraint".to_string());
    }
    if t == "*" || t == "x" || t == "X" {
        return Ok(vec![(">=".to_string(), "0.0.0".to_string())]);
    }
    // Caret: ^X.Y.Z — allow changes that don't modify the left-most non-zero.
    if let Some(rest) = t.strip_prefix('^') {
        let (parts, _) =
            parse_num_prefix(rest).ok_or_else(|| format!("invalid caret range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let patch = *parts.get(2).unwrap_or(&0);
        let lower = fmt_triple(major, minor, patch);
        let upper = if major > 0 {
            fmt_triple(major + 1, 0, 0)
        } else if minor > 0 {
            fmt_triple(0, minor + 1, 0)
        } else {
            fmt_triple(0, 0, patch + 1)
        };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // Tilde: ~X.Y.Z — allow patch-level changes. ~X.Y == ~X.Y.0 == >=X.Y.0,<X.(Y+1).0.
    if let Some(rest) = t.strip_prefix('~') {
        let (parts, _) =
            parse_num_prefix(rest).ok_or_else(|| format!("invalid tilde range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let patch = *parts.get(2).unwrap_or(&0);
        let lower = fmt_triple(major, minor, patch);
        let upper = if parts.len() >= 2 {
            fmt_triple(major, minor + 1, 0)
        } else {
            fmt_triple(major + 1, 0, 0)
        };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // x-range: 1.2.x, 1.x, 1.2.*
    if t.contains(".x") || t.contains(".X") || t.contains(".*") {
        let (parts, depth) =
            parse_num_prefix(t).ok_or_else(|| format!("invalid x-range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let lower = fmt_triple(major, minor, 0);
        let upper =
            if depth == 1 { fmt_triple(major + 1, 0, 0) } else { fmt_triple(major, minor + 1, 0) };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // Bare constraint (>=, <=, ==, !=, >, <).
    let (op, v) = parse_constraint(t)?;
    Ok(vec![(op.to_string(), v.to_string())])
}

fn eval_op(op: &str, ord: Ordering) -> bool {
    match op {
        ">=" => ord != Ordering::Less,
        "<=" => ord != Ordering::Greater,
        ">" => ord == Ordering::Greater,
        "<" => ord == Ordering::Less,
        "==" => ord == Ordering::Equal,
        "!=" => ord != Ordering::Equal,
        _ => false,
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
fn satisfies(version: &str, constraint: &str, ecosystem: &str) -> PyResult<bool> {
    // Evaluate OR branches left-to-right.
    for branch in constraint.split("||") {
        let mut branch_ok = true;
        let mut branch_has_clause = false;
        for part in branch.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            branch_has_clause = true;
            let clauses = expand_shorthand(part).map_err(PyValueError::new_err)?;
            for (op, cv) in clauses {
                let ord = compare_str_with_ecosystem(version, &cv, ecosystem)?;
                if !eval_op(&op, ord) {
                    branch_ok = false;
                    break;
                }
            }
            if !branch_ok {
                break;
            }
        }
        if branch_has_clause && branch_ok {
            return Ok(true);
        }
    }
    Ok(false)
}

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
fn stable_versions<'py>(
    _py: Python<'py>,
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
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
fn latest_stable<'py>(
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
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

#[pyfunction]
fn bump_major(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.0.0")
}

#[pyfunction]
fn bump_minor(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.{minor}.0")
}

#[pyfunction]
fn bump_patch(version: &str) -> String {
    let v = parse(version);
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let patch = match v.segments.get(2) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    format!("{major}.{minor}.{patch}")
}

fn release_triple(v: &ParsedVersion) -> (u64, u64, u64) {
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    let patch = match v.segments.get(2) {
        Some(Seg::Num(n)) => *n,
        _ => 0,
    };
    (major, minor, patch)
}

/// Bump the prerelease counter on a version.
///
/// Semantics:
///   - If the version has a prerelease segment with the same `tag`, the
///     first numeric segment following that tag is incremented.
///   - Otherwise the existing prerelease is replaced with `-{tag}.0`.
///   - If no release triple is present, it's filled with `0.0.0`.
#[pyfunction]
#[pyo3(signature = (version, tag = "alpha"))]
fn bump_prerelease(version: &str, tag: &str) -> String {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v);
    let tag_lower = tag.to_ascii_lowercase();

    // Find existing prerelease tag index
    let first_text = v.segments.iter().position(|s| matches!(s, Seg::Text(_)));
    if let Some(pos) = first_text
        && let Seg::Text(existing) = &v.segments[pos]
        && *existing == tag_lower
    {
        let next_num = v.segments.get(pos + 1).and_then(|s| match s {
            Seg::Num(n) => Some(*n),
            Seg::Text(_) => None,
        });
        let n = next_num.map_or(0, |n| n + 1);
        return format!("{major}.{minor}.{patch}-{tag_lower}.{n}");
    }
    format!("{major}.{minor}.{patch}-{tag_lower}.0")
}

/// Return the release form of a version, stripping any prerelease/postrelease
/// tags and build metadata. `1.2.3-alpha.1+build` → `1.2.3`.
#[pyfunction]
fn next_stable(version: &str) -> String {
    let v = parse(version);
    let (major, minor, patch) = release_triple(&v);
    format!("{major}.{minor}.{patch}")
}

#[pymodule]
fn _anyver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVersion>()?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(compare_semver_strict, m)?)?;
    m.add_function(wrap_pyfunction!(sort_versions, m)?)?;
    m.add_function(wrap_pyfunction!(batch_compare, m)?)?;
    m.add_function(wrap_pyfunction!(gt, m)?)?;
    m.add_function(wrap_pyfunction!(ge, m)?)?;
    m.add_function(wrap_pyfunction!(gte, m)?)?;
    m.add_function(wrap_pyfunction!(lt, m)?)?;
    m.add_function(wrap_pyfunction!(le, m)?)?;
    m.add_function(wrap_pyfunction!(lte, m)?)?;
    m.add_function(wrap_pyfunction!(eq, m)?)?;
    m.add_function(wrap_pyfunction!(ne, m)?)?;
    m.add_function(wrap_pyfunction!(max_version, m)?)?;
    m.add_function(wrap_pyfunction!(min_version, m)?)?;
    m.add_function(wrap_pyfunction!(stable_versions, m)?)?;
    m.add_function(wrap_pyfunction!(latest_stable, m)?)?;
    m.add_function(wrap_pyfunction!(satisfies, m)?)?;
    m.add_function(wrap_pyfunction!(bump_major, m)?)?;
    m.add_function(wrap_pyfunction!(bump_minor, m)?)?;
    m.add_function(wrap_pyfunction!(bump_patch, m)?)?;
    m.add_function(wrap_pyfunction!(bump_prerelease, m)?)?;
    m.add_function(wrap_pyfunction!(next_stable, m)?)?;
    Ok(())
}
