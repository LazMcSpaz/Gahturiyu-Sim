//! Food (Part 5 of `docs/items-design.md`).

use super::*;
use crate::sim::effects::lasting;

const H: f32 = 3600.0;
const ME: Reach = Reach::Caster;

/// A cooked dish with its meal buff (Part 5, "Meal buffs": one at a time,
/// some hours; the grade scales it, `World::meal_buff`).
const fn dish(key: &'static str, name: &'static str, weight: f32, value: f32, nourishment: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Food(nourishment), weight, value, effects }
}

pub(super) static ITEMS: &[ItemDef] = &[
    // --- Food (placeholder names) ---------------------------------------------
    food("dried_fish", "Dried fish", 0.3, 4.0, 25.0),
    food("flatbread", "Flatbread", 0.4, 3.0, 30.0),
    food("salted_meat", "Salted meat", 0.5, 6.0, 40.0),
    food("wild_berries", "Moorberry", 0.1, 1.0, 8.0),
    food("mussels", "Mussels", 0.3, 2.0, 14.0),
    food("raw_meat", "Raw meat", 0.5, 2.0, 18.0),
    food("egg", "Egg", 0.1, 1.0, 6.0),
    // Raw, for cooking (0.6): what fishers land, and a Mossback's yield.
    food("fish", "Raw fish", 0.4, 2.0, 14.0),
    food("milk", "Mossback milk", 0.5, 1.0, 6.0),
    food("cheese", "Mossback cheese", 0.3, 4.0, 18.0),
    // Dishes ("Food recipes"): cooked fills more than raw. Over a campfire:
    // (Campfire food fills, and gives a small buff at most.)
    food("roast_meat", "Roast meat", 0.4, 4.0, 30.0),
    food("spit_fish", "Spit-roast fish", 0.3, 4.0, 24.0),
    food("fried_puffcaps", "Fried puffcaps", 0.2, 3.0, 12.0),
    dish("camp_porridge", "Camp porridge", 0.4, 2.0, 22.0, &[lasting(Does::Breath, 0.15, 3.0 * H, ME)]),
    dish("moorbell_tea", "Moorbell tea", 0.3, 2.0, 4.0, &[lasting(Does::Warmth, 0.15, 2.0 * H, ME)]),
    // At a stove: full meals, with a proper buff.
    dish("sweetroot_stew", "Sweetroot stew", 0.5, 6.0, 45.0, &[lasting(Does::Mending, 0.3, 6.0 * H, ME)]),
    dish("fish_broth", "Fish and kale broth", 0.5, 5.0, 35.0, &[lasting(Does::Warmth, 0.35, 6.0 * H, ME)]),
    dish("mussel_pot", "Mussel pot", 0.5, 6.0, 40.0, &[lasting(Does::Breath, 0.3, 6.0 * H, ME)]),
    dish("stonebean_stew", "Stonebean stew", 0.6, 5.0, 55.0, &[lasting(Does::Carry, 8.0, 6.0 * H, ME)]),
    food("hardgrain_bread", "Hardgrain bread", 0.4, 3.0, 30.0),
    dish("shellhen_bake", "Shellhen bake", 0.4, 7.0, 45.0, &[lasting(Does::Mending, 0.15, 4.0 * H, ME)]),
    dish("glazed_meat", "Ashberry-glazed meat", 0.4, 7.0, 40.0, &[lasting(Does::Warmth, 0.3, 6.0 * H, ME)]),
    dish("trail_cake", "Trail cake", 0.2, 4.0, 22.0, &[lasting(Does::Breath, 0.1, 3.0 * H, ME)]),
    dish("stoneapple_cake", "Stoneapple cake", 0.4, 7.0, 35.0, &[lasting(Does::Breath, 0.3, 6.0 * H, ME)]),
    dish("firepod_stew", "Firepod stew", 0.6, 9.0, 45.0, &[lasting(Does::Warmth, 0.5, 8.0 * H, ME)]),
];

/// The cooked dishes: each comes in every grade (Crude to Masterwork), from
/// the cook's skill (`Catalogue::build`); a better one fills more.
pub static DISHES: &[&str] = &[
    "roast_meat", "spit_fish", "fried_puffcaps", "camp_porridge", "moorbell_tea",
    "sweetroot_stew", "fish_broth", "mussel_pot", "stonebean_stew", "hardgrain_bread",
    "shellhen_bake", "glazed_meat", "trail_cake", "stoneapple_cake", "firepod_stew",
];

/// How much a dish of each grade fills (and how strong its buff), against a
/// Common one.
pub const DISH_FILLS: [f32; 4] = [0.8, 1.0, 1.15, 1.3];

/// Teas and broths quench thirst too (Part 5, rule 6): how much.
pub static QUENCH: &[(&str, f32)] = &[("moorbell_tea", 30.0), ("fish_broth", 20.0)];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
