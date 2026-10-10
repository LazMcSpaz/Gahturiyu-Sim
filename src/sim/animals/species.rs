//! The species table: every animal is one line of data.
//!
//! Nothing here is a real-world animal, and the English names are
//! placeholders. Yields are only names and amounts ("meat 4, hide 1"): what
//! they become (items, goods, prices) is for the society side to decide.

use serde::{Deserialize, Serialize};

use super::super::stealth;
use super::super::tide;
use super::super::world::{DAY, HOUR};

/// Which animal. The order is the order of `SPECIES`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sp {
    // Domestic
    Turiyu,
    Shellhen,
    Mossback,
    Raftback,
    Plodder,
    // Wild prey and grazers
    WildTuriyu,
    Brushleaper,
    Wallowback,
    CragGrazer,
    Dustrunner,
    Tidepicker,
    Silkcrawler,
    // Wild predators and dangers
    Ridgehound,
    ChasmLurker,
    Mirejaw,
    Cragmaw,
    SilkMother,
    Bonepicker,
    Briarback,
    // Sea (numbers only, no bodies in the world)
    Silverling,
    Slatefin,
    Ribbonback,
    Gulpjaw,
    Deepcoil,
}

pub const N_SPECIES: usize = 24;

pub const ALL_SPECIES: [Sp; N_SPECIES] = [
    Sp::Turiyu,
    Sp::Shellhen,
    Sp::Mossback,
    Sp::Raftback,
    Sp::Plodder,
    Sp::WildTuriyu,
    Sp::Brushleaper,
    Sp::Wallowback,
    Sp::CragGrazer,
    Sp::Dustrunner,
    Sp::Tidepicker,
    Sp::Silkcrawler,
    Sp::Ridgehound,
    Sp::ChasmLurker,
    Sp::Mirejaw,
    Sp::Cragmaw,
    Sp::SilkMother,
    Sp::Bonepicker,
    Sp::Briarback,
    Sp::Silverling,
    Sp::Slatefin,
    Sp::Ribbonback,
    Sp::Gulpjaw,
    Sp::Deepcoil,
];

/// The fish that are caught: a number per stretch of coast, nothing more.
pub const FISH: [Sp; 4] = [Sp::Silverling, Sp::Slatefin, Sp::Ribbonback, Sp::Gulpjaw];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Kept by people: lives in a pen with an owner.
    Domestic,
    /// Wild, eaten by others.
    Prey,
    /// Wild, dangerous.
    Predator,
    /// A stock per stretch of coast.
    Fish,
    /// Lives in the sea and is only ever a danger value.
    Sea,
}

/// The kinds of country animals live in. Worked out from the land itself
/// (see `region.rs`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Habitat {
    /// Open hill grassland.
    Grass,
    Scrub,
    /// Woods and their edges.
    Wood,
    /// Low wet ground: coastal marsh and valley bottoms.
    Marsh,
    /// Mountain ledges.
    Mountain,
    /// The high dry plateau.
    Plateau,
    /// The shore and its rock pools.
    Shore,
    /// Steep hill faces.
    Cliff,
    /// Chasm edges and escarpment ledges.
    Ledge,
    /// The central massif.
    Massif,
}

pub const N_HABITATS: usize = 10;
pub const HABITATS: [Habitat; N_HABITATS] = [Habitat::Grass, Habitat::Scrub, Habitat::Wood, Habitat::Marsh, Habitat::Mountain, Habitat::Plateau, Habitat::Shore, Habitat::Cliff, Habitat::Ledge, Habitat::Massif];

impl Habitat {
    pub fn name(self) -> &'static str {
        match self {
            Habitat::Grass => "grassland",
            Habitat::Scrub => "scrub",
            Habitat::Wood => "woodland",
            Habitat::Marsh => "marsh",
            Habitat::Mountain => "mountain",
            Habitat::Plateau => "plateau",
            Habitat::Shore => "shore",
            Habitat::Cliff => "hill cliffs",
            Habitat::Ledge => "ledges",
            Habitat::Massif => "massif",
        }
    }
}

/// When a species is up and about. Read from `stealth::daylight(t)` and the
/// tide clock, never from a schedule of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Active {
    Day,
    /// By day, lying up through the middle of it.
    DayNoonRest,
    Night,
    DawnDusk,
    DuskNight,
    Dawn,
    LowTide,
    Always,
}

/// How a species behaves toward people.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toward {
    /// Calm and slow to flee.
    Calm,
    Bolts,
    /// Ignores people; fights if attacked.
    ChargesIfCornered,
    ClimbsAway,
    Outruns,
    /// Harmless: neither runs nor attacks; nips if handled.
    Harmless,
    /// Attacks the weak and the lone, keeps away from strong groups.
    PackHunter,
    /// Lies in wait and strikes whoever comes within reach.
    Ambusher,
    /// Attacks anything inside its territory.
    Territorial,
    /// Attacks anything nearby.
    Hostile,
    /// Only interested in the dead.
    Scavenger,
    /// Kept by people.
    Tame,
}

impl Toward {
    /// Does this kind ever start a fight with people?
    pub fn dangerous(self) -> bool {
        matches!(self, Toward::PackHunter | Toward::Ambusher | Toward::Territorial | Toward::Hostile)
    }

    /// Runs when attacked rather than fighting back.
    pub fn flees(self) -> bool {
        matches!(self, Toward::Calm | Toward::Bolts | Toward::ClimbsAway | Toward::Outruns)
    }

    pub fn words(self) -> &'static str {
        match self {
            Toward::Calm => "calm, slow to flee",
            Toward::Bolts => "bolts",
            Toward::ChargesIfCornered => "ignores people; charges if cornered",
            Toward::ClimbsAway => "climbs out of reach",
            Toward::Outruns => "outruns everything",
            Toward::Harmless => "harmless",
            Toward::PackHunter => "hunts the weak and the lone",
            Toward::Ambusher => "lies in wait",
            Toward::Territorial => "attacks anything in its territory",
            Toward::Hostile => "attacks anything nearby",
            Toward::Scavenger => "scavenges the dead",
            Toward::Tame => "domestic",
        }
    }
}

/// The stand-in shape drawn for a species: a few plain solids sized to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Build {
    /// A low barrel on short legs (Turiyu).
    Barrel,
    /// A smooth dome on stubby legs (Shellhen).
    Dome,
    /// Tall shoulders, low head (Mossback, Wallowback, Plodder).
    Grazer,
    /// Flat and wide, afloat (Raftback).
    Raft,
    /// Lean, four long legs (Brushleaper, Ridgehound, Crag grazer).
    Runner,
    /// Upright on two legs (Dustrunner, Bonepicker).
    Bird,
    /// Long and low to the ground (Mirejaw, Chasm lurker).
    Lurker,
    /// A heavy mass on thick legs (Cragmaw).
    Hulk,
    /// Many-legged (Silk Mother, Silkcrawler, Tidepicker).
    Crawler,
    /// A spiked hump (Briarback).
    Thorn,
}

#[derive(Clone, Copy, Debug)]
pub struct Looks {
    pub build: Build,
    /// Nose to tail, shoulder height and width, metres.
    pub len: f32,
    pub tall: f32,
    pub wide: f32,
    pub coat: [f32; 3],
    pub mark: [f32; 3],
}

/// What an animal is in a fight. These are read into a `combat::Fighter`,
/// so animals fight under exactly the rules people do.
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub attack: &'static str,
    pub strength: f32,
    pub agility: f32,
    pub toughness: f32,
    /// A multiplier on the health toughness gives.
    pub hp: f32,
    pub cut: f32,
    pub blunt: f32,
    pub reach: f32,
    pub skill: f32,
    pub dodge: f32,
    /// How dangerous one of them is, on the same scale as a person's might.
    pub might: f32,
    /// Breaks off when its health falls below this share (0 = never).
    pub nerve: f32,
    /// Breaks off when the other side is this many times stronger (0 = never).
    pub odds: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Species {
    pub sp: Sp,
    pub key: &'static str,
    pub name: &'static str,
    pub class: Class,
    pub describe: &'static str,
    pub looks: Looks,
    pub habitats: &'static [Habitat],
    /// Smallest and largest group.
    pub group: (u8, u8),
    pub active: Active,
    pub diet: &'static str,
    /// What it hunts (which species' numbers it holds down).
    pub prey: &'static [Sp],
    pub toward: Toward,
    /// Flat-out speed, m/s.
    pub speed: f32,
    pub body: Body,
    /// What one animal gives when killed or culled: names and amounts.
    pub yields: &'static [(&'static str, f32)],
    /// What one well-fed animal gives each day while alive.
    pub daily: &'static [(&'static str, f32)],
    /// Animals per square kilometre of its habitat when the land is full.
    pub density: f32,
    /// Share by which a group grows each day when far below its limit.
    pub growth: f32,
    /// How far from home a group ranges, metres.
    pub range: f32,
    /// How far off it notices people (or how far its strike or territory
    /// reaches), metres.
    pub sense: f32,
    /// How long a hunter shadows its quarry before closing, minutes.
    pub stalk: (f32, f32),
    /// How close people get before it runs, metres.
    pub flight: f32,
    /// Days before an emptied home range is taken up again.
    pub restock_days: f32,
    /// Not to be hunted (a flag for the society side; nothing here enforces it).
    pub protected: bool,
    pub tameable: bool,
    /// Feed one animal needs a day, and the share of it they find for
    /// themselves at pasture. Domestic only.
    pub feed: f32,
    pub forage: f32,
    /// What one can carry, kilograms (pack animals only).
    pub carry: f32,
}

const NO_BODY: Body = Body { attack: "nothing", strength: 5.0, agility: 20.0, toughness: 5.0, hp: 0.2, cut: 0.0, blunt: 0.0, reach: 0.4, skill: 5.0, dodge: 5.0, might: 0.2, nerve: 0.0, odds: 0.0 };

const BLANK: Species = Species {
    sp: Sp::Turiyu,
    key: "",
    name: "",
    class: Class::Prey,
    describe: "",
    looks: Looks { build: Build::Barrel, len: 1.0, tall: 0.5, wide: 0.5, coat: [0.5, 0.5, 0.5], mark: [0.3, 0.3, 0.3] },
    habitats: &[],
    group: (1, 1),
    active: Active::Day,
    diet: "grazes",
    prey: &[],
    toward: Toward::Harmless,
    speed: 3.0,
    body: NO_BODY,
    yields: &[],
    daily: &[],
    density: 0.0,
    growth: 0.04,
    range: 300.0,
    sense: 60.0,
    stalk: (0.0, 0.0),
    flight: 0.0,
    restock_days: 6.0,
    protected: false,
    tameable: false,
    feed: 0.0,
    forage: 0.0,
    carry: 0.0,
};

/// The table. One entry per `Sp`, in the same order.
pub const SPECIES: [Species; N_SPECIES] = [
    // ---- Domestic -------------------------------------------------------------
    Species {
        sp: Sp::Turiyu,
        key: "turiyu",
        name: "Turiyu",
        class: Class::Domestic,
        describe: "Knee-high and neckless: a low barrel body, broad flat snout, clawed toes, a short velvety grey-brown coat with dark ring bands.",
        looks: Looks { build: Build::Barrel, len: 0.95, tall: 0.5, wide: 0.5, coat: [0.46, 0.40, 0.34], mark: [0.22, 0.19, 0.17] },
        group: (4, 14),
        toward: Toward::Tame,
        speed: 3.5,
        body: Body { attack: "claws", strength: 25.0, agility: 30.0, toughness: 30.0, hp: 0.6, cut: 2.0, blunt: 1.0, reach: 0.6, skill: 20.0, dodge: 15.0, might: 4.0, nerve: 0.9, odds: 1.0 },
        yields: &[("meat", 2.0), ("light_hide", 1.0)],
        growth: 0.03,
        feed: 1.0,
        forage: 1.0,
        ..BLANK
    },
    Species {
        sp: Sp::Shellhen,
        key: "shellhen",
        name: "Shellhen",
        class: Class::Domestic,
        describe: "Cat-sized: a smooth cream dome speckled rust, a soft grey face, stubby legs.",
        looks: Looks { build: Build::Dome, len: 0.45, tall: 0.3, wide: 0.38, coat: [0.90, 0.85, 0.72], mark: [0.62, 0.34, 0.22] },
        group: (6, 30),
        toward: Toward::Tame,
        speed: 2.0,
        body: Body { attack: "peck", strength: 5.0, agility: 25.0, toughness: 6.0, hp: 0.2, cut: 0.5, blunt: 0.0, reach: 0.3, skill: 10.0, dodge: 20.0, might: 0.3, nerve: 0.9, odds: 1.0 },
        yields: &[("meat", 0.5), ("shell", 1.0)],
        daily: &[("egg", 1.0)],
        growth: 0.05,
        feed: 0.25,
        forage: 0.3,
        ..BLANK
    },
    Species {
        sp: Sp::Mossback,
        key: "mossback",
        name: "Mossback",
        class: Class::Domestic,
        describe: "A tall-shouldered heavy grazer with a small low-hung head and smooth slate-grey hide, moss growing along its back.",
        looks: Looks { build: Build::Grazer, len: 2.6, tall: 1.7, wide: 1.0, coat: [0.36, 0.40, 0.44], mark: [0.30, 0.46, 0.24] },
        group: (2, 10),
        toward: Toward::Tame,
        speed: 4.0,
        body: Body { attack: "shoulder", strength: 70.0, agility: 20.0, toughness: 75.0, hp: 1.8, cut: 0.0, blunt: 8.0, reach: 1.1, skill: 25.0, dodge: 5.0, might: 20.0, nerve: 0.3, odds: 3.0 },
        yields: &[("meat", 10.0), ("hide", 2.0)],
        daily: &[("milk", 3.0)],
        growth: 0.015,
        feed: 4.0,
        forage: 0.6,
        ..BLANK
    },
    Species {
        sp: Sp::Raftback,
        key: "raftback",
        name: "Raftback",
        class: Class::Domestic,
        describe: "Huge and flat, a living raft on the water: a shell-crusted rubbery back with a frilled edge.",
        looks: Looks { build: Build::Raft, len: 9.0, tall: 0.7, wide: 6.0, coat: [0.30, 0.36, 0.38], mark: [0.70, 0.68, 0.60] },
        group: (1, 3),
        toward: Toward::Tame,
        speed: 1.0,
        body: Body { attack: "nothing", strength: 60.0, agility: 5.0, toughness: 95.0, hp: 3.0, cut: 0.0, blunt: 0.0, reach: 0.5, skill: 5.0, dodge: 0.0, might: 6.0, nerve: 0.0, odds: 0.0 },
        yields: &[("oil", 12.0), ("meat", 20.0)],
        daily: &[("oil", 0.4)],
        growth: 0.004,
        feed: 6.0,
        forage: 1.0,
        ..BLANK
    },
    Species {
        sp: Sp::Plodder,
        key: "plodder",
        name: "Plodder",
        class: Class::Domestic,
        describe: "Horse-sized, with a flat wedge head carried low, wide splayed toes and wrinkled ochre hide.",
        looks: Looks { build: Build::Grazer, len: 2.4, tall: 1.5, wide: 0.9, coat: [0.72, 0.56, 0.30], mark: [0.52, 0.40, 0.22] },
        group: (2, 8),
        toward: Toward::Tame,
        speed: 6.0,
        body: Body { attack: "kick", strength: 80.0, agility: 30.0, toughness: 60.0, hp: 1.5, cut: 0.0, blunt: 9.0, reach: 1.1, skill: 30.0, dodge: 10.0, might: 18.0, nerve: 0.4, odds: 2.5 },
        yields: &[("meat", 8.0), ("hide", 2.0)],
        growth: 0.01,
        feed: 3.0,
        forage: 0.5,
        carry: 120.0,
        ..BLANK
    },
    // ---- Wild prey and grazers ------------------------------------------------
    Species {
        sp: Sp::WildTuriyu,
        key: "wild_turiyu",
        name: "Wild Turiyu",
        class: Class::Prey,
        describe: "Knee-high and neckless: a low barrel body, broad flat snout, clawed toes, a short velvety grey-brown coat with dark ring bands.",
        looks: Looks { build: Build::Barrel, len: 0.95, tall: 0.5, wide: 0.5, coat: [0.42, 0.36, 0.30], mark: [0.20, 0.17, 0.15] },
        habitats: &[Habitat::Grass],
        group: (6, 15),
        active: Active::Day,
        toward: Toward::Calm,
        speed: 3.5,
        body: Body { attack: "claws", strength: 25.0, agility: 30.0, toughness: 30.0, hp: 0.6, cut: 2.0, blunt: 1.0, reach: 0.6, skill: 20.0, dodge: 15.0, might: 4.0, nerve: 0.9, odds: 1.0 },
        density: 3.4,
        growth: 0.03,
        range: 350.0,
        sense: 40.0,
        flight: 14.0,
        protected: true,
        ..BLANK
    },
    Species {
        sp: Sp::Brushleaper,
        key: "brushleaper",
        name: "Brushleaper",
        class: Class::Prey,
        describe: "A slight, long-legged browser of the scrub and wood edge, all spring and nerves.",
        looks: Looks { build: Build::Runner, len: 1.3, tall: 0.95, wide: 0.35, coat: [0.58, 0.47, 0.32], mark: [0.86, 0.80, 0.66] },
        habitats: &[Habitat::Scrub, Habitat::Wood, Habitat::Grass],
        group: (3, 6),
        active: Active::DawnDusk,
        toward: Toward::Bolts,
        speed: 9.0,
        body: Body { attack: "kick", strength: 20.0, agility: 60.0, toughness: 20.0, hp: 0.5, cut: 0.0, blunt: 2.0, reach: 0.8, skill: 20.0, dodge: 55.0, might: 4.0, nerve: 0.9, odds: 1.0 },
        yields: &[("meat", 3.0), ("light_hide", 1.0)],
        density: 2.4,
        growth: 0.05,
        range: 450.0,
        sense: 90.0,
        flight: 70.0,
        ..BLANK
    },
    Species {
        sp: Sp::Wallowback,
        key: "wallowback",
        name: "Wallowback",
        class: Class::Prey,
        describe: "A great mud-caked grazer of the wet ground, slow to anger and hard to stop.",
        looks: Looks { build: Build::Grazer, len: 2.9, tall: 1.6, wide: 1.3, coat: [0.38, 0.32, 0.27], mark: [0.26, 0.22, 0.19] },
        habitats: &[Habitat::Marsh],
        group: (8, 20),
        active: Active::DayNoonRest,
        toward: Toward::ChargesIfCornered,
        speed: 5.0,
        body: Body { attack: "tusks", strength: 75.0, agility: 20.0, toughness: 70.0, hp: 1.6, cut: 3.0, blunt: 12.0, reach: 1.3, skill: 35.0, dodge: 8.0, might: 30.0, nerve: 0.25, odds: 4.0 },
        yields: &[("meat", 14.0), ("thick_hide", 2.0)],
        density: 6.0,
        growth: 0.02,
        range: 300.0,
        sense: 30.0,
        ..BLANK
    },
    Species {
        sp: Sp::CragGrazer,
        key: "crag_grazer",
        name: "Crag grazer",
        class: Class::Prey,
        describe: "A sure-footed climber armoured in overlapping horn plates.",
        looks: Looks { build: Build::Runner, len: 1.5, tall: 1.0, wide: 0.5, coat: [0.50, 0.48, 0.45], mark: [0.24, 0.22, 0.20] },
        habitats: &[Habitat::Mountain, Habitat::Massif],
        group: (2, 4),
        active: Active::Day,
        toward: Toward::ClimbsAway,
        speed: 6.0,
        body: Body { attack: "horns", strength: 40.0, agility: 50.0, toughness: 40.0, hp: 0.8, cut: 0.0, blunt: 6.0, reach: 0.9, skill: 30.0, dodge: 40.0, might: 9.0, nerve: 0.8, odds: 1.2 },
        yields: &[("horn_plate", 4.0), ("meat", 3.0)],
        density: 0.9,
        growth: 0.03,
        range: 400.0,
        sense: 110.0,
        flight: 60.0,
        ..BLANK
    },
    Species {
        sp: Sp::Dustrunner,
        key: "dustrunner",
        name: "Dustrunner",
        class: Class::Prey,
        describe: "A tall two-legged runner of the dry plateau, in flocks that move like blown dust.",
        looks: Looks { build: Build::Bird, len: 1.1, tall: 1.4, wide: 0.4, coat: [0.80, 0.68, 0.48], mark: [0.50, 0.36, 0.24] },
        habitats: &[Habitat::Plateau],
        group: (10, 30),
        active: Active::DayNoonRest,
        toward: Toward::Outruns,
        speed: 11.0,
        body: Body { attack: "peck", strength: 15.0, agility: 70.0, toughness: 15.0, hp: 0.45, cut: 2.0, blunt: 1.0, reach: 0.7, skill: 20.0, dodge: 60.0, might: 3.0, nerve: 0.9, odds: 1.0 },
        yields: &[("meat", 2.0), ("tough_skin", 1.0)],
        density: 4.5,
        growth: 0.05,
        range: 700.0,
        sense: 160.0,
        flight: 120.0,
        ..BLANK
    },
    Species {
        sp: Sp::Tidepicker,
        key: "tidepicker",
        name: "Tidepicker",
        class: Class::Prey,
        describe: "Hand-sized scuttlers that pour out over the rock pools when the water drops.",
        looks: Looks { build: Build::Crawler, len: 0.22, tall: 0.1, wide: 0.2, coat: [0.72, 0.42, 0.34], mark: [0.90, 0.80, 0.70] },
        habitats: &[Habitat::Shore],
        group: (20, 60),
        active: Active::LowTide,
        toward: Toward::Harmless,
        speed: 3.0,
        body: Body { attack: "nip", strength: 5.0, agility: 40.0, toughness: 3.0, hp: 0.15, cut: 1.0, blunt: 0.0, reach: 0.4, skill: 15.0, dodge: 35.0, might: 0.4, nerve: 0.0, odds: 0.0 },
        yields: &[("small_meat", 0.2), ("egg", 0.2)],
        density: 550.0,
        growth: 0.12,
        range: 160.0,
        sense: 10.0,
        ..BLANK
    },
    Species {
        sp: Sp::Silkcrawler,
        key: "silkcrawler",
        name: "Silkcrawler",
        class: Class::Prey,
        describe: "Pale finger-long spinners that hang their cocoons in hundreds on the hill cliffs.",
        looks: Looks { build: Build::Crawler, len: 0.14, tall: 0.06, wide: 0.1, coat: [0.88, 0.86, 0.78], mark: [0.60, 0.58, 0.50] },
        habitats: &[Habitat::Cliff],
        group: (40, 80),
        active: Active::Day,
        toward: Toward::Harmless,
        speed: 1.2,
        body: Body { attack: "nip", strength: 4.0, agility: 20.0, toughness: 3.0, hp: 0.15, cut: 0.5, blunt: 0.0, reach: 0.3, skill: 10.0, dodge: 10.0, might: 0.2, nerve: 0.0, odds: 0.0 },
        yields: &[],
        density: 0.0,
        growth: 0.08,
        range: 14.0,
        sense: 5.0,
        ..BLANK
    },
    // ---- Wild predators and dangers -------------------------------------------
    Species {
        sp: Sp::Ridgehound,
        key: "ridgehound",
        name: "Ridgehound",
        class: Class::Predator,
        describe: "A rangy pack hunter of the hills that follows herds and roads after dark.",
        looks: Looks { build: Build::Runner, len: 1.35, tall: 0.75, wide: 0.4, coat: [0.30, 0.27, 0.25], mark: [0.62, 0.52, 0.38] },
        habitats: &[Habitat::Grass, Habitat::Wood, Habitat::Scrub],
        group: (4, 7),
        active: Active::DuskNight,
        diet: "runs down Brushleapers and Wallowback young",
        prey: &[Sp::Brushleaper, Sp::Wallowback],
        toward: Toward::PackHunter,
        speed: 7.5,
        body: Body { attack: "bite", strength: 40.0, agility: 55.0, toughness: 35.0, hp: 0.7, cut: 6.0, blunt: 1.5, reach: 0.9, skill: 42.0, dodge: 38.0, might: 14.0, nerve: 0.4, odds: 1.8 },
        yields: &[("hide", 1.0), ("teeth", 4.0)],
        density: 0.42,
        growth: 0.02,
        range: 900.0,
        sense: 170.0,
        stalk: (5.0, 20.0),
        flight: 45.0,
        restock_days: 10.0,
        tameable: true,
        ..BLANK
    },
    Species {
        sp: Sp::ChasmLurker,
        key: "chasm_lurker",
        name: "Chasm lurker",
        class: Class::Predator,
        describe: "A long, flat-bodied night hunter that clings under a ledge until something walks past above.",
        looks: Looks { build: Build::Lurker, len: 3.2, tall: 0.6, wide: 0.9, coat: [0.20, 0.20, 0.24], mark: [0.50, 0.46, 0.56] },
        habitats: &[Habitat::Ledge],
        group: (1, 1),
        active: Active::Night,
        diet: "whatever passes the ledge",
        toward: Toward::Ambusher,
        speed: 6.0,
        body: Body { attack: "hooked claws", strength: 65.0, agility: 50.0, toughness: 55.0, hp: 1.1, cut: 13.0, blunt: 4.0, reach: 1.6, skill: 60.0, dodge: 30.0, might: 42.0, nerve: 0.3, odds: 3.0 },
        yields: &[("lurker_trophy", 1.0)],
        density: 0.16,
        growth: 0.0,
        range: 25.0,
        sense: 18.0,
        restock_days: 20.0,
        ..BLANK
    },
    Species {
        sp: Sp::Mirejaw,
        key: "mirejaw",
        name: "Mirejaw",
        class: Class::Predator,
        describe: "It lies submerged at the water's edge with only its eyes showing, and takes whatever comes to drink or cross.",
        looks: Looks { build: Build::Lurker, len: 3.8, tall: 0.5, wide: 1.1, coat: [0.22, 0.30, 0.22], mark: [0.36, 0.42, 0.28] },
        habitats: &[Habitat::Marsh],
        group: (1, 1),
        active: Active::Dawn,
        diet: "drinkers and crossers",
        toward: Toward::Ambusher,
        speed: 4.0,
        body: Body { attack: "jaws", strength: 80.0, agility: 30.0, toughness: 65.0, hp: 1.3, cut: 16.0, blunt: 6.0, reach: 1.4, skill: 50.0, dodge: 10.0, might: 45.0, nerve: 0.3, odds: 3.0 },
        yields: &[("thick_hide", 2.0)],
        density: 0.6,
        growth: 0.0,
        range: 20.0,
        sense: 14.0,
        restock_days: 14.0,
        ..BLANK
    },
    Species {
        sp: Sp::Cragmaw,
        key: "cragmaw",
        name: "Cragmaw",
        class: Class::Predator,
        describe: "A mountain of plated muscle that holds a stretch of the massif against everything.",
        looks: Looks { build: Build::Hulk, len: 4.2, tall: 2.6, wide: 2.2, coat: [0.34, 0.33, 0.36], mark: [0.62, 0.78, 0.86] },
        habitats: &[Habitat::Massif],
        group: (1, 1),
        active: Active::Day,
        diet: "anything that trespasses",
        toward: Toward::Territorial,
        speed: 6.5,
        body: Body { attack: "maw", strength: 95.0, agility: 35.0, toughness: 95.0, hp: 2.2, cut: 14.0, blunt: 18.0, reach: 2.0, skill: 65.0, dodge: 12.0, might: 110.0, nerve: 0.0, odds: 0.0 },
        yields: &[("cragmaw_trophy", 1.0)],
        density: 0.0,
        growth: 0.0,
        range: 120.0,
        sense: 160.0,
        restock_days: 40.0,
        ..BLANK
    },
    Species {
        sp: Sp::SilkMother,
        key: "silk_mother",
        name: "Silk Mother",
        class: Class::Predator,
        describe: "The one great spinner at the heart of every Silkcrawler colony.",
        looks: Looks { build: Build::Crawler, len: 2.2, tall: 1.1, wide: 2.0, coat: [0.80, 0.78, 0.70], mark: [0.36, 0.24, 0.30] },
        habitats: &[Habitat::Cliff],
        group: (1, 1),
        active: Active::Always,
        diet: "what blunders into the silk",
        toward: Toward::Territorial,
        speed: 6.0,
        body: Body { attack: "fangs", strength: 60.0, agility: 50.0, toughness: 60.0, hp: 1.4, cut: 12.0, blunt: 2.0, reach: 1.5, skill: 58.0, dodge: 30.0, might: 50.0, nerve: 0.0, odds: 0.0 },
        yields: &[("silk_gland", 1.0)],
        density: 0.0,
        growth: 0.0,
        range: 10.0,
        sense: 16.0,
        restock_days: 30.0,
        ..BLANK
    },
    Species {
        sp: Sp::Bonepicker,
        key: "bonepicker",
        name: "Bonepicker",
        class: Class::Predator,
        describe: "Bald-headed carrion fliers that find a body within the hour and leave nothing.",
        looks: Looks { build: Build::Bird, len: 0.7, tall: 0.6, wide: 0.35, coat: [0.16, 0.15, 0.17], mark: [0.80, 0.52, 0.44] },
        group: (5, 12),
        active: Active::Day,
        diet: "the dead",
        toward: Toward::Scavenger,
        speed: 9.0,
        body: Body { attack: "beak", strength: 12.0, agility: 55.0, toughness: 8.0, hp: 0.3, cut: 3.0, blunt: 0.0, reach: 0.6, skill: 30.0, dodge: 50.0, might: 2.0, nerve: 0.9, odds: 1.0 },
        ..BLANK
    },
    Species {
        sp: Sp::Briarback,
        key: "briarback",
        name: "Briarback",
        class: Class::Predator,
        describe: "A hunched thing grown through with thorn and vine, wherever the Overgrowth runs high.",
        looks: Looks { build: Build::Thorn, len: 1.8, tall: 1.2, wide: 1.1, coat: [0.20, 0.34, 0.16], mark: [0.52, 0.20, 0.34] },
        group: (2, 4),
        active: Active::Always,
        diet: "anything",
        toward: Toward::Hostile,
        speed: 5.5,
        body: Body { attack: "thorns", strength: 55.0, agility: 35.0, toughness: 50.0, hp: 1.0, cut: 10.0, blunt: 3.0, reach: 1.1, skill: 45.0, dodge: 20.0, might: 28.0, nerve: 0.0, odds: 0.0 },
        yields: &[("briar_heart", 1.0)],
        growth: 0.1,
        range: 250.0,
        sense: 70.0,
        restock_days: 3.0,
        ..BLANK
    },
    // ---- Sea ------------------------------------------------------------------
    Species { sp: Sp::Silverling, key: "silverling", name: "Silverling", class: Class::Fish, describe: "The cheap shoal fish of every shore.", yields: &[("fish", 1.0)], density: 900.0, growth: 0.20, ..BLANK },
    Species { sp: Sp::Slatefin, key: "slatefin", name: "Slatefin", class: Class::Fish, describe: "A firm grey fish of the rocky shallows.", yields: &[("fish", 2.0)], density: 300.0, growth: 0.12, ..BLANK },
    Species { sp: Sp::Ribbonback, key: "ribbonback", name: "Ribbonback", class: Class::Fish, describe: "Long and banded, found only in the kelp beds; prized.", yields: &[("prized_fish", 1.0)], density: 120.0, growth: 0.08, ..BLANK },
    Species { sp: Sp::Gulpjaw, key: "gulpjaw", name: "Gulpjaw", class: Class::Fish, describe: "A rare wide-mouthed fish of the deep water.", yields: &[("fish", 8.0)], density: 25.0, growth: 0.04, ..BLANK },
    Species { sp: Sp::Deepcoil, key: "deepcoil", name: "Deepcoil", class: Class::Sea, describe: "Something long that keeps to the offshore deeps and the shellbeds.", ..BLANK },
];

impl Sp {
    pub fn def(self) -> &'static Species {
        &SPECIES[self as usize]
    }

    pub fn name(self) -> &'static str {
        self.def().name
    }
}

/// The species with this key ("ridgehound"), if there is one.
pub fn species(key: &str) -> Option<Sp> {
    SPECIES.iter().find(|s| s.key == key).map(|s| s.sp)
}

// ---- The time of day ----------------------------------------------------------

/// The part of the day, read off the sunlight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Night,
    Dawn,
    Day,
    Dusk,
}

pub fn phase(t: f64) -> Phase {
    let l = stealth::daylight(t);
    if l >= 0.999 {
        Phase::Day
    } else if l <= stealth::daylight(0.0) + 1e-4 {
        Phase::Night
    } else if t.rem_euclid(DAY) < 12.0 * HOUR {
        Phase::Dawn
    } else {
        Phase::Dusk
    }
}

/// How far out the tide has to be for the shore's animals to come out
/// (`tide::level`: −1 is dead low water).
pub const LOW_WATER: f32 = -0.45;

impl Species {
    /// More than one of them. (A Turiyu's name doesn't take an s.)
    pub fn plural(&self) -> String {
        if self.name.ends_with("Turiyu") {
            self.name.to_string()
        } else {
            format!("{}s", self.name)
        }
    }

    /// "the Chasm lurker" for one, "the Brushleapers" for more (BL-73).
    pub fn the(&self, n: usize) -> String {
        if n == 1 {
            format!("the {}", self.name)
        } else {
            format!("the {}", self.plural())
        }
    }

    /// "a Cragmaw", "10 Tidepickers".
    pub fn counted(&self, n: usize) -> String {
        if n == 1 {
            let an = matches!(self.name.chars().next(), Some('A' | 'E' | 'I' | 'O' | 'U'));
            format!("{} {}", if an { "an" } else { "a" }, self.name)
        } else {
            format!("{n} {}", self.plural())
        }
    }

    /// How much there is of one to cut up, against a middling beast (a
    /// hand-sized Tidepicker is a tenth of the work of a Mossback: BL-57).
    pub fn bulk(&self) -> f32 {
        (self.looks.len / 2.0).clamp(0.1, 2.0)
    }

    /// The species of this name, if any.
    pub fn named(name: &str) -> Option<&'static Species> {
        SPECIES.iter().find(|s| s.name == name)
    }
}

impl Active {
    /// Is a species with these hours up and about at time `t`?
    pub fn at(self, t: f64) -> bool {
        let p = phase(t);
        match self {
            Active::Day => p == Phase::Day,
            Active::DayNoonRest => {
                let h = t.rem_euclid(DAY) / HOUR;
                p == Phase::Day && !(11.0..13.0).contains(&h)
            }
            Active::Night => p == Phase::Night,
            Active::DawnDusk => matches!(p, Phase::Dawn | Phase::Dusk),
            Active::DuskNight => matches!(p, Phase::Dusk | Phase::Night),
            Active::Dawn => p == Phase::Dawn,
            Active::LowTide => tide::level(t) < LOW_WATER,
            Active::Always => true,
        }
    }

    pub fn words(self) -> &'static str {
        match self {
            Active::Day => "by day",
            Active::DayNoonRest => "by day, resting at noon",
            Active::Night => "by night",
            Active::DawnDusk => "at dawn and dusk",
            Active::DuskNight => "from dusk through the night",
            Active::Dawn => "mostly at dawn",
            Active::LowTide => "at low tide",
            Active::Always => "at all hours",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_lines_up_with_the_names() {
        for (i, s) in SPECIES.iter().enumerate() {
            assert_eq!(s.sp as usize, i, "{} is out of place", s.name);
            assert_eq!(ALL_SPECIES[i], s.sp);
            assert!(!s.key.is_empty() && !s.name.is_empty());
            assert!(s.group.0 >= 1 && s.group.0 <= s.group.1);
        }
        assert_eq!(species("ridgehound"), Some(Sp::Ridgehound));
    }

    #[test]
    fn hours_follow_the_sun_and_the_tide() {
        let noon = 12.0 * HOUR;
        let midnight = 0.0;
        assert!(Active::Day.at(noon) && !Active::Day.at(midnight));
        assert!(!Active::DayNoonRest.at(noon) && Active::DayNoonRest.at(9.0 * HOUR));
        assert!(Active::DuskNight.at(midnight) && Active::DuskNight.at(20.0 * HOUR) && !Active::DuskNight.at(noon));
        assert!(Active::Dawn.at(6.0 * HOUR) && !Active::Dawn.at(20.0 * HOUR));
        assert!(Active::LowTide.at(tide::low_near(3.0 * DAY)) && !Active::LowTide.at(tide::next_high(3.0 * DAY)));
    }
}
