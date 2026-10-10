//! Looting the beaten (the playable-MVP list): a squad member sent to a
//! knocked-out or dead foe goes over to them, and while standing by can take
//! what they wear and carry, piece by piece or all at once. Kenshi's money
//! and gear come mostly this way.
//!
//! Who can be looted: anyone down (out cold or dead) who was against you:
//! bandits, and members of any group that fought the squad. Townsfolk aren't
//! fair game (stealing from them is a crime the town's watch deals with).

use serde::{Deserialize, Serialize};

use super::inventory::Entry;
use super::items::{self, item, Slot, SLOTS};
use super::person::PersonId;
use super::squad::REACH;
use super::world::World;

/// A squad member sent to go through someone's things.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Looting {
    pub who: PersonId,
    pub body: PersonId,
}

/// One thing on a body: worn in a slot, or in their pack (by entry index).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootRef {
    Worn(Slot),
    Pack(usize),
}

/// How much a bandit carries in coin, at most.
pub const BANDIT_PURSE: u64 = 40;

impl World {
    /// Can the squad go through this person's things?
    pub fn can_loot(&self, body: PersonId) -> bool {
        let p = &self.people[body as usize];
        if p.in_squad || self.fighting.contains_key(&body) || !self.is_down(body) {
            return false;
        }
        let hostile = p.bandit || self.group_of[body as usize].and_then(|g| self.group(g)).is_some_and(|g| g.hostile);
        hostile && p.detail.is_some()
    }

    /// Send a squad member to go through a body's things.
    pub fn order_loot(&mut self, who: PersonId, body: PersonId) -> bool {
        if !self.can_loot(body) {
            return false;
        }
        if self.squad.index(who).is_none() {
            return false;
        }
        let pos = self.person_pos(body);
        self.send(who, pos);
        self.looting.retain(|l| l.who != who);
        self.looting.push(Looting { who, body });
        true
    }

    /// The body this member is standing over and going through, if any.
    pub fn looting_now(&self, who: PersonId) -> Option<PersonId> {
        let l = self.looting.iter().find(|l| l.who == who)?;
        let k = self.squad.index(who)?;
        (self.squad.at[k].dist(self.person_pos(l.body)) <= REACH * 1.5 && self.can_loot(l.body)).then_some(l.body)
    }

    /// Stop going through a body (walked off, or done).
    pub fn stop_looting(&mut self, who: PersonId) {
        self.looting.retain(|l| l.who != who);
    }

    /// Everything on a body: (where, item, how many).
    pub fn loot_of(&self, body: PersonId) -> Vec<(LootRef, items::ItemId, u16)> {
        let Some(d) = self.people[body as usize].detail.as_ref() else { return Vec::new() };
        let mut out: Vec<(LootRef, items::ItemId, u16)> = SLOTS.iter().filter_map(|&s| d.gear.in_slot(s).map(|it| (LootRef::Worn(s), it, 1))).collect();
        out.extend(d.gear.bag.iter().enumerate().map(|(k, e)| (LootRef::Pack(k), e.0, e.1)));
        out
    }

    /// Take one thing (a whole stack, from the pack) off a body.
    pub fn take_loot(&mut self, who: PersonId, body: PersonId, what: LootRef) -> bool {
        if self.looting_now(who) != Some(body) {
            return false;
        }
        let Some(d) = self.people[body as usize].detail.as_mut() else { return false };
        let taken: Vec<Entry> = match what {
            LootRef::Worn(s) => {
                let Some(it) = d.gear.in_slot(s) else { return false };
                let piece = d.gear.piece(s).copied();
                d.gear.discard(s);
                vec![Entry(it, 1, piece)]
            }
            LootRef::Pack(k) => {
                if k >= d.gear.bag.len() {
                    return false;
                }
                vec![d.gear.bag.remove(k)]
            }
        };
        self.people[body as usize].recompute_might();
        let mut names = Vec::new();
        if let Some(dd) = self.people[who as usize].detail.as_mut() {
            for e in &taken {
                match e.2 {
                    Some(pc) => dd.gear.add_piece(e.0, pc),
                    None => dd.gear.add(e.0, e.1),
                }
                names.push(if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() });
            }
        }
        self.people[who as usize].recompute_might();
        self.settle_condition(who, self.time);
        let (a, b) = (self.people[who as usize].name().unwrap_or("someone").to_string(), self.people[body as usize].name().unwrap_or("them").to_string());
        self.log.push_front((self.time, format!("{a} takes {} from {b}.", names.join(", "))));
        self.log.truncate(14);
        true
    }

    /// Take everything off a body.
    pub fn take_all_loot(&mut self, who: PersonId, body: PersonId) -> usize {
        let mut n = 0;
        while let Some(&(what, _, _)) = self.loot_of(body).first() {
            if !self.take_loot(who, body, what) {
                break;
            }
            n += 1;
        }
        n
    }

    /// Drop lootings whose looter has gone (left the squad, or the body
    /// got up and walked off).
    pub(super) fn tidy_looting(&mut self) {
        let keep: Vec<Looting> = self.looting.iter().copied().filter(|l| self.squad.index(l.who).is_some() && self.can_loot(l.body)).collect();
        self.looting = keep;
        // A body that has been moved (or settled where its band left it):
        // the looter goes on to where it is now.
        for l in self.looting.clone() {
            let Some(k) = self.squad.index(l.who) else { continue };
            let at = self.person_pos(l.body);
            if self.squad.goal[k].dist(at) > REACH && self.squad.at[k].dist(self.squad.goal[k]) < 1e-3 {
                self.send(l.who, at);
            }
        }
    }
}

/// A bandit's purse, worked out from who they are (so the same whenever
/// they're first met).
pub fn bandit_purse(seed: u64) -> u16 {
    (5 + super::rng::Rng::from_keys(&[seed, 0x5055_5253]).below(BANDIT_PURSE as usize)) as u16
}
