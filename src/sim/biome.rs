//! The kinds of country (Laz, 11 Oct 2026). These seven are the only ones
//! on the map:
//!
//! - **rolling plains**, the open grassland that covers most of the land
//!   (the raised south-east included);
//! - **forests**;
//! - **the coast**: the beach and the land just behind it;
//! - **bog**: the low wet flats behind the coast and the wet valley bottoms;
//! - **rocky ground**: the mountains below the snow, the massif, steep faces,
//!   ravines and chasms, and the south-east highland's broken rim;
//! - **snowy mountains**, the high mountain ground;
//! - **the Overgrowth**, wherever it has taken hold.
//!
//! There is no desert, steppe or dry plateau anywhere: the Qotiro came from
//! desert country, but here they live in Roduro lands, away from home.
//!
//! The first six are read off the land itself (a pure function of the seed
//! and the map's hand edits, like the ground underfoot); the Overgrowth lies
//! on top of them, region by region (`World::overgrowth`), so a spot's
//! biome is `World::biome_at`. Everything that sorts country into kinds
//! (animals' homes, the weather's regions, what the land gives) starts
//! from here.

use serde::{Deserialize, Serialize};

use super::animals::{region_of, wooded};
use super::geo::{self, V2};
use super::terrain::{self, Terrain};
use super::World;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    Plains,
    Forest,
    Coast,
    Bog,
    Rocky,
    Snowy,
    Overgrowth,
}

pub const BIOMES: usize = 7;

impl Biome {
    pub const ALL: [Biome; BIOMES] = [Biome::Plains, Biome::Forest, Biome::Coast, Biome::Bog, Biome::Rocky, Biome::Snowy, Biome::Overgrowth];

    /// In plain words, for hovers and panels.
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "rolling plains",
            Biome::Forest => "forest",
            Biome::Coast => "coast",
            Biome::Bog => "bog",
            Biome::Rocky => "rocky ground",
            Biome::Snowy => "snowy mountains",
            Biome::Overgrowth => "the Overgrowth",
        }
    }
}

/// The beach: this far back from the water, metres.
pub const SHORE: f32 = 60.0;
/// The coast reaches this far inland, metres (beyond that it's whatever the
/// land is: plains, forest, bog...).
pub const COAST_BAND: f32 = 400.0;
/// Mountain ground above this height, metres, is snowy mountain; below it,
/// rocky. (Where snow lies on a given day is the weather's business: it can
/// come lower in winter and draw back in summer.)
pub const SNOW_COUNTRY: f32 = 560.0;
/// Ground steeper than this (rise per metre) is rocky wherever it is.
pub const STEEP: f32 = 0.35;
/// A region is the Overgrowth when its level is above this.
pub const OVERGROWN: f32 = 0.5;

/// Mountain ground: well into the ranges or the massif, or steep and high.
pub fn mountain_ground(t: &Terrain, p: V2) -> bool {
    t.mountains(p) > 0.55 || (t.slope(p) > STEEP && t.height(p) > 250.0)
}

/// Bog: the low, flat ground behind the coast (not under sea cliffs), and
/// wet valley bottoms (flat ground well below what stands round it). Within
/// `COAST_BAND` of the sea it's the coast instead.
pub fn bog_ground(t: &Terrain, p: V2) -> bool {
    let (d, h, s) = (geo::inland(p), t.height(p), t.slope(p));
    let low_coast = d < 700.0 && h < 9.0 && s < 0.05 && terrain::cliff_mask(t.seed(), p.y) < 0.3;
    if low_coast {
        return true;
    }
    if !(s < 0.08 && h < 160.0) {
        return false;
    }
    let mut sum = 0.0;
    for k in 0..8 {
        let a = k as f32 / 8.0 * std::f32::consts::TAU;
        sum += t.height(p.add(V2::new(a.cos(), a.sin()).scale(300.0)));
    }
    sum / 8.0 - h > 12.0
}

/// The kind of country the land itself makes at a spot, Overgrowth aside;
/// `None` at sea.
pub fn land_biome(world_seed: u64, t: &Terrain, p: V2) -> Option<Biome> {
    // The land decides (authored bays and stacks included), as for the sea.
    if t.is_sea(p) {
        return None;
    }
    let d = geo::inland(p);
    if d < SHORE {
        return Some(Biome::Coast);
    }
    if mountain_ground(t, p) {
        return Some(if t.height(p) >= SNOW_COUNTRY { Biome::Snowy } else { Biome::Rocky });
    }
    if t.slope(p) > STEEP || t.on_rim(p) {
        return Some(Biome::Rocky);
    }
    if d < COAST_BAND {
        return Some(Biome::Coast);
    }
    if bog_ground(t, p) {
        return Some(Biome::Bog);
    }
    if wooded(world_seed, p) > 0.5 {
        return Some(Biome::Forest);
    }
    Some(Biome::Plains)
}

impl World {
    /// The kind of country at a spot: the Overgrowth where it has taken
    /// hold, otherwise what the land makes. `None` at sea.
    pub fn biome_at(&self, p: V2) -> Option<Biome> {
        let land = land_biome(self.seed, &self.terrain, p)?;
        if self.overgrowth(region_of(p)) > OVERGROWN {
            return Some(Biome::Overgrowth);
        }
        Some(land)
    }
}

/// Each biome's share of the land, sampled every `step` metres across the
/// map (the Overgrowth included).
pub fn shares(w: &World, step: f32) -> [f32; BIOMES] {
    let mut n = [0usize; BIOMES];
    let mut all = 0usize;
    let k = (geo::WORLD_SIZE / step) as usize;
    for j in 0..k {
        for i in 0..k {
            let p = V2::new((i as f32 + 0.5) * step, (j as f32 + 0.5) * step);
            if let Some(b) = w.biome_at(p) {
                n[b as usize] += 1;
                all += 1;
            }
        }
    }
    let mut out = [0.0; BIOMES];
    for b in 0..BIOMES {
        out[b] = n[b] as f32 / all.max(1) as f32;
    }
    out
}

/// A printed survey: each biome's share, every town and the country it
/// stands in, and where each kind of animal makes its home
/// (`headless biomes [seed]`).
pub fn report(w: &World) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let s = shares(w, 100.0);
    let _ = writeln!(out, "Land by biome (seed {}):", w.seed);
    for b in Biome::ALL {
        let _ = writeln!(out, "  {:<16} {:>5.1}%", b.name(), s[b as usize] * 100.0);
    }
    // How high the mountain ground stands (where the snow country begins).
    let mut hs: Vec<f32> = Vec::new();
    let k = (geo::WORLD_SIZE / 100.0) as usize;
    for j in 0..k {
        for i in 0..k {
            let p = V2::new((i as f32 + 0.5) * 100.0, (j as f32 + 0.5) * 100.0);
            if !w.terrain.is_sea(p) && mountain_ground(&w.terrain, p) {
                hs.push(w.terrain.height(p));
            }
        }
    }
    hs.sort_by(|a, b| a.total_cmp(b));
    if !hs.is_empty() {
        let q = |f: f32| hs[((hs.len() - 1) as f32 * f) as usize];
        let _ = writeln!(out, "  (mountain ground: half below {:.0} m, a quarter above {:.0} m, a tenth above {:.0} m, highest {:.0} m; snowy from {:.0} m)", q(0.5), q(0.75), q(0.9), q(1.0), SNOW_COUNTRY);
    }
    let _ = writeln!(out, "\nTowns:");
    for t in &w.settlements {
        let b = w.biome_at(t.pos).map(|b| b.name()).unwrap_or("sea");
        let _ = writeln!(out, "  {:<16} ({:>6.0}, {:>6.0})  {:<8} {}", t.name, t.pos.x, t.pos.y, format!("{:?}", t.founders), b);
    }
    let _ = writeln!(out, "\nAnimals' homes, by biome:");
    let mut by: Vec<(&'static str, [usize; BIOMES + 1])> = Vec::new();
    for h in &w.animals.herds {
        let name = h.sp.def().name;
        let k = match w.biome_at(h.home) {
            Some(b) => b as usize,
            None => BIOMES,
        };
        match by.iter_mut().find(|(n, _)| *n == name) {
            Some((_, c)) => c[k] += 1,
            None => {
                let mut c = [0; BIOMES + 1];
                c[k] = 1;
                by.push((name, c));
            }
        }
    }
    for (name, c) in by {
        let parts: Vec<String> = (0..=BIOMES).filter(|&k| c[k] > 0).map(|k| format!("{} {}", if k < BIOMES { Biome::ALL[k].name() } else { "water" }, c[k])).collect();
        let _ = writeln!(out, "  {:<16} {}", name, parts.join(", "));
    }
    out
}
