//! Homebrew package revision ordering with generic upstream version ordering.

use super::{SortKey, generic_sort_key};
use crate::parser::parse_generic;

pub(super) fn parse_homebrew(input: &str) -> Result<SortKey, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("empty Homebrew version".to_string());
    }
    let (upstream, revision) = match input.rsplit_once('_') {
        Some((upstream, revision))
            if !upstream.is_empty()
                && !revision.is_empty()
                && revision.bytes().all(|b| b.is_ascii_digit()) =>
        {
            (upstream, revision.trim_start_matches('0'))
        }
        _ => (input, ""),
    };
    let head = upstream == "HEAD" || upstream.starts_with("HEAD-");
    let mut key = vec![(i32::from(head), 0, String::new())];
    if !head {
        key.extend(generic_sort_key(&parse_generic(upstream)));
    }
    key.push((0, revision.len() as u64, revision.to_string()));
    Ok(key)
}
