use super::*;

// ====================================================================
// parse_constraint — parser for satisfies()
// ====================================================================

#[test]
fn test_parse_constraint_ge() {
    assert_eq!(parse_constraint(">=1.0.0").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_le() {
    assert_eq!(parse_constraint("<=2.0.0").unwrap(), ("<=", "2.0.0"));
}
#[test]
fn test_parse_constraint_gt() {
    assert_eq!(parse_constraint(">1.0").unwrap(), (">", "1.0"));
}
#[test]
fn test_parse_constraint_lt() {
    assert_eq!(parse_constraint("<2.0").unwrap(), ("<", "2.0"));
}
#[test]
fn test_parse_constraint_eq() {
    assert_eq!(parse_constraint("==1.0").unwrap(), ("==", "1.0"));
}
#[test]
fn test_parse_constraint_ne() {
    assert_eq!(parse_constraint("!=1.0").unwrap(), ("!=", "1.0"));
}

// Whitespace handling: both around op and between op/version
#[test]
fn test_parse_constraint_space_around() {
    assert_eq!(parse_constraint("  >=1.0.0  ").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_space_between() {
    assert_eq!(parse_constraint(">= 1.0.0").unwrap(), (">=", "1.0.0"));
}
#[test]
fn test_parse_constraint_space_both() {
    assert_eq!(parse_constraint(" < 2.0 ").unwrap(), ("<", "2.0"));
}

// Operator precedence: >= must bind before > (same for <= before <, == before (not relevant), != before (not relevant))
#[test]
fn test_parse_constraint_ge_not_gt() {
    let (op, _) = parse_constraint(">=1.0").unwrap();
    assert_eq!(op, ">=");
}
#[test]
fn test_parse_constraint_le_not_lt() {
    let (op, _) = parse_constraint("<=1.0").unwrap();
    assert_eq!(op, "<=");
}

// Errors: no operator, empty
#[test]
fn test_parse_constraint_no_operator() {
    assert!(parse_constraint("1.0.0").is_err());
}
#[test]
fn test_parse_constraint_empty() {
    assert!(parse_constraint("").is_err());
}
#[test]
fn test_parse_constraint_error_message() {
    let err = parse_constraint("1.0.0").unwrap_err();
    assert!(err.contains("invalid constraint"));
}
