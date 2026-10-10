//! Putting words together: compounds and endings, tongue by tongue.
//!
//! This works on finished words of one tongue (a new compound is made by
//! that tongue's speakers out of their own words), and mends the place
//! where the two meet so the result still sounds like the tongue. The
//! patterns are data (`assets/lang/grammar.ron`).

use serde::Deserialize;

use super::sound::{self, is_vowel, Step};
use super::Tongue;

/// Which word comes first in a compound.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// The describing word, then the main word: "earth-kind".
    HeadLast,
    /// The main word, then the describing word: "kind of earth".
    HeadFirst,
}

/// Which end of a word is kept when it is cut short.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    Start,
    End,
}

/// Which syllable is said hardest.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stress {
    First,
    NextToLast,
    Last,
}

/// Whether an affix goes before the word or after it.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Before,
    After,
}

/// The endings (and beginnings) naming uses.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Affix {
    /// A people: "earth-kind".
    Kind,
    /// One who does it: "judge-r".
    Agent,
    /// Where it is.
    Place,
    Small,
    Great,
    /// The highest: the mark of the Trinity.
    Most,
}

/// One tongue's naming grammar.
#[derive(Deserialize, Clone, Debug)]
pub struct Grammar {
    pub tongue: Tongue,
    pub order: Order,
    pub longest: usize,
    pub keep: Keep,
    pub stress: Stress,
    pub kind: (String, Side),
    pub agent: (String, Side),
    pub place: (String, Side),
    pub small: (String, Side),
    pub great: (String, Side),
    pub most: (String, Side),
    pub of: String,
    pub male: Vec<String>,
    pub female: Vec<String>,
    pub either: Vec<String>,
}

impl Grammar {
    pub fn affix(&self, a: Affix) -> &(String, Side) {
        match a {
            Affix::Kind => &self.kind,
            Affix::Agent => &self.agent,
            Affix::Place => &self.place,
            Affix::Small => &self.small,
            Affix::Great => &self.great,
            Affix::Most => &self.most,
        }
    }
}

/// A tongue's grammar.
pub fn grammar(tongue: Tongue) -> &'static Grammar {
    super::data().grammar.iter().find(|g| g.tongue == tongue).unwrap_or_else(|| panic!("assets/lang/grammar.ron has nothing for {tongue:?}"))
}

/// Where each beat's vowel sits in a word: (first sound, one past the last).
/// A Ṭaḍoro glide (`ai au ei ae`) is one beat; everywhere else each vowel is.
fn beats(w: &[char], tongue: Tongue) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < w.len() {
        if is_vowel(w[i]) {
            let glide = tongue == Tongue::Tadoro && i + 1 < w.len() && matches!((w[i], w[i + 1]), ('a', 'i') | ('a', 'u') | ('e', 'i') | ('a', 'e'));
            let end = if glide { i + 2 } else { i + 1 };
            out.push((i, end));
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

/// Cut a word down to `n` beats, keeping one end.
fn cut(w: &[char], n: usize, keep: Keep, tongue: Tongue) -> Vec<char> {
    let at = beats(w, tongue);
    if at.len() <= n || n == 0 {
        return w.to_vec();
    }
    match keep {
        // Up to and including the nth beat.
        Keep::Start => w[..at[n - 1].1].to_vec(),
        Keep::End => {
            // From the consonant that opens the nth beat from the end.
            let v = at[at.len() - n].0;
            let start = if v > 0 && !is_vowel(w[v - 1]) { v - 1 } else { v };
            w[start..].to_vec()
        }
    }
}

/// Put two finished words of a tongue together, `a` first, mending the
/// place where they meet.
pub fn join(tongue: Tongue, a: &str, b: &str) -> String {
    let (a, b) = (sound::sounds(a), sound::sounds(b));
    if a.is_empty() || b.is_empty() {
        return sound::spell(&[a, b].concat());
    }
    let (end, start) = (*a.last().unwrap(), b[0]);
    let mut w: Vec<char>;
    match tongue {
        Tongue::First | Tongue::Roduro => {
            // Two vowels never touch: the catch goes between. A syllable said
            // twice over the join is said once.
            w = a.clone();
            let twice = a.len() >= 2 && b.len() >= 2 && a[a.len() - 2..] == b[..2] && !is_vowel(b[0]);
            if is_vowel(end) && is_vowel(start) {
                w.push('ʻ');
            }
            w.extend_from_slice(if twice { &b[2..] } else { &b });
        }
        Tongue::Qotiro => {
            // Consonants may meet, two at most (three if the last two are a
            // stop and r, which start a syllable together); otherwise an `a`
            // comes between. The same consonant twice is said once.
            w = a.clone();
            let trail = a.iter().rev().take_while(|c| !is_vowel(**c)).count();
            let lead = b.iter().take_while(|c| !is_vowel(**c)).count();
            let same = !is_vowel(end) && end == start;
            let met = trail + lead - same as usize;
            let stop_r = lead >= 2 && matches!(b[lead - 2], 'p' | 't' | 'k' | 'q' | 'd' | 'g') && b[lead - 1] == 'r';
            if met > 3 || (met == 3 && !stop_r) {
                w.push('a');
                w.extend_from_slice(&b);
            } else {
                w.extend_from_slice(if same { &b[1..] } else { &b });
            }
            sound::run(&mut w, &[Step::NoVowelPairs, Step::Devoice]);
        }
        Tongue::Horaro => {
            w = [a, b].concat();
            sound::run(&mut w, &[Step::VowelRun]);
        }
        Tongue::Tadoro => {
            w = a.clone();
            if !is_vowel(end) && !is_vowel(start) {
                w.push('e');
            }
            w.extend_from_slice(&b);
            sound::run(&mut w, &[Step::VowelsMeet]);
        }
    }
    sound::spell(&w)
}

/// A compound in a tongue: `describing` says what kind of `main` it is
/// ("sea" + "wind" is a wind of the sea). The tongue's own order is used,
/// and the second word is cut short if the whole would run too long.
pub fn compound(tongue: Tongue, describing: &str, main: &str) -> String {
    let g = grammar(tongue);
    let (first, second) = match g.order {
        Order::HeadLast => (describing, main),
        Order::HeadFirst => (main, describing),
    };
    let (a, b) = (sound::sounds(first), sound::sounds(second));
    let room = g.longest.saturating_sub(beats(&a, tongue).len()).max(1);
    let b = cut(&b, room, g.keep, tongue);
    join(tongue, &sound::spell(&a), &sound::spell(&b))
}

/// A word with one of the tongue's endings (or beginnings) on it.
pub fn with(tongue: Tongue, word: &str, affix: Affix) -> String {
    let (form, side) = grammar(tongue).affix(affix);
    match side {
        Side::Before => join(tongue, form, word),
        Side::After => join(tongue, word, form),
    }
}

/// "A of B" as two words with the tongue's small word between.
pub fn of(tongue: Tongue, a: &str, b: &str) -> String {
    format!("{a} {} {b}", grammar(tongue).of)
}
