//! Finding a way on foot: round whatever is solid (`solid.rs`), and in and
//! out of buildings, pens and fenced yards only by their doors and gates.
//!
//! One finder for everyone who walks: the squad's orders, townsfolk going
//! about their day, the watch on a chase, fighters in a fight. A way is a
//! pure function of where things stand: asked again, at any time, by anyone,
//! it comes out the same (rules 3 and 16), so a walk looked up from the clock
//! needs nothing stored. Ways already found are kept (`solid::Cache`) for as
//! long as nothing they were found among has changed.
//!
//! **How.** A walker is `solid::BODY` wide each side. The shortest way round
//! convex things turns only at their corners, so the finder looks for the
//! shortest chain of straight, clear stretches from corner to corner (kept
//! `CORNER` off each), trying the most promising first and only checking a
//! stretch is clear when it comes to use it. Out of doors a hollow thing is
//! one solid block to go round; a walk that starts or ends inside one goes
//! through its opening (just outside, then just inside), and is found among
//! the things inside it. A spot that can't be stood on, or that is boxed in
//! indoors so no way in reaches it (`Floor`), is swapped for the nearest
//! that can (`World::free_spot`); if no way gets there, the walk ends at the
//! nearest spot one does reach, and says so (`Found::reached`). Where
//! things stand close, a way turns tighter than `CORNER` (`CLOSER`), so a
//! gap a hair wider than a walker is still used.

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;

use super::geo::V2;
use super::solid::{enclosure, free_spot_in, Id, Part, Poly, Solids, Thing, BODY};
use super::world::World;

/// How far off a corner a way turns, on top of a walker's width.
pub const CORNER: f32 = 0.06;
/// Shares of `CORNER` a way may also turn at where something else stands
/// within `CORNER` of a corner (furniture packed close). (No finer: far
/// from the map's corner a position is good to about 2 mm.)
const CLOSER: [f32; 3] = [1.0, 0.25, 0.1];
/// How much ground round a walk is searched at first (a share of its
/// length, at least `ROUND_MIN` metres), and at most.
const ROUND: f32 = 0.6;
const ROUND_MIN: f32 = 14.0;
const ROUND_MAX: f32 = 260.0;

/// A way found.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    /// The points to walk to in turn, after the start; the last is where
    /// the walk ends. Empty: it ends where it starts.
    pub way: Vec<V2>,
    /// Does it end where it was asked to (or the nearest spot to stand
    /// beside that)? False: no way there; it ends as near as one gets.
    pub reached: bool,
    /// The door or gate of a building it goes in by (the last, if several).
    pub enters: Option<Id>,
    /// Where it would wait for that door: just outside it. (How many of
    /// `way`'s points get it that far.)
    pub at_door: usize,
}

/// One thing in the way, for a search.
struct Block {
    poly: Poly,
    /// A circle round it (middle and reach), for a quick no: grown by a
    /// margin `m`, it reaches no further than `r + m * spike` from `c`.
    c: V2,
    r: f32,
    /// How much further than the margin its sharpest corner sticks out
    /// when grown (a box's: the square root of 2).
    spike: f32,
}

fn seg_dist(p: V2, a: V2, b: V2) -> f32 {
    let d = b.sub(a);
    let l2 = d.x * d.x + d.y * d.y;
    let t = if l2 > 0.0 { (((p.x - a.x) * d.x + (p.y - a.y) * d.y) / l2).clamp(0.0, 1.0) } else { 0.0 };
    p.dist(a.add(d.scale(t)))
}

fn block(poly: Poly) -> Block {
    let c = poly.centre();
    let r = poly.corners().iter().map(|q| q.dist(c)).fold(0.0, f32::max);
    let spike = poly.grown(1.0).corners().iter().zip(poly.corners()).map(|(g, q)| g.dist(*q)).fold(1.0, f32::max);
    Block { poly, c, r, spike }
}

impl Block {
    /// Is `p` too far from it to be within `m` of it?
    fn far(&self, p: V2, m: f32) -> bool {
        self.c.dist(p) >= self.r + m * self.spike
    }
}

fn clear(blocks: &[Block], a: V2, b: V2, body: f32) -> bool {
    // (Always taken from the same end, so a stretch is clear both ways or neither.)
    let (a, b) = if (a.x, a.y) <= (b.x, b.y) { (a, b) } else { (b, a) };
    !blocks.iter().any(|k| seg_dist(k.c, a, b) < k.r + body * k.spike && k.poly.crossed(a, b, body))
}

fn stands(blocks: &[Block], p: V2, body: f32) -> bool {
    !blocks.iter().any(|k| !k.far(p, body) && k.poly.contains(p, body - 1e-3))
}

/// A stretch waiting to be tried: from `from` on to `to`.
#[derive(PartialEq)]
struct Try {
    /// How long the whole way would be at best if this stretch is clear.
    f: f32,
    g: f32,
    to: u32,
    from: u32,
}

impl Eq for Try {}
impl Ord for Try {
    fn cmp(&self, o: &Try) -> Ordering {
        // (The heap gives the greatest: the shortest first, then the
        // lowest numbers, so equal ways are chosen the same every time.)
        o.f.total_cmp(&self.f).then(o.to.cmp(&self.to)).then(o.from.cmp(&self.from))
    }
}
impl PartialOrd for Try {
    fn partial_cmp(&self, o: &Try) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// The shortest way from `a` to `b` among `blocks`, turning only at
/// `corners`, for a walker `body` wide: the points after `a`, and whether it
/// gets to `b`. If it can't, the way to the corner nearest `b` it does reach.
fn shortest(blocks: &[Block], corners: &[V2], a: V2, b: V2, body: f32) -> (Vec<V2>, bool) {
    if clear(blocks, a, b, body) {
        return (vec![b], true);
    }
    // Points: 0 is a, 1 is b, then the corners.
    let pt = |k: u32| match k {
        0 => a,
        1 => b,
        k => corners[k as usize - 2],
    };
    let n = corners.len() as u32 + 2;
    let mut came: Vec<Option<(u32, f32)>> = vec![None; n as usize];
    came[0] = Some((0, 0.0));
    let mut heap = BinaryHeap::new();
    let push_from = |heap: &mut BinaryHeap<Try>, came: &[Option<(u32, f32)>], from: u32, g: f32| {
        let p = pt(from);
        for to in 1..n {
            if came[to as usize].is_none() {
                let q = pt(to);
                let g2 = g + p.dist(q);
                heap.push(Try { f: g2 + q.dist(b), g: g2, to, from });
            }
        }
    };
    push_from(&mut heap, &came, 0, 0.0);
    while let Some(t) = heap.pop() {
        if came[t.to as usize].is_some() || !clear(blocks, pt(t.from), pt(t.to), body) {
            continue;
        }
        came[t.to as usize] = Some((t.from, t.g));
        if t.to == 1 {
            break;
        }
        push_from(&mut heap, &came, t.to, t.g);
    }
    let reached = came[1].is_some();
    // Not there: as near as any corner reached gets (or stay put).
    let end = if reached {
        1
    } else {
        (0..n).filter(|&k| k != 1 && came[k as usize].is_some()).min_by(|&x, &y| pt(x).dist(b).total_cmp(&pt(y).dist(b)).then(x.cmp(&y))).unwrap_or(0)
    };
    let mut way = Vec::new();
    let mut k = end;
    while k != 0 {
        way.push(pt(k));
        k = came[k as usize].map(|c| c.0).unwrap_or(0);
    }
    way.reverse();
    (way, reached)
}

/// The things in the way of a walk from `a` to `b`, `round` metres round
/// it, and the corners a way could turn at. Out of doors (`inside` None):
/// every solid thing standing in the open, a hollow one as a whole. Inside
/// a hollow thing: what stands in it, all of its floor whatever the walk;
/// its own wall is the edge of the ground.
fn ground(sets: &[Arc<Solids>], inside: Option<&Thing>, a: V2, b: V2, round: f32, body: f32) -> (Vec<Block>, Vec<V2>) {
    let (lo, hi) = match inside {
        Some(th) => {
            let (c, r) = (th.shape.centre(), th.shape.reach());
            (V2::new(c.x - r, c.y - r), V2::new(c.x + r, c.y + r))
        }
        None => (V2::new(a.x.min(b.x) - round, a.y.min(b.y) - round), V2::new(a.x.max(b.x) + round, a.y.max(b.y) + round)),
    };
    let mut blocks = Vec::new();
    for set in sets {
        for p in set.pieces_in(lo, hi, body) {
            let ok = match inside {
                None => p.room.is_none() && p.part != Part::Wall,
                Some(th) => p.room == Some(th.id) && p.part == Part::Body,
            };
            if ok {
                blocks.push(block(p.poly));
            }
        }
    }
    // Where the ground ends, indoors: a walker's width in from the wall.
    let floor = inside.map(|th| th.floor(body));
    // (Where a corner has less than `CORNER` to spare before something else,
    // or the wall, the way may need to turn tighter there.)
    let roomy_floor = inside.map(|th| th.floor(body + CORNER));
    let mut corners = Vec::new();
    for (i, k) in blocks.iter().enumerate() {
        let rings = CLOSER.map(|m| k.poly.grown(body + CORNER * m));
        for j in 0..k.poly.corners().len() {
            for ring in &rings {
                let c = ring.corners()[j];
                if c.x < lo.x || c.y < lo.y || c.x > hi.x || c.y > hi.y {
                    break;
                }
                if floor.is_none_or(|f| f.contains(c)) && stands(&blocks, c, body) {
                    corners.push(c);
                }
                let roomy = roomy_floor.is_none_or(|f| f.contains(c)) && blocks.iter().enumerate().all(|(o, b)| o == i || b.far(c, body + CORNER) || !b.poly.contains(c, body + CORNER - 1e-3));
                if roomy {
                    break;
                }
            }
        }
    }
    (blocks, corners)
}

/// The way from `a` to `b` on one kind of ground (both out of doors, or
/// both inside `inside`), searching wider if the first look finds none.
fn across(sets: &[Arc<Solids>], inside: Option<&Thing>, a: V2, b: V2, body: f32) -> (Vec<V2>, bool) {
    if a.dist(b) < 1e-3 {
        return (Vec::new(), true);
    }
    let mut round = (a.dist(b) * ROUND).clamp(ROUND_MIN, ROUND_MAX);
    loop {
        let (blocks, corners) = ground(sets, inside, a, b, round, body);
        let (way, reached) = shortest(&blocks, &corners, a, b, body);
        if reached || round >= ROUND_MAX || inside.is_some() {
            return (way, reached);
        }
        round = (round * 4.0).min(ROUND_MAX);
    }
}

/// The floor of a hollow thing as the finder sees it, and the corners on it
/// that the way in from its opening gets to: for telling a spot a walker
/// can get to from one boxed in by what stands round it (between two beds,
/// a chest at their heads and a wall at their feet).
pub struct Floor {
    blocks: Vec<Block>,
    /// The point just inside the opening, then every corner reached from it.
    reached: Vec<V2>,
    body: f32,
}

impl Floor {
    /// The floor of `th` for a walker `body` wide. None: it has no opening.
    pub fn of(sets: &[Arc<Solids>], th: &Thing, body: f32) -> Option<Floor> {
        let door = th.way()?.inside;
        let (blocks, corners) = ground(sets, Some(th), door, door, 0.0, body);
        let mut got = vec![false; corners.len()];
        let mut reached = vec![door];
        let mut k = 0;
        while k < reached.len() {
            let p = reached[k];
            for (j, &c) in corners.iter().enumerate() {
                if !got[j] && clear(&blocks, p, c, body) {
                    got[j] = true;
                    reached.push(c);
                }
            }
            k += 1;
        }
        Some(Floor { blocks, reached, body })
    }

    /// Does a way in from the opening get to `p` (a spot a walker can
    /// stand on)? The same answer as the finder's, by the same ground.
    pub fn gets_to(&self, p: V2) -> bool {
        self.reached.iter().any(|&c| clear(&self.blocks, c, p, self.body))
    }
}

/// How near the first turn of a way can be and be passed by, if there's a
/// straight way on to the turn after it, metres.
const PAST: f32 = 1.5;

/// The way from `a` to `b` on foot for a walker `body` wide: both must be
/// spots a walker can stand on (`World::free_spot`).
pub fn find(sets: &[Arc<Solids>], a: V2, b: V2, body: f32) -> Found {
    let mut f = find_turns(sets, a, b, body);
    // On the doorstep already, or a step round a corner: straight on from
    // where they stand, so someone finding their way afresh each step (the
    // watch, a fighter) doesn't step back to the turn each time.
    while f.way.len() >= 2 && f.way[0].dist(a) < PAST && sets.iter().all(|s| s.crossing(a, f.way[1], body).is_none()) {
        f.way.remove(0);
        f.at_door = f.at_door.saturating_sub(1);
    }
    f
}

fn find_turns(sets: &[Arc<Solids>], a: V2, b: V2, body: f32) -> Found {
    let (from, to) = (enclosure(sets, a), enclosure(sets, b));
    let mut out = Found { way: Vec::new(), reached: true, enters: None, at_door: 0 };
    if from.map(|t| t.id) == to.map(|t| t.id) {
        let (way, reached) = across(sets, from.as_ref(), a, b, body);
        return Found { way, reached, ..out };
    }
    let mut cur = a;
    // Out by its opening.
    if let Some(th) = &from {
        let Some(way) = th.way() else { return Found { reached: false, ..out } };
        let (w, reached) = across(sets, Some(th), cur, way.inside, body);
        out.way.extend(w);
        if !reached {
            return Found { reached: false, ..out };
        }
        out.way.push(way.outside);
        cur = way.outside;
    }
    // Across the open to the other's opening (or to `b` itself).
    let door = to.as_ref().and_then(|th| th.way());
    if to.is_some() && door.is_none() {
        return Found { reached: false, ..out };
    }
    let dest = door.map(|w| w.outside).unwrap_or(b);
    let (w, reached) = across(sets, None, cur, dest, body);
    out.way.extend(w);
    if !reached {
        return Found { reached: false, ..out };
    }
    // And in.
    if let (Some(th), Some(way)) = (&to, door) {
        out.enters = Some(th.id);
        out.at_door = out.way.len();
        out.way.push(way.inside);
        let (w, reached) = across(sets, Some(th), way.inside, b, body);
        out.way.extend(w);
        out.reached = reached;
    }
    out
}

/// Where someone at `a` stepping straight for `b` (a short step: a
/// fighter's, a tick at a time) ends up: at `b` if nothing solid is in the
/// way; else slid along the edge of what is, as far along the step as that
/// goes; else where they are. Something they're already within `body` of,
/// they may step away from or along, not further into.
pub fn slide(sets: &[Arc<Solids>], a: V2, b: V2, body: f32) -> V2 {
    let r = a.dist(b);
    if r < 1e-6 {
        return b;
    }
    let mut near = Vec::new();
    for set in sets {
        for pc in set.pieces_in(V2::new(a.x - r, a.y - r), V2::new(a.x + r, a.y + r), body) {
            if pc.part != Part::Hull {
                near.push(pc.poly);
            }
        }
    }
    let ok = |q: V2| {
        near.iter().all(|p| {
            let at = p.depth(a);
            if at < body - 1e-3 { p.depth(q) >= at - 1e-4 } else { !p.crossed(a, q, body) }
        })
    };
    if ok(b) {
        return b;
    }
    // Along an edge of whatever is in the way: the one keeping closest to
    // the step (the first such, so the same step slides the same way).
    let step = b.sub(a);
    let mut best: Option<(f32, V2)> = None;
    for p in &near {
        let cs = p.corners();
        for k in 0..cs.len() {
            let e = cs[(k + 1) % cs.len()].sub(cs[k]);
            let l = e.len();
            if l < 1e-4 {
                continue;
            }
            let along = (step.x * e.x + step.y * e.y) / l;
            if along.abs() < 1e-4 || best.is_some_and(|x| along.abs() <= x.0) {
                continue;
            }
            let q = a.add(e.scale(along / l));
            if ok(q) {
                best = Some((along.abs(), q));
            }
        }
    }
    best.map(|x| x.1).unwrap_or(a)
}

/// Does a way (from `a` through `way`) keep clear of everything solid?
/// What it runs into first, if not. (For checking ways; the finder doesn't
/// need it.)
pub fn runs_into(sets: &[Arc<Solids>], a: V2, way: &[V2], body: f32) -> Option<Id> {
    let mut p = a;
    for &q in way {
        for set in sets {
            if let Some(pc) = set.crossing(p, q, body) {
                return Some(set.things[pc.thing as usize].id);
            }
        }
        p = q;
    }
    None
}

/// Where a walker setting off from `a` along `way` is after `d` metres (at
/// its end, if it's shorter).
pub fn along(a: V2, way: &[V2], d: f32) -> V2 {
    let (mut at, mut left) = (a, d.max(0.0));
    for &q in way {
        let len = at.dist(q);
        if left <= len {
            return if len > 1e-6 { at.lerp(q, left / len) } else { q };
        }
        left -= len;
        at = q;
    }
    at
}

impl World {
    /// What's solid round a fight, for it to be fought on (`combat::Around`):
    /// every list with something near its fighters. Open ground if it has
    /// none yet.
    pub fn around_fight(&self, b: &super::combat::Battle) -> super::combat::Around {
        let n = b.fighters.len();
        if n == 0 {
            return super::combat::Around::default();
        }
        let c = b.fighters.iter().fold(V2::default(), |s, f| s.add(f.pos)).scale(1.0 / n as f32);
        let r = b.fighters.iter().map(|f| f.pos.dist(c)).fold(0.0, f32::max);
        super::combat::Around(Some(Arc::new(self.solid_sets(c, r + 60.0))))
    }

    /// Give every fight being stepped that has no ground yet (new, or just
    /// loaded) its ground, before it's advanced.
    pub(super) fn around_fights(&mut self) {
        for i in 0..self.battles.len() {
            if self.battles[i].around.0.is_none() {
                let g = self.around_fight(&self.battles[i]);
                self.battles[i].around = g;
            }
        }
    }

    /// Where someone at `a` heading for `b` is after `d` metres on foot,
    /// along the way (`find_way`; locks don't stop them: for the watch).
    pub fn step_toward(&self, a: V2, b: V2, d: f32) -> V2 {
        along(a, &self.find_way(a, b).way, d)
    }

    /// The way on foot from `a` to `b`, round everything solid and through
    /// doors and gates. An end that can't be stood on (in a counter, in a
    /// wall) is taken as the nearest spot beside it. A pure function of
    /// where things stand; ways found are kept until that changes.
    pub fn find_way(&self, a: V2, b: V2) -> Arc<Found> {
        let (sets, mark) = self.solid_sets_marked(a.lerp(b, 0.5), a.dist(b) * 0.5 + 40.0);
        let ends = [a.x.to_bits(), a.y.to_bits(), b.x.to_bits(), b.y.to_bits()];
        if let Some(f) = self.solid_cache.way(ends, mark) {
            return f;
        }
        let (from, to) = (free_spot_in(&sets, a, BODY), free_spot_in(&sets, b, BODY));
        let mut f = find(&sets, from, to, BODY);
        // (Out of whatever they were standing in first.)
        if from.dist(a) > 1e-3 {
            f.way.insert(0, from);
            f.at_door += 1;
        }
        let f = Arc::new(f);
        self.solid_cache.keep_way(ends, mark, f.clone());
        f
    }
}
