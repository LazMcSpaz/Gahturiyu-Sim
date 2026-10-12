//! Clothes: the under layers and the cloak (Part 3 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    armor("cloth_shirt", "Cloth shirt", Slot::BodyUnder, BODY, 0.90, 0.10, 0.10, 0.00, 0.4, 8.0),
    armor("wraps", "Silk wraps", Slot::BodyUnder, BODY, 0.90, 0.22, 0.18, 0.00, 0.6, 70.0),
    armor("trousers", "Trousers", Slot::LegsUnder, LEGS, 0.90, 0.10, 0.10, 0.00, 0.6, 6.0),
    armor("hood", "Hood", Slot::Head, HEAD, 0.60, 0.05, 0.05, 0.00, 0.4, 6.0),
    armor("soft_shoes", "Soft shoes", Slot::Feet, LEGS, 0.25, 0.05, 0.05, 0.00, 0.4, 8.0),
    // The cloak slot: warmth, weather and cover more than protection.
    armor("cloak", "Cloak", Slot::Cloak, BODY, 0.55, 0.06, 0.06, 0.00, 1.5, 20.0),
    armor("oilskin_cape", "Oilskin cape", Slot::Cloak, BODY, 0.55, 0.05, 0.05, 0.00, 1.8, 30.0),
    armor("fur_mantle", "Fur mantle", Slot::Cloak, BODY, 0.60, 0.10, 0.10, 0.02, 4.0, 60.0),
];

pub(super) static FORMS: &[Form] = &[
    f("cloth_shirt", "shirt", M::Cloth, &[M::Cloth, M::Tentsilk], &[]),
    f("wraps", "wraps", M::Tentsilk, &[M::Tentsilk, M::Cloth], &[]),
    f("trousers", "trousers", M::Cloth, &[M::Cloth], &[]),
    f("hood", "hood", M::Cloth, &[M::Cloth, M::Fishskin], &[]),
    f("soft_shoes", "soft shoes", M::Cloth, &[M::Cloth, M::Leather], &[]),
    f("cloak", "cloak", M::Cloth, &[M::Cloth, M::Leather], &[]),
    f("oilskin_cape", "oilskin cape", M::Seareed, &[M::Seareed], &[]),
    f("fur_mantle", "fur mantle", M::Leather, &[M::Leather], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("hood", 2, 2),
    ("soft_shoes", 2, 1),
];

/// What wearing each costs and gives beyond protection (`Wears`).
pub(super) static WEARS: &[(&str, Wears)] = &[
    ("cloth_shirt", Wears { warmth: 0.15, soaks: true, ..NONE }),
    ("wraps", Wears { warmth: 0.1, ..NONE }),
    ("trousers", Wears { warmth: 0.15, soaks: true, ..NONE }),
    // Shades the face: harder to know again.
    ("hood", Wears { warmth: 0.3, dry: 0.3, hides: 0.2, ..NONE }),
    // The quietest feet; slower on rough ground.
    ("soft_shoes", Wears { noise: -0.1, slow: 0.03, ..NONE }),
    // A big bonus for a medium weapon and the armour beneath.
    ("cloak", Wears { warmth: 0.4, dry: 0.3, hides: 0.5, ..NONE }),
    ("oilskin_cape", Wears { warmth: 0.2, dry: 0.9, hides: 0.4, ..NONE }),
    ("fur_mantle", Wears { warmth: 1.0, hides: 0.3, ..NONE }),
];
