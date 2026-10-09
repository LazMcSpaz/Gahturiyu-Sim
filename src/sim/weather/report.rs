//! A year's weather added up, region by region: how grey, how wet, how many
//! storm days and fog mornings. Used by the tests (the climate must come out
//! as the brief asks) and by `headless weather`, so the tables can be tuned
//! against what they actually produce.

use super::{weather_in, Kind, Region, GALE, THICK_FOG};
use crate::sim::tide::YEAR_DAYS;
use crate::sim::world::{DAY, HOUR};

/// A month, for "days a month" figures. (The year itself is a placeholder
/// 48 days, so this is simply 30 days.)
pub const MONTH: f32 = 30.0;

/// What a stretch of days was like at a region's typical spot.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tally {
    pub days: f32,
    /// Shares of the daylight hours (07:00 to 19:00).
    pub clear: f32,
    pub broken: f32,
    /// Grey sky, or rain or snow falling.
    pub grey: f32,
    pub raining: f32,
    pub snowing: f32,
    /// Days a month with a storm (a gale or worse, or a storm at half strength).
    pub storm_days: f32,
    pub thunder_days: f32,
    /// Mornings a month with sight under 100 m at some point between 04:00 and 10:00.
    pub fog_mornings: f32,
    /// Mornings a month with fog or mist of any kind.
    pub misty_mornings: f32,
    /// Days a month with snow falling at some point.
    pub snow_days: f32,
    pub wind: f32,
    pub temperature: f32,
    pub coldest: f32,
    pub warmest: f32,
}

/// Add up `days` days from day `first`, keeping only days whose season is
/// `season` (0 spring, 1 summer, 2 autumn, 3 winter), or all if `None`.
pub fn tally(seed: u64, region: Region, first: i64, days: i64, season: Option<usize>) -> Tally {
    let mut t = Tally { coldest: f32::MAX, warmest: f32::MIN, ..Default::default() };
    let (mut light, mut all) = (0.0f32, 0.0f32);
    for day in first..first + days {
        if let Some(s) = season {
            if season_of(day) != s {
                continue;
            }
        }
        t.days += 1.0;
        let (mut storm, mut thunder, mut fog, mut misty, mut snow) = (false, false, false, false, false);
        // Every 20 minutes.
        for k in 0..72 {
            let hour = k as f32 / 3.0;
            let w = weather_in(seed, region, day as f64 * DAY + hour as f64 * HOUR);
            all += 1.0;
            t.wind += w.wind;
            t.temperature += w.temperature;
            t.coldest = t.coldest.min(w.temperature);
            t.warmest = t.warmest.max(w.temperature);
            storm |= w.storm >= 0.5 || w.wind >= GALE;
            thunder |= w.kind == Kind::Thunderstorm;
            snow |= w.snow > 0.03;
            if (4.0..10.0).contains(&hour) {
                fog |= w.visibility < THICK_FOG && w.fog > 0.3;
                misty |= w.fog > 0.3;
            }
            if (7.0..19.0).contains(&hour) {
                light += 1.0;
                let wet = w.rain > 0.03 || w.snow > 0.03;
                if wet {
                    t.raining += if w.rain >= w.snow { 1.0 } else { 0.0 };
                    t.snowing += if w.snow > w.rain { 1.0 } else { 0.0 };
                }
                if w.cloud >= 0.7 || wet {
                    t.grey += 1.0;
                } else if w.cloud < 0.25 && w.fog < 0.3 {
                    t.clear += 1.0;
                } else {
                    t.broken += 1.0;
                }
            }
        }
        t.storm_days += storm as u8 as f32;
        t.thunder_days += thunder as u8 as f32;
        t.fog_mornings += fog as u8 as f32;
        t.misty_mornings += misty as u8 as f32;
        t.snow_days += snow as u8 as f32;
    }
    if t.days == 0.0 {
        return t;
    }
    for v in [&mut t.clear, &mut t.broken, &mut t.grey, &mut t.raining, &mut t.snowing] {
        *v /= light;
    }
    for v in [&mut t.storm_days, &mut t.thunder_days, &mut t.fog_mornings, &mut t.misty_mornings, &mut t.snow_days] {
        *v *= MONTH / t.days;
    }
    t.wind /= all;
    t.temperature /= all;
    t
}

/// Which season a day falls in: 0 spring, 1 summer, 2 autumn, 3 winter.
pub fn season_of(day: i64) -> usize {
    let phase = ((day as f64 + 0.5) / YEAR_DAYS).rem_euclid(1.0);
    ((phase + 0.125) * 4.0).floor() as usize % 4
}

pub const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];

/// Each region's weather over `years` years, as a table to read.
pub fn yearly(seed: u64, years: i64) -> String {
    use std::fmt::Write;
    let days = (years as f64 * YEAR_DAYS) as i64;
    let mut out = String::new();
    let _ = writeln!(out, "Weather over {years} years ({days} days), world {seed}. Shares are of daylight hours; days and mornings are per 30 days.");
    for r in Region::ALL {
        let _ = writeln!(out, "\n{}", r.name());
        let _ = writeln!(out, "  {:<8} {:>6} {:>7} {:>6} {:>6} {:>6} {:>7} {:>8} {:>6} {:>6} {:>6} {:>6} {:>12}", "", "clear", "broken", "grey", "rain", "snow", "storms", "thunder", "fog", "misty", "snowy", "wind", "temp (range)");
        for (name, season) in [("year", None), (SEASONS[0], Some(0)), (SEASONS[1], Some(1)), (SEASONS[2], Some(2)), (SEASONS[3], Some(3))] {
            let t = tally(seed, r, 0, days, season);
            let _ = writeln!(
                out,
                "  {:<8} {:>5.0}% {:>6.0}% {:>5.0}% {:>5.0}% {:>5.0}% {:>7.1} {:>8.1} {:>6.1} {:>6.1} {:>6.1} {:>6.1} {:>5.1} ({:.0}..{:.0})",
                name,
                t.clear * 100.0,
                t.broken * 100.0,
                t.grey * 100.0,
                t.raining * 100.0,
                t.snowing * 100.0,
                t.storm_days,
                t.thunder_days,
                t.fog_mornings,
                t.misty_mornings,
                t.snow_days,
                t.wind,
                t.temperature,
                t.coldest,
                t.warmest
            );
        }
    }
    out
}

/// How much of the map each region covers, and how high its ground runs.
pub fn regions(terrain: &crate::sim::terrain::Terrain) -> String {
    use crate::sim::geo::{V2, WORLD_SIZE};
    use std::fmt::Write;
    let mut n = [0u32; super::REGIONS];
    let mut lo = [f32::MAX; super::REGIONS];
    let mut hi = [f32::MIN; super::REGIONS];
    let mut sum = [0.0f64; super::REGIONS];
    let step = 150.0;
    let side = (WORLD_SIZE / step) as usize;
    for j in 0..side {
        for i in 0..side {
            let p = V2::new((i as f32 + 0.5) * step, (j as f32 + 0.5) * step);
            let r = super::region_at(terrain, p) as usize;
            let h = terrain.surface(p);
            n[r] += 1;
            lo[r] = lo[r].min(h);
            hi[r] = hi[r].max(h);
            sum[r] += h as f64;
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "Climate regions (share of the map; ground height lowest / usual / highest, metres)");
    for r in Region::ALL {
        let k = r as usize;
        let _ = writeln!(out, "  {:<28} {:>5.1}%   {:>4.0} / {:>4.0} / {:>4.0}", r.name(), n[k] as f32 * 100.0 / (side * side) as f32, lo[k].min(hi[k]), sum[k] / n[k].max(1) as f64, hi[k]);
    }
    out
}

/// How often each omen comes, against the band it is meant to keep to.
pub fn omen_table(terrain: &crate::sim::terrain::Terrain, landmarks: &[super::Landmark], seed: u64, years: i64) -> String {
    use super::{omen_rates as rates, strike_rate, typical_rate as typical};
    use super::OmenKind;
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "\nOmens over {years} years, world {seed}: times a year (a typical region; then each region)");
    let _ = writeln!(out, "  {:<28} {:<10} {:>11} {:>8}   sea  coast    low     up  mount plateau", "", "rarity", "band", "typical");
    for (kind, by) in rates(seed, 0, years) {
        let (lo, hi) = kind.band();
        let (rate, each) = if kind == OmenKind::LandmarkStrike {
            (strike_rate(terrain, seed, landmarks, 0, years), "  (whole country)".to_string())
        } else {
            (typical(kind, &by), Region::ALL.iter().map(|r| if kind.happens_in(*r) { format!("{:>6.2}", by[*r as usize]) } else { "     -".to_string() }).collect::<Vec<_>>().join(" "))
        };
        let mark = if rate < lo || rate > hi { "  OUT OF BAND" } else { "" };
        let _ = writeln!(out, "  {:<28} {:<10} {:>4.2}..{:<5.2} {:>8.2}  {each}{mark}", kind.name(), kind.rarity().name(), lo, hi, rate);
    }
    // The first of each, so one can be gone and looked at.
    let _ = writeln!(out, "\nThe first of each (day counted from 0; hour of the day):");
    for kind in OmenKind::ALL {
        let found = (0..years * YEAR_DAYS as i64).find_map(|day| {
            if kind == OmenKind::LandmarkStrike {
                super::strike_omens(terrain, seed, landmarks, day).into_iter().next()
            } else {
                Region::ALL.iter().find_map(|r| super::omens(seed, *r, day).into_iter().find(|o| o.kind == kind))
            }
        });
        match found {
            Some(o) => {
                let _ = writeln!(out, "  day {:>4}, {:>5.2} h, {:<28} {}", (o.at / DAY).floor(), o.at.rem_euclid(DAY) / HOUR, o.region.name(), o.text);
            }
            None => {
                let _ = writeln!(out, "  {}: none in {years} years", kind.name());
            }
        }
    }
    out
}
