//! What the weather does to people and things out of doors: sight, hearing,
//! aim, travel, footing, cold, fire, boats, crops, work, and whether folk
//! run for cover.
//!
//! These are plain answers read off the weather at a spot and a moment
//! (`weather_effects(pos, time)`), so they are as order-free as the weather
//! itself: ask about any place and any time, in any order. Nothing in the
//! game reads them yet. `docs/weather-hooks.md` says where each belongs.
//! Every number is in `data/weather/climate.ron` (`effects`).
//!
//! They are for someone standing in the open. Under a roof none apply.

use super::climate::{climate, seasonal, year_phase};
use super::local::Weather;
use super::noise::smooth;
use super::region::{Mix, Region};
use super::weather_and_shares;
use crate::sim::geo::V2;
use crate::sim::stealth;
use crate::sim::terrain::Terrain;
use crate::sim::world::HOUR;

/// What the weather at a spot does. A multiplier of 1 means "as on a fine
/// day"; lower is worse (higher, for fire, is worse too).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effects {
    /// How far people see, as a share of the usual: fog, rain and snow
    /// close it in; a storm's gloom and a clouded night dim it further.
    /// (The hour's own light is not in here: `stealth::daylight` has that.)
    pub sight_mult: f32,
    /// How far people hear: less in rain, wind and falling snow.
    pub hearing_mult: f32,
    /// How well bows and thrown things fly true: less in wind, rain, snow.
    pub ranged_accuracy_mult: f32,
    /// Walking pace: less on soaked ground, in lying snow, and against a
    /// gale on open ground.
    pub travel_speed_mult: f32,
    /// How treacherous rock and steps are, 0 sure-footed .. 1 as bad as it
    /// gets (ice). Only means anything where the footing is stone.
    pub slip_risk: f32,
    /// Cold and wet together, 0 comfortable .. 1 bitter.
    pub exposure: f32,
    /// How readily fire spreads: near 0 on soaked ground in the rain,
    /// above 1 in a dry wind.
    pub fire_spread_mult: f32,
    /// Whether small boats would put out (`sea_danger` under a half).
    pub boats_can_sail: bool,
    /// How dangerous the water is, 0 safe .. 1 deadly: surf, wind, fog,
    /// lightning. (Ask about a spot on the shore or the water.)
    pub sea_danger: f32,
    /// How fast crops grow just now: by season, warmth, water and frost.
    pub crop_growth_mult: f32,
    /// Whether people would carry on working outside.
    pub outdoor_work_ok: bool,
    /// The chance someone with no pressing reason to be out heads indoors.
    pub shelter_seeking: f32,
    /// How cold it feels, deg C: the air, less the wind's bite and the wet.
    pub feels_like: f32,
}

impl Effects {
    /// A fine day: nothing changed.
    pub const NONE: Effects = Effects {
        sight_mult: 1.0,
        hearing_mult: 1.0,
        ranged_accuracy_mult: 1.0,
        travel_speed_mult: 1.0,
        slip_risk: 0.0,
        exposure: 0.0,
        fire_spread_mult: 1.0,
        boats_can_sail: true,
        sea_danger: 0.0,
        crop_growth_mult: 1.0,
        outdoor_work_ok: true,
        shelter_seeking: 0.0,
        feels_like: 15.0,
    };
}

impl Effects {
    /// The same with a different walking pace (for comparing answers worked
    /// out with and without the land: only the pace depends on it).
    pub fn with_travel(self, travel_speed_mult: f32) -> Effects {
        Effects { travel_speed_mult, ..self }
    }
}

/// What the weather at a spot at a moment does there.
pub fn weather_effects(terrain: &Terrain, seed: u64, pos: V2, t: f64) -> Effects {
    let (w, shares) = weather_and_shares(terrain, seed, pos, t);
    effects_mixed(&w, &shares, t)
}

/// What a given weather does at a spot that belongs to the regions in
/// these shares (`weather::mix`). How open the ground is to the wind fades
/// across region borders, like the weather itself.
pub fn effects_mixed(w: &Weather, shares: &Mix, t: f64) -> Effects {
    let open = &climate().effects.open_ground;
    let open: f32 = Region::ALL.iter().map(|r| shares[*r as usize] * open[*r as usize]).sum();
    effects_with(w, open, t)
}

/// What a given weather does, at a moment (the hour and the season count).
/// Use this when the weather is already in hand (a forecast, forced weather).
pub fn effects_of(w: &Weather, t: f64) -> Effects {
    effects_with(w, climate().effects.open_ground[w.region as usize], t)
}

fn effects_with(w: &Weather, open: f32, t: f64) -> Effects {
    let e = &climate().effects;
    let part = |x: f32| x.clamp(0.0, 1.0);

    // ---- Sight ------------------------------------------------------------------
    let day = part((stealth::daylight(t) - 0.12) / 0.88);
    let thick = (w.visibility / e.sight_full).clamp(e.sight_least, 1.0);
    let gloom = 1.0 - e.storm_gloom * part(w.storm) * day;
    let moonless = 1.0 - e.dark_night * smooth(0.5, 0.9, w.cloud) * (1.0 - day);
    let sight_mult = (thick * gloom * moonless).max(e.sight_least);

    // ---- Hearing ------------------------------------------------------------------
    let hearing_mult = ((1.0 - e.hearing_rain * part(w.rain).powf(0.7)) * (1.0 - e.hearing_wind * smooth(e.quiet_wind, e.loud_wind, w.wind)) * (1.0 - e.hearing_snow * part(w.snow))).max(e.hearing_least);

    // ---- Aim ------------------------------------------------------------------------
    let gusting = w.wind * (1.0 + 0.5 * w.gust);
    let ranged_accuracy_mult = ((1.0 - e.aim_wind * smooth(e.steady_wind, e.wild_wind, gusting)) * (1.0 - e.aim_rain * part(w.rain)) * (1.0 - e.aim_snow * part(w.snow))).max(e.aim_least);

    // ---- Travel -----------------------------------------------------------------------
    let snowed = part(w.snow_cover);
    let travel_speed_mult = (1.0 - e.mud * part(w.wetness) * (1.0 - snowed)) * (1.0 - e.deep_snow * snowed) * (1.0 - e.gale_drag * open * smooth(e.gale_from, e.gale_full, w.wind));

    // ---- Footing ------------------------------------------------------------------------
    let wet = part(w.wetness.max(w.rain));
    let frozen = smooth(1.0, -2.0, w.temperature);
    let slip_risk = part((e.slip_wet * wet).max(e.slip_ice * wet * frozen).max(e.slip_snow * snowed));

    // ---- Cold and wet ---------------------------------------------------------------------
    let feels_like = w.temperature - e.wind_chill * w.wind.min(e.chill_wind) - e.wet_chill * part(w.rain.max(0.6 * w.snow));
    let exposure = part((e.comfort - feels_like) / (e.comfort - e.bitter));

    // ---- Fire ---------------------------------------------------------------------------------
    let fire_spread_mult = ((1.0 - e.fire_damp * part(w.wetness)) * (1.0 - e.fire_rain * part(w.rain).sqrt()) * (1.0 - e.fire_snow * snowed.max(part(w.snow))) * (1.0 + (e.fire_fanned - 1.0) * (w.wind / e.fanning_wind).min(1.5))).max(e.fire_least);

    // ---- The sea ----------------------------------------------------------------------------------
    // Each danger is scaled so that the point where boats stay in is a half.
    let surf = 0.5 * w.sea / e.boats_sea;
    let blow = 0.5 * w.wind / e.boats_wind;
    // (Fog alone keeps the boats in, but a blind calm is not a deadly sea.)
    let blind = (0.5 * e.boats_sight / w.visibility.max(1.0)).min(0.7);
    let struck = 0.8 * smooth(0.0, 0.5, w.lightning);
    let sea_danger = part(surf.max(blow).max(blind).max(struck));
    let boats_can_sail = sea_danger < 0.5;

    // ---- Crops ----------------------------------------------------------------------------------------
    let season = seasonal(&e.growing, year_phase(t)).max(0.0);
    let watered = e.dry_growth + (1.0 - e.dry_growth) * smooth(0.0, 0.3, w.wetness);
    let drowning = 1.0 - (1.0 - e.drowned) * smooth(0.6, 1.0, w.rain);
    let crop_growth_mult = season * smooth(e.frost, e.mild, w.temperature) * watered * drowning * (1.0 - snowed);

    // ---- Work and shelter -----------------------------------------------------------------------------------
    let outdoor_work_ok = w.rain <= e.work_rain && w.wind <= e.work_wind && w.snow <= e.work_snow && w.temperature >= e.work_cold && w.visibility >= e.work_sight && w.lightning <= 0.05;
    let shelter_seeking = part(
        smooth(e.shrug_rain, e.drive_rain, w.rain)
            .max(smooth(e.shrug_wind, e.drive_wind, w.wind))
            .max(smooth(e.shrug_cold, e.drive_cold, w.temperature))
            .max(e.shelter_snow * smooth(0.05, 0.5, w.snow))
            .max(smooth(0.3, 0.7, w.storm))
            .max(smooth(0.0, 0.5, w.lightning)),
    );

    Effects { sight_mult, hearing_mult, ranged_accuracy_mult, travel_speed_mult, slip_risk, exposure, fire_spread_mult, boats_can_sail, sea_danger, crop_growth_mult, outdoor_work_ok, shelter_seeking, feels_like }
}

/// How well crops grew at a spot between two moments: the average of
/// `crop_growth_mult`, looked at every hour. For a day's tally at dawn.
pub fn crop_growth_over(terrain: &Terrain, seed: u64, pos: V2, from: f64, to: f64) -> f32 {
    let hours = (((to - from) / HOUR).ceil() as usize).max(1);
    let step = (to - from) / hours as f64;
    (0..hours).map(|k| weather_effects(terrain, seed, pos, from + (k as f64 + 0.5) * step).crop_growth_mult).sum::<f32>() / hours as f32
}
