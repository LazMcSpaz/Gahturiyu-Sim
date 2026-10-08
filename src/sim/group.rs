//! Groups on the move, and the schedules that move them.
//!
//! A group does not walk step by step. When it sets out, its whole journey is
//! written down as a schedule: leave here at this time, arrive there at that
//! time, stay until this time, head home. Where the group is at any moment is
//! then just "look up the time on the schedule".
//!
//! This is what makes the bands safe. A group far away might only be looked at
//! once a game-minute, a group beside you sixty times a second, and both land in
//! exactly the same place, because neither is accumulating its own steps. It is
//! the same rule as the project's NPC routines: read the clock, set the state.

use super::geo::{self, V2};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;

pub type GroupId = u32;

#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub from: V2,
    pub to: V2,
    /// Game seconds.
    pub depart: f64,
    pub arrive: f64,
    /// The settlement at the end of this leg, if any.
    pub dest: Option<SettlementId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// Out from home, a stay or two, and back. The schedule is complete from
    /// the moment it leaves.
    Journey { home: SettlementId },
    /// Belongs nowhere. Each next leg is drawn from its seed and leg number,
    /// so it can be written down later without changing the outcome.
    Wanderer { rest_min: f64, rest_max: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub seed: u64,
    pub members: Vec<PersonId>,
    pub kind: Kind,
    pub legs: Vec<Leg>,
    /// Metres per second: the slowest member's pace.
    pub speed: f32,
    /// When the journey is over and the group dissolves. Infinite for wanderers.
    pub ends: f64,
    /// How many legs have ever been written (old wanderer legs get dropped,
    /// but the count keeps each new leg's random draw stable).
    pub written: u64,

    // --- Cached view, refreshed at the group's band rate ------------------
    pub pos: V2,
    pub last_update: f64,
    pub band: u8,
}

impl Group {
    /// Where the group is at time `t`, read straight off the schedule.
    pub fn position_at(&self, t: f64) -> V2 {
        let first = &self.legs[0];
        if t <= first.depart {
            return first.from;
        }
        // Legs are few; a linear scan from the end is fine and simple.
        for leg in self.legs.iter().rev() {
            if t >= leg.depart {
                if t >= leg.arrive {
                    return leg.to;
                }
                let span = (leg.arrive - leg.depart).max(1e-9);
                return leg.from.lerp(leg.to, ((t - leg.depart) / span) as f32);
            }
        }
        first.from
    }

    /// The leg in progress (or the stay after it) at time `t`.
    pub fn current_leg(&self, t: f64) -> Option<&Leg> {
        self.legs.iter().rev().find(|l| t >= l.depart).or(self.legs.first())
    }

    pub fn is_moving(&self, t: f64) -> bool {
        self.current_leg(t).map(|l| t >= l.depart && t < l.arrive).unwrap_or(false)
    }

    /// Wanderers write their schedule a leg at a time. Make sure it reaches at
    /// least time `t`. Purely a function of the seed and leg number, so it does
    /// not matter when this gets called.
    pub fn extend_to(&mut self, t: f64) {
        let Kind::Wanderer { rest_min, rest_max } = self.kind else { return };
        while self.legs.last().map(|l| l.arrive < t).unwrap_or(false) {
            let n = self.written;
            let last = self.legs.last().unwrap().clone();
            let mut rng = Rng::from_keys(&[self.seed, n, 0x5741_4E44]);
            let rest = rest_min + (rest_max - rest_min) * rng.f64();
            let mut to = last.to;
            for _ in 0..12 {
                let ang = rng.f32() * std::f32::consts::TAU;
                let dist = rng.range(1500.0, 5000.0);
                let cand = geo::clamp_to_world(last.to.add(V2::new(ang.cos(), ang.sin()).scale(dist)), 300.0);
                if geo::is_land(cand) && geo::inland(cand) > 200.0 {
                    to = cand;
                    break;
                }
            }
            let depart = last.arrive + rest;
            let arrive = depart + last.to.dist(to) as f64 / self.speed as f64;
            self.legs.push(Leg { from: last.to, to, depart, arrive, dest: None });
            self.written += 1;
            if self.legs.len() > 3 {
                self.legs.remove(0);
            }
        }
    }
}
