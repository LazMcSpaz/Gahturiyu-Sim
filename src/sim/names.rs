//! Name generators, one per race, following the sound signatures in the
//! project's `languages.md`.
//!
//! - Roduro: open (C)V syllables, deep consonants (ṭ ḍ q ʻ th), slow and even.
//! - Qotiro: clusters and hard closed endings (k q t x, trilled r), clipped.
//! - Horaro: liquid and vowel-heavy (l m n w h), long vowel runs, no hard stops.
//! - Ṭaḍoro: breathy (h s sh th f, w y), diphthongs, light and quick.
//!
//! A name is a pure function of a seed, so the same unborn person always gets
//! the same name, whenever and wherever the game first needs it.

use super::race::Race;
use super::rng::Rng;

pub fn person_name(race: Race, seed: u64) -> String {
    let mut rng = Rng::from_keys(&[seed, 0x4E41_4D45]);
    capitalize(&match race {
        Race::Roduro => roduro(&mut rng, 2, 4),
        Race::Qotiro => qotiro(&mut rng),
        Race::Horaro => horaro(&mut rng, 2, 4),
        Race::Tadoro => tadoro(&mut rng),
    })
}

pub fn place_name(race: Race, seed: u64) -> String {
    let mut rng = Rng::from_keys(&[seed, 0x504C_4143]);
    capitalize(&match race {
        Race::Roduro => roduro(&mut rng, 3, 4),
        Race::Qotiro => {
            let a = qotiro(&mut rng);
            if rng.chance(0.3) {
                a
            } else {
                format!("{}{}", a, qotiro_tail(&mut rng))
            }
        }
        Race::Horaro => horaro(&mut rng, 3, 4),
        Race::Tadoro => tadoro(&mut rng),
    })
}

fn weighted<'a>(rng: &mut Rng, items: &[(&'a str, f32)]) -> &'a str {
    let w: Vec<f32> = items.iter().map(|(_, w)| *w).collect();
    items[rng.weighted(&w).unwrap_or(0)].0
}

fn syllable_count(rng: &mut Rng, lo: usize, hi: usize) -> usize {
    // Favour the middle of the range.
    let span = hi - lo;
    lo + ((rng.below(span + 1) + rng.below(span + 1) + 1) / 2).min(span)
}

// ---- Roduro: open, heavy, archaic -----------------------------------------

const RODURO_C: &[(&str, f32)] = &[
    ("g", 3.0), ("d", 3.0), ("ḍ", 2.5), ("ṭ", 2.5), ("q", 2.5), ("h", 2.5),
    ("r", 3.0), ("l", 2.0), ("th", 1.5), ("y", 1.5), ("sh", 1.2), ("t", 1.5),
];
const RODURO_V: &[(&str, f32)] = &[("a", 3.0), ("e", 2.0), ("i", 2.0), ("ì", 2.0), ("o", 3.0), ("u", 2.5)];

fn roduro(rng: &mut Rng, lo: usize, hi: usize) -> String {
    let n = syllable_count(rng, lo, hi);
    let mut s = String::new();
    for i in 0..n {
        // The glottal catch only ever sits between vowels.
        if i > 0 && rng.chance(0.10) {
            s.push('ʻ');
        } else if i > 0 || rng.chance(0.9) {
            s.push_str(weighted(rng, RODURO_C));
        }
        s.push_str(weighted(rng, RODURO_V));
    }
    s
}

// ---- Qotiro: clustered, closed, forged -------------------------------------

const QOTIRO_ONSET: &[(&str, f32)] = &[
    ("k", 3.0), ("q", 2.5), ("t", 2.0), ("d", 2.5), ("g", 2.0), ("p", 1.0), ("x", 0.8),
    ("kr", 1.5), ("dr", 1.5), ("gr", 1.5), ("tr", 1.0), ("qr", 0.8), ("thr", 0.8), ("r", 1.2),
];
const QOTIRO_V: &[(&str, f32)] = &[("a", 3.0), ("o", 3.0), ("u", 3.0), ("e", 1.0), ("i", 0.5)];
const QOTIRO_CODA: &[(&str, f32)] = &[
    ("k", 3.0), ("t", 2.0), ("x", 1.5), ("rk", 1.5), ("rg", 1.2), ("rt", 1.0), ("xt", 1.2),
    ("kt", 1.0), ("n", 1.5), ("rn", 1.0), ("g", 1.2), ("q", 1.0), ("r", 1.0),
];

fn qotiro(rng: &mut Rng) -> String {
    let mut s = String::new();
    s.push_str(weighted(rng, QOTIRO_ONSET));
    s.push_str(weighted(rng, QOTIRO_V));
    if rng.chance(0.55) {
        // Two syllables: the first is usually closed too, hammer-forward.
        if rng.chance(0.6) {
            s.push_str(weighted(rng, &[("r", 2.0), ("g", 1.0), ("k", 1.0), ("x", 0.6), ("n", 1.0)]));
        }
        s.push_str(weighted(rng, &[("g", 2.0), ("d", 2.0), ("th", 1.5), ("k", 1.5), ("t", 1.0), ("q", 1.0)]));
        s.push_str(weighted(rng, QOTIRO_V));
    }
    s.push_str(weighted(rng, QOTIRO_CODA));
    s
}

fn qotiro_tail(rng: &mut Rng) -> String {
    weighted(rng, &[("dun", 1.0), ("gar", 1.0), ("thok", 1.0), ("kur", 1.0), ("rax", 0.7)]).to_string()
}

// ---- Horaro: liquid, vowel-heavy, murmuring --------------------------------

const HORARO_C: &[(&str, f32)] = &[("l", 3.0), ("m", 2.5), ("n", 2.5), ("w", 2.0), ("r", 1.5), ("h", 1.2)];
const HORARO_V: &[(&str, f32)] = &[
    ("a", 3.0), ("o", 2.5), ("u", 2.0), ("e", 1.8), ("i", 1.8),
    ("oa", 1.0), ("ea", 0.8), ("aa", 0.6), ("ia", 0.9), ("ai", 0.7), ("ua", 0.6),
];

fn horaro(rng: &mut Rng, lo: usize, hi: usize) -> String {
    let n = syllable_count(rng, lo, hi);
    let mut s = String::new();
    for i in 0..n {
        // Occasional vowel onsets let runs of vowels form ("Wenaia").
        if !(i > 0 && rng.chance(0.18)) && !(i == 0 && rng.chance(0.12)) {
            s.push_str(weighted(rng, HORARO_C));
        }
        s.push_str(weighted(rng, HORARO_V));
    }
    s
}

// ---- Ṭaḍoro: breathy, light, quick -----------------------------------------

const TADORO_C: &[(&str, f32)] = &[("h", 2.5), ("s", 2.5), ("sh", 1.5), ("th", 1.5), ("f", 2.0), ("w", 1.5), ("y", 1.5)];
const TADORO_V: &[(&str, f32)] = &[
    ("e", 2.5), ("a", 2.0), ("i", 1.5), ("u", 1.5),
    ("ai", 1.2), ("ae", 1.0), ("au", 0.8), ("ei", 0.8),
];

fn tadoro(rng: &mut Rng) -> String {
    let n = 2 + rng.below(2) * usize::from(rng.chance(0.4));
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(weighted(rng, TADORO_C));
        s.push_str(weighted(rng, TADORO_V));
    }
    if rng.chance(0.45) {
        s.push_str(weighted(rng, &[("h", 2.0), ("th", 1.5), ("s", 0.6)]));
    }
    s
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::ALL_RACES;

    #[test]
    fn names_are_stable_and_varied() {
        for race in ALL_RACES {
            assert_eq!(person_name(race, 42), person_name(race, 42));
            let mut set = std::collections::HashSet::new();
            for s in 0..200u64 {
                set.insert(person_name(race, s));
            }
            assert!(set.len() > 150, "{:?} produced only {} distinct names", race, set.len());
        }
    }

    #[test]
    fn signatures_hold() {
        for s in 0..300u64 {
            let q = person_name(Race::Qotiro, s);
            let last = q.chars().last().unwrap();
            assert!(!"aeiou".contains(last), "Qotiro names close on a consonant: {q}");
            let h = person_name(Race::Horaro, s).to_lowercase();
            assert!(!h.contains(['k', 'q', 't', 'p', 'ṭ', 'ḍ']), "Horaro has no hard stops: {h}");
            let r = person_name(Race::Roduro, s);
            assert!(!r.starts_with('ʻ'), "glottal never leads: {r}");
        }
    }
}
