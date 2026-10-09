//! A site's land: the block of rock it started as, the rock recipe, and the
//! weathering run on it. Each concept is a different recipe and starting
//! block; the shapes come out of the weathering.

use crate::concepts::{at, MARGIN, SITE_R};
use crate::erode::{Inflow, Kind, Land, Params};
use crate::field::Field;
use crate::geology::{Fault, HardBody, Rock};
use crate::noise::{fbm, lerp, ridged, smooth};
use gahturiyu_sim::sim::geo::{self, V2};
use gahturiyu_sim::sim::terrain::Terrain;

/// The land grid's cell size and extent round the site.
pub const CELL: f32 = 2.0;
/// How far the uneroded block of rock reached out to sea (the sea has eaten
/// it back since).
pub const BLOCK_OUT: f32 = 350.0;

pub struct Site {
    pub rock: Rock,
    pub params: Params,
    /// Height of the block's top at the shore, before weathering.
    pub block_top: f32,
}

/// The world's land outside the site, roughened so the mountains read as rock.
pub fn backdrop(t: &Terrain, seed: u64, p: V2) -> (f32, f32) {
    let h = t.base_height(p);
    let detail = (ridged(seed ^ 0xD1, p.x, p.y, 160.0, 4) - 0.45) * 40.0 * smooth(40.0, 260.0, h);
    let h = h + detail;
    let rock = smooth(150.0, 330.0, h) * smooth(0.5, 0.75, ridged(seed ^ 0xD2, p.x, p.y, 90.0, 2));
    (h, rock)
}

/// The block before weathering: the world's land, with a raised shelf of
/// rock pushed out past today's coast inside the site.
fn starting_block(t: &Terrain, seed: u64, site: &Site, centre: V2, p: V2) -> (f32, f32) {
    let world = backdrop(t, seed, p).0;
    let seed = site.rock.seed;
    let u = geo::inland(p);
    // The raised shelf runs along the coast and fades out along it; inland
    // the world's own rise takes over.
    let m = 1.0 - smooth(SITE_R, SITE_R + MARGIN * 2.0, (p.y - centre.y).abs());
    // Rolling moor with a few hard knolls, rising toward the mountains and
    // running on down under the sea: a drowned landscape, whose hills meet
    // the sea as headlands and whose hollows meet it as bays. The sea then
    // cuts cliffs into what it reaches.
    let roll = (fbm(seed ^ 0xB1, p.x, p.y, 520.0, 4) - 0.5) * 120.0;
    let knoll = smooth(0.55, 0.85, ridged(seed ^ 0xB2, p.x, p.y, 160.0, 3)).powf(1.5) * 22.0;
    let top = site.block_top + roll + knoll + u * if u > 0.0 { 0.07 } else { 0.16 };
    let seabed = -1.0 - (-u).clamp(0.0, 600.0) * 0.03;
    // Out past the block the world's own sea floor takes over.
    let out = smooth(-BLOCK_OUT - 120.0, -BLOCK_OUT, u);
    let shelf = lerp(seabed, top, out).max(seabed);
    let raised = if world > shelf { world } else { shelf };
    (lerp(world, raised, m), m)
}

pub fn make(t: &Terrain, seed: u64, site: &Site, centre: V2) -> Land {
    let (x0, y0) = (centre.x - 700.0, centre.y - 700.0);
    let n = (1400.0 / CELL) as usize;
    let mut land = Land::new(x0, y0, CELL, n, n, &site.rock, |p| starting_block(t, seed, site, centre, p));
    land.weather(&site.params);
    land
}

/// The picture-ready field: the weathered land inside, the world outside.
pub fn field(t: &Terrain, seed: u64, land: &Land, centre: V2) -> Field {
    let (x0, y0, cell) = (centre.x - 1500.0, centre.y - 2600.0, 2.0);
    let (w, h) = (3600, 3300);
    let mut f = Field::build(x0, y0, cell, w, h, |x, y| {
        let p = V2::new(x, y);
        if land.inside(p) {
            (land.height(p), 0.0)
        } else {
            backdrop(t, seed, p)
        }
    });
    // Surface kinds from the land, on the field's matching cells.
    for k in 0..f.z.len() {
        let (x, y) = f.cell_pos(k);
        let p = V2::new(x, y);
        if land.inside(p) {
            let lk = land_cell(land, p);
            let kind = land.kind(lk);
            f.kind[k] = match kind {
                Kind::Turf => 0,
                Kind::Rock => 1,
                Kind::Scree => 2,
                Kind::Shingle => 3,
                Kind::Wet => 4,
            };
            if kind == Kind::Turf && land.stream(lk) > 0.5 && land.z[lk] > 0.0 {
                f.kind[k] = 5;
            }
            f.rock[k] = if kind == Kind::Rock { 1.0 } else { 0.0 };
        } else {
            f.rock[k] = backdrop(t, seed, p).1;
        }
    }
    f
}

/// Bare rock faces coloured by their layers: pale where hard, dark where soft.
pub fn face_colour(land: &Land) -> impl Fn(V2, f32) -> Option<[f32; 3]> + Sync + '_ {
    move |p: V2, z: f32| {
        if !land.inside(p) {
            return None;
        }
        let hard = land.hardness_at(p, z);
        let fine = crate::noise::noise(0x51, z * 3.0, (p.x - p.y) * 0.4, 1.0);
        let soft = [0.36, 0.30, 0.25];
        let hardc = [0.60, 0.58, 0.54];
        let t = (hard * 1.1 - 0.05 + (fine - 0.5) * 0.2).clamp(0.0, 1.0);
        Some([soft[0] + (hardc[0] - soft[0]) * t, soft[1] + (hardc[1] - soft[1]) * t, soft[2] + (hardc[2] - soft[2]) * t])
    }
}

fn land_cell(land: &Land, p: V2) -> usize {
    let i = (((p.x - land.x0) / land.cell).round().max(0.0) as usize).min(land.w - 1);
    let j = (((p.y - land.y0) / land.cell).round().max(0.0) as usize).min(land.h - 1);
    j * land.w + i
}

// ---- The three recipes --------------------------------------------------------

/// B: a stream from the mountains has cut a glen through flat-lying layers.
pub fn glen_site(seed: u64) -> Site {
    Site {
        rock: Rock {
            seed: seed ^ 0xB,
            dip_deg: 3.0,
            dip_toward_deg: 200.0,
            layer_m: 7.0,
            hard_share: 0.3,
            joint_dirs_deg: [25.0, 110.0],
            joint_spacing: 42.0,
            joint_weak: 0.5,
            faults: vec![],
            bodies: vec![],
        },
        params: Params {
            epochs: 260,
            stream_cut: 0.03,
            waves: 2.2,
            rain: 1400,
            inflows: vec![Inflow { at: at(620.0, -140.0), flow: 120 }],
            swell_from_deg: 170.0,
        },
        block_top: 58.0,
    }
}

/// A: a long fault zone runs inland from the sea; the sea has cut a narrow
/// inlet along it. The layers dip north, so the north wall benches.
pub fn ledge_site(seed: u64) -> Site {
    Site {
        rock: Rock {
            seed: seed ^ 0xA,
            dip_deg: 6.0,
            dip_toward_deg: 90.0,
            layer_m: 9.0,
            hard_share: 0.32,
            joint_dirs_deg: [15.0, 95.0],
            joint_spacing: 48.0,
            joint_weak: 0.45,
            faults: vec![Fault { through: at(100.0, 0.0), angle_deg: 8.0, width: 22.0, weak: 0.92 }],
            bodies: vec![],
        },
        params: Params {
            epochs: 260,
            stream_cut: 0.03,
            waves: 2.2,
            rain: 1200,
            inflows: vec![Inflow { at: at(640.0, 200.0), flow: 14 }],
            swell_from_deg: 170.0,
        },
        block_top: 62.0,
    }
}

/// C: a hard dyke runs out to sea along the headland; cracks across it let
/// the sea cut it into stacks.
pub fn stack_site(seed: u64) -> Site {
    Site {
        rock: Rock {
            seed: seed ^ 0xC,
            dip_deg: 5.0,
            dip_toward_deg: 160.0,
            layer_m: 8.0,
            hard_share: 0.28,
            joint_dirs_deg: [80.0, 165.0],
            joint_spacing: 44.0,
            joint_weak: 0.55,
            faults: vec![
                Fault { through: at(60.0, 120.0), angle_deg: 150.0, width: 40.0, weak: 0.6 },
                // Cracks across the dyke, for the sea to cut it into stacks.
                Fault { through: at(-190.0, 0.0), angle_deg: 78.0, width: 9.0, weak: 0.9 },
                Fault { through: at(-140.0, 0.0), angle_deg: 84.0, width: 7.0, weak: 0.9 },
                Fault { through: at(-95.0, 0.0), angle_deg: 80.0, width: 8.0, weak: 0.9 },
            ],
            bodies: vec![HardBody { through: at(-120.0, 0.0), angle_deg: 172.0, length: 520.0, width: 70.0, harder: 0.9 }],
        },
        params: Params {
            epochs: 260,
            stream_cut: 0.03,
            waves: 2.2,
            rain: 1200,
            inflows: vec![Inflow { at: at(640.0, 260.0), flow: 14 }],
            swell_from_deg: 170.0,
        },
        block_top: 54.0,
    }
}
