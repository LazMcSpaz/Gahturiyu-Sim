//! What is solid: every physical thing standing in the world, with its real
//! outline. The one source of truth for walking, clicking and aiming.
//!
//! **What's listed.** Town buildings (their outer walls with the doorway,
//! the door itself, inner walls, furniture and containers), a town's hearth
//! ring, workplaces (halls, shops and their counters, market stalls, sheds,
//! fenced yards, the lumps of a tender's yard, crafting stations in the
//! open), animal pens (the fence, its gate, a coop), bandit camps (the fire
//! and the lean-tos), ruins (wall pieces and columns) and lairs, chests out
//! in the wild, everything built at a base (walls, gates, towers, sheds,
//! fenced plots), rocks placed by hand, the homes of a forged town, standing
//! torches and tents. `things_of_town` and `things_of_wild` are the whole
//! list; nothing else in the sim decides what is solid.
//!
//! **What isn't.** People and animals; things lying on the ground; trees and
//! bushes (drawing only: where they stand depends on the graphics settings,
//! so they can't change what happens); signposts and banner poles (a post a
//! hand thick); docks, walkways and stilt platforms (ground to stand on, not
//! things in the way: `buildings.rs`).
//!
//! **A thing** (`Thing`) is an id, what it is, its outline on the map
//! (`Shape`: a box at its own turn, or an ellipse where the thing is round),
//! how tall it stands, whether it stops a walker, and what it stands inside
//! (`within`). A building, a pen or a fenced yard is *hollow*: its outline is
//! a wall with one opening (`Way`: where it is, how wide, and the points
//! just outside and just inside it), and the ground inside is free. Things
//! that don't stop a walker are listed too (a stilt home, a gate, a door, a
//! bedroll, a torch, a tent), because they can still be clicked or aimed at.
//!
//! **Ids** (`Id`) name the thing, not its place in a list: a building is its
//! town and number (the same pair as its `DoorId`), a container is its
//! `ContainerId`, a base building its base and its own id, a camp thing its
//! gang. They are the same after a save and a load, and a thing that moves
//! or changes keeps its id. A rock or a standing torch has no number of its
//! own, so its id is made from where it stands (and when it was lit).
//!
//! **Asking.** Everything is asked by place, never by walking a list:
//!
//! - `World::solids_near(p, within)`: every thing whose outline comes within
//!   `within` of `p` (clicking: what could be under the mouse; spells: what
//!   an area reaches).
//! - `World::solid_at(p)`: the thing at `p`, the smallest if several (a
//!   chest rather than the house it's in).
//! - `World::solid(id)`: one thing by id, as it stands now.
//! - `World::blocked_at(p, body)`: the thing a walker `body` wide can't
//!   stand at `p` for, if any.
//! - `World::way_clear(a, b, body)`: can a walker go straight from `a` to
//!   `b`? Walls stop it except at their opening.
//! - `World::free_spot(p)`: `p`, or the nearest spot beside whatever `p` is
//!   inside (a walk to a counter ends beside it); indoors, one the way in
//!   by the door gets to.
//! - `World::enclosure_at(p)`: the building, pen or yard whose wall is round `p`.
//!
//! **How it's kept.** The list is a pure function of the world (nothing new
//! is saved). It's built a town at a time, and once for everything out in
//! the wild, and kept until what it was built from changes (`Cache`; a
//! mark of the town's buildings and workplaces, or of the camps, pens,
//! bases and rocks). The marks are worked out again only after something
//! they're made from has changed: whatever changes one of those calls
//! `World::solid_changed` (the rocks and the counts are watched by
//! themselves), so an ask costs a look at a few numbers. Tents and standing
//! torches come and go, so they are added fresh on each ask.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::animals::Sp;
use super::base::{self, BaseId};
use super::buildings::{door_of, Door, DoorId};
use super::containers::{self, ContainerId};
use super::crafting::Station;
use super::geo::{self, V2};
use super::group::GroupId;
use super::jobs::PlaceKind;
use super::layout::{self, Furn, Holder, Rect};
use super::rng;
use super::ruins::RuinKind;
use super::settlement::{BuildingKind, Settlement, SettlementId};
use super::society::Workplace;
use super::world::World;

/// Half a walker's width, metres: how far clear of a solid thing a path keeps.
pub const BODY: f32 = 0.3;
/// How wide a building's doorway is, metres.
pub const DOOR: f32 = 1.3;
/// How wide a gate in a fence is, metres.
pub const GATE: f32 = 2.4;
/// How thick a fence is, metres.
pub const FENCE: f32 = 0.12;
/// Most corners an outline has.
pub const MAX_CORNERS: usize = 16;
/// Side of a cell of the index, metres.
const CELL: f32 = 12.0;
/// A hair: two things that only touch don't block each other.
const HAIR: f32 = 1e-3;

// ---- Ids ----------------------------------------------------------------------

/// Which thing. See the top of the file.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Id {
    /// A town building as a whole: its outer walls. (Town, building: its `DoorId`.)
    Building(SettlementId, u16),
    /// The door of that building.
    Door(SettlementId, u16),
    /// A stretch of inner wall: `Door::wall_pieces()[k]`.
    InnerWall(SettlementId, u16, u8),
    /// A piece of furniture: the variant's `furniture[k]`.
    Furniture(SettlementId, u16, u8),
    /// A container, indoors or out: its `ContainerId`.
    Container(SettlementId, u16, u8),
    /// Part `k` of a town's workplace `j` (0 is its main body).
    Place(SettlementId, u16, u8),
    /// The gate of a workplace's fenced yard.
    PlaceGate(SettlementId, u16),
    /// Crafting station `k` of a workplace, standing in the open.
    Station(SettlementId, u16, u8),
    /// A trade home's work yard (`layout::Yard`): its fence, its gate, and
    /// piece `k` standing in it (the yard's `pieces[k]`).
    Yard(SettlementId, u16),
    YardGate(SettlementId, u16),
    YardPiece(SettlementId, u16, u8),
    /// A bandit camp's fire (by its gang), and its lean-to `k`.
    CampFire(GroupId),
    CampShelter(GroupId, u8),
    /// Part `k` of a ruin (by `Ruin::id`): its wall pieces, then its columns.
    Ruin(u32, u8),
    /// A lair's rock arch (by `Ruin::id`).
    Lair(u32),
    /// An animal pen's fence (by `Pen::id`), its gate, and its coop.
    Pen(u32),
    PenGate(u32),
    PenHut(u32),
    /// Salt Isle's fenced briar patch, and its gate (`island.rs`).
    Briar,
    BriarGate,
    /// Salt Isle's compound palisade, stretch `k`, and its gate.
    Palisade(u8),
    PalisadeGate,
    /// A building at a base: the base, and the building's own id.
    Base(BaseId, u32),
    /// A rock placed by hand, named by where it stands.
    Rock(u64),
    /// A home of a forged town (its number in the founding).
    ForgeHome(u32),
    /// A standing torch, named by where and when it was lit.
    Torch(u64),
    /// A tent the squad has pitched (its `Bedding::id`), and a band of
    /// wanderers' tent.
    Tent(u32),
    GroupTent(GroupId),
    /// A bedroll laid out (its `Bedding::id`).
    Bedroll(u32),
    /// A campfire the squad laid (its `Campfire::id`).
    SquadFire(u32),
    /// The test quarter's wall, stretch `k` (`testquarter.rs`), and its gate.
    Quarter(u8),
    QuarterGate,
}

impl Id {
    /// The door this thing is, or is the building of.
    pub fn door(self) -> Option<DoorId> {
        match self {
            Id::Building(s, i) | Id::Door(s, i) => Some((s, i)),
            _ => None,
        }
    }
    /// The container this thing is.
    pub fn container(self) -> Option<ContainerId> {
        match self {
            Id::Container(s, b, k) => Some((s, b, k)),
            _ => None,
        }
    }
}

/// What a thing is, as far as anyone looking at it can tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    /// A building's outer walls.
    Building,
    Door,
    /// A wall on its own (inner, a palisade, a ruin's).
    Wall,
    Fence,
    Gate,
    Furniture(Furn),
    Container(Holder),
    /// A stall's counter, a cookhouse's block.
    Counter,
    Station(Station),
    /// A fire in its ring.
    Fire,
    Torch,
    /// Rock, a grown lump, a column.
    Stone,
    /// A lean-to, a hut, a shed: a roof you can't go into.
    Shelter,
    /// A stack of logs, a drying rack.
    Pile,
    Tent,
}

impl What {
    pub fn name(self) -> &'static str {
        match self {
            What::Building => "building",
            What::Door => "door",
            What::Wall => "wall",
            What::Fence => "fence",
            What::Gate => "gate",
            What::Furniture(f) => f.name(),
            What::Container(h) => h.name(),
            What::Counter => "counter",
            What::Station(s) => s.name(),
            What::Fire => "fire",
            What::Torch => "torch",
            What::Stone => "stone",
            What::Shelter => "shelter",
            What::Pile => "pile",
            What::Tent => "tent",
        }
    }
}

// ---- Outlines -------------------------------------------------------------------

fn dot(a: V2, b: V2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn unit(a: f32) -> V2 {
    V2::new(a.cos(), a.sin())
}

/// A convex outline: its corners, counter-clockwise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Poly {
    n: u8,
    v: [V2; MAX_CORNERS],
}

impl Poly {
    /// From corners in either order (at most `MAX_CORNERS`).
    pub fn new(pts: &[V2]) -> Poly {
        let n = pts.len().min(MAX_CORNERS);
        let mut v = [V2::default(); MAX_CORNERS];
        v[..n].copy_from_slice(&pts[..n]);
        // (Measured from its first corner: far from the map's corner the
        // plain sum loses a small outline in the rounding.)
        let mut area = 0.0;
        for k in 1..n.saturating_sub(1) {
            let (a, b) = (v[k].sub(v[0]), v[k + 1].sub(v[0]));
            area += a.x * b.y - a.y * b.x;
        }
        if area < 0.0 {
            v[..n].reverse();
        }
        Poly { n: n as u8, v }
    }

    pub fn corners(&self) -> &[V2] {
        &self.v[..self.n as usize]
    }

    /// Each edge: a corner, and the way straight out from the edge after it.
    fn edges(&self) -> impl Iterator<Item = (V2, V2)> + '_ {
        let n = self.n as usize;
        (0..n).map(move |k| {
            let (a, b) = (self.v[k], self.v[(k + 1) % n]);
            let d = b.sub(a);
            let l = d.len().max(1e-6);
            (a, V2::new(d.y / l, -d.x / l))
        })
    }

    pub fn centre(&self) -> V2 {
        let n = self.n.max(1) as f32;
        self.corners().iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / n)
    }

    /// How far outside `p` is (negative: that far inside), measured from the
    /// edges. Never more than the true distance.
    pub fn depth(&self, p: V2) -> f32 {
        self.edges().map(|(a, n)| dot(p.sub(a), n)).fold(f32::MIN, f32::max)
    }

    /// Is `p` inside, or within `margin` of it?
    pub fn contains(&self, p: V2, margin: f32) -> bool {
        self.depth(p) < margin
    }

    /// Does the straight line from `a` to `b` pass through the outline grown
    /// by `margin`? (Touching its edge isn't passing through.)
    pub fn crossed(&self, a: V2, b: V2, margin: f32) -> bool {
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        let lim = margin - HAIR;
        for (c, n) in self.edges() {
            let (fa, fb) = (dot(a.sub(c), n) - lim, dot(b.sub(c), n) - lim);
            if fa >= 0.0 && fb >= 0.0 {
                return false;
            }
            if fa < 0.0 && fb < 0.0 {
                continue;
            }
            let t = fa / (fa - fb);
            if fa >= 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
        t1 - t0 > 1e-6
    }

    /// The same outline, `m` bigger all round (corners kept sharp).
    pub fn grown(&self, m: f32) -> Poly {
        let n = self.n as usize;
        let e: Vec<(V2, V2)> = self.edges().collect();
        let mut v = [V2::default(); MAX_CORNERS];
        for k in 0..n {
            // Corner k lies between edge k-1 and edge k.
            let (n0, n1) = (e[(k + n - 1) % n].1, e[k].1);
            let s = n0.add(n1);
            let l2 = dot(s, s).max(1e-6);
            // Along the bisector, far enough to move both edges out by m.
            v[k] = self.v[k].add(s.scale(2.0 * m / l2));
        }
        Poly { n: self.n, v }
    }

    /// The nearest spot to `p` at least `m` clear of the outline: `p` itself
    /// if it's already clear.
    pub fn pushed_out(&self, p: V2, m: f32) -> V2 {
        let mut best: Option<(f32, V2)> = None;
        for (c, n) in self.edges() {
            let f = dot(p.sub(c), n);
            if f >= m {
                return p;
            }
            if best.is_none_or(|b| f > b.0) {
                best = Some((f, n));
            }
        }
        match best {
            Some((f, n)) => p.add(n.scale(m - f)),
            None => p,
        }
    }

    /// Do two outlines overlap by more than `slack`?
    pub fn overlaps(&self, o: &Poly, slack: f32) -> bool {
        for (p, q) in [(self, o), (o, self)] {
            for (c, n) in p.edges() {
                // Is every corner of the other beyond this edge?
                if q.corners().iter().all(|&x| dot(x.sub(c), n) >= -slack) {
                    return false;
                }
            }
        }
        true
    }
}

/// A thing's outline on the map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A box at its own turn (`layout::Rect`).
    Box(Rect),
    /// An ellipse: its middle, its turn, and how far it reaches along the
    /// turn and across it (a circle when they're the same).
    Round { c: V2, rot: f32, rx: f32, ry: f32 },
    /// An outline of its own (`layout::Shape::Poly`; it may curve in): its
    /// middle, its turn, how far it reaches along the turn and across, its
    /// corners as shares of those, counter-clockwise, and how much bigger
    /// all round it is than those corners (negative: smaller).
    Outline { c: V2, rot: f32, hx: f32, hy: f32, pts: &'static [(f32, f32)], grow: f32 },
}

impl Shape {
    pub fn circle(c: V2, r: f32) -> Shape {
        Shape::Round { c, rot: 0.0, rx: r, ry: r }
    }

    pub fn centre(&self) -> V2 {
        match self {
            Shape::Box(r) => r.c,
            Shape::Round { c, .. } | Shape::Outline { c, .. } => *c,
        }
    }

    /// How far from its middle it reaches at most.
    pub fn reach(&self) -> f32 {
        match self {
            Shape::Box(r) => (r.hw * r.hw + r.hd * r.hd).sqrt(),
            Shape::Round { rx, ry, .. } => rx.max(*ry) * ROUND_OUT,
            Shape::Outline { hx, hy, pts, grow, .. } => layout::outline_reach(pts, *hx, *hy) + grow.max(0.0),
        }
    }

    /// `p` in the shape's own frame: along its turn, and across.
    fn local(&self, p: V2) -> (f32, f32) {
        let (c, rot) = match self {
            Shape::Box(r) => (r.c, r.rot),
            Shape::Round { c, rot, .. } | Shape::Outline { c, rot, .. } => (*c, *rot),
        };
        let d = p.sub(c);
        let (s, k) = rot.sin_cos();
        (d.x * k + d.y * s, -d.x * s + d.y * k)
    }

    pub fn contains(&self, p: V2) -> bool {
        let (x, y) = self.local(p);
        match self {
            Shape::Box(r) => x.abs() < r.hw && y.abs() < r.hd,
            Shape::Round { rx, ry, .. } => (x / rx.max(1e-3)).powi(2) + (y / ry.max(1e-3)).powi(2) < 1.0,
            Shape::Outline { hx, hy, pts, grow, .. } => {
                if *grow == 0.0 {
                    layout::inside_outline(pts, *hx, *hy, V2::new(x, y))
                } else {
                    layout::outline_nearest(pts, *hx, *hy, V2::new(x, y)).0 + grow > 0.0
                }
            }
        }
    }

    /// The same shape, `m` bigger all round.
    pub fn grown(&self, m: f32) -> Shape {
        match *self {
            Shape::Box(r) => Shape::Box(r.grow(m)),
            Shape::Round { c, rot, rx, ry } => Shape::Round { c, rot, rx: rx + m, ry: ry + m },
            Shape::Outline { c, rot, hx, hy, pts, grow } => Shape::Outline { c, rot, hx, hy, pts, grow: grow + m },
        }
    }

    /// How far `p` is inside the shape's edge (negative: outside). Never
    /// more than the truth.
    pub fn margin(&self, p: V2) -> f32 {
        let (x, y) = self.local(p);
        match self {
            Shape::Box(r) => (r.hw - x.abs()).min(r.hd - y.abs()),
            Shape::Round { rx, ry, .. } => (1.0 - ((x / rx.max(1e-3)).powi(2) + (y / ry.max(1e-3)).powi(2)).sqrt()) * rx.min(*ry),
            Shape::Outline { hx, hy, pts, grow, .. } => layout::outline_nearest(pts, *hx, *hy, V2::new(x, y)).0 + grow,
        }
    }

    /// `p`, or if it's outside the shape, the nearest spot just inside it.
    pub fn pulled_in(&self, p: V2) -> V2 {
        if self.contains(p) {
            return p;
        }
        let (x, y) = self.local(p);
        let (c, rot, x, y) = match *self {
            Shape::Box(r) => (r.c, r.rot, x.clamp(-(r.hw - 0.02).max(0.0), (r.hw - 0.02).max(0.0)), y.clamp(-(r.hd - 0.02).max(0.0), (r.hd - 0.02).max(0.0))),
            Shape::Round { c, rot, rx, ry } => {
                let k = 0.98 / ((x / rx.max(1e-3)).powi(2) + (y / ry.max(1e-3)).powi(2)).sqrt().max(1e-3);
                (c, rot, x * k, y * k)
            }
            Shape::Outline { c, rot, hx, hy, pts, grow } => {
                // Straight in from the nearest edge, just past the line; if a
                // nook in the outline still leaves it out, toward the middle.
                let (_, q, n) = layout::outline_nearest(pts, hx, hy, V2::new(x, y));
                let mut at = q.add(n.scale(0.02 - grow));
                for k in 1..=20 {
                    if layout::outline_nearest(pts, hx, hy, at).0 + grow > 0.0 {
                        break;
                    }
                    at = q.add(n.scale(0.02 - grow)).scale(1.0 - k as f32 / 20.0);
                }
                (c, rot, at.x, at.y)
            }
        };
        c.add(unit(rot).scale(x)).add(unit(rot + std::f32::consts::FRAC_PI_2).scale(y))
    }

    /// Its outline as corners: a box's four; for a round thing, twelve
    /// flat sides drawn just outside the curve, so nothing inside the
    /// outline is ever outside the thing.
    pub fn outline(&self) -> Poly {
        match self {
            Shape::Box(r) => Poly::new(&r.corners()),
            Shape::Round { .. } => Poly::new(&self.ring(ROUND_SIDES, ROUND_OUT)),
            Shape::Outline { .. } => self.hull_of_outline(),
        }
    }

    /// An outline of its own as one convex block round all of it: sixteen
    /// flat sides, each touching the outline at its farthest that way.
    fn hull_of_outline(&self) -> Poly {
        let Shape::Outline { grow, .. } = *self else { return self.outline() };
        let pts = self.corners_of_outline();
        let n = MAX_CORNERS;
        let face = |k: usize| {
            let d = unit(k as f32 / n as f32 * std::f32::consts::TAU);
            (d, pts.iter().map(|p| dot(*p, d)).fold(f32::MIN, f32::max) + grow)
        };
        // Where each side meets the next; a side that touches the outline
        // only where its neighbours already do gives corners out of order
        // (a tiny loop), so the convex hull of them all is taken.
        let corners: Vec<V2> = (0..n)
            .map(|k| {
                let ((a, ha), (b, hb)) = (face(k), face(k + 1));
                let det = a.x * b.y - a.y * b.x;
                V2::new((ha * b.y - hb * a.y) / det, (a.x * hb - b.x * ha) / det)
            })
            .collect();
        Poly::new(&convex_hull(corners))
    }

    /// An outline of its own's corners on the map (as drawn: not grown).
    pub fn corners_of_outline(&self) -> Vec<V2> {
        let Shape::Outline { c, rot, hx, hy, pts, .. } = *self else { return Vec::new() };
        let (u, w) = (unit(rot), unit(rot + std::f32::consts::FRAC_PI_2));
        pts.iter().map(|&(x, y)| c.add(u.scale(x * hx)).add(w.scale(y * hy))).collect()
    }

    /// `n` points evenly round a round thing's edge (`out` times as far
    /// from its middle), the first straight along its turn.
    fn ring(&self, n: usize, out: f32) -> Vec<V2> {
        let Shape::Round { c, rot, rx, ry } = *self else { return Vec::new() };
        let (u, w) = (unit(rot), unit(rot + std::f32::consts::FRAC_PI_2));
        (0..n)
            .map(|k| {
                let a = k as f32 / n as f32 * std::f32::consts::TAU;
                c.add(u.scale(a.cos() * rx * out)).add(w.scale(a.sin() * ry * out))
            })
            .collect()
    }
}

/// The convex hull of some points (counter-clockwise, no three in a line).
fn convex_hull(mut pts: Vec<V2>) -> Vec<V2> {
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    pts.dedup_by(|a, b| a.dist(*b) < 1e-4);
    if pts.len() < 3 {
        return pts;
    }
    let cross = |o: V2, a: V2, b: V2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut lower: Vec<V2> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 1e-6 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<V2> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 1e-6 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Flat sides a round thing's outline is given, and how far out their
/// corners stand so the sides clear the curve.
pub const ROUND_SIDES: usize = 12;
const ROUND_OUT: f32 = 1.035_276_2; // 1 / cos(pi / 12)

/// The opening in a hollow thing's wall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Way {
    /// The middle of the opening, on the wall.
    pub at: V2,
    pub width: f32,
    /// Just outside it, and just inside.
    pub outside: V2,
    pub inside: V2,
}

/// A hollow thing's wall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hollow {
    /// How thick the wall is, inward from the outline.
    pub wall: f32,
    /// How many straight stretches a round wall is made of (a fence's
    /// spans between its posts); a box has its four sides.
    pub spans: u8,
    pub way: Option<Way>,
}

/// One physical thing. See the top of the file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thing {
    pub id: Id,
    pub what: What,
    pub shape: Shape,
    /// How tall it stands, metres.
    pub height: f32,
    /// Does it stop a walker? (A hollow thing: does its wall?)
    pub blocks: bool,
    pub hollow: Option<Hollow>,
    /// The hollow thing it stands inside (a chest's house, a coop's pen).
    pub within: Option<Id>,
}

impl Thing {
    fn solid(id: Id, what: What, shape: Shape, height: f32) -> Thing {
        Thing { id, what, shape, height, blocks: true, hollow: None, within: None }
    }
    fn open(self) -> Thing {
        Thing { blocks: false, ..self }
    }
    fn inside(self, of: Id) -> Thing {
        Thing { within: Some(of), ..self }
    }
    fn hollowed(self, wall: f32, spans: u8, way: Option<Way>) -> Thing {
        Thing { hollow: Some(Hollow { wall, spans, way }), ..self }
    }

    pub fn name(&self) -> &'static str {
        self.what.name()
    }

    /// The opening in its wall, if it's hollow and has one.
    pub fn way(&self) -> Option<Way> {
        self.hollow.and_then(|h| h.way)
    }

    /// The thing's whole outline as one block: what a walk that isn't going
    /// in has to go round. A fence's is the line of its posts (up to
    /// `MAX_CORNERS` of them; a bigger pen's is drawn just outside them).
    pub fn hull(&self) -> Poly {
        match (self.hollow, self.shape) {
            (Some(h), Shape::Round { .. }) if h.wall < 0.2 => {
                let n = (h.spans as usize).max(8);
                if n <= MAX_CORNERS {
                    Poly::new(&self.shape.ring(n, 1.0))
                } else {
                    Poly::new(&self.shape.ring(MAX_CORNERS, 1.0 / (std::f32::consts::PI / MAX_CORNERS as f32).cos()))
                }
            }
            _ => self.shape.outline(),
        }
    }

    /// The ground inside a hollow thing where a walker `body` wide can
    /// stand: its outline, in by its wall and the walker's width.
    pub fn floor(&self, body: f32) -> Shape {
        self.shape.grown(-(self.hollow.map(|h| h.wall).unwrap_or(0.0) + body))
    }

    /// A hollow thing's wall as solid stretches, the opening left out.
    pub fn walls(&self) -> Vec<Rect> {
        let Some(h) = self.hollow else { return Vec::new() };
        let pts: Vec<V2> = match self.shape {
            Shape::Box(r) => r.corners().to_vec(),
            Shape::Round { .. } => self.shape.ring((h.spans as usize).max(8), 1.0),
            Shape::Outline { .. } => self.shape.corners_of_outline(),
        };
        // (An outline of its own may curve in: its corners go counter-
        // clockwise, so in is to the left of each stretch.)
        let ccw = matches!(self.shape, Shape::Outline { .. });
        let mid = self.shape.centre();
        let mut out = Vec::new();
        for k in 0..pts.len() {
            let (a, b) = (pts[k], pts[(k + 1) % pts.len()]);
            let len = a.dist(b);
            if len < 1e-3 {
                continue;
            }
            let along = b.sub(a).scale(1.0 / len);
            // Inward from this stretch.
            let n = V2::new(-along.y, along.x);
            let n = if !ccw && dot(mid.sub(a), n) < 0.0 { n.scale(-1.0) } else { n };
            // (An outline of its own may turn in at a corner, where two
            // stretches set in by the wall's thickness part a little: each
            // stretch runs on a touch past its corners to close that.)
            let ext = if ccw { h.wall * 0.8 } else { 0.0 };
            // (Not into the opening, which may be wider than a stretch.)
            let clear = |p: V2| h.way.is_none_or(|w| p.dist(w.at) > w.width * 0.5 + ext + 0.1);
            let (ext_a, ext_b) = (if clear(a) { ext } else { 0.0 }, if clear(b) { ext } else { 0.0 });
            let mut piece = |from: f32, to: f32| {
                if to - from > 0.05 {
                    let (from, to) = (if from == 0.0 { -ext_a } else { from }, if to == len { len + ext_b } else { to });
                    let c = a.add(along.scale((from + to) * 0.5)).add(n.scale(h.wall * 0.5));
                    out.push(Rect::new(c, along.y.atan2(along.x), to - from, h.wall));
                }
            };
            // The opening: whatever of this stretch lies within half its
            // width of the line through it (out to in), near the opening
            // itself (that line meets the far wall too).
            let gap = h.way.and_then(|w| {
                let through = w.outside.sub(w.inside);
                let through = through.scale(1.0 / through.len().max(1e-3));
                let side = V2::new(-through.y, through.x);
                // Sideways from the line, as the stretch is walked: l0 + s * dl.
                let (l0, dl) = (dot(a.sub(w.at), side), dot(along, side));
                let half = w.width * 0.5;
                if dl.abs() < 1e-4 {
                    return None;
                }
                let (s0, s1) = ((-half - l0) / dl, (half - l0) / dl);
                let (s0, s1) = (s0.min(s1).max(0.0), s0.max(s1).min(len));
                let near = a.add(along.scale((s0 + s1) * 0.5)).dist(w.at) < w.width;
                (s1 > s0 && near).then_some((s0, s1))
            });
            match gap {
                Some((g0, g1)) => {
                    piece(0.0, g0);
                    piece(g1, len);
                }
                None => piece(0.0, len),
            }
        }
        out
    }
}

// ---- The list, indexed ------------------------------------------------------------

/// What a piece of the index is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// A solid thing's whole outline.
    Body,
    /// A stretch of a hollow thing's wall.
    Wall,
    /// A hollow thing's whole outline, as if it had no opening (what a
    /// walk that isn't going in must go round).
    Hull,
}

/// One convex piece that stops a walker.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    /// Which thing it's of (an index into `Solids::things`).
    pub thing: u32,
    pub part: Part,
    pub poly: Poly,
    /// The hollow thing it's inside (for a wall: the thing it's the wall of).
    pub room: Option<Id>,
}

/// A set of things with an index by place.
#[derive(Debug, Default)]
pub struct Solids {
    pub things: Vec<Thing>,
    /// Each thing's outline as corners (`Shape::outline`), in step with `things`.
    pub outlines: Vec<Poly>,
    pub pieces: Vec<Piece>,
    by_cell: HashMap<(i32, i32), (Vec<u32>, Vec<u32>)>,
}

fn cell(p: V2) -> (i32, i32) {
    ((p.x / CELL).floor() as i32, (p.y / CELL).floor() as i32)
}

impl Solids {
    pub fn new(things: Vec<Thing>) -> Solids {
        let outlines = things.iter().map(|t| t.shape.outline()).collect();
        let mut s = Solids { things, outlines, pieces: Vec::new(), by_cell: HashMap::new() };
        for (k, th) in s.things.iter().enumerate() {
            let (c, r) = (th.shape.centre(), th.shape.reach());
            let (lo, hi) = (cell(V2::new(c.x - r, c.y - r)), cell(V2::new(c.x + r, c.y + r)));
            for i in lo.0..=hi.0 {
                for j in lo.1..=hi.1 {
                    s.by_cell.entry((i, j)).or_default().0.push(k as u32);
                }
            }
            if !th.blocks {
                continue;
            }
            let mut put = |part: Part, poly: Poly, room: Option<Id>| s.pieces.push(Piece { thing: k as u32, part, poly, room });
            if th.hollow.is_some() {
                put(Part::Hull, th.hull(), th.within);
                for w in th.walls() {
                    put(Part::Wall, Poly::new(&w.corners()), Some(th.id));
                }
            } else {
                put(Part::Body, s.outlines[k], th.within);
            }
        }
        for (k, p) in s.pieces.iter().enumerate() {
            let (mut lo, mut hi) = (p.poly.corners()[0], p.poly.corners()[0]);
            for q in p.poly.corners() {
                lo = V2::new(lo.x.min(q.x), lo.y.min(q.y));
                hi = V2::new(hi.x.max(q.x), hi.y.max(q.y));
            }
            let (lo, hi) = (cell(lo), cell(hi));
            for i in lo.0..=hi.0 {
                for j in lo.1..=hi.1 {
                    s.by_cell.entry((i, j)).or_default().1.push(k as u32);
                }
            }
        }
        s
    }

    /// The cells a box from `lo` to `hi` touches, each handed to `f` once.
    fn cells(&self, lo: V2, hi: V2, mut f: impl FnMut(&(Vec<u32>, Vec<u32>))) {
        let (lo, hi) = (cell(lo), cell(hi));
        for i in lo.0..=hi.0 {
            for j in lo.1..=hi.1 {
                if let Some(c) = self.by_cell.get(&(i, j)) {
                    f(c);
                }
            }
        }
    }

    /// Things whose outline comes within `within` of `p`, in list order.
    pub fn things_near(&self, p: V2, within: f32) -> Vec<&Thing> {
        let mut ids: Vec<u32> = Vec::new();
        self.cells(V2::new(p.x - within, p.y - within), V2::new(p.x + within, p.y + within), |c| ids.extend(&c.0));
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter().filter(|&k| self.outlines[k as usize].depth(p) < within).map(|k| &self.things[k as usize]).collect()
    }

    /// Pieces whose outline may come within `within` of the box `lo`..`hi`, in list order.
    pub fn pieces_in(&self, lo: V2, hi: V2, within: f32) -> Vec<&Piece> {
        let mut ids: Vec<u32> = Vec::new();
        self.cells(V2::new(lo.x - within, lo.y - within), V2::new(hi.x + within, hi.y + within), |c| ids.extend(&c.1));
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter().map(|k| &self.pieces[k as usize]).collect()
    }

    /// The piece that stops a walker `body` wide standing at `p`, if any.
    /// (Walls and bodies; a hull doesn't count: inside it the ground is free.)
    pub fn blocker_at(&self, p: V2, body: f32) -> Option<&Piece> {
        let mut found: Option<&Piece> = None;
        self.cells(V2::new(p.x - body, p.y - body), V2::new(p.x + body, p.y + body), |c| {
            for &k in &c.1 {
                let pc = &self.pieces[k as usize];
                if pc.part != Part::Hull && pc.poly.contains(p, body - HAIR) && found.is_none_or(|f| pc.thing < f.thing) {
                    found = Some(pc);
                }
            }
        });
        found
    }

    /// The first piece (in list order) a walker `body` wide would have to
    /// pass through going straight from `a` to `b`, if any.
    pub fn crossing(&self, a: V2, b: V2, body: f32) -> Option<&Piece> {
        let (lo, hi) = (V2::new(a.x.min(b.x), a.y.min(b.y)), V2::new(a.x.max(b.x), a.y.max(b.y)));
        self.pieces_in(lo, hi, body).into_iter().find(|pc| pc.part != Part::Hull && pc.poly.crossed(a, b, body))
    }
}

// ---- Keeping it ---------------------------------------------------------------------

/// The lists as last built, with the mark of what each was built from.
/// Never saved; a copy of the world starts with none.
#[derive(Default)]
pub struct Cache(Mutex<Kept>, AtomicU64);

#[derive(Default)]
struct Kept {
    towns: Vec<Option<(u64, Arc<Solids>)>>,
    wild: Option<(u64, Arc<Solids>)>,
    /// The marks as last worked out (`None`: not yet, for a town), and the
    /// quick look (`World::quick_mark`) they were worked out at.
    marks: Option<([u64; 6], Option<u64>, Vec<Option<u64>>)>,
    /// Ways already found (`walk.rs`), by their two ends, each with the
    /// mark of the lists it was found among.
    ways: HashMap<[u32; 4], (u64, Arc<super::walk::Found>)>,
    /// Spots to stand already found (`World::free_spot`, `stand_by`), by
    /// what was asked, each with the room round it (`free_spot` only) and
    /// the mark of the lists it was found among.
    spots: HashMap<[u32; 5], (u64, Option<V2>, f32)>,
}

/// Most ways (and, apart, spots) kept before the lot is dropped and found
/// afresh as asked for.
const WAYS_KEPT: usize = 40_000;

impl Kept {
    /// The mark `World::solid_sets_marked` would give for `p`, if every
    /// list it needs is built and nothing has changed since (`q`, the quick
    /// look); with `out`, those lists too.
    fn mark_near(&self, w: &World, q: [u64; 6], p: V2, within: f32, mut out: Option<&mut Vec<Arc<Solids>>>) -> Option<u64> {
        let marks = self.marks.as_ref().filter(|m| m.0 == q)?;
        let (wm, wild) = self.wild.as_ref()?;
        if marks.1 != Some(*wm) {
            return None;
        }
        let mut all = *wm;
        if let Some(o) = out.as_deref_mut() {
            o.push(wild.clone());
        }
        for s in &w.settlements {
            if s.pos.dist(p) < s.reach + within + 30.0 {
                let (tm, set) = self.towns.get(s.id as usize)?.as_ref()?;
                if marks.2.get(s.id as usize).copied().flatten() != Some(*tm) {
                    return None;
                }
                all = mark(all ^ tm, s.id as u64);
                if let Some(o) = out.as_deref_mut() {
                    o.push(set.clone());
                }
            }
        }
        Some(all)
    }
}

impl Cache {
    /// A way kept for these two ends, found among lists with this mark.
    pub(super) fn way(&self, ends: [u32; 4], mark: u64) -> Option<Arc<super::walk::Found>> {
        let kept = self.0.lock().unwrap_or_else(|e| e.into_inner());
        kept.ways.get(&ends).filter(|w| w.0 == mark).map(|w| w.1.clone())
    }

    pub(super) fn keep_way(&self, ends: [u32; 4], mark: u64, way: Arc<super::walk::Found>) {
        let mut kept = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if kept.ways.len() >= WAYS_KEPT {
            kept.ways.clear();
        }
        kept.ways.insert(ends, (mark, way));
    }

    fn spot(&self, key: [u32; 5], mark: u64) -> Option<(Option<V2>, f32)> {
        let kept = self.0.lock().unwrap_or_else(|e| e.into_inner());
        kept.spots.get(&key).filter(|s| s.0 == mark).map(|s| (s.1, s.2))
    }

    fn keep_spot(&self, key: [u32; 5], mark: u64, spot: Option<V2>, room: f32) {
        let mut kept = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if kept.spots.len() >= WAYS_KEPT {
            kept.spots.clear();
        }
        kept.spots.insert(key, (mark, spot, room));
    }
}

impl Clone for Cache {
    fn clone(&self) -> Cache {
        Cache::default()
    }
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("solid::Cache")
    }
}

/// Check every quick look against the full marks, and stop the world if
/// something they're made from changed without `World::solid_changed`
/// (tests).
static CHECK: AtomicBool = AtomicBool::new(false);

/// Turn the check on (`CHECK`). For tests: the marks are then worked out
/// in full on every ask as well, which is slow.
pub fn check_marks(on: bool) {
    CHECK.store(on, Ordering::Relaxed);
}

fn mark(h: u64, x: u64) -> u64 {
    (h ^ x).wrapping_mul(0x0000_0100_0000_01B3).rotate_left(23)
}

fn mark_v2(h: u64, p: V2) -> u64 {
    mark(h, (p.x.to_bits() as u64) << 32 | p.y.to_bits() as u64)
}

impl World {
    /// Something the solid lists are built from has changed (a camp gone, a
    /// base building placed or standing, a town's workplaces laid out
    /// again): their marks are worked out afresh at the next ask. Anything
    /// that changes what `town_mark` or `wild_mark` reads calls this; a
    /// rock edit or a new camp, pen, base or town is also seen by itself
    /// (`quick_mark`).
    pub fn solid_changed(&self) {
        self.solid_cache.1.fetch_add(1, Ordering::Relaxed);
    }

    /// What's looked at on every ask: the changes told of, and the counts
    /// and rock versions that move by themselves.
    fn quick_mark(&self) -> [u64; 6] {
        let rocks = |e: &super::mapedit::MapEdits| (e.rocks.len() as u64) << 32 | e.rocks_v as u64;
        [
            self.solid_cache.1.load(Ordering::Relaxed),
            (self.settlements.len() as u64) << 32 | self.camps.len() as u64,
            (self.bases.len() as u64) << 32 | self.animals.pens.len() as u64,
            rocks(&self.terrain.edits),
            rocks(&self.terrain.authored),
            self.ruins.len() as u64,
        ]
    }

    /// The marks kept for the wild (`None`) or a town, if nothing has
    /// changed since they were worked out; otherwise worked out again.
    fn kept_mark(&self, town: Option<SettlementId>) -> u64 {
        let q = self.quick_mark();
        let full = || match town {
            Some(t) => self.town_mark(t),
            None => self.wild_mark(),
        };
        let have = {
            let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
            if kept.marks.as_ref().map(|m| m.0) != Some(q) {
                kept.marks = Some((q, None, vec![None; self.settlements.len()]));
            }
            let m = kept.marks.as_ref().unwrap();
            match town {
                Some(t) => m.2.get(t as usize).copied().flatten(),
                None => m.1,
            }
        };
        if let Some(h) = have {
            if CHECK.load(Ordering::Relaxed) {
                assert_eq!(h, full(), "the solid lists of {town:?} changed without World::solid_changed");
            }
            return h;
        }
        let h = full();
        let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = kept.marks.as_mut().filter(|m| m.0 == q) {
            match town {
                Some(t) => {
                    if let Some(slot) = m.2.get_mut(t as usize) {
                        *slot = Some(h);
                    }
                }
                None => m.1 = Some(h),
            }
        }
        h
    }

    /// `solid_sets_marked` with one look under the lock, when nothing has
    /// changed since every list it needs was built (the usual ask).
    fn sets_kept(&self, p: V2, within: f32) -> Option<(Vec<Arc<Solids>>, u64)> {
        let q = self.quick_mark();
        let kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        let all = kept.mark_near(self, q, p, within, Some(&mut out))?;
        drop(kept);
        self.check_mark_near(p, within, all);
        Some((out, all))
    }

    fn check_mark_near(&self, p: V2, within: f32, all: u64) {
        if CHECK.load(Ordering::Relaxed) {
            assert_eq!(all, self.full_mark_near(p, within), "the solid lists near {p:?} changed without World::solid_changed");
        }
    }

    /// The mark of the lists near `p`, worked out in full (`CHECK`).
    fn full_mark_near(&self, p: V2, within: f32) -> u64 {
        let mut all = self.wild_mark();
        for s in &self.settlements {
            if s.pos.dist(p) < s.reach + within + 30.0 {
                all = mark(all ^ self.town_mark(s.id), s.id as u64);
            }
        }
        all
    }

    /// A mark of everything a town's list is built from.
    fn town_mark(&self, t: SettlementId) -> u64 {
        let s = &self.settlements[t as usize];
        let mut h = 0x9E37_79B9_7F4A_7C15u64 ^ s.buildings.len() as u64;
        for (i, b) in s.buildings.iter().enumerate() {
            h = mark_v2(h, b.pos);
            h = mark(h, (b.size.to_bits() as u64) << 32 | b.rot.to_bits() as u64);
            h = mark(h, b.seed ^ (b.kind as u64) << 56);
            if let Some(Some(k)) = s.styles.get(i) {
                h = mark(h, k.as_ptr() as u64 ^ (k.len() as u64) << 48);
            }
        }
        if let Some(tl) = self.society.towns.get(t as usize) {
            h = mark(h, tl.places.len() as u64);
            for p in &tl.places {
                h = mark_v2(h, p.pos);
                h = mark(h, (p.kind.size().to_bits() as u64) << 32 | p.rot.to_bits() as u64);
                h = mark(h, p.seed);
            }
        }
        h
    }

    /// A mark of everything the list of the wild is built from.
    fn wild_mark(&self) -> u64 {
        let mut h = 0x243F_6A88_85A3_08D3u64;
        h = mark(h, self.camps.len() as u64);
        for c in &self.camps {
            h = mark_v2(mark(h, c.group as u64), c.pos);
        }
        h = mark(h, self.ruins.len() as u64);
        for r in &self.ruins {
            h = mark_v2(mark(h, r.id as u64), r.pos);
        }
        h = mark(h, self.animals.pens.len() as u64);
        for p in &self.animals.pens {
            h = mark_v2(mark(h, p.radius.to_bits() as u64 ^ (p.sp as u64) << 40), p.home);
        }
        for b in &self.bases {
            h = mark(h, (b.id as u64) << 32 | b.buildings.len() as u64);
            for bl in &b.buildings {
                let state = match bl.state {
                    base::State::Site(_) => 1u64,
                    base::State::Standing { .. } => 2,
                    base::State::Ruin => 3,
                };
                h = mark_v2(mark(h, (bl.id as u64) << 32 | (bl.def as u64) << 8 | state), bl.at);
                h = mark(h, (bl.w.to_bits() as u64) << 32 | bl.rot.to_bits() as u64);
            }
        }
        for e in [&self.terrain.edits, &self.terrain.authored] {
            h = mark(h, (e.rocks.len() as u64) << 32 | e.rocks_v as u64);
        }
        if let Some(q) = &self.test_quarter {
            h = mark_v2(mark(h, 0x7E57 + q.open as u64), q.min);
        }
        let wild = self.containers.range((containers::WILD, 0, 0)..=(containers::WILD, u16::MAX, u8::MAX));
        for (id, c) in wild {
            h = mark_v2(mark(h, (id.1 as u64) << 8 | id.2 as u64), c.pos);
        }
        h = mark(h, self.forge.as_ref().map(|f| f.founding.homes.len() as u64 + 1).unwrap_or(0));
        h
    }

    /// The list for one town, as it stands now.
    pub fn town_solids(&self, t: SettlementId) -> Arc<Solids> {
        self.town_solids_marked(t).1
    }

    fn town_solids_marked(&self, t: SettlementId) -> (u64, Arc<Solids>) {
        let m = self.kept_mark(Some(t));
        {
            let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
            if kept.towns.len() <= t as usize {
                kept.towns.resize(t as usize + 1, None);
            }
            if let Some((have, s)) = &kept.towns[t as usize] {
                if *have == m {
                    return (m, s.clone());
                }
            }
        }
        let s = Arc::new(Solids::new(self.things_of_town(t)));
        let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        kept.towns[t as usize] = Some((m, s.clone()));
        (m, s)
    }

    /// The list for everything outside the towns, as it stands now.
    pub fn wild_solids(&self) -> Arc<Solids> {
        self.wild_solids_marked().1
    }

    fn wild_solids_marked(&self) -> (u64, Arc<Solids>) {
        let m = self.kept_mark(None);
        {
            let kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((have, s)) = &kept.wild {
                if *have == m {
                    return (m, s.clone());
                }
            }
        }
        let s = Arc::new(Solids::new(self.things_of_wild()));
        let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        kept.wild = Some((m, s.clone()));
        (m, s)
    }

    /// Every list with something within `within` of `p`.
    pub fn solid_sets(&self, p: V2, within: f32) -> Vec<Arc<Solids>> {
        self.solid_sets_marked(p, within).0
    }

    /// `solid_sets`, with one mark for the lot: it changes when anything
    /// any of them was built from does.
    pub fn solid_sets_marked(&self, p: V2, within: f32) -> (Vec<Arc<Solids>>, u64) {
        if let Some(kept) = self.sets_kept(p, within) {
            return kept;
        }
        let (mut all, wild) = self.wild_solids_marked();
        let mut out = vec![wild];
        for s in &self.settlements {
            if s.pos.dist(p) < s.reach + within + 30.0 {
                let (m, set) = self.town_solids_marked(s.id);
                all = mark(all ^ m, s.id as u64);
                out.push(set);
            }
        }
        (out, all)
    }
}

// ---- Building the list -----------------------------------------------------------------

/// A building's outline from its door (`buildings.rs`).
pub fn shell_of(d: &Door) -> Shape {
    if let Some(pts) = d.outline() {
        Shape::Outline { c: d.centre, rot: d.rot, hx: d.half.x, hy: d.half.y, pts, grow: 0.0 }
    } else if d.round {
        Shape::Round { c: d.centre, rot: d.rot, rx: d.half.x, ry: d.half.y }
    } else {
        Shape::Box(Rect::new(d.centre, d.rot, d.half.x * 2.0, d.half.y * 2.0))
    }
}

/// A stretch of inner wall as it stands: cut off where it meets a round
/// building's outer wall (None if none of it is inside).
pub fn inner_wall(d: &Door, r: Rect) -> Option<Rect> {
    let [u, _] = r.axes();
    let (a, z) = (r.c.sub(u.scale(r.hw)), r.c.add(u.scale(r.hw)));
    if let Some(pts) = d.outline() {
        // From where it first comes inside to where it last leaves: the
        // places it crosses the outline, and which stretches between are in.
        let (pa, pz) = (d.frame(a), d.frame(z));
        let mut ts = vec![0.0f32, 1.0];
        for k in 0..pts.len() {
            let (p, q) = (pts[k], pts[(k + 1) % pts.len()]);
            let (p, q) = (V2::new(p.0 * d.half.x, p.1 * d.half.y), V2::new(q.0 * d.half.x, q.1 * d.half.y));
            let (e, f) = (pz.sub(pa), q.sub(p));
            let den = e.x * f.y - e.y * f.x;
            if den.abs() < 1e-9 {
                continue;
            }
            let g = p.sub(pa);
            let (t, s) = ((g.x * f.y - g.y * f.x) / den, (g.x * e.y - g.y * e.x) / den);
            if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s) {
                ts.push(t);
            }
        }
        ts.sort_by(f32::total_cmp);
        let inside: Vec<(f32, f32)> = ts.windows(2).filter(|w| w[1] - w[0] > 1e-4 && layout::inside_outline(pts, d.half.x, d.half.y, pa.lerp(pz, (w[0] + w[1]) * 0.5))).map(|w| (w[0], w[1])).collect();
        let (t0, t1) = (inside.first()?.0, inside.last()?.1);
        if t0 == 0.0 && t1 == 1.0 {
            return Some(r);
        }
        let (a2, z2) = (a.lerp(z, t0), a.lerp(z, t1));
        return (t1 - t0 > 1e-3).then(|| Rect::new(a2.lerp(z2, 0.5), r.rot, a2.dist(z2), r.hd * 2.0));
    }
    if !d.round {
        return Some(r);
    }
    // Where along it (0 at `a`, 1 at `z`) it is inside the outline.
    let (pa, pz) = (d.to_local(a), d.to_local(z));
    let (dx, dy) = (pz.0 - pa.0, pz.1 - pa.1);
    let (qa, qb, qc) = (dx * dx + dy * dy, 2.0 * (pa.0 * dx + pa.1 * dy), pa.0 * pa.0 + pa.1 * pa.1 - 1.0);
    let disc = qb * qb - 4.0 * qa * qc;
    if qa < 1e-9 || disc <= 0.0 {
        return (qc < 0.0).then_some(r);
    }
    let (t0, t1) = (((-qb - disc.sqrt()) / (2.0 * qa)).max(0.0), ((-qb + disc.sqrt()) / (2.0 * qa)).min(1.0));
    if t1 - t0 < 1e-3 {
        return None;
    }
    let (a2, z2) = (a.lerp(z, t0), a.lerp(z, t1));
    Some(Rect::new(a2.lerp(z2, 0.5), r.rot, a2.dist(z2), r.hd * 2.0))
}

/// The doorway in a building's outer wall: `inside` is the first step past
/// the wall (nearer than `Door::inside`, which may have a table by it).
pub fn doorway_of(d: &Door) -> Way {
    let dir = unit(d.rot);
    let at = d.outside.sub(dir.scale(1.2));
    Way { at, width: DOOR, outside: d.outside, inside: at.sub(dir.scale(layout::WALL_IN + BODY + 0.1)) }
}

/// A workplace that lies inside one of the town's buildings (Salt Isle's
/// hall: the compound's stone hall is the place of work): the building
/// stands for it, so it has no parts of its own, solid or drawn.
pub fn place_in_building(s: &Settlement, wp: &Workplace) -> bool {
    (0..s.buildings.len() as u16).any(|i| door_of(s, i).is_some_and(|d| d.contains(wp.pos)))
}

/// The solid parts of a workplace, as the window draws them (`view/scene.rs`,
/// `workplace`): what each is, its outline, how tall, and whether it's in
/// the way. Part 0 is its main body. A place out on the water has none, and
/// nor has one inside a building (`place_in_building`).
pub fn place_parts(wp: &Workplace) -> Vec<(What, Shape, f32, bool)> {
    use PlaceKind as P;
    let k = wp.kind;
    let size = k.size();
    if geo::inland(wp.pos) < 0.0 {
        return Vec::new();
    }
    let (along, across) = (unit(wp.rot), unit(wp.rot + std::f32::consts::FRAC_PI_2));
    let at = |a: f32, c: f32| wp.pos.add(along.scale(a)).add(across.scale(c));
    let u = |n: u64| (rng::key(&[wp.seed, n]) >> 40) as f32 / (1u64 << 24) as f32;
    let bx = |c: V2, rot: f32, w: f32, d: f32| Shape::Box(Rect::new(c, rot, w, d));
    let block = |w: f32, d: f32, h: f32| (What::Shelter, bx(wp.pos, wp.rot, w, d), h, true);
    match k {
        P::Fields | P::KelpBeds | P::Dock | P::DivePlatform => Vec::new(),
        // A lean-to and a drying rack.
        P::Wilds => vec![block(3.0, 2.2, 1.6), (What::Pile, bx(at(4.1, 0.0), wp.rot, 2.4, 0.2), 1.8, true)],
        // A stack of logs.
        P::Woodlot => vec![(What::Pile, bx(at(0.0, 1.1), wp.rot, 4.0, 2.7), 0.55, true)],
        // The spoil heap, and cut blocks round it.
        P::Quarry => {
            let mut v = vec![(What::Stone, Shape::Round { c: wp.pos, rot: 0.0, rx: 4.0, ry: 3.5 }, 2.2, true)];
            for i in 0..6u64 {
                let a = i as f32 * 1.1 + u(2) * 6.0;
                v.push((What::Stone, bx(wp.pos.add(unit(a).scale(4.0 + u(9 + i) * 3.0)), a, 1.6, 1.3), 1.0 + u(3 + i) * 0.8, true));
            }
            v
        }
        P::CharcoalPit => vec![(What::Fire, Shape::circle(wp.pos, 2.6), 1.8, true)],
        // A hull turned over on trestles.
        P::Boatyard => vec![(What::Pile, Shape::Round { c: wp.pos, rot: 0.0, rx: 3.4, ry: 1.1 }, 1.9, true)],
        // The cookhouse's block under its roof, the fire and pots on it.
        P::Kitchen | P::Deck => vec![(What::Counter, bx(wp.pos, wp.rot, 6.0, 4.2), 1.0, true)],
        P::MessHall | P::Hall => vec![block(size, size * 0.6, 4.0)],
        // Five stalls in a ring; the middle is open.
        P::Market => (0..5)
            .map(|i| {
                let a = i as f32 / 5.0 * std::f32::consts::TAU + u(1);
                (What::Counter, bx(wp.pos.add(unit(a).scale(size * 0.33)), a + 1.57, 2.6, 2.6 * 0.7), 1.0, true)
            })
            .collect(),
        // The shop, and the counter under its awning out front.
        P::Shop(_) => vec![block(size * 0.8, size * 0.6, 3.2), (What::Counter, bx(at(size * 0.5, 0.0), wp.rot + 1.57, 2.8, 2.8 * 0.7), 1.0, true)],
        P::Inn => vec![block(size, size * 0.75, 7.2)],
        P::Shrine => vec![(What::Stone, bx(wp.pos, wp.rot, 4.0, 4.0), 3.8, true)],
        // A platform on four posts: you walk under it.
        P::GuardPost => vec![(What::Shelter, bx(wp.pos, wp.rot, 3.4, 3.4), 6.0, false)],
        P::HealingHouse | P::HealersHouse => vec![block(size, size * 0.7, 3.6)],
        P::TeachingHouse | P::ExchangeHouse | P::LettersHouse => vec![block(size, size * 0.7, 6.2)],
        // Lumps of growing stone in an arc behind the grower's bed (its
        // front, toward the town, is open).
        P::TendersYard => (0..5u64)
            .map(|i| {
                let a = wp.rot + std::f32::consts::PI + (i as f32 - 2.0) * 0.55;
                (What::Stone, Shape::Round { c: wp.pos.add(unit(a).scale(size * 0.5)), rot: 0.0, rx: 1.2, ry: 1.0 }, 1.0 + u(5 + i) * 0.8, true)
            })
            .collect(),
        // A fenced yard (part 0, hollow: `yard_gate`) and its fire, at the
        // back, clear of the stations across its middle.
        P::Workyard => vec![(What::Fence, bx(wp.pos, wp.rot, size, size), 0.8, true), (What::Fire, Shape::circle(at(-size * 0.3, 0.0), 0.6), 0.9, true)],
        P::Forge | P::Bench | P::Desk | P::AlchemyTable | P::WeaversShed | P::Workshop => vec![block(size * 0.8, size * 0.7, 3.0)],
    }
}

/// The gate of a fenced workyard: in the middle of the side that faces the town.
pub fn yard_gate(wp: &Workplace) -> Way {
    let along = unit(wp.rot);
    let half = wp.kind.size() * 0.5;
    let at = wp.pos.add(along.scale(half));
    Way { at, width: GATE, outside: at.add(along.scale(1.2)), inside: at.sub(along.scale(1.2)) }
}

/// A workyard's fence as solid stretches, its gate left open (the window
/// draws a rail along each).
pub fn yard_walls(wp: &Workplace) -> Vec<Rect> {
    let size = wp.kind.size();
    Thing::solid(Id::Place(0, 0, 0), What::Fence, Shape::Box(Rect::new(wp.pos, wp.rot, size, size)), 0.8).hollowed(FENCE, 4, Some(yard_gate(wp))).walls()
}

/// How big a crafting station standing in the open is: across, deep, tall
/// (as the window draws it).
pub fn station_size(st: Station) -> (f32, f32, f32) {
    match st {
        Station::Forge => (2.0, 1.6, 1.0),
        Station::Bench => (2.2, 0.9, 0.9),
        Station::Desk => (1.4, 0.8, 0.8),
        Station::AlchemyTable => (1.6, 0.9, 0.85),
        Station::Loom => (2.0, 0.5, 1.9),
        Station::Workbench => (2.0, 0.9, 0.85),
        Station::GrowerBed => (2.4, 1.6, 0.35),
        // (Never stand in the open as stations: a fire is its own solid, a
        // stove a hearth indoors.)
        Station::Campfire => (1.4, 1.4, 0.3),
        Station::Stove => (1.4, 1.0, 1.0),
    }
}

/// Where a workplace's crafting stations stand (`World::place_stations`
/// puts them there): side by side across its front, close enough to stay
/// inside a fenced yard.
pub fn station_spots(wp: &Workplace) -> Vec<(V2, Station)> {
    let sts = wp.kind.stations();
    let n = sts.len();
    let side = unit(wp.rot + std::f32::consts::FRAC_PI_2);
    let gap = if n > 1 { 4.0f32.min((wp.kind.size() - 4.0) / (n - 1) as f32) } else { 0.0 };
    sts.iter().enumerate().map(|(k, &st)| (wp.pos.add(side.scale((k as f32 - (n as f32 - 1.0) * 0.5) * gap)), st)).collect()
}

/// The turn the window draws a station at (from where it stands).
pub fn station_turn(at: V2) -> f32 {
    at.x * 0.13
}

/// How many spans a pen's fence has (a post between each), and its gate:
/// the first span, on its east side.
pub fn pen_fence(home: V2, radius: f32) -> (u8, Way) {
    let n = ((radius * std::f32::consts::TAU / 2.6).round() as usize).clamp(8, 40);
    let step = std::f32::consts::TAU / n as f32;
    let (p, q) = (home.add(unit(0.0).scale(radius)), home.add(unit(step).scale(radius)));
    let at = p.lerp(q, 0.5);
    let out = unit(step * 0.5);
    (n as u8, Way { at, width: p.dist(q) - 0.2, outside: at.add(out.scale(1.2)), inside: at.sub(out.scale(0.8)) })
}

/// A shellhen pen's coop: where it stands (on the side away from the
/// gate) and its turn.
pub fn coop_of(home: V2, radius: f32, seed: u64) -> Rect {
    Rect::new(home.sub(V2::new(radius * 0.45, 0.0)), (seed % 6283) as f32 / 1000.0, 2.4, 1.8)
}

/// A rock's outline: boulder, slab, pillar and outcrop by their kind (an
/// outcrop's leaning stones taken as one, a little bigger). Scree is a
/// heap of small stones you walk over.
pub fn rock_shape(r: &super::mapedit::Rock) -> (Shape, f32, bool) {
    let s = r.size;
    let (rx, ry, h, blocks) = match r.kind {
        0 => (s * 0.6, s * 0.5, s * 0.6, true),
        1 => (s * 0.75, s * 0.5, s * 0.25, true),
        2 => (s * 0.35, s * 0.3, s * 1.8, true),
        3 => (s * 1.3, s * 1.3, s * 0.4, false),
        _ => (s * 0.6 * 1.3, s * 0.45 * 1.3, s * 0.75, true),
    };
    (Shape::Round { c: r.pos, rot: r.rot, rx, ry }, h, blocks)
}

/// A base building as a thing (and, for a fenced plot, where its gate is).
pub fn built_thing(base: BaseId, bl: &base::Built) -> Option<Thing> {
    use base::Kind as K;
    if matches!(bl.state, base::State::Ruin) {
        return None;
    }
    let d = bl.def();
    let id = Id::Base(base, bl.id);
    // (A footprint's `w` runs across its facing, its depth along it.)
    let shape = Shape::Box(Rect::new(bl.at, bl.rot, d.d, bl.w));
    Some(match d.kind {
        K::Marker => Thing::solid(id, What::Fire, Shape::circle(bl.at, 1.45), d.h),
        K::Field | K::Pen => {
            let f = unit(bl.rot);
            let at = bl.at.add(f.scale(d.d * 0.5));
            let way = Way { at, width: 1.6, outside: at.add(f.scale(1.2)), inside: at.sub(f.scale(1.2)) };
            Thing::solid(id, What::Fence, shape, d.h).hollowed(FENCE, 4, Some(way))
        }
        K::Wall => Thing::solid(id, What::Wall, shape, d.h),
        K::Gate => Thing::solid(id, What::Gate, shape, d.h).open(),
        K::Well => Thing::solid(id, What::Stone, shape, d.h),
        K::Shed => match d.station {
            Some(st) => Thing::solid(id, What::Station(st), shape, d.h),
            None => Thing::solid(id, What::Shelter, shape, d.h),
        },
        K::Shelter | K::Store | K::Kitchen | K::Tower => Thing::solid(id, What::Shelter, shape, d.h),
    })
}

/// The home of a forged town at `at`: its size and the variant it's drawn as.
fn forge_home_shape(h: &super::forge::Home) -> Option<Shape> {
    let seed = rng::key(&[h.at.x.to_bits() as u64, h.at.y.to_bits() as u64, 0x464F_5247]);
    let size = if h.eldest { 12.0 } else { 8.5 + (seed >> 50) as f32 / (1u64 << 14) as f32 * 3.0 };
    let v = layout::variant_for(BuildingKind::RoduroHome, size, seed)?;
    Some(Shape::Round { c: h.at, rot: h.rot, rx: v.half.0 * size, ry: v.half.1 * size })
}

/// Every physical thing of a building with a door: its outer walls with the
/// doorway, the door, its inner walls, furniture and containers. A pure
/// function of the door (`buildings::door_of`, `door_with`).
pub fn things_of_building(d: &Door) -> Vec<Thing> {
    let (t, i) = d.id;
    let id = Id::Building(t, i);
    let v = d.variant();
    let way = doorway_of(d);
    let mut out = Vec::new();
    out.push(Thing::solid(id, What::Building, shell_of(d), v.storeys as f32 * 3.0).hollowed(layout::WALL_IN, match d.outline() {
        Some(pts) => pts.len().min(255) as u8,
        None if d.round => 16,
        None => 4,
    }, Some(way)));
    out.push(Thing::solid(Id::Door(t, i), What::Door, Shape::Box(Rect::new(way.at.sub(unit(d.rot).scale(layout::WALL_IN * 0.5)), d.rot, layout::WALL_IN, DOOR)), 2.0).open().inside(id));
    for (what, r) in layout::footprints(d) {
        out.push(match what {
            layout::Thing::Furniture(k) => {
                let f = v.furniture[k].what;
                let th = Thing::solid(Id::Furniture(t, i, k as u8), What::Furniture(f), Shape::Box(r), f.size().2).inside(id);
                // A bedroll lies flat; a hanging screen is pushed aside.
                if matches!(f, Furn::Bedroll | Furn::Screen) { th.open() } else { th }
            }
            layout::Thing::Container(k) => Thing::solid(Id::Container(t, i, k as u8), What::Container(v.holders[k].what), Shape::Box(r), v.holders[k].what.size().2).inside(id),
            // (In a round building an inner wall is written out to the
            // outline's corners: it ends at the outer wall.)
            layout::Thing::Wall(k) => match inner_wall(d, r) {
                Some(r) => Thing::solid(Id::InnerWall(t, i, k as u8), What::Wall, Shape::Box(r), 2.6).inside(id),
                None => continue,
            },
            layout::Thing::Extra(_) => continue,
        });
    }
    if let Some(y) = v.yard {
        out.extend(things_of_yard(d, y));
    }
    out
}

/// A trade home's work yard (`layout::Yard`): its fence with the gate in
/// it, the gate, and what stands inside. Where the fence runs along the
/// house's wall it doubles it, which changes nothing.
pub fn things_of_yard(d: &Door, y: &'static layout::Yard) -> Vec<Thing> {
    let (t, i) = d.id;
    let id = Id::Yard(t, i);
    let shape = Shape::Outline { c: d.centre, rot: d.rot, hx: d.half.x, hy: d.half.y, pts: y.outline, grow: 0.0 };
    let way = home_yard_gate(d, y);
    let mut out = vec![Thing::solid(id, What::Fence, shape, y.fence).hollowed(FENCE, y.outline.len().min(255) as u8, Some(way))];
    let turn = (way.outside.y - way.at.y).atan2(way.outside.x - way.at.x);
    out.push(Thing::solid(Id::YardGate(t, i), What::Gate, Shape::Box(Rect::new(way.at, turn, FENCE, way.width)), y.fence * 0.8).open());
    for (k, p) in y.pieces.iter().enumerate() {
        let (w, dd, h) = p.what.size();
        out.push(Thing::solid(Id::YardPiece(t, i, k as u8), What::Furniture(p.what), Shape::Box(Rect::new(d.to_world(p.at), d.rot + p.rot, w, dd)), h).inside(id));
    }
    out
}

/// The gate in a yard's fence: on the fence, with a step outside and inside.
pub fn home_yard_gate(d: &Door, y: &layout::Yard) -> Way {
    let at = d.to_world(y.gate);
    let (_, q, inward) = layout::outline_nearest(y.outline, d.half.x, d.half.y, d.frame(at));
    let dir = unit(d.rot);
    let side = unit(d.rot + std::f32::consts::FRAC_PI_2);
    let at = d.from_frame(q);
    let inward = dir.scale(inward.x).add(side.scale(inward.y));
    Way { at, width: y.gate_width, outside: at.sub(inward.scale(1.2)), inside: at.add(inward.scale(0.8)) }
}

/// The crafting stations standing in a building's yard, if it has one.
pub fn yard_stations(d: &Door) -> Vec<(V2, Station)> {
    let Some(y) = d.variant().yard else { return Vec::new() };
    y.pieces.iter().filter_map(|p| Station::of_furniture(p.what).map(|st| (d.to_world(p.at), st))).collect()
}

impl World {
    /// Every physical thing of one town: its buildings and what's in them,
    /// and its workplaces.
    pub fn things_of_town(&self, t: SettlementId) -> Vec<Thing> {
        let s = &self.settlements[t as usize];
        let mut out = Vec::new();
        for i in 0..s.buildings.len() as u16 {
            let b = &s.buildings[i as usize];
            let id = Id::Building(t, i);
            let Some(d) = door_of(s, i) else {
                match b.kind {
                    // The stones round the town's fire.
                    BuildingKind::Hearth => out.push(Thing::solid(id, What::Fire, Shape::circle(b.pos, 2.9), 1.4)),
                    // A stilt home stands on its platform over the water.
                    _ => {
                        if let Some(v) = layout::variant_in(s, i) {
                            let (hx, hy) = (v.half.0 * b.size, v.half.1 * b.size);
                            let shape = match v.shape {
                                layout::Shape::Round => Shape::Round { c: b.pos, rot: b.rot, rx: hx, ry: hy },
                                layout::Shape::Rect => Shape::Box(Rect::new(b.pos, b.rot, hx * 2.0, hy * 2.0)),
                                layout::Shape::Poly(pts) => Shape::Outline { c: b.pos, rot: b.rot, hx, hy, pts, grow: 0.0 },
                            };
                            out.push(Thing::solid(id, What::Building, shape, 3.0).open());
                        }
                    }
                }
                continue;
            };
            out.extend(things_of_building(&d));
        }
        let Some(tl) = self.society.towns.get(t as usize) else { return out };
        for (j, wp) in tl.places.iter().enumerate() {
            let j = j as u16;
            if place_in_building(s, wp) {
                continue;
            }
            let yard = (wp.kind == PlaceKind::Workyard).then_some(Id::Place(t, j, 0));
            for (k, (what, shape, h, blocks)) in place_parts(wp).into_iter().enumerate() {
                let th = Thing::solid(Id::Place(t, j, k as u8), what, shape, h);
                let th = if blocks { th } else { th.open() };
                match (yard, k) {
                    (Some(_), 0) => {
                        let way = yard_gate(wp);
                        out.push(th.hollowed(FENCE, 4, Some(way)));
                        out.push(Thing::solid(Id::PlaceGate(t, j), What::Gate, Shape::Box(Rect::new(way.at, wp.rot, FENCE, way.width)), 0.8).open());
                    }
                    (Some(y), _) => out.push(th.inside(y)),
                    _ => out.push(th),
                }
            }
            // A place with a roof keeps its one station indoors (the place
            // itself stands for it); in a yard they stand in the open.
            let roofed = place_parts(wp).first().is_some_and(|p| p.0 == What::Shelter);
            if !roofed && geo::inland(wp.pos) >= 0.0 {
                for (k, (at, st)) in station_spots(wp).into_iter().enumerate() {
                    let (w, d, h) = station_size(st);
                    let th = Thing::solid(Id::Station(t, j, k as u8), What::Station(st), Shape::Box(Rect::new(at, station_turn(at), w, d)), h);
                    out.push(match yard {
                        Some(y) => th.inside(y),
                        None => th,
                    });
                }
            }
        }
        out
    }

    /// Every physical thing outside the towns: pens, camps, ruins and lairs,
    /// chests in the wild, bases, rocks, a forged town's homes.
    pub fn things_of_wild(&self) -> Vec<Thing> {
        let mut out = Vec::new();
        for p in &self.animals.pens {
            // (Raftbacks float off the shore: no fence.)
            if p.sp == Sp::Raftback {
                continue;
            }
            let (spans, way) = pen_fence(p.home, p.radius);
            let h = match p.sp {
                Sp::Shellhen => 0.5,
                Sp::Plodder => 1.2,
                _ => 1.0,
            };
            out.push(Thing::solid(Id::Pen(p.id), What::Fence, Shape::circle(p.home, p.radius), h).hollowed(FENCE, spans, Some(way)));
            out.push(Thing::solid(Id::PenGate(p.id), What::Gate, Shape::Box(Rect::new(way.at, (way.outside.y - way.at.y).atan2(way.outside.x - way.at.x), FENCE, way.width)), h).open());
            if p.sp == Sp::Shellhen {
                out.push(Thing::solid(Id::PenHut(p.id), What::Shelter, Shape::Box(coop_of(p.home, p.radius, p.seed)), 1.4).inside(Id::Pen(p.id)));
            }
        }
        // Salt Isle's compound: a palisade with one gate, always open (the
        // watch stands at it); and the briar patch, a fence with a gate.
        if let Some(isle) = self.island() {
            use super::island::{PALISADE_H, PALISADE_T};
            let way = isle.gate();
            for (k, r) in isle.palisade_walls().into_iter().enumerate() {
                out.push(Thing::solid(Id::Palisade(k as u8), What::Wall, Shape::Box(r), PALISADE_H));
            }
            out.push(Thing::solid(Id::PalisadeGate, What::Gate, Shape::Box(Rect::new(way.at, 0.0, way.width, PALISADE_T)), PALISADE_H * 0.8).open());
        }
        if let Some(isle) = self.island() {
            use super::island::{Spot, BRIAR_FENCE, BRIAR_RADIUS};
            let c = isle.spot(Spot::Briar);
            let (spans, way) = pen_fence(c, BRIAR_RADIUS);
            out.push(Thing::solid(Id::Briar, What::Fence, Shape::circle(c, BRIAR_RADIUS), BRIAR_FENCE).hollowed(FENCE, spans, Some(way)));
            out.push(Thing::solid(Id::BriarGate, What::Gate, Shape::Box(Rect::new(way.at, (way.outside.y - way.at.y).atan2(way.outside.x - way.at.x), FENCE, way.width)), BRIAR_FENCE).open());
        }
        for c in &self.camps {
            out.push(Thing::solid(Id::CampFire(c.group), What::Fire, Shape::circle(c.pos, 0.9), 0.7));
            for (k, &a) in super::encounters::CLEAR_OF_THE_BAND.iter().enumerate() {
                out.push(Thing::solid(Id::CampShelter(c.group, k as u8), What::Shelter, Shape::Box(Rect::new(c.pos.add(unit(a).scale(LEAN_TO_OUT)), a, 3.2, 2.4)), 1.6));
            }
        }
        for ru in &self.ruins {
            for (k, what, shape, h, blocks) in self.ruin_parts(ru) {
                let id = if matches!(ru.kind, RuinKind::Lair(_)) { Id::Lair(ru.id) } else { Id::Ruin(ru.id, k) };
                let th = Thing::solid(id, what, shape, h);
                out.push(if blocks { th } else { th.open() });
            }
        }
        for (id, c) in self.containers.range((containers::WILD, 0, 0)..=(containers::WILD, u16::MAX, u8::MAX)) {
            let (w, d, h) = c.what.size();
            out.push(Thing::solid(Id::Container(id.0, id.1, id.2), What::Container(c.what), Shape::Box(Rect::new(c.pos, c.rot, w, d)), h));
        }
        for b in &self.bases {
            out.extend(b.buildings.iter().filter_map(|bl| built_thing(b.id, bl)));
        }
        for r in self.terrain.edits.rocks.iter().chain(self.terrain.authored.rocks.iter()) {
            let (shape, h, blocks) = rock_shape(r);
            let th = Thing::solid(Id::Rock(rng::key(&[r.pos.x.to_bits() as u64, r.pos.y.to_bits() as u64, r.seed as u64])), What::Stone, shape, h);
            out.push(if blocks { th } else { th.open() });
        }
        // The test quarter's wall, and its gate (shut until the key opens it).
        if let Some(q) = &self.test_quarter {
            for (k, r) in q.walls().into_iter().enumerate() {
                out.push(Thing::solid(Id::Quarter(k as u8), What::Wall, Shape::Box(r), super::testquarter::WALL_H));
            }
            let gate = Thing::solid(Id::QuarterGate, What::Gate, Shape::Box(q.gate_leaf()), super::testquarter::WALL_H * 0.8);
            out.push(if q.open { gate.open() } else { gate });
        }
        if let Some(f) = &self.forge {
            for (k, h) in f.founding.homes.iter().enumerate() {
                if let Some(shape) = forge_home_shape(h) {
                    out.push(Thing::solid(Id::ForgeHome(k as u32), What::Building, shape, 4.0));
                }
            }
        }
        out
    }

    /// The standing parts of a ruin or lair (the window draws these): its
    /// number, what it is, its outline, how tall, and whether it's in the
    /// way. A ruin: up to fourteen stretches of broken wall round its floor
    /// (some fallen; none where a chest stands), a column standing and one
    /// fallen. A lair: a rock arch you walk in under.
    pub fn ruin_parts(&self, ru: &super::ruins::Ruin) -> Vec<(u8, What, Shape, f32, bool)> {
        let mut out = Vec::new();
        match ru.kind {
            RuinKind::Ruin => {
                let chests: Vec<Poly> = self
                    .containers
                    .range((containers::WILD, 0, 0)..=(containers::WILD, u16::MAX, u8::MAX))
                    .filter(|(_, c)| c.pos.dist(ru.pos) < 14.0)
                    .map(|(_, c)| Poly::new(&Rect::new(c.pos, c.rot, c.what.size().0, c.what.size().1).grow(BODY * 2.0 + 0.3).corners()))
                    .collect();
                let rr = |i: u64| ((ru.id as u64 * 977 + i * 131) % 100) as f32 / 100.0;
                for i in 0..14u64 {
                    // (A gap in the wall.)
                    if rr(i + 40) < 0.25 {
                        continue;
                    }
                    let a = i as f32 / 14.0 * std::f32::consts::TAU;
                    let at = ru.pos.add(V2::new(a.cos() * 9.0, a.sin() * 7.0));
                    let r = Rect::new(at, a + std::f32::consts::FRAC_PI_2, 3.6, 0.9);
                    if chests.iter().any(|c| c.overlaps(&Poly::new(&r.corners()), 0.0)) {
                        continue;
                    }
                    out.push((i as u8, What::Wall, Shape::Box(r), 0.4 + rr(i) * 3.2, true));
                }
                out.push((14, What::Stone, Shape::circle(ru.pos.add(V2::new(2.0, 1.0)), 0.55), 4.2, true));
                let (f0, f1) = (ru.pos.add(V2::new(-4.5, -2.5)), ru.pos.add(V2::new(-0.7, -1.3)));
                out.push((15, What::Stone, Shape::Box(Rect::new(f0.lerp(f1, 0.5), (f1.y - f0.y).atan2(f1.x - f0.x), f0.dist(f1), 0.5)), 0.9, true));
            }
            RuinKind::Lair(_) => out.push((0, What::Stone, Shape::Round { c: ru.pos.add(V2::new(0.0, -3.0)), rot: 0.0, rx: 6.0, ry: 4.5 }, 4.0, false)),
        }
        out
    }

    /// What comes and goes: standing torches while they burn, the squad's
    /// tent while someone sleeps under it, a resting band of wanderers'.
    /// None of them is in the way.
    pub fn passing_things(&self) -> Vec<Thing> {
        let mut out = Vec::new();
        for st in self.standing.iter().filter(|s| s.burning(self.time)) {
            let id = Id::Torch(rng::key(&[st.pos.x.to_bits() as u64, st.pos.y.to_bits() as u64, st.lit_at.to_bits()]));
            out.push(Thing::solid(id, What::Torch, Shape::circle(st.pos, 0.12), 1.8).open());
        }
        for (id, at) in self.tents() {
            out.push(Thing::solid(id, What::Tent, Shape::circle(at, TENT), TENT * 1.5).open());
        }
        for b in self.beds.iter().filter(|b| b.kind == super::bedding::BedKind::Bedroll) {
            out.push(Thing::solid(Id::Bedroll(b.id), What::Furniture(Furn::Bedroll), Shape::Box(Rect::new(b.pos, b.rot, 0.45, 1.0)), 0.15).open());
        }
        // The squad's campfires, laid until they burn out.
        for f in self.campfires.iter().filter(|f| self.time >= f.since && self.time < f.out_at) {
            out.push(Thing::solid(Id::SquadFire(f.id), What::Fire, Shape::circle(f.pos, super::campfire::FIRE_RADIUS), 0.5).open());
        }
        out
    }

    /// The tents standing now, and where: one by each squad member who
    /// carries a tent while someone near sleeps under it, and one by a
    /// band of wanderers stopped for a rest out of doors.
    pub fn tents(&self) -> Vec<(Id, V2)> {
        let mut out = Vec::new();
        for b in self.beds.iter().filter(|b| b.kind == super::bedding::BedKind::Tent) {
            out.push((Id::Tent(b.id), b.pos));
        }
        for g in &self.groups {
            if g.band == 1 && matches!(g.kind, super::group::Kind::Wanderer { .. }) && !g.is_moving(self.time) && !g.hostile && !self.lodging(g) {
                out.push((Id::GroupTent(g.id), g.pos.add(V2::new(3.5, 2.0))));
            }
        }
        out
    }
}

/// How far out from a camp's fire its lean-tos stand, and a tent's radius.
pub const LEAN_TO_OUT: f32 = 12.0;
pub const TENT: f32 = 2.2;

// ---- Asking ---------------------------------------------------------------------------

impl World {
    /// Every thing whose outline comes within `within` of `p`.
    pub fn solids_near(&self, p: V2, within: f32) -> Vec<Thing> {
        let mut out: Vec<Thing> = Vec::new();
        for set in self.solid_sets(p, within) {
            out.extend(set.things_near(p, within).into_iter().copied());
        }
        out.extend(self.passing_things().into_iter().filter(|t| t.shape.centre().dist(p) < t.shape.reach() + within));
        out
    }

    /// The thing at `p`: the smallest of those `p` is inside.
    pub fn solid_at(&self, p: V2) -> Option<Thing> {
        self.solids_near(p, 0.0).into_iter().filter(|t| t.shape.contains(p)).min_by(|a, b| a.shape.reach().total_cmp(&b.shape.reach()))
    }

    /// One thing by its id, as it stands now.
    pub fn solid(&self, id: Id) -> Option<Thing> {
        let town = match id {
            Id::Building(s, _) | Id::Door(s, _) | Id::InnerWall(s, _, _) | Id::Furniture(s, _, _) | Id::Place(s, _, _) | Id::PlaceGate(s, _) | Id::Station(s, _, _) => Some(s),
            Id::Yard(s, _) | Id::YardGate(s, _) | Id::YardPiece(s, _, _) => Some(s),
            Id::Container(s, _, _) if s != containers::WILD => Some(s),
            _ => None,
        };
        match town {
            Some(s) if (s as usize) < self.settlements.len() => self.town_solids(s).things.iter().find(|t| t.id == id).copied(),
            Some(_) => None,
            None => self.wild_solids().things.iter().find(|t| t.id == id).copied().or_else(|| self.passing_things().into_iter().find(|t| t.id == id)),
        }
    }

    /// The hollow thing (a building, a pen, a fenced yard) whose wall is
    /// round `p`: the smallest, if one stands inside another.
    pub fn enclosure_at(&self, p: V2) -> Option<Thing> {
        enclosure(&self.solid_sets(p, 0.0), p)
    }

    /// Can a walker `body` wide stand at `p`? Out of doors: clear of every
    /// solid thing, a hollow one taken whole. Inside a hollow thing: on its
    /// floor (a walker's width in from its wall) and clear of what stands
    /// in it. The way-finder goes by the same rule (`walk.rs`).
    pub fn can_stand(&self, p: V2, body: f32) -> bool {
        can_stand_in(&self.solid_sets(p, body), p, body)
    }

    /// The thing a walker `body` wide can't stand at `p` for, if any.
    pub fn blocked_at(&self, p: V2, body: f32) -> Option<Id> {
        self.solid_sets(p, body).iter().find_map(|set| set.blocker_at(p, body).map(|pc| set.things[pc.thing as usize].id))
    }

    /// Can a walker `body` wide go straight from `a` to `b`? If not, what
    /// is in the way (the first found, not the nearest).
    pub fn in_the_way(&self, a: V2, b: V2, body: f32) -> Option<Id> {
        let mid = a.lerp(b, 0.5);
        self.solid_sets(mid, a.dist(b) * 0.5 + body).iter().find_map(|set| set.crossing(a, b, body).map(|pc| set.things[pc.thing as usize].id))
    }

    pub fn way_clear(&self, a: V2, b: V2, body: f32) -> bool {
        self.in_the_way(a, b, body).is_none()
    }

    /// `p`, or the nearest spot beside whatever `p` is inside, where a
    /// walker can stand (`can_stand`). (A spot hemmed in on every side is
    /// given up on after a few tries: the nearest clear spot on a widening
    /// ring. A spot inside a building stays inside it, one outside stays out.
    /// Indoors it must also be one a way in by the door gets to: a spot boxed
    /// in between beds and a chest is swapped for the nearest that isn't.)
    pub fn free_spot(&self, p: V2) -> V2 {
        self.spot_kept(p, None).0.unwrap_or(p)
    }

    /// `free_spot`, and how far (up to `ROOM_MOST`) a walker standing there
    /// could step any way and still stand: how much a figure idling there
    /// may drift.
    pub fn free_spot_room(&self, p: V2) -> (V2, f32) {
        let (s, room) = self.spot_kept(p, None);
        (s.unwrap_or(p), room)
    }

    /// Like `free_spot`, but only a spot within `within` of `near` will do
    /// (where to stand to open a chest that has beds packed round it).
    /// None: there's no such spot.
    pub fn stand_by(&self, p: V2, near: V2, within: f32) -> Option<V2> {
        self.spot_kept(p, Some((near, within))).0
    }

    /// `free_spot` (with its room) or `stand_by`, kept (`Cache`) until what
    /// stands near `p` changes.
    fn spot_kept(&self, p: V2, near: Option<(V2, f32)>) -> (Option<V2>, f32) {
        let (n, r) = near.unwrap_or((p, f32::INFINITY));
        let key = [p.x.to_bits(), p.y.to_bits(), n.x.to_bits(), n.y.to_bits(), r.to_bits()];
        // The usual ask: one look under the lock, nothing built.
        let q = self.quick_mark();
        let hit = {
            let kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
            kept.mark_near(self, q, p, 10.0, None).and_then(|m| kept.spots.get(&key).filter(|s| s.0 == m).map(|s| (m, (s.1, s.2))))
        };
        if let Some((m, s)) = hit {
            self.check_mark_near(p, 10.0, m);
            return s;
        }
        let (sets, mark) = self.solid_sets_marked(p, 10.0);
        if let Some(s) = self.solid_cache.spot(key, mark) {
            return s;
        }
        let s = match near {
            None => {
                let s = free_spot_in(&sets, p, BODY);
                (Some(s), room_at(&sets, s, BODY))
            }
            Some((n, r)) => (stand_by_in(&sets, p, n, r, BODY), 0.0),
        };
        self.solid_cache.keep_spot(key, mark, s.0, s.1);
        s
    }
}

/// The hollow thing whose wall is round `p`, if any: the smallest.
pub fn enclosure(sets: &[Arc<Solids>], p: V2) -> Option<Thing> {
    let mut best: Option<Thing> = None;
    for set in sets {
        for t in set.things_near(p, 0.0) {
            if t.hollow.is_some() && t.blocks && t.shape.contains(p) && best.is_none_or(|b| t.shape.reach() < b.shape.reach()) {
                best = Some(*t);
            }
        }
    }
    best
}

/// `World::can_stand`, among these lists.
pub fn can_stand_in(sets: &[Arc<Solids>], p: V2, body: f32) -> bool {
    better_spot(sets, p, body).is_none()
}

/// If a walker can't stand at `p`, a spot one step nearer to where they
/// can: out of the thing `p` is in, or in off a wall. None: `p` is fine.
fn better_spot(sets: &[Arc<Solids>], p: V2, body: f32) -> Option<V2> {
    let within = enclosure(sets, p);
    if let Some(th) = &within {
        let floor = th.floor(body);
        if !floor.contains(p) {
            return Some(floor.pulled_in(p));
        }
    }
    let room = within.map(|t| t.id);
    for set in sets {
        // (The first in the list, so the same spot is always moved the same way.)
        let mut hit: Option<u32> = None;
        set.cells(V2::new(p.x - body, p.y - body), V2::new(p.x + body, p.y + body), |c| {
            for &k in &c.1 {
                let pc = &set.pieces[k as usize];
                let counts = match room {
                    None => pc.room.is_none() && pc.part != Part::Wall,
                    Some(r) => pc.room == Some(r) && pc.part == Part::Body,
                };
                if counts && pc.poly.contains(p, body - HAIR) && hit.is_none_or(|h| k < h) {
                    hit = Some(k);
                }
            }
        });
        if let Some(k) = hit {
            return Some(set.pieces[k as usize].poly.pushed_out(p, body + 0.02));
        }
    }
    None
}

/// `World::free_spot`, among these lists.
pub fn free_spot_in(sets: &[Arc<Solids>], p: V2, body: f32) -> V2 {
    spot_where(sets, p, body, None).unwrap_or_else(|q| q)
}

/// Most room round a spot that `World::free_spot_room` looks for, metres.
pub const ROOM_MOST: f32 = 3.0;

/// How far a walker `body` wide standing at `p` could step any way and
/// still stand (`can_stand`), up to `ROOM_MOST`: 0 if they can't stand
/// there. (Measured from each thing's edges, so never more than the truth.)
pub fn room_at(sets: &[Arc<Solids>], p: V2, body: f32) -> f32 {
    let within = enclosure(sets, p);
    let mut room = ROOM_MOST;
    if let Some(th) = &within {
        room = room.min(th.floor(body).margin(p));
    }
    let id = within.map(|t| t.id);
    // (Far enough out that a thing's sharp corner, nearer by its edges than
    // by a straight line, is still looked at.)
    let r = (ROOM_MOST + body) * 1.5;
    for set in sets {
        for pc in set.pieces_in(V2::new(p.x - r, p.y - r), V2::new(p.x + r, p.y + r), 0.0) {
            let counts = match id {
                None => pc.room.is_none() && pc.part != Part::Wall,
                Some(i) => pc.room == Some(i) && pc.part == Part::Body,
            };
            if counts {
                room = room.min(pc.poly.depth(p) - body);
            }
        }
    }
    room.max(0.0)
}

/// `World::stand_by`, among these lists.
pub fn stand_by_in(sets: &[Arc<Solids>], p: V2, near: V2, within: f32, body: f32) -> Option<V2> {
    spot_where(sets, p, body, Some((near, within))).ok()
}

/// The nearest spot to `p` a walker can stand on and, indoors, that a way
/// in by the door gets to (and within `near.1` of `near.0`, if given).
/// Not found: the best tried (`free_spot`'s last resort).
fn spot_where(sets: &[Arc<Solids>], p: V2, body: f32, near: Option<(V2, f32)>) -> Result<V2, V2> {
    let home = enclosure(sets, p);
    let home_id = home.map(|t| t.id);
    // Indoors, a spot must also be one the way in gets to (`walk::Floor`).
    let floor = home.as_ref().and_then(|th| super::walk::Floor::of(sets, th, body));
    let close = |q: V2| near.is_none_or(|(n, r)| q.dist(n) <= r);
    let fine = |q: V2| close(q) && enclosure(sets, q).map(|t| t.id) == home_id && floor.as_ref().is_none_or(|f| f.gets_to(q));
    let mut q = p;
    for _ in 0..8 {
        match better_spot(sets, q, body) {
            None if fine(q) => return Ok(q),
            None => break,
            Some(next) => q = next,
        }
    }
    // (No further out than could still be near enough.)
    let reach = near.map(|(n, r)| p.dist(n) + r).unwrap_or(f32::MAX);
    // (Near a given spot, the rings are finer and closer: a chest's only
    // standing room may be a narrow strip beside it.)
    let (step, gap) = if near.is_some() { (0.15, 0.1) } else { (0.4, f32::MAX) };
    for ring in 1..=(9.6 / step) as usize {
        let r = ring as f32 * step;
        if r > reach {
            break;
        }
        let n = ((r * std::f32::consts::TAU / gap).ceil() as usize).max(16);
        for k in 0..n {
            let c = p.add(unit(k as f32 / n as f32 * std::f32::consts::TAU).scale(r));
            if better_spot(sets, c, body).is_none() && fine(c) {
                return Ok(c);
            }
        }
    }
    Err(q)
}
