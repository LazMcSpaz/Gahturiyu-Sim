//! Packs and tools (Part 6 of `docs/items-design.md`; packs, Part 2).

use super::*;

pub(super) static ITEMS: &[ItemDef] = &[
    // --- Packs -------------------------------------------------------------
    ItemDef { key: "small_pack", name: "Small pack", slot: Slot::Back, kind: Kind::Pack(20.0), weight: 1.5, value: 30.0, effects: &[] },
    ItemDef { key: "large_pack", name: "Large pack", slot: Slot::Back, kind: Kind::Pack(45.0), weight: 3.0, value: 90.0, effects: &[] },
    ItemDef {
        key: "porters_belt_pack",
        name: "Porter's frame pack",
        slot: Slot::Back,
        kind: Kind::Pack(60.0),
        weight: 4.0,
        value: 200.0,
        effects: &[worn(Does::Carry, 20.0)],
    },
    // --- Tools ----------------------------------------------------------------
    ItemDef { key: "lockpick", name: "Lockpick", slot: Slot::MainHand, kind: Kind::Tool, weight: 0.05, value: 8.0, effects: &[] },
    ItemDef { key: "torch", name: "Torch", slot: Slot::OffHand, kind: Kind::Torch(crate::sim::torch::TORCH_HOURS), weight: 0.6, value: 4.0, effects: &[] },
    ItemDef { key: "standing_torch", name: "Standing torch", slot: Slot::MainHand, kind: Kind::StandingTorch(crate::sim::torch::STANDING_HOURS), weight: 2.0, value: 10.0, effects: &[] },
    ItemDef { key: "tent", name: "Tent", slot: Slot::MainHand, kind: Kind::Tool, weight: 6.0, value: 60.0, effects: &[] },
    ItemDef { key: "bedroll", name: "Bedroll", slot: Slot::MainHand, kind: Kind::Tool, weight: 2.0, value: 10.0, effects: &[] },
    ItemDef { key: "mortar_and_pestle", name: "Mortar and pestle", slot: Slot::MainHand, kind: Kind::Tool, weight: 1.5, value: 25.0, effects: &[] },
    // Binds a cut: stops bleeding (combat finds it by what it does).
    ItemDef { key: "bandage", name: "Bandage", slot: Slot::MainHand, kind: Kind::Tool, weight: 0.05, value: 2.0, effects: &[now(Does::Staunch, 1.0, Reach::Target)] },
];

pub(super) static FORMS: &[Form] = &[
    f("small_pack", "small pack", M::Leather, &[M::Leather, M::Seareed], &[]),
    f("large_pack", "large pack", M::Leather, &[M::Leather, M::Seareed, M::Tentsilk], &[]),
];

/// Grid cells (wide, tall) by form key, where the kind's default won't do.
pub(super) static SIZES: &[(&str, u8, u8)] = &[
    ("small_pack", 2, 2),
    ("large_pack", 3, 3),
    ("porters_belt_pack", 3, 3),
    ("tent", 2, 3),
    ("bedroll", 1, 3),
    ("standing_torch", 1, 3),
    ("torch", 1, 2),
    ("mortar_and_pestle", 1, 1),
    ("bandage", 1, 1),
];
