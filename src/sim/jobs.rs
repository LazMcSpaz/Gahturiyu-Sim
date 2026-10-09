//! Jobs, the places people work, the goods work makes, and the services a
//! town offers — as data tables. (Rates are placeholders to tune.)

use serde::{Deserialize, Serialize};

use super::crafting::Station;
use super::stats::Calling;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Job {
    /// No place in a town's work: the squad, bandits, wanderers.
    None,
    Farmer,
    /// Fisher or diver.
    Fisher,
    /// Forager or hunter.
    Forager,
    /// Woodcutter or quarrier.
    Woodcutter,
    StoneTender,
    Cook,
    Runner,
    Merchant,
    Caravaner,
    Innkeeper,
    /// Guard; `Life::shift` 1 is the night watch.
    Guard,
    Official,
    Priest,
    Healer,
    Teacher,
    Exchanger,
    Arbiter,
    /// Labourer or porter.
    Labourer,
    Drifter,
    KelpGatherer,
    Boatwright,
    // The four crafters that already have stations. (Part 2 adds the
    // other crafts.)
    Smith,
    Armourer,
    Alchemist,
    Scribe,
    // Part 2: the other crafts.
    /// Reed, shell, fishskin and tentsilk; sealing with pitch.
    Weaver,
    /// Hides into leather.
    Tanner,
    Leatherworker,
    Tailor,
    Woodworker,
    /// Wood into charcoal (and ash).
    CharcoalBurner,
}

pub const ALL_JOBS: [Job; 31] = [
    Job::Farmer,
    Job::Fisher,
    Job::Forager,
    Job::Woodcutter,
    Job::StoneTender,
    Job::Cook,
    Job::Runner,
    Job::Merchant,
    Job::Caravaner,
    Job::Innkeeper,
    Job::Guard,
    Job::Official,
    Job::Priest,
    Job::Healer,
    Job::Teacher,
    Job::Exchanger,
    Job::Arbiter,
    Job::Labourer,
    Job::Drifter,
    Job::KelpGatherer,
    Job::Boatwright,
    Job::Smith,
    Job::Armourer,
    Job::Alchemist,
    Job::Scribe,
    Job::Weaver,
    Job::Tanner,
    Job::Leatherworker,
    Job::Tailor,
    Job::Woodworker,
    Job::CharcoalBurner,
];

impl Job {
    pub fn name(self) -> &'static str {
        match self {
            Job::None => "No trade",
            Job::Farmer => "Farmer",
            Job::Fisher => "Fisher and diver",
            Job::Forager => "Forager and hunter",
            Job::Woodcutter => "Woodcutter and quarrier",
            Job::StoneTender => "Stone Tender",
            Job::Cook => "Cook",
            Job::Runner => "Meal runner",
            Job::Merchant => "Merchant",
            Job::Caravaner => "Caravaner",
            Job::Innkeeper => "Innkeeper",
            Job::Guard => "Guard",
            Job::Official => "Official",
            Job::Priest => "Priest",
            Job::Healer => "Healer",
            Job::Teacher => "Teacher",
            Job::Exchanger => "Exchanger",
            Job::Arbiter => "Arbiter",
            Job::Labourer => "Labourer and porter",
            Job::Drifter => "Drifter",
            Job::KelpGatherer => "Kelp gatherer",
            Job::Boatwright => "Boatwright",
            Job::Smith => "Smith",
            Job::Armourer => "Armourer",
            Job::Alchemist => "Alchemist",
            Job::Scribe => "Scribe",
            Job::Weaver => "Weaver and sealer",
            Job::Tanner => "Tanner",
            Job::Leatherworker => "Leatherworker",
            Job::Tailor => "Tailor",
            Job::Woodworker => "Woodworker",
            Job::CharcoalBurner => "Charcoal burner",
        }
    }

    /// How light the work is (who gets the garden): 0 heavy … 1 light.
    pub fn lightness(self) -> f32 {
        match self {
            Job::Drifter | Job::None => 1.0,
            Job::Scribe | Job::Teacher | Job::Exchanger | Job::Arbiter | Job::Official => 0.8,
            Job::Priest | Job::Merchant | Job::Healer | Job::Alchemist => 0.7,
            Job::Runner | Job::Cook | Job::Innkeeper | Job::StoneTender => 0.5,
            Job::KelpGatherer | Job::Labourer | Job::Caravaner => 0.35,
            Job::Guard | Job::Boatwright | Job::Smith | Job::Armourer | Job::CharcoalBurner | Job::Tanner => 0.3,
            Job::Weaver | Job::Leatherworker | Job::Tailor | Job::Woodworker => 0.6,
            Job::Farmer | Job::Forager | Job::Fisher | Job::Woodcutter => 0.2,
        }
    }

    /// Someone whose loss leaves a gap a town fills from its labourers.
    pub fn is_post(self) -> bool {
        !matches!(self, Job::None | Job::Labourer | Job::Drifter)
    }

    /// What an hour of this work brings into the town's store, given what
    /// the land round town offers (`src`, by good, 0..1) and whether it's
    /// worked from the stilts. (Crafters and charcoal burners turn one thing
    /// into another instead; see `making.rs`.)
    pub fn gathers(self, src: &[f32], offshore: bool, at: Option<PlaceKind>) -> Vec<(Good, f32)> {
        let s = |g: Good| src.get(g.index()).copied().unwrap_or(0.0);
        match self {
            Job::Farmer => vec![(Good::Grain, FARM_PER_HOUR), (Good::Fibre, 0.12)],
            Job::Fisher => {
                if offshore {
                    // Divers: the catch goes by the dawn boats; shell, pearl
                    // and what lies on the seabed go into the store.
                    vec![(Good::Fishskin, 0.06), (Good::Nacre, 0.05 * s(Good::Nacre)), (Good::Pearl, 0.004 * s(Good::Pearl)), (Good::Salvage, 0.015 * s(Good::Salvage))]
                } else {
                    vec![(Good::Fish, FISH_PER_HOUR), (Good::Fishskin, 0.06)]
                }
            }
            Job::KelpGatherer => vec![(Good::Kelp, KELP_PER_HOUR), (Good::Seareed, 0.3 * s(Good::Seareed).max(0.3))],
            Job::Forager => vec![(Good::Game, 0.3), (Good::Hides, 0.06), (Good::Herbs, 0.05), (Good::Tentsilk, 0.03 * s(Good::Tentsilk))],
            // Woodcutters and quarriers split their hours over what the land offers.
            // (At the woodlot only wood and resin; at the mine only stone, ore
            // and veins; with neither, whatever the land offers.)
            Job::Woodcutter => {
                let here = |g: Good| match at {
                    Some(PlaceKind::Woodlot) => matches!(g, Good::Timber | Good::Pitch),
                    Some(PlaceKind::Quarry) => !matches!(g, Good::Timber | Good::Pitch),
                    _ => true,
                };
                let total: f32 = DIG.iter().filter(|d| here(d.0)).map(|d| s(d.0)).sum();
                if total <= 0.0 {
                    return vec![(Good::Timber, 0.6)];
                }
                DIG.iter().filter(|d| here(d.0) && s(d.0) > 0.0).map(|&(g, r)| (g, r * s(g) / total)).collect()
            }
            Job::Boatwright => vec![(Good::Wares, 0.12)],
            Job::Labourer if offshore => vec![(Good::Kelp, KELP_PER_HOUR * 0.4)],
            Job::Labourer => vec![(Good::Rock, 0.15 * s(Good::Rock).max(0.3)), (Good::Clay, 0.05 * s(Good::Clay))],
            _ => vec![],
        }
    }

    /// The service the squad can use while this worker is at work.
    pub fn service(self) -> Option<Service> {
        Some(match self {
            Job::Merchant => Service::Trade,
            Job::Innkeeper => Service::Lodging,
            Job::Healer => Service::Healing,
            Job::Alchemist => Service::Alchemy,
            Job::Smith => Service::Smithing,
            Job::Armourer => Service::Armoury,
            Job::Scribe => Service::Letters,
            Job::Exchanger => Service::Exchange,
            Job::Teacher => Service::Teaching,
            Job::Priest => Service::Shrine,
            Job::Official | Job::Arbiter => Service::Hall,
            Job::Cook => Service::Meals,
            Job::Guard => Service::Watch,
            Job::Weaver => Service::Weaving,
            Job::Tanner | Job::Leatherworker | Job::Tailor | Job::Woodworker => Service::Basics,
            Job::StoneTender => Service::Growing,
            _ => return None,
        })
    }

    /// The craft this job works (and can mend), if any.
    pub fn craft(self) -> Option<super::materials::Craft> {
        use super::materials::Craft as C;
        Some(match self {
            Job::Smith => C::Smithing,
            Job::Armourer => C::Armoring,
            Job::StoneTender => C::Tending,
            Job::Weaver => C::Weaving,
            Job::Tanner | Job::Leatherworker | Job::Tailor | Job::Woodworker => C::Handcraft,
            Job::Scribe => C::Inscription,
            Job::Alchemist => C::Alchemy,
            _ => return None,
        })
    }

    /// How well someone's calling and temper suit the work (before their
    /// people's leaning is applied).
    pub fn fit(self, calling: Calling, t: &super::race::Traits) -> f32 {
        let c = match (self, calling) {
            (Job::Guard, Calling::Warrior) => 5.0,
            (Job::Caravaner, Calling::Warrior) => 1.5,
            (Job::Forager, Calling::Hunter) => 5.0,
            (Job::Fisher, Calling::Hunter) => 2.0,
            (Job::Guard, Calling::Hunter) => 1.5,
            (Job::Healer | Job::Teacher | Job::Alchemist, Calling::Mage) => 4.0,
            (Job::Priest | Job::Scribe, Calling::Mage) => 2.5,
            (Job::Guard, Calling::Mage) => 0.4,
            (Job::Guard, Calling::Common) => 0.35,
            _ => 1.0,
        };
        let s = match self {
            Job::Merchant | Job::Innkeeper | Job::Runner | Job::Official => 0.4 + t.sociability * 1.5,
            Job::Caravaner | Job::Drifter => 0.2 + t.wanderlust * 2.0,
            Job::Farmer | Job::StoneTender | Job::Teacher | Job::Scribe | Job::Arbiter => 0.4 + t.patience * 1.4,
            Job::Guard | Job::Fisher | Job::Woodcutter => 0.4 + t.boldness * 1.4,
            _ => 1.0,
        };
        // The restless keep away from work that ties them to one spot.
        let roots = match self {
            Job::Farmer | Job::StoneTender | Job::Innkeeper | Job::Official => 1.3 - t.wanderlust,
            _ => 1.0,
        };
        c * s * roots.max(0.1)
    }
}

// --- Work rates (placeholders) -----------------------------------------------

/// Grain from an hour in the fields, in person-days of food.
pub const FARM_PER_HOUR: f32 = 0.5;
/// Catch from an hour at sea.
pub const FISH_PER_HOUR: f32 = 0.7;
pub const KELP_PER_HOUR: f32 = 0.5;
/// Meals one cook turns out per hour at work, person-days.
pub const MEALS_PER_COOK_HOUR: f32 = 5.5;
/// Meal pots one runner carries out on a midday round, person-days.
pub const POTS_PER_RUN: f32 = 30.0;
/// What a woodcutter or quarrier brings in an hour of each, where the land
/// offers it fully (their hours split by how much each is on offer).
pub const DIG: [(Good, f32); 8] = [
    (Good::Timber, 0.6),
    (Good::Rock, 0.6),
    (Good::Ore, 0.8),
    (Good::Clay, 0.5),
    (Good::Sand, 0.6),
    (Good::Pitch, 0.15),
    (Good::Gold, 0.02),
    (Good::EdgeSeed, 0.01),
];

// --- Goods -------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Good {
    Grain,
    Fish,
    Kelp,
    Game,
    Timber,
    /// Rock feedstock: building stone, and what the Tenders grow stone from.
    Rock,
    Hides,
    Herbs,
    /// Made things sold by the piece: torches, draughts, pots, arrows.
    Wares,
    // Raw materials (Part 2).
    Ore,
    Charcoal,
    Ash,
    Clay,
    Sand,
    Gold,
    EdgeSeed,
    Seareed,
    Pitch,
    Nacre,
    Pearl,
    Fishskin,
    Salvage,
    Tentsilk,
    Fibre,
    // Worked materials.
    Leather,
    Cloth,
    /// Forgeiron ingots.
    Ingots,
    Bronze,
    Paper,
    Ink,
    // Grown stone, from the Tenders' beds.
    Ringstone,
    Slatewing,
    Edgeglass,
    Hearthclay,
}

pub const N_GOODS: usize = 34;
pub const GOODS: [Good; N_GOODS] = [
    Good::Grain,
    Good::Fish,
    Good::Kelp,
    Good::Game,
    Good::Timber,
    Good::Rock,
    Good::Hides,
    Good::Herbs,
    Good::Wares,
    Good::Ore,
    Good::Charcoal,
    Good::Ash,
    Good::Clay,
    Good::Sand,
    Good::Gold,
    Good::EdgeSeed,
    Good::Seareed,
    Good::Pitch,
    Good::Nacre,
    Good::Pearl,
    Good::Fishskin,
    Good::Salvage,
    Good::Tentsilk,
    Good::Fibre,
    Good::Leather,
    Good::Cloth,
    Good::Ingots,
    Good::Bronze,
    Good::Paper,
    Good::Ink,
    Good::Ringstone,
    Good::Slatewing,
    Good::Edgeglass,
    Good::Hearthclay,
];
/// Food goods, in the order they're eaten (what spoils first, first).
pub const FOODS: [Good; 4] = [Good::Kelp, Good::Fish, Good::Game, Good::Grain];

impl Good {
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Good::Grain => "Grain and greens",
            Good::Fish => "Fish",
            Good::Kelp => "Kelp",
            Good::Game => "Game",
            Good::Timber => "Timber",
            Good::Rock => "Rock",
            Good::Hides => "Hides",
            Good::Herbs => "Herbs",
            Good::Wares => "Wares",
            Good::Ore => "Ore",
            Good::Charcoal => "Charcoal",
            Good::Ash => "Ash",
            Good::Clay => "Clay",
            Good::Sand => "Sand",
            Good::Gold => "Gold",
            Good::EdgeSeed => "Edgeglass seed",
            Good::Seareed => "Seareed",
            Good::Pitch => "Pitch",
            Good::Nacre => "Nacre",
            Good::Pearl => "Pearls",
            Good::Fishskin => "Fishskin",
            Good::Salvage => "Salvage",
            Good::Tentsilk => "Tentsilk",
            Good::Fibre => "Fibre",
            Good::Leather => "Leather",
            Good::Cloth => "Cloth",
            Good::Ingots => "Forgeiron ingots",
            Good::Bronze => "Bronze",
            Good::Paper => "Scrollpaper",
            Good::Ink => "Ink",
            Good::Ringstone => "Ringstone",
            Good::Slatewing => "Slatewing",
            Good::Edgeglass => "Edgeglass",
            Good::Hearthclay => "Hearthclay",
        }
    }
    pub fn is_food(self) -> bool {
        FOODS.contains(&self)
    }
    /// What a unit is worth, coin (for a material, what one of it is worth).
    pub fn value(self) -> f32 {
        match self {
            Good::Grain => 2.0,
            Good::Fish => 3.0,
            Good::Kelp => 1.5,
            Good::Game => 4.0,
            Good::Timber => 3.0,
            Good::Rock => 1.0,
            Good::Hides => 6.0,
            Good::Herbs => 3.0,
            Good::Wares => 10.0,
            Good::Ore => 4.0,
            Good::Charcoal => 2.0,
            Good::Ash => 1.0,
            Good::Clay => 1.0,
            Good::Sand => 0.5,
            Good::Gold => 45.0,
            Good::EdgeSeed => 40.0,
            Good::Seareed => 2.0,
            Good::Pitch => 4.0,
            Good::Nacre => 12.0,
            Good::Pearl => 30.0,
            Good::Fishskin => 4.0,
            Good::Salvage => 8.0,
            Good::Tentsilk => 10.0,
            Good::Fibre => 1.5,
            Good::Leather => 10.0,
            Good::Cloth => 4.0,
            Good::Ingots => 12.0,
            Good::Bronze => 8.0,
            Good::Paper => 3.0,
            Good::Ink => 5.0,
            Good::Ringstone => 12.0,
            Good::Slatewing => 20.0,
            Good::Edgeglass => 70.0,
            Good::Hearthclay => 3.0,
        }
    }
    /// Share lost to spoiling each day.
    pub fn spoils(self) -> f32 {
        match self {
            Good::Kelp => 0.15,
            Good::Fish => 0.05,
            Good::Game => 0.04,
            Good::Grain => 0.02,
            Good::Hides => 0.02,
            Good::Seareed => 0.02,
            _ => 0.0,
        }
    }
    /// The things a merchant hands over for it (item keys).
    pub fn items(self) -> &'static [&'static str] {
        match self {
            Good::Grain => &["flatbread"],
            Good::Fish => &["dried_fish", "mussels"],
            Good::Kelp => &["kelp_frond"],
            Good::Game => &["salted_meat", "wild_berries"],
            Good::Timber => &["timber"],
            Good::Rock => &["rock"],
            Good::Hides => &["hide"],
            Good::Herbs => &["ash_moss", "ghostcap", "salt_crystal", "emberroot"],
            Good::Wares => &["torch", "healing_draught", "mana_tonic", "lockpick", "arrows", "bolts", "sling_stones", "hearthclay_pot", "sandglass_flask"],
            Good::Ore => &["iron_ore"],
            Good::Charcoal => &["charcoal"],
            Good::Ash => &["ash"],
            Good::Clay => &["clay"],
            Good::Sand => &["sand"],
            Good::Gold => &["gold_nugget"],
            Good::EdgeSeed => &["edge_seed"],
            Good::Seareed => &["seareed"],
            Good::Pitch => &["pitch"],
            Good::Nacre => &["nacre"],
            Good::Pearl => &["pearl"],
            Good::Fishskin => &["fishskin"],
            Good::Salvage => &["salvage"],
            Good::Tentsilk => &["tentsilk"],
            Good::Fibre => &["fibre"],
            Good::Leather => &["leather"],
            Good::Cloth => &["cloth"],
            Good::Ingots => &["iron_ingot"],
            Good::Bronze => &["bronze_ingot"],
            Good::Paper => &["reed_paper"],
            Good::Ink => &["squid_ink"],
            Good::Ringstone => &["ringstone"],
            Good::Slatewing => &["slatewing"],
            Good::Edgeglass => &["edgeglass"],
            Good::Hearthclay => &["hearthclay"],
        }
    }
    /// Which shop sells it, in towns where services are split.
    pub fn shelf(self) -> Shelf {
        match self {
            Good::Grain | Good::Fish | Good::Kelp | Good::Game => Shelf::Food,
            Good::Wares => Shelf::Goods,
            _ => Shelf::Materials,
        }
    }
}

/// The good a stocked item counts as, if any.
pub fn good_of(key: &str) -> Option<Good> {
    GOODS.iter().copied().find(|g| g.items().contains(&key))
}

/// The good a material is traded as (for its price in a town).
pub fn good_of_material(m: super::materials::Material) -> Option<Good> {
    use super::materials::Material as M;
    Some(match m {
        M::None => return None,
        M::PlainIron | M::Forgeiron => Good::Ingots,
        M::Wood => Good::Timber,
        M::Leather => Good::Leather,
        M::Cloth => Good::Cloth,
        M::Ringstone => Good::Ringstone,
        M::Hearthclay => Good::Hearthclay,
        M::Slatewing => Good::Slatewing,
        M::Edgeglass => Good::Edgeglass,
        M::Bronze => Good::Bronze,
        M::Sandglass => Good::Sand,
        M::Gold => Good::Gold,
        M::Seareed => Good::Seareed,
        M::Nacre => Good::Nacre,
        M::Fishskin => Good::Fishskin,
        M::Tentsilk => Good::Tentsilk,
    })
}

// --- Places ------------------------------------------------------------------

/// The three kinds of shop in a town that splits its trade.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shelf {
    Food,
    Materials,
    Goods,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceKind {
    Fields,
    /// The woods and scrub foragers and hunters work.
    Wilds,
    Woodlot,
    /// A landing on the shore, for fishers from the land side.
    Dock,
    /// The stilt village's diving platform.
    DivePlatform,
    KelpBeds,
    Boatyard,
    /// The hearth kitchen.
    Kitchen,
    /// A mess hall (a minority's own, or a second kitchen).
    MessHall,
    /// A shared cooking deck.
    Deck,
    /// One market square with stalls (combined towns).
    Market,
    /// One shop (split towns).
    Shop(Shelf),
    Inn,
    Shrine,
    GuardPost,
    Hall,
    /// Healer, herbalist and alchemist under one roof (combined towns).
    HealingHouse,
    /// A healer's own house (split towns).
    HealersHouse,
    TeachingHouse,
    ExchangeHouse,
    /// Exchange, letters, maps and teaching under one roof: combined towns,
    /// or a strong Ṭaḍoro minority's own house.
    LettersHouse,
    /// Where Stone Tenders keep their beds of growing stone.
    TendersYard,
    /// Forge and armourer's bench in one yard (combined towns).
    Workyard,
    Forge,
    Bench,
    Desk,
    AlchemyTable,
    /// Weaver's frames and a sealing pit (split towns).
    WeaversShed,
    /// Leather, cloth and wood (split towns).
    Workshop,
    /// Mine and quarry face: rock, ore, clay, sand and rare veins.
    Quarry,
    /// Where wood is burned down to charcoal.
    CharcoalPit,
}

impl PlaceKind {
    pub fn name(self) -> &'static str {
        match self {
            PlaceKind::Fields => "Fields",
            PlaceKind::Wilds => "Hunting grounds",
            PlaceKind::Woodlot => "Woodlot",
            PlaceKind::Quarry => "Mine and quarry",
            PlaceKind::CharcoalPit => "Charcoal pit",
            PlaceKind::Dock => "Fishing dock",
            PlaceKind::DivePlatform => "Dive platform",
            PlaceKind::KelpBeds => "Kelp beds",
            PlaceKind::Boatyard => "Boatyard",
            PlaceKind::Kitchen => "Hearth kitchen",
            PlaceKind::MessHall => "Mess hall",
            PlaceKind::Deck => "Cooking deck",
            PlaceKind::Market => "Market square",
            PlaceKind::Shop(Shelf::Food) => "Food shop",
            PlaceKind::Shop(Shelf::Materials) => "Materials shop",
            PlaceKind::Shop(Shelf::Goods) => "Goods shop",
            PlaceKind::Inn => "Inn",
            PlaceKind::Shrine => "Shrine",
            PlaceKind::GuardPost => "Guard post",
            PlaceKind::Hall => "Hall",
            PlaceKind::HealingHouse => "Healing house",
            PlaceKind::HealersHouse => "Healer's house",
            PlaceKind::TeachingHouse => "Teaching house",
            PlaceKind::ExchangeHouse => "Exchange house",
            PlaceKind::LettersHouse => "Letters house",
            PlaceKind::TendersYard => "Tenders' yard",
            PlaceKind::Workyard => "Workyard",
            PlaceKind::Forge => "Smithy",
            PlaceKind::Bench => "Armourer's shop",
            PlaceKind::Desk => "Scribe's room",
            PlaceKind::AlchemyTable => "Alchemist's shop",
            PlaceKind::WeaversShed => "Weavers' shed",
            PlaceKind::Workshop => "Workshop",
        }
    }

    /// Out of town: workers here eat what the runners bring.
    pub fn is_away(self) -> bool {
        matches!(self, PlaceKind::Fields | PlaceKind::Wilds | PlaceKind::Woodlot | PlaceKind::Quarry | PlaceKind::CharcoalPit | PlaceKind::Dock | PlaceKind::DivePlatform | PlaceKind::KelpBeds | PlaceKind::Boatyard)
    }

    /// Footprint, metres.
    pub fn size(self) -> f32 {
        match self {
            PlaceKind::Fields => 34.0,
            PlaceKind::Wilds | PlaceKind::Woodlot | PlaceKind::Quarry => 22.0,
            PlaceKind::CharcoalPit => 12.0,
            PlaceKind::KelpBeds => 26.0,
            PlaceKind::Market | PlaceKind::Workyard => 16.0,
            PlaceKind::DivePlatform | PlaceKind::Deck | PlaceKind::Boatyard => 10.0,
            PlaceKind::Dock => 9.0,
            PlaceKind::Inn | PlaceKind::HealingHouse | PlaceKind::LettersHouse | PlaceKind::MessHall | PlaceKind::Hall => 11.0,
            _ => 7.0,
        }
    }

    /// Crafting stations set up here.
    pub fn stations(self) -> &'static [Station] {
        match self {
            PlaceKind::Workyard => &[Station::Forge, Station::Bench, Station::Loom, Station::Workbench, Station::GrowerBed],
            PlaceKind::TendersYard => &[Station::GrowerBed],
            PlaceKind::WeaversShed | PlaceKind::Boatyard => &[Station::Loom],
            PlaceKind::Workshop => &[Station::Workbench],
            PlaceKind::Forge => &[Station::Forge],
            PlaceKind::Bench => &[Station::Bench],
            PlaceKind::HealingHouse | PlaceKind::AlchemyTable => &[Station::AlchemyTable],
            PlaceKind::LettersHouse | PlaceKind::Desk => &[Station::Desk],
            _ => &[],
        }
    }

    /// Does work in this place go on in the stilt village?
    pub fn offshore(self) -> bool {
        matches!(self, PlaceKind::DivePlatform | PlaceKind::KelpBeds | PlaceKind::Boatyard)
    }
}

/// What a town can do for you, open only while someone who does it is at work.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Service {
    Trade,
    Lodging,
    Healing,
    Alchemy,
    Smithing,
    Armoury,
    Letters,
    Exchange,
    Teaching,
    Shrine,
    Hall,
    Meals,
    Watch,
    Weaving,
    Basics,
    Growing,
}

pub const SERVICES: [Service; 16] = [
    Service::Trade,
    Service::Meals,
    Service::Lodging,
    Service::Healing,
    Service::Alchemy,
    Service::Smithing,
    Service::Armoury,
    Service::Letters,
    Service::Exchange,
    Service::Teaching,
    Service::Shrine,
    Service::Hall,
    Service::Watch,
    Service::Weaving,
    Service::Basics,
    Service::Growing,
];

impl Service {
    pub fn name(self) -> &'static str {
        match self {
            Service::Trade => "Trade",
            Service::Lodging => "Beds and meals",
            Service::Healing => "Healing",
            Service::Alchemy => "Alchemy",
            Service::Smithing => "Smithing",
            Service::Armoury => "Armour",
            Service::Letters => "Letters and maps",
            Service::Exchange => "Exchange",
            Service::Teaching => "Teaching",
            Service::Shrine => "Shrine",
            Service::Hall => "Hall",
            Service::Meals => "Kitchen",
            Service::Watch => "Watch",
            Service::Weaving => "Weaving and sealing",
            Service::Basics => "Leather, cloth, wood",
            Service::Growing => "Grown goods",
        }
    }
}
