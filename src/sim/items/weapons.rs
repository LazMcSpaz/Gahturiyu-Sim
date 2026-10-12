//! Weapons and what they shoot (Part 1 of `docs/items-design.md`): Laz's 28
//! forms, each with a reason to carry it. Numbers are the design's table B3
//! (usual material, Common grade); materials and grades multiply them
//! (`FORMS`). The fight rules read the `Fits` laid over each line.

use super::*;

/// A ranged weapon held in one hand (the hand crossbow).
const fn one_handed(mut d: ItemDef) -> ItemDef {
    if let Kind::Weapon(mut w) = d.kind {
        w.two_handed = false;
        d.kind = Kind::Weapon(w);
    }
    d
}

pub(super) static ITEMS: &[ItemDef] = &[
    //      key            name             skill          cut   blunt reach windup rec   2h     parry wt   value
    // --- Blades (Blade skill) ----------------------------------------------
    fits(weapon("knife", "Knife", Skill::Blade, 6.0, 1.0, 0.6, 0.30, 0.30, false, 0.05, 0.25, 10.0), Fits { bleed: 0.30, space: Space::Short, sneak: 4.0, conceal: 0.8, offhand: true, strength: 10.0, ..FIT }),
    fits(weapon("dirk", "Dirk", Skill::Blade, 7.0, 1.0, 0.75, 0.35, 0.35, false, 0.35, 0.5, 30.0), Fits { bleed: 0.30, space: Space::Short, sneak: 2.5, conceal: 0.75, offhand: true, strength: 15.0, ..FIT }),
    fits(weapon("short_sword", "Short sword", Skill::Blade, 10.0, 2.0, 0.9, 0.40, 0.40, false, 0.20, 1.0, 60.0), Fits { bleed: 0.40, space: Space::Short, conceal: 0.2, offhand: true, strength: 25.0, ..FIT }),
    fits(weapon("cutlass", "Cutlass", Skill::Blade, 12.0, 2.0, 0.95, 0.42, 0.42, false, 0.15, 1.2, 70.0), Fits { bleed: 0.70, space: Space::Medium, conceal: 0.1, strength: 30.0, ..FIT }),
    fits(weapon("longsword", "Longsword", Skill::Blade, 15.0, 3.0, 1.1, 0.55, 0.55, false, 0.30, 1.5, 140.0), Fits { bleed: 0.50, space: Space::Medium, conceal: 0.05, strength: 35.0, ..FIT }),
    fits(weapon("greatsword", "Greatsword", Skill::Blade, 22.0, 5.0, 1.5, 0.85, 0.75, true, 0.20, 3.0, 300.0), Fits { bleed: 0.60, space: Space::Sweep, sweep: 3, sneak: 1.5, strength: 55.0, ..FIT }),
    fits(weapon("hatchet", "Hatchet", Skill::Blade, 8.0, 4.0, 0.7, 0.45, 0.45, false, 0.05, 0.8, 14.0), Fits { bleed: 0.30, space: Space::Short, conceal: 0.35, thrown: true, breaks: 0.08, strength: 20.0, ..FIT }),
    fits(weapon("battle_axe", "Battle axe", Skill::Blade, 24.0, 6.0, 1.3, 0.85, 0.80, true, 0.05, 2.5, 220.0), Fits { bleed: 0.80, space: Space::Sweep, sweep: 2, sneak: 1.5, strength: 50.0, ..FIT }),
    // --- Blunt (Blunt skill: the knockout family) --------------------------
    fits(weapon("sap", "Sap", Skill::Blunt, 0.0, 5.0, 0.5, 0.35, 0.35, false, 0.0, 0.4, 12.0), Fits { space: Space::Short, sneak: 2.0, stuns: true, conceal: 0.85, strength: 10.0, ..FIT }),
    fits(weapon("club", "Club", Skill::Blunt, 0.0, 10.0, 0.9, 0.50, 0.50, false, 0.10, 1.2, 15.0), Fits { space: Space::Medium, conceal: 0.15, strength: 25.0, ..FIT }),
    fits(weapon("mace", "Mace", Skill::Blunt, 0.0, 13.0, 0.9, 0.55, 0.55, false, 0.10, 1.6, 90.0), Fits { space: Space::Medium, conceal: 0.1, strength: 35.0, ..FIT }),
    fits(weapon("war_pick", "War-pick", Skill::Blunt, 8.0, 11.0, 1.0, 0.60, 0.60, false, 0.10, 1.8, 130.0), Fits { bleed: 0.20, space: Space::Medium, conceal: 0.1, strength: 40.0, ..FIT }),
    fits(weapon("stone_maul", "Stone maul", Skill::Blunt, 0.0, 24.0, 1.3, 1.00, 0.90, true, 0.10, 5.0, 120.0), Fits { space: Space::Sweep, sweep: 2, sneak: 1.5, strength: 65.0, ..FIT }),
    fits(weapon("staff", "Staff", Skill::Blunt, 0.0, 7.0, 1.8, 0.45, 0.45, true, 0.35, 1.8, 20.0), Fits { space: Space::Long, sneak: 1.5, strength: 20.0, ..FIT }),
    // Worn over the fists: bare-handed blows, harder (4 + 5 blunt).
    fits(weapon("knuckle_guards", "Knuckle guards", Skill::Unarmed, 0.0, 9.0, 0.6, 0.35, 0.35, false, 0.0, 0.3, 15.0), Fits { space: Space::Short, conceal: 0.9, strength: 10.0, ..FIT }),
    // --- Spears (Spear skill) ----------------------------------------------
    fits(weapon("spear", "Spear", Skill::Spear, 10.0, 3.0, 2.2, 0.55, 0.50, true, 0.20, 2.2, 50.0), Fits { bleed: 0.40, space: Space::Long, sneak: 1.5, strength: 30.0, ..FIT }),
    fits(weapon("short_spear", "Short spear", Skill::Spear, 9.0, 2.0, 1.6, 0.45, 0.45, false, 0.15, 1.4, 40.0), Fits { bleed: 0.40, space: Space::Medium, thrown: true, breaks: 0.05, strength: 25.0, ..FIT }),
    fits(weapon("trident", "Trident", Skill::Spear, 11.0, 2.0, 2.0, 0.55, 0.55, true, 0.30, 2.5, 90.0), Fits { bleed: 0.40, space: Space::Long, sneak: 1.5, strength: 35.0, ..FIT }),
    fits(weapon("glaive", "Glaive", Skill::Spear, 16.0, 4.0, 2.4, 0.80, 0.70, true, 0.25, 3.0, 180.0), Fits { bleed: 0.50, space: Space::Long, sweep: 2, sneak: 1.5, strength: 50.0, ..FIT }),
    // --- Ranged and thrown (Marksman skill) --------------------------------
    //      key            name         cut   blunt range windup reload ammo      wt   value
    fits(ranged("short_bow", "Short bow", 12.0, 0.0, 30.0, 0.8, 0.5, "arrows", 0.8, 60.0), Fits { space: Space::Short, strength: 25.0, ..FIT }),
    fits(ranged("longbow", "Longbow", 18.0, 0.0, 45.0, 1.4, 0.5, "arrows", 1.2, 110.0), Fits { space: Space::Short, strength: 55.0, ..FIT }),
    fits(ranged("crossbow", "Crossbow", 18.0, 4.0, 38.0, 0.4, 2.6, "bolts", 4.0, 160.0), Fits { space: Space::Short, strength: 20.0, ..FIT }),
    one_handed(fits(ranged("hand_crossbow", "Hand crossbow", 10.0, 1.0, 15.0, 0.3, 3.5, "bolts", 1.2, 140.0), Fits { space: Space::Short, conceal: 0.6, strength: 10.0, ..FIT })),
    fits(ranged("sling", "Sling", 0.0, 9.0, 22.0, 0.6, 0.8, "sling_stones", 0.1, 8.0), Fits { space: Space::Short, conceal: 0.7, strength: 15.0, ..FIT }),
    // Thrown, a few at a time; each flies, lands, and may break (`thrown.rs`).
    fits(weapon("throwing_knives", "Throwing knives", Skill::Marksman, 6.0, 0.0, 0.6, 0.40, 0.40, false, 0.0, 0.15, 8.0), Fits { bleed: 0.30, space: Space::Short, conceal: 0.8, thrown: true, breaks: 0.12, strength: 10.0, ..FIT }),
    fits(weapon("javelin", "Javelin", Skill::Marksman, 14.0, 2.0, 1.4, 0.70, 0.60, false, 0.05, 1.0, 15.0), Fits { bleed: 0.40, space: Space::Medium, thrown: true, breaks: 0.10, strength: 35.0, ..FIT }),
    ItemDef {
        key: "net",
        name: "Weighted net",
        slot: Slot::MainHand,
        // (Thrown, it tangles whoever it lands on; it seldom breaks.)
        kind: Kind::Weapon(WeaponDef { skill: Skill::Spear, cut: 0.0, blunt: 2.0, reach: 2.4, windup: 0.8, recover: 1.0, two_handed: true, entangle: 0.45, space: Space::Medium, thrown: true, breaks: 0.02, strength: 20.0, ..FISTS }),
        weight: 2.0,
        value: 35.0,
        effects: &[],
    },
    // Shot from bows and slings.
    ItemDef { key: "arrows", name: "Arrows", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.04, value: 1.0, effects: &[] },
    ItemDef { key: "bolts", name: "Crossbow bolts", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.06, value: 2.0, effects: &[] },
    ItemDef { key: "sling_stones", name: "Sling stones", slot: Slot::MainHand, kind: Kind::Ammo, weight: 0.05, value: 0.2, effects: &[] },
];

pub(super) static FORMS: &[Form] = &[
    f("knife", "knife", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[]),
    f("dirk", "dirk", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron, M::Edgeglass], &[]),
    f("hatchet", "hatchet", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron], &[]),
    f("short_sword", "short blade", M::Bronze, &[M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[M::Forgeiron]),
    f("cutlass", "cutlass", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron, M::Nacre], &[]),
    f("longsword", "sword", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("greatsword", "greatsword", M::Forgeiron, &[M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("battle_axe", "battle axe", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("sap", "sap", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("club", "club", M::Wood, &[M::Wood], &[]),
    f("mace", "mace", M::Bronze, &[M::Bronze, M::Forgeiron, M::Ringstone], &[]),
    f("war_pick", "war-pick", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("stone_maul", "maul", M::Ringstone, &[M::Ringstone, M::Forgeiron], &[]),
    f("knuckle_guards", "knuckle guards", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron], &[]),
    f("spear", "spear", M::Wood, &[M::Wood, M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[]),
    f("short_spear", "short spear", M::Bronze, &[M::Bronze, M::Forgeiron, M::Nacre], &[]),
    f("trident", "trident", M::Nacre, &[M::Nacre, M::Bronze, M::Forgeiron], &[]),
    f("glaive", "glaive", M::Forgeiron, &[M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("net", "net", M::Seareed, &[M::Seareed], &[]),
    f("staff", "staff", M::Wood, &[M::Wood], &[]),
    f("short_bow", "bow", M::Wood, &[M::Wood], &[]),
    f("longbow", "longbow", M::Wood, &[M::Wood], &[]),
    f("crossbow", "crossbow", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("hand_crossbow", "hand crossbow", M::Bronze, &[M::Bronze, M::Forgeiron], &[]),
    f("sling", "sling", M::Leather, &[M::Leather, M::Fishskin], &[]),
    f("throwing_knives", "throwing knives", M::PlainIron, &[M::PlainIron, M::Forgeiron, M::Edgeglass], &[]),
    f("javelin", "javelin", M::Wood, &[M::Wood, M::Bronze, M::Forgeiron], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("knife", 1, 2),
    ("dirk", 1, 2),
    ("hatchet", 1, 2),
    ("short_sword", 1, 3),
    ("cutlass", 1, 3),
    ("longsword", 1, 4),
    ("greatsword", 1, 5),
    ("battle_axe", 2, 4),
    ("sap", 1, 1),
    ("club", 1, 3),
    ("mace", 1, 3),
    ("war_pick", 1, 3),
    ("stone_maul", 2, 4),
    ("knuckle_guards", 1, 1),
    ("spear", 1, 6),
    ("short_spear", 1, 4),
    ("trident", 1, 5),
    ("glaive", 1, 6),
    ("net", 2, 2),
    ("staff", 1, 5),
    ("short_bow", 2, 4),
    ("longbow", 1, 6),
    ("crossbow", 3, 3),
    ("hand_crossbow", 2, 2),
    ("sling", 1, 1),
    ("throwing_knives", 1, 2),
    ("javelin", 1, 5),
];
