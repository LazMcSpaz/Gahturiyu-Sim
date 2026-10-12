//! Trinkets, jewellery, money and errands (Part 7 of `docs/items-design.md`).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    // --- Made for trade ---------------------------------------------------
    trinket("hearthclay_pot", "Hearthclay pot", Slot::MainHand, 1.2, 6.0, &[]),
    trinket("sandglass_flask", "Sandglass flask", Slot::MainHand, 0.3, 12.0, &[]),
    trinket("gold_ring", "Gold ring", Slot::Ring, 0.05, 90.0, &[]),
    trinket("pearl_necklace", "Pearl necklace", Slot::Neck, 0.1, 140.0, &[]),
    // --- Belts and brooches (accessories: show and enchanting) ---------------
    trinket("plain_belt", "Plain belt", Slot::Belt, 0.3, 8.0, &[]),
    trinket("bandolier_belt", "Bandolier belt", Slot::Belt, 0.5, 20.0, &[]),
    trinket("sash", "Sash", Slot::Belt, 0.2, 10.0, &[]),
    trinket("fine_belt", "Fine belt", Slot::Belt, 0.4, 120.0, &[]),
    trinket("bronze_brooch", "Bronze brooch", Slot::Brooch, 0.05, 15.0, &[]),
    // --- Money and errands ------------------------------------------------
    ItemDef { key: "coin", name: "Coin", slot: Slot::MainHand, kind: Kind::Coin, weight: 0.005, value: 1.0, effects: &[] },
    // Fifty coin on Ṭaḍoro paper: next to nothing to carry, but as easily stolen or lost.
    ItemDef { key: "note", name: "Note (50 coin)", slot: Slot::MainHand, kind: Kind::Coin, weight: 0.001, value: 50.0, effects: &[] },
    ItemDef { key: "sealed_letter", name: "Sealed letter", slot: Slot::MainHand, kind: Kind::Errand, weight: 0.02, value: 0.0, effects: &[] },
    // --- Enchanted pieces -------------------------------------------------
    trinket("ring_swiftness", "Ring of Swiftness", Slot::Ring, 0.1, 300.0, &[worn(Does::MoveSpeed, 0.15), worn(Does::Attr(Attr::Agility), 5.0)]),
    trinket("ring_might", "Ring of the Ox", Slot::Ring, 0.1, 320.0, &[worn(Does::Attr(Attr::Strength), 12.0)]),
    trinket("amulet_wellspring", "Wellspring Amulet", Slot::Neck, 0.2, 350.0, &[worn(Does::MaxEnergy, 30.0), worn(Does::EnergyRegen, 0.6)]),
    trinket("amulet_clear_mind", "Amulet of the Clear Mind", Slot::Neck, 0.2, 280.0, &[worn(Does::ResistParalysis, 0.6), worn(Does::Attr(Attr::Willpower), 6.0)]),
    trinket("ring_hearth", "Hearthstone Ring", Slot::Ring, 0.1, 260.0, &[worn(Does::ResistElements, 0.4)]),
    // Laz's starter set (items design, Part 7): small buffs, found or bought.
    trinket("ring_warding", "Ring of Warding", Slot::Ring, 0.05, 280.0, &[worn(Does::ResistElements, 0.15)]),
    trinket("steady_torc", "Steady Torc", Slot::Neck, 0.4, 220.0, &[worn(Does::Attr(Attr::Strength), 4.0)]),
    trinket("keen_brooch", "Keen-eye Brooch", Slot::Brooch, 0.05, 240.0, &[worn(Does::Skill(Skill::Sneak), 4.0)]),
    trinket("travellers_brooch", "Traveller's Brooch", Slot::Brooch, 0.05, 160.0, &[worn(Does::Skill(Skill::Athletics), 5.0)]),
    trinket("belt_of_bearing", "Fine Belt of Bearing", Slot::Belt, 0.4, 260.0, &[worn(Does::Carry, 10.0)]),
];

pub(super) static FORMS: &[Form] = &[
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
];
