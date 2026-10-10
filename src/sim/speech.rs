//! Native words in what people say (Laz): a Roduro calls a friend *oqe*, a
//! Horaro wishes you *helo*, the way anyone in a foreign land hears the
//! locals' own words for things. A line in `data/lines/` asks for one with
//! `{w:friend}` (or `{W:friend}` to start a sentence); the word comes from
//! the speaker's tongue (`crate::names::word`).
//!
//! In the text that's handed on, such a word is marked `⟦native|meaning⟧`.
//! The window draws the native word set apart and shows the meaning on
//! hover; anything that can't hover (the play tool, a remark over someone's
//! head) uses `plain`, which gives "native (meaning)".

use crate::names::{self, Tongue};

const OPEN: char = '⟦';
const CLOSE: char = '⟧';

/// The marked word for a root in a tongue (capitalised if `cap`); the plain
/// English meaning if the root isn't known.
pub fn native_word(root: &str, tongue: Tongue, cap: bool) -> String {
    let Some(r) = names::root(root) else { return root.replace('_', " ") };
    let Some(w) = names::word(root, tongue) else { return r.gloss.clone() };
    let w = if cap { capitalise(&w) } else { w };
    // The first sense of the gloss ("luck, chance" is "luck").
    let first = r.gloss.split(',').next().unwrap_or(&r.gloss).trim();
    let meaning = ["a ", "an ", "the "].iter().find_map(|a| first.strip_prefix(a)).unwrap_or(first).to_string();
    format!("{OPEN}{w}|{meaning}{CLOSE}")
}

/// A thing's name as the speaker says it, marked with its English name.
pub fn native_thing(english: &str, tongue: Tongue) -> String {
    match names::name_in_speech(english, tongue) {
        Some(w) => format!("{OPEN}{w}|{}{CLOSE}", english.to_lowercase()),
        None => english.to_lowercase(),
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// The text cut into runs: plain text, or a native word with its meaning.
pub fn spans(text: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(a) = rest.find(OPEN) {
        if a > 0 {
            out.push((rest[..a].to_string(), None));
        }
        let after = &rest[a + OPEN.len_utf8()..];
        let Some(b) = after.find(CLOSE) else {
            out.push((rest[a..].to_string(), None));
            return out;
        };
        let inner = &after[..b];
        let (word, meaning) = inner.split_once('|').unwrap_or((inner, ""));
        out.push((word.to_string(), Some(meaning.to_string())));
        rest = &after[b + CLOSE.len_utf8()..];
    }
    if !rest.is_empty() {
        out.push((rest.to_string(), None));
    }
    out
}

/// The text with each native word followed by its meaning: "oqe (friend)".
pub fn plain(text: &str) -> String {
    spans(text).into_iter().map(|(t, m)| match m {
        Some(m) if !m.is_empty() => format!("{t} ({m})"),
        _ => t,
    }).collect()
}

/// The text with native words only, the meanings dropped.
pub fn bare(text: &str) -> String {
    spans(text).into_iter().map(|(t, _)| t).collect()
}
