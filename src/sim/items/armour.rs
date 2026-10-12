//! Armour and shields (Part 2 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    armor("padded_jacket", "Padded jacket", Slot::BodyOver, BODY, 0.90, 0.25, 0.30, 0.02, 4.0, 40.0),
    armor("hide_coat", "Hide coat", Slot::BodyOver, BODY, 0.85, 0.35, 0.25, 0.04, 6.0, 70.0),
    armor("scale_hauberk", "Scale hauberk", Slot::BodyOver, BODY, 0.85, 0.60, 0.30, 0.10, 14.0, 260.0),
    armor("plate_cuirass", "Plate cuirass", Slot::BodyOver, TORSO, 0.95, 0.55, 0.40, 0.08, 12.0, 240.0),
    armor("leather_cap", "Leather cap", Slot::Head, HEAD, 0.70, 0.25, 0.20, 0.00, 0.8, 15.0),
    armor("iron_helm", "Iron helm", Slot::Head, HEAD, 0.85, 0.60, 0.40, 0.03, 3.0, 120.0),
    armor("hide_leggings", "Hide leggings", Slot::LegsOver, LEGS, 0.85, 0.30, 0.20, 0.02, 3.0, 40.0),
    armor("scale_greaves", "Scale greaves", Slot::LegsOver, LEGS, 0.80, 0.55, 0.30, 0.06, 7.0, 160.0),
    armor("leather_gloves", "Leather gloves", Slot::Hands, ARMS, 0.35, 0.20, 0.15, 0.00, 0.5, 12.0),
    armor("boots", "Boots", Slot::Feet, LEGS, 0.30, 0.20, 0.15, 0.00, 1.2, 15.0),
    // --- Shields -----------------------------------------------------------
    ItemDef { key: "buckler", name: "Buckler", slot: Slot::OffHand, kind: Kind::Shield(0.35), weight: 2.5, value: 50.0, effects: &[] },
    ItemDef { key: "round_shield", name: "Round shield", slot: Slot::OffHand, kind: Kind::Shield(0.5), weight: 5.0, value: 120.0, effects: &[] },
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
    f("padded_jacket", "padded jacket", M::Cloth, &[M::Cloth, M::Seareed], &[]),
    f("hide_coat", "jerkin", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("scale_hauberk", "scale coat", M::Forgeiron, &[M::Forgeiron, M::Bronze, M::Nacre, M::Slatewing], &[M::Seareed, M::Slatewing, M::Leather]),
    f("plate_cuirass", "plate cuirass", M::Bronze, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("leather_cap", "cap", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("iron_helm", "helm", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing], &[]),
    f("hide_leggings", "leggings", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("scale_greaves", "greaves", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Slatewing, M::Nacre], &[]),
    f("leather_gloves", "bracers", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("boots", "boots", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("buckler", "buckler", M::Wood, &[M::Wood, M::Bronze, M::Nacre], &[]),
    f("round_shield", "round shield", M::Forgeiron, &[M::Wood, M::Forgeiron, M::Bronze, M::Ringstone], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("buckler", 2, 2),
    ("round_shield", 3, 3),
];
