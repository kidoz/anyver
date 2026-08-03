//! Regression tests for the range engine. Each block pins behaviour that was
//! previously wrong in a way that produced a *silent* answer rather than an
//! error.

use crate::parser::Ecosystem;
use crate::ranges::RangeSet;

// ====================================================================
// Empty ranges must not match everything
// ====================================================================

#[test]
fn test_empty_range_is_rejected() {
    for input in ["", "   ", ",", " , ", "||"] {
        let err = RangeSet::parse(input).unwrap_err();
        assert!(err.contains("empty range"), "{input:?} should be rejected, got: {err}");
    }
}

#[test]
fn test_empty_branch_in_or_is_rejected() {
    assert!(RangeSet::parse(">=1.0 || ").is_err());
}

#[test]
fn test_star_still_matches_everything() {
    let range = RangeSet::parse("*").unwrap();
    assert!(range.contains(Ecosystem::Generic, "0.0.1").unwrap());
    assert!(range.contains(Ecosystem::Generic, "99.0.0").unwrap());
}
