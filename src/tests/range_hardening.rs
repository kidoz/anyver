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

// ====================================================================
// Operator + wildcard (`>=1.2.x`)
// ====================================================================

fn contains(range: &str, version: &str) -> bool {
    RangeSet::parse(range).unwrap().contains(Ecosystem::Generic, version).unwrap()
}

#[test]
fn test_ge_wildcard_lands_on_the_lower_edge() {
    assert!(contains(">=1.2.x", "1.2.0"));
    assert!(contains(">=1.2.x", "1.5.0"));
    assert!(!contains(">=1.2.x", "1.1.9"));
}

#[test]
fn test_gt_wildcard_clears_the_whole_window() {
    // `>1.2.x` is "above everything in 1.2", i.e. `>=1.3.0`.
    assert!(!contains(">1.2.x", "1.2.9"));
    assert!(contains(">1.2.x", "1.3.0"));
}

#[test]
fn test_le_wildcard_includes_the_whole_window() {
    assert!(contains("<=1.2.x", "1.2.9"));
    assert!(!contains("<=1.2.x", "1.3.0"));
}

#[test]
fn test_lt_wildcard_excludes_the_whole_window() {
    assert!(contains("<1.2.x", "1.1.0"));
    assert!(!contains("<1.2.x", "1.2.0"));
}

#[test]
fn test_bare_wildcard_range_unchanged() {
    assert!(contains("1.2.x", "1.2.5"));
    assert!(!contains("1.2.x", "1.3.0"));
}

#[test]
fn test_range_increment_overflow_is_an_error() {
    for input in [
        "18446744073709551615.x",
        "1.18446744073709551615.x",
        "^18446744073709551615.0.0",
        "~1.18446744073709551615.0",
        "^0.0.18446744073709551615",
    ] {
        assert!(crate::ranges::RangeSet::parse(input).is_err(), "{input}");
    }
}

#[test]
fn test_npm_prerelease_lower_bound_and_admission() {
    let range = crate::ranges::RangeSet::parse("^1.2.3-beta.1").unwrap();
    assert!(range.contains(Ecosystem::Npm, "1.2.3-beta.1").unwrap());
    assert!(!range.contains(Ecosystem::Npm, "1.2.4-beta.1").unwrap());
    let range = crate::ranges::RangeSet::parse(">=1.0.0").unwrap();
    assert!(!range.contains(Ecosystem::Npm, "1.1.0-alpha").unwrap());
    assert!(range.contains_bounds(Ecosystem::Npm, "1.1.0-alpha").unwrap());
}
