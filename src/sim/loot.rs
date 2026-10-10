//! Looting the beaten (the playable-MVP list): a squad member sent to a
//! knocked-out or dead foe goes over to them, and while standing by can take
//! what they wear and carry, piece by piece or all at once. Kenshi's money
//! and gear come mostly this way.
//!
//! Who can be looted: anyone down (out cold or dead) who was against you:
//! bandits, and members of any group that fought the squad. Townsfolk aren't
//! fair game (stealing from them is a crime the town's watch deals with).
//!
//! The same going-through serves containers in buildings (`containers.rs`):
//! a `Looting` has a `Source`, a body or a chest, and taking from either goes
//! through `take_from`. Taking from a container is theft (`took_from`).

use serde::{Deserialize, Serialize};

use super::containers::ContainerId;
use super::inventory::Entry;
use super::items::{self, item, Slot, SLOTS};
use super::person::PersonId;
use super::squad::REACH;
use super::world::World;

/// What's being gone through.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Source {
    /// A beaten foe.
    Body(PersonId),
    /// A container in a building.
    Chest(ContainerId),
}

/// A squad member sent to go through someone's (or something's) things.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Looting {
    pub who: PersonId,
    pub from: Source,
}

/// One thing on a body: worn in a slot, or in their pack (by entry index).
/// In a container, everything is `Pack`.
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

    /// Why a body can't be gone through, in a line for the player.
    pub fn why_cant_loot(&self, body: PersonId) -> String {
        let p = &self.people[body as usize];
        let name = p.name().unwrap_or("They").to_string();
        if p.in_squad {
            return format!("{name} is one of yours: use their pack.");
        }
        if self.fighting.contains_key(&body) {
            return format!("{name} is still fighting.");
        }
        if !self.is_down(body) {
            return format!("{name} isn't down: they're on their feet (beaten foes get up again after a while).");
        }
        let hostile = p.bandit || self.group_of[body as usize].and_then(|g| self.group(g)).is_some_and(|g| g.hostile);
        if !hostile {
            return format!("Only beaten enemies can be gone through; {name} isn't one.");
        }
        format!("There's nothing on {name}.")
    }

    /// Can this still be gone through (a body still down; a container open)?
    fn source_ok(&self, src: Source) -> bool {
        match src {
            Source::Body(b) => self.can_loot(b),
            Source::Chest(c) => self.container(c).is_some() && !self.container_locked(c),
        }
    }

    fn source_pos(&self, src: Source) -> Option<super::geo::V2> {
        match src {
            Source::Body(b) => Some(self.person_pos(b)),
            Source::Chest(c) => self.container(c).map(|c| c.pos),
        }
    }

    /// Send a squad member to go through a body's things.
    pub fn order_loot(&mut self, who: PersonId, body: PersonId) -> bool {
        if !self.can_loot(body) {
            return false;
        }
        if self.squad.index(who).is_none() || self.is_down(who) {
            return false;
        }
        let pos = self.person_pos(body);
        self.send(who, pos);
        self.rouse(who);
        self.looting.retain(|l| l.who != who);
        self.looting.push(Looting { who, from: Source::Body(body) });
        true
    }

    /// What this member is standing at and going through, if anything.
    pub fn source_now(&self, who: PersonId) -> Option<Source> {
        let l = self.looting.iter().find(|l| l.who == who)?;
        let k = self.squad.index(who)?;
        let at = self.source_pos(l.from)?;
        let reach = match l.from {
            Source::Body(_) => REACH * 1.5,
            // A step in front of it, plus its own size.
            Source::Chest(_) => REACH * 1.2,
        };
        (self.squad.at[k].dist(at) <= reach && self.source_ok(l.from) && self.can_act(who)).then_some(l.from)
    }

    /// The body this member is standing over and going through, if any.
    pub fn looting_now(&self, who: PersonId) -> Option<PersonId> {
        match self.source_now(who)? {
            Source::Body(b) => Some(b),
            Source::Chest(_) => None,
        }
    }

    /// Stop going through a body or a container (walked off, or done).
    pub fn stop_looting(&mut self, who: PersonId) {
        self.looting.retain(|l| l.who != who);
    }

    /// Everything on a body: (where, item, how many).
    pub fn loot_of(&self, body: PersonId) -> Vec<(LootRef, items::ItemId, u16)> {
        self.contents(Source::Body(body))
    }

    /// Everything in a body's kit or a container: (where, item, how many).
    pub fn contents(&self, src: Source) -> Vec<(LootRef, items::ItemId, u16)> {
        match src {
            Source::Body(body) => {
                let Some(d) = self.people[body as usize].detail.as_ref() else { return Vec::new() };
                let mut out: Vec<(LootRef, items::ItemId, u16)> = SLOTS.iter().filter_map(|&s| d.gear.in_slot(s).map(|it| (LootRef::Worn(s), it, 1))).collect();
                out.extend(d.gear.bag.iter().enumerate().map(|(k, e)| (LootRef::Pack(k), e.0, e.1)));
                out
            }
            Source::Chest(c) => self.container(c).map(|c| c.items.iter().enumerate().map(|(k, e)| (LootRef::Pack(k), e.0, e.1)).collect()).unwrap_or_default(),
        }
    }

    /// Take one thing (a whole stack, from the pack) off a body.
    pub fn take_loot(&mut self, who: PersonId, body: PersonId, what: LootRef) -> bool {
        self.take_from(who, Source::Body(body), what)
    }

    /// Take one thing (a whole stack) from a body or a container.
    pub fn take_from(&mut self, who: PersonId, src: Source, what: LootRef) -> bool {
        if self.source_now(who) != Some(src) {
            return false;
        }
        let taken: Vec<Entry> = match (src, what) {
            (Source::Body(body), LootRef::Worn(s)) => {
                let Some(d) = self.people[body as usize].detail.as_mut() else { return false };
                let Some(it) = d.gear.in_slot(s) else { return false };
                let piece = d.gear.piece(s).copied();
                d.gear.discard(s);
                vec![Entry(it, 1, piece)]
            }
            (Source::Body(body), LootRef::Pack(k)) => {
                let Some(d) = self.people[body as usize].detail.as_mut() else { return false };
                if k >= d.gear.bag.len() {
                    return false;
                }
                vec![d.gear.bag.remove(k)]
            }
            (Source::Chest(c), LootRef::Pack(k)) => {
                let Some(c) = self.containers.get_mut(&c) else { return false };
                if k >= c.items.len() {
                    return false;
                }
                vec![c.items.remove(k)]
            }
            (Source::Chest(_), LootRef::Worn(_)) => return false,
        };
        if let Source::Body(body) = src {
            self.people[body as usize].recompute_might();
        }
        let mut names = Vec::new();
        let mut worth = 0.0;
        if let Some(dd) = self.people[who as usize].detail.as_mut() {
            for e in &taken {
                match e.2 {
                    Some(pc) => dd.gear.add_piece(e.0, pc),
                    None => dd.gear.add(e.0, e.1),
                }
                worth += item(e.0).value * e.1 as f32;
                names.push(if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() });
            }
        }
        self.people[who as usize].recompute_might();
        self.settle_condition(who, self.time);
        let a = self.people[who as usize].name().unwrap_or("someone").to_string();
        let from = match src {
            Source::Body(b) => self.people[b as usize].name().unwrap_or("them").to_string(),
            Source::Chest(c) => format!("a {}", self.container(c).map(|c| c.what.name()).unwrap_or("container")),
        };
        self.log.push_front((self.time, format!("{a} takes {} from {from}.", names.join(", "))));
        self.log.truncate(14);
        if let Source::Chest(c) = src {
            self.took_from(who, c, worth);
        }
        true
    }

    /// Put something from a member's pack (a whole stack, entry `k`) into the
    /// container they have open. Not a crime, but what's put in someone
    /// else's chest is theirs now (taking it back is taking from them).
    pub fn put_in(&mut self, who: PersonId, k: usize) -> bool {
        let Some(Source::Chest(c)) = self.source_now(who) else { return false };
        let Some(d) = self.people[who as usize].detail.as_mut() else { return false };
        if k >= d.gear.bag.len() {
            return false;
        }
        let e = d.gear.bag.remove(k);
        let Some(chest) = self.containers.get_mut(&c) else {
            if let Some(d) = self.people[who as usize].detail.as_mut() {
                d.gear.bag.insert(k, e);
            }
            return false;
        };
        match chest.items.iter_mut().find(|x| x.0 == e.0 && x.2.is_none() && e.2.is_none()) {
            Some(x) => x.1 += e.1,
            None => chest.items.push(e),
        }
        let what = chest.what.name();
        self.people[who as usize].recompute_might();
        self.settle_condition(who, self.time);
        let a = self.people[who as usize].name().unwrap_or("someone").to_string();
        let n = if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() };
        self.log.push_front((self.time, format!("{a} puts {n} in the {what}.")));
        self.log.truncate(14);
        true
    }

    /// Take everything off a body.
    pub fn take_all_loot(&mut self, who: PersonId, body: PersonId) -> usize {
        self.take_all_from(who, Source::Body(body))
    }

    /// Take everything from a body or a container (stops if caught).
    pub fn take_all_from(&mut self, who: PersonId, src: Source) -> usize {
        let mut n = 0;
        while let Some(&(what, _, _)) = self.contents(src).first() {
            if !self.take_from(who, src, what) {
                break;
            }
            n += 1;
            // Caught in the act: arrested or chased off, either way it stops here.
            if self.source_now(who) != Some(src) {
                break;
            }
        }
        n
    }

    /// Drop lootings whose looter has gone (left the squad, or the body
    /// got up and walked off, or the container is gone).
    pub(super) fn tidy_looting(&mut self) {
        let keep: Vec<Looting> = self.looting.iter().copied().filter(|l| self.squad.index(l.who).is_some() && self.source_ok(l.from)).collect();
        self.looting = keep;
        // A body that has been moved (or settled where its band left it):
        // the looter goes on to where it is now.
        for l in self.looting.clone() {
            let (Some(k), Source::Body(body)) = (self.squad.index(l.who), l.from) else { continue };
            let at = self.person_pos(body);
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
