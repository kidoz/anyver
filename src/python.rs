use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
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

/// Resolve comparison ordering between two `PyVersion` values.
/// When both share the same non-generic ecosystem, use that ecosystem's
/// comparator. Otherwise fall back to generic comparison.
fn richcmp_ord(a: &PyVersion, b: &PyVersion) -> Ordering {
    let eco = if a.eco == b.eco { a.eco } else { Ecosystem::Generic };
    if eco == Ecosystem::Generic {
        cmp_parsed(&a.inner, &b.inner)
    } else {
        // Re-parse through the ecosystem strategy for correct semantics
        let Ok(left) = parse_for_ecosystem(eco, &a.inner.raw) else {
            return cmp_parsed(&a.inner, &b.inner);
        };
        let Ok(right) = parse_for_ecosystem(eco, &b.inner.raw) else {
            return cmp_parsed(&a.inner, &b.inner);
        };
        compare_for_ecosystem(eco, &left, &right)
    }
}

#[pyclass(name = "Version", from_py_object)]
#[derive(Debug, Clone)]
pub(crate) struct PyVersion {
    inner: ParsedVersion,
    eco: Ecosystem,
}

#[pymethods]
impl PyVersion {
    #[new]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    fn new(version: &str, ecosystem: &str) -> PyResult<Self> {
        let eco = if ecosystem.eq_ignore_ascii_case("auto") {
            autodetect_ecosystem(version)
        } else {
            eco_from_str(ecosystem)?
        };
        Ok(PyVersion { inner: parse(version), eco })
    }

    fn __richcmp__(&self, other: &PyVersion, op: CompareOp) -> bool {
        let ord = richcmp_ord(self, other);
        match op {
            CompareOp::Lt => ord == Ordering::Less,
            CompareOp::Le => ord != Ordering::Greater,
            CompareOp::Eq => ord == Ordering::Equal,
            CompareOp::Ne => ord != Ordering::Equal,
            CompareOp::Gt => ord == Ordering::Greater,
            CompareOp::Ge => ord != Ordering::Less,
        }
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
        // Hash the ecosystem so that versions from different ecosystems
        // with the same raw string but different comparison semantics
        // don't collide.
        self.eco.hash(&mut hasher);
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
        let left = parse_for_ecosystem(eco, &self.inner.raw).map_err(PyValueError::new_err)?;
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
        dict.set_item("epoch", self.inner.epoch)?;
        dict.set_item("major", self.major())?;
        dict.set_item("minor", self.minor())?;
        dict.set_item("patch", self.patch())?;
        dict.set_item("build", &self.inner.build)?;
        dict.set_item("is_prerelease", self.inner.is_prerelease)?;
        dict.set_item("is_postrelease", self.inner.is_postrelease)?;
        Ok(dict)
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

        #[allow(clippy::cast_possible_wrap)]
        tuples.push(mk_tuple(py, 2, self.inner.epoch as i64, "")?);

        for seg in prefix.iter().chain(suffix.iter()) {
            #[allow(clippy::cast_possible_wrap)]
            let t = match seg {
                Seg::Num(n) => mk_tuple(py, 2, *n as i64, "")?,
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
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        eco_from_str(ecosystem)?
    };
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
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        eco_from_str(ecosystem)?
    };
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
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        eco_from_str(ecosystem)?
    };
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

#[pyfunction(signature = (version, constraint, ecosystem = "generic"))]
fn satisfies(version: &str, constraint: &str, ecosystem: &str) -> PyResult<bool> {
    for part in constraint.split(',') {
        let (op, cv) = parse_constraint(part).map_err(PyValueError::new_err)?;
        let ord = compare_str_with_ecosystem(version, cv, ecosystem)?;
        let ok = match op {
            ">=" => ord != Ordering::Less,
            "<=" => ord != Ordering::Greater,
            ">" => ord == Ordering::Greater,
            "<" => ord == Ordering::Less,
            "==" => ord == Ordering::Equal,
            "!=" => ord != Ordering::Equal,
            _ => unreachable!(),
        };
        if !ok {
            return Ok(false);
        }
    }
    Ok(true)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
fn stable_versions<'py>(
    _py: Python<'py>,
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        eco_from_str(ecosystem)?
    };
    versions
        .into_iter()
        .filter_map(|obj| {
            let raw = if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
                v.inner.raw.clone()
            } else if let Ok(s) = obj.extract::<String>() {
                s
            } else {
                return Some(Err(PyTypeError::new_err("expected Version or str")));
            };
            let parsed = parse(&raw);
            if parsed.is_prerelease {
                None
            } else {
                // Validate against ecosystem if needed
                match parse_for_ecosystem(eco, &raw) {
                    Ok(_) => Some(Ok(obj)),
                    Err(e) => Some(Err(PyValueError::new_err(e))),
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
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        eco_from_str(ecosystem)?
    };
    let mut stable: Vec<(ParsedRepr, Bound<'py, PyAny>)> = Vec::new();
    for obj in versions {
        let raw = if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
            v.inner.raw.clone()
        } else if let Ok(s) = obj.extract::<String>() {
            s
        } else {
            return Err(PyTypeError::new_err("expected Version or str"));
        };
        let parsed = parse(&raw);
        if !parsed.is_prerelease {
            let repr = extract_parsed_for_ecosystem(&obj, eco)?;
            stable.push((repr, obj));
        }
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

#[pymodule]
fn anyver(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    Ok(())
}
