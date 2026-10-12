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
];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
