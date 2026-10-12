//! Clothes: the under layers and the cloak (Part 3 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    //     key               name                slot        covers cover  cut   blunt dodge  wt    value
    armor("cloth_shirt", "Cloth shirt", Slot::BodyUnder, BODY, 0.90, 0.10, 0.10, 0.00, 1.0, 8.0),
    armor("wraps", "Silk wraps", Slot::BodyUnder, BODY, 0.90, 0.22, 0.18, 0.00, 0.6, 70.0),
    armor("trousers", "Trousers", Slot::LegsUnder, LEGS, 0.90, 0.10, 0.10, 0.00, 1.0, 6.0),
    // A cloak: warmth and cover more than protection.
    armor("cloak", "Cloak", Slot::Cloak, BODY, 0.55, 0.06, 0.06, 0.00, 1.5, 20.0),
];

pub(super) static FORMS: &[Form] = &[
    f("cloth_shirt", "shirt", M::Cloth, &[M::Cloth, M::Tentsilk], &[]),
    f("wraps", "wraps", M::Tentsilk, &[M::Tentsilk, M::Cloth], &[]),
    f("trousers", "trousers", M::Cloth, &[M::Cloth], &[]),
    f("cloak", "cloak", M::Cloth, &[M::Cloth, M::Leather], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
