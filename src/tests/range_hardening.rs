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

// ====================================================================
// Intersection must not depend on clause order
// ====================================================================

fn intersects(left: &str, right: &str) -> bool {
    RangeSet::parse(left)
        .unwrap()
        .intersects(Ecosystem::Generic, &RangeSet::parse(right).unwrap())
        .unwrap()
}

#[test]
fn test_intersect_folds_repeated_lower_bounds() {
    // Both spellings mean `>=2.0`, which cannot overlap `<1.5`.
    assert!(!intersects(">=2.0,>=1.0", "<1.5"));
    assert!(!intersects(">=1.0,>=2.0", "<1.5"));
}

#[test]
fn test_intersect_folds_repeated_upper_bounds() {
    assert!(!intersects("<1.0,<2.0", ">=1.5"));
    assert!(!intersects("<2.0,<1.0", ">=1.5"));
}

#[test]
fn test_intersect_still_finds_real_overlap() {
    assert!(intersects(">=1.0,<2.0", ">=1.5,<3.0"));
    assert!(intersects(">=1.0", "<=1.0"));
}

#[test]
fn test_intersect_respects_not_equals_on_a_point_overlap() {
    // The overlap is exactly 1.0, which the `!=` excludes.
    assert!(!intersects(">=1.0,<=1.0", "!=1.0"));
    assert!(!intersects("!=1.0", ">=1.0,<=1.0"));
    // A wider overlap survives the same exclusion.
    assert!(intersects(">=1.0,<=2.0", "!=1.0"));
}
