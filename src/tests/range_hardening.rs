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
        let err = RangeSet::parse(input, Ecosystem::Generic).unwrap_err();
        assert!(err.contains("empty range"), "{input:?} should be rejected, got: {err}");
    }
}

#[test]
fn test_empty_branch_in_or_is_rejected() {
    assert!(RangeSet::parse(">=1.0 || ", Ecosystem::Generic).is_err());
}

#[test]
fn test_star_still_matches_everything() {
    let range = RangeSet::parse("*", Ecosystem::Generic).unwrap();
    assert!(range.contains(Ecosystem::Generic, "0.0.1").unwrap());
    assert!(range.contains(Ecosystem::Generic, "99.0.0").unwrap());
}

// ====================================================================
// Intersection must not depend on clause order
// ====================================================================

fn intersects(left: &str, right: &str) -> bool {
    RangeSet::parse(left, Ecosystem::Generic)
        .unwrap()
        .intersects(Ecosystem::Generic, &RangeSet::parse(right, Ecosystem::Generic).unwrap())
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
    RangeSet::parse(range, Ecosystem::Generic)
        .unwrap()
        .contains(Ecosystem::Generic, version)
        .unwrap()
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
        assert!(crate::ranges::RangeSet::parse(input, Ecosystem::Generic).is_err(), "{input}");
    }
}

#[test]
fn test_npm_prerelease_lower_bound_and_admission() {
    let range = crate::ranges::RangeSet::parse("^1.2.3-beta.1", Ecosystem::Npm).unwrap();
    assert!(range.contains(Ecosystem::Npm, "1.2.3-beta.1").unwrap());
    assert!(!range.contains(Ecosystem::Npm, "1.2.4-beta.1").unwrap());
    let range = crate::ranges::RangeSet::parse(">=1.0.0", Ecosystem::Npm).unwrap();
    assert!(!range.contains(Ecosystem::Npm, "1.1.0-alpha").unwrap());
    assert!(range.contains_bounds(Ecosystem::Npm, "1.1.0-alpha").unwrap());
}

// ====================================================================
// Wildcards at any depth and with an epoch
// ====================================================================

fn contains_in(ecosystem: Ecosystem, range: &str, version: &str) -> bool {
    RangeSet::parse(range, ecosystem)
        .unwrap_or_else(|error| panic!("{range:?}: {error}"))
        .contains(ecosystem, version)
        .unwrap()
}

#[test]
fn test_wildcard_beyond_the_release_triple() {
    // `take(3)` used to miss a wildcard in the fourth component, so these
    // raised `invalid comparator` instead of matching.
    assert!(contains("==1.2.3.*", "1.2.3"));
    assert!(contains("==1.2.3.*", "1.2.3.9"));
    assert!(!contains("==1.2.3.*", "1.2.4"));
    assert!(!contains("==1.2.3.*", "1.2"));
    assert!(contains("1.2.3.4.x", "1.2.3.4.7"));
    assert!(!contains("1.2.3.4.x", "1.2.3.5"));
    assert!(contains(">=1.2.3.4.x", "1.2.3.4"));
    assert!(!contains("<=1.2.3.4.x", "1.2.3.5"));
}

#[test]
fn test_fourth_component_wildcard_narrows_to_its_prefix() {
    // `1.2.3.x` leaves only the fourth component free: `>=1.2.3,<1.2.4`.
    let nuget = Ecosystem::Nuget;
    assert!(contains_in(nuget, "1.2.3.x", "1.2.3.4"));
    assert!(contains_in(nuget, "1.2.3.x", "1.2.3"));
    assert!(!contains_in(nuget, "1.2.3.x", "1.2.4"));
    assert!(contains_in(nuget, ">1.2.3.x", "1.2.4"));
    assert!(!contains_in(nuget, ">1.2.3.x", "1.2.3.9"));
    let npm = Ecosystem::Npm;
    assert!(contains_in(npm, "1.2.3.x", "1.2.3"));
    assert!(!contains_in(npm, "1.2.3.x", "1.2.4"));
    assert!(!contains_in(npm, "1.2.3.x", "1.2.0"));
}

#[test]
fn test_wildcard_keeps_the_epoch() {
    let pep440 = Ecosystem::Pep440;
    assert!(contains_in(pep440, ">=1!1.2.*", "1!1.2.9"));
    assert!(!contains_in(pep440, ">=1!1.2.*", "1.3.0"));
    let rpm = Ecosystem::Rpm;
    assert!(contains_in(rpm, "1:2.x", "1:2.5-1"));
    assert!(!contains_in(rpm, "1:2.x", "1:3.1-1"));
    assert!(!contains_in(rpm, "1:2.x", "2.5-1"));
}

// ====================================================================
// PEP 440 prefix matching
// ====================================================================

#[test]
fn test_pep440_prefix_match_includes_boundary_prereleases() {
    // `==1.2.*` covers every 1.2.x release, including the pre- and
    // development releases of 1.2 itself.
    let pep440 = Ecosystem::Pep440;
    for version in ["1.2.0a1", "1.2.0rc1", "1.2.0.dev1", "1.2", "1.2.9", "1.2.0.post1"] {
        assert!(contains_in(pep440, "==1.2.*", version), "{version}");
    }
    for version in ["1.1.9", "1.1.post1", "1.3.dev0", "1.3.0a1"] {
        assert!(!contains_in(pep440, "==1.2.*", version), "{version}");
    }
    assert!(contains_in(pep440, "==1.2.0.*", "1.2.0rc1"));
    assert!(!contains_in(pep440, "==1.2.0.*", "1.2.1.dev0"));
    assert!(contains_in(pep440, "==1!1.2.*", "1!1.2.0a1"));
    assert!(!contains_in(pep440, "==1!1.2.*", "1.2.0"));
}

#[test]
fn test_prefix_match_is_pep440_only() {
    // Elsewhere the wildcarded window still opens at the filled-in release,
    // so a prerelease of that release stays outside it.
    assert!(contains("==1.2.*", "1.2.0"));
    assert!(!contains("==1.2.*", "1.2.0-rc.1"));
    assert!(!contains("==1.2.*", "1.3.0"));
}

// ====================================================================
// npm syntax that used to be rejected
// ====================================================================

#[test]
fn test_npm_bare_version_is_an_exact_match() {
    let npm = Ecosystem::Npm;
    assert!(contains_in(npm, "1.2.3", "1.2.3"));
    assert!(!contains_in(npm, "1.2.3", "1.2.4"));
    assert!(contains_in(npm, "=1.2.3", "1.2.3"));
    assert!(contains_in(npm, "=v1.2.3", "1.2.3"));
    assert!(contains_in(npm, "=1.2.x", "1.2.9"));
    assert!(contains_in(npm, "1.2.3 || >=2.0.0", "2.5.0"));
    // Other ecosystems keep requiring an operator.
    assert!(RangeSet::parse("1.2.3", Ecosystem::Generic).is_err());
}

#[test]
fn test_npm_hyphen_ranges() {
    let npm = Ecosystem::Npm;
    assert!(contains_in(npm, "1.2.3 - 2.3.4", "1.2.3"));
    assert!(contains_in(npm, "1.2.3 - 2.3.4", "2.3.4"));
    assert!(!contains_in(npm, "1.2.3 - 2.3.4", "2.3.5"));
    assert!(!contains_in(npm, "1.2.3 - 2.3.4", "1.2.2"));
    // A partial upper bound stops short of the next release.
    assert!(contains_in(npm, "1.2.3 - 2.3", "2.3.9"));
    assert!(!contains_in(npm, "1.2.3 - 2.3", "2.4.0"));
    assert!(!contains_in(npm, "1.2.3 - 2", "3.0.0"));
    // The lower bound admits its own prerelease, as any npm comparator does.
    assert!(contains_in(npm, "1.2.3-pre - 2.3.4", "1.2.3-pre"));
    assert!(!contains_in(npm, "1.2.3 - 2.3.4", "2.3.5-pre"));
}

#[test]
fn test_npm_partial_versions_are_x_ranges() {
    // npm reads a bare partial version as an x-range, so it must not reach the
    // strict three-component parser as an exact bound.
    let npm = Ecosystem::Npm;
    for spec in ["1.2", "v1.2", "=1.2", "=V1.2"] {
        assert!(contains_in(npm, spec, "1.2.0"), "{spec}");
        assert!(contains_in(npm, spec, "1.2.9"), "{spec}");
        assert!(!contains_in(npm, spec, "1.3.0"), "{spec}");
        assert!(!contains_in(npm, spec, "1.1.9"), "{spec}");
    }
    assert!(contains_in(npm, "1", "1.9.9"));
    assert!(!contains_in(npm, "1", "2.0.0"));
    assert!(!contains_in(npm, "1", "0.9.9"));
    // A complete bare version stays an exact match.
    assert!(contains_in(npm, "1.2.0", "1.2.0"));
    assert!(!contains_in(npm, "1.2.0", "1.2.1"));
}

#[test]
fn test_npm_hyphen_ranges_pad_partial_lower_bounds() {
    let npm = Ecosystem::Npm;
    for spec in ["1.2 - 2.3.4", "v1.2 - 2.3.4"] {
        assert!(contains_in(npm, spec, "1.2.0"), "{spec}");
        assert!(contains_in(npm, spec, "1.5.0"), "{spec}");
        assert!(!contains_in(npm, spec, "1.1.9"), "{spec}");
        assert!(!contains_in(npm, spec, "2.3.5"), "{spec}");
    }
    assert!(contains_in(npm, "1 - 2.3.4", "1.0.0"));
    assert!(!contains_in(npm, "1 - 2.3.4", "0.9.9"));
    // A `v` prefix on a partial upper bound reads the same way.
    assert!(contains_in(npm, "1.2.3 - v2.3", "2.3.9"));
    assert!(!contains_in(npm, "1.2.3 - v2.3", "2.4.0"));
}

#[test]
fn test_wildcards_accept_a_v_prefix() {
    assert!(contains_in(Ecosystem::Npm, "v1.2.x", "1.2.5"));
    assert!(contains_in(Ecosystem::Npm, ">=v1.2.x", "1.2.5"));
    assert!(!contains_in(Ecosystem::Npm, ">=v1.2.x", "1.1.9"));
    assert!(contains_in(Ecosystem::Go, "v1.2.x", "v1.2.5"));
    assert!(contains_in(Ecosystem::Pep440, "==v1.2.*", "1.2.9"));
}

// ====================================================================
// Mistyped operators must not become text
// ====================================================================

#[test]
fn test_operator_inside_a_comparator_version_is_an_error() {
    for input in [">>1.0", "<<2.0", ">=1.0 <<2.0", "<=<2.0", "==1.0=2"] {
        assert!(
            RangeSet::parse(input, Ecosystem::Generic).is_err(),
            "{input:?} must not be read as a version"
        );
    }
}
