use super::*;

#[test]
fn test_go_requires_lowercase_v_prefix() {
    assert!(compare_str_with_ecosystem("1.0.0", "1.0.1", "go").is_err());
    assert!(compare_str_with_ecosystem("V1.0.0", "V1.0.1", "go").is_err());
    assert_eq!(compare_str_with_ecosystem("v1.0.0", "v1.0.1", "go").unwrap(), Less);
}

#[test]
fn test_new_ecosystem_aliases_parse() {
    for name in [
        "conda",
        "arch",
        "gentoo",
        "cran",
        "spack",
        "dart",
        "julia",
        "terraform",
        "helm",
        "cpan",
        "nix",
        "homebrew",
        "freebsd",
    ] {
        assert!(Ecosystem::from_str(name).is_ok(), "{name}");
    }
}

#[test]
fn test_new_generic_ecosystem_smoke_ordering() {
    assert_eq!(compare_str_with_ecosystem("1.0", "1.1", "conda").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1:1.0", "2:1.0", "arch").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0_alpha", "1.0", "gentoo").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0-1", "1.0-2", "cran").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0", "2.0", "spack").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.1.0", "terraform").unwrap(), Less);
}

#[test]
fn test_new_semver_ecosystem_smoke_ordering() {
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "dart").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "julia").unwrap(), Less);
    assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "helm").unwrap(), Less);
}
