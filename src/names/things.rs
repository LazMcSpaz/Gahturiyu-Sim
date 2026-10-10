//! The names of things: jobs, crafts, materials, items, buildings,
//! creatures, weather. The list is data (`assets/lang/things.ron`); each
//! entry is the English name the game shows and a short recipe saying which
//! roots the native name is built from. The native name itself is never
//! typed in: it is worked out from the recipe by the tongue's own rules, so
//! it changes with the rules and can be built in any of the four tongues.
//!
//! A recipe is one of:
//!
//!   `stone`             a root, by its id in roots.ron
//!   `black+water`       the first describes the second: "black water"
//!   `forge:agent`       with an ending: agent (one who does it), place,
//!                       small, great, kind, most
//!   `stone+tend:agent`  both
//!   `ring/swift`        two words: "ring of swiftness"
//!
//! and a part may also be `=Another Thing` (that thing's own native name),
//! `*turiyu` (a First Speech form, worn down by the tongue) or `@qotihiqi`
//! (a god, by id).

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::grammar::{self, Affix};
use super::{capital, pronounce, Tongue};

/// What sort of thing it is (how the table is laid out).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Group {
    Job,
    Craft,
    Material,
    Weapon,
    Armour,
    Gear,
    Food,
    Treasure,
    Book,
    Architecture,
    TownPlace,
    Station,
    BaseBuilding,
    Creature,
    Fish,
    Weather,
    Everyday,
    World,
}

impl Group {
    pub const ALL: [Group; 18] = [
        Group::Job,
        Group::Craft,
        Group::Material,
        Group::Weapon,
        Group::Armour,
        Group::Gear,
        Group::Food,
        Group::Treasure,
        Group::Book,
        Group::Architecture,
        Group::TownPlace,
        Group::Station,
        Group::BaseBuilding,
        Group::Creature,
        Group::Fish,
        Group::Weather,
        Group::Everyday,
        Group::World,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Group::Job => "Jobs",
            Group::Craft => "Crafts",
            Group::Material => "Materials and goods",
            Group::Weapon => "Weapons",
            Group::Armour => "Armour and clothing",
            Group::Gear => "Gear",
            Group::Food => "Food, herbs and draughts",
            Group::Treasure => "Named treasures",
            Group::Book => "Books and writing",
            Group::Architecture => "Buildings and their parts",
            Group::TownPlace => "Places in a town",
            Group::Station => "Work stations",
            Group::BaseBuilding => "What the squad can build",
            Group::Creature => "Creatures",
            Group::Fish => "Fish and the sea",
            Group::Weather => "Weather",
            Group::Everyday => "Everyday things",
            Group::World => "The world",
        }
    }
}

/// One named thing.
#[derive(Deserialize, Clone, Debug)]
pub struct Thing {
    /// The name the game shows. Never changed by this system.
    pub english: String,
    pub group: Group,
    /// Whose thing it is: its native name is in this tongue. `None`: every
    /// people has it, and each has its own word.
    #[serde(default)]
    pub belongs: Option<Tongue>,
    /// The recipe for the native name (see the top of this file).
    pub made: String,
    /// Other English names the code uses for the same thing ("Pearls").
    #[serde(default)]
    pub also: Vec<String>,
}

/// One part of a recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Part {
    Root(String),
    Thing(String),
    First(String),
    God(String),
}

/// One word of a recipe: one or two parts and perhaps an ending.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Word {
    parts: Vec<Part>,
    affix: Option<Affix>,
}

fn parse_part(s: &str) -> Part {
    let s = s.trim();
    if let Some(t) = s.strip_prefix('=') {
        Part::Thing(t.to_string())
    } else if let Some(f) = s.strip_prefix('*') {
        Part::First(f.to_string())
    } else if let Some(g) = s.strip_prefix('@') {
        Part::God(g.to_string())
    } else {
        Part::Root(s.to_string())
    }
}

fn parse_word(s: &str) -> Result<Word, String> {
    let (body, affix) = match s.rsplit_once(':') {
        Some((b, a)) => (
            b,
            Some(match a.trim() {
                "agent" => Affix::Agent,
                "place" => Affix::Place,
                "small" => Affix::Small,
                "great" => Affix::Great,
                "kind" => Affix::Kind,
                "most" => Affix::Most,
                other => return Err(format!("no ending called `{other}`")),
            }),
        ),
        None => (s, None),
    };
    let parts: Vec<Part> = body.split('+').map(parse_part).collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err(format!("`{s}`: a word is built from one or two parts"));
    }
    Ok(Word { parts, affix })
}

/// A recipe, read: one word, or "A of B".
fn parse(made: &str) -> Result<Vec<Word>, String> {
    let words: Result<Vec<Word>, String> = made.split('/').map(parse_word).collect();
    let words = words?;
    if words.len() > 2 {
        return Err(format!("`{made}`: at most \"A of B\""));
    }
    Ok(words)
}

struct Table {
    things: Vec<Thing>,
    by_name: HashMap<String, usize>,
}

fn table() -> &'static Table {
    static T: OnceLock<Table> = OnceLock::new();
    T.get_or_init(|| {
        let things: Vec<Thing> = ron::from_str(include_str!("../../assets/lang/things.ron")).unwrap_or_else(|e| panic!("assets/lang/things.ron: {e}"));
        let mut by_name = HashMap::new();
        for (i, t) in things.iter().enumerate() {
            for n in std::iter::once(&t.english).chain(t.also.iter()) {
                if by_name.insert(n.to_lowercase(), i).is_some() {
                    panic!("assets/lang/things.ron names `{n}` twice");
                }
            }
        }
        Table { things, by_name }
    })
}

/// Every named thing, in the order of the file.
pub fn things() -> &'static [Thing] {
    &table().things
}

/// The thing with this English name (capitals don't matter).
pub fn thing(english: &str) -> Option<&'static Thing> {
    let t = table();
    t.by_name.get(&english.to_lowercase()).map(|i| &t.things[*i])
}

fn part_word(p: &Part, tongue: Tongue, depth: u8) -> Result<String, String> {
    match p {
        Part::Root(id) => super::word(id, tongue).ok_or_else(|| format!("no root `{id}`")),
        Part::First(f) => Ok(super::derive(f, tongue)),
        // A god keeps the capital: only ever used as a word on its own ("... of Qotihiqì").
        Part::God(id) => super::god(id).map(|g| g.name(tongue)).ok_or_else(|| format!("no god `{id}`")),
        Part::Thing(e) => {
            if depth > 4 {
                return Err(format!("`{e}` is built from itself"));
            }
            let t = thing(e).ok_or_else(|| format!("no thing called `{e}`"))?;
            build(&t.made, tongue, depth + 1)
        }
    }
}

pub(crate) fn build(made: &str, tongue: Tongue, depth: u8) -> Result<String, String> {
    let mut out = Vec::new();
    for w in parse(made)? {
        let mut word = part_word(&w.parts[0], tongue, depth)?;
        if let Some(main) = w.parts.get(1) {
            let main = part_word(main, tongue, depth)?;
            // A part that is itself two words ("ring of ...") can't be folded in.
            if word.contains(' ') || main.contains(' ') {
                return Err(format!("`{made}`: a part is more than one word"));
            }
            word = grammar::compound(tongue, &word, &main);
        }
        if let Some(a) = w.affix {
            word = grammar::with(tongue, &word, a);
        }
        out.push(word);
    }
    Ok(match out.len() {
        1 => out.remove(0),
        _ => grammar::of(tongue, &out[0], &out[1]),
    })
}

impl Thing {
    /// Its name as speakers of a tongue would build it from their own words.
    pub fn in_tongue(&self, tongue: Tongue) -> String {
        build(&self.made, tongue, 0).unwrap_or_else(|e| panic!("things.ron, {}: {e}", self.english))
    }

    /// Its native name: in its own people's tongue if it belongs to one
    /// (everyone else uses their word for it), otherwise the asker's.
    pub fn native(&self, asker: Tongue) -> (Tongue, String) {
        let t = self.belongs.unwrap_or(asker);
        (t, self.in_tongue(t))
    }

    /// What the native name says, word for word: "stone-tend-er".
    pub fn literal(&self) -> String {
        literal(&self.made, 0)
    }

    /// Whether the recipe can be read (for the tests).
    pub fn check(&self) -> Result<(), String> {
        for t in Tongue::SPOKEN {
            build(&self.made, t, 0)?;
        }
        Ok(())
    }
}

fn literal(made: &str, depth: u8) -> String {
    let Ok(words) = parse(made) else { return String::new() };
    let part = |p: &Part| match p {
        // Ids are plain English words; the few that aren't ("fish_for") give their meaning.
        Part::Root(id) if id.contains('_') => super::root(id).map(|r| r.gloss.split([',', ';']).next().unwrap_or("").trim_start_matches("to ").to_string()).unwrap_or_default(),
        Part::Root(id) => id.clone(),
        Part::First(f) => format!("\"{f}\""),
        Part::God(id) => super::god(id).map(|g| g.english.clone()).unwrap_or_default(),
        Part::Thing(e) => match thing(e) {
            Some(t) if depth < 4 => literal(&t.made, depth + 1),
            _ => e.to_lowercase(),
        },
    };
    let said: Vec<String> = words
        .iter()
        .map(|w| {
            let body = w.parts.iter().map(part).collect::<Vec<_>>().join("-");
            match w.affix {
                None => body,
                Some(Affix::Agent) => format!("{body}-er"),
                Some(Affix::Place) => format!("{body}-place"),
                Some(Affix::Small) => format!("little {body}"),
                Some(Affix::Great) => format!("great {body}"),
                Some(Affix::Kind) => format!("{body}-kind"),
                Some(Affix::Most) => format!("highest {body}"),
            }
        })
        .collect();
    said.join(" of ")
}

/// The native name of a thing by its English name, as `asker`'s people
/// would say it: the owner's word if the thing belongs to one people.
pub fn native_name(english: &str, asker: Tongue) -> Option<String> {
    thing(english).map(|t| t.native(asker).1)
}

/// Things whose native names come out the same in a tongue though they are
/// built differently (for the tests and `lang things-clashes`).
pub fn clashes(tongue: Tongue) -> Vec<(String, Vec<&'static str>)> {
    let mut seen: std::collections::BTreeMap<String, Vec<&'static Thing>> = Default::default();
    for t in things() {
        seen.entry(t.in_tongue(tongue)).or_default().push(t);
    }
    seen.into_iter()
        .filter(|(_, v)| v.iter().any(|t| literal(&t.made, 0) != literal(&v[0].made, 0)))
        .map(|(w, v)| (w, v.iter().map(|t| t.english.as_str()).collect()))
        .collect()
}

/// The whole table as Markdown: this is `docs/things.md`.
pub fn tables() -> String {
    let mut s = String::new();
    s.push_str("# The names of things\n\n");
    s.push_str("*Generated by `cargo run --release --bin lang -- things` from `assets/lang/things.ron`. Don't edit by hand: change the recipe there and run it again.*\n\n");
    s.push_str("The game shows the English name. The native name sits beneath it or on hover, with how to say it.\n\n");
    s.push_str("- A thing that **belongs to one people** (their craft, their material, their building, a creature of their country) has one native name, in that people's tongue. Everyone else uses that word.\n");
    s.push_str("- A thing **every people has** has a word in each tongue, built the same way from each tongue's own roots.\n");
    s.push_str("- *Word for word* is what the native name says, root by root. \"-er\" is one who does it; \"-place\" is where it is done.\n\n");
    let total = things().len();
    let owned = things().iter().filter(|t| t.belongs.is_some()).count();
    s.push_str(&format!("{total} things: {owned} belong to one people, {} are shared.\n\n", total - owned));
    for g in Group::ALL {
        let all: Vec<&Thing> = things().iter().filter(|t| t.group == g).collect();
        if all.is_empty() {
            continue;
        }
        s.push_str(&format!("## {}\n\n", g.title()));
        let owned: Vec<&&Thing> = all.iter().filter(|t| t.belongs.is_some()).collect();
        let shared: Vec<&&Thing> = all.iter().filter(|t| t.belongs.is_none()).collect();
        if !owned.is_empty() {
            s.push_str("| English | Whose | Native name | Say it | Word for word |\n|---|---|---|---|---|\n");
            for t in owned {
                let tongue = t.belongs.unwrap();
                let n = t.in_tongue(tongue);
                s.push_str(&format!("| {} | {} | **{}** | {} | {} |\n", t.english, tongue.name(), capital(&n), pronounce(&n, tongue), t.literal()));
            }
            s.push('\n');
        }
        if !shared.is_empty() {
            s.push_str("| English | Roduro | Qotiro | Horaro | Ṭaḍoro | Word for word |\n|---|---|---|---|---|---|\n");
            for t in shared {
                let w: Vec<String> = Tongue::SPOKEN.iter().map(|x| capital(&t.in_tongue(*x))).collect();
                s.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n", t.english, w[0], w[1], w[2], w[3], t.literal()));
            }
            s.push('\n');
        }
    }
    s
}
