//! People's names: a given name with a meaning, and a byname in the shape
//! of the person's culture.
//!
//! **Given names** are native words. Each is built from one or two roots
//! (the same recipes things use: `stone`, `bright+stone`, `wave:small`) and
//! finished with one of its tongue's name endings for a man, a woman or
//! either (`assets/lang/grammar.ron`). So every name means something, and
//! the meaning is kept with it. Most people get a name from the hand-kept
//! lists (`assets/lang/names/`); some get one newly made from the roots
//! each people likes to name from (`assets/lang/people.ron`); and some carry
//! a name kept in their line.
//!
//! **Bynames** show in English, with the native form kept beside it:
//!
//! - Roduro: the house ("of the Ninth-Ring House"), and the job where there
//!   is one ("the Tender").
//! - Qotiro: a rank until the first deed ("Third Blade"), then the deed
//!   ("Iron-breaker"), replaced by each new one.
//! - Horaro: the mother's line ("of Wenaia's line"), or the sea-place they
//!   were born by ("of the Kelp Reef").
//! - Ṭaḍoro: the teacher ("Hesuth's student") or the road ("of the North
//!   Road"), changing as they travel.
//!
//! Everything is a plain function of the seed and the context handed in.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::Deserialize;

use super::grammar::grammar;
use super::sound::{self, is_vowel, Step};
use super::{capital, familiar, lookalike, pronounce, syllables, things, unfortunate, Field, Tongue};
use crate::sim::rng::{self, Rng};

/// Who a given name is for.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gender {
    Male,
    Female,
    /// A name anyone may carry; also a person who is neither.
    Either,
}

impl Gender {
    pub const ALL: [Gender; 3] = [Gender::Male, Gender::Female, Gender::Either];

    pub fn word(self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Either => "either",
        }
    }
    fn they(self) -> &'static str {
        match self {
            Gender::Male => "he",
            Gender::Female => "she",
            Gender::Either => "they",
        }
    }
    fn their(self) -> &'static str {
        match self {
            Gender::Male => "his",
            Gender::Female => "her",
            Gender::Either => "their",
        }
    }
    fn them(self) -> &'static str {
        match self {
            Gender::Male => "him",
            Gender::Female => "her",
            Gender::Either => "them",
        }
    }
    /// "is" or "are", to go with `they()`.
    fn is(self) -> &'static str {
        if self == Gender::Either {
            "are"
        } else {
            "is"
        }
    }
}

// ---- The data -------------------------------------------------------------------

#[derive(Deserialize, Clone, Debug)]
struct Pool {
    heads: Vec<String>,
    firsts: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct PeoplePool {
    tongue: Tongue,
    heads: Vec<String>,
    firsts: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Houses {
    rings: Vec<String>,
    marks: Vec<String>,
    sites: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Ranks {
    counts: Vec<String>,
    arms: Vec<(String, String)>,
}

#[derive(Deserialize, Clone, Debug)]
struct SeaPlaces {
    marks: Vec<String>,
    sites: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Data {
    means: Vec<(String, String)>,
    means_first: Vec<(String, String)>,
    men: Vec<String>,
    women: Vec<String>,
    shared: Pool,
    plain: Vec<String>,
    peoples: Vec<PeoplePool>,
    short_jobs: Vec<(String, String)>,
    no_byname: Vec<String>,
    houses: Houses,
    deeds: Vec<(String, String, String)>,
    ranks: Ranks,
    sea_places: SeaPlaces,
    roads: Vec<String>,
}

/// One hand-kept given name (`assets/lang/names/<tongue>.ron`).
#[derive(Deserialize, Clone, Debug)]
pub struct Listed {
    /// The name as the rules make it from `made` and `end` (a test holds the two together).
    pub name: String,
    pub gender: Gender,
    /// The recipe: `stone`, `bright+stone`, `wave:small`.
    pub made: String,
    /// The name ending put on it.
    pub end: String,
}

struct Loaded {
    data: Data,
    lists: [Vec<Listed>; 4],
    /// Each tongue's root words, with the root each belongs to.
    root_words: [std::collections::HashMap<String, Vec<String>>; 4],
    /// Each tongue's other words: built things, gods, the peoples.
    words: [HashSet<String>; 4],
}

fn slot(t: Tongue) -> usize {
    match t {
        Tongue::Roduro | Tongue::First => 0,
        Tongue::Qotiro => 1,
        Tongue::Horaro => 2,
        Tongue::Tadoro => 3,
    }
}

fn loaded() -> &'static Loaded {
    static L: OnceLock<Loaded> = OnceLock::new();
    L.get_or_init(|| {
        let read_list = |file: &str, text: &str| -> Vec<Listed> { ron::from_str(text).unwrap_or_else(|e| panic!("assets/lang/names/{file}: {e}")) };
        let data: Data = ron::from_str(include_str!("../../assets/lang/people.ron")).unwrap_or_else(|e| panic!("assets/lang/people.ron: {e}"));
        let lists = [
            read_list("roduro.ron", include_str!("../../assets/lang/names/roduro.ron")),
            read_list("qotiro.ron", include_str!("../../assets/lang/names/qotiro.ron")),
            read_list("horaro.ron", include_str!("../../assets/lang/names/horaro.ron")),
            read_list("tadoro.ron", include_str!("../../assets/lang/names/tadoro.ron")),
        ];
        // Every word a tongue already has: a name must not be one of them
        // (but may be its own root's word: a child can be called "Stone").
        let root_words = Tongue::SPOKEN.map(|t| {
            let mut w: std::collections::HashMap<String, Vec<String>> = Default::default();
            for r in super::roots() {
                if let Some(x) = super::word(&r.id, t) {
                    w.entry(x).or_default().push(r.id.clone());
                }
            }
            w
        });
        let words = Tongue::SPOKEN.map(|t| {
            let mut w: HashSet<String> = HashSet::new();
            for th in things::things().iter().filter(|th| super::root(&th.made).is_none()) {
                w.extend(th.in_tongue(t).split(' ').map(|x| x.to_lowercase()));
            }
            w.extend(super::sacred().gods.iter().map(|g| g.name(t).to_lowercase()));
            w.extend(super::sacred().terms.iter().filter_map(|x| super::make(&x.made, t)).map(|x| x.to_lowercase()));
            w.extend(Tongue::SPOKEN.iter().filter_map(|of| super::people_name(*of, t)).map(|x| x.to_lowercase()));
            w
        });
        Loaded { data, lists, root_words, words }
    })
}

/// The hand-kept given names of a tongue.
pub fn listed(tongue: Tongue) -> &'static [Listed] {
    &loaded().lists[slot(tongue)]
}

/// The endings a given name may take in a tongue.
pub fn endings(tongue: Tongue, gender: Gender) -> &'static [String] {
    let g = grammar(tongue);
    match gender {
        Gender::Male => &g.male,
        Gender::Female => &g.female,
        Gender::Either => &g.either,
    }
}

// ---- Making a given name ----------------------------------------------------------

/// Put a name ending on a finished word.
///
/// - Roduro and Horaro names end on a vowel: the word's last vowel gives
///   way to the ending.
/// - Qotiro names end on a consonant: a word that already ends on it stands
///   as it is; otherwise the ending follows an echo of the word's last vowel.
/// - Ṭaḍoro: a vowel ending takes the place of the word's last vowel (or
///   glide); a breath ending follows it.
pub fn finish(tongue: Tongue, base: &str, ending: &str) -> String {
    let mut w = sound::sounds(base);
    let end = sound::sounds(ending);
    if w.is_empty() || end.is_empty() {
        return base.to_string();
    }
    let vowel_end = is_vowel(end[0]);
    match tongue {
        Tongue::Roduro | Tongue::First | Tongue::Horaro => {
            if is_vowel(*w.last().unwrap()) {
                w.pop();
            }
            w.extend_from_slice(&end);
            if tongue == Tongue::Horaro {
                sound::run(&mut w, &[Step::VowelRun]);
            }
        }
        Tongue::Qotiro => {
            if w.ends_with(&end) {
                // As it is.
            } else if is_vowel(*w.last().unwrap()) {
                w.extend_from_slice(&end);
            } else {
                let echo = w.iter().rev().find(|c| is_vowel(**c)).copied().unwrap_or('a');
                w.push(echo);
                w.extend_from_slice(&end);
            }
        }
        Tongue::Tadoro => {
            if vowel_end {
                // The last vowel, or the glide it ends, gives way.
                let mut cut = 0;
                while cut < 2 && w.len() > 1 && is_vowel(*w.last().unwrap()) {
                    w.pop();
                    cut += 1;
                }
                w.extend_from_slice(&end);
                sound::run(&mut w, &[Step::VowelsMeet]);
            } else {
                if !is_vowel(*w.last().unwrap()) {
                    w.pop();
                }
                w.extend_from_slice(&end);
            }
        }
    }
    sound::spell(&w)
}

/// A given name from its recipe and ending, in small letters.
pub fn given(tongue: Tongue, made: &str, ending: &str) -> Option<String> {
    let base = things::build(made, tongue, 0).ok()?;
    if base.contains(' ') {
        return None;
    }
    Some(finish(tongue, &base, ending))
}

fn root_means(id: &str, describing: bool) -> String {
    let d = &loaded().data;
    let over = if describing { d.means_first.iter().find(|m| m.0 == id) } else { None };
    match over.or_else(|| d.means.iter().find(|m| m.0 == id)) {
        Some(m) => m.1.clone(),
        None => id.replace('_', " "),
    }
}

/// What a name's recipe says: "bright stone", "little wave".
pub fn meaning(made: &str) -> String {
    let (body, affix) = match made.rsplit_once(':') {
        Some((b, a)) => (b, Some(a)),
        None => (made, None),
    };
    let parts: Vec<&str> = body.split('+').map(|p| p.trim()).collect();
    let said = parts.iter().enumerate().map(|(k, p)| root_means(p, k + 1 < parts.len())).collect::<Vec<_>>().join(" ");
    match affix {
        Some("small") => format!("little {said}"),
        Some("great") => format!("great {said}"),
        _ => said,
    }
}

/// The roots of a recipe.
fn roots_of(made: &str) -> Vec<&str> {
    made.split(':').next().unwrap_or("").split('+').map(|p| p.trim()).collect()
}

/// How long a given name may be, in beats.
fn beats(tongue: Tongue) -> (usize, usize) {
    match tongue {
        Tongue::Qotiro => (1, 3),
        _ => (2, 4),
    }
}

/// Whether a made name will do: the right length, not already a word of
/// its tongue, nothing an English reader would wince at or take for a
/// well-known name, and no stammer. `Err` says why not.
pub fn acceptable(tongue: Tongue, name: &str, made: &str) -> Result<(), String> {
    let n = syllables(name, tongue);
    let (least, most) = beats(tongue);
    if n < least || n > most {
        return Err(format!("{n} beats"));
    }
    let letters = sound::sounds(name);
    if letters.len() < 3 || letters.len() > 10 {
        return Err(format!("{} sounds", letters.len()));
    }
    if let Some(b) = unfortunate(name) {
        return Err(format!("reads as `{b}`"));
    }
    if let Some(k) = familiar(name) {
        return Err(format!("too like `{k}`"));
    }
    // Its own root's word is fine ("Stone"); any other word is not.
    let l = loaded();
    if l.words[slot(tongue)].contains(name) {
        return Err("already a word".to_string());
    }
    if let Some(ids) = l.root_words[slot(tongue)].get(name) {
        if !(ids.len() == 1 && ids[0] == made) {
            return Err("already a word".to_string());
        }
    }
    // No stammer: the same letter three times, or the same beat three times.
    if letters.windows(3).any(|w| w[0] == w[1] && w[1] == w[2]) {
        return Err("a letter three times".to_string());
    }
    if letters.len() >= 6 && letters.windows(6).any(|w| w[0..2] == w[2..4] && w[2..4] == w[4..6]) {
        return Err("a beat three times".to_string());
    }
    // ... or three beats running that start on the same sound ("yiyiyai"),
    // or a name that is one half said twice ("gorgor").
    let onsets: Vec<char> = letters.iter().enumerate().filter(|(i, c)| !is_vowel(**c) && letters.get(i + 1).map(|n| is_vowel(*n)).unwrap_or(false)).map(|(_, c)| *c).collect();
    if onsets.windows(3).any(|w| w[0] == w[1] && w[1] == w[2]) {
        return Err("three beats on one sound".to_string());
    }
    if letters.len() >= 4 && letters.len() % 2 == 0 && letters[..letters.len() / 2] == letters[letters.len() / 2..] {
        return Err("one half said twice".to_string());
    }
    Ok(())
}

/// The roots a people names from: (id, how often). Their own favourites
/// count double.
fn pool(tongue: Tongue, heads: bool) -> Vec<(&'static str, f32)> {
    let d = &loaded().data;
    let mut out: Vec<(&'static str, f32)> = Vec::new();
    let own = d.peoples.iter().find(|p| p.tongue == tongue);
    if let Some(p) = own {
        out.extend((if heads { &p.heads } else { &p.firsts }).iter().map(|x| (x.as_str(), 2.0)));
    }
    for x in if heads { &d.shared.heads } else { &d.shared.firsts } {
        if !out.iter().any(|o| o.0 == x.as_str()) {
            out.push((x.as_str(), 1.0));
        }
    }
    out
}

const TIMES: [&str; 13] = ["dawn", "dusk", "night", "day", "winter", "summer", "moon", "star", "sun", "rain", "snow", "frost", "storm"];
const ELEMENTS: [&str; 6] = ["earth", "fire", "water", "air", "wind", "sea"];

/// Whether `first` may describe `head` in a name.
fn goes_with(first: &str, head: &str) -> bool {
    if first == head {
        return false;
    }
    // Not two times of day or year, nor two elements, in one name.
    if (TIMES.contains(&first) && TIMES.contains(&head)) || (ELEMENTS.contains(&first) && ELEMENTS.contains(&head)) {
        return false;
    }
    let field = |id: &str| super::root(id).map(|r| r.field);
    // A direction describes only a piece of the land, sea or sky ("north star", not "south father").
    if matches!(first, "north" | "south" | "east" | "west") {
        return matches!(field(head), Some(Field::Land | Field::Sea | Field::Sky));
    }
    // One virtue to a name.
    if field(first) == Some(Field::Feeling) && field(head) == Some(Field::Feeling) {
        return false;
    }
    // A colour (or a taste, a weight) describes only what can be seen and touched.
    if loaded().data.plain.iter().any(|p| p == first) {
        return matches!(field(head), Some(Field::Land | Field::Sea | Field::Sky | Field::Plant | Field::Animal | Field::Material));
    }
    true
}

/// Whether a recipe may name someone of this gender ("father" names no daughter).
fn suits(made: &str, gender: Gender) -> bool {
    let d = &loaded().data;
    let has = |list: &[String]| roots_of(made).iter().any(|r| list.iter().any(|x| x == r));
    match gender {
        Gender::Male => !has(&d.women),
        Gender::Female => !has(&d.men),
        Gender::Either => !has(&d.men) && !has(&d.women),
    }
}

/// A name that could be made, for the lists (`lang name-candidates`).
#[derive(Clone, Debug)]
pub struct Candidate {
    pub name: String,
    pub gender: Gender,
    pub made: String,
    pub end: String,
}

/// Every name the roots a people names from could make, and that will do.
pub fn candidates(tongue: Tongue) -> Vec<Candidate> {
    let heads = pool(tongue, true);
    let firsts = pool(tongue, false);
    let mut recipes: Vec<String> = Vec::new();
    for (h, _) in &heads {
        recipes.push(h.to_string());
        recipes.push(format!("{h}:small"));
        recipes.push(format!("{h}:great"));
    }
    for (f, _) in &firsts {
        if !heads.iter().any(|h| h.0 == *f) {
            recipes.push(f.to_string());
        }
        for (h, _) in &heads {
            if goes_with(f, h) {
                recipes.push(format!("{f}+{h}"));
            }
        }
    }
    let mut out = Vec::new();
    for made in recipes {
        for gender in Gender::ALL.into_iter().filter(|g| suits(&made, *g)) {
            for end in endings(tongue, gender) {
                if let Some(name) = given(tongue, &made, end) {
                    if acceptable(tongue, &name, &made).is_ok() {
                        out.push(Candidate { name, gender, made: made.clone(), end: end.clone() });
                    }
                }
            }
        }
    }
    out
}

/// A given name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Given {
    /// The native name, with its capital.
    pub name: String,
    /// What it means: "bright stone".
    pub meaning: String,
    /// Its recipe.
    pub made: String,
    /// Whether it is one of the hand-kept names (otherwise newly made).
    pub listed: bool,
}

fn pick<'a>(rng: &mut Rng, items: &[(&'a str, f32)]) -> &'a str {
    let w: Vec<f32> = items.iter().map(|x| x.1).collect();
    items[rng.weighted(&w).unwrap_or(0)].0
}

/// A newly made name from a people's roots, if a good one turns up.
fn fresh(tongue: Tongue, gender: Gender, rng: &mut Rng) -> Option<Given> {
    let heads = pool(tongue, true);
    let firsts = pool(tongue, false);
    let ends = endings(tongue, gender);
    for _ in 0..16 {
        let head = pick(rng, &heads);
        let form = rng.f32();
        let made = if form < 0.2 {
            head.to_string()
        } else if form < 0.3 {
            format!("{head}:{}", if rng.chance(0.5) { "small" } else { "great" })
        } else {
            let first = pick(rng, &firsts);
            if !goes_with(first, head) {
                continue;
            }
            format!("{first}+{head}")
        };
        if !suits(&made, gender) {
            continue;
        }
        let end = &ends[rng.below(ends.len())];
        let Some(name) = given(tongue, &made, end) else { continue };
        if acceptable(tongue, &name, &made).is_err() {
            continue;
        }
        // Never one letter off a name on the lists.
        if listed(tongue).iter().any(|l| l.name != name && lookalike(&l.name, &name)) {
            continue;
        }
        return Some(Given { name: capital(&name), meaning: meaning(&made), made, listed: false });
    }
    None
}

const TAG_GIVEN: u64 = 0x4E41_4D45_4749_5645;
const TAG_LINE: u64 = 0x4E41_4D45_4C49_4E45;
const TAG_HOUSE: u64 = 0x4E41_4D45_484F_5553;
const TAG_DEED: u64 = 0x4E41_4D45_4445_4544;
const TAG_SEA: u64 = 0x4E41_4D45_5345_4120;
const TAG_ROAD: u64 = 0x4E41_4D45_524F_4144;
const TAG_KEPT: u64 = 0x4E41_4D45_4B45_5054;

/// How often a name is newly made rather than taken from the lists.
const FRESH: f32 = 0.2;
/// How often a child is given a name kept in the line.
const KEPT: f32 = 0.2;
/// How many names a line keeps for each of sons and daughters.
const KEPT_NAMES: usize = 2;

fn from_list(tongue: Tongue, gender: Gender, rng: &mut Rng) -> Given {
    let all = listed(tongue);
    // A man's name is a man's or (now and then) one for either; and so on.
    let want = if gender != Gender::Either && rng.chance(0.12) { Gender::Either } else { gender };
    let of: Vec<&Listed> = all.iter().filter(|l| l.gender == want).collect();
    let of = if of.is_empty() { all.iter().collect() } else { of };
    assert!(!of.is_empty(), "assets/lang/names has no names for {tongue:?}");
    let l = of[rng.below(of.len())];
    Given { name: capital(&l.name), meaning: meaning(&l.made), made: l.made.clone(), listed: true }
}

/// A given name for someone of a people: the same seed, the same name.
pub fn given_name(tongue: Tongue, gender: Gender, seed: u64) -> Given {
    let mut rng = Rng::from_keys(&[seed, TAG_GIVEN]);
    if rng.chance(FRESH) {
        if let Some(g) = fresh(tongue, gender, &mut rng) {
            return g;
        }
    }
    from_list(tongue, gender, &mut rng)
}

/// A name off the lists only (the founders of lines and houses, teachers:
/// people only ever spoken of).
fn listed_name(tongue: Tongue, gender: Gender, seed: u64) -> Given {
    from_list(tongue, gender, &mut Rng::from_keys(&[seed, TAG_GIVEN, 1]))
}

// ---- Bynames ----------------------------------------------------------------------

/// What is known of a person that their name can draw on. Leave out what
/// isn't known. People who share a `lineage` share a line (and sometimes a
/// name); people who share a `home` share a house.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// The line they were born into (for Horaro, the mother's).
    pub lineage: Option<u64>,
    /// The home they live in. A new home (marriage, a home grown for them)
    /// is a new house name.
    pub home: Option<u64>,
    /// Their job, by its English name ("Stone Tender").
    pub job: Option<String>,
    /// Where they were born.
    pub birthplace: Option<u64>,
    /// How many times their life has turned: a deed done (Qotiro), a new
    /// teacher or a new road (Ṭaḍoro). Each turn is a new byname.
    pub turns: u32,
}

/// A person's whole name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonName {
    pub tongue: Tongue,
    pub gender: Gender,
    /// The given name: native, with its capital.
    pub given: String,
    /// How to say it.
    pub say: String,
    /// What it means.
    pub meaning: String,
    /// The byname as the game shows it: "of the Ninth-Ring House".
    pub byname: String,
    /// The same in their own tongue, for hover.
    pub byname_native: String,
    /// A job byname, English and native: ("the Tender", "Leḍaqe").
    pub title: Option<(String, String)>,
    /// What the name hints at, in a sentence or two.
    pub story: String,
    /// What joins the given name and the byname (" " or ", ").
    join: &'static str,
}

impl PersonName {
    /// The name as shown: "Ḍalìqa the Tender, of the Ninth-Ring House".
    pub fn full(&self) -> String {
        match &self.title {
            Some((t, _)) => format!("{} {t}, {}", self.given, self.byname),
            None => format!("{}{}{}", self.given, self.join, self.byname),
        }
    }
    /// The name wholly in their own tongue.
    pub fn full_native(&self) -> String {
        match &self.title {
            Some((_, n)) => format!("{} {n}, {}", self.given, self.byname_native),
            None => format!("{} {}", self.given, self.byname_native),
        }
    }
}

fn title_case(id: &str) -> String {
    capital(&id.replace('_', " "))
}

fn ordinal(count: &str) -> &'static str {
    match count {
        "one" => "First",
        "two" => "Second",
        "three" => "Third",
        "four" => "Fourth",
        "five" => "Fifth",
        "six" => "Sixth",
        "seven" => "Seventh",
        "eight" => "Eighth",
        "nine" => "Ninth",
        "ten" => "Tenth",
        "hundred" => "Hundred",
        _ => "Many",
    }
}

fn built(recipe: &str, tongue: Tongue) -> String {
    things::build(recipe, tongue, 0).unwrap_or_else(|e| panic!("assets/lang/people.ron, `{recipe}`: {e}"))
}

struct By {
    english: String,
    native: String,
    story: String,
    join: &'static str,
}

/// Roduro: the house. A grown home shows its age in its rings, so an old
/// house is named for the count; others for what marks the place, or for
/// whoever it was first grown for.
fn house(home: u64, who: Gender) -> By {
    let t = Tongue::Roduro;
    let h = &loaded().data.houses;
    let of = &grammar(t).of;
    let mut rng = Rng::from_keys(&[home, TAG_HOUSE]);
    let kind = rng.f32();
    if kind < 0.3 {
        let count = &h.rings[rng.below(h.rings.len())];
        let age = match count.as_str() {
            "three" | "four" => "a young house",
            "five" | "six" => "a house of middling age",
            "hundred" => "a house older than anyone can reckon",
            _ => "an old house",
        };
        By {
            english: format!("of the {}-Ring House", ordinal(count)),
            native: format!("{of} {}", capital(&built(&format!("{count}+ring"), t))),
            story: format!("{} house is named for the rings in its stone: {age}.", capital(who.their())),
            join: " ",
        }
    } else if kind < 0.75 {
        let mark = &h.marks[rng.below(h.marks.len())];
        let site = &h.sites[rng.below(h.sites.len())];
        By {
            english: format!("of the {}-{} House", title_case(mark), title_case(site)),
            native: format!("{of} {}", capital(&built(&format!("{mark}+{site}"), t))),
            story: format!("{} house is named for where it grew: the {site}, and the {mark} there.", capital(who.their())),
            join: " ",
        }
    } else {
        let founder = listed_name(t, if rng.chance(0.5) { Gender::Male } else { Gender::Female }, rng::key(&[home, TAG_HOUSE, 1]));
        By {
            english: format!("of {}'s House", founder.name),
            native: format!("{of} {} {of} {}", super::word("house", t).unwrap(), founder.name),
            story: format!("{} house still carries the name of {}, for whom it was first grown.", capital(who.their()), founder.name),
            join: " ",
        }
    }
}

/// A job as a byname: ("the Tender", its native word).
fn job_title(job: &str, tongue: Tongue) -> Option<(String, String)> {
    let d = &loaded().data;
    let th = things::thing(job)?;
    if d.no_byname.iter().any(|n| n.eq_ignore_ascii_case(job)) {
        return None;
    }
    let short = d.short_jobs.iter().find(|s| s.0.eq_ignore_ascii_case(job)).map(|s| s.1.clone()).unwrap_or_else(|| th.english.clone());
    Some((format!("the {short}"), capital(&th.native(tongue).1)))
}

/// Qotiro: a place in the ranks until the first deed, then the deed.
fn deed(seed: u64, turns: u32, who: Gender) -> By {
    let t = Tongue::Qotiro;
    let d = &loaded().data;
    if turns == 0 {
        let mut rng = Rng::from_keys(&[seed, TAG_DEED]);
        let count = &d.ranks.counts[rng.below(d.ranks.counts.len())];
        let arm = &d.ranks.arms[rng.below(d.ranks.arms.len())];
        By {
            english: format!("{} {}", ordinal(count), arm.1),
            native: capital(&built(&format!("{count}+{}", arm.0), t)),
            story: format!("No deed yet: {} {} known by {} place in the ranks.", who.they(), who.is(), who.their()),
            join: " ",
        }
    } else {
        let mut rng = Rng::from_keys(&[seed, turns as u64, TAG_DEED]);
        let deed = &d.deeds[rng.below(d.deeds.len())];
        By {
            english: deed.2.clone(),
            native: capital(&built(&format!("{}+{}:agent", deed.0, deed.1), t)),
            story: format!("{} earned the name by a deed, and the next deed will replace it.", capital(who.they())),
            join: " ",
        }
    }
}

/// Horaro: the mother's line, or the sea-place they were born by.
fn sea_line(seed: u64, ctx: &Context, who: Gender) -> By {
    let t = Tongue::Horaro;
    let s = &loaded().data.sea_places;
    let of = &grammar(t).of;
    let mut rng = Rng::from_keys(&[seed, TAG_SEA]);
    let by_line = match (ctx.lineage, ctx.birthplace) {
        (Some(_), Some(_)) => rng.chance(0.7),
        (Some(_), None) => true,
        _ => false,
    };
    if by_line {
        let mother = listed_name(t, Gender::Female, rng::key(&[ctx.lineage.unwrap(), TAG_LINE]));
        By {
            english: format!("of {}'s line", mother.name),
            native: format!("{of} {} {of} {}", super::word("line", t).unwrap(), mother.name),
            story: format!("{} counts {} family through the mothers, back to {}.", capital(who.they()), who.their(), mother.name),
            join: " ",
        }
    } else {
        let mut place = Rng::from_keys(&[ctx.birthplace.unwrap_or(seed), TAG_SEA, 1]);
        let mark = &s.marks[place.below(s.marks.len())];
        let site = &s.sites[place.below(s.sites.len())];
        By {
            english: format!("of the {} {}", title_case(mark), title_case(site)),
            native: format!("{of} {}", capital(&built(&format!("{mark}+{site}"), t))),
            story: format!("{} {} born by the water there, and {} named for the place.", capital(who.they()), if who == Gender::Either { "were" } else { "was" }, who.is()),
            join: " ",
        }
    }
}

/// Ṭaḍoro: the teacher, or the road. Each turn of the life is a new one.
fn road(seed: u64, turns: u32, who: Gender) -> By {
    let t = Tongue::Tadoro;
    let d = &loaded().data;
    let of = &grammar(t).of;
    let mut rng = Rng::from_keys(&[seed, turns as u64, TAG_ROAD]);
    if rng.chance(0.5) {
        let teacher = listed_name(t, Gender::ALL[rng.below(3)], rng::key(&[seed, turns as u64, TAG_ROAD, 1]));
        By {
            english: format!("{}'s student", teacher.name),
            native: format!("{} {of} {}", built("teach:agent", t), teacher.name),
            story: format!("{} learns from {}; the name will change with the next teacher.", capital(who.they()), teacher.name).replace("They learns", "They learn"),
            join: ", ",
        }
    } else {
        let way = &d.roads[rng.below(d.roads.len())];
        By {
            english: format!("of the {} Road", title_case(way)),
            native: format!("{of} {}", capital(&built(&format!("{way}+road"), t))),
            story: format!("{} came lately by that road; the name changes as {} {}.", capital(who.they()), who.they(), if who == Gender::Either { "travel" } else { "travels" }),
            join: " ",
        }
    }
}

/// Every byname the data can make, apart from those built on a person's
/// name: (tongue, English, native). For the tests and the docs.
pub fn every_byname() -> Vec<(Tongue, String, String)> {
    let d = &loaded().data;
    let mut out = Vec::new();
    let of = |t: Tongue| grammar(t).of.clone();
    for count in &d.houses.rings {
        out.push((Tongue::Roduro, format!("of the {}-Ring House", ordinal(count)), format!("{} {}", of(Tongue::Roduro), capital(&built(&format!("{count}+ring"), Tongue::Roduro)))));
    }
    for mark in &d.houses.marks {
        for site in &d.houses.sites {
            out.push((Tongue::Roduro, format!("of the {}-{} House", title_case(mark), title_case(site)), format!("{} {}", of(Tongue::Roduro), capital(&built(&format!("{mark}+{site}"), Tongue::Roduro)))));
        }
    }
    for count in &d.ranks.counts {
        for arm in &d.ranks.arms {
            out.push((Tongue::Qotiro, format!("{} {}", ordinal(count), arm.1), capital(&built(&format!("{count}+{}", arm.0), Tongue::Qotiro))));
        }
    }
    for deed in &d.deeds {
        out.push((Tongue::Qotiro, deed.2.clone(), capital(&built(&format!("{}+{}:agent", deed.0, deed.1), Tongue::Qotiro))));
    }
    for mark in &d.sea_places.marks {
        for site in &d.sea_places.sites {
            out.push((Tongue::Horaro, format!("of the {} {}", title_case(mark), title_case(site)), format!("{} {}", of(Tongue::Horaro), capital(&built(&format!("{mark}+{site}"), Tongue::Horaro)))));
        }
    }
    for way in &d.roads {
        out.push((Tongue::Tadoro, format!("of the {} Road", title_case(way)), format!("{} {}", of(Tongue::Tadoro), capital(&built(&format!("{way}+road"), Tongue::Tadoro)))));
    }
    out
}

/// What a given name hints at.
fn given_story(g: &Given, tongue: Tongue, who: Gender, kept: bool) -> String {
    if kept {
        return format!("\"{}\": a name kept in {} line, handed down.", capital(&g.meaning), who.their());
    }
    let roots = roots_of(&g.made);
    let own_element = match tongue {
        Tongue::Roduro | Tongue::First => ["earth", "stone"],
        Tongue::Qotiro => ["fire", "spear"],
        Tongue::Horaro => ["water", "sea"],
        Tongue::Tadoro => ["wind", "air"],
    };
    let born = roots.iter().find_map(|r| {
        Some(match *r {
            "dawn" => "at dawn",
            "dusk" => "at dusk",
            "night" => "by night",
            "day" => "in broad day",
            "winter" => "in winter",
            "summer" => "in summer",
            "moon" => "under a full moon",
            "star" => "under a clear sky",
            "rain" => "in the rain",
            "snow" => "in snow",
            "frost" => "in a frost",
            "storm" => "in a storm",
            _ => return None,
        })
    });
    let meaning = capital(&g.meaning);
    if let Some(when) = born {
        format!("\"{meaning}\": a child born {when}.")
    } else if roots.iter().any(|r| super::root(r).map(|x| x.field) == Some(Field::Feeling)) {
        format!("\"{meaning}\": what {} parents wished for {}.", who.their(), who.them())
    } else if roots.iter().any(|r| own_element.contains(r)) {
        format!("\"{meaning}\": named for {} people's own element.", who.their())
    } else if roots.iter().any(|r| matches!(*r, "ancestor" | "elder" | "mother" | "father")) {
        format!("\"{meaning}\": a name that looks back to those who came before.")
    } else {
        format!("\"{meaning}\".")
    }
}

/// A person's name: a given name and the byname their culture gives them.
/// The same seed and context always give the same name; change the context
/// (a new home, a deed, a new teacher) and the byname changes with it.
pub fn generate_person(tongue: Tongue, gender: Gender, seed: u64, ctx: &Context) -> PersonName {
    // The given name: their own, or (now and then) one kept in the line.
    let mut rng = Rng::from_keys(&[seed, TAG_KEPT]);
    let kept = ctx.lineage.filter(|_| gender != Gender::Either && rng.chance(KEPT));
    let g = match kept {
        Some(line) => listed_name(tongue, gender, rng::key(&[line, gender as u64, rng.below(KEPT_NAMES) as u64, TAG_KEPT])),
        None => given_name(tongue, gender, seed),
    };
    let mut title = None;
    let by = match tongue {
        Tongue::Roduro | Tongue::First => {
            title = ctx.job.as_deref().and_then(|j| job_title(j, Tongue::Roduro));
            house(ctx.home.or(ctx.lineage).unwrap_or(seed), gender)
        }
        Tongue::Qotiro => deed(seed, ctx.turns, gender),
        Tongue::Horaro => sea_line(seed, ctx, gender),
        Tongue::Tadoro => road(seed, ctx.turns, gender),
    };
    let mut story = given_story(&g, tongue, gender, kept.is_some());
    story.push(' ');
    story.push_str(&by.story);
    if let Some((t, _)) = &title {
        story.push_str(&format!(" {} {} {t} there.", capital(gender.they()), gender.is()));
    }
    PersonName { tongue, gender, say: pronounce(&g.name, tongue), given: g.name, meaning: g.meaning, byname: by.english, byname_native: by.native, title, story, join: by.join }
}

/// The same, for someone joining people already named (a town): never the
/// very same name as one of them, and never a given name one letter off
/// another's (two such neighbours would be mixed up). Sharing a given name
/// is fine: the bynames tell them apart. `taken` is the names already
/// given, in the order they were given.
pub fn generate_person_among(tongue: Tongue, gender: Gender, seed: u64, ctx: &Context, taken: &[PersonName]) -> PersonName {
    let clashes = |p: &PersonName| taken.iter().any(|t| (t.given != p.given && lookalike(&t.given, &p.given)) || t.full() == p.full());
    let mut p = generate_person(tongue, gender, seed, ctx);
    let mut tries = 0u64;
    while clashes(&p) && tries < 12 {
        tries += 1;
        p = generate_person(tongue, gender, rng::key(&[seed, tries, TAG_GIVEN]), ctx);
    }
    p
}

// ---- The readable copies ------------------------------------------------------------

/// The hand-kept lists as Markdown: this is `docs/names.md`.
pub fn tables() -> String {
    let mut s = String::new();
    s.push_str("# Given names\n\n");
    s.push_str("*Generated by `cargo run --release --bin lang -- names` from `assets/lang/names/`. Don't edit by hand: to drop a name, delete its line there; to add one, add a line with its recipe and ending (the tests say what the name must then be).*\n\n");
    s.push_str("Every name is built from roots (`docs/glossary.md`) and finished with one of its tongue's name endings. The meaning is what a character can say their name means.\n\n");
    for t in Tongue::SPOKEN {
        let g = grammar(t);
        s.push_str(&format!("## {}\n\n", t.name()));
        s.push_str(&format!("Endings: a man's name ends in {}; a woman's in {}; a name for either in {}.\n\n", ends(&g.male), ends(&g.female), ends(&g.either)));
        for gender in Gender::ALL {
            let of: Vec<&Listed> = listed(t).iter().filter(|l| l.gender == gender).collect();
            s.push_str(&format!("### {} {} names ({})\n\n", t.name(), gender.word(), of.len()));
            s.push_str("| Name | Say it | Means | Name | Say it | Means |\n|---|---|---|---|---|---|\n");
            for pair in of.chunks(2) {
                let cell = |l: &Listed| format!("**{}** | {} | {}", capital(&l.name), pronounce(&l.name, t), meaning(&l.made));
                s.push_str(&format!("| {} | {} |\n", cell(pair[0]), pair.get(1).map(|l| cell(l)).unwrap_or_else(|| " | | ".to_string())));
            }
            s.push('\n');
        }
    }
    s
}

fn ends(e: &[String]) -> String {
    e.iter().map(|x| format!("`-{x}`")).collect::<Vec<_>>().join(", ")
}

/// The jobs a sample person of a people might have.
const SAMPLE_JOBS: [[&str; 5]; 4] = [
    ["Stone Tender", "Farmer", "Innkeeper", "Healer", "Forager and hunter"],
    ["Smith", "Guard", "Mason", "Armourer", "Priest"],
    ["Fisher and diver", "Boatwright", "Weaver and sealer", "Kelp gatherer", "Cook"],
    ["Scribe", "Teacher", "Alchemist", "Exchanger", "Caravaner"],
];

/// The context of the nth sample person of a people (a few share a line
/// and a home, so the tables show what families look like).
pub fn sample_context(tongue: Tongue, n: u64) -> Context {
    let k = slot(tongue) as u64;
    Context {
        // Samples come in households of four.
        lineage: Some(1000 * (k + 1) + 13 * (n / 4)),
        home: Some(2000 * (k + 1) + 7 * (n / 4)),
        job: if n % 3 == 0 { Some(SAMPLE_JOBS[slot(tongue)][(n as usize / 3) % 5].to_string()) } else { None },
        birthplace: Some(3000 * (k + 1) + n / 6),
        turns: (n % 4) as u32,
    }
}

/// Twenty sample people of each people, as Markdown (`docs/people.md`).
pub fn samples() -> String {
    let mut s = String::new();
    s.push_str("# Sample people\n\n");
    s.push_str("*Generated by `cargo run --release --bin lang -- people`: twenty of each people, from seeds 1 to 20. They come in households of four, so some share a house or a line.*\n\n");
    for t in Tongue::SPOKEN {
        s.push_str(&format!("## {}\n\n", t.name()));
        s.push_str("| Name as shown | Say it | In their own tongue | What the name hints at |\n|---|---|---|---|\n");
        let mut town: Vec<PersonName> = Vec::new();
        for n in 0..20u64 {
            let gender = match n % 7 {
                6 => Gender::Either,
                x if x % 2 == 0 => Gender::Female,
                _ => Gender::Male,
            };
            let p = generate_person_among(t, gender, rng::key(&[n + 1, slot(t) as u64, 0x5341_4D50]), &sample_context(t, n), &town);
            s.push_str(&format!("| **{}** | {} | {} | {} |\n", p.full(), p.say, p.full_native(), p.story));
            town.push(p);
        }
        s.push('\n');
    }
    s
}
