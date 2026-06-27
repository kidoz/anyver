use super::*;

// ====================================================================
// AUTODETECT ECOSYSTEM
// ====================================================================
// Ensures each branch of autodetect_ecosystem fires for the expected
// inputs — and does NOT fire for inputs that superficially resemble
// the markers. Covers the .el/.fc/.amzn trailing-digit fix (a0f1610).

// --- Unambiguous characters ---

#[test]
fn test_autodetect_bang_pep440() {
    assert_eq!(autodetect_ecosystem("1!0.1"), Ecosystem::Pep440);
}
#[test]
fn test_autodetect_tilde_debian() {
    assert_eq!(autodetect_ecosystem("1.0~alpha"), Ecosystem::Debian);
}
#[test]
fn test_autodetect_caret_rpm() {
    assert_eq!(autodetect_ecosystem("1.0^git1"), Ecosystem::Rpm);
}

// --- Strong substring markers ---

#[test]
fn test_autodetect_post_pep440() {
    assert_eq!(autodetect_ecosystem("1.0.post1"), Ecosystem::Pep440);
}
#[test]
fn test_autodetect_dev_pep440() {
    assert_eq!(autodetect_ecosystem("1.0.dev1"), Ecosystem::Pep440);
}
#[test]
fn test_autodetect_incompatible_go() {
    assert_eq!(autodetect_ecosystem("v2.0.0+incompatible"), Ecosystem::Go);
}
#[test]
fn test_autodetect_snapshot_maven() {
    assert_eq!(autodetect_ecosystem("1.0-SNAPSHOT"), Ecosystem::Maven);
}
#[test]
fn test_autodetect_underscore_alpha_alpine() {
    assert_eq!(autodetect_ecosystem("1.0_alpha1"), Ecosystem::Alpine);
}
#[test]
fn test_autodetect_underscore_rc_alpine() {
    assert_eq!(autodetect_ecosystem("1.0_rc1"), Ecosystem::Alpine);
}
#[test]
fn test_autodetect_underscore_pre_alpine() {
    assert_eq!(autodetect_ecosystem("1.0_pre1"), Ecosystem::Alpine);
}
#[test]
fn test_autodetect_plus_deb() {
    assert_eq!(autodetect_ecosystem("1.0+deb9u1"), Ecosystem::Debian);
}
#[test]
fn test_autodetect_plus_ubuntu() {
    assert_eq!(autodetect_ecosystem("1.0+ubuntu1"), Ecosystem::Debian);
}

// --- .elN / .fcN / .amznN — require trailing digit (a0f1610 fix) ---

#[test]
fn test_autodetect_el_with_digit_rpm() {
    assert_eq!(autodetect_ecosystem("5.14.0-362.13.1.el9_3"), Ecosystem::Rpm);
}
#[test]
fn test_autodetect_fc_with_digit_rpm() {
    assert_eq!(autodetect_ecosystem("252-14.fc40"), Ecosystem::Rpm);
}
#[test]
fn test_autodetect_amzn_with_digit_rpm() {
    assert_eq!(autodetect_ecosystem("1.0-1.amzn2"), Ecosystem::Rpm);
}

// Regression: .el/.fc/.amzn without trailing digit must NOT be Rpm
#[test]
fn test_autodetect_elegant_not_rpm() {
    assert_ne!(autodetect_ecosystem("1.0.elegant"), Ecosystem::Rpm);
}
#[test]
fn test_autodetect_fcomm_not_rpm() {
    assert_ne!(autodetect_ecosystem("1.0.fcomm"), Ecosystem::Rpm);
}
#[test]
fn test_autodetect_amznraw_not_rpm() {
    assert_ne!(autodetect_ecosystem("1.0.amznraw"), Ecosystem::Rpm);
}
#[test]
fn test_autodetect_el_at_end_not_rpm() {
    // ".el" with nothing after — no trailing digit → not Rpm
    assert_ne!(autodetect_ecosystem("1.0.el"), Ecosystem::Rpm);
}

// --- digit-letter-digit patterns → PEP 440 ---

#[test]
fn test_autodetect_a_pattern_pep440() {
    assert_eq!(autodetect_ecosystem("1.0a1"), Ecosystem::Pep440);
}
#[test]
fn test_autodetect_b_pattern_pep440() {
    assert_eq!(autodetect_ecosystem("1.0b2"), Ecosystem::Pep440);
}
#[test]
fn test_autodetect_rc_pattern_pep440() {
    assert_eq!(autodetect_ecosystem("1.0rc1"), Ecosystem::Pep440);
}

// --- Alpine revision suffix -rN ---

#[test]
fn test_autodetect_alpine_revision() {
    assert_eq!(autodetect_ecosystem("1.2.3-r5"), Ecosystem::Alpine);
}

// --- Ruby dot-qualifier patterns (no '-' or '_') ---

#[test]
fn test_autodetect_ruby_pre() {
    assert_eq!(autodetect_ecosystem("1.0.0.pre"), Ecosystem::Ruby);
}
#[test]
fn test_autodetect_ruby_rc() {
    assert_eq!(autodetect_ecosystem("1.0.0.rc"), Ecosystem::Ruby);
}
#[test]
fn test_autodetect_ruby_alpha_digit() {
    assert_eq!(autodetect_ecosystem("1.0.0.alpha1"), Ecosystem::Ruby);
}
#[test]
fn test_autodetect_ruby_beta_digit() {
    assert_eq!(autodetect_ecosystem("1.0.0.beta2"), Ecosystem::Ruby);
}

// --- CalVer (year-based first component with '.') ---

#[test]
fn test_autodetect_calver_ubuntu() {
    assert_eq!(autodetect_ecosystem("24.04"), Ecosystem::Generic); // 24 not in [1990, 2100]
}
#[test]
fn test_autodetect_calver_4digit_year() {
    assert_eq!(autodetect_ecosystem("2024.01"), Ecosystem::Calver);
}
#[test]
fn test_autodetect_calver_far_year() {
    assert_eq!(autodetect_ecosystem("2099.12.31"), Ecosystem::Calver);
}
#[test]
fn test_autodetect_year_too_old_not_calver() {
    // 1989 is below the 1990 cutoff
    assert_ne!(autodetect_ecosystem("1989.1.1"), Ecosystem::Calver);
}
#[test]
fn test_autodetect_year_too_new_not_calver() {
    assert_ne!(autodetect_ecosystem("2101.1.1"), Ecosystem::Calver);
}
#[test]
fn test_autodetect_year_no_dot_not_calver() {
    // Year without '.' — can't be CalVer (no minor field)
    assert_ne!(autodetect_ecosystem("2024"), Ecosystem::Calver);
}

// --- Generic fallback ---

#[test]
fn test_autodetect_plain_numeric_generic() {
    assert_eq!(autodetect_ecosystem("1.0.0"), Ecosystem::Generic);
}
#[test]
fn test_autodetect_four_component_generic() {
    // No markers, no year — stays Generic
    assert_eq!(autodetect_ecosystem("1.2.3.4"), Ecosystem::Generic);
}
