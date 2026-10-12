//! Blade poisons (the items design, Part 4 rule 9): a brewed bottle with
//! something harmful in it coats a squad member's hand weapon for a few
//! hits that wound (more with Alchemy skill). The coating is kept here,
//! comes into a fight on their fighter (`Fighter::coating`) and is written
//! back with the hits left (`fights.rs`). Only blows that wound carry it:
//! armour that stops the blow stops the poison (`combat::resolve_blow`).
//!
//! TODO(law): using poison is a crime where a town's customs say so; the
//! hook belongs with the customs (`culture.rs`), not here.

use serde::{Deserialize, Serialize};

use super::items::herbs::Mix;
use super::person::PersonId;
use super::world::World;

/// Hits a coating lasts for an average alchemist, and one more for every
/// `COAT_SKILL` points of Alchemy.
pub const COAT_HITS: u16 = 3;
pub const COAT_SKILL: f32 = 25.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Coating {
    pub pid: PersonId,
    pub mix: Mix,
    pub hits: u16,
}

impl World {
    /// What's on someone's blade, if anything.
    pub fn coating_of(&self, pid: PersonId) -> Option<&Coating> {
        self.coatings.iter().find(|c| c.pid == pid && c.hits > 0)
    }

    /// Why `pid` can't coat their weapon from pack entry `k`, if they can't.
    pub fn why_cant_coat(&self, pid: PersonId, k: usize) -> Option<&'static str> {
        let d = self.people[pid as usize].detail.as_ref()?;
        let mix = d.gear.bag.get(k).and_then(|e| e.2).and_then(|p| p.mix);
        if !mix.is_some_and(|m| m.harms()) {
            return Some("only a brew with something harmful in it will do");
        }
        if d.gear.in_slot(super::items::Slot::MainHand).and_then(|id| super::items::item(id).weapon().copied()).is_none_or(|w| w.range > 0.0) {
            return Some("they need a hand weapon to coat");
        }
        if self.fighting.contains_key(&pid) {
            return Some("not in the middle of a fight");
        }
        None
    }

    /// Coat `pid`'s hand weapon from the brewed bottle at pack entry `k`
    /// (the bottle is used up; a coating already on is wiped off).
    pub fn coat_weapon(&mut self, pid: PersonId, k: usize) -> bool {
        if self.why_cant_coat(pid, k).is_some() {
            return false;
        }
        let skill = self.people[pid as usize].effective_stats().skill(super::stats::Skill::Alchemy);
        let Some(d) = self.people[pid as usize].detail.as_mut() else { return false };
        let Some(mix) = d.gear.bag.get(k).and_then(|e| e.2).and_then(|p| p.mix) else { return false };
        if !d.gear.take_mix(&mix) {
            return false;
        }
        self.people[pid as usize].recompute_might();
        let hits = COAT_HITS + (skill / COAT_SKILL) as u16;
        self.coatings.retain(|c| c.pid != pid);
        self.coatings.push(Coating { pid, mix, hits });
        let name = self.name_of(pid);
        self.say(self.time, format!("{name} coats their blade with {} ({hits} blows).", mix.name().to_lowercase()));
        true
    }
}
