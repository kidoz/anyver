//! Generic segment comparator shared by most ecosystems.
//!
//! Operates on the `Vec<Seg>` produced by the generic parser: trailing-zero
//! normalization, pairwise segment comparison, and the epoch-aware
//! `cmp_parsed` entry point.

use crate::parser::{ParsedVersion, Seg, tag_weight};
use std::cmp::Ordering;

pub(crate) fn normalized(segs: &[Seg]) -> &[Seg] {
    // Strip trailing transparent tokens: Num(0) and release-alias tags
    // (weight 30). This keeps trailing-zero equivalence (`1.0 == 1.0.0`)
    // and treats Maven-style `1.0-final` / `1.0-ga` as equal to `1.0`.
    let mut end = segs.len();
    while end > 0 {
        match &segs[end - 1] {
            Seg::Num(0) => end -= 1,
            Seg::Text(t) if tag_weight(t) == Some(30) => end -= 1,
            _ => break,
        }
    }
    &segs[..end]
}

pub(crate) fn cmp_two(a: &Seg, b: &Seg) -> Ordering {
    match (a, b) {
        (Seg::Num(x), Seg::Num(y)) => x.cmp(y),
        (Seg::Text(x), Seg::Text(y)) => {
            let wx = tag_weight(x);
            let wy = tag_weight(y);
            match (wx, wy) {
                (Some(wa), Some(wb)) => wa.cmp(&wb),
                (Some(wa), None) => wa.cmp(&29),
                (None, Some(wb)) => 29_i32.cmp(&wb),
                (None, None) => x.cmp(y),
            }
        }
        (Seg::Num(_), Seg::Text(_)) => Ordering::Greater,
        (Seg::Text(_), Seg::Num(_)) => Ordering::Less,
    }
}

pub(crate) fn text_effective_weight(s: &str) -> i32 {
    tag_weight(s).unwrap_or(29)
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn cmp_segments(left: &[Seg], right: &[Seg]) -> Ordering {
    // Normalize: strip trailing Num(0) to ensure transitivity
    // (e.g., 1.0 == 1.0.0 and 1.0 < 1.0.post1 implies 1.0.0 < 1.0.post1)
    let nl = normalized(left);
    let nr = normalized(right);
    let n = nl.len().max(nr.len());
    for i in 0..n {
        match (nl.get(i), nr.get(i)) {
            (None, None) => return Ordering::Equal,
            (None, Some(Seg::Num(v))) => {
                if *v == 0 {
                    continue;
                } // non-trailing zero before text
                return Ordering::Less;
            }
            (Some(Seg::Num(v)), None) => {
                if *v == 0 {
                    continue;
                }
                return Ordering::Greater;
            }
            (None, Some(Seg::Text(tag))) => {
                if text_effective_weight(tag) < 30 {
                    return Ordering::Greater;
                } // we are release, they pre
                return Ordering::Less; // they are post
            }
            (Some(Seg::Text(tag)), None) => {
                if text_effective_weight(tag) < 30 {
                    return Ordering::Less;
                } // we are pre
                return Ordering::Greater; // we are post
            }
            (Some(sa), Some(sb)) => {
                let ord = cmp_two(sa, sb);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
    Ordering::Equal
}

pub(crate) fn cmp_parsed(a: &ParsedVersion, b: &ParsedVersion) -> Ordering {
    a.epoch.cmp(&b.epoch).then_with(|| cmp_segments(&a.segments, &b.segments))
}
