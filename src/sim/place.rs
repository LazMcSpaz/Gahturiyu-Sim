//! What kind of place a spot is (P26, P15): in a town, just outside one,
//! at a bandit camp, at the squad's outpost, at a ruin or lair, on a road,
//! on the water, or out in the wilds.
//!
//! One answer, read off the world as it stands: talk uses it for what can
//! be asked where, and the window can use it for the town panel. Nothing
//! is stored.

use serde::{Deserialize, Serialize};

use super::base::BaseId;
use super::geo::V2;
use super::group::GroupId;
use super::settlement::SettlementId;
use super::world::World;

/// How far past a town's edge its fields, woodlots and outskirts reach,
/// metres (the same as `World::town_at`).
pub const OUTSKIRTS: f32 = 150.0;
/// This near a ruin or lair is at it, metres.
pub const AT_RUIN: f32 = 40.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Among a town's buildings (or out on its stilts).
    Town(SettlementId),
    /// Past its edge but still its ground: fields, woodlot, the road in.
    Outskirts(SettlementId),
    /// On a bandit camp's ground.
    Camp(GroupId),
    /// Within the squad's outpost.
    Base(BaseId),
    /// At a ruin or a beast's lair (`World::ruins` id).
    Ruin(u32),
    /// On a road, between towns.
    Road,
    /// Out on the sea.
    Water,
    /// None of the above.
    Wilds,
}

impl Place {
    /// The tag lines are chosen by (`data/lines/FORMAT.md`): `place=town`
    /// and so on.
    pub fn key(self) -> &'static str {
        match self {
            Place::Town(_) => "town",
            Place::Outskirts(_) => "outskirts",
            Place::Camp(_) => "camp",
            Place::Base(_) => "base",
            Place::Ruin(_) => "ruin",
            Place::Road => "road",
            Place::Water => "water",
            Place::Wilds => "wilds",
        }
    }

    /// The town this is in or just outside, if any.
    pub fn town(self) -> Option<SettlementId> {
        match self {
            Place::Town(t) | Place::Outskirts(t) => Some(t),
            _ => None,
        }
    }

    /// Among people: a town or the outpost.
    pub fn settled(self) -> bool {
        matches!(self, Place::Town(_) | Place::Base(_))
    }
}

impl World {
    /// What kind of place `p` is. A town's own ground comes first, then the
    /// outpost, a camp, a ruin, a town's outskirts, the water, a road, and
    /// otherwise the wilds.
    pub fn place_at(&self, p: V2) -> Place {
        let near = |extra: f32| self.settlements.iter().filter(|s| s.pos.dist(p) <= s.reach + extra).min_by(|a, b| a.pos.dist(p).total_cmp(&b.pos.dist(p))).map(|s| s.id);
        if let Some(t) = near(0.0) {
            return Place::Town(t);
        }
        if let Some(b) = self.base_at(p) {
            return Place::Base(b);
        }
        if let Some(c) = self.camps.iter().find(|c| c.pos.dist(p) <= super::encounters::GUARD_RING) {
            return Place::Camp(c.group);
        }
        if let Some(r) = self.ruins.iter().find(|r| r.pos.dist(p) <= AT_RUIN) {
            return Place::Ruin(r.id);
        }
        if let Some(t) = near(OUTSKIRTS) {
            return Place::Outskirts(t);
        }
        if self.terrain.is_sea(p) && !self.on_walkway(p) {
            return Place::Water;
        }
        if self.terrain.on_road(p) {
            return Place::Road;
        }
        Place::Wilds
    }
}
