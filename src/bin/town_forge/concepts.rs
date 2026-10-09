//! Step 0: three site concepts, sketched over the real world land at the
//! chosen spot. These are sketches, not the landform: Step 1 builds the
//! chosen one properly.
//!
//! Each concept works in a coast frame: `u` = metres inland from the
//! world's shoreline (negative = out to sea), `v` = metres south of the
//! site's middle line.

use crate::field::Field;
use crate::noise::{fbm, lerp, noise, ridged, smooth};
use crate::render::{Cam, Thing, Way, WayKind};
use gahturiyu_sim::sim::geo::{self, V2};
use gahturiyu_sim::sim::rng::Rng;
use gahturiyu_sim::sim::terrain::Terrain;

/// Where the demo town goes: the north coast, where the northern range
/// meets the sea (around today's Mouawiala).
pub const SITE_Y: f32 = 4050.0;
/// The site's radius (it spans about 800 m) and the blend margin beyond it.
pub const SITE_R: f32 = 400.0;
pub const MARGIN: f32 = 150.0;

pub struct Concept {
    pub key: char,
    pub name: &'static str,
    pub hook: &'static str,
    pub notes: Vec<String>,
    pub centre: V2,
    pub field: Field,
    pub things: Vec<Thing>,
    pub ways: Vec<Way>,
    pub labels: Vec<(V2, String)>,
    pub cam: Cam,
}

pub fn at(u: f32, v: f32) -> V2 {
    let y = SITE_Y + v;
    V2::new(geo::coast_x(y) + u, y)
}

fn uv(p: V2) -> (f32, f32) {
    (geo::inland(p), p.y - SITE_Y)
}

/// The world's own land with some rough detail added (the 30 m grid is
/// smooth), so the mountains read as rock in the sketches.
fn backdrop(t: &Terrain, seed: u64, p: V2) -> (f32, f32) {
    let h = t.base_height(p);
    let detail = (ridged(seed ^ 0xD1, p.x, p.y, 160.0, 4) - 0.45) * 40.0 * smooth(40.0, 260.0, h);
    let h = h + detail;
    let rock = smooth(150.0, 330.0, h) * smooth(0.5, 0.75, ridged(seed ^ 0xD2, p.x, p.y, 90.0, 2));
    (h, rock)
}

/// Rough moorland on the clifftops, with crags pushing up through it.
/// `top` is the clifftop height at the shore (it wanders ±14 m).
fn moor(seed: u64, p: V2, u: f32, v: f32, top: f32, crag_amt: f32) -> (f32, f32) {
    let roll = (fbm(seed ^ 0x11, p.x, p.y, 240.0, 3) - 0.5) * 16.0;
    let wander = (fbm(seed ^ 0x44, p.x, p.y, 380.0, 2) - 0.5) * 28.0;
    let rise = top + wander + u.max(0.0) * 0.05 + smooth(220.0, 520.0, -v) * 70.0;
    // Crags: sheer-sided ribs and blocks of rock, gathered in patches.
    let cn = ridged(seed ^ 0x12, p.x, p.y, 60.0, 3);
    let gather = smooth(0.45, 0.66, fbm(seed ^ 0x13, p.x, p.y, 110.0, 2));
    let tall = 4.0 + 11.0 * fbm(seed ^ 0x14, p.x, p.y, 35.0, 2);
    let rough = (noise(seed ^ 0x15, p.x, p.y, 4.0) - 0.5) * 1.5;
    let crag = (smooth(0.70, 0.75, cn) * tall + rough * smooth(0.66, 0.71, cn)) * gather * crag_amt;
    (rise + roll + crag.max(0.0), smooth(0.6, 2.0, crag))
}

/// The sea floor, deepening offshore the way the world's does (so the
/// sketch meets the world without a seam), a little deeper under cliffs.
fn seabed(seed: u64, p: V2, u: f32) -> f32 {
    -1.0 - (-u).clamp(0.0, 600.0) * 0.03 - 2.5 * (1.0 - smooth(0.0, 150.0, -u)) + (noise(seed ^ 0x21, p.x, p.y, 30.0) - 0.5) * 1.5
}

/// A cliff edge: 0 at the sea side, 1 on top, sheer over a few metres.
fn cliff(e: f32) -> f32 {
    smooth(0.0, 4.0, e)
}

/// How far inland of the cliff edge a point is: the edge wanders in points
/// and coves at two sizes.
fn edge_of(seed: u64, p: V2, u: f32, wobble: f32) -> f32 {
    u + ((fbm(seed ^ 0x45, p.x, p.y, 260.0, 2) - 0.5) * 80.0 + (noise(seed ^ 0x31, p.x, p.y, 40.0) - 0.5) * 22.0) * wobble
}

/// A sea cliff up to the land at `top`: a ledge partway up whose height and
/// width wander, and boulders at the foot for the surf to break on.
fn sea_cliff(seed: u64, p: V2, e: f32, top: f32, sea: f32) -> f32 {
    if e < 0.0 {
        let near = smooth(-14.0, 0.0, e);
        let b = noise(seed ^ 0x43, p.x, p.y, 5.0);
        return sea.max(near * (-1.4 + 2.8 * smooth(0.5, 0.85, b)) - (1.0 - near) * 50.0);
    }
    let n = noise(seed ^ 0x41, p.x, p.y, 70.0);
    let lh = top * (0.2 + 0.35 * n);
    let lw = 1.5 + 7.0 * noise(seed ^ 0x42, p.x, p.y, 45.0);
    lh * smooth(0.0, 2.5, e) + (top - lh) * smooth(lw, lw + 3.0, e)
}

/// Sea stacks: (u, v, radius, height). Returns the stack's height here, if
/// a stack stands here.
fn stacks(seed: u64, p: V2, u: f32, v: f32, list: &[(f32, f32, f32, f32)]) -> Option<f32> {
    let mut best: Option<f32> = None;
    for &(su, sv, r, top) in list {
        let d = ((u - su).powi(2) + (v - sv).powi(2)).sqrt()
            + (noise(seed ^ 0x37, p.x, p.y, 14.0) - 0.5) * 9.0
            + (noise(seed ^ 0x39, p.x, p.y, 5.0) - 0.5) * 3.0;
        if d > r + 14.0 {
            continue;
        }
        let skirt = (1.0 - smooth(r, r + 12.0, d)) * 2.4 - 1.2; // fallen rock at the foot
        let crown = top + (noise(seed ^ 0x38, p.x, p.y, 14.0) - 0.5) * 3.0 - smooth(r * 0.4, r, d) * 2.0;
        // A lower plinth on some sides, cut by the waves.
        let plinth_w = 5.0 * smooth(0.5, 0.7, noise(seed ^ 0x3A, p.x, p.y, 20.0));
        let body = (crown * cliff(r - d)).max(top * 0.3 * cliff(r + plinth_w - d));
        let h = skirt.max(body);
        best = Some(best.map_or(h, |b: f32| b.max(h)));
    }
    best
}

/// Blend a concept's land into the world's around the site.
fn blend(world: (f32, f32), site: (f32, f32), p: V2, centre: V2) -> (f32, f32) {
    let m = 1.0 - smooth(SITE_R, SITE_R + MARGIN, p.dist(centre));
    (lerp(world.0, site.0, m), lerp(world.1, site.1, m))
}

fn build_field(t: &Terrain, seed: u64, centre: V2, site: impl Fn(V2, f32, f32) -> (f32, f32) + Sync) -> Field {
    let (x0, y0, cell) = (centre.x - 1500.0, centre.y - 2600.0, 2.0);
    let (w, h) = (3600, 3300);
    Field::build(x0, y0, cell, w, h, |x, y| {
        let p = V2::new(x, y);
        let world = backdrop(t, seed, p);
        if p.dist(centre) > SITE_R + MARGIN {
            return world;
        }
        let (u, v) = uv(p);
        blend(world, site(p, u, v), p, centre)
    })
}

/// Scatter homes over candidate spots: flat enough, not too close to each other.
fn scatter(f: &Field, rng: &mut Rng, spots: &[V2], spacing: f32, max_slope: f32, r: (f32, f32), taken: &mut Vec<(V2, f32)>, out: &mut Vec<Thing>) {
    for &p in spots {
        let rad = rng.range(r.0, r.1);
        if f.height(p) < 2.0 || f.slope(p.x, p.y) > max_slope {
            continue;
        }
        // Flat under most of the footprint.
        let flat = (0..6).all(|k| {
            let a = k as f32 * 1.047;
            let q = V2::new(p.x + rad * 0.8 * a.cos(), p.y + rad * 0.8 * a.sin());
            (f.height(q) - f.height(p)).abs() < 2.5
        });
        if !flat || taken.iter().any(|&(q, qr)| q.dist(p) < rad + qr + spacing) {
            continue;
        }
        taken.push((p, rad));
        out.push(Thing::Home { at: p, r: rad, eldest: false });
    }
}

fn jitter_grid(rng: &mut Rng, lo: (f32, f32), hi: (f32, f32), step: f32, keep: impl Fn(f32, f32) -> bool) -> Vec<V2> {
    let mut out = Vec::new();
    let mut u = lo.0;
    while u < hi.0 {
        let mut v = lo.1;
        while v < hi.1 {
            let (ju, jv) = (u + rng.range(-0.4, 0.4) * step, v + rng.range(-0.4, 0.4) * step);
            if keep(ju, jv) {
                out.push(at(ju, jv));
            }
            v += step;
        }
        u += step;
    }
    out
}

fn line(pts: &[(f32, f32)]) -> Vec<V2> {
    pts.iter().map(|&(u, v)| at(u, v)).collect()
}

// ---- A: the ledge town ------------------------------------------------------

mod ledge {
    use super::*;
    /// The inlet's middle line and half-width at `u`.
    pub fn mid(u: f32) -> f32 {
        22.0 * (u / 150.0).sin()
    }
    pub fn half(u: f32) -> f32 {
        lerp(36.0, 22.0, (u / 360.0).clamp(0.0, 1.0))
    }
    pub const HEAD: f32 = 360.0;
    /// Signed distance from the inlet's water (negative inside), and whether
    /// the point is on the north side.
    pub fn edge(u: f32, v: f32) -> (f32, bool) {
        let d = if u <= HEAD { (v - mid(u)).abs() - half(u) } else { ((u - HEAD).powi(2) + (v - mid(HEAD)).powi(2)).sqrt() - half(HEAD) };
        (d, v < mid(u.min(HEAD)))
    }
    /// The ledges cut into the north wall: (height, start, width).
    pub const LEDGES: [(f32, f32, f32); 4] = [(3.5, 0.0, 12.0), (17.0, 14.5, 14.0), (31.0, 31.0, 14.0), (45.0, 47.5, 13.0)];
    pub fn north_wall(e: f32) -> f32 {
        let mut h = 3.5 * smooth(-0.5, 0.5, e);
        for &(lh, start, _) in LEDGES.iter().skip(1) {
            h = h.max(lh * smooth(start - 2.5, start, e));
        }
        // The last riser up to the moor.
        h.max(200.0 * smooth(60.5, 63.0, e)) + e.max(0.0) * 0.03
    }
}

pub fn ledge_town(t: &Terrain, seed: u64) -> Concept {
    use ledge::*;
    let centre = at(150.0, -20.0);
    let field_seed = seed ^ 0xA;
    let mut f = build_field(t, field_seed, centre, |p, u, v| {
        // South of the inlet the land drops to a lower rocky headland, so the
        // ledges show over it from the sea.
        let low = 34.0 * smooth(-5.0, 25.0, v - mid(u.min(HEAD))) * (1.0 - smooth(170.0, 290.0, v)) * (1.0 - smooth(330.0, 450.0, u));
        let (moor_h, moor_rock) = moor(field_seed, p, u, v, 60.0 - low, 1.0);
        let e0 = edge_of(field_seed, p, u, 1.0);
        let mut h = sea_cliff(field_seed, p, e0, moor_h, seabed(field_seed, p, u));
        let mut rock = if e0 > 0.0 { moor_rock } else { 0.0 };
        if let Some(sh) = stacks(field_seed, p, u, v, &[(-70.0, -170.0, 14.0, 40.0), (-105.0, -128.0, 8.0, 24.0), (-40.0, 190.0, 9.0, 21.0)]) {
            if sh > h {
                h = sh;
                rock = 0.3;
            }
        }
        if u > -60.0 {
            let (e, north) = edge(u, v);
            if e < 0.0 {
                // In the inlet: water, shallowing to a shingle strip at the head.
                let inlet = lerp(-5.0, 0.8, smooth(HEAD - 60.0, HEAD + 5.0, u)) + (noise(field_seed ^ 0x32, p.x, p.y, 12.0) - 0.5);
                let floor = lerp(seabed(field_seed, p, u), inlet, smooth(-60.0, 0.0, u));
                h = h.min(floor);
                rock = 0.0;
            } else {
                let ledged = smooth(20.0, 80.0, u) * (1.0 - smooth(HEAD - 5.0, HEAD + 25.0, u));
                let wall_n = north_wall(e) + (noise(field_seed ^ 0x33, p.x, p.y, 8.0) - 0.5) * 0.8;
                let sheer = 200.0 * cliff(e) - 3.5;
                let wall = if north { lerp(sheer, wall_n, ledged) } else { sheer };
                if wall < h {
                    h = wall;
                }
            }
        }
        (h, rock)
    });
    crate::render::colour_ground(&mut f, field_seed);

    let mut rng = Rng::from_keys(&[seed, 0xA, 0x484F_4D45]);
    let mut things = Vec::new();
    let mut taken = Vec::new();
    // The Eldest Home: the top ledge, nearest the sea, seen from the water.
    let eld = at(95.0, mid(95.0) - half(95.0) - 54.0);
    things.push(Thing::Home { at: eld, r: 9.5, eldest: true });
    taken.push((eld, 9.5));
    // Homes along ledges 1–3 and the top ledge.
    for &(_, start, w) in LEDGES.iter().skip(1) {
        let mut u = 70.0;
        while u < HEAD - 15.0 {
            let ju = u + rng.range(-3.0, 3.0);
            let e = start + w * 0.5 + rng.range(-1.0, 1.0);
            let spot = at(ju, mid(ju) - half(ju) - e);
            scatter(&f, &mut rng, &[spot], 3.0, 22.0, (4.2, 5.8), &mut taken, &mut things);
            u += rng.range(13.0, 19.0);
        }
    }
    // A few on the clifftop above, among the crags.
    let top = jitter_grid(&mut rng, (60.0, -170.0), (330.0, -60.0), 34.0, |u, v| edge(u, v).0 > 70.0);
    scatter(&f, &mut rng, &top, 9.0, 14.0, (4.5, 6.5), &mut taken, &mut things);

    // Zigzag stairs between ledges, alternating ends; each ledge is a lane.
    let on = |u: f32, e: f32| at(u, mid(u) - half(u) - e);
    let mut ways = Vec::new();
    let ledge_mid = |k: usize| LEDGES[k].1 + LEDGES[k].2 * 0.5;
    let flights = [(0, 345.0, 322.0), (1, 140.0, 163.0), (2, 330.0, 307.0), (3, 120.0, 143.0)];
    for &(k, u0, u1) in &flights {
        let e0 = ledge_mid(k);
        let e1 = if k + 1 < LEDGES.len() { ledge_mid(k + 1) } else { 66.0 };
        ways.push(Way { kind: WayKind::Stairs, pts: vec![on(u0, e0), on(u0, e0 + 3.0), on(u1, e1 - 3.0), on(u1, e1)] });
    }
    for k in 1..LEDGES.len() {
        let pts: Vec<V2> = (0..=14).map(|i| on(80.0 + i as f32 * 19.0, ledge_mid(k))).collect();
        ways.push(Way { kind: WayKind::Lane, pts });
    }
    ways.push(Way { kind: WayKind::Road, pts: line(&[(760.0, 150.0), (560.0, 60.0), (430.0, -80.0), (250.0, -110.0), (143.0, mid(143.0) - half(143.0) - 66.0)]) });
    for w in &ways {
        f.paint_line(&w.pts, w.kind.width(), w.kind.colour(), 0.85);
    }
    // Harbour quay along the bottom ledge, stilts near the mouth, tents on
    // the south clifftop, the Qotiro hall past the head, beacon on the point.
    things.push(Thing::Quay { a: on(210.0, 1.0), b: on(352.0, 1.0), w: 4.0 });
    for k in 0..6 {
        let u = 10.0 + k as f32 * 24.0;
        things.push(Thing::Stilt { at: at(u, mid(u) + half(u) - 9.0 - (k % 2) as f32 * 6.0), r: 4.0 });
    }
    for k in 0..7 {
        let (u, v) = (40.0 + k as f32 * 15.0 + rng.range(-4.0, 4.0), 85.0 + rng.range(-15.0, 25.0));
        things.push(Thing::Tent { at: at(u, v) });
    }
    things.push(Thing::Hall { at: at(440.0, -30.0), size: 16.0, yaw: 0.3 });
    things.push(Thing::Beacon { at: at(18.0, 60.0) });
    let labels = vec![
        (at(-60.0, 0.0), "Inlet".into()),
        (at(280.0, mid(280.0) - half(280.0) + 12.0), "Harbour".into()),
        (eld.add(V2::new(0.0, -16.0)), "Eldest Home".into()),
        (at(230.0, -95.0), "Ledges".into()),
        (at(90.0, 145.0), "Ṭaḍoro tents".into()),
        (at(440.0, -2.0), "Qotiro hall".into()),
        (at(18.0, 82.0), "Beacon".into()),
        (at(60.0, 32.0), "Horaro stilts".into()),
        (at(560.0, 85.0), "Road in".into()),
    ];
    let homes = things.iter().filter(|t| matches!(t, Thing::Home { .. })).count();
    Concept {
        key: 'A',
        name: "Ledge town",
        hook: "Homes stacked on four ledges up the north wall of a narrow sea-cut inlet, joined by zigzag stairs, with the harbour at the cliff foot where the inlet ends.",
        notes: vec![
            format!("Clifftop about 60 m; four ledges at 3.5, 17, 31 and 45 m on the north wall; south of the inlet a lower rocky headland (~25 m) so the ledges show from the sea. {homes} homes sketched."),
            "The town is hidden until you reach the inlet's lip: the first view is straight down the ledges to the water.".into(),
            "Hard to grow: once the ledges are full, new homes spill onto the clifftop, away from the drama.".into(),
        ],
        centre,
        field: f,
        things,
        ways,
        labels,
        cam: Cam { pos: at(-430.0, 250.0), z: 14.0, look: at(170.0, -50.0), pitch_deg: 5.5, fov_deg: 60.0, w: 1600, h: 760 },
    }
}

// ---- B: the glen-mouth town -------------------------------------------------

mod glen {
    use super::*;
    /// The glen's middle line, from the beach up toward the mountains.
    pub const LINE: [(f32, f32); 7] = [(-40.0, 10.0), (0.0, 0.0), (150.0, -20.0), (300.0, -70.0), (450.0, -160.0), (600.0, -280.0), (760.0, -420.0)];
    /// Distance from the middle line (signed: + north) and how far along it.
    pub fn frame(u: f32, v: f32) -> (f32, f32) {
        let p = V2::new(u, v);
        let (mut best, mut s_at, mut side, mut s) = (f32::MAX, 0.0, 1.0, -40.0f32);
        for w in LINE.windows(2) {
            let (a, b) = (V2::new(w[0].0, w[0].1), V2::new(w[1].0, w[1].1));
            let ab = b.sub(a);
            let len = ab.len();
            let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / (len * len)).clamp(0.0, 1.0);
            let q = a.add(ab.scale(t));
            let d = p.dist(q);
            if d < best {
                best = d;
                s_at = s + t * len;
                // North is the left of the line walking inland (y grows south).
                side = if ab.x * (p.y - a.y) - ab.y * (p.x - a.x) < 0.0 { 1.0 } else { -1.0 };
            }
            s += len;
        }
        (best * side, s_at)
    }
    /// The shoulders either side of the gorge (the glen floor), by distance along.
    pub fn floor(s: f32) -> f32 {
        1.5 + smooth(60.0, 150.0, s) * 38.0 + s.max(0.0) * 0.05
    }
    /// The stream bed in the gorge.
    pub fn bed(s: f32) -> f32 {
        let low = 0.5 + s.max(0.0) * 0.07;
        lerp(low, floor(s) - 1.5, smooth(300.0, 380.0, s))
    }
    pub fn width(s: f32) -> f32 {
        lerp(55.0, 30.0, smooth(40.0, 140.0, s))
    }
}

pub fn glen_town(t: &Terrain, seed: u64) -> Concept {
    use glen::*;
    let centre = at(170.0, -40.0);
    let field_seed = seed ^ 0xB;
    let mut f = build_field(t, field_seed, centre, |p, u, v| {
        let (moor_h, moor_rock) = moor(field_seed, p, u, v, 52.0, 1.0);
        let e0 = edge_of(field_seed, p, u, 1.0);
        let mut h = sea_cliff(field_seed, p, e0, moor_h, seabed(field_seed, p, u));
        let mut rock = if e0 > 0.0 { moor_rock } else { 0.0 };
        if let Some(sh) = stacks(field_seed, p, u, v, &[(-80.0, 150.0, 12.0, 32.0), (-58.0, 196.0, 7.0, 17.0), (-60.0, -210.0, 10.0, 28.0)]) {
            if sh > h {
                h = sh;
                rock = 0.3;
            }
        }
        let (d, s) = frame(u, v);
        let ad = d.abs();
        if s > -30.0 && ad < 260.0 {
            // Walls rise from the floor in benches about 9 m high.
            // Benches of uneven height and width, wavy along the wall.
            let over = (ad - width(s) + (noise(field_seed ^ 0x3B, p.x, p.y, 30.0) - 0.5) * 10.0).max(0.0) * 0.55;
            let lift = 7.0 + 5.0 * noise(field_seed ^ 0x3C, p.x, p.y, 90.0);
            let step = over / lift;
            let flat = 0.4 + 0.25 * noise(field_seed ^ 0x3D, p.x, p.y, 60.0);
            let benched = (step.floor() + smooth(flat, 1.0, step.fract())) * lift;
            let wall = floor(s) + benched + (noise(field_seed ^ 0x34, p.x, p.y, 20.0) - 0.5) * 2.0;
            // Out at the mouth the floor is a beach running into the sea.
            let beach = lerp(seabed(field_seed, p, u + 30.0), floor(s), smooth(-25.0, 15.0, s));
            let glen_h = if ad < width(s) { beach.min(floor(s)) } else { wall };
            if glen_h < h {
                h = glen_h;
                rock *= smooth(0.0, 30.0, over);
            }
            // The gorge: a slot cut by the stream, sheer-sided.
            let slot = lerp(bed(s), 400.0, smooth(5.0, 8.5, ad));
            if s > 40.0 && slot < h {
                h = slot;
                rock = 1.0 * smooth(4.0, 6.0, ad);
            }
        }
        (h, rock)
    });
    crate::render::colour_ground(&mut f, field_seed);
    let on = |s: f32, d: f32| -> V2 {
        // Walk the line to `s`, then step `d` to the north side.
        let mut acc = -40.0;
        for w in LINE.windows(2) {
            let (a, b) = (V2::new(w[0].0, w[0].1), V2::new(w[1].0, w[1].1));
            let len = a.dist(b);
            if s <= acc + len || w[1] == LINE[LINE.len() - 1] {
                let t = ((s - acc) / len).clamp(0.0, 1.0);
                let q = a.lerp(b, t);
                let dir = b.sub(a).scale(1.0 / len);
                let north = V2::new(dir.y, -dir.x);
                let r = q.add(north.scale(d));
                return at(r.x, r.y);
            }
            acc += len;
        }
        at(0.0, 0.0)
    };

    let mut rng = Rng::from_keys(&[seed, 0xB, 0x484F_4D45]);
    let mut things = Vec::new();
    let mut taken = Vec::new();
    let eld = on(115.0, 62.0);
    things.push(Thing::Home { at: eld, r: 10.0, eldest: true });
    taken.push((eld, 10.0));
    let mut spots = Vec::new();
    let mut s = 70.0;
    while s < 540.0 {
        for side in [-1.0f32, 1.0] {
            let mut d = 14.0;
            while d < 120.0 {
                spots.push(on(s + rng.range(-5.0, 5.0), side * (d + rng.range(-3.0, 3.0))));
                d += 12.0;
            }
        }
        s += 14.0;
    }
    // Shuffle so the scatter doesn't fill one side first.
    for i in (1..spots.len()).rev() {
        let j = rng.below(i + 1);
        spots.swap(i, j);
    }
    let keep: Vec<V2> = spots.into_iter().take(160).collect();
    scatter(&f, &mut rng, &keep, 6.0, 16.0, (4.0, 6.0), &mut taken, &mut things);
    things.retain(|t| !matches!(t, Thing::Home { at, eldest: false, .. } if frame(uv(*at).0, uv(*at).1).0.abs() < 10.0));

    let bridge_s = 225.0;
    let (bn, bs) = (on(bridge_s, 13.0), on(bridge_s, -13.0));
    things.push(Thing::Bridge { a: bn, za: f.height(bn) + 0.3, b: bs, zb: f.height(bs) + 0.3, sag: 0.0 });
    let mut ways = vec![
        Way { kind: WayKind::Stream, pts: (0..=40).map(|k| on(760.0 - k as f32 * 20.0, 0.0)).collect() },
        Way { kind: WayKind::Stairs, pts: vec![on(10.0, 22.0), on(40.0, 30.0), on(60.0, 16.0), on(95.0, 26.0), on(120.0, 14.0), on(150.0, 20.0)] },
        Way { kind: WayKind::Stairs, pts: vec![on(20.0, -24.0), on(55.0, -32.0), on(80.0, -18.0), on(120.0, -26.0), on(150.0, -18.0)] },
        Way { kind: WayKind::Lane, pts: vec![on(150.0, 20.0), on(bridge_s, 14.0), on(330.0, 18.0), on(430.0, 16.0)] },
        Way { kind: WayKind::Lane, pts: vec![on(150.0, -18.0), on(bridge_s, -14.0), on(330.0, -20.0)] },
        Way { kind: WayKind::Road, pts: vec![on(760.0, 30.0), on(600.0, 24.0), on(430.0, 16.0)] },
    ];
    for w in &ways {
        f.paint_line(&w.pts, w.kind.width(), w.kind.colour(), 0.85);
    }
    // The stream down the gorge stays painted; the bridge isn't a way.
    ways.retain(|_| true);
    for k in 0..6 {
        let (u, v) = (-35.0 - k as f32 * 13.0, -35.0 + (k as f32 * 17.0) % 60.0);
        things.push(Thing::Stilt { at: at(u, v), r: 4.0 });
    }
    things.push(Thing::Quay { a: at(-5.0, -24.0), b: at(-45.0, -30.0), w: 4.0 });
    for k in 0..7 {
        let (u, v) = (60.0 + k as f32 * 16.0 + rng.range(-4.0, 4.0), 150.0 + rng.range(-15.0, 30.0));
        things.push(Thing::Tent { at: at(u, v) });
    }
    things.push(Thing::Hall { at: on(470.0, 0.0), size: 15.0, yaw: -0.5 });
    things.push(Thing::Beacon { at: at(15.0, -100.0) });
    let labels = vec![
        (at(-20.0, 30.0), "Beach".into()),
        (on(bridge_s, 0.0).add(V2::new(30.0, 26.0)), "High bridge".into()),
        (eld.add(V2::new(0.0, -17.0)), "Eldest Home".into()),
        (on(330.0, 0.0).add(V2::new(10.0, 28.0)), "Gorge".into()),
        (at(110.0, 215.0), "Ṭaḍoro tents".into()),
        (on(470.0, 0.0).add(V2::new(0.0, 24.0)), "Qotiro hall".into()),
        (at(15.0, -122.0), "Beacon".into()),
        (at(-75.0, -45.0), "Horaro stilts".into()),
        (on(640.0, 30.0).add(V2::new(0.0, -14.0)), "Road in".into()),
    ];
    let homes = things.iter().filter(|t| matches!(t, Thing::Home { .. })).count();
    Concept {
        key: 'B',
        name: "Glen-mouth town",
        hook: "A steep glen breaks through the cliffs to a small beach; the town climbs both benched walls, split by a stream gorge and joined by one high bridge.",
        notes: vec![
            format!("Clifftops about 52 m; glen floor rises from the beach to ~45 m; gorge ~35 m deep; walls benched every 9 m. {homes} homes sketched."),
            "The road comes down the glen from the mountains: the first view opens over the bridge to the sea.".into(),
            "Grows naturally: more benches up the walls and further up the glen.".into(),
        ],
        centre,
        field: f,
        things,
        ways,
        labels,
        cam: Cam { pos: at(-430.0, 170.0), z: 14.0, look: at(190.0, -70.0), pitch_deg: 5.5, fov_deg: 60.0, w: 1600, h: 760 },
    }
}

// ---- C: the stack town ------------------------------------------------------

mod stack {
    /// The sea stacks off the headland: (u, v, radius, height).
    pub const STACKS: [(f32, f32, f32, f32); 6] =
        [(-300.0, -8.0, 24.0, 58.0), (-348.0, 48.0, 17.0, 50.0), (-352.0, -62.0, 19.0, 54.0), (-398.0, 4.0, 15.0, 44.0), (-412.0, 74.0, 10.0, 34.0), (-282.0, 78.0, 13.0, 46.0)];
    pub const TIP: f32 = -255.0;
    /// The headland's half-width at `u`.
    pub fn half(u: f32) -> f32 {
        if u > 0.0 {
            42.0 + u * 0.4
        } else {
            42.0 + 32.0 * super::smooth(0.0, -70.0, u) - 20.0 * super::smooth(-180.0, -255.0, u)
        }
    }
}

pub fn stack_town(t: &Terrain, seed: u64) -> Concept {
    use stack::*;
    let centre = at(-110.0, 20.0);
    let field_seed = seed ^ 0xC;
    let mut f = build_field(t, field_seed, centre, |p, u, v| {
        let (moor_h, moor_rock) = moor(field_seed, p, u, v, 44.0, 0.9);
        // A cove bites into the mainland south of the neck.
        let bite = 95.0 * smooth(40.0, 90.0, v) * (1.0 - smooth(200.0, 260.0, v));
        let e0 = edge_of(field_seed, p, u - bite, 1.0 - 0.85 * bite / 95.0);
        let mut h = sea_cliff(field_seed, p, e0, moor_h, seabed(field_seed, p, u));
        let mut rock = if e0 > 0.0 { moor_rock } else { 0.0 };
        // The headland.
        if u < 40.0 && u > TIP - 10.0 {
            let e = half(u) - v.abs() + (noise(field_seed ^ 0x35, p.x, p.y, 30.0) - 0.5) * 22.0;
            let tipcut = TIP - u + (noise(field_seed ^ 0x36, p.x, p.y, 25.0) - 0.5) * 16.0;
            let on = cliff(e.min(-tipcut));
            let top = moor_h + 3.0 * smooth(0.0, -150.0, u);
            let hh = sea_cliff(field_seed, p, e.min(-tipcut), top, seabed(field_seed, p, u));
            if hh > h {
                h = hh;
                rock = moor_rock * on;
            }
        }
        // The stacks.
        if let Some(sh) = stacks(field_seed, p, u, v, &STACKS) {
            if sh > h {
                h = sh;
                rock = 0.3;
            }
        }
        // A shingle landing at the cove's head.
        if v > 110.0 && v < 190.0 && e0 > -18.0 && e0 < 0.5 {
            let w = smooth(-18.0, -8.0, e0) * smooth(110.0, 125.0, v) * smooth(190.0, 175.0, v);
            h = lerp(h, 0.9, w);
            rock *= 1.0 - w;
        }
        (h, rock)
    });
    crate::render::colour_ground(&mut f, field_seed);

    let mut rng = Rng::from_keys(&[seed, 0xC, 0x484F_4D45]);
    let mut things = Vec::new();
    let mut taken = Vec::new();
    let s0 = STACKS[0];
    let eld = at(s0.0 + 2.0, s0.1 + 2.0);
    things.push(Thing::Home { at: eld, r: 12.0, eldest: true });
    taken.push((eld, 12.0));
    // The old homes: one or two on each stack.
    for &(su, sv, r, _) in STACKS.iter().skip(1) {
        let n = if r > 18.0 { 2 } else { 1 };
        for k in 0..n {
            let a = rng.range(0.0, 6.28);
            let off = if n == 1 { 0.0 } else { r * 0.42 };
            let p = at(su + off * a.cos() + k as f32 * 0.0, sv + off * a.sin() * if k == 0 { 1.0 } else { -1.0 });
            let rad = (r * 0.45).min(8.0);
            if taken.iter().all(|&(q, qr)| q.dist(p) > qr + rad + 1.0) {
                taken.push((p, rad));
                things.push(Thing::Home { at: p, r: rad, eldest: false });
            }
        }
    }
    // Newer homes along the headland and onto the mainland at its neck.
    let spots = jitter_grid(&mut rng, (-240.0, -80.0), (220.0, 110.0), 21.0, |u, v| {
        if u < 40.0 {
            v.abs() < half(u) - 10.0
        } else {
            v > -140.0 && v < 70.0
        }
    });
    scatter(&f, &mut rng, &spots, 6.0, 15.0, (4.0, 6.0), &mut taken, &mut things);

    let edge_pt = |a: (f32, f32), b: (f32, f32), ra: f32| -> (f32, f32) {
        let (du, dv) = (b.0 - a.0, b.1 - a.1);
        let l = (du * du + dv * dv).sqrt();
        (a.0 + du / l * ra, a.1 + dv / l * ra)
    };
    let bridge = |a: (f32, f32, f32), b: (f32, f32, f32), sag: f32, things: &mut Vec<Thing>| {
        let pa = edge_pt((a.0, a.1), (b.0, b.1), a.2 - 3.0);
        let pb = edge_pt((b.0, b.1), (a.0, a.1), b.2 - 3.0);
        let (pa, pb) = (at(pa.0, pa.1), at(pb.0, pb.1));
        things.push(Thing::Bridge { a: pa, za: f.height(pa) + 0.4, b: pb, zb: f.height(pb) + 0.4, sag });
    };
    let st = |k: usize| (STACKS[k].0, STACKS[k].1, STACKS[k].2);
    bridge((TIP + 15.0, -4.0, 12.0), st(0), 0.4, &mut things);
    bridge(st(0), st(1), 2.5, &mut things);
    bridge(st(0), st(2), 2.5, &mut things);
    bridge(st(1), st(3), 2.0, &mut things);
    bridge(st(1), st(5), 1.5, &mut things);
    bridge(st(1), st(4), 1.5, &mut things);
    let mut ways = vec![
        Way { kind: WayKind::Lane, pts: line(&[(TIP + 15.0, -4.0), (-180.0, 4.0), (-100.0, -6.0), (-20.0, 2.0), (60.0, -10.0), (160.0, -20.0)]) },
        Way { kind: WayKind::Stairs, pts: line(&[(-40.0, 30.0), (-55.0, 52.0), (-30.0, 58.0), (-45.0, 70.0), (-25.0, 78.0)]) },
        Way { kind: WayKind::Stairs, pts: line(&[(110.0, 60.0), (100.0, 85.0), (120.0, 95.0), (100.0, 108.0), (90.0, 135.0)]) },
        Way { kind: WayKind::Road, pts: line(&[(160.0, -20.0), (300.0, -10.0), (480.0, 60.0), (700.0, 140.0)]) },
    ];
    for w in &ways {
        f.paint_line(&w.pts, w.kind.width(), w.kind.colour(), 0.85);
    }
    ways.retain(|_| true);
    for k in 0..7 {
        let (u, v) = (-50.0 + k as f32 * 18.0, 120.0 + (k % 3) as f32 * 22.0);
        things.push(Thing::Stilt { at: at(u, v), r: 4.2 });
    }
    things.push(Thing::Quay { a: at(-25.0, 82.0), b: at(-5.0, 110.0), w: 4.0 });
    for k in 0..7 {
        let (u, v) = (70.0 + k as f32 * 14.0 + rng.range(-4.0, 4.0), -190.0 + rng.range(-20.0, 20.0));
        things.push(Thing::Tent { at: at(u, v) });
    }
    things.push(Thing::Hall { at: at(230.0, 30.0), size: 16.0, yaw: 0.2 });
    let s3 = STACKS[3];
    things.push(Thing::Beacon { at: at(s3.0, s3.1) });
    let labels = vec![
        (eld.add(V2::new(0.0, -20.0)), "Eldest Home".into()),
        (at(5.0, 215.0), "Horaro stilts".into()),
        (at(-120.0, -75.0), "Headland".into()),
        (at(80.0, 160.0), "Landing".into()),
        (at(110.0, -225.0), "Ṭaḍoro tents".into()),
        (at(230.0, 58.0), "Qotiro hall".into()),
        (at(s3.0, s3.1 - 26.0), "Beacon".into()),
        (at(560.0, 70.0), "Road in".into()),
        (at(-330.0, 120.0), "Stacks".into()),
    ];
    let homes = things.iter().filter(|t| matches!(t, Thing::Home { .. })).count();
    Concept {
        key: 'C',
        name: "Stack town",
        hook: "A headland breaking into sea stacks: the oldest homes stand on the stacks, joined by rope and stone bridges, with Horaro stilts in the sheltered cove beside the headland.",
        notes: vec![
            format!("Headland top about 45 m; six stacks 34–58 m tall (the tallest above the headland), 15–25 m channels between; a cove with a shingle landing. {homes} homes sketched."),
            "The Eldest Home on the tallest stack is the landmark from every side; the bridges are the drama.".into(),
            "Grows back along the headland and onto the mainland; the stacks stay the old town.".into(),
        ],
        centre,
        field: f,
        things,
        ways,
        labels,
        cam: Cam { pos: at(-640.0, 270.0), z: 12.0, look: at(-180.0, -20.0), pitch_deg: 5.0, fov_deg: 60.0, w: 1600, h: 760 },
    }
}
