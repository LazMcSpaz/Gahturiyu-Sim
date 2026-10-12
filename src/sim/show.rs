//! Showy jewellery (the items design, 0.4): rings, an amulet, a brooch, a
//! fine belt worn where they can be seen raise how people read your wealth.
//! They're better disposed to you for it (respect in talk), and bandits
//! weighing up travellers find the well-dressed more tempting. Plain things
//! (iron, leather) count for little: it's what they'd fetch.

use super::items::{item, Kind, Slot};
use super::person::PersonId;
use super::world::World;

/// The slots where finery is seen.
pub const SHOWN: [Slot; 5] = [Slot::Neck, Slot::Ring, Slot::Ring2, Slot::Brooch, Slot::Belt];
/// Disposition for each coin of finery shown, and the most it adds.
pub const RESPECT_PER_COIN: f32 = 1.0 / 40.0;
pub const RESPECT_MOST: f32 = 8.0;
/// Bandits reckon travellers this much weaker for each coin of finery they
/// see: they're worth the risk.
pub const TEMPTS_PER_COIN: f32 = 1.0 / 600.0;

impl World {
    /// What someone's finery on show would fetch, coin.
    pub fn shown_wealth(&self, pid: PersonId) -> f32 {
        let gear = self.people[pid as usize].kit();
        SHOWN.iter().filter_map(|&s| gear.in_slot(s)).filter(|&id| matches!(item(id).kind, Kind::Trinket) || item(id).slot.accessory()).map(|id| item(id).value).sum()
    }

    /// How much better disposed people are to someone for their finery.
    pub fn finery_respect(&self, pid: PersonId) -> f32 {
        (self.shown_wealth(pid) * RESPECT_PER_COIN).min(RESPECT_MOST)
    }
}
