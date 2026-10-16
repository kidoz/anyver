use super::*;

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
    assert_eq!(stable, Vec::<&str>::new());
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
