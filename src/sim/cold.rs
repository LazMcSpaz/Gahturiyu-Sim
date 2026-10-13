//! Cold and wet on the body (the items design, 0.3: weather on the body;
//! `docs/weather-hooks.md`, exposure). The weather's exposure where a squad
//! member stands (cold and wet together, after wind chill) is read at the
//! top of each hour, like the rain (`thirst.rs`), and set against what they
//! wear: each worn piece's warmth, its dryness in rain or snow, and its wind
//! cover in a wind (`items::wears`), a Warmth draught, a bedroll to sleep in and the warmth of walking. A roof, a tent or
//! a burning fire close by keeps it out altogether. What gets through
//! sets the level `Condition::cold` heads for this hour (`cold_to`), climbing
//! at `COLD_GAIN` or easing at `COLD_EASE` until it gets there. Chilled (30), breath comes back
//! slower; cold (60), attributes and pace suffer; freezing (90), they
//! collapse, knocked out, and it never kills. Stages are found by solving
//! for the moment they're crossed, and a fire going out or catching close
//! by is a moment on the timeline too.

use super::condition::Shelter;
use super::effects::Does;
use super::geo::V2;
use super::items::wears;
use super::person::PersonId;
use super::torch::FlameAt;
use super::world::{World, HOUR};

/// Cold heads for this much per unit of exposure that gets through (so
/// mild weather settles at a light chill, and only bitter exposure with poor
/// cover reaches freezing), gaining at `COLD_GAIN` an hour, shedding at
/// `COLD_EASE`.
pub const COLD_PER_EXPOSURE: f32 = 130.0;
pub const COLD_GAIN: f32 = 30.0;
pub const COLD_EASE: f32 = 25.0;
/// Exposure below this is nothing.
pub const EXPOSURE_FLOOR: f32 = 0.05;
/// How much of the exposure each kind of cover takes off: warmth always,
/// dryness in rain or snow, wind cover in a wind. All together at most
/// `COVER_MOST`.
pub const WARMTH_COVER: f32 = 0.45;
pub const DRY_COVER: f32 = 0.30;
pub const WIND_COVER: f32 = 0.20;
pub const COVER_MOST: f32 = 0.9;
/// Asleep in a bedroll, and on the move: this much more cover.
pub const BEDDING_COVER: f32 = 0.4;
pub const MOVING_COVER: f32 = 0.15;
/// A wind this strong (m/s) counts for wind cover.
pub const WINDY: f32 = 6.0;
/// A fire this close keeps the cold off.
pub const WARM_REACH: f32 = 4.0;

impl World {
    /// Is there a fire burning within reach of `p` (a campfire, a hearth, a
    /// camp's fire)? Torches don't count.
    pub fn warm_fire_at(&self, p: V2, t: f64) -> bool {
        // The squad's own fires by the clock (they catch and go out on the
        // timeline); the rest as they stand.
        self.campfires.iter().any(|f| f.pos.dist(p) <= WARM_REACH && f.burning(t)) || self.flames_near(p, WARM_REACH).iter().any(|f| matches!(f.0, FlameAt::Fire(id) if !matches!(id, super::solid::Id::SquadFire(_))) && f.2)
    }

    /// How much of the weather's exposure `pid`'s clothes (and a Warmth
    /// draught) keep out, in this weather, 0..`COVER_MOST`.
    pub fn cover_of(&self, pid: PersonId, rain: f32, snow: f32, wind: f32, t: f64) -> f32 {
        let gear = self.people[pid as usize].kit();
        let (mut warmth, mut dry, mut windproof) = (0.0, 0.0, 0.0);
        for id in gear.equipped() {
            let w = wears(id);
            warmth += w.warmth;
            dry += w.dry;
            windproof += w.wind;
        }
        warmth += self.boon_at(pid, Does::Warmth, t);
        let wet = (rain + snow).clamp(0.0, 1.0);
        let windy = if wind >= WINDY { 1.0 } else { 0.0 };
        (WARMTH_COVER * warmth + DRY_COVER * dry * wet + WIND_COVER * windproof * windy).clamp(0.0, COVER_MOST)
    }

    /// Where `pid`'s cold is heading this hour, from what gets through
    /// (0 under a roof or by a fire).
    pub(super) fn cold_target(&self, pid: PersonId, t: f64) -> f32 {
        if !self.rules.weather_on_squad {
            return 0.0;
        }
        let at = self.person_pos(pid);
        if self.shelter_of(pid) != Shelter::Open || self.warm_fire_at(at, t) {
            return 0.0;
        }
        let hour = (t / HOUR).floor() * HOUR;
        let fx = self.weather_effects(at, hour);
        let w = self.weather_at(at, hour);
        let body = match self.people[pid as usize].cond.as_ref().map(|c| c.activity) {
            Some(super::condition::Activity::Sleeping) => BEDDING_COVER,
            Some(super::condition::Activity::Walking) => MOVING_COVER,
            _ => 0.0,
        };
        let cover = (self.cover_of(pid, w.rain, w.snow, w.wind, t) + body).min(COVER_MOST);
        let e = fx.exposure * (1.0 - cover);
        if e <= EXPOSURE_FLOOR {
            0.0
        } else {
            (COLD_PER_EXPOSURE * e).min(100.0)
        }
    }

    /// When a campfire close to `pid` catches or goes out (the cold's rate
    /// changes then).
    pub(super) fn fire_turns_near(&self, pid: PersonId) -> Vec<f64> {
        let at = self.person_pos(pid);
        self.campfires.iter().filter(|f| f.pos.dist(at) <= WARM_REACH).flat_map(|f| [f.lit_at, f.out_at]).collect()
    }
}

impl World {
    /// Cold now (0..100), for a squad member.
    pub fn cold_of(&self, pid: PersonId) -> Option<f32> {
        self.people[pid as usize].cond.as_ref().map(|c| c.cold_at(self.time))
    }
}
