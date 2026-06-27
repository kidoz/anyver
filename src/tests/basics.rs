use super::*;

// --- Basic numeric ---

#[test]
fn test_basic_1_lt_2() {
    assert_eq!(cmpg("1", "2"), Less);
}
#[test]
fn test_basic_2_gt_1() {
    assert_eq!(cmpg("2", "1"), Greater);
}
#[test]
fn test_basic_equal() {
    assert_eq!(cmpg("1", "1"), Equal);
}
#[test]
fn test_trailing_zero_eq() {
    assert_eq!(cmpg("1.0", "1.0.0"), Equal);
}
#[test]
fn test_patch_lt() {
    assert_eq!(cmpg("1.0.0", "1.0.1"), Less);
}
#[test]
fn test_minor_lt() {
    assert_eq!(cmpg("1.0.1", "1.1.0"), Less);
}
#[test]
fn test_major_lt() {
    assert_eq!(cmpg("1.1.0", "2.0.0"), Less);
}
#[test]
fn test_double_digit_major() {
    assert_eq!(cmpg("10.0.0", "9.0.0"), Greater);
}
#[test]
fn test_same_version() {
    assert_eq!(cmpg("1.2.3", "1.2.3"), Equal);
}
#[test]
fn test_tiny_patch() {
    assert_eq!(cmpg("0.0.1", "0.0.2"), Less);
}
#[test]
fn test_numeric_sort() {
    assert_eq!(cmpg("1.2", "1.10"), Less);
}
#[test]
fn test_many_trailing_zeros() {
    assert_eq!(cmpg("1.0.0.0.0", "1.0.0"), Equal);
}
#[test]
fn test_zero_eq() {
    assert_eq!(cmpg("0", "0.0.0"), Equal);
}

// --- v-prefix ---

#[test]
fn test_vprefix_lower() {
    assert_eq!(cmpg("v1.0.0", "1.0.0"), Equal);
}
#[test]
fn test_vprefix_upper() {
    assert_eq!(cmpg("V1.0.0", "1.0.0"), Equal);
}
#[test]
fn test_vprefix_compare() {
    assert_eq!(cmpg("v2.0.0", "v1.0.0"), Greater);
}
#[test]
fn test_vprefix_lt() {
    assert_eq!(cmpg("v0.1.0", "v0.2.0"), Less);
}

// --- SemVer pre-release ---

#[test]
fn test_sv_alpha_lt_alpha1() {
    assert_eq!(cmpg("1.0.0-alpha", "1.0.0-alpha.1"), Less);
}
#[test]
fn test_sv_alpha_lt_beta() {
    assert_eq!(cmpg("1.0.0-alpha", "1.0.0-beta"), Less);
}
#[test]
fn test_sv_beta_lt_rc() {
    assert_eq!(cmpg("1.0.0-beta", "1.0.0-rc"), Less);
}
#[test]
fn test_sv_rc_lt_release() {
    assert_eq!(cmpg("1.0.0-rc", "1.0.0"), Less);
}
#[test]
fn test_sv_alpha_lt_release() {
    assert_eq!(cmpg("1.0.0-alpha", "1.0.0"), Less);
}
#[test]
fn test_sv_build_ignored() {
    assert_eq!(cmpg("1.0.0+build", "1.0.0"), Equal);
}
#[test]
fn test_sv_build_both_ignored() {
    assert_eq!(cmpg("1.0.0+build.1", "1.0.0+build.2"), Equal);
}
#[test]
fn test_sv_build_timestamp() {
    assert_eq!(cmpg("1.0.0+20130313144700", "1.0.0"), Equal);
}

// --- PEP 440 ---

#[test]
fn test_pep_a1_lt_a2() {
    assert_eq!(cmpg("1.0a1", "1.0a2"), Less);
}
#[test]
fn test_pep_a_lt_b() {
    assert_eq!(cmpg("1.0a2", "1.0b1"), Less);
}
#[test]
fn test_pep_b_lt_rc() {
    assert_eq!(cmpg("1.0b1", "1.0rc1"), Less);
}
#[test]
fn test_pep_rc_lt_rel() {
    assert_eq!(cmpg("1.0rc1", "1.0"), Less);
}
#[test]
fn test_pep_dev_lt_a() {
    assert_eq!(cmpg("1.0.dev1", "1.0a1"), Less);
}
#[test]
fn test_pep_dev_lt_rel() {
    assert_eq!(cmpg("1.0.dev1", "1.0"), Less);
}
#[test]
fn test_pep_rel_lt_post() {
    assert_eq!(cmpg("1.0", "1.0.post1"), Less);
}
#[test]
fn test_pep_post1_lt_post2() {
    assert_eq!(cmpg("1.0.post1", "1.0.post2"), Less);
}
#[test]
fn test_pep_epoch() {
    assert_eq!(cmpg("1!0.1", "2.0"), Greater);
}

// --- Debian/dpkg ---

#[test]
fn test_deb_tilde_alpha_lt_beta() {
    assert_eq!(cmpg("1.0~alpha", "1.0~beta"), Less);
}
#[test]
fn test_deb_tilde_rc_lt_rel() {
    assert_eq!(cmpg("1.0~rc1", "1.0"), Less);
}
#[test]
fn test_deb_tilde_alpha_lt_rel() {
    assert_eq!(cmpg("1.0~alpha", "1.0"), Less);
}
#[test]
fn test_deb_epoch_colon() {
    assert_eq!(cmpg("1:0.1", "2.0"), Greater);
}
#[test]
fn test_deb_epoch_compare() {
    assert_eq!(cmpg("2:1.0", "1:2.0"), Greater);
}
#[test]
fn test_deb_epoch_zero() {
    assert_eq!(cmpg("0:1.0", "1.0"), Equal);
}
#[test]
fn test_deb_build_stripped() {
    assert_eq!(cmpg("1.0+deb9u1", "1.0+deb9u2"), Equal);
}

// --- RPM ---

#[test]
fn test_rpm_tilde_rc_lt_rel() {
    assert_eq!(cmpg("1.0~rc1", "1.0"), Less);
}
#[test]
fn test_rpm_rel_lt_caret() {
    assert_eq!(cmpg("1.0", "1.0^git1"), Less);
}
#[test]
fn test_rpm_caret_lt_revision() {
    assert_eq!(cmpg("1.0^git1", "1.0-1.fc33"), Less);
}

// --- Go modules ---

#[test]
fn test_go_basic() {
    assert_eq!(cmpg("v1.0.0", "v1.0.1"), Less);
}
#[test]
fn test_go_alpha() {
    assert_eq!(cmpg("v1.0.0-alpha", "v1.0.0"), Less);
}
#[test]
fn test_go_incompatible() {
    assert_eq!(cmpg("v2.0.0+incompatible", "v2.0.0"), Equal);
}
#[test]
fn test_go_incompatible_lt() {
    assert_eq!(cmpg("v2.0.0+incompatible", "v2.0.1+incompatible"), Less);
}

// --- Ruby Gems ---

#[test]
fn test_ruby_pre_lt_rel() {
    assert_eq!(cmpg("1.0.0.pre", "1.0.0"), Less);
}
#[test]
fn test_ruby_alpha_lt_beta() {
    assert_eq!(cmpg("1.0.0.alpha", "1.0.0.beta"), Less);
}
#[test]
fn test_ruby_beta_lt_rc() {
    assert_eq!(cmpg("1.0.0.beta", "1.0.0.rc1"), Less);
}
#[test]
fn test_ruby_rc_lt_rel() {
    assert_eq!(cmpg("1.0.0.rc1", "1.0.0"), Less);
}
#[test]
fn test_ruby_numeric() {
    assert_eq!(cmpg("3.2", "3.10"), Less);
}
