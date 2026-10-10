//! Building variants: what each building is like inside (playable-MVP step 6).
//!
//! A building's variant is worked out from the building itself (its kind,
//! size and seed), so nothing about it is stored and it never changes. Each
//! variant is one line of data: its outline (round grown stone or a quarried
//! box), how big it is against the plot the town gave it, where its door is,
//! the inner walls that split it into rooms (each with a doorway gap), the
//! furniture, and the spots where containers stand (`containers.rs`).
//!
//! **Coordinates.** Layouts are written in the building's own frame, scaled
//! to its outline: `x` runs from the back wall (-1) to the front wall (+1),
//! where the door is; `y` runs across, -1 to +1. A round building's wall is
//! the circle `x² + y² = 1` in these units. `Door::to_world` turns a layout
//! point into a place on the map.
//!
//! Furniture is placeholder blocks, drawn by the window; only walls and
//! containers matter to the sim (walls for walking, containers for what's
//! in them). Horaro stilt homes have variants for their look, but no door
//! yet (the sea is off-limits to the squad).

use super::rng::Rng;
use super::settlement::{Building, BuildingKind};

/// A building's outline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Grown stone: an ellipse.
    Round,
    /// Quarried: a box.
    Rect,
}

/// What a building is mostly for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    Home,
    /// A craft is worked here (with living space, often).
    Workshop,
    /// Goods are sold over a counter.
    Shop,
    /// Many households under one roof.
    Quarters,
    /// People eat or gather here.
    Hall,
    Shrine,
}

/// A piece of furniture (drawn as blocks; it changes nothing).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Furn {
    Bed,
    Bedroll,
    Table,
    LongTable,
    Bench,
    Shelf,
    Hearth,
    Workbench,
    Anvil,
    Forge,
    Kiln,
    Loom,
    Counter,
    Rack,
    GrowerBed,
    Altar,
    /// A hanging cloth or reed screen (a soft partition).
    Screen,
}

impl Furn {
    /// Width (along its own x), depth and height, metres.
    pub fn size(self) -> (f32, f32, f32) {
        match self {
            Furn::Bed => (2.0, 1.0, 0.5),
            Furn::Bedroll => (1.9, 0.8, 0.15),
            Furn::Table => (1.4, 0.9, 0.8),
            Furn::LongTable => (3.6, 0.9, 0.8),
            Furn::Bench => (1.8, 0.4, 0.45),
            Furn::Shelf => (1.6, 0.4, 1.8),
            Furn::Hearth => (1.0, 1.0, 0.4),
            Furn::Workbench => (2.0, 0.8, 0.9),
            Furn::Anvil => (0.7, 0.4, 0.7),
            Furn::Forge => (1.4, 1.2, 1.1),
            Furn::Kiln => (1.3, 1.3, 1.5),
            Furn::Loom => (1.6, 0.9, 1.7),
            Furn::Counter => (2.4, 0.7, 1.0),
            Furn::Rack => (1.8, 0.35, 1.5),
            Furn::GrowerBed => (1.8, 1.1, 0.35),
            Furn::Altar => (1.6, 0.9, 1.1),
            Furn::Screen => (1.6, 0.1, 1.7),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Furn::Bed => "bed",
            Furn::Bedroll => "bedroll",
            Furn::Table => "table",
            Furn::LongTable => "long table",
            Furn::Bench => "bench",
            Furn::Shelf => "shelves",
            Furn::Hearth => "hearth",
            Furn::Workbench => "workbench",
            Furn::Anvil => "anvil",
            Furn::Forge => "forge",
            Furn::Kiln => "kiln",
            Furn::Loom => "loom",
            Furn::Counter => "counter",
            Furn::Rack => "rack",
            Furn::GrowerBed => "grower's bed",
            Furn::Altar => "altar",
            Furn::Screen => "screen",
        }
    }
}

/// A kind of container.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Holder {
    /// Valuables; usually locked.
    Chest,
    /// Materials and tools of a trade.
    Crate,
    /// Clothes, small things; sometimes locked.
    Cupboard,
    /// Food and drink.
    Barrel,
}

impl Holder {
    /// Width, depth and height, metres.
    pub fn size(self) -> (f32, f32, f32) {
        match self {
            Holder::Chest => (1.1, 0.6, 0.6),
            Holder::Crate => (0.8, 0.8, 0.7),
            Holder::Cupboard => (1.2, 0.5, 1.7),
            Holder::Barrel => (0.7, 0.7, 0.95),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Holder::Chest => "chest",
            Holder::Crate => "crate",
            Holder::Cupboard => "cupboard",
            Holder::Barrel => "barrel",
        }
    }
}

/// An inner wall from `a` to `b` (layout units), with a doorway gap centred
/// `gap` of the way along (None: solid, or open at an end).
#[derive(Clone, Copy, Debug)]
pub struct Wall {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub gap: Option<f32>,
}

/// Width of a doorway in an inner wall, metres.
pub const DOORWAY: f32 = 1.2;

/// A piece of furniture at a spot, turned `rot` radians from the building's facing.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub what: Furn,
    pub at: (f32, f32),
    pub rot: f32,
}

/// Where a container stands.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub what: Holder,
    pub at: (f32, f32),
    pub rot: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub key: &'static str,
    pub name: &'static str,
    pub kind: BuildingKind,
    pub shape: Shape,
    pub use_: Use,
    /// Half its depth (front to back) and half its width, as shares of the
    /// building's size.
    pub half: (f32, f32),
    /// The building sizes (metres) it's chosen for.
    pub sizes: (f32, f32),
    pub weight: u32,
    /// Where the door is across the front (-1..1 of the half-width).
    pub door: f32,
    /// Storeys seen from outside (drawing only).
    pub storeys: u8,
    pub walls: &'static [Wall],
    pub furniture: &'static [Piece],
    pub holders: &'static [Spot],
}

const fn w(a: (f32, f32), b: (f32, f32), gap: Option<f32>) -> Wall {
    Wall { a, b, gap }
}
const fn f(what: Furn, x: f32, y: f32, rot: f32) -> Piece {
    Piece { what, at: (x, y), rot }
}
const fn h(what: Holder, x: f32, y: f32, rot: f32) -> Spot {
    Spot { what, at: (x, y), rot }
}

const Q: f32 = std::f32::consts::FRAC_PI_2;

use BuildingKind as K;
use Furn::*;
use Holder::*;

/// Every variant. Cultures are tendencies elsewhere, but buildings are
/// canon architecture (`architecture.md`), so they go by building kind.
pub const VARIANTS: &[Variant] = &[
    // ---- Roduro: grown stone, round ----------------------------------------
    Variant {
        key: "roduro_cottage",
        name: "Grown cottage",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Home,
        half: (0.5, 0.5),
        sizes: (0.0, 9.6),
        weight: 3,
        door: 0.35,
        storeys: 1,
        // A screen walls off the sleeping alcove at the back left.
        walls: &[w((-0.1, 1.0), (-0.1, 0.15), None)],
        furniture: &[f(Bed, -0.55, 0.5, 0.0), f(Hearth, 0.05, -0.35, 0.0), f(Table, 0.35, -0.05, Q), f(Bench, 0.35, 0.3, Q), f(Shelf, -0.85, -0.3, Q)],
        holders: &[h(Chest, -0.72, 0.05, Q), h(Barrel, 0.5, -0.65, 0.0)],
    },
    Variant {
        key: "roduro_longhouse",
        name: "Grown longhouse",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Home,
        half: (0.36, 0.5),
        sizes: (8.0, 13.0),
        weight: 3,
        door: -0.3,
        storeys: 1,
        // Split down the middle: the hall with the hearth, and the sleeping room.
        walls: &[w((1.0, 0.2), (-1.0, 0.2), Some(0.45))],
        furniture: &[
            f(Hearth, 0.0, -0.45, 0.0),
            f(LongTable, -0.45, -0.5, 0.0),
            f(Bench, -0.15, -0.75, 0.0),
            f(Bed, -0.4, 0.55, 0.0),
            f(Bed, 0.35, 0.6, 0.0),
            f(Shelf, -0.85, 0.75, Q),
        ],
        holders: &[h(Cupboard, 0.3, 0.8, 0.0), h(Chest, -0.35, 0.85, 0.0), h(Barrel, 0.5, -0.72, 0.0)],
    },
    Variant {
        key: "roduro_tender",
        name: "Stone-tender's workshop",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Workshop,
        half: (0.5, 0.5),
        sizes: (8.5, 13.0),
        weight: 2,
        door: 0.0,
        storeys: 2,
        // Grower's beds fill the front; a sleeping nook behind a wall at the back.
        walls: &[w((-0.45, -0.9), (-0.45, 0.9), Some(0.5))],
        furniture: &[
            f(GrowerBed, 0.3, -0.5, Q),
            f(GrowerBed, 0.3, 0.5, Q),
            f(Workbench, -0.15, -0.6, Q),
            f(Shelf, -0.25, 0.75, Q),
            f(Bedroll, -0.75, 0.15, Q),
            f(Hearth, -0.7, -0.4, 0.0),
        ],
        holders: &[h(Crate, 0.0, 0.25, 0.0), h(Crate, 0.0, -0.2, 0.3), h(Chest, -0.7, 0.55, Q)],
    },
    Variant {
        key: "roduro_forge",
        name: "Maker's forge",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Workshop,
        half: (0.42, 0.5),
        sizes: (8.0, 13.0),
        weight: 2,
        door: 0.4,
        storeys: 1,
        // The forge floor in front; a storeroom behind.
        walls: &[w((-0.35, -1.0), (-0.35, 1.0), Some(0.3))],
        furniture: &[
            f(Forge, 0.15, -0.6, 0.0),
            f(Anvil, 0.35, -0.15, 0.3),
            f(Workbench, 0.0, 0.55, Q),
            f(Rack, 0.55, -0.7, 0.4),
            f(Shelf, -0.7, 0.0, Q),
        ],
        holders: &[h(Crate, -0.65, 0.5, 0.0), h(Crate, -0.6, -0.55, 0.2), h(Barrel, 0.6, 0.65, 0.0), h(Chest, -0.8, 0.15, Q)],
    },
    Variant {
        key: "roduro_trader",
        name: "Trader's house",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Shop,
        half: (0.5, 0.5),
        sizes: (8.0, 13.0),
        weight: 2,
        door: -0.35,
        storeys: 2,
        // A shop at the front behind a counter; living space at the back.
        walls: &[w((0.0, -1.0), (0.0, 1.0), Some(0.72))],
        furniture: &[
            f(Counter, 0.35, 0.2, Q),
            f(Shelf, 0.25, 0.85, 0.0),
            f(Shelf, 0.15, -0.8, 0.0),
            f(Bed, -0.6, -0.4, 0.0),
            f(Table, -0.45, 0.35, 0.0),
            f(Hearth, -0.75, 0.05, 0.0),
        ],
        holders: &[h(Cupboard, 0.05, 0.5, 0.0), h(Crate, 0.6, 0.55, 0.0), h(Chest, -0.8, -0.35, Q)],
    },
    Variant {
        key: "roduro_great",
        name: "Great house",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Home,
        half: (0.5, 0.5),
        sizes: (11.0, 13.0),
        weight: 4,
        door: 0.0,
        storeys: 2,
        // A hearth hall in the middle, a room either side.
        walls: &[w((0.75, 0.38), (-0.75, 0.38), Some(0.5)), w((0.75, -0.38), (-0.75, -0.38), Some(0.5))],
        furniture: &[
            f(Hearth, -0.1, 0.0, 0.0),
            f(LongTable, 0.4, 0.0, 0.0),
            f(Bench, 0.4, 0.2, 0.0),
            f(Bench, 0.4, -0.2, 0.0),
            f(Bed, -0.2, 0.7, 0.0),
            f(Bed, 0.35, 0.7, 0.0),
            f(Bed, -0.2, -0.7, 0.0),
            f(Shelf, 0.4, -0.75, 0.0),
        ],
        holders: &[h(Chest, -0.55, 0.6, Q), h(Chest, -0.55, -0.6, Q), h(Cupboard, -0.85, 0.0, Q), h(Barrel, 0.65, -0.55, 0.0)],
    },
    // ---- Qotiro: quarried, stepped, boxes -------------------------------------
    Variant {
        key: "qotiro_quarters",
        name: "Living quarters",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Quarters,
        half: (0.5, 0.4),
        sizes: (0.0, 30.0),
        weight: 4,
        door: 0.0,
        storeys: 3,
        // One entrance, a corridor to the back, rooms off either side.
        walls: &[
            // Each corridor wall in two stretches, one doorway each (a wall
            // has one gap; two copies of one wall would cover each other's).
            w((0.75, 0.22), (-0.15, 0.22), Some(0.43)),
            w((-0.15, 0.22), (-1.0, 0.22), Some(0.42)),
            w((0.75, -0.22), (-0.15, -0.22), Some(0.43)),
            w((-0.15, -0.22), (-1.0, -0.22), Some(0.42)),
            w((-0.15, 0.22), (-0.15, 1.0), None),
            w((-0.15, -0.22), (-0.15, -1.0), None),
        ],
        furniture: &[
            f(Bed, 0.35, 0.75, 0.0),
            f(Bedroll, 0.35, 0.45, 0.0),
            f(Bed, -0.6, 0.75, 0.0),
            f(Table, -0.55, 0.45, 0.0),
            f(Bed, 0.35, -0.75, 0.0),
            f(Shelf, 0.6, -0.5, Q),
            f(Bed, -0.6, -0.75, 0.0),
            f(Bedroll, -0.55, -0.45, 0.0),
        ],
        holders: &[h(Chest, 0.05, 0.85, 0.0), h(Chest, -0.9, 0.6, Q), h(Cupboard, 0.05, -0.85, 0.0), h(Chest, -0.9, -0.6, Q), h(Barrel, -0.85, 0.0, 0.0)],
    },
    Variant {
        key: "qotiro_court",
        name: "Courtyard house",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Home,
        half: (0.5, 0.4),
        sizes: (15.0, 30.0),
        weight: 3,
        door: 0.0,
        storeys: 2,
        // Rooms round an open court in the middle.
        walls: &[
            w((0.4, -0.45), (0.4, 0.45), Some(0.5)),
            w((-0.4, -0.45), (-0.4, 0.45), Some(0.5)),
            w((0.4, 0.45), (-0.4, 0.45), Some(0.5)),
            w((0.4, -0.45), (-0.4, -0.45), Some(0.5)),
        ],
        furniture: &[
            f(Hearth, 0.0, 0.0, 0.0),
            f(Bench, 0.2, 0.25, 0.0),
            f(Bed, -0.7, 0.6, 0.0),
            f(Bed, -0.7, -0.6, 0.0),
            f(Table, 0.0, 0.75, 0.0),
            f(Loom, 0.0, -0.75, 0.0),
            f(Shelf, -0.9, 0.0, Q),
        ],
        holders: &[h(Cupboard, 0.65, 0.75, 0.0), h(Chest, -0.85, 0.35, Q), h(Barrel, 0.65, -0.75, 0.0), h(Barrel, 0.55, -0.6, 0.0)],
    },
    Variant {
        key: "qotiro_workyard",
        name: "Smiths' and potters' yard",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Workshop,
        half: (0.5, 0.4),
        sizes: (0.0, 30.0),
        weight: 2,
        door: -0.4,
        storeys: 2,
        // The work floor at the front; a locked storeroom at the back.
        walls: &[w((-0.45, -1.0), (-0.45, 1.0), Some(0.7))],
        furniture: &[
            f(Forge, 0.3, 0.65, 0.0),
            f(Anvil, 0.35, 0.3, 0.0),
            f(Kiln, -0.1, 0.7, 0.0),
            f(Workbench, 0.1, -0.15, Q),
            f(Workbench, -0.2, -0.6, 0.0),
            f(Rack, 0.65, -0.8, 0.0),
            f(Shelf, -0.65, -0.85, 0.0),
        ],
        holders: &[h(Crate, -0.7, 0.75, 0.0), h(Crate, -0.75, 0.35, 0.3), h(Barrel, 0.75, 0.15, 0.0), h(Chest, -0.85, -0.35, Q)],
    },
    Variant {
        key: "qotiro_market",
        name: "Market hall",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Shop,
        half: (0.5, 0.4),
        sizes: (15.0, 30.0),
        weight: 2,
        door: 0.0,
        storeys: 2,
        // Stalls down both sides; the stock behind a wall at the back.
        walls: &[w((-0.55, -1.0), (-0.55, 1.0), Some(0.5))],
        furniture: &[
            f(Counter, 0.45, 0.65, 0.0),
            f(Counter, -0.1, 0.65, 0.0),
            f(Counter, 0.45, -0.65, 0.0),
            f(Counter, -0.1, -0.65, 0.0),
            f(Shelf, 0.45, 0.92, 0.0),
            f(Shelf, -0.1, -0.92, 0.0),
            f(Rack, -0.85, 0.5, Q),
        ],
        holders: &[h(Crate, -0.75, -0.6, 0.0), h(Crate, -0.8, -0.2, 0.2), h(Cupboard, -0.85, 0.15, Q), h(Chest, -0.7, 0.8, 0.0)],
    },
    Variant {
        key: "qotiro_mess",
        name: "Mess hall",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Hall,
        half: (0.5, 0.4),
        sizes: (0.0, 30.0),
        weight: 2,
        door: 0.0,
        storeys: 2,
        // Long tables in the hall; the kitchen and pantry at the back.
        walls: &[w((-0.4, -1.0), (-0.4, 0.4), Some(0.5))],
        furniture: &[
            f(LongTable, 0.25, 0.45, 0.0),
            f(Bench, 0.25, 0.65, 0.0),
            f(Bench, 0.25, 0.25, 0.0),
            f(LongTable, 0.25, -0.45, 0.0),
            f(Bench, 0.25, -0.65, 0.0),
            f(Bench, 0.25, -0.25, 0.0),
            f(Hearth, -0.75, -0.3, 0.0),
            f(Table, -0.6, -0.75, 0.0),
        ],
        holders: &[h(Barrel, -0.85, 0.8, 0.0), h(Barrel, -0.7, 0.8, 0.0), h(Barrel, -0.85, -0.85, 0.0), h(Cupboard, -0.5, 0.85, 0.0)],
    },
    Variant {
        key: "qotiro_temple",
        name: "Temple-fortress",
        kind: K::QotiroTemple,
        shape: Shape::Rect,
        use_: Use::Shrine,
        half: (0.5, 0.5),
        sizes: (0.0, 200.0),
        weight: 1,
        door: 0.0,
        storeys: 3,
        // The great hall, with a sanctum behind.
        walls: &[w((-0.5, -1.0), (-0.5, 1.0), Some(0.5))],
        furniture: &[f(Altar, -0.75, 0.0, Q), f(Bench, 0.3, 0.4, Q), f(Bench, 0.3, -0.4, Q), f(Bench, -0.05, 0.4, Q), f(Bench, -0.05, -0.4, Q), f(Shelf, -0.8, 0.6, Q)],
        holders: &[h(Chest, -0.8, -0.5, Q), h(Cupboard, -0.85, 0.35, Q)],
    },
    // The island (diaspora) hall: Qotiro shape, local dark stone (`architecture.md`).
    // Every Qotiro building is in that dark stone; no sandstone (Laz).
    Variant {
        key: "qotiro_island_hall",
        name: "Qotiro hall",
        kind: K::QotiroHall,
        shape: Shape::Rect,
        use_: Use::Hall,
        half: (0.48, 0.4),
        sizes: (0.0, 40.0),
        weight: 1,
        door: 0.0,
        storeys: 2,
        // A communal hall, with a small shrine room and a store at the back.
        walls: &[w((-0.45, -1.0), (-0.45, 1.0), Some(0.25)), w((-0.45, 0.1), (-1.0, 0.1), None)],
        furniture: &[f(LongTable, 0.3, 0.0, 0.0), f(Bench, 0.3, 0.25, 0.0), f(Bench, 0.3, -0.25, 0.0), f(Hearth, -0.2, 0.6, 0.0), f(Altar, -0.8, 0.55, Q)],
        holders: &[h(Chest, -0.8, -0.35, Q), h(Barrel, -0.6, -0.8, 0.0), h(Cupboard, 0.6, -0.85, 0.0)],
    },
    // ---- Horaro: stilt homes (look only, for now) ------------------------------
    Variant { key: "horaro_dome", name: "Woven dome", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.5, 0.5), sizes: (0.0, 99.0), weight: 4, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[] },
    Variant { key: "horaro_twin", name: "Twin domes", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.36, 0.5), sizes: (8.0, 99.0), weight: 2, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[] },
    Variant { key: "horaro_hull", name: "Hull house", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.5, 0.32), sizes: (0.0, 99.0), weight: 2, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[] },
    Variant { key: "horaro_net_loft", name: "Net loft", kind: K::HoraroStilt, shape: Shape::Rect, use_: Use::Workshop, half: (0.42, 0.34), sizes: (0.0, 99.0), weight: 1, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[] },
];

/// The variants a kind of building can be.
pub fn variants_of(kind: BuildingKind) -> impl Iterator<Item = &'static Variant> {
    VARIANTS.iter().filter(move |v| v.kind == kind)
}

/// A building's variant (None for the hearth). One keyed roll among the
/// variants of its kind that suit its size.
pub fn variant_of(b: &Building) -> Option<&'static Variant> {
    variant_for(b.kind, b.size, b.seed)
}

pub fn variant_for(kind: BuildingKind, size: f32, seed: u64) -> Option<&'static Variant> {
    let fits: Vec<&Variant> = variants_of(kind).filter(|v| size >= v.sizes.0 && size < v.sizes.1).collect();
    let fits = if fits.is_empty() { variants_of(kind).collect() } else { fits };
    let total: u32 = fits.iter().map(|v| v.weight).sum();
    if total == 0 {
        return None;
    }
    let mut roll = Rng::from_keys(&[seed, 0x5641_5249]).below(total as usize) as u32;
    for v in &fits {
        if roll < v.weight {
            return Some(v);
        }
        roll -= v.weight;
    }
    None
}

/// The index of a variant in `VARIANTS`.
pub fn index_of(v: &Variant) -> usize {
    VARIANTS.iter().position(|x| x.key == v.key).unwrap_or(0)
}

/// A seed and size that give this variant (for showing every variant).
pub fn example_of(v: &Variant) -> (f32, u64) {
    let size = ((v.sizes.0.max(7.5) + v.sizes.1.min(v.sizes.0.max(7.5) + 4.0)) * 0.5).max(v.sizes.0);
    let size = match v.kind {
        BuildingKind::QotiroBlock => size.clamp(15.5, 19.0),
        BuildingKind::QotiroTemple => 40.0,
        BuildingKind::QotiroHall => 16.0,
        BuildingKind::HoraroStilt => 9.0,
        _ => size.clamp(7.5, 12.4),
    };
    let seed = (0..100_000u64).find(|&s| variant_for(v.kind, size, s).map(|x| x.key) == Some(v.key)).unwrap_or(0);
    (size, seed)
}
