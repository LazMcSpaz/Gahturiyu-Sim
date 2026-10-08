//! The shape of the land.
//!
//! Heights are built once, from the world seed, into a grid; everything else
//! samples that grid. The land rises gently from the coast into rolling hills,
//! walls up into mountains along the north and east edges and one inland
//! massif, and lifts into a flat-topped, cliff-edged plateau in the south-east
//! — the arid Qotiro homeland. Some stretches of shore are low cliffs instead
//! of beaches.
//!
//! The sea is at height 0. Land at the waterline sits just above it.

use super::geo::{self, V2, WORLD_SIZE};
use super::rng;

/// Grid spacing in metres. Finer detail than this is the view's business.
pub const CELL: f32 = 30.0;
const N: usize = (WORLD_SIZE / CELL) as usize + 2;

#[derive(Clone, Debug)]
pub struct Terrain {
    h: Vec<f32>,
    seed: u64,
}

// ---- Noise ------------------------------------------------------------------

/// A pseudo-random unit gradient for a lattice corner.
fn gradient(seed: u64, ix: i64, iy: i64) -> (f32, f32) {
    let a = (rng::key(&[seed, ix as u64, iy as u64]) >> 40) as f32 / (1u64 << 24) as f32 * std::f32::consts::TAU;
    (a.cos(), a.sin())
}

/// Gradient (Perlin) noise, roughly -1..1. Unlike plain value noise it has no
/// grid-aligned blockiness, which matters for ridges.
fn perlin(seed: u64, x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (tx, ty) = (x - ix, y - iy);
    let (ix, iy) = (ix as i64, iy as i64);
    let s = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let dot = |cx: i64, cy: i64, dx: f32, dy: f32| {
        let (gx, gy) = gradient(seed, cx, cy);
        gx * dx + gy * dy
    };
    let a = dot(ix, iy, tx, ty);
    let b = dot(ix + 1, iy, tx - 1.0, ty);
    let c = dot(ix, iy + 1, tx, ty - 1.0);
    let d = dot(ix + 1, iy + 1, tx - 1.0, ty - 1.0);
    let (sx, sy) = (s(tx), s(ty));
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    (top + (bot - top) * sy) * 1.41
}

/// Smooth noise in 0..1 at the given feature size.
pub fn noise(seed: u64, x: f32, y: f32, scale: f32) -> f32 {
    (perlin(seed, x / scale, y / scale) * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Each octave is turned a little so their grids never line up.
fn rotated(x: f32, y: f32, o: u32) -> (f32, f32) {
    let (s, c) = (o as f32 * 0.83 + 0.3).sin_cos();
    (x * c - y * s, x * s + y * c)
}

/// Several octaves of noise summed, 0..1.
fn fbm(seed: u64, x: f32, y: f32, scale: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut sc) = (0.0, 1.0, 0.0, scale);
    for o in 0..octaves {
        let (rx, ry) = rotated(x, y, o);
        sum += noise(seed.wrapping_add(o as u64 * 101), rx, ry, sc) * amp;
        norm += amp;
        amp *= 0.5;
        sc *= 0.5;
    }
    sum / norm
}

/// Ridged noise: sharp crests where plain noise crosses zero. 0..1. Each
/// octave is weighted by the one before, so fine ridges gather on the big
/// crests rather than speckling the valleys.
fn ridged(seed: u64, x: f32, y: f32, scale: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut sc, mut weight) = (0.0, 1.0, 0.0, scale, 1.0f32);
    for o in 0..octaves {
        let (rx, ry) = rotated(x, y, o);
        let n = 1.0 - perlin(seed.wrapping_add(o as u64 * 977), rx / sc, ry / sc).abs().min(1.0);
        let n = n * n * weight;
        weight = (n * 1.6).clamp(0.0, 1.0);
        sum += n * amp;
        norm += amp;
        amp *= 0.5;
        sc *= 0.5;
    }
    sum / norm
}

/// Push a point around by large, slow noise, so region edges wander instead
/// of following straight lines.
fn warp(seed: u64, p: V2, amount: f32) -> V2 {
    V2::new(
        p.x + (noise(seed ^ 0xA1, p.x, p.y, 4200.0) - 0.5) * amount,
        p.y + (noise(seed ^ 0xA2, p.x, p.y, 4200.0) - 0.5) * amount,
    )
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---- Regions ----------------------------------------------------------------

/// How mountainous a spot is meant to be, 0..1, before any noise.
pub fn mountain_mask(p: V2) -> f32 {
    let north = smooth(5200.0, 1600.0, p.y);
    let east = smooth(16_800.0, 20_000.0, p.x);
    let massif = smooth(2600.0, 900.0, p.dist(V2::new(8_600.0, 12_600.0)));
    north.max(east).max(massif)
}

/// How far into the Qotiro plateau a spot is, 0..1, before its eroded edge.
pub fn plateau_mask(p: V2) -> f32 {
    smooth(10_900.0, 12_600.0, p.x) * smooth(9_900.0, 11_600.0, p.y) * (1.0 - smooth(16_000.0, 18_000.0, p.x))
}

/// How cliffy the shore is at this latitude, 0..1.
pub fn cliff_mask(seed: u64, y: f32) -> f32 {
    smooth(0.56, 0.70, noise(seed ^ 0xC11F, 0.0, y, 2600.0))
}

/// How much a point belongs to the plateau, 0..1, with its eroded edge.
fn plateau_at(seed: u64, p: V2) -> f32 {
    let pm = plateau_mask(warp(seed, p, 2600.0));
    smooth(0.34, 0.56, pm + (fbm(seed ^ 0x9A7E, p.x, p.y, 1400.0, 3) - 0.5) * 0.5)
}

/// How much a point belongs to the mountains, 0..1.
fn mountains_at(seed: u64, p: V2) -> f32 {
    smooth(0.05, 0.85, mountain_mask(warp(seed, p, 2600.0)))
}

/// The height formula itself. Expensive; only used to fill the grid.
fn raw_height(seed: u64, p: V2) -> f32 {
    let d = geo::inland(p);
    if d < 0.0 {
        return -1.0 - (-d).min(600.0) * 0.03;
    }
    let (x, y) = (p.x, p.y);

    // Lowland rising slowly away from the sea, and rolling hills that fade
    // out toward the shore so the coast stays walkable.
    let lowland = 0.35 + 50.0 * smooth(0.0, 6500.0, d) + 40.0 * smooth(5000.0, 15_000.0, d);
    let hills = (fbm(seed ^ 0x4111, x, y, 2400.0, 5) - 0.5) * 2.0;
    let hills_amp = 95.0 * smooth(300.0, 2600.0, d);

    // The Qotiro plateau: flat on top, with an eroded escarpment around it.
    let edge = plateau_at(seed, p);
    let plateau = edge * (115.0 + (noise(seed ^ 0x77, x, y, 3000.0) - 0.5) * 18.0);
    let rolling = hills * hills_amp * (1.0 - 0.75 * edge);

    // Mountains: ridged crests, masked to their regions and kept off the coast
    // (the northern range comes closest and meets the sea in steep ground).
    let mm = mountains_at(seed, p) * smooth(0.0, 900.0, d);
    let crest = ridged(seed ^ 0x3017, x, y, 4200.0, 5);
    let mountains = mm * (120.0 + 620.0 * crest.powf(1.2)) * (0.7 + 0.6 * noise(seed ^ 0x51, x, y, 7000.0));

    // Low sea cliffs along some stretches of shore.
    let cliff = cliff_mask(seed, y) * 26.0 * smooth(12.0, 80.0, d);

    let h = lowland + rolling + plateau + mountains + cliff;
    h.max(0.35 + d.min(400.0) * 0.004)
}

impl Terrain {
    pub fn generate(seed: u64) -> Terrain {
        let tseed = rng::key(&[seed, 0x5445_5252]);
        let mut h = vec![0.0f32; N * N];
        for j in 0..N {
            for i in 0..N {
                h[j * N + i] = raw_height(tseed, V2::new(i as f32 * CELL, j as f32 * CELL));
            }
        }
        Terrain { h, seed: tseed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// 0..1: how far this spot is into the Qotiro plateau (arid scrub).
    pub fn plateau(&self, p: V2) -> f32 {
        plateau_at(self.seed, p)
    }

    /// 0..1: how far this spot is into mountain country.
    pub fn mountains(&self, p: V2) -> f32 {
        mountains_at(self.seed, p)
    }

    fn at(&self, i: i64, j: i64) -> f32 {
        let i = i.clamp(0, N as i64 - 1) as usize;
        let j = j.clamp(0, N as i64 - 1) as usize;
        self.h[j * N + i]
    }

    /// Ground height in metres at any point (sea floor below 0 offshore).
    pub fn height(&self, p: V2) -> f32 {
        let (fx, fy) = (p.x / CELL, p.y / CELL);
        let (ix, iy) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - ix, fy - iy);
        let (ix, iy) = (ix as i64, iy as i64);
        let a = self.at(ix, iy);
        let b = self.at(ix + 1, iy);
        let c = self.at(ix, iy + 1);
        let d = self.at(ix + 1, iy + 1);
        let top = a + (b - a) * tx;
        let bot = c + (d - c) * tx;
        top + (bot - top) * ty
    }

    /// Height on land, or 0 (the sea surface) offshore.
    pub fn surface(&self, p: V2) -> f32 {
        if geo::is_land(p) {
            self.height(p).max(0.3)
        } else {
            0.0
        }
    }

    /// Steepest rise per metre at a point (0 = flat, 1 = 45°).
    pub fn slope(&self, p: V2) -> f32 {
        let e = CELL;
        let dx = (self.height(p.add(V2::new(e, 0.0))) - self.height(p.sub(V2::new(e, 0.0)))) / (2.0 * e);
        let dy = (self.height(p.add(V2::new(0.0, e))) - self.height(p.sub(V2::new(0.0, e)))) / (2.0 * e);
        (dx * dx + dy * dy).sqrt()
    }

    /// Surface normal as (x, up, y).
    pub fn normal(&self, p: V2, e: f32) -> (f32, f32, f32) {
        let dx = (self.height(p.add(V2::new(e, 0.0))) - self.height(p.sub(V2::new(e, 0.0)))) / (2.0 * e);
        let dy = (self.height(p.add(V2::new(0.0, e))) - self.height(p.sub(V2::new(0.0, e)))) / (2.0 * e);
        let l = (dx * dx + 1.0 + dy * dy).sqrt();
        (-dx / l, 1.0 / l, -dy / l)
    }

    /// How walking-time cost compares to flat ground over a stretch: the
    /// stretch's length divided by Tobler's hiking speed (uphill slow, gentle
    /// downhill slightly fast, steep downhill slow again).
    pub fn effort(&self, a: V2, b: V2) -> f32 {
        let len = a.dist(b);
        if len < 1e-3 {
            return 0.0;
        }
        len / walk_factor((self.height(b) - self.height(a)) / len)
    }
}

/// Walking speed on a grade, relative to flat ground (Tobler's hiking
/// function, normalised so flat = 1).
pub fn walk_factor(grade: f32) -> f32 {
    ((-3.5 * (grade + 0.05).abs()).exp() / (-3.5f32 * 0.05).exp()).max(0.08)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coast_meets_the_sea_and_mountains_rise() {
        let t = Terrain::generate(1);
        for y in (500..20_500).step_by(500) {
            let y = y as f32;
            let shore = V2::new(geo::coast_x(y) + 5.0, y);
            assert!(t.height(shore) < 6.0, "shore at y={y} is {} m up", t.height(shore));
        }
        let peaks = (0..200)
            .map(|i| t.height(V2::new(1000.0 + i as f32 * 95.0, 1200.0)))
            .fold(0.0f32, f32::max);
        assert!(peaks > 300.0, "northern range tops out at only {peaks} m");
    }

    #[test]
    fn walking_uphill_is_slower() {
        assert!((walk_factor(-0.05) - 1.19).abs() < 0.02);
        assert!(walk_factor(0.2) < 0.5);
        assert!(walk_factor(0.0) > 0.99 && walk_factor(0.0) < 1.01 * 1.0 + 0.0001);
    }
}
