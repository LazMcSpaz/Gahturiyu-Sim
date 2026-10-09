//! Omens: weather unusual enough that people might read something into it.
//!
//! Like the rest of the weather these are looked up, not noticed as they
//! happen: `omens(region, day)` works out from the seed whether that day
//! holds any, for any day past or to come. Nothing is posted anywhere and
//! nothing reads them yet (see `docs/weather-hooks.md`).
//!
//! Each omen is real weather: on a day with "a dead calm" the wind that
//! `weather_at` gives really does drop to nothing. What counts as each, and
//! how rare each is meant to be, is in `data/weather/climate.ron` (`omens`).

use super::climate::{climate, year_phase, Rarity};
use super::lightning::strikes;
use super::local::localise;
use super::region::{region_at, Region};
use super::sky::Maker;
use crate::sim::geo::V2;
use crate::sim::terrain::Terrain;
use crate::sim::world::{DAY, HOUR};

/// The kinds of omen there are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OmenKind {
    WinterThunder,
    NoonFog,
    ShoreSnow,
    StormRun,
    DeadCalm,
    LandmarkStrike,
}

impl OmenKind {
    pub const ALL: [OmenKind; 6] = [OmenKind::WinterThunder, OmenKind::NoonFog, OmenKind::ShoreSnow, OmenKind::StormRun, OmenKind::DeadCalm, OmenKind::LandmarkStrike];

    pub fn name(self) -> &'static str {
        match self {
            OmenKind::WinterThunder => "thunder in midwinter",
            OmenKind::NoonFog => "fog at noon in high summer",
            OmenKind::ShoreSnow => "snow down to the shore",
            OmenKind::StormRun => "three storm days in a row",
            OmenKind::DeadCalm => "a dead calm on the coast",
            OmenKind::LandmarkStrike => "lightning on a landmark",
        }
    }

    /// How rare it is meant to be (from the data file).
    pub fn rarity(self) -> Rarity {
        let o = &climate().omens;
        match self {
            OmenKind::WinterThunder => o.winter_thunder,
            OmenKind::NoonFog => o.noon_fog,
            OmenKind::ShoreSnow => o.shore_snow,
            OmenKind::StormRun => o.storm_run,
            OmenKind::DeadCalm => o.dead_calm,
            OmenKind::LandmarkStrike => o.landmark_strike,
        }
    }

    /// Times a year it should come, fewest and most. For the omens of the
    /// sky: in one region where it can happen. For lightning on a landmark:
    /// anywhere in the country.
    pub fn band(self) -> (f32, f32) {
        climate().omens.band(self.rarity())
    }

    /// Whether this omen can come in a region at all.
    pub fn happens_in(self, region: Region) -> bool {
        match self {
            OmenKind::ShoreSnow => matches!(region, Region::OpenSea | Region::ExposedCoast | Region::Lowland),
            OmenKind::DeadCalm => region.maritime(),
            _ => true,
        }
    }
}

/// One omen: unusual weather on one day.
#[derive(Clone, Debug, PartialEq)]
pub struct Omen {
    pub kind: OmenKind,
    pub rarity: Rarity,
    /// The region it was seen in.
    pub region: Region,
    /// When it shows itself, game seconds (within the day asked about).
    pub at: f64,
    /// Where, for one that happens at a spot (a lightning strike).
    pub pos: Option<V2>,
    /// Which landmark, for lightning on one: its place in the list given.
    pub landmark: Option<usize>,
    /// A plain line saying what happened.
    pub text: String,
}

/// Somewhere with a name that lightning might strike.
#[derive(Clone, Copy, Debug)]
pub struct Landmark<'a> {
    pub name: &'a str,
    pub pos: V2,
}

/// The omens of the sky over a region on a day (day 0 starts at time 0), as
/// the weather comes in off the sea. Lightning on landmarks is separate
/// (`strike_omens`): it needs to know the land.
pub fn omens(seed: u64, region: Region, day: i64) -> Vec<Omen> {
    omens_lagging(seed, region, day, 0.0)
}

/// The same for a place the weather reaches `lag` seconds later
/// (`weather::lag(pos)`): the day is the place's own, midnight to midnight.
pub fn omens_lagging(seed: u64, region: Region, day: i64, lag: f64) -> Vec<Omen> {
    let m = Maker::lagging(seed, region, lag);
    let o = &climate().omens;
    let mut out = Vec::new();
    let start = day as f64 * DAY;
    let phase = year_phase(start + DAY / 2.0);
    let year = crate::sim::tide::YEAR_DAYS as f32;
    // Days from a point in the year, the short way round.
    let days_from = |when: f32| {
        let d = (phase - when).rem_euclid(1.0);
        d.min(1.0 - d) * year
    };
    let mut add = |kind: OmenKind, at: f64, text: String| out.push(Omen { kind, rarity: kind.rarity(), region, at, pos: None, landmark: None, text });
    let place = match region {
        Region::OpenSea => "out at sea",
        Region::ExposedCoast => "along the open coast",
        Region::Lowland => "over the low country",
        Region::Upland => "over the uplands",
        Region::Mountain => "in the mountains",
        Region::Plateau => "over the plateau",
    };

    // ---- Thunder in midwinter ------------------------------------------------
    if days_from(0.75) <= o.midwinter_days {
        if let Some(t) = thunder_begins(&m, day) {
            add(OmenKind::WinterThunder, t, format!("Thunder {place} in the dead of winter."));
        }
    }

    // ---- Fog at noon in high summer ----------------------------------------------
    if days_from(0.25) <= o.high_summer_days {
        let noon = start + 12.0 * HOUR;
        if m.sky(noon).fog >= o.noon_fog_thick {
            add(OmenKind::NoonFog, noon, format!("Fog still lying {place} at noon, at the height of summer."));
        }
    }

    // ---- Snow down to the shore -----------------------------------------------------
    if OmenKind::ShoreSnow.happens_in(region) {
        if let Some(t) = shore_snow(&m, region, day) {
            if shore_snow(&m, region, day - 1).is_none() {
                add(OmenKind::ShoreSnow, t, "Snow fell right down to the shore.".to_string());
            }
        }
    }

    // ---- Storm days in a row -----------------------------------------------------------
    let run = o.storm_run_days.max(1) as i64;
    if let Some(t) = m.storm_day(day) {
        if (1..run).all(|back| m.storm_day(day - back).is_some()) && m.storm_day(day - run).is_none() {
            add(OmenKind::StormRun, t, format!("{} days of storm {place}, one after another.", count_word(run)));
        }
    }

    // ---- A dead calm ------------------------------------------------------------------------
    if OmenKind::DeadCalm.happens_in(region) {
        if let Some(t) = dead_calm(&m, day) {
            let text = if region == Region::OpenSea { "A dead calm at sea: not a breath of wind for hours." } else { "A dead calm on the open coast: not a breath of wind for hours." };
            add(OmenKind::DeadCalm, t, text.to_string());
        }
    }

    out.sort_by(|a, b| a.at.total_cmp(&b.at));
    out
}

fn count_word(n: i64) -> String {
    match n {
        2 => "Two".to_string(),
        3 => "Three".to_string(),
        4 => "Four".to_string(),
        5 => "Five".to_string(),
        n => n.to_string(),
    }
}

/// When a thunderstorm's thunder is first heard, if that falls on this day.
/// (Once a storm: a storm that runs past midnight is not counted again.)
fn thunder_begins(m: &Maker, day: i64) -> Option<f64> {
    let (from, to) = (day as f64 * DAY, (day + 1) as f64 * DAY);
    m.thunderstorms(from, to)
        .iter()
        // Thunder carries once the storm is a good part built.
        .map(|s| s.start + m.lag() + 0.45 * s.build as f64 * HOUR)
        .filter(|&t| t >= from && t < to)
        .min_by(|a, b| a.total_cmp(b))
}

/// The first moment of a day that snow is falling at sea level.
fn shore_snow(m: &Maker, region: Region, day: i64) -> Option<f64> {
    let c = climate();
    let start = day as f64 * DAY;
    // Most of the year it cannot be cold enough whatever the weather does.
    if m.warmth(start + DAY / 2.0) - c.temp_wander - c.storms.chill - 6.0 > c.snow_temp + 1.0 {
        return None;
    }
    // (It may run on past midnight: look a few hours into the next day.)
    let need = (c.omens.shore_snow_hours / 0.5).ceil() as usize;
    let (mut run, mut began) = (0usize, 0.0f64);
    for k in 0..48 + need {
        let t = start + k as f64 * 0.5 * HOUR;
        if localise(&m.sky(t), region, 0.0, 0.0).snow >= c.omens.shore_snow_falls {
            if run == 0 {
                if k >= 48 {
                    return None;
                }
                began = t;
            }
            run += 1;
            if run > need {
                return Some(began);
            }
        } else {
            run = 0;
        }
    }
    None
}

/// When the wind first drops to nothing for hours together, by day.
fn dead_calm(m: &Maker, day: i64) -> Option<f64> {
    let o = &climate().omens;
    let start = day as f64 * DAY;
    // No lull about, no calm.
    if m.lull_at(start + 9.0 * HOUR) > 0.7 && m.lull_at(start + 13.0 * HOUR) > 0.7 && m.lull_at(start + 17.0 * HOUR) > 0.7 {
        return None;
    }
    let step = 0.5;
    let need = (o.calm_hours / step).ceil() as usize;
    let (mut run, mut began) = (0usize, 0.0f64);
    let mut k = 0;
    while 6.0 + k as f32 * step <= 20.0 {
        let t = start + (6.0 + k as f32 * step) as f64 * HOUR;
        if m.true_wind(t) < o.calm_wind {
            if run == 0 {
                began = t;
            }
            run += 1;
            // (`need` half-hours apart span `need - 1` steps; one more makes the hours.)
            if run > need {
                return Some(began);
            }
        } else {
            run = 0;
        }
        k += 1;
    }
    None
}

/// Lightning striking landmarks on a day: at most one omen a landmark.
pub fn strike_omens(terrain: &Terrain, seed: u64, landmarks: &[Landmark], day: i64) -> Vec<Omen> {
    let o = &climate().omens;
    let mut out: Vec<Omen> = Vec::new();
    if landmarks.is_empty() {
        return out;
    }
    for s in strikes(terrain, seed, day as f64 * DAY, (day + 1) as f64 * DAY) {
        if s.power < o.strike_power {
            continue;
        }
        let mut best: Option<(usize, f32)> = None;
        for (k, l) in landmarks.iter().enumerate() {
            let d = l.pos.dist(s.pos);
            if d <= o.strike_reach && best.map(|(_, b)| d < b).unwrap_or(true) {
                best = Some((k, d));
            }
        }
        let Some((k, _)) = best else { continue };
        if out.iter().any(|x| x.landmark == Some(k)) {
            continue;
        }
        let kind = OmenKind::LandmarkStrike;
        out.push(Omen { kind, rarity: kind.rarity(), region: region_at(terrain, landmarks[k].pos), at: s.t, pos: Some(s.pos), landmark: Some(k), text: format!("Lightning struck {}.", landmarks[k].name) });
    }
    out
}

/// How often each omen comes: times a year, over `years` years from year
/// `first`. For the omens of the sky, the count in each region (0 where it
/// cannot happen); `OmenKind::LandmarkStrike` is left at 0 here (it needs
/// the land: use `strike_rate`).
pub fn rates(seed: u64, first: i64, years: i64) -> Vec<(OmenKind, [f32; super::REGIONS])> {
    let per_year = crate::sim::tide::YEAR_DAYS as i64;
    let mut n = [[0.0f32; super::REGIONS]; 6];
    for r in Region::ALL {
        for day in first * per_year..(first + years) * per_year {
            for o in omens(seed, r, day) {
                n[OmenKind::ALL.iter().position(|k| *k == o.kind).unwrap()][r as usize] += 1.0;
            }
        }
    }
    OmenKind::ALL.iter().enumerate().map(|(k, kind)| (*kind, n[k].map(|v| v / years as f32))).collect()
}

/// How often lightning strikes a landmark, anywhere: times a year.
pub fn strike_rate(terrain: &Terrain, seed: u64, landmarks: &[Landmark], first: i64, years: i64) -> f32 {
    let per_year = crate::sim::tide::YEAR_DAYS as i64;
    let mut n = 0usize;
    for day in first * per_year..(first + years) * per_year {
        n += strike_omens(terrain, seed, landmarks, day).len();
    }
    n as f32 / years as f32
}

/// The rate of an omen in a typical region: the average over the regions
/// it can happen in.
pub fn typical(kind: OmenKind, by_region: &[f32; super::REGIONS]) -> f32 {
    let (mut sum, mut n) = (0.0, 0.0);
    for r in Region::ALL {
        if kind.happens_in(r) {
            sum += by_region[r as usize];
            n += 1.0;
        }
    }
    sum / f32::max(n, 1.0)
}
