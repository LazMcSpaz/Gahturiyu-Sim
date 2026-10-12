//! What things are made from: raw materials, animal parts, grown stock, and
//! the alchemist's ingredients (Part 4 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //        key               name                weight value
    material("kelp_frond", "Kelp frond", 0.2, 2.0),
    material("ghostcap", "Ghostcap", 0.1, 4.0),
    material("bloodleaf", "Bloodleaf", 0.1, 2.0),
    material("emberroot", "Emberroot", 0.2, 6.0),
    material("salt_crystal", "Saltcrust", 0.3, 3.0),
    material("storm_glass", "Storm glass", 0.4, 12.0),
    material("reed_paper", "Reed paper", 0.05, 3.0),
    material("ink", "Ink", 0.2, 5.0),
    material("iron_ore", "Iron ore", 3.0, 4.0),
    material("iron_ingot", "Iron ingot", 2.0, 12.0),
    material("hide", "Hide", 2.0, 6.0),
    material("leather", "Leather", 1.0, 10.0),
    material("timber", "Timber", 2.5, 3.0),
    // What hunted animals give (placeholder names; `labour::YIELD_ITEMS`).
    material("light_hide", "Light hide", 1.0, 3.0),
    material("thick_hide", "Thick hide", 4.0, 14.0),
    material("tough_skin", "Tough skin", 2.0, 8.0),
    material("shell", "Shell", 0.5, 3.0),
    material("horn_plate", "Horn plate", 1.5, 10.0),
    material("teeth", "Teeth", 0.05, 1.0),
    material("animal_oil", "Tallow", 0.5, 3.0),
    material("silk_gland", "Silk gland", 0.3, 40.0),
    material("briar_heart", "Briar heart", 1.0, 80.0),
    material("lurker_trophy", "Lurker trophy", 2.0, 120.0),
    material("cragmaw_trophy", "Cragmaw trophy", 5.0, 300.0),
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
    // Grown at an outpost's field plot; baked into flatbread.
    material("grain", "Hardgrain", 1.0, 1.0),
    material("cloth", "Cloth", 0.5, 4.0),
    // Grown stock, from the Tenders' beds.
    material("ringstone", "Ringstone block", 6.0, 12.0),
    material("slatewing", "Slatewing sheet", 1.0, 20.0),
    material("edgeglass", "Edgeglass blank", 0.6, 70.0),
    material("hearthclay", "Hearthclay", 1.5, 3.0),
    // Made for trade.
    material("bronze_ingot", "Bronze ingot", 2.0, 8.0),
];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
