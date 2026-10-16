use super::*;

// --- Cross-ecosystem packages ---

#[test]
fn test_django_major() {
    assert_eq!(cmpg("4.2", "5.0a1"), Less);
}
#[test]
fn test_django_alpha_lt_beta() {
    assert_eq!(cmpg("5.0a1", "5.0b1"), Less);
}
#[test]
fn test_django_rc_lt_rel() {
    assert_eq!(cmpg("5.0rc1", "5.0"), Less);
}
#[test]
fn test_rails_beta_lt_rc() {
    assert_eq!(cmpg("7.1.0.beta1", "7.1.0.rc1"), Less);
}
#[test]
fn test_rails_rc_lt_rel() {
    assert_eq!(cmpg("7.1.0.rc1", "7.1.0"), Less);
}
#[test]
fn test_node() {
    assert_eq!(cmpg("18.17.0", "20.0.0"), Less);
}
#[test]
fn test_kernel() {
    assert_eq!(cmpg("5.15.0", "6.1.0"), Less);
}
#[test]
fn test_spring() {
    assert_eq!(cmpg("5.3.30", "6.0.0"), Less);
}
#[test]
fn test_requests() {
    assert_eq!(cmpg("2.31.0", "2.32.0"), Less);
}

// --- Parser properties ---

#[test]
fn test_parser_segments() {
    let v = parse("1.2.3-alpha.1+build.42");
    assert_eq!(v.epoch, 0);
    assert_eq!(v.build, "build.42");
    assert!(v.is_prerelease);
    assert!(!v.is_postrelease);
    assert_eq!(v.segments.len(), 5);
    assert_eq!(v.segments[0], Seg::Num(1));
    assert_eq!(v.segments[3], Seg::Text("alpha".into()));
    assert_eq!(v.segments[4], Seg::Num(1));
}

#[test]
fn test_parser_epoch_bang() {
    let v = parse("1!2.3");
    assert_eq!(v.epoch, 1);
    assert_eq!(v.segments, vec![Seg::Num(2), Seg::Num(3)]);
}

#[test]
fn test_parser_epoch_colon() {
    let v = parse("1:2.3");
    assert_eq!(v.epoch, 1);
    assert_eq!(v.segments, vec![Seg::Num(2), Seg::Num(3)]);
}

#[test]
fn test_parser_postrelease() {
    let v = parse("1.0.post1");
    assert!(v.is_postrelease);
    assert!(!v.is_prerelease);
}

#[test]
fn test_parser_caret_postrelease() {
    let v = parse("1.0^git1");
    assert!(v.is_postrelease);
}

#[test]
fn test_parser_tilde_prerelease() {
    let v = parse("1.0~rc1");
    assert!(v.is_prerelease);
}

#[test]
fn test_parser_empty() {
    let v = parse("");
    assert_eq!(v.segments.len(), 0);
    assert_eq!(v.epoch, 0);
    assert_eq!(v.build, "");
}

#[test]
fn test_parser_vprefix() {
    let v = parse("v1.0.0");
    assert_eq!(v.segments, vec![Seg::Num(1), Seg::Num(0), Seg::Num(0)]);
}

#[test]
fn test_normalization_consistency() {
    let pa = parse("1.0");
    let pb = parse("1.0.0");
    let a = normalized(&pa.segments);
    let b = normalized(&pb.segments);
    assert_eq!(a, b);
}

// --- SemVer strict ---

#[test]
fn test_semver_strict_alpha_lt_alpha1() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("1.0.0-alpha").unwrap(),
            &parse_semver_strict("1.0.0-alpha.1").unwrap()
        ),
        Less
    );
}

#[test]
fn test_semver_strict_alpha1_lt_alpha_beta() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("1.0.0-alpha.1").unwrap(),
            &parse_semver_strict("1.0.0-alpha.beta").unwrap()
        ),
        Less
    );
}

#[test]
fn test_semver_strict_beta2_lt_beta11() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("1.0.0-beta.2").unwrap(),
            &parse_semver_strict("1.0.0-beta.11").unwrap()
        ),
        Less
    );
}

#[test]
fn test_semver_strict_rc_lt_release() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("1.0.0-rc.1").unwrap(),
            &parse_semver_strict("1.0.0").unwrap()
        ),
        Less
    );
}

#[test]
fn test_semver_strict_build_ignored() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("1.0.0+build").unwrap(),
            &parse_semver_strict("1.0.0").unwrap()
        ),
        Equal
    );
}

#[test]
fn test_semver_strict_basic() {
    assert_eq!(
        cmp_semver_strict(
            &parse_semver_strict("0.0.0").unwrap(),
            &parse_semver_strict("0.0.1").unwrap()
        ),
        Less
    );
}

#[test]
fn test_semver_strict_invalid() {
    assert!(parse_semver_strict("1.0").is_err());
}

// --- Unknown text ordering ---

#[test]
fn test_unknown_text_lt_release() {
    assert_eq!(cmpg("1.0.0.foo", "1.0.0"), Less);
}
#[test]
fn test_unknown_text_gt_known_pre() {
    assert_eq!(cmpg("1.0.0-foo", "1.0.0-rc"), Greater);
}
#[test]
fn test_unknown_text_lt_known_post() {
    assert_eq!(cmpg("1.0.0-foo", "1.0.0-post"), Less);
}

// --- Numeric overflow (Fix #1) ---

#[test]
fn test_overflow_saturates() {
    // A number that overflows u64 should saturate at u64::MAX, not become 0
    let v = parse("99999999999999999999999999999999");
    assert_eq!(v.segments, vec![Seg::Num(u64::MAX)]);
}

#[test]
fn test_overflow_greater_than_normal() {
    // Saturated version should be greater than any normal version
    assert_eq!(cmpg("99999999999999999999999999999999", "999"), Greater);
}

// --- Strict SemVer validation (Fix #2) ---

#[test]
fn test_semver_strict_leading_zero_major() {
    assert!(parse_semver_strict("01.0.0").is_err());
}

#[test]
fn test_semver_strict_leading_zero_minor() {
    assert!(parse_semver_strict("1.01.0").is_err());
}

#[test]
fn test_semver_strict_leading_zero_patch() {
    assert!(parse_semver_strict("1.0.01").is_err());
}

#[test]
fn test_semver_strict_leading_zero_prerelease() {
    assert!(parse_semver_strict("1.0.0-01").is_err());
}

#[test]
fn test_semver_strict_empty_prerelease() {
    assert!(parse_semver_strict("1.0.0-").is_err());
}

#[test]
fn test_semver_strict_empty_prerelease_ident() {
    assert!(parse_semver_strict("1.0.0-alpha..1").is_err());
}

#[test]
fn test_semver_strict_valid_prerelease() {
    let sv = parse_semver_strict("1.0.0-alpha.1.x-y").unwrap();
    assert_eq!(sv.major, 1);
    assert_eq!(sv.pre, Some("alpha.1.x-y".to_string()));
}

#[test]
fn test_semver_strict_zero_major_ok() {
    let sv = parse_semver_strict("0.0.0").unwrap();
    assert_eq!(sv.major, 0);
}

#[test]
fn test_semver_strict_zero_prerelease_ok() {
    let sv = parse_semver_strict("1.0.0-0").unwrap();
    assert_eq!(sv.pre, Some("0".to_string()));
}

#[test]
fn test_ecosystem_dispatch_generic_vs_semver() {
    assert_eq!(cmpg("1.0.0-alpha.1", "1.0.0-alpha.beta"), Greater);
    assert_eq!(
        compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "semver").unwrap(),
        Less
    );
}

#[test]
fn test_ecosystem_dispatch_invalid_name() {
    assert!(compare_str_with_ecosystem("1.0.0", "2.0.0", "invalid").is_err());
}

#[test]
fn test_strict_semver_build_metadata_validation() {
    for input in ["1.2.3+", "1.2.3+foo..bar", "1.2.3+foo_bar", "1.2.3+foo+bar"] {
        assert!(parse_semver_strict(input).is_err(), "{input}");
    }
    assert!(parse_semver_strict("1.2.3+001.foo-bar").is_ok());
}
