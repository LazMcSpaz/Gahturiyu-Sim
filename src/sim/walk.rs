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
//! the things inside it. A spot that can't be stood on is swapped for the
//! nearest that can (`World::free_spot`); if no way gets there, the walk
//! ends at the nearest spot one does reach, and says so (`Found::reached`).

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;

use super::geo::V2;
use super::solid::{enclosure, free_spot_in, Id, Part, Poly, Solids, Thing, BODY};
use super::world::World;

/// How far off a corner a way turns, on top of a walker's width.
pub const CORNER: f32 = 0.06;
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
    /// A circle round it (middle and reach), for a quick no.
    c: V2,
    r: f32,
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
    Block { poly, c, r }
}

fn clear(blocks: &[Block], a: V2, b: V2, body: f32) -> bool {
    !blocks.iter().any(|k| seg_dist(k.c, a, b) < k.r + body && k.poly.crossed(a, b, body))
}

fn stands(blocks: &[Block], p: V2, body: f32) -> bool {
    !blocks.iter().any(|k| k.c.dist(p) < k.r + body && k.poly.contains(p, body - 1e-3))
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
/// a hollow thing: what stands in it; its own wall is the edge of the ground.
fn ground(sets: &[Arc<Solids>], inside: Option<&Thing>, a: V2, b: V2, round: f32, body: f32) -> (Vec<Block>, Vec<V2>) {
    let (lo, hi) = (V2::new(a.x.min(b.x) - round, a.y.min(b.y) - round), V2::new(a.x.max(b.x) + round, a.y.max(b.y) + round));
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
    let mut corners = Vec::new();
    for k in &blocks {
        for &c in k.poly.grown(body + CORNER).corners() {
            if c.x < lo.x || c.y < lo.y || c.x > hi.x || c.y > hi.y {
                continue;
            }
            if floor.is_none_or(|f| f.contains(c)) && stands(&blocks, c, body) {
                corners.push(c);
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

/// The way from `a` to `b` on foot for a walker `body` wide: both must be
/// spots a walker can stand on (`World::free_spot`).
pub fn find(sets: &[Arc<Solids>], a: V2, b: V2, body: f32) -> Found {
    let f = find_(sets, a, b, body);
    if !f.reached && std::env::var("GAHT_WALK_DEBUG").is_ok() {
        let (from, to) = (enclosure(sets, a), enclosure(sets, b));
        eprintln!("NOWAY from {:?} in {:?} to {:?} in {:?}; got {} pts, last {:?}", a, from.map(|t| t.id), b, to.map(|t| t.id), f.way.len(), f.way.last());
        if let Some(th) = from { let w = th.way().unwrap(); let r = across(sets, Some(&th), a, w.inside, body); eprintln!("   out-leg reached {} ({} pts) inside pt {:?}", r.1, r.0.len(), w.inside); }
        if let Some(th) = to { let w = th.way().unwrap(); let r = across(sets, Some(&th), w.inside, b, body); eprintln!("   in-leg reached {} ({} pts)", r.1, r.0.len());
            let (blocks, corners) = ground(sets, Some(&th), w.inside, b, 14.0, body); eprintln!("   in-leg ground: {} blocks, {} corners; b stands {} ; inside stands {}", blocks.len(), corners.len(), stands(&blocks, b, body), stands(&blocks, w.inside, body)); }
    }
    f
}

fn find_(sets: &[Arc<Solids>], a: V2, b: V2, body: f32) -> Found {
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

impl World {
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
