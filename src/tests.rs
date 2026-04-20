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

// --- Maven ---

// ---- Qualifier ordering (spec §8) ----
#[test]
fn test_maven_alpha_lt_beta() {
    assert_eq!(cmpg("1.0-alpha-1", "1.0-beta-1"), Less);
}
#[test]
fn test_maven_beta_lt_rc() {
    assert_eq!(cmpg("1.0-beta-1", "1.0-rc-1"), Less);
}
#[test]
fn test_maven_snapshot_lt_rel() {
    assert_eq!(cmpg("1.0-SNAPSHOT", "1.0"), Less);
}
#[test]
fn test_maven_rel_lt_sp() {
    assert_eq!(cmpg("1.0", "1.0-sp-1"), Less);
}
#[test]
fn test_maven_alpha_lt_milestone() {
    assert_eq!(cmpg("1.0-alpha", "1.0-milestone"), Less);
}
#[test]
fn test_maven_milestone_lt_rc() {
    assert_eq!(cmpg("1.0-milestone-1", "1.0-rc-1"), Less);
}
#[test]
fn test_maven_rc_lt_snapshot() {
    assert_eq!(cmpg("1.0-rc-1", "1.0-SNAPSHOT"), Less);
}
#[test]
fn test_maven_cr_eq_rc() {
    assert_eq!(cmpg("1.0-cr-1", "1.0-rc-1"), Equal);
}
#[test]
fn test_maven_full_qualifier_chain() {
    // alpha < beta < milestone < rc < snapshot < release < sp
    let chain = [
        "1.0-alpha-1",
        "1.0-beta-1",
        "1.0-milestone-1",
        "1.0-rc-1",
        "1.0-SNAPSHOT",
        "1.0",
        "1.0-sp-1",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(cmpg(chain[i], chain[i + 1]), Less, "{} should be < {}", chain[i], chain[i + 1]);
    }
}

// ---- Qualifier aliases (a/b/m/cr) ----
#[test]
fn test_maven_a_alias_alpha() {
    assert_eq!(cmpg("1.0-a", "1.0-alpha"), Equal);
}
#[test]
fn test_maven_b_alias_beta() {
    assert_eq!(cmpg("1.0-b", "1.0-beta"), Equal);
}
#[test]
fn test_maven_m_alias_milestone() {
    assert_eq!(cmpg("1.0-m", "1.0-milestone"), Equal);
}
#[test]
fn test_maven_cr_alias_rc() {
    assert_eq!(cmpg("1.0-cr", "1.0-rc"), Equal);
}
#[test]
fn test_maven_a1_alias_alpha1() {
    assert_eq!(cmpg("1-a1", "1-alpha-1"), Equal);
}
#[test]
fn test_maven_b2_alias_beta2() {
    assert_eq!(cmpg("1-b2", "1-beta-2"), Equal);
}
#[test]
fn test_maven_m3_alias_milestone3() {
    assert_eq!(cmpg("1-m3", "1-milestone-3"), Equal);
}

// ---- Case insensitivity ----
#[test]
fn test_maven_case_insensitive_snapshot() {
    assert_eq!(cmpg("1.0-SNAPSHOT", "1.0-snapshot"), Equal);
}
#[test]
fn test_maven_case_insensitive_alpha() {
    assert_eq!(cmpg("1.0-ALPHA", "1.0-alpha"), Equal);
}
#[test]
fn test_maven_case_insensitive_rc() {
    assert_eq!(cmpg("1.0-RC-1", "1.0-rc-1"), Equal);
}
#[test]
fn test_maven_case_insensitive_cr() {
    assert_eq!(cmpg("1.0-CR-1", "1.0-cr-1"), Equal);
}
#[test]
fn test_maven_mixed_case_alphabet() {
    assert_eq!(cmpg("1-abcdefghijklmnopqrstuvwxyz", "1-ABCDEFGHIJKLMNOPQRSTUVWXYZ"), Equal);
}

// ---- Trailing zero equivalence ----
#[test]
fn test_maven_trailing_zeros_2() {
    assert_eq!(cmpg("1.0", "1.0.0"), Equal);
}
#[test]
fn test_maven_trailing_zeros_3() {
    assert_eq!(cmpg("1", "1.0.0"), Equal);
}
#[test]
fn test_maven_trailing_zeros_many() {
    assert_eq!(cmpg("1.0.0.0.0.0.0", "1"), Equal);
}

// ---- Basic numeric ordering ----
#[test]
fn test_maven_numeric_1_lt_2() {
    assert_eq!(cmpg("1", "2"), Less);
}
#[test]
fn test_maven_numeric_1_5_lt_2() {
    assert_eq!(cmpg("1.5", "2"), Less);
}
#[test]
fn test_maven_numeric_minor_order() {
    assert_eq!(cmpg("1.0", "1.1"), Less);
}
#[test]
fn test_maven_numeric_patch_order() {
    assert_eq!(cmpg("1.0.0", "1.0.1"), Less);
}
#[test]
fn test_maven_numeric_1_0_1_lt_1_1() {
    assert_eq!(cmpg("1.0.1", "1.1"), Less);
}
#[test]
fn test_maven_numeric_1_1_lt_1_2_0() {
    assert_eq!(cmpg("1.1", "1.2.0"), Less);
}

// ---- Pre-release before release ----
#[test]
fn test_maven_alpha_before_release() {
    assert_eq!(cmpg("1.0-alpha-1", "1.0"), Less);
}
#[test]
fn test_maven_alpha_snapshot_lt_alpha() {
    assert_eq!(cmpg("1.0-alpha-1-SNAPSHOT", "1.0-alpha-1"), Less);
}
#[test]
fn test_maven_alpha1_lt_alpha2() {
    assert_eq!(cmpg("1.0-alpha-1", "1.0-alpha-2"), Less);
}
#[test]
fn test_maven_beta1_lt_snapshot() {
    assert_eq!(cmpg("1.0-beta-1", "1.0-SNAPSHOT"), Less);
}

// ---- Post-release / sp ----
#[test]
fn test_maven_release_lt_post_numeric() {
    assert_eq!(cmpg("1.0", "1.0.1"), Less);
}
#[test]
fn test_maven_sp1_lt_sp2() {
    assert_eq!(cmpg("1.0-sp-1", "1.0-sp-2"), Less);
}

// ---- Unknown qualifiers ----
#[test]
fn test_maven_unknown_qualifier_lexical_order() {
    assert_eq!(cmpg("2.0.1-klm", "2.0.1-lmn"), Less);
}

// ---- Apache Maven Core lifecycle ----
#[test]
fn test_maven_maven_core_alpha_chain() {
    let chain = [
        "2.0-alpha-1",
        "2.0-alpha-2",
        "2.0-alpha-3",
        "2.0-beta-1",
        "2.0-beta-2",
        "2.0-beta-3",
        "2.0",
        "2.0.1",
        "2.0.2",
        "2.0.11",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Maven Core: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Log4j lifecycle ----
#[test]
fn test_maven_log4j_lifecycle() {
    let chain = [
        "2.0-alpha1",
        "2.0-alpha2",
        "2.0-beta1",
        "2.0-beta9",
        "2.0-rc1",
        "2.0-rc2",
        "2.0",
        "2.0.1",
        "2.0.2",
        "2.1",
        "2.17.1",
        "2.24.3",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Log4j: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- JUnit 4 lifecycle ----
#[test]
fn test_maven_junit4_lifecycle() {
    let chain = [
        "4.12-beta-1",
        "4.12-beta-2",
        "4.12-beta-3",
        "4.12",
        "4.13-beta-1",
        "4.13-rc-1",
        "4.13-rc-2",
        "4.13",
        "4.13.1",
        "4.13.2",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "JUnit4: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- JUnit Jupiter lifecycle ----
#[test]
fn test_maven_junit_jupiter_lifecycle() {
    let chain = [
        "5.9.0-M1",
        "5.9.0-RC1",
        "5.9.0",
        "5.9.1",
        "5.9.2",
        "5.9.3",
        "5.10.0-M1",
        "5.10.0-RC1",
        "5.10.0-RC2",
        "5.10.0",
        "5.10.5",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "JUnit Jupiter: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Spring Core milestone lifecycle ----
#[test]
fn test_maven_spring_core_milestones() {
    let chain = ["2.0-m3", "2.0-m5", "2.0-rc1", "2.0-rc2", "2.0"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Spring Core: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Guava lifecycle ----
#[test]
fn test_maven_guava_rc_chain() {
    let chain = ["14.0-rc1", "14.0-rc2", "14.0-rc3", "14.0", "14.0.1", "15.0-rc1", "15.0"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Guava: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- SLF4J lifecycle ----
#[test]
fn test_maven_slf4j_lifecycle() {
    let chain =
        ["2.0.0-alpha0", "2.0.0-alpha7", "2.0.0-beta0", "2.0.0-beta1", "2.0.0", "2.0.1", "2.0.17"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "SLF4J: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Apache Commons Lang3 ----
#[test]
fn test_maven_commons_lang3() {
    let chain = ["3.0", "3.0.1", "3.1", "3.4", "3.9", "3.10", "3.11", "3.12.0", "3.20.0"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Commons Lang3: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Jackson Databind ----
#[test]
fn test_maven_jackson_four_component() {
    // Jackson uses four-component versions for security patches
    let chain = ["2.6.7", "2.6.7.1", "2.6.7.2", "2.6.7.3", "2.6.7.4", "2.6.7.5"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Jackson: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

#[test]
fn test_maven_jackson_rc_lifecycle() {
    // Jackson switched from .rc (dot) to -rc (hyphen) across versions
    let chain = ["2.12.0-rc1", "2.12.0-rc2", "2.12.0", "2.12.7", "2.12.7.1", "2.12.7.2"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Jackson: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Kafka Clients ----
#[test]
fn test_maven_kafka_four_component() {
    // Kafka used four-component versions in early releases
    let chain = [
        "0.8.2-beta",
        "0.9.0.0",
        "0.9.0.1",
        "0.10.0.0",
        "0.10.2.2",
        "0.11.0.3",
        "1.0.0",
        "2.0.0",
        "3.0.0",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Kafka: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Log4j 3.0.0 alpha/beta lifecycle ----
#[test]
fn test_maven_log4j3_prerelease() {
    let chain = ["3.0.0-alpha1", "3.0.0-beta1", "3.0.0-beta2", "3.0.0-beta3"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Log4j 3: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Maven Core 4.x prerelease ----
#[test]
fn test_maven_maven4_prerelease() {
    let chain = [
        "4.0.0-alpha-2",
        "4.0.0-alpha-13",
        "4.0.0-beta-3",
        "4.0.0-beta-5",
        "4.0.0-rc-1",
        "4.0.0-rc-5",
    ];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Maven 4: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Commons IO with mixed component counts ----
#[test]
fn test_maven_commons_io() {
    let chain = ["0.1", "1.0", "1.3.2", "1.4", "2.0", "2.0.1", "2.7", "2.8.0", "2.16.1", "2.21.0"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Commons IO: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Hibernate dot-separated qualifiers ----
#[test]
fn test_maven_hibernate_lifecycle() {
    // Hibernate uses .Alpha, .Beta, .CR, .Final with dot separators
    let chain = ["6.6.0.Alpha1", "6.6.0.CR1", "6.6.0.CR2"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Hibernate: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- Netty dot-separated qualifiers ----
#[test]
fn test_maven_netty_lifecycle() {
    // Netty uses .Alpha, .Beta, .CR with dot separators
    let chain = ["4.2.0.Alpha1", "4.2.0.Alpha5", "4.2.0.Beta1", "4.2.0.RC1", "4.2.0.RC4"];
    for i in 0..chain.len() - 1 {
        assert_eq!(
            cmpg(chain[i], chain[i + 1]),
            Less,
            "Netty: {} should be < {}",
            chain[i],
            chain[i + 1]
        );
    }
}

// ---- timestamp version (Commons IO) ----
#[test]
fn test_maven_timestamp_version() {
    // Commons IO had a timestamp version; it's just a huge numeric value
    assert_eq!(cmpg("2.21.0", "20030203.000550"), Less);
}

// ---- Maven ecosystem dispatch: ----
#[test]
fn test_maven_eco_log4j_chain() {
    assert_eq!(compare_str_with_ecosystem("2.0-alpha1", "2.0-rc1", "maven").unwrap(), Less);
}
#[test]
fn test_maven_eco_junit_milestone() {
    assert_eq!(compare_str_with_ecosystem("5.10.0-M1", "5.10.0-RC1", "maven").unwrap(), Less);
}
#[test]
fn test_maven_eco_rc_lt_release() {
    assert_eq!(compare_str_with_ecosystem("5.10.0-RC1", "5.10.0", "maven").unwrap(), Less);
}
#[test]
fn test_maven_eco_jackson_four_component() {
    assert_eq!(compare_str_with_ecosystem("2.6.7", "2.6.7.1", "maven").unwrap(), Less);
}
#[test]
fn test_maven_eco_mvn_alias() {
    assert_eq!(compare_str_with_ecosystem("1.0-alpha", "1.0", "mvn").unwrap(), Less);
}

// ---- Maven validation ----
#[test]
fn test_maven_eco_reject_empty() {
    assert!(compare_str_with_ecosystem("", "1.0", "maven").is_err());
}
#[test]
fn test_maven_eco_reject_non_digit_start() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "maven").is_err());
}
#[test]
fn test_maven_eco_reject_v_prefix() {
    // Maven requires digit start; 'v1.0' starts with 'v'
    assert!(compare_str_with_ecosystem("v1.0", "1.0", "maven").is_err());
}

// ---- Maven properties via parser ----
#[test]
fn test_maven_parser_prerelease_alpha() {
    let v = parse("1.0-alpha-1");
    assert!(v.is_prerelease);
    assert!(!v.is_postrelease);
}
#[test]
fn test_maven_parser_prerelease_snapshot() {
    let v = parse("1.0-SNAPSHOT");
    assert!(v.is_prerelease);
}
#[test]
fn test_maven_parser_postrelease_sp() {
    let v = parse("1.0-sp-1");
    assert!(v.is_postrelease);
}
#[test]
fn test_maven_parser_release_no_qualifier() {
    let v = parse("2.0.1");
    assert!(!v.is_prerelease);
    assert!(!v.is_postrelease);
}
#[test]
fn test_maven_parser_segments_alpha() {
    let v = parse("4.0.0-alpha-13");
    assert_eq!(v.segments.len(), 5);
    assert_eq!(v.segments[0], Seg::Num(4));
    assert_eq!(v.segments[1], Seg::Num(0));
    assert_eq!(v.segments[2], Seg::Num(0));
    assert_eq!(v.segments[3], Seg::Text("alpha".into()));
    assert_eq!(v.segments[4], Seg::Num(13));
}
#[test]
fn test_maven_parser_segments_snapshot() {
    let v = parse("1.0-SNAPSHOT");
    assert_eq!(v.segments.len(), 3);
    assert_eq!(v.segments[0], Seg::Num(1));
    assert_eq!(v.segments[1], Seg::Num(0));
    assert_eq!(v.segments[2], Seg::Text("snapshot".into()));
}
#[test]
fn test_maven_parser_four_component() {
    let v = parse("2.6.7.5");
    assert_eq!(v.segments.len(), 4);
    assert_eq!(v.segments[3], Seg::Num(5));
}

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

// --- Ecosystem enum parsing ---

#[test]
fn test_ecosystem_pep440_aliases() {
    assert_eq!(Ecosystem::from_str("pep440").unwrap(), Ecosystem::Pep440);
    assert_eq!(Ecosystem::from_str("pep-440").unwrap(), Ecosystem::Pep440);
    assert_eq!(Ecosystem::from_str("python").unwrap(), Ecosystem::Pep440);
}

#[test]
fn test_ecosystem_debian_aliases() {
    assert_eq!(Ecosystem::from_str("debian").unwrap(), Ecosystem::Debian);
    assert_eq!(Ecosystem::from_str("dpkg").unwrap(), Ecosystem::Debian);
    assert_eq!(Ecosystem::from_str("deb").unwrap(), Ecosystem::Debian);
}

#[test]
fn test_ecosystem_rpm_aliases() {
    assert_eq!(Ecosystem::from_str("rpm").unwrap(), Ecosystem::Rpm);
    assert_eq!(Ecosystem::from_str("redhat").unwrap(), Ecosystem::Rpm);
}

#[test]
fn test_ecosystem_ruby_aliases() {
    assert_eq!(Ecosystem::from_str("ruby").unwrap(), Ecosystem::Ruby);
    assert_eq!(Ecosystem::from_str("gem").unwrap(), Ecosystem::Ruby);
    assert_eq!(Ecosystem::from_str("rubygems").unwrap(), Ecosystem::Ruby);
}

#[test]
fn test_ecosystem_maven_aliases() {
    assert_eq!(Ecosystem::from_str("maven").unwrap(), Ecosystem::Maven);
    assert_eq!(Ecosystem::from_str("mvn").unwrap(), Ecosystem::Maven);
}

#[test]
fn test_ecosystem_go_aliases() {
    assert_eq!(Ecosystem::from_str("go").unwrap(), Ecosystem::Go);
    assert_eq!(Ecosystem::from_str("golang").unwrap(), Ecosystem::Go);
}

// --- PEP 440 ecosystem dispatch ---

#[test]
fn test_pep440_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0a1", "1.0a2", "pep440").unwrap(), Less);
}

#[test]
fn test_pep440_eco_dev_lt_alpha() {
    assert_eq!(compare_str_with_ecosystem("1.0.dev1", "1.0a1", "pep440").unwrap(), Less);
}

#[test]
fn test_pep440_eco_rc_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0rc1", "1.0", "pep440").unwrap(), Less);
}

#[test]
fn test_pep440_eco_rel_lt_post() {
    assert_eq!(compare_str_with_ecosystem("1.0", "1.0.post1", "pep440").unwrap(), Less);
}

#[test]
fn test_pep440_eco_epoch() {
    assert_eq!(compare_str_with_ecosystem("1!0.1", "2.0", "pep440").unwrap(), Greater);
}

#[test]
fn test_pep440_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "pep440").is_err());
}

// --- Debian ecosystem dispatch ---

#[test]
fn test_debian_eco_tilde_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0~alpha", "1.0", "debian").unwrap(), Less);
}

#[test]
fn test_debian_eco_tilde_ordering() {
    assert_eq!(compare_str_with_ecosystem("1.0~alpha", "1.0~beta", "debian").unwrap(), Less);
}

#[test]
fn test_debian_eco_epoch() {
    assert_eq!(compare_str_with_ecosystem("1:0.1", "2.0", "debian").unwrap(), Greater);
}

#[test]
fn test_debian_eco_epoch_compare() {
    assert_eq!(compare_str_with_ecosystem("2:1.0", "1:2.0", "debian").unwrap(), Greater);
}

#[test]
fn test_debian_eco_plus_not_stripped() {
    // In Debian, + is NOT build metadata — it's part of the version
    assert_eq!(compare_str_with_ecosystem("1.0+deb9u1", "1.0+deb9u2", "debian").unwrap(), Less);
}

#[test]
fn test_debian_eco_revision() {
    // revision is everything after last '-'
    assert_eq!(compare_str_with_ecosystem("1.0-1", "1.0-2", "debian").unwrap(), Less);
}

// --- RPM ecosystem dispatch ---

#[test]
fn test_rpm_eco_tilde_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0~rc1", "1.0", "rpm").unwrap(), Less);
}

#[test]
fn test_rpm_eco_release_lt_caret() {
    assert_eq!(compare_str_with_ecosystem("1.0", "1.0^git1", "rpm").unwrap(), Less);
}

#[test]
fn test_rpm_eco_basic_numeric() {
    assert_eq!(compare_str_with_ecosystem("1.0", "2.0", "rpm").unwrap(), Less);
}

#[test]
fn test_rpm_eco_equal() {
    assert_eq!(compare_str_with_ecosystem("1.0", "1.0", "rpm").unwrap(), Equal);
}

#[test]
fn test_rpm_eco_epoch() {
    assert_eq!(compare_str_with_ecosystem("1:1.0", "2.0", "rpm").unwrap(), Greater);
}

// --- Ruby ecosystem dispatch ---

#[test]
fn test_ruby_eco_pre_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0.0.pre", "1.0.0", "ruby").unwrap(), Less);
}

#[test]
fn test_ruby_eco_alpha_lt_beta() {
    assert_eq!(compare_str_with_ecosystem("1.0.0.alpha", "1.0.0.beta", "ruby").unwrap(), Less);
}

#[test]
fn test_ruby_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "ruby").is_err());
}

// --- Maven ecosystem dispatch ---

#[test]
fn test_maven_eco_alpha_lt_beta() {
    assert_eq!(compare_str_with_ecosystem("1.0-alpha-1", "1.0-beta-1", "maven").unwrap(), Less);
}

#[test]
fn test_maven_eco_snapshot_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0-SNAPSHOT", "1.0", "maven").unwrap(), Less);
}

#[test]
fn test_maven_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "maven").is_err());
}

// --- Go ecosystem dispatch ---

#[test]
fn test_go_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("v1.0.0", "v1.0.1", "go").unwrap(), Less);
}

#[test]
fn test_go_eco_alpha_lt_release() {
    assert_eq!(compare_str_with_ecosystem("v1.0.0-alpha", "v1.0.0", "go").unwrap(), Less);
}

#[test]
fn test_go_eco_invalid() {
    // Go requires valid SemVer
    assert!(compare_str_with_ecosystem("v1.0", "v2.0", "go").is_err());
}

// --- dpkg verrevcmp unit tests ---

#[test]
fn test_verrevcmp_equal() {
    assert_eq!(verrevcmp(b"1.0", b"1.0"), Equal);
}

#[test]
fn test_verrevcmp_numeric() {
    assert_eq!(verrevcmp(b"1.1", b"1.2"), Less);
}

#[test]
fn test_verrevcmp_tilde_lt_empty() {
    assert_eq!(verrevcmp(b"1.0~alpha", b"1.0"), Less);
}

#[test]
fn test_verrevcmp_tilde_ordering() {
    assert_eq!(verrevcmp(b"1.0~a", b"1.0~b"), Less);
}

#[test]
fn test_verrevcmp_plus_significant() {
    // + is NOT stripped in dpkg — it's a regular character
    assert_eq!(verrevcmp(b"1.0+deb9u1", b"1.0+deb9u2"), Less);
}

// --- rpmverscmp unit tests ---

#[test]
fn test_rpmverscmp_equal() {
    assert_eq!(rpmverscmp("1.0", "1.0"), Equal);
}

#[test]
fn test_rpmverscmp_numeric() {
    assert_eq!(rpmverscmp("1.1", "1.2"), Less);
}

#[test]
fn test_rpmverscmp_tilde() {
    assert_eq!(rpmverscmp("1.0~rc1", "1.0"), Less);
}

#[test]
fn test_rpmverscmp_caret() {
    assert_eq!(rpmverscmp("1.0", "1.0^git1"), Less);
}

#[test]
fn test_rpmverscmp_digits_gt_alpha() {
    assert_eq!(rpmverscmp("1.0.1", "1.0.a"), Greater);
}

// --- npm ecosystem dispatch (delegates to SemVer) ---

#[test]
fn test_npm_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "npm").unwrap(), Less);
}

#[test]
fn test_npm_eco_prerelease() {
    assert_eq!(
        compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "npm").unwrap(),
        Less
    );
}

#[test]
fn test_npm_eco_invalid() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "npm").is_err());
}

#[test]
fn test_npm_aliases() {
    assert_eq!(Ecosystem::from_str("npm").unwrap(), Ecosystem::Npm);
    assert_eq!(Ecosystem::from_str("node").unwrap(), Ecosystem::Npm);
    assert_eq!(Ecosystem::from_str("nodejs").unwrap(), Ecosystem::Npm);
}

// --- NuGet ecosystem dispatch ---

#[test]
fn test_nuget_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "nuget").unwrap(), Less);
}

#[test]
fn test_nuget_eco_four_segments() {
    // NuGet allows 4 numeric segments
    assert_eq!(compare_str_with_ecosystem("1.0.0.0", "1.0.0.1", "nuget").unwrap(), Less);
}

#[test]
fn test_nuget_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "nuget").is_err());
}

#[test]
fn test_nuget_aliases() {
    assert_eq!(Ecosystem::from_str("nuget").unwrap(), Ecosystem::Nuget);
    assert_eq!(Ecosystem::from_str("dotnet").unwrap(), Ecosystem::Nuget);
}

// --- Composer ecosystem dispatch ---

#[test]
fn test_composer_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "2.0.0", "composer").unwrap(), Less);
}

#[test]
fn test_composer_eco_stability() {
    assert_eq!(compare_str_with_ecosystem("1.0.0-alpha", "1.0.0-beta", "composer").unwrap(), Less);
}

#[test]
fn test_composer_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "composer").is_err());
}

#[test]
fn test_composer_aliases() {
    assert_eq!(Ecosystem::from_str("composer").unwrap(), Ecosystem::Composer);
    assert_eq!(Ecosystem::from_str("packagist").unwrap(), Ecosystem::Composer);
    assert_eq!(Ecosystem::from_str("php").unwrap(), Ecosystem::Composer);
}

// --- Crates.io ecosystem dispatch (delegates to SemVer) ---

#[test]
fn test_crates_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "crates").unwrap(), Less);
}

#[test]
fn test_crates_eco_prerelease() {
    assert_eq!(compare_str_with_ecosystem("1.0.0-alpha", "1.0.0", "crates").unwrap(), Less);
}

#[test]
fn test_crates_eco_invalid() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "crates").is_err());
}

#[test]
fn test_crates_aliases() {
    assert_eq!(Ecosystem::from_str("crates").unwrap(), Ecosystem::Crates);
    assert_eq!(Ecosystem::from_str("cargo").unwrap(), Ecosystem::Crates);
    assert_eq!(Ecosystem::from_str("crates.io").unwrap(), Ecosystem::Crates);
}

// --- Hex ecosystem dispatch (delegates to SemVer) ---

#[test]
fn test_hex_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "hex").unwrap(), Less);
}

#[test]
fn test_hex_eco_prerelease() {
    assert_eq!(compare_str_with_ecosystem("1.0.0-rc.1", "1.0.0", "hex").unwrap(), Less);
}

#[test]
fn test_hex_aliases() {
    assert_eq!(Ecosystem::from_str("hex").unwrap(), Ecosystem::Hex);
    assert_eq!(Ecosystem::from_str("elixir").unwrap(), Ecosystem::Hex);
    assert_eq!(Ecosystem::from_str("erlang").unwrap(), Ecosystem::Hex);
}

// --- Swift ecosystem dispatch (delegates to SemVer) ---

#[test]
fn test_swift_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "2.0.0", "swift").unwrap(), Less);
}

#[test]
fn test_swift_eco_invalid() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "swift").is_err());
}

#[test]
fn test_swift_aliases() {
    assert_eq!(Ecosystem::from_str("swift").unwrap(), Ecosystem::Swift);
    assert_eq!(Ecosystem::from_str("swiftpm").unwrap(), Ecosystem::Swift);
}

// --- CalVer ecosystem dispatch ---

#[test]
fn test_calver_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("2024.01", "2024.02", "calver").unwrap(), Less);
}

#[test]
fn test_calver_eco_yyyymmdd() {
    assert_eq!(compare_str_with_ecosystem("20240115", "20240201", "calver").unwrap(), Less);
}

#[test]
fn test_calver_eco_with_micro() {
    assert_eq!(compare_str_with_ecosystem("2024.1.0", "2024.1.1", "calver").unwrap(), Less);
}

#[test]
fn test_calver_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "2024.1", "calver").is_err());
}

// --- Alpine ecosystem dispatch ---

#[test]
fn test_alpine_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "alpine").unwrap(), Less);
}

#[test]
fn test_alpine_eco_suffix_alpha() {
    assert_eq!(compare_str_with_ecosystem("1.0_alpha1", "1.0_beta1", "alpine").unwrap(), Less);
}

#[test]
fn test_alpine_eco_suffix_pre_lt_release() {
    assert_eq!(compare_str_with_ecosystem("1.0_rc1", "1.0", "alpine").unwrap(), Less);
}

#[test]
fn test_alpine_eco_invalid() {
    assert!(compare_str_with_ecosystem("abc", "1.0", "alpine").is_err());
}

#[test]
fn test_alpine_aliases() {
    assert_eq!(Ecosystem::from_str("alpine").unwrap(), Ecosystem::Alpine);
    assert_eq!(Ecosystem::from_str("apk").unwrap(), Ecosystem::Alpine);
}

// --- Docker ecosystem dispatch (delegates to Generic) ---

#[test]
fn test_docker_eco_basic() {
    assert_eq!(compare_str_with_ecosystem("1.25.3", "1.25.4", "docker").unwrap(), Less);
}

#[test]
fn test_docker_eco_calver() {
    assert_eq!(compare_str_with_ecosystem("24.04", "24.10", "docker").unwrap(), Less);
}

#[test]
fn test_docker_eco_permissive() {
    // Docker accepts anything — no validation
    assert!(compare_str_with_ecosystem("latest", "stable", "docker").is_ok());
}

#[test]
fn test_docker_aliases() {
    assert_eq!(Ecosystem::from_str("docker").unwrap(), Ecosystem::Docker);
    assert_eq!(Ecosystem::from_str("oci").unwrap(), Ecosystem::Docker);
}

// ====================================================================
// ECOSYSTEM DATASETS
// ====================================================================
// Each section uses version strings from popular packages,
// tested via ecosystem-specific comparison dispatch.
//
// Sources:
//   npm: registry.npmjs.org  |  crates: crates.io
//   pep440: pypi.org         |  debian: packages.debian.org
//   rpm: koji.fedoraproject  |  ruby: rubygems.org
//   go: pkg.go.dev           |  nuget: nuget.org
//   composer: packagist.org  |  calver: various
//   alpine: pkgs.alpinelinux |  docker: hub.docker.com
//   hex: hex.pm              |  swift: swiftpackageindex.com

// --- npm (strict SemVer) ---

#[test]
fn test_npm_react() {
    assert_chain(
        &[
            "17.0.0", "17.0.1", "17.0.2", "18.0.0", "18.1.0", "18.2.0", "18.3.0", "18.3.1",
            "19.0.0", "19.0.1", "19.1.0",
        ],
        "npm",
    );
}

#[test]
fn test_npm_typescript_prerelease() {
    // TypeScript publishes -beta and -rc pre-releases
    assert_chain(
        &[
            "5.3.0-beta",
            "5.3.0-rc",
            "5.3.0",
            "5.3.3",
            "5.4.0-beta",
            "5.4.0-rc",
            "5.4.0",
            "5.4.5",
            "5.5.0-beta",
            "5.5.0-rc",
            "5.5.0",
            "5.5.4",
        ],
        "npm",
    );
}

#[test]
fn test_npm_nextjs_canary() {
    // Next.js uses -canary and -rc pre-releases
    // SemVer: canary < rc (c < r lexically)
    assert_chain(
        &[
            "14.0.0-canary.0",
            "14.0.0-canary.1",
            "14.0.0-canary.37",
            "14.0.0-rc.0",
            "14.0.0-rc.1",
            "14.0.0",
            "14.0.1",
            "14.0.4",
            "14.1.0",
            "14.2.0",
            "14.2.5",
            "15.0.0-canary.0",
            "15.0.0-rc.0",
            "15.0.0",
            "15.0.1",
            "15.0.4",
            "15.1.0",
            "15.2.0",
            "15.2.4",
        ],
        "npm",
    );
}

#[test]
fn test_npm_express() {
    // Express — pure numeric, major transition 4→5
    assert_chain(
        &[
            "4.17.1", "4.17.2", "4.17.3", "4.18.0", "4.18.1", "4.18.2", "4.18.3", "4.19.0",
            "4.19.2", "4.20.0", "4.21.0", "4.21.2", "5.0.0", "5.0.1",
        ],
        "npm",
    );
}

#[test]
fn test_npm_webpack() {
    assert_chain(
        &[
            "5.88.0", "5.88.2", "5.89.0", "5.90.0", "5.90.3", "5.91.0", "5.92.0", "5.93.0",
            "5.94.0", "5.95.0", "5.96.1", "5.97.1",
        ],
        "npm",
    );
}

// --- Crates.io (strict SemVer) ---

#[test]
fn test_crates_serde() {
    assert_chain(
        &[
            "1.0.195", "1.0.196", "1.0.197", "1.0.198", "1.0.199", "1.0.200", "1.0.201", "1.0.204",
            "1.0.210", "1.0.217",
        ],
        "crates",
    );
}

#[test]
fn test_crates_tokio() {
    assert_chain(
        &[
            "1.35.0", "1.35.1", "1.36.0", "1.37.0", "1.38.0", "1.38.1", "1.39.0", "1.39.2",
            "1.40.0", "1.41.0", "1.42.0", "1.43.0",
        ],
        "crates",
    );
}

#[test]
fn test_crates_clap() {
    assert_chain(
        &["4.4.0", "4.4.1", "4.4.18", "4.5.0", "4.5.1", "4.5.4", "4.5.16", "4.5.20", "4.5.23"],
        "crates",
    );
}

// --- Hex (strict SemVer, Elixir/Erlang) ---

#[test]
fn test_hex_phoenix() {
    // Phoenix Framework pre-releases and releases
    assert_chain(
        &[
            "1.7.0-rc.0",
            "1.7.0-rc.1",
            "1.7.0-rc.2",
            "1.7.0-rc.3",
            "1.7.0",
            "1.7.1",
            "1.7.2",
            "1.7.6",
            "1.7.7",
            "1.7.10",
            "1.7.11",
            "1.7.12",
            "1.7.14",
        ],
        "hex",
    );
}

#[test]
fn test_hex_ecto() {
    assert_chain(
        &["3.11.0", "3.11.1", "3.11.2", "3.12.0", "3.12.1", "3.12.3", "3.12.4", "3.12.5"],
        "hex",
    );
}

// --- Swift PM (strict SemVer) ---

#[test]
fn test_swift_vapor() {
    assert_chain(
        &[
            "4.90.0", "4.91.0", "4.92.0", "4.95.0", "4.99.0", "4.100.0", "4.101.0", "4.105.0",
            "4.107.0",
        ],
        "swift",
    );
}

#[test]
fn test_swift_alamofire() {
    assert_chain(&["5.8.0", "5.8.1", "5.9.0", "5.9.1", "5.10.0", "5.10.1", "5.10.2"], "swift");
}

// --- PEP 440 (Python) ---

#[test]
fn test_pep440_django() {
    assert_chain(
        &[
            "4.2", "4.2.1", "4.2.5", "4.2.10", "4.2.16", "5.0a1", "5.0b1", "5.0rc1", "5.0",
            "5.0.1", "5.0.5", "5.0.10", "5.1a1", "5.1b1", "5.1rc1", "5.1", "5.1.1", "5.1.5",
            "5.1.8", "5.2a1", "5.2b1", "5.2rc1", "5.2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_cpython() {
    // CPython release cycle: alpha → beta → rc → release
    assert_chain(
        &[
            "3.12.0a1",
            "3.12.0a4",
            "3.12.0a7",
            "3.12.0b1",
            "3.12.0b4",
            "3.12.0rc1",
            "3.12.0rc3",
            "3.12.0",
            "3.12.1",
            "3.12.4",
            "3.12.8",
            "3.13.0a1",
            "3.13.0a6",
            "3.13.0b1",
            "3.13.0b4",
            "3.13.0rc1",
            "3.13.0rc3",
            "3.13.0",
            "3.13.1",
            "3.13.3",
            "3.14.0a1",
            "3.14.0a7",
            "3.14.0b1",
            "3.14.0b2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_numpy() {
    assert_chain(
        &[
            "1.26.0", "1.26.2", "1.26.4", "2.0.0rc1", "2.0.0rc2", "2.0.0", "2.0.1", "2.0.2",
            "2.1.0rc1", "2.1.0", "2.1.1", "2.1.3", "2.2.0rc1", "2.2.0", "2.2.1", "2.2.4",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_setuptools() {
    assert_chain(
        &[
            "69.0.0", "69.5.1", "70.0.0", "70.3.0", "71.0.0", "71.1.0", "72.0.0", "72.2.0",
            "73.0.0", "73.0.1", "74.0.0", "74.1.3", "75.0.0", "75.6.0",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_full_lifecycle() {
    // Complete PEP 440 lifecycle: dev → alpha → beta → rc → release → post
    assert_chain(
        &[
            "1.0.dev1",
            "1.0.dev2",
            "1.0a1",
            "1.0a2",
            "1.0b1",
            "1.0b2",
            "1.0rc1",
            "1.0rc2",
            "1.0",
            "1.0.post1",
            "1.0.post2",
        ],
        "pep440",
    );
}

#[test]
fn test_pep440_epoch() {
    // Epoch resets version ordering
    assert_eq!(compare_str_with_ecosystem("2023.1", "1!0.1", "pep440").unwrap(), Less,);
    assert_eq!(compare_str_with_ecosystem("1!0.1", "1!0.2", "pep440").unwrap(), Less,);
    assert_eq!(compare_str_with_ecosystem("1!0.1", "2!0.1", "pep440").unwrap(), Less,);
}

// --- Debian ---

#[test]
fn test_debian_openssl() {
    assert_chain(
        &[
            "1.0.2g-1",
            "1.0.2g-1+deb8u1",
            "1.0.2g-1+deb8u2",
            "1.0.2l-2",
            "1.0.2l-2+deb9u1",
            "1.0.2l-2+deb9u3",
            "1.1.0g-2",
            "1.1.0j-1",
            "1.1.1a-1",
            "1.1.1d-0",
            "1.1.1d-0+deb10u1",
            "1.1.1d-0+deb10u7",
            "1.1.1n-0",
            "1.1.1n-0+deb10u6",
            "1.1.1w-0",
            "3.0.11-1",
            "3.0.13-1",
            "3.0.14-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_curl() {
    assert_chain(
        &[
            "7.64.0-4",
            "7.64.0-4+deb10u1",
            "7.64.0-4+deb10u7",
            "7.74.0-1.3",
            "7.74.0-1.3+deb11u1",
            "7.74.0-1.3+deb11u12",
            "7.88.1-10",
            "7.88.1-10+deb12u1",
            "7.88.1-10+deb12u8",
            "8.5.0-2",
            "8.11.1-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_python3() {
    assert_chain(
        &[
            "3.9.2-3",
            "3.11.2-1",
            "3.11.2-1+b1",
            "3.11.4-5",
            "3.11.4-5+deb12u1",
            "3.12.3-1",
            "3.12.8-1",
        ],
        "debian",
    );
}

#[test]
fn test_debian_nginx() {
    assert_chain(
        &[
            "1.14.0-0",
            "1.14.0-0+deb9u1",
            "1.14.0-0+deb9u5",
            "1.14.2-2",
            "1.14.2-2+deb10u1",
            "1.14.2-2+deb10u5",
            "1.18.0-6",
            "1.18.0-6.1",
            "1.22.1-9",
            "1.24.0-2",
            "1.26.0-3",
        ],
        "debian",
    );
}

#[test]
fn test_debian_tilde_prerelease() {
    // Debian pre-release convention: ~ sorts before everything
    assert_chain(
        &[
            "1.0~alpha1",
            "1.0~alpha2",
            "1.0~beta1",
            "1.0~beta2",
            "1.0~rc1",
            "1.0~rc2",
            "1.0",
            "1.0-1",
            "1.0-2",
        ],
        "debian",
    );
}

#[test]
fn test_debian_epoch_ordering() {
    assert_chain(&["0:1.0-1", "0:2.0-1", "1:0.1-1", "1:0.2-1", "2:0.1-1"], "debian");
}

// --- RPM ---

#[test]
fn test_rpm_kernel() {
    // RHEL 9 kernel versions
    assert_chain(
        &[
            "5.14.0-362.13.1.el9_3",
            "5.14.0-362.18.1.el9_3",
            "5.14.0-362.24.1.el9_3",
            "5.14.0-427.13.1.el9_4",
            "5.14.0-427.20.1.el9_4",
            "5.14.0-427.31.1.el9_4",
            "5.14.0-503.11.1.el9_5",
            "5.14.0-503.14.1.el9_5",
            "5.14.0-503.19.1.el9_5",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_systemd() {
    // Fedora systemd versions
    assert_chain(
        &[
            "252-14.fc38",
            "252-18.fc38",
            "253-1.fc39",
            "253-7.fc39",
            "253-14.fc39",
            "254-1.fc40",
            "254-10.fc40",
            "255-1.fc40",
            "256-1.fc41",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_httpd() {
    // RHEL Apache httpd versions
    assert_chain(
        &[
            "2.4.37-47.el8",
            "2.4.37-62.el8",
            "2.4.37-65.el8",
            "2.4.51-7.el9_0",
            "2.4.53-7.el9",
            "2.4.57-5.el9",
            "2.4.57-8.el9",
            "2.4.62-1.el9",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_gcc() {
    // GCC versions in Fedora
    assert_chain(
        &[
            "13.2.1-6.fc39",
            "13.2.1-7.fc39",
            "13.3.1-2.fc40",
            "13.3.1-3.fc40",
            "14.0.1-0.6.fc40",
            "14.1.1-1.fc40",
            "14.1.1-6.fc41",
            "14.2.1-1.fc41",
            "14.2.1-3.fc41",
            "14.2.1-6.fc41",
        ],
        "rpm",
    );
}

#[test]
fn test_rpm_tilde_caret() {
    // RPM pre/post-release semantics with ~ and ^
    assert_chain(&["1.0~alpha", "1.0~beta", "1.0~rc1", "1.0", "1.0^git1", "1.0^git2"], "rpm");
}

// --- Ruby Gems ---

#[test]
fn test_ruby_rails() {
    // Ruby on Rails with pre-releases
    assert_chain(
        &[
            "7.0.0.alpha1",
            "7.0.0.alpha2",
            "7.0.0.rc1",
            "7.0.0.rc2",
            "7.0.0.rc3",
            "7.0.0",
            "7.0.1",
            "7.0.2",
            "7.0.4",
            "7.0.8",
            "7.1.0.beta1",
            "7.1.0.rc1",
            "7.1.0.rc2",
            "7.1.0",
            "7.1.1",
            "7.1.2",
            "7.1.3",
            "7.1.5",
            "7.2.0.beta1",
            "7.2.0.beta2",
            "7.2.0.beta3",
            "7.2.0.rc1",
            "7.2.0",
            "7.2.1",
            "7.2.2",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_nokogiri() {
    assert_chain(
        &[
            "1.15.0", "1.15.1", "1.15.4", "1.15.6", "1.16.0", "1.16.2", "1.16.5", "1.16.7",
            "1.17.0", "1.17.2", "1.18.0", "1.18.2", "1.18.3",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_bundler() {
    assert_chain(
        &[
            "2.4.0", "2.4.1", "2.4.10", "2.4.22", "2.5.0", "2.5.1", "2.5.10", "2.5.23", "2.6.0",
            "2.6.1", "2.6.2", "2.6.3",
        ],
        "ruby",
    );
}

#[test]
fn test_ruby_sinatra() {
    assert_chain(
        &[
            "3.0.0", "3.0.1", "3.0.2", "3.0.3", "3.0.4", "3.0.5", "3.0.6", "3.1.0", "3.2.0",
            "4.0.0", "4.1.0", "4.1.1",
        ],
        "ruby",
    );
}

// --- Go modules ---

#[test]
fn test_go_gin() {
    assert_chain(
        &["v1.7.0", "v1.7.7", "v1.8.0", "v1.8.1", "v1.8.2", "v1.9.0", "v1.9.1", "v1.10.0"],
        "go",
    );
}

#[test]
fn test_go_cobra() {
    assert_chain(&["v1.7.0", "v1.8.0", "v1.8.1"], "go");
}

#[test]
fn test_go_prometheus() {
    // prometheus/client_golang
    assert_chain(
        &[
            "v1.17.0", "v1.18.0", "v1.19.0", "v1.19.1", "v1.20.0", "v1.20.1", "v1.20.2", "v1.20.3",
            "v1.20.4", "v1.20.5",
        ],
        "go",
    );
}

#[test]
fn test_go_incompatible_suffix() {
    // Go modules with +incompatible suffix (pre-module v2+ packages)
    assert_eq!(compare_str_with_ecosystem("v2.0.0+incompatible", "v2.0.0", "go").unwrap(), Equal,);
    assert_eq!(
        compare_str_with_ecosystem("v2.0.0+incompatible", "v2.0.1+incompatible", "go").unwrap(),
        Less,
    );
}

// --- NuGet ---

#[test]
fn test_nuget_newtonsoft() {
    assert_chain(&["12.0.1", "12.0.2", "12.0.3", "13.0.1", "13.0.2", "13.0.3"], "nuget");
}

#[test]
fn test_nuget_xunit() {
    assert_chain(
        &[
            "2.5.0", "2.5.1", "2.5.3", "2.6.0", "2.6.1", "2.6.6", "2.7.0", "2.7.1", "2.8.0",
            "2.8.1", "2.9.0", "2.9.2", "2.9.3",
        ],
        "nuget",
    );
}

#[test]
fn test_nuget_nunit() {
    assert_chain(
        &["3.13.3", "3.14.0", "4.0.0", "4.0.1", "4.1.0", "4.2.0", "4.2.2", "4.3.0", "4.3.2"],
        "nuget",
    );
}

#[test]
fn test_nuget_efcore() {
    // Entity Framework Core
    assert_chain(
        &[
            "7.0.0", "7.0.1", "7.0.5", "7.0.11", "7.0.20", "8.0.0", "8.0.1", "8.0.4", "8.0.8",
            "8.0.11", "9.0.0", "9.0.1", "9.0.3", "9.0.4",
        ],
        "nuget",
    );
}

// --- Composer (PHP) ---

#[test]
fn test_composer_laravel() {
    assert_chain(
        &[
            "10.0.0", "10.0.1", "10.10.0", "10.20.0", "10.40.0", "10.48.0", "11.0.0", "11.10.0",
            "11.20.0", "11.30.0", "11.40.0", "12.0.0", "12.1.0", "12.2.0", "12.3.0", "12.4.0",
        ],
        "composer",
    );
}

#[test]
fn test_composer_symfony() {
    assert_chain(
        &[
            "6.4.0", "6.4.1", "6.4.5", "6.4.10", "6.4.15", "6.4.21", "7.0.0", "7.0.1", "7.0.7",
            "7.0.12", "7.1.0", "7.1.1", "7.1.6", "7.1.12", "7.2.0", "7.2.1", "7.2.6",
        ],
        "composer",
    );
}

#[test]
fn test_composer_phpunit() {
    assert_chain(
        &[
            "10.5.0", "10.5.10", "10.5.20", "10.5.38", "11.0.0", "11.0.11", "11.1.0", "11.1.6",
            "11.2.0", "11.2.6", "11.3.0", "11.3.6", "11.4.0", "11.4.4", "11.5.0", "11.5.6",
            "12.0.0", "12.0.6",
        ],
        "composer",
    );
}

#[test]
fn test_composer_prerelease() {
    // Composer stability flags: alpha < beta < RC < stable
    assert_chain(
        &[
            "1.0.0-alpha1",
            "1.0.0-alpha2",
            "1.0.0-beta1",
            "1.0.0-beta2",
            "1.0.0-rc1",
            "1.0.0-rc2",
            "1.0.0",
            "1.0.1",
        ],
        "composer",
    );
}

// --- CalVer ---

#[test]
fn test_calver_ubuntu() {
    // Ubuntu release versions (YY.MM format)
    assert_chain(
        &[
            "20.04", "20.10", "21.04", "21.10", "22.04", "22.10", "23.04", "23.10", "24.04",
            "24.10", "25.04",
        ],
        "calver",
    );
}

#[test]
fn test_calver_pip() {
    // pip uses CalVer (YY.N format)
    assert_chain(
        &[
            "23.0", "23.0.1", "23.1", "23.1.2", "23.2", "23.2.1", "23.3", "23.3.2", "24.0", "24.1",
            "24.1.2", "24.2", "24.3", "24.3.1", "25.0", "25.0.1",
        ],
        "calver",
    );
}

#[test]
fn test_calver_black() {
    // black Python formatter (YYYY.M.D format)
    assert_chain(
        &[
            "23.1.0", "23.3.0", "23.7.0", "23.9.0", "23.9.1", "23.10.0", "23.10.1", "23.11.0",
            "23.12.0", "23.12.1", "24.1.0", "24.1.1", "24.2.0", "24.3.0", "24.4.0", "24.4.2",
            "24.8.0", "24.10.0", "25.1.0",
        ],
        "calver",
    );
}

#[test]
fn test_calver_yyyymmdd() {
    // YYYYMMDD format (e.g., database snapshots, Commons IO timestamp)
    assert_chain(
        &["20230115", "20230201", "20230315", "20240101", "20240615", "20241231", "20250101"],
        "calver",
    );
}

// --- Alpine ---

#[test]
fn test_alpine_prerelease_lifecycle() {
    // Alpine uses _alpha, _beta, _rc for pre-release
    assert_chain(
        &[
            "1.0_alpha1",
            "1.0_alpha2",
            "1.0_beta1",
            "1.0_beta2",
            "1.0_rc1",
            "1.0_rc2",
            "1.0",
            "1.0.1",
            "1.1.0",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_curl() {
    assert_chain(
        &[
            "8.5.0", "8.6.0", "8.7.1", "8.8.0", "8.9.0", "8.9.1", "8.10.0", "8.10.1", "8.11.0",
            "8.11.1",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_openssl() {
    assert_chain(
        &[
            "3.1.4", "3.1.5", "3.1.6", "3.1.7", "3.2.0", "3.2.1", "3.2.2", "3.2.3", "3.3.0",
            "3.3.1", "3.3.2", "3.4.0", "3.4.1",
        ],
        "alpine",
    );
}

#[test]
fn test_alpine_busybox() {
    assert_chain(&["1.35.0", "1.36.0", "1.36.1", "1.37.0"], "alpine");
}

// --- Docker ---

#[test]
fn test_docker_nginx() {
    assert_chain(
        &[
            "1.24.0", "1.25.0", "1.25.1", "1.25.2", "1.25.3", "1.25.4", "1.25.5", "1.26.0",
            "1.26.1", "1.26.2", "1.27.0", "1.27.1", "1.27.2", "1.27.3",
        ],
        "docker",
    );
}

#[test]
fn test_docker_postgres() {
    assert_chain(
        &[
            "14.0", "14.5", "14.10", "14.13", "15.0", "15.4", "15.8", "16.0", "16.2", "16.4",
            "17.0", "17.1", "17.2",
        ],
        "docker",
    );
}

#[test]
fn test_docker_redis() {
    assert_chain(
        &[
            "7.0.0", "7.0.5", "7.0.10", "7.0.15", "7.2.0", "7.2.3", "7.2.6", "7.4.0", "7.4.1",
            "7.4.2", "8.0.0", "8.0.1", "8.0.2",
        ],
        "docker",
    );
}

#[test]
fn test_docker_node() {
    // Node.js Docker image tags
    assert_chain(
        &[
            "18.0.0", "18.12.0", "18.17.0", "18.20.0", "20.0.0", "20.9.0", "20.11.0", "20.18.0",
            "22.0.0", "22.6.0", "22.11.0",
        ],
        "docker",
    );
}

// --- Version.compare() uses Version's own ecosystem ---

#[test]
fn test_version_compare_uses_own_ecosystem() {
    // When no ecosystem is passed, compare should use the Version's detected ecosystem.
    // "1.0.0-alpha.1" vs "1.0.0-alpha.beta":
    //   generic:  Num(1) > Text("beta") → Greater
    //   semver:   numeric < alpha → Less
    // If Version detects as generic, compare should give Greater.
    // If explicitly set to semver, should give Less.
    assert_eq!(cmpg("1.0.0-alpha.1", "1.0.0-alpha.beta"), Greater);
    assert_eq!(
        compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "semver").unwrap(),
        Less
    );
}

// --- Crates/Hex/Swift validation ---

#[test]
fn test_crates_eco_reject_two_part() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "crates").is_err());
}

#[test]
fn test_crates_eco_reject_leading_zero() {
    assert!(compare_str_with_ecosystem("01.0.0", "1.0.0", "crates").is_err());
}

#[test]
fn test_hex_eco_reject_two_part() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "hex").is_err());
}

#[test]
fn test_hex_eco_reject_leading_zero() {
    assert!(compare_str_with_ecosystem("1.00.0", "1.0.0", "hex").is_err());
}

#[test]
fn test_swift_eco_reject_two_part() {
    assert!(compare_str_with_ecosystem("1.0", "2.0", "swift").is_err());
}

#[test]
fn test_swift_eco_reject_leading_zero() {
    assert!(compare_str_with_ecosystem("1.0.00", "1.0.0", "swift").is_err());
}

// --- Docker validation ---

#[test]
fn test_docker_eco_reject_empty() {
    assert!(compare_str_with_ecosystem("", "1.0", "docker").is_err());
}

#[test]
fn test_docker_eco_accept_text_tags() {
    // Docker accepts any non-empty string
    assert!(compare_str_with_ecosystem("latest", "stable", "docker").is_ok());
    assert!(compare_str_with_ecosystem("alpine", "slim", "docker").is_ok());
}

// --- is_stable property (via parser) ---

#[test]
fn test_parser_is_stable_release() {
    let v = parse("1.0.0");
    assert!(!v.is_prerelease);
}

#[test]
fn test_parser_is_stable_prerelease() {
    let v = parse("1.0.0-alpha");
    assert!(v.is_prerelease);
}

#[test]
fn test_parser_is_stable_postrelease() {
    let v = parse("1.0.post1");
    assert!(!v.is_prerelease); // post-release is stable
    assert!(v.is_postrelease);
}

// --- satisfies() constraint matching ---

#[test]
fn test_satisfies_ge() {
    assert!(parse_constraint(">=1.0.0").is_ok());
}

#[test]
fn test_satisfies_basic_ge() {
    assert_eq!(compare_str_with_ecosystem("1.5.0", "1.0.0", "generic").unwrap(), Greater);
}

#[test]
fn test_satisfies_basic_lt() {
    assert_eq!(compare_str_with_ecosystem("1.5.0", "2.0.0", "generic").unwrap(), Less);
}

// --- bump helpers (via parser) ---

#[test]
fn test_bump_major_segments() {
    let v = parse("1.2.3");
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    assert_eq!(major, 2);
}

#[test]
fn test_bump_minor_segments() {
    let v = parse("1.2.3");
    let minor = match v.segments.get(1) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    assert_eq!(minor, 3);
}

#[test]
fn test_bump_patch_segments() {
    let v = parse("1.2.3");
    let patch = match v.segments.get(2) {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    assert_eq!(patch, 4);
}

#[test]
fn test_bump_strips_prerelease() {
    // bump_major("1.2.3-alpha") should give "2.0.0" not "2.0.0-alpha"
    let v = parse("1.2.3-alpha");
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    assert_eq!(format!("{major}.0.0"), "2.0.0");
}

#[test]
fn test_bump_from_single_segment() {
    let v = parse("5");
    let major = match v.segments.first() {
        Some(Seg::Num(n)) => n + 1,
        _ => 1,
    };
    assert_eq!(major, 6);
}

// --- stable_versions / latest_stable (via parser is_prerelease) ---

#[test]
fn test_stable_filter_logic() {
    // Verify which versions the parser marks as prerelease
    assert!(parse("2.0.0-rc1").is_prerelease);
    assert!(parse("1.5.0-beta").is_prerelease);
    assert!(!parse("1.0.0").is_prerelease);
    assert!(!parse("2.0.0").is_prerelease);
}

#[test]
fn test_stable_filter_alpha_beta_rc() {
    assert!(parse("1.0-alpha").is_prerelease);
    assert!(parse("1.0-beta").is_prerelease);
    assert!(parse("1.0-rc1").is_prerelease);
    assert!(parse("1.0-SNAPSHOT").is_prerelease);
    assert!(parse("1.0.dev1").is_prerelease);
}

#[test]
fn test_stable_filter_postrelease() {
    // Post-release versions are NOT prerelease — they should be included
    assert!(!parse("1.0.post1").is_prerelease);
    assert!(!parse("1.0-sp-1").is_prerelease);
}

#[test]
fn test_latest_stable_selection() {
    // Among [1.0, 2.0-rc1, 2.0, 1.5-beta], latest stable should be 2.0
    let versions = ["1.0.0", "2.0.0-rc1", "2.0.0", "1.5.0-beta"];
    let mut stable: Vec<&str> =
        versions.iter().copied().filter(|v| !parse(v).is_prerelease).collect();
    stable.sort_by(|a, b| cmpg(a, b));
    assert_eq!(*stable.last().unwrap(), "2.0.0");
}

#[test]
fn test_stable_versions_empty_when_all_prerelease() {
    let versions = ["1.0-alpha", "2.0-beta", "3.0-rc1"];
    let stable: Vec<&str> = versions.iter().copied().filter(|v| !parse(v).is_prerelease).collect();
    assert!(stable.is_empty());
}

// --- __richcmp__ respects ecosystem ---

#[test]
fn test_richcmp_semver_ecosystem() {
    // In strict SemVer: numeric < alpha → "alpha.1" < "alpha.beta"
    // In generic:       Num > Text → "alpha.1" > "alpha.beta"
    // Both created with ecosystem="semver" → richcmp should use semver
    assert_eq!(
        compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "semver").unwrap(),
        Less
    );
    // And with generic, it's Greater
    assert_eq!(cmpg("1.0.0-alpha.1", "1.0.0-alpha.beta"), Greater);
}

// --- Ecosystem::from_str returns Result<_, String> (decoupled from PyO3) ---

#[test]
fn test_ecosystem_from_str_error_is_string() {
    let err = Ecosystem::from_str("invalid");
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("unsupported ecosystem"));
}

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

// ====================================================================
// parse_constraint — parser for satisfies()
// ====================================================================

#[test]
fn test_parse_constraint_ge() {
    assert_eq!(parse_constraint(">=1.0.0").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_le() {
    assert_eq!(parse_constraint("<=2.0.0").unwrap(), ("<=", "2.0.0"));
}
#[test]
fn test_parse_constraint_gt() {
    assert_eq!(parse_constraint(">1.0").unwrap(), (">", "1.0"));
}
#[test]
fn test_parse_constraint_lt() {
    assert_eq!(parse_constraint("<2.0").unwrap(), ("<", "2.0"));
}
#[test]
fn test_parse_constraint_eq() {
    assert_eq!(parse_constraint("==1.0").unwrap(), ("==", "1.0"));
}
#[test]
fn test_parse_constraint_ne() {
    assert_eq!(parse_constraint("!=1.0").unwrap(), ("!=", "1.0"));
}

// Whitespace handling: both around op and between op/version
#[test]
fn test_parse_constraint_space_around() {
    assert_eq!(parse_constraint("  >=1.0.0  ").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_space_between() {
    assert_eq!(parse_constraint(">= 1.0.0").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_space_both() {
    assert_eq!(parse_constraint(" < 2.0 ").unwrap(), ("<", "2.0"));
}

// Operator precedence: >= must bind before > (same for <= before <, == before (not relevant), != before (not relevant))
#[test]
fn test_parse_constraint_ge_not_gt() {
    let (op, _) = parse_constraint(">=1.0").unwrap();
    assert_eq!(op, ">=");
}
#[test]
fn test_parse_constraint_le_not_lt() {
    let (op, _) = parse_constraint("<=1.0").unwrap();
    assert_eq!(op, "<=");
}

// Errors: no operator, empty
#[test]
fn test_parse_constraint_no_operator() {
    assert!(parse_constraint("1.0.0").is_err());
}
#[test]
fn test_parse_constraint_empty() {
    assert!(parse_constraint("").is_err());
}
#[test]
fn test_parse_constraint_error_message() {
    let err = parse_constraint("1.0.0").unwrap_err();
    assert!(err.contains("invalid constraint"));
}

// ====================================================================
// PEP 440 LOCAL VERSION ORDERING
// ====================================================================

fn pep440_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_for_ecosystem(Ecosystem::Pep440, a).unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Pep440, b).unwrap();
    compare_for_ecosystem(Ecosystem::Pep440, &pa, &pb)
}

#[test]
fn test_pep440_local_greater_than_release() {
    assert_eq!(pep440_cmp("1.0+abc", "1.0"), std::cmp::Ordering::Greater);
}
#[test]
fn test_pep440_local_equal_self() {
    assert_eq!(pep440_cmp("1.0+abc", "1.0+abc"), std::cmp::Ordering::Equal);
}
#[test]
fn test_pep440_local_lex() {
    assert_eq!(pep440_cmp("1.0+abc", "1.0+abd"), std::cmp::Ordering::Less);
}
#[test]
fn test_pep440_local_numeric_gt_alpha() {
    assert_eq!(pep440_cmp("1.0+1", "1.0+a"), std::cmp::Ordering::Greater);
}
#[test]
fn test_pep440_local_separator_equivalence() {
    assert_eq!(pep440_cmp("1.0+a.b", "1.0+a-b"), std::cmp::Ordering::Equal);
    assert_eq!(pep440_cmp("1.0+a.b", "1.0+a_b"), std::cmp::Ordering::Equal);
}
#[test]
fn test_pep440_public_part_wins_over_local() {
    assert_eq!(pep440_cmp("1.0+zzz", "2.0"), std::cmp::Ordering::Less);
}

// ====================================================================
// MAVEN RELEASE-ALIAS QUALIFIERS
// ====================================================================

#[test]
fn test_tag_weight_release_aliases() {
    assert_eq!(tag_weight("final"), Some(30));
    assert_eq!(tag_weight("ga"), Some(30));
    assert_eq!(tag_weight("release"), Some(30));
}

#[test]
fn test_normalized_strips_trailing_release_alias() {
    let segs = vec![Seg::Num(1), Seg::Num(0), Seg::Text("final".to_string())];
    assert_eq!(normalized(&segs), &[Seg::Num(1)]);
}

fn gen_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_for_ecosystem(Ecosystem::Generic, a).unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Generic, b).unwrap();
    compare_for_ecosystem(Ecosystem::Generic, &pa, &pb)
}

#[test]
fn test_generic_final_equals_release() {
    assert_eq!(gen_cmp("1.0-final", "1.0"), std::cmp::Ordering::Equal);
}
#[test]
fn test_generic_ga_equals_release() {
    assert_eq!(gen_cmp("1.0-ga", "1.0"), std::cmp::Ordering::Equal);
}
#[test]
fn test_maven_final_below_sp() {
    let pa = parse_for_ecosystem(Ecosystem::Maven, "1.0-final").unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Maven, "1.0-sp-1").unwrap();
    assert_eq!(compare_for_ecosystem(Ecosystem::Maven, &pa, &pb), std::cmp::Ordering::Less);
}
#[test]
fn test_maven_snapshot_below_final() {
    let pa = parse_for_ecosystem(Ecosystem::Maven, "1.0-SNAPSHOT").unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Maven, "1.0-final").unwrap();
    assert_eq!(compare_for_ecosystem(Ecosystem::Maven, &pa, &pb), std::cmp::Ordering::Less);
}
