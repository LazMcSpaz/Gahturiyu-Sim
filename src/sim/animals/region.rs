//! Regions: the world cut into squares, each knowing what kinds of country
//! it holds. Far from anyone, wildlife is just numbers per region; those
//! numbers are the sum of the herds whose home is there (see `herd.rs`).
//!
//! Also here, because they are per region or per spot and nothing else:
//! the Overgrowth level (a stored stand-in until that system exists), fish
//! stocks along the coast, and the danger of diving at a spot.

use serde::{Deserialize, Serialize};

use super::super::geo::{self, V2, WORLD_SIZE};
use super::super::rng::{self, Rng};
use super::super::terrain::{self, Ground, Terrain};
use super::super::world::{World, DAY};
use super::species::{Class, Habitat, Sp, FISH, N_HABITATS};

/// One side of a region, metres.
pub const REGION: f32 = 1500.0;
/// Regions along one side of the map.
pub const ACROSS: usize = (WORLD_SIZE / REGION) as usize;
/// Sample points along one side of a region when its country is surveyed.
const SURVEY: usize = 10;
/// The middle of the central massif (the same spot `terrain` raises it at).
pub const MASSIF: V2 = V2::new(8_600.0, 12_600.0);
pub const MASSIF_RADIUS: f32 = 2_600.0;
/// How far back from the water the shore reaches, metres.
pub const SHORE_STRIP: f32 = 60.0;

pub type RegionId = u16;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Region {
    /// Square kilometres of each kind of country here (they can overlap).
    pub habitat: [f32; N_HABITATS],
    /// Kilometres of shoreline.
    pub coast: f32,
    /// How far the Overgrowth has taken hold, 0..1. A stored stand-in: the
    /// system that moves it doesn't exist yet (see `World::set_overgrowth`).
    pub overgrowth: f32,
    /// The herds whose home is here (indices into `Animals::herds`).
    pub herds: Vec<u32>,
}

/// A stock of one kind of fish off one region's shore: a number that grows
/// back toward what the water can hold.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FishStock {
    pub region: RegionId,
    pub sp: Sp,
    pub cap: f32,
    /// The stock at time `at`.
    pub n: f32,
    pub at: f64,
}

impl FishStock {
    pub fn at_time(&self, t: f64) -> f32 {
        logistic(self.n, self.cap, self.sp.def().growth as f64 / DAY, t - self.at)
    }
}

/// A number growing toward a limit: slow at first, fastest halfway, slowing
/// as it fills. Worked out in one go for any stretch of time, so nothing
/// needs counting day by day. (Above the limit it sinks back toward it.)
pub fn logistic(n0: f32, limit: f32, rate_per_sec: f64, dt: f64) -> f32 {
    if n0 <= 0.0 || limit <= 0.0 {
        return 0.0;
    }
    if rate_per_sec <= 0.0 || dt <= 0.0 {
        return n0;
    }
    let (n0, k) = (n0 as f64, limit as f64);
    (k / (1.0 + (k - n0) / n0 * (-rate_per_sec * dt).exp())) as f32
}

pub fn region_of(p: V2) -> RegionId {
    let i = ((p.x / REGION).floor().max(0.0) as usize).min(ACROSS - 1);
    let j = ((p.y / REGION).floor().max(0.0) as usize).min(ACROSS - 1);
    (j * ACROSS + i) as RegionId
}

pub fn region_centre(r: RegionId) -> V2 {
    let (i, j) = (r as usize % ACROSS, r as usize / ACROSS);
    V2::new((i as f32 + 0.5) * REGION, (j as f32 + 0.5) * REGION)
}

/// The regions touching `r`, itself included.
pub fn neighbours(r: RegionId) -> Vec<RegionId> {
    let (i, j) = ((r as usize % ACROSS) as i64, (r as usize / ACROSS) as i64);
    let mut out = Vec::with_capacity(9);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (x, y) = (i + di, j + dj);
            if x >= 0 && y >= 0 && (x as usize) < ACROSS && (y as usize) < ACROSS {
                out.push((y as usize * ACROSS + x as usize) as RegionId);
            }
        }
    }
    out
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How wooded a spot is, 0..1. The same slow noise the window grows its
/// trees from, so the animals of the woods are found where the woods are
/// drawn.
pub fn wooded(world_seed: u64, p: V2) -> f32 {
    smooth(0.52, 0.66, terrain::noise(world_seed ^ 0xF0E5_7D11, p.x, p.y, 750.0))
}

fn bit(h: Habitat) -> u16 {
    1 << (h as u16)
}

/// Which kinds of country a spot on land is. A bit per `Habitat`.
pub fn country(world_seed: u64, t: &Terrain, p: V2) -> u16 {
    if !geo::is_land(p) {
        return 0;
    }
    let d = geo::inland(p);
    let h = t.height(p);
    let s = t.slope(p);
    let m = t.mountains(p);
    let pl = t.plateau(p);
    let mut out = 0u16;
    if d < SHORE_STRIP {
        return bit(Habitat::Shore);
    }
    let high = m > 0.55 || (s > 0.35 && h > 250.0);
    if high {
        out |= bit(Habitat::Mountain);
        if p.dist(MASSIF) < MASSIF_RADIUS {
            out |= bit(Habitat::Massif);
        }
    }
    // Ledges: the steepest ground anywhere, and the plateau's broken edge.
    if s > 0.5 || (pl > 0.2 && pl < 0.7 && s > 0.28) {
        out |= bit(Habitat::Ledge);
    }
    if high {
        return out;
    }
    if pl > 0.6 {
        return out | bit(Habitat::Plateau);
    }
    if s > 0.26 && s < 0.7 && h > 30.0 && d > 300.0 && pl < 0.4 {
        out |= bit(Habitat::Cliff);
    }
    // Wet ground: the low flat coast behind a beach, and valley bottoms
    // (ground well below what stands round it).
    let low_coast = d < 700.0 && h < 9.0 && s < 0.05 && terrain::cliff_mask(t.seed(), p.y) < 0.3;
    let hollow = || {
        let mut sum = 0.0;
        for k in 0..8 {
            let a = k as f32 / 8.0 * std::f32::consts::TAU;
            sum += t.height(p.add(V2::new(a.cos(), a.sin()).scale(300.0)));
        }
        sum / 8.0 - h
    };
    if low_coast || (s < 0.08 && h < 160.0 && hollow() > 12.0) {
        return out | bit(Habitat::Marsh);
    }
    if s > 0.3 {
        return out;
    }
    if wooded(world_seed, p) > 0.5 {
        out |= bit(Habitat::Wood);
    } else if t.ground(p) == Ground::Scrub {
        out |= bit(Habitat::Scrub);
    } else {
        out |= bit(Habitat::Grass);
    }
    out
}

/// Look a region over: sample points on land with what country each is,
/// and the length of shore.
pub(super) fn survey(world_seed: u64, t: &Terrain, r: RegionId) -> (Vec<(V2, u16)>, f32) {
    let c = region_centre(r);
    let step = REGION / SURVEY as f32;
    let (x0, y0) = (c.x - REGION / 2.0, c.y - REGION / 2.0);
    let mut out = Vec::with_capacity(SURVEY * SURVEY);
    for j in 0..SURVEY {
        for i in 0..SURVEY {
            let p = V2::new(x0 + (i as f32 + 0.5) * step, y0 + (j as f32 + 0.5) * step);
            let c = country(world_seed, t, p);
            if c != 0 {
                out.push((p, c));
            }
        }
    }
    // The shore is a strip too narrow for the grid to find: walk it.
    let mut coast = 0.0;
    for j in 0..SURVEY {
        let y = y0 + (j as f32 + 0.5) * step;
        let x = geo::coast_x(y);
        if x >= x0 && x < x0 + REGION {
            coast += step / 1000.0;
            let p = V2::new(x + SHORE_STRIP * 0.25, y);
            if terrain::cliff_mask(t.seed(), y) < 0.5 {
                out.push((p, bit(Habitat::Shore)));
            }
        }
    }
    (out, coast)
}

impl Region {
    pub(super) fn from_survey(points: &[(V2, u16)], coast: f32) -> Region {
        let cell = (REGION / SURVEY as f32 / 1000.0).powi(2);
        let mut habitat = [0.0; N_HABITATS];
        for (_, c) in points {
            for (k, h) in habitat.iter_mut().enumerate() {
                if c & (1 << k) != 0 {
                    *h += cell;
                }
            }
        }
        // (The shore's area is its length times its width, not a count of
        // grid points.)
        habitat[Habitat::Shore as usize] = coast * SHORE_STRIP / 1000.0;
        Region { habitat, coast, overgrowth: 0.0, herds: Vec::new() }
    }

    /// Square kilometres here that suit a species.
    pub fn room_for(&self, sp: Sp) -> f32 {
        sp.def().habitats.iter().map(|h| self.habitat[*h as usize]).sum()
    }
}

/// How dangerous it is to dive at a spot, 0..1: where the Deepcoil keeps.
/// Fixed for a spot by the world seed (deeper water and certain shellbeds
/// are worse). What the danger does to a diver is for others to decide.
pub fn dive_danger(world_seed: u64, p: V2) -> f32 {
    let off = -geo::inland(p);
    if off <= 0.0 {
        return 0.0;
    }
    let deep = smooth(150.0, 900.0, off);
    let patch = terrain::noise(world_seed ^ 0xDEE9_C011, p.x, p.y, 600.0);
    let shellbed = smooth(0.72, 0.82, terrain::noise(world_seed ^ 0x5E11_BED5, p.x, p.y, 380.0));
    (deep * (0.25 + 1.3 * patch * patch) + 0.3 * shellbed).clamp(0.0, 1.0)
}

/// What a stretch of water can hold of one kind of fish.
pub(super) fn fish_cap(world_seed: u64, t: &Terrain, r: RegionId, coast_km: f32, sp: Sp) -> f32 {
    let c = region_centre(r);
    let mut k = Rng::from_keys(&[world_seed, r as u64, sp as u64, 0xF15B]);
    let luck = 0.6 + 0.8 * k.f32();
    let rocky = terrain::cliff_mask(t.seed(), c.y);
    let share = match sp {
        Sp::Silverling => 1.0,
        Sp::Slatefin => 0.15 + rocky,
        Sp::Ribbonback => smooth(0.45, 0.7, terrain::noise(world_seed ^ 0x6E19, 0.0, c.y, 3000.0)),
        Sp::Gulpjaw => 0.5,
        _ => 0.0,
    };
    sp.def().density * coast_km * share * luck
}

impl World {
    // ---- Regions --------------------------------------------------------------

    /// How many of a species live in a region right now (the far-away view
    /// of wildlife: a number per region). Wild animals only.
    pub fn population(&self, r: RegionId, sp: Sp) -> f32 {
        self.population_at(r, sp, self.time)
    }

    pub fn population_at(&self, r: RegionId, sp: Sp, t: f64) -> f32 {
        if sp.def().class == Class::Fish {
            return self.fish_stock(r, sp, t);
        }
        let Some(reg) = self.animals.regions.get(r as usize) else { return 0.0 };
        reg.herds.iter().map(|&h| &self.animals.herds[h as usize]).filter(|h| h.sp == sp).map(|h| h.alive(t) as f32).sum()
    }

    /// How many of a species a region can hold: every home range full.
    pub fn capacity(&self, r: RegionId, sp: Sp) -> f32 {
        if sp.def().class == Class::Fish {
            return self.animals.fish.iter().filter(|f| f.region == r && f.sp == sp).map(|f| f.cap).sum();
        }
        let Some(reg) = self.animals.regions.get(r as usize) else { return 0.0 };
        reg.herds.iter().map(|&h| &self.animals.herds[h as usize]).filter(|h| h.sp == sp).map(|h| h.cap as f32).sum()
    }

    /// Everything wild across the whole world, by species.
    pub fn wildlife_census(&self) -> Vec<(Sp, f32)> {
        let t = self.time;
        super::species::ALL_SPECIES
            .iter()
            .map(|&sp| (sp, self.animals.herds.iter().filter(|h| h.sp == sp).map(|h| h.alive(t) as f32).sum::<f32>()))
            .filter(|(_, n)| *n > 0.0)
            .collect()
    }

    // ---- The Overgrowth (a stand-in) ------------------------------------------

    /// The Overgrowth level in a region, 0..1.
    ///
    /// This is the one small function animals read the Overgrowth through.
    /// For now it returns a stored value (0 unless someone sets it); when the
    /// Overgrowth exists as a system, replace the body of this function.
    pub fn overgrowth(&self, r: RegionId) -> f32 {
        self.animals.regions.get(r as usize).map(|x| x.overgrowth).unwrap_or(0.0)
    }

    /// Set a region's Overgrowth level (for whoever owns the Overgrowth, and
    /// for tests). Briarbacks appear where it stands above
    /// `BRIAR_THRESHOLD`, and die back where it falls below.
    pub fn set_overgrowth(&mut self, r: RegionId, level: f32) {
        self.set_overgrowth_at(r, level, self.time)
    }

    /// The same, as of a given moment (for a caller on the world's timeline).
    pub fn set_overgrowth_at(&mut self, r: RegionId, level: f32, t: f64) {
        let Some(reg) = self.animals.regions.get_mut(r as usize) else { return };
        reg.overgrowth = level.clamp(0.0, 1.0);
        self.briars_follow_overgrowth(r, t);
    }

    // ---- Fish -----------------------------------------------------------------

    /// How much of one kind of fish there is off a region's shore.
    pub fn fish_stock(&self, r: RegionId, sp: Sp, t: f64) -> f32 {
        self.animals.fish.iter().filter(|f| f.region == r && f.sp == sp).map(|f| f.at_time(t)).sum()
    }

    /// Take fish from a region's water: up to `want`, and never more than
    /// half of what's there at one go. Returns what was caught; the stock
    /// grows back on its own. (For the society side to call; nothing here
    /// decides who fishes or what a catch is worth.)
    pub fn catch_fish(&mut self, r: RegionId, sp: Sp, want: f32) -> f32 {
        self.catch_fish_at(r, sp, want, self.time)
    }

    /// The same, as of a given moment (for a caller on the world's timeline).
    pub fn catch_fish_at(&mut self, r: RegionId, sp: Sp, want: f32, t: f64) -> f32 {
        let Some(f) = self.animals.fish.iter_mut().find(|f| f.region == r && f.sp == sp) else { return 0.0 };
        let t = t.max(f.at);
        let have = f.at_time(t);
        let got = want.max(0.0).min(have * 0.5);
        f.n = have - got;
        f.at = t;
        got
    }

    /// The coastal region nearest a point (for "catch from the water here").
    pub fn fishing_region(&self, p: V2) -> Option<RegionId> {
        let here = region_of(V2::new(geo::coast_x(p.y), p.y));
        self.animals.regions.get(here as usize).filter(|r| r.coast > 0.0).map(|_| here)
    }

    /// How dangerous it is to dive at a spot, 0..1 (the Deepcoil).
    pub fn dive_danger(&self, p: V2) -> f32 {
        dive_danger(self.seed, p)
    }

    pub(super) fn stock_the_sea(&mut self) {
        let seed = self.seed;
        let mut fish = Vec::new();
        for (r, reg) in self.animals.regions.iter().enumerate() {
            if reg.coast <= 0.0 {
                continue;
            }
            for sp in FISH {
                let cap = fish_cap(seed, &self.terrain, r as RegionId, reg.coast, sp);
                if cap >= 1.0 {
                    let fill = 0.7 + 0.3 * rng::Rng::from_keys(&[seed, r as u64, sp as u64, 0xF111]).f32();
                    fish.push(FishStock { region: r as RegionId, sp, cap, n: cap * fill, at: self.time });
                }
            }
        }
        self.animals.fish = fish;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_grow_toward_their_limit_and_no_further() {
        let day = 86_400.0;
        let r = 0.1 / day;
        let a = logistic(10.0, 100.0, r, 10.0 * day);
        let b = logistic(10.0, 100.0, r, 20.0 * day);
        let far = logistic(10.0, 100.0, r, 2000.0 * day);
        assert!(a > 10.0 && b > a && far <= 100.0 && far > 99.0);
        // In two steps or one: the same.
        let two = logistic(a, 100.0, r, 10.0 * day);
        assert!((two - b).abs() < 1e-3, "{two} vs {b}");
        // Over the limit sinks back.
        assert!(logistic(150.0, 100.0, r, 30.0 * day) < 150.0);
        assert_eq!(logistic(0.0, 100.0, r, day), 0.0);
    }

    #[test]
    fn regions_tile_the_map() {
        assert_eq!(region_of(V2::new(0.0, 0.0)), 0);
        assert_eq!(region_of(V2::new(WORLD_SIZE - 1.0, WORLD_SIZE - 1.0)) as usize, ACROSS * ACROSS - 1);
        let r = region_of(V2::new(4_000.0, 9_000.0));
        assert!(region_centre(r).dist(V2::new(4_000.0, 9_000.0)) < REGION);
        assert_eq!(neighbours(0).len(), 4);
    }
}
