//! Regression tests for the VERS grammar, the OSV ecosystem-name mapping and
//! event timelines, and PURL construction.

use crate::parser::Ecosystem;
use crate::ranges::RangeSet;
use crate::security::{OsvEvent, Purl, Vers, osv_ecosystem, osv_range_windows, validate_purl_type};

// ====================================================================
// VERS — spec grammar and containment
// ====================================================================

#[test]
fn test_vers_pipe_separator() {
    let vers = Vers::parse("vers:npm/>=2.0.0|<5.0.0").unwrap();
    assert!(!vers.contains("1.0.0").unwrap());
    assert!(vers.contains("2.0.0").unwrap());
    assert!(vers.contains("3.0.0").unwrap());
    assert!(!vers.contains("5.0.0").unwrap());
    assert!(!vers.contains("9.0.0").unwrap());
}

#[test]
fn test_vers_constraints_are_a_set_not_a_conjunction() {
    // The bare `1.2.3` is an equality constraint alongside the interval.
    let vers = Vers::parse("vers:npm/1.2.3|>=2.0.0|<5.0.0").unwrap();
    assert!(vers.contains("1.2.3").unwrap());
    assert!(!vers.contains("1.5.0").unwrap());
    assert!(vers.contains("3.0.0").unwrap());
}

#[test]
fn test_vers_disjoint_windows() {
    let vers = Vers::parse("vers:npm/<1.0.0|>=2.0.0").unwrap();
    assert!(vers.contains("0.5.0").unwrap());
    assert!(!vers.contains("1.5.0").unwrap());
    assert!(vers.contains("3.0.0").unwrap());
}

#[test]
fn test_vers_exclusion_overrides_everything() {
    let vers = Vers::parse("vers:npm/>=1.0.0|<5.0.0|!=2.0.0").unwrap();
    assert!(vers.contains("1.5.0").unwrap());
    assert!(!vers.contains("2.0.0").unwrap());
}

#[test]
fn test_vers_pypi_scheme() {
    let vers = Vers::parse("vers:pypi/>=1.0|<2.0").unwrap();
    assert_eq!(vers.ecosystem, Ecosystem::Pep440);
    assert!(vers.contains("1.5").unwrap());
    assert!(!vers.contains("2.0").unwrap());
}

#[test]
fn test_vers_scheme_names_round_trip() {
    for eco in [
        Ecosystem::Pep440,
        Ecosystem::Debian,
        Ecosystem::Ruby,
        Ecosystem::Go,
        Ecosystem::Crates,
        Ecosystem::Alpine,
        Ecosystem::Npm,
        Ecosystem::Maven,
    ] {
        let scheme = eco.vers_scheme();
        assert_eq!(Ecosystem::from_str(scheme).unwrap(), eco, "scheme '{scheme}' must round-trip");
    }
}

#[test]
fn test_vers_renders_canonically() {
    // Comma-joined legacy input still parses, and re-renders with `|` and the
    // spec's scheme name.
    let vers = Vers::new(Ecosystem::Pep440, ">=1.0.0,<2.0.0").unwrap();
    assert_eq!(vers.to_vers(), "vers:pypi/>=1.0.0|<2.0.0");
}

#[test]
fn test_vers_wildcard() {
    let vers = Vers::parse("vers:npm/*").unwrap();
    assert!(vers.contains("0.0.1").unwrap());
    assert!(vers.contains("99.0.0").unwrap());
    assert_eq!(vers.to_vers(), "vers:npm/*");
}

#[test]
fn test_vers_empty_range_rejected() {
    assert!(Vers::parse("vers:npm/").is_err());
    assert!(Vers::parse("vers:npm/  ").is_err());
}

#[test]
fn test_vers_requires_prefix_and_scheme() {
    assert!(Vers::parse("npm/>=1.0").is_err());
    assert!(Vers::parse("vers:npm").is_err());
    assert!(Vers::parse("vers:nosuchthing/>=1.0").is_err());
}

// ====================================================================
// OSV ecosystem names
// ====================================================================

fn osv(name: &str) -> Ecosystem {
    osv_ecosystem(name).unwrap_or_else(|error| panic!("{name}: {error}"))
}

#[test]
fn test_osv_ecosystem_pypi() {
    assert_eq!(osv("PyPI"), Ecosystem::Pep440);
}

#[test]
fn test_osv_ecosystem_strips_release_suffix() {
    assert_eq!(osv("Debian:11"), Ecosystem::Debian);
    assert_eq!(osv("Alpine:v3.18"), Ecosystem::Alpine);
    assert_eq!(osv("Ubuntu:22.04:LTS"), Ecosystem::Debian);
    assert_eq!(osv("Rocky Linux:8"), Ecosystem::Rpm);
}

#[test]
fn test_osv_ecosystem_known_names() {
    assert_eq!(osv("npm"), Ecosystem::Npm);
    assert_eq!(osv("crates.io"), Ecosystem::Crates);
    assert_eq!(osv("RubyGems"), Ecosystem::Ruby);
    assert_eq!(osv("Packagist"), Ecosystem::Composer);
    assert_eq!(osv("Wolfi"), Ecosystem::Alpine);
    assert_eq!(osv("Bioconductor"), Ecosystem::Cran);
    assert_eq!(osv("Haskell Hackage"), Ecosystem::Hackage);
}

#[test]
fn test_osv_go_orders_as_lenient_semver() {
    // OSV writes Go module versions without the mandatory `v` prefix.
    assert_eq!(osv("Go"), Ecosystem::Semver);
    let affected = [OsvEvent::Introduced("1.2.0".into()), OsvEvent::Fixed("1.2.4".into())];
    let windows = osv_range_windows(osv("Go"), &affected).unwrap();
    let windows = RangeSet { branches: windows };
    assert!(windows.contains_bounds(Ecosystem::Semver, "v1.2.2").unwrap());
    assert!(windows.contains_bounds(Ecosystem::Semver, "1.2.2").unwrap());
    assert!(!windows.contains_bounds(Ecosystem::Semver, "1.2.4").unwrap());
}

#[test]
fn test_osv_ecosystem_unknown_is_an_error() {
    // An unmapped ecosystem must not silently fall back to a heuristic
    // comparator; the caller opts into `generic` explicitly instead.
    for name in ["GitHub Actions", "OSS-Fuzz", "Android"] {
        let error = osv_ecosystem(name).unwrap_err();
        assert!(error.contains("unsupported OSV ecosystem"), "{name}: {error}");
        assert!(error.contains("ecosystem='generic'"), "{name}: {error}");
    }
}

// ====================================================================
// OSV event timelines
// ====================================================================

fn affected(ecosystem: Ecosystem, events: &[OsvEvent], version: &str) -> bool {
    let windows = osv_range_windows(ecosystem, events).unwrap();
    RangeSet { branches: windows }.contains_bounds(ecosystem, version).unwrap()
}

fn introduced(version: &str) -> OsvEvent {
    OsvEvent::Introduced(version.into())
}
fn fixed(version: &str) -> OsvEvent {
    OsvEvent::Fixed(version.into())
}
fn limit(version: &str) -> OsvEvent {
    OsvEvent::Limit(version.into())
}

#[test]
fn test_osv_limit_gates_the_whole_range() {
    // A version is affected only when it sorts before at least one listed
    // limit, whatever window it falls in. A later `introduced` does not make
    // versions above the limit affected again.
    let events = [introduced("0"), limit("5.2.0"), introduced("6.0.0"), fixed("6.0.3")];
    let npm = Ecosystem::Npm;
    assert!(affected(npm, &events, "4.0.0"));
    assert!(!affected(npm, &events, "5.2.0"));
    assert!(!affected(npm, &events, "5.9.9"));
    assert!(!affected(npm, &events, "6.0.0"));
    assert!(!affected(npm, &events, "6.0.1"));
    assert!(!affected(npm, &events, "6.0.3"));
}

#[test]
fn test_osv_limit_gates_every_window_wherever_it_is_listed() {
    let npm = Ecosystem::Npm;
    let leading = [limit("0.5.0"), introduced("1.0.0"), fixed("2.0.0")];
    assert!(!affected(npm, &leading, "1.5.0"));
    assert!(!affected(npm, &leading, "0.2.0"));
    let trailing = [introduced("0"), fixed("1.0.0"), introduced("2.0.0"), limit("0.5.0")];
    assert!(affected(npm, &trailing, "0.2.0"));
    assert!(!affected(npm, &trailing, "0.7.0"));
    assert!(!affected(npm, &trailing, "2.5.0"));
}

#[test]
fn test_osv_limit_gate_does_not_replace_the_window() {
    // Sorting before the limit is necessary, not sufficient: the version must
    // also fall in a window.
    let events = [introduced("2.0.0"), fixed("3.0.0"), limit("5.0.0")];
    let npm = Ecosystem::Npm;
    assert!(!affected(npm, &events, "1.0.0"));
    assert!(affected(npm, &events, "2.5.0"));
    assert!(!affected(npm, &events, "3.0.0"));
}

#[test]
fn test_osv_version_needs_to_precede_a_single_limit() {
    let events = [introduced("2.0.0"), limit("3.0.0"), limit("2.1.0")];
    let npm = Ecosystem::Npm;
    assert!(affected(npm, &events, "2.9.9"));
    assert!(!affected(npm, &events, "3.0.0"));
}

#[test]
fn test_osv_star_limit_lifts_every_limit() {
    let events = [limit("*"), introduced("0"), fixed("1.0.0"), introduced("2.0.0"), limit("2.1.0")];
    let npm = Ecosystem::Npm;
    assert!(affected(npm, &events, "0.9.0"));
    assert!(affected(npm, &events, "2.9.9"));
    assert!(!affected(npm, &events, "1.5.0"));
}

#[test]
fn test_osv_invalid_limit_is_an_error() {
    let events = [introduced("0"), limit("not-a-version")];
    assert!(osv_range_windows(Ecosystem::Npm, &events).is_err());
}

#[test]
fn test_osv_events_are_sorted_before_evaluation() {
    // A `fixed` listed before its `introduced` still reads as one window.
    let events = [fixed("1.2.0"), introduced("1.0.0")];
    let npm = Ecosystem::Npm;
    assert!(!affected(npm, &events, "0.5.0"));
    assert!(affected(npm, &events, "1.1.0"));
    assert!(!affected(npm, &events, "2.0.0"));
}

#[test]
fn test_osv_equal_versions_keep_input_order() {
    // Branch boundaries that share a version stay closed-then-opened, and a
    // window that opens and closes on one version stays empty.
    let npm = Ecosystem::Npm;
    let branches = [introduced("0"), fixed("1.0.0"), introduced("1.0.0"), fixed("2.0.0")];
    assert!(affected(npm, &branches, "0.9.0"));
    assert!(affected(npm, &branches, "1.0.0"));
    assert!(!affected(npm, &branches, "2.0.0"));
    let empty = [introduced("1.0.0"), fixed("1.0.0")];
    assert!(!affected(npm, &empty, "1.0.0"));
    assert!(!affected(npm, &empty, "9.0.0"));
}

#[test]
fn test_osv_maven_timeline_does_not_panic() {
    // Maven's comparator has a cycle; sorting must tolerate it.
    let events =
        [introduced("1-foo.sp-0"), introduced("1-foo-alpha"), fixed("1-foo"), introduced("1")];
    assert!(osv_range_windows(Ecosystem::Maven, &events).is_ok());
}

#[test]
fn test_osv_invalid_event_version_is_an_error() {
    let events = [introduced("not-a-version")];
    assert!(osv_range_windows(Ecosystem::Npm, &events).is_err());
}

// ====================================================================
// PURL
// ====================================================================

#[test]
fn test_purl_components_cannot_change_the_structure() {
    let purl = Purl {
        package_type: "npm".into(),
        namespace: Some("a@b/c?d".into()),
        name: "foo/bar@1.0.0".into(),
        version: Some("1.0.0+build/x".into()),
        qualifiers: Some("a=b#c".into()),
        subpath: Some("dir with space/file".into()),
    };
    assert_eq!(
        purl.to_purl(),
        "pkg:npm/a%40b/c%3Fd/foo%2Fbar%401.0.0@1.0.0+build%2Fx?a=b%23c#dir%20with%20space/file"
    );
    let parsed = Purl::parse(&purl.to_purl()).unwrap();
    assert_eq!(parsed.name, "foo%2Fbar%401.0.0");
    assert_eq!(parsed.version.as_deref(), Some("1.0.0+build%2Fx"));
    assert_eq!(parsed.qualifiers.as_deref(), Some("a=b%23c"));
}

#[test]
fn test_purl_subpath_encodes_the_fragment_separator() {
    let purl = Purl {
        package_type: "npm".into(),
        namespace: None,
        name: "name".into(),
        version: None,
        qualifiers: None,
        subpath: Some("docs/index#section".into()),
    };
    assert_eq!(purl.to_purl(), "pkg:npm/name#docs/index%23section");
    let parsed = Purl::parse(&purl.to_purl()).unwrap();
    assert_eq!(parsed.subpath.as_deref(), Some("docs/index%23section"));
}

#[test]
fn test_purl_keeps_version_and_name_characters_literal() {
    let purl = Purl {
        package_type: "deb".into(),
        namespace: Some("debian".into()),
        name: "libstdc++6".into(),
        version: Some("1:12.2.0-3+build1".into()),
        qualifiers: Some("arch=i386&distro=jessie".into()),
        subpath: None,
    };
    assert_eq!(
        purl.to_purl(),
        "pkg:deb/debian/libstdc++6@1:12.2.0-3+build1?arch=i386&distro=jessie"
    );
}

#[test]
fn test_purl_type_must_be_a_type() {
    assert!(validate_purl_type("npm").is_ok());
    assert!(validate_purl_type("github").is_ok());
    assert!(validate_purl_type("oci").is_ok());
    for invalid in ["", "npm/x", "npm?y", "npm#z", "1npm", "np m", "npm@x"] {
        assert!(validate_purl_type(invalid).is_err(), "{invalid:?}");
    }
}

// ====================================================================
// VERS scheme names
// ====================================================================

#[test]
fn test_vers_alpine_uses_the_purl_type() {
    // Alpine's registered PURL type is `apk`; `alpine` is an accepted alias.
    let vers = Vers::new(Ecosystem::Alpine, ">=1.0.0-r1").unwrap();
    assert_eq!(vers.to_vers(), "vers:apk/>=1.0.0-r1");
    assert!(Vers::parse("vers:apk/>=1.0.0-r1").is_ok());
    assert!(Vers::parse("vers:alpine/>=1.0.0-r1").is_ok());
}
