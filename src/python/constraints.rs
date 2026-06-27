//! Version constraint parsing and evaluation: bare operators (`>=1.0,<2.0`),
//! OR groups (`^1.0 || ^2.0`), and npm/cargo-style shorthand ranges
//! (`^1.2.3`, `~1.2`, `1.2.x`, `*`), plus the `satisfies` entry point.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::cmp::Ordering;

use super::compare::compare_str_with_ecosystem;

pub(crate) fn parse_constraint(spec: &str) -> Result<(&str, &str), String> {
    let s = spec.trim();
    if let Some(v) = s.strip_prefix(">=") {
        Ok((">=", v.trim()))
    } else if let Some(v) = s.strip_prefix("<=") {
        Ok(("<=", v.trim()))
    } else if let Some(v) = s.strip_prefix("!=") {
        Ok(("!=", v.trim()))
    } else if let Some(v) = s.strip_prefix("==") {
        Ok(("==", v.trim()))
    } else if let Some(v) = s.strip_prefix('>') {
        Ok((">", v.trim()))
    } else if let Some(v) = s.strip_prefix('<') {
        Ok(("<", v.trim()))
    } else {
        Err(format!(
            "invalid constraint: '{s}'; expected operator (>=, <=, >, <, ==, !=) followed by version"
        ))
    }
}

/// Parse 1-3 numeric parts from a version prefix, treating `x`, `X`, and `*`
/// as wildcards. Returns `Some((parts, wildcard_depth))` where
/// `wildcard_depth` is 3 if no wildcard, else the index at which it appears.
fn parse_num_prefix(input: &str) -> Option<(Vec<u64>, usize)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(3);
    let mut wildcard_at: Option<usize> = None;
    for (i, raw) in s.split('.').enumerate() {
        if i >= 3 {
            break;
        }
        if raw == "x" || raw == "X" || raw == "*" {
            wildcard_at = Some(i);
            break;
        }
        let n: u64 = raw.parse().ok()?;
        parts.push(n);
    }
    if parts.is_empty() && wildcard_at != Some(0) {
        return None;
    }
    Some((parts, wildcard_at.unwrap_or(3)))
}

fn fmt_triple(major: u64, minor: u64, patch: u64) -> String {
    format!("{major}.{minor}.{patch}")
}

/// Expand a shorthand range token (`^1.2.3`, `~1.2`, `1.2.x`, `*`) into a list
/// of basic constraints. Bare constraints (`>=1.0.0`, `==1.0.0`) pass through.
fn expand_shorthand(token: &str) -> Result<Vec<(String, String)>, String> {
    let t = token.trim();
    if t.is_empty() {
        return Err("empty constraint".to_string());
    }
    if t == "*" || t == "x" || t == "X" {
        return Ok(vec![(">=".to_string(), "0.0.0".to_string())]);
    }
    // Caret: ^X.Y.Z — allow changes that don't modify the left-most non-zero.
    if let Some(rest) = t.strip_prefix('^') {
        let (parts, _) =
            parse_num_prefix(rest).ok_or_else(|| format!("invalid caret range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let patch = *parts.get(2).unwrap_or(&0);
        let lower = fmt_triple(major, minor, patch);
        let upper = if major > 0 {
            fmt_triple(major + 1, 0, 0)
        } else if minor > 0 {
            fmt_triple(0, minor + 1, 0)
        } else {
            fmt_triple(0, 0, patch + 1)
        };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // Tilde: ~X.Y.Z — allow patch-level changes. ~X.Y == ~X.Y.0 == >=X.Y.0,<X.(Y+1).0.
    if let Some(rest) = t.strip_prefix('~') {
        let (parts, _) =
            parse_num_prefix(rest).ok_or_else(|| format!("invalid tilde range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let patch = *parts.get(2).unwrap_or(&0);
        let lower = fmt_triple(major, minor, patch);
        let upper = if parts.len() >= 2 {
            fmt_triple(major, minor + 1, 0)
        } else {
            fmt_triple(major + 1, 0, 0)
        };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // x-range: 1.2.x, 1.x, 1.2.*
    if t.contains(".x") || t.contains(".X") || t.contains(".*") {
        let (parts, depth) =
            parse_num_prefix(t).ok_or_else(|| format!("invalid x-range: '{t}'"))?;
        let major = *parts.first().unwrap_or(&0);
        let minor = *parts.get(1).unwrap_or(&0);
        let lower = fmt_triple(major, minor, 0);
        let upper =
            if depth == 1 { fmt_triple(major + 1, 0, 0) } else { fmt_triple(major, minor + 1, 0) };
        return Ok(vec![(">=".to_string(), lower), ("<".to_string(), upper)]);
    }
    // Bare constraint (>=, <=, ==, !=, >, <).
    let (op, v) = parse_constraint(t)?;
    Ok(vec![(op.to_string(), v.to_string())])
}

fn eval_op(op: &str, ord: Ordering) -> bool {
    match op {
        ">=" => ord != Ordering::Less,
        "<=" => ord != Ordering::Greater,
        ">" => ord == Ordering::Greater,
        "<" => ord == Ordering::Less,
        "==" => ord == Ordering::Equal,
        "!=" => ord != Ordering::Equal,
        _ => false,
    }
}

/// Check whether `version` satisfies `constraint`.
///
/// Supported syntax:
///   - Operators: `>=`, `<=`, `>`, `<`, `==`, `!=`
///   - AND via comma: `>=1.0,<2.0`
///   - OR via `||`: `^1.0 || ^2.0`
///   - Shorthand: `^1.2.3`, `~1.2`, `1.2.x`, `*`
#[pyfunction(signature = (version, constraint, ecosystem = "generic"))]
pub(crate) fn satisfies(version: &str, constraint: &str, ecosystem: &str) -> PyResult<bool> {
    // Evaluate OR branches left-to-right.
    for branch in constraint.split("||") {
        let mut branch_ok = true;
        let mut branch_has_clause = false;
        for part in branch.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            branch_has_clause = true;
            let clauses = expand_shorthand(part).map_err(PyValueError::new_err)?;
            for (op, cv) in clauses {
                let ord = compare_str_with_ecosystem(version, &cv, ecosystem)?;
                if !eval_op(&op, ord) {
                    branch_ok = false;
                    break;
                }
            }
            if !branch_ok {
                break;
            }
        }
        if branch_has_clause && branch_ok {
            return Ok(true);
        }
    }
    Ok(false)
}
