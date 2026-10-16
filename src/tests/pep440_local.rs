use super::*;

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

#[test]
fn test_pep440_public_normalization_and_local_boundaries() {
    for (a, b, expected) in [
        ("1.0rc1", "1.0.0rc1", Equal),
        ("1.0-1", "1.0.post1", Equal),
        ("1.0a", "1.0alpha0", Equal),
        ("1.0+abc1", "1.0+abc.1", Greater),
        ("1.0+abc.0", "1.0+abc", Greater),
        ("1.0a1.dev1", "1.0a1", Less),
        ("1.0.post1.dev1", "1.0.post1", Less),
    ] {
        assert_eq!(pep440_cmp(a, b), expected, "{a} versus {b}");
    }
}

#[test]
fn test_pep440_full_grammar_validation() {
    for input in ["1..0", "1.0+", "1!2!3", "1..a1", "1.0a..1", "1.0+abc..1"] {
        assert!(parse_for_ecosystem(Ecosystem::Pep440, input).is_err(), "{input}");
    }
}
