use super::*;

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
