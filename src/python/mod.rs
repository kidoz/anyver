//! `PyO3` bindings for `anyver` — the `anyver` Python extension module.
//!
//! This `mod.rs` is a thin facade. The bindings are split by responsibility —
//! see `.agents/contexts/conventions/RUST_CODE_DECOMPOSITION.md`:
//!
//! - [`conv`]        — ecosystem resolution and `Version`/`str` extraction
//! - [`pyversion`]   — the `Version` class (`PyVersion`)
//! - [`compare`]     — comparison functions and predicates
//! - [`constraints`] — constraint parsing and `satisfies`
//! - [`stable`]      — stability-aware selectors
//! - [`bump`]        — version-bumping helpers
//!
//! Only the surface the crate's unit tests reach (`crate::python::*`) is
//! re-exported below; everything else is registered into the module here.

use pyo3::prelude::*;

mod bump;
mod compare;
mod constraints;
mod conv;
mod pyversion;
mod stable;

// Reached only by the crate's unit tests (`mod tests`, via `crate::python::*`).
#[cfg(test)]
pub(crate) use compare::compare_str_with_ecosystem;
#[cfg(test)]
pub(crate) use constraints::parse_constraint;

#[pymodule]
fn _anyver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<pyversion::PyVersion>()?;
    m.add_function(wrap_pyfunction!(compare::version, m)?)?;
    m.add_function(wrap_pyfunction!(compare::compare, m)?)?;
    m.add_function(wrap_pyfunction!(compare::compare_semver_strict, m)?)?;
    m.add_function(wrap_pyfunction!(compare::sort_versions, m)?)?;
    m.add_function(wrap_pyfunction!(compare::batch_compare, m)?)?;
    m.add_function(wrap_pyfunction!(compare::gt, m)?)?;
    m.add_function(wrap_pyfunction!(compare::ge, m)?)?;
    m.add_function(wrap_pyfunction!(compare::gte, m)?)?;
    m.add_function(wrap_pyfunction!(compare::lt, m)?)?;
    m.add_function(wrap_pyfunction!(compare::le, m)?)?;
    m.add_function(wrap_pyfunction!(compare::lte, m)?)?;
    m.add_function(wrap_pyfunction!(compare::eq, m)?)?;
    m.add_function(wrap_pyfunction!(compare::ne, m)?)?;
    m.add_function(wrap_pyfunction!(compare::max_version, m)?)?;
    m.add_function(wrap_pyfunction!(compare::min_version, m)?)?;
    m.add_function(wrap_pyfunction!(stable::stable_versions, m)?)?;
    m.add_function(wrap_pyfunction!(stable::latest_stable, m)?)?;
    m.add_function(wrap_pyfunction!(constraints::satisfies, m)?)?;
    m.add_function(wrap_pyfunction!(bump::bump_major, m)?)?;
    m.add_function(wrap_pyfunction!(bump::bump_minor, m)?)?;
    m.add_function(wrap_pyfunction!(bump::bump_patch, m)?)?;
    m.add_function(wrap_pyfunction!(bump::bump_prerelease, m)?)?;
    m.add_function(wrap_pyfunction!(bump::next_stable, m)?)?;
    Ok(())
}
