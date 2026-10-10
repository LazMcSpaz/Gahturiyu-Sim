//! Spelling and saying: the plain-letters fallback for a word, and an
//! English pronunciation hint ("ho-rah-HIH-dah") for hover text.
//!
//! Each tongue has one spelling. Only the canon's own marks are used:
//! `ṭ ḍ` (t and d with the tongue curled back), `ì` (the i of "sit"), and
//! `ʻ` (the catch in "uh-oh"). `th` and `sh` are single sounds; Qotiro's
//! `x` is the ch of "loch"; `q` is a k made far back in the throat.

use super::grammar::{grammar, Stress};
use super::sound::{is_vowel, sounds};
use super::Tongue;

/// A word in plain letters, for fonts and file names that can't take the
/// marks: `ṭ ḍ ì` lose their marks and the catch becomes an apostrophe.
pub fn ascii(word: &str) -> String {
    word.chars()
        .map(|c| match c {
            'ṭ' => 't',
            'Ṭ' => 'T',
            'ḍ' => 'd',
            'Ḍ' => 'D',
            'ì' => 'i',
            'Ì' => 'I',
            'ʻ' => '\'',
            c => c,
        })
        .collect()
}

/// The same with nothing but letters, for file names.
pub fn file_name(word: &str) -> String {
    ascii(word).chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_lowercase()
}

/// A word with its first letter made a capital (names).
pub fn capital(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// One syllable: the consonants before the vowel, the vowel (or vowels that
/// are said as one), the consonants after.
#[derive(Debug, Default, Clone)]
struct Syllable {
    onset: Vec<char>,
    vowel: Vec<char>,
    coda: Vec<char>,
}

/// Split a word into syllables.
fn split(w: &[char], tongue: Tongue) -> Vec<Syllable> {
    // Vowels said as one: Ṭaḍoro's glides. (Horaro's vowels each get their
    // own beat, so a long vowel is simply said twice.)
    let one = |a: char, b: char| tongue == Tongue::Tadoro && matches!((a, b), ('a', 'i') | ('a', 'u') | ('e', 'i') | ('a', 'e'));
    let mut out: Vec<Syllable> = Vec::new();
    let mut waiting: Vec<char> = Vec::new();
    let mut i = 0;
    while i < w.len() {
        let c = w[i];
        if is_vowel(c) {
            let mut s = Syllable::default();
            // Consonants since the last vowel: one starts this syllable (two, a
            // stop and r, when three meet), the rest close the one before.
            let keep = if out.is_empty() {
                waiting.len()
            } else if waiting.len() >= 3 && waiting[waiting.len() - 1] == 'r' && matches!(waiting[waiting.len() - 2], 'p' | 't' | 'k' | 'q' | 'd' | 'g') {
                2
            } else {
                waiting.len().min(1)
            };
            let cut = waiting.len() - keep;
            if let Some(prev) = out.last_mut() {
                prev.coda.extend_from_slice(&waiting[..cut]);
            }
            s.onset = waiting[cut..].to_vec();
            waiting.clear();
            s.vowel.push(c);
            if i + 1 < w.len() && is_vowel(w[i + 1]) && one(c, w[i + 1]) {
                s.vowel.push(w[i + 1]);
                i += 1;
            }
            out.push(s);
        } else {
            waiting.push(c);
        }
        i += 1;
    }
    match out.last_mut() {
        Some(last) => last.coda.extend_from_slice(&waiting),
        None => out.push(Syllable { onset: waiting, ..Default::default() }),
    }
    out
}

/// How many beats a word has in a tongue (Ṭaḍoro's glides count as one;
/// each Horaro vowel is its own). For several words, the longest.
pub fn syllables(word: &str, tongue: Tongue) -> usize {
    word.split_whitespace().map(|w| split(&sounds(&w.to_lowercase()), tongue).len()).max().unwrap_or(0)
}

fn consonant(c: char) -> &'static str {
    match c {
        'q' => "k",
        'x' => "kh",
        'ṭ' => "t",
        'ḍ' => "d",
        'θ' => "th",
        'š' => "sh",
        // The catch is only a break between syllables.
        'ʻ' => "",
        'p' => "p",
        't' => "t",
        'k' => "k",
        'd' => "d",
        'g' => "g",
        'm' => "m",
        'n' => "n",
        's' => "s",
        'h' => "h",
        'r' => "r",
        'l' => "l",
        'w' => "w",
        'y' => "y",
        'f' => "f",
        _ => "",
    }
}

/// `bare`: nothing before the vowel in its syllable.
fn vowel(v: &[char], closed: bool, bare: bool) -> &'static str {
    match (v, closed) {
        // "eye" alone, but "sy", "wy" after a consonant ("seye" reads wrong).
        (['a', 'i'], _) if bare => "eye",
        (['a', 'i'], _) => "y",
        (['a', 'u'], _) => "ow",
        (['e', 'i'], _) => "ay",
        (['a', 'e'], _) => "ah-eh",
        (['a'], false) => "ah",
        (['a'], true) => "a",
        (['e'], false) => "eh",
        (['e'], true) => "e",
        (['i'], false) => "ee",
        (['i'], true) => "i",
        (['ì'], _) => "ih",
        (['o'], false) => "oh",
        (['o'], true) => "o",
        (['u'], _) => "oo",
        _ => "",
    }
}

/// How to say a word of a tongue, in plain English letters, with the
/// stressed syllable in capitals: `pronounce("horahìda", Roduro)` is
/// "hoh-rah-HIH-dah". A name of several words is said word by word.
pub fn pronounce(word: &str, tongue: Tongue) -> String {
    word.split_whitespace().map(|w| pronounce_one(w, tongue)).collect::<Vec<_>>().join(" ")
}

fn pronounce_one(word: &str, tongue: Tongue) -> String {
    let w = sounds(&word.to_lowercase());
    let syl = split(&w, tongue);
    let n = syl.len();
    let stress = match if tongue == Tongue::First { Stress::NextToLast } else { grammar(tongue).stress } {
        Stress::First => 0,
        Stress::NextToLast => n.saturating_sub(2),
        Stress::Last => n.saturating_sub(1),
    };
    let mut parts = Vec::with_capacity(n);
    for (k, s) in syl.iter().enumerate() {
        let mut p = String::new();
        for &c in &s.onset {
            p.push_str(consonant(c));
        }
        // The catch closes nothing: it is only the break before the next vowel.
        let closed = s.coda.iter().any(|c| *c != 'ʻ');
        // After a y the glide ai needs spelling out ("yigh", not "yy").
        if s.onset.last() == Some(&'y') && s.vowel == ['a', 'i'] {
            p.push_str("igh");
        } else {
            p.push_str(vowel(&s.vowel, closed, s.onset.iter().all(|c| *c == 'ʻ')));
        }
        for &c in &s.coda {
            p.push_str(consonant(c));
        }
        parts.push(if k == stress { p.to_uppercase() } else { p });
    }
    parts.retain(|p| !p.is_empty());
    parts.join("-")
}

/// Guess which tongue a name is in from its sounds alone: how likely each
/// tongue's own words are to use these sounds, and to end the way it ends.
/// (The measure is taken from the root list, so it moves with the rules.)
pub fn guess_tongue(name: &str) -> Tongue {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    // For each tongue: how often each sound turns up, each sound ends a word,
    // and each pair of neighbouring sounds turns up, in its own words.
    struct Model {
        sound: HashMap<char, f32>,
        last: HashMap<char, f32>,
        pair: HashMap<(char, char), f32>,
        total: f32,
        words: f32,
        pairs: f32,
    }
    static M: OnceLock<Vec<Model>> = OnceLock::new();
    let models = M.get_or_init(|| {
        Tongue::SPOKEN
            .iter()
            .map(|t| {
                let mut m = Model { sound: HashMap::new(), last: HashMap::new(), pair: HashMap::new(), total: 0.0, words: 0.0, pairs: 0.0 };
                for r in super::roots() {
                    let w = sounds(&super::word(&r.id, *t).unwrap_or_default());
                    for c in &w {
                        *m.sound.entry(*c).or_insert(0.0) += 1.0;
                        m.total += 1.0;
                    }
                    for p in w.windows(2) {
                        *m.pair.entry((p[0], p[1])).or_insert(0.0) += 1.0;
                        m.pairs += 1.0;
                    }
                    if let Some(c) = w.last() {
                        *m.last.entry(*c).or_insert(0.0) += 1.0;
                        m.words += 1.0;
                    }
                }
                m
            })
            .collect()
    });
    let mut best = (Tongue::Roduro, f32::NEG_INFINITY);
    for (t, m) in Tongue::SPOKEN.iter().zip(models.iter()) {
        let mut score = 0.0;
        for word in name.split_whitespace() {
            let w = sounds(&word.to_lowercase());
            // A sound the tongue never uses all but rules it out.
            for c in &w {
                score += ((m.sound.get(c).copied().unwrap_or(0.0) + 0.02) / (m.total + 1.0)).ln();
            }
            for p in w.windows(2) {
                score += 0.5 * ((m.pair.get(&(p[0], p[1])).copied().unwrap_or(0.0) + 0.5) / (m.pairs + 50.0)).ln();
            }
            if let Some(c) = w.last() {
                score += ((m.last.get(c).copied().unwrap_or(0.0) + 0.05) / (m.words + 1.0)).ln();
            }
        }
        if score > best.1 {
            best = (*t, score);
        }
    }
    best.0
}
