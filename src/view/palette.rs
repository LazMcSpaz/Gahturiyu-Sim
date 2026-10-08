//! Ground colours, shared by the 3D view and the map so the two agree.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    geo::{self, V2},
    terrain::{self, Terrain},
};

const GRASS: Color = Color::new(0.30, 0.40, 0.24, 1.0);
const GRASS_DRY: Color = Color::new(0.43, 0.45, 0.29, 1.0);
const HEATH: Color = Color::new(0.38, 0.36, 0.27, 1.0);
const SHINGLE: Color = Color::new(0.47, 0.47, 0.44, 1.0);
const ROCK: Color = Color::new(0.40, 0.39, 0.38, 1.0);
const SNOW: Color = Color::new(0.88, 0.90, 0.92, 1.0);
const SCRUB: Color = Color::new(0.64, 0.54, 0.36, 1.0);
const SANDROCK: Color = Color::new(0.74, 0.56, 0.36, 1.0);
pub const SEA: Color = Color::new(0.22, 0.32, 0.34, 1.0);
pub const SEA_DEEP: Color = Color::new(0.14, 0.22, 0.27, 1.0);
pub const ROAD: Color = Color::new(0.45, 0.36, 0.25, 1.0);

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1.0)
}

pub fn scale(c: Color, k: f32) -> Color {
    Color::new(c.r * k, c.g * k, c.b * k, c.a)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Unlit colour of the sea surface at a point.
pub fn sea(p: V2) -> Color {
    mix(SEA, SEA_DEEP, -geo::inland(p) / 700.0)
}

/// Unlit colour of the ground at a point, given its height and how upright
/// its surface is (`up` = 1 flat, smaller on slopes).
pub fn ground(t: &Terrain, p: V2, h: f32, up: f32) -> Color {
    let s = t.seed();
    let d = geo::inland(p);
    let n1 = terrain::noise(s ^ 0xC0, p.x, p.y, 260.0);
    let n2 = terrain::noise(s ^ 0xC1, p.x, p.y, 40.0);

    // Lowland grass, drier and patchier uphill.
    let mut c = mix(GRASS, GRASS_DRY, n1 * 0.9 + n2 * 0.25 - 0.1);
    c = mix(c, HEATH, smooth(140.0, 320.0, h) * 0.8);
    // The Qotiro plateau is arid scrub; its escarpments are bare sandstone.
    let arid = t.plateau(p);
    c = mix(c, mix(SCRUB, GRASS_DRY, n2 * 0.4), arid * 0.9);
    // Bare rock where it's steep or high.
    let steep = smooth(0.86, 0.66, up) * (0.75 + n1 * 0.5);
    let rock = if arid > 0.4 { SANDROCK } else { ROCK };
    c = mix(c, rock, steep.max(smooth(380.0, 520.0, h) * 0.85));
    // Snow on the high flats.
    c = mix(c, SNOW, smooth(640.0, 760.0, h) * smooth(0.78, 0.90, up) * 0.85);
    // Shingle along the waterline.
    c = mix(SHINGLE, c, d / 35.0);
    scale(c, 0.9 + n2 * 0.18)
}
