//! Drinks (Part 5, "Alcohol"; Laz, 2026-10-11): bought only for now. Each has
//! a strength (how much drunker a bottle makes you, `Kind::Drink`) and one
//! small effect, a weak version of one from its main ingredient, for about
//! an hour (`World::drink`).

use super::*;
use crate::sim::effects::lasting;

const H: f32 = 3600.0;
const ME: Reach = Reach::Caster;

const fn drink(key: &'static str, name: &'static str, weight: f32, value: f32, strength: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Drink(strength), weight, value, effects }
}

/// How drunk a bottle makes you, by kind (points on the 0..100 scale).
const MILD: f32 = 12.0;
const MEDIUM: f32 = 22.0;
const STRONG: f32 = 35.0;

pub(super) static ITEMS: &[ItemDef] = &[
    drink("hardgrain_ale", "Hardgrain ale", 0.6, 2.0, MILD, &[now(Does::Stamina, 15.0, ME)]),
    drink("stonebean_dark", "Stonebean dark", 0.6, 2.0, MILD + 2.0, &[lasting(Does::Warmth, 0.1, H, ME)]),
    drink("stoneapple_cider", "Stoneapple cider", 0.6, 2.0, MILD, &[now(Does::Rouse, 1.0, ME)]),
    drink("moorberry_wine", "Moorberry wine", 0.5, 5.0, MEDIUM, &[lasting(Does::EnergyRegen, 0.2, H, ME)]),
    drink("ashberry_wine", "Ashberry wine", 0.5, 6.0, MEDIUM, &[lasting(Does::Warmth, 0.15, H, ME)]),
    drink("nightberry_wine", "Nightberry wine", 0.5, 18.0, MEDIUM, &[lasting(Does::Hide, 0.1, H, ME)]),
    drink("brine_wine", "Brine wine", 0.5, 6.0, MEDIUM, &[lasting(Does::DeepBreath, 0.3, H, ME)]),
    drink("stone_spirit", "Stone spirit", 0.4, 10.0, STRONG, &[lasting(Does::Steady, 0.15, H, ME)]),
    drink("sacrament_spirit", "Sacrament spirit", 0.4, 40.0, STRONG, &[lasting(Does::ResistFire, 0.2, H, ME)]),
    drink("gold_cordial", "Gold thread cordial", 0.4, 14.0, MILD - 2.0, &[now(Does::Energy, 10.0, ME)]),
];

pub(super) static FORMS: &[Form] = &[];

pub(super) static SIZES: &[(&str, u8, u8)] = &[];

/// What else a drink does (rules 5 and 6): thirst taken away (ale and cider
/// quench a little; wine and spirits add to it, negative), and toxicity
/// (spirits a little).
pub static DRINK_SIDES: &[(&str, f32, f32)] = &[
    ("hardgrain_ale", 10.0, 0.0),
    ("stonebean_dark", 10.0, 0.0),
    ("stoneapple_cider", 10.0, 0.0),
    ("moorberry_wine", -5.0, 0.0),
    ("ashberry_wine", -5.0, 0.0),
    ("nightberry_wine", -5.0, 0.0),
    ("brine_wine", -5.0, 0.0),
    ("stone_spirit", -10.0, 4.0),
    ("sacrament_spirit", -10.0, 4.0),
    ("gold_cordial", 0.0, 0.0),
];
