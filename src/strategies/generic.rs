//! Generic segment comparator shared by most ecosystems.
//!
//! Operates on the `Vec<Seg>` produced by the generic parser: trailing-zero
//! normalization, pairwise segment comparison, and the epoch-aware
//! `cmp_parsed` entry point.

use crate::parser::{ParsedVersion, Seg, tag_weight};
use std::borrow::Cow;
use std::cmp::Ordering;

use super::SortKey;

pub(crate) fn normalized(segs: &[Seg]) -> &[Seg] {
    let mut end = segs.len();
    while end > 0
        && (matches!(&segs[end - 1], Seg::Num(0))
            || matches!(&segs[end - 1], Seg::Text(t) if tag_weight(t) == Some(30)))
    {
        end -= 1;
    }
    &segs[..end]
}

/// Canonical segments shared by generic comparison and portable keys.
/// Common releases borrow their input rather than allocating a copy.
pub(crate) fn canonical_segments(segs: &[Seg]) -> Cow<'_, [Seg]> {
    let segs = normalized(segs);
    let needs_copy = segs.iter().enumerate().any(|(i, seg)| {
        matches!(seg, Seg::Text(t) if tag_weight(t) == Some(30))
            || matches!(seg, Seg::Text(_)) && i > 0 && segs[i - 1] == Seg::Num(0)
    });
    if !needs_copy {
        return Cow::Borrowed(segs);
    }
    let mut out = Vec::with_capacity(segs.len());
    for seg in segs {
        if let Seg::Text(tag) = seg {
            // Release aliases are transparent, including within a suffix.
            if tag_weight(tag) == Some(30) {
                continue;
            }
            // An omitted numeric zero must have the same effect before a tag
            // as at the end; otherwise release/prerelease/postrelease cycles
            // are possible (1.0-rc > 1-post > 1 > 1.0-rc).
            while out.last() == Some(&Seg::Num(0)) {
                out.pop();
            }
        }
        out.push(seg.clone());
    }
    while out.last() == Some(&Seg::Num(0)) {
        out.pop();
    }
    Cow::Owned(out)
}

/// Canonical digit strings compare by length, then lexically.
fn cmp_wide(a: &str, b: &str) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

pub(crate) fn cmp_two(a: &Seg, b: &Seg) -> Ordering {
    match (a, b) {
        (Seg::Num(x), Seg::Num(y)) => x.cmp(y),
        (Seg::Wide(x), Seg::Wide(y)) => cmp_wide(x, y),
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
        // A wide number never fits in u64, so it outranks every `Num`. Both
        // kinds of number outrank text.
        (Seg::Num(_) | Seg::Wide(_), Seg::Text(_)) | (Seg::Wide(_), Seg::Num(_)) => {
            Ordering::Greater
        }
        (Seg::Text(_), Seg::Num(_) | Seg::Wide(_)) | (Seg::Num(_), Seg::Wide(_)) => Ordering::Less,
    }
}

pub(crate) fn text_effective_weight(s: &str) -> i32 {
    tag_weight(s).unwrap_or(29)
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn cmp_segments(left: &[Seg], right: &[Seg]) -> Ordering {
    // Normalize: strip trailing Num(0) to ensure transitivity
    // (e.g., 1.0 == 1.0.0 and 1.0 < 1.0.post1 implies 1.0.0 < 1.0.post1)
    let nl = canonical_segments(left);
    let nr = canonical_segments(right);
    let n = nl.len().max(nr.len());
    for i in 0..n {
        match (nl.get(i), nr.get(i)) {
            (None, None) => return Ordering::Equal,
            // A wide number is never zero, so it is a real trailing component.
            (None, Some(Seg::Wide(_))) => return Ordering::Less,
            (Some(Seg::Wide(_)), None) => return Ordering::Greater,
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

pub(crate) fn generic_sort_key(version: &ParsedVersion) -> SortKey {
    let mut key = vec![(2, version.epoch, String::new())];
    key.extend(canonical_segments(&version.segments).iter().map(|segment| match segment {
        Seg::Num(number) => (100, *number, String::new()),
        // Rank 101 keeps a wide number above every u64 component.
        Seg::Wide(digits) => (101, digits.len() as u64, digits.clone()),
        Seg::Text(text) => match tag_weight(text) {
            Some(weight) => (weight, 0, String::new()),
            None => (29, 0, text.clone()),
        },
    }));
    key.push((30, 0, String::new()));
    key
}
