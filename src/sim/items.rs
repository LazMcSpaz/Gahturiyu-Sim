//! Every kind of thing that can be carried, worn or wielded.
//!
//! Placeholders, but with real numbers behind them. Weapons do two kinds of
//! damage, Kenshi-style: **cut** (edges and points — deadly against flesh,
//! stopped well by good armour) and **blunt** (weight — gets through armour,
//! wears you down). Armour covers some body parts, catches a share of the blows
//! that land there, and takes a share off each kind of damage. Enchanted pieces
//! carry effects that change their wearer's numbers, Morrowind-style.

use serde::{Deserialize, Serialize};

use super::body::Part;
use super::effects::{now, worn, Does, Effect, Reach};
use super::stats::{Attr, Skill};

pub type ItemId = u16;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
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
    /// Metres it shoots, for bows and the like (0 = a hand weapon).
    pub range: f32,
    /// What it shoots (an item key), for ranged weapons.
    #[serde(with = "super::save::opt_name")]
    pub ammo: Option<super::save::Name>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ArmorDef {
    #[serde(with = "super::save::covers")]
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
    /// Something to make things from.
    Material,
    /// Drunk: its effects (`ItemDef::effects`) work on whoever drinks it.
    Potion,
    /// Read aloud: casts the spell with this key once, with no energy and no
    /// chance of failing.
    Scroll(&'static str),
    /// Notes on a structured spell (by key): read them to learn it.
    Notes(&'static str),
    /// A rare text on a ritual (by key): read it to learn it.
    Text(&'static str),
    /// Shot from a ranged weapon; stacks in the pack.
    Ammo,
    /// Money.
    Coin,
    /// Eaten: takes this much off hunger (0–100).
    Food(f32),
    /// Carried for someone else (a letter to deliver).
    Errand,
    /// Held in the off hand and lit: burns this many hours (see `torch`).
    Torch(f32),
    /// Set in the ground and lit: burns this many hours.
    StandingTorch(f32),
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
    /// What it does: while worn (enchantments), or when drunk (potions).
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
        kind: Kind::Weapon(WeaponDef { skill, cut, blunt, reach, windup, recover, two_handed, parry, range: 0.0, ammo: None }),
        weight,
        value,
        effects: &[],
    }
}

#[allow(clippy::too_many_arguments)]
const fn armor(key: &'static str, name: &'static str, slot: Slot, covers: &'static [Part], coverage: f32, cut: f32, blunt: f32, dodge_penalty: f32, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot, kind: Kind::Armor(ArmorDef { covers, coverage, cut, blunt, dodge_penalty }), weight, value, effects: &[] }
}

const fn material(key: &'static str, name: &'static str, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Material, weight, value, effects: &[] }
}

/// A bow or the like: `windup` is drawing and aiming, `recover` reloading.
#[allow(clippy::too_many_arguments)]
const fn ranged(key: &'static str, name: &'static str, cut: f32, blunt: f32, range: f32, windup: f32, recover: f32, ammo: &'static str, weight: f32, value: f32) -> ItemDef {
    ItemDef {
        key,
        name,
        slot: Slot::MainHand,
        kind: Kind::Weapon(WeaponDef { skill: Skill::Marksman, cut, blunt, reach: 1.0, windup, recover, two_handed: true, parry: 0.0, range, ammo: Some(ammo) }),
        weight,
        value,
        effects: &[],
    }
}

const fn food(key: &'static str, name: &'static str, weight: f32, value: f32, nourishment: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Food(nourishment), weight, value, effects: &[] }
}

const fn scroll(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Scroll(spell), weight: 0.05, value, effects: &[] }
}

const fn notes(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Notes(spell), weight: 0.1, value, effects: &[] }
}

const fn text(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Text(spell), weight: 0.5, value, effects: &[] }
}

const fn potion(key: &'static str, name: &'static str, value: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Potion, weight: 0.3, value, effects }
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
    //      key            name         cut   blunt range windup reload ammo      wt   value
    ranged("short_bow", "Short bow", 12.0, 0.0, 30.0, 0.8, 0.5, "arrows", 1.5, 60.0),
    ranged("crossbow", "Crossbow", 18.0, 4.0, 38.0, 0.4, 2.6, "bolts", 5.0, 160.0),
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
    ItemDef { key: "torch", name: "Torch", slot: Slot::OffHand, kind: Kind::Torch(super::torch::TORCH_HOURS), weight: 0.6, value: 4.0, effects: &[] },
    ItemDef { key: "standing_torch", name: "Standing torch", slot: Slot::MainHand, kind: Kind::StandingTorch(super::torch::STANDING_HOURS), weight: 2.0, value: 10.0, effects: &[] },
    ItemDef { key: "tent", name: "Tent", slot: Slot::MainHand, kind: Kind::Tool, weight: 6.0, value: 60.0, effects: &[] },
    ItemDef { key: "mortar_and_pestle", name: "Mortar and pestle", slot: Slot::MainHand, kind: Kind::Tool, weight: 1.5, value: 25.0, effects: &[] },
    // --- Materials ------------------------------------------------------------
    //        key               name                weight value
    material("kelp_frond", "Kelp frond", 0.2, 2.0),
    material("ghostcap", "Ghostcap", 0.1, 4.0),
    material("emberroot", "Emberroot", 0.2, 6.0),
    material("salt_crystal", "Salt crystal", 0.3, 3.0),
    material("ash_moss", "Ash moss", 0.1, 2.0),
    material("storm_glass", "Storm glass", 0.4, 12.0),
    material("reed_paper", "Reed paper", 0.05, 3.0),
    material("squid_ink", "Squid ink", 0.2, 5.0),
    material("iron_ore", "Iron ore", 3.0, 4.0),
    material("iron_ingot", "Iron ingot", 2.0, 12.0),
    material("hide", "Hide", 2.0, 6.0),
    material("leather", "Leather", 1.0, 10.0),
    material("timber", "Timber", 2.5, 3.0),
    // --- Food (placeholder names) ---------------------------------------------
    food("dried_fish", "Dried fish", 0.3, 4.0, 25.0),
    food("flatbread", "Flatbread", 0.4, 3.0, 30.0),
    food("salted_meat", "Salted meat", 0.5, 6.0, 40.0),
    food("wild_berries", "Wild berries", 0.1, 1.0, 8.0),
    food("mussels", "Mussels", 0.3, 2.0, 14.0),
    ItemDef { key: "arrows", name: "Arrows", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.04, value: 1.0, effects: &[] },
    ItemDef { key: "bolts", name: "Crossbow bolts", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.06, value: 2.0, effects: &[] },
    ItemDef { key: "coin", name: "Coin", slot: Slot::MainHand, kind: Kind::Coin, weight: 0.005, value: 1.0, effects: &[] },
    ItemDef { key: "sealed_letter", name: "Sealed letter", slot: Slot::MainHand, kind: Kind::Errand, weight: 0.02, value: 0.0, effects: &[] },
    // --- Potions and scrolls --------------------------------------------------
    potion("healing_draught", "Healing draught", 25.0, &[now(Does::Heal, 25.0, Reach::Caster)]),
    potion("greater_healing", "Greater healing draught", 70.0, &[now(Does::Heal, 55.0, Reach::Caster)]),
    potion("mana_tonic", "Mana tonic", 35.0, &[now(Does::Energy, 40.0, Reach::Caster)]),
    scroll("scroll_heal", "Scroll of mending", "mend", 40.0),
    scroll("scroll_paralyze", "Scroll of paralysis", "paralyze", 60.0),
    scroll("scroll_fireball", "Scroll of fireball", "fireball", 70.0),
    scroll("scroll_lightning", "Scroll of lightning", "lightning_bolt", 60.0),
    // --- Notes and texts: read to learn ------------------------------------
    notes("notes_paralyze", "Notes on paralysis", "paralyze", 90.0),
    notes("notes_fireball", "Notes on fireball", "fireball", 110.0),
    notes("notes_lightning", "Notes on lightning", "lightning_bolt", 90.0),
    notes("notes_blind", "Notes on blinding", "blind", 70.0),
    notes("notes_barrier", "Notes on barriers", "barrier", 70.0),
    notes("notes_haste", "Notes on haste", "haste", 80.0),
    notes("notes_stone_spikes", "Notes on stone spikes", "stone_spikes", 85.0),
    notes("notes_calm", "Notes on calming", "calm", 70.0),
    notes("notes_fear", "Notes on fear", "fear", 75.0),
    notes("notes_sway", "Notes on persuasion", "sway", 60.0),
    notes("notes_hide", "Notes on hiding", "hide", 80.0),
    notes("notes_decoy", "Notes on decoys", "decoy", 75.0),
    notes("notes_disguise", "Notes on disguise", "disguise", 90.0),
    notes("notes_might", "Notes on might", "might", 80.0),
    notes("notes_toughen", "Notes on toughening", "toughen", 70.0),
    notes("notes_resist", "Notes on resisting the elements", "resist", 65.0),
    notes("notes_dispel", "Notes on dispelling", "dispel", 90.0),
    text("text_restore", "Rite of Restoring", "restore", 220.0),
    text("text_sanctuary", "Rite of Sanctuary", "sanctuary", 260.0),
    text("text_sustain", "Rite of Sustaining", "sustain", 240.0),
    text("text_regrow", "Rite of Regrowth", "regrow", 600.0),
    text("text_veil", "Rite of the Veil", "veil", 230.0),
    text("text_dominate", "Rite of Mastery", "dominate", 320.0),
    text("text_firestorm", "Rite of the Firestorm", "firestorm", 260.0),
    text("text_tremor", "Rite of the Shaking Ground", "tremor", 240.0),
    // --- Enchanted pieces -------------------------------------------------
    trinket("ring_swiftness", "Ring of Swiftness", Slot::Ring, 0.1, 300.0, &[worn(Does::MoveSpeed, 0.15), worn(Does::Attr(Attr::Agility), 5.0)]),
    trinket("ring_might", "Ring of the Ox", Slot::Ring, 0.1, 320.0, &[worn(Does::Attr(Attr::Strength), 12.0)]),
    trinket("amulet_wellspring", "Wellspring Amulet", Slot::Neck, 0.2, 350.0, &[worn(Does::MaxEnergy, 30.0), worn(Does::EnergyRegen, 0.6)]),
    trinket("amulet_clear_mind", "Amulet of the Clear Mind", Slot::Neck, 0.2, 280.0, &[worn(Does::ResistParalysis, 0.6), worn(Does::Attr(Attr::Willpower), 6.0)]),
    trinket("ring_hearth", "Hearthstone Ring", Slot::Ring, 0.1, 260.0, &[worn(Does::ResistElements, 0.4)]),
    ItemDef {
        key: "seers_hood",
        name: "Seer's hood",
        slot: Slot::Head,
        kind: Kind::Armor(ArmorDef { covers: HEAD, coverage: 0.6, cut: 0.15, blunt: 0.10, dodge_penalty: 0.0 }),
        weight: 0.6,
        value: 240.0,
        effects: &[worn(Does::ResistBlind, 0.7), worn(Does::Skill(Skill::Structured), 8.0)],
    },
    ItemDef {
        key: "striders_boots",
        name: "Strider's boots",
        slot: Slot::Feet,
        kind: Kind::Armor(ArmorDef { covers: LEGS, coverage: 0.30, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0 }),
        weight: 1.0,
        value: 260.0,
        effects: &[worn(Does::MoveSpeed, 0.20), worn(Does::Skill(Skill::Athletics), 10.0)],
    },
    ItemDef {
        key: "porters_belt_pack",
        name: "Porter's frame pack",
        slot: Slot::Back,
        kind: Kind::Pack(60.0),
        weight: 4.0,
        value: 200.0,
        effects: &[worn(Does::Carry, 20.0)],
    },
    ItemDef {
        key: "duelists_gloves",
        name: "Duelist's gloves",
        slot: Slot::Hands,
        kind: Kind::Armor(ArmorDef { covers: ARMS, coverage: 0.35, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0 }),
        weight: 0.4,
        value: 230.0,
        effects: &[worn(Does::Skill(Skill::Blade), 10.0), worn(Does::Skill(Skill::Block), 6.0)],
    },
];

/// Can this be worn or held (rather than used up or crafted with)?
pub fn equippable(id: ItemId) -> bool {
    matches!(item(id).kind, Kind::Weapon(_) | Kind::Armor(_) | Kind::Shield(_) | Kind::Pack(_) | Kind::Trinket | Kind::Torch(_))
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
pub const FISTS: WeaponDef = WeaponDef { skill: Skill::Unarmed, cut: 0.0, blunt: 4.0, reach: 0.8, windup: 0.35, recover: 0.35, two_handed: false, parry: 0.0, range: 0.0, ammo: None };
