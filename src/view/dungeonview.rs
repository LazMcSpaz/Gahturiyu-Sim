//! Drawing a dungeon site (`sim/dungeon.rs`), plainly: Laz has put visual
//! work on hold, so this is only enough to see it and click in it. Floors at
//! their heights, ledge faces between them, low cut-away walls (like a
//! building the squad is in), water surfaces, decks, ladders, doors, plaques,
//! the chest and the exit. Drawing only.
//!
//! What the squad sees (`World::update_sight`: 40 m, not past walls) is drawn
//! as it is; what it has seen before, darker; what it hasn't seen, black.
//! Caverns (natural rock) and built halls have their own colours.

use bevy::prelude::*;

use gahturiyu_sim::sim::dungeon::{Ground, ObjKind, Site};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::world::World;

use super::mesh::Builder;
use super::palette::{Rgb, TIMBER};

/// How far a wall stands above the highest floor beside it (cut away, so the
/// camera sees into the rooms).
pub const WALL_UP: f32 = 1.4;

/// Built halls: dressed grey stone. Caverns: brown natural rock.
const HALL_WALL: Rgb = [0.40, 0.41, 0.44];
const HALL_FLOOR: Rgb = [0.55, 0.53, 0.49];
const CAVE_WALL: Rgb = [0.27, 0.19, 0.12];
const CAVE_FLOOR: Rgb = [0.42, 0.34, 0.25];
const RUBBLE: Rgb = [0.34, 0.31, 0.27];
const UNSEEN: Rgb = [0.015, 0.015, 0.015];
/// What's been seen but isn't in sight now: this much of its colour.
const REMEMBERED: f32 = 0.35;
const PIT: Rgb = [0.07, 0.06, 0.06];
const WATER: Rgb = [0.16, 0.34, 0.48];
const DECK: Rgb = [0.48, 0.36, 0.22];
const IRON: Rgb = [0.30, 0.31, 0.33];
const PLAQUE: Rgb = [0.78, 0.66, 0.36];
const EXIT: Rgb = [0.40, 0.80, 0.50];

/// What a site's mesh was drawn from: rebuilt when this changes (a door,
/// the clearance, what's in sight).
pub fn key(s: &Site, sight: Option<&Vec<bool>>) -> u64 {
    let mut k = s.id as u64 ^ 0x9e37_79b9_7f4a_7c15;
    if let Some(v) = sight {
        for (i, &b) in v.iter().enumerate() {
            if b {
                k = k.wrapping_mul(0x100_0000_01b3).wrapping_add(i as u64 + 1);
            }
        }
    }
    for o in &s.objects {
        k = k.wrapping_mul(31).wrapping_add(o.open as u64 + 2 * o.index as u64);
    }
    k ^ s.clearance.as_ref().map(|c| c.restored as u64 + 7).unwrap_or(0)
}

/// The top of a wall cell: the highest floor or deck beside it plus `WALL_UP`;
/// solid rock with nothing beside it at the datum plus `WALL_UP`.
pub fn wall_top(s: &Site, x: i32, y: i32) -> f32 {
    let mut top: Option<f32> = None;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if let Some(c) = s.cell(x + dx, y + dy) {
                if c.ground != Ground::Wall {
                    let h = c.deck.unwrap_or(c.floor).max(c.water.unwrap_or(f32::MIN));
                    top = Some(top.map_or(h, |t: f32| t.max(h)));
                }
            }
        }
    }
    top.unwrap_or(0.0) + WALL_UP
}

fn dim(c: Rgb, f: f32) -> Rgb {
    [c[0] * f, c[1] * f, c[2] * f]
}

/// The site's mesh, in world coordinates, as far as the squad has seen it
/// (`sight`: what's in sight now; None: everything, as for a site nobody is in).
pub fn build(s: &Site, sight: Option<&Vec<bool>>) -> Builder {
    let mut b = Builder::new();
    let wd = s.w as usize;
    // 2: in sight; 1: seen before; 0: never.
    let state = |x: i32, y: i32| -> u8 {
        let i = y as usize * wd + x as usize;
        match sight {
            None => 2,
            Some(v) if v.get(i).copied().unwrap_or(false) => 2,
            Some(_) if s.seen.get(i).copied().unwrap_or(false) => 1,
            Some(_) => 0,
        }
    };
    let shade = |c: Rgb, st: u8| match st {
        2 => c,
        1 => dim(c, REMEMBERED),
        _ => UNSEEN,
    };
    let o = s.origin;
    let at = |x: f32, y: f32, h: f32| vec3(o.x + x, s.base_h + h, o.y + y);
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
            let h = top(x, y);
            let st = state(x, y);
            // Never seen: not drawn at all (the black below shows), so
            // nothing of its shape is given away.
            if st == 0 {
                continue;
            }
            let wall = if c.natural { CAVE_WALL } else { HALL_WALL };
            let floor = if c.natural { CAVE_FLOOR } else { HALL_FLOOR };
            let col = shade(
                match c.ground {
                    Ground::Wall => wall,
                    Ground::Floor => floor,
                    Ground::Rubble => RUBBLE,
                    Ground::Pit => PIT,
                    Ground::Water => PIT,
                },
                st,
            );
            b.quad([at(fx, fy, h), at(fx + 1.0, fy, h), at(fx + 1.0, fy + 1.0, h), at(fx, fy + 1.0, h)], Vec3::Y, col);
            // The face down to each lower neighbour (east and south, then
            // west and north, so every edge is drawn once from its high side).
            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let nh = match s.cell(x + dx, y + dy) {
                    Some(_) => top(x + dx, y + dy),
                    None => continue,
                };
                if nh >= h - 1e-3 {
                    continue;
                }
                let (a, e) = match (dx, dy) {
                    (1, 0) => ((fx + 1.0, fy), (fx + 1.0, fy + 1.0)),
                    (-1, 0) => ((fx, fy + 1.0), (fx, fy)),
                    (0, 1) => ((fx + 1.0, fy + 1.0), (fx, fy + 1.0)),
                    _ => ((fx, fy), (fx + 1.0, fy)),
                };
                let n = vec3(dx as f32, 0.0, dy as f32);
                let side = shade(dim(if c.ground == Ground::Wall { wall } else { RUBBLE }, 0.75), st);
                b.quad([at(a.0, a.1, h), at(e.0, e.1, h), at(e.0, e.1, nh), at(a.0, a.1, nh)], n, side);
            }
            if st == 0 {
                continue;
            }
            if let Some(w) = c.water.filter(|w| *w > c.floor) {
                b.quad([at(fx, fy, w), at(fx + 1.0, fy, w), at(fx + 1.0, fy + 1.0, w), at(fx, fy + 1.0, w)], Vec3::Y, shade(WATER, st));
            }
            if let Some(d) = c.deck {
                b.block(at(fx + 0.5, fy + 0.5, d - 0.12), 1.0, 1.0, 0.12, 0.0, shade(DECK, st));
            }
        }
    }
    // Black under the whole site and well round it, below its lowest floor:
    // what hasn't been seen, and the gaps between what has.
    let low = s.cells.iter().map(|c| c.floor).fold(0.0f32, f32::min) - 2.0;
    let m = 300.0;
    let (w, hh) = (s.w as f32, s.h as f32);
    b.quad([at(-m, -m, low), at(w + m, -m, low), at(w + m, hh + m, low), at(-m, hh + m, low)], Vec3::Y, UNSEEN);
    for ob in &s.objects {
        let p = ob.at;
        // Things are drawn where they've been seen (a door from either side).
        let st = [(0.0, 0.0), (0.5, 0.5), (-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5)]
            .iter()
            .map(|(ox, oy)| ((p.x + ox).floor() as i32, (p.y + oy).floor() as i32))
            .filter(|&(x, y)| s.cell(x, y).is_some())
            .map(|(x, y)| state(x, y))
            .max()
            .unwrap_or(0);
        if st == 0 {
            continue;
        }
        let k = if st == 2 { 1.0 } else { REMEMBERED };
        let (timber, iron) = (dim(TIMBER, k), dim(IRON, k));
        match ob.kind {
            ObjKind::Ladder => {
                let c = s.cell_at(p).unwrap();
                let (lo, hi) = (c.floor, c.deck.unwrap_or(c.floor + 2.4));
                for side in [-0.25f32, 0.25] {
                    b.stick(at(p.x + side, p.y - 0.3, lo), at(p.x + side, p.y - 0.3, hi + 0.9), 0.06, timber);
                }
                let mut r = lo + 0.3;
                while r < hi + 0.8 {
                    b.stick(at(p.x - 0.25, p.y - 0.3, r), at(p.x + 0.25, p.y - 0.3, r), 0.04, timber);
                    r += 0.3;
                }
            }
            ObjKind::Door => {
                let along = vec3(ob.axis[0] as f32, 0.0, ob.axis[1] as f32);
                let through = vec3(-along.z, 0.0, along.x);
                let half = ob.cells as f32 * 0.5;
                let col = if ob.style.contains("Iron") { iron } else { timber };
                let ht = ob.opening[1];
                let c = at(p.x, p.y, ob.z + ht * 0.5);
                if ob.open {
                    // Swung back against the jamb.
                    let hinge = c - along * half;
                    b.cuboid(hinge + through * (half * 0.5), through * (half * 0.5), Vec3::Y * (ht * 0.5), along * 0.04, col);
                } else {
                    b.cuboid(c, along * half, Vec3::Y * (ht * 0.5), through * 0.06, col);
                }
            }
            ObjKind::Plaque | ObjKind::Memorial | ObjKind::Other => {
                b.block(at(p.x, p.y, ob.z), 0.45, 0.12, 0.7, 0.0, dim(PLAQUE, k));
            }
            ObjKind::Chest => {
                b.block(at(p.x, p.y, ob.z), 0.9, 0.55, 0.5, 0.0, timber);
                let lid = if ob.open { 0.3 } else { 0.0 };
                b.block(at(p.x, p.y + lid, ob.z + 0.5), 0.92, 0.57, 0.08, 0.0, iron);
            }
            ObjKind::Exit => {
                b.column(at(p.x, p.y, ob.z), 0.35, 0.05, 2.2, 8, dim(EXIT, k));
            }
        }
    }
    b
}

/// The squad's height in a site, for the camera: the first member in one.
pub fn ground(w: &World) -> Option<f32> {
    w.squad.members.iter().find_map(|&m| w.site_height(m))
}

/// Where the mouse's ray first meets a site's floors, decks, water or wall
/// tops: the site, the spot (world metres) and its world height.
pub fn pick(w: &World, ray: (Vec3, Vec3)) -> Option<(u32, V2, f32)> {
    let (o, d) = ray;
    if d.y >= -1e-4 {
        return None;
    }
    let mut best: Option<(f32, u32, V2, f32)> = None;
    for s in &w.sites.sites {
        let (lo, hi) = (s.base_h - 6.0, s.base_h + 8.0);
        let t0 = ((hi - o.y) / d.y).max(0.0);
        let t1 = (lo - o.y) / d.y;
        if t1 <= t0 {
            continue;
        }
        let mut prev: Option<f32> = None;
        let mut t = t0;
        while t <= t1 {
            let p = o + d * t;
            let l = s.to_local(V2::new(p.x, p.z));
            let rel = p.y - s.base_h;
            let (cx, cy) = (l.x.floor() as i32, l.y.floor() as i32);
            // Only what's been seen can be clicked.
            let known = cx >= 0 && cy >= 0 && s.seen.get(cy as usize * s.w as usize + cx as usize).copied().unwrap_or(false);
            if let (Some(c), Some(pr), true) = (s.cell(cx, cy), prev, known) {
                let mut levels: Vec<f32> = Vec::new();
                if c.ground == Ground::Wall {
                    levels.push(wall_top(s, cx, cy));
                } else {
                    for n in s.nodes_in(cx, cy) {
                        levels.push(s.level(n).unwrap().0);
                    }
                    if levels.is_empty() {
                        levels.push(c.floor);
                    }
                }
                if let Some(h) = levels.into_iter().filter(|&h| pr > h && rel <= h).fold(None, |m: Option<f32>, h| Some(m.map_or(h, |m| m.max(h)))) {
                    if best.map_or(true, |b| t < b.0) {
                        best = Some((t, s.id, s.to_world(l), s.base_h + h));
                    }
                    break;
                }
            }
            prev = Some(rel);
            t += 0.05;
        }
    }
    best.map(|b| (b.1, b.2, b.3))
}


