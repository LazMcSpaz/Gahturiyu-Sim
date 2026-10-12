//! Clothes: the under layers and the cloak (Part 3 of `docs/items-design.md`).

use super::*;
use crate::sim::race::Race;

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
    // Each people's own cut (Part 3): anyone may wear any style.
    armor("banded_tunic", "Banded tunic", Slot::BodyUnder, BODY, 0.90, 0.12, 0.12, 0.00, 0.9, 14.0),
    armor("sash_robe", "Sash robe", Slot::BodyUnder, BODY, 0.90, 0.08, 0.08, 0.00, 0.6, 16.0),
    armor("sea_wrap", "Sea wrap", Slot::BodyUnder, BODY, 0.85, 0.10, 0.08, 0.00, 0.5, 14.0),
    armor("wool_overcoat", "Wool overcoat", Slot::Cloak, BODY, 0.60, 0.08, 0.08, 0.00, 2.2, 30.0),
    armor("wool_cap", "Wool cap", Slot::Head, HEAD, 0.60, 0.04, 0.08, 0.00, 0.2, 5.0),
    armor("rain_hat", "Brimmed rain hat", Slot::Head, HEAD, 0.60, 0.04, 0.04, 0.00, 0.3, 6.0),
    // Role garments (disguise): worn outermost.
    armor("guard_tabard", "Guard's tabard", Slot::Cloak, TORSO, 0.50, 0.04, 0.04, 0.00, 0.8, 25.0),
    armor("vestment", "Vestment", Slot::Cloak, BODY, 0.60, 0.04, 0.04, 0.00, 1.2, 40.0),
    armor("merchants_coat", "Merchant's coat", Slot::Cloak, BODY, 0.60, 0.06, 0.06, 0.00, 1.8, 45.0),
    armor("labourers_smock", "Labourer's smock", Slot::Cloak, BODY, 0.55, 0.04, 0.04, 0.00, 0.8, 8.0),
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
    f("banded_tunic", "banded tunic", M::Cloth, &[M::Cloth], &[]),
    f("sash_robe", "sash robe", M::Cloth, &[M::Cloth, M::Tentsilk], &[]),
    f("sea_wrap", "sea wrap", M::Fishskin, &[M::Fishskin, M::Seareed], &[]),
    f("wool_overcoat", "wool overcoat", M::Cloth, &[M::Cloth], &[]),
    f("wool_cap", "wool cap", M::Cloth, &[M::Cloth], &[]),
    f("rain_hat", "rain hat", M::Seareed, &[M::Seareed, M::Fishskin], &[]),
    f("guard_tabard", "tabard", M::Cloth, &[M::Cloth], &[]),
    f("vestment", "vestment", M::Cloth, &[M::Cloth, M::Tentsilk], &[]),
    f("merchants_coat", "merchant's coat", M::Cloth, &[M::Cloth, M::Leather], &[]),
    f("labourers_smock", "smock", M::Cloth, &[M::Cloth], &[]),
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
    ("cloak", Wears { warmth: 0.4, dry: 0.3, hides: 0.5, wind: 0.4, ..NONE }),
    ("oilskin_cape", Wears { warmth: 0.2, dry: 0.9, hides: 0.4, wind: 0.5, ..NONE }),
    ("fur_mantle", Wears { warmth: 1.0, hides: 0.3, wind: 0.7, ..NONE }),
    ("banded_tunic", Wears { warmth: 0.6, dry: 0.4, ..NONE }),
    // The desert cut, out of its element here.
    ("sash_robe", Wears { warmth: -0.1, ..NONE }),
    ("sea_wrap", Wears { warmth: 0.1, dry: 0.8, ..NONE }),
    ("wool_overcoat", Wears { warmth: 0.6, dry: 0.3, wind: 0.6, hides: 0.4, ..NONE }),
    ("wool_cap", Wears { warmth: 0.5, ..NONE }),
    ("rain_hat", Wears { dry: 0.7, ..NONE }),
    ("vestment", Wears { warmth: 0.2, hides: 0.3, ..NONE }),
    ("merchants_coat", Wears { warmth: 0.4, dry: 0.3, hides: 0.4, wind: 0.3, ..NONE }),
    ("labourers_smock", Wears { hides: 0.2, ..NONE }),
];

/// How each reads when worn (Part 3): a people's cut, a role, or plain.
pub(super) static LOOKS: &[(&str, Look)] = &[
    ("banded_tunic", Look { people: Some(Race::Roduro), role: None }),
    ("wool_cap", Look { people: Some(Race::Roduro), role: None }),
    ("sash_robe", Look { people: Some(Race::Qotiro), role: None }),
    ("wool_overcoat", Look { people: Some(Race::Qotiro), role: None }),
    ("sea_wrap", Look { people: Some(Race::Horaro), role: None }),
    ("rain_hat", Look { people: Some(Race::Horaro), role: None }),
    ("oilskin_cape", Look { people: Some(Race::Horaro), role: None }),
    ("travelling_robe", Look { people: Some(Race::Tadoro), role: None }),
    ("guard_tabard", Look { people: None, role: Some(Role::Guard) }),
    ("vestment", Look { people: None, role: Some(Role::Priest) }),
    ("merchants_coat", Look { people: None, role: Some(Role::Merchant) }),
    ("labourers_smock", Look { people: None, role: Some(Role::Labourer) }),
];
