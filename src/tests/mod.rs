//! Unit tests, split by topic per
//! `.agents/contexts/conventions/RUST_CODE_DECOMPOSITION.md`. This `mod.rs`
//! holds the shared imports and the `cmpg`/`assert_chain` helpers; each
//! submodule pulls them in via `use super::*`.

use crate::parser::*;
use crate::python::*;
use crate::strategies::*;
use std::cmp::Ordering;
use std::cmp::Ordering::*;

fn cmpg(a: &str, b: &str) -> Ordering {
    cmp_parsed(&parse(a), &parse(b))
}

fn assert_chain(chain: &[&str], eco: &str) {
    for i in 0..chain.len() - 1 {
        assert_eq!(
            compare_str_with_ecosystem(chain[i], chain[i + 1], eco).unwrap(),
            Less,
            "{eco}: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

mod autodetect;
mod basics;
mod constraints;
mod datasets_native;
mod datasets_semver_family;
mod dispatch;
mod ecosystem_expansion;
mod maven;
mod maven_aliases;
mod maven_comparable;
mod parser_semver;
mod pep440_local;
mod python_api;
mod range_hardening;
