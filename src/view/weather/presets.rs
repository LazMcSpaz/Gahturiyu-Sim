//! Forced weather, for looking at each kind on demand (the debug panel, the
//! U key, `GAHT_PRESET`). A preset is a sky laid over every region; the rest
//! of the drawing treats it exactly like real weather, so height still turns
//! rain to snow and fog still pools in the hollows.

use gahturiyu_sim::sim::weather::Sky;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Clear,
    Overcast,
    Drizzle,
    SeaFog,
    Downpour,
    Gale,
    Thunderstorm,
    Snow,
}

pub const PRESETS: [Preset; 8] = [Preset::Clear, Preset::Overcast, Preset::Drizzle, Preset::SeaFog, Preset::Downpour, Preset::Gale, Preset::Thunderstorm, Preset::Snow];

impl Preset {
    pub fn name(self) -> &'static str {
        match self {
            Preset::Clear => "clear",
            Preset::Overcast => "overcast",
            Preset::Drizzle => "drizzle",
            Preset::SeaFog => "sea fog",
            Preset::Downpour => "downpour",
            Preset::Gale => "gale",
            Preset::Thunderstorm => "thunderstorm",
            Preset::Snow => "snow",
        }
    }

    pub fn parse(s: &str) -> Option<Preset> {
        let s = s.to_lowercase().replace(['_', '-'], " ");
        PRESETS.into_iter().find(|p| p.name() == s || p.name().replace(' ', "") == s.replace(' ', ""))
    }

    /// The preset after this one (`None` is real weather, at both ends).
    pub fn step(now: Option<Preset>, back: bool) -> Option<Preset> {
        let n = PRESETS.len() as i32 + 1;
        let i = now.map(|p| PRESETS.iter().position(|&q| q == p).unwrap() as i32 + 1).unwrap_or(0);
        let j = (i + if back { -1 } else { 1 }).rem_euclid(n);
        (j > 0).then(|| PRESETS[j as usize - 1])
    }

    /// The sky this preset stands for, at a strength 0..1.
    pub fn sky(self, strength: f32) -> Sky {
        let s = strength.clamp(0.0, 1.0);
        // Everything blows in off the sea, from the west.
        let base = Sky { cloud_base: 900.0, temp0: 10.0, fog_depth: 35.0, gust: 0.2, ..Default::default() };
        let mut k = match self {
            Preset::Clear => Sky { cloud: 0.08, wind: 3.0, temp0: 14.0, cloud_base: 1500.0, ..base },
            Preset::Overcast => Sky { cloud: 0.75 + 0.25 * s, wind: 5.0, temp0: 9.0, cloud_base: 700.0, ..base },
            Preset::Drizzle => Sky { cloud: 0.93, precip: 0.06 + 0.14 * s, wind: 5.0, temp0: 9.0, wetness: 0.4 + 0.3 * s, cloud_base: 600.0, ..base },
            Preset::SeaFog => Sky { cloud: 0.5, fog: 0.45 + 0.55 * s, fog_depth: 25.0 + 25.0 * s, wind: 1.2, temp0: 11.0, wetness: 0.3, ..base },
            Preset::Downpour => Sky { cloud: 1.0, precip: 0.6 + 0.4 * s, wind: 8.0, temp0: 10.0, wetness: 1.0, storm: 0.3, gust: 0.4, sea: 0.5, cloud_base: 400.0, ..base },
            Preset::Gale => Sky { cloud: 0.9, precip: 0.15, wind: 17.0 + 10.0 * s, temp0: 8.0, wetness: 0.5, storm: 0.3 + 0.4 * s, gust: 0.8, sea: 0.9, cloud_base: 500.0, ..base },
            Preset::Thunderstorm => Sky { cloud: 1.0, precip: 0.7 + 0.3 * s, wind: 19.0 + 7.0 * s, temp0: 12.0, wetness: 1.0, storm: 0.7 + 0.3 * s, lightning: 4.0 * s, gust: 0.85, sea: 1.0, cloud_base: 350.0, ..base },
            Preset::Snow => Sky { cloud: 0.95, precip: 0.25 + 0.55 * s, wind: 3.0 + 6.0 * s, temp0: -3.0, wetness: 0.2, cloud_base: 500.0, ..base },
        };
        k.wind_x = k.wind;
        k.wind_y = k.wind * 0.25;
        // Snow lies where it is cold enough for it; in the snow preset, everywhere.
        k.snow_lying = if self == Preset::Snow { -200.0 } else { (k.temp0 - 1.0) / 0.0065 };
        k
    }
}
