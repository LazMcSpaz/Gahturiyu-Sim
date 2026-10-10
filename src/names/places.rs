//! Place names: towns and the land itself, named for what is really there.
//!
//! A place is handed in as the features of its site (a cove, a wooded
//! hillside, a pass) and, if it has one, a founding element (a resource, a
//! creature, an event, the founder, a god). Its name is two roots, the
//! first describing the second, picked from what such a place is called
//! and what is found there (`assets/lang/places.ron`). That one recipe is
//! then said in each of the four tongues by each tongue's own rules, and in
//! English: "stone" + "brow" is Stonebrow, and *Doqoṭiha* to the Roduro
//! who founded it.
//!
//! The name shown is the English one. The founders' form is the place's own
//! name; the other three are what the other peoples call it.

use std::sync::OnceLock;

use serde::Deserialize;

use super::grammar::{self, grammar};
use super::people::{self, Gender};
use super::{borrow, capital, familiar, lookalike, pronounce, syllables, things, unfortunate, Tongue};
use crate::sim::rng::Rng;

/// What the land is like at a place. Give the strongest first.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feature {
    Cove,
    Bay,
    Headland,
    Cliff,
    Shore,
    RiverMouth,
    Island,
    Stack,
    /// A rocky hillside.
    Hillside,
    Hill,
    Valley,
    Hollow,
    Ridge,
    Mountain,
    Peak,
    Pass,
    Plateau,
    Plain,
    Wood,
    Marsh,
    Heath,
    Spring,
    Stream,
}

/// What a place was founded on or for, if anything.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Founding {
    /// A resource, a creature or an event, by its root ("kelp", "hound", "fight").
    Root(String),
    /// Whoever founded it: a seed their name is drawn from.
    Founder(u64),
    /// A god it is given to, by id ("horahida").
    God(String),
}

/// A place's name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceName {
    /// The name the game shows: "Stonebrow".
    pub english: String,
    /// Whose place it is: their form is its own name.
    pub main: Tongue,
    /// The name in each tongue, in the order of `Tongue::SPOKEN`.
    pub native: [String; 4],
    /// What it says, word for word: "stone brow".
    pub meaning: String,
    /// How to say its own name.
    pub say: String,
}

impl PlaceName {
    /// The name as speakers of a tongue say it.
    pub fn in_tongue(&self, tongue: Tongue) -> &str {
        let k = Tongue::SPOKEN.iter().position(|t| *t == tongue).unwrap_or(0);
        &self.native[k]
    }
    /// The place's own name, in its founders' tongue.
    pub fn name(&self) -> &str {
        self.in_tongue(self.main)
    }
}

#[derive(Deserialize, Clone, Debug)]
struct FeatureWords {
    feature: Feature,
    heads: Vec<String>,
    marks: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct PeopleMarks {
    tongue: Tongue,
    marks: Vec<String>,
}

/// One sample place for the docs.
#[derive(Deserialize, Clone, Debug)]
pub struct Sample {
    pub what: String,
    pub features: Vec<Feature>,
    pub culture: Tongue,
    pub town: bool,
    #[serde(default)]
    pub founding: Option<Founding>,
    pub seed: u64,
}

#[derive(Deserialize, Clone, Debug)]
struct Data {
    features: Vec<FeatureWords>,
    steads: Vec<String>,
    shared: Vec<String>,
    peoples: Vec<PeopleMarks>,
    english: Vec<(String, String, Vec<String>)>,
    samples: Vec<Sample>,
}

fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| ron::from_str(include_str!("../../assets/lang/places.ron")).unwrap_or_else(|e| panic!("assets/lang/places.ron: {e}")))
}

/// The sample places (`samples` in `places.ron`).
pub fn samples() -> &'static [Sample] {
    &data().samples
}

fn words(f: Feature) -> &'static FeatureWords {
    data().features.iter().find(|w| w.feature == f).unwrap_or_else(|| panic!("assets/lang/places.ron has nothing for {f:?}"))
}

/// A root as the first part of an English place name: "Stone".
fn english_first(root: &str) -> String {
    match data().english.iter().find(|e| e.0 == root) {
        Some(e) => e.1.clone(),
        None => capital(&root.replace('_', " ")),
    }
}

/// A root as the last part: one of "dale", "vale". Which one comes first
/// goes by what it is joined to, never by chance, so the same two roots
/// always make the same English name (and two places can't share a native
/// name under two English ones).
fn english_last(root: &str, with: &str) -> Vec<String> {
    match data().english.iter().find(|e| e.0 == root) {
        Some(e) => {
            let k = (crate::sim::rng::key(&[with.bytes().fold(0u64, |h, b| h.wrapping_mul(131).wrapping_add(b as u64)), TAG_PLACE]) % e.2.len() as u64) as usize;
            (0..e.2.len()).map(|i| e.2[(k + i) % e.2.len()].clone()).collect()
        }
        None => vec![root.replace('_', " ")],
    }
}

/// Join the two parts of a town's English name: "Stone" + "brow".
/// `None` if every ending would double a letter across the join ("Marshham").
fn solid(first: &str, lasts: &[String]) -> Option<String> {
    let end = first.chars().last().map(|c| c.to_ascii_lowercase());
    lasts.iter().find(|l| l.chars().next() != end).map(|l| format!("{first}{l}"))
}

fn pick<'a>(rng: &mut Rng, items: &[(&'a str, f32)]) -> &'a str {
    let w: Vec<f32> = items.iter().map(|x| x.1).collect();
    items[rng.weighted(&w).unwrap_or(0)].0
}

const TAG_PLACE: u64 = 0x4E41_4D45_504C_4143;

fn every_tongue(f: impl Fn(Tongue) -> String) -> [String; 4] {
    Tongue::SPOKEN.map(f)
}

/// One try at a name. `None` if the pieces picked don't go together.
fn attempt(features: &[Feature], founding: Option<&Founding>, town: bool, culture: Tongue, rng: &mut Rng) -> Option<PlaceName> {
    let d = data();
    let primary = features.first().copied().unwrap_or(Feature::Plain);
    let second = features.get(1).copied();
    // What the place is called: a word for the land, or (a town, three times
    // in ten) for what people made of it. A god's place is named for the land.
    let land = &words(primary).heads;
    let stead = town && !matches!(founding, Some(Founding::God(_))) && rng.chance(0.3);
    let head: &str = if stead {
        &d.steads[rng.below(d.steads.len())]
    } else if town {
        &land[rng.below(land.len())]
    } else {
        // A piece of the land is mostly called what it plainly is (a stream is a "stream").
        let w: Vec<(&str, f32)> = land.iter().enumerate().map(|(k, h)| (h.as_str(), if k == 0 { 3.0 } else { 1.0 })).collect();
        pick(rng, &w)
    };

    // A founder or a god: "Doqu's Brow", and in each tongue "the brow of Doqu".
    let named: Option<(String, [String; 4])> = match founding {
        Some(Founding::Founder(seed)) => {
            let gender = if seed % 2 == 0 { Gender::Female } else { Gender::Male };
            let name = people::listed_name(culture, gender, *seed).name;
            // The founder's own people say the name as it is; the others bend it.
            Some((name.clone(), every_tongue(|t| if t == culture { name.clone() } else { borrow(&name, t) })))
        }
        Some(Founding::God(id)) => {
            let god = super::god(id)?;
            Some((god.name(Tongue::Roduro), every_tongue(|t| god.name(t))))
        }
        _ => None,
    };
    if let Some((shown, names)) = named {
        // Standing on its own, the word is the plain one ("House", not "ham").
        let last = english_first(head);
        let native = every_tongue(|t| {
            let k = Tongue::SPOKEN.iter().position(|x| *x == t).unwrap();
            grammar::of(t, &capital(&super::word(head, t).unwrap_or_default()), &names[k])
        });
        return Some(PlaceName { english: format!("{shown}'s {last}"), main: culture, say: pronounce(&native[Tongue::SPOKEN.iter().position(|x| *x == culture).unwrap()], culture), native, meaning: format!("{shown}'s {}", head.replace('_', " ")) });
    }

    // Otherwise a mark: the founding resource, creature or event; or what is found there.
    let mark: String = match founding {
        Some(Founding::Root(r)) => r.clone(),
        _ => {
            // What is found at such a place.
            let mut site: Vec<(&str, f32)> = words(primary).marks.iter().map(|m| (m.as_str(), 2.0)).collect();
            if let Some(s) = second {
                site.extend(words(s).marks.iter().map(|m| (m.as_str(), 2.0)));
            }
            if stead {
                // "What people made of it" says nothing of the land, so the
                // first part must: the land's own word ("Covestead"), or
                // something found there ("Kelpmeet").
                if rng.chance(0.55) {
                    land[rng.below(land.len())].clone()
                } else {
                    pick(rng, &site).to_string()
                }
            } else if let Some(s) = second.filter(|_| rng.chance(0.45)) {
                // The other thing the site is: a wooded cove is "Woodcove".
                words(s).heads[0].clone()
            } else {
                if let Some(p) = d.peoples.iter().find(|p| p.tongue == culture) {
                    site.extend(p.marks.iter().map(|m| (m.as_str(), if town { 1.2 } else { 0.6 })));
                }
                site.extend(d.shared.iter().map(|m| (m.as_str(), 0.6)));
                pick(rng, &site).to_string()
            }
        }
    };
    if mark == head {
        return None;
    }
    let recipe = format!("{mark}+{head}");
    let mut whole = true;
    let native = every_tongue(|t| things::build(&recipe, t, 0).map(|w| capital(&w)).unwrap_or_default());
    whole &= native.iter().all(|n| !n.is_empty());
    if !whole {
        return None;
    }
    let lasts = english_last(head, &mark);
    let english = if town { solid(&english_first(&mark), &lasts)? } else { format!("{} {}", english_first(&mark), english_first(head)) };
    let k = Tongue::SPOKEN.iter().position(|x| *x == culture).unwrap_or(0);
    Some(PlaceName { english, main: culture, say: pronounce(&native[k], culture), native, meaning: format!("{} {}", mark.replace('_', " "), head.replace('_', " ")) })
}

/// Whether a made name will do.
fn acceptable(p: &PlaceName, taken: &[PlaceName]) -> bool {
    // Not a place already named, nor a letter off one, in English or in any tongue.
    if taken.iter().any(|t| lookalike(&t.english, &p.english) || (0..4).any(|k| t.native[k] == p.native[k])) {
        return false;
    }
    if unfortunate(&p.english).is_some() || familiar(&p.english).is_some() {
        return false;
    }
    let e: Vec<char> = p.english.to_lowercase().chars().collect();
    if e.windows(3).any(|w| w[0] == w[1] && w[1] == w[2]) || e.len() > 16 {
        return false;
    }
    for (k, t) in Tongue::SPOKEN.iter().enumerate() {
        let n = &p.native[k];
        // Up to five beats a word; nothing rude; not already a word of the tongue.
        if syllables(n, *t) > 5 || unfortunate(n).is_some() {
            return false;
        }
        if !n.contains(' ') && people::is_word(*t, n) {
            return false;
        }
    }
    true
}

/// Name a place from its features, an optional founding element, and who
/// named it. `town`: a settled place (one English word, "Stonebrow") or a
/// piece of the land (two, "Gull Stack"). `taken`: the places already
/// named, which this one will not repeat (in English or in any tongue) or
/// come within a letter of. The same arguments always give the same name.
pub fn generate_place_with(features: &[Feature], founding: Option<&Founding>, town: bool, culture: Tongue, seed: u64, taken: &[PlaceName]) -> PlaceName {
    let mut last = None;
    for n in 0..40u64 {
        let mut rng = Rng::from_keys(&[seed, n, TAG_PLACE]);
        // After a good many tries, let go of the founding element rather than fail.
        let founding = if n < 24 { founding } else { None };
        if let Some(p) = attempt(features, founding, town, culture, &mut rng) {
            if acceptable(&p, taken) {
                return p;
            }
            last = Some(p);
        }
    }
    last.unwrap_or_else(|| panic!("no name could be made for {features:?}"))
}

/// Name a town from the features of its site: `generate_place(features, culture, seed)`.
pub fn generate_place(features: &[Feature], culture: Tongue, seed: u64) -> PlaceName {
    generate_place_with(features, None, true, culture, seed, &[])
}

/// Every root the place data uses (for the tests).
pub fn roots_used() -> Vec<&'static str> {
    let d = data();
    let mut out: Vec<&'static str> = Vec::new();
    for f in &d.features {
        out.extend(f.heads.iter().map(|x| x.as_str()));
        out.extend(f.marks.iter().map(|x| x.as_str()));
    }
    out.extend(d.steads.iter().map(|x| x.as_str()));
    out.extend(d.shared.iter().map(|x| x.as_str()));
    out.extend(d.peoples.iter().flat_map(|p| p.marks.iter().map(|x| x.as_str())));
    out.extend(d.english.iter().map(|e| e.0.as_str()));
    out
}

/// Every kind of feature.
pub const FEATURES: [Feature; 23] = [
    Feature::Cove,
    Feature::Bay,
    Feature::Headland,
    Feature::Cliff,
    Feature::Shore,
    Feature::RiverMouth,
    Feature::Island,
    Feature::Stack,
    Feature::Hillside,
    Feature::Hill,
    Feature::Valley,
    Feature::Hollow,
    Feature::Ridge,
    Feature::Mountain,
    Feature::Peak,
    Feature::Pass,
    Feature::Plateau,
    Feature::Plain,
    Feature::Wood,
    Feature::Marsh,
    Feature::Heath,
    Feature::Spring,
    Feature::Stream,
];

/// The sample places as Markdown: this is `docs/places.md`.
pub fn tables() -> String {
    let mut s = String::new();
    s.push_str("# Sample places\n\n");
    s.push_str("*Generated by `cargo run --release --bin lang -- places` from the `samples` in `assets/lang/places.ron`.*\n\n");
    s.push_str("The game shows the English name. The founders' form (in bold) is the place's own name; the other three are what the other peoples call it. A town is one English word; a piece of the land is two.\n\n");
    s.push_str("| What it is | Named by | Shown as | Say its own name | Roduro | Qotiro | Horaro | Ṭaḍoro | Word for word |\n|---|---|---|---|---|---|---|---|---|\n");
    let mut taken: Vec<PlaceName> = Vec::new();
    for x in samples() {
        let p = generate_place_with(&x.features, x.founding.as_ref(), x.town, x.culture, x.seed, &taken);
        let cells: Vec<String> = Tongue::SPOKEN.iter().map(|t| if *t == p.main { format!("**{}**", p.in_tongue(*t)) } else { p.in_tongue(*t).to_string() }).collect();
        s.push_str(&format!("| {} | {} | **{}** | {} | {} | {} | {} | {} | {} |\n", x.what, grammar(x.culture).tongue.name(), p.english, p.say, cells[0], cells[1], cells[2], cells[3], p.meaning));
        taken.push(p);
    }
    s
}
