//! The alchemist's ingredients (Part 4 of `docs/items-design.md`, Laz's
//! approved list after the balance review): each carries four effects, the
//! same in every world; a potion carries the effects shared by two or more of
//! the ingredients in it, from at least two different kinds. This file is the
//! data, with what each effect does in a dose (`Alch::dose`), and what a
//! brewed bottle carries (`Mix`: Laz's option (a), each bottle its own mix;
//! bottles stack only when their mixes are the same).

use super::*;
use crate::sim::effects::{lasting, now, Effect, Lasts};
use crate::sim::materials::Grade;
use crate::sim::stats::{Attr, Skill};
use Alch as E;
use Herb as H;
use Wild as W;

/// What an ingredient can do, helpful or harmful.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Alch {
    Heal,
    Stanch,
    RestoreStamina,
    RestoreEnergy,
    Rouse,
    Steady,
    Cleanse,
    Strength,
    Swiftness,
    Shadow,
    NightSight,
    Warmth,
    ResistFire,
    Toughen,
    ClearMind,
    KeenSenses,
    DeepBreath,
    // Harmful.
    Poison,
    Weakness,
    Sluggish,
    Daze,
    Slumber,
    Blind,
    Nausea,
    ThinBlood,
    Burn,
    Smoke,
}

/// How hard an effect should be to come by (the balance review's tiers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Common,
    Moderate,
    Powerful,
}

impl Alch {
    pub fn harmful(self) -> bool {
        matches!(self, E::Poison | E::Weakness | E::Sluggish | E::Daze | E::Slumber | E::Blind | E::Nausea | E::ThinBlood | E::Burn | E::Smoke)
    }

    pub fn tier(self) -> Tier {
        match self {
            E::Heal | E::RestoreStamina | E::Warmth | E::Cleanse | E::Rouse | E::Stanch | E::Nausea | E::Daze | E::Weakness | E::Sluggish => Tier::Common,
            E::Strength | E::Swiftness | E::Toughen | E::Shadow | E::NightSight | E::Poison | E::Slumber => Tier::Powerful,
            _ => Tier::Moderate,
        }
    }

    /// What one plain dose does to whoever drinks it (a Common potion from a
    /// bare bench; brewing scales the power and the time). Burn and Smoke are
    /// for thrown flasks: drunk, they only sicken (toxicity).
    pub fn dose(self) -> &'static [Effect] {
        match self {
            E::Heal => {
                const D: &[Effect] = &[now(Does::Heal, 20.0, Reach::Caster)];
                D
            }
            E::Stanch => {
                const D: &[Effect] = &[now(Does::Staunch, 1.0, Reach::Caster)];
                D
            }
            E::RestoreStamina => {
                const D: &[Effect] = &[now(Does::Stamina, 30.0, Reach::Caster)];
                D
            }
            E::RestoreEnergy => {
                const D: &[Effect] = &[now(Does::Energy, 20.0, Reach::Caster)];
                D
            }
            E::Rouse => {
                const D: &[Effect] = &[now(Does::Rouse, 1.0, Reach::Caster)];
                D
            }
            E::Steady => {
                const D: &[Effect] = &[lasting(Does::Steady, 0.4, 300.0, Reach::Caster)];
                D
            }
            E::Cleanse => {
                const D: &[Effect] = &[now(Does::Cleanse, 25.0, Reach::Caster)];
                D
            }
            E::Strength => {
                const D: &[Effect] = &[lasting(Does::Attr(Attr::Strength), 10.0, 300.0, Reach::Caster)];
                D
            }
            E::Swiftness => {
                const D: &[Effect] = &[lasting(Does::Haste, 0.2, 120.0, Reach::Caster)];
                D
            }
            E::Shadow => {
                const D: &[Effect] = &[lasting(Does::Hide, 0.3, 600.0, Reach::Caster)];
                D
            }
            E::NightSight => {
                const D: &[Effect] = &[lasting(Does::Nightsight, 0.6, 1800.0, Reach::Caster)];
                D
            }
            E::Warmth => {
                const D: &[Effect] = &[lasting(Does::Warmth, 0.5, 3600.0, Reach::Caster)];
                D
            }
            E::ResistFire => {
                const D: &[Effect] = &[lasting(Does::ResistFire, 0.4, 300.0, Reach::Caster)];
                D
            }
            E::Toughen => {
                const D: &[Effect] = &[lasting(Does::Toughen, 0.15, 300.0, Reach::Caster)];
                D
            }
            E::ClearMind => {
                const D: &[Effect] = &[lasting(Does::Skill(Skill::Structured), 10.0, 600.0, Reach::Caster), lasting(Does::EnergyRegen, 0.3, 600.0, Reach::Caster)];
                D
            }
            E::KeenSenses => {
                const D: &[Effect] = &[lasting(Does::KeenSenses, 0.3, 900.0, Reach::Caster)];
                D
            }
            E::DeepBreath => {
                const D: &[Effect] = &[lasting(Does::DeepBreath, 1.0, 600.0, Reach::Caster)];
                D
            }
            E::Poison => {
                const D: &[Effect] = &[lasting(Does::Poison, 0.5, 30.0, Reach::Caster)];
                D
            }
            E::Weakness => {
                const D: &[Effect] = &[lasting(Does::Attr(Attr::Strength), -10.0, 300.0, Reach::Caster)];
                D
            }
            E::Sluggish => {
                const D: &[Effect] = &[lasting(Does::Slow, 0.2, 60.0, Reach::Caster)];
                D
            }
            E::Daze => {
                const D: &[Effect] = &[now(Does::Daze, 2.0, Reach::Caster)];
                D
            }
            E::Slumber => {
                const D: &[Effect] = &[lasting(Does::Slumber, 1.0, 20.0, Reach::Caster)];
                D
            }
            E::Blind => {
                const D: &[Effect] = &[lasting(Does::Blind, 0.5, 20.0, Reach::Caster)];
                D
            }
            E::Nausea => {
                const D: &[Effect] = &[lasting(Does::Nausea, 0.5, 60.0, Reach::Caster)];
                D
            }
            E::ThinBlood => {
                const D: &[Effect] = &[lasting(Does::ThinBlood, 0.5, 300.0, Reach::Caster)];
                D
            }
            E::Burn | E::Smoke => &[],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            E::Heal => "Heal",
            E::Stanch => "Stanch",
            E::RestoreStamina => "Restore stamina",
            E::RestoreEnergy => "Restore energy",
            E::Rouse => "Rouse",
            E::Steady => "Steady",
            E::Cleanse => "Cleanse",
            E::Strength => "Strength",
            E::Swiftness => "Swiftness",
            E::Shadow => "Shadow",
            E::NightSight => "Night sight",
            E::Warmth => "Warmth",
            E::ResistFire => "Resist fire",
            E::Toughen => "Toughen",
            E::ClearMind => "Clear mind",
            E::KeenSenses => "Keen senses",
            E::DeepBreath => "Deep breath",
            E::Poison => "Poison",
            E::Weakness => "Weakness",
            E::Sluggish => "Sluggish",
            E::Daze => "Daze",
            E::Slumber => "Slumber",
            E::Blind => "Blind",
            E::Nausea => "Nausea",
            E::ThinBlood => "Thin blood",
            E::Burn => "Burn",
            E::Smoke => "Smoke",
        }
    }
}

/// The kinds of ingredient (a potion wants two different kinds). Oils are
/// their own kind; fungi and mosses are two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Herb {
    Berry,
    Flower,
    Leaf,
    Root,
    Mineral,
    Fungus,
    Moss,
    Animal,
    Crop,
    Spice,
    Oil,
}

/// Where it's found (the island's biomes, Laz 2026-10-11), grown, or brought
/// in by ship.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wild {
    Plains,
    Forest,
    Coast,
    Bog,
    Rocky,
    Snowy,
    Overgrowth,
    Farmed,
    Imported,
    /// Pressed or rendered from another ingredient at the alchemy bench.
    Extracted,
}

/// One ingredient: its item, kind, four effects, where it's found, and how
/// easily it's had (3 common, 2 uncommon, 1 rare or costly).
#[derive(Clone, Copy, Debug)]
pub struct Ingredient {
    pub key: &'static str,
    pub kind: Herb,
    pub effects: [Alch; 4],
    pub found: Wild,
    pub common: u8,
}

const fn ing(key: &'static str, kind: Herb, effects: [Alch; 4], found: Wild, common: u8) -> Ingredient {
    Ingredient { key, kind, effects, found, common }
}

/// Every ingredient (the approved list, Part 4).
pub static INGREDIENTS: &[Ingredient] = &[
    // Berries.
    ing("wild_berries", H::Berry, [E::RestoreStamina, E::Heal, E::ClearMind, E::Nausea], W::Plains, 3),
    ing("ashberry", H::Berry, [E::Warmth, E::Rouse, E::ResistFire, E::Nausea], W::Rocky, 3),
    ing("nightberry", H::Berry, [E::NightSight, E::Shadow, E::Poison, E::ThinBlood], W::Forest, 2),
    // Flowers.
    ing("stormbell", H::Flower, [E::Heal, E::ClearMind, E::Steady, E::Daze], W::Coast, 2),
    ing("moorbell", H::Flower, [E::Rouse, E::Warmth, E::KeenSenses, E::Daze], W::Bog, 3),
    ing("mirebloom", H::Flower, [E::Shadow, E::Cleanse, E::Slumber, E::Poison], W::Bog, 2),
    // Leaves.
    ing("bloodleaf", H::Leaf, [E::Stanch, E::Heal, E::Cleanse, E::Sluggish], W::Forest, 3),
    ing("frostleaf", H::Leaf, [E::Warmth, E::Steady, E::RestoreEnergy, E::Weakness], W::Snowy, 2),
    ing("creeperleaf", H::Leaf, [E::Strength, E::Swiftness, E::KeenSenses, E::Poison], W::Overgrowth, 3),
    // Roots.
    ing("sweetroot", H::Root, [E::RestoreStamina, E::Heal, E::Rouse, E::Daze], W::Plains, 3),
    ing("emberroot", H::Root, [E::Strength, E::ResistFire, E::RestoreStamina, E::Burn], W::Rocky, 2),
    ing("peatroot", H::Root, [E::Cleanse, E::RestoreEnergy, E::Slumber, E::Nausea], W::Bog, 3),
    // Minerals.
    ing("salt_crystal", H::Mineral, [E::Cleanse, E::RestoreStamina, E::DeepBreath, E::Weakness], W::Coast, 3),
    ing("storm_glass", H::Mineral, [E::Swiftness, E::KeenSenses, E::ClearMind, E::Blind], W::Coast, 2),
    ing("rimestone", H::Mineral, [E::Stanch, E::NightSight, E::Steady, E::Sluggish], W::Snowy, 2),
    // Fungi.
    ing("ghostcap", H::Fungus, [E::RestoreEnergy, E::NightSight, E::ClearMind, E::Daze], W::Forest, 2),
    ing("puffcap", H::Fungus, [E::KeenSenses, E::Rouse, E::Smoke, E::Blind], W::Plains, 3),
    ing("vinecap", H::Fungus, [E::Toughen, E::Heal, E::RestoreEnergy, E::Nausea], W::Overgrowth, 3),
    // Mosses.
    ing("weepmoss", H::Moss, [E::Swiftness, E::RestoreStamina, E::ThinBlood, E::Poison], W::Bog, 3),
    ing("greymoss", H::Moss, [E::Stanch, E::Warmth, E::DeepBreath, E::Smoke], W::Coast, 3),
    // Animal parts.
    ing("wallowback_fat", H::Animal, [E::Warmth, E::RestoreStamina, E::Toughen, E::Burn], W::Bog, 3),
    ing("tidepicker_roe", H::Animal, [E::RestoreEnergy, E::Heal, E::DeepBreath, E::Nausea], W::Coast, 3),
    ing("ridgehound_musk", H::Animal, [E::KeenSenses, E::Rouse, E::Strength, E::Nausea], W::Plains, 2),
    ing("silk_gland", H::Animal, [E::Steady, E::Stanch, E::Slumber, E::Weakness], W::Rocky, 2),
    ing("briar_heart", H::Animal, [E::Strength, E::Toughen, E::Steady, E::Poison], W::Overgrowth, 1),
    ing("cragmaw_venom", H::Animal, [E::Swiftness, E::Poison, E::Sluggish, E::Daze], W::Rocky, 1),
    // Farm crops.
    ing("grain", H::Crop, [E::RestoreStamina, E::Steady, E::Warmth, E::Sluggish], W::Farmed, 3),
    ing("stonebean", H::Crop, [E::RestoreStamina, E::RestoreEnergy, E::Warmth, E::Nausea], W::Farmed, 3),
    ing("hearthleaf", H::Crop, [E::Cleanse, E::Heal, E::KeenSenses, E::Daze], W::Farmed, 3),
    ing("stoneapple", H::Crop, [E::Rouse, E::ClearMind, E::Heal, E::ThinBlood], W::Farmed, 3),
    ing("bluestem", H::Crop, [E::Stanch, E::Steady, E::RestoreStamina, E::Weakness], W::Farmed, 3),
    ing("firepod", H::Crop, [E::ResistFire, E::Warmth, E::Strength, E::Burn], W::Farmed, 2),
    ing("brinekale", H::Crop, [E::DeepBreath, E::Cleanse, E::RestoreStamina, E::Weakness], W::Farmed, 3),
    // Imported spices.
    ing("sunbark", H::Spice, [E::Swiftness, E::Warmth, E::ClearMind, E::ThinBlood], W::Imported, 1),
    ing("duskseed", H::Spice, [E::Shadow, E::NightSight, E::RestoreEnergy, E::Slumber], W::Imported, 1),
    ing("gold_thread", H::Spice, [E::Heal, E::RestoreEnergy, E::KeenSenses, E::Daze], W::Imported, 1),
];

// ---- Brewing ------------------------------------------------------------------

/// The most ingredients in one brew, and the most effects a bottle carries.
pub const MIX_INGREDIENTS: usize = 4;
pub const MIX_MOST: usize = 6;

/// What a brewed bottle carries: its effects (each shared by two or more of
/// the ingredients), how strong they are, and how long the helpful and the
/// harmful ones last (1 = a plain dose). Kept in the bottle's `Piece`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Mix {
    pub effects: [Option<Alch>; MIX_MOST],
    pub power: f32,
    pub good_for: f32,
    pub bad_for: f32,
}

/// The tools fitted to an alchemy table, each by its grade (none: not
/// fitted).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Kit {
    /// A small boost to everything.
    pub mortar: Option<Grade>,
    /// Shortens harmful effects.
    pub retort: Option<Grade>,
    /// Lengthens helpful effects.
    pub calcinator: Option<Grade>,
    /// Raises strength.
    pub alembic: Option<Grade>,
}

/// How much a tool of this grade does (none fitted: nothing).
fn tool(g: Option<Grade>) -> f32 {
    match g {
        None => 0.0,
        Some(Grade::Crude) => 0.5,
        Some(Grade::Common) => 1.0,
        Some(Grade::Fine) => 1.5,
        Some(Grade::Masterwork) => 2.0,
    }
}

/// Mixes are kept to this step, so bottles brewed the same way stack.
const MIX_STEP: f32 = 0.05;

fn stepped(x: f32) -> f32 {
    (x / MIX_STEP).round() * MIX_STEP
}

/// The effects a brew of these ingredients carries: every effect shared by
/// at least two of them, if they're of at least two kinds (two berries alone
/// make nothing). In a fixed order: helpful first, strongest first.
pub fn shared(ings: &[ItemId]) -> Vec<Alch> {
    let list: Vec<&Ingredient> = ings.iter().filter_map(|&i| super::ingredient(i)).collect();
    if list.len() < 2 || list.iter().all(|i| i.kind == list[0].kind) {
        return Vec::new();
    }
    let mut out: Vec<Alch> = Vec::new();
    for (k, a) in list.iter().enumerate() {
        for &e in &a.effects {
            if out.contains(&e) {
                continue;
            }
            if list.iter().enumerate().any(|(j, b)| j != k && b.effects.contains(&e)) {
                out.push(e);
            }
        }
    }
    out.sort_by_key(|e| (e.harmful(), std::cmp::Reverse(e.tier()), *e));
    out
}

impl Mix {
    /// A brew of `effects` by someone of this Alchemy skill (0..100) at a
    /// table with this kit. A bare table and an unskilled hand: weak, short,
    /// and the harm lasts long; good tools and skill turn the harm into a
    /// flicker, but never remove it.
    pub fn brew(effects: &[Alch], skill: f32, kit: Kit) -> Mix {
        let s = (skill / 100.0).clamp(0.0, 1.0);
        let mortar = 0.1 * tool(kit.mortar);
        let hand = 0.5 + s;
        let mut fx = [None; MIX_MOST];
        for (k, &e) in effects.iter().take(MIX_MOST).enumerate() {
            fx[k] = Some(e);
        }
        Mix {
            effects: fx,
            power: stepped(0.6 + 0.6 * s + mortar + 0.25 * tool(kit.alembic) * hand),
            good_for: stepped(0.6 + 0.6 * s + mortar + 0.3 * tool(kit.calcinator) * hand),
            bad_for: stepped((1.6 - 0.6 * s - mortar - 0.35 * tool(kit.retort) * hand).max(0.1)),
        }
    }

    pub fn carries(&self) -> impl Iterator<Item = Alch> + '_ {
        self.effects.iter().flatten().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.effects[0].is_none()
    }

    /// What drinking it does: each effect's plain dose, its strength and its
    /// time scaled by the mix.
    pub fn effects(&self) -> Vec<Effect> {
        let mut out = Vec::new();
        for a in self.carries() {
            let long = if a.harmful() { self.bad_for } else { self.good_for };
            for e in a.dose() {
                let mut e = *e;
                match e.lasts {
                    Lasts::Secs(s) => {
                        e.lasts = Lasts::Secs(s * long);
                        e.power *= self.power;
                    }
                    // A daze's power is its length.
                    _ if e.does == Does::Daze => e.power *= long,
                    _ => e.power *= self.power,
                }
                out.push(e);
            }
        }
        out
    }

    /// The effect it's named for: the strongest helpful one, else the
    /// strongest harmful one.
    pub fn main(&self) -> Option<Alch> {
        self.carries().next()
    }

    /// "Draught of night sight", or "Tincture of slumber" for one that only
    /// harms.
    pub fn name(&self) -> String {
        match self.main() {
            Some(a) if !a.harmful() => format!("Draught of {}", a.name().to_lowercase()),
            Some(a) => format!("Tincture of {}", a.name().to_lowercase()),
            None => "Murky water".to_string(),
        }
    }

    /// Does it carry anything harmful?
    pub fn harms(&self) -> bool {
        self.carries().any(|a| a.harmful())
    }

    /// Its grade: from its strength, less for harm that lingers.
    pub fn grade(&self) -> Grade {
        let score = self.power - if self.harms() { 0.5 * self.bad_for } else { 0.0 };
        match score {
            x if x >= 1.6 => Grade::Masterwork,
            x if x >= 1.15 => Grade::Fine,
            x if x >= 0.7 => Grade::Common,
            _ => Grade::Crude,
        }
    }

    /// Toxicity a bottle adds when drunk: more the stronger it is, and more
    /// for every harmful effect, by how long it lingers.
    pub fn tox(&self) -> f32 {
        let harm = self.carries().filter(|a| a.harmful()).count() as f32;
        8.0 * (0.5 + 0.5 * self.power) + 4.0 * harm * self.bad_for
    }

    /// What a bottle is worth, in coin.
    pub fn worth(&self) -> f32 {
        let each = |a: Alch| match a.tier() {
            Tier::Common => 8.0,
            Tier::Moderate => 14.0,
            Tier::Powerful => 24.0,
        };
        let good: f32 = self.carries().filter(|a| !a.harmful()).map(each).sum::<f32>() * self.power * self.good_for.sqrt();
        let bad: f32 = self.carries().filter(|a| a.harmful()).count() as f32 * 3.0 * self.bad_for;
        (good - bad).max(2.0)
    }
}

/// Oils: three of the base make one, keeping two of its effects, stronger
/// (an oil is its own kind; its other two places are the same two again).
pub static OILS: &[(Ingredient, &str)] = &[
    (ing("bluestem_oil", H::Oil, [E::Stanch, E::RestoreStamina, E::Stanch, E::RestoreStamina], W::Extracted, 2), "bluestem"),
    (ing("ember_oil", H::Oil, [E::ResistFire, E::Burn, E::ResistFire, E::Burn], W::Extracted, 2), "emberroot"),
    (ing("animal_oil", H::Oil, [E::Warmth, E::Toughen, E::Warmth, E::Toughen], W::Extracted, 2), "wallowback_fat"),
];

pub(super) static ITEMS: &[ItemDef] = &[
    //        key               name                weight value
    material("ashberry", "Ashberry", 0.1, 2.0),
    material("nightberry", "Nightberry", 0.1, 6.0),
    material("stormbell", "Stormbell", 0.05, 6.0),
    material("moorbell", "Moorbell", 0.05, 2.0),
    material("mirebloom", "Mirebloom", 0.05, 7.0),
    material("frostleaf", "Frostleaf", 0.1, 6.0),
    material("creeperleaf", "Creeperleaf", 0.1, 5.0),
    material("sweetroot", "Sweetroot", 0.3, 1.0),
    material("peatroot", "Peatroot", 0.3, 1.0),
    material("rimestone", "Rimestone", 0.3, 10.0),
    material("puffcap", "Puffcap", 0.1, 2.0),
    material("vinecap", "Vinecap", 0.1, 4.0),
    material("weepmoss", "Weepmoss", 0.1, 2.0),
    material("greymoss", "Greymoss", 0.1, 2.0),
    material("wallowback_fat", "Wallowback fat", 0.5, 3.0),
    material("tidepicker_roe", "Tidepicker roe", 0.2, 4.0),
    material("ridgehound_musk", "Ridgehound musk", 0.05, 15.0),
    material("cragmaw_venom", "Cragmaw venom", 0.1, 120.0),
    material("stonebean", "Stonebean", 0.2, 1.0),
    material("hearthleaf", "Hearthleaf", 0.05, 1.0),
    material("stoneapple", "Stoneapple", 0.2, 1.0),
    material("bluestem", "Bluestem", 0.1, 1.0),
    material("firepod", "Firepod", 0.1, 8.0),
    material("brinekale", "Brinekale", 0.4, 1.0),
    material("sunbark", "Sunbark", 0.05, 30.0),
    material("duskseed", "Duskseed", 0.02, 35.0),
    material("gold_thread", "Gold thread", 0.01, 60.0),
    material("bluestem_oil", "Bluestem oil", 0.2, 6.0),
    material("ember_oil", "Ember oil", 0.2, 20.0),
];

pub(super) static FORMS: &[Form] = &[];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[];
