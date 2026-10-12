//! What the squad sleeps on out of doors (Laz, 11 Oct): nobody sleeps on the
//! bare ground. A bedroll is laid out where it's wanted (a minute) and stays
//! there, an object, until someone packs it up; a tent is pitched the same
//! way (ten minutes; it holds `IN_TENT`). Sleeping is lying down in one, or
//! in a bed or bedroll indoors (`layout`'s furniture), never just stopping.
//!
//! Orders are walk-then-do, like picking things up: `order_lay`,
//! `order_pack` and `order_sleep_in` send someone, and `do_bed_jobs` (the
//! squad's own step: the player exception, like walking) does it when they
//! get there. The beddings and the jobs waiting are saved.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items::{self, ItemId};
use super::layout::Furn;
use super::person::PersonId;
use super::world::World;

/// How long laying out a bedroll and pitching a tent take, game seconds. A
/// bedroll is down at once (so lying down in it starts the moment it's laid,
/// whatever the step: rule 8); the window unrolls it over `UNROLL_SECS`.
pub const LAY_SECS: f64 = 0.0;
pub const UNROLL_SECS: f64 = 4.0;
pub const PITCH_SECS: f64 = 600.0;
/// How many can sleep in one tent.
pub const IN_TENT: usize = 3;
/// How near a bedding (or a bed indoors) counts as lying in it, metres.
pub const BED_REACH: f32 = 1.5;
/// A tent's radius (the solid list's `TENT`), metres.
pub const TENT_REACH: f32 = 2.2;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedKind {
    Bedroll,
    Tent,
}

impl BedKind {
    pub fn item(self) -> ItemId {
        items::id(match self {
            BedKind::Bedroll => "bedroll",
            BedKind::Tent => "tent",
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            BedKind::Bedroll => "bedroll",
            BedKind::Tent => "tent",
        }
    }
    fn secs(self) -> f64 {
        match self {
            BedKind::Bedroll => LAY_SECS,
            BedKind::Tent => PITCH_SECS,
        }
    }
}

/// A bedroll laid out or a tent pitched, out in the world.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Bedding {
    pub id: u32,
    pub kind: BedKind,
    pub pos: V2,
    /// Which way it lies (radians).
    pub rot: f32,
    /// Laid out (or pitched) from this moment on; up and usable from `ready_at`.
    pub since: f64,
    pub ready_at: f64,
}

impl Bedding {
    pub fn ready(&self, t: f64) -> bool {
        t >= self.ready_at
    }
    /// How far up it is, 0..1 (a tent rising as it's pitched).
    pub fn up(&self, t: f64) -> f32 {
        let span = (self.ready_at - self.since).max(if self.kind == BedKind::Bedroll { UNROLL_SECS } else { 0.0 });
        if span <= 0.0 {
            return 1.0;
        }
        (((t - self.since) / span) as f32).clamp(0.0, 1.0)
    }
}

/// A squad member on their way to lay out, pack up, or (once laid) sleep.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum BedJob {
    /// Lay out (or pitch) `kind` at `at`; then lie down in it if `sleep`.
    Lay { who: PersonId, kind: BedKind, at: V2, sleep: bool },
    Pack { who: PersonId, bed: u32 },
    /// Lie down in this bedding once it's up.
    SleepWhenUp { who: PersonId, bed: u32 },
}

impl BedJob {
    fn who(&self) -> PersonId {
        match *self {
            BedJob::Lay { who, .. } | BedJob::Pack { who, .. } | BedJob::SleepWhenUp { who, .. } => who,
        }
    }
}

impl World {
    pub fn bedding(&self, id: u32) -> Option<&Bedding> {
        self.beds.iter().find(|b| b.id == id)
    }

    fn carries(&self, who: PersonId, it: ItemId) -> bool {
        self.people[who as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == it))
    }

    /// Can `who` lay out (or pitch) one of these? (They carry it.)
    pub fn can_lay(&self, who: PersonId, kind: BedKind) -> bool {
        self.squad.index(who).is_some() && self.carries(who, kind.item())
    }

    /// Send someone to lay out their bedroll (or pitch their tent) at `at`;
    /// with `sleep`, they lie down in it once it's laid.
    pub fn order_lay(&mut self, who: PersonId, kind: BedKind, at: V2, sleep: bool) -> Result<(), &'static str> {
        if !self.can_lay(who, kind) {
            return Err(match kind {
                BedKind::Bedroll => "has no bedroll",
                BedKind::Tent => "has no tent",
            });
        }
        if !self.free_to_order(who) || self.fighting.contains_key(&who) {
            return Err("can't now");
        }
        // At their feet: no walk (a walk's end falls on a step, and lying
        // down must start at the moment it's ordered: rule 8).
        let k = self.squad.index(who).unwrap();
        if self.squad.at[k].dist(at) <= 1.0 {
            self.squad.goal[k] = self.squad.at[k];
            self.squad.route[k].clear();
        } else {
            self.order_members(&[who], at);
        }
        self.bed_jobs.retain(|j| j.who() != who);
        self.bed_jobs.push(BedJob::Lay { who, kind, at, sleep });
        self.do_bed_jobs();
        Ok(())
    }

    /// Send someone to pack up a bedroll or tent (it goes in their pack).
    pub fn order_pack(&mut self, who: PersonId, bed: u32) -> Result<(), &'static str> {
        let Some(b) = self.bedding(bed).copied() else { return Err("nothing there") };
        if self.sleepers_in(&b).next().is_some() {
            return Err("someone is sleeping in it");
        }
        if self.squad.index(who).is_none() || !self.free_to_order(who) {
            return Err("can't now");
        }
        self.order_members(&[who], b.pos);
        self.bed_jobs.retain(|j| j.who() != who);
        self.bed_jobs.push(BedJob::Pack { who, bed });
        self.do_bed_jobs();
        Ok(())
    }

    /// Send someone to lie down in a bedroll or tent.
    pub fn order_sleep_in(&mut self, who: PersonId, bed: u32) -> Result<(), &'static str> {
        let Some(b) = self.bedding(bed).copied() else { return Err("nothing there") };
        if !b.ready(self.time) {
            return Err(if b.kind == BedKind::Tent { "the tent isn't up yet" } else { "it isn't laid out yet" });
        }
        let room = match b.kind {
            BedKind::Bedroll => 1,
            BedKind::Tent => IN_TENT,
        };
        if self.sleepers_in(&b).filter(|&p| p != who).count() >= room {
            return Err(if b.kind == BedKind::Tent { "the tent is full" } else { "someone is sleeping in it" });
        }
        self.order_sleep_at(who, b.pos);
        Ok(())
    }

    /// Who of the squad is asleep (or bedding down) in this bedding.
    fn sleepers_in<'a>(&'a self, b: &'a Bedding) -> impl Iterator<Item = PersonId> + 'a {
        let reach = if b.kind == BedKind::Tent { TENT_REACH } else { BED_REACH };
        // (Those on their way to it count: they've a place in it.)
        self.squad.members.iter().enumerate().filter(move |(k, _)| self.squad.resting[*k] && (self.squad.at[*k].dist(b.pos) <= reach || self.squad.goal[*k].dist(b.pos) <= reach)).map(|(_, &m)| m)
    }

    /// The bedding someone at `at` would be lying in (ready), if any.
    pub fn bedding_at(&self, at: V2) -> Option<&Bedding> {
        let t = self.time;
        self.beds
            .iter()
            .filter(|b| b.ready(t) && b.pos.dist(at) <= if b.kind == BedKind::Tent { TENT_REACH } else { BED_REACH })
            .min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at)))
    }

    /// Is `at` somewhere to sleep: a bedroll laid out, a tent up, or a bed
    /// or bedroll in a building?
    pub fn sleep_place_at(&self, at: V2) -> bool {
        if self.bedding_at(at).is_some() {
            return true;
        }
        // A bed paid for at an inn (`care.rs`).
        if self.rooms.iter().any(|r| r.until > self.time && r.at.dist(at) <= BED_REACH) {
            return true;
        }
        let Some(d) = self.building_at(at) else { return false };
        d.variant().furniture.iter().any(|f| matches!(f.what, Furn::Bed | Furn::Bedroll) && d.to_world(f.at).dist(at) <= BED_REACH)
    }

    /// Anyone standing where they were sent lays out, pitches or packs up.
    pub(super) fn do_bed_jobs(&mut self) {
        let t = self.time;
        let mut done = Vec::new();
        let mut later = Vec::new();
        for (n, job) in self.bed_jobs.clone().into_iter().enumerate() {
            let who = job.who();
            let Some(k) = self.squad.index(who) else {
                done.push(n);
                continue;
            };
            if self.is_down(who) || self.fighting.contains_key(&who) {
                done.push(n);
                continue;
            }
            let here = self.squad.at[k];
            let name = self.name_of(who);
            match job {
                BedJob::Lay { kind, at, sleep, .. } => {
                    if here.dist(at) > 1.0 {
                        continue;
                    }
                    done.push(n);
                    let it = kind.item();
                    let took = self.people[who as usize].detail.as_mut().is_some_and(|d| d.gear.take(it));
                    if !took {
                        continue;
                    }
                    self.people[who as usize].recompute_might();
                    self.next_bed += 1;
                    let rot = (self.seed ^ self.next_bed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) as f32 / u64::MAX as f32 * std::f32::consts::TAU;
                    let id = self.next_bed;
                    self.beds.push(Bedding { id, kind, pos: at, rot, since: t, ready_at: t + kind.secs() });
                    self.say(t, match kind {
                        BedKind::Bedroll => format!("{name} lays out a bedroll."),
                        BedKind::Tent => format!("{name} pitches the tent."),
                    });
                    if sleep {
                        if kind.secs() <= 0.0 {
                            // Down at once: lie down in it now.
                            self.squad.resting[k] = true;
                        } else {
                            later.push(BedJob::SleepWhenUp { who, bed: id });
                        }
                    }
                }
                BedJob::Pack { bed, .. } => {
                    let Some(i) = self.beds.iter().position(|b| b.id == bed) else {
                        done.push(n);
                        continue;
                    };
                    if here.dist(self.beds[i].pos) > 1.2 {
                        continue;
                    }
                    done.push(n);
                    let b = self.beds[i];
                    if self.sleepers_in(&b).next().is_some() {
                        continue;
                    }
                    self.beds.remove(i);
                    if let Some(d) = self.people[who as usize].detail.as_mut() {
                        d.gear.add(b.kind.item(), 1);
                    }
                    self.people[who as usize].recompute_might();
                    self.say(t, format!("{name} packs up the {}.", b.kind.name()));
                }
                BedJob::SleepWhenUp { bed, .. } => {
                    let Some(b) = self.bedding(bed).copied() else {
                        done.push(n);
                        continue;
                    };
                    if b.ready(t) {
                        done.push(n);
                        if here.dist(b.pos) <= BED_REACH {
                            self.squad.resting[k] = true;
                        }
                    }
                }
            }
        }
        for n in done.into_iter().rev() {
            self.bed_jobs.remove(n);
        }
        self.bed_jobs.extend(later);
    }
}
