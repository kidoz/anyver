//! Per-ecosystem version parsing and comparison strategies.
//!
//! This `mod.rs` is a thin facade. The implementation is split by
//! responsibility — see `.agents/contexts/conventions/RUST_CODE_DECOMPOSITION.md`:
//!
//! - [`generic`]    — generic segment comparator (`cmp_parsed`, `cmp_segments`)
//! - [`semver`]     — strict `SemVer` (also Go, crates.io, Hex, Swift, npm)
//! - [`pep440`]     — PEP 440 incl. local-version labels
//! - [`debian`]     — dpkg `verrevcmp` (and the shared epoch splitter)
//! - [`rpm`]        — RPM `rpmvercmp`
//! - [`maven`]      — Apache Maven `ComparableVersion`
//! - [`validators`] — per-ecosystem input preconditions
//! - [`registry`]   — `VersionStrategy` trait, `ParsedRepr`, ecosystem dispatch
//!
//! The per-ecosystem modules are used only by [`registry`] (via `super::`), so
//! the facade re-exports just the surface the rest of the crate links against:
//! the dispatch entry points and `ParsedRepr` from [`registry`], plus the two
//! generic comparator helpers `python.rs` calls directly. The
//! `crate::strategies::<name>` paths used elsewhere are unchanged by the split.

mod debian;
mod generic;
mod maven;
mod pep440;
mod registry;
mod rpm;
mod semver;
mod validators;

pub(crate) use generic::*;
pub(crate) use registry::*;

// `parser`/`debian`/`rpm`/`semver` internals are otherwise reached only through
// the strategy dispatch, but the crate's unit tests (`mod tests`, via
// `crate::strategies::*`) call a handful of the raw comparators directly.
#[cfg(test)]
pub(crate) use debian::verrevcmp;
#[cfg(test)]
pub(crate) use maven::parse_maven;
#[cfg(test)]
pub(crate) use rpm::rpmverscmp;
#[cfg(test)]
pub(crate) use semver::{cmp_semver_strict, parse_semver_strict};
