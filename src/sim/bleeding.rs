//! Bleeding out of a fight (the items design, 0.2 rule 1). In a fight a cut
//! opens a bleed (`combat.rs`, `Does::Bleed`); when the fight is over a squad
//! member still bleeding carries it in their wounds (`body::Wounds::bleed`),
//! on the condition timeline (rule 8): the torso loses blood at a steady
//! rate, and the moment it kills is solved for (`condition.rs`). Only a
//! bandage (anything that staunches, `combat::is_bandage`) or healing stops
//! it. Everyone else is bound by their own people when a fight ends.

use super::combat::is_bandage;
use super::items::ItemId;
use super::person::PersonId;
use super::world::World;

/// How near a squadmate must be to bind someone out of a fight, metres.
pub const BIND_NEAR: f32 = 4.0;

impl World {
    /// Torso health a squad member is losing to a bleed, per hour (0 if
    /// none).
    pub fn bleeding(&self, pid: PersonId) -> f32 {
        self.people[pid as usize].wounds.bleed
    }

    /// The first bandage in someone's pack.
    pub fn bandage_of(&self, pid: PersonId) -> Option<ItemId> {
        let d = self.people[pid as usize].detail.as_ref()?;
        d.gear.bag.iter().map(|e| e.0).find(|&it| is_bandage(it))
    }

    /// Stop `pid`'s bleed at `t` (the wounds are settled first, so the blood
    /// already lost stays lost).
    pub(super) fn stop_bleeding(&mut self, pid: PersonId, t: f64) -> bool {
        if self.people[pid as usize].wounds.bleed <= 0.0 {
            return false;
        }
        if self.people[pid as usize].cond.is_some() {
            self.settle_condition(pid, t);
        } else {
            let p = &mut self.people[pid as usize];
            p.wounds.lost = p.wounds.lost_at(t);
            p.wounds.at = t;
        }
        self.people[pid as usize].wounds.bleed = 0.0;
        true
    }

    /// `by` binds `who`'s bleeding with a bandage from `by`'s pack (out of a
    /// fight: in one it takes a few seconds, `Battle::begin_bind`).
    pub(super) fn bind_wounds(&mut self, by: PersonId, who: PersonId, t: f64) -> bool {
        let Some(it) = self.bandage_of(by) else { return false };
        if !self.stop_bleeding(who, t) {
            return false;
        }
        if let Some(d) = self.people[by as usize].detail.as_mut() {
            d.gear.take(it);
        }
        self.people[by as usize].recompute_might();
        let (a, b) = (self.name_of(by), self.name_of(who));
        let line = if by == who { format!("{a} binds their cuts.") } else { format!("{a} binds {b}'s cuts.") };
        self.log.push_front((t, line));
        self.log.truncate(14);
        true
    }

    /// Once a fight is over, a squad member still bleeding is bound with a
    /// bandage from their own pack, or from a squadmate's beside them.
    pub(super) fn bind_after_fight(&mut self, who: PersonId, t: f64) {
        if self.bleeding(who) <= 0.0 {
            return;
        }
        let at = self.person_pos(who);
        let mut by: Vec<PersonId> = vec![who];
        by.extend(self.squad.members.iter().copied().filter(|&m| m != who && !self.is_down(m) && !self.people[m as usize].dead && self.person_pos(m).dist(at) <= BIND_NEAR));
        if let Some(b) = by.into_iter().find(|&m| self.bandage_of(m).is_some()) {
            self.bind_wounds(b, who, t);
        }
    }

    /// Have one squad member bind another's bleeding with a bandage from
    /// their own pack: at once if they're beside them, else they walk over
    /// and do it there (`do_binding`).
    pub fn order_bind(&mut self, by: PersonId, who: PersonId) -> Result<String, String> {
        if self.squad.index(by).is_none() || self.squad.index(who).is_none() {
            return Err("Only between squad members.".into());
        }
        let (a, b) = (self.name_of(by), self.name_of(who));
        if self.is_down(by) {
            return Err(format!("{a} is down too."));
        }
        if self.fighting.contains_key(&by) {
            return Err("In a fight, it's done from the pack (it takes a few seconds).".into());
        }
        if self.bandage_of(by).is_none() {
            return Err(format!("{a} has no bandage."));
        }
        if self.bleeding(who) <= 0.0 {
            return Err(format!("{b} isn't bleeding."));
        }
        self.binding.retain(|x| x.0 != by);
        if self.person_pos(by).dist(self.person_pos(who)) <= BIND_NEAR {
            let t = self.time;
            self.bind_wounds(by, who, t);
            return Ok(if by == who { format!("{a} binds their cuts.") } else { format!("{a} binds {b}'s cuts.") });
        }
        let at = self.person_pos(who);
        self.order_members(&[by], at);
        self.binding.push((by, who));
        Ok(format!("{a} goes over to {b}."))
    }

    /// Squad members on their way to bind someone: once near, they do.
    pub(super) fn do_binding(&mut self) {
        let mut k = 0;
        while k < self.binding.len() {
            let (g, p) = self.binding[k];
            if self.squad.index(g).is_none() || self.squad.index(p).is_none() || self.is_down(g) || self.bandage_of(g).is_none() || self.bleeding(p) <= 0.0 {
                self.binding.remove(k);
                continue;
            }
            if self.person_pos(g).dist(self.person_pos(p)) <= BIND_NEAR * 0.5 {
                self.binding.remove(k);
                let t = self.time;
                self.bind_wounds(g, p, t);
                continue;
            }
            k += 1;
        }
    }

    /// Who a bandage used by `by` out of a fight goes on: themselves if
    /// they're bleeding, else the worst-bleeding squadmate near them.
    pub fn bandage_for(&self, by: PersonId) -> Option<PersonId> {
        if self.bleeding(by) > 0.0 {
            return Some(by);
        }
        let at = self.person_pos(by);
        self.squad.members.iter().copied().filter(|&m| self.bleeding(m) > 0.0 && !self.people[m as usize].dead && self.person_pos(m).dist(at) <= BIND_NEAR).max_by(|&a, &b| self.bleeding(a).total_cmp(&self.bleeding(b)))
    }
}
