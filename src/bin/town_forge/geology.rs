//! What the ground is made of: tilted layers of rock of differing hardness,
//! cracked along a few directions, with harder bodies here and there.
//! Everything else (cliffs, inlets, stacks, benches) is what the sea, rain
//! and gravity make of it in `erode.rs`.

use crate::noise::{fbm, noise, smooth};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::rng::Rng;

/// A long crack zone the sea or a stream can pick out: a line through
/// `through` at `angle_deg` (0 = east, 90 = south), `width` metres of
/// shattered rock, `weak` 0..1 how much softer it is.
#[derive(Clone, Copy, Debug)]
pub struct Fault {
    pub through: V2,
    pub angle_deg: f32,
    pub width: f32,
    pub weak: f32,
}

/// A body of harder rock: a band along a line, `width` across, or a round
/// knob (`length` 0).
#[derive(Clone, Copy, Debug)]
pub struct HardBody {
    pub through: V2,
    pub angle_deg: f32,
    pub length: f32,
    pub width: f32,
    /// How much harder, e.g. 0.5 = half again.
    pub harder: f32,
}

/// The rock recipe for a site.
#[derive(Clone, Debug)]
pub struct Rock {
    pub seed: u64,
    /// Layers dip this many degrees toward `dip_toward_deg` (0 = east, 90 = south).
    pub dip_deg: f32,
    pub dip_toward_deg: f32,
    /// Typical layer thickness in metres and how much it varies.
    pub layer_m: f32,
    /// Share of layers that are hard (0..1).
    pub hard_share: f32,
    /// Fine crack sets: directions and spacing in metres.
    pub joint_dirs_deg: [f32; 2],
    pub joint_spacing: f32,
    /// How much a crack weakens the rock (0..1).
    pub joint_weak: f32,
    pub faults: Vec<Fault>,
    pub bodies: Vec<HardBody>,
}

/// The strata as a lookup: hardness by position in the pile (0.5 m steps).
pub struct Strata {
    lut: Vec<f32>,
    lo: f32,
}

const STRATA_LO: f32 = -300.0;
const STRATA_HI: f32 = 900.0;
const STRATA_STEP: f32 = 0.5;

impl Strata {
    pub fn new(r: &Rock) -> Strata {
        let mut rng = Rng::from_keys(&[r.seed, 0x5354_5241]);
        let mut lut = Vec::new();
        let mut s = STRATA_LO;
        let mut last_hard = false;
        while s < STRATA_HI {
            let thick = r.layer_m * rng.range(0.35, 2.2);
            let roll = rng.f32();
            // Hard layers rarely sit on hard layers; soft on soft is common.
            let hard = roll < r.hard_share * if last_hard { 0.4 } else { 1.3 };
            let h = if hard {
                rng.range(0.82, 1.0)
            } else if rng.chance(0.45) {
                rng.range(0.45, 0.68)
            } else {
                rng.range(0.14, 0.36)
            };
            last_hard = hard;
            let n = (thick / STRATA_STEP).max(1.0) as usize;
            // A soft parting between layers.
            for i in 0..n {
                let edge = i == 0 && !hard;
                lut.push(if edge { h * 0.6 } else { h });
            }
            s += thick;
        }
        Strata { lut, lo: STRATA_LO }
    }

    #[inline]
    pub fn at(&self, s: f32) -> f32 {
        let i = ((s - self.lo) / STRATA_STEP) as isize;
        let i = i.clamp(0, self.lut.len() as isize - 1) as usize;
        self.lut[i]
    }
}

/// Everything about the rock that depends only on (x, y), precomputed per
/// cell so erosion can ask for hardness cheaply.
pub struct Column {
    /// Subtract from z to get the position in the layer pile (dip and folding).
    pub shift: f32,
    /// Multiply the layers' hardness by this: cracks and faults (< 1), patches.
    pub factor: f32,
    /// How much of the column is a hard body (0..1), which ignores the layers.
    pub body: f32,
    /// The cracks and faults alone (< 1), which cut hard bodies too.
    pub cracks: f32,
}

pub fn column(r: &Rock, p: V2) -> Column {
    let dip = r.dip_deg.to_radians().tan();
    let (ca, sa) = (r.dip_toward_deg.to_radians().cos(), r.dip_toward_deg.to_radians().sin());
    // Layers rise against the dip direction; gentle folding so they aren't dead flat.
    let shift = -(p.x * ca + p.y * sa) * dip + (fbm(r.seed ^ 0xF0, p.x, p.y, 420.0, 2) - 0.5) * 14.0;
    let mut factor = 1.0;
    // Fine cracks in two directions, spaced unevenly, a few of them deep.
    for (k, dir) in r.joint_dirs_deg.iter().enumerate() {
        let (c, s) = (dir.to_radians().cos(), dir.to_radians().sin());
        let along = p.x * c + p.y * s + (noise(r.seed ^ (0x10 + k as u64), p.x, p.y, 90.0) - 0.5) * r.joint_spacing * 0.8;
        let cell = (along / r.joint_spacing).floor();
        let frac = along / r.joint_spacing - cell;
        let id = gahturiyu_sim::sim::rng::key(&[r.seed, 0x4A4E, k as u64, cell as i64 as u64]);
        let (deep, off) = (((id >> 8) & 0xF) < 5, ((id & 0xFF) as f32 / 255.0) * 0.6 + 0.2);
        let half_w = if deep { 3.0 } else { 1.2 } / r.joint_spacing;
        let d = (frac - off).abs();
        let w = 1.0 - smooth(half_w * 0.5, half_w, d);
        factor *= 1.0 - r.joint_weak * w * if deep { 1.5 } else { 0.6 };
    }
    for f in &r.faults {
        let (c, s) = (f.angle_deg.to_radians().cos(), f.angle_deg.to_radians().sin());
        // Distance across the line, wobbling a little along it.
        let rel = p.sub(f.through);
        let across = -rel.x * s + rel.y * c + (noise(r.seed ^ 0x20, p.x, p.y, 60.0) - 0.5) * f.width * 0.8;
        let w = 1.0 - smooth(f.width * 0.35, f.width * 0.5, across.abs());
        factor *= 1.0 - f.weak * w;
    }
    let cracks = factor;
    let mut body = 0.0f32;
    for b in &r.bodies {
        let (c, s) = (b.angle_deg.to_radians().cos(), b.angle_deg.to_radians().sin());
        let rel = p.sub(b.through);
        let along = rel.x * c + rel.y * s;
        let across = -rel.x * s + rel.y * c + (noise(r.seed ^ 0x21, p.x, p.y, 40.0) - 0.5) * b.width * 0.5;
        let w = (1.0 - smooth(b.width * 0.35, b.width * 0.5, across.abs())) * (1.0 - smooth(b.length * 0.5, b.length * 0.5 + 20.0, along.abs()));
        body = body.max(w * b.harder.min(1.0));
    }
    // Harder and softer bodies in the rock itself, at two sizes.
    let patch = fbm(r.seed ^ 0x30, p.x, p.y, 180.0, 2) * 0.65 + fbm(r.seed ^ 0x31, p.x, p.y, 50.0, 2) * 0.35;
    factor *= 0.5 + 1.0 * patch;
    Column { shift, factor, body, cracks }
}
