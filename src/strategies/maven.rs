//! Apache Maven version semantics — a port of
//! `org.apache.maven.artifact.versioning.ComparableVersion`.
//!
//! Maven does not compare a flat segment list the way [`super::generic`] does:
//!
//! - `.` separates items inside the current list, `-` opens a **nested** list,
//!   and a digit↔alpha transition implies a separator (`1.0rc1` == `1.0-rc-1`)
//! - qualifiers rank by a fixed table — `alpha < beta < milestone < rc <
//!   snapshot < "" (release) < sp` — with `ga`/`final`/`release` aliased to the
//!   empty (release) qualifier and `cr` to `rc`
//! - `a`/`b`/`m` are shorthand for `alpha`/`beta`/`milestone` **only** when
//!   directly followed by a digit (`1-m3`), never on their own (`1-m`)
//! - *unknown* qualifiers sort **above** every known one and above the release
//!   itself, so `1.0-jre > 1.0 > 1.0-rc1`; unknown qualifiers compare lexically
//! - trailing nulls (zero, the release qualifier, empty lists) are stripped, so
//!   `1 == 1.0 == 1.0.0 == 1-0 == 1.0-final`
//! - numeric items are arbitrary-precision (Maven falls back to `BigInteger`)
//!
//! Deliberate deviations from `ComparableVersion`, each of them explicit
//! rather than a silent difference in ordering:
//!
//! - input is trimmed, matching the rest of this crate; Maven would read the
//!   trailing space of `"1.0 "` as an unknown qualifier
//! - nesting deeper than [`MAX_DEPTH`] is **rejected**, so an adversarial
//!   `1-1-1-…` cannot drive unbounded recursion in [`cmp_maven`]. Real versions
//!   nest ≤ 4 deep; every accepted input is ordered exactly like Maven.
//! - non-ASCII numerals are rejected by `validate_maven`. Java's
//!   `Character.isDigit` accepts them and `BigInteger` parses them (Maven reads
//!   `1.٢` as `1.2`), which this ASCII tokenizer cannot reproduce.

use std::cmp::Ordering;

/// Known qualifiers in ascending order. The empty string is the release itself
/// and sits at [`RELEASE_RANK`]; anything absent from this table is an *unknown*
/// qualifier and outranks every entry here (Maven's `"7-<qualifier>"` key).
const QUALIFIERS: [&str; 7] = ["alpha", "beta", "milestone", "rc", "snapshot", "", "sp"];
const RELEASE_RANK: usize = 5;

/// Maximum list-nesting depth; see the module docs. Deeper input is rejected
/// rather than flattened, so ordering is never silently changed.
const MAX_DEPTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// Canonical decimal digits — no leading zeros, empty means zero — so
    /// values wider than any integer type still compare exactly.
    Int(String),
    /// Alias-resolved qualifier; empty means "the release itself".
    Str(String),
    List(Vec<Item>),
}

impl Item {
    fn zero() -> Self {
        Item::Int(String::new())
    }

    /// Maven's `isNull()`: the item carries no information and is strippable
    /// when trailing.
    fn is_null(&self) -> bool {
        match self {
            Item::Int(digits) => digits.is_empty(),
            Item::Str(qualifier) => qualifier.is_empty(),
            Item::List(items) => items.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MavenVersion {
    items: Vec<Item>,
}

impl MavenVersion {
    /// Maven's `getCanonical()`: the normalized item tree rendered back to a
    /// version string, so every version that compares equal renders the same
    /// text (`1.0-1` and `1.0.0-1` both canonicalize to `1-1`). `Version`'s
    /// `__hash__` hashes this instead of the raw parse, which would otherwise
    /// give equal Maven versions different hashes.
    pub(crate) fn canonical(&self) -> String {
        fn render(items: &[Item]) -> String {
            let mut buf = String::new();
            for item in items {
                if !buf.is_empty() {
                    buf.push(if matches!(item, Item::List(_)) { '-' } else { '.' });
                }
                match item {
                    Item::Int(digits) => buf.push_str(if digits.is_empty() { "0" } else { digits }),
                    Item::Str(qualifier) => buf.push_str(qualifier),
                    Item::List(sub) => buf.push_str(&render(sub)),
                }
            }
            buf
        }
        render(&self.items)
    }
}

fn qualifier_rank(qualifier: &str) -> Option<usize> {
    QUALIFIERS.iter().position(|q| *q == qualifier)
}

/// Maven's `comparableQualifier` ordering: known qualifiers by table position,
/// unknown ones after all of them and lexically among themselves. The lexical
/// tie-break compares UTF-16 code units, because Maven compares Java `String`s
/// — Rust's byte order would disagree above the BMP (`1-😀` vs `1-\u{e000}`).
fn cmp_qualifier(a: &str, b: &str) -> Ordering {
    match (qualifier_rank(a), qualifier_rank(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.encode_utf16().cmp(b.encode_utf16()),
    }
}

/// Resolve shorthands and aliases exactly as Maven's `StringItem` constructor:
/// single-letter `a`/`b`/`m` expand only when the qualifier is followed by a
/// digit, then `ga`/`final`/`release` collapse to the release and `cr` to `rc`.
fn canonical_qualifier(raw: &str, followed_by_digit: bool) -> String {
    let expanded = if followed_by_digit && raw.len() == 1 {
        match raw {
            "a" => "alpha",
            "b" => "beta",
            "m" => "milestone",
            other => other,
        }
    } else {
        raw
    };
    match expanded {
        "ga" | "final" | "release" => "",
        "cr" => "rc",
        other => other,
    }
    .to_string()
}

fn int_item(digits: &str) -> Item {
    Item::Int(digits.trim_start_matches('0').to_string())
}

fn parse_item(is_digit: bool, buf: &str, followed_by_digit: bool) -> Item {
    if is_digit { int_item(buf) } else { Item::Str(canonical_qualifier(buf, followed_by_digit)) }
}

/// Maven's `list.add(list = new ListItem())`: descend one nesting level. The
/// child is attached to its parent when the stack unwinds — equivalent, since
/// Maven normalizes innermost-first and never revisits a parent mid-parse.
fn push_list(stack: &mut Vec<Vec<Item>>, input: &str) -> Result<(), String> {
    if stack.len() >= MAX_DEPTH {
        return Err(format!("Maven version nests more than {MAX_DEPTH} levels deep: '{input}'"));
    }
    stack.push(Vec::new());
    Ok(())
}

/// Maven's `ListItem.normalize()`: drop trailing null items, looking past
/// non-null nested lists rather than stopping at them.
fn normalize(items: &mut Vec<Item>) {
    let mut i = items.len();
    while i > 0 {
        i -= 1;
        if items[i].is_null() {
            items.remove(i);
        } else if !matches!(items[i], Item::List(_)) {
            break;
        }
    }
}

pub(crate) fn parse_maven(input: &str) -> Result<MavenVersion, String> {
    let chars: Vec<char> = input.trim().to_lowercase().chars().collect();
    let text = |range: std::ops::Range<usize>| -> String { chars[range].iter().collect() };

    let mut stack: Vec<Vec<Item>> = vec![Vec::new()];
    let mut is_digit = false;
    let mut start = 0usize;

    for (i, &c) in chars.iter().enumerate() {
        if c == '.' || c == '-' {
            let item = if i == start {
                Item::zero()
            } else {
                parse_item(is_digit, &text(start..i), false)
            };
            stack.last_mut().expect("stack is never empty").push(item);
            start = i + 1;
            if c == '-' {
                push_list(&mut stack, input)?;
            }
        } else if c.is_ascii_digit() {
            if !is_digit && i > start {
                // `1.0.0.x1 < 1.0.0-x2`: a qualifier that runs into a digit
                // starts its own nested list, so `.x` behaves like `-x`.
                if !stack.last().expect("stack is never empty").is_empty() {
                    push_list(&mut stack, input)?;
                }
                stack.last_mut().expect("stack is never empty").push(parse_item(
                    false,
                    &text(start..i),
                    true,
                ));
                start = i;
                // …and the digits that follow nest one level deeper again, so
                // `1-a1` parses identically to `1-alpha-1`.
                push_list(&mut stack, input)?;
            }
            is_digit = true;
        } else {
            if is_digit && i > start {
                stack.last_mut().expect("stack is never empty").push(parse_item(
                    true,
                    &text(start..i),
                    false,
                ));
                start = i;
                push_list(&mut stack, input)?;
            }
            is_digit = false;
        }
    }

    if chars.len() > start {
        if !is_digit && !stack.last().expect("stack is never empty").is_empty() {
            push_list(&mut stack, input)?;
        }
        let item = parse_item(is_digit, &text(start..chars.len()), false);
        stack.last_mut().expect("stack is never empty").push(item);
    }

    while stack.len() > 1 {
        let mut child = stack.pop().expect("stack has at least two entries");
        normalize(&mut child);
        stack.last_mut().expect("stack is never empty").push(Item::List(child));
    }
    let mut items = stack.pop().expect("stack is never empty");
    normalize(&mut items);
    Ok(MavenVersion { items })
}

/// Arbitrary-precision compare of two canonical (leading-zero-free) digit runs.
fn cmp_int(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Compare an item against an absent one — Maven's `compareTo(null)`, i.e. the
/// rule that decides whether the longer version wins (`1.1 > 1`) or loses
/// (`1.0-rc1 < 1.0`).
fn cmp_item_null(item: &Item) -> Ordering {
    match item {
        Item::Int(digits) => {
            if digits.is_empty() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
        Item::Str(qualifier) => {
            qualifier_rank(qualifier).map_or(Ordering::Greater, |rank| rank.cmp(&RELEASE_RANK))
        }
        Item::List(items) => {
            for item in items {
                let ord = cmp_item_null(item);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            Ordering::Equal
        }
    }
}

/// Ordering between items of different kinds: a qualifier ranks below a nested
/// list, which ranks below a number — `1-rc < 1-1 < 1.1`.
fn kind_rank(item: &Item) -> u8 {
    match item {
        Item::Str(_) => 0,
        Item::List(_) => 1,
        Item::Int(_) => 2,
    }
}

fn cmp_item(a: &Item, b: &Item) -> Ordering {
    match (a, b) {
        (Item::Int(x), Item::Int(y)) => cmp_int(x, y),
        (Item::Str(x), Item::Str(y)) => cmp_qualifier(x, y),
        (Item::List(x), Item::List(y)) => cmp_list(x, y),
        _ => kind_rank(a).cmp(&kind_rank(b)),
    }
}

fn cmp_list(left: &[Item], right: &[Item]) -> Ordering {
    for i in 0..left.len().max(right.len()) {
        let ord = match (left.get(i), right.get(i)) {
            (Some(l), Some(r)) => cmp_item(l, r),
            (Some(l), None) => cmp_item_null(l),
            (None, Some(r)) => cmp_item_null(r).reverse(),
            (None, None) => Ordering::Equal,
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

pub(crate) fn cmp_maven(a: &MavenVersion, b: &MavenVersion) -> Ordering {
    cmp_list(&a.items, &b.items)
}
