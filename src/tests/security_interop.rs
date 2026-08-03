//! Regression tests for the VERS grammar.

use crate::parser::Ecosystem;
use crate::security::Vers;

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
