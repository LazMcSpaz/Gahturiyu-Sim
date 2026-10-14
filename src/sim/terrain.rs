//! The shape of the land.
//!
//! The land is Laz's map (`atlas.rs`): the atlas gives each spot its class,
//! its base height (smoothed) and its distance to the sea; the world seed
//! adds the small relief on top (`micro_relief`), so no two worlds walk the
//! same ground but every one is the map. Hand edits and the forge's land
//! sit on that. Some stretches of shore are low cliffs instead of beaches.
//!
//! The sea is at height 0. Land at the waterline sits just above it.

use serde::{Deserialize, Serialize};

use super::atlas::{atlas, Class, PX};
use super::geo::{self, V2, WORLD_SIZE};
use super::rng;

/// The atlas's grid spacing in metres (one map pixel). Finer detail than
/// this is the seed's small relief and the view's business.
pub const CELL: f32 = PX;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Terrain {
    seed: u64,
    /// Where the roads run (set once the road network is built).
    roads: RoadIndex,
    /// Hand edits on top of the seed's land (`mapedit.rs`).
    #[serde(default)]
    pub edits: super::mapedit::MapEdits,
    /// Land authored by a tool (the town forge), under the hand edits: the
    /// same layers, kept apart so the player's own map stays their own.
    #[serde(default)]
    pub authored: super::mapedit::MapEdits,
}

/// On a road the weather slows a walker at most this much (the mud doesn't
/// count on a made road).
pub const ROAD_WEATHER_FLOOR: f32 = 0.85;
/// Along a leg the weather is read this often (metres), at its departure.
pub const WEATHER_EVERY: f32 = 2000.0;

/// What the ground underfoot is like. Each has a small effect on walking pace.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    Road,
    Grass,
    Scrub,
    Rock,
    Sand,
    // Only where painted by hand:
    Dirt,
    Mud,
    Gravel,
    Snow,
}

impl Ground {
    /// Walking pace on this ground, relative to grass.
    pub fn pace(self) -> f32 {
        match self {
            Ground::Road => ROAD_PACE,
            Ground::Grass => 1.0,
            Ground::Scrub => 0.92,
            Ground::Rock => 0.8,
            Ground::Sand => 0.85,
            Ground::Dirt => 0.97,
            Ground::Mud => 0.7,
            Ground::Gravel => 0.9,
            Ground::Snow => 0.72,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Ground::Road => "road",
            Ground::Grass => "grass",
            Ground::Scrub => "scrub",
            Ground::Rock => "rock",
            Ground::Sand => "sand",
            Ground::Dirt => "dirt",
            Ground::Mud => "mud",
            Ground::Gravel => "gravel",
            Ground::Snow => "snow",
        }
    }
}

/// Walking pace on a road, relative to grass.
/// Land rises from the waterline to its full height over this many metres.
pub const SHORE_RISE_M: f32 = 25.0;
pub const ROAD_PACE: f32 = 1.3;
/// How far either side of a road's centre line still counts as road, metres.
pub const ROAD_HALF_WIDTH: f32 = 6.0;
// (Coarse, for a continent: 390 km a side is 640 cells, not 3 900.)
const ROAD_CELL: f32 = 600.0;
const RN: usize = (WORLD_SIZE / ROAD_CELL) as usize + 1;

/// Road segments filed by grid cell, so "is this on a road?" is quick.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct RoadIndex {
    segs: Vec<(V2, V2)>,
    cells: Vec<Vec<u32>>,
}

impl RoadIndex {
    fn build(roads: &[Vec<V2>]) -> RoadIndex {
        let mut idx = RoadIndex { segs: Vec::new(), cells: vec![Vec::new(); RN * RN] };
        for road in roads {
            for w in road.windows(2) {
                let id = idx.segs.len() as u32;
                idx.segs.push((w[0], w[1]));
                let pad = ROAD_HALF_WIDTH + 1.0;
                let (x0, x1) = (w[0].x.min(w[1].x) - pad, w[0].x.max(w[1].x) + pad);
                let (y0, y1) = (w[0].y.min(w[1].y) - pad, w[0].y.max(w[1].y) + pad);
                let cl = |v: f32| ((v / ROAD_CELL).floor().max(0.0) as usize).min(RN - 1);
                for j in cl(y0)..=cl(y1) {
                    for i in cl(x0)..=cl(x1) {
                        idx.cells[j * RN + i].push(id);
                    }
                }
            }
        }
        idx
    }

    fn near(&self, p: V2) -> bool {
        if self.cells.is_empty() {
            return false;
        }
        let i = ((p.x / ROAD_CELL).floor().max(0.0) as usize).min(RN - 1);
        let j = ((p.y / ROAD_CELL).floor().max(0.0) as usize).min(RN - 1);
        self.cells[j * RN + i].iter().any(|&k| {
            let (a, b) = self.segs[k as usize];
            seg_dist(a, b, p) <= ROAD_HALF_WIDTH
        })
    }
}

/// Distance from `c` to the segment `a`–`b`.
pub fn seg_dist(a: V2, b: V2, c: V2) -> f32 {
    let ab = b.sub(a);
    let l2 = ab.x * ab.x + ab.y * ab.y;
    if l2 < 1e-6 {
        return a.dist(c);
    }
    let t = (((c.x - a.x) * ab.x + (c.y - a.y) * ab.y) / l2).clamp(0.0, 1.0);
    a.lerp(b, t).dist(c)
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

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---- Regions ----------------------------------------------------------------

/// How mountainous a spot is, 0..1: the map's snowy mountains and rocky
/// ground (`atlas.rs`), blurred.
pub fn mountain_mask(p: V2) -> f32 {
    atlas().mountain(p)
}

/// How far into the Qotiro plateau a spot is, 0..1. There's none on Laz's
/// map: always 0 (kept for what still asks).
pub fn plateau_mask(_p: V2) -> f32 {
    0.0
}

/// How cliffy the shore is at this latitude, 0..1.
pub fn cliff_mask(seed: u64, y: f32) -> f32 {
    smooth(0.56, 0.70, noise(seed ^ 0xC11F, 0.0, y, 2600.0))
}

/// How much a point belongs to the plateau, 0..1: none.
fn plateau_at(_seed: u64, _p: V2) -> f32 {
    0.0
}

/// How much a point belongs to the mountains, 0..1.
fn mountains_at(_seed: u64, p: V2) -> f32 {
    smooth(0.08, 0.7, mountain_mask(p))
}

/// The seed's small relief on top of the atlas: rolling ground in the
/// lowland, broken ground in the rock and the snow, nothing on water. The
/// amplitude by class, metres.
fn relief_amp(c: Class) -> f32 {
    match c {
        Class::Sea | Class::Water => 0.0,
        Class::Coast => 4.0,
        Class::Bog => 1.5,
        Class::Plains => 10.0,
        Class::Forest => 14.0,
        Class::Overgrowth => 12.0,
        Class::Rock => 60.0,
        Class::Snow => 120.0,
    }
}

/// The relief itself: two bands of noise, 700 m and 90 m across.
fn micro_relief(seed: u64, p: V2) -> f32 {
    // (The strength blended across the pixels round it: a step in it at a
    // forest's edge would be a little cliff.)
    let amp = atlas().blend(p, relief_amp);
    if amp <= 0.0 {
        return 0.0;
    }
    let broad = (fbm(seed ^ 0x4111, p.x, p.y, 700.0, 3) - 0.5) * 2.0 * amp;
    let fine = (noise(seed ^ 0x51, p.x, p.y, 90.0) - 0.5) * amp * 0.12;
    broad + fine
}

impl Terrain {
    /// No land at all: a stand-in while a save is read.
    pub fn empty() -> Terrain {
        Terrain { seed: 0, roads: RoadIndex::default(), edits: Default::default(), authored: Default::default() }
    }

    /// The map's land with this seed's small relief on it.
    pub fn generate(seed: u64) -> Terrain {
        let tseed = rng::key(&[seed, 0x5445_5252]);
        let _ = atlas();
        Terrain { seed: tseed, roads: RoadIndex::default(), edits: Default::default(), authored: Default::default() }
    }

    /// What the map says the ground is here.
    pub fn class(&self, p: V2) -> Class {
        atlas().class_at(p)
    }

    /// The same land with these hand edits on it.
    pub fn with_edits(mut self, edits: super::mapedit::MapEdits) -> Terrain {
        self.edits = edits;
        self
    }

    /// The same land with this authored land under the hand edits.
    pub fn with_authored(mut self, authored: super::mapedit::MapEdits) -> Terrain {
        self.authored = authored;
        self
    }

    /// Below the sea: the land itself decides (authored bays and stacks
    /// included), not the world's broad coastline.
    pub fn is_sea(&self, p: V2) -> bool {
        self.height(p) < 0.0
    }

    /// Lay the road network onto the land (done once, after the roads are
    /// found). From then on roads count in walking times.
    pub fn set_roads(&mut self, roads: &[Vec<V2>]) {
        self.roads = RoadIndex::build(roads);
    }

    pub fn on_road(&self, p: V2) -> bool {
        self.roads.near(p)
    }

    /// The ground underfoot, read off the land itself: road where a road
    /// runs; sand along the shore; rock on steep slopes and high mountains;
    /// scrub on the dry plateau and in patches elsewhere; grass otherwise.
    pub fn ground(&self, p: V2) -> Ground {
        if self.on_road(p) {
            return Ground::Road;
        }
        // Ground painted by hand (or authored), laid on thickly enough.
        for layer in [&self.edits, &self.authored] {
            if let Some((tex, w)) = layer.paint_at(p) {
                if let (true, Some(t)) = (w >= 0.5, super::mapedit::TEXTURES.get(tex as usize)) {
                    return t.ground;
                }
            }
        }
        // The map's own ground (`atlas.rs`).
        match self.class(p) {
            Class::Snow => return Ground::Snow,
            Class::Rock => return Ground::Rock,
            Class::Bog | Class::Water => return Ground::Mud,
            Class::Overgrowth => return Ground::Scrub,
            _ => {}
        }
        if geo::inland(p) < 60.0 {
            return Ground::Sand;
        }
        if self.slope(p) > 0.35 || self.mountains(p) > 0.55 {
            return Ground::Rock;
        }
        if self.plateau(p) > 0.5 || noise(self.seed ^ 0x5C2B, p.x, p.y, 450.0) > 0.7 {
            return Ground::Scrub;
        }
        Ground::Grass
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

    /// Ground height in metres at any point (sea floor below 0 offshore),
    /// hand edits included.
    pub fn height(&self, p: V2) -> f32 {
        self.base_height(p) + self.authored.height.at(p) + self.edits.height.at(p)
    }

    /// The land as the map and the seed made it, before any hand edits:
    /// the atlas's base height with the seed's relief, kept above the
    /// waterline on land and below it at sea.
    pub fn base_height(&self, p: V2) -> f32 {
        // Salt Isle: its own low shape, the same in every world (`island.rs`).
        if geo::isle_inland(p) > 0.0 {
            return super::island::height(p);
        }
        let a = atlas();
        let base = a.base_height(p);
        let h = base + micro_relief(self.seed, p);
        let d = a.inland(p);
        if d > 0.0 {
            // (Rising from the waterline over the first `SHORE_RISE_M`, so a
            // cliff's top follows the shore's curve, not the map's pixels.)
            0.5 + (h.max(0.5) - 0.5) * smooth(0.0, SHORE_RISE_M, d)
        } else {
            h.min(-0.5)
        }
    }

    /// Height on land, or 0 (the sea surface) offshore.
    pub fn surface(&self, p: V2) -> f32 {
        let h = self.height(p);
        if h < 0.0 { 0.0 } else { h.max(0.3) }
    }

    /// Steepest rise per metre at a point (0 = flat, 1 = 45°).
    pub fn slope(&self, p: V2) -> f32 {
        // (At the walking scale, not the map's 150 m: the seed's relief counts.)
        let e = 30.0;
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

    /// How walking-time cost compares to flat grass over a stretch: the
    /// stretch's length divided by Tobler's hiking speed (uphill slow, gentle
    /// downhill slightly fast, steep downhill slow again) and by the pace of
    /// the ground at its middle (roads quick, rock and sand slow).
    /// Walking pace in the weather at `p` at `t` (soaked ground, lying snow,
    /// a gale on open ground: `weather_effects::travel_speed_mult`); on a
    /// road the mud doesn't count, so it never falls below
    /// `ROAD_WEATHER_FLOOR` there.
    pub fn weather_pace(&self, p: V2, t: f64) -> f32 {
        let pace = super::weather::weather_effects(self, self.seed, p, t).travel_speed_mult;
        if self.on_road(p) {
            pace.max(ROAD_WEATHER_FLOOR)
        } else {
            pace
        }
    }

    pub fn effort(&self, a: V2, b: V2) -> f32 {
        let len = a.dist(b);
        if len < 1e-3 {
            return 0.0;
        }
        len / (walk_factor((self.height(b) - self.height(a)) / len) * self.ground(a.lerp(b, 0.5)).pace())
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
    fn the_map_is_land_in_a_sea_and_its_mountains_rise() {
        let t = Terrain::generate(1);
        let a = atlas();
        // Off the map's edges: sea, below the waterline.
        assert!(t.height(V2::new(50.0, 50.0)) < 0.0 && t.is_sea(V2::new(50.0, 50.0)));
        // Somewhere on land, just above it; a shore spot is low.
        let land = a.land_spot(7);
        assert!(t.height(land) >= 0.5, "land at {land:?}: {}", t.height(land));
        let shore = a.shore_spot(land, 20.0);
        assert!(t.height(shore) < 12.0, "the shore at {shore:?} is {} m up", t.height(shore));
        // The highest ground is well up (the snowy mountains), if the map has any.
        if a.census()[Class::Snow as usize] > 0 {
            let peak = (0..4000u64).map(|k| t.height(a.land_spot(k * 977 + 13))).fold(0.0f32, f32::max);
            assert!(peak > 300.0, "the mountains top out at only {peak} m");
        }
    }

    #[test]
    fn walking_uphill_is_slower() {
        assert!((walk_factor(-0.05) - 1.19).abs() < 0.02);
        assert!(walk_factor(0.2) < 0.5);
        assert!(walk_factor(0.0) > 0.99 && walk_factor(0.0) < 1.01 * 1.0 + 0.0001);
    }
}
