//! Livestock: the animals people keep, as animals.
//!
//! A pen is a group of one kind of animal with a home spot (a pasture, coop,
//! stable or mooring), an owner, a store of feed, a hunger level and what
//! they've produced. As everywhere else, none of it is counted day by day:
//! a pen's state is written down at one moment and worked out for any later
//! one (feed running down, hunger rising once it's gone, production falling
//! with hunger, the herd growing toward its limit). Every change — feeding,
//! a cull, the dawn stock-take — settles the pen at that moment.
//!
//! Nothing here decides who feeds a pen, who collects from it, what an egg
//! is worth or who the owner is: those are for the society side, which
//! reads and drives pens through the plain functions at the bottom.
//!
//! Tamed Ridgehounds are here too: they follow their owner, fight beside
//! them and get hungry.

use serde::{Deserialize, Serialize};

use super::super::geo::V2;
use super::super::person::PersonId;
use super::super::rng::{self, Rng};
use super::super::settlement::SettlementId;
use super::super::world::{World, DAY};
use super::herd::{h_yields, Hurt};
use super::region::{logistic, region_of, RegionId};
use super::species::{Sp, Species};

/// Days without feed before a pen is as hungry as it gets.
pub const STARVE_DAYS: f32 = 3.0;
/// Days of feeding to bring a starved pen back.
pub const RECOVER_DAYS: f32 = 1.0;
/// Share of their produce the starving stop giving.
pub const HUNGER_LOSS: f32 = 0.8;
/// How many dawn stock-takes a pen remembers (for "what has it made since").
pub const PEN_HISTORY: usize = 24;
/// How hungry a tamed hound gets in a day with nothing to eat (1 = starving).
pub const HOUND_HUNGER_PER_DAY: f32 = 0.5;
/// Days a hound stays with an owner who lets it starve.
pub const HOUND_LEAVES_DAYS: f64 = 2.0;
/// How much weaker a starving hound's bite is.
pub const HOUND_STARVED: f32 = 0.6;
/// A fed, grown tamed hound next to a wild one.
pub const HOUND_SIZE: f32 = 1.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Pen {
    /// Its place in `Animals::pens`.
    pub id: u32,
    pub sp: Sp,
    /// The town it stands by, if any.
    pub town: Option<SettlementId>,
    /// Whose it is: an opaque number for the society side to fill in
    /// (0 = nobody yet).
    pub owner: u64,
    /// The pasture, coop, stable or mooring, and how far it reaches.
    pub home: V2,
    pub radius: f32,
    pub region: RegionId,
    pub seed: u64,
    /// The most animals it grows to.
    pub limit: f32,

    // --- As of `at` -----------------------------------------------------------
    pub at: f64,
    pub count: f32,
    /// 0 = well fed, 1 = starving.
    pub hunger: f32,
    /// Feed in store (one unit feeds one Turiyu for a day).
    pub feed: f32,
    /// Everything made since the pen began, one total per daily product.
    pub made: Vec<f32>,
    /// The totals at earlier stock-takes: (when, totals).
    pub past: Vec<(f64, Vec<f32>)>,
}

/// A pen's state at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct PenNow {
    pub count: f32,
    pub hunger: f32,
    pub feed: f32,
    pub made: Vec<f32>,
}

/// Hunger moving steadily toward `target` for `days`: where it ends, and
/// its average over the stretch times the days (for adding up production).
fn ramp(h0: f32, target: f32, per_day: f32, days: f64) -> (f32, f64) {
    if days <= 0.0 {
        return (h0, 0.0);
    }
    let gap = (target - h0).abs() as f64;
    let reach = if per_day > 0.0 { gap / per_day as f64 } else { f64::INFINITY };
    if days <= reach {
        let h1 = h0 + (target - h0).signum() * per_day * days as f32;
        (h1, (h0 + h1) as f64 * 0.5 * days)
    } else {
        (target, (h0 + target) as f64 * 0.5 * reach + target as f64 * (days - reach))
    }
}

impl Pen {
    pub fn def(&self) -> &'static Species {
        self.sp.def()
    }

    /// Whole animals at the last stock-take.
    pub fn head(&self) -> usize {
        (self.count + 1e-3).floor().max(0.0) as usize
    }

    /// The pen at time `t`, worked out from how it stood at `self.at`.
    pub fn now(&self, t: f64) -> PenNow {
        let d = self.def();
        let days = ((t - self.at) / DAY).max(0.0);
        let head = self.head() as f32;
        // What they eat from the store each day (the rest they find at pasture).
        let eaten = head * d.feed * (1.0 - d.forage);
        let fed_days = if eaten <= 0.0 {
            f64::INFINITY
        } else {
            (self.feed / eaten) as f64
        };
        let worst = (1.0 - d.forage).clamp(0.0, 1.0);
        let fed = days.min(fed_days);
        let (h1, a1) = ramp(self.hunger, 0.0, 1.0 / RECOVER_DAYS, fed);
        let (h2, a2) = ramp(h1, worst, 1.0 / STARVE_DAYS, days - fed);
        let good_days = (days - HUNGER_LOSS as f64 * (a1 + a2)).max(0.0);
        let made = d.daily.iter().enumerate().map(|(k, (_, per))| self.made.get(k).copied().unwrap_or(0.0) + head * per * good_days as f32).collect();
        let feed = if eaten <= 0.0 { self.feed } else { (self.feed - eaten * fed as f32).max(0.0) };
        // The hungry don't breed.
        let rate = d.growth as f64 * (1.0 - self.hunger).max(0.0) as f64 / DAY;
        let count = logistic(self.count, self.limit + 0.5, rate, t - self.at).min(self.limit.max(self.count));
        PenNow { count, hunger: h2, feed, made }
    }

    /// Write the pen's state down as of `t` (a stock-take).
    pub fn settle(&mut self, t: f64) {
        if t <= self.at {
            return;
        }
        let now = self.now(t);
        self.count = now.count;
        self.hunger = now.hunger;
        self.feed = now.feed;
        self.made = now.made;
        self.at = t;
    }

    fn remember(&mut self) {
        self.past.push((self.at, self.made.clone()));
        if self.past.len() > PEN_HISTORY {
            self.past.remove(0);
        }
    }

    /// The running totals at time `t`: exact from the last stock-take on,
    /// read between remembered stock-takes before that.
    fn made_at(&self, t: f64) -> Vec<f32> {
        if t >= self.at {
            return self.now(t).made;
        }
        let mut after = (self.at, &self.made);
        for (when, totals) in self.past.iter().rev() {
            if *when <= t {
                let f = ((t - when) / (after.0 - when).max(1e-6)) as f32;
                return totals.iter().zip(after.1.iter()).map(|(a, b)| a + (b - a) * f.clamp(0.0, 1.0)).collect();
            }
            after = (*when, totals);
        }
        after.1.clone()
    }

    /// What the pen has produced between `since` and `now`: names and amounts.
    pub fn yield_between(&self, since: f64, now: f64) -> Vec<(&'static str, f32)> {
        let (a, b) = (self.made_at(since.min(now)), self.made_at(now));
        self.def().daily.iter().enumerate().map(|(k, (name, _))| (*name, (b[k] - a[k]).max(0.0))).collect()
    }

    /// Where the animal in place `k` stands at time `t`.
    pub fn animal_at(&self, k: usize, t: f64) -> V2 {
        let mut r = Rng::from_keys(&[self.seed, k as u64, 0x50_454E]);
        let a = r.f32() * std::f32::consts::TAU;
        let rad = self.radius * 0.85 * r.f32().sqrt();
        let phase = r.f32() * 100.0;
        let tt = (t / 120.0) as f32 + phase;
        let drift = self.def().looks.len.clamp(0.2, 1.5) * 0.5;
        self.home.add(V2::new(a.cos(), a.sin()).scale(rad)).add(V2::new(tt.sin(), (tt * 0.7).cos()).scale(drift))
    }
}

/// A pack animal out of its pen, following someone.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Led {
    pub pen: u32,
    pub leader: PersonId,
    /// What it's carrying, kilograms (for whoever loads it to keep).
    pub load: f32,
}

/// A tamed Ridgehound.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hound {
    pub id: u32,
    pub owner: PersonId,
    pub seed: u64,
    pub size: f32,
    pub hurt: Hurt,
    /// Hunger as of `fed_at` (0 = fed, 1 = starving); it climbs from there.
    pub hunger: f32,
    pub fed_at: f64,
    pub tamed_at: f64,
    /// Dead, or gone back to the wild.
    pub gone: bool,
}

impl Hound {
    pub fn hunger_at(&self, t: f64) -> f32 {
        (self.hunger + HOUND_HUNGER_PER_DAY * ((t - self.fed_at).max(0.0) / DAY) as f32).clamp(0.0, 1.0)
    }

    /// When an unfed hound gives up on its owner.
    pub fn leaves_at(&self) -> f64 {
        self.fed_at + ((1.0 - self.hunger) / HOUND_HUNGER_PER_DAY) as f64 * DAY + HOUND_LEAVES_DAYS * DAY
    }

    /// How hard it bites: a starving hound is a weak one.
    pub fn power(&self, t: f64) -> f32 {
        if self.hunger_at(t) >= 1.0 {
            HOUND_STARVED
        } else {
            1.0
        }
    }
}

impl World {
    // ---- Pens: reading --------------------------------------------------------

    pub fn pen(&self, id: u32) -> Option<&Pen> {
        self.animals.pens.get(id as usize)
    }

    /// A pen's head count, hunger, feed and totals right now.
    pub fn pen_now(&self, id: u32) -> Option<PenNow> {
        self.pen(id).map(|p| p.now(self.time))
    }

    /// The pens standing by a town ("which herds belong to this town").
    pub fn pens_of_town(&self, town: SettlementId) -> Vec<u32> {
        self.animals.pens.iter().filter(|p| p.town == Some(town)).map(|p| p.id).collect()
    }

    /// The pens with this owner number.
    pub fn pens_of_owner(&self, owner: u64) -> Vec<u32> {
        self.animals.pens.iter().filter(|p| p.owner == owner).map(|p| p.id).collect()
    }

    /// What a pen has produced since time `since` (eggs, milk, oil): names
    /// and amounts. Reading it takes nothing away; whoever collects keeps
    /// their own note of when they last did.
    pub fn pen_yield_since(&self, id: u32, since: f64) -> Vec<(&'static str, f32)> {
        self.pen_yield_between(id, since, self.time)
    }

    /// What a pen produced between two moments (for callers on the world's
    /// timeline, who work to the top of the hour and not to "now").
    pub fn pen_yield_between(&self, id: u32, since: f64, until: f64) -> Vec<(&'static str, f32)> {
        self.pen(id).map(|p| p.yield_between(since, until)).unwrap_or_default()
    }

    /// A pen's head count, hunger, feed and totals at a given moment.
    pub fn pen_at(&self, id: u32, t: f64) -> Option<PenNow> {
        self.pen(id).map(|p| p.now(t))
    }

    /// The same for one animal of the pen: its share.
    pub fn animal_yield_since(&self, id: u32, since: f64) -> Vec<(&'static str, f32)> {
        let Some(p) = self.pen(id) else { return Vec::new() };
        let head = (p.now(self.time).count.floor()).max(1.0);
        p.yield_between(since, self.time).into_iter().map(|(k, a)| (k, a / head)).collect()
    }

    /// Raftbacks afloat, as places to stand: where each one lies and how
    /// far its back reaches. (A Raftback can serve as a floating platform;
    /// what's built or done on one is for others.)
    pub fn floating_platforms(&self) -> Vec<(V2, f32)> {
        let t = self.time;
        let mut out = Vec::new();
        for p in self.animals.pens.iter().filter(|p| p.sp == Sp::Raftback) {
            let lk = &p.def().looks;
            for k in 0..p.now(t).count.floor() as usize {
                out.push((p.animal_at(k, t), lk.wide * 0.5));
            }
        }
        out
    }

    /// What one pack animal can carry, kilograms.
    pub fn plodder_capacity(&self) -> f32 {
        Sp::Plodder.def().carry
    }

    // ---- Pens: driving --------------------------------------------------------

    /// A new pen of `count` animals at `home`. Returns its id.
    pub fn add_pen(&mut self, sp: Sp, home: V2, town: Option<SettlementId>, limit: f32, count: f32) -> u32 {
        let id = self.animals.pens.len() as u32;
        let d = sp.def();
        let radius = match sp {
            Sp::Raftback => 14.0,
            Sp::Shellhen => 5.0,
            _ => (4.0 + limit.sqrt() * d.looks.len * 1.6).min(26.0),
        };
        self.animals.pens.push(Pen {
            id,
            sp,
            town,
            owner: 0,
            home,
            radius,
            region: region_of(home),
            seed: rng::key(&[self.seed, id as u64, 0x50_454E_5345]),
            limit,
            at: self.time,
            count: count.min(limit),
            hunger: 0.0,
            feed: 0.0,
            made: vec![0.0; d.daily.len()],
            past: Vec::new(),
        });
        id
    }

    pub fn set_pen_owner(&mut self, id: u32, owner: u64) {
        if let Some(p) = self.animals.pens.get_mut(id as usize) {
            p.owner = owner;
        }
    }

    /// The most animals a pen grows to.
    pub fn set_pen_limit(&mut self, id: u32, limit: f32) {
        self.set_pen_limit_at(id, limit, self.time)
    }

    /// The same, as of a given moment. (Every `_at` here is for callers on
    /// the world's timeline: an hourly or dawn tally should pass its own
    /// moment, not act at whatever time the step happens to have reached.
    /// A moment earlier than the pen's last change counts as that change.)
    pub fn set_pen_limit_at(&mut self, id: u32, limit: f32, t: f64) {
        if let Some(p) = self.animals.pens.get_mut(id as usize) {
            p.settle(t);
            p.limit = limit.max(0.0);
        }
    }

    /// Put feed into a pen's store (one unit feeds one Turiyu for a day;
    /// bigger animals eat more). Returns false if there's no such pen.
    pub fn feed_pen(&mut self, id: u32, amount: f32) -> bool {
        self.feed_pen_at(id, amount, self.time)
    }

    pub fn feed_pen_at(&mut self, id: u32, amount: f32, t: f64) -> bool {
        let Some(p) = self.animals.pens.get_mut(id as usize) else { return false };
        p.settle(t);
        p.feed += amount.max(0.0);
        true
    }

    /// Kill one animal of a pen. Returns what it gives (names and amounts),
    /// or nothing if the pen is empty.
    pub fn cull_pen(&mut self, id: u32) -> Vec<(&'static str, f32)> {
        self.cull_pen_at(id, self.time)
    }

    pub fn cull_pen_at(&mut self, id: u32, t: f64) -> Vec<(&'static str, f32)> {
        let Some(p) = self.animals.pens.get_mut(id as usize) else { return Vec::new() };
        p.settle(t);
        if p.head() == 0 {
            return Vec::new();
        }
        p.count -= 1.0;
        let (sp, head) = (p.sp, p.head());
        // A pack animal that was out being led may have been the one.
        let mut out = 0;
        self.animals.led.retain(|l| {
            if l.pen != id {
                return true;
            }
            out += 1;
            out <= head
        });
        h_yields(sp, 1.0)
    }

    /// A pack animal from this pen follows `leader` from now on. False if
    /// the pen has none to spare (or isn't a pen of pack animals).
    pub fn lead_plodder(&mut self, pen: u32, leader: PersonId) -> bool {
        let t = self.time;
        let Some(p) = self.animals.pens.get(pen as usize) else { return false };
        let out = self.animals.led.iter().filter(|l| l.pen == pen).count();
        if p.def().carry <= 0.0 || p.now(t).count.floor() as usize <= out {
            return false;
        }
        self.animals.led.push(Led { pen, leader, load: 0.0 });
        true
    }

    /// Everything `leader` is leading goes back to its pen.
    pub fn release_plodders(&mut self, leader: PersonId) {
        self.animals.led.retain(|l| l.leader != leader);
    }

    /// Where a led pack animal walks: at its leader's shoulder.
    pub fn led_pos(&self, k: usize) -> Option<V2> {
        let l = self.animals.led.get(k)?;
        let n = self.animals.led.iter().take(k).filter(|x| x.leader == l.leader).count() as f32;
        Some(self.person_pos(l.leader).add(V2::new(-2.2 - n * 2.6, 1.4)))
    }

    /// The dawn stock-take of every pen.
    pub(super) fn settle_pens(&mut self, t: f64) {
        for p in &mut self.animals.pens {
            p.settle(t);
            p.remember();
        }
        let pens = &self.animals.pens;
        let people = &self.people;
        self.animals.led.retain(|l| !people[l.leader as usize].dead && pens[l.pen as usize].head() > 0);
    }

    /// Domestic Turiyu at pasture in a region.
    pub(super) fn turiyu_at_pasture(&self, r: RegionId, t: f64) -> f32 {
        self.animals.pens.iter().filter(|p| p.sp == Sp::Turiyu && p.region == r).map(|p| p.now(t).count.floor()).sum()
    }

    // ---- Tamed hounds ---------------------------------------------------------

    /// Every tamed hound still with its owner.
    pub fn tamed(&self) -> Vec<&Hound> {
        self.animals.hounds.iter().filter(|h| !h.gone).collect()
    }

    /// How hungry a tamed hound is: 0 fed, 1 starving. None if there's no such hound.
    pub fn hound_hunger(&self, id: u32) -> Option<f32> {
        self.animals.hounds.iter().find(|h| h.id == id && !h.gone).map(|h| h.hunger_at(self.time))
    }

    /// Feed a tamed hound: `amount` 1.0 is a full day's meat.
    pub fn feed_hound(&mut self, id: u32, amount: f32) -> bool {
        let t = self.time;
        let Some(h) = self.animals.hounds.iter_mut().find(|h| h.id == id && !h.gone) else { return false };
        h.hunger = (h.hunger_at(t) - amount.max(0.0) * HOUND_HUNGER_PER_DAY).max(0.0);
        h.fed_at = t;
        true
    }

    /// Where a tamed hound is: at its owner's heel.
    pub fn hound_pos(&self, id: u32) -> Option<V2> {
        let k = self.animals.hounds.iter().position(|h| h.id == id)?;
        let h = &self.animals.hounds[k];
        if let Some(p) = self.fray_pos_of_hound(id) {
            return Some(p);
        }
        let n = self.animals.hounds.iter().take(k).filter(|x| x.owner == h.owner && !x.gone).count() as f32;
        let a = 2.4 + n * 0.9;
        Some(self.person_pos(h.owner).add(V2::new(a.cos(), a.sin()).scale(1.6 + n * 0.5)))
    }

    /// Hounds whose owner is dead, or who've been left to starve, go back
    /// to the wild (the dawn stock-take).
    pub(super) fn settle_hounds(&mut self, t: f64) {
        let people = &self.people;
        let mut left = Vec::new();
        for h in self.animals.hounds.iter_mut().filter(|h| !h.gone) {
            if people[h.owner as usize].dead || !people[h.owner as usize].in_squad || t >= h.leaves_at() {
                h.gone = true;
                left.push(people[h.owner as usize].in_squad && !people[h.owner as usize].dead);
            }
        }
        for starved in left {
            if starved {
                self.log.push_front((t, "A hound, unfed, slinks off into the hills.".to_string()));
                self.log.truncate(14);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pen(sp: Sp, count: f32) -> Pen {
        Pen { id: 0, sp, town: None, owner: 0, home: V2::new(5_000.0, 5_000.0), radius: 6.0, region: 0, seed: 5, limit: count, at: 0.0, count, hunger: 0.0, feed: 0.0, made: vec![0.0; sp.def().daily.len()], past: Vec::new() }
    }

    #[test]
    fn a_fed_coop_lays_more_than_a_hungry_one() {
        let mut fed = pen(Sp::Shellhen, 10.0);
        let hungry = fed.clone();
        fed.feed = 100.0;
        let week = 7.0 * DAY;
        let (a, b) = (fed.now(week), hungry.now(week));
        assert!(a.hunger < 0.01 && b.hunger > 0.6, "{} / {}", a.hunger, b.hunger);
        assert!((a.made[0] - 70.0).abs() < 0.5, "ten hens, a week: {}", a.made[0]);
        assert!(b.made[0] < a.made[0] * 0.7, "hungry {} vs fed {}", b.made[0], a.made[0]);
        assert!(a.feed < 100.0 && a.feed > 80.0);
    }

    #[test]
    fn a_pen_worked_out_in_one_go_or_in_pieces_is_the_same() {
        let mut whole = pen(Sp::Mossback, 6.0);
        whole.feed = 12.0;
        let mut pieces = whole.clone();
        let end = 9.0 * DAY;
        for k in 1..=18 {
            pieces.settle(end * k as f64 / 18.0);
        }
        let (a, b) = (whole.now(end), pieces.now(end));
        assert!((a.hunger - b.hunger).abs() < 1e-3 && (a.feed - b.feed).abs() < 1e-3);
        assert!((a.made[0] - b.made[0]).abs() < 0.05, "{} vs {}", a.made[0], b.made[0]);
    }

    #[test]
    fn what_was_made_since_a_past_morning_can_still_be_read() {
        let mut p = pen(Sp::Shellhen, 10.0);
        p.feed = 1000.0;
        for d in 1..=5 {
            p.settle(d as f64 * DAY);
            p.remember();
        }
        let eggs = p.yield_between(2.0 * DAY, 5.5 * DAY)[0].1;
        assert!((eggs - 35.0).abs() < 0.5, "3.5 days of ten hens: {eggs}");
        assert_eq!(p.yield_between(2.5 * DAY, 2.5 * DAY)[0].1, 0.0);
    }

    #[test]
    fn a_hound_gets_hungry_and_feeding_helps() {
        let mut h = Hound { id: 0, owner: 0, seed: 1, size: 1.0, hurt: Hurt::default(), hunger: 0.0, fed_at: 0.0, tamed_at: 0.0, gone: false };
        assert!(h.hunger_at(DAY) > 0.4 && h.hunger_at(10.0 * DAY) == 1.0);
        assert!(h.power(10.0 * DAY) < 1.0);
        h.hunger = 0.2;
        h.fed_at = 3.0 * DAY;
        assert!(h.leaves_at() > 3.0 * DAY + DAY);
    }
}
