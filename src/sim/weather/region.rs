//! Climate regions: which kind of country a spot is, read off the land.
//!
//! Each region has its own weather, and nothing travels from one to the
//! next. So that there are no hard lines, a spot near a border belongs
//! partly to each side (`mix`), fading over about 200 m.

use super::climate::climate;
use super::noise::smooth;
use crate::sim::geo::{self, V2};
use crate::sim::terrain::{self, Terrain};

/// The kinds of country, in the order of the climate file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    OpenSea = 0,
    ExposedCoast = 1,
    /// Sheltered coast and the low country behind it.
    Lowland = 2,
    Upland = 3,
    Mountain = 4,
    /// The dry Qotiro plateau in the south-east.
    Plateau = 5,
}

pub const REGIONS: usize = 6;

/// How much a spot belongs to each region; the shares add up to 1.
pub type Mix = [f32; REGIONS];

impl Region {
    pub const ALL: [Region; REGIONS] = [Region::OpenSea, Region::ExposedCoast, Region::Lowland, Region::Upland, Region::Mountain, Region::Plateau];

    pub fn name(self) -> &'static str {
        match self {
            Region::OpenSea => "open sea",
            Region::ExposedCoast => "exposed coast",
            Region::Lowland => "sheltered coast and lowland",
            Region::Upland => "upland",
            Region::Mountain => "mountain",
            Region::Plateau => "plateau",
        }
    }

    /// By the sea: where sea mist and surf belong.
    pub fn maritime(self) -> bool {
        matches!(self, Region::OpenSea | Region::ExposedCoast)
    }
}

/// How open to the sea the shore is at this latitude, 0..1: sea cliffs,
/// headlands that stick out into the water, and where the mountains come
/// down to the sea.
pub fn shore_exposure(terrain: &Terrain, y: f32) -> f32 {
    let cx = geo::coast_x(y);
    let cliffs = terrain::cliff_mask(terrain.seed(), y);
    let round = (geo::coast_x(y - 1500.0) + geo::coast_x(y - 750.0) + geo::coast_x(y + 750.0) + geo::coast_x(y + 1500.0)) / 4.0;
    let headland = smooth(40.0, 160.0, round - cx);
    let steep = smooth(0.3, 0.6, terrain.mountains(V2::new(cx + 700.0, y)));
    cliffs.max(headland).max(steep)
}

/// Which region a single point falls in, with soft edges only where the
/// land itself changes gently (the shore). The plateau's rim and the foot of
/// the mountains are sharp here; `mix` softens them.
fn mix_at_point(terrain: &Terrain, p: V2) -> Mix {
    let b = &climate().borders;
    let half = b.blend / 2.0;
    let d = geo::inland(p);
    let mut out = [0.0; REGIONS];
    let mut rest = 1.0f32;
    let mut take = |r: Region, share: f32, rest: &mut f32| {
        out[r as usize] = *rest * share;
        *rest *= 1.0 - share;
    };
    take(Region::OpenSea, smooth(-(b.sea_from - half), -(b.sea_from + half), d), &mut rest);
    if rest > 0.0 {
        let band = smooth(b.coast_reach + half, b.coast_reach - half, d);
        let exposed = if band > 0.0 { band * shore_exposure(terrain, p.y) } else { 0.0 };
        take(Region::ExposedCoast, exposed, &mut rest);
    }
    if rest > 0.0 {
        take(Region::Mountain, smooth(0.42, 0.52, terrain.mountains(p)) * smooth(0.0, 2.0 * half, d), &mut rest);
    }
    if rest > 0.0 {
        take(Region::Plateau, smooth(0.42, 0.58, terrain.plateau(p)), &mut rest);
    }
    if rest > 0.0 {
        take(Region::Upland, smooth(b.upland_from - 20.0, b.upland_from + 20.0, terrain.height(p)), &mut rest);
    }
    out[Region::Lowland as usize] = rest;
    out
}

/// How much a spot belongs to each region: the regions found at the spot and
/// in two rings round it, averaged, so every border fades over about the
/// blend distance however sharp the land's own edge is there.
pub fn mix(terrain: &Terrain, p: V2) -> Mix {
    let reach = climate().borders.blend * 0.6;
    let mut out = mix_at_point(terrain, p);
    let mut n = 1.0;
    for (r, count, turn) in [(reach * 0.5, 6, 0.0f32), (reach, 8, 0.39)] {
        for k in 0..count {
            let a = turn + k as f32 / count as f32 * std::f32::consts::TAU;
            let m = mix_at_point(terrain, V2::new(p.x + r * a.cos(), p.y + r * a.sin()));
            for i in 0..REGIONS {
                out[i] += m[i];
            }
            n += 1.0;
        }
    }
    for v in &mut out {
        *v /= n;
    }
    out
}

/// The region a spot mostly belongs to.
pub fn region_at(terrain: &Terrain, p: V2) -> Region {
    strongest(&mix(terrain, p))
}

pub fn strongest(m: &Mix) -> Region {
    let mut best = 0;
    for k in 1..REGIONS {
        if m[k] > m[best] {
            best = k;
        }
    }
    Region::ALL[best]
}

/// The lowest ground (or water) round about a spot: the floor that fog
/// pools on. A hilltop's floor is the valley below it; a plain's is itself.
pub fn floor(terrain: &Terrain, p: V2) -> f32 {
    let mut low = terrain.surface(p);
    for (r, n, turn) in [(160.0f32, 6, 0.0f32), (420.0, 8, 0.4)] {
        for k in 0..n {
            let a = turn + k as f32 / n as f32 * std::f32::consts::TAU;
            low = low.min(terrain.surface(V2::new(p.x + r * a.cos(), p.y + r * a.sin())));
        }
    }
    low
}
