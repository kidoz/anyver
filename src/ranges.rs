//! Ecosystem-aware version range parsing and set operations.
//!
//! This module keeps range semantics in pure Rust. The Python layer only
//! translates arguments and errors.

use crate::parser::Ecosystem;
use crate::strategies::{compare_for_ecosystem, parse_for_ecosystem};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangeOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Comparator {
    pub(crate) op: RangeOp,
    pub(crate) version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RangeSet {
    pub(crate) branches: Vec<Vec<Comparator>>,
}

#[derive(Debug, Clone)]
struct Interval {
    lower: Option<(String, bool)>,
    upper: Option<(String, bool)>,
    equals: Vec<String>,
    not_equals: Vec<String>,
}

fn cmp_versions(ecosystem: Ecosystem, a: &str, b: &str) -> Result<Ordering, String> {
    let pa = parse_for_ecosystem(ecosystem, a)?;
    let pb = parse_for_ecosystem(ecosystem, b)?;
    Ok(compare_for_ecosystem(ecosystem, &pa, &pb))
}

fn increment(n: u64) -> Result<u64, String> {
    n.checked_add(1).ok_or_else(|| "range upper bound exceeds u64".to_string())
}

fn lower_bound(rest: &str, major: u64, minor: u64, patch: u64) -> String {
    let suffix = rest.find(['-', '+']).map_or("", |i| &rest[i..]);
    format!("{major}.{minor}.{patch}{suffix}")
}

fn fmt_triple(major: u64, minor: u64, patch: u64) -> String {
    format!("{major}.{minor}.{patch}")
}

fn parse_num_prefix(input: &str) -> Option<(Vec<u64>, usize)> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(3);
    let mut wildcard_at: Option<usize> = None;
    for (i, raw) in s.split(['-', '+']).next()?.split('.').enumerate() {
        if i >= 3 {
            return None;
        }
        if raw == "x" || raw == "X" || raw == "*" {
            wildcard_at = Some(i);
            break;
        }
        let numeric = raw;
        if numeric.is_empty() {
            return None;
        }
        let n: u64 = numeric.parse().ok()?;
        parts.push(n);
    }
    if parts.is_empty() && wildcard_at != Some(0) {
        return None;
    }
    Some((parts, wildcard_at.unwrap_or(3)))
}

/// Split a leading comparison operator off a token, if it has one.
fn split_operator(t: &str) -> Option<(RangeOp, &str)> {
    // Two-character operators must be tested before their one-character
    // prefixes so `>=` never binds as `>`.
    const OPS: [(&str, RangeOp); 6] = [
        (">=", RangeOp::Ge),
        ("<=", RangeOp::Le),
        ("!=", RangeOp::Ne),
        ("==", RangeOp::Eq),
        (">", RangeOp::Gt),
        ("<", RangeOp::Lt),
    ];
    OPS.iter().find_map(|(prefix, op)| t.strip_prefix(prefix).map(|rest| (*op, rest.trim())))
}

fn parse_comparator(token: &str) -> Result<Comparator, String> {
    let t = token.trim();
    let Some((op, version)) = split_operator(t) else {
        return Err(format!(
            "invalid comparator: '{t}'; expected operator (>=, <=, >, <, ==, !=) followed by version"
        ));
    };
    if version.is_empty() {
        return Err(format!("missing version in comparator '{token}'"));
    }
    Ok(Comparator { op, version: version.to_string() })
}

fn has_wildcard(token: &str) -> bool {
    token.contains(".x") || token.contains(".X") || token.contains(".*")
}

/// Resolve an operator applied to a wildcarded version (`>=1.2.x`, `<=2.x`).
/// The bound has to land on the *edge* of the wildcarded window, which for the
/// exclusive operators is not the wildcard's own prefix: `>1.2.x` means "above
/// everything in 1.2", i.e. `>=1.3.0`, and `<=1.2.x` means `<1.3.0`. This is
/// npm's reading of the same syntax.
fn expand_operator_wildcard(op: RangeOp, rest: &str) -> Result<Vec<Comparator>, String> {
    let (parts, depth) =
        parse_num_prefix(rest).ok_or_else(|| format!("invalid wildcard range: '{rest}'"))?;
    if depth == 0 {
        // A bare `>=x` constrains nothing.
        return Ok(Vec::new());
    }
    let major = *parts.first().unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let lower = fmt_triple(major, minor, 0);
    let upper = if depth == 1 {
        fmt_triple(increment(major)?, 0, 0)
    } else {
        fmt_triple(major, increment(minor)?, 0)
    };
    Ok(match op {
        RangeOp::Ge | RangeOp::Lt => vec![Comparator { op, version: lower }],
        RangeOp::Gt => vec![Comparator { op: RangeOp::Ge, version: upper }],
        RangeOp::Le => vec![Comparator { op: RangeOp::Lt, version: upper }],
        RangeOp::Eq => vec![
            Comparator { op: RangeOp::Ge, version: lower },
            Comparator { op: RangeOp::Lt, version: upper },
        ],
        RangeOp::Ne => {
            return Err(format!("'!=' does not support wildcard versions: '!={rest}'"));
        }
    })
}

fn expand_caret(rest: &str) -> Result<Vec<Comparator>, String> {
    let (parts, _) =
        parse_num_prefix(rest).ok_or_else(|| format!("invalid caret range: '^{rest}'"))?;
    let major = *parts.first().unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let patch = *parts.get(2).unwrap_or(&0);
    let upper = if major > 0 || parts.len() == 1 {
        fmt_triple(increment(major)?, 0, 0)
    } else if minor > 0 || parts.len() == 2 {
        fmt_triple(0, increment(minor)?, 0)
    } else {
        fmt_triple(0, 0, increment(patch)?)
    };
    Ok(vec![
        Comparator { op: RangeOp::Ge, version: lower_bound(rest, major, minor, patch) },
        Comparator { op: RangeOp::Lt, version: upper },
    ])
}

fn expand_tilde(rest: &str) -> Result<Vec<Comparator>, String> {
    let (parts, _) =
        parse_num_prefix(rest).ok_or_else(|| format!("invalid tilde range: '~{rest}'"))?;
    let major = *parts.first().unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let patch = *parts.get(2).unwrap_or(&0);
    let upper = if parts.len() >= 2 {
        fmt_triple(major, increment(minor)?, 0)
    } else {
        fmt_triple(increment(major)?, 0, 0)
    };
    Ok(vec![
        Comparator { op: RangeOp::Ge, version: lower_bound(rest, major, minor, patch) },
        Comparator { op: RangeOp::Lt, version: upper },
    ])
}

fn expand_x_range(token: &str) -> Result<Vec<Comparator>, String> {
    let (parts, depth) =
        parse_num_prefix(token).ok_or_else(|| format!("invalid x-range: '{token}'"))?;
    if depth == 0 {
        // Leading wildcard (`x.1`) — nothing is constrained.
        return Ok(Vec::new());
    }
    let major = *parts.first().unwrap_or(&0);
    let minor = *parts.get(1).unwrap_or(&0);
    let lower = fmt_triple(major, minor, 0);
    let upper = if depth == 1 {
        fmt_triple(increment(major)?, 0, 0)
    } else {
        fmt_triple(major, increment(minor)?, 0)
    };
    Ok(vec![
        Comparator { op: RangeOp::Ge, version: lower },
        Comparator { op: RangeOp::Lt, version: upper },
    ])
}

fn expand_interval(token: &str) -> Result<Vec<Comparator>, String> {
    let t = token.trim();
    let bytes = t.as_bytes();
    if bytes.len() < 2 {
        return Err(format!("invalid interval range: '{token}'"));
    }
    let left_inclusive = match bytes[0] {
        b'[' => true,
        b'(' => false,
        _ => return Err(format!("invalid interval range: '{token}'")),
    };
    let right_inclusive = match bytes[bytes.len() - 1] {
        b']' => true,
        b')' => false,
        _ => return Err(format!("invalid interval range: '{token}'")),
    };
    let body = &t[1..t.len() - 1];
    let parts: Vec<&str> = body.split(',').collect();
    if parts.len() != 2 {
        return Err(format!("invalid interval range: '{token}'"));
    }
    let mut out = Vec::with_capacity(2);
    let lower = parts[0].trim();
    if !lower.is_empty() {
        out.push(Comparator {
            op: if left_inclusive { RangeOp::Ge } else { RangeOp::Gt },
            version: lower.to_string(),
        });
    }
    let upper = parts[1].trim();
    if !upper.is_empty() {
        out.push(Comparator {
            op: if right_inclusive { RangeOp::Le } else { RangeOp::Lt },
            version: upper.to_string(),
        });
    }
    Ok(out)
}

fn expand_token(token: &str) -> Result<Vec<Comparator>, String> {
    let t = token.trim();
    if t.is_empty() {
        return Ok(Vec::new());
    }
    if t == "*" || t == "x" || t == "X" {
        return Ok(Vec::new());
    }
    if (t.starts_with('[') || t.starts_with('(')) && (t.ends_with(']') || t.ends_with(')')) {
        return expand_interval(t);
    }
    if let Some(rest) = t.strip_prefix("^") {
        return expand_caret(rest);
    }
    if let Some(rest) = t.strip_prefix("~>") {
        let (parts, _) = parse_num_prefix(rest).ok_or("invalid pessimistic range")?;
        if parts.len() < 3 {
            let major = *parts.first().ok_or("missing pessimistic version")?;
            let minor = *parts.get(1).unwrap_or(&0);
            return Ok(vec![
                Comparator { op: RangeOp::Ge, version: lower_bound(rest, major, minor, 0) },
                Comparator { op: RangeOp::Lt, version: fmt_triple(increment(major)?, 0, 0) },
            ]);
        }
        return expand_tilde(rest);
    }
    if let Some(rest) = t.strip_prefix("~=") {
        return expand_tilde(rest);
    }
    if let Some(rest) = t.strip_prefix('~') {
        return expand_tilde(rest);
    }
    // An operator has to be stripped *before* the wildcard test, or `>=1.2.x`
    // reaches `expand_x_range` with the operator still glued to the version and
    // fails to parse.
    if let Some((op, rest)) = split_operator(t)
        && has_wildcard(rest)
    {
        return expand_operator_wildcard(op, rest);
    }
    if has_wildcard(t) {
        return expand_x_range(t);
    }
    Ok(vec![parse_comparator(t)?])
}

fn split_branch(branch: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut interval_depth = 0usize;
    for ch in branch.chars() {
        match ch {
            '[' | '(' => {
                interval_depth += 1;
                buf.push(ch);
            }
            ']' | ')' => {
                interval_depth = interval_depth.saturating_sub(1);
                buf.push(ch);
            }
            ',' if interval_depth == 0 => {
                if !buf.trim().is_empty() {
                    out.push(buf.trim().to_string());
                }
                buf.clear();
            }
            c if c.is_whitespace() && interval_depth == 0 => {
                if !buf.trim().is_empty() {
                    out.push(buf.trim().to_string());
                }
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() {
        out.push(buf.trim().to_string());
    }

    let mut merged = Vec::with_capacity(out.len());
    let mut i = 0usize;
    while i < out.len() {
        if matches!(out[i].as_str(), ">=" | "<=" | "!=" | "==" | ">" | "<") && i + 1 < out.len() {
            merged.push(format!("{}{}", out[i], out[i + 1]));
            i += 2;
        } else {
            merged.push(out[i].clone());
            i += 1;
        }
    }
    merged
}

impl RangeSet {
    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let mut branches = Vec::new();
        for branch in input.split("||") {
            let tokens = split_branch(branch);
            // A branch with no tokens is an *absent* constraint, not an
            // unconstrained one. Silently treating it as "matches everything"
            // turns a blank range into a universal match, which is the wrong
            // direction to fail in for vulnerability matching — `*` is the way
            // to say "any version".
            if tokens.is_empty() {
                return Err(format!("empty range: '{input}'; use '*' to match every version"));
            }
            let mut clauses = Vec::new();
            for token in tokens {
                clauses.extend(expand_token(&token)?);
            }
            branches.push(clauses);
        }
        Ok(Self { branches })
    }

    pub(crate) fn contains(&self, ecosystem: Ecosystem, version: &str) -> Result<bool, String> {
        let target = parse_for_ecosystem(ecosystem, version)?;
        for branch in &self.branches {
            if ecosystem == Ecosystem::Npm
                && let crate::strategies::ParsedRepr::Semver(target) = &target
                && target.pre.is_some()
            {
                let mut admitted = false;
                for clause in branch {
                    if clause.op != RangeOp::Ne
                        && let crate::strategies::ParsedRepr::Semver(bound) =
                            parse_for_ecosystem(ecosystem, &clause.version)?
                        && bound.pre.is_some()
                        && (target.major, target.minor, target.patch)
                            == (bound.major, bound.minor, bound.patch)
                    {
                        admitted = true;
                    }
                }
                if !admitted {
                    continue;
                }
            }
            if branch_contains(ecosystem, version, branch)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Raw interval containment, without package-manager prerelease admission.
    /// OSV event windows use version ordering directly, even for npm records.
    pub(crate) fn contains_bounds(
        &self,
        ecosystem: Ecosystem,
        version: &str,
    ) -> Result<bool, String> {
        for branch in &self.branches {
            if branch_contains(ecosystem, version, branch)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn intersects(&self, ecosystem: Ecosystem, other: &Self) -> Result<bool, String> {
        for left in &self.branches {
            for right in &other.branches {
                if branches_intersect(ecosystem, left, right)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

fn branch_contains(
    ecosystem: Ecosystem,
    version: &str,
    branch: &[Comparator],
) -> Result<bool, String> {
    for clause in branch {
        let ord = cmp_versions(ecosystem, version, &clause.version)?;
        let matched = match clause.op {
            RangeOp::Gt => ord == Ordering::Greater,
            RangeOp::Ge => ord != Ordering::Less,
            RangeOp::Lt => ord == Ordering::Less,
            RangeOp::Le => ord != Ordering::Greater,
            RangeOp::Eq => ord == Ordering::Equal,
            RangeOp::Ne => ord != Ordering::Equal,
        };
        if !matched {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Collapse a branch's clauses (which are `AND`ed) into one interval. Repeated
/// bounds have to be *folded* — keeping only the last would make the result
/// depend on clause order, so `>=2.0,>=1.0` would read as `>=1.0` and report a
/// spurious overlap with `<1.5`.
fn interval_for(ecosystem: Ecosystem, branch: &[Comparator]) -> Result<Interval, String> {
    let mut interval =
        Interval { lower: None, upper: None, equals: Vec::new(), not_equals: Vec::new() };
    for clause in branch {
        let bound = Some((clause.version.clone(), matches!(clause.op, RangeOp::Ge | RangeOp::Le)));
        match clause.op {
            RangeOp::Gt | RangeOp::Ge => {
                interval.lower = tighter_lower(ecosystem, interval.lower.take(), bound)?;
            }
            RangeOp::Lt | RangeOp::Le => {
                interval.upper = tighter_upper(ecosystem, interval.upper.take(), bound)?;
            }
            RangeOp::Eq => interval.equals.push(clause.version.clone()),
            RangeOp::Ne => interval.not_equals.push(clause.version.clone()),
        }
    }
    Ok(interval)
}

fn version_in_branch(
    ecosystem: Ecosystem,
    branch: &[Comparator],
    version: &str,
) -> Result<bool, String> {
    RangeSet { branches: vec![branch.to_vec()] }.contains(ecosystem, version)
}

fn branches_intersect(
    ecosystem: Ecosystem,
    left: &[Comparator],
    right: &[Comparator],
) -> Result<bool, String> {
    let li = interval_for(ecosystem, left)?;
    let ri = interval_for(ecosystem, right)?;
    for eq in &li.equals {
        if version_in_branch(ecosystem, right, eq)? && version_in_branch(ecosystem, left, eq)? {
            return Ok(true);
        }
    }
    for eq in &ri.equals {
        if version_in_branch(ecosystem, left, eq)? && version_in_branch(ecosystem, right, eq)? {
            return Ok(true);
        }
    }
    if !li.equals.is_empty() || !ri.equals.is_empty() {
        return Ok(false);
    }

    let lower = tighter_lower(ecosystem, li.lower, ri.lower)?;
    let upper = tighter_upper(ecosystem, li.upper, ri.upper)?;
    let (Some((lv, lower_inc)), Some((uv, upper_inc))) = (&lower, &upper) else {
        // Unbounded on at least one side, and neither side pins an exact
        // version — the overlap is infinite, so no `!=` can empty it.
        return Ok(true);
    };
    match cmp_versions(ecosystem, lv, uv)? {
        Ordering::Less => Ok(true),
        Ordering::Greater => Ok(false),
        Ordering::Equal => {
            if !(*lower_inc && *upper_inc) {
                return Ok(false);
            }
            // The overlap has collapsed to the single version `lv`; a `!=` on
            // either side excludes exactly that version and empties it.
            for excluded in li.not_equals.iter().chain(ri.not_equals.iter()) {
                if cmp_versions(ecosystem, lv, excluded)? == Ordering::Equal {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

/// The more restrictive of two lower bounds — the greater version, and the
/// exclusive one when they name the same version.
fn tighter_lower(
    ecosystem: Ecosystem,
    a: Option<(String, bool)>,
    b: Option<(String, bool)>,
) -> Result<Option<(String, bool)>, String> {
    match (a, b) {
        (None, None) => Ok(None),
        (Some(x), None) | (None, Some(x)) => Ok(Some(x)),
        (Some(x), Some(y)) => {
            let ord = cmp_versions(ecosystem, &x.0, &y.0)?;
            Ok(Some(match ord {
                Ordering::Less => y,
                Ordering::Greater => x,
                Ordering::Equal => (x.0, x.1 && y.1),
            }))
        }
    }
}

/// The more restrictive of two upper bounds — the lesser version, and the
/// exclusive one when they name the same version.
fn tighter_upper(
    ecosystem: Ecosystem,
    a: Option<(String, bool)>,
    b: Option<(String, bool)>,
) -> Result<Option<(String, bool)>, String> {
    match (a, b) {
        (None, None) => Ok(None),
        (Some(x), None) | (None, Some(x)) => Ok(Some(x)),
        (Some(x), Some(y)) => {
            let ord = cmp_versions(ecosystem, &x.0, &y.0)?;
            Ok(Some(match ord {
                Ordering::Less => x,
                Ordering::Greater => y,
                Ordering::Equal => (x.0, x.1 && y.1),
            }))
        }
    }
}
