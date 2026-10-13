//! Food going off (the items design, 0.6: spoilage). Every food the squad
//! carries has a shelf life in days, and goes fresh, then stale, then
//! spoiled. Each member keeps one freshness per food they carry (`Larder`:
//! the days left, as of a moment), beside their pack, which stays as it is;
//! food that comes in joins it, the two averaged by how many of each. It
//! goes off at a rate that is a value plus a rate like everything on the
//! condition timeline (rule 8): faster soaked through, slower in the cold
//! (the hour's temperature where they are), slower kept in a sealed
//! strongbox or oilskin; it's read again at the top of each hour with their
//! soaking (`soak.rs`), and the moments a food turns stale or spoils are
//! solved for. Stale food fills less; spoiled fills little and turns the
//! stomach. Members eat what's going off first.
//!
//! Only the squad's food goes off (like wear, rule 17): a town's or a
//! base's store keeps.

use serde::{Deserialize, Serialize};

use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::world::{World, DAY};

/// Raw food, for cooking (or a hound): eaten only when there's nothing
/// else, not first because it's going off.
pub const RAW: &[&str] = &["raw_meat", "fish"];

/// Shelf life in days, by food. TODO(items): a field on the food line
/// (`Kind::Food`) once the food data grows; the rest keep `SHELF_DEFAULT`.
pub const SHELF: &[(&str, f32)] = &[
    ("dried_fish", 30.0),
    ("salted_meat", 40.0),
    ("flatbread", 6.0),
    ("wild_berries", 4.0),
    ("mussels", 2.0),
    ("raw_meat", 2.0),
    ("egg", 7.0),
    ("fish", 2.0),
    ("milk", 2.0),
    ("cheese", 20.0),
    // Dishes: cooked food keeps a few days; bread for weeks, trail cake
    // for months (0.6).
    ("roast_meat", 3.0),
    ("spit_fish", 2.0),
    ("fried_puffcaps", 2.0),
    ("camp_porridge", 2.0),
    ("moorbell_tea", 1.0),
    ("sweetroot_stew", 3.0),
    ("fish_broth", 2.0),
    ("mussel_pot", 2.0),
    ("stonebean_stew", 3.0),
    ("hardgrain_bread", 21.0),
    ("shellhen_bake", 3.0),
    ("glazed_meat", 4.0),
    ("trail_cake", 90.0),
    ("stoneapple_cake", 6.0),
    ("firepod_stew", 3.0),
    // Drinks: ale and cider spoil; wine, spirits and cordial keep.
    ("hardgrain_ale", 8.0),
    ("stonebean_dark", 8.0),
    ("stoneapple_cider", 8.0),
    ("moorberry_wine", 3650.0),
    ("ashberry_wine", 3650.0),
    ("nightberry_wine", 3650.0),
    ("brine_wine", 3650.0),
    ("stone_spirit", 3650.0),
    ("sacrament_spirit", 3650.0),
    ("gold_cordial", 3650.0),
];
pub const SHELF_DEFAULT: f32 = 5.0;
/// Stale once this share of its shelf life is left.
pub const STALE_AT: f32 = 0.4;
/// What each stage fills, as a share of the fresh food's.
pub const FILLS: [f32; 3] = [1.0, 0.6, 0.3];
/// Going off faster soaked through, slower in the cold (below `COLD_BELOW`
/// deg C), slower in a sealed strongbox or oilskin.
pub const WET_SPOILS: f32 = 2.0;
pub const COLD_KEEPS: f32 = 0.5;
pub const COLD_BELOW: f32 = 5.0;
pub const SEALED_KEEPS: f32 = 0.5;

/// Fresh, stale or spoiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fresh {
    Fresh,
    Stale,
    Spoiled,
}

/// One member's freshness for one food: how many they had when last seen,
/// and the days left as of `at`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Larder {
    pub pid: PersonId,
    pub item: ItemId,
    pub count: u16,
    pub left: f32,
    pub at: f64,
}

/// A food's shelf life, days.
pub fn shelf_of(id: ItemId) -> f32 {
    let key = item(super::items::info(id).form).key;
    SHELF.iter().find(|s| s.0 == key).map(|s| s.1).unwrap_or(SHELF_DEFAULT)
}

/// Food, or a drink (ale and cider go off; wine and spirits keep).
pub fn is_food(id: ItemId) -> bool {
    matches!(item(id).kind, Kind::Food(_) | Kind::Drink(_))
}

/// Days of freshness left at `t`, going off at `rate` (days a day) since
/// it was settled.
pub fn left_at(l: &Larder, rate: f32, t: f64) -> f32 {
    l.left - rate * ((t - l.at).max(0.0) / DAY) as f32
}

/// Its stage with `left` days to go.
pub fn stage(id: ItemId, left: f32) -> Fresh {
    if left <= 0.0 {
        Fresh::Spoiled
    } else if left <= shelf_of(id) * STALE_AT {
        Fresh::Stale
    } else {
        Fresh::Fresh
    }
}

impl World {
    /// How fast `pid`'s food goes off now, days a day (before any sealed
    /// box): soaked through, or cold.
    pub(super) fn food_rate(&self, pid: PersonId, t: f64) -> f32 {
        // Switched off (`rules::Rules::food_spoils`): nothing goes off.
        if !self.rules.food_spoils {
            return 0.0;
        }
        let mut r = 1.0;
        if self.soak_at(pid, t).0 >= 1.0 - 1e-4 {
            r *= WET_SPOILS;
        }
        let at = self.person_pos(pid);
        let hour = (t / super::world::HOUR).floor() * super::world::HOUR;
        if self.weather_at(at, hour).temperature < COLD_BELOW {
            r *= COLD_KEEPS;
        }
        r
    }

    /// The rate for one food: what of it is kept in a sealed box goes off
    /// slower (the stack by its share).
    fn stack_rate(&self, pid: PersonId, it: ItemId, rate: f32) -> f32 {
        let Some(d) = self.people[pid as usize].detail.as_ref() else { return rate };
        let n: u16 = d.gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum();
        let boxed = super::soak::guarded(&d.gear, false).iter().find(|g| g.0 == it).map(|g| g.1).unwrap_or(0);
        let share = if n == 0 { 0.0 } else { boxed.min(n) as f32 / n as f32 };
        rate * (1.0 - share * (1.0 - SEALED_KEEPS))
    }

    /// The member's current rate (from their soaking).
    fn rate_of(&self, pid: PersonId) -> f32 {
        self.soaks.iter().find(|s| s.pid == pid).map(|s| s.food_rate).unwrap_or(1.0)
    }

    /// Keep `pid`'s larder in step with their pack: a food new to them is
    /// fresh from now; more of one joins it, the two averaged by how many of
    /// each; one eaten or given away leaves the rest as they were.
    pub(super) fn stamp_food(&mut self, pid: PersonId, t: f64) {
        let rate = self.rate_of(pid);
        let Some(d) = self.people[pid as usize].detail.as_ref() else { return };
        let mut held: Vec<(ItemId, u16)> = Vec::new();
        for e in d.gear.bag.iter().filter(|e| is_food(e.0)) {
            match held.iter_mut().find(|h| h.0 == e.0) {
                Some(h) => h.1 += e.1,
                None => held.push((e.0, e.1)),
            }
        }
        let rates: Vec<(ItemId, f32)> = held.iter().map(|h| (h.0, self.stack_rate(pid, h.0, rate))).collect();
        self.larder.retain(|l| l.pid != pid || held.iter().any(|h| h.0 == l.item));
        for (it, n) in held {
            let r = rates.iter().find(|x| x.0 == it).map(|x| x.1).unwrap_or(rate);
            match self.larder.iter_mut().find(|l| l.pid == pid && l.item == it) {
                Some(l) if n > l.count => {
                    let now = left_at(l, r, t);
                    l.left = (now * l.count as f32 + shelf_of(it) * (n - l.count) as f32) / n as f32;
                    l.at = t;
                    l.count = n;
                }
                Some(l) => l.count = n,
                None => self.larder.push(Larder { pid, item: it, count: n, left: shelf_of(it), at: t }),
            }
        }
    }

    /// Bring `pid`'s freshness forward to `t` at the old rate (before a new
    /// rate starts), saying what turned stale or spoiled.
    pub(super) fn settle_food(&mut self, pid: PersonId, rate: f32, t: f64) {
        let rates: Vec<(ItemId, f32)> = self.larder.iter().filter(|l| l.pid == pid).map(|l| (l.item, self.stack_rate(pid, l.item, rate))).collect();
        let mut turned: Vec<(ItemId, Fresh)> = Vec::new();
        for l in self.larder.iter_mut().filter(|l| l.pid == pid) {
            let r = rates.iter().find(|x| x.0 == l.item).map(|x| x.1).unwrap_or(rate);
            let was = stage(l.item, l.left);
            l.left = left_at(l, r, t);
            l.at = t;
            let now = stage(l.item, l.left);
            if now > was {
                turned.push((l.item, now));
            }
        }
        if turned.is_empty() || self.squad.index(pid).is_none() {
            return;
        }
        let name = self.name_of(pid);
        for (it, s) in turned {
            let how = if s == Fresh::Spoiled { "has spoiled" } else { "has gone stale" };
            self.log.push_front((t, format!("{name}'s {} {how}.", item(it).name.to_lowercase())));
        }
        self.log.truncate(14);
    }

    /// When the next of `pid`'s foods turns stale or spoils, going off at
    /// `rate`.
    pub(super) fn next_food_turn(&self, pid: PersonId, rate: f32) -> Option<f64> {
        let mut best: Option<f64> = None;
        for l in self.larder.iter().filter(|l| l.pid == pid) {
            let r = self.stack_rate(pid, l.item, rate);
            if r <= 0.0 {
                continue;
            }
            for mark in [shelf_of(l.item) * STALE_AT, 0.0] {
                if l.left > mark {
                    let tt = l.at + ((l.left - mark) / r) as f64 * DAY;
                    best = Some(best.map_or(tt, |b: f64| b.min(tt)));
                    break;
                }
            }
        }
        best
    }

    /// Days of freshness `pid`'s `it` has left at `t` (a full shelf life if
    /// it's new to them).
    pub fn days_left(&self, pid: PersonId, it: ItemId, t: f64) -> f32 {
        match self.larder.iter().find(|l| l.pid == pid && l.item == it) {
            Some(l) => left_at(l, self.stack_rate(pid, it, self.rate_of(pid)), t),
            None => shelf_of(it),
        }
    }

    /// How fresh `pid`'s `it` is at `t`.
    pub fn freshness(&self, pid: PersonId, it: ItemId, t: f64) -> Fresh {
        stage(it, self.days_left(pid, it, t))
    }

    /// Take one of `it` from `pid`'s pack: how fresh it was.
    pub(super) fn take_food(&mut self, pid: PersonId, it: ItemId, t: f64) -> Option<Fresh> {
        let s = self.freshness(pid, it, t);
        let d = self.people[pid as usize].detail.as_mut()?;
        if !d.gear.take(it) {
            return None;
        }
        if let Some(l) = self.larder.iter_mut().find(|l| l.pid == pid && l.item == it) {
            l.count = l.count.saturating_sub(1);
        }
        Some(s)
    }

    /// Of `pid`'s foods, one that's going off (stale but not spoiled), to
    /// eat before the rest.
    pub(super) fn food_going_off(&self, pid: PersonId, t: f64) -> Option<ItemId> {
        let d = self.people[pid as usize].detail.as_ref()?;
        let mut kinds: Vec<ItemId> = d.gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Food(_))).map(|e| e.0).collect();
        kinds.dedup();
        kinds.into_iter().filter(|&it| !RAW.contains(&item(it).key)).find(|&it| self.freshness(pid, it, t) == Fresh::Stale)
    }
}
