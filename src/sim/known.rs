//! What the squad knows of the world, for the map (UI-Map, #154): a bandit
//! camp is on the map once someone of the squad has seen it, or heard of it
//! in talk. Towns and roads are known from the start; ruins and lairs by
//! their own `found` mark (`ruins.rs`); herds only while in sight (drawing).
//!
//! The mark changes nothing that happens: it's what the window may show.
//! Seeing is the player exception (checked in the squad's step, like ruins
//! being found); hearing comes from talk.

use serde::{Deserialize, Serialize};

use super::group::GroupId;
use super::world::World;

/// How near one of the squad must come to a camp to have seen it, metres
/// (as near as a ruin is found from).
pub const CAMP_SEEN: f32 = super::ruins::FIND_AT;

/// What the squad knows of a camp.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Known {
    /// Nothing.
    #[default]
    No,
    /// Someone told them of it.
    Heard,
    /// One of them has seen it.
    Seen,
}

impl World {
    /// Camps the squad comes near are seen (the squad's step).
    pub(super) fn note_camps_seen(&mut self) {
        let spots: Vec<_> = self.squad.members.iter().filter(|&&m| !self.people[m as usize].dead).map(|&m| self.person_pos(m)).collect();
        for c in self.camps.iter_mut().filter(|c| c.known != Known::Seen) {
            if spots.iter().any(|p| p.dist(c.pos) <= CAMP_SEEN) {
                c.known = Known::Seen;
            }
        }
    }

    /// Someone told the squad of this band's camp.
    pub(super) fn hear_of_camp(&mut self, gid: GroupId) {
        for c in self.camps.iter_mut().filter(|c| c.group == gid) {
            c.known = c.known.max(Known::Heard);
        }
    }

    /// What the squad knows of this band's camp.
    pub fn camp_known(&self, gid: GroupId) -> Known {
        self.camps.iter().find(|c| c.group == gid).map(|c| c.known).unwrap_or_default()
    }
}
