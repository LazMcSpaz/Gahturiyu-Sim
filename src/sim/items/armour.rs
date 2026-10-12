//! Armour and shields (Part 2 of `docs/items-design.md`): Laz's forms for
//! the head, the body under and over, hands, legs under and over, feet and the
//! off hand. Every piece has a reason to wear it, a trade of protection
//! against what it costs (weight, dodge, noise, sight, warmth); those other
//! numbers are in `WEARS`. Materials and grades multiply them (`FORMS`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    // --- Head ----------------------------------------------------------------
    armor("leather_cap", "Leather cap", Slot::Head, HEAD, 0.70, 0.25, 0.20, 0.00, 0.8, 15.0),
    armor("padded_cap", "Padded cap", Slot::Head, HEAD, 0.70, 0.10, 0.25, 0.00, 0.5, 10.0),
    armor("mail_coif", "Mail coif", Slot::Head, HEAD, 0.85, 0.55, 0.15, 0.01, 2.0, 70.0),
    armor("iron_helm", "Open helm", Slot::Head, HEAD, 0.85, 0.60, 0.40, 0.02, 1.8, 120.0),
    armor("full_helm", "Full helm", Slot::Head, HEAD, 0.95, 0.70, 0.50, 0.03, 3.0, 200.0),
    // --- Body, under (beneath a coat; most are hard to spot) -----------------
    armor("padded_vest", "Padded vest", Slot::BodyUnder, TORSO, 0.90, 0.20, 0.35, 0.00, 2.0, 30.0),
    armor("mail_shirt", "Mail shirt", Slot::BodyUnder, BODY, 0.85, 0.60, 0.15, 0.03, 9.0, 180.0),
    armor("fishskin_undercoat", "Fishskin undercoat", Slot::BodyUnder, BODY, 0.85, 0.25, 0.15, 0.00, 1.5, 45.0),
    armor("fur_jerkin", "Fur jerkin", Slot::BodyUnder, TORSO, 0.85, 0.20, 0.25, 0.01, 3.0, 40.0),
    // --- Body, over -----------------------------------------------------------
    armor("padded_jacket", "Padded jacket", Slot::BodyOver, BODY, 0.90, 0.25, 0.35, 0.02, 3.0, 40.0),
    armor("hide_coat", "Hide coat", Slot::BodyOver, BODY, 0.85, 0.35, 0.25, 0.03, 5.0, 70.0),
    armor("brigandine", "Brigandine", Slot::BodyOver, BODY, 0.85, 0.50, 0.35, 0.05, 8.0, 200.0),
    armor("scale_hauberk", "Scale hauberk", Slot::BodyOver, BODY, 0.85, 0.60, 0.30, 0.06, 10.0, 260.0),
    armor("plate_cuirass", "Plate cuirass", Slot::BodyOver, TORSO, 0.95, 0.70, 0.45, 0.03, 12.0, 300.0),
    armor("stone_cuirass", "Stone cuirass", Slot::BodyOver, TORSO, 0.95, 0.55, 0.70, 0.12, 20.0, 280.0),
    armor("travelling_robe", "Travelling robe", Slot::BodyOver, BODY, 0.90, 0.08, 0.08, 0.00, 1.5, 40.0),
    // --- Hands ----------------------------------------------------------------
    armor("leather_gloves", "Leather gloves", Slot::Hands, ARMS, 0.35, 0.20, 0.15, 0.00, 0.5, 12.0),
    armor("bracers", "Bracers", Slot::Hands, ARMS, 0.35, 0.30, 0.20, 0.00, 0.6, 25.0),
    armor("gauntlets", "Gauntlets", Slot::Hands, ARMS, 0.45, 0.55, 0.35, 0.01, 1.5, 90.0),
    // --- Legs, under and over ---------------------------------------------------
    armor("mail_leggings", "Mail leggings", Slot::LegsUnder, LEGS, 0.85, 0.55, 0.15, 0.02, 5.0, 110.0),
    armor("hide_leggings", "Hide leggings", Slot::LegsOver, LEGS, 0.85, 0.30, 0.20, 0.02, 3.0, 40.0),
    armor("scale_greaves", "Scale greaves", Slot::LegsOver, LEGS, 0.80, 0.55, 0.30, 0.04, 6.0, 160.0),
    armor("plate_greaves", "Plate greaves", Slot::LegsOver, LEGS, 0.90, 0.70, 0.45, 0.06, 8.0, 220.0),
    // --- Feet -------------------------------------------------------------------
    armor("boots", "Boots", Slot::Feet, LEGS, 0.30, 0.20, 0.15, 0.00, 1.2, 15.0),
    armor("fishskin_boots", "Fishskin boots", Slot::Feet, LEGS, 0.30, 0.20, 0.15, 0.00, 1.0, 25.0),
    armor("fur_boots", "Fur-lined boots", Slot::Feet, LEGS, 0.30, 0.20, 0.15, 0.00, 1.6, 30.0),
    armor("armoured_boots", "Armoured boots", Slot::Feet, LEGS, 0.35, 0.50, 0.30, 0.02, 3.0, 90.0),
    // --- Shields ------------------------------------------------------------------
    ItemDef { key: "buckler", name: "Buckler", slot: Slot::OffHand, kind: Kind::Shield(0.35), weight: 2.5, value: 50.0, effects: &[] },
    ItemDef { key: "round_shield", name: "Round shield", slot: Slot::OffHand, kind: Kind::Shield(0.5), weight: 5.0, value: 120.0, effects: &[] },
    ItemDef { key: "tower_shield", name: "Tower shield", slot: Slot::OffHand, kind: Kind::Shield(0.7), weight: 9.0, value: 160.0, effects: &[] },
    // --- Enchanted pieces ------------------------------------------------
    ItemDef {
        key: "seers_hood",
        name: "Seer's hood",
        slot: Slot::Head,
        kind: Kind::Armor(ArmorDef { covers: HEAD, coverage: 0.6, cut: 0.15, blunt: 0.10, dodge_penalty: 0.0, conceal: 0.0 }),
        weight: 0.6,
        value: 240.0,
        effects: &[worn(Does::ResistBlind, 0.7), worn(Does::Skill(Skill::Structured), 8.0)],
    },
    ItemDef {
        key: "striders_boots",
        name: "Strider's boots",
        slot: Slot::Feet,
        kind: Kind::Armor(ArmorDef { covers: LEGS, coverage: 0.30, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0, conceal: 0.4 }),
        weight: 1.0,
        value: 260.0,
        effects: &[worn(Does::MoveSpeed, 0.20), worn(Does::Skill(Skill::Athletics), 10.0)],
    },
    ItemDef {
        key: "duelists_gloves",
        name: "Duelist's gloves",
        slot: Slot::Hands,
        kind: Kind::Armor(ArmorDef { covers: ARMS, coverage: 0.35, cut: 0.20, blunt: 0.15, dodge_penalty: 0.0, conceal: 0.4 }),
        weight: 0.4,
        value: 230.0,
        effects: &[worn(Does::Skill(Skill::Blade), 10.0), worn(Does::Skill(Skill::Block), 6.0)],
    },
];

pub(super) static FORMS: &[Form] = &[
    f("leather_cap", "cap", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("padded_cap", "padded cap", M::Cloth, &[M::Cloth, M::Seareed], &[]),
    f("mail_coif", "mail coif", M::PlainIron, &[M::PlainIron, M::Forgeiron], &[]),
    f("iron_helm", "open helm", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("full_helm", "full helm", M::Forgeiron, &[M::Forgeiron, M::Slatewing], &[]),
    f("padded_vest", "padded vest", M::Cloth, &[M::Cloth, M::Seareed], &[]),
    f("mail_shirt", "mail shirt", M::PlainIron, &[M::PlainIron, M::Forgeiron], &[]),
    f("fishskin_undercoat", "fishskin undercoat", M::Fishskin, &[M::Fishskin], &[]),
    f("fur_jerkin", "fur jerkin", M::Leather, &[M::Leather], &[]),
    f("padded_jacket", "padded jacket", M::Cloth, &[M::Cloth, M::Seareed], &[]),
    f("hide_coat", "jerkin", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("brigandine", "brigandine", M::Leather, &[M::Leather], &[M::Forgeiron, M::Bronze]),
    f("scale_hauberk", "scale coat", M::Forgeiron, &[M::Forgeiron, M::Bronze, M::Nacre, M::Slatewing], &[M::Seareed, M::Slatewing, M::Leather]),
    f("plate_cuirass", "plate cuirass", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("stone_cuirass", "stone cuirass", M::Ringstone, &[M::Ringstone], &[]),
    f("travelling_robe", "travelling robe", M::Tentsilk, &[M::Tentsilk, M::Cloth], &[]),
    f("leather_gloves", "gloves", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("bracers", "bracers", M::Leather, &[M::Leather, M::Bronze], &[]),
    f("gauntlets", "gauntlets", M::Forgeiron, &[M::Forgeiron, M::Bronze], &[]),
    f("mail_leggings", "mail leggings", M::PlainIron, &[M::PlainIron, M::Forgeiron], &[]),
    f("hide_leggings", "leggings", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("scale_greaves", "greaves", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing, M::Nacre], &[]),
    f("plate_greaves", "plate greaves", M::Forgeiron, &[M::Forgeiron, M::Bronze], &[]),
    f("boots", "boots", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("fishskin_boots", "fishskin boots", M::Fishskin, &[M::Fishskin], &[]),
    f("fur_boots", "fur-lined boots", M::Leather, &[M::Leather], &[]),
    f("armoured_boots", "armoured boots", M::Forgeiron, &[M::Forgeiron, M::Bronze], &[]),
    f("buckler", "buckler", M::Wood, &[M::Wood, M::Bronze, M::Nacre], &[]),
    f("round_shield", "round shield", M::Forgeiron, &[M::Wood, M::Forgeiron, M::Bronze, M::Ringstone], &[]),
    f("tower_shield", "tower shield", M::Wood, &[M::Wood, M::Forgeiron, M::Ringstone], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("buckler", 2, 2),
    ("round_shield", 3, 3),
    ("tower_shield", 3, 5),
    ("full_helm", 2, 2),
    ("stone_cuirass", 3, 3),
    ("plate_cuirass", 3, 3),
    ("plate_greaves", 2, 4),
    ("travelling_robe", 2, 4),
];

/// What wearing each form costs and gives beyond protection (Part 2's rules
/// 4 to 7): see `Wears`. Forms not listed cost and give nothing more.
pub(super) static WEARS: &[(&str, Wears)] = &[
    ("leather_cap", Wears { ko_guard: 0.15, ..NONE }),
    ("padded_cap", Wears { padding: 0.5, ko_guard: 0.3, warmth: 0.2, ..NONE }),
    ("mail_coif", Wears { ko_guard: 0.1, noise: 0.12, warmth: -0.1, ..NONE }),
    ("iron_helm", Wears { ko_guard: 0.6, noise: 0.05, ..NONE }),
    ("full_helm", Wears { ko_guard: 0.85, noise: 0.08, sight: 0.3, warmth: -0.1, ..NONE }),
    ("padded_vest", Wears { padding: 0.5, warmth: 0.3, soaks: true, ..NONE }),
    ("mail_shirt", Wears { noise: 0.25, warmth: -0.1, ..NONE }),
    ("fishskin_undercoat", Wears { dry: 0.8, ..NONE }),
    ("fur_jerkin", Wears { padding: 0.2, warmth: 0.8, ..NONE }),
    ("padded_jacket", Wears { padding: 0.6, warmth: 0.4, soaks: true, ..NONE }),
    ("hide_coat", Wears { padding: 0.15, warmth: 0.3, dry: 0.3, hides: 0.3, ..NONE }),
    ("brigandine", Wears { padding: 0.25, noise: 0.08, warmth: 0.2, hides: 0.2, ..NONE }),
    ("scale_hauberk", Wears { noise: 0.2, slow: 0.05, warmth: -0.1, ..NONE }),
    ("plate_cuirass", Wears { noise: 0.25, slow: 0.08, warmth: -0.2, ..NONE }),
    ("stone_cuirass", Wears { padding: 0.3, noise: 0.35, slow: 0.15, warmth: -0.2, ..NONE }),
    ("travelling_robe", Wears { warmth: 0.5, hides: 0.6, ..NONE }),
    ("leather_gloves", Wears { warmth: 0.2, ..NONE }),
    ("bracers", Wears { ..NONE }),
    ("gauntlets", Wears { noise: 0.08, warmth: -0.05, ..NONE }),
    ("mail_leggings", Wears { noise: 0.2, ..NONE }),
    ("hide_leggings", Wears { warmth: 0.2, dry: 0.2, ..NONE }),
    ("scale_greaves", Wears { noise: 0.12, slow: 0.03, ..NONE }),
    ("plate_greaves", Wears { noise: 0.2, slow: 0.08, warmth: -0.1, ..NONE }),
    ("boots", Wears { noise: 0.05, warmth: 0.2, ..NONE }),
    ("fishskin_boots", Wears { dry: 0.8, ..NONE }),
    ("fur_boots", Wears { warmth: 0.7, noise: 0.05, ..NONE }),
    ("armoured_boots", Wears { noise: 0.25, slow: 0.04, ..NONE }),
    ("tower_shield", Wears { slow: 0.08, ..NONE }),
];
