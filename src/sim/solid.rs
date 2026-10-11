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
//!   inside (a walk to a counter ends beside it).
//! - `World::enclosure_at(p)`: the building, pen or yard whose wall is round `p`.
//!
//! **How it's kept.** The list is a pure function of the world (nothing new
//! is saved). It's built a town at a time, and once for everything out in
//! the wild, and kept until what it was built from changes (`Cache`; a
//! cheap mark of the town's buildings and workplaces, or of the camps,
//! pens, bases and rocks, is checked on every ask). Tents and standing
//! torches come and go, so they are added fresh on each ask.

use std::collections::HashMap;
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
use super::person::PersonId;
use super::rng;
use super::ruins::RuinKind;
use super::settlement::{BuildingKind, SettlementId};
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
    /// A building at a base: the base, and the building's own id.
    Base(BaseId, u32),
    /// A rock placed by hand, named by where it stands.
    Rock(u64),
    /// A home of a forged town (its number in the founding).
    ForgeHome(u32),
    /// A standing torch, named by where and when it was lit.
    Torch(u64),
    /// The tent a squad member has put up, and a band of wanderers' tent.
    Tent(PersonId),
    GroupTent(GroupId),
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
}

impl Shape {
    pub fn circle(c: V2, r: f32) -> Shape {
        Shape::Round { c, rot: 0.0, rx: r, ry: r }
    }

    pub fn centre(&self) -> V2 {
        match self {
            Shape::Box(r) => r.c,
            Shape::Round { c, .. } => *c,
        }
    }

    /// How far from its middle it reaches at most.
    pub fn reach(&self) -> f32 {
        match self {
            Shape::Box(r) => (r.hw * r.hw + r.hd * r.hd).sqrt(),
            Shape::Round { rx, ry, .. } => rx.max(*ry) * ROUND_OUT,
        }
    }

    /// `p` in the shape's own frame: along its turn, and across.
    fn local(&self, p: V2) -> (f32, f32) {
        let (c, rot) = match self {
            Shape::Box(r) => (r.c, r.rot),
            Shape::Round { c, rot, .. } => (*c, *rot),
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
        }
    }

    /// The same shape, `m` bigger all round.
    pub fn grown(&self, m: f32) -> Shape {
        match *self {
            Shape::Box(r) => Shape::Box(r.grow(m)),
            Shape::Round { c, rot, rx, ry } => Shape::Round { c, rot, rx: rx + m, ry: ry + m },
        }
    }

    /// Its outline as corners: a box's four; for a round thing, twelve
    /// flat sides drawn just outside the curve, so nothing inside the
    /// outline is ever outside the thing.
    pub fn outline(&self) -> Poly {
        match self {
            Shape::Box(r) => Poly::new(&r.corners()),
            Shape::Round { .. } => Poly::new(&self.ring(ROUND_SIDES, ROUND_OUT)),
        }
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

    /// A hollow thing's wall as solid stretches, the opening left out.
    pub fn walls(&self) -> Vec<Rect> {
        let Some(h) = self.hollow else { return Vec::new() };
        let pts: Vec<V2> = match self.shape {
            Shape::Box(r) => r.corners().to_vec(),
            Shape::Round { .. } => self.shape.ring((h.spans as usize).max(8), 1.0),
        };
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
            let n = if dot(mid.sub(a), n) < 0.0 { n.scale(-1.0) } else { n };
            let mut piece = |from: f32, to: f32| {
                if to - from > 0.05 {
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
                put(Part::Hull, s.outlines[k], th.within);
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
pub struct Cache(Mutex<Kept>);

#[derive(Default)]
struct Kept {
    towns: Vec<Option<(u64, Arc<Solids>)>>,
    wild: Option<(u64, Arc<Solids>)>,
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

fn mark(h: u64, x: u64) -> u64 {
    (h ^ x).wrapping_mul(0x0000_0100_0000_01B3).rotate_left(23)
}

fn mark_v2(h: u64, p: V2) -> u64 {
    mark(h, (p.x.to_bits() as u64) << 32 | p.y.to_bits() as u64)
}

impl World {
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
        let wild = self.containers.range((containers::WILD, 0, 0)..=(containers::WILD, u16::MAX, u8::MAX));
        for (id, c) in wild {
            h = mark_v2(mark(h, (id.1 as u64) << 8 | id.2 as u64), c.pos);
        }
        h = mark(h, self.forge.as_ref().map(|f| f.founding.homes.len() as u64 + 1).unwrap_or(0));
        h
    }

    /// The list for one town, as it stands now.
    pub fn town_solids(&self, t: SettlementId) -> Arc<Solids> {
        let m = self.town_mark(t);
        let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        if kept.towns.len() <= t as usize {
            kept.towns.resize(t as usize + 1, None);
        }
        if let Some((have, s)) = &kept.towns[t as usize] {
            if *have == m {
                return s.clone();
            }
        }
        let s = Arc::new(Solids::new(self.things_of_town(t)));
        kept.towns[t as usize] = Some((m, s.clone()));
        s
    }

    /// The list for everything outside the towns, as it stands now.
    pub fn wild_solids(&self) -> Arc<Solids> {
        let m = self.wild_mark();
        let mut kept = self.solid_cache.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((have, s)) = &kept.wild {
            if *have == m {
                return s.clone();
            }
        }
        let s = Arc::new(Solids::new(self.things_of_wild()));
        kept.wild = Some((m, s.clone()));
        s
    }

    /// Every list with something within `within` of `p`.
    pub fn solid_sets(&self, p: V2, within: f32) -> Vec<Arc<Solids>> {
        let mut out = vec![self.wild_solids()];
        for s in &self.settlements {
            if s.pos.dist(p) < s.reach + within + 30.0 {
                out.push(self.town_solids(s.id));
            }
        }
        out
    }
}

// ---- Building the list -----------------------------------------------------------------

/// A building's outline from its door (`buildings.rs`).
pub fn shell_of(d: &Door) -> Shape {
    if d.round {
        Shape::Round { c: d.centre, rot: d.rot, rx: d.half.x, ry: d.half.y }
    } else {
        Shape::Box(Rect::new(d.centre, d.rot, d.half.x * 2.0, d.half.y * 2.0))
    }
}

/// The doorway in a building's outer wall: `inside` is the first step past
/// the wall (nearer than `Door::inside`, which may have a table by it).
pub fn doorway_of(d: &Door) -> Way {
    let dir = unit(d.rot);
    let at = d.outside.sub(dir.scale(1.2));
    Way { at, width: DOOR, outside: d.outside, inside: at.sub(dir.scale(layout::WALL_IN + BODY + 0.1)) }
}

/// The solid parts of a workplace, as the window draws them (`view/scene.rs`,
/// `workplace`): what each is, its outline, how tall, and whether it's in
/// the way. Part 0 is its main body. A place out on the water has none.
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
                            let shape = if v.shape == layout::Shape::Round { Shape::Round { c: b.pos, rot: b.rot, rx: hx, ry: hy } } else { Shape::Box(Rect::new(b.pos, b.rot, hx * 2.0, hy * 2.0)) };
                            out.push(Thing::solid(id, What::Building, shape, 3.0).open());
                        }
                    }
                }
                continue;
            };
            let v = d.variant();
            let way = doorway_of(&d);
            out.push(Thing::solid(id, What::Building, shell_of(&d), v.storeys as f32 * 3.0).hollowed(layout::WALL_IN, if d.round { 16 } else { 4 }, Some(way)));
            out.push(Thing::solid(Id::Door(t, i), What::Door, Shape::Box(Rect::new(way.at.sub(unit(d.rot).scale(layout::WALL_IN * 0.5)), d.rot, layout::WALL_IN, DOOR)), 2.0).open().inside(id));
            for (what, r) in layout::footprints(&d) {
                out.push(match what {
                    layout::Thing::Furniture(k) => {
                        let f = v.furniture[k].what;
                        let th = Thing::solid(Id::Furniture(t, i, k as u8), What::Furniture(f), Shape::Box(r), f.size().2).inside(id);
                        // A bedroll lies flat; a hanging screen is pushed aside.
                        if matches!(f, Furn::Bedroll | Furn::Screen) { th.open() } else { th }
                    }
                    layout::Thing::Container(k) => Thing::solid(Id::Container(t, i, k as u8), What::Container(v.holders[k].what), Shape::Box(r), v.holders[k].what.size().2).inside(id),
                    layout::Thing::Wall(k) => Thing::solid(Id::InnerWall(t, i, k as u8), What::Wall, Shape::Box(r), 2.6).inside(id),
                    layout::Thing::Extra(_) => continue,
                });
            }
        }
        let Some(tl) = self.society.towns.get(t as usize) else { return out };
        for (j, wp) in tl.places.iter().enumerate() {
            let j = j as u16;
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
        out
    }

    /// The tents standing now, and where: one by each squad member who
    /// carries a tent while someone near sleeps under it, and one by a
    /// band of wanderers stopped for a rest out of doors.
    pub fn tents(&self) -> Vec<(Id, V2)> {
        use super::condition::{Activity, Shelter};
        let mut out = Vec::new();
        let tent = super::items::id("tent");
        for (j, &m) in self.squad.members.iter().enumerate() {
            let at = self.squad.at[j];
            if !self.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == tent)) {
                continue;
            }
            let under = self.squad.members.iter().enumerate().any(|(i, &o)| self.squad.at[i].dist(at) < 15.0 && self.people[o as usize].cond.as_ref().is_some_and(|c| c.activity == Activity::Sleeping && c.shelter == Shelter::Tent));
            if under {
                out.push((Id::Tent(m), at.add(V2::new(2.6, 1.6))));
            }
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
        let mut best: Option<Thing> = None;
        for set in self.solid_sets(p, 0.0) {
            for t in set.things_near(p, 0.0) {
                if t.hollow.is_some() && t.shape.contains(p) && best.is_none_or(|b| t.shape.reach() < b.shape.reach()) {
                    best = Some(*t);
                }
            }
        }
        best
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
    /// walker can stand. (A spot hemmed in on every side is given up on
    /// after a few tries: the nearest clear spot on a widening ring.)
    pub fn free_spot(&self, p: V2) -> V2 {
        let sets = self.solid_sets(p, 8.0);
        let blocker = |q: V2| sets.iter().find_map(|set| set.blocker_at(q, BODY).map(|pc| pc.poly));
        let mut q = p;
        for _ in 0..6 {
            match blocker(q) {
                None => return q,
                Some(poly) => q = poly.pushed_out(q, BODY + 0.02),
            }
        }
        for ring in 1..=16 {
            let r = ring as f32 * 0.5;
            for k in 0..16 {
                let c = p.add(unit(k as f32 / 16.0 * std::f32::consts::TAU).scale(r));
                if blocker(c).is_none() {
                    return c;
                }
            }
        }
        p
    }
}
