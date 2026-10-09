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
use super::materials::{Craft, Grade, Material, GRADES};
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
    /// Share of the target's armour it gets through (Forgeiron's weight and
    /// point; Edgeglass has none).
    #[serde(default)]
    pub pierce: f32,
    /// Chance a hit tangles the target up for a few seconds (nets).
    #[serde(default)]
    pub entangle: f32,
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
    /// A Ṭaḍoro manual on a craft: read it to take the craft up.
    Manual(Skill),
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
        kind: Kind::Weapon(WeaponDef { skill, cut, blunt, reach, windup, recover, two_handed, parry, range: 0.0, ammo: None, pierce: 0.0, entangle: 0.0 }),
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
        kind: Kind::Weapon(WeaponDef { skill: Skill::Marksman, cut, blunt, reach: 1.0, windup, recover, two_handed: true, parry: 0.0, range, ammo: Some(ammo), pierce: 0.0, entangle: 0.0 }),
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

const fn manual(key: &'static str, name: &'static str, skill: Skill) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Manual(skill), weight: 0.6, value: 120.0, effects: &[] }
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
    weapon("hatchet", "Hatchet", Skill::Blade, 8.0, 4.0, 0.9, 0.45, 0.45, false, 0.05, 1.2, 14.0),
    weapon("trident", "Trident", Skill::Spear, 12.0, 2.0, 2.1, 0.55, 0.55, true, 0.20, 3.2, 90.0),
    ranged("sling", "Sling", 0.0, 9.0, 22.0, 0.6, 0.8, "sling_stones", 0.2, 8.0),
    ItemDef {
        key: "net",
        name: "Weighted net",
        slot: Slot::MainHand,
        kind: Kind::Weapon(WeaponDef { skill: Skill::Spear, cut: 0.0, blunt: 2.0, reach: 2.4, windup: 0.8, recover: 0.9, two_handed: true, parry: 0.0, range: 0.0, ammo: None, pierce: 0.0, entangle: 0.45 }),
        weight: 2.5,
        value: 35.0,
        effects: &[],
    },
    // --- Body armour -----------------------------------------------------
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    armor("cloth_shirt", "Cloth shirt", Slot::Body, BODY, 0.90, 0.10, 0.10, 0.00, 1.0, 8.0),
    armor("wraps", "Silk wraps", Slot::Body, BODY, 0.90, 0.22, 0.18, 0.00, 0.6, 70.0),
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
    // Raw materials (each is also a good in a town's store).
    material("rock", "Rock feedstock", 3.0, 1.0),
    material("ash", "Ash", 0.5, 1.0),
    material("edge_seed", "Edgeglass seed crystal", 0.2, 40.0),
    material("clay", "Clay", 2.0, 1.0),
    material("charcoal", "Charcoal", 1.0, 2.0),
    material("sand", "Sand", 2.0, 0.5),
    material("gold_nugget", "Gold", 0.3, 45.0),
    material("seareed", "Seareed", 0.5, 2.0),
    material("pitch", "Pitch", 0.8, 4.0),
    material("nacre", "Nacre", 0.4, 12.0),
    material("pearl", "Pearl", 0.05, 30.0),
    material("fishskin", "Fishskin", 0.3, 4.0),
    material("salvage", "Salvage", 2.0, 8.0),
    material("tentsilk", "Tentsilk", 0.2, 10.0),
    material("fibre", "Fibre", 0.5, 1.5),
    material("cloth", "Cloth", 0.5, 4.0),
    // Grown stock, from the Tenders' beds.
    material("ringstone", "Ringstone block", 6.0, 12.0),
    material("slatewing", "Slatewing sheet", 1.0, 20.0),
    material("edgeglass", "Edgeglass blank", 0.6, 70.0),
    material("hearthclay", "Hearthclay", 1.5, 3.0),
    // Made for trade.
    material("bronze_ingot", "Bronze ingot", 2.0, 8.0),
    trinket("hearthclay_pot", "Hearthclay pot", Slot::MainHand, 1.2, 6.0, &[]),
    trinket("sandglass_flask", "Sandglass flask", Slot::MainHand, 0.3, 12.0, &[]),
    trinket("gold_ring", "Gold ring", Slot::Ring, 0.05, 90.0, &[]),
    trinket("pearl_necklace", "Pearl necklace", Slot::Neck, 0.1, 140.0, &[]),
    // Ṭaḍoro manuals: read one to take up a craft.
    manual("manual_handcraft", "Manual of leather, cloth and wood", Skill::Handcraft),
    manual("manual_smithing", "Manual of smithing", Skill::Smithing),
    manual("manual_armoring", "Manual of armouring", Skill::Armoring),
    manual("manual_tending", "Manual of stone-tending", Skill::Tending),
    manual("manual_weaving", "Manual of weaving and sealing", Skill::Weaving),
    manual("manual_inscription", "Manual of paper and ink", Skill::Inscription),
    manual("manual_alchemy", "Manual of alchemy", Skill::Alchemy),
    manual("manual_carpentry", "Manual of carpentry", Skill::Carpentry),
    manual("manual_masonry", "Manual of masonry", Skill::Masonry),
    // --- Food (placeholder names) ---------------------------------------------
    food("dried_fish", "Dried fish", 0.3, 4.0, 25.0),
    food("flatbread", "Flatbread", 0.4, 3.0, 30.0),
    food("salted_meat", "Salted meat", 0.5, 6.0, 40.0),
    food("wild_berries", "Wild berries", 0.1, 1.0, 8.0),
    food("mussels", "Mussels", 0.3, 2.0, 14.0),
    ItemDef { key: "arrows", name: "Arrows", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.04, value: 1.0, effects: &[] },
    ItemDef { key: "bolts", name: "Crossbow bolts", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.06, value: 2.0, effects: &[] },
    ItemDef { key: "sling_stones", name: "Sling stones", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.05, value: 0.2, effects: &[] },
    ItemDef { key: "coin", name: "Coin", slot: Slot::MainHand, kind: Kind::Coin, weight: 0.005, value: 1.0, effects: &[] },
    // Fifty coin on Ṭaḍoro paper: next to nothing to carry, but as easily stolen or lost.
    ItemDef { key: "note", name: "Note (50 coin)", slot: Slot::MainHand, kind: Kind::Coin, weight: 0.001, value: 50.0, effects: &[] },
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
    notes("notes_shatter", "Notes on shattering", "shatter", 95.0),
    notes("notes_unlock", "Notes on unlocking", "unlock", 70.0),
    notes("notes_shrink", "Notes on shrinking", "shrink", 70.0),
    notes("notes_enlarge", "Notes on enlarging", "enlarge", 70.0),
    notes("notes_rust", "Notes on rust", "rust", 65.0),
    notes("notes_spirit_beast", "Notes on spirit beasts", "spirit_beast", 110.0),
    notes("notes_pack_spirit", "Notes on pack spirits", "pack_spirit", 70.0),
    notes("notes_wither", "Notes on withering", "wither", 95.0),
    notes("notes_raise_thrall", "Notes on raising the dead", "raise_thrall", 120.0),
    text("text_restore", "Rite of Restoring", "restore", 220.0),
    text("text_grave_call", "Rite of the Grave Call", "grave_call", 320.0),
    text("text_blight", "Rite of Blight", "blight", 280.0),
    text("text_guardian", "Rite of the Guardian", "guardian", 300.0),
    text("text_swarm", "Rite of the Swarm", "swarm", 260.0),
    text("text_transmute", "Rite of Changing", "transmute", 280.0),
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
    &catalogue().defs[id as usize]
}

/// Look an item up by its key. Panics on a typo, which is what tests want.
pub fn id(key: &str) -> ItemId {
    *catalogue().by_key.get(key).unwrap_or_else(|| panic!("no item called {key}"))
}

/// What a catalogue entry is made of: its form (the plain item it's a version
/// of), main and second material, and grade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemInfo {
    pub form: ItemId,
    pub main: Material,
    pub second: Material,
    pub grade: Grade,
}

pub fn info(id: ItemId) -> &'static ItemInfo {
    &catalogue().info[id as usize]
}

/// The version of `form` in these materials at this grade, if there is one.
pub fn variant(form: ItemId, main: Material, second: Material, grade: Grade) -> Option<ItemId> {
    catalogue().variants.get(&(form, main, second, grade)).copied()
}

/// How many blows it takes before it's worn out (0: it doesn't wear).
pub fn max_durability(id: ItemId) -> f32 {
    let i = info(id);
    if i.main == Material::None || !equippable(id) || matches!(item(id).kind, Kind::Trinket | Kind::Torch(_)) {
        return 0.0;
    }
    let d = i.main.def().durability;
    let d = if i.second != Material::None { (d + i.second.def().durability) * 0.5 * 1.2 } else { d };
    d * i.grade.durability()
}

/// Can it be mended at all?
pub fn repairable(id: ItemId) -> bool {
    let i = info(id);
    i.main.def().repairable && (i.second == Material::None || i.second.def().repairable)
}

/// The craft that mends it (the main material's).
pub fn craft_of(id: ItemId) -> Craft {
    info(id).main.def().craft
}

pub fn catalogue() -> &'static Catalogue {
    static CAT: std::sync::OnceLock<Catalogue> = std::sync::OnceLock::new();
    CAT.get_or_init(Catalogue::build)
}

/// Every item there is: the plain table above, then every version of each
/// form in each material and grade, worked out once.
pub struct Catalogue {
    pub defs: Vec<ItemDef>,
    pub info: Vec<ItemInfo>,
    by_key: std::collections::HashMap<&'static str, ItemId>,
    variants: std::collections::HashMap<(ItemId, Material, Material, Grade), ItemId>,
}

/// A form: what it's usually made of, and what else it can be made of
/// (main materials, and backing materials it can be made on).
pub struct Form {
    pub key: &'static str,
    pub noun: &'static str,
    pub usual: Material,
    pub mains: &'static [Material],
    pub seconds: &'static [Material],
}

use Material as M;

const fn f(key: &'static str, noun: &'static str, usual: Material, mains: &'static [Material], seconds: &'static [Material]) -> Form {
    Form { key, noun, usual, mains, seconds }
}

/// Each made form and its materials. Shared basics are in plain iron, wood,
/// leather and cloth; the traditions' materials are the step up. A second
/// material backs the main one (an Edgeglass edge on a Forgeiron spine,
/// Nacre scales on a Slatewing frame); any listed pairing can turn up.
pub static FORMS: &[Form] = &[
    f("knife", "knife", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[]),
    f("hatchet", "hatchet", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron], &[]),
    f("short_sword", "short blade", M::Bronze, &[M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[M::Forgeiron]),
    f("longsword", "sword", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("club", "club", M::Wood, &[M::Wood], &[]),
    f("war_pick", "war-pick", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("stone_maul", "maul", M::Ringstone, &[M::Ringstone, M::Forgeiron], &[]),
    f("spear", "spear", M::Wood, &[M::Wood, M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[]),
    f("harpoon", "harpoon", M::Nacre, &[M::Nacre, M::Bronze, M::Forgeiron], &[]),
    f("trident", "trident", M::Nacre, &[M::Nacre, M::Bronze, M::Forgeiron], &[]),
    f("glaive", "glaive", M::Forgeiron, &[M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("net", "net", M::Seareed, &[M::Seareed], &[]),
    f("staff", "staff", M::Wood, &[M::Wood], &[]),
    f("short_bow", "bow", M::Wood, &[M::Wood], &[]),
    f("crossbow", "crossbow", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("sling", "sling", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("cloth_shirt", "shirt", M::Cloth, &[M::Cloth, M::Tentsilk], &[]),
    f("padded_jacket", "padded jacket", M::Cloth, &[M::Cloth, M::Seareed], &[]),
    f("wraps", "wraps", M::Tentsilk, &[M::Tentsilk, M::Cloth], &[]),
    f("hide_coat", "jerkin", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("scale_hauberk", "scale coat", M::Forgeiron, &[M::Forgeiron, M::Bronze, M::Nacre, M::Slatewing], &[M::Seareed, M::Slatewing, M::Leather]),
    f("sandstone_lamellar", "plate cuirass", M::Bronze, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("leather_cap", "cap", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("iron_helm", "helm", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("trousers", "trousers", M::Cloth, &[M::Cloth], &[]),
    f("hide_leggings", "leggings", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("scale_greaves", "greaves", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing, M::Nacre], &[]),
    f("leather_gloves", "bracers", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("boots", "boots", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("buckler", "buckler", M::Wood, &[M::Wood, M::Bronze, M::Nacre], &[]),
    f("kite_shield", "shield", M::Forgeiron, &[M::Forgeiron, M::Ringstone, M::Bronze], &[]),
    f("small_pack", "small pack", M::Leather, &[M::Leather, M::Seareed], &[]),
    f("large_pack", "large pack", M::Leather, &[M::Leather, M::Seareed, M::Tentsilk], &[]),
];

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl Catalogue {
    fn build() -> Catalogue {
        let mut defs: Vec<ItemDef> = ITEMS.to_vec();
        let mut info: Vec<ItemInfo> = (0..ITEMS.len())
            .map(|i| {
                let main = FORMS.iter().find(|f| f.key == ITEMS[i].key).map(|f| f.usual).unwrap_or(M::None);
                ItemInfo { form: i as ItemId, main, second: M::None, grade: Grade::Common }
            })
            .collect();
        let mut variants = std::collections::HashMap::new();
        for form in FORMS {
            let fid = ITEMS.iter().position(|d| d.key == form.key).unwrap_or_else(|| panic!("form {} has no item", form.key)) as ItemId;
            let base = ITEMS[fid as usize];
            // The plain version gets its usual material's point too.
            if let Kind::Weapon(w) = &mut defs[fid as usize].kind {
                w.pierce = form.usual.def().pierce;
            }
            for &main in form.mains {
                for second in std::iter::once(M::None).chain(form.seconds.iter().copied()) {
                    if second == main {
                        continue;
                    }
                    for grade in GRADES {
                        if main == form.usual && second == M::None && grade == Grade::Common {
                            variants.insert((fid, main, second, grade), fid);
                            continue;
                        }
                        let id = defs.len() as ItemId;
                        defs.push(made(&base, form, main, second, grade));
                        info.push(ItemInfo { form: fid, main, second, grade });
                        variants.insert((fid, main, second, grade), id);
                    }
                }
            }
        }
        let by_key = defs.iter().enumerate().map(|(i, d)| (d.key, i as ItemId)).collect();
        Catalogue { defs, info, by_key, variants }
    }
}

/// A form's numbers in other materials at another grade, scaled from its
/// usual version.
fn made(base: &ItemDef, form: &Form, main: Material, second: Material, grade: Grade) -> ItemDef {
    let (u, a) = (form.usual.def(), main.def());
    let b = if second == M::None { a } else { second.def() };
    let mass = (a.mass + b.mass) * 0.5;
    let density = (a.density + b.density) * 0.5;
    let turn_blunt = (a.turn_blunt + b.turn_blunt) * 0.5;
    let p = grade.power();
    let kind = match base.kind {
        Kind::Weapon(w) => Kind::Weapon(WeaponDef {
            cut: w.cut * a.edge / u.edge * p,
            blunt: w.blunt * mass / u.mass * p,
            pierce: a.pierce.max(b.pierce),
            ..w
        }),
        Kind::Armor(x) => Kind::Armor(ArmorDef {
            cut: (x.cut * a.turn_cut / u.turn_cut * p).min(0.92),
            blunt: (x.blunt * turn_blunt / u.turn_blunt * p).min(0.92),
            dodge_penalty: x.dodge_penalty * density / u.density,
            ..x
        }),
        Kind::Shield(s) => Kind::Shield((s * (a.turn_cut + turn_blunt) / (u.turn_cut + u.turn_blunt) * p).min(0.8)),
        k => k,
    };
    let backing = if second == M::None { String::new() } else { format!(" on {}", second.name()) };
    let grade_word = if grade == Grade::Common { String::new() } else { format!("{} ", grade.name()) };
    let worth = a.worth + if second == M::None { 0.0 } else { b.worth * 0.3 };
    ItemDef {
        key: leak(format!("{}~{:?}~{:?}~{:?}", form.key, main, second, grade)),
        name: leak(format!("{grade_word}{} {}{backing}", main.name(), form.noun)),
        slot: base.slot,
        kind,
        weight: base.weight * density / u.density,
        value: (base.value * worth / u.worth * grade.worth()).max(1.0),
        effects: base.effects,
    }
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
pub const FISTS: WeaponDef = WeaponDef { skill: Skill::Unarmed, cut: 0.0, blunt: 4.0, reach: 0.8, windup: 0.35, recover: 0.35, two_handed: false, parry: 0.0, range: 0.0, ammo: None, pierce: 0.0, entangle: 0.0 };
