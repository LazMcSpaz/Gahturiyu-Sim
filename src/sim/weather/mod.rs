//! Weather: what it is doing, anywhere and at any time.
//!
//! The weather is **looked up, never run**. `weather_at(place, time)` works
//! the answer out from the world seed, the place and the time alone: there is
//! no weather state in the world, nothing is stepped, and nothing is saved.
//! Asking about yesterday or next week costs the same as asking about now,
//! and asking in any order gives the same answer. So a forecast is just the
//! same question asked about later hours, and nothing here can depend on how
//! finely the world is stepped or where the squad stands.
//!
//! Weather has a **character by region**. The land is divided into kinds of
//! country (`region.rs`): open sea, exposed coast, sheltered coast and
//! lowland, upland, mountain, and the dry plateau. Each has its own climate
//! table. A spot near a border gets a mix of both sides, fading over about
//! 200 m.
//!
//! The **big weather is shared** and **comes in off the sea**. The slow
//! curves and the storms belong to the whole country; each region takes its
//! own share of them, and they reach places further east a little later
//! (`lag`). A storm darkens the sea, then the coast, then the hills, and the
//! dry plateau often misses it. Still nothing is run: a place's weather is
//! its regions' skies read at `t - lag(place)`.
//!
//! What a region's weather tends to be (how grey, how wet, how windy, how
//! often it storms or fogs, season by season) is all in
//! `data/weather/climate.ron`. `sky.rs` turns those tables into a region's
//! weather at a moment; `local.rs` brings it down to a particular spot (colder
//! with height, snow above the snowline, fog pooling in the hollows).
//!
//! Nothing else in the simulation reads the weather yet. The hooks for that
//! are plain functions here, for whoever wires them in.

mod climate;
mod lightning;
mod local;
mod noise;
mod region;
pub mod report;
mod sky;

pub use climate::{climate, seasonal, year_phase, Climate, RegionClimate};
pub use local::{localise, quarter, sight_words, wind_word, Kind, Weather, GALE, STORM_WIND, THICK_FOG};
pub use region::{floor, mix, region_at, shore_exposure, strongest, Mix, Region, REGIONS};
pub use lightning::{strikes, Strike};
pub use sky::{sky, sky_at, FogBank, Maker, Sky, Storm};

use super::geo::{V2, WORLD_SIZE};
use super::terrain::Terrain;
use super::world::{World, HOUR};

/// How long after the sea's edge the weather reaches a spot, seconds. It
/// crosses the map from a little north of west, on a front that bulges and
/// lags here and there (the same shape every time: it is the lie of the
/// land, not the storm).
pub fn lag(pos: V2) -> f64 {
    let (x, y) = (pos.x.clamp(0.0, WORLD_SIZE), pos.y.clamp(0.0, WORLD_SIZE));
    let bulge = 1300.0 * (y / 3100.0 + 0.7).sin() + 500.0 * (y / 1270.0 + 2.1).sin() + 500.0 * (x / 1900.0 + y / 2300.0).sin();
    let along = ((x + 0.22 * y + bulge + 2300.0) / (1.22 * WORLD_SIZE + 4600.0)).clamp(0.0, 1.0);
    (along * climate().crossing_hours) as f64 * HOUR
}

/// The weather at a spot at a moment. `seed` is the world's seed.
pub fn weather_at(terrain: &Terrain, seed: u64, pos: V2, t: f64) -> Weather {
    let shares = mix(terrain, pos);
    let behind = lag(pos);
    let mut blended = Sky::default();
    for r in Region::ALL {
        let share = shares[r as usize];
        if share > 1e-4 {
            blended.add(&sky_at(seed, r, t, behind), share);
        }
    }
    localise(&blended, strongest(&shares), terrain.surface(pos), floor(terrain, pos))
}

/// Every region's sky at a moment, in `Region::ALL` order, as the weather
/// stands `behind` seconds in from the sea's edge (`lag(pos)`). For asking
/// about many places that are about as far east as each other at one time:
/// work the skies out once, then use `weather_with` for each place.
pub fn skies(seed: u64, t: f64, behind: f64) -> [Sky; REGIONS] {
    Region::ALL.map(|r| sky_at(seed, r, t, behind))
}

/// The regions' skies mixed in the shares a spot belongs to each.
pub fn blend(skies: &[Sky; REGIONS], shares: &Mix) -> Sky {
    let mut blended = Sky::default();
    for r in Region::ALL {
        let share = shares[r as usize];
        if share > 1e-4 {
            blended.add(&skies[r as usize], share);
        }
    }
    blended
}

/// The weather at a spot whose region shares, height and fog floor are
/// already known (`mix`, `Terrain::surface`, `floor`), from `skies`. Given
/// the skies for the spot's own `lag`, this is exactly what `weather_at` gives.
pub fn weather_with(skies: &[Sky; REGIONS], shares: &Mix, height: f32, low: f32) -> Weather {
    localise(&blend(skies, shares), strongest(shares), height, low)
}

/// A region's weather at a moment, at a typical spot in it (its usual
/// height, on its low ground), as it comes in off the sea: places further
/// east get it up to `crossing_hours` later.
pub fn weather_in(seed: u64, region: Region, t: f64) -> Weather {
    let h = climate().of(region).ref_height;
    localise(&sky(seed, region, t), region, h, h)
}

/// A region's weather hour by hour, starting at `from`.
pub fn forecast(seed: u64, region: Region, from: f64, hours: usize) -> Vec<Weather> {
    (0..hours).map(|k| weather_in(seed, region, from + k as f64 * HOUR)).collect()
}

/// The weather at one spot hour by hour, starting at `from`.
pub fn forecast_at(terrain: &Terrain, seed: u64, pos: V2, from: f64, hours: usize) -> Vec<Weather> {
    let (shares, behind, height, low) = (mix(terrain, pos), lag(pos), terrain.surface(pos), floor(terrain, pos));
    (0..hours).map(|k| weather_with(&skies(seed, from + k as f64 * HOUR, behind), &shares, height, low)).collect()
}

impl World {
    /// The weather at a spot at a moment (any moment: past, now or to come).
    pub fn weather_at(&self, pos: V2, t: f64) -> Weather {
        weather_at(&self.terrain, self.seed, pos, t)
    }

    /// The weather at a spot now.
    pub fn weather(&self, pos: V2) -> Weather {
        self.weather_at(pos, self.time)
    }

    /// A region's weather hour by hour, starting at `from`.
    pub fn forecast(&self, region: Region, from: f64, hours: usize) -> Vec<Weather> {
        forecast(self.seed, region, from, hours)
    }

    /// The weather at one spot hour by hour, starting at `from`.
    pub fn forecast_at(&self, pos: V2, from: f64, hours: usize) -> Vec<Weather> {
        forecast_at(&self.terrain, self.seed, pos, from, hours)
    }

    /// Lightning strikes anywhere on the map between two moments.
    pub fn strikes(&self, from: f64, to: f64) -> Vec<Strike> {
        strikes(&self.terrain, self.seed, from, to)
    }

    /// The climate region a spot mostly belongs to.
    pub fn climate_region(&self, pos: V2) -> Region {
        region_at(&self.terrain, pos)
    }
}
