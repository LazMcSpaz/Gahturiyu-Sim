//! Names in all four tongues.
//!
//! Every word starts as a root of the **First Speech** (`assets/lang/roots.ron`)
//! and becomes a Roduro, Qotiro, Horaro or Ṭaḍoro word by that tongue's sound
//! rules (`assets/lang/sounds.ron`, carried out by `sound.rs`). So the same
//! root shows up in every tongue, worn a different way: related words rhyme
//! faintly across the map, and a new word needs only a root.
//!
//! - `derive(first, tongue)`: a First Speech form run through a tongue's rules.
//! - `word(id, tongue)`: a root's word in a tongue, by its English key. This
//!   is the one to use: it also knows the exceptions (`irregular.ron`) and
//!   the borrowed words (`loans.ron`).
//!
//! - `grammar.rs`: putting words together (compounds, endings), tongue by tongue.
//! - `sacred.rs`: the gods, the elements, the words of worship and rule, and
//!   what each people calls the others, in every tongue.
//! - `say.rs`: plain-letter spelling and English pronunciation hints.
//!
//! Everything here is a plain function of its inputs: no state, no randomness.
//! (Stages 1 and 2 of the naming work. Things, people and places come next;
//! `docs/naming.md` keeps the running account.)

pub mod grammar;
pub mod sacred;
pub mod say;
pub mod sound;
pub mod things;

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

pub use grammar::{compound, grammar, join, with, Affix, Grammar};
pub use sacred::{god, make, people_english, people_meaning, people_name, sacred, God, Made, Sacred};
pub use say::{ascii, capital, file_name, pronounce, syllables};
pub use sound::{At, Step};
pub use things::{native_name, thing, things, Group, Thing};

/// The First Speech and its four daughters.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tongue {
    /// The reconstructed ancestor. Nobody speaks it.
    First,
    /// Gogìḍu, the Earth-kind's tongue.
    Roduro,
    /// The Fire-kind's tongue.
    Qotiro,
    /// The Water-kind's tongue.
    Horaro,
    /// The Air-kind's tongue.
    Tadoro,
}

impl Tongue {
    /// The four living tongues.
    pub const SPOKEN: [Tongue; 4] = [Tongue::Roduro, Tongue::Qotiro, Tongue::Horaro, Tongue::Tadoro];

    /// What the tongue is called here (the Roduro names for the peoples: the
    /// other three have no canon name for their own speech yet).
    pub fn name(self) -> &'static str {
        match self {
            Tongue::First => "First Speech",
            Tongue::Roduro => "Roduro",
            Tongue::Qotiro => "Qotiro",
            Tongue::Horaro => "Horaro",
            Tongue::Tadoro => "Ṭaḍoro",
        }
    }
}

/// The area of meaning a root belongs to.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Land,
    Sea,
    Sky,
    Colour,
    Plant,
    Animal,
    Material,
    Craft,
    Kin,
    Quality,
    Number,
    Feeling,
    Sacred,
}

/// One root of the First Speech.
#[derive(Deserialize, Clone, Debug)]
pub struct Root {
    /// The English key other data refers to.
    pub id: String,
    /// Its First Speech form.
    pub first: String,
    pub gloss: String,
    pub field: Field,
    /// The existing Roduro word it was worked backwards from, if any.
    #[serde(default)]
    pub canon: Option<String>,
}

/// The sound rules, as read from `sounds.ron`.
#[derive(Deserialize, Clone, Debug)]
pub struct Sounds {
    pub consonants: Vec<String>,
    pub vowels: Vec<String>,
    pub earth: Vec<Step>,
    pub roduro: Vec<Step>,
    pub qotiro: Vec<Step>,
    pub water: Vec<Step>,
    pub horaro: Vec<Step>,
    pub tadoro: Vec<Step>,
    pub tadoro_borrows: Vec<Step>,
}

/// A word that does not follow its tongue's rules, and why.
#[derive(Deserialize, Clone, Debug)]
pub struct Irregular {
    pub root: String,
    pub tongue: Tongue,
    pub form: String,
    pub why: String,
}

/// A word one tongue took from another instead of inheriting it.
#[derive(Deserialize, Clone, Debug)]
pub struct Loan {
    pub root: String,
    pub into: Tongue,
    pub from: Tongue,
    pub why: String,
}

/// Something already named before this system existed.
#[derive(Deserialize, Clone, Debug)]
pub struct Canon {
    pub word: String,
    /// The tongue it is in; `None` for an English name with no native word yet.
    #[serde(default)]
    pub tongue: Option<Tongue>,
    pub kind: Kind,
    /// What it means or is, where known.
    pub meaning: String,
    /// Where it comes from.
    pub source: String,
    /// The words it is built from, where that is recorded.
    #[serde(default)]
    pub parts: Vec<String>,
}

/// What sort of thing a canon entry names.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// An ordinary word of the language.
    Word,
    /// A bit of grammar: an article, a prefix, an ending.
    Grammar,
    /// Two or more words used together.
    Phrase,
    God,
    People,
    Tongue,
    Place,
    Creature,
    Material,
    Job,
    /// A made-up name given only as an example of how a tongue sounds.
    Sample,
    Other,
}

struct Data {
    sounds: Sounds,
    roots: Vec<Root>,
    by_id: HashMap<String, usize>,
    irregular: Vec<Irregular>,
    loans: Vec<Loan>,
    canon: Vec<Canon>,
    grammar: Vec<Grammar>,
    sacred: Sacred,
    avoid: Avoid,
}

/// Words a name must not look like in English (`assets/lang/avoid.ron`).
#[derive(Deserialize, Clone, Debug)]
struct Avoid {
    whole: Vec<String>,
    inside: Vec<String>,
}

fn read<T: for<'de> Deserialize<'de>>(file: &str, text: &str) -> T {
    ron::from_str(text).unwrap_or_else(|e| panic!("assets/lang/{file}: {e}"))
}

fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| {
        let roots: Vec<Root> = read("roots.ron", include_str!("../../assets/lang/roots.ron"));
        let by_id = roots.iter().enumerate().map(|(i, r)| (r.id.clone(), i)).collect();
        Data {
            sounds: read("sounds.ron", include_str!("../../assets/lang/sounds.ron")),
            roots,
            by_id,
            irregular: read("irregular.ron", include_str!("../../assets/lang/irregular.ron")),
            loans: read("loans.ron", include_str!("../../assets/lang/loans.ron")),
            canon: read("canon.ron", include_str!("../../assets/lang/canon.ron")),
            grammar: read("grammar.ron", include_str!("../../assets/lang/grammar.ron")),
            sacred: read("sacred.ron", include_str!("../../assets/lang/sacred.ron")),
            avoid: read("avoid.ron", include_str!("../../assets/lang/avoid.ron")),
        }
    })
}

/// The sound rules.
pub fn sounds() -> &'static Sounds {
    &data().sounds
}

/// Every root, in the order of the file.
pub fn roots() -> &'static [Root] {
    &data().roots
}

/// A root by its English key.
pub fn root(id: &str) -> Option<&'static Root> {
    data().by_id.get(id).map(|&i| &data().roots[i])
}

/// The words that break their tongue's rules.
pub fn irregulars() -> &'static [Irregular] {
    &data().irregular
}

/// The borrowed words.
pub fn loans() -> &'static [Loan] {
    &data().loans
}

/// Everything that was already named before this system.
pub fn canon() -> &'static [Canon] {
    &data().canon
}

/// A First Speech form run through a tongue's sound rules. This is the
/// rules alone: for a real word use `word`, which also knows the exceptions.
pub fn derive(first: &str, tongue: Tongue) -> String {
    let s = sounds();
    let mut w = sound::sounds(first);
    match tongue {
        Tongue::First => {}
        Tongue::Roduro => {
            sound::run(&mut w, &s.earth);
            sound::run(&mut w, &s.roduro);
        }
        Tongue::Qotiro => {
            sound::run(&mut w, &s.earth);
            sound::run(&mut w, &s.qotiro);
        }
        Tongue::Horaro => {
            sound::run(&mut w, &s.water);
            sound::run(&mut w, &s.horaro);
        }
        Tongue::Tadoro => {
            sound::run(&mut w, &s.water);
            sound::run(&mut w, &s.tadoro);
        }
    }
    sound::spell(&w)
}

/// A word of one tongue fitted to another's mouth, as when it is borrowed.
/// (Only Ṭaḍoro has borrowing rules so far; the others take a word as it is.)
pub fn borrow(word: &str, into: Tongue) -> String {
    let mut w = sound::sounds(word);
    if into == Tongue::Tadoro {
        sound::run(&mut w, &sounds().tadoro_borrows);
    }
    sound::spell(&w)
}

/// A root's word in a tongue: the exception if there is one, the borrowed
/// word if it is a loan, and otherwise the root by the sound rules.
pub fn word(id: &str, tongue: Tongue) -> Option<String> {
    let r = root(id)?;
    if let Some(x) = irregulars().iter().find(|x| x.root == id && x.tongue == tongue) {
        return Some(x.form.clone());
    }
    if let Some(l) = loans().iter().find(|l| l.root == id && l.into == tongue) {
        return Some(borrow(&word(id, l.from)?, tongue));
    }
    Some(derive(&r.first, tongue))
}

/// How a word came to be what it is in a tongue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Handed down from the First Speech by the sound rules.
    Inherited,
    /// An exception to the rules.
    Irregular,
    /// Borrowed from another tongue.
    Borrowed(Tongue),
}

pub fn origin(id: &str, tongue: Tongue) -> Origin {
    if irregulars().iter().any(|x| x.root == id && x.tongue == tongue) {
        Origin::Irregular
    } else if let Some(l) = loans().iter().find(|l| l.root == id && l.into == tongue) {
        Origin::Borrowed(l.from)
    } else {
        Origin::Inherited
    }
}

/// The rude English word a name looks like, if it does: read in plain
/// letters, as someone who doesn't know the tongue would. Each word of a
/// name of several is looked at on its own.
pub fn unfortunate(name: &str) -> Option<&'static str> {
    let a = &data().avoid;
    for w in name.split_whitespace() {
        let plain: String = ascii(&w.to_lowercase()).chars().filter(|c| c.is_ascii_alphabetic()).collect();
        if let Some(b) = a.whole.iter().find(|b| **b == plain) {
            return Some(b);
        }
        if let Some(b) = a.inside.iter().find(|b| plain.contains(b.as_str())) {
            return Some(b);
        }
    }
    None
}

/// Whether a First Speech form is well made: (C)V syllables from the First
/// Speech's own sounds, the catch only between vowels.
pub fn well_formed(first: &str) -> bool {
    let s = sounds();
    let cons: Vec<char> = s.consonants.iter().map(|c| sound::sounds(c)[0]).collect();
    let vows: Vec<char> = s.vowels.iter().map(|c| sound::sounds(c)[0]).collect();
    let w = sound::sounds(first);
    let mut i = 0;
    if w.is_empty() {
        return false;
    }
    while i < w.len() {
        if cons.contains(&w[i]) {
            if w[i] == 'ʻ' && i == 0 {
                return false;
            }
            i += 1;
        } else if i > 0 {
            return false;
        }
        if i >= w.len() || !vows.contains(&w[i]) {
            return false;
        }
        i += 1;
    }
    true
}

/// The cognate sets shown in `docs/naming.md`: one root in all four tongues.
pub const COGNATE_SETS: [&str; 20] = [
    "waterway", "earth", "sea", "fire", "wind", "air", "stone", "island", "sky", "moon", "night", "light", "tree", "fruit", "mother", "child", "house", "kind", "dream", "hunt",
];

/// The whole root list as a table to read: the root, its meaning, and its
/// word in each tongue. `docs/glossary.md` is this, and a test keeps the
/// file in step.
pub fn glossary() -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# Glossary: the First Speech roots in all four tongues\n");
    let _ = writeln!(out, "Made from `assets/lang/` by `cargo run --release --bin lang -- glossary` (a test checks this file is up to date). Don't edit it by hand: change the roots or the sound rules.\n");
    let _ = writeln!(out, "- **canon**: the Roduro word already existed, and the root was worked backwards from it.");
    let _ = writeln!(out, "- *italic*: an exception to the sound rules (`irregular.ron`).");
    let _ = writeln!(out, "- a word marked ‹R›, ‹Q› or ‹H›: borrowed from that tongue (`loans.ron`).\n");
    let mut field = None;
    for r in roots() {
        if field != Some(heading(r.field)) {
            field = Some(heading(r.field));
            let _ = writeln!(out, "\n## {}\n", heading(r.field));
            let _ = writeln!(out, "| Meaning | First Speech | Roduro | Qotiro | Horaro | Ṭaḍoro |");
            let _ = writeln!(out, "|---|---|---|---|---|---|");
        }
        let _ = write!(out, "| {}{} | {} |", r.gloss, if r.canon.is_some() { " (**canon**)" } else { "" }, r.first);
        for t in Tongue::SPOKEN {
            let w = word(&r.id, t).unwrap();
            let shown = match origin(&r.id, t) {
                Origin::Inherited => w,
                Origin::Irregular => format!("*{w}*"),
                Origin::Borrowed(from) => format!("{w} ‹{}›", from.name().chars().next().unwrap()),
            };
            let _ = write!(out, " {shown} |");
        }
        let _ = writeln!(out);
    }
    out
}

fn heading(f: Field) -> &'static str {
    match f {
        Field::Land | Field::Sea => "Land and sea",
        Field::Sky => "Weather and sky",
        Field::Colour => "Light, dark and colours",
        Field::Plant | Field::Animal => "Plants and animals",
        Field::Material => "Materials and made things",
        Field::Craft => "Crafts and actions",
        Field::Kin => "Kin and home",
        Field::Quality => "Qualities",
        Field::Number => "Numbers",
        Field::Feeling => "Virtues and feelings",
        Field::Sacred => "The sacred",
    }
}
