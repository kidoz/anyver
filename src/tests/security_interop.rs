//! Regression tests for the VERS grammar and the OSV ecosystem-name mapping.

use crate::parser::Ecosystem;
use crate::security::{Purl, Vers, osv_ecosystem, validate_purl_type};

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

#[test]
fn test_osv_ecosystem_pypi() {
    assert_eq!(osv_ecosystem("PyPI"), Ecosystem::Pep440);
}

#[test]
fn test_osv_ecosystem_strips_release_suffix() {
    assert_eq!(osv_ecosystem("Debian:11"), Ecosystem::Debian);
    assert_eq!(osv_ecosystem("Alpine:v3.18"), Ecosystem::Alpine);
    assert_eq!(osv_ecosystem("Ubuntu:22.04:LTS"), Ecosystem::Debian);
    assert_eq!(osv_ecosystem("Rocky Linux:8"), Ecosystem::Rpm);
}

#[test]
fn test_osv_ecosystem_known_names() {
    assert_eq!(osv_ecosystem("npm"), Ecosystem::Npm);
    assert_eq!(osv_ecosystem("crates.io"), Ecosystem::Crates);
    assert_eq!(osv_ecosystem("RubyGems"), Ecosystem::Ruby);
    assert_eq!(osv_ecosystem("Packagist"), Ecosystem::Composer);
    assert_eq!(osv_ecosystem("Wolfi"), Ecosystem::Alpine);
    assert_eq!(osv_ecosystem("Bioconductor"), Ecosystem::Cran);
}

#[test]
fn test_osv_ecosystem_unknown_falls_back_to_generic() {
    // An unmapped ecosystem must not fail the whole record.
    assert_eq!(osv_ecosystem("GitHub Actions"), Ecosystem::Generic);
    assert_eq!(osv_ecosystem("OSS-Fuzz"), Ecosystem::Generic);
    assert_eq!(osv_ecosystem("Android"), Ecosystem::Generic);
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
