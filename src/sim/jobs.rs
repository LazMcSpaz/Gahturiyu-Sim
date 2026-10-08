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
}

pub const ALL_JOBS: [Job; 25] = [
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
            Job::Guard | Job::Boatwright | Job::Smith | Job::Armourer => 0.3,
            Job::Farmer | Job::Forager | Job::Fisher | Job::Woodcutter => 0.2,
        }
    }

    /// Someone whose loss leaves a gap a town fills from its labourers.
    pub fn is_post(self) -> bool {
        !matches!(self, Job::None | Job::Labourer | Job::Drifter)
    }

    /// What one hour of this work adds to the town's stockpile.
    pub fn yields(self) -> Option<(Good, f32)> {
        Some(match self {
            Job::Farmer => (Good::Grain, FARM_PER_HOUR),
            Job::Fisher => (Good::Fish, FISH_PER_HOUR),
            Job::KelpGatherer => (Good::Kelp, KELP_PER_HOUR),
            Job::Forager => (Good::Game, 0.3),
            Job::Woodcutter => (Good::Timber, 0.6),
            Job::Boatwright | Job::Smith | Job::Armourer | Job::Alchemist | Job::Scribe => (Good::Wares, 0.12),
            Job::Labourer => (Good::Stone, 0.15),
            _ => return None,
        })
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

// --- Goods -------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Good {
    Grain,
    Fish,
    Kelp,
    Game,
    Timber,
    Stone,
    Hides,
    Herbs,
    /// Made things: tools, torches, draughts, fittings.
    Wares,
}

pub const N_GOODS: usize = 9;
pub const GOODS: [Good; N_GOODS] = [Good::Grain, Good::Fish, Good::Kelp, Good::Game, Good::Timber, Good::Stone, Good::Hides, Good::Herbs, Good::Wares];
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
            Good::Stone => "Stone",
            Good::Hides => "Hides",
            Good::Herbs => "Herbs",
            Good::Wares => "Wares",
        }
    }
    pub fn is_food(self) -> bool {
        FOODS.contains(&self)
    }
    /// What a unit is worth, coin.
    pub fn value(self) -> f32 {
        match self {
            Good::Grain => 2.0,
            Good::Fish => 3.0,
            Good::Kelp => 1.5,
            Good::Game => 4.0,
            Good::Timber => 3.0,
            Good::Stone => 2.0,
            Good::Hides => 6.0,
            Good::Herbs => 3.0,
            Good::Wares => 10.0,
        }
    }
    /// Share lost to spoiling each day.
    pub fn spoils(self) -> f32 {
        match self {
            Good::Kelp => 0.15,
            Good::Fish => 0.05,
            Good::Game => 0.04,
            Good::Grain => 0.02,
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
            Good::Stone => &[],
            Good::Hides => &["hide", "leather"],
            Good::Herbs => &["ash_moss", "ghostcap", "salt_crystal"],
            Good::Wares => &["torch", "healing_draught", "lockpick", "iron_ingot", "reed_paper", "squid_ink", "arrows"],
        }
    }
    /// Which shop sells it, in towns where services are split.
    pub fn shelf(self) -> Shelf {
        match self {
            Good::Grain | Good::Fish | Good::Kelp | Good::Game => Shelf::Food,
            Good::Timber | Good::Stone | Good::Hides | Good::Herbs => Shelf::Materials,
            Good::Wares => Shelf::Goods,
        }
    }
}

/// The good a stocked item counts as, if any.
pub fn good_of(key: &str) -> Option<Good> {
    GOODS.iter().copied().find(|g| g.items().contains(&key))
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
}

impl PlaceKind {
    pub fn name(self) -> &'static str {
        match self {
            PlaceKind::Fields => "Fields",
            PlaceKind::Wilds => "Hunting grounds",
            PlaceKind::Woodlot => "Woodlot and quarry",
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
        }
    }

    /// Out of town: workers here eat what the runners bring.
    pub fn is_away(self) -> bool {
        matches!(self, PlaceKind::Fields | PlaceKind::Wilds | PlaceKind::Woodlot | PlaceKind::Dock | PlaceKind::DivePlatform | PlaceKind::KelpBeds | PlaceKind::Boatyard)
    }

    /// Footprint, metres.
    pub fn size(self) -> f32 {
        match self {
            PlaceKind::Fields => 34.0,
            PlaceKind::Wilds | PlaceKind::Woodlot => 22.0,
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
            PlaceKind::Workyard => &[Station::Forge, Station::Bench],
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
}

pub const SERVICES: [Service; 13] = [
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
        }
    }
}
