use super::*;

// ====================================================================
// MAVEN RELEASE-ALIAS QUALIFIERS
// ====================================================================

#[test]
fn test_tag_weight_release_aliases() {
    assert_eq!(tag_weight("final"), Some(30));
    assert_eq!(tag_weight("ga"), Some(30));
    assert_eq!(tag_weight("release"), Some(30));
}

#[test]
fn test_normalized_strips_trailing_release_alias() {
    let segs = vec![Seg::Num(1), Seg::Num(0), Seg::Text("final".to_string())];
    assert_eq!(normalized(&segs), &[Seg::Num(1)]);
}

fn gen_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let pa = parse_for_ecosystem(Ecosystem::Generic, a).unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Generic, b).unwrap();
    compare_for_ecosystem(Ecosystem::Generic, &pa, &pb)
}

#[test]
fn test_generic_final_equals_release() {
    assert_eq!(gen_cmp("1.0-final", "1.0"), std::cmp::Ordering::Equal);
}
#[test]
fn test_generic_ga_equals_release() {
    assert_eq!(gen_cmp("1.0-ga", "1.0"), std::cmp::Ordering::Equal);
}
#[test]
fn test_maven_final_below_sp() {
    let pa = parse_for_ecosystem(Ecosystem::Maven, "1.0-final").unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Maven, "1.0-sp-1").unwrap();
    assert_eq!(compare_for_ecosystem(Ecosystem::Maven, &pa, &pb), std::cmp::Ordering::Less);
}
#[test]
fn test_maven_snapshot_below_final() {
    let pa = parse_for_ecosystem(Ecosystem::Maven, "1.0-SNAPSHOT").unwrap();
    let pb = parse_for_ecosystem(Ecosystem::Maven, "1.0-final").unwrap();
    assert_eq!(compare_for_ecosystem(Ecosystem::Maven, &pa, &pb), std::cmp::Ordering::Less);
}
