//! Every kind of thing that can be carried, worn or wielded.
//!
//! Placeholders, but with real numbers behind them. Weapons do two kinds of
//! damage, Kenshi-style: **cut** (edges and points — deadly against flesh,
//! stopped well by good armour) and **blunt** (weight — gets through armour,
//! wears you down). Armour covers some body parts, catches a share of the blows
//! that land there, and takes a share off each kind of damage. Enchanted pieces
//! carry effects that change their wearer's numbers, Morrowind-style.

use super::body::Part;
use super::stats::{Attr, Skill};

pub type ItemId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    MainHand,
    OffHand,
    Head,
    Body,
    Hands,
    Legs,
    Feet,
    Back,
    Ring,
    Neck,
}

pub const SLOTS: [Slot; 10] = [
    Slot::MainHand,
    Slot::OffHand,
    Slot::Head,
    Slot::Body,
    Slot::Hands,
    Slot::Legs,
    Slot::Feet,
    Slot::Back,
    Slot::Ring,
    Slot::Neck,
];

impl Slot {
    pub fn name(self) -> &'static str {
        match self {
            Slot::MainHand => "Main hand",
            Slot::OffHand => "Off hand",
            Slot::Head => "Head",
            Slot::Body => "Body",
            Slot::Hands => "Hands",
            Slot::Legs => "Legs",
            Slot::Feet => "Feet",
            Slot::Back => "Back",
            Slot::Ring => "Ring",
            Slot::Neck => "Neck",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponDef {
    pub skill: Skill,
    pub cut: f32,
    pub blunt: f32,
    /// Metres.
    pub reach: f32,
    /// Seconds from starting a swing to the blow landing.
    pub windup: f32,
    /// Seconds after a blow before the next can start.
    pub recover: f32,
    pub two_handed: bool,
    /// How well it parries, 0..1 (a shield does this better).
    pub parry: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArmorDef {
    pub covers: &'static [Part],
    /// Chance a blow on a covered part meets the armour.
    pub coverage: f32,
    /// Share of cut damage stopped.
    pub cut: f32,
    /// Share of blunt damage stopped.
    pub blunt: f32,
    /// Taken off the chance to dodge.
    pub dodge_penalty: f32,
}

/// Changes an item makes to whoever has it equipped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Attr(Attr, f32),
    Skill(Skill, f32),
    MaxMana(f32),
    /// Mana regained per game minute.
    ManaRegen(f32),
    /// Extra carrying capacity, kg.
    Carry(f32),
    /// Footspeed, as a fraction (0.1 = 10% faster).
    MoveSpeed(f32),
    /// Chance to shrug off paralysis outright.
    ResistParalysis(f32),
    /// Chance to shrug off blindness outright.
    ResistBlind(f32),
    /// Share of fire and lightning damage ignored.
    ResistElements(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Weapon(WeaponDef),
    Armor(ArmorDef),
    /// Off-hand blocking, 0..1.
    Shield(f32),
    /// Extra capacity, kg.
    Pack(f32),
    Trinket,
    /// Used for a job, not worn: lockpicks, tools of a trade.
    Tool,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub key: &'static str,
    pub name: &'static str,
    pub slot: Slot,
    pub kind: Kind,
    /// Kilograms.
    pub weight: f32,
    /// What it would fetch, in coin; also how NPC kit is budgeted.
    pub value: f32,
    pub effects: &'static [Effect],
}

const ARMS: &[Part] = &[Part::LeftArm, Part::RightArm];
const LEGS: &[Part] = &[Part::LeftLeg, Part::RightLeg];
const BODY: &[Part] = &[Part::Torso, Part::LeftArm, Part::RightArm];
const TORSO: &[Part] = &[Part::Torso];
const HEAD: &[Part] = &[Part::Head];

const fn weapon(key: &'static str, name: &'static str, skill: Skill, cut: f32, blunt: f32, reach: f32, windup: f32, recover: f32, two_handed: bool, parry: f32, weight: f32, value: f32) -> ItemDef {
    ItemDef {
        key,
        name,
        slot: Slot::MainHand,
        kind: Kind::Weapon(WeaponDef { skill, cut, blunt, reach, windup, recover, two_handed, parry }),
        weight,
        value,
        effects: &[],
    }
}

#[allow(clippy::too_many_arguments)]
const fn armor(key: &'static str, name: &'static str, slot: Slot, covers: &'static [Part], coverage: f32, cut: f32, blunt: f32, dodge_penalty: f32, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot, kind: Kind::Armor(ArmorDef { covers, coverage, cut, blunt, dodge_penalty }), weight, value, effects: &[] }
}

const fn trinket(key: &'static str, name: &'static str, slot: Slot, weight: f32, value: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot, kind: Kind::Trinket, weight, value, effects }
}

/// The whole catalogue. An `ItemId` is an index into this list.
pub static ITEMS: &[ItemDef] = &[
    // --- Weapons ---------------------------------------------------------
    //      key            name             skill          cut   blunt reach windup rec   2h     parry wt   value
    weapon("knife", "Knife", Skill::Blade, 6.0, 1.0, 0.9, 0.35, 0.35, false, 0.05, 0.5, 10.0),
    weapon("short_sword", "Short sword", Skill::Blade, 11.0, 2.0, 1.1, 0.45, 0.45, false, 0.20, 2.5, 60.0),
    weapon("longsword", "Longsword", Skill::Blade, 15.0, 3.0, 1.3, 0.60, 0.60, false, 0.25, 4.0, 140.0),
    weapon("club", "Club", Skill::Blunt, 0.0, 10.0, 1.0, 0.50, 0.50, false, 0.10, 2.0, 15.0),
    weapon("war_pick", "War-pick", Skill::Blunt, 8.0, 12.0, 1.1, 0.60, 0.60, false, 0.10, 4.0, 130.0),
    weapon("stone_maul", "Stone maul", Skill::Blunt, 0.0, 22.0, 1.3, 1.00, 0.90, true, 0.10, 9.0, 120.0),
    weapon("spear", "Spear", Skill::Spear, 9.0, 3.0, 2.2, 0.55, 0.50, true, 0.20, 3.0, 50.0),
    weapon("harpoon", "Harpoon", Skill::Spear, 11.0, 2.0, 2.0, 0.55, 0.55, true, 0.15, 3.0, 70.0),
    weapon("glaive", "Glaive", Skill::Spear, 16.0, 4.0, 2.4, 0.80, 0.70, true, 0.25, 5.0, 180.0),
    weapon("staff", "Staff", Skill::Blunt, 0.0, 7.0, 1.8, 0.50, 0.50, true, 0.30, 2.0, 20.0),
    // --- Body armour -----------------------------------------------------
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    armor("cloth_shirt", "Cloth shirt", Slot::Body, BODY, 0.90, 0.10, 0.10, 0.00, 1.0, 8.0),
    armor("padded_jacket", "Padded jacket", Slot::Body, BODY, 0.90, 0.25, 0.30, 0.02, 4.0, 40.0),
    armor("hide_coat", "Hide coat", Slot::Body, BODY, 0.85, 0.35, 0.25, 0.04, 6.0, 70.0),
    armor("scale_hauberk", "Scale hauberk", Slot::Body, BODY, 0.85, 0.60, 0.30, 0.10, 14.0, 260.0),
    armor("sandstone_lamellar", "Sandstone lamellar", Slot::Body, TORSO, 0.95, 0.55, 0.40, 0.08, 12.0, 240.0),
    armor("leather_cap", "Leather cap", Slot::Head, HEAD, 0.70, 0.25, 0.20, 0.00, 0.8, 15.0),
    armor("iron_helm", "Iron helm", Slot::Head, HEAD, 0.85, 0.60, 0.40, 0.03, 3.0, 120.0),
    armor("trousers", "Trousers", Slot::Legs, LEGS, 0.90, 0.10, 0.10, 0.00, 1.0, 6.0),
    armor("hide_leggings", "Hide leggings", Slot::Legs, LEGS, 0.85, 0.30, 0.20, 0.02, 3.0, 40.0),
    armor("scale_greaves", "Scale greaves", Slot::Legs, LEGS, 0.80, 0.55, 0.30, 0.06, 7.0, 160.0),
    armor("leather_gloves", "Leather gloves", Slot::Hands, ARMS, 0.35, 0.20, 0.15, 0.00, 0.5, 12.0),
    armor("boots", "Boots", Slot::Feet, LEGS, 0.30, 0.20, 0.15, 0.00, 1.2, 15.0),
    // --- Shields and packs -----------------------------------------------
    ItemDef { key: "buckler", name: "Buckler", slot: Slot::OffHand, kind: Kind::Shield(0.35), weight: 2.5, value: 50.0, effects: &[] },
    ItemDef { key: "kite_shield", name: "Kite shield", slot: Slot::OffHand, kind: Kind::Shield(0.55), weight: 6.0, value: 140.0, effects: &[] },
    ItemDef { key: "small_pack", name: "Small pack", slot: Slot::Back, kind: Kind::Pack(20.0), weight: 1.5, value: 30.0, effects: &[] },
    ItemDef { key: "large_pack", name: "Large pack", slot: Slot::Back, kind: Kind::Pack(45.0), weight: 3.0, value: 90.0, effects: &[] },
    // --- Tools ----------------------------------------------------------------
    ItemDef { key: "lockpick", name: "Lockpick", slot: Slot::MainHand, kind: Kind::Tool, weight: 0.05, value: 8.0, effects: &[] },
    // --- Enchanted pieces -------------------------------------------------
    trinket("ring_swiftness", "Ring of Swiftness", Slot::Ring, 0.1, 300.0, &[Effect::MoveSpeed(0.15), Effect::Attr(Attr::Agility, 5.0)]),
    trinket("ring_might", "Ring of the Ox", Slot::Ring, 0.1, 320.0, &[Effect::Attr(Attr::Strength, 12.0)]),
    trinket("amulet_wellspring", "Wellspring Amulet", Slot::Neck, 0.2, 350.0, &[Effect::MaxMana(30.0), Effect::ManaRegen(0.6)]),
    trinket("amulet_clear_mind", "Amulet of the Clear Mind", Slot::Neck, 0.2, 280.0, &[Effect::ResistParalysis(0.6), Effect::Attr(Attr::Willpower, 6.0)]),
    trinket("ring_hearth", "Hearthstone Ring", Slot::Ring, 0.1, 260.0, &[Effect::ResistElements(0.4)]),
    ItemDef {
        key: "seers_hood",
        name: "Seer's hood",
        slot: Slot::Head,
        kind: Kind::Armor(ArmorDef { covers: HEAD, coverage: 0.6, cut: 0.15, blunt: 0.10, dodge_penalty: 0.0 }),
        weight: 0.6,
        value: 240.0,
        effects: &[Effect::ResistBlind(0.7), Effect::Skill(Skill::Illusion, 8.0)],
    },
    ItemDef {
        key: "striders_boots",
        name: "Strider's boots",
        slot: Slot::Feet,
        kind: Kind::Armor(ArmorDef { covers: LEGS, coverage: 0.30, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0 }),
        weight: 1.0,
        value: 260.0,
        effects: &[Effect::MoveSpeed(0.20), Effect::Skill(Skill::Athletics, 10.0)],
    },
    ItemDef {
        key: "porters_belt_pack",
        name: "Porter's frame pack",
        slot: Slot::Back,
        kind: Kind::Pack(60.0),
        weight: 4.0,
        value: 200.0,
        effects: &[Effect::Carry(20.0)],
    },
    ItemDef {
        key: "duelists_gloves",
        name: "Duelist's gloves",
        slot: Slot::Hands,
        kind: Kind::Armor(ArmorDef { covers: ARMS, coverage: 0.35, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0 }),
        weight: 0.4,
        value: 230.0,
        effects: &[Effect::Skill(Skill::Blade, 10.0), Effect::Skill(Skill::Block, 6.0)],
    },
];

/// Can this be worn or held (rather than used up or crafted with)?
pub fn equippable(id: ItemId) -> bool {
    matches!(item(id).kind, Kind::Weapon(_) | Kind::Armor(_) | Kind::Shield(_) | Kind::Pack(_) | Kind::Trinket)
}

pub fn item(id: ItemId) -> &'static ItemDef {
    &ITEMS[id as usize]
}

/// Look an item up by its key. Panics on a typo, which is what tests want.
pub fn id(key: &str) -> ItemId {
    ITEMS.iter().position(|d| d.key == key).unwrap_or_else(|| panic!("no item called {key}")) as ItemId
}

impl ItemDef {
    pub fn weapon(&self) -> Option<&WeaponDef> {
        match &self.kind {
            Kind::Weapon(w) => Some(w),
            _ => None,
        }
    }
    pub fn armor(&self) -> Option<&ArmorDef> {
        match &self.kind {
            Kind::Armor(a) => Some(a),
            _ => None,
        }
    }
}

/// Bare hands, when nothing is held.
pub const FISTS: WeaponDef = WeaponDef { skill: Skill::Unarmed, cut: 0.0, blunt: 4.0, reach: 0.8, windup: 0.35, recover: 0.35, two_handed: false, parry: 0.0 };
