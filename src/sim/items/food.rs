//! Food (Part 5 of `docs/items-design.md`).

use super::*;

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
    food("roast_meat", "Roast meat", 0.4, 4.0, 30.0),
    food("spit_fish", "Spit-roast fish", 0.3, 4.0, 24.0),
    food("fried_puffcaps", "Fried puffcaps", 0.2, 3.0, 12.0),
    food("camp_porridge", "Camp porridge", 0.4, 2.0, 22.0),
    food("moorbell_tea", "Moorbell tea", 0.3, 2.0, 4.0),
    // At a stove:
    food("sweetroot_stew", "Sweetroot stew", 0.5, 6.0, 45.0),
    food("fish_broth", "Fish and kale broth", 0.5, 5.0, 35.0),
    food("mussel_pot", "Mussel pot", 0.5, 6.0, 40.0),
    food("stonebean_stew", "Stonebean stew", 0.6, 5.0, 55.0),
    food("hardgrain_bread", "Hardgrain bread", 0.4, 3.0, 30.0),
    food("shellhen_bake", "Shellhen bake", 0.4, 7.0, 45.0),
    food("glazed_meat", "Ashberry-glazed meat", 0.4, 7.0, 40.0),
    food("trail_cake", "Trail cake", 0.2, 4.0, 22.0),
    food("stoneapple_cake", "Stoneapple cake", 0.4, 7.0, 35.0),
    food("firepod_stew", "Firepod stew", 0.6, 9.0, 45.0),
];

/// The cooked dishes: each comes in every grade (Crude to Masterwork), from
/// the cook's skill (`Catalogue::build`); a better one fills more.
pub static DISHES: &[&str] = &[
    "roast_meat", "spit_fish", "fried_puffcaps", "camp_porridge", "moorbell_tea",
    "sweetroot_stew", "fish_broth", "mussel_pot", "stonebean_stew", "hardgrain_bread",
    "shellhen_bake", "glazed_meat", "trail_cake", "stoneapple_cake", "firepod_stew",
];

/// How much a dish of each grade fills, against a Common one.
pub const DISH_FILLS: [f32; 4] = [0.8, 1.0, 1.15, 1.3];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
