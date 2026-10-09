//! Where and when lightning strikes.
//!
//! Like the rest of the weather this is looked up, not run: the map is
//! offered one possible strike every few seconds (a keyed roll for where and
//! exactly when), and it lands if there is a thunderstorm over that spot at
//! that moment, as often as the storm's flashes a minute say. So the strikes
//! between any two moments are the same however the question is cut up.

use super::climate::climate;
use super::sky::{flashes, stormy_between, Maker, WEATHER};
use super::{lag, mix, Region};
use crate::sim::geo::{V2, WORLD_SIZE};
use crate::sim::rng::Rng;
use crate::sim::terrain::Terrain;
use crate::sim::world::HOUR;

/// Seconds between possible strikes.
pub const EVERY: f64 = 5.0;
const STRIKE: u64 = 11;

/// One lightning strike.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub t: f64,
    pub pos: V2,
    /// How big, 0.4 .. 1.
    pub power: f32,
}

/// The lightning strikes anywhere on the map from `from` up to (not
/// including) `to`, in time order. Cheap when no storm is about; in a
/// thunderstorm it costs a little per five seconds asked about, so ask
/// about hours or days, not years.
pub fn strikes(terrain: &Terrain, seed: u64, from: f64, to: f64) -> Vec<Strike> {
    let mut out = Vec::new();
    if to <= from || !stormy_between(seed, from, to) {
        return out;
    }
    // Each region's thunderstorms round these hours, found once.
    let far = (climate().crossing_hours as f64 + 1.0) * HOUR;
    let storms = Region::ALL.map(|r| Maker::new(seed, r).thunderstorms(from - far, to + far));
    if storms.iter().all(|s| s.is_empty()) {
        return out;
    }
    // No storm flashes faster than this, so most rolls can be turned away
    // before anything is worked out.
    let most = climate().storms.flashes * (EVERY / 60.0) as f32;
    let (first, last) = ((from / EVERY).floor() as i64, (to / EVERY).floor() as i64);
    for k in first..=last {
        let mut r = Rng::from_keys(&[seed, WEATHER, STRIKE, k as u64]);
        let t = (k as f64 + r.f64()) * EVERY;
        if t < from || t >= to {
            continue;
        }
        let pos = V2::new(r.f32() * WORLD_SIZE, r.f32() * WORLD_SIZE);
        let (luck, power) = (r.f32(), 0.4 + 0.6 * r.f32());
        if luck >= most {
            continue;
        }
        // Storms that reach nowhere near this spot are the common case.
        let then = t - lag(pos);
        let over = Region::ALL.map(|r| flashes(storms[r as usize].iter(), then));
        if over.iter().all(|&f| f <= 0.0) {
            continue;
        }
        let shares = mix(terrain, pos);
        let mut here = 0.0;
        for r in Region::ALL {
            let share = shares[r as usize];
            if share > 1e-4 {
                here += over[r as usize] * share;
            }
        }
        if luck < here * (EVERY / 60.0) as f32 {
            out.push(Strike { t, pos, power });
        }
    }
    out
}
