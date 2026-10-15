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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Grown stone: an ellipse.
    Round,
    /// Quarried: a box.
    Rect,
    /// An outline of its own, any shape (grown stone in lobes): corners in
    /// layout units, counter-clockwise, reaching ±1 at its widest each way.
    /// Never crosses itself; may curve in.
    Poly(&'static [(f32, f32)]),
}

impl Shape {
    /// An outline's corners, if it has its own.
    pub fn points(self) -> Option<&'static [(f32, f32)]> {
        match self {
            Shape::Poly(p) => Some(p),
            _ => None,
        }
    }
}

// ---- An outline of its own (`Shape::Poly`) ---------------------------------------
//
// Worked in metres in the building's own frame: `x` along its facing, `y`
// across, the corners scaled by the half depth `hx` and half width `hy`.

/// Corner `k` of an outline in metres.
fn corner(pts: &[(f32, f32)], hx: f32, hy: f32, k: usize) -> V2 {
    let (x, y) = pts[k % pts.len()];
    V2::new(x * hx, y * hy)
}

/// Is `p` (metres, the building's frame) inside the outline?
pub fn inside_outline(pts: &[(f32, f32)], hx: f32, hy: f32, p: V2) -> bool {
    let n = pts.len();
    let mut inside = false;
    for k in 0..n {
        let (a, b) = (corner(pts, hx, hy, k), corner(pts, hx, hy, k + 1));
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            inside = !inside;
        }
    }
    inside
}

/// How far `p` (metres, the building's frame) is inside the outline
/// (negative: outside), the nearest spot on it, and the way in from there
/// (straight in from that edge).
pub fn outline_nearest(pts: &[(f32, f32)], hx: f32, hy: f32, p: V2) -> (f32, V2, V2) {
    let mut best = (f32::MAX, p, V2::new(1.0, 0.0));
    for k in 0..pts.len() {
        let (a, b) = (corner(pts, hx, hy, k), corner(pts, hx, hy, k + 1));
        let q = on_seg(p, a, b);
        let d = q.dist(p);
        if d < best.0 {
            let e = b.sub(a);
            let l = e.len().max(1e-6);
            best = (d, q, V2::new(-e.y / l, e.x / l));
        }
    }
    let sign = if inside_outline(pts, hx, hy, p) { 1.0 } else { -1.0 };
    (best.0 * sign, best.1, best.2)
}

/// The farthest an outline reaches from the building's middle, metres.
pub fn outline_reach(pts: &[(f32, f32)], hx: f32, hy: f32) -> f32 {
    (0..pts.len()).map(|k| corner(pts, hx, hy, k).len()).fold(0.0, f32::max)
}

/// Where the line across the building at `y` metres leaves the outline at
/// the front: the largest `x` it crosses at (metres).
pub fn outline_front(pts: &[(f32, f32)], hx: f32, hy: f32, y: f32) -> f32 {
    let mut front = f32::MIN;
    for k in 0..pts.len() {
        let (a, b) = (corner(pts, hx, hy, k), corner(pts, hx, hy, k + 1));
        if (a.y > y) != (b.y > y) {
            front = front.max(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
        }
    }
    if front == f32::MIN { hx } else { front }
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

/// A piece of furniture, drawn as blocks. It changes nothing, except that a
/// forge, anvil, loom, workbench or grower's bed is that work station for
/// anyone standing in the building (`crafting::Station::furniture`).
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

/// A work yard grown against a building's wall (Laz, 15 Oct): a fence
/// round a patch of ground beside the house, one side of it the house's
/// own wall, a gate in it, and the trade's stations standing in the open
/// inside. Layout units in the building's frame (so they run past ±1).
#[derive(Clone, Copy, Debug)]
pub struct Yard {
    /// Its outline, counter-clockwise; where it meets the house it runs
    /// just inside the house's wall.
    pub outline: &'static [(f32, f32)],
    /// The gate: a spot on the outline, and how wide, metres.
    pub gate: (f32, f32),
    pub gate_width: f32,
    /// How tall the fence stands, metres.
    pub fence: f32,
    /// What stands in it: a forge, an anvil, a bench, racks. The ones that
    /// are crafting stations (`crafting::Station::of_furniture`) count as
    /// the trade's, out in the open.
    pub pieces: &'static [Piece],
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
    /// A work yard against one wall, for a trade home.
    pub yard: Option<&'static Yard>,
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
        yard: None,
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
        // Split down the middle: the hall with the hearth (a bench by it, the
        // cupboard under the shelf), and the sleeping room (two beds with
        // their heads to the far wall, a chest between; one bed stands a
        // little further in, so there's a way round its foot to the chest).
        // Everything can be got to at every size (`tests/buildings.rs`).
        walls: &[w((1.0, 0.05), (-1.0, 0.05), Some(0.2))],
        furniture: &[
            f(Hearth, -0.05, -0.48, 0.0),
            f(Table, -0.49, -0.4, Q),
            f(Bench, 0.19, -0.61, Q),
            f(Bed, -0.49, 0.35, Q),
            f(Bed, 0.26, 0.51, Q),
            f(Shelf, -0.42, -0.04, 0.0),
        ],
        holders: &[h(Cupboard, -0.57, -0.15, 0.0), h(Chest, -0.13, 0.72, 0.0), h(Barrel, 0.44, -0.6, 0.0)],
        yard: None,
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
        yard: None,
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
        yard: None,
    },
    Variant {
        key: "roduro_loomroom",
        name: "Weaver's house",
        kind: K::RoduroHome,
        shape: Shape::Round,
        use_: Use::Workshop,
        half: (0.5, 0.5),
        sizes: (8.0, 13.0),
        weight: 2,
        door: -0.2,
        storeys: 1,
        // Two looms and a cutting bench in front; the living and sleeping
        // room behind a wall at the back.
        walls: &[w((-0.25, -1.0), (-0.25, 1.0), Some(0.7))],
        furniture: &[
            f(Loom, 0.1, -0.6, 0.0),
            f(Loom, 0.31, -0.29, Q),
            f(Table, 0.31, 0.54, Q),
            f(Bed, -0.57, 0.3, Q),
            f(Hearth, -0.57, -0.42, 0.0),
            f(Shelf, -0.81, -0.03, Q),
        ],
        holders: &[h(Crate, 0.6, 0.17, 0.0), h(Cupboard, -0.05, 0.75, 0.0), h(Chest, -0.57, -0.05, 0.0), h(Barrel, -0.4, -0.65, 0.0)],
        yard: None,
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
            f(Workbench, 0.3, -0.55, 0.0),
            f(Workbench, 0.12, 0.6, 0.0),
            f(Rack, 0.29, 0.36, 0.4),
            f(Rack, 0.12, -0.23, 0.0),
            f(Bed, -0.57, 0.3, Q),
            f(Hearth, -0.57, -0.42, 0.0),
        ],
        holders: &[h(Crate, 0.55, -0.1, 0.0), h(Crate, 0.25, 0.15, 0.3), h(Chest, -0.56, -0.22, 0.0), h(Barrel, -0.74, -0.06, 0.0)],
        yard: None,
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
        holders: &[h(Cupboard, 0.51, 0.25, Q), h(Crate, 0.57, 0.5, 0.0), h(Chest, -0.74, -0.25, Q)],
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        yard: None,
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
        // A communal hall, with a small shrine room and a store at the back,
        // each with its doorway onto the hall (the shrine's facing the altar).
        walls: &[w((-0.45, -1.0), (-0.45, 0.1), Some(0.45)), w((-0.45, 0.1), (-0.45, 1.0), Some(0.5)), w((-0.45, 0.1), (-1.0, 0.1), None)],
        furniture: &[f(LongTable, 0.3, 0.0, 0.0), f(Bench, 0.3, 0.25, 0.0), f(Bench, 0.3, -0.25, 0.0), f(Hearth, -0.2, 0.6, 0.0), f(Altar, -0.8, 0.55, Q)],
        holders: &[h(Chest, -0.8, -0.35, Q), h(Barrel, -0.6, -0.8, 0.0), h(Cupboard, 0.6, -0.85, 0.0)],
        yard: None,
    },
    // ---- Horaro: stilt homes (look only, for now) ------------------------------
    Variant { key: "horaro_dome", name: "Woven dome", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.5, 0.5), sizes: (0.0, 99.0), weight: 4, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[], yard: None },
    Variant { key: "horaro_twin", name: "Twin domes", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.36, 0.5), sizes: (8.0, 99.0), weight: 2, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[], yard: None },
    Variant { key: "horaro_hull", name: "Hull house", kind: K::HoraroStilt, shape: Shape::Round, use_: Use::Home, half: (0.5, 0.32), sizes: (0.0, 99.0), weight: 2, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[], yard: None },
    Variant { key: "horaro_net_loft", name: "Net loft", kind: K::HoraroStilt, shape: Shape::Rect, use_: Use::Workshop, half: (0.42, 0.34), sizes: (0.0, 99.0), weight: 1, door: 0.0, storeys: 1, walls: &[], furniture: &[], holders: &[], yard: None },
    // ---- Placed by hand: the catalogue's own buildings (`claude/placeholder-shapes.md`) ----
    // Weight 0: never rolled for a town; a place file (or the showcase) names
    // them. Appended to the list, so saved doors' variant numbers keep their
    // meaning. Written by `tools/buildings/roduro_forms.py rust` from
    // `assets/buildings/*.json`; edit the files, not these.
    // ---- generated: catalogue buildings (tools/buildings/roduro_forms.py rust) ----
    Variant {
        key: "roduro_hill_est_01",
        name: "Hillside home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_HILL_EST_01_OUTLINE),
        use_: Use::Home,
        half: (0.3424, 0.5),
        sizes: (11.9, 12.4),
        weight: 0,
        door: -0.0241,
        storeys: 1,
        walls: &[w((-0.1846, 0.9934), (-0.1433, 0.4662), None), w((0.9894, -0.3134), (-0.7913, -0.3786), Some(0.5549)), w((0.9098, 0.5047), (-0.1466, 0.4662), Some(0.6224)), w((-0.1466, 0.4662), (-0.9254, 0.4372), Some(0.3921))],
        furniture: &[
            f(Bed, 0.0959, -0.771, -2.8351),
            f(Hearth, -0.0353, 0.2426, 0.0),
            f(Table, 0.0849, -0.0454, 0.0),
            f(Bench, -0.0353, 0.078, 0.0536),
            f(Shelf, -0.6363, 0.0369, 1.8482),
        ],
        holders: &[
            h(Chest, -0.505, -0.7299, 2.1764),
            h(Cupboard, 0.7459, 0.2015, -1.7226),
            h(Chest, 0.542, 0.7131, -0.7759),
            h(Crate, -0.5916, 0.7705, 0.6569),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_hill_great_01",
        name: "Hillside great house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_HILL_GREAT_01_OUTLINE),
        use_: Use::Home,
        half: (0.4282, 0.5),
        sizes: (13.33, 13.83),
        weight: 0,
        door: 0.2393,
        storeys: 2,
        walls: &[w((-0.2038, -0.2209), (0.0279, -0.9841), None), w((0.9943, -0.0499), (-0.2038, -0.2209), Some(0.6555)), w((-0.2038, -0.2209), (-0.9967, -0.4424), Some(0.3492))],
        furniture: &[
            f(Bed, -0.577, 0.7445, -1.3165),
            f(Bed, -0.8349, -0.1024, Q),
            f(Hearth, -0.7919, 0.3395, 0.0),
            f(Bench, -0.8779, 0.5604, 1.5687),
            f(Hearth, 0.1831, -0.8099, -2.8824),
            f(Table, 0.3551, -0.4417, 0.0),
            f(Shelf, 0.828, -0.3312, -1.8644),
        ],
        holders: &[
            h(Barrel, 0.5701, -0.6626, -2.3794),
            h(Barrel, 0.0112, -0.368, 1.8252),
            h(Crate, -0.534, -0.8586, 3.0704),
            h(Crate, -0.8349, -0.6377, Q),
            h(Chest, -0.233, -0.8586, 3.1416),
            h(Barrel, -0.276, -0.5641, -1.3164),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_hill_old_01",
        name: "Hillside house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_HILL_OLD_01_OUTLINE),
        use_: Use::Home,
        half: (0.3342, 0.5),
        sizes: (15.1, 15.6),
        weight: 0,
        door: 0.072,
        storeys: 2,
        walls: &[w((-0.1172, 0.997), (-0.1068, 0.2146), None), w((0.9622, 0.2208), (-0.1086, 0.2146), Some(0.6098)), w((-0.1086, 0.2146), (-0.9329, 0.2098), Some(0.5715))],
        furniture: &[
            f(Bed, -0.6659, -0.3566, 1.4681),
            f(Bed, 0.455, -0.6498, -2.4082),
            f(Bed, 0.6499, -0.2915, -1.689),
            f(Hearth, -0.4222, -0.6498, 0.0),
            f(LongTable, -0.0324, -0.2915, 0.0),
            f(Bench, 0.1138, -0.1937, 0.0088),
            f(Bench, 0.26, -0.096, 0.0088),
            f(Shelf, -0.276, -0.8452, 3.1184),
            f(Hearth, 0.6641, 0.5403, -1.1376),
            f(Table, 0.2743, 0.5078, Q),
            f(Shelf, 0.4205, 0.7683, -0.549),
        ],
        holders: &[
            h(Cupboard, -0.7146, -0.0309, 1.8558),
            h(Crate, -0.7573, 0.6014, 1.2429),
            h(Crate, -0.5136, 0.7643, 0.5651),
            h(Chest, -0.3187, 0.5689, -1.562),
            h(Barrel, -0.3674, 0.3408, -3.1328),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_hill_young_01",
        name: "Hillside home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_HILL_YOUNG_01_OUTLINE),
        use_: Use::Home,
        half: (0.4342, 0.5),
        sizes: (7.81, 8.31),
        weight: 0,
        door: 0.4352,
        storeys: 1,
        walls: &[w((0.1334, -0.373), (0.4003, -0.9181), None), w((-0.8581, 0.096), (0.1486, 0.468), Some(0.728)), w((-0.4838, -0.6007), (0.1314, -0.3737), None), w((0.1314, -0.3737), (0.9921, -0.0558), Some(0.3589))],
        furniture: &[
            f(Hearth, -0.0571, -0.0521, -2.7394),
        ],
        holders: &[
            h(Chest, 0.1777, 0.7127, -0.5283),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_ancient_01",
        name: "Kidney great house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_ANCIENT_01_OUTLINE),
        use_: Use::Home,
        half: (0.3764, 0.5),
        sizes: (19.21, 19.71),
        weight: 0,
        door: -0.4526,
        storeys: 3,
        walls: &[w((0.569, -0.9238), (0.216, -0.2718), None), w((0.9005, -0.0618), (0.2132, -0.2727), Some(0.6401)), w((0.2132, -0.2727), (-0.5125, -0.4954), Some(0.3884))],
        furniture: &[
            f(Hearth, -0.2048, -0.6843, 2.2345),
            f(Table, 0.0683, -0.6072, Q),
            f(Shelf, 0.2048, -0.8898, 3.0759),
            f(Bed, -0.7542, 0.1937, 1.8755),
            f(Bed, 0.6452, 0.091, -1.1702),
            f(Bed, -0.6176, -0.0889, 2.0405),
            f(Hearth, 0.338, 0.4764, 0.0),
            f(LongTable, 0.065, 0.3222, -1.006),
            f(Bench, 0.1332, 0.3993, -0.9283),
            f(Bench, 0.0309, 0.6048, -0.4972),
            f(Shelf, -0.7542, 0.6305, 1.0348),
        ],
        holders: &[
            h(Barrel, 0.3072, -0.71, -1.1837),
            h(Crate, 0.7962, -0.4716, -1.5279),
            h(Crate, 0.7279, -0.6515, -2.0837),
            h(Chest, 0.7962, -0.2404, -1.528),
            h(Barrel, 0.4549, -0.4459, 1.9579),
            h(Cupboard, 0.1332, 0.7847, -0.5049),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_est_01",
        name: "Long vault",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_EST_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.3064),
        sizes: (13.13, 13.63),
        weight: 0,
        door: 0.1195,
        storeys: 1,
        walls: &[w((-0.2118, 0.822), (-0.1178, -0.865), Some(0.5033))],
        furniture: &[
            f(Hearth, 0.2451, -0.3139, 0.0),
            f(Table, 0.3199, 0.1129, 0.0),
            f(Bench, 0.2078, 0.2959, -0.0222),
            f(Shelf, 0.0957, 0.6618, 0.195),
            f(Bed, -0.5631, 0.4869, 0.2533),
            f(Bed, -0.7499, -0.1229, 1.5279),
            f(Shelf, -0.2641, -0.4278, -1.48),
        ],
        holders: &[
            h(Cupboard, 0.5815, 0.7227, -0.1529),
            h(Chest, -0.6378, -0.6718, 2.776),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_est_02",
        name: "Long cottage",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_EST_02_OUTLINE),
        use_: Use::Home,
        half: (0.3412, 0.5),
        sizes: (11.65, 12.15),
        weight: 0,
        door: 0.175,
        storeys: 1,
        walls: &[w((0.0308, 0.9852), (-0.033, 0.4255), None), w((0.932, -0.4403), (-0.9746, -0.3387), Some(0.519)), w((0.982, 0.3713), (-0.0365, 0.4257), Some(0.6277)), w((-0.0365, 0.4257), (-0.8726, 0.4703), Some(0.3636))],
        furniture: &[
            f(Bed, -0.1887, -0.7965, 2.9626),
            f(Hearth, 0.4795, -0.0955, 0.0),
            f(Table, -0.0131, 0.0306, 0.0),
            f(Bench, 0.1101, 0.1566, -0.078),
            f(Shelf, -0.7519, 0.1987, 1.3921),
        ],
        holders: &[
            h(Chest, 0.4886, -0.7124, -2.2406),
            h(Cupboard, -0.6903, -0.1795, 3.1972),
            h(Chest, 0.5827, 0.7146, -0.8177),
            h(Crate, -0.4455, 0.7617, 0.767),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_hall_01",
        name: "Long hall",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_HALL_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.26),
        sizes: (19.75, 20.25),
        weight: 0,
        door: 0.0,
        storeys: 1,
        walls: &[w((-0.6, 0.9148), (-0.6, -0.9148), Some(0.5))],
        furniture: &[
            f(Bed, 0.45, -0.7673, -2.9958),
            f(Bed, -0.425, 0.7712, 0.1175),
            f(Bed, 0.425, 0.7712, -0.1175),
            f(Hearth, -0.35, -0.6231, 0.0),
            f(LongTable, 0.125, 0.0019, 0.0),
            f(Bench, 0.175, 0.1462, -0.0208),
            f(Bench, 0.025, 0.2904, -0.0063),
            f(Shelf, -0.15, 0.8673, 0.0153),
            f(Hearth, -0.7289, -0.6348, 2.6748),
            f(Table, -0.7789, -0.0098, Q),
            f(Shelf, -0.9289, -0.0098, 1.581),
        ],
        holders: &[
            h(Cupboard, 0.75, -0.6712, -2.6748),
            h(Barrel, -0.8789, -0.3944, 1.9101),
            h(Barrel, -0.8789, 0.3748, 1.2315),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_old_01",
        name: "Great house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_OLD_01_OUTLINE),
        use_: Use::Home,
        half: (0.3619, 0.5),
        sizes: (16.11, 16.61),
        weight: 0,
        door: 0.1226,
        storeys: 2,
        walls: &[w((-0.3826, -0.9688), (-0.0306, -0.1683), None), w((-0.8994, 0.0318), (-0.0375, -0.1666), Some(0.6442)), w((-0.0375, -0.1666), (0.6927, -0.3348), Some(0.3642))],
        furniture: &[
            f(Bed, 0.7099, 0.5165, -1.2134),
            f(Bed, -0.5146, 0.3943, 1.0235),
            f(Bed, -0.1346, 0.761, 0.6169),
            f(Hearth, 0.4988, -0.0336, -2.2376),
            f(LongTable, 0.1187, 0.4248, 0.0),
            f(Bench, 0.161, 0.5471, -0.229),
            f(Bench, 0.2454, 0.6388, -0.3819),
            f(Shelf, 0.4143, 0.8221, -0.4602),
            f(Hearth, -0.5359, -0.7427, 2.2648),
            f(Table, -0.4937, -0.4065, 0.0),
            f(Shelf, -0.747, -0.1925, 1.5757),
        ],
        holders: &[
            h(Cupboard, -0.6413, 0.1498, 2.5658),
            h(Crate, 0.2831, -0.6295, -2.2526),
            h(Crate, 0.0297, -0.7823, -2.5388),
            h(Chest, 0.452, -0.4461, -2.045),
            h(Barrel, -0.0125, -0.5378, 1.2627),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_long_young_01",
        name: "Small vault",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_LONG_YOUNG_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.4633),
        sizes: (7.24, 7.74),
        weight: 0,
        door: 0.1412,
        storeys: 1,
        walls: &[w((-0.3463, 0.9011), (0.0126, -0.9155), Some(0.5068))],
        furniture: &[
            f(Hearth, 0.4224, -0.2386, 0.0),
            f(Table, 0.2222, 0.4098, Q),
            f(Bed, -0.6152, -0.3519, 1.7967),
        ],
        holders: &[
            h(Chest, -0.6152, 0.5127, 0.6718),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_ancient_01",
        name: "Clover tower house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_ANCIENT_01_OUTLINE),
        use_: Use::Home,
        half: (0.424, 0.5),
        sizes: (17.59, 18.09),
        weight: 0,
        door: 0.1787,
        storeys: 3,
        walls: &[w((0.0643, -0.2697), (0.353, -0.9904), None), w((0.9307, -0.0202), (0.0674, -0.2688), Some(0.6727)), w((0.0674, -0.2688), (-0.7976, -0.518), Some(0.3229))],
        furniture: &[
            f(Hearth, -0.4105, 0.1581, 0.0),
            f(LongTable, 0.3298, 0.3823, 0.0),
            f(Bench, 0.3298, 0.2982, 0.0),
            f(Bench, 0.3298, 0.4664, 0.0),
            f(Bed, -0.3179, 0.7859, 0.0),
            f(Bed, 0.0654, 0.7186, 0.0),
            f(Bed, -0.6749, 0.3711, Q),
            f(Shelf, -0.8468, -0.0998, Q),
            f(Hearth, 0.5414, -0.6267, 0.0),
            f(Table, 0.4092, -0.4025, 0.0),
            f(Shelf, 0.7265, -0.4473, Q),
        ],
        holders: &[
            h(Cupboard, -0.6088, 0.6289, 0.6),
            h(Chest, -0.3179, 0.6457, 0.0),
            h(Barrel, 0.4224, -0.7836, 0.0),
            h(Barrel, 0.6207, -0.2567, 0.0),
            h(Crate, -0.1857, -0.6603, 0.0),
            h(Crate, 0.0258, -0.7724, 0.3),
            h(Chest, -0.503, -0.5482, 0.33),
            h(Barrel, -0.3179, -0.5426, 0.0),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_est_01",
        name: "Round home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_EST_01_OUTLINE),
        use_: Use::Home,
        half: (0.4825, 0.5),
        sizes: (10.05, 10.55),
        weight: 0,
        door: -0.1675,
        storeys: 1,
        walls: &[w((-0.0258, 0.1792), (-0.4527, 0.8977), None), w((0.6692, 0.5637), (-0.0173, 0.1839), Some(0.6402)), w((-0.0173, 0.1839), (-0.866, -0.2854), Some(0.3781))],
        furniture: &[
            f(Hearth, -0.3151, -0.4159, 0.0),
            f(Table, 0.1879, -0.2703, 0.0),
            f(Bench, 0.5401, -0.3188, 0.0503),
            f(Shelf, -0.2145, -0.7557, 2.7414),
            f(Bed, -0.659, 0.4, 1.0073),
        ],
        holders: &[
            h(Cupboard, 0.691, 0.2637, -1.0404),
            h(Crate, 0.2026, 0.7617, -0.556),
            h(Barrel, -0.1495, 0.8103, 0.1843),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_est_02",
        name: "Round home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_EST_02_OUTLINE),
        use_: Use::Home,
        half: (0.4655, 0.5),
        sizes: (9.32, 9.82),
        weight: 0,
        door: 0.035,
        storeys: 2,
        walls: &[w((-0.1425, -0.1427), (-0.4898, -0.8617), None), w((-0.9828, 0.209), (-0.1477, -0.1406), Some(0.6728)), w((-0.1477, -0.1406), (0.7875, -0.5325), Some(0.3665))],
        furniture: &[
            f(Bed, 0.4469, 0.6096, -0.5806),
            f(Bed, -0.3387, 0.6096, 0.5464),
            f(Bench, -0.2265, 0.4529, 0.5312),
            f(Shelf, 0.6714, -0.1741, -0.1599),
            f(Hearth, -0.0348, -0.6974, -3.074),
        ],
        holders: &[
            h(Barrel, 0.358, -0.6451, -2.6555),
            h(Crate, -0.6903, -0.3152, 1.9253),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_old_01",
        name: "Clover drum",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_OLD_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.4949),
        sizes: (13.56, 14.06),
        weight: 0,
        door: 0.0715,
        storeys: 2,
        walls: &[w((-0.1085, 0.207), (-0.6627, 0.8596), None), w((0.5586, 0.785), (-0.1117, 0.2041), Some(0.6642)), w((-0.1117, 0.2041), (-0.7635, -0.3608), Some(0.3747))],
        furniture: &[
            f(Bed, 0.7034, 0.4149, -0.9134),
            f(Bed, -0.4552, -0.5361, 2.1395),
            f(Bed, 0.5586, -0.3898, -2.3322),
            f(Hearth, 0.0879, -0.6458, 0.0),
            f(LongTable, 0.1965, -0.1703, 0.7091),
            f(Bench, 0.4138, 0.1589, 0.7091),
            f(Bench, 0.4138, 0.2686, 0.7091),
            f(Shelf, -0.2017, -0.8287, 2.7357),
            f(Hearth, -0.7445, -0.0206, 1.9718),
            f(Table, -0.5635, 0.2354, -2.4324),
            f(Shelf, -0.8531, 0.3086, 1.5087),
        ],
        holders: &[
            h(Cupboard, 0.7396, -0.0972, -0.277),
            h(Barrel, -0.7083, 0.6012, 0.9187),
            h(Crate, -0.0417, 0.8289, -0.2156),
            h(Crate, 0.1755, 0.7191, -0.3227),
            h(Chest, -0.3314, 0.8289, 0.299),
            h(Barrel, -0.1865, 0.5362, 2.2798),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_young_01",
        name: "Round home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_YOUNG_01_OUTLINE),
        use_: Use::Home,
        half: (0.4723, 0.5),
        sizes: (6.79, 7.29),
        weight: 0,
        door: -0.0778,
        storeys: 1,
        walls: &[w((0.292, 0.9707), (-0.031, 0.1767), None), w((0.2241, 0.0841), (-0.0265, 0.175), None), w((-0.0265, 0.175), (-0.8626, 0.479), Some(0.3657))],
        furniture: &[
            f(Hearth, -0.034, -0.2537, 0.0),
            f(Table, -0.5603, -0.2537, Q),
            f(Shelf, 0.342, -0.6088, -2.5863),
        ],
        holders: &[
            h(Chest, 0.6457, 0.3764, -1.0904),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_round_young_02",
        name: "Round cottage",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_ROUND_YOUNG_02_OUTLINE),
        use_: Use::Home,
        half: (0.4008, 0.5),
        sizes: (7.36, 7.86),
        weight: 0,
        door: 0.2983,
        storeys: 1,
        walls: &[w((0.0154, 0.4176), (-0.0705, 0.9905), None), w((0.9623, -0.241), (-0.9121, -0.4218), Some(0.4848)), w((0.8806, 0.5009), (0.5105, 0.4654), None), w((0.0062, 0.4168), (-0.9901, 0.3206), Some(0.3465))],
        furniture: &[
            f(Hearth, -0.4265, -0.0762, 1.6577),
        ],
        holders: &[
            h(Chest, -0.0784, -0.7356, 2.8249),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_smith_home_01",
        name: "Smith's home and yard",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_SMITH_HOME_01_OUTLINE),
        use_: Use::Workshop,
        half: (0.4825, 0.5),
        sizes: (10.05, 10.55),
        weight: 0,
        door: -0.1675,
        storeys: 1,
        walls: &[w((-0.0258, 0.1792), (-0.4527, 0.8977), None), w((0.6692, 0.5637), (-0.0173, 0.1839), Some(0.6402)), w((-0.0173, 0.1839), (-0.866, -0.2854), Some(0.3781))],
        furniture: &[
            f(Hearth, -0.3151, -0.4159, 0.0),
            f(Table, 0.1879, -0.2703, 0.0),
            f(Bench, 0.5401, -0.3188, 0.0503),
            f(Shelf, -0.2145, -0.7557, 2.7414),
            f(Bed, -0.659, 0.4, 1.0073),
        ],
        holders: &[
            h(Cupboard, 0.691, 0.2637, -1.0404),
            h(Crate, 0.2026, 0.7617, -0.556),
            h(Barrel, -0.1495, 0.8103, 0.1843),
        ],
        yard: Some(&RODURO_SMITH_HOME_01_YARD),
    },
    Variant {
        key: "roduro_tanner_home_01",
        name: "Tanner's home and yard",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_TANNER_HOME_01_OUTLINE),
        use_: Use::Workshop,
        half: (0.3412, 0.5),
        sizes: (11.65, 12.15),
        weight: 0,
        door: 0.175,
        storeys: 1,
        walls: &[w((0.0308, 0.9852), (-0.033, 0.4255), None), w((0.932, -0.4403), (-0.9746, -0.3387), Some(0.519)), w((0.982, 0.3713), (-0.0365, 0.4257), Some(0.6277)), w((-0.0365, 0.4257), (-0.8726, 0.4703), Some(0.3636))],
        furniture: &[
            f(Bed, -0.1887, -0.7965, 2.9626),
            f(Hearth, 0.4795, -0.0955, 0.0),
            f(Table, -0.0131, 0.0306, 0.0),
            f(Bench, 0.1101, 0.1566, -0.078),
            f(Shelf, -0.7519, 0.1987, 1.3921),
        ],
        holders: &[
            h(Chest, 0.4886, -0.7124, -2.2406),
            h(Cupboard, -0.6903, -0.1795, 3.1972),
            h(Chest, 0.5827, 0.7146, -0.8177),
            h(Crate, -0.4455, 0.7617, 0.767),
        ],
        yard: Some(&RODURO_TANNER_HOME_01_YARD),
    },
    Variant {
        key: "roduro_tower_cottage_01",
        name: "Tower cottage",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_TOWER_COTTAGE_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.4744),
        sizes: (7.55, 8.05),
        weight: 0,
        door: 0.1014,
        storeys: 2,
        walls: &[w((-0.359, 0.9518), (-0.359, -0.9518), Some(0.5))],
        furniture: &[
            f(Bed, 0.2179, -0.6729, -2.9722),
            f(Hearth, 0.0897, -0.1324, 0.0),
            f(Bench, 0.5385, -0.2, -0.2314),
        ],
        holders: &[
            h(Crate, -0.7538, -0.0143, 1.5819),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_tower_house_01",
        name: "Tower house",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_TOWER_HOUSE_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.4515),
        sizes: (13.04, 13.54),
        weight: 0,
        door: 0.3146,
        storeys: 3,
        walls: &[w((0.0444, -0.265), (0.2907, -0.9388), None), w((0.9628, 0.0205), (0.0444, -0.265), Some(0.6918)), w((0.0444, -0.265), (-0.7013, -0.5766), Some(0.3423))],
        furniture: &[
            f(Bed, -0.1959, 0.61, 1.9554),
            f(Bed, 0.2932, 0.8183, -0.0624),
            f(Hearth, 0.4436, 0.1517, 0.0),
            f(Table, 0.2932, 0.4433, 0.0),
            f(Bench, 0.1803, 0.5683, -0.0061),
            f(Bench, 0.6694, 0.4433, 0.3472),
            f(Hearth, 0.4206, -0.662, -2.4641),
            f(Table, 0.6087, -0.3703, Q),
        ],
        holders: &[
            h(Cupboard, 0.6694, 0.7349, -0.6579),
            h(Crate, -0.1093, -0.8398, 2.9984),
            h(Crate, -0.3726, -0.7148, 2.566),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_twin_home_01",
        name: "Twin home",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_TWIN_HOME_01_OUTLINE),
        use_: Use::Home,
        half: (0.4541, 0.5),
        sizes: (14.13, 14.63),
        weight: 0,
        door: 0.3634,
        storeys: 1,
        walls: &[w((-0.6236, 0.0805), (0.2524, -0.4179), Some(0.5709)), w((-0.4176, 0.7694), (0.2438, -0.4921), Some(0.2494))],
        furniture: &[
            f(Hearth, -0.0427, 0.6999, 0.0),
            f(LongTable, 0.3401, 0.3522, 0.0),
            f(Bench, 0.7613, 0.4565, 0.2311),
            f(Bench, 0.7613, 0.6303, -0.9948),
            f(Shelf, 0.3784, 0.8737, -0.2126),
            f(Bed, -0.7378, -0.6925, 2.234),
        ],
        holders: &[
            h(Cupboard, 0.5316, -0.2389, -2.7114),
            h(Chest, -0.6613, -0.1014, 0.6451),
            h(Crate, -0.4727, 0.2121, 1.6941),
            h(Chest, -0.243, 0.0382, 2.5819),
        ],
        yard: None,
    },
    Variant {
        key: "roduro_vault_tall_01",
        name: "Two-storey vault",
        kind: K::RoduroHome,
        shape: Shape::Poly(RODURO_VAULT_TALL_01_OUTLINE),
        use_: Use::Home,
        half: (0.5, 0.3707),
        sizes: (11.35, 11.85),
        weight: 0,
        door: 0.1163,
        storeys: 2,
        walls: &[w((-0.2586, 0.9846), (-0.2586, -0.9846), Some(0.5))],
        furniture: &[
            f(Hearth, 0.4741, 0.5023, 0.0),
            f(Table, 0.3017, -0.0209, 0.0),
            f(Bench, 0.6034, -0.1372, -0.0463),
            f(Shelf, 0.1724, -0.8349, -3.0962),
            f(Bed, -0.7914, -0.2426, 1.7488),
            f(Bed, -0.5759, 0.6295, 0.4674),
            f(Shelf, -0.4897, -0.2426, -1.5708),
        ],
        holders: &[
            h(Cupboard, 0.1724, 0.793, -0.0464),
            h(Chest, -0.5328, -0.7077, 2.8501),
        ],
        yard: None,
    },
    // ---- end generated ----
];

// ---- generated outlines (tools/buildings/roduro_forms.py rust) ----
/// `roduro_hill_est_01`'s ground tier (`assets/buildings/roduro_hill_est_01.json`), layout units, counter-clockwise.
pub const RODURO_HILL_EST_01_OUTLINE: &[(f32, f32)] = &[
    (-0.7884, -0.0617), (-0.8004, -0.1638), (-0.8149, -0.2807), (-0.7908, -0.4058), (-0.786, -0.5704), (-0.7668, -0.7909),
    (-0.6009, -0.9391), (-0.3582, -1.0), (-0.0937, -0.9786), (0.1466, -0.9391), (0.375, -0.8947), (0.5697, -0.8058),
    (0.7283, -0.6955), (0.8389, -0.5671), (0.9303, -0.4436), (0.9952, -0.3202), (0.9783, -0.1868), (0.9639, -0.0683),
    (0.9615, 0.0436), (1.0, 0.1654), (0.9952, 0.2971), (0.9423, 0.4255), (0.8894, 0.577), (0.8101, 0.7514),
    (0.6274, 0.8667), (0.399, 0.9309), (0.1538, 0.9687), (-0.0986, 0.9951), (-0.3654, 1.0), (-0.6298, 0.9539),
    (-0.8678, 0.865), (-1.0, 0.6922), (-0.9327, 0.4486), (-0.899, 0.2856), (-0.8509, 0.149), (-0.8125, 0.0387),
];

/// `roduro_hill_great_01`'s ground tier (`assets/buildings/roduro_hill_great_01.json`), layout units, counter-clockwise.
pub const RODURO_HILL_GREAT_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9948, 0.8954), (-0.9983, 0.7187), (-0.9983, 0.539), (-1.0, 0.3594), (-1.0, 0.1811), (-1.0, 0.0015),
    (-1.0, -0.1782), (-1.0, -0.3579), (-1.0, -0.5376), (-1.0, -0.7158), (-0.9983, -0.8954), (-0.8005, -0.9543),
    (-0.5959, -0.9882), (-0.3861, -1.0), (-0.1763, -1.0), (0.0318, -0.9867), (0.2365, -0.9514), (0.4325, -0.8881),
    (0.6114, -0.7968), (0.7627, -0.673), (0.8779, -0.5243), (0.9536, -0.3564), (0.9914, -0.1797), (1.0, -0.0015),
    (0.9914, 0.1782), (0.9553, 0.3549), (0.8796, 0.5214), (0.7644, 0.6716), (0.6148, 0.7953), (0.4359, 0.8881),
    (0.2399, 0.9499), (0.0353, 0.9867), (-0.1745, 1.0), (-0.3844, 1.0), (-0.5924, 0.9882), (-0.7988, 0.9558),
];

/// `roduro_hill_old_01`'s ground tier (`assets/buildings/roduro_hill_old_01.json`), layout units, counter-clockwise.
pub const RODURO_HILL_OLD_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9162, -0.0606), (-0.9103, -0.1713), (-0.9006, -0.2899), (-0.9298, -0.445), (-0.9318, -0.6339), (-0.8519, -0.8267),
    (-0.6335, -0.9244), (-0.386, -0.9857), (-0.1228, -1.0), (0.1306, -0.9557), (0.3606, -0.8906), (0.5556, -0.7928),
    (0.7329, -0.7003), (0.8791, -0.5896), (0.9591, -0.4528), (0.9942, -0.3147), (1.0, -0.1844), (0.9864, -0.0645),
    (0.9376, 0.0476), (0.961, 0.1635), (0.9727, 0.2938), (0.9396, 0.4293), (0.8908, 0.5831), (0.7856, 0.7368),
    (0.5945, 0.8358), (0.384, 0.9322), (0.1365, 0.9857), (-0.1248, 1.0), (-0.3918, 0.9896), (-0.6491, 0.9336),
    (-0.883, 0.8423), (-1.0, 0.6664), (-0.9961, 0.4632), (-0.9688, 0.2977), (-0.9181, 0.1583), (-0.9181, 0.0476),
];

/// `roduro_hill_young_01`'s ground tier (`assets/buildings/roduro_hill_young_01.json`), layout units, counter-clockwise.
pub const RODURO_HILL_YOUNG_01_OUTLINE: &[(f32, f32)] = &[
    (-0.7715, -0.062), (-0.6915, -0.1687), (-0.6229, -0.263), (-0.5429, -0.3449), (-0.5229, -0.4739), (-0.4829, -0.6253),
    (-0.3886, -0.7618), (-0.2657, -0.9305), (-0.08, -1.0), (0.1229, -1.0), (0.3143, -0.9578), (0.4943, -0.8859),
    (0.6544, -0.7866), (0.7944, -0.6749), (0.9172, -0.5459), (0.9915, -0.3945), (0.9915, -0.2283), (1.0001, -0.0769),
    (0.983, 0.072), (0.9315, 0.2109), (0.8429, 0.3325), (0.7744, 0.4566), (0.6944, 0.5831), (0.5829, 0.6973),
    (0.4486, 0.794), (0.2943, 0.8834), (0.1172, 0.9628), (-0.08, 1.0), (-0.2857, 0.9901), (-0.48, 0.9305),
    (-0.6486, 0.8288), (-0.8001, 0.7097), (-0.9315, 0.5757), (-1.0001, 0.4119), (-0.9001, 0.2134), (-0.8544, 0.0645),
];

/// `roduro_long_ancient_01`'s ground tier (`assets/buildings/roduro_long_ancient_01.json`), layout units, counter-clockwise.
pub const RODURO_LONG_ANCIENT_01_OUTLINE: &[(f32, f32)] = &[
    (-0.8089, -0.0565), (-0.7311, -0.149), (-0.6683, -0.2354), (-0.5877, -0.3083), (-0.5604, -0.4173), (-0.4976, -0.5262),
    (-0.4075, -0.6434), (-0.2942, -0.7975), (-0.1208, -0.9455), (0.1031, -1.0), (0.3379, -0.9866), (0.5604, -0.9301),
    (0.7638, -0.8386), (0.9277, -0.7122), (1.0, -0.5375), (0.9973, -0.3587), (0.9796, -0.2045), (0.9058, -0.0647),
    (0.8212, 0.0504), (0.757, 0.1511), (0.7243, 0.2539), (0.7051, 0.3741), (0.6437, 0.4902), (0.5522, 0.6033),
    (0.4389, 0.7246), (0.2983, 0.8664), (0.099, 0.9476), (-0.129, 1.0), (-0.3679, 0.9949), (-0.5905, 0.9291),
    (-0.7775, 0.8181), (-0.9017, 0.665), (-0.9932, 0.5128), (-1.0, 0.3422), (-0.9236, 0.1799), (-0.8895, 0.0534),
];

/// `roduro_long_est_01`'s ground tier (`assets/buildings/roduro_long_est_01.json`), layout units, counter-clockwise.
pub const RODURO_LONG_EST_01_OUTLINE: &[(f32, f32)] = &[
    (-0.991, -0.1342), (-0.9791, -0.4195), (-0.9268, -0.6952), (-0.8161, -0.922), (-0.6188, -1.0001), (-0.426, -0.9806),
    (-0.272, -0.9415), (-0.148, -0.883), (-0.0463, -0.8391), (0.0433, -0.8367), (0.139, -0.8464), (0.2451, -0.8537),
    (0.3662, -0.8464), (0.5247, -0.8513), (0.7339, -0.8318), (0.8999, -0.6781), (0.9731, -0.4171), (1.0, -0.1342),
    (0.9895, 0.1488), (0.9806, 0.4366), (0.9312, 0.7171), (0.8221, 0.9489), (0.6054, 1.0001), (0.4081, 0.9611),
    (0.2496, 0.8854), (0.136, 0.8464), (0.0419, 0.8147), (-0.0448, 0.8171), (-0.136, 0.8269), (-0.2362, 0.8269),
    (-0.3572, 0.8367), (-0.5277, 0.8684), (-0.7489, 0.8635), (-0.9088, 0.6976), (-0.9716, 0.4317), (-1.0, 0.1512),
];

/// `roduro_long_est_02`'s ground tier (`assets/buildings/roduro_long_est_02.json`), layout units, counter-clockwise.
pub const RODURO_LONG_EST_02_OUTLINE: &[(f32, f32)] = &[
    (-0.9999, -0.037), (-0.9876, -0.1597), (-0.9753, -0.2908), (-0.9901, -0.4571), (-0.9531, -0.637), (-0.8152, -0.7849),
    (-0.6157, -0.8975), (-0.3744, -0.9681), (-0.1158, -1.0), (0.1453, -0.9966), (0.4015, -0.958), (0.628, -0.8739),
    (0.7857, -0.7277), (0.894, -0.5765), (0.9433, -0.4202), (0.9556, -0.2756), (0.9802, -0.1529), (0.9999, -0.0353),
    (0.9876, 0.0824), (0.9778, 0.2), (0.9852, 0.3328), (0.9925, 0.4908), (0.9211, 0.642), (0.7857, 0.7748),
    (0.6059, 0.8891), (0.3817, 0.9563), (0.1379, 0.9798), (-0.1108, 1.0), (-0.362, 0.9832), (-0.5911, 0.9092),
    (-0.7413, 0.7613), (-0.8448, 0.6118), (-0.8817, 0.4521), (-0.9063, 0.3176), (-0.9433, 0.1983), (-0.9728, 0.0824),
];

/// `roduro_long_hall_01`'s ground tier (`assets/buildings/roduro_long_hall_01.json`), layout units, counter-clockwise.
pub const RODURO_LONG_HALL_01_OUTLINE: &[(f32, f32)] = &[
    (1.0, 0.0), (0.993, 0.2769), (0.945, 0.5385), (0.845, 0.7346), (0.716, 0.8596), (0.576, 0.9308),
    (0.433, 0.9712), (0.289, 0.9923), (0.145, 0.9981), (0.0, 1.0), (-0.145, 0.9981), (-0.289, 0.9923),
    (-0.433, 0.9712), (-0.576, 0.9308), (-0.716, 0.8596), (-0.845, 0.7346), (-0.945, 0.5385), (-0.993, 0.2769),
    (-1.0, 0.0), (-0.993, -0.2769), (-0.945, -0.5385), (-0.845, -0.7346), (-0.716, -0.8596), (-0.576, -0.9308),
    (-0.433, -0.9712), (-0.289, -0.9923), (-0.145, -0.9981), (-0.0, -1.0), (0.145, -0.9981), (0.289, -0.9923),
    (0.433, -0.9712), (0.576, -0.9308), (0.716, -0.8596), (0.845, -0.7346), (0.945, -0.5385), (0.993, -0.2769),
];

/// `roduro_long_old_01`'s ground tier (`assets/buildings/roduro_long_old_01.json`), layout units, counter-clockwise.
pub const RODURO_LONG_OLD_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9593, -0.0636), (-0.9847, -0.1932), (-0.9999, -0.3411), (-0.9864, -0.5037), (-0.9374, -0.6834), (-0.7921, -0.8252),
    (-0.5861, -0.9169), (-0.3581, -0.978), (-0.1182, -1.0), (0.1182, -0.9499), (0.3108, -0.8337), (0.4611, -0.7127),
    (0.5405, -0.5575), (0.608, -0.4401), (0.6857, -0.3484), (0.7584, -0.2567), (0.8259, -0.1614), (0.885, -0.0575),
    (0.8969, 0.055), (0.961, 0.1834), (0.9999, 0.3337), (0.9796, 0.4927), (0.9188, 0.6601), (0.7651, 0.7848),
    (0.5726, 0.8802), (0.3614, 0.967), (0.125, 1.0), (-0.1149, 0.9804), (-0.3192, 0.8667), (-0.4864, 0.7567),
    (-0.6148, 0.6369), (-0.7178, 0.5196), (-0.804, 0.4071), (-0.8513, 0.2861), (-0.8428, 0.1614), (-0.8901, 0.0538),
];

/// `roduro_long_young_01`'s ground tier (`assets/buildings/roduro_long_young_01.json`), layout units, counter-clockwise.
pub const RODURO_LONG_YOUNG_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9386, -0.0922), (-0.9306, -0.2709), (-0.9519, -0.4813), (-0.9119, -0.6916), (-0.7917, -0.853), (-0.6235, -0.9625),
    (-0.4339, -1.0), (-0.239, -0.9596), (-0.0788, -0.9337), (0.0708, -0.9135), (0.215, -0.8818), (0.3698, -0.8674),
    (0.53, -0.8242), (0.7303, -0.7954), (0.8825, -0.6714), (0.9653, -0.4899), (0.992, -0.2911), (1.0, -0.098),
    (0.9493, 0.0865), (0.9306, 0.268), (0.9386, 0.4726), (0.8825, 0.6657), (0.773, 0.8328), (0.6235, 0.9625),
    (0.4312, 1.0), (0.2417, 0.9827), (0.0734, 0.9481), (-0.0761, 0.8991), (-0.2256, 0.8905), (-0.3965, 0.9135),
    (-0.5728, 0.8761), (-0.7517, 0.8069), (-0.8932, 0.6714), (-0.9813, 0.4899), (-1.0, 0.2853), (-0.9466, 0.0865),
];

/// `roduro_round_ancient_01`'s ground tier (`assets/buildings/roduro_round_ancient_01.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_ANCIENT_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9909, -0.0661), (-1.0001, -0.2197), (-0.9353, -0.3632), (-0.8349, -0.4899), (-0.7079, -0.5953), (-0.5599, -0.6738),
    (-0.4039, -0.7321), (-0.2479, -0.7881), (-0.0879, -0.88), (0.1051, -0.9776), (0.3219, -1.0), (0.5308, -0.9507),
    (0.7159, -0.8554), (0.815, -0.6805), (0.8626, -0.5022), (0.8639, -0.3318), (0.8851, -0.1928), (0.9208, -0.0605),
    (0.9631, 0.0796), (1.0001, 0.2343), (0.971, 0.3913), (0.8732, 0.5247), (0.7159, 0.6121), (0.5506, 0.6704),
    (0.4052, 0.7388), (0.2598, 0.8206), (0.0972, 0.9182), (-0.0985, 1.0), (-0.31, 1.0), (-0.507, 0.9361),
    (-0.6617, 0.8139), (-0.7529, 0.6502), (-0.8084, 0.4899), (-0.8692, 0.3531), (-0.9155, 0.2164), (-0.9287, 0.0774),
];

/// `roduro_round_est_01`'s ground tier (`assets/buildings/roduro_round_est_01.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_EST_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9477, -0.0835), (-0.8954, -0.2427), (-0.8129, -0.3845), (-0.6982, -0.501), (-0.6157, -0.633), (-0.5131, -0.765),
    (-0.3682, -0.8466), (-0.2113, -0.9107), (-0.0483, -0.9767), (0.1308, -1.0), (0.3079, -0.9592), (0.4648, -0.8777),
    (0.6218, -0.8019), (0.7666, -0.701), (0.8854, -0.5709), (0.9638, -0.4155), (0.992, -0.2466), (1.0001, -0.0816),
    (0.9497, 0.0757), (0.8874, 0.2175), (0.8109, 0.3456), (0.7344, 0.468), (0.656, 0.5922), (0.5513, 0.701),
    (0.4507, 0.8485), (0.3059, 0.9534), (0.1308, 0.9922), (-0.0503, 1.0), (-0.2294, 0.9728), (-0.4065, 0.9243),
    (-0.5715, 0.8447), (-0.7163, 0.7301), (-0.8411, 0.5961), (-0.9357, 0.4388), (-1.0001, 0.268), (-0.998, 0.0874),
];

/// `roduro_round_est_02`'s ground tier (`assets/buildings/roduro_round_est_02.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_EST_02_OUTLINE: &[(f32, f32)] = &[
    (-0.9933, -0.0596), (-0.9551, -0.2205), (-0.899, -0.373), (-0.8339, -0.5277), (-0.7464, -0.6823), (-0.6072, -0.7973),
    (-0.4478, -0.8913), (-0.2772, -0.9666), (-0.0887, -1.0), (0.1021, -0.9896), (0.2862, -0.9436), (0.4523, -0.8683),
    (0.5937, -0.7576), (0.715, -0.6364), (0.8137, -0.5047), (0.9192, -0.3751), (0.9866, -0.2226), (0.9978, -0.0596),
    (0.9911, 0.1014), (1.0, 0.2685), (0.9664, 0.4357), (0.8878, 0.5946), (0.7666, 0.7283), (0.6027, 0.8119),
    (0.4344, 0.8746), (0.2772, 0.9561), (0.0999, 1.0), (-0.0842, 0.9937), (-0.2548, 0.931), (-0.4074, 0.8495),
    (-0.5511, 0.7659), (-0.6948, 0.6761), (-0.8205, 0.5611), (-0.9192, 0.4232), (-0.9798, 0.2665), (-1.0, 0.1034),
];

/// `roduro_round_old_01`'s ground tier (`assets/buildings/roduro_round_old_01.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_OLD_01_OUTLINE: &[(f32, f32)] = &[
    (-0.8972, -0.0944), (-0.819, -0.2392), (-0.7596, -0.3768), (-0.7089, -0.5231), (-0.6235, -0.6533), (-0.5105, -0.766),
    (-0.3845, -0.8669), (-0.2411, -0.9591), (-0.076, -1.0001), (0.092, -0.9796), (0.2397, -0.883), (0.3744, -0.8069),
    (0.4989, -0.7221), (0.6032, -0.6153), (0.6843, -0.4923), (0.7842, -0.3797), (0.8841, -0.2524), (0.9537, -0.0988),
    (0.9841, 0.071), (1.0, 0.2539), (0.945, 0.4265), (0.8465, 0.5772), (0.7031, 0.6855), (0.5641, 0.785),
    (0.4178, 0.8698), (0.2571, 0.9211), (0.0934, 0.9635), (-0.0789, 0.9986), (-0.26, 1.0001), (-0.4395, 0.9562),
    (-0.6206, 0.8933), (-0.7784, 0.7806), (-0.9088, 0.6343), (-1.0, 0.4602), (-0.9899, 0.2553), (-0.9667, 0.071),
];

/// `roduro_round_young_01`'s ground tier (`assets/buildings/roduro_round_young_01.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_YOUNG_01_OUTLINE: &[(f32, f32)] = &[
    (-1.0, -0.0881), (-0.9759, -0.25), (-0.9158, -0.4034), (-0.8316, -0.5483), (-0.7534, -0.7045), (-0.6271, -0.8352),
    (-0.4617, -0.9148), (-0.2842, -0.9602), (-0.1068, -0.9972), (0.0767, -1.0), (0.2511, -0.9375), (0.3985, -0.8409),
    (0.5429, -0.7585), (0.6722, -0.6562), (0.7774, -0.5284), (0.8647, -0.3949), (0.9218, -0.2443), (0.9429, -0.0852),
    (0.9639, 0.0739), (1.0, 0.25), (0.9489, 0.4176), (0.8586, 0.5682), (0.7383, 0.7017), (0.597, 0.8153),
    (0.4436, 0.9176), (0.2692, 0.9858), (0.0797, 1.0), (-0.1068, 0.9858), (-0.2842, 0.946), (-0.4677, 0.9148),
    (-0.6481, 0.8494), (-0.7865, 0.7244), (-0.8526, 0.5483), (-0.8917, 0.3778), (-0.9429, 0.2273), (-0.988, 0.0739),
];

/// `roduro_round_young_02`'s ground tier (`assets/buildings/roduro_round_young_02.json`), layout units, counter-clockwise.
pub const RODURO_ROUND_YOUNG_02_OUTLINE: &[(f32, f32)] = &[
    (-0.9213, -0.0775), (-0.9082, -0.2063), (-0.9278, -0.3561), (-0.9049, -0.5191), (-0.7901, -0.6426), (-0.6524, -0.7556),
    (-0.5016, -0.8712), (-0.3115, -0.9369), (-0.1082, -0.9658), (0.1049, -0.9974), (0.3279, -1.0), (0.5344, -0.9369),
    (0.7115, -0.8318), (0.8524, -0.6978), (0.9344, -0.5375), (0.9344, -0.364), (0.9737, -0.2221), (1.0, -0.0828),
    (0.9836, 0.0565), (0.9606, 0.1932), (0.9475, 0.343), (0.8918, 0.4901), (0.8262, 0.6531), (0.7213, 0.816),
    (0.5377, 0.9185), (0.3279, 0.979), (0.1082, 1.0), (-0.1148, 0.9947), (-0.3279, 0.9606), (-0.5311, 0.8922),
    (-0.7115, 0.7976), (-0.8426, 0.661), (-0.9344, 0.5112), (-1.0, 0.3614), (-0.9869, 0.1984), (-0.9377, 0.0539),
];

/// `roduro_smith_home_01`'s ground tier (`assets/buildings/roduro_smith_home_01.json`), layout units, counter-clockwise.
pub const RODURO_SMITH_HOME_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9477, -0.0835), (-0.8954, -0.2427), (-0.8129, -0.3845), (-0.6982, -0.501), (-0.6157, -0.633), (-0.5131, -0.765),
    (-0.3682, -0.8466), (-0.2113, -0.9107), (-0.0483, -0.9767), (0.1308, -1.0), (0.3079, -0.9592), (0.4648, -0.8777),
    (0.6218, -0.8019), (0.7666, -0.701), (0.8854, -0.5709), (0.9638, -0.4155), (0.992, -0.2466), (1.0001, -0.0816),
    (0.9497, 0.0757), (0.8874, 0.2175), (0.8109, 0.3456), (0.7344, 0.468), (0.656, 0.5922), (0.5513, 0.701),
    (0.4507, 0.8485), (0.3059, 0.9534), (0.1308, 0.9922), (-0.0503, 1.0), (-0.2294, 0.9728), (-0.4065, 0.9243),
    (-0.5715, 0.8447), (-0.7163, 0.7301), (-0.8411, 0.5961), (-0.9357, 0.4388), (-1.0001, 0.268), (-0.998, 0.0874),
];

/// `roduro_smith_home_01`'s work yard, round the house's +y side, layout units (past ±1).
pub const RODURO_SMITH_HOME_01_YARD_OUTLINE: &[(f32, f32)] = &[
    (-0.4773, 0.8357), (-0.2185, 0.9252), (-0.0475, 0.9513), (0.1239, 0.9439), (0.2841, 0.9083), (0.3725, 0.8447),
    (0.4304, 0.8567), (0.7833, 0.9878), (1.0449, 1.1932), (1.1604, 1.3598), (1.2317, 1.5499), (1.254, 1.9534),
    (1.1604, 2.2771), (1.0878, 2.3915), (0.9479, 2.5375), (0.7254, 2.6788), (0.4304, 2.7802), (0.1437, 2.8221),
    (-0.2427, 2.8254), (-0.5268, 2.7924), (-0.8216, 2.7052), (-1.05, 2.5784), (-1.1999, 2.4437), (-1.3154, 2.2771),
    (-1.4001, 2.0206), (-1.4089, 1.6835), (-1.3443, 1.4212), (-1.2427, 1.2454), (-1.05, 1.0584), (-0.8216, 0.9317),
];

/// `roduro_smith_home_01`'s work yard: fenced, gated, its stations in the open.
pub static RODURO_SMITH_HOME_01_YARD: Yard = Yard { outline: RODURO_SMITH_HOME_01_YARD_OUTLINE, gate: (1.2113, 1.8317), gate_width: 2.24, fence: 1.3, pieces: &[f(Forge, -1.1993, 1.8175, -0.07), f(Anvil, -0.7948, 1.6816, -0.07), f(Workbench, 0.3622, 2.3087, -1.57)] };

/// `roduro_tanner_home_01`'s ground tier (`assets/buildings/roduro_tanner_home_01.json`), layout units, counter-clockwise.
pub const RODURO_TANNER_HOME_01_OUTLINE: &[(f32, f32)] = &[
    (-0.9999, -0.037), (-0.9876, -0.1597), (-0.9753, -0.2908), (-0.9901, -0.4571), (-0.9531, -0.637), (-0.8152, -0.7849),
    (-0.6157, -0.8975), (-0.3744, -0.9681), (-0.1158, -1.0), (0.1453, -0.9966), (0.4015, -0.958), (0.628, -0.8739),
    (0.7857, -0.7277), (0.894, -0.5765), (0.9433, -0.4202), (0.9556, -0.2756), (0.9802, -0.1529), (0.9999, -0.0353),
    (0.9876, 0.0824), (0.9778, 0.2), (0.9852, 0.3328), (0.9925, 0.4908), (0.9211, 0.642), (0.7857, 0.7748),
    (0.6059, 0.8891), (0.3817, 0.9563), (0.1379, 0.9798), (-0.1108, 1.0), (-0.362, 0.9832), (-0.5911, 0.9092),
    (-0.7413, 0.7613), (-0.8448, 0.6118), (-0.8817, 0.4521), (-0.9063, 0.3176), (-0.9433, 0.1983), (-0.9728, 0.0824),
];

/// `roduro_tanner_home_01`'s work yard, round the house's +y side, layout units (past ±1).
pub const RODURO_TANNER_HOME_01_YARD_OUTLINE: &[(f32, f32)] = &[
    (-0.4901, 0.8953), (-0.3453, 0.942), (-0.1116, 0.9578), (0.3579, 0.9161), (0.6416, 0.961), (0.882, 1.0237),
    (1.1157, 1.1155), (1.3253, 1.2407), (1.441, 1.3425), (1.5312, 1.4568), (1.612, 1.6442), (1.6329, 1.8395),
    (1.612, 2.0348), (1.5312, 2.2222), (1.441, 2.3365), (1.3253, 2.4383), (1.1157, 2.5635), (0.8019, 2.6792),
    (0.4044, 2.7575), (-0.0483, 2.7887), (-0.6394, 2.7738), (-1.0307, 2.718), (-1.3504, 2.6282), (-1.6484, 2.4839),
    (-1.8302, 2.3365), (-1.9814, 2.0987), (-2.022, 1.8395), (-1.9814, 1.5803), (-1.8302, 1.3425), (-1.6484, 1.1951),
    (-1.3504, 1.0508), (-0.9509, 0.9455),
];

/// `roduro_tanner_home_01`'s work yard: fenced, gated, its stations in the open.
pub static RODURO_TANNER_HOME_01_YARD: Yard = Yard { outline: RODURO_TANNER_HOME_01_YARD_OUTLINE, gate: (1.579, 1.8029), gate_width: 2.06, fence: 1.3, pieces: &[f(Workbench, -0.9605, 2.3227, -0.77), f(Rack, 0.2734, 1.4639, 1.57), f(Rack, 0.2734, 1.7664, 1.57)] };

/// `roduro_tower_cottage_01`'s ground tier (`assets/buildings/roduro_tower_cottage_01.json`), layout units, counter-clockwise.
pub const RODURO_TOWER_COTTAGE_01_OUTLINE: &[(f32, f32)] = &[
    (1.0, 0.0), (0.9923, 0.1838), (0.9564, 0.3648), (0.8897, 0.5324), (0.7897, 0.6837), (0.6615, 0.808),
    (0.5103, 0.8999), (0.3462, 0.9621), (0.1744, 0.9918), (-0.0, 0.9999), (-0.1744, 0.9918), (-0.3462, 0.9621),
    (-0.5103, 0.8999), (-0.6615, 0.808), (-0.7897, 0.6837), (-0.8897, 0.5324), (-0.9564, 0.3648), (-0.9923, 0.1838),
    (-1.0, -0.0), (-0.9923, -0.1838), (-0.9564, -0.3648), (-0.8897, -0.5324), (-0.7897, -0.6837), (-0.6615, -0.808),
    (-0.5103, -0.8999), (-0.3462, -0.9621), (-0.1744, -0.9918), (0.0, -0.9999), (0.1744, -0.9918), (0.3462, -0.9621),
    (0.5103, -0.8999), (0.6615, -0.808), (0.7897, -0.6837), (0.8897, -0.5324), (0.9564, -0.3648), (0.9923, -0.1838),
];

/// `roduro_tower_house_01`'s ground tier (`assets/buildings/roduro_tower_house_01.json`), layout units, counter-clockwise.
pub const RODURO_TOWER_HOUSE_01_OUTLINE: &[(f32, f32)] = &[
    (0.9774, 0.0683), (1.0, 0.2533), (0.9865, 0.44), (0.9293, 0.615), (0.8284, 0.7649), (0.696, 0.8783),
    (0.5395, 0.9516), (0.374, 0.9883), (0.2054, 0.9999), (0.0369, 0.9983), (-0.1317, 0.9999), (-0.3017, 0.9966),
    (-0.4688, 0.9716), (-0.6298, 0.9166), (-0.7743, 0.8216), (-0.8901, 0.6866), (-0.9669, 0.52), (-1.0, 0.3366),
    (-0.9925, 0.15), (-0.9564, -0.0317), (-0.9007, -0.2083), (-0.8254, -0.3766), (-0.7351, -0.535), (-0.6298, -0.68),
    (-0.5049, -0.8066), (-0.3634, -0.9083), (-0.2054, -0.9733), (-0.0384, -0.9999), (0.1302, -0.9916), (0.2927, -0.9416),
    (0.4432, -0.8566), (0.5741, -0.7399), (0.6885, -0.6016), (0.7863, -0.4483), (0.8676, -0.285), (0.9323, -0.1117),
];

/// `roduro_twin_home_01`'s ground tier (`assets/buildings/roduro_twin_home_01.json`), layout units, counter-clockwise.
pub const RODURO_TWIN_HOME_01_OUTLINE: &[(f32, f32)] = &[
    (0.2512, -0.42), (0.4319, -0.3922), (0.6064, -0.3296), (0.7596, -0.2323), (0.8806, -0.1029), (0.9617, 0.0501),
    (1.0, 0.2156), (0.9969, 0.3853), (0.9541, 0.5508), (0.8683, 0.701), (0.7427, 0.8275), (0.5865, 0.9207),
    (0.4119, 0.9777), (0.2251, 1.0), (0.0383, 0.9903), (-0.1409, 0.9458), (-0.3063, 0.8651), (-0.4441, 0.751),
    (-0.5452, 0.6078), (-0.6034, 0.4465), (-0.6202, 0.2782), (-0.5972, 0.1099), (-0.6983, 0.0125), (-0.8469, -0.089),
    (-0.951, -0.2295), (-1.0, -0.3936), (-0.9939, -0.5633), (-0.9342, -0.7232), (-0.8193, -0.8567), (-0.6631, -0.9499),
    (-0.4839, -0.9958), (-0.2971, -1.0), (-0.1149, -0.9597), (0.0459, -0.8748), (0.1685, -0.7469), (0.2374, -0.5897),
];

/// `roduro_vault_tall_01`'s ground tier (`assets/buildings/roduro_vault_tall_01.json`), layout units, counter-clockwise.
pub const RODURO_VAULT_TALL_01_OUTLINE: &[(f32, f32)] = &[
    (1.0, 0.0), (0.9948, 0.2186), (0.9655, 0.4325), (0.8931, 0.6279), (0.7793, 0.7814), (0.6379, 0.8883),
    (0.4828, 0.9511), (0.3241, 0.9837), (0.1621, 0.9976), (-0.0, 1.0), (-0.1621, 0.9976), (-0.3241, 0.9837),
    (-0.4828, 0.9511), (-0.6379, 0.8883), (-0.7793, 0.7814), (-0.8931, 0.6279), (-0.9655, 0.4325), (-0.9948, 0.2186),
    (-1.0, -0.0), (-0.9948, -0.2186), (-0.9655, -0.4325), (-0.8931, -0.6279), (-0.7793, -0.7814), (-0.6379, -0.8883),
    (-0.4828, -0.9511), (-0.3241, -0.9837), (-0.1621, -0.9976), (0.0, -1.0), (0.1621, -0.9976), (0.3241, -0.9837),
    (0.4828, -0.9511), (0.6379, -0.8883), (0.7793, -0.7814), (0.8931, -0.6279), (0.9655, -0.4325), (0.9948, -0.2186),
];
// ---- end generated outlines ----

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

/// Whether a building is itself a place people come to for a service, and
/// so hangs a sign for players to find it (Laz: signs are for finding
/// services). Only the temple and the island hall (the ritual shrines for
/// now). A trade home doesn't: its trade is sold at the town's workplaces,
/// which carry signposts. Its `sign_of` mark stays as its trade (hover, tests).
pub fn serves(v: &Variant) -> bool {
    matches!(v.key, "qotiro_temple" | "qotiro_island_hall")
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
    if v.weight == 0 {
        // Placed by hand: its usual size, and any seed.
        return ((v.sizes.0 + v.sizes.1) * 0.5, 0);
    }
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
    if let Some(pts) = d.outline() {
        // Every corner that far in, and no corner of the outline poking in
        // over the box (an outline may curve in).
        let inside = |p: V2| outline_nearest(pts, d.half.x, d.half.y, d.frame(p)).0 >= margin - 1e-4;
        let wide = r.grow(margin);
        let [u, w] = wide.axes();
        let poked = pts.iter().any(|&k| {
            let o = d.to_world(k).sub(wide.c);
            dot(o, u).abs() < wide.hw && dot(o, w).abs() < wide.hd
        });
        return r.corners().into_iter().all(inside) && !poked;
    }
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
            let edge = d.outline_edge(p);
            if edge.is_some_and(|e| e.0 < 0.3) {
                continue;
            }
            let n = if let Some((_, _, inward)) = edge {
                inward.scale(-1.0)
            } else if d.round {
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
    let (at, n) = if d.round || d.outline().is_some() {
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
    let run = if d.round || d.outline().is_some() { 1.0 } else { 2.0 };
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
    if let Some((_, q, inward)) = d.outline_edge(p) {
        return (q.add(inward.scale(FACE)), inward);
    }
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
    if h.outer && (d.round || d.outline().is_some()) { outer_face(d, p) } else { (p, h.inward) }
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
    corner_avoiding(d, extras, &[])
}

/// `lodger_corner`, keeping clear of `avoid` too.
fn corner_avoiding(d: &Door, extras: &[(V2, f32)], avoid: &[Rect]) -> Option<Corner> {
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
        let taken: Vec<Rect> = all.iter().filter(|t| t.0 != own).map(|t| t.1).chain(avoid.iter().copied()).collect();
        if let Some(mut c) = fit_corner(d, at, rot, bed, &taken, &clear) {
            c.on = Some(own);
            return Some(c);
        }
    }
    // A fresh bedroll for the lodger, where a whole corner fits round it:
    // with room round it if there's any, else squeezed in.
    let taken: Vec<Rect> = all.iter().map(|t| t.1).chain(avoid.iter().copied()).collect();
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
    sleeping_plan_avoiding(d, residents, lodger, &[])
}

/// `sleeping_plan`, every bed and the corner also keeping clear of `avoid`
/// (belongings already lying on the floor, `World::sleeping_plan`).
pub fn sleeping_plan_avoiding(d: &Door, residents: usize, lodger: bool, avoid: &[Rect]) -> (Vec<(V2, f32)>, Option<Corner>) {
    let n = extras_wanted(d, residents);
    let extras = extras_avoiding(d, n, avoid);
    if !lodger {
        return (extras, None);
    }
    if let Some(c) = corner_avoiding(d, &extras, avoid) {
        return (extras, Some(c));
    }
    match corner_avoiding(d, &[], avoid) {
        Some(c) => {
            let mut keep: Vec<Rect> = c.pieces().into_iter().map(|p| p.1).collect();
            keep.push(c.bed_rect());
            keep.extend(avoid.iter().copied());
            let n = if c.on.is_none() { n.saturating_sub(1) } else { n };
            (extras_avoiding(d, n, &keep), Some(c))
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
