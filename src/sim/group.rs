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
//!
//! A leg follows a path over the land, not a straight line. Each stretch of the
//! path costs walking time according to its slope, so the schedule already
//! knows that the climb out of the valley is slow and the road down is quick.

use serde::{Deserialize, Serialize};

use super::geo::{self, V2};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::terrain::Terrain;

pub type GroupId = u32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Leg {
    pub from: V2,
    pub to: V2,
    /// Game seconds.
    pub depart: f64,
    pub arrive: f64,
    /// The settlement at the end of this leg, if any.
    pub dest: Option<SettlementId>,
    /// The way walked, from `from` to `to`.
    pub path: Vec<V2>,
    /// Walking effort (flat-ground metres) used up by the time each point of
    /// `path` is reached. Time along the leg is proportional to this.
    pub effort: Vec<f32>,
}

impl Leg {
    /// A leg along `path`, leaving at `depart`, walked at `speed` m/s on flat
    /// ground and slower or faster as the slope demands.
    pub fn along(path: Vec<V2>, depart: f64, speed: f32, dest: Option<SettlementId>, terrain: &Terrain) -> Leg {
        let mut effort = Vec::with_capacity(path.len());
        let mut total = 0.0f32;
        effort.push(0.0);
        for w in path.windows(2) {
            total += terrain.effort(w[0], w[1]);
            effort.push(total);
        }
        Leg {
            from: path[0],
            to: *path.last().unwrap(),
            depart,
            arrive: depart + total as f64 / speed as f64,
            dest,
            path,
            effort,
        }
    }

    /// A straight walk from `a` to `b`, cut into short stretches so the
    /// slope is felt along the way.
    pub fn straight(a: V2, b: V2, depart: f64, speed: f32, dest: Option<SettlementId>, terrain: &Terrain) -> Leg {
        let n = ((a.dist(b) / 150.0).ceil() as usize).max(1);
        let path = (0..=n).map(|k| a.lerp(b, k as f32 / n as f32)).collect();
        Leg::along(path, depart, speed, dest, terrain)
    }

    /// Standing still at `p` until `until`.
    pub fn wait(p: V2, until: f64) -> Leg {
        Leg { from: p, to: p, depart: 0.0, arrive: until, dest: None, path: vec![p, p], effort: vec![0.0, 0.0] }
    }

    /// Standing still at `p` from `from` until `until`.
    pub fn stay(p: V2, from: f64, until: f64) -> Leg {
        Leg { from: p, to: p, depart: from, arrive: until, dest: None, path: vec![p, p], effort: vec![0.0, 0.0] }
    }

    /// This leg cut short at time `t`: same path and pace up to that moment,
    /// ending wherever the walker was. Returns the cut leg and how far along
    /// the path it got (index of the next point not reached).
    pub fn cut_at(&self, t: f64) -> (Leg, usize) {
        let total = *self.effort.last().unwrap_or(&0.0);
        let span = (self.arrive - self.depart).max(1e-9);
        let e = (((t - self.depart) / span) as f32 * total).clamp(0.0, total);
        let k = self.effort.partition_point(|&x| x <= e).clamp(1, self.path.len() - 1);
        let here = self.point_at(t);
        let mut path = self.path[..k].to_vec();
        let mut effort = self.effort[..k].to_vec();
        path.push(here);
        effort.push(e);
        (Leg { from: self.from, to: here, depart: self.depart, arrive: t, dest: None, path, effort }, k)
    }

    /// When the walker first comes within `r` metres of `c` on this leg, if
    /// they do. Exact: the path is straight between points and the pace is
    /// steady along each stretch, so it's a line meeting a circle.
    pub fn first_within(&self, c: V2, r: f32) -> Option<f64> {
        let total = *self.effort.last().unwrap_or(&0.0);
        if !self.arrive.is_finite() || total <= 0.0 || self.path.len() < 2 {
            return None;
        }
        let span = self.arrive - self.depart;
        for k in 1..self.path.len() {
            let (a, b) = (self.path[k - 1], self.path[k]);
            let d = b.sub(a);
            let f = a.sub(c);
            let (qa, qb, qc) = (d.x * d.x + d.y * d.y, 2.0 * (f.x * d.x + f.y * d.y), f.x * f.x + f.y * f.y - r * r);
            let s = if qc <= 0.0 {
                0.0
            } else if qa < 1e-9 {
                continue;
            } else {
                let disc = qb * qb - 4.0 * qa * qc;
                if disc < 0.0 {
                    continue;
                }
                let s = (-qb - disc.sqrt()) / (2.0 * qa);
                if !(0.0..=1.0).contains(&s) {
                    continue;
                }
                s
            };
            let e = self.effort[k - 1] + s * (self.effort[k] - self.effort[k - 1]);
            return Some(self.depart + (e / total) as f64 * span);
        }
        None
    }

    fn point_at(&self, t: f64) -> V2 {
        let span = (self.arrive - self.depart).max(1e-9);
        let total = *self.effort.last().unwrap_or(&0.0);
        if total <= 0.0 || self.path.len() < 2 {
            return self.from.lerp(self.to, ((t - self.depart) / span) as f32);
        }
        let e = (((t - self.depart) / span) as f32 * total).clamp(0.0, total);
        let k = self.effort.partition_point(|&x| x <= e).clamp(1, self.path.len() - 1);
        let (e0, e1) = (self.effort[k - 1], self.effort[k]);
        let f = if e1 > e0 { (e - e0) / (e1 - e0) } else { 0.0 };
        self.path[k - 1].lerp(self.path[k], f)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Out from home, a stay or two, and back. The schedule is complete from
    /// the moment it leaves.
    Journey { home: SettlementId },
    /// Belongs nowhere. Each next leg is drawn from its seed and leg number,
    /// so it can be written down later without changing the outcome.
    Wanderer { rest_min: f64, rest_max: f64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    /// Bandits and the like: they attack anyone who isn't one of them.
    pub hostile: bool,

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
                return leg.point_at(t);
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
    /// not matter when this gets called. Returns how many legs were added.
    pub fn extend_to(&mut self, t: f64, terrain: &Terrain) -> usize {
        let Kind::Wanderer { rest_min, rest_max } = self.kind else { return 0 };
        let mut added = 0;
        while self.legs.last().map(|l| l.arrive < t).unwrap_or(false) {
            let n = self.written;
            let last = self.legs.last().unwrap().clone();
            let mut rng = Rng::from_keys(&[self.seed, n, 0x5741_4E44]);
            let rest = rest_min + (rest_max - rest_min) * rng.f64();
            let mut to = last.to;
            for _ in 0..16 {
                let ang = rng.f32() * std::f32::consts::TAU;
                let dist = rng.range(1500.0, 5000.0);
                let cand = geo::clamp_to_world(last.to.add(V2::new(ang.cos(), ang.sin()).scale(dist)), 300.0);
                // Somewhere on dry land that isn't a mountainside.
                if geo::is_land(cand) && geo::inland(cand) > 200.0 && terrain.height(cand) < 380.0 && terrain.slope(cand) < 0.25 {
                    to = cand;
                    break;
                }
            }
            let depart = last.arrive + rest;
            self.legs.push(Leg::straight(last.to, to, depart, self.speed, None, terrain));
            self.written += 1;
            added += 1;
        }
        added
    }

    /// Forget a wanderer's legs that finished long ago. Depends only on `now`,
    /// so calling it more or less often leaves the same legs in the end.
    pub fn trim(&mut self, now: f64) {
        if !matches!(self.kind, Kind::Wanderer { .. }) {
            return;
        }
        while self.legs.len() > 2 && self.legs[1].depart <= now - 3600.0 {
            self.legs.remove(0);
        }
    }
}
