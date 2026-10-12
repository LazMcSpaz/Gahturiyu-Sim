//! Packs and tools (Part 6 of `docs/items-design.md`; packs, Part 2).
//!
//! What each container is like (0.4: "Packs and containers") is a line in
//! `CONTAINERS`, by key: how bulky it is when full, how much it slows, how
//! well it keeps water and fire out, what it holds, and for a belt how many
//! quick slots it gives.

use super::*;

/// How well something keeps water (or fire) out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proof {
    None,
    /// Cuts the chance and the amount.
    Resists,
    /// Nothing gets through until it's damaged or its seal wears off.
    Sealed,
}

/// What a container's grid takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holds {
    Anything,
    /// Arrows, bolts and sling stones (a quiver).
    Ammo,
    /// Quick slots on a belt: draughts, scrolls, bandages and throwing
    /// weapons, quicker to use in a fight than from a pack.
    Quick,
}

/// What a worn or carried container is like (0.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Container {
    /// The sneak and dodge it costs when full (it grows with how full).
    pub bulk: f32,
    /// How much it slows its carrier when full, travelling and fighting.
    pub slow: f32,
    pub water: Proof,
    pub fire: Proof,
    pub holds: Holds,
    /// Quick slots (a belt).
    pub quick: u8,
    /// Carried in both hands (a duffel): nothing else can be held, and it's
    /// put down when a fight starts.
    pub hands: bool,
}

const C: Container = Container { bulk: 0.0, slow: 0.0, water: Proof::None, fire: Proof::None, holds: Holds::Anything, quick: 0, hands: false };

/// Every container and belt, by form key (Part 2, "Worn and carried").
pub(super) static CONTAINERS: &[(&str, Container)] = &[
    ("satchel", Container { bulk: 0.08, slow: 0.0, water: Proof::Resists, ..C }),
    ("small_pack", Container { bulk: 0.2, slow: 0.03, ..C }),
    ("large_pack", Container { bulk: 0.45, slow: 0.08, ..C }),
    ("sea_bag", Container { bulk: 0.25, slow: 0.04, water: Proof::Sealed, ..C }),
    ("duffel", Container { bulk: 1.0, slow: 0.2, hands: true, ..C }),
    ("porters_belt_pack", Container { bulk: 0.9, slow: 0.25, ..C }),
    ("quiver", Container { bulk: 0.05, water: Proof::Resists, holds: Holds::Ammo, ..C }),
    ("plain_belt", Container { quick: 2, holds: Holds::Quick, ..C }),
    ("bandolier", Container { bulk: 0.03, quick: 4, holds: Holds::Quick, ..C }),
    ("sash", Container { quick: 1, holds: Holds::Quick, ..C }),
];

/// What a container (or belt) is like, if it is one.
pub fn container(id: ItemId) -> Option<Container> {
    let key = item(info(id).form).key;
    CONTAINERS.iter().find(|c| c.0 == key).map(|c| c.1)
}

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
    ItemDef { key: "satchel", name: "Satchel", slot: Slot::Back, kind: Kind::Pack(8.0), weight: 0.8, value: 25.0, effects: &[] },
    ItemDef { key: "sea_bag", name: "Sea bag", slot: Slot::Back, kind: Kind::Pack(20.0), weight: 2.0, value: 70.0, effects: &[] },
    // Carried in both hands.
    ItemDef { key: "duffel", name: "Duffel", slot: Slot::MainHand, kind: Kind::Pack(70.0), weight: 1.5, value: 20.0, effects: &[] },
    ItemDef { key: "quiver", name: "Quiver", slot: Slot::Belt, kind: Kind::Pack(3.0), weight: 0.4, value: 15.0, effects: &[] },
    // --- Belts (quick slots) ------------------------------------------------
    ItemDef { key: "bandolier", name: "Bandolier belt", slot: Slot::Belt, kind: Kind::Trinket, weight: 0.5, value: 20.0, effects: &[] },
    ItemDef { key: "sash", name: "Sash", slot: Slot::Belt, kind: Kind::Trinket, weight: 0.2, value: 6.0, effects: &[] },
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
    ("satchel", 2, 2),
    ("sea_bag", 2, 3),
    ("duffel", 3, 3),
    ("quiver", 1, 3),
    ("bandolier", 2, 1),
    ("sash", 2, 1),
];
