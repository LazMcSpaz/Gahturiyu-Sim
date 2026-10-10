//! Sound changes: how a First Speech root becomes a word in each tongue.
//!
//! The rules themselves are data (`assets/lang/sounds.ron`): an ordered list
//! for each branch and each tongue. This file only knows how to carry out a
//! step. A word is worked on as a row of single sounds; `th` and `sh` count
//! as one sound each (held here as `θ` and `š`, and spelled back on the way
//! out).

use serde::Deserialize;

/// Where in a word a plain change happens.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum At {
    Anywhere,
    /// Only as the word's first sound.
    Start,
    /// Anywhere but first.
    Inside,
    /// A vowel that is not the word's first (the first carries the stress).
    LaterVowels,
}

/// One step of a tongue's rule list. The plain-language account of each
/// named step is beside it in `sounds.ron`.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum Step {
    /// One sound becomes another (or several, or nothing).
    Change(String, String, At),
    EchoVowel,
    RColour,
    DropLastVowel,
    NoVowelPairs,
    DropSecondVowel,
    NoDoubles,
    Devoice,
    YMelts,
    HFades,
    VowelRun,
    ShedLastVowel,
    OpenO,
    VowelsMeet,
    BlurMiddle,
    PartClusters,
    EndOnVowel,
    CatchVowels,
}

/// Turn spelling into single sounds.
pub fn sounds(spelled: &str) -> Vec<char> {
    spelled.replace("th", "θ").replace("sh", "š").chars().collect()
}

/// Turn single sounds back into spelling.
pub fn spell(word: &[char]) -> String {
    let mut s = String::new();
    for &c in word {
        match c {
            'θ' => s.push_str("th"),
            'š' => s.push_str("sh"),
            c => s.push(c),
        }
    }
    s
}

pub fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'ì' | 'o' | 'u')
}

/// How many vowels a word has (in the First Speech, its syllables).
pub fn vowels(w: &[char]) -> usize {
    w.iter().filter(|c| is_vowel(**c)).count()
}

/// Carry out a list of steps, in order.
pub fn run(word: &mut Vec<char>, steps: &[Step]) {
    for s in steps {
        apply(word, s);
    }
}

fn one(s: &str) -> char {
    let v = sounds(s);
    assert!(v.len() == 1, "a change starts from one sound, not `{s}`");
    v[0]
}

fn apply(w: &mut Vec<char>, step: &Step) {
    match step {
        Step::Change(from, to, at) => {
            let (from, to) = (one(from), sounds(to));
            let mut out = Vec::with_capacity(w.len() + 2);
            let mut seen_vowel = false;
            for (i, &c) in w.iter().enumerate() {
                let here = match at {
                    At::Anywhere => true,
                    At::Start => i == 0,
                    At::Inside => i > 0,
                    At::LaterVowels => seen_vowel,
                };
                if c == from && here {
                    out.extend_from_slice(&to);
                } else {
                    out.push(c);
                }
                seen_vowel |= is_vowel(c);
            }
            *w = out;
        }
        Step::EchoVowel => {
            if vowels(w) >= 3 && w.len() >= 4 && !is_vowel(w[0]) && is_vowel(w[1]) && w[2] == 'r' && w[3] == w[1] && !matches!(w[0], 'r' | 'n') {
                w.remove(1);
            }
        }
        Step::RColour => {
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 2);
            for (i, &c) in w.iter().enumerate() {
                let plain = match c {
                    'ṭ' => 't',
                    'ḍ' => 'd',
                    _ => {
                        out.push(c);
                        continue;
                    }
                };
                let r_close = out.iter().rev().take(3).any(|&x| x == 'r');
                if i == 0 {
                    out.extend([plain, 'r']);
                } else if r_close {
                    out.push(plain);
                } else {
                    out.extend(['r', plain]);
                }
            }
            *w = out;
        }
        Step::DropLastVowel => {
            if vowels(w) >= 2 && w.last().map(|c| is_vowel(*c)).unwrap_or(false) {
                let lost = w.pop().unwrap();
                if vowels(w) == 1 {
                    let i = w.iter().position(|c| is_vowel(*c)).unwrap();
                    if w[i] == 'a' && matches!(lost, 'i' | 'e') {
                        w[i] = 'e';
                    } else if w[i] == 'a' && matches!(lost, 'o' | 'u') {
                        w[i] = 'o';
                    }
                }
            }
        }
        Step::NoVowelPairs => {
            let mut out: Vec<char> = Vec::with_capacity(w.len());
            for &c in w.iter() {
                if is_vowel(c) && out.last().map(|p| is_vowel(*p)).unwrap_or(false) {
                    continue;
                }
                out.push(c);
            }
            *w = out;
        }
        Step::DropSecondVowel => {
            if vowels(w) < 3 {
                return;
            }
            let i = w.iter().enumerate().filter(|(_, c)| is_vowel(**c)).nth(1).unwrap().0;
            // One consonant either side, a vowel beyond each.
            if i < 2 || i + 2 >= w.len() {
                return;
            }
            let (a, b) = (w[i - 1], w[i + 1]);
            if !is_vowel(a) && !is_vowel(b) && is_vowel(w[i - 2]) && is_vowel(w[i + 2]) && can_meet(a, b) {
                w.remove(i);
            }
        }
        Step::NoDoubles => {
            let mut out: Vec<char> = Vec::with_capacity(w.len());
            for &c in w.iter() {
                if !is_vowel(c) && out.last() == Some(&c) {
                    continue;
                }
                out.push(c);
            }
            *w = out;
        }
        Step::Devoice => {
            for i in 0..w.len().saturating_sub(1) {
                if matches!(w[i + 1], 'p' | 't' | 'k' | 'q' | 'x') {
                    w[i] = match w[i] {
                        'd' => 't',
                        'g' => 'k',
                        c => c,
                    };
                }
            }
        }
        Step::YMelts => {
            let old = w.clone();
            let mut out = Vec::with_capacity(old.len());
            for (i, &c) in old.iter().enumerate() {
                let beside_i = (i > 0 && old[i - 1] == 'i') || (i + 1 < old.len() && old[i + 1] == 'i');
                if c == 'y' && beside_i {
                    continue;
                }
                out.push(c);
            }
            *w = out;
        }
        Step::HFades => {
            let old = w.clone();
            let mut out = Vec::with_capacity(old.len());
            for (i, &c) in old.iter().enumerate() {
                if c == 'h' && i > 0 && i + 1 < old.len() && old[i - 1] == old[i + 1] && is_vowel(old[i - 1]) {
                    continue;
                }
                out.push(c);
            }
            *w = out;
        }
        Step::VowelRun => {
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 1);
            let mut run = 0;
            for &c in w.iter() {
                if is_vowel(c) {
                    run += 1;
                    if run > 3 {
                        out.push(if matches!(out.last(), Some('o') | Some('u')) { 'w' } else { 'l' });
                        run = 1;
                    }
                } else {
                    run = 0;
                }
                out.push(c);
            }
            // Three of the same vowel is only a long one.
            let mut k = 0;
            while k + 2 < out.len() {
                if is_vowel(out[k]) && out[k] == out[k + 1] && out[k] == out[k + 2] {
                    out.remove(k);
                } else {
                    k += 1;
                }
            }
            *w = out;
        }
        Step::ShedLastVowel => {
            let n = w.len();
            if vowels(w) >= 3 && n >= 2 && is_vowel(w[n - 1]) && matches!(w[n - 2], 'h' | 'θ' | 's' | 'š') {
                w.pop();
            }
        }
        Step::OpenO => {
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 1);
            let mut first = true;
            for (i, &c) in w.iter().enumerate() {
                if is_vowel(c) {
                    if c == 'o' {
                        if first && i + 1 < w.len() && !is_vowel(w[i + 1]) {
                            out.extend(['a', 'u']);
                        } else {
                            out.push('e');
                        }
                    } else {
                        out.push(c);
                    }
                    first = false;
                } else {
                    out.push(c);
                }
            }
            *w = out;
        }
        Step::VowelsMeet => {
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 2);
            for &c in w.iter() {
                if let Some(&a) = out.last() {
                    if is_vowel(c) && is_vowel(a) && !matches!((a, c), ('a', 'i') | ('a', 'u') | ('e', 'i') | ('a', 'e')) {
                        out.push(if a == c {
                            'h'
                        } else if matches!(a, 'i' | 'e') {
                            'y'
                        } else {
                            'w'
                        });
                    }
                }
                out.push(c);
            }
            *w = out;
        }
        Step::BlurMiddle => {
            let at: Vec<usize> = w.iter().enumerate().filter(|(_, c)| is_vowel(**c)).map(|(i, _)| i).collect();
            for (k, &i) in at.iter().enumerate() {
                let alone = !(i + 1 < w.len() && is_vowel(w[i + 1])) && !(i > 0 && is_vowel(w[i - 1]));
                if k > 0 && k + 1 < at.len() && matches!(w[i], 'a' | 'o' | 'u') && alone {
                    w[i] = 'e';
                }
            }
        }
        Step::PartClusters => {
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 2);
            for &c in w.iter() {
                if !is_vowel(c) && out.last().map(|p| !is_vowel(*p)).unwrap_or(false) {
                    out.push('e');
                }
                out.push(c);
            }
            // A word may end on a breath, but not on a glide.
            if matches!(out.last(), Some('w') | Some('y') | Some('f')) {
                out.push('e');
            }
            *w = out;
        }
        Step::EndOnVowel => {
            // A word that ends on a consonant is given an echo of its last vowel.
            if w.last().map(|c| !is_vowel(*c)).unwrap_or(false) {
                let echo = w.iter().rev().find(|c| is_vowel(**c)).copied().unwrap_or('a');
                w.push(echo);
            }
        }
        Step::CatchVowels => {
            // Two vowels never touch: the catch goes between.
            let mut out: Vec<char> = Vec::with_capacity(w.len() + 2);
            for &c in w.iter() {
                if is_vowel(c) && out.last().map(|p| is_vowel(*p)).unwrap_or(false) {
                    out.push('ʻ');
                }
                out.push(c);
            }
            *w = out;
        }
    }
}

/// Whether two consonants can stand together inside a Qotiro word once the
/// vowel between them is lost.
fn can_meet(a: char, b: char) -> bool {
    if a == b {
        return false;
    }
    matches!(a, 'r' | 'n' | 'x') || b == 'r' || (matches!(a, 'p' | 't' | 'k' | 'q' | 'd' | 'g') && matches!(b, 't' | 'k'))
}
