//! Colours, shared by the 3D view, the map and the panels so they agree.
//!
//! Colours are written as sRGB values 0..1 (what a paint program shows).
//! `lin` turns one into the linear light the 3D renderer works in, `eg` into
//! an egui colour for the panels.

use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    geo::{self, V2},
    race::Race,
    terrain::{self, Terrain},
};

pub type Rgb = [f32; 3];

pub const GRASS: Rgb = [0.30, 0.40, 0.24];
const GRASS_DRY: Rgb = [0.43, 0.45, 0.29];
const HEATH: Rgb = [0.38, 0.36, 0.27];
const SHINGLE: Rgb = [0.47, 0.47, 0.44];
const ROCK: Rgb = [0.40, 0.39, 0.38];
const SNOW: Rgb = [0.88, 0.90, 0.92];
const SCRUB: Rgb = [0.64, 0.54, 0.36];
const SANDROCK: Rgb = [0.74, 0.56, 0.36];
pub const SEA: Rgb = [0.22, 0.32, 0.34];
pub const SEA_DEEP: Rgb = [0.14, 0.22, 0.27];
pub const ROAD: Rgb = [0.45, 0.36, 0.25];
/// A forged town's paved lane, its stair treads and their risers.
pub const COBBLES: Rgb = [0.40, 0.39, 0.37];
pub const PATH: Rgb = [0.40, 0.34, 0.26];
pub const STAIR: Rgb = [0.50, 0.48, 0.44];
pub const STAIR_RISER: Rgb = [0.30, 0.29, 0.27];

pub const STONE: Rgb = [0.38, 0.39, 0.41];
pub const SANDSTONE: Rgb = [0.78, 0.63, 0.42];
pub const WEAVE: Rgb = [0.24, 0.21, 0.16];
pub const TIMBER: Rgb = [0.36, 0.27, 0.18];
pub const METAL_GOLD: Rgb = [0.95, 0.76, 0.28];
pub const EMBER: Rgb = [1.0, 0.62, 0.26];
pub const WINDOW: Rgb = [1.0, 0.70, 0.36];
pub const TENT: Rgb = [0.62, 0.64, 0.74];
pub const CAMP_HIDE: Rgb = [0.42, 0.24, 0.18];

// Panels.
pub const TEXT: Rgb = [0.90, 0.91, 0.88];
pub const DIM: Rgb = [0.62, 0.66, 0.63];
pub const GOLD: Rgb = [1.0, 0.85, 0.35];
pub const WARN: Rgb = [0.95, 0.55, 0.3];
pub const SNEAK: Rgb = [0.62, 0.70, 0.95];
pub const MANA: Rgb = [0.35, 0.55, 1.0];
pub const WHITE: Rgb = [1.0, 1.0, 1.0];

pub fn race_color(r: Race) -> Rgb {
    match r {
        Race::Roduro => [0.80, 0.72, 0.60], // weathered stone
        Race::Qotiro => [0.95, 0.47, 0.22], // ember
        Race::Horaro => [0.38, 0.68, 0.96], // sea
        Race::Tadoro => [0.78, 0.68, 0.98], // pale violet
    }
}

/// Skin by people, for the head.
pub fn skin(r: Race) -> Rgb {
    match r {
        Race::Roduro => [0.58, 0.47, 0.39],
        Race::Qotiro => [0.56, 0.57, 0.59], // grey
        Race::Horaro => [0.36, 0.52, 0.78], // blue
        Race::Tadoro => [0.80, 0.75, 0.72],
    }
}

pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn scale(c: Rgb, k: f32) -> Rgb {
    [c[0] * k, c[1] * k, c[2] * k]
}

fn to_lin(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear RGBA for a vertex colour.
pub fn lin(c: Rgb) -> [f32; 4] {
    [to_lin(c[0]), to_lin(c[1]), to_lin(c[2]), 1.0]
}

pub fn eg(c: Rgb) -> Color32 {
    Color32::from_rgb((c[0].clamp(0.0, 1.0) * 255.0) as u8, (c[1].clamp(0.0, 1.0) * 255.0) as u8, (c[2].clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn ega(c: Rgb, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied((c[0].clamp(0.0, 1.0) * 255.0) as u8, (c[1].clamp(0.0, 1.0) * 255.0) as u8, (c[2].clamp(0.0, 1.0) * 255.0) as u8, (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub fn bevy(c: Rgb) -> bevy::color::Color {
    bevy::color::Color::srgb(c[0], c[1], c[2])
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Colour of the sea surface at a point.
pub fn sea(p: V2) -> Rgb {
    mix(SEA, SEA_DEEP, -geo::inland(p) / 700.0)
}

/// Colour of the ground at a point, given its height and how upright its
/// surface is (`up` = 1 flat, smaller on slopes).
pub fn ground(t: &Terrain, p: V2, h: f32, up: f32) -> Rgb {
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
    // Shingle along the waterline: by height where the land was authored
    // (its shore is wherever it meets the sea), by distance from the world's
    // coastline elsewhere (that shore is nearly flat).
    let shore = if t.authored.height.at(p).abs() > 0.01 { h / 3.0 } else { d / 35.0 };
    c = mix(SHINGLE, c, shore);
    // Ground painted: authored first, then by hand on top.
    for layer in [&t.authored, &t.edits] {
        if let Some((k, w)) = layer.paint_at(p) {
            c = mix(c, texture(k), w);
        }
    }
    scale(c, 0.9 + n2 * 0.18)
}

/// The colour of each paintable ground (`mapedit::TEXTURES`, in order).
pub fn texture(k: u8) -> Rgb {
    const T: [Rgb; 13] = [
        [0.30, 0.42, 0.22], // grass
        GRASS_DRY,
        HEATH,
        SCRUB,
        [0.42, 0.33, 0.24], // dirt
        [0.28, 0.23, 0.18], // mud
        [0.80, 0.72, 0.52], // sand
        SHINGLE,
        [0.55, 0.53, 0.49], // gravel
        ROCK,
        SANDROCK,
        SNOW,
        [0.24, 0.36, 0.20], // moss
    ];
    T[(k as usize).min(T.len() - 1)]
}
