//! The climate tables: read once from `data/weather/climate.ron`, which is
//! where every number that shapes the weather lives.

use std::sync::OnceLock;

use serde::Deserialize;

use super::Region;
use crate::sim::tide::YEAR_DAYS;
use crate::sim::world::DAY;

/// A value for each season: spring, summer, autumn, winter.
pub type BySeason = [f32; 4];

/// Reads a by-season list written as `[spring, summer, autumn, winter]`.
fn four<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BySeason, D::Error> {
    let v = Vec::<f32>::deserialize(d)?;
    <[f32; 4]>::try_from(v.as_slice()).map_err(|_| serde::de::Error::custom(format!("expected four values (spring, summer, autumn, winter), found {}", v.len())))
}

#[derive(Deserialize, Clone, Debug)]
pub struct RegionClimate {
    pub name: String,
    #[serde(deserialize_with = "four")]
    pub clear: BySeason,
    #[serde(deserialize_with = "four")]
    pub overcast: BySeason,
    #[serde(deserialize_with = "four")]
    pub rain: BySeason,
    #[serde(deserialize_with = "four")]
    pub wind: BySeason,
    #[serde(deserialize_with = "four")]
    pub storm_days: BySeason,
    #[serde(deserialize_with = "four")]
    pub fog_mornings: BySeason,
    #[serde(deserialize_with = "four")]
    pub mist_mornings: BySeason,
    #[serde(deserialize_with = "four")]
    pub temp: BySeason,
    #[serde(deserialize_with = "four")]
    pub swing: BySeason,
    #[serde(deserialize_with = "four")]
    pub thunder: BySeason,
    pub fog_depth: f32,
    pub deep_fog: f32,
    pub cloud_base: (f32, f32),
    pub wind_from: f32,
    pub surf: f32,
    pub ref_height: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Borders {
    pub sea_from: f32,
    pub coast_reach: f32,
    pub upland_from: f32,
    pub blend: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Storms {
    pub build: (f32, f32),
    pub hold: (f32, f32),
    pub clear: (f32, f32),
    pub wind_leads: f32,
    pub cloud_leads: f32,
    pub strength: (f32, f32),
    pub wind: f32,
    pub chill: f32,
    pub flashes: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Fog {
    pub forms: (f32, f32),
    pub thickens_over: (f32, f32),
    pub burns_off: (f32, f32),
    pub lingers_until: (f32, f32),
    pub fades_over: (f32, f32),
    pub calm: f32,
    pub breezy: f32,
    pub after_storm: f32,
    pub stills_wind: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Sight {
    pub clear: f32,
    pub fog: f32,
    pub rain: f32,
    pub snow: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Climate {
    pub regions: Vec<RegionClimate>,
    pub borders: Borders,
    pub lapse: f32,
    pub snow_temp: f32,
    pub temp_wander: f32,
    pub cloud_hours: f32,
    pub rain_hours: f32,
    pub wind_hours: f32,
    pub shared: f32,
    pub crossing_hours: f32,
    pub spell_days: f32,
    pub unsettled: f32,
    pub wind_swing: f32,
    pub storms: Storms,
    pub fog: Fog,
    pub sight: Sight,
    pub dry_hours: f32,
    pub wetting: f32,
    pub sea_calm: f32,
    pub sea_rough: f32,
}

const TEXT: &str = include_str!("../../../data/weather/climate.ron");

/// Read the tables from text (the tests use this to check the file).
pub fn parse(text: &str) -> Result<Climate, String> {
    let c: Climate = ron::from_str(text).map_err(|e| format!("data/weather/climate.ron: {e}"))?;
    if c.regions.len() != Region::ALL.len() {
        return Err(format!("data/weather/climate.ron: {} regions, expected {}", c.regions.len(), Region::ALL.len()));
    }
    Ok(c)
}

/// The climate tables.
pub fn climate() -> &'static Climate {
    static C: OnceLock<Climate> = OnceLock::new();
    C.get_or_init(|| parse(TEXT).unwrap_or_else(|e| panic!("{e}")))
}

impl Climate {
    pub fn of(&self, r: Region) -> &RegionClimate {
        &self.regions[r as usize]
    }
}

/// Where in the year a moment falls: 0 at the spring turn, 0.25 at
/// midsummer, 0.5 at the autumn turn, 0.75 at midwinter. (The same year as
/// `tide::season`, which is the sine of this.)
pub fn year_phase(t: f64) -> f32 {
    (t / DAY / YEAR_DAYS).rem_euclid(1.0) as f32
}

/// A by-season value at a point in the year: passes through all four
/// seasons' values and slides smoothly between them, year after year.
pub fn seasonal(v: &BySeason, phase: f32) -> f32 {
    let [sp, su, au, wi] = *v;
    let a = phase * std::f32::consts::TAU;
    let mean = (sp + su + au + wi) / 4.0;
    mean + (sp - au) / 2.0 * a.cos() + (su - wi) / 2.0 * a.sin() + (sp + au - su - wi) / 4.0 * (2.0 * a).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_climate_file_loads_and_seasons_pass_through_their_values() {
        let c = parse(TEXT).expect("the climate file should load");
        assert_eq!(c.regions.len(), 6);
        let v = [1.0, 5.0, 2.0, -3.0];
        for (k, want) in v.iter().enumerate() {
            assert!((seasonal(&v, k as f32 * 0.25) - want).abs() < 1e-4);
        }
        for r in &c.regions {
            for k in 0..4 {
                assert!(r.clear[k] + r.overcast[k] < 0.95, "{}: clear and overcast leave no room for broken cloud", r.name);
            }
        }
    }
}
