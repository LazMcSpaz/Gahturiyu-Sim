//! The squad's own campfires (the items design, 0.6: a campfire needs fuel).
//! Someone carrying timber is sent to a spot; there they lay a fire from one
//! timber and light it: a minute with a tinderbox, five without
//! (`TINDER_SECS`, `LIGHT_SECS`). It burns `BURN_HOURS` by the clock
//! (`Campfire::out_at`, never ticked), and more wood keeps it going. While
//! it burns it is a fire like any other: in the solid list (`passing_things`),
//! a light (`fixed_lights_at`), somewhere to cook (`crafting::fire_at`), and
//! Douse puts it out. Burnt out, its ashes stay a day.
//!
//! Orders are walk-then-do, like bedding (`bedding.rs`): `order_campfire`
//! and `order_feed_fire` send someone, and `do_fire_jobs` (the squad's own
//! step: the player exception, like walking) does it when they get there.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items;
use super::person::PersonId;
use super::world::{World, DAY, HOUR};

/// What a fire is laid from, and fed with.
pub const FUEL: &str = "timber";
/// How long it takes to get one going, game seconds: with a tinderbox, and
/// without (flint and patience).
pub const TINDER_SECS: f64 = 60.0;
pub const LIGHT_SECS: f64 = 300.0;
/// How long one timber burns.
pub const BURN_HOURS: f64 = 4.0;
/// A fire's ring of stones, metres.
pub const FIRE_RADIUS: f32 = 0.6;
/// How far its light reaches, and how strong.
pub const FIRE_LIGHT: f32 = 16.0;
pub const FIRE_POWER: f32 = 0.65;
/// How near the fire someone must be to lay or feed it, metres.
const AT_FIRE: f32 = 1.8;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Campfire {
    pub id: u32,
    pub pos: V2,
    /// Who laid it.
    pub by: PersonId,
    /// Laid at `since`, burning from `lit_at` until `out_at`.
    pub since: f64,
    pub lit_at: f64,
    pub out_at: f64,
}

impl Campfire {
    pub fn burning(&self, t: f64) -> bool {
        t >= self.lit_at && t < self.out_at
    }
    /// Still being got going.
    pub fn lighting(&self, t: f64) -> bool {
        t >= self.since && t < self.lit_at
    }
}

/// A squad member on their way to lay a fire, or to feed one.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum FireJob {
    Lay { who: PersonId, at: V2 },
    Feed { who: PersonId, fire: u32 },
}

impl FireJob {
    fn who(&self) -> PersonId {
        match *self {
            FireJob::Lay { who, .. } | FireJob::Feed { who, .. } => who,
        }
    }
}

impl World {
    /// A fire lit at `at` right now, by the lead (the dev panel's "spawn
    /// fire"): a campfire like any the squad lays, but it burns its hours
    /// whatever the weather (a testing tool: a forced downpour would put a
    /// real one out at once).
    pub fn dev_fire(&mut self, at: V2) -> u32 {
        let t = self.time;
        let by = self.squad.members.first().copied().unwrap_or(0);
        let id = self.next_fire;
        self.next_fire += 1;
        let out_at = t + BURN_HOURS * HOUR;
        self.campfires.push(Campfire { id, pos: at, by, since: t, lit_at: t, out_at });
        self.solid_changed();
        id
    }

    pub fn campfire(&self, id: u32) -> Option<&Campfire> {
        self.campfires.iter().find(|f| f.id == id)
    }

    /// Can `who` lay a fire? (Of the squad, carrying wood.)
    pub fn can_make_fire(&self, who: PersonId) -> bool {
        self.squad.index(who).is_some() && self.count_of(who, FUEL) > 0
    }

    /// Where to stand to tend a fire at `at`, coming from `from`.
    fn beside_fire(&self, from: V2, at: V2) -> V2 {
        let d = from.sub(at);
        let dir = if d.len() > 0.1 { d.scale(1.0 / d.len()) } else { V2::new(1.0, 0.0) };
        self.free_spot(at.add(dir.scale(FIRE_RADIUS + 0.8)))
    }

    /// Send someone to lay a campfire at `at` and light it.
    pub fn order_campfire(&mut self, who: PersonId, at: V2) -> Result<(), &'static str> {
        if !self.can_make_fire(who) {
            return Err("has no wood to burn");
        }
        if !self.free_to_order(who) || self.fighting.contains_key(&who) {
            return Err("can't now");
        }
        if self.building_at(at).is_some() {
            return Err("not indoors");
        }
        if self.terrain.is_sea(at) {
            return Err("not in the sea");
        }
        let from = self.person_pos(who);
        let stand = self.beside_fire(from, at);
        self.order_members(&[who], stand);
        self.fire_jobs.retain(|j| j.who() != who);
        self.fire_jobs.push(FireJob::Lay { who, at });
        self.do_fire_jobs();
        Ok(())
    }

    /// Send someone to put more wood on a fire.
    pub fn order_feed_fire(&mut self, who: PersonId, fire: u32) -> Result<(), &'static str> {
        let Some(f) = self.campfire(fire).copied() else { return Err("nothing there") };
        if !self.can_make_fire(who) {
            return Err("has no wood to burn");
        }
        if !self.free_to_order(who) || self.fighting.contains_key(&who) {
            return Err("can't now");
        }
        let from = self.person_pos(who);
        let stand = self.beside_fire(from, f.pos);
        self.order_members(&[who], stand);
        self.fire_jobs.retain(|j| j.who() != who);
        self.fire_jobs.push(FireJob::Feed { who, fire });
        self.do_fire_jobs();
        Ok(())
    }

    /// Anyone standing where they were sent lays or feeds the fire.
    pub(super) fn do_fire_jobs(&mut self) {
        let t = self.time;
        let mut done = Vec::new();
        for (n, job) in self.fire_jobs.clone().into_iter().enumerate() {
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
            let at = match job {
                FireJob::Lay { at, .. } => at,
                FireJob::Feed { fire, .. } => match self.campfire(fire) {
                    Some(f) => f.pos,
                    None => {
                        done.push(n);
                        continue;
                    }
                },
            };
            if here.dist(at) > AT_FIRE + FIRE_RADIUS {
                continue;
            }
            done.push(n);
            let wood = items::id(FUEL);
            if !self.people[who as usize].detail.as_mut().is_some_and(|d| d.gear.take(wood)) {
                continue;
            }
            self.people[who as usize].recompute_might();
            let name = self.name_of(who);
            let secs = if self.count_of(who, "tinderbox") > 0 { TINDER_SECS } else { LIGHT_SECS };
            match job {
                FireJob::Lay { .. } => {
                    // Ashes a day cold are gone.
                    self.campfires.retain(|f| t < f.out_at + DAY);
                    self.next_fire += 1;
                    let lit_at = t + secs;
                    let out_at = self.drowned_by(at, lit_at, lit_at + BURN_HOURS * HOUR);
                    self.campfires.push(Campfire { id: self.next_fire, pos: at, by: who, since: t, lit_at, out_at });
                    self.say(t, format!("{name} lays a campfire and gets it going."));
                }
                FireJob::Feed { fire, .. } => {
                    let Some(f) = self.campfires.iter().find(|f| f.id == fire) else { continue };
                    let (pos, burning, out) = (f.pos, t < f.out_at, f.out_at);
                    let (from, until) = if burning { (out, out + BURN_HOURS * HOUR) } else { (t + secs, t + secs + BURN_HOURS * HOUR) };
                    let out_at = self.drowned_by(pos, from, until);
                    let Some(f) = self.campfires.iter_mut().find(|f| f.id == fire) else { continue };
                    if burning {
                        f.out_at = out_at;
                    } else {
                        // Gone out (or doused): wood on the ashes, lit again.
                        f.since = t;
                        f.lit_at = t + secs;
                        f.out_at = out_at;
                        f.by = who;
                    }
                    self.say(t, format!("{name} puts more wood on the fire."));
                }
            }
        }
        for n in done.into_iter().rev() {
            self.fire_jobs.remove(n);
        }
    }
}
