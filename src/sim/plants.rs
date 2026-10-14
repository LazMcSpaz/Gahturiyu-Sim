//! Where the land's own plants grow: one rule for the window's foliage
//! (`view/foliage.rs`) and the place builder's picture of the land
//! (`grounds.rs`). Pure functions of the world; nothing here is saved.
//!
//! Plants stand one to a cell of a grid per layer; whether a cell grows one,
//! and where in the cell, is a roll keyed to the world's seed and the cell.

use super::geo::{self, V2};
use super::rng::Rng;
use super::terrain::{self, Ground};
use super::world::World;

/// Metres between grass tufts, bushes and trees (one roll per cell).
pub const GRASS_GRID: f32 = 1.7;
pub const BUSH_GRID: f32 = 9.0;
pub const TREE_GRID: f32 = 13.0;
/// How broad the woods' patches are, metres.
pub const WOOD_SCALE: f32 = 750.0;
/// Trees thin out above this height.
pub const TREE_LINE: f32 = 420.0;
/// A drawn town's thickest wood: the chance of a tree in a tree cell, and
/// of a bush in a bush cell, where the drawing is darkest.
pub const DRAWN_TREE: f32 = 0.95;
pub const DRAWN_BUSH: f32 = 0.35;
/// Metres between trees in drawn woods (the usual wood is `TREE_GRID`).
pub const DRAWN_GRID: f32 = 5.5;

/// A layer's key in the cell rolls.
pub const GRASS: u64 = 1;
pub const BUSH: u64 = 2;
pub const TREE: u64 = 3;
pub const DRAWN: u64 = 4;

pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A plant's place in its grid cell, and a roll for it, from the seed.
pub fn cell(w: &World, kind: u64, i: i64, j: i64, grid: f32) -> (V2, Rng) {
    let mut r = Rng::from_keys(&[w.seed, 0xF011_A6E0 ^ kind, i as u64, j as u64]);
    let p = V2::new((i as f32 + r.f32()) * grid, (j as f32 + r.f32()) * grid);
    (p, r)
}

/// A plant's turn, size and variant, rolled before whether it grows (so the
/// window and the builder read the same rolls in the same order).
pub fn looks(r: &mut Rng) -> (f32, f32, usize) {
    let yaw = r.f32() * std::f32::consts::TAU;
    let scale = 0.8 + r.f32() * 0.45;
    let variant = r.below(3);
    (yaw, scale, variant)
}

/// How wooded the land is outside Laz's drawn forest, at most: scattered
/// trees and the odd copse, never a wood he didn't draw.
pub const OPEN_WOOD: f32 = 0.25;

/// How wooded a place is, 0..1 (before ground and height): the forest drawn
/// on Laz's map (`atlas::Class::Forest`) is wooded through, a little uneven
/// and thinning where its edge is; elsewhere only `OPEN_WOOD` at the most,
/// where the seed's slow noise is high.
pub fn wood(w: &World, p: V2) -> f32 {
    let noise = smooth(0.52, 0.66, terrain::noise(w.seed ^ 0xF0E5_7D11, p.x, p.y, WOOD_SCALE));
    let a = super::atlas::atlas();
    let r = super::atlas::PX * 0.75;
    let around = [V2::new(0.0, 0.0), V2::new(r, 0.0), V2::new(-r, 0.0), V2::new(0.0, r), V2::new(0.0, -r)];
    let drawn = around.iter().filter(|d| a.class_at(p.add(**d)) == super::atlas::Class::Forest).count() as f32 / around.len() as f32;
    (drawn * (0.8 + 0.2 * noise)).max((1.0 - drawn) * OPEN_WOOD * noise)
}

/// The ground at a plant's spot, if anything can grow there at all.
pub struct Spot {
    pub h: f32,
    pub slope: f32,
    pub arid: f32,
    pub shore: f32,
    pub ground: Ground,
}

pub fn spot(w: &World, p: V2) -> Option<Spot> {
    let t = &w.terrain;
    let shore = geo::inland(p);
    if t.is_sea(p) || t.height(p) < 2.0 {
        return None;
    }
    let ground = t.ground(p);
    if ground == Ground::Road || t.on_road(p) {
        return None;
    }
    Some(Spot { h: t.height(p), slope: t.slope(p), arid: t.plateau(p), shore, ground })
}

pub fn grass_at(w: &World, p: V2, r: &mut Rng, density: f32) -> bool {
    let Some(s) = spot(w, p) else { return false };
    if s.ground != Ground::Grass {
        return false;
    }
    let chance = 0.8 * (1.0 - s.arid) * smooth(0.75, 0.45, s.slope) * smooth(620.0, 460.0, s.h) * smooth(6.0, 22.0, s.shore) * (1.0 - 0.5 * wood(w, p));
    let (m, add) = w.terrain.edits.plants_at(p)[0];
    r.f32() < (chance * m + add * smooth(0.9, 0.6, s.slope)) * density
}

/// Whether a bush grows in this cell: in a drawn wood as drawn, else as the
/// land has it.
pub fn bush_at(w: &World, p: V2, r: &mut Rng, density: f32) -> bool {
    bush_with(w, p, r, density, w.plan_woods_at(p))
}

/// Whether the land itself would grow a bush here, whatever a town's plan
/// says (the place builder's starting picture).
pub fn land_bush_at(w: &World, p: V2, r: &mut Rng, density: f32) -> bool {
    bush_with(w, p, r, density, None)
}

fn bush_with(w: &World, p: V2, r: &mut Rng, density: f32, drawn: Option<f32>) -> bool {
    let Some(s) = spot(w, p) else { return false };
    if let Some(d) = drawn {
        // Drawn woods, whatever the ground: undergrowth with the trees, none on the open ground.
        return r.f32() < DRAWN_BUSH * d * density;
    }
    if !matches!(s.ground, Ground::Grass | Ground::Scrub | Ground::Dirt) {
        return false;
    }
    let wd = wood(w, p);
    let chance = (0.10 + 0.35 * wd * (1.0 - wd) * 4.0 * 0.5 + 0.25 * s.arid) * smooth(0.8, 0.5, s.slope) * smooth(720.0, 560.0, s.h) * smooth(6.0, 14.0, s.shore);
    let (m, add) = w.terrain.edits.plants_at(p)[1];
    r.f32() < (chance * m + add * smooth(0.9, 0.6, s.slope)) * density
}

/// Whether a tree grows in this cell, and if so which kind (0 broadleaf,
/// 1 conifer).
pub fn tree_at(w: &World, p: V2, r: &mut Rng, density: f32) -> Option<u8> {
    tree_with(w, p, r, density, w.plan_woods_at(p))
}

/// Whether the land itself would grow a tree here, whatever a town's plan
/// says (the place builder's starting picture).
pub fn land_tree_at(w: &World, p: V2, r: &mut Rng, density: f32) -> Option<u8> {
    tree_with(w, p, r, density, None)
}

fn tree_with(w: &World, p: V2, r: &mut Rng, density: f32, drawn: Option<f32>) -> Option<u8> {
    let s = spot(w, p)?;
    // (Drawn woods grow whatever the ground: the drawing says so.)
    if drawn.is_none() && !matches!(s.ground, Ground::Grass | Ground::Scrub | Ground::Dirt) {
        return None;
    }
    let wd = wood(w, p);
    let chance = match drawn {
        // Drawn woods (`plan.rs`): trees where the drawing has them, none where it's open.
        Some(d) => DRAWN_TREE * d,
        None => {
            let (m, add) = w.terrain.edits.plants_at(p)[2];
            (0.85 * wd + 0.025) * (1.0 - 0.95 * s.arid) * smooth(TREE_LINE + 120.0, TREE_LINE, s.h) * smooth(0.7, 0.4, s.slope) * smooth(15.0, 80.0, s.shore) * m + add * smooth(0.8, 0.5, s.slope)
        }
    };
    if r.f32() >= chance * density {
        return None;
    }
    // Conifers up the slopes, broadleaf below, mixed between.
    let conifer = r.f32() < smooth(160.0, 340.0, s.h) * 0.9 + 0.1 * wd;
    Some(conifer as u8)
}

/// The trees the land itself grows in a box, at full density: where, which
/// kind (0 broadleaf, 1 conifer) and how big, as the window would draw them
/// with no town plan over them.
pub fn land_trees_in(w: &World, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<(V2, u8, f32)> {
    let mut out = Vec::new();
    for j in (y0 / TREE_GRID).floor() as i64..=(y1 / TREE_GRID).floor() as i64 {
        for i in (x0 / TREE_GRID).floor() as i64..=(x1 / TREE_GRID).floor() as i64 {
            let (p, mut r) = cell(w, TREE, i, j, TREE_GRID);
            if p.x < x0 || p.x >= x1 || p.y < y0 || p.y >= y1 {
                continue;
            }
            let (_, scale, _) = looks(&mut r);
            if let Some(k) = land_tree_at(w, p, &mut r, 1.0) {
                out.push((p, k, scale));
            }
        }
    }
    out
}

/// The bushes the land itself grows in a box, at full density, and how big.
pub fn land_bushes_in(w: &World, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<(V2, f32)> {
    let mut out = Vec::new();
    for j in (y0 / BUSH_GRID).floor() as i64..=(y1 / BUSH_GRID).floor() as i64 {
        for i in (x0 / BUSH_GRID).floor() as i64..=(x1 / BUSH_GRID).floor() as i64 {
            let (p, mut r) = cell(w, BUSH, i, j, BUSH_GRID);
            if p.x < x0 || p.x >= x1 || p.y < y0 || p.y >= y1 {
                continue;
            }
            let (_, scale, _) = looks(&mut r);
            if land_bush_at(w, p, &mut r, 1.0) {
                out.push((p, scale));
            }
        }
    }
    out
}
