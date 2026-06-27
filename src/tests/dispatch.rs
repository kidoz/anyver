use super::*;

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

#[test]
fn test_go_eco_requires_v_prefix() {
    // Spec: Go modules always carry a mandatory lowercase `v` prefix.
    assert!(compare_str_with_ecosystem("1.0.0", "1.0.1", "go").is_err());
}

#[test]
fn test_go_eco_rejects_uppercase_v() {
    assert!(compare_str_with_ecosystem("V1.0.0", "V1.0.1", "go").is_err());
}

#[test]
fn test_go_eco_accepts_incompatible_and_pseudo() {
    // The `v` requirement must not reject legitimate Go forms.
    assert_eq!(compare_str_with_ecosystem("v2.0.0+incompatible", "v2.0.0", "go").unwrap(), Equal);
    assert_eq!(
        compare_str_with_ecosystem(
            "v0.0.0-20170915032832-14c0d48ead0c",
            "v0.0.0-20180915032832-14c0d48ead0c",
            "go"
        )
        .unwrap(),
        Less
    );
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
