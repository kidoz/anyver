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

#[test]
fn test_wide_components_keep_their_order() {
    // apk and the generic comparator order digit runs exactly, so a component
    // wider than u64 must not wrap around or saturate onto another number.
    let wide = u128::from(u64::MAX) + 1;
    for eco in ["alpine", "generic", "homebrew", "conda"] {
        let compare = |a: &String, b: &str| compare_str_with_ecosystem(a, b, eco).unwrap();
        assert_eq!(compare(&format!("1.{wide}"), "1.0"), Greater, "{eco}: 2^64 must not wrap to 0");
        assert_eq!(compare(&format!("1.{}", wide + 5), "1.5"), Greater, "{eco}: must not wrap");
        assert_eq!(compare(&format!("1.{}", wide + 1), &format!("1.{wide}")), Greater, "{eco}");
    }
    // apk compares a zero-prefixed component lexically, so leading zeros only
    // vanish in the generic modes.
    for eco in ["generic", "homebrew", "conda"] {
        let compare = |a: &str| compare_str_with_ecosystem(a, &format!("1.0{wide}"), eco).unwrap();
        assert_eq!(compare(&format!("1.{wide}")), Equal, "{eco}: leading zeros");
    }
}
