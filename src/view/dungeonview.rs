//! Drawing a dungeon site (`sim/dungeon.rs`), plainly: Laz has put visual
//! work on hold, so this is only enough to see it and move in it. Floors at
//! their heights, ledge faces between them, walls up to ceilings over every
//! walkable cell (third person: the camera is inside, `thirdperson.rs`),
//! water surfaces, decks, ladders, doors, plaques, the chest and the exit.
//! The whole site is drawn (Laz: with the camera behind the character there's
//! no need to draw only what's in sight). Caverns (natural rock) and built
//! halls have their own colours. Drawing only.

use bevy::prelude::*;

use gahturiyu_sim::sim::dungeon::{Ground, ObjKind, Site};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::world::World;

use super::mesh::Builder;
use super::palette::{Rgb, TIMBER};

/// How high a ceiling stands over the highest floor, deck or water round
/// a cell (the generator's doors are up to 3.4 m tall, its arches 4.05 m).
pub const CEILING_UP: f32 = 4.2;

/// Built halls: dressed grey stone. Caverns: brown natural rock.
const HALL_WALL: Rgb = [0.40, 0.41, 0.44];
const HALL_FLOOR: Rgb = [0.55, 0.53, 0.49];
const HALL_CEILING: Rgb = [0.30, 0.30, 0.32];
const CAVE_WALL: Rgb = [0.27, 0.19, 0.12];
const CAVE_FLOOR: Rgb = [0.42, 0.34, 0.25];
const CAVE_CEILING: Rgb = [0.20, 0.15, 0.10];
const RUBBLE: Rgb = [0.34, 0.31, 0.27];
const PIT: Rgb = [0.07, 0.06, 0.06];
const WATER: Rgb = [0.16, 0.34, 0.48];
const DECK: Rgb = [0.48, 0.36, 0.22];
const IRON: Rgb = [0.30, 0.31, 0.33];
const PLAQUE: Rgb = [0.78, 0.66, 0.36];
const EXIT: Rgb = [0.40, 0.80, 0.50];
const DARK: Rgb = [0.015, 0.015, 0.015];

/// What a site's mesh was drawn from: rebuilt when this changes (a door, the
/// clearance).
pub fn key(s: &Site) -> u64 {
    let mut k = s.id as u64 ^ 0x9e37_79b9_7f4a_7c15;
    for o in &s.objects {
        k = k.wrapping_mul(31).wrapping_add(o.open as u64 + 2 * o.index as u64);
    }
    k ^ s.clearance.as_ref().map(|c| c.restored as u64 + 7).unwrap_or(0)
}

/// The highest floor, deck or water surface of a walkable cell.
fn top_level(s: &Site, x: i32, y: i32) -> Option<f32> {
    let c = s.cell(x, y)?;
    (c.ground != Ground::Wall).then(|| c.deck.unwrap_or(c.floor).max(c.water.unwrap_or(f32::MIN)))
}

/// The ceiling over a cell: `CEILING_UP` over the highest level in it and
/// round it (so ceilings step gently). A wall cell's is its top.
pub fn ceiling(s: &Site, x: i32, y: i32) -> f32 {
    let mut top: Option<f32> = None;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if let Some(h) = top_level(s, x + dx, y + dy) {
                top = Some(top.map_or(h, |t: f32| t.max(h)));
            }
        }
    }
    top.unwrap_or(0.0) + CEILING_UP
}

/// The top of a wall cell: up to the ceilings beside it.
pub fn wall_top(s: &Site, x: i32, y: i32) -> f32 {
    let mut top: Option<f32> = None;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if top_level(s, x + dx, y + dy).is_some() {
                let h = ceiling(s, x + dx, y + dy);
                top = Some(top.map_or(h, |t: f32| t.max(h)));
            }
        }
    }
    top.unwrap_or(CEILING_UP)
}

fn dim(c: Rgb, f: f32) -> Rgb {
    [c[0] * f, c[1] * f, c[2] * f]
}

/// The site's mesh, in world coordinates.
pub fn build(s: &Site) -> Builder {
    let mut b = Builder::new();
    let o = s.origin;
    let at = |x: f32, y: f32, h: f32| vec3(o.x + x, s.base_h + h, o.y + y);
    // What each cell's floor surface is drawn at (walls: their tops).
    let top = |x: i32, y: i32| -> f32 {
        let c = s.cell(x, y).unwrap();
        if c.ground == Ground::Wall {
            wall_top(s, x, y)
        } else {
            c.floor
        }
    };
    for y in 0..s.h as i32 {
        for x in 0..s.w as i32 {
            let c = *s.cell(x, y).unwrap();
            let (fx, fy) = (x as f32, y as f32);
            let wall = if c.natural { CAVE_WALL } else { HALL_WALL };
            let open_side = |dx: i32, dy: i32| s.cell(x + dx, y + dy).is_some_and(|n| n.ground != Ground::Wall);
            // Solid rock with nothing open beside it: never seen from inside.
            if c.ground == Ground::Wall && !(-1..=1).any(|dy| (-1..=1).any(|dx| open_side(dx, dy))) {
                continue;
            }
            let h = top(x, y);
            let col = match c.ground {
                Ground::Wall => wall,
                Ground::Floor => {
                    if c.natural {
                        CAVE_FLOOR
                    } else {
                        HALL_FLOOR
                    }
                }
                Ground::Rubble => RUBBLE,
                Ground::Pit | Ground::Water => PIT,
            };
            b.quad([at(fx, fy, h), at(fx + 1.0, fy, h), at(fx + 1.0, fy + 1.0, h), at(fx, fy + 1.0, h)], Vec3::Y, col);
            // The face down to each lower neighbour: a wall's face down to
            // the floor beside it, a ledge's down to the floor below.
            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let nh = match s.cell(x + dx, y + dy) {
                    Some(_) => top(x + dx, y + dy),
                    None => continue,
                };
                if nh >= h - 1e-3 {
                    continue;
                }
                let (a, e) = edge(fx, fy, dx, dy);
                let n = vec3(dx as f32, 0.0, dy as f32);
                let side = dim(if c.ground == Ground::Wall { wall } else { RUBBLE }, 0.8);
                b.quad([at(a.0, a.1, h), at(e.0, e.1, h), at(e.0, e.1, nh), at(a.0, a.1, nh)], n, side);
            }
            if c.ground == Ground::Wall {
                continue;
            }
            // The ceiling over a walkable cell, and the step between it and a
            // neighbour's higher one.
            let ceil = ceiling(s, x, y);
            let ccol = if c.natural { CAVE_CEILING } else { HALL_CEILING };
            b.quad([at(fx, fy, ceil), at(fx, fy + 1.0, ceil), at(fx + 1.0, fy + 1.0, ceil), at(fx + 1.0, fy, ceil)], -Vec3::Y, ccol);
            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                if !open_side(dx, dy) {
                    continue;
                }
                let nc = ceiling(s, x + dx, y + dy);
                if nc <= ceil + 1e-3 {
                    continue;
                }
                let (a, e) = edge(fx, fy, dx, dy);
                b.quad([at(a.0, a.1, ceil), at(e.0, e.1, ceil), at(e.0, e.1, nc), at(a.0, a.1, nc)], -vec3(dx as f32, 0.0, dy as f32), ccol);
            }
            if let Some(w) = c.water.filter(|w| *w > c.floor) {
                b.quad([at(fx, fy, w), at(fx + 1.0, fy, w), at(fx + 1.0, fy + 1.0, w), at(fx, fy + 1.0, w)], Vec3::Y, WATER);
            }
            if let Some(d) = c.deck {
                b.block(at(fx + 0.5, fy + 0.5, d - 0.12), 1.0, 1.0, 0.12, 0.0, DECK);
            }
        }
    }
    // Black well below the site, so nothing of the world shows through a gap.
    let low = s.lowest() - 2.0;
    let m = 300.0;
    let (w, hh) = (s.w as f32, s.h as f32);
    b.quad([at(-m, -m, low), at(w + m, -m, low), at(w + m, hh + m, low), at(-m, hh + m, low)], Vec3::Y, DARK);
    for ob in &s.objects {
        let p = ob.at;
        match ob.kind {
            ObjKind::Ladder => {
                let c = s.cell_at(p).unwrap();
                let (lo, hi) = (c.floor, c.deck.unwrap_or(c.floor + 2.4));
                for side in [-0.25f32, 0.25] {
                    b.stick(at(p.x + side, p.y - 0.3, lo), at(p.x + side, p.y - 0.3, hi + 0.9), 0.06, TIMBER);
                }
                let mut r = lo + 0.3;
                while r < hi + 0.8 {
                    b.stick(at(p.x - 0.25, p.y - 0.3, r), at(p.x + 0.25, p.y - 0.3, r), 0.04, TIMBER);
                    r += 0.3;
                }
            }
            ObjKind::Door => {
                let along = vec3(ob.axis[0] as f32, 0.0, ob.axis[1] as f32);
                let through = vec3(-along.z, 0.0, along.x);
                let half = ob.cells as f32 * 0.5;
                let col = if ob.style.contains("Iron") { IRON } else { TIMBER };
                let ht = ob.opening[1];
                let c = at(p.x, p.y, ob.z + ht * 0.5);
                if ob.open {
                    // Swung back against the jamb.
                    let hinge = c - along * half;
                    b.cuboid(hinge + through * (half * 0.5), through * (half * 0.5), Vec3::Y * (ht * 0.5), along * 0.04, col);
                } else {
                    b.cuboid(c, along * half, Vec3::Y * (ht * 0.5), through * 0.06, col);
                }
                // The wall over the doorway, up to the ceiling.
                let lintel = ceiling(s, p.x.floor() as i32, p.y.floor() as i32);
                let from = ob.z + ht;
                if lintel > from {
                    b.cuboid(at(p.x, p.y, (from + lintel) * 0.5), along * half, Vec3::Y * ((lintel - from) * 0.5), through * 0.5, HALL_WALL);
                }
            }
            ObjKind::Plaque | ObjKind::Memorial | ObjKind::Other => {
                b.block(at(p.x, p.y, ob.z), 0.45, 0.12, 0.7, 0.0, PLAQUE);
            }
            ObjKind::Chest => {
                b.block(at(p.x, p.y, ob.z), 0.9, 0.55, 0.5, 0.0, TIMBER);
                let lid = if ob.open { 0.3 } else { 0.0 };
                b.block(at(p.x, p.y + lid, ob.z + 0.5), 0.92, 0.57, 0.08, 0.0, IRON);
            }
            ObjKind::Exit => {
                b.column(at(p.x, p.y, ob.z), 0.35, 0.05, 2.2, 8, EXIT);
            }
        }
    }
    b
}

/// A cell's edge toward a neighbour: its two corners, in turn.
fn edge(fx: f32, fy: f32, dx: i32, dy: i32) -> ((f32, f32), (f32, f32)) {
    match (dx, dy) {
        (1, 0) => ((fx + 1.0, fy), (fx + 1.0, fy + 1.0)),
        (-1, 0) => ((fx, fy + 1.0), (fx, fy)),
        (0, 1) => ((fx + 1.0, fy + 1.0), (fx, fy + 1.0)),
        _ => ((fx, fy), (fx + 1.0, fy)),
    }
}

/// The squad's height in a site, for the camera: the first member in one.
pub fn ground(w: &World) -> Option<f32> {
    w.squad.members.iter().find_map(|&m| w.site_height(m))
}

/// Is a point (world) inside something solid of a site: a wall, below a
/// floor, above a ceiling? (For the camera, `thirdperson.rs`.) None if
/// it's outside every site.
pub fn solid_at(w: &World, p: Vec3) -> Option<bool> {
    let s = w.sites.sites.iter().find(|s| s.contains_world(V2::new(p.x, p.z)))?;
    let l = s.to_local(V2::new(p.x, p.z));
    let (cx, cy) = (l.x.floor() as i32, l.y.floor() as i32);
    let c = s.cell(cx, cy)?;
    let rel = p.y - s.base_h;
    if c.ground == Ground::Wall {
        return Some(true);
    }
    let lowest = s.nodes_in(cx, cy).into_iter().filter_map(|n| s.level(n)).map(|l| l.0).fold(c.floor, f32::min);
    Some(rel < lowest || rel > ceiling(s, cx, cy) - 0.05)
}

/// Where a ray first meets a site's floors, decks, water, walls or
/// ceilings, if it's a floor, deck, water or wall: the site, the spot (world
/// metres) and its world height. (Hitting a ceiling first is nothing.)
pub fn pick(w: &World, ray: (Vec3, Vec3)) -> Option<(u32, V2, f32)> {
    let (o, d) = ray;
    let s = w.sites.sites.iter().find(|s| s.contains_world(V2::new(o.x, o.z)))?;
    let mut prev: Option<f32> = None;
    let mut t = 0.0;
    while t < 80.0 {
        let p = o + d * t;
        let l = s.to_local(V2::new(p.x, p.z));
        let rel = p.y - s.base_h;
        let (cx, cy) = (l.x.floor() as i32, l.y.floor() as i32);
        let Some(c) = s.cell(cx, cy) else { return None };
        if c.ground == Ground::Wall {
            // A wall: the spot in front of it, at this height.
            return (rel < wall_top(s, cx, cy)).then(|| (s.id, s.to_world(l), s.base_h + rel));
        }
        if rel > ceiling(s, cx, cy) {
            return None;
        }
        if let Some(pr) = prev {
            let mut levels: Vec<f32> = s.nodes_in(cx, cy).into_iter().filter_map(|n| s.level(n)).map(|l| l.0).collect();
            if levels.is_empty() {
                levels.push(c.floor);
            }
            if let Some(h) = levels.into_iter().filter(|&h| pr > h && rel <= h).fold(None, |m: Option<f32>, h| Some(m.map_or(h, |m| m.max(h)))) {
                return Some((s.id, s.to_world(l), s.base_h + h));
            }
        }
        prev = Some(rel);
        t += 0.05;
    }
    None
}
