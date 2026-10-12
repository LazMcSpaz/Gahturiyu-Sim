//! Mundane goods, folk charms and valuables (Part 7 of
//! `docs/items-design.md`). None has a use beyond being owned: they're sold,
//! traded, stolen, given as bribes, and found in homes, packs and loot so the
//! world feels lived-in. Folk charms carry no magic; people notice them.

use super::*;

/// A thing that's only owned (held in the hand, if anything).
const fn goods(key: &'static str, name: &'static str, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Trinket, weight, value, effects: &[] }
}

pub(super) static ITEMS: &[ItemDef] = &[
    // --- Household ------------------------------------------------------------
    goods("clay_cup", "Clay cup", 0.2, 1.0),
    goods("clay_bowl", "Clay bowl", 0.3, 1.0),
    goods("hearthclay_jug", "Hearthclay jug", 1.0, 4.0),
    goods("wooden_spoon", "Wooden spoon", 0.05, 0.5),
    goods("ladle", "Ladle", 0.2, 1.0),
    goods("trencher", "Carved trencher", 0.3, 2.0),
    goods("tallow_candle", "Tallow candle", 0.1, 1.0),
    goods("candle_stub", "Candle stub", 0.03, 0.2),
    goods("rushlight", "Rushlight", 0.02, 0.3),
    goods("pot_hook", "Iron pot hook", 0.4, 3.0),
    goods("bronze_lamp", "Bronze lamp", 0.6, 12.0),
    goods("woven_basket", "Woven basket", 0.5, 3.0),
    goods("reed_mat", "Reed mat", 0.8, 2.0),
    goods("wool_blanket", "Wool blanket", 1.5, 8.0),
    goods("broom", "Broom", 0.8, 2.0),
    // --- Personal ---------------------------------------------------------------
    goods("bone_comb", "Bone comb", 0.03, 2.0),
    goods("horn_comb", "Horn comb", 0.04, 3.0),
    goods("pipe", "Pipe", 0.1, 4.0),
    goods("bone_dice", "Bone dice", 0.02, 2.0),
    goods("game_stones", "Set of game stones", 0.3, 3.0),
    goods("sewing_kit", "Sewing kit", 0.2, 6.0),
    goods("thread_spool", "Spool of thread", 0.05, 1.0),
    goods("bone_needles", "Bone needles", 0.01, 1.0),
    goods("bronze_mirror", "Small mirror", 0.2, 15.0),
    goods("purse", "Purse", 0.1, 3.0),
    // --- Writing and reading (the Ṭaḍoro trade) ---------------------------------
    goods("blank_sheets", "Blank sheets", 0.05, 3.0),
    goods("quill", "Quill", 0.01, 1.0),
    goods("letter", "Letter", 0.02, 1.0),
    goods("ledger", "Ledger", 0.8, 10.0),
    goods("map_fragment", "Old map fragment", 0.02, 15.0),
    goods("book_of_songs", "Book of songs", 0.6, 25.0),
    goods("book_of_prayers", "Book of prayers", 0.6, 25.0),
    // --- Work leftovers -----------------------------------------------------------
    goods("bent_nails", "Bent nails", 0.1, 0.5),
    goods("iron_scrap", "Iron scrap", 0.5, 2.0),
    goods("broken_blade", "Broken blade", 0.4, 3.0),
    goods("net_fragment", "Net fragment", 0.3, 1.0),
    goods("frayed_rope", "Frayed rope", 0.4, 1.0),
    goods("cracked_pot", "Cracked pot", 0.6, 0.5),
    goods("tool_handle", "Worn tool handle", 0.3, 0.5),
    // --- Folk charms (no magic; each people's own, and some anyone keeps) -------
    goods("ring_stone_pebble", "Ring-stone pebble", 0.05, 3.0),
    goods("home_chip", "Home chip", 0.05, 2.0),
    goods("carved_turiyu", "Carved Turiyu", 0.1, 6.0),
    goods("sea_cord", "Knotted sea cord", 0.03, 2.0),
    goods("shell_rattle", "Shell rattle", 0.1, 3.0),
    goods("drowned_bell", "Drowned bell", 0.05, 8.0),
    goods("sun_charm", "Sun charm", 0.05, 4.0),
    goods("ember_bead", "Ember bead", 0.01, 3.0),
    goods("sand_vial", "Sand vial", 0.05, 4.0),
    goods("folded_prayer", "Folded prayer", 0.01, 2.0),
    goods("quill_charm", "Feather quill charm", 0.01, 2.0),
    goods("road_token", "Road token", 0.02, 5.0),
    goods("ashberry_ward", "Ashberry ward", 0.05, 1.0),
    goods("stormbell_locket", "Stormbell locket", 0.05, 10.0),
    goods("lucky_stone", "Lucky stone", 0.1, 2.0),
    goods("moorbell_posy", "Moorbell posy", 0.02, 1.0),
    // --- Valuables (much worth in little weight; what thieves want) -------------
    goods("storm_glass_bead", "Storm glass bead", 0.01, 20.0),
    goods("bronze_bangle", "Bronze bangle", 0.2, 25.0),
    trinket("silver_ring", "Silver ring", Slot::Ring, 0.02, 45.0, &[]),
    goods("silver_bar", "Silver bar", 0.3, 80.0),
    goods("carved_nacre", "Carved Nacre", 0.1, 60.0),
    goods("cut_gem", "Cut gem", 0.01, 150.0),
    trinket("gold_chain", "Gold chain", Slot::Neck, 0.05, 180.0, &[]),
    goods("edgeglass_sliver", "Edgeglass sliver", 0.02, 120.0),
    goods("gold_cup", "Engraved gold cup", 0.8, 400.0),
    goods("black_pearl", "Black pearl", 0.01, 500.0),
];

pub(super) static FORMS: &[Form] = &[];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("hearthclay_jug", 1, 2),
    ("woven_basket", 2, 2),
    ("reed_mat", 1, 3),
    ("wool_blanket", 2, 2),
    ("broom", 1, 4),
    ("ledger", 1, 2),
    ("book_of_songs", 1, 2),
    ("book_of_prayers", 1, 2),
    ("frayed_rope", 2, 1),
    ("cracked_pot", 2, 2),
    ("tool_handle", 1, 2),
    ("broken_blade", 1, 2),
    ("gold_cup", 1, 2),
];

/// Each people's folk charms (Part 7): what a traveller of theirs might carry.
pub const CHARMS: &[(crate::sim::race::Race, &[&str])] = &[
    (crate::sim::race::Race::Roduro, &["ring_stone_pebble", "home_chip", "carved_turiyu"]),
    (crate::sim::race::Race::Horaro, &["sea_cord", "shell_rattle", "drowned_bell"]),
    (crate::sim::race::Race::Qotiro, &["sun_charm", "ember_bead", "sand_vial"]),
    (crate::sim::race::Race::Tadoro, &["folded_prayer", "quill_charm", "road_token"]),
];

/// Charms anyone keeps.
pub const ANYONE_CHARMS: &[&str] = &["ashberry_ward", "stormbell_locket", "lucky_stone", "moorbell_posy"];

/// What a home's cupboards hold besides clothes.
pub const HOUSEHOLD: &[&str] = &[
    "clay_cup", "clay_bowl", "hearthclay_jug", "wooden_spoon", "ladle", "trencher", "tallow_candle", "candle_stub", "rushlight", "pot_hook", "woven_basket", "reed_mat", "wool_blanket", "broom",
    "bone_comb", "horn_comb", "pipe", "bone_dice", "game_stones", "sewing_kit", "thread_spool", "bone_needles", "purse", "quill", "letter", "bent_nails", "iron_scrap", "frayed_rope", "cracked_pot", "tool_handle",
];

/// What a chest might hide that's worth stealing.
pub const VALUABLES: &[&str] = &["storm_glass_bead", "bronze_bangle", "silver_ring", "silver_bar", "carved_nacre", "bronze_mirror", "bronze_lamp", "ledger", "book_of_songs", "book_of_prayers", "map_fragment"];

/// The rarest, for ruins and the rich.
pub const TREASURE: &[&str] = &["cut_gem", "gold_chain", "edgeglass_sliver", "gold_cup", "black_pearl"];
