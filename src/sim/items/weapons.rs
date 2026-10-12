//! Weapons and what they shoot (Part 1 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //      key            name             skill          cut   blunt reach windup rec   2h     parry wt   value
    // (`fits` lays the fight rules over a line: Part 1, B3 of the design.)
    fits(weapon("knife", "Knife", Skill::Blade, 6.0, 1.0, 0.9, 0.35, 0.35, false, 0.05, 0.5, 10.0), Fits { bleed: 0.30, space: Space::Short, sneak: 4.0, conceal: 0.8, offhand: true, ..FIT }),
    fits(weapon("short_sword", "Short sword", Skill::Blade, 11.0, 2.0, 1.1, 0.45, 0.45, false, 0.20, 2.5, 60.0), Fits { bleed: 0.40, space: Space::Short, conceal: 0.2, offhand: true, ..FIT }),
    fits(weapon("longsword", "Longsword", Skill::Blade, 15.0, 3.0, 1.3, 0.60, 0.60, false, 0.25, 4.0, 140.0), Fits { bleed: 0.50, space: Space::Medium, conceal: 0.05, ..FIT }),
    fits(weapon("club", "Club", Skill::Blunt, 0.0, 10.0, 1.0, 0.50, 0.50, false, 0.10, 2.0, 15.0), Fits { space: Space::Medium, conceal: 0.15, ..FIT }),
    fits(weapon("war_pick", "War-pick", Skill::Blunt, 8.0, 12.0, 1.1, 0.60, 0.60, false, 0.10, 4.0, 130.0), Fits { bleed: 0.20, space: Space::Medium, conceal: 0.1, ..FIT }),
    fits(weapon("stone_maul", "Stone maul", Skill::Blunt, 0.0, 22.0, 1.3, 1.00, 0.90, true, 0.10, 9.0, 120.0), Fits { space: Space::Sweep, sweep: 2, sneak: 1.5, ..FIT }),
    fits(weapon("spear", "Spear", Skill::Spear, 9.0, 3.0, 2.2, 0.55, 0.50, true, 0.20, 3.0, 50.0), Fits { bleed: 0.40, space: Space::Long, sneak: 1.5, ..FIT }),
    fits(weapon("glaive", "Glaive", Skill::Spear, 16.0, 4.0, 2.4, 0.80, 0.70, true, 0.25, 5.0, 180.0), Fits { bleed: 0.50, space: Space::Long, sweep: 2, sneak: 1.5, ..FIT }),
    //      key            name         cut   blunt range windup reload ammo      wt   value
    fits(ranged("short_bow", "Short bow", 12.0, 0.0, 30.0, 0.8, 0.5, "arrows", 1.5, 60.0), Fits { space: Space::Short, conceal: 0.0, ..FIT }),
    fits(ranged("crossbow", "Crossbow", 18.0, 4.0, 38.0, 0.4, 2.6, "bolts", 5.0, 160.0), Fits { space: Space::Short, conceal: 0.0, ..FIT }),
    fits(weapon("staff", "Staff", Skill::Blunt, 0.0, 7.0, 1.8, 0.50, 0.50, true, 0.30, 2.0, 20.0), Fits { space: Space::Long, sneak: 1.5, ..FIT }),
    fits(weapon("hatchet", "Hatchet", Skill::Blade, 8.0, 4.0, 0.9, 0.45, 0.45, false, 0.05, 1.2, 14.0), Fits { bleed: 0.30, space: Space::Short, conceal: 0.35, thrown: true, breaks: 0.08, ..FIT }),
    fits(weapon("trident", "Trident", Skill::Spear, 12.0, 2.0, 2.1, 0.55, 0.55, true, 0.20, 3.2, 90.0), Fits { bleed: 0.40, space: Space::Long, sneak: 1.5, ..FIT }),
    fits(ranged("sling", "Sling", 0.0, 9.0, 22.0, 0.6, 0.8, "sling_stones", 0.2, 8.0), Fits { space: Space::Short, conceal: 0.7, ..FIT }),
    ItemDef {
        key: "net",
        name: "Weighted net",
        slot: Slot::MainHand,
        // (Thrown, it tangles whoever it lands on; it seldom breaks.)
        kind: Kind::Weapon(WeaponDef { skill: Skill::Spear, cut: 0.0, blunt: 2.0, reach: 2.4, windup: 0.8, recover: 0.9, two_handed: true, entangle: 0.45, space: Space::Medium, thrown: true, breaks: 0.02, ..FISTS }),
        weight: 2.5,
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
    f("hatchet", "hatchet", M::PlainIron, &[M::PlainIron, M::Bronze, M::Forgeiron], &[]),
    f("short_sword", "short blade", M::Bronze, &[M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[M::Forgeiron]),
    f("longsword", "sword", M::Forgeiron, &[M::Bronze, M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("club", "club", M::Wood, &[M::Wood], &[]),
    f("war_pick", "war-pick", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("stone_maul", "maul", M::Ringstone, &[M::Ringstone, M::Forgeiron], &[]),
    f("spear", "spear", M::Wood, &[M::Wood, M::Bronze, M::Forgeiron, M::Edgeglass, M::Nacre], &[]),
    f("trident", "trident", M::Nacre, &[M::Nacre, M::Bronze, M::Forgeiron], &[]),
    f("glaive", "glaive", M::Forgeiron, &[M::Forgeiron, M::Edgeglass], &[M::Forgeiron]),
    f("net", "net", M::Seareed, &[M::Seareed], &[]),
    f("staff", "staff", M::Wood, &[M::Wood], &[]),
    f("short_bow", "bow", M::Wood, &[M::Wood], &[]),
    f("crossbow", "crossbow", M::Forgeiron, &[M::Bronze, M::Forgeiron], &[]),
    f("sling", "sling", M::Leather, &[M::Leather, M::Fishskin], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("knife", 1, 2),
    ("hatchet", 1, 2),
    ("short_sword", 1, 3),
    ("longsword", 1, 4),
    ("club", 1, 3),
    ("war_pick", 1, 3),
    ("stone_maul", 2, 4),
    ("spear", 1, 6),
    ("trident", 1, 5),
    ("glaive", 1, 6),
    ("net", 2, 2),
    ("staff", 1, 5),
    ("short_bow", 2, 4),
    ("crossbow", 3, 3),
    ("sling", 1, 1),
];
