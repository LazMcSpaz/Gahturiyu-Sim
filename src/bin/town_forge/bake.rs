//! Turning the weathered land into what the game loads: the land editor's
//! layers (height, paint, rocks) over the world's own land, faded to
//! nothing at the rim so there is no seam.

use crate::erode::{Kind, Land};
use crate::noise::{noise, smooth};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::mapedit::{MapEdits, Rock, EDIT_CELL};
use gahturiyu_sim::sim::rng::Rng;
use gahturiyu_sim::sim::terrain::Terrain;

/// Metres at the grid's rim over which the authored land fades to the world's.
pub const RIM: f32 = 120.0;

/// Texture indices in `mapedit::TEXTURES`.
const TEX_HEATH: u8 = 2;
const TEX_SHINGLE: u8 = 7;
const TEX_GRAVEL: u8 = 8;
const TEX_ROCK: u8 = 9;
const TEX_MOSS: u8 = 12;
/// Rock kinds in `mapedit::ROCKS`.
const ROCK_BOULDER: u8 = 0;
const ROCK_SLAB: u8 = 1;
const ROCK_SCREE: u8 = 3;
const ROCK_OUTCROP: u8 = 4;

fn paint(tex: u8, weight: u8) -> u8 {
    ((tex + 1) << 4) | weight.min(15)
}

/// How much of the authored land applies here (1 inside, 0 at the rim).
pub fn rim_fade(land: &Land, p: V2) -> f32 {
    let dx = (p.x - land.x0).min(land.x0 + (land.w - 1) as f32 * land.cell - p.x);
    let dy = (p.y - land.y0).min(land.y0 + (land.h - 1) as f32 * land.cell - p.y);
    smooth(0.0, RIM, dx.min(dy))
}

pub fn bake(land: &Land, terrain: &Terrain, seed: u64) -> MapEdits {
    let mut e = MapEdits::default();
    let i0 = (land.x0 / EDIT_CELL).ceil() as i64;
    let i1 = ((land.x0 + (land.w - 1) as f32 * land.cell) / EDIT_CELL).floor() as i64;
    let j0 = (land.y0 / EDIT_CELL).ceil() as i64;
    let j1 = ((land.y0 + (land.h - 1) as f32 * land.cell) / EDIT_CELL).floor() as i64;
    let mut rng = Rng::from_keys(&[seed, 0x4241_4B45]);
    for j in j0..=j1 {
        for i in i0..=i1 {
            let p = V2::new(i as f32 * EDIT_CELL, j as f32 * EDIT_CELL);
            let fade = rim_fade(land, p);
            if fade <= 0.0 {
                continue;
            }
            let dh = (land.height(p) - terrain.base_height(p)) * fade;
            if dh.abs() > 0.02 {
                e.height.set(i, j, dh);
            }
            if fade < 0.5 {
                continue;
            }
            let k = land.cell_of(p);
            let slope = land.slope_deg(k);
            let z = land.z[k];
            // Ground: what the weathering left on the surface.
            let tex = match land.kind(k) {
                Kind::Rock => Some(paint(TEX_ROCK, 15)),
                Kind::Wet => Some(paint(TEX_ROCK, 12)),
                Kind::Scree => Some(paint(TEX_GRAVEL, 13)),
                Kind::Shingle => Some(paint(TEX_SHINGLE, 14)),
                Kind::Turf => {
                    let n = noise(seed ^ 0x7E, p.x, p.y, 60.0);
                    if z > 30.0 && n > 0.62 {
                        Some(paint(TEX_HEATH, 9))
                    } else if slope > 18.0 && n < 0.35 {
                        Some(paint(TEX_MOSS, 8))
                    } else {
                        None
                    }
                }
            };
            if let Some(t) = tex {
                e.paint.set(i, j, t);
            }
            // Rocks: boulders and scree heaps where rock has fallen, outcrops
            // on crag tops, slabs on bare rock.
            let kind = land.kind(k);
            let roll = rng.f32();
            let rock = match kind {
                Kind::Scree if roll < 0.18 => Some((if roll < 0.06 { ROCK_BOULDER } else { ROCK_SCREE }, rng.range(1.2, 3.0))),
                Kind::Rock if slope < 25.0 && roll < 0.12 => Some((ROCK_OUTCROP, rng.range(3.0, 7.0))),
                Kind::Rock if slope >= 25.0 && roll < 0.05 => Some((ROCK_SLAB, rng.range(2.0, 4.0))),
                Kind::Turf if z > 8.0 && roll < 0.012 => Some((ROCK_BOULDER, rng.range(1.0, 2.2))),
                _ => None,
            };
            if let Some((kind, size)) = rock {
                let pos = V2::new(p.x + rng.range(-2.0, 2.0), p.y + rng.range(-2.0, 2.0));
                e.rocks.push(Rock { pos, kind, size, rot: rng.range(0.0, 6.28), seed: rng.next_u64() as u32 });
            }
        }
    }
    e.changed(true, true);
    e
}
