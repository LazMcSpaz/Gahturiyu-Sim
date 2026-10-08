//! Simulation bands: how much attention each thing gets.
//!
//! Every person and group is banded by its own distance from the squad, so
//! the edge of band 1 is a smooth circle and a town is split by it person by
//! person rather than switching on and off all at once.
//!
//! - Band 1, near: everyone is an individual, with a name and gear.
//! - Band 2, middle: groups are simulated as groups, refreshed every few seconds.
//! - Band 3, far: groups are numbers on a schedule, refreshed once a game-minute.
//!
//! At 5,000 people a straight distance check per thing is far cheaper than it
//! sounds. If the population grows by orders of magnitude, a coarse grid can
//! go back in front of this to skip whole regions at once.

use serde::{Deserialize, Serialize};

use super::geo::V2;

/// Outer edge of band 1, metres from the squad.
pub const BAND1_RADIUS: f32 = 500.0;
/// Outer edge of band 2. Everything beyond is band 3.
pub const BAND2_RADIUS: f32 = 2500.0;

/// How often a group in each band has its position refreshed, in game seconds.
/// Band 1 is every step. These are the knobs that buy speed.
pub const REFRESH: [f64; 4] = [0.0, 0.0, 5.0, 60.0];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BandMap {
    centre: V2,
}

impl BandMap {
    pub fn new(squad: V2) -> BandMap {
        BandMap { centre: squad }
    }

    pub fn update(&mut self, squad: V2) {
        self.centre = squad;
    }

    pub fn band_at(&self, p: V2) -> u8 {
        let d2 = (p.x - self.centre.x).powi(2) + (p.y - self.centre.y).powi(2);
        if d2 <= BAND1_RADIUS * BAND1_RADIUS {
            1
        } else if d2 <= BAND2_RADIUS * BAND2_RADIUS {
            2
        } else {
            3
        }
    }

    /// Whether any part of a circle reaches into band 1. Lets whole towns be
    /// skipped cheaply when nobody in them could be close.
    pub fn touches_band1(&self, p: V2, radius: f32) -> bool {
        p.dist(self.centre) <= BAND1_RADIUS + radius
    }
}
