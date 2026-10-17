//! Horaro stilt buildings as placeholder shapes (Laz, 17 Oct): woven lobes
//! lashed to each other with rope, standing on rough stone supports under
//! a plank deck with a rope rail, reached by ladders, ramps, stairs and
//! docks, and the gear of their work decks. Designed in
//! `tools/buildings/horaro_designs.py` from the five paintings and the kit
//! (`claude/horaro-kit.md`); drawn here from a `HoraroLook`, drawing only
//! (the sim knows the deck's outline, nothing more: the sea is off-limits
//! to the squad, so nobody walks these).
//!
//! Heights are metres over the water (the frame's base); spots are layout
//! units; sizes are metres at the form's `at_size`, scaled with it.

use bevy::math::{vec3, Vec3};
use gahturiyu_sim::sim::geo::V2;

use super::cam::to3;
use super::interiors::{fm, inset, Frame};
use super::mesh::Builder;
use super::palette::{self, Rgb};
use super::stone;

/// Everything that stands on, under and against a Horaro building.
pub struct HoraroLook {
    /// Under the deck: kind, spot, radius (m).
    pub supports: &'static [(HSup, (f32, f32), f32)],
    /// The deck's floor over the water (m); its slab hangs `SLAB` below.
    pub deck: f32,
    /// Runs of rope rail round the deck's edge: fractions of the way round
    /// (from the outline's first corner), each a start and an end.
    pub rail: &'static [(f32, f32)],
    /// A stone quay beside the deck: outline (layout units) and its top (m).
    pub quay: Option<(&'static [(f32, f32)], f32)>,
    /// The woven lobes on the deck: kind, centre, turn, (length, width,
    /// height) at the form's size.
    pub hulls: &'static [(HHull, (f32, f32), f32, (f32, f32, f32))],
    /// Pairs of hulls lashed together: a rope runs round where they meet.
    pub seams: &'static [(usize, usize)],
    /// Round-topped doorways in the hulls: spot on the hull, facing.
    pub doors: &'static [((f32, f32), f32)],
    /// Porthole windows: spot, facing, height over the floor.
    pub portholes: &'static [((f32, f32), f32, f32)],
    /// Ways on and off: kind, the deck-edge spot, the way out (radians in
    /// the frame), length (m).
    pub access: &'static [(HAcc, (f32, f32), f32, f32)],
    /// Gear on the deck: kind, spot, turn, (w, d, h), height over the floor.
    pub props: &'static [(HProp, (f32, f32), f32, (f32, f32, f32), f32)],
}

/// What holds a deck up.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HSup {
    /// A rough rock pillar, rope-lashed near its top.
    Rock,
    /// Stacked stone drums, rope-lashed at their joints.
    Drum,
    /// A squared brick pier.
    Pier,
    /// A grown-stone root base: a trunk flaring into roots.
    Root,
    /// A timber pile.
    Post,
}

/// The shapes of a woven lobe.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HHull {
    /// An egg: bulging from the floor, round on top.
    Egg,
    /// The rock home's dome: upright sides, a rounded, flattish crown.
    Dome,
    /// The stilt hull: boxier in plan, sloping shoulders.
    Hull,
    /// A barrel-vaulted longhouse with woven end walls.
    Longhouse,
    /// A low woven lid (the weaving hall's canopy), open underneath.
    Lid,
    /// A covered walk: the barrel's top on arched ribs, open at the sides
    /// and ends.
    Arcade,
}

/// Ways on and off a deck.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HAcc {
    /// A ladder down to the water.
    Ladder,
    /// A plank ramp down to a small landing just over the water.
    Ramp,
    /// Steps down to a quay.
    Stair,
    /// A low plank dock on timber piles, running out from the deck's edge.
    Dock,
    /// A plank walkway at deck height.
    Walkway,
}

/// Gear on a deck.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum HProp {
    /// A market stall: a thatch roof on posts over a counter.
    Stall,
    /// A drying frame hung with fish.
    Frame,
    /// A round table.
    Table,
    /// A small woven smoke oven.
    Oven,
    /// A stone hearth with a fire and cauldrons on a tripod.
    Hearth,
    /// A cluster of clay jars.
    Jars,
    /// A salt pan: a shallow basin, white with salt.
    Pan,
    /// A well-sweep: a post, a pivoting beam, a basket on a rope.
    Sweep,
    /// Sacks of salt.
    Sacks,
    /// A basket.
    Basket,
    /// Coils of rope.
    Coils,
    /// A spinning wheel.
    Wheel,
    /// A loom: an upright frame strung with warp.
    Loom,
    /// Dye pots.
    Dyepots,
    /// A line of dyed yarn hanks.
    Yarnline,
    /// A tree-like post holding up a canopy (`HHull::Lid`).
    Treepost,
    /// A bench.
    Bench,
    /// A small boat moored alongside.
    Boat,
    /// A cloth awning on poles.
    Awning,
    /// A hanging lamp.
    Lamp,
    /// A mooring post.
    Mooring,
    /// A chimney pipe with smoke.
    Chimney,
    /// A thatch roof on posts with nothing under it.
    Roof,
}

/// How thick the deck's planking and beams are.
pub const SLAB: f32 = 0.3;
/// How far the supports reach below the water (what's under is unseen).
const UNDER: f32 = 3.0;

const ROPE: Rgb = [0.30, 0.25, 0.16];
const BASALT: Rgb = [0.20, 0.20, 0.22];
const ROOTSTONE: Rgb = [0.33, 0.33, 0.33];
const BEAM: Rgb = [0.24, 0.18, 0.12];
const WOOD: Rgb = [0.42, 0.33, 0.23];
const PALE_WOOD: Rgb = [0.56, 0.47, 0.35];
const THATCH: Rgb = [0.52, 0.46, 0.30];
const CLAY: Rgb = [0.55, 0.38, 0.26];
const SALT: Rgb = [0.92, 0.92, 0.90];
const LINEN: Rgb = [0.80, 0.76, 0.66];
const FISH: Rgb = [0.78, 0.60, 0.55];
const DARK_IN: Rgb = [0.07, 0.06, 0.05];
const WEAVE_COL: Rgb = [0.62, 0.60, 0.54];
const PLANK_COL: Rgb = [0.58, 0.56, 0.52];
const BRICK_DARK: Rgb = [0.36, 0.30, 0.28];
const DYES: [Rgb; 4] = [[0.58, 0.24, 0.20], [0.26, 0.36, 0.52], [0.70, 0.56, 0.26], [0.44, 0.50, 0.28]];

fn unit(seed: u64, k: u64) -> f32 {
    let x = (seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    ((x >> 40) as f32) / (1u64 << 24) as f32
}

/// A Horaro building: supports, quay, deck, rail, hulls, seams, doors,
/// portholes, ways on and off, gear.
pub(super) fn horaro_form(b: &mut Builder, gl: &mut Builder, deck_pts: &[(f32, f32)], look: &HoraroLook, f: &Frame, k: f32, seed: u64) {
    let floor = look.deck * k;
    let at = |x: f32, y: f32, z: f32| f.at(x, y, z);
    // What holds it up: from under the water to the slab's underside.
    for (i, &(kind, (x, y), r)) in look.supports.iter().enumerate() {
        let base = at(x, y, -UNDER);
        let h = floor - SLAB + UNDER;
        support(b, kind, base, r * k, h, seed ^ (i as u64 * 0x51));
    }
    if let Some((qpts, qz)) = look.quay {
        quay(b, f, qpts, qz * k, seed);
    }
    deck(b, f, deck_pts, floor);
    rail(b, f, deck_pts, floor, look.rail);
    // The lobes, their lashings, doors and portholes.
    let hulls: Vec<Lobe> = look
        .hulls
        .iter()
        .map(|&(kind, (x, y), yaw, (l, w, h))| Lobe { kind, c: at(x, y, floor), yaw: f.rot + yaw, a: l * 0.5 * k, bb: w * 0.5 * k, h: h * k })
        .collect();
    for (i, lobe) in hulls.iter().enumerate() {
        shell(b, lobe, seed ^ (i as u64 * 0x9D));
    }
    for &(i, j) in look.seams {
        if let (Some(p), Some(q)) = (hulls.get(i), hulls.get(j)) {
            seam(b, p, q);
        }
    }
    for &((x, y), yaw) in look.doors {
        round_door(b, at(x, y, floor), f.rot + yaw, 1.0 * k.sqrt(), 1.9 * k.sqrt());
    }
    for &((x, y), yaw, z) in look.portholes {
        porthole(b, gl, at(x, y, floor + z * k), f.rot + yaw, 0.32 * k.sqrt());
    }
    for &(kind, (x, y), yaw, len) in look.access {
        access(b, kind, at(x, y, floor), f.rot + yaw, len * k, floor);
    }
    for (i, &(kind, (x, y), rot, (w, d, h), z)) in look.props.iter().enumerate() {
        prop(b, gl, kind, at(x, y, floor + z * k), f.rot + rot, (w * k, d * k, h * k), seed ^ (i as u64 * 0x77));
    }
}

// ---- Supports ------------------------------------------------------------------

fn support(b: &mut Builder, kind: HSup, base: Vec3, r: f32, h: f32, seed: u64) {
    match kind {
        HSup::Rock => {
            // Rough facets: rings up the pillar, each corner its own radius,
            // the top a little narrower; two rope wraps under the deck.
            let rings = 5;
            let sides = 7;
            let pt = |i: usize, k: usize| {
                let a = (k % sides) as f32 / sides as f32 * std::f32::consts::TAU + i as f32 * 0.17;
                let jit = 0.68 + 0.64 * unit(seed, (i * 31 + k % sides) as u64);
                let taper = 1.08 - 0.26 * (i as f32 / rings as f32);
                let rr = r * jit * taper;
                base + vec3(a.cos() * rr, h * i as f32 / rings as f32, a.sin() * rr)
            };
            for i in 0..rings {
                for k in 0..sides {
                    let q = [pt(i, k), pt(i, k + 1), pt(i + 1, k + 1), pt(i + 1, k)];
                    let n = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or_zero();
                    let tone = 0.85 + 0.3 * unit(seed, (i * 97 + k) as u64);
                    b.quad(q, n, palette::scale(BASALT, tone));
                }
            }
            let top: Vec<Vec3> = (0..sides).map(|k| pt(rings, k)).collect();
            b.quad([top[0], top[2], top[4], top[6 % sides]], Vec3::Y, BASALT);
            for z in [h - 0.9, h - 1.4] {
                wrap(b, base + Vec3::Y * z, r * 0.92, 0.07, 10);
            }
        }
        HSup::Drum => {
            // Stacked stone drums of slightly different sizes, lashed at
            // their joints.
            let n = ((h / 0.9).ceil() as usize).max(2);
            let dh = h / n as f32;
            for i in 0..n {
                let rr = r * (0.86 + 0.24 * unit(seed, i as u64 + 3));
                let r2 = r * (0.86 + 0.24 * unit(seed, i as u64 + 11));
                let tone = 0.85 + 0.3 * unit(seed, i as u64 + 40);
                b.column(base + Vec3::Y * (dh * i as f32), rr, r2, dh, 9, palette::scale(BASALT, tone));
                if i > 0 && i % 2 == 0 {
                    wrap(b, base + Vec3::Y * (dh * i as f32 + 0.1), rr.max(r2) * 0.95, 0.07, 9);
                }
            }
        }
        HSup::Pier => {
            // Brick, laid in courses: the quarried texture, dark.
            tex_block(b, base, r * 1.8, r * 1.8, h, 0.0, BRICK_DARK, unit(seed, 5) * 6.0);
        }
        HSup::Root => {
            // A grown trunk flaring at the foot into roots that run out
            // and down.
            let rings = 6;
            let sides = 10;
            let pt = |i: usize, k: usize| {
                let s = i as f32 / rings as f32;
                let a = (k % sides) as f32 / sides as f32 * std::f32::consts::TAU;
                let flare = 1.0 + 1.3 * (1.0 - s).powi(3);
                let lobes = 1.0 + 0.22 * (3.0 * a + s * 2.0).sin() * (1.0 - s * 0.5);
                let rr = r * flare * lobes;
                base + vec3(a.cos() * rr, h * s, a.sin() * rr)
            };
            for i in 0..rings {
                for k in 0..sides {
                    let q = [pt(i, k), pt(i, k + 1), pt(i + 1, k + 1), pt(i + 1, k)];
                    let n = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or_zero();
                    let tone = 0.9 + 0.2 * unit(seed, (i * 53 + k) as u64);
                    b.quad(q, n, palette::scale(ROOTSTONE, tone));
                }
            }
            for k in 0..4 {
                let a = k as f32 / 4.0 * std::f32::consts::TAU + unit(seed, k as u64 + 60) * 0.8;
                let dir = vec3(a.cos(), 0.0, a.sin());
                let from = base + dir * (r * 1.6) + Vec3::Y * (UNDER * 0.4 + 0.3);
                let to = base + dir * (r * 3.4) + Vec3::Y * (UNDER * 0.1);
                b.stick(from, to, r * 0.5, ROOTSTONE);
            }
        }
        HSup::Post => {
            b.column(base, r * 1.05, r * 0.9, h, 8, BEAM);
            wrap(b, base + Vec3::Y * (h - 0.5), r * 0.95, 0.05, 8);
        }
    }
}

/// A facing turned to point away from `axis` (a point on the lobe's
/// middle line level with the face): the mesh winds either way.
fn outward(n: Vec3, at: Vec3, axis: Vec3) -> Vec3 {
    let out = vec3(at.x - axis.x, (at.y - axis.y).max(0.0) * 0.3, at.z - axis.z);
    if n.dot(out) < 0.0 { -n } else { n }
}

/// A rope wrapped round something upright: a ring of sticks.
fn wrap(b: &mut Builder, c: Vec3, r: f32, t: f32, n: usize) {
    for k in 0..n {
        let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
        b.stick(c + vec3(a0.cos() * r, 0.0, a0.sin() * r), c + vec3(a1.cos() * r, 0.0, a1.sin() * r), t, ROPE);
    }
}

/// A box faced with the quarried texture (laid from `off` metres along).
#[allow(clippy::too_many_arguments)]
fn tex_block(b: &mut Builder, base: Vec3, w: f32, d: f32, h: f32, rot: f32, col: Rgb, off: f32) {
    let c = palette::lin(col);
    let (s, cs) = rot.sin_cos();
    let ax = vec3(cs, 0.0, s) * (w * 0.5);
    let az = vec3(-s, 0.0, cs) * (d * 0.5);
    let up = vec3(0.0, h, 0.0);
    let p = [base - ax - az, base + ax - az, base + ax + az, base - ax + az];
    let mut along = off;
    for k in 0..4 {
        let (a, bb) = (p[k], p[(k + 1) % 4]);
        let n = (bb - a).cross(up).normalize_or_zero();
        let n = if n.dot((a + bb) * 0.5 - base) < 0.0 { -n } else { n };
        let len = (bb - a).length();
        let (u0, u1) = (stone::u_ashlar(along % stone::ASHLAR_W), stone::u_ashlar((along % stone::ASHLAR_W + len).min(stone::ASHLAR_W)));
        let (v0, v1) = (0.0, h / stone::TEX_H);
        b.quad_tex([a, bb, bb + up, a + up], [n; 4], [c; 4], [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
        along += len;
    }
    b.quad([p[0] + up, p[1] + up, p[2] + up, p[3] + up], Vec3::Y, col);
}

// ---- Quay, deck, rail --------------------------------------------------------------

/// A stone quay: dark brick faces, a slate top, mooring posts along its
/// outer edge.
fn quay(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], top: f32, seed: u64) {
    let ring = inset(f, pts, 0.0);
    let n = ring.len();
    let c = palette::lin(BRICK_DARK);
    let mut along = unit(seed, 9) * 8.0;
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (fm(f, ring[k]), fm(f, ring[j]));
        let e = bb.sub(a);
        let len = e.len().max(1e-4);
        let nn = vec3(e.y, 0.0, -e.x).normalize_or_zero();
        let (y0, y1) = (f.y - UNDER, f.y + top);
        let (u0, u1) = (stone::u_ashlar(along % stone::ASHLAR_W), stone::u_ashlar((along % stone::ASHLAR_W + len).min(stone::ASHLAR_W)));
        let (v0, v1) = (0.0, (y1 - y0) / stone::TEX_H);
        b.quad_tex([to3(a, y0), to3(bb, y0), to3(bb, y1), to3(a, y1)], [nn; 4], [c; 4], [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]);
        along += len;
    }
    // The top, as a fan, slate; a coping line of lighter stone.
    let cen = ring.iter().fold(V2::new(0.0, 0.0), |s, p| s.add(*p)).scale(1.0 / n as f32);
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (fm(f, ring[k]), fm(f, ring[j]));
        let m = fm(f, cen);
        b.quad([to3(m, f.y + top), to3(a, f.y + top), to3(bb, f.y + top), to3(m, f.y + top)], Vec3::Y, [0.30, 0.30, 0.32]);
    }
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (fm(f, ring[k]), fm(f, ring[j]));
        b.stick(to3(a, f.y + top + 0.04), to3(bb, f.y + top + 0.04), 0.16, [0.42, 0.41, 0.42]);
    }
}

/// The plank deck: a dark beam round the edge, the boards on top (the
/// plank texture, boards running along the frame), the underside dark.
fn deck(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], floor: f32) {
    let ring = inset(f, pts, 0.0);
    let n = ring.len();
    let (y0, y1) = (f.y + floor - SLAB, f.y + floor);
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (fm(f, ring[k]), fm(f, ring[j]));
        let e = bb.sub(a);
        let nn = vec3(e.y, 0.0, -e.x).normalize_or_zero();
        b.quad([to3(a, y0), to3(bb, y0), to3(bb, y1), to3(a, y1)], nn, BEAM);
    }
    // The boards: a fan from the middle, texture coordinates from the
    // frame's own metres (boards along x).
    let cen = ring.iter().fold(V2::new(0.0, 0.0), |s, p| s.add(*p)).scale(1.0 / n as f32);
    let c = palette::lin(PLANK_COL);
    let uv = |q: V2| [stone::u_plank((q.y + stone::PLANK_W * 0.5).clamp(0.0, stone::PLANK_W)), q.x / stone::TEX_H];
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, bb) = (ring[k], ring[j]);
        b.quad_tex([to3(fm(f, cen), y1), to3(fm(f, a), y1), to3(fm(f, bb), y1), to3(fm(f, cen), y1)], [Vec3::Y; 4], [c; 4], [uv(cen), uv(a), uv(bb), uv(cen)]);
        b.quad([to3(fm(f, cen), y0), to3(fm(f, bb), y0), to3(fm(f, a), y0), to3(fm(f, cen), y0)], -Vec3::Y, BEAM);
    }
}

/// A rope rail along the runs given (fractions of the way round): posts
/// every stride and a half, two ropes between, the upper one sagging.
fn rail(b: &mut Builder, f: &Frame, pts: &[(f32, f32)], floor: f32, runs: &[(f32, f32)]) {
    let ring = inset(f, pts, 0.22);
    let n = ring.len();
    let lens: Vec<f32> = (0..n).map(|k| ring[k].dist(ring[(k + 1) % n])).collect();
    let total: f32 = lens.iter().sum();
    let point_at = |t: f32| -> V2 {
        let mut d = t.rem_euclid(1.0) * total;
        for k in 0..n {
            if d <= lens[k] || k + 1 == n {
                return ring[k].lerp(ring[(k + 1) % n], (d / lens[k].max(1e-4)).min(1.0));
            }
            d -= lens[k];
        }
        ring[0]
    };
    let y = f.y + floor;
    for &(t0, t1) in runs {
        let len = (t1 - t0).max(0.0) * total;
        if len < 0.5 {
            continue;
        }
        let bays = (len / 1.6).ceil().max(1.0) as usize;
        let posts: Vec<V2> = (0..=bays).map(|i| fm(f, point_at(t0 + (t1 - t0) * i as f32 / bays as f32))).collect();
        for p in &posts {
            b.stick(to3(*p, y - 0.1), to3(*p, y + 1.0), 0.11, BEAM);
        }
        for w in posts.windows(2) {
            let (a, c) = (to3(w[0], y + 0.95), to3(w[1], y + 0.95));
            let m = (a + c) * 0.5 - Vec3::Y * 0.12;
            b.stick(a, m, 0.035, ROPE);
            b.stick(m, c, 0.035, ROPE);
            b.stick(to3(w[0], y + 0.5), to3(w[1], y + 0.5), 0.03, ROPE);
        }
    }
}

// ---- Hulls ---------------------------------------------------------------------------

/// One woven lobe: its kind, floor centre, turn, half length, half width,
/// height.
struct Lobe {
    kind: HHull,
    c: Vec3,
    yaw: f32,
    a: f32,
    bb: f32,
    h: f32,
}

impl Lobe {
    /// The plan's radius factor at height fraction `s` (0 the floor, 1 the
    /// crown) and the exponent of its plan (2 an ellipse, more a rounded
    /// box).
    /// Every lobe is widest at the floor (Laz, 17 Oct: "the widest part
    /// of the shape should sit on the deck"): upright to `c` of its
    /// height, then rounding in to the crown.
    fn profile(&self, s: f32) -> f32 {
        let s = s.clamp(0.0, 1.0);
        let (c, e) = match self.kind {
            HHull::Egg => (0.12, 2.0),
            HHull::Dome => (0.35, 2.5),
            HHull::Hull => (0.45, 3.0),
            HHull::Longhouse | HHull::Arcade => (0.0, 2.0),
            HHull::Lid => (0.0, 1.6),
        };
        if s < c {
            1.0
        } else {
            (1.0 - ((s - c) / (1.0 - c)).powf(e)).max(0.0).powf(1.0 / e)
        }
    }
    fn plan_exp(&self) -> f32 {
        match self.kind {
            HHull::Hull => 2.8,
            HHull::Longhouse | HHull::Arcade => 4.0,
            _ => 2.0,
        }
    }
    /// A point on the surface: ring fraction `s`, angle `phi` round.
    fn point(&self, s: f32, phi: f32) -> Vec3 {
        let e = self.plan_exp();
        let (cp, sp) = (phi.cos(), phi.sin());
        let r = (cp.abs().powf(e) / self.a.powf(e) + sp.abs().powf(e) / self.bb.powf(e)).powf(-1.0 / e);
        let rho = self.profile(s);
        let (lx, ly) = (cp * r * rho, sp * r * rho);
        let (sn, cs) = self.yaw.sin_cos();
        self.c + vec3(lx * cs - ly * sn, self.h * s, lx * sn + ly * cs)
    }
    /// Is a point inside this lobe?
    fn inside(&self, p: Vec3) -> bool {
        let d = p - self.c;
        let s = d.y / self.h;
        if !(0.0..=1.0).contains(&s) {
            return false;
        }
        let (sn, cs) = self.yaw.sin_cos();
        let (lx, ly) = (d.x * cs + d.z * sn, -d.x * sn + d.z * cs);
        let e = self.plan_exp();
        let rho = self.profile(s).max(1e-3);
        (lx.abs() / (self.a * rho)).powf(e) + (ly.abs() / (self.bb * rho)).powf(e) < 1.0
    }
}

/// The woven shell of a lobe: rings up it, the weave texture laid round in
/// laps, rope hoops every few rings and, on the rounder kinds, upright
/// withies. A longhouse is a barrel with woven end walls; a lid is open
/// below.
fn shell(b: &mut Builder, l: &Lobe, seed: u64) {
    if matches!(l.kind, HHull::Longhouse | HHull::Arcade) {
        return longhouse(b, l, seed);
    }
    let rings = if l.kind == HHull::Lid { 5 } else { 11 };
    let sides = ((l.a + l.bb) * 2.2).round().clamp(18.0, 44.0) as usize;
    let c = palette::lin(WEAVE_COL);
    let mut v = 0.0f32;
    let s_lo = if l.kind == HHull::Lid { 0.55 } else { 0.0 };
    for i in 0..rings {
        let (s0, s1) = (s_lo + (1.0 - s_lo) * i as f32 / rings as f32, s_lo + (1.0 - s_lo) * (i + 1) as f32 / rings as f32);
        let dv = (l.point(s1, 0.0) - l.point(s0, 0.0)).length() / stone::TEX_H;
        let mut girth = unit(seed, i as u64) * 3.0;
        for k in 0..sides {
            let (p0, p1) = (k as f32 / sides as f32 * std::f32::consts::TAU, (k + 1) as f32 / sides as f32 * std::f32::consts::TAU);
            let q = [l.point(s0, p0), l.point(s0, p1), l.point(s1, p1), l.point(s1, p0)];
            let n = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or_zero();
            let n = if n.length_squared() < 0.5 { (q[1] - q[0]).cross(q[2] - q[0]).normalize_or_zero() } else { n };
            let n = outward(n, (q[0] + q[2]) * 0.5, l.c);
            let w = (q[1] - q[0]).length().max(0.01);
            let mut u0 = girth % stone::WEAVE_W;
            let mut u1 = u0 + w;
            if u1 > stone::WEAVE_W {
                u0 -= u1 - stone::WEAVE_W;
                u1 = stone::WEAVE_W;
            }
            let (u0, u1) = (stone::u_weave(u0.max(0.0)), stone::u_weave(u1));
            // The top ring closes to a point: a triangle.
            if i + 1 == rings && l.kind != HHull::Lid {
                b.quad_tex([q[0], q[1], q[2], q[2]], [n; 4], [c; 4], [[u0, v], [u1, v], [u1, v + dv], [u1, v + dv]]);
            } else {
                b.quad_tex(q, [n; 4], [c; 4], [[u0, v], [u1, v], [u1, v + dv], [u0, v + dv]]);
            }
            girth += w;
        }
        v += dv;
        // Hoops and withies.
        if i % 3 == 1 && i + 1 < rings {
            for k in 0..sides {
                let (p0, p1) = (k as f32 / sides as f32 * std::f32::consts::TAU, (k + 1) as f32 / sides as f32 * std::f32::consts::TAU);
                let (q0, q1) = (l.point(s1, p0), l.point(s1, p1));
                let out = |q: Vec3| q + (q - vec3(l.c.x, q.y, l.c.z)).normalize_or_zero() * 0.05;
                b.stick(out(q0), out(q1), 0.07, ROPE);
            }
        }
    }
    if matches!(l.kind, HHull::Dome | HHull::Egg) {
        let n_withy = (sides / 4).max(4);
        for k in 0..n_withy {
            let phi = k as f32 / n_withy as f32 * std::f32::consts::TAU + 0.2;
            for i in 0..rings - 1 {
                let (s0, s1) = (i as f32 / rings as f32, (i + 1) as f32 / rings as f32);
                let (q0, q1) = (l.point(s0, phi), l.point(s1, phi));
                let out = |q: Vec3| q + (q - vec3(l.c.x, q.y, l.c.z)).normalize_or_zero() * 0.04;
                b.stick(out(q0), out(q1), 0.06, palette::scale(ROPE, 0.8));
            }
        }
    }
    if l.kind == HHull::Lid {
        // The underside, dark, so the lid reads as a roof from below.
        let s = s_lo;
        for k in 0..sides {
            let (p0, p1) = (k as f32 / sides as f32 * std::f32::consts::TAU, (k + 1) as f32 / sides as f32 * std::f32::consts::TAU);
            let top = l.c + Vec3::Y * (l.h * 0.98);
            b.quad([top, l.point(s, p1), l.point(s, p0), top], -Vec3::Y, palette::scale(WEAVE_COL, 0.35));
        }
    }
}

/// A barrel-vaulted longhouse: arched ribs along it, the weave between,
/// woven end walls.
fn longhouse(b: &mut Builder, l: &Lobe, seed: u64) {
    let open = l.kind == HHull::Arcade;
    let arcs = 9;
    // A covered walk is woven over its top only; the ribs go all the way
    // down.
    let (ta, tb) = if open { (0.24 * std::f32::consts::PI, 0.76 * std::f32::consts::PI) } else { (0.0, std::f32::consts::PI) };
    let along = ((l.a * 2.0) / 1.0).round().clamp(6.0, 40.0) as usize;
    let (sn, cs) = l.yaw.sin_cos();
    let fwd = vec3(cs, 0.0, sn);
    let side = vec3(-sn, 0.0, cs);
    let c = palette::lin(WEAVE_COL);
    // A point: x along (-a..a), angle t over the arch (0 one eave, pi the other).
    let pt = |x: f32, t: f32| {
        // Rounded ends: the arch shrinks a little over the last metre.
        let endf = (1.0 - ((x.abs() - (l.a - 1.0)).max(0.0) / 1.0).powi(2) * 0.25).max(0.6);
        l.c + fwd * x + side * (t.cos() * l.bb * endf) + Vec3::Y * (t.sin() * l.h * endf)
    };
    for j in 0..arcs {
        let (t0, t1) = (ta + (tb - ta) * j as f32 / arcs as f32, ta + (tb - ta) * (j + 1) as f32 / arcs as f32);
        let v0 = j as f32 / arcs as f32 * (std::f32::consts::PI * (l.bb + l.h) * 0.5) / stone::TEX_H;
        let v1 = (j + 1) as f32 / arcs as f32 * (std::f32::consts::PI * (l.bb + l.h) * 0.5) / stone::TEX_H;
        let mut girth = unit(seed, j as u64) * 3.0;
        for i in 0..along {
            let (x0, x1) = (-l.a + 2.0 * l.a * i as f32 / along as f32, -l.a + 2.0 * l.a * (i + 1) as f32 / along as f32);
            let q = [pt(x0, t0), pt(x1, t0), pt(x1, t1), pt(x0, t1)];
            let n = (q[1] - q[0]).cross(q[3] - q[0]).normalize_or_zero();
            let n = outward(n, (q[0] + q[2]) * 0.5, l.c + fwd * ((x0 + x1) * 0.5));
            let w = (q[1] - q[0]).length();
            let mut u0 = girth % stone::WEAVE_W;
            let mut u1 = u0 + w;
            if u1 > stone::WEAVE_W {
                u0 -= u1 - stone::WEAVE_W;
                u1 = stone::WEAVE_W;
            }
            b.quad_tex(q, [n; 4], [c; 4], [[stone::u_weave(u0.max(0.0)), v0], [stone::u_weave(u1), v0], [stone::u_weave(u1), v1], [stone::u_weave(u0.max(0.0)), v1]]);
            girth += w;
        }
    }
    // Ribs: an arch every metre and a half, and the ridge.
    let n_rib = ((l.a * 2.0) / 1.5).round().max(2.0) as usize;
    for r in 0..=n_rib {
        let x = -l.a + 2.0 * l.a * r as f32 / n_rib as f32;
        for j in 0..arcs {
            let (t0, t1) = (j as f32 / arcs as f32 * std::f32::consts::PI, (j + 1) as f32 / arcs as f32 * std::f32::consts::PI);
            let out = |q: Vec3| q + (q - (l.c + fwd * x)).normalize_or_zero() * 0.05;
            b.stick(out(pt(x, t0)), out(pt(x, t1)), 0.08, ROPE);
        }
    }
    b.stick(pt(-l.a, std::f32::consts::FRAC_PI_2) + Vec3::Y * 0.06, pt(l.a, std::f32::consts::FRAC_PI_2) + Vec3::Y * 0.06, 0.1, BEAM);
    if open {
        return;
    }
    // The end walls: woven, flat, a fan from the eave's middle.
    for (x, dir) in [(-l.a, -1.0f32), (l.a, 1.0)] {
        let mid = l.c + fwd * x + Vec3::Y * (l.h * 0.35);
        let n = fwd * dir;
        for j in 0..arcs {
            let (t0, t1) = (j as f32 / arcs as f32 * std::f32::consts::PI, (j + 1) as f32 / arcs as f32 * std::f32::consts::PI);
            let (p0, p1) = (pt(x, t0) + n * 0.02, pt(x, t1) + n * 0.02);
            let uv = |q: Vec3| {
                let d = q - mid;
                [stone::u_weave((d.dot(side) + 4.0).clamp(0.0, stone::WEAVE_W)), d.y / stone::TEX_H]
            };
            b.quad_tex([mid, p0, p1, mid], [n; 4], [c; 4], [uv(mid), uv(p0), uv(p1), uv(mid)]);
        }
        // The eave line along the floor.
        b.stick(pt(x, 0.0), pt(x, std::f32::consts::PI), 0.1, BEAM);
    }
    let _ = seed;
}

/// Rope lashed round where lobe `q` meets lobe `p`: for each upright line
/// of `q`, the spot where it comes out of `p`; those joined up and swept
/// with rope, a knot every stride.
fn seam(b: &mut Builder, p: &Lobe, q: &Lobe) {
    let sides = 48;
    let steps = 40;
    let mut pts: Vec<Vec3> = Vec::new();
    for k in 0..sides {
        let phi = k as f32 / sides as f32 * std::f32::consts::TAU;
        let mut last_in: Option<Vec3> = None;
        let mut found = None;
        for i in 0..=steps {
            let s = i as f32 / steps as f32;
            let pt = q.point(s, phi);
            if p.inside(pt) {
                last_in = Some(pt);
            } else if let Some(li) = last_in {
                found = Some((li + pt) * 0.5);
                break;
            } else {
                break;
            }
        }
        pts.push(match found {
            Some(f) => f,
            None => Vec3::NAN,
        });
    }
    // Sweep the rope along the runs of found points.
    let n = pts.len();
    for k in 0..n {
        let (a, c) = (pts[k], pts[(k + 1) % n]);
        if a.is_nan() || c.is_nan() {
            continue;
        }
        if a.distance(c) > 2.5 {
            continue;
        }
        let out = |v: Vec3| v + (v - vec3(q.c.x, v.y, q.c.z)).normalize_or_zero() * 0.06;
        b.stick(out(a), out(c), 0.09, ROPE);
        if k % 6 == 0 {
            let m = out(a);
            b.block(m - Vec3::Y * 0.09, 0.2, 0.2, 0.18, 0.0, palette::scale(ROPE, 0.85));
        }
    }
}

/// A round-topped doorway in a woven wall: the dark opening, a plank leaf
/// set back in it, a rope hoop round.
fn round_door(b: &mut Builder, c: Vec3, facing: f32, w: f32, h: f32) {
    let out = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let base = c + out * 0.12;
    let r = w * 0.5;
    let straight = h - r;
    // The opening: a rectangle and a half-disc, dark.
    b.quad([base - right * r, base + right * r, base + right * r + Vec3::Y * straight, base - right * r + Vec3::Y * straight], out, DARK_IN);
    let top = base + Vec3::Y * straight;
    let n = 8;
    for k in 0..n {
        let (a0, a1) = (k as f32 / n as f32 * std::f32::consts::PI, (k + 1) as f32 / n as f32 * std::f32::consts::PI);
        let p0 = top + right * (a0.cos() * r) + Vec3::Y * (a0.sin() * r);
        let p1 = top + right * (a1.cos() * r) + Vec3::Y * (a1.sin() * r);
        b.quad([top, p0, p1, top], out, DARK_IN);
    }
    // The leaf, planked, before the dark of the opening with a little of
    // it showing round the edge.
    let leaf = base + out * 0.03;
    let lw = r * 0.82;
    b.quad([leaf - right * lw, leaf + right * lw, leaf + right * lw + Vec3::Y * (straight + r * 0.55), leaf - right * lw + Vec3::Y * (straight + r * 0.55)], out, PALE_WOOD);
    for k in 1..4 {
        let x = -lw + 2.0 * lw * k as f32 / 4.0;
        b.stick(leaf + right * x, leaf + right * x + Vec3::Y * (straight + r * 0.5), 0.02, palette::scale(PALE_WOOD, 0.6));
    }
    // The hoop round the opening.
    for k in 0..n {
        let (a0, a1) = (k as f32 / n as f32 * std::f32::consts::PI, (k + 1) as f32 / n as f32 * std::f32::consts::PI);
        let rr = r + 0.08;
        let p0 = top + right * (a0.cos() * rr) + Vec3::Y * (a0.sin() * rr);
        let p1 = top + right * (a1.cos() * rr) + Vec3::Y * (a1.sin() * rr);
        b.stick(p0 + out * 0.02, p1 + out * 0.02, 0.08, ROPE);
    }
    for sgn in [-1.0f32, 1.0] {
        b.stick(base + right * (sgn * (r + 0.08)), top + right * (sgn * (r + 0.08)), 0.08, ROPE);
    }
    b.stick(base - right * (r + 0.1) - Vec3::Y * 0.02, base + right * (r + 0.1) - Vec3::Y * 0.02, 0.1, BEAM);
}

/// A porthole: a rope ring round a lit round window.
fn porthole(b: &mut Builder, gl: &mut Builder, c: Vec3, facing: f32, r: f32) {
    let out = vec3(facing.cos(), 0.0, facing.sin());
    let right = vec3(-facing.sin(), 0.0, facing.cos());
    let n = 10;
    let cc = c + out * 0.1;
    for k in 0..n {
        let (a0, a1) = (k as f32 / n as f32 * std::f32::consts::TAU, (k + 1) as f32 / n as f32 * std::f32::consts::TAU);
        let p0 = cc + right * (a0.cos() * r) + Vec3::Y * (a0.sin() * r);
        let p1 = cc + right * (a1.cos() * r) + Vec3::Y * (a1.sin() * r);
        b.stick(p0, p1, 0.07, ROPE);
    }
    gl.patch(cc - out * 0.03, r * 1.5, r * 1.5, facing, palette::WINDOW);
}

// ---- Ways on and off ----------------------------------------------------------------

/// `at` is the spot on the deck's edge at floor height; `yaw` the way out.
fn access(b: &mut Builder, kind: HAcc, at: Vec3, yaw: f32, len: f32, floor: f32) {
    let out = vec3(yaw.cos(), 0.0, yaw.sin());
    let right = vec3(-yaw.sin(), 0.0, yaw.cos());
    let water = at.y - floor;
    match kind {
        HAcc::Ladder => {
            // Two rails leaning out from the deck down past the water,
            // rungs between.
            let top = at + Vec3::Y * 0.9;
            let foot = at + out * ((floor + 0.9) * 0.3) - Vec3::Y * (floor + 0.6);
            for sgn in [-1.0f32, 1.0] {
                b.stick(top + right * (sgn * 0.3), foot + right * (sgn * 0.3), 0.08, PALE_WOOD);
            }
            let n = ((floor + 1.5) / 0.32).round() as usize;
            for i in 1..n {
                let t = i as f32 / n as f32;
                let p = top.lerp(foot, t);
                b.stick(p - right * 0.3, p + right * 0.3, 0.05, PALE_WOOD);
            }
            let _ = water;
        }
        HAcc::Ramp => {
            // Planks sloping down to a landing a metre over the water,
            // rope rails either side, the landing on two piles.
            let low = at + out * len - Vec3::Y * (floor - 1.0);
            let w = 1.6;
            let mid = (at + low) * 0.5;
            let d = low - at;
            let up = d.cross(right).normalize_or_zero();
            let up = if up.y < 0.0 { -up } else { up };
            b.cuboid(mid - up * 0.06, d * 0.5, right * (w * 0.5), up * 0.06, PALE_WOOD);
            let steps = (d.length() / 0.5).round() as usize;
            for i in 0..steps {
                let p = at.lerp(low, (i as f32 + 0.5) / steps as f32) + up * 0.02;
                b.stick(p - right * (w * 0.5), p + right * (w * 0.5), 0.05, palette::scale(PALE_WOOD, 0.7));
            }
            for sgn in [-1.0f32, 1.0] {
                let posts = 3;
                for i in 0..=posts {
                    let p = at.lerp(low, i as f32 / posts as f32) + right * (sgn * (w * 0.5 + 0.05));
                    b.stick(p, p + Vec3::Y * 0.95, 0.09, BEAM);
                    if i > 0 {
                        let q = at.lerp(low, (i - 1) as f32 / posts as f32) + right * (sgn * (w * 0.5 + 0.05));
                        b.stick(q + Vec3::Y * 0.9, p + Vec3::Y * 0.9, 0.035, ROPE);
                    }
                }
            }
            // The landing.
            let land = low + out * 1.2;
            b.block(land - Vec3::Y * 0.12, 2.4, w + 0.6, 0.12, yaw, PALE_WOOD);
            for sgn in [-1.0f32, 1.0] {
                let p = land + out * 0.9 + right * (sgn * (w * 0.5));
                b.column(p - Vec3::Y * (UNDER + 1.0), 0.14, 0.13, UNDER + 0.9, 7, BEAM);
            }
        }
        HAcc::Stair => {
            // Steps down `len` metres out to a quay (its top at a third of
            // the deck's height).
            let drop = floor * 0.6;
            let n = (drop / 0.25).round().max(2.0) as usize;
            let w = 1.6;
            for i in 0..n {
                let t = i as f32 / n as f32;
                let p = at + out * (len * t) - Vec3::Y * (drop * t);
                b.block(p - Vec3::Y * (drop * (1.0 - t) * 0.0 + 0.2), len / n as f32 + 0.02, w, 0.2, yaw, PALE_WOOD);
            }
            for sgn in [-1.0f32, 1.0] {
                let a = at + right * (sgn * (w * 0.5 + 0.05));
                let c = at + out * len - Vec3::Y * drop + right * (sgn * (w * 0.5 + 0.05));
                b.stick(a, a + Vec3::Y * 0.9, 0.09, BEAM);
                b.stick(c, c + Vec3::Y * 0.9, 0.09, BEAM);
                b.stick(a + Vec3::Y * 0.9, c + Vec3::Y * 0.9, 0.035, ROPE);
            }
        }
        HAcc::Dock => {
            // A low dock a metre over the water, a short ladder down to it
            // from the deck, piles under, a rail along one side.
            let w = 2.4;
            let base = at - Vec3::Y * (floor - 1.0) + out * (len * 0.5 + 0.3);
            b.block(base - Vec3::Y * 0.15, len, w, 0.15, yaw, PALE_WOOD);
            let piles = (len / 2.0).round().max(1.0) as usize;
            for i in 0..=piles {
                for sgn in [-1.0f32, 1.0] {
                    let p = at + out * (0.3 + len * i as f32 / piles as f32) + right * (sgn * (w * 0.45)) - Vec3::Y * (floor - 1.0);
                    b.column(p - Vec3::Y * (UNDER + 1.0), 0.15, 0.14, UNDER + 1.05, 7, BEAM);
                    if sgn > 0.0 {
                        b.stick(p, p + Vec3::Y * 0.9, 0.09, BEAM);
                    }
                }
            }
            for i in 0..piles {
                let p = at + out * (0.3 + len * i as f32 / piles as f32) + right * (w * 0.45) - Vec3::Y * (floor - 1.0) + Vec3::Y * 0.85;
                let q = at + out * (0.3 + len * (i + 1) as f32 / piles as f32) + right * (w * 0.45) - Vec3::Y * (floor - 1.0) + Vec3::Y * 0.85;
                b.stick(p, q, 0.035, ROPE);
            }
            // The short ladder from the deck's edge down to it.
            let top = at + Vec3::Y * 0.7;
            let foot = at + out * 0.5 - Vec3::Y * (floor - 1.0);
            for sgn in [-1.0f32, 1.0] {
                b.stick(top + right * (sgn * 0.3), foot + right * (sgn * 0.3), 0.08, PALE_WOOD);
            }
            let n = ((floor - 0.3) / 0.32).round().max(2.0) as usize;
            for i in 1..n {
                let p = top.lerp(foot, i as f32 / n as f32);
                b.stick(p - right * 0.3, p + right * 0.3, 0.05, PALE_WOOD);
            }
        }
        HAcc::Walkway => {
            let w = 1.6;
            let base = at + out * (len * 0.5);
            b.block(base - Vec3::Y * 0.2, len, w, 0.2, yaw, PALE_WOOD);
            let piles = (len / 3.0).round().max(1.0) as usize;
            for i in 0..=piles {
                for sgn in [-1.0f32, 1.0] {
                    let p = at + out * (len * i as f32 / piles as f32) + right * (sgn * (w * 0.4));
                    b.column(p - Vec3::Y * (floor + UNDER), 0.15, 0.14, floor + UNDER - 0.2, 7, BEAM);
                    b.stick(p, p + Vec3::Y * 0.9, 0.09, BEAM);
                }
                if i > 0 {
                    for sgn in [-1.0f32, 1.0] {
                        let p = at + out * (len * (i - 1) as f32 / piles as f32) + right * (sgn * (w * 0.4)) + Vec3::Y * 0.85;
                        let q = at + out * (len * i as f32 / piles as f32) + right * (sgn * (w * 0.4)) + Vec3::Y * 0.85;
                        b.stick(p, q, 0.035, ROPE);
                    }
                }
            }
        }
    }
}

// ---- Gear -------------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn prop(b: &mut Builder, gl: &mut Builder, kind: HProp, base: Vec3, rot: f32, (w, d, h): (f32, f32, f32), seed: u64) {
    let (sn, cs) = rot.sin_cos();
    let fwd = vec3(cs, 0.0, sn);
    let side = vec3(-sn, 0.0, cs);
    match kind {
        HProp::Stall | HProp::Roof => {
            // Four posts, a thatch roof pitched a little toward the front,
            // and for a stall a counter with jars on it.
            let corners = [base + fwd * (w * 0.5) + side * (d * 0.5), base + fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) + side * (d * 0.5)];
            for (i, c) in corners.iter().enumerate() {
                let hh = if i < 2 { h - 0.3 } else { h };
                b.stick(*c, *c + Vec3::Y * hh, 0.1, BEAM);
            }
            let front = base + fwd * (w * 0.5 + 0.3) + Vec3::Y * (h - 0.3);
            let back = base - fwd * (w * 0.5 + 0.3) + Vec3::Y * h;
            let mid = (front + back) * 0.5 + Vec3::Y * 0.08;
            let along = (front - back) * 0.5;
            let up = along.cross(side).normalize_or_zero();
            let up = if up.y < 0.0 { -up } else { up };
            b.cuboid(mid, along, side * (d * 0.5 + 0.35), up * 0.09, THATCH);
            if kind == HProp::Roof {
                return;
            }
            b.block(base + fwd * (w * 0.2), w * 0.5, d * 0.9, 0.9, rot, WOOD);
            for i in 0..3 {
                let p = base + fwd * (w * 0.2) + side * ((i as f32 - 1.0) * d * 0.3) + Vec3::Y * 0.9;
                b.column(p, 0.14, 0.1, 0.35 + 0.1 * unit(seed, i as u64), 7, CLAY);
            }
        }
        HProp::Frame => {
            // Two posts, a crossbar, lines with fish hung in a row.
            for sgn in [-1.0f32, 1.0] {
                let p = base + side * (sgn * w * 0.5);
                b.stick(p, p + Vec3::Y * h, 0.08, BEAM);
            }
            for row in 0..2 {
                let y = h - 0.1 - row as f32 * 0.55;
                b.stick(base - side * (w * 0.5) + Vec3::Y * y, base + side * (w * 0.5) + Vec3::Y * y, 0.03, ROPE);
                let n = (w / 0.3) as usize;
                for i in 0..n {
                    let x = -w * 0.45 + w * 0.9 * (i as f32 + 0.5) / n as f32;
                    let p = base + side * x + Vec3::Y * (y - 0.22);
                    b.patch(p, 0.12, 0.36, rot, FISH);
                }
            }
        }
        HProp::Table => {
            b.column(base, 0.08, 0.08, h - 0.05, 6, BEAM);
            b.column(base + Vec3::Y * (h - 0.05), w * 0.5, w * 0.5, 0.05, 12, PALE_WOOD);
        }
        HProp::Oven => {
            let l = Lobe { kind: HHull::Dome, c: base, yaw: rot, a: w * 0.5, bb: d * 0.5, h };
            shell(b, &l, seed);
            b.patch(base + fwd * (w * 0.5) + Vec3::Y * 0.3, 0.45, 0.5, rot, DARK_IN);
            b.column(base + Vec3::Y * (h - 0.1), 0.12, 0.1, 0.4, 6, CLAY);
            b.smoke(base + Vec3::Y * (h + 0.3), 0.3);
        }
        HProp::Hearth => {
            // A ring of stones, the fire, a tripod with cauldrons.
            let n = 9;
            for i in 0..n {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                let p = base + vec3(a.cos(), 0.0, a.sin()) * (w * 0.5);
                b.block(p, 0.3, 0.28, 0.25 + 0.1 * unit(seed, i as u64), a, [0.3, 0.3, 0.3]);
            }
            b.flame(base + Vec3::Y * 0.05, w * 0.25, 0.9);
            let top = base + Vec3::Y * h;
            for i in 0..3 {
                let a = i as f32 / 3.0 * std::f32::consts::TAU;
                b.stick(base + vec3(a.cos(), 0.0, a.sin()) * (w * 0.55), top, 0.06, BEAM);
            }
            b.stick(top, top - Vec3::Y * 0.5, 0.025, ROPE);
            b.column(top - Vec3::Y * 0.85, 0.3, 0.26, 0.35, 8, [0.14, 0.14, 0.15]);
        }
        HProp::Jars => {
            let n = 4;
            for i in 0..n {
                let a = i as f32 / n as f32 * std::f32::consts::TAU + unit(seed, 1) * 2.0;
                let rr = if i == 0 { 0.0 } else { w * 0.3 };
                let p = base + vec3(a.cos(), 0.0, a.sin()) * rr;
                let s = 0.7 + 0.5 * unit(seed, i as u64 + 2);
                b.column(p, 0.16 * s, 0.13 * s, h * s * 0.6, 8, CLAY);
                b.column(p + Vec3::Y * (h * s * 0.6), 0.13 * s, 0.09 * s, h * s * 0.25, 8, CLAY);
            }
        }
        HProp::Pan => {
            b.column(base, w * 0.5, w * 0.5, h, 14, BEAM);
            b.column(base + Vec3::Y * (h - 0.03), w * 0.44, w * 0.44, 0.04, 14, SALT);
        }
        HProp::Sweep => {
            // The post, the beam over its top sloping down the way it
            // faces, a rope and basket at the low end.
            b.stick(base, base + Vec3::Y * h, 0.14, BEAM);
            let piv = base + Vec3::Y * (h - 0.1);
            let low = piv + fwd * (w * 0.6) - Vec3::Y * (h * 0.35);
            let high = piv - fwd * (w * 0.4) + Vec3::Y * (h * 0.25);
            b.stick(high, low, 0.1, BEAM);
            b.stick(low, low - Vec3::Y * (h * 0.5), 0.03, ROPE);
            b.column(low - Vec3::Y * (h * 0.5 + 0.4), 0.22, 0.3, 0.4, 8, palette::WEAVE);
        }
        HProp::Sacks => {
            for i in 0..3 {
                let p = base + side * ((i as f32 - 1.0) * 0.45) + fwd * (0.15 * unit(seed, i as u64));
                b.column(p, 0.3, 0.22, 0.55, 8, LINEN);
                b.column(p + Vec3::Y * 0.55, 0.12, 0.1, 0.1, 6, LINEN);
            }
        }
        HProp::Basket => {
            b.column(base, w * 0.4, w * 0.5, h, 9, palette::scale(WEAVE_COL, 0.55));
        }
        HProp::Coils => {
            for i in 0..3 {
                let p = base + side * ((i as f32 - 1.0) * 0.5) + Vec3::Y * 0.08;
                wrap(b, p, 0.28, 0.14, 10);
                wrap(b, p + Vec3::Y * 0.12, 0.26, 0.12, 10);
            }
        }
        HProp::Wheel => {
            let hub = base + Vec3::Y * (h * 0.6);
            let out = fwd;
            let right = side;
            let n = 12;
            for k in 0..n {
                let (a0, a1) = (k as f32 / n as f32 * std::f32::consts::TAU, (k + 1) as f32 / n as f32 * std::f32::consts::TAU);
                let p0 = hub + right * (a0.cos() * w * 0.5) + Vec3::Y * (a0.sin() * w * 0.5);
                let p1 = hub + right * (a1.cos() * w * 0.5) + Vec3::Y * (a1.sin() * w * 0.5);
                b.stick(p0, p1, 0.05, PALE_WOOD);
                if k % 3 == 0 {
                    b.stick(hub, p0, 0.03, PALE_WOOD);
                }
            }
            b.stick(base + out * 0.2, hub, 0.06, BEAM);
            b.stick(base - out * 0.2, hub, 0.06, BEAM);
            b.block(base, 0.8, 0.5, 0.1, rot, BEAM);
        }
        HProp::Loom => {
            // An upright frame: two posts, top and bottom beams, the warp
            // strung between, a weaver's bench before it.
            for sgn in [-1.0f32, 1.0] {
                let p = base + side * (sgn * w * 0.5);
                b.stick(p, p + Vec3::Y * h, 0.09, BEAM);
            }
            b.stick(base - side * (w * 0.5) + Vec3::Y * h, base + side * (w * 0.5) + Vec3::Y * h, 0.1, BEAM);
            b.stick(base - side * (w * 0.5) + Vec3::Y * 0.3, base + side * (w * 0.5) + Vec3::Y * 0.3, 0.1, BEAM);
            let n = (w / 0.09) as usize;
            for i in 0..n {
                let x = -w * 0.45 + w * 0.9 * (i as f32 + 0.5) / n as f32;
                b.stick(base + side * x + Vec3::Y * 0.32, base + side * x + Vec3::Y * (h - 0.05), 0.012, LINEN);
            }
            let cloth = base + Vec3::Y * 0.7 - fwd * 0.02;
            b.patch(cloth, w * 0.9, 0.7, rot, DYES[(seed % 4) as usize]);
            b.block(base - fwd * 0.7, 0.9, 0.3, 0.42, rot, WOOD);
        }
        HProp::Dyepots => {
            for i in 0..3 {
                let p = base + side * ((i as f32 - 1.0) * 0.55);
                b.column(p, 0.24, 0.2, 0.5, 8, CLAY);
                b.column(p + Vec3::Y * 0.46, 0.18, 0.18, 0.05, 8, DYES[(i + seed as usize) % 4]);
            }
        }
        HProp::Yarnline => {
            for sgn in [-1.0f32, 1.0] {
                let p = base + side * (sgn * w * 0.5);
                b.stick(p, p + Vec3::Y * h, 0.07, BEAM);
            }
            b.stick(base - side * (w * 0.5) + Vec3::Y * (h - 0.05), base + side * (w * 0.5) + Vec3::Y * (h - 0.05), 0.025, ROPE);
            let n = (w / 0.35) as usize;
            for i in 0..n {
                let x = -w * 0.45 + w * 0.9 * (i as f32 + 0.5) / n as f32;
                b.patch(base + side * x + Vec3::Y * (h - 0.4), 0.16, 0.6, rot, DYES[(i + seed as usize) % 4]);
            }
        }
        HProp::Treepost => {
            // A trunk with branches spreading to hold a canopy `h` up.
            b.column(base, 0.22, 0.16, h * 0.62, 8, BEAM);
            let fork = base + Vec3::Y * (h * 0.6);
            for i in 0..4 {
                let a = i as f32 / 4.0 * std::f32::consts::TAU + 0.4;
                let tip = fork + vec3(a.cos(), 0.0, a.sin()) * (w * 0.5) + Vec3::Y * (h * 0.38);
                b.stick(fork, tip, 0.1, BEAM);
            }
        }
        HProp::Bench => {
            b.block(base + Vec3::Y * (h - 0.06), w, d, 0.06, rot, PALE_WOOD);
            for sgn in [-1.0f32, 1.0] {
                b.block(base + fwd * (sgn * (w * 0.4)), 0.08, d * 0.8, h - 0.06, rot, BEAM);
            }
        }
        HProp::Boat => {
            // A little boat: a dark hull, pale inside, a pointed bow.
            let c = base;
            b.block(c, w * 0.75, d, h, rot, BEAM);
            b.block(c + Vec3::Y * (h - 0.04), w * 0.7, d * 0.8, 0.04, rot, PALE_WOOD);
            let bow = c + fwd * (w * 0.5) + Vec3::Y * (h * 0.8);
            for sgn in [-1.0f32, 1.0] {
                b.stick(c + fwd * (w * 0.375) + side * (sgn * d * 0.5) + Vec3::Y * h, bow, 0.08, BEAM);
                b.stick(c - fwd * (w * 0.375) + side * (sgn * d * 0.5) + Vec3::Y * h, c - fwd * (w * 0.5) + Vec3::Y * (h * 0.75), 0.08, BEAM);
            }
        }
        HProp::Awning => {
            let corners = [base + fwd * (w * 0.5) + side * (d * 0.5), base + fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) - side * (d * 0.5), base - fwd * (w * 0.5) + side * (d * 0.5)];
            for (i, c) in corners.iter().enumerate() {
                let hh = if i < 2 { h } else { h + 0.25 };
                b.stick(*c, *c + Vec3::Y * hh, 0.06, BEAM);
            }
            let p = [corners[0] + Vec3::Y * h, corners[1] + Vec3::Y * h, corners[2] + Vec3::Y * (h + 0.25), corners[3] + Vec3::Y * (h + 0.25)];
            b.quad(p, Vec3::Y, LINEN);
        }
        HProp::Lamp => {
            b.stick(base + Vec3::Y * h, base + Vec3::Y * (h - 0.3), 0.02, ROPE);
            gl.patch(base + Vec3::Y * (h - 0.42), 0.22, 0.26, rot, palette::WINDOW);
            gl.patch(base + Vec3::Y * (h - 0.42), 0.22, 0.26, rot + std::f32::consts::FRAC_PI_2, palette::WINDOW);
        }
        HProp::Mooring => {
            b.column(base, 0.16, 0.13, h, 7, BEAM);
            wrap(b, base + Vec3::Y * (h - 0.25), 0.17, 0.05, 8);
        }
        HProp::Chimney => {
            b.column(base, 0.14, 0.12, h, 7, CLAY);
            b.smoke(base + Vec3::Y * (h + 0.1), 0.3);
        }
    }
}
