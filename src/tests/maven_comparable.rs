//! `ecosystem="maven"` parity with Apache Maven's `ComparableVersion`.
//!
//! Expectations come from `ComparableVersion` (maven-artifact 3.9.16), with
//! the upstream cycle checked directly against 3.10.0. They do not come from
//! the generic comparator — see `src/strategies/maven.rs`. The cases in `maven.rs` cover the *generic*
//! comparator's Maven-flavoured behaviour, which differs on purpose.

use super::*;

fn mvn(a: &str, b: &str) -> Ordering {
    compare_str_with_ecosystem(a, b, "maven").unwrap()
}

// ---- Known-qualifier ranking ----
#[test]
fn test_cv_qualifier_chain() {
    assert_chain(
        &[
            "1.0-alpha-1",
            "1.0-beta-1",
            "1.0-milestone-1",
            "1.0-rc-1",
            "1.0-SNAPSHOT",
            "1.0",
            "1.0-sp-1",
        ],
        "maven",
    );
}
#[test]
fn test_cv_cr_aliases_rc() {
    assert_eq!(mvn("1.0-cr-1", "1.0-rc-1"), Equal);
}
#[test]
fn test_cv_release_aliases() {
    assert_eq!(mvn("1.0-ga", "1.0"), Equal);
    assert_eq!(mvn("1.0-final", "1.0"), Equal);
    assert_eq!(mvn("1.0-release", "1.0"), Equal);
}
#[test]
fn test_cv_release_alias_below_sp() {
    assert_eq!(mvn("1.0-final", "1.0-sp-1"), Less);
}
#[test]
fn test_cv_case_insensitive() {
    assert_eq!(mvn("1.0-SNAPSHOT", "1.0-snapshot"), Equal);
    assert_eq!(mvn("1.0-RC-1", "1.0-rc-1"), Equal);
}

// ---- Unknown qualifiers outrank every known one, and the release ----
#[test]
fn test_cv_unknown_qualifier_above_release() {
    assert_eq!(mvn("1.0-jre", "1.0"), Greater);
    assert_eq!(mvn("1.0-incubating", "1.0"), Greater);
}
#[test]
fn test_cv_unknown_qualifier_above_sp() {
    assert_eq!(mvn("1.0-sp-1", "1.0-foo"), Less);
}
#[test]
fn test_cv_unknown_qualifiers_compare_lexically() {
    assert_eq!(mvn("2.0.1-klm", "2.0.1-lmn"), Less);
}

// ---- `a`/`b`/`m` are shorthand only when directly followed by a digit ----
#[test]
fn test_cv_short_qualifier_with_digit_expands() {
    assert_eq!(mvn("1-a1", "1-alpha-1"), Equal);
    assert_eq!(mvn("1a1", "1-alpha-1"), Equal);
    assert_eq!(mvn("1-b2", "1-beta-2"), Equal);
    assert_eq!(mvn("1-m3", "1-milestone-3"), Equal);
}
#[test]
fn test_cv_bare_short_qualifier_is_unknown() {
    // No trailing digit, so `a` stays the unknown qualifier "a" — which puts
    // it *above* the release, not at alpha's rank.
    assert_eq!(mvn("1.0-a", "1.0-alpha"), Greater);
    assert_eq!(mvn("1.0-a", "1.0"), Greater);
}

// ---- `-` nests, `.` does not ----
#[test]
fn test_cv_hyphen_nests_below_dot() {
    assert_eq!(mvn("1-1", "1.1"), Less);
    assert_eq!(mvn("1.0-1", "1.0.1"), Less);
    assert_eq!(mvn("2.0-1", "2.0.0.1"), Less);
}
#[test]
fn test_cv_dot_qualifier_behaves_like_hyphen() {
    assert_eq!(mvn("1.0.0.x1", "1.0.0-x2"), Less);
}

// ---- Trailing-null normalization ----
#[test]
fn test_cv_trailing_nulls_stripped() {
    assert_eq!(mvn("1", "1.0"), Equal);
    assert_eq!(mvn("1", "1.0.0"), Equal);
    assert_eq!(mvn("1", "1-0"), Equal);
    assert_eq!(mvn("1.0.0.0.0", "1"), Equal);
}
#[test]
fn test_cv_leading_zeros_ignored() {
    assert_eq!(mvn("1.01", "1.1"), Equal);
}

// ---- Arbitrary-precision numeric items (Maven falls back to BigInteger) ----
#[test]
fn test_cv_numbers_wider_than_u64() {
    assert_eq!(mvn("18446744073709551616", "18446744073709551617"), Less);
    assert_eq!(mvn("99999999999999999999999999", "1"), Greater);
}

// ---- Maven has no build metadata and no epoch: both are plain text ----
#[test]
fn test_cv_build_metadata_is_a_qualifier() {
    assert_eq!(mvn("1.0+build1", "1.0"), Greater);
    assert_eq!(mvn("1.0+build1", "1.0+build2"), Less);
}
#[test]
fn test_cv_colon_is_not_an_epoch() {
    assert_eq!(mvn("1:2.0", "2.0"), Less);
}

// ---- Real-world lifecycles ----
#[test]
fn test_cv_maven4_lifecycle() {
    assert_chain(
        &["4.0.0-alpha-13", "4.0.0-beta-5", "4.0.0-rc-1", "4.0.0-rc-5", "4.0.0", "4.0.1"],
        "maven",
    );
}
#[test]
fn test_cv_hibernate_lifecycle() {
    assert_chain(
        &["6.6.0.Alpha1", "6.6.0.CR1", "6.6.0.CR2", "6.6.0.Final", "6.6.1.Final"],
        "maven",
    );
}
#[test]
fn test_cv_netty_lifecycle() {
    assert_chain(&["4.2.0.Alpha1", "4.2.0.Beta1", "4.2.0.RC1", "4.2.0.Final"], "maven");
}
#[test]
fn test_cv_junit_jupiter_lifecycle() {
    assert_chain(&["5.10.0-M1", "5.10.0-RC1", "5.10.0", "5.10.5"], "maven");
}
#[test]
fn test_cv_guava_flavour_qualifiers() {
    // Both flavours are unknown qualifiers, so they rank lexically and above
    // the bare release.
    assert_chain(&["31.0-android", "31.0-jre", "31.0.1-jre"], "maven");
}
#[test]
fn test_cv_jackson_four_component() {
    assert_chain(&["2.6.7", "2.6.7.1", "2.6.7.5", "2.7.0"], "maven");
}

// ---- Deep nesting: exact up to the limit, rejected past it ----
#[test]
fn test_cv_deep_nesting_stays_exact() {
    // Each `-` nests one level. Maven orders `<64 levels>.1` above `<65
    // levels>`; a cap that silently stopped nesting would call them equal.
    let p64: String = vec!["1"; 64].join("-");
    let p65: String = vec!["1"; 65].join("-");
    assert_eq!(mvn(&format!("{p64}.1"), &p65), Greater);
    assert_eq!(mvn(&p64, &p65), Less);
    assert_eq!(mvn(&p64, "1"), Greater);
}
#[test]
fn test_cv_nesting_past_limit_is_rejected() {
    // Past MAX_DEPTH the parse errors instead of flattening — an explicit
    // failure rather than a silently different order.
    let deep: String = vec!["1"; 5000].join("-");
    // Checked at the strategy layer: `PyErr`'s message needs an interpreter.
    let err = parse_for_ecosystem(Ecosystem::Maven, &deep).unwrap_err();
    assert!(err.contains("nests more than"), "unexpected error: {err}");
    assert!(compare_str_with_ecosystem(&deep, "1", "maven").is_err());
}

// ---- Unicode ----
#[test]
fn test_cv_non_ascii_numerals_rejected() {
    // Maven reads Arabic-Indic digits numerically (`1.٢` == `1.2`); the ASCII
    // tokenizer cannot, so the input is refused rather than mis-ordered.
    assert!(compare_str_with_ecosystem("1.\u{0662}", "1.2", "maven").is_err());
}
#[test]
fn test_cv_unknown_qualifiers_use_utf16_order() {
    // Java compares String by UTF-16 code unit, so an astral-plane character
    // sorts *below* U+E000; Rust's byte order would say the opposite.
    assert_eq!(mvn("1-\u{1F600}", "1-\u{E000}"), Less);
    assert_eq!(mvn("1-\u{E9}", "1-z"), Greater);
}

// ---- Canonical form (drives `Version.__hash__`) ----
#[test]
fn test_cv_canonical_form() {
    let canon = |v: &str| parse_maven(v).unwrap().canonical();
    // Values taken from ComparableVersion.getCanonical() (maven-artifact 3.9.16).
    assert_eq!(canon("1-a1"), "1-alpha-1");
    assert_eq!(canon("1a1"), "1-alpha-1");
    assert_eq!(canon("1.0.0.x1"), "1-x-1");
    assert_eq!(canon("1release2"), "1-2");
    assert_eq!(canon("1jre.foo.foo.c"), "1-jre.foo.foo-c");
    assert_eq!(canon("1.0-final"), "1");
    assert_eq!(canon("1.1"), "1.1");
}
#[test]
fn test_cv_equal_versions_share_a_canonical_form() {
    // The hash contract rides on this: equal versions must canonicalize alike.
    for (a, b) in [("1.0-1", "1.0.0-1"), ("1-a1", "1-alpha-1"), ("1.0-ga", "1"), ("1.01", "1.1")] {
        assert_eq!(mvn(a, b), Equal, "{a} should equal {b}");
        assert_eq!(
            parse_maven(a).unwrap().canonical(),
            parse_maven(b).unwrap().canonical(),
            "{a} and {b} are equal but canonicalize differently"
        );
    }
}

// ---- Validation (unchanged: Maven mode still requires a digit start) ----
#[test]
fn test_cv_rejects_non_digit_start() {
    assert!(compare_str_with_ecosystem("v1.0", "1.0", "maven").is_err());
    assert!(compare_str_with_ecosystem("", "1.0", "maven").is_err());
}

// ---- Upstream non-transitivity (maven-artifact 3.10.0) ----
#[test]
fn test_cv_upstream_ordering_cycle_is_preserved() {
    // Direct ComparableVersion.compareTo results. `sort_key` cannot reproduce
    // this cycle; it keeps the other two edges and sorts alpha < release < sp.
    for (a, b) in [("1-foo.sp-0", "1-foo-alpha"), ("1-foo-alpha", "1-foo"), ("1-foo", "1-foo.sp-0")]
    {
        assert_eq!(mvn(a, b), Less, "{a} versus {b}");
        assert_eq!(mvn(b, a), Greater, "{b} versus {a}");
    }
    assert_eq!(parse_maven("1-foo.sp-0").unwrap().canonical(), "1-foo.sp");
}
