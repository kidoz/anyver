pub mod parser;
pub mod python;
pub mod strategies;

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use crate::python::*;
    use crate::strategies::*;
    use std::cmp::Ordering;
    use std::cmp::Ordering::*;

    fn cmpg(a: &str, b: &str) -> Ordering {
        cmp_parsed(&parse(a), &parse(b))
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
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "{} should be < {}",
                chain[i],
                chain[i + 1]
            );
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

    // ---- Real-world: Apache Maven Core lifecycle ----
    #[test]
    fn test_maven_rw_maven_core_alpha_chain() {
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

    // ---- Real-world: Log4j lifecycle ----
    #[test]
    fn test_maven_rw_log4j_lifecycle() {
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

    // ---- Real-world: JUnit 4 lifecycle ----
    #[test]
    fn test_maven_rw_junit4_lifecycle() {
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

    // ---- Real-world: JUnit Jupiter lifecycle ----
    #[test]
    fn test_maven_rw_junit_jupiter_lifecycle() {
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

    // ---- Real-world: Spring Core milestone lifecycle ----
    #[test]
    fn test_maven_rw_spring_core_milestones() {
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

    // ---- Real-world: Guava lifecycle ----
    #[test]
    fn test_maven_rw_guava_rc_chain() {
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

    // ---- Real-world: SLF4J lifecycle ----
    #[test]
    fn test_maven_rw_slf4j_lifecycle() {
        let chain = [
            "2.0.0-alpha0",
            "2.0.0-alpha7",
            "2.0.0-beta0",
            "2.0.0-beta1",
            "2.0.0",
            "2.0.1",
            "2.0.17",
        ];
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

    // ---- Real-world: Apache Commons Lang3 ----
    #[test]
    fn test_maven_rw_commons_lang3() {
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

    // ---- Real-world: Jackson Databind ----
    #[test]
    fn test_maven_rw_jackson_four_component() {
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
    fn test_maven_rw_jackson_rc_lifecycle() {
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

    // ---- Real-world: Kafka Clients ----
    #[test]
    fn test_maven_rw_kafka_four_component() {
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

    // ---- Real-world: Log4j 3.0.0 alpha/beta lifecycle ----
    #[test]
    fn test_maven_rw_log4j3_prerelease() {
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

    // ---- Real-world: Maven Core 4.x prerelease ----
    #[test]
    fn test_maven_rw_maven4_prerelease() {
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

    // ---- Real-world: Commons IO with mixed component counts ----
    #[test]
    fn test_maven_rw_commons_io() {
        let chain =
            ["0.1", "1.0", "1.3.2", "1.4", "2.0", "2.0.1", "2.7", "2.8.0", "2.16.1", "2.21.0"];
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

    // ---- Real-world: Hibernate dot-separated qualifiers ----
    #[test]
    fn test_maven_rw_hibernate_lifecycle() {
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

    // ---- Real-world: Netty dot-separated qualifiers ----
    #[test]
    fn test_maven_rw_netty_lifecycle() {
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

    // ---- Real-world: timestamp version (Commons IO) ----
    #[test]
    fn test_maven_rw_timestamp_version() {
        // Commons IO had a timestamp version; it's just a huge numeric value
        assert_eq!(cmpg("2.21.0", "20030203.000550"), Less);
    }

    // ---- Maven ecosystem dispatch: real-world ----
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

    // --- Real-world packages ---

    #[test]
    fn test_rw_django_major() {
        assert_eq!(cmpg("4.2", "5.0a1"), Less);
    }
    #[test]
    fn test_rw_django_alpha_lt_beta() {
        assert_eq!(cmpg("5.0a1", "5.0b1"), Less);
    }
    #[test]
    fn test_rw_django_rc_lt_rel() {
        assert_eq!(cmpg("5.0rc1", "5.0"), Less);
    }
    #[test]
    fn test_rw_rails_beta_lt_rc() {
        assert_eq!(cmpg("7.1.0.beta1", "7.1.0.rc1"), Less);
    }
    #[test]
    fn test_rw_rails_rc_lt_rel() {
        assert_eq!(cmpg("7.1.0.rc1", "7.1.0"), Less);
    }
    #[test]
    fn test_rw_node() {
        assert_eq!(cmpg("18.17.0", "20.0.0"), Less);
    }
    #[test]
    fn test_rw_kernel() {
        assert_eq!(cmpg("5.15.0", "6.1.0"), Less);
    }
    #[test]
    fn test_rw_spring() {
        assert_eq!(cmpg("5.3.30", "6.0.0"), Less);
    }
    #[test]
    fn test_rw_requests() {
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
        assert_eq!(
            compare_str_with_ecosystem("1.0.0-alpha", "1.0.0-beta", "composer").unwrap(),
            Less
        );
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
}
