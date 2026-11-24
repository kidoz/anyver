//! Ecosystem-aware version range parsing and set operations.
//!
//! This module keeps range semantics in pure Rust. The Python layer only
//! translates arguments and errors.

use crate::parser::{Ecosystem, strip_v};
use crate::strategies::{
    SemVer, cmp_semver_strict, compare_for_ecosystem, compatible_upper_bound, parse_for_ecosystem,
    parse_semver_strict,
};
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangeOp {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

impl RangeOp {
    fn from_lower(inclusive: bool) -> Self {
        if inclusive { Self::Ge } else { Self::Gt }
    }

    fn from_upper(inclusive: bool) -> Self {
        if inclusive { Self::Le } else { Self::Lt }
    }

    fn matches(self, ordering: Ordering) -> bool {
        match self {
            Self::Gt => ordering.is_gt(),
            Self::Ge => !ordering.is_lt(),
            Self::Lt => ordering.is_lt(),
            Self::Le => !ordering.is_gt(),
            Self::Eq => ordering.is_eq(),
            Self::Ne => !ordering.is_eq(),
        }
    }
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

pub(crate) fn cmp_versions(ecosystem: Ecosystem, a: &str, b: &str) -> Result<Ordering, String> {
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
    // No version contains a comparison operator. One here is a mistyped
    // operator (`>>1.0`, `>=1.0 <<2.0`) that a permissive ecosystem would
    // otherwise read as text and answer with a confident boolean.
    if version.contains(['<', '>', '=']) {
        return Err(format!("invalid comparator version: '{token}'"));
    }
    Ok(Comparator { op, version: version.to_string() })
}

fn is_wildcard(part: &str) -> bool {
    matches!(part, "x" | "X" | "*")
}

/// A wildcard anywhere in the release part, at any depth: `1.2.x`, `1.2.3.4.*`.
fn has_wildcard(token: &str) -> bool {
    let core = token.split(['-', '+']).next().unwrap_or_default();
    core.split('.').any(is_wildcard)
}

/// Split a leading `N!` (PEP 440) or `N:` (Debian, RPM) epoch off a range bound.
fn split_epoch(rest: &str) -> (&str, &str) {
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    match rest.as_bytes().get(digits) {
        Some(b'!' | b':') if digits > 0 => (&rest[..=digits], &rest[digits + 1..]),
        _ => ("", rest),
    }
}

/// Split an epoch and an optional `v` off a wildcarded version. The first
/// element is the prefix to put back on the bounds, so that `v1.2.x` yields
/// `v1.2.0` and `1!1.2.*` yields `1!1.2.0`.
fn split_wildcard_prefix(rest: &str) -> (&str, &str) {
    let rest = rest.trim();
    let (epoch, after_epoch) = split_epoch(rest);
    let core = strip_v(after_epoch);
    let prefix_len = epoch.len() + (after_epoch.len() - core.len());
    (&rest[..prefix_len], core)
}

/// The numeric components before a wildcard and the wildcard's position:
/// `1.2.x` → `([1, 2], 2)`, `1.2.3.4.x` → `([1, 2, 3, 4], 4)`.
fn parse_wildcard(core: &str) -> Option<(Vec<u64>, usize)> {
    let core = core.split(['-', '+']).next()?;
    let mut parts = Vec::new();
    for (depth, raw) in core.split('.').enumerate() {
        if is_wildcard(raw) {
            return Some((parts, depth));
        }
        parts.push(raw.parse().ok()?);
    }
    None
}

/// How many release components a bare numeric version (`1.2`: digits and dots
/// only) has, when it has fewer than the three of a complete release.
fn partial_release_count(bare: &str) -> Option<usize> {
    if !bare.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let count = parse_num_prefix(bare)?.0.len();
    (count < 3).then_some(count)
}

/// The edges of the window a wildcard names: `1.2.x` → `>=1.2.0,<1.3.0` and,
/// once the release triple is complete, `1.2.3.x` → `>=1.2.3,<1.2.4`.
fn wildcard_window(parts: &[u64], depth: usize) -> Result<(String, String), String> {
    let mut upper = parts.to_vec();
    let last = upper.last_mut().ok_or("missing version before wildcard")?;
    *last = increment(*last)?;
    let joined = |values: &[u64]| values.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
    if depth >= 3 {
        return Ok((joined(parts), joined(&upper)));
    }
    // A partial release is padded to the triple: `1` → `1.0.0`, `2` → `2.0.0`.
    let padded = |values: &[u64]| {
        fmt_triple(values.first().copied().unwrap_or(0), values.get(1).copied().unwrap_or(0), 0)
    };
    Ok((padded(parts), padded(&upper)))
}

/// PEP 440 prefix matching: `==1.2.*` admits every release whose public
/// version starts with `1.2`, including the pre- and development releases of
/// `1.2` itself. The window therefore opens at `1.2.dev0`, not at `1.2`.
fn pep440_prefix_window(prefix: &str, parts: &[u64]) -> Result<Vec<Comparator>, String> {
    let mut upper = parts.to_vec();
    let last = upper.last_mut().ok_or("missing version before wildcard")?;
    *last = increment(*last)?;
    let lower = parts.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
    let upper = upper.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
    Ok(vec![
        Comparator { op: RangeOp::Ge, version: format!("{prefix}{lower}.dev0") },
        Comparator { op: RangeOp::Lt, version: format!("{prefix}{upper}.dev0") },
    ])
}

/// Resolve an operator applied to a wildcarded version (`>=1.2.x`, `<=2.x`).
/// The bound has to land on the *edge* of the wildcarded window, which for the
/// exclusive operators is not the wildcard's own prefix: `>1.2.x` means "above
/// everything in 1.2", i.e. `>=1.3.0`, and `<=1.2.x` means `<1.3.0`. This is
/// npm's reading of the same syntax.
fn expand_operator_wildcard(
    ecosystem: Ecosystem,
    op: RangeOp,
    rest: &str,
) -> Result<Vec<Comparator>, String> {
    let (prefix, core) = split_wildcard_prefix(rest);
    let (parts, depth) =
        parse_wildcard(core).ok_or_else(|| format!("invalid wildcard range: '{rest}'"))?;
    if depth == 0 {
        return match op {
            RangeOp::Eq | RangeOp::Ge | RangeOp::Le => Ok(Vec::new()),
            RangeOp::Lt | RangeOp::Gt => Ok(vec![
                Comparator { op: RangeOp::Ge, version: "0.0.0".into() },
                Comparator { op: RangeOp::Lt, version: "0.0.0".into() },
            ]),
            RangeOp::Ne => Err(format!("'!=' does not support wildcard versions: '!={rest}'")),
        };
    }
    if ecosystem == Ecosystem::Pep440 && op == RangeOp::Eq {
        return pep440_prefix_window(prefix, &parts);
    }
    let (lower, upper) = wildcard_window(&parts, depth)?;
    let comparator = |op, version: &str| Comparator { op, version: format!("{prefix}{version}") };
    Ok(match op {
        RangeOp::Ge | RangeOp::Lt => vec![comparator(op, &lower)],
        RangeOp::Gt => vec![comparator(RangeOp::Ge, &upper)],
        RangeOp::Le => vec![comparator(RangeOp::Lt, &upper)],
        RangeOp::Eq => vec![comparator(RangeOp::Ge, &lower), comparator(RangeOp::Lt, &upper)],
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

fn expand_x_range(ecosystem: Ecosystem, token: &str) -> Result<Vec<Comparator>, String> {
    let (prefix, core) = split_wildcard_prefix(token);
    let (parts, depth) =
        parse_wildcard(core).ok_or_else(|| format!("invalid x-range: '{token}'"))?;
    if depth == 0 {
        // Leading wildcard (`x.1`) — nothing is constrained.
        return Ok(Vec::new());
    }
    if ecosystem == Ecosystem::Pep440 {
        return pep440_prefix_window(prefix, &parts);
    }
    let (lower, upper) = wildcard_window(&parts, depth)?;
    Ok(vec![
        Comparator { op: RangeOp::Ge, version: format!("{prefix}{lower}") },
        Comparator { op: RangeOp::Lt, version: format!("{prefix}{upper}") },
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
            op: RangeOp::from_lower(left_inclusive),
            version: lower.to_string(),
        });
    }
    let upper = parts[1].trim();
    if !upper.is_empty() {
        out.push(Comparator {
            op: RangeOp::from_upper(right_inclusive),
            version: upper.to_string(),
        });
    }
    Ok(out)
}

fn expand_token(token: &str, ecosystem: Ecosystem) -> Result<Vec<Comparator>, String> {
    let mut t = token.trim();
    if t.is_empty() {
        return Ok(Vec::new());
    }
    // npm writes plain equality as `=1.2.3` and `=1.2.x`.
    if ecosystem == Ecosystem::Npm
        && let Some(rest) = t.strip_prefix('=')
        && rest
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'v' | b'V' | b'x' | b'X' | b'*'))
    {
        t = rest;
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
        if ecosystem == Ecosystem::Pep440 {
            return Ok(vec![
                Comparator { op: RangeOp::Ge, version: rest.to_string() },
                Comparator { op: RangeOp::Lt, version: compatible_upper_bound(rest)? },
            ]);
        }
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
        return expand_operator_wildcard(ecosystem, op, rest);
    }
    if has_wildcard(t) {
        return expand_x_range(ecosystem, t);
    }
    // npm reads a bare complete version as an exact match (`1.2.3` is
    // `==1.2.3`) and a bare partial one as an x-range (`1.2` is `1.2.x`).
    if ecosystem == Ecosystem::Npm && !t.starts_with(['<', '>', '=', '!']) {
        if let Some(count) = partial_release_count(strip_v(t)) {
            let padded = format!("{}{}", strip_v(t), ".x".repeat(3 - count));
            return expand_x_range(ecosystem, &padded);
        }
        return Ok(vec![Comparator { op: RangeOp::Eq, version: t.to_string() }]);
    }
    Ok(vec![parse_comparator(t)?])
}

/// npm hyphen ranges: `1.2.3 - 2.3.4` is `>=1.2.3 <=2.3.4`. A partial lower
/// bound (`1.2`) is padded to the release (`>=1.2.0`), and a partial upper
/// bound (`2.3`, `2`) stops short of the next release (`<2.4.0`, `<3.0.0`).
fn npm_hyphen_ranges(tokens: Vec<String>) -> Vec<String> {
    fn is_version(token: &str) -> bool {
        token.as_bytes().first().is_some_and(|b| b.is_ascii_digit() || matches!(b, b'v' | b'V'))
    }
    fn without_equals(token: &str) -> &str {
        token.strip_prefix('=').unwrap_or(token)
    }
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0usize;
    while i < tokens.len() {
        let upper = tokens.get(i + 2).filter(|_| tokens.get(i + 1).is_some_and(|t| t == "-"));
        let Some(upper) = upper else {
            out.push(tokens[i].clone());
            i += 1;
            continue;
        };
        let (lower, upper) = (without_equals(&tokens[i]), without_equals(upper));
        if !is_version(lower) || !is_version(upper) {
            out.push(tokens[i].clone());
            i += 1;
            continue;
        }
        let lower = match partial_release_count(strip_v(lower)) {
            Some(count) => format!("{}{}", strip_v(lower), ".0".repeat(3 - count)),
            None => lower.to_string(),
        };
        out.push(format!(">={lower}"));
        out.push(match partial_upper_bound(upper) {
            Some(bound) => format!("<{bound}"),
            None => format!("<={upper}"),
        });
        i += 3;
    }
    out
}

/// The exclusive bound a partial npm hyphen-range limit names: `2.3` →
/// `2.4.0`. `None` when the limit is complete (`2.3.4`) or a wildcard (`2.x`).
fn partial_upper_bound(upper: &str) -> Option<String> {
    let bare = strip_v(upper);
    partial_release_count(bare)?;
    let (mut parts, _) = parse_num_prefix(bare)?;
    let last = parts.last_mut()?;
    *last = last.checked_add(1)?;
    parts.resize(3, 0);
    Some(parts.iter().map(u64::to_string).collect::<Vec<_>>().join("."))
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
        if matches!(out[i].as_str(), ">=" | "<=" | "!=" | "==" | "~=" | ">" | "<")
            && i + 1 < out.len()
        {
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
    pub(crate) fn parse(input: &str, ecosystem: Ecosystem) -> Result<Self, String> {
        let mut branches = Vec::new();
        for branch in input.split("||") {
            let tokens = split_branch(branch);
            let tokens =
                if ecosystem == Ecosystem::Npm { npm_hyphen_ranges(tokens) } else { tokens };
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
                clauses.extend(expand_token(&token, ecosystem)?);
            }
            branches.push(clauses);
        }
        Ok(Self { branches })
    }

    pub(crate) fn contains(&self, ecosystem: Ecosystem, version: &str) -> Result<bool, String> {
        if ecosystem == Ecosystem::Npm {
            let target = parse_semver_strict(version)?;
            for branch in &self.branches {
                if NpmBranch::parse(branch)?.contains(&target) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        parse_for_ecosystem(ecosystem, version)?;
        for branch in &self.branches {
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
        if !clause.op.matches(ord) {
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

/// An npm AND-branch admits prereleases only on cores mentioned by one
/// of its own positive comparators. Keep that rule when intersecting branches.
#[derive(Default)]
struct NpmBranch {
    lower: Option<(SemVer, bool)>,
    upper: Option<(SemVer, bool)>,
    excluded: HashSet<SemVer>,
    prerelease_cores: HashSet<(u64, u64, u64)>,
}

impl NpmBranch {
    fn parse(branch: &[Comparator]) -> Result<Self, String> {
        let mut bounds = Self::default();
        for clause in branch {
            let value = parse_semver_strict(&clause.version)?;
            let op = clause.op;
            if value.pre.is_some() && op != RangeOp::Ne {
                bounds.prerelease_cores.insert(value.core());
            }
            if matches!(op, RangeOp::Gt | RangeOp::Ge | RangeOp::Eq) {
                Self::tighten(&mut bounds.lower, &value, op != RangeOp::Gt, Ordering::Greater);
            }
            if matches!(op, RangeOp::Lt | RangeOp::Le | RangeOp::Eq) {
                Self::tighten(&mut bounds.upper, &value, op != RangeOp::Lt, Ordering::Less);
            }
            if op == RangeOp::Ne {
                bounds.excluded.insert(value);
            }
        }
        Ok(bounds)
    }

    fn tighten(
        bound: &mut Option<(SemVer, bool)>,
        value: &SemVer,
        inclusive: bool,
        tighter: Ordering,
    ) {
        if let Some((previous, was_inclusive)) = bound {
            let ordering = cmp_semver_strict(value, previous);
            if ordering.is_eq() {
                *was_inclusive &= inclusive;
            }
            if ordering != tighter {
                return;
            }
        }
        *bound = Some((value.clone(), inclusive));
    }

    fn contains(&self, target: &SemVer) -> bool {
        self.lower.as_ref().is_none_or(|(v, inclusive)| {
            RangeOp::from_lower(*inclusive).matches(cmp_semver_strict(target, v))
        }) && self.upper.as_ref().is_none_or(|(v, inclusive)| {
            RangeOp::from_upper(*inclusive).matches(cmp_semver_strict(target, v))
        }) && !self.excluded.contains(target)
            && (target.pre.is_none() || self.prerelease_cores.contains(&target.core()))
    }

    fn intersects(&self, other: &Self) -> bool {
        let accepts = |v: &SemVer| self.contains(v) && other.contains(v);
        if accepts(&SemVer::release(0, 0, 0)) {
            return true;
        }
        for &(major, minor, patch) in self.prerelease_cores.intersection(&other.prerelease_cores) {
            if accepts(&SemVer { pre: Some("0".into()), ..SemVer::release(major, minor, patch) }) {
                return true;
            }
        }
        // Any remaining first match is a lower endpoint, the stable release
        // of its core, or the successor of a strict endpoint or exclusion.
        // Within a prerelease core the immediate successor of p is p.0.
        for branch in [self, other] {
            let endpoints = branch.lower.iter().map(|(v, _)| v).chain(&branch.excluded);
            for v in endpoints {
                let (major, minor, patch) = v.core();
                if accepts(v) || accepts(&SemVer::release(major, minor, patch)) {
                    return true;
                }
                let next = if let Some(pre) = &v.pre {
                    Some(SemVer {
                        pre: Some(format!("{pre}.0")),
                        ..SemVer::release(major, minor, patch)
                    })
                } else {
                    v.next_release()
                };
                if next.is_some_and(|v| accepts(&v)) {
                    return true;
                }
            }
        }
        false
    }
}

fn branches_intersect(
    ecosystem: Ecosystem,
    left: &[Comparator],
    right: &[Comparator],
) -> Result<bool, String> {
    if ecosystem == Ecosystem::Npm {
        return Ok(NpmBranch::parse(left)?.intersects(&NpmBranch::parse(right)?));
    }
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
