//! Authored people: a town's roster (`docs/authored-people.md`, Laz's
//! brief `docs/npc-tool-prompt.md`). Laz decides who lives in a town; the
//! engine stays as it is.
//!
//! - **Where.** `assets/world/people/<town key>.ron` is a town's roster (the
//!   key is the mark's in `assets/world/towns.ron`); `wanderers.ron` there
//!   holds the people with no place. A town with no file is peopled from
//!   the seed as before.
//! - **At world creation** (`worldgen`): a town with a roster gets exactly
//!   its authored characters and its stored crowd, and nobody seeded (anyone
//!   the seed would have sent there goes to another town). Each becomes an
//!   ordinary `Person` with the stored fields in place of the rolled ones:
//!   people, sex (a seed whose reading is that sex: names and offices read
//!   it), name, the traits given (the rest rolled from the people as
//!   today), home; and once the town's life is founded, profession,
//!   workplace, honour and the household's purse by wealth (`apply_rosters`).
//! - **Every stored fact stays on the person** (`Person::authored`, saved),
//!   for the dialogue's conditions later: faction, wealth, background, the
//!   personal detail, relations, schedule, looks, a lines file, and that
//!   they're authored at all.
//!
//! Not yet (stages 2-4): the editor, the crowd filler, households from the
//! crowd's own numbers (the town's customs still group them), the schedule
//! and relations acting on anything, and the ruler taking the seat.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::jobs::Job;
use super::race::Race;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sex {
    Man,
    Woman,
}

/// How well off: the household's purse starts at so many days' keep.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Wealth {
    Destitute,
    Poor,
    #[default]
    Comfortable,
    Rich,
    VeryRich,
}

impl Wealth {
    /// Days of the household's costs in hand at the start.
    pub fn days(self) -> f32 {
        match self {
            Wealth::Destitute => 0.0,
            Wealth::Poor => 2.0,
            Wealth::Comfortable => 7.0,
            Wealth::Rich => 30.0,
            Wealth::VeryRich => 120.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelKind {
    Spouse,
    Child,
    Parent,
    Sibling,
    Rival,
    Friend,
    Lover,
    Debtor,
    Creditor,
}

/// The sim's traits, 0..1 each. Those left out are rolled from the people
/// as for anyone. (`wanderlust`, `boldness`, `sociability`, `patience` are
/// the person's; `honour` and `asks`, how readily they'd ask an outsider for
/// help, are their habits.)
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Traits {
    pub wanderlust: Option<f32>,
    pub boldness: Option<f32>,
    pub sociability: Option<f32>,
    pub patience: Option<f32>,
    pub honour: Option<f32>,
    pub asks: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Relation {
    /// Another authored character's key, in any roster.
    pub to: String,
    pub kind: RelKind,
}

/// Where a schedule puts them.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum At {
    Home,
    Work,
    Hearth,
    /// A workplace of the town (`society.towns[t].places`).
    Place(u16),
    /// A building of the town's layout.
    Building(u16),
}

/// An hour range of the day spent somewhere, over the day plan.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Stay {
    pub from: f32,
    pub to: f32,
    pub at: At,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Appearance {
    pub style: String,
    pub colours: Vec<String>,
    pub marks: Vec<String>,
}

/// An authored character.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Character {
    /// Stable, unique across every roster.
    pub key: String,
    pub name: String,
    pub people: Race,
    pub sex: Sex,
    pub profession: Job,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub sub_faction: Option<String>,
    #[serde(default)]
    pub wealth: Wealth,
    #[serde(default)]
    pub traits: Traits,
    /// A building index in the town's layout.
    #[serde(default)]
    pub home: Option<u16>,
    /// A workplace index (`society.towns[t].places`).
    #[serde(default)]
    pub workplace: Option<u16>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub background: String,
    /// Why they matter: a note for Laz, never shown to the player.
    #[serde(default)]
    pub importance: String,
    #[serde(default)]
    pub schedule: Vec<Stay>,
    /// The schedule in words, as the page writes it ("at the smoke hall every
    /// night from dusk"): a note for whoever turns it into stays.
    #[serde(default)]
    pub schedule_note: String,
    #[serde(default)]
    pub appearance: Appearance,
    /// `data/lines/people/<key>.txt`.
    #[serde(default)]
    pub lines: Option<String>,
}

/// The four peoples' shares in a crowd row.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Mix {
    pub roduro: f32,
    pub qotiro: f32,
    pub horaro: f32,
    pub tadoro: f32,
}

/// A role and how many: the crowd filler makes the crowd from these.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct CrowdRow {
    pub role: Job,
    pub count: u16,
    /// None: the town's founders' mix.
    #[serde(default)]
    pub mix: Option<Mix>,
}

/// One of the crowd, as generated (and perhaps edited since).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CrowdPerson {
    /// `<town key>.<row>.<n>`.
    pub key: String,
    pub name: String,
    pub people: Race,
    pub sex: Sex,
    pub profession: Job,
    /// Which of the crowd's households (by number within the town).
    #[serde(default)]
    pub household: u32,
    #[serde(default)]
    pub home: Option<u16>,
    #[serde(default)]
    pub traits: Traits,
    #[serde(default)]
    pub wealth: Wealth,
    /// A clothing variant.
    #[serde(default)]
    pub appearance: u8,
    /// One personal detail their talk can mention.
    #[serde(default)]
    pub detail: String,
}

/// A town's roster (or, with an empty `town`, the wanderers').
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Roster {
    pub town: String,
    /// An authored character's key.
    pub ruler: Option<String>,
    pub characters: Vec<Character>,
    pub crowd_rows: Vec<CrowdRow>,
    pub crowd: Vec<CrowdPerson>,
}

impl Roster {
    pub fn from_ron(text: &str) -> Result<Roster, String> {
        ron::from_str(text).map_err(|e| e.to_string())
    }

    /// As the tools write it: pretty, one field a line.
    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default().struct_names(false)).expect("a roster writes")
    }

    pub fn load(path: &Path) -> Result<Roster, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Roster::from_ron(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_ron())
    }

    /// Everyone in it, characters first, as they become people.
    pub fn people(&self) -> impl Iterator<Item = Authored> + '_ {
        self.characters.iter().cloned().map(Authored::Character).chain(self.crowd.iter().cloned().map(Authored::Crowd))
    }

    /// How many people it gives.
    pub fn len(&self) -> usize {
        self.characters.len() + self.crowd.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Every roster there is: by town key, and the wanderers'.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Rosters {
    pub towns: BTreeMap<String, Roster>,
    pub wanderers: Option<Roster>,
}

/// Where the rosters live, relative to the game's folder.
pub const PEOPLE_DIR: &str = "assets/world/people";

impl Rosters {
    /// The rosters in `assets/world/people/` (none there: none at all). A
    /// file that won't read is said on the console and skipped, so a typo
    /// leaves that town as the seed makes it rather than stopping the game.
    pub fn load() -> Rosters {
        match people_dir() {
            Some(dir) => Rosters::load_from(&dir),
            None => Rosters::default(),
        }
    }

    pub fn load_from(dir: &Path) -> Rosters {
        let mut out = Rosters::default();
        let Ok(entries) = std::fs::read_dir(dir) else { return out };
        let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ron")).collect();
        files.sort();
        for path in files {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            match Roster::load(&path) {
                Ok(r) if stem == "wanderers" => out.wanderers = Some(r),
                Ok(r) => {
                    out.towns.insert(stem, r);
                }
                Err(e) => eprintln!("roster skipped: {e}"),
            }
        }
        out
    }

    pub fn for_town(&self, key: &str) -> Option<&Roster> {
        self.towns.get(key).filter(|r| !r.is_empty())
    }

    pub fn is_empty(&self) -> bool {
        self.towns.values().all(Roster::is_empty) && self.wanderers.as_ref().is_none_or(Roster::is_empty)
    }
}

fn people_dir() -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.extend(cwd.ancestors().take(4).map(Path::to_path_buf));
    }
    if let Ok(exe) = std::env::current_exe() {
        roots.extend(exe.ancestors().skip(1).take(4).map(Path::to_path_buf));
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(dir));
    }
    roots.into_iter().map(|r| r.join(PEOPLE_DIR)).find(|p| p.is_dir())
}

/// What's kept of an authored person, on the `Person` (saved): the roster's
/// entry as written, so every fact can be read later.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Authored {
    Character(Character),
    Crowd(CrowdPerson),
}

impl Authored {
    pub fn key(&self) -> &str {
        match self {
            Authored::Character(c) => &c.key,
            Authored::Crowd(c) => &c.key,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Authored::Character(c) => &c.name,
            Authored::Crowd(c) => &c.name,
        }
    }
    pub fn people(&self) -> Race {
        match self {
            Authored::Character(c) => c.people,
            Authored::Crowd(c) => c.people,
        }
    }
    pub fn sex(&self) -> Sex {
        match self {
            Authored::Character(c) => c.sex,
            Authored::Crowd(c) => c.sex,
        }
    }
    pub fn profession(&self) -> Job {
        match self {
            Authored::Character(c) => c.profession,
            Authored::Crowd(c) => c.profession,
        }
    }
    pub fn wealth(&self) -> Wealth {
        match self {
            Authored::Character(c) => c.wealth,
            Authored::Crowd(c) => c.wealth,
        }
    }
    pub fn traits(&self) -> Traits {
        match self {
            Authored::Character(c) => c.traits,
            Authored::Crowd(c) => c.traits,
        }
    }
    pub fn home(&self) -> Option<u16> {
        match self {
            Authored::Character(c) => c.home,
            Authored::Crowd(c) => c.home,
        }
    }
    pub fn workplace(&self) -> Option<u16> {
        match self {
            Authored::Character(c) => c.workplace,
            Authored::Crowd(_) => None,
        }
    }
    /// One of the crowd (they talk less, later), not a character.
    pub fn is_crowd(&self) -> bool {
        matches!(self, Authored::Crowd(_))
    }
    /// The seed to build them from: keyed by the world and their key, and
    /// one whose reading (names, offices) is their sex.
    pub fn seed(&self, world_seed: u64) -> u64 {
        let want = match self.sex() {
            Sex::Man => crate::names::Gender::Male,
            Sex::Woman => crate::names::Gender::Female,
        };
        let k = text_key(self.key());
        (0..1024u64).map(|n| super::rng::key(&[world_seed, k, 0x4155_5448, n])).find(|&s| super::names::gender(s) == want).unwrap_or_else(|| super::rng::key(&[world_seed, k, 0x4155_5448]))
    }
}

/// A stable number for a text key (FNV-1a): the same in every build.
pub fn text_key(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3))
}

/// A town key from a label: lower-case, words joined by underscores
/// ("Imperial capital" → `imperial_capital`).
pub fn key_of_label(label: &str) -> String {
    let mut out = String::new();
    for w in label.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()) {
        if !out.is_empty() {
            out.push('_');
        }
        out.push_str(&w.to_lowercase());
    }
    out
}

impl super::world::World {
    /// Once the towns' life is founded (`worldgen`): each authored person's
    /// profession and workplace over what was handed out, their honour and
    /// asking over what was rolled, and their household's purse by the
    /// richest of its authored members (`Wealth::days` of its keep).
    pub(super) fn apply_rosters(&mut self) {
        let mut purses: BTreeMap<u32, f32> = BTreeMap::new();
        for p in 0..self.people.len() {
            let Some(a) = self.people[p].authored.clone() else { continue };
            let t = a.traits();
            let job = a.profession();
            let home = self.people[p].home;
            let life = &mut self.society.lives[p];
            if let Some(h) = t.honour {
                life.habits.honour = h.clamp(0.0, 1.0);
            }
            if let Some(x) = t.asks {
                life.habits.asks = x.clamp(0.0, 1.0);
            }
            life.job = job;
            let community = life.community;
            let household = life.household;
            let places = home.map(|h| self.society.towns[h as usize].places.len()).unwrap_or(0);
            let place = match (job, community) {
                (Job::None | Job::Drifter, _) | (_, None) => None,
                (_, Some(ci)) => a.workplace().filter(|&k| (k as usize) < places).or_else(|| self.places_for(ci, job).first().copied()),
            };
            self.society.lives[p].place = place;
            if let Some(h) = household {
                let days = purses.entry(h).or_insert(0.0);
                *days = days.max(a.wealth().days());
            }
        }
        for (h, days) in purses {
            let coin = self.daily_cost(h) * days;
            self.society.households[h as usize].purse.coin = coin;
        }
    }
}
