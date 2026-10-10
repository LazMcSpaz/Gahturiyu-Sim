//! News of your crimes, carried town to town by travellers.
//!
//! A bounty is owed in the town where the crime was seen, and only that town
//! knows at first. When someone sets out from a town that has heard, they
//! carry the news to wherever they're going, and it's known there from the
//! moment they arrive. Departures are decided hour by hour on the world's
//! timeline, and each journey's arrival is part of its schedule, so news
//! spreads the same way however the world is stepped.
//!
//! Not every traveller passes it on (`NEWS_CHANCE`).
//!
//! A town that has heard treats you as it would its own victims: lower
//! disposition, talk of the crime, and you can settle the bounty there.
//! Paying clears it everywhere.
//!
//! (A traveller who is waylaid on the road still counts as delivering the
//! news; news isn't tracked through ambushes yet.)

use super::group::Kind;
use super::rng::Rng;
use super::group::GroupId;
use super::settlement::SettlementId;
use super::world::World;

/// The chance that a traveller from a town that has heard passes the news on
/// where they're going (not everyone gossips about strangers' crimes).
pub const NEWS_CHANCE: f32 = 0.3;

impl World {
    /// When a town heard of the bounty owed in `origin`, if it has.
    pub fn heard_at(&self, town: SettlementId, origin: SettlementId) -> Option<f64> {
        self.news.get(&(town, origin)).copied()
    }

    /// Has this town heard of the bounty owed in `origin`, by now?
    pub fn has_heard(&self, town: SettlementId, origin: SettlementId) -> bool {
        // The town where it happened always knows.
        (town == origin && self.bounty.contains_key(&origin)) || self.heard_at(town, origin).map(|t| t <= self.time).unwrap_or(false)
    }

    /// The bounties a town knows of by now, by the town they're owed in.
    pub fn bounties_known_in(&self, town: SettlementId) -> Vec<(SettlementId, f32)> {
        let mut out: Vec<(SettlementId, f32)> = self.bounty.iter().filter(|(o, b)| **b > 0.0 && self.has_heard(town, **o)).map(|(o, b)| (*o, *b)).collect();
        out.sort_by_key(|o| o.0);
        out
    }

    /// The total a town holds against you.
    pub fn bounty_known_in(&self, town: SettlementId) -> f32 {
        self.bounties_known_in(town).iter().map(|b| b.1).sum()
    }

    /// How many towns have heard of the bounty owed in `origin`, by now.
    pub fn towns_heard(&self, origin: SettlementId) -> usize {
        (0..self.settlements.len() as SettlementId).filter(|&t| self.has_heard(t, origin)).count()
    }

    /// A crime has been seen in `town`: it knows from now (if it didn't).
    pub(super) fn crime_known(&mut self, town: SettlementId) {
        let t = self.time;
        let e = self.news.entry((town, town)).or_insert(t);
        *e = e.min(t);
    }

    /// A bounty is paid: the matter is closed everywhere.
    pub(super) fn bounty_settled(&mut self, origin: SettlementId) {
        self.bounty.remove(&origin);
        self.news.retain(|k, _| k.1 != origin);
        // What was stolen there is settled with it (`law.rs`).
        self.hot.retain(|h| h.town != origin);
    }

    /// A group has just set out from home: if home had heard anything by the
    /// time they leave, their destination hears it when they get there.
    pub(super) fn carry_news(&mut self, gid: GroupId) {
        self.carry_tidings(gid);
        if self.news.is_empty() {
            return;
        }
        let Some(g) = self.group(gid) else { return };
        let Kind::Journey { home } = g.kind else { return };
        let Some(leg) = g.legs.iter().find(|l| l.dest.map(|d| d != home).unwrap_or(false)) else { return };
        let (dest, depart, arrive) = (leg.dest.unwrap(), leg.depart, leg.arrive);
        let mut carried: Vec<SettlementId> = self.news.iter().filter(|((town, _), t)| *town == home && **t <= depart).map(|((_, origin), _)| *origin).collect();
        carried.sort_unstable();
        for origin in carried {
            // Whether this lot mention it is decided by who they are and what news it is.
            if Rng::from_keys(&[self.seed, gid as u64, origin as u64, 0x4E45_5753]).f32() >= NEWS_CHANCE {
                continue;
            }
            let e = self.news.entry((dest, origin)).or_insert(arrive);
            *e = e.min(arrive);
        }
    }
}
