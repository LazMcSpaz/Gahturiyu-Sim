//! Where animals touch the rest of the world: the Overgrowth and grazing,
//! Briarbacks, silk colonies, Bonepickers and bodies, carcasses, territory,
//! and the taming check.
//!
//! Each is a plain question or a plain action for other systems to call.
//! Nothing here reaches into towns, jobs, goods or law.

use serde::{Deserialize, Serialize};

use super::super::geo::{self, V2};
use super::super::person::PersonId;
use super::super::rng::{self, Rng};
use super::super::world::{World, DAY, HOUR};
use super::herd::{h_yields, Herd};
use super::region::{region_centre, region_of, RegionId, REGION};
use super::species::{Active, Sp};

/// Above this Overgrowth level a region breeds Briarbacks.
pub const BRIAR_THRESHOLD: f32 = 0.6;
/// Groups of Briarbacks per whole step of Overgrowth above the threshold.
pub const BRIAR_GROUPS: f32 = 8.0;
/// Grazing pressure from one Turiyu (a region with a hundred grazes at 1.0).
pub const GRAZE_PER_HEAD: f32 = 0.01;
/// The most cocoons a colony hangs, and how many a full colony spins a day.
pub const COCOON_CAP: f32 = 40.0;
pub const COCOONS_PER_DAY: f32 = 5.0;
/// How long each day a Silk Mother is off hunting, hours.
pub const MOTHER_OUT_HOURS: f64 = 2.5;
/// How close someone has to be to take cocoons, metres.
pub const TAKE_REACH: f32 = 8.0;
/// How often someone sneaking gets to the cocoons unnoticed.
pub const UNSEEN_CHANCE: f32 = 0.6;
/// Bonepickers reach a body this many minutes after the death (by day).
pub const BONE_ARRIVE: (f64, f64) = (20.0, 60.0);
/// ...and have it picked clean this many minutes after that.
pub const BONE_PICK: f64 = 40.0;
/// An animal's carcass nobody has taken lies this long, hours.
pub const CARCASS_HOURS: f64 = 6.0;
/// How often a taming attempt works, until it's tied to a skill.
pub const TAME_CHANCE: f32 = 0.6;
/// How close someone has to be to an animal to tame it, metres.
pub const TAME_REACH: f32 = 4.0;

/// A Silkcrawler colony on a hill cliff: the crawlers, the cocoons they
/// hang, and the Silk Mother who guards them.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Colony {
    pub id: u32,
    pub pos: V2,
    pub seed: u64,
    /// The herd of crawlers, and the herd of one that is the Silk Mother.
    pub crawlers: u32,
    pub mother: u32,
    /// Cocoons hanging as of `at`, and how many more a day from then.
    pub cocoons: f32,
    pub per_day: f32,
    pub at: f64,
}

impl Colony {
    pub fn cocoons_at(&self, t: f64) -> f32 {
        (self.cocoons + self.per_day * ((t - self.at).max(0.0) / DAY) as f32).min(COCOON_CAP)
    }
}

/// What came of trying to take cocoons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Take {
    /// Got this many, and nobody the wiser (or nobody left to mind).
    Taken(u32),
    /// The Silk Mother is on you: this fight has started.
    Fight(u32),
    /// She's there and watching; whoever asked isn't someone she can be
    /// fought by yet (only the squad's fights are played out here).
    Guarded,
    Empty,
    TooFar,
    NoSuchColony,
}

/// The body of an animal, lying where it fell.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Carcass {
    pub id: u32,
    pub pos: V2,
    pub sp: Sp,
    pub size: f32,
    pub at: f64,
    /// When it's gone: picked clean, or rotted away.
    pub gone_at: f64,
    /// What its fate is keyed by: whose body it was (herd and animal), not
    /// how many carcasses the world happened to have seen before it.
    pub key: u64,
}

/// When Bonepickers come down on a body and when they've finished with it,
/// if they come at all. They fly by day; a body that falls too near dark
/// lies where it is until it rots.
///
/// `died` is the body's clock (when it fell, or was last set down, or until
/// when it is preserved); how long after that the birds take is keyed by
/// whose body it is alone, so moving that clock moves their coming by the
/// same amount and no more.
pub fn flock_times(world_seed: u64, body: u64, died: f64) -> Option<(f64, f64)> {
    let mut r = Rng::from_keys(&[world_seed, body, 0xB04E]);
    let arrive = died + (BONE_ARRIVE.0 + r.f64() * (BONE_ARRIVE.1 - BONE_ARRIVE.0)) * 60.0;
    Active::Day.at(arrive).then_some((arrive, arrive + BONE_PICK * 60.0))
}

/// How many birds come down on a body.
pub fn flock_size(world_seed: u64, body: u64) -> usize {
    let (lo, hi) = Sp::Bonepicker.def().group;
    lo as usize + Rng::from_keys(&[world_seed, body, 0xF10C]).below((hi - lo + 1) as usize)
}

/// Does a taming attempt work?
///
/// **This is the hook for the society side to tie to a skill.** For now it
/// is a plain keyed roll: the same person trying the same animal for the
/// n-th time always gets the same answer.
pub fn taming_check(w: &World, who: PersonId, herd: u32, ident: u16, attempt: u32) -> bool {
    Rng::from_keys(&[w.seed, who as u64, herd as u64, ident as u64, attempt as u64, 0x7A4E]).f32() < TAME_CHANCE
}

impl Herd {
    /// Is a Silk Mother off hunting at time `t`? A stretch of each day,
    /// drawn from her seed and the day.
    pub fn mother_out(&self, t: f64) -> bool {
        if self.sp != Sp::SilkMother {
            return false;
        }
        let day = (t / DAY).floor();
        let start = Rng::from_keys(&[self.seed, day as u64, 0x0077_A4A7]).f64() * (24.0 - MOTHER_OUT_HOURS);
        let h = (t - day * DAY) / HOUR;
        h >= start && h < start + MOTHER_OUT_HOURS
    }
}

impl World {
    // ---- Grazing and the Overgrowth ----------------------------------------------

    /// How hard Turiyu are grazing a region: wild herds and pastured ones
    /// together. (Their grazing is what pushes the Overgrowth back; what
    /// that does to the Overgrowth level is for whoever owns it.)
    pub fn grazing_pressure(&self, r: RegionId) -> f32 {
        self.grazing_pressure_at(r, self.time)
    }

    /// The same at a given moment (for callers on the world's timeline).
    pub fn grazing_pressure_at(&self, r: RegionId, t: f64) -> f32 {
        (self.population_at(r, Sp::WildTuriyu, t) + self.turiyu_at_pasture(r, t)).max(0.0) * GRAZE_PER_HEAD + 0.0
    }

    /// Briarbacks in a region follow its Overgrowth: above the threshold
    /// they appear (more the higher it stands); below it they die back.
    pub(super) fn briars_follow_overgrowth(&mut self, r: RegionId, t: f64) {
        let level = self.overgrowth(r);
        let seed = self.seed;
        let have: Vec<u32> = self.animals.regions[r as usize].herds.iter().copied().filter(|&h| self.animals.herds[h as usize].sp == Sp::Briarback).collect();
        if level <= BRIAR_THRESHOLD {
            for h in have {
                let h = &mut self.animals.herds[h as usize];
                h.n = h.count_at(t);
                h.at = t;
                h.limit = 0.0;
            }
            return;
        }
        let want = (((level - BRIAR_THRESHOLD) * BRIAR_GROUPS).ceil() as usize).max(1);
        for &h in &have {
            let h = &mut self.animals.herds[h as usize];
            if h.limit <= 0.0 {
                h.n = h.count_at(t).max((h.cap as f32).min(2.0));
                h.at = t;
                h.limit = h.cap as f32 + 0.5;
                h.restock_at = f64::INFINITY;
            }
        }
        let (lo, hi) = Sp::Briarback.def().group;
        for k in have.len()..want {
            let mut rr = Rng::from_keys(&[seed, r as u64, k as u64, 0xB21A]);
            let c = region_centre(r);
            let mut home = None;
            for _ in 0..24 {
                let p = V2::new(c.x + rr.range(-0.5, 0.5) * REGION, c.y + rr.range(-0.5, 0.5) * REGION);
                if geo::is_land(p) && geo::inland(p) > 40.0 && self.terrain.slope(p) < 0.4 {
                    home = Some(p);
                    break;
                }
            }
            let Some(home) = home else { continue };
            let cap = lo + rr.below((hi - lo + 1) as usize) as u8;
            self.add_herd(Sp::Briarback, home, cap, cap as f32, rng::key(&[seed, r as u64, k as u64, 0xB21B]));
        }
        self.animals.near_at = V2::new(-1e9, -1e9);
    }

    // ---- Territory ------------------------------------------------------------

    /// The Cragmaw whose territory covers this point, if one's does (and
    /// it's alive). What trespassing means is for others to decide; the
    /// Cragmaw itself will come for anyone it finds there by day.
    pub fn cragmaw_territory(&self, p: V2) -> Option<u32> {
        let t = self.time;
        let reach = Sp::Cragmaw.def().sense;
        self.animals.herds.iter().find(|h| h.sp == Sp::Cragmaw && h.alive(t) > 0 && h.home.dist(p) <= reach).map(|h| h.id)
    }

    // ---- Silk colonies ----------------------------------------------------------

    pub fn colonies(&self) -> &[Colony] {
        &self.animals.colonies
    }

    /// Whole cocoons hanging in a colony right now.
    pub fn cocoons(&self, colony: u32) -> u32 {
        self.animals.colonies.get(colony as usize).map(|c| c.cocoons_at(self.time).floor() as u32).unwrap_or(0)
    }

    /// Is the colony's Silk Mother alive, on her feet and at home? (Fighting
    /// someone at the colony counts as at home.)
    pub fn mother_guarding(&self, colony: u32) -> bool {
        let t = self.time;
        let Some(c) = self.animals.colonies.get(colony as usize) else { return false };
        let m = &self.animals.herds[c.mother as usize];
        m.alive(t) > 0 && !m.down(0, t) && !m.mother_out(t) && (m.away.map(|a| t >= a.until).unwrap_or(true) || self.herd_in_fray(c.mother))
    }

    /// `who` reaches for the cocoons.
    ///
    /// It works without a fight only if the Silk Mother is dead, down, away,
    /// or doesn't notice (someone sneaking has a fair chance). Otherwise she
    /// attacks: a squad member's fight starts there and then, and nothing is
    /// taken. If she can't be fought by whoever asked (they aren't in the
    /// squad, or the squad is already fighting, her included) the answer is
    /// `Guarded` and nothing is taken either.
    pub fn take_cocoons(&mut self, who: PersonId, colony: u32) -> Take {
        let Some(c) = self.animals.colonies.get(colony as usize).cloned() else { return Take::NoSuchColony };
        let t = self.time.max(c.at);
        if self.person_pos(who).dist(c.pos) > TAKE_REACH {
            return Take::TooFar;
        }
        if self.mother_guarding(colony) {
            let unseen = !self.herd_in_fray(c.mother) && self.is_sneaking(who) && Rng::from_keys(&[self.seed, colony as u64, who as u64, (t / 300.0).floor() as u64, 0x5EE4]).f32() < UNSEEN_CHANCE;
            if !unseen {
                if self.squad.index(who).is_none() || self.squad_battle().is_some() {
                    return Take::Guarded;
                }
                return match self.herd_attacks_squad(c.mother, 0.0) {
                    Some(id) => Take::Fight(id),
                    None => Take::Guarded,
                };
            }
        }
        let n = c.cocoons_at(t).floor();
        if n < 1.0 {
            return Take::Empty;
        }
        let col = &mut self.animals.colonies[colony as usize];
        col.cocoons = col.cocoons_at(t) - n;
        col.at = t;
        Take::Taken(n as u32)
    }

    /// The dawn stock-take: how fast each colony spins follows how many
    /// crawlers it has.
    pub(super) fn settle_colonies(&mut self, t: f64) {
        for k in 0..self.animals.colonies.len() {
            let h = &self.animals.herds[self.animals.colonies[k].crawlers as usize];
            let share = h.alive(t) as f32 / h.cap.max(1) as f32;
            let c = &mut self.animals.colonies[k];
            // (Never backwards: someone may have taken cocoons since.)
            let t = t.max(c.at);
            c.cocoons = c.cocoons_at(t);
            c.at = t;
            c.per_day = COCOONS_PER_DAY * share;
        }
    }

    // ---- Bodies and Bonepickers ------------------------------------------------

    /// When Bonepickers will have cleared (or did clear) this person's body.
    /// `None`: the body isn't going to be cleared by them (none came, it's
    /// already rotted away, or the person isn't dead on the ground).
    pub fn body_cleared_at(&self, pid: PersonId) -> Option<f64> {
        if let Some(c) = self.corpses.iter().find(|c| c.3 == pid) {
            return self.flock_for(c).map(|x| x.1);
        }
        self.animals.cleared.iter().rev().find(|c| c.0 == pid).map(|c| c.1)
    }

    /// Is this person's body still lying there to be found (or raised)?
    pub fn body_there(&self, pid: PersonId) -> bool {
        self.corpses.iter().any(|c| c.3 == pid)
    }

    /// The earliest body due to be picked clean: (when, who).
    pub(super) fn next_body_cleared(&self) -> Option<(f64, PersonId)> {
        self.corpses.iter().filter_map(|c| self.flock_for(c).map(|x| (x.1, c.3))).min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
    }

    /// When the birds come to this body and when they are done, if they
    /// come. They leave alone a body that is being carried, one lying in or
    /// by a town (people keep them off), and one in the middle of a fight
    /// (brought there to be raised).
    fn flock_for(&self, c: &(V2, super::super::race::Race, f64, PersonId)) -> Option<(f64, f64)> {
        let pid = c.3;
        if self.carried.contains_key(&pid) || self.in_town(c.0) {
            return None;
        }
        if self.battles.iter().any(|b| !b.over && b.fighters.iter().any(|f| f.pid == pid)) {
            return None;
        }
        flock_times(self.seed, pid as u64, c.2)
    }

    /// Is a point in or right by a town?
    pub(super) fn in_town(&self, p: V2) -> bool {
        self.settlements.iter().any(|s| s.pos.dist(p) < s.radius() + 40.0)
    }

    pub(super) fn clear_body(&mut self, pid: PersonId, t: f64) {
        self.corpses.retain(|c| c.3 != pid);
        self.animals.cleared.push((pid, t));
        if self.animals.cleared.len() > 64 {
            self.animals.cleared.remove(0);
        }
        self.animals.stats.bodies_cleared += 1;
    }

    /// Flocks on the ground right now: where, and how many birds. (For drawing.)
    pub fn flocks(&self) -> Vec<(V2, usize)> {
        let t = self.time;
        let mut out = Vec::new();
        for c in &self.corpses {
            if let Some((a, b)) = self.flock_for(c) {
                if t >= a && t < b {
                    out.push((c.0, flock_size(self.seed, c.3 as u64)));
                }
            }
        }
        for c in &self.animals.carcasses {
            if self.in_town(c.pos) {
                continue;
            }
            if let Some((a, b)) = flock_times(self.seed, c.key, c.at) {
                if t >= a && t < b.min(c.gone_at) {
                    out.push((c.pos, flock_size(self.seed, c.key)));
                }
            }
        }
        out
    }

    // ---- Carcasses --------------------------------------------------------------

    /// An animal's body is left lying. `whose` names the animal (its herd
    /// and its own number), so what becomes of the body doesn't depend on
    /// how many others have fallen anywhere else.
    pub(super) fn leave_carcass(&mut self, sp: Sp, size: f32, pos: V2, t: f64, whose: u64) {
        let id = self.animals.next_carcass;
        self.animals.next_carcass += 1;
        let key = rng::key(&[whose, 0xCA2C_A55]);
        let rots = t + CARCASS_HOURS * HOUR;
        let birds = if self.in_town(pos) { None } else { flock_times(self.seed, key, t) };
        let gone_at = birds.map(|x| x.1.min(rots)).unwrap_or(rots);
        self.animals.carcasses.push(Carcass { id, pos, sp, size, at: t, gone_at, key });
    }

    /// Animal carcasses lying within `radius` of a point.
    pub fn carcasses_near(&self, p: V2, radius: f32) -> Vec<&Carcass> {
        self.animals.carcasses.iter().filter(|c| c.pos.dist(p) <= radius).collect()
    }

    /// Take everything a carcass has to give (names and amounts). It's gone
    /// afterwards.
    pub fn butcher(&mut self, id: u32) -> Vec<(&'static str, f32)> {
        let Some(k) = self.animals.carcasses.iter().position(|c| c.id == id) else { return Vec::new() };
        let c = self.animals.carcasses.remove(k);
        h_yields(c.sp, c.size)
    }

    // ---- Taming -----------------------------------------------------------------

    /// Can the animal in this place of this herd be tamed right now? Only a
    /// Ridgehound that is knocked out or young.
    pub fn tameable(&self, herd: u32, slot: usize) -> bool {
        let t = self.time;
        let Some(h) = self.animals.herds.get(herd as usize) else { return false };
        // (Not one that is in a fight: it is tamed when the fight is over.)
        h.def().tameable && slot < h.alive(t) && (h.down(slot, t) || h.young(slot, t)) && !self.animal_fighting(herd, h.ident(slot))
    }

    /// `who` (a squad member standing over it) finishes off a wild animal
    /// that is down. It leaves a carcass. False if it isn't down, isn't in
    /// reach, or is still in a fight.
    ///
    /// (Animals knocked down in a fight get up again in time, as people do.
    /// Prey your squad hunted and beat is finished off for you; this is for
    /// anything else left lying: a hound you'd rather not see again, say.)
    pub fn finish_off(&mut self, who: PersonId, herd: u32, slot: usize) -> bool {
        let t = self.time;
        let Some(h) = self.animals.herds.get(herd as usize) else { return false };
        if self.squad.index(who).is_none() || slot >= h.alive(t) || !h.down(slot, t) || self.animal_fighting(herd, h.ident(slot)) {
            return false;
        }
        let pos = self.animal_pos(herd, slot, t);
        if self.person_pos(who).dist(pos) > TAME_REACH {
            return false;
        }
        let h = &mut self.animals.herds[herd as usize];
        let (sp, size, ident) = (h.sp, h.size(slot, t), h.ident(slot));
        h.remove(slot, t);
        self.leave_carcass(sp, size, pos, t, rng::key(&[herd as u64, ident as u64]));
        self.animals.stats.animals_killed += 1;
        true
    }

    /// `who` (a squad member) tries to tame the animal in this place of
    /// this herd. On success it leaves its pack for good and follows them;
    /// returns the tamed hound's id.
    pub fn tame(&mut self, who: PersonId, herd: u32, slot: usize) -> Result<u32, &'static str> {
        let t = self.time;
        if self.squad.index(who).is_none() {
            return Err("only someone in the squad can keep a hound");
        }
        if !self.tameable(herd, slot) {
            return Err("only a Ridgehound that is down or young can be tamed");
        }
        if self.person_pos(who).dist(self.animal_pos(herd, slot, t)) > TAME_REACH {
            return Err("too far away");
        }
        let ident = self.animals.herds[herd as usize].ident(slot);
        let attempt = self.animals.tame_tries;
        self.animals.tame_tries += 1;
        if !taming_check(self, who, herd, ident, attempt) {
            return Err("it won't have it");
        }
        let h = &mut self.animals.herds[herd as usize];
        let size = h.size(slot, t);
        let hurt = super::herd::Hurt { lost: h.lost(slot, t), at: t };
        let seed = rng::key(&[h.seed, ident as u64, 0x7A3E]);
        h.remove(slot, t);
        let id = self.animals.next_hound;
        self.animals.next_hound += 1;
        self.animals.hounds.push(super::livestock::Hound { id, owner: who, seed, size, hurt, hunger: 0.3, fed_at: t, tamed_at: t, gone: false });
        let name = self.people[who as usize].name().unwrap_or("Someone").to_string();
        self.log.push_front((t, format!("{name} has a Ridgehound at heel.")));
        self.log.truncate(14);
        Ok(id)
    }

    /// The region a point is in (re-exported here for callers who only have a `World`).
    pub fn region_at(&self, p: V2) -> RegionId {
        region_of(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bonepickers_come_by_day_and_finish_inside_two_hours() {
        let mut came = 0;
        for k in 0..200u64 {
            let died = 9.0 * HOUR + k as f64 * 137.0;
            if let Some((a, b)) = flock_times(3, k, died) {
                came += 1;
                assert!(a > died + 19.0 * 60.0 && b - a == BONE_PICK * 60.0);
                assert!(b - died < 2.0 * HOUR, "they'd be beaten to it by rot");
            }
            // Nobody comes in the dark.
            assert!(flock_times(3, k, 23.0 * HOUR + k as f64).is_none());
        }
        assert_eq!(came, 200);
    }

    #[test]
    fn cocoons_build_up_to_a_cap() {
        let c = Colony { id: 0, pos: V2::default(), seed: 1, crawlers: 0, mother: 1, cocoons: 3.0, per_day: 5.0, at: 0.0 };
        assert!((c.cocoons_at(2.0 * DAY) - 13.0).abs() < 1e-3);
        assert_eq!(c.cocoons_at(100.0 * DAY), COCOON_CAP);
    }
}
