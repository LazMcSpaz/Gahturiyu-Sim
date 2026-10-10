//! Building variants: what each building is like inside (playable-MVP step 6).
//!
//! A town's buildings have their variant chosen once, when the world is
//! made, from the trades of the households living there (`style_for`,
//! `World::style_buildings`), and stored by key in `Settlement::styles`; it
//! never changes after, even when people change jobs. One table
//! (`TRADE_STYLES`) decides the building, who keeps it (`keeps`,
//! `World::keeper`), and with the variant alone its sign (`sign_of`). A building with no stored
//! style (one added later) falls back to a keyed roll on its own kind, size
//! and seed (`variant_of`). Each
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

use super::buildings::Door;
use super::geo::V2;
use super::jobs::Job;
use super::rng::Rng;
use super::settlement::{Building, BuildingKind, Settlement};

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
        walls: &[w((-0.05, 1.0), (-0.05, 0.15), None)],
        furniture: &[f(Bed, -0.42, 0.4, 0.0), f(Hearth, 0.05, -0.35, 0.0), f(Table, 0.35, -0.05, Q), f(Bench, 0.58, -0.05, Q), f(Shelf, -0.62, -0.35, Q)],
        holders: &[h(Chest, 0.2, 0.69, 0.0), h(Barrel, 0.47, -0.58, 0.0)],
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
        // Split down the middle: the hall with the hearth, and the sleeping
        // room (two beds with their heads to the far wall, a chest between).
        walls: &[w((1.0, 0.05), (-1.0, 0.05), Some(0.2))],
        furniture: &[
            f(Hearth, 0.1, -0.45, 0.0),
            f(Table, -0.49, -0.4, Q),
            f(Bench, -0.23, -0.4, Q),
            f(Bed, -0.49, 0.35, Q),
            f(Bed, 0.21, 0.35, Q),
            f(Shelf, -0.42, -0.04, 0.0),
        ],
        holders: &[h(Cupboard, -0.13, -0.77, 0.0), h(Chest, -0.14, 0.725, 0.0), h(Barrel, 0.44, -0.6, 0.0)],
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
            f(Shelf, -0.22, 0.68, Q),
            f(Bedroll, -0.74, 0.14, Q),
            f(Hearth, -0.65, -0.37, 0.0),
        ],
        holders: &[h(Crate, 0.0, 0.25, 0.0), h(Crate, 0.0, -0.2, 0.3), h(Chest, -0.57, 0.51, Q)],
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
        walls: &[w((-0.35, -1.0), (-0.35, 1.0), Some(0.4))],
        furniture: &[
            f(Forge, 0.14, -0.56, 0.0),
            f(Anvil, -0.05, -0.32, 0.3),
            f(Workbench, 0.0, 0.55, Q),
            f(Rack, 0.59, -0.27, 0.4),
            f(Shelf, -0.7, 0.12, Q),
        ],
        holders: &[h(Crate, -0.56, 0.48, 0.0), h(Crate, -0.55, -0.5, 0.2), h(Barrel, 0.5, 0.57, 0.0), h(Chest, -0.71, -0.25, Q)],
    },
    Variant {
        key: "roduro_loomroom",
        name: "Weaver's house",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Workshop,
        half: (0.45, 0.5),
        sizes: (8.0, 13.0),
        weight: 2,
        door: -0.2,
        storeys: 1,
        // Two looms and a cutting bench in front; the living and sleeping
        // room behind a wall at the back.
        walls: &[w((-0.25, -1.0), (-0.25, 1.0), Some(0.7))],
        furniture: &[
            f(Loom, 0.1, -0.6, 0.0),
            f(Loom, 0.45, -0.25, Q),
            f(Table, 0.3, 0.55, Q),
            f(Bed, -0.57, 0.3, Q),
            f(Hearth, -0.57, -0.42, 0.0),
            f(Shelf, -0.81, -0.03, Q),
        ],
        holders: &[h(Crate, 0.6, 0.15, 0.0), h(Cupboard, -0.05, 0.75, 0.0), h(Chest, -0.45, -0.04, 0.0), h(Barrel, -0.4, -0.65, 0.0)],
    },
    Variant {
        key: "roduro_benchroom",
        name: "Bench house",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Workshop,
        half: (0.45, 0.5),
        sizes: (8.0, 13.0),
        weight: 2,
        door: 0.25,
        storeys: 1,
        // Tanners', woodworkers' and masons' benches and racks of hides and
        // timber in front; the living room behind.
        walls: &[w((-0.2, -1.0), (-0.2, 1.0), Some(0.3))],
        furniture: &[
            f(Workbench, 0.3, -0.55, Q),
            f(Workbench, 0.12, 0.6, 0.0),
            f(Rack, 0.5, 0.41, 0.4),
            f(Rack, 0.12, -0.23, 0.0),
            f(Bed, -0.57, 0.3, Q),
            f(Hearth, -0.57, -0.42, 0.0),
        ],
        holders: &[h(Crate, 0.55, -0.1, 0.0), h(Crate, 0.25, 0.15, 0.3), h(Chest, -0.45, -0.04, 0.0), h(Barrel, -0.78, -0.05, 0.0)],
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
            f(Shelf, 0.24, 0.74, 0.0),
            f(Shelf, 0.23, -0.74, 0.0),
            f(Bed, -0.4, -0.5, 0.0),
            f(Table, -0.45, 0.35, 0.0),
            f(Hearth, -0.75, 0.05, 0.0),
        ],
        holders: &[h(Cupboard, 0.31, 0.58, 0.0), h(Crate, 0.57, 0.5, 0.0), h(Chest, -0.74, -0.25, Q)],
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
            f(Bed, 0.33, 0.68, 0.0),
            f(Bed, -0.2, -0.7, 0.0),
            f(Shelf, 0.38, -0.73, 0.0),
        ],
        holders: &[h(Chest, -0.55, 0.6, Q), h(Chest, -0.55, -0.6, Q), h(Cupboard, -0.85, 0.0, Q), h(Barrel, 0.64, -0.54, 0.0)],
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
            f(Bench, 0.0, -0.75, 0.0),
            f(Shelf, -0.9, 0.0, Q),
        ],
        holders: &[h(Cupboard, 0.65, 0.75, 0.0), h(Chest, -0.85, 0.35, Q), h(Barrel, 0.65, -0.75, 0.0), h(Barrel, 0.55, -0.6, 0.0)],
    },
    Variant {
        key: "qotiro_workyard",
        name: "Smiths' yard",
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
            f(Forge, -0.1, 0.7, 0.0),
            f(Workbench, 0.1, -0.15, Q),
            f(Workbench, -0.2, -0.6, 0.0),
            f(Rack, 0.65, -0.8, 0.0),
            f(Shelf, -0.65, -0.85, 0.0),
        ],
        holders: &[h(Crate, -0.7, 0.75, 0.0), h(Crate, -0.75, 0.35, 0.3), h(Barrel, 0.75, 0.15, 0.0), h(Chest, -0.85, -0.35, Q)],
    },
    Variant {
        key: "qotiro_weavehall",
        name: "Weaving court",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Workshop,
        half: (0.5, 0.4),
        sizes: (0.0, 30.0),
        weight: 2,
        door: 0.35,
        storeys: 2,
        // Rooms round an open court, the court full of looms.
        walls: &[
            w((0.4, -0.45), (0.4, 0.45), Some(0.5)),
            w((-0.4, -0.45), (-0.4, 0.45), Some(0.5)),
            w((0.4, 0.45), (-0.4, 0.45), Some(0.5)),
            w((0.4, -0.45), (-0.4, -0.45), Some(0.5)),
        ],
        furniture: &[
            f(Loom, 0.15, 0.2, 0.0),
            f(Loom, 0.15, -0.2, 0.0),
            f(Loom, -0.2, 0.0, Q),
            f(Bed, -0.7, 0.6, 0.0),
            f(Bed, -0.7, -0.6, 0.0),
            f(Table, 0.0, 0.75, 0.0),
            f(Rack, 0.0, -0.75, 0.0),
        ],
        holders: &[h(Crate, 0.65, 0.75, 0.0), h(Chest, -0.85, 0.35, Q), h(Cupboard, 0.65, -0.75, 0.0), h(Barrel, -0.85, -0.3, 0.0)],
    },
    Variant {
        key: "qotiro_benchyard",
        name: "Benchworkers' yard",
        kind: K::QotiroBlock,
        shape: Shape::Rect,
        use_: Use::Workshop,
        half: (0.5, 0.4),
        sizes: (0.0, 30.0),
        weight: 2,
        door: 0.4,
        storeys: 2,
        // Benches and racks of hides, timber and dressed stone on the work
        // floor; a storeroom at the back.
        walls: &[w((-0.45, -1.0), (-0.45, 1.0), Some(0.3))],
        furniture: &[
            f(Workbench, 0.3, -0.6, 0.0),
            f(Workbench, 0.3, 0.1, Q),
            f(Workbench, -0.15, 0.6, 0.0),
            f(Rack, 0.65, 0.75, 0.0),
            f(Rack, -0.2, -0.8, 0.0),
            f(Shelf, -0.65, 0.85, 0.0),
        ],
        holders: &[h(Crate, -0.7, -0.75, 0.0), h(Crate, -0.75, -0.35, 0.3), h(Barrel, 0.75, -0.15, 0.0), h(Chest, -0.85, 0.35, Q)],
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
            f(Shelf, 0.45, 0.9, 0.0),
            f(Shelf, -0.1, -0.9, 0.0),
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

/// Which trade asks for which variant, by kind of building. One line per
/// group of jobs; a job not listed (farmers, guards, alchemists, scribes...)
/// asks for nothing, and its household gets a plain home. The same table
/// decides who keeps the building (`keeps`) and, with the variant alone, its
/// sign (`sign_of`). Changing the mapping is a change to this table only.
/// Fire and metal.
const SMITHS: &[Job] = &[Job::Smith, Job::Armourer, Job::CharcoalBurner];
/// Cloth.
const LOOMWORK: &[Job] = &[Job::Weaver, Job::Tailor];
/// Benchwork: hide, wood and stone.
const BENCHWORK: &[Job] = &[Job::Tanner, Job::Leatherworker, Job::Woodworker, Job::Carpenter, Job::Mason, Job::Boatwright];
const TRADERS: &[Job] = &[Job::Merchant, Job::Exchanger];
pub const TRADE_STYLES: &[(BuildingKind, &[Job], &str)] = &[
    (K::RoduroHome, SMITHS, "roduro_forge"),
    (K::RoduroHome, LOOMWORK, "roduro_loomroom"),
    (K::RoduroHome, BENCHWORK, "roduro_benchroom"),
    (K::RoduroHome, &[Job::StoneTender], "roduro_tender"),
    (K::RoduroHome, TRADERS, "roduro_trader"),
    (K::QotiroBlock, &[Job::Smith, Job::Armourer], "qotiro_workyard"),
    (K::QotiroBlock, BENCHWORK, "qotiro_benchyard"),
    (K::QotiroBlock, LOOMWORK, "qotiro_weavehall"),
    // (The mess hall is no one's home: cooks work at their community's kitchen.)
    (K::QotiroBlock, TRADERS, "qotiro_market"),
];

/// Plain homes by how many live there: (variant, most residents it suits).
/// The first that suits the head count and fits the plot wins; if none
/// fits, the nearest in this list that does.
pub const PLAIN_HOMES: &[(BuildingKind, &str, usize)] = &[
    (K::RoduroHome, "roduro_cottage", 3),
    (K::RoduroHome, "roduro_longhouse", 4),
    (K::RoduroHome, "roduro_great", usize::MAX),
    (K::QotiroBlock, "qotiro_court", 5),
    (K::QotiroBlock, "qotiro_quarters", usize::MAX),
];

/// A building's sign over the door.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Anvil,
    Sprout,
    Coin,
    Scales,
    Loom,
    Hammer,
    Bowl,
    Sun,
    Flask,
    Scroll,
}

/// The sign a building of this variant hangs: decided by the variant alone,
/// never by who lives there. None for plain homes, quarters and stilts.
pub fn sign_of(v: &Variant) -> Option<Sign> {
    Some(match v.key {
        "roduro_forge" | "qotiro_workyard" => Sign::Anvil,
        "roduro_tender" => Sign::Sprout,
        "roduro_trader" => Sign::Coin,
        "qotiro_market" => Sign::Scales,
        "roduro_loomroom" | "qotiro_weavehall" => Sign::Loom,
        "roduro_benchroom" | "qotiro_benchyard" => Sign::Hammer,
        "qotiro_mess" => Sign::Bowl,
        "qotiro_temple" | "qotiro_island_hall" => Sign::Sun,
        _ => return None,
    })
}

/// Does someone with this job keep a building of this variant? Exactly
/// when the job would have asked for it (`TRADE_STYLES`).
pub fn keeps(v: &Variant, job: Job) -> bool {
    trade_style(v.kind, job) == Some(v.key)
}

/// Is this variant one a trade asks for?
pub fn is_trade_style(v: &Variant) -> bool {
    TRADE_STYLES.iter().any(|t| t.0 == v.kind && t.2 == v.key)
}

/// The variant with this key.
pub fn by_key(key: &str) -> Option<&'static Variant> {
    VARIANTS.iter().find(|v| v.key == key)
}

/// How many of a building's residents must share a trade for it to shape
/// the building: one in `TRADE_SHARE`, and at least one.
pub const TRADE_SHARE: usize = 6;

pub fn trade_need(residents: usize) -> usize {
    (residents / TRADE_SHARE).max(1)
}

/// The variant a job asks for in this kind of building, if any.
pub fn trade_style(kind: BuildingKind, job: Job) -> Option<&'static str> {
    TRADE_STYLES.iter().find(|(k, jobs, _)| *k == kind && jobs.contains(&job)).map(|t| t.2)
}

/// The variant for a building given the jobs of everyone living there,
/// **head of house first** (`World::style_buildings` puts the eldest first).
/// A Roduro home is a workshop only when its head works a trade in
/// `TRADE_STYLES`; a Qotiro block is one when enough of those living there
/// work it (at least one, and at least one in `TRADE_SHARE`), so a big block
/// isn't a workshop for one smith; the strongest trade wins, ties by a keyed
/// roll. Otherwise a plain home (or quarters) by head count (`plain_home`):
/// never a trade's building without that trade. Temples, halls and stilt
/// homes keep their roll. Pure: the same inputs, the same answer.
pub fn style_for(kind: BuildingKind, size: f32, seed: u64, jobs: &[Job]) -> Option<&'static Variant> {
    if !matches!(kind, K::RoduroHome | K::QotiroBlock) {
        return variant_for(kind, size, seed);
    }
    let fits = |v: &&Variant| size >= v.sizes.0 && size < v.sizes.1;
    let mut r = Rng::from_keys(&[seed, 0x5354_594C]);
    let (trade_jobs, need) = match kind {
        K::RoduroHome => (&jobs[..jobs.len().min(1)], 1),
        _ => (jobs, trade_need(jobs.len())),
    };
    let mut best: Vec<(&'static Variant, usize)> = Vec::new();
    for v in variants_of(kind).filter(fits) {
        let n = trade_jobs.iter().filter(|&&j| keeps(v, j)).count();
        if n < need {
            continue;
        }
        match best.first().map(|b| b.1) {
            Some(top) if n < top => {}
            Some(top) if n == top => best.push((v, n)),
            _ => best = vec![(v, n)],
        }
    }
    if !best.is_empty() {
        return Some(best[r.below(best.len())].0);
    }
    plain_home(kind, size, jobs.len()).or_else(|| {
        // Nothing plain fits the plot: the plain home for the head count anyway.
        let list: Vec<&(BuildingKind, &str, usize)> = PLAIN_HOMES.iter().filter(|p| p.0 == kind).collect();
        let p = list.iter().find(|p| jobs.len() <= p.2).or(list.last())?;
        by_key(p.1)
    })
}

/// The plain home for a head count (`PLAIN_HOMES`): the first that suits
/// that many and fits the plot, else the nearest in the list that fits.
pub fn plain_home(kind: BuildingKind, size: f32, heads: usize) -> Option<&'static Variant> {
    let list: Vec<(&'static Variant, usize)> = PLAIN_HOMES.iter().filter(|p| p.0 == kind).filter_map(|p| VARIANTS.iter().find(|v| v.key == p.1).map(|v| (v, p.2))).collect();
    let want = list.iter().position(|p| heads <= p.1)?;
    let fits = |v: &Variant| size >= v.sizes.0 && size < v.sizes.1;
    (0..list.len()).filter(|&k| fits(list[k].0)).min_by_key(|&k| (k.abs_diff(want), std::cmp::Reverse(k))).map(|k| list[k].0)
}

/// Building `i` of a town's variant: the style chosen for it when the world
/// was made, or (for one added since) its own roll.
pub fn variant_in(s: &Settlement, i: u16) -> Option<&'static Variant> {
    let b = s.buildings.get(i as usize)?;
    match s.styles.get(i as usize).copied().flatten().and_then(by_key) {
        Some(v) if v.kind == b.kind => Some(v),
        _ => variant_of(b),
    }
}

/// The variants a kind of building can be.
pub fn variants_of(kind: BuildingKind) -> impl Iterator<Item = &'static Variant> {
    VARIANTS.iter().filter(move |v| v.kind == kind)
}

/// A building's own roll (None for the hearth): one keyed roll among the
/// variants of its kind that suit its size. Town buildings use their stored
/// style instead (`variant_in`); this is the fallback.
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

// ---- The floor plan: where everything stands -----------------------------------
//
// Pure geometry on the map (metres), for the window to draw and the tests to
// check: every piece's footprint, the extra bedrolls a crowded house lays
// out, and a Ṭaḍoro lodger's corner. Nothing here changes what happens.

/// Most extra bedrolls laid out in living quarters, and in any other building.
pub const MAX_EXTRA_QUARTERS: usize = 16;
pub const MAX_EXTRA: usize = 8;
/// How far in from the outline the outer wall's inside face is, metres.
pub const WALL_IN: f32 = 0.34;
/// Thickness of an inner wall, metres.
pub const INNER_WALL: f32 = 0.2;
/// The clear floor kept round anything laid out on top of the variant
/// (extra bedrolls, a lodger's corner), metres: room to walk.
pub const WALKWAY: f32 = 0.4;

fn dot(a: V2, b: V2) -> f32 {
    a.x * b.x + a.y * b.y
}

/// A box on the floor: its middle, its turn (radians), and half its width
/// (along the turn) and depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub c: V2,
    pub rot: f32,
    pub hw: f32,
    pub hd: f32,
}

impl Rect {
    /// A box `w` by `d` (full sizes) at `c`, turned `rot`.
    pub fn new(c: V2, rot: f32, w: f32, d: f32) -> Rect {
        Rect { c, rot, hw: w * 0.5, hd: d * 0.5 }
    }
    pub fn axes(&self) -> [V2; 2] {
        let (s, c) = self.rot.sin_cos();
        [V2::new(c, s), V2::new(-s, c)]
    }
    /// A point `x` along and `y` across it, from its middle.
    pub fn at(&self, x: f32, y: f32) -> V2 {
        let [u, w] = self.axes();
        self.c.add(u.scale(x)).add(w.scale(y))
    }
    pub fn corners(&self) -> [V2; 4] {
        [self.at(-self.hw, -self.hd), self.at(self.hw, -self.hd), self.at(self.hw, self.hd), self.at(-self.hw, self.hd)]
    }
    /// The same box, `m` bigger all round.
    pub fn grow(&self, m: f32) -> Rect {
        Rect { hw: self.hw + m, hd: self.hd + m, ..*self }
    }
    fn reach(&self, n: V2) -> f32 {
        let [u, w] = self.axes();
        self.hw * dot(u, n).abs() + self.hd * dot(w, n).abs()
    }
    /// How far apart two boxes are along the axis that parts them most
    /// (negative: they overlap by that much). Never more than the true gap.
    pub fn gap(&self, o: &Rect) -> f32 {
        let d = o.c.sub(self.c);
        self.axes().into_iter().chain(o.axes()).map(|n| dot(d, n).abs() - self.reach(n) - o.reach(n)).fold(f32::MIN, f32::max)
    }
    /// Are they at least `m` apart?
    pub fn clear_of(&self, o: &Rect, m: f32) -> bool {
        self.gap(o) >= m
    }
}

/// What a footprint is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thing {
    /// The variant's `furniture[k]`.
    Furniture(usize),
    /// The variant's `holders[k]` (a container's spot).
    Container(usize),
    /// `Door::wall_pieces()[k]`, an inner wall.
    Wall(usize),
    /// Extra bedroll `k` (`extra_sleepers`).
    Extra(usize),
}

/// The footprint of every piece of furniture, every container spot and
/// every stretch of inner wall in a building.
pub fn footprints(d: &Door) -> Vec<(Thing, Rect)> {
    let v = d.variant();
    let mut out = Vec::new();
    for (k, p) in v.furniture.iter().enumerate() {
        let (w, dd, _) = p.what.size();
        out.push((Thing::Furniture(k), Rect::new(d.to_world(p.at), d.rot + p.rot, w, dd)));
    }
    for (k, s) in v.holders.iter().enumerate() {
        let (w, dd, _) = s.what.size();
        out.push((Thing::Container(k), Rect::new(d.to_world(s.at), d.rot + s.rot, w, dd)));
    }
    for (k, (a, z)) in d.wall_pieces().into_iter().enumerate() {
        let dz = z.sub(a);
        out.push((Thing::Wall(k), Rect::new(a.lerp(z, 0.5), dz.y.atan2(dz.x), a.dist(z), INNER_WALL)));
    }
    out
}

/// Floor that must stay clear: the way in from the door to just past its
/// inside point, and a stretch either side of every inner doorway.
pub fn keep_clear(d: &Door) -> Vec<Rect> {
    let dir = V2::new(d.rot.cos(), d.rot.sin());
    let face = d.outside.sub(dir.scale(1.2));
    let end = d.inside.sub(dir.scale(0.5));
    let mut out = vec![Rect::new(face.lerp(end, 0.5), d.rot, face.dist(end), 1.2)];
    for wl in d.variant().walls {
        if let Some(g) = wl.gap {
            let (a, z) = (d.to_world(wl.a), d.to_world(wl.b));
            let dz = z.sub(a);
            out.push(Rect::new(a.lerp(z, g), dz.y.atan2(dz.x), DOORWAY - 0.25, 1.2));
        }
    }
    out
}

/// Does a box lie wholly inside the outer walls, `margin` in from the outline?
pub fn fits_inside(d: &Door, r: &Rect, margin: f32) -> bool {
    let (hx, hy) = (d.half.x - margin, d.half.y - margin);
    if hx <= 0.0 || hy <= 0.0 {
        return false;
    }
    r.corners().into_iter().all(|p| {
        let (x, y) = d.to_local(p);
        let (x, y) = (x * d.half.x / hx, y * d.half.y / hy);
        if d.round { x * x + y * y <= 1.0 } else { x.abs() <= 1.0 && y.abs() <= 1.0 }
    })
}

/// How many extra bedrolls a building lays out for `residents` living
/// there: one each beyond its own beds and bedrolls, up to the most.
pub fn extras_wanted(d: &Door, residents: usize) -> usize {
    let v = d.variant();
    let own = v.furniture.iter().filter(|p| matches!(p.what, Furn::Bed | Furn::Bedroll)).count();
    let most = if v.use_ == Use::Quarters { MAX_EXTRA_QUARTERS } else { MAX_EXTRA };
    residents.saturating_sub(own).min(most)
}

/// A bedroll's footprint at a spot.
pub fn bedroll(at: V2, rot: f32) -> Rect {
    let (w, dd, _) = Furn::Bedroll.size();
    Rect::new(at, rot, w, dd)
}

/// Spots on a grid over the floor (about 0.35 m apart), each with the turn
/// that lies along the nearest outer wall and the one across it, the back of
/// the house (farthest from the door) first.
fn floor_grid(d: &Door) -> Vec<(V2, f32, f32)> {
    let (nx, ny) = (((d.half.x * 2.0 / 0.35) as usize).max(4), ((d.half.y * 2.0 / 0.35) as usize).max(4));
    let dir = V2::new(d.rot.cos(), d.rot.sin());
    let side = V2::new(-dir.y, dir.x);
    let mut out: Vec<(f32, V2, f32, f32)> = Vec::new();
    for i in 0..=nx {
        for j in 0..=ny {
            let (x, y) = (i as f32 / nx as f32 * 1.9 - 0.95, j as f32 / ny as f32 * 1.9 - 0.95);
            if d.round && x * x + y * y > 0.9 {
                continue;
            }
            let p = d.to_world((x, y));
            let n = if d.round {
                dir.scale(x / d.half.x).add(side.scale(y / d.half.y))
            } else if (1.0 - x.abs()) * d.half.x <= (1.0 - y.abs()) * d.half.y {
                dir.scale(x.signum())
            } else {
                side.scale(y.signum())
            };
            let normal = n.y.atan2(n.x);
            out.push((p.dist(d.inside), p, normal + std::f32::consts::FRAC_PI_2, normal));
        }
    }
    out.sort_by(|a, z| z.0.total_cmp(&a.0));
    out.into_iter().map(|(_, p, a, c)| (p, a, c)).collect()
}

/// Is a new box clear of everything already on the floor (by `m`), of the
/// way in and the doorways, and inside the walls?
fn free(d: &Door, r: &Rect, m: f32, taken: &[Rect], clear: &[Rect]) -> bool {
    fits_inside(d, r, WALL_IN + if m >= WALKWAY { 0.05 } else { 0.005 }) && taken.iter().all(|t| r.clear_of(t, m)) && clear.iter().all(|c| r.clear_of(c, 0.05))
}

/// Up to `n` extra bedrolls (centre, turn) for residents beyond the
/// building's own beds: on free floor, along the walls where they can, the
/// back of the house first, never touching furniture, containers, inner
/// walls, the way in, a doorway or each other, with `WALKWAY` clear round
/// each. Fewer if the floor is full.
pub fn extra_sleepers(d: &Door, n: usize) -> Vec<(V2, f32)> {
    extras_avoiding(d, n, &[])
}

/// `extra_sleepers`, keeping `WALKWAY` clear of `avoid` too.
fn extras_avoiding(d: &Door, n: usize, avoid: &[Rect]) -> Vec<(V2, f32)> {
    if n == 0 {
        return Vec::new();
    }
    let mut taken: Vec<Rect> = footprints(d).into_iter().map(|f| f.1).chain(avoid.iter().copied()).collect();
    let clear = keep_clear(d);
    let grid = floor_grid(d);
    let mut out: Vec<(V2, f32)> = Vec::new();
    // Roomy first, then down to the walkway: a crowded bunkroom packs close.
    for m in [0.9f32, 0.6, WALKWAY] {
        for &(p, along, across) in &grid {
            if out.len() >= n {
                return out;
            }
            for rot in [along, across] {
                let r = bedroll(p, rot);
                if free(d, &r, m, &taken, &clear) {
                    out.push((p, rot));
                    taken.push(r);
                    break;
                }
            }
        }
    }
    out
}

/// A Ṭaḍoro lodger's corner round one sleeping place: a rug under it, cloth
/// draped over it with a flap down the room side, a cushion, papers, an ink
/// pot, a dish of candles and a satchel on the room side, and parchment and a
/// lantern on the nearest wall. Every piece's footprint is here.
#[derive(Clone, Copy, Debug)]
pub struct Corner {
    /// The sleeping place: where, its turn (head toward +x), raised or not.
    pub at: V2,
    pub rot: f32,
    pub bed: bool,
    /// Which sleeping place it is: a variant piece, an extra bedroll, or
    /// (None) a bedroll laid out for the lodger.
    pub on: Option<Thing>,
    /// Which long side faces the room (+1 or -1 across the bed).
    pub side: f32,
    pub rug: Rect,
    /// The drape over the bed and its flap.
    pub drape: Rect,
    pub cushion: Option<Rect>,
    pub papers: Rect,
    pub candles: Rect,
    pub ink: Rect,
    pub satchel: Rect,
    /// Where the parchments are pinned: a point on the wall's face, the way
    /// into the room from it, and how high (metres over the floor).
    pub wall_at: V2,
    pub inward: V2,
    pub up: f32,
    /// Each sheet of parchment (`SHEETS`): its spot on the wall's face and
    /// the way into the room there (a round wall curves under them). The
    /// lantern hangs over the middle one.
    pub sheets: [(V2, V2); 3],
    /// The patch of floor in front of the parchments and lantern.
    pub wall_strip: Rect,
}

impl Corner {
    /// The bed (or bedroll) it's round.
    pub fn bed_rect(&self) -> Rect {
        let (w, dd, _) = if self.bed { Furn::Bed.size() } else { Furn::Bedroll.size() };
        Rect::new(self.at, self.rot, w, dd)
    }
    /// The things set down round the bed (not the rug, drape or wall).
    pub fn props(&self) -> Vec<(&'static str, Rect)> {
        let mut v = vec![("papers", self.papers), ("candles", self.candles), ("ink pot", self.ink), ("satchel", self.satchel)];
        if let Some(c) = self.cushion {
            v.push(("cushion", c));
        }
        v
    }
    /// Every footprint of the corner, named.
    pub fn pieces(&self) -> Vec<(&'static str, Rect)> {
        let mut v = vec![("rug", self.rug), ("drape", self.drape), ("parchment wall", self.wall_strip)];
        v.extend(self.props());
        v
    }
}

/// How snugly a corner is laid round its bed.
#[derive(Clone, Copy)]
struct Snug {
    /// Rug reach past the bed's ends and past its back (the side away from
    /// the room), how far the drape hangs over the back, and the cushion.
    ends: f32,
    back: f32,
    drape_back: f32,
    cushion: bool,
    /// The props at the head and foot instead of along the room side.
    at_ends: bool,
}

/// Roomiest first; a corner takes the first that fits.
const SNUGS: [Snug; 6] = [
    Snug { ends: 0.45, back: 0.35, drape_back: 0.15, cushion: true, at_ends: false },
    Snug { ends: 0.12, back: 0.05, drape_back: 0.05, cushion: true, at_ends: false },
    Snug { ends: 0.0, back: 0.0, drape_back: 0.0, cushion: true, at_ends: false },
    Snug { ends: 0.0, back: 0.0, drape_back: 0.0, cushion: false, at_ends: false },
    Snug { ends: 0.0, back: 0.0, drape_back: 0.0, cushion: false, at_ends: true },
    Snug { ends: 0.0, back: 0.0, drape_back: 0.05, cushion: false, at_ends: true },
];

/// How far a fresh bedroll laid for a lodger keeps from anything else
/// (its corner keeps `CORNER_GAP` round it too).
pub const FRESH_GAP: f32 = 0.25;

/// How far a lodger's corner (rug, drape, props) keeps from anything else on
/// the floor, metres: it lies flat or low, so it needs no walkway round it.
pub const CORNER_GAP: f32 = 0.05;

/// The drape over a bed `b` (`bw` long, `bd` wide): over the body from just
/// past the foot (the pillow left clear), hanging `snug.drape_back` over the
/// back and its flap down the room side. Snug, it keeps to the bed's foot.
fn drape(b: &Rect, bw: f32, bd: f32, side: f32, snug: Snug) -> Rect {
    let over = if snug.drape_back >= 0.15 { 0.12 } else { 0.0 };
    let len = bw * 0.8 + 0.08 + over;
    let mid = -bw * 0.5 - over + len * 0.5;
    Rect::new(b.at(mid, side * (0.18 - snug.drape_back) * 0.5), b.rot, len, bd + 0.18 + snug.drape_back)
}

/// The corner round a sleeping place, its room side `side`, laid out `snug`,
/// without its wall yet.
fn corner_round(at: V2, rot: f32, bed: bool, side: f32, snug: Snug) -> Corner {
    let (bw, bd, _) = if bed { Furn::Bed.size() } else { Furn::Bedroll.size() };
    let b = Rect::new(at, rot, bw, bd);
    let edge = bd * 0.5;
    // Along the bed from its middle (head at +x), and out from the room-side edge.
    let put = |x: f32, out: f32, w: f32, dd: f32, turn: f32| Rect::new(b.at(x, side * (edge + out)), rot + turn, w, dd);
    let cushion = snug.cushion.then(|| put(-0.25, 0.55, 0.5, 0.5, 0.15));
    let reach = if snug.cushion { 0.95 } else { 0.62 };
    if snug.at_ends {
        // Papers, candles and ink in a row past the head; the satchel lying
        // across the foot; the rug under the bed and both of them.
        let head = bw * 0.5 + 0.06;
        let put_at = |x: f32, y: f32, w: f32, dd: f32, turn: f32| Rect::new(b.at(x, side * y), rot + turn, w, dd);
        let (front, back) = (0.42, 0.36);
        return Corner {
            at,
            rot,
            bed,
            on: None,
            side,
            rug: Rect::new(b.at((front - back) * 0.5, 0.0), rot, bw + front + back + 0.08, bd),
            drape: drape(&b, bw, bd, side, snug),
            cushion: None,
            papers: put_at(head + 0.17, -0.14, 0.32, 0.44, 0.0),
            candles: put_at(head + 0.14, 0.24, 0.26, 0.26, 0.0),
            ink: put_at(head + 0.09, 0.47, 0.16, 0.16, 0.0),
            satchel: put_at(-bw * 0.5 - 0.2, 0.0, 0.2, 0.44, 0.2),
            wall_at: at,
            inward: V2::new(1.0, 0.0),
            up: 0.95,
            sheets: [(at, V2::new(1.0, 0.0)); 3],
            wall_strip: Rect::new(at, 0.0, 0.0, 0.0),
        };
    }
    let rug_w = bw + snug.ends * 2.0;
    let rug_d = bd + snug.back + reach;
    Corner {
        at,
        rot,
        bed,
        on: None,
        side,
        rug: Rect::new(b.at(0.0, side * (reach - snug.back) * 0.5), rot, rug_w, rug_d),
        drape: drape(&b, bw, bd, side, snug),
        cushion,
        papers: put(0.59, 0.4, 0.44, 0.32, 0.0),
        candles: put(0.22, 0.36, 0.26, 0.26, 0.0),
        ink: put(0.91, 0.33, 0.16, 0.16, 0.0),
        // By the foot; where the cushion would be if there's none.
        satchel: if snug.cushion { put(-bw * 0.5 + 0.17, 0.36, 0.44, 0.2, 0.2) } else { put(-0.3, 0.36, 0.44, 0.2, 0.2) },
        wall_at: at,
        inward: V2::new(1.0, 0.0),
        up: 0.95,
        sheets: [(at, V2::new(1.0, 0.0)); 3],
        wall_strip: Rect::new(at, 0.0, 0.0, 0.0),
    }
}

/// A wall near a spot that the parchments could go on.
struct Hang {
    /// A point on its face, and the way into the room from it.
    at: V2,
    inward: V2,
    /// How high the sheets hang.
    up: f32,
    /// How far the wall runs clear along its face from `at`, each way
    /// (backward, forward along the inward normal turned left).
    back: f32,
    on: f32,
    /// The outer wall (curved, for a round building).
    outer: bool,
}

/// Walls near a spot, nearest first: the outer wall, and every stretch of
/// inner wall.
fn walls_near(d: &Door, p: V2) -> Vec<Hang> {
    let dir = V2::new(d.rot.cos(), d.rot.sin());
    let side = V2::new(-dir.y, dir.x);
    let (lx, ly) = d.to_local(p);
    let mut out: Vec<(f32, Hang)> = Vec::new();
    let (at, n) = if d.round {
        outer_face(d, p)
    } else {
        let walls = [((1.0 - lx) * d.half.x, 0usize), ((1.0 + lx) * d.half.x, 1), ((1.0 - ly) * d.half.y, 2), ((1.0 + ly) * d.half.y, 3)];
        let (_, k) = walls.into_iter().min_by(|a, z| a.0.total_cmp(&z.0)).unwrap_or((0.0, 1));
        let (ex, ey) = (1.0 - FACE / d.half.x, 1.0 - FACE / d.half.y);
        let (px, py, nx, ny) = match k {
            0 => (ex, ly, -1.0, 0.0),
            1 => (-ex, ly, 1.0, 0.0),
            2 => (lx, ey, 0.0, -1.0),
            _ => (lx, -ey, 0.0, 1.0),
        };
        (d.to_world((px, py)), dir.scale(nx).add(side.scale(ny)))
    };
    // A flat wall runs on to the corners; a round one curves (each sheet is
    // put on the curve, `on_face`), so allow a metre or so either way.
    let run = if d.round { 1.0 } else { 2.0 };
    out.push((at.dist(p), Hang { at, inward: n, up: 0.95, back: run, on: run, outer: true }));
    for (a, z) in d.wall_pieces() {
        let c = on_seg(p, a, z);
        let dist = c.dist(p);
        if dist < 0.05 {
            continue;
        }
        let n = p.sub(c).scale(1.0 / dist);
        let along = V2::new(-n.y, n.x);
        let (ta, tz) = (dot(a.sub(c), along), dot(z.sub(c), along));
        let (back, on) = (-ta.min(tz), ta.max(tz));
        // Slid along the wall, if the nearest spot is too near its end.
        let slide = if on < SHEETS_REACH { on - SHEETS_REACH } else if back < SHEETS_REACH { SHEETS_REACH - back } else { 0.0 };
        let at = c.add(n.scale(INNER_WALL * 0.5 + 0.01)).add(along.scale(slide));
        out.push((dist, Hang { at, inward: n, up: 0.82, back: back + slide, on: on - slide, outer: false }));
    }
    out.sort_by(|a, z| a.0.total_cmp(&z.0));
    out.into_iter().map(|x| x.1).collect()
}

/// How far in from the outline things are pinned on the outer wall's face.
const FACE: f32 = WALL_IN + 0.02;

/// The nearest spot on a round building's outer wall face to `p` (straight
/// out from the middle), and the way into the room there.
fn outer_face(d: &Door, p: V2) -> (V2, V2) {
    let dir = V2::new(d.rot.cos(), d.rot.sin());
    let side = V2::new(-dir.y, dir.x);
    let (a, b) = ((d.half.x - FACE).max(0.1), (d.half.y - FACE).max(0.1));
    let (lx, ly) = d.to_local(p);
    let (x, y) = (lx * d.half.x, ly * d.half.y);
    let t = 1.0 / ((x / a).powi(2) + (y / b).powi(2)).sqrt().max(1e-6);
    let (x, y) = (x * t, y * t);
    let n = dir.scale(x / (a * a)).add(side.scale(y / (b * b)));
    let at = d.centre.add(dir.scale(x)).add(side.scale(y));
    (at, n.scale(-1.0 / n.len().max(1e-6)))
}

/// A spot `off` along a wall's face from where the parchments are pinned:
/// along the straight line for an inner or flat wall, on the curve for a
/// round outer wall.
fn on_face(d: &Door, h: &Hang, off: f32) -> (V2, V2) {
    let along = V2::new(-h.inward.y, h.inward.x);
    let p = h.at.add(along.scale(off));
    if h.outer && d.round { outer_face(d, p) } else { (p, h.inward) }
}

/// The nearest point to `p` on the segment `a`–`z`.
pub fn on_seg(p: V2, a: V2, z: V2) -> V2 {
    let d = z.sub(a);
    let l2 = dot(d, d);
    if l2 < 1e-6 {
        return a;
    }
    a.lerp(z, (dot(p.sub(a), d) / l2).clamp(0.0, 1.0))
}

/// Where the three sheets of parchment are pinned along their wall from
/// `wall_at`, metres; they and the lantern over the middle one take half a
/// metre either way.
pub const SHEETS: [f32; 3] = [-0.38, 0.0, 0.36];
const SHEETS_REACH: f32 = 0.5;

/// A corner fully laid out, if every piece of it fits: nothing it puts down
/// touches anything in `taken` (by `CORNER_GAP`), the way in or a
/// doorway, it's all inside the walls, and its parchments hang on a stretch
/// of wall with nothing tall in front.
fn fit_corner(d: &Door, at: V2, rot: f32, bed: bool, taken: &[Rect], clear: &[Rect]) -> Option<Corner> {
    let to_mid = d.centre.sub(at);
    let across = V2::new(-rot.sin(), rot.cos());
    let room = if dot(across, to_mid) >= 0.0 { 1.0 } else { -1.0 };
    let dir = V2::new(d.rot.cos(), d.rot.sin());
    let door_face = d.outside.sub(dir.scale(1.2));
    for snug in SNUGS {
        for side in [room, -room] {
            let mut c = corner_round(at, rot, bed, side, snug);
            let bed_r = c.bed_rect();
            let floor = [c.rug, c.drape];
            let ok = floor.iter().chain(c.props().iter().map(|p| &p.1)).all(|r| free(d, r, CORNER_GAP, taken, clear))
                && c.props().iter().all(|p| p.1.clear_of(&bed_r, 0.02))
                && c.props().iter().enumerate().all(|(i, a)| c.props()[i + 1..].iter().all(|z| a.1.clear_of(&z.1, 0.01)));
            if !ok {
                continue;
            }
            for h in walls_near(d, at) {
                if h.back < SHEETS_REACH || h.on < SHEETS_REACH || h.at.dist(at) > 2.5 {
                    continue;
                }
                let along = V2::new(-h.inward.y, h.inward.x);
                // The floor in front of the sheets, from a hand off the face.
                let strip = Rect::new(h.at.add(h.inward.scale(0.25)), along.y.atan2(along.x), SHEETS_REACH * 2.0, 0.3);
                if strip.c.dist(door_face) < 2.0 || !fits_inside(d, &strip, WALL_IN) || !taken.iter().all(|t| strip.clear_of(t, 0.02)) {
                    continue;
                }
                let (wall_at, inward, up) = (h.at, h.inward, h.up);
                c.sheets = SHEETS.map(|off| on_face(d, &h, off));
                c.wall_at = wall_at;
                c.inward = inward;
                c.up = up;
                c.wall_strip = strip;
                return Some(c);
            }
        }
    }
    None
}

/// Where a Ṭaḍoro lodger's corner goes, given the extra bedrolls laid out:
/// round the sleeping place farthest from the door whose whole corner fits
/// (snugger, or on its other side, if the roomy one doesn't); if none does,
/// round a fresh bedroll laid on free floor where it fits. None if the floor
/// has no room at all.
pub fn lodger_corner(d: &Door, extras: &[(V2, f32)]) -> Option<Corner> {
    let v = d.variant();
    let fp = footprints(d);
    let clear = keep_clear(d);
    let mut places: Vec<(V2, f32, bool, Thing)> = v
        .furniture
        .iter()
        .enumerate()
        .filter(|(_, p)| matches!(p.what, Furn::Bed | Furn::Bedroll))
        .map(|(k, p)| (d.to_world(p.at), d.rot + p.rot, p.what == Furn::Bed, Thing::Furniture(k)))
        .collect();
    places.extend(extras.iter().enumerate().map(|(k, &(at, rot))| (at, rot, false, Thing::Extra(k))));
    places.sort_by(|a, z| z.0.dist(d.inside).total_cmp(&a.0.dist(d.inside)));
    let all: Vec<(Thing, Rect)> = fp.iter().copied().chain(extras.iter().enumerate().map(|(k, &(at, rot))| (Thing::Extra(k), bedroll(at, rot)))).collect();
    for (at, rot, bed, own) in places {
        let taken: Vec<Rect> = all.iter().filter(|t| t.0 != own).map(|t| t.1).collect();
        if let Some(mut c) = fit_corner(d, at, rot, bed, &taken, &clear) {
            c.on = Some(own);
            return Some(c);
        }
    }
    // A fresh bedroll for the lodger, where a whole corner fits round it:
    // with room round it if there's any, else squeezed in.
    let taken: Vec<Rect> = all.iter().map(|t| t.1).collect();
    let grid = floor_grid(d);
    for gap in [FRESH_GAP, CORNER_GAP] {
        for &(p, along, across) in &grid {
            for rot in [along, across] {
                if !free(d, &bedroll(p, rot), gap, &taken, &clear) {
                    continue;
                }
                if let Some(c) = fit_corner(d, p, rot, false, &taken, &clear) {
                    return Some(c);
                }
            }
        }
    }
    None
}


/// Everything a building's sleepers need laid out: the extra bedrolls for
/// `residents` beyond its own beds, and (if a Ṭaḍoro lodges there,
/// `lodger`) their corner. The corner goes round a bed or bedroll if a whole
/// one fits among the bedrolls; otherwise it's laid first (round one of the
/// building's own beds, or a fresh bedroll that then counts as one of the
/// extras) and the extras are laid round it.
pub fn sleeping_plan(d: &Door, residents: usize, lodger: bool) -> (Vec<(V2, f32)>, Option<Corner>) {
    let n = extras_wanted(d, residents);
    let extras = extra_sleepers(d, n);
    if !lodger {
        return (extras, None);
    }
    if let Some(c) = lodger_corner(d, &extras) {
        return (extras, Some(c));
    }
    match lodger_corner(d, &[]) {
        Some(c) => {
            let mut avoid: Vec<Rect> = c.pieces().into_iter().map(|p| p.1).collect();
            avoid.push(c.bed_rect());
            let n = if c.on.is_none() { n.saturating_sub(1) } else { n };
            (extras_avoiding(d, n, &avoid), Some(c))
        }
        None => (extras, None),
    }
}

// ---- Loose belongings ----------------------------------------------------------

/// The floor a thing lying loose takes, metres a side: room for it at any turn.
pub const LOOSE: f32 = 0.66;

/// Can things be set on this piece's top?
pub fn holds_things(f: Furn) -> bool {
    matches!(f, Furn::Table | Furn::LongTable | Furn::Shelf | Furn::Counter)
}

/// Where one of a building's loose belongings lies: a spot on the floor, or
/// on top of the variant's `furniture[k]` (`on`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loose {
    pub at: V2,
    pub on: Option<usize>,
}

/// The spots on a piece's top where a thing can be set: near each end,
/// clear of what's drawn on it already.
fn top_spots(d: &Door, p: &Piece) -> Vec<V2> {
    let (w, dd, _) = p.what.size();
    let r = Rect::new(d.to_world(p.at), d.rot + p.rot, w, dd);
    let x = r.hw - 0.2;
    if x <= 0.0 {
        return vec![r.c];
    }
    vec![r.at(-x, 0.0), r.at(x, 0.0)]
}

/// What a thing lying at `p` in a building rests on: the height of the top
/// of the piece of furniture it's on (over the floor) and the turn it lies
/// at there, or None on the floor. Drawing only.
pub fn resting(d: &Door, p: V2) -> Option<(f32, f32)> {
    d.variant().furniture.iter().filter(|f| holds_things(f.what)).find_map(|f| {
        let (w, dd, h) = f.what.size();
        let r = Rect::new(d.to_world(f.at), d.rot + f.rot, w, dd);
        let [u, v] = r.axes();
        let q = p.sub(r.c);
        let inside = dot(q, u).abs() <= r.hw && dot(q, v).abs() <= r.hd;
        // Along a shelf; across a table or counter (clear of the bowls).
        inside.then(|| (h, if f.what == Furn::Shelf { r.rot } else { r.rot + std::f32::consts::FRAC_PI_2 }))
    })
}

/// Where a building's `n` loose belongings lie, given who sleeps there
/// (`sleeping_plan`): each on the shelves or a table's top (about half), or on
/// free floor clear of the furniture, containers, inner walls, the beds laid
/// out, a lodger's corner, the way in, the door's inside point, the doorways
/// and each other. The spots come from a roll keyed to `seed` alone.
pub fn loose_spots(d: &Door, residents: usize, lodger: bool, n: usize, seed: u64) -> Vec<Loose> {
    let (extras, corner) = sleeping_plan(d, residents, lodger);
    let mut taken: Vec<Rect> = footprints(d).into_iter().map(|f| f.1).collect();
    taken.extend(extras.iter().map(|&(at, rot)| bedroll(at, rot)));
    if let Some(c) = &corner {
        taken.extend(c.pieces().into_iter().map(|p| p.1));
        taken.push(c.bed_rect());
    }
    let mut clear = keep_clear(d);
    clear.push(Rect::new(d.inside, d.rot, 1.0, 1.0));
    clear.extend(d.doorways().into_iter().map(|g| Rect::new(g, d.rot, 1.0, 1.0)));
    let mut tops: Vec<Loose> = d
        .variant()
        .furniture
        .iter()
        .enumerate()
        .filter(|(_, p)| holds_things(p.what))
        .flat_map(|(k, p)| top_spots(d, p).into_iter().map(move |at| Loose { at, on: Some(k) }))
        .collect();
    let grid: Vec<V2> = floor_grid(d).into_iter().map(|g| g.0).collect();
    let mut r = Rng::from_keys(&[seed, 0x4C4F_4F53]);
    let mut out: Vec<Loose> = Vec::new();
    for _ in 0..n {
        let on_top = !tops.is_empty() && r.chance(0.5);
        if on_top {
            out.push(tops.swap_remove(r.below(tops.len())));
            continue;
        }
        let mut found = None;
        for m in [0.15f32, 0.05] {
            let ok: Vec<V2> = grid.iter().copied().filter(|&p| free(d, &Rect::new(p, d.rot, LOOSE, LOOSE), m, &taken, &clear)).collect();
            if !ok.is_empty() {
                found = Some(ok[r.below(ok.len())]);
                break;
            }
        }
        match found {
            Some(p) => {
                taken.push(Rect::new(p, d.rot, LOOSE, LOOSE));
                out.push(Loose { at: p, on: None });
            }
            // No floor left: on a top if any is free, else beside the last.
            None if !tops.is_empty() => out.push(tops.swap_remove(r.below(tops.len()))),
            None => out.push(out.last().copied().unwrap_or(Loose { at: d.centre, on: None })),
        }
    }
    out
}
