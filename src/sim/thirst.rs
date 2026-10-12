//! Water (the items design, 0.6: thirst, light). Where the squad can drink
//! and fill its skins: a town (its wells, pumps and rain barrels: anywhere
//! within its reach), or a base (its well, or water fetched for it). Rain falls on anyone
//! out in it (`condition.rs`). The terrain has no rivers or lakes yet, so
//! those come later; the sea doesn't count.

use super::condition::{Condition, SKIN_DRINKS, SKIN_QUENCH};
use super::geo::V2;
use super::items;
use super::person::PersonId;
use super::world::{World, HOUR};

/// A march this many hours of thirst beyond what they carry gets a warning.
pub const MARCH_WARN: f32 = 60.0;

impl World {
    /// Is there water to drink and fill skins from at `p`?
    pub fn water_at(&self, p: V2) -> bool {
        if self.settlements.iter().any(|s| s.pos.dist(p) <= s.reach) {
            return true;
        }
        // A base: its well, or water fetched for it (those living there
        // never go without).
        self.bases.iter().any(|b| b.at.dist(p) <= b.reach() + 10.0)
    }

    /// How many waterskins someone carries.
    pub fn skins_of(&self, pid: PersonId) -> f32 {
        let skin = items::id("waterskin");
        self.people[pid as usize].detail.as_ref().map(|d| d.gear.bag.iter().filter(|e| e.0 == skin).map(|e| e.1 as f32).sum()).unwrap_or(0.0)
    }

    /// The rain on someone at `t` (none under a roof): the hour's, read at
    /// the top of the hour.
    pub(super) fn rain_on(&self, pid: PersonId, t: f64) -> f32 {
        if self.shelter_of(pid) != super::condition::Shelter::Open {
            return 0.0;
        }
        let at = self.person_pos(pid);
        let w = self.weather_at(at, (t / HOUR).floor() * HOUR);
        w.rain.clamp(0.0, 1.0)
    }

    /// Has someone water to drink: at a source, or in their skins?
    pub(super) fn water_for(&self, pid: PersonId, c: &Condition) -> bool {
        c.water >= 1.0 || self.water_at(self.person_pos(pid))
    }

    /// Drink at `t`: their fill at a source (and their skins filled), else a
    /// drink from a skin. False if there was nothing to drink.
    pub(super) fn drink_water(&mut self, pid: PersonId, t: f64) -> bool {
        let at_source = self.water_at(self.person_pos(pid));
        let cap = self.skins_of(pid) * SKIN_DRINKS;
        self.settle_condition(pid, t);
        let Some(c) = self.people[pid as usize].cond.as_mut() else { return false };
        if at_source {
            c.thirst = 0.0;
            c.water = cap;
        } else if c.water >= 1.0 {
            c.water -= 1.0;
            c.thirst = (c.thirst - SKIN_QUENCH).max(0.0);
        } else {
            return false;
        }
        true
    }

    /// Squad members at water fill their skins (whenever they come by it).
    pub(super) fn fill_skins(&mut self) {
        for pid in self.squad.members.clone() {
            let cap = self.skins_of(pid) * SKIN_DRINKS;
            let at = self.person_pos(pid);
            let full = self.people[pid as usize].cond.as_ref().is_none_or(|c| c.water >= cap - 1e-3);
            if !full && self.water_at(at) {
                if let Some(c) = self.people[pid as usize].cond.as_mut() {
                    c.water = cap;
                }
            }
        }
    }

    /// Before a long march: does anyone carry too little water for it? A
    /// line in the log if so (0.6: the squad warns before a long march with
    /// little water).
    pub(super) fn warn_of_thirst(&mut self, to: V2) {
        if self.water_at(to) {
            return;
        }
        for pid in self.squad.members.clone() {
            let from = self.person_pos(pid);
            let hours = from.dist(to) / (self.member_speed(pid).max(0.3) * 3600.0);
            let Some(c) = self.people[pid as usize].cond.clone() else { continue };
            let walk = super::condition::THIRST_PER_HOUR * super::condition::THIRST_WALKING;
            let ahead = c.thirst + walk * hours - c.water * SKIN_QUENCH;
            if ahead >= MARCH_WARN && hours >= 2.0 {
                let name = self.name_of(pid);
                self.log.push_front((self.time, format!("{name} has little water for a march that long.")));
                self.log.truncate(14);
                return;
            }
        }
    }
}
