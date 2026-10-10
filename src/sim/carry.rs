//! Carrying the fallen, Kenshi-style.
//!
//! Any squad member on their feet can be sent to pick up someone who's down:
//! a knocked-out or dead squadmate, or a knocked-out stranger. They walk over
//! and shoulder them. The body (and everything it carries) counts toward the
//! carrier's load, so the usual overload slow-downs apply — most people can
//! carry another person, but not quickly.
//!
//! Carrying a body has its own pace (`carry_pace`) rather than counting as
//! ordinary overload: a strong carrier with a light body manages better than
//! a weak one with a heavy one, but nobody crawls. The body still counts as
//! load for stamina and hunger.
//!
//! A carrier can't fight: in a fight they can still be ordered to move, but
//! they won't swing, cast or block until they put their burden down. If a
//! carrier is knocked out, whoever they were carrying drops beside them.
//! A carried squadmate who comes round is set down so they can walk.
//!
//! A stranger put down stays where they were left until they come round;
//! then they go back to whatever they were doing.

use super::body;
use super::geo::V2;
use super::person::PersonId;
use super::race::Race;
use super::squad::REACH;
use super::world::World;

/// Pace while carrying someone, as a share of normal: for a carrier of
/// average Strength (40) with a body of `CARRY_BODY` kg.
pub const CARRY_PACE: f32 = 0.6;
/// The body weight `CARRY_PACE` is set for, kg.
pub const CARRY_BODY: f32 = 85.0;
/// Carrying pace never goes outside these.
pub const CARRY_PACE_RANGE: (f32, f32) = (0.3, 0.85);

/// Rough body weight by people, kg (placeholders).
pub fn body_weight(r: Race) -> f32 {
    match r {
        Race::Roduro => 85.0,
        Race::Qotiro => 100.0,
        Race::Horaro => 70.0,
        Race::Tadoro => 60.0,
    }
}

impl World {
    /// Is this person down (knocked out or dead)?
    pub fn is_down(&self, pid: PersonId) -> bool {
        if let Some(f) = self.fighter(pid) {
            return f.ko || f.dead;
        }
        let p = &self.people[pid as usize];
        p.dead || body::knocked_out(&p.wounds.hp_at(&p.stats, self.time))
    }

    /// Who this person is carrying, if anyone.
    pub fn carrying(&self, carrier: PersonId) -> Option<PersonId> {
        self.carried.iter().find(|(_, &c)| c == carrier).map(|(&p, _)| p)
    }

    /// Who's carrying this person, if anyone.
    pub fn carried_by(&self, pid: PersonId) -> Option<PersonId> {
        self.carried.get(&pid).copied()
    }

    /// Kilograms on someone's shoulders beyond their own kit: the body they
    /// carry and everything on it.
    pub fn burden_weight(&self, carrier: PersonId) -> f32 {
        match self.carrying(carrier) {
            Some(c) => {
                let p = &self.people[c as usize];
                body_weight(p.race) + p.kit().weight()
            }
            None => 0.0,
        }
    }

    /// How much carrying someone slows this person: 1 if they carry nobody.
    /// Stronger carriers and lighter bodies go faster.
    pub fn carry_pace(&self, carrier: PersonId) -> f32 {
        if self.carrying(carrier).is_none() {
            return 1.0;
        }
        let p = &self.people[carrier as usize];
        let strength = p.effective_stats().attr(super::stats::Attr::Strength);
        let body = self.burden_weight(carrier).max(1.0);
        (CARRY_PACE * (1.0 + (strength - 40.0) / 120.0) * (CARRY_BODY / body).sqrt()).clamp(CARRY_PACE_RANGE.0, CARRY_PACE_RANGE.1)
    }

    /// Can this person be picked up at all?
    pub fn can_carry(&self, carrier: PersonId, target: PersonId) -> bool {
        if !self.valid_person(target) || carrier == target || self.squad.index(carrier).is_none() || self.is_down(carrier) {
            return false;
        }
        if self.carrying(carrier).is_some() || self.carried.contains_key(&target) || self.carried.values().any(|&c| c == target) {
            return false;
        }
        if self.fighting.contains_key(&target) || self.fighting.contains_key(&carrier) {
            return false;
        }
        let p = &self.people[target as usize];
        // Dead strangers are left where they lie; your own dead you can bring home.
        if p.dead {
            return p.in_squad && self.corpses.iter().any(|c| c.3 == target);
        }
        self.is_down(target)
    }

    /// Where a body is lying (or standing) right now.
    pub fn body_pos(&self, pid: PersonId) -> V2 {
        if self.people[pid as usize].dead {
            if let Some(c) = self.corpses.iter().find(|c| c.3 == pid) {
                return c.0;
            }
        }
        self.person_pos(pid)
    }

    /// Send a squad member to pick someone up.
    pub fn order_carry(&mut self, carrier: PersonId, target: PersonId) -> bool {
        if !self.can_carry(carrier, target) {
            return false;
        }
        let Some(k) = self.squad.index(carrier) else { return false };
        let at = self.body_pos(target);
        let (path, _) = self.route(self.member_pos(k), at);
        self.squad.goal[k] = *path.last().unwrap_or(&at);
        self.squad.route[k] = path;
        self.squad.resting[k] = false;
        self.want_carry.retain(|w| w.0 != carrier);
        self.want_carry.push((carrier, target));
        self.do_carrying();
        true
    }

    /// Set down whoever this member is carrying, at their feet.
    pub fn put_down(&mut self, carrier: PersonId) -> bool {
        let Some(c) = self.carrying(carrier) else { return false };
        let at = self.person_pos(carrier).add(V2::new(0.9, 0.6));
        self.carried.remove(&c);
        self.lay(c, at);
        let (a, b) = (self.name_of_or(carrier), self.name_of_or(c));
        self.log.push_front((self.time, format!("{a} puts {b} down.")));
        self.log.truncate(14);
        true
    }

    fn name_of_or(&self, pid: PersonId) -> String {
        let p = &self.people[pid as usize];
        p.name().map(|s| s.to_string()).unwrap_or_else(|| super::names::person_name(p.race, p.seed))
    }

    /// Leave a body at `at`.
    fn lay(&mut self, pid: PersonId, at: V2) {
        if let Some(k) = self.squad.index(pid) {
            self.squad.at[k] = at;
            self.squad.goal[k] = at;
            self.squad.route[k].clear();
        } else if self.people[pid as usize].dead {
            let t = self.time;
            if let Some(c) = self.corpses.iter_mut().find(|c| c.3 == pid) {
                c.0 = at;
                c.2 = t;
            }
        } else {
            self.set_down.insert(pid, at);
        }
    }

    /// Pick-ups that have arrived, bodies that move with their carriers,
    /// carriers who collapse, and the carried who come round.
    pub(super) fn do_carrying(&mut self) {
        // Arrivals.
        let mut done = Vec::new();
        for (i, &(carrier, target)) in self.want_carry.iter().enumerate() {
            if !self.can_carry(carrier, target) {
                done.push(i);
                continue;
            }
            let Some(k) = self.squad.index(carrier) else {
                done.push(i);
                continue;
            };
            if self.squad.at[k].dist(self.body_pos(target)) <= REACH + 0.5 {
                done.push(i);
            }
        }
        for &i in done.iter().rev() {
            let (carrier, target) = self.want_carry.remove(i);
            let ok = self.can_carry(carrier, target) && self.squad.index(carrier).map(|k| self.squad.at[k].dist(self.body_pos(target)) <= REACH + 0.5).unwrap_or(false);
            if ok {
                self.carried.insert(target, carrier);
                self.set_down.remove(&target);
                let (a, b) = (self.name_of_or(carrier), self.name_of_or(target));
                self.log.push_front((self.time, format!("{a} picks up {b}.")));
                self.log.truncate(14);
            }
        }
        // Carriers who've gone down drop their burden; the carried who come
        // round (squad) are set down to walk.
        let links: Vec<(PersonId, PersonId)> = self.carried.iter().map(|(&a, &b)| (a, b)).collect();
        for (body, carrier) in links {
            let carrier_down = self.is_down(carrier) || self.squad.index(carrier).is_none();
            let woke = !self.people[body as usize].dead && !self.is_down(body);
            if carrier_down {
                let at = self.person_pos(carrier).add(V2::new(0.8, -0.5));
                self.carried.remove(&body);
                self.lay(body, at);
                let (a, b) = (self.name_of_or(carrier), self.name_of_or(body));
                self.log.push_front((self.time, format!("{a} falls, dropping {b}.")));
                self.log.truncate(14);
            } else if woke {
                self.put_down(carrier);
            }
        }
        // Bodies go where their carriers go.
        let links: Vec<(PersonId, PersonId)> = self.carried.iter().map(|(&a, &b)| (a, b)).collect();
        let t = self.time;
        for (body, carrier) in links {
            let at = self.person_pos(carrier);
            if let Some(k) = self.squad.index(body) {
                self.squad.at[k] = at;
                self.squad.goal[k] = at;
            }
            if let Some(c) = self.corpses.iter_mut().find(|c| c.3 == body) {
                c.0 = at;
                c.2 = t; // a carried body isn't left to rot
            }
        }
        // Strangers who've come round get up and go back to what they were
        // doing: one with a band walks back to it from where they lay.
        let mut up: Vec<PersonId> = self.set_down.keys().copied().filter(|&p| !self.is_down(p)).collect();
        up.sort_unstable();
        let t = self.time;
        for p in up {
            if let Some(at) = self.set_down.remove(&p) {
                if self.group_of[p as usize].and_then(|g| self.group(g)).is_some() {
                    self.getting_up.insert(p, (at, t));
                }
            }
        }
        // Back with their band (or the band is gone): done walking.
        let mut back: Vec<PersonId> = self.getting_up.keys().copied().collect();
        back.sort_unstable();
        for p in back {
            let (from, since) = self.getting_up[&p];
            let band = self.group_of[p as usize].and_then(|g| self.group(g)).map(|g| g.pos);
            if band.is_none_or(|b| (t - since) as f32 * super::world::WALK_BACK >= b.dist(from) + 10.0) {
                self.getting_up.remove(&p);
            }
        }
    }
}
