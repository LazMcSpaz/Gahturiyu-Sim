//! The sacred and the shared, in every tongue: the gods, the elements, the
//! words of worship and rule, and what each people calls the others. The
//! data is `assets/lang/sacred.ron`.

use serde::Deserialize;

use super::grammar::{compound, with, Affix};
use super::say::capital;
use super::{derive, word, Tongue};

/// How a word is made.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum Made {
    /// A root's own word.
    Root(String),
    /// The first root describes the second.
    Compound(String, String),
    /// A root with one of the tongue's endings.
    With(String, Affix),
    /// A First Speech form, worn down by each tongue's rules.
    First(String),
}

/// A word made in a tongue. `None` if it names a root that does not exist.
pub fn make(made: &Made, tongue: Tongue) -> Option<String> {
    Some(match made {
        Made::Root(id) => word(id, tongue)?,
        Made::Compound(describing, main) => compound(tongue, &word(describing, tongue)?, &word(main, tongue)?),
        Made::With(id, affix) => with(tongue, &word(id, tongue)?, *affix),
        Made::First(first) => derive(first, tongue),
    })
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    /// The three above all others (and their name together).
    Trinity,
    /// The greater gods, each over a sphere of the world.
    Domain,
    /// The children of two gods.
    Child,
}

#[derive(Deserialize, Clone, Debug)]
pub struct God {
    pub id: String,
    /// The Roduro name, as the lore has it.
    pub canon: String,
    /// The name shown first.
    pub english: String,
    pub domain: String,
    pub rank: Rank,
    /// The name in the First Speech.
    pub first: String,
    /// The roots it is built from.
    pub roots: Vec<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    /// Names that broke their tongue's rules.
    #[serde(default)]
    pub own: Vec<(Tongue, String)>,
}

impl God {
    /// The god's name in a tongue.
    pub fn name(&self, tongue: Tongue) -> String {
        let n = match self.own.iter().find(|o| o.0 == tongue) {
            Some(o) => o.1.clone(),
            None => derive(&self.first, tongue),
        };
        capital(&n)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Element {
    pub id: String,
    pub english: String,
    pub root: String,
}

/// What one people calls each of the four: the root it names each by.
#[derive(Deserialize, Clone, Debug)]
pub struct Naming {
    pub speaker: Tongue,
    pub earth: String,
    pub fire: String,
    pub water: String,
    pub air: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Term {
    pub id: String,
    pub english: String,
    pub made: Made,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Sacred {
    pub gods: Vec<God>,
    pub elements: Vec<Element>,
    pub peoples: Vec<Naming>,
    pub terms: Vec<Term>,
}

pub fn sacred() -> &'static Sacred {
    &super::data().sacred
}

pub fn god(id: &str) -> Option<&'static God> {
    sacred().gods.iter().find(|g| g.id == id)
}

/// The root a speaker names a people by, and what that makes the name mean.
fn people_root(of: Tongue, by: Tongue) -> Option<&'static str> {
    let n = sacred().peoples.iter().find(|n| n.speaker == by)?;
    Some(match of {
        Tongue::Roduro => &n.earth,
        Tongue::Qotiro => &n.fire,
        Tongue::Horaro => &n.water,
        Tongue::Tadoro => &n.air,
        Tongue::First => return None,
    })
}

/// What the people who speak `by` call the people who speak `of`.
/// `people_name(Qotiro, Roduro)` is "Qotiro"; `people_name(Qotiro, Qotiro)`
/// is what the Fire-kind call themselves.
pub fn people_name(of: Tongue, by: Tongue) -> Option<String> {
    let root = people_root(of, by)?;
    Some(capital(&with(by, &word(root, by)?, Affix::Kind)))
}

/// What that name means, in English: "spear-kind".
pub fn people_meaning(of: Tongue, by: Tongue) -> Option<String> {
    Some(format!("{}-kind", people_root(of, by)?))
}

/// The plain English for a people: "the Earth-kind".
pub fn people_english(of: Tongue) -> &'static str {
    match of {
        Tongue::Roduro => "the Earth-kind",
        Tongue::Qotiro => "the Fire-kind",
        Tongue::Horaro => "the Water-kind",
        Tongue::Tadoro => "the Air-kind",
        Tongue::First => "the first people",
    }
}

/// Everything in `sacred.ron`, in every tongue, as tables to read:
/// `docs/sacred.md` is this, and a test keeps the file in step.
pub fn tables() -> String {
    use super::say::pronounce;
    use std::fmt::Write;
    let cell = |w: &str, t: Tongue| format!("{w} (*{}*)", pronounce(w, t));
    let mut out = String::new();
    let _ = writeln!(out, "# The sacred and the shared, in all four tongues\n");
    let _ = writeln!(out, "Made from `assets/lang/sacred.ron` by `cargo run --release --bin lang -- sacred` (a test checks this file is up to date). Don't edit it by hand.\n");
    let _ = writeln!(out, "Each name is followed by how to say it, the loud syllable in capitals. The English name is the one the game shows first.\n");
    let s = sacred();

    let _ = writeln!(out, "## The gods\n");
    let _ = writeln!(out, "The Roduro names are canon (`pantheon.md`). The gods are older than the split of the tongues, so the other three names are the same old name worn down each tongue's way.\n");
    for (rank, title) in [(Rank::Trinity, "The Trinity, above all others"), (Rank::Domain, "The domain gods"), (Rank::Child, "The children of the gods")] {
        let _ = writeln!(out, "### {title}\n");
        let _ = writeln!(out, "| Shown as | Over | Roduro | Qotiro | Horaro | Ṭaḍoro |");
        let _ = writeln!(out, "|---|---|---|---|---|---|");
        for g in s.gods.iter().filter(|g| g.rank == rank) {
            let _ = write!(out, "| {} | {} |", g.english, g.domain);
            for t in Tongue::SPOKEN {
                let _ = write!(out, " {} |", cell(&g.name(t), t));
            }
            let _ = writeln!(out);
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## The four elements\n");
    let _ = writeln!(out, "| Element | First Speech | Roduro | Qotiro | Horaro | Ṭaḍoro |");
    let _ = writeln!(out, "|---|---|---|---|---|---|");
    for e in &s.elements {
        let _ = write!(out, "| {} | {} |", e.english, super::root(&e.root).map(|r| r.first.as_str()).unwrap_or("?"));
        for t in Tongue::SPOKEN {
            let _ = write!(out, " {} |", cell(&word(&e.root, t).unwrap_or_default(), t));
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "\n## What each people calls each people\n");
    let _ = writeln!(out, "Read along a row: the speaker's names for all four. The Roduro row is canon, and those four names stay the ones the game shows. Each name's meaning is under it.\n");
    let _ = writeln!(out, "| Speaker | {} | {} | {} | {} |", people_english(Tongue::Roduro), people_english(Tongue::Qotiro), people_english(Tongue::Horaro), people_english(Tongue::Tadoro));
    let _ = writeln!(out, "|---|---|---|---|---|");
    for by in Tongue::SPOKEN {
        let _ = write!(out, "| **{}** |", by.name());
        for of in Tongue::SPOKEN {
            let n = people_name(of, by).unwrap_or_default();
            let _ = write!(out, " {}<br>{} |", cell(&n, by), people_meaning(of, by).unwrap_or_default());
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "\n## Words of worship and rule\n");
    let _ = writeln!(out, "| English | Roduro | Qotiro | Horaro | Ṭaḍoro |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for term in &s.terms {
        let _ = write!(out, "| {} |", term.english);
        for t in Tongue::SPOKEN {
            let _ = write!(out, " {} |", cell(&make(&term.made, t).unwrap_or_default(), t));
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "\n## How the tongues put words together\n");
    let _ = writeln!(out, "| | Roduro | Qotiro | Horaro | Ṭaḍoro |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    let g: Vec<_> = Tongue::SPOKEN.iter().map(|t| super::grammar::grammar(*t)).collect();
    let row = |out: &mut String, label: &str, f: &dyn Fn(&super::grammar::Grammar) -> String| {
        let _ = writeln!(out, "| {label} | {} | {} | {} | {} |", f(g[0]), f(g[1]), f(g[2]), f(g[3]));
    };
    row(&mut out, "Order in a compound", &|g| match g.order {
        super::grammar::Order::HeadLast => "describing word first".to_string(),
        super::grammar::Order::HeadFirst => "main word first".to_string(),
    });
    let affix = |a: Affix| {
        move |g: &super::grammar::Grammar| {
            let (form, side) = g.affix(a);
            match side {
                super::grammar::Side::Before => format!("{form}-"),
                super::grammar::Side::After => format!("-{form}"),
            }
        }
    };
    row(&mut out, "A people", &affix(Affix::Kind));
    row(&mut out, "One who does it", &affix(Affix::Agent));
    row(&mut out, "Where it is", &affix(Affix::Place));
    row(&mut out, "A little one", &affix(Affix::Small));
    row(&mut out, "A great one", &affix(Affix::Great));
    row(&mut out, "The highest", &affix(Affix::Most));
    row(&mut out, "\"of\"", &|g| g.of.clone());
    row(&mut out, "Stress", &|g| match g.stress {
        super::grammar::Stress::First => "first syllable".to_string(),
        super::grammar::Stress::NextToLast => "next to last".to_string(),
        super::grammar::Stress::Last => "last syllable".to_string(),
    });
    row(&mut out, "A man's name ends in", &|g| g.male.join(", "));
    row(&mut out, "A woman's name ends in", &|g| g.female.join(", "));
    row(&mut out, "Either", &|g| g.either.join(", "));
    out
}
