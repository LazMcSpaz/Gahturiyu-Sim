//! Work, food, goods and money — worked out from the clock.
//!
//! **Production.** At the top of every hour each town works out, from
//! everyone's day plan, how much work its people will do in the coming hour,
//! and so how fast each good comes in. The stockpile is a running total plus
//! that rate: read at any moment, settled when the hour turns. No one is
//! stepped through their day.
//!
//! **Food** comes by three paths, and the town's cooking custom decides how
//! much it leans on each: home **gardens**; the **kitchens** (a hearth kitchen
//! with cooks and meal runners, a shared deck, or each household's own pot,
//! all cooked from the town's stock); and the **dawn boats** that bring the
//! stilt village's catch ashore. Each dawn the day before is tallied: what
//! each path brought against its share of what was needed. A path that breaks
//! — cooks dead, runners gone, the boats stopped — leaves its share unmet;
//! the others don't cover for it.
//!
//! **Money.** One coin, struck by the Ṭaḍoro's presses, and notes for large
//! sums. The merchants' purse refills as fast as the town prospers; the town
//! taxes its households into a treasury that pays its guards.
//!
//! **Caravans** carry a town's surplus to a town that lacks it, on the same
//! journeys as everyone else. The cargo is real: it arrives when they do,
//! and bandits who beat them take it.

use serde::{Deserialize, Serialize};

use super::group::{Group, GroupId, Kind, Leg};
use super::items::{self, ItemId};
use super::jobs::{Good, Job, PlaceKind, Shelf, FOODS, GOODS, MEALS_PER_COOK_HOUR, N_GOODS, POTS_PER_RUN};
use super::making::{Shelved, ASH_PER_TIMBER, BURN_PER_HOUR, CHARCOAL_PER_TIMBER};
use super::person::PersonId;
use super::rng::{self, Rng};
use super::routine::{Doing, RUN_HOURS};
use super::settlement::SettlementId;
use super::society::{food_plan, Flow, DAWN};
use super::world::{World, DAY, HOUR};

/// Food a tended garden gives each day, person-days.
pub const GARDEN_FOOD: f32 = 3.0;
/// An untended garden still gives this share.
pub const GARDEN_WILD: f32 = 0.35;
/// Of garden food beyond what its path was needed for, this much reaches the market.
pub const GARDEN_SURPLUS: f32 = 0.5;
/// Coin each person's household pays the town each day.
pub const TAX_PER_HEAD: f32 = 0.6;
/// A guard's pay, coin a day.
pub const GUARD_WAGE: f32 = 12.0;
/// The merchants' purse: most coin per merchant, and how fast it refills
/// each hour per merchant in a fully prosperous town.
pub const PURSE_PER_MERCHANT: f32 = 160.0;
pub const PURSE_REFILL: f32 = 6.0;
/// Stock worth this much coin a head counts as a well-stocked town.
pub const STOCK_PER_HEAD: f32 = 20.0;
/// Merchants sell at this share over value, and buy at this share of it.
pub const BUY_MARKUP: f32 = 1.25;
pub const SELL_SHARE: f32 = 0.6;
/// A note is worth this many coin; the exchange keeps this many per swap.
pub const NOTE_VALUE: u16 = 50;
pub const EXCHANGE_FEE: u16 = 1;
/// Chance an idle caravaner sets out in a daytime hour, when there's a surplus.
pub const CARAVAN_CHANCE: f32 = 0.15;
/// Goods one caravaner or porter carries.
pub const CARGO_PER_HEAD: f32 = 40.0;
/// Share of a good's worth a caravan gets for it at market.
pub const CARAVAN_PRICE: f32 = 0.8;
/// Days of food a town keeps back before it trades any away.
pub const KEEP_DAYS: f32 = 3.0;
/// Coin's worth of each material a town keeps back, a head.
pub const KEEP_COIN: f32 = 0.6;
/// Kelp a garden takes as fertiliser each dawn, and what a full dressing adds.
pub const KELP_PER_GARDEN: f32 = 0.3;
pub const KELP_BOOST: f32 = 0.2;

/// Goods on the road with a caravan.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Cargo {
    pub good: Good,
    pub amount: f32,
    pub from: SettlementId,
    pub to: SettlementId,
    /// Coin from the sale, on its way home.
    pub coin: f32,
    pub delivered: bool,
    /// Taken by bandits.
    pub robbed: bool,
}

/// Something that happens to a caravan at a fixed moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CargoDue {
    /// They reach the town they're carrying to.
    Arrive,
    /// They're home.
    Home,
}

impl World {
    /// The hour beginning at `h` hours: settle the last hour's work, tally the
    /// day at dawn, and work out the rates for this one.
    pub(super) fn society_hour(&mut self, h: i64) {
        let t = h as f64 * HOUR;
        let s = &mut self.society;
        for tl in &mut s.towns {
            for f in &mut tl.stock {
                f.settle();
            }
        }
        for c in &mut s.communities {
            c.cooked.settle();
            c.carried.settle();
            c.catch.settle();
        }
        s.rates_from = t;
        for town in 0..self.settlements.len() {
            self.settle_making(town as SettlementId, t);
        }
        self.rot_gear(t);
        // Storylines due this hour are acted out.
        self.story_hour(h, t);
        if h.rem_euclid(24) == DAWN {
            for town in 0..self.settlements.len() {
                self.dawn(town as SettlementId, t);
            }
            for town in 0..self.settlements.len() {
                self.dawn_lives(town as SettlementId, t);
            }
            for town in 0..self.settlements.len() {
                self.dawn_law(town as SettlementId, t);
            }
            for town in 0..self.settlements.len() {
                self.place_jobless(town as SettlementId);
            }
            // Jobs and contracts, then each town's ring and storyteller.
            self.dawn_opps(t);
            self.dawn_contracts(t);
            for town in 0..self.settlements.len() {
                self.dawn_ring(town as SettlementId, t);
                self.dawn_stories(town as SettlementId, t);
            }
            self.shunned.retain(|s| s.2 > t);
            self.check_orders(t);
            self.tend_beds(t);
        }
        self.set_rates();
    }

    /// How fast everything comes in during the hour from `rates_from`, and
    /// what the town's crafters work on.
    pub(super) fn set_rates(&mut self) {
        let t = self.society.rates_from;
        let hour = (t / HOUR).round() as i64;
        let day = World::day_of(t);
        let h0 = (t.rem_euclid(DAY) / HOUR) as f32;
        let h1 = h0 + 1.0;
        for town in 0..self.settlements.len() {
            let mut stock = vec![0.0f32; N_GOODS];
            let mut work = Vec::new();
            for &p in &self.settlements[town].residents {
                let pp = &self.people[p as usize];
                if pp.dead || self.busy_until[p as usize] > t || super::body::knocked_out(&pp.wounds.hp_at(&pp.stats, t)) {
                    continue;
                }
                let l = self.society.lives[p as usize];
                if !l.job.is_post() && l.job != Job::Labourer {
                    continue;
                }
                let plan = self.day_plan(p, day);
                let worked = plan.hours_of(Doing::Work, h0, h1);
                let ran = plan.hours_of(Doing::Run, h0, h1);
                work.push((p, l, worked, ran));
            }
            let src = self.society.towns[town].sources.clone();
            let mut crafters = Vec::new();
            for (p, l, worked, ran) in work {
                let Some(ci) = l.community else { continue };
                let offshore = self.society.communities[ci as usize].stilts;
                match l.job {
                    Job::Cook => self.society.communities[ci as usize].cooked.rate += worked * MEALS_PER_COOK_HOUR,
                    Job::Runner => self.society.communities[ci as usize].carried.rate += ran / RUN_HOURS * POTS_PER_RUN,
                    Job::Fisher if offshore => {
                        self.society.communities[ci as usize].catch.rate += worked * super::jobs::FISH_PER_HOUR;
                        for (g, r) in Job::Fisher.gathers(&src, true, None) {
                            stock[g.index()] += worked * r;
                        }
                    }
                    // Wood into charcoal and ash, while there's wood in store.
                    Job::CharcoalBurner => {
                        let burn = worked * BURN_PER_HOUR;
                        if self.society.towns[town].stock[Good::Timber.index()].base >= burn {
                            // The wood goes in at the top of the hour.
                            self.society.towns[town].stock[Good::Timber.index()].base -= burn;
                            stock[Good::Charcoal.index()] += burn * CHARCOAL_PER_TIMBER;
                            stock[Good::Ash.index()] += burn * ASH_PER_TIMBER;
                        }
                    }
                    j if j.craft().is_some() => crafters.push((p, j, worked)),
                    j => {
                        let at = l.place.and_then(|i| self.society.towns[town].places.get(i as usize)).map(|w| w.kind);
                        for (g, r) in j.gathers(&src, offshore, at) {
                            stock[g.index()] += worked * r;
                        }
                    }
                }
            }
            for (k, f) in self.society.towns[town].stock.iter_mut().enumerate() {
                f.rate = stock[k];
            }
            self.plan_making(town as SettlementId, hour, &crafters);
        }
    }

    // ---- Dawn -----------------------------------------------------------------

    /// Take food from a town's stock, what spoils first first. Returns what was had.
    fn draw_food(&mut self, town: SettlementId, want: f32) -> f32 {
        let mut left = want;
        for g in FOODS {
            let f = &mut self.society.towns[town as usize].stock[g.index()];
            let take = f.base.min(left).max(0.0);
            f.base -= take;
            left -= take;
        }
        want - left
    }

    /// The day just gone, tallied: food, spoiling, tax and pay, prosperity,
    /// the merchants' purse — then the town's changes for the new day.
    fn dawn(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        let tl = self.society.towns[town as usize].clone();
        let alive = |w: &World, p: PersonId| !w.people[p as usize].dead;

        // The stilt village eats first from its catch, then sends the rest ashore.
        let mut landed = 0.0;
        if let Some(si) = tl.stilts {
            let need = self.members_of(si).count() as f32;
            let c = self.society.communities[si as usize].clone();
            let (share, home_part, _) = food_plan(&c, false, true);
            let mut catch = c.catch.base;
            let boats = catch.min(need * share[2]);
            catch -= boats;
            let k_need = need * share[1];
            let want = k_need * home_part + (k_need * (1.0 - home_part)).min(c.cooked.base);
            let from_catch = catch.min(want);
            catch -= from_catch;
            let kitchens = from_catch + self.draw_food(town, want - from_catch);
            let crew = self.members_of(si).any(|p| self.society.lives[p as usize].job == Job::Fisher && self.society.lives[p as usize].shift == 1);
            if !c.withdrawn && crew {
                landed = catch;
            }
            self.set_food(si, need, share, [0.0, kitchens, boats]);
        }
        self.society.towns[town as usize].landed = landed;

        // The land: gardens, the boats' catch, then the kitchens from stock.
        {
            let si = tl.shore;
            let need = self.members_of(si).count() as f32;
            let c = self.society.communities[si as usize].clone();
            let (share, home_part, runner_part) = food_plan(&c, !tl.gardens.is_empty(), tl.stilts.is_some());
            // Tended if the gardener is alive, still lives here and isn't away.
            let tended = |w: &World, p: PersonId| alive(w, p) && w.people[p as usize].home == Some(town) && w.busy_until[p as usize] <= t;
            let grown: f32 = tl.gardens.iter().map(|g| if g.gardener.map(|p| tended(self, p)).unwrap_or(false) { GARDEN_FOOD } else { GARDEN_FOOD * GARDEN_WILD }).sum();
            // Kelp from the store, dug in, feeds the gardens.
            let kelp = &mut self.society.towns[town as usize].stock[Good::Kelp.index()];
            let dug = kelp.base.clamp(0.0, tl.gardens.len() as f32 * KELP_PER_GARDEN);
            kelp.base -= dug;
            let grown = grown * (1.0 + KELP_BOOST * dug / (tl.gardens.len() as f32 * KELP_PER_GARDEN).max(1e-6));
            let gardens = grown.min(need * share[0]);
            let boats = landed.min(need * share[2]);
            {
                let st = &mut self.society.towns[town as usize].stock;
                st[Good::Grain.index()].base += (grown - gardens) * GARDEN_SURPLUS;
                st[Good::Fish.index()].base += landed - boats;
            }
            let k_need = need * share[1];
            let at_home = k_need * home_part;
            let cooked = (k_need - at_home).min(c.cooked.base);
            // Pots for the workers out of town go by runner.
            let hearth_part = if home_part < 1.0 { runner_part / (1.0 - home_part) } else { 0.0 };
            let out = cooked * hearth_part * c.away;
            let carried = out.min(c.carried.base);
            let kitchens = self.draw_food(town, at_home + cooked - out + carried);
            self.set_food(si, need, share, [gardens, kitchens, boats]);
        }
        for ci in std::iter::once(tl.shore).chain(tl.stilts) {
            let c = &mut self.society.communities[ci as usize];
            c.cooked.base = 0.0;
            c.carried.base = 0.0;
            c.catch.base = 0.0;
        }

        // Spoiling.
        for g in GOODS {
            let f = &mut self.society.towns[town as usize].stock[g.index()];
            f.base *= 1.0 - g.spoils();
        }

        // Tax in, guards paid.
        let heads: usize = std::iter::once(tl.shore).chain(tl.stilts).map(|ci| self.members_of(ci).count()).sum();
        let guards = self.settlements[town as usize].residents.iter().filter(|&&p| alive(self, p) && self.society.lives[p as usize].job == Job::Guard).count();
        let merchants = self.settlements[town as usize].residents.iter().filter(|&&p| alive(self, p) && self.society.lives[p as usize].job == Job::Merchant).count();
        let food = self.town_food(town);
        let value: f32 = GOODS.iter().map(|g| self.society.towns[town as usize].stock[g.index()].base * g.value()).sum();
        let tlm = &mut self.society.towns[town as usize];
        tlm.treasury += heads as f32 * TAX_PER_HEAD * (0.5 + 0.5 * tlm.prosperity.min(1.0));
        let owed = guards as f32 * GUARD_WAGE + tlm.owed;
        let paid = owed.min(tlm.treasury);
        tlm.treasury -= paid;
        tlm.owed = owed - paid;

        // Prosperity, and the merchants' coin.
        tlm.prosperity = (0.5 * food + 0.5 * (value / (heads.max(1) as f32 * STOCK_PER_HEAD)).min(1.5)).clamp(0.0, 1.5);
        tlm.purse = purse_at(tlm, t);
        tlm.purse_at = t;
        tlm.purse_cap = merchants as f32 * PURSE_PER_MERCHANT * (0.5 + tlm.prosperity);
        tlm.purse_rate = merchants as f32 * PURSE_REFILL * tlm.prosperity;

        self.tidy_making(town);
        self.locals_buy(town, t);
        self.dawn_changes(town, day);
    }

    fn set_food(&mut self, ci: u32, need: f32, share: [f32; 3], supply: [f32; 3]) {
        let mut suff = [1.0f32; 3];
        for i in 0..3 {
            if share[i] > 0.0 && need > 0.0 {
                suff[i] = (supply[i] / (need * share[i])).min(1.0);
            }
        }
        let overall = (0..3).map(|i| share[i] * suff[i]).sum::<f32>();
        let f = &mut self.society.communities[ci as usize].food;
        f.need = need;
        f.share = share;
        f.supply = supply;
        f.suff = suff;
        f.overall = overall;
    }

    /// How well a town ate yesterday, land and stilts together, 0..1.
    pub fn town_food(&self, town: SettlementId) -> f32 {
        let tl = &self.society.towns[town as usize];
        let mut need = 0.0;
        let mut fed = 0.0;
        for ci in std::iter::once(tl.shore).chain(tl.stilts) {
            let f = &self.society.communities[ci as usize].food;
            need += f.need;
            fed += f.need * f.overall;
        }
        if need > 0.0 {
            fed / need
        } else {
            1.0
        }
    }

    /// The opening books, as the world is made: a few days' food in store,
    /// some goods, a full purse, a little in the treasury.
    pub(super) fn open_books(&mut self, town: SettlementId) {
        let tl = self.society.towns[town as usize].clone();
        let n: usize = std::iter::once(tl.shore).chain(tl.stilts).map(|ci| self.members_of(ci).count()).sum();
        let nf = n as f32;
        for ci in std::iter::once(tl.shore).chain(tl.stilts) {
            let c = self.society.communities[ci as usize].clone();
            let (share, _, _) = food_plan(&c, !c.stilts && !tl.gardens.is_empty(), c.stilts || tl.stilts.is_some());
            let need = self.members_of(ci).count() as f32;
            self.set_food(ci, need, share, share.map(|s| s * need));
        }
        let merchants = self.settlements[town as usize].residents.iter().filter(|&&p| self.society.lives[p as usize].job == Job::Merchant).count();
        let tlm = &mut self.society.towns[town as usize];
        // Food for a few days; materials as the land round town offers them.
        let src = tlm.sources.clone();
        let s = |g: Good| src[g.index()];
        for g in GOODS {
            tlm.stock[g.index()].base = match g {
                Good::Grain => nf * KEEP_DAYS,
                Good::Fish => if tl.stilts.is_some() { nf * 0.5 } else { 0.0 },
                Good::Kelp => nf * 0.2,
                Good::Game => nf * 0.3,
                Good::Timber | Good::Rock => 30.0 * s(g).max(0.3),
                Good::Hides | Good::Leather | Good::Cloth | Good::Fibre => 10.0,
                Good::Herbs => 15.0,
                Good::Wares => nf * 0.3,
                Good::Charcoal => 8.0 * s(Good::Timber),
                Good::Ash => 5.0,
                Good::Ore => 30.0 * s(g),
                Good::Ingots | Good::Bronze => 4.0 * s(Good::Ore),
                Good::Paper | Good::Ink => 4.0,
                Good::Gold | Good::EdgeSeed | Good::Pearl | Good::Salvage => 2.0 * s(g),
                Good::Ringstone | Good::Slatewing | Good::Hearthclay => 2.0 * s(Good::Rock),
                Good::Edgeglass => 0.0,
                _ => 12.0 * s(g),
            };
        }
        tlm.treasury = nf * 2.0;
        tlm.prosperity = 0.7;
        tlm.purse_cap = merchants as f32 * PURSE_PER_MERCHANT * (0.5 + tlm.prosperity);
        tlm.purse_rate = merchants as f32 * PURSE_REFILL * tlm.prosperity;
        tlm.purse = tlm.purse_cap;
        tlm.purse_at = self.time;
    }

    // ---- Reading the books ------------------------------------------------------

    /// Goods in a town's store right now.
    pub fn stock_now(&self, town: SettlementId, g: Good) -> f32 {
        self.society.towns[town as usize].stock[g.index()].at(self.society.rates_from, self.time)
    }

    /// Coin the merchants have right now.
    pub fn purse_now(&self, town: SettlementId) -> f32 {
        purse_at(&self.society.towns[town as usize], self.time)
    }

    fn spend_purse(&mut self, town: SettlementId, delta: f32) {
        let t = self.time;
        let tl = &mut self.society.towns[town as usize];
        tl.purse = (purse_at(tl, t) + delta).max(0.0);
        tl.purse_at = t;
    }

    // ---- Trading with the squad -------------------------------------------------

    /// Who does the squad's trading: whoever is talking, else anyone standing.
    fn trader(&self) -> Option<PersonId> {
        let alive = |m: PersonId| !self.people[m as usize].dead;
        self.talk.as_ref().map(|c| c.with).filter(|&m| alive(m) && self.squad.index(m).is_some()).or_else(|| self.squad.members.iter().copied().find(|&m| alive(m)))
    }

    /// The town a merchant trades for and what they keep on their shelves, if
    /// they're at work.
    fn shelves(&self, npc: PersonId) -> Option<(SettlementId, Option<Shelf>)> {
        let l = self.life(npc);
        if l.job != Job::Merchant || !self.at_work(npc, self.time) {
            return None;
        }
        let town = self.people[npc as usize].home?;
        // Shunned: their people won't trade with you.
        if let (Some(ci), Some(me)) = (l.community, self.trader()) {
            if self.shunned_by(me, ci) {
                return None;
            }
        }
        let shelf = match self.workplace_of(npc).map(|w| w.kind) {
            Some(PlaceKind::Shop(s)) => Some(s),
            _ => None,
        };
        Some((town, shelf))
    }

    /// When a merchant is next at their stall (now, if they are), looking up
    /// to two days ahead in their day plans.
    pub fn trades_next(&self, npc: PersonId) -> Option<f64> {
        if self.life(npc).job != Job::Merchant {
            return None;
        }
        let step = 15.0 * 60.0;
        let mut t = self.time;
        while t < self.time + 2.0 * super::world::DAY {
            if self.at_work(npc, t) {
                return Some(t);
            }
            t = (t / step).floor() * step + step;
        }
        None
    }

    /// Is this person a merchant, at their stall now?
    pub fn is_trading(&self, npc: PersonId) -> bool {
        self.shelves(npc).is_some()
    }

    /// What a merchant has for sale: (item, how many, price each). Made
    /// things on the town's shelf first, then one of each kind of good, then
    /// more of each.
    pub fn for_sale(&self, npc: PersonId) -> Vec<(ItemId, u16, u16)> {
        let Some((town, shelf)) = self.shelves(npc) else { return vec![] };
        let mut out: Vec<(ItemId, u16, u16)> = Vec::new();
        if shelf.map(|s| s == Shelf::Goods).unwrap_or(true) {
            out.extend(self.shelf_for_sale(town).into_iter().map(|(it, n, p, _)| (it, n, p)));
        }
        let mut by_good: Vec<Vec<(ItemId, u16, u16)>> = Vec::new();
        for g in GOODS {
            let mut v = Vec::new();
            if shelf.map(|s| s != g.shelf()).unwrap_or(false) {
                continue;
            }
            let units = self.stock_now(town, g);
            let keep = if g.is_food() { self.society.communities[self.society.towns[town as usize].shore as usize].food.need } else { 0.0 };
            let spare = (units - keep).max(0.0);
            for key in g.items() {
                let it = items::id(key);
                let n = ((spare / g.items().len() as f32) / Self::units_of(g, it)).floor().min(20.0) as u16;
                if n > 0 {
                    v.push((it, n, (self.worth_in(town, it, None) * BUY_MARKUP).ceil().max(1.0) as u16));
                }
            }
            by_good.push(v);
        }
        for k in 0..by_good.iter().map(|v| v.len()).max().unwrap_or(0) {
            out.extend(by_good.iter().filter_map(|v| v.get(k).copied()));
        }
        out
    }

    /// What a merchant would pay for a plain one of these, if they'd take it at all.
    pub fn offer_for(&self, npc: PersonId, it: ItemId) -> Option<u16> {
        self.offer(npc, it, None)
    }

    /// What a merchant would pay for this (worn, marked...) thing.
    pub fn offer(&self, npc: PersonId, it: ItemId, piece: Option<&super::materials::Piece>) -> Option<u16> {
        let (town, shelf) = self.shelves(npc)?;
        let def = items::item(it);
        if matches!(def.kind, items::Kind::Coin | items::Kind::Errand) || piece.map(|p| p.left_at(items::info(it).main.def().rots, self.time) <= 0.0).unwrap_or(false) {
            return None;
        }
        let good = super::jobs::good_of(def.key);
        let takes = match (shelf, good) {
            (None, _) => true,
            (Some(s), Some(g)) => g.shelf() == s,
            // Anything else goes to the goods shop.
            (Some(s), None) => s == Shelf::Goods,
        };
        let price = (self.worth_in(town, it, piece) * SELL_SHARE).floor() as u16;
        (takes && price > 0 && self.purse_now(town) >= price as f32).then_some(price)
    }

    /// The squad buys one of something, at the first price it's offered at.
    pub fn buy(&mut self, npc: PersonId, it: ItemId) -> bool {
        let Some(&(_, _, price)) = self.for_sale(npc).iter().find(|x| x.0 == it) else { return false };
        self.buy_at(npc, it, price)
    }

    /// The squad buys one of something at this price. False if they can't
    /// pay or it's gone.
    pub fn buy_at(&mut self, npc: PersonId, it: ItemId, price: u16) -> bool {
        let Some((town, _)) = self.shelves(npc) else { return false };
        if !self.for_sale(npc).iter().any(|x| x.0 == it && x.2 == price) || self.squad_count(items::id("coin")) < price {
            return false;
        }
        let Some(buyer) = self.trader() else { return false };
        let good = super::jobs::good_of(items::item(it).key);
        let mut bought: Option<Shelved> = None;
        if good.is_none() {
            let Some(&(_, _, _, k)) = self.shelf_for_sale(town).iter().find(|x| x.0 == it && x.2 == price) else { return false };
            bought = Some(self.society.towns[town as usize].shelf.remove(k));
        }
        self.settle_condition(buyer, self.time);
        self.take_from_squad(items::id("coin"), price);
        if let Some(g) = good {
            self.society.towns[town as usize].stock[g.index()].base -= Self::units_of(g, it);
        }
        self.spend_purse(town, price as f32);
        if let Some(d) = self.people[buyer as usize].detail.as_mut() {
            match bought.and_then(|b| b.piece) {
                Some(pc) => d.gear.add_piece(it, pc),
                None => d.gear.add(it, 1),
            }
        }
        self.passed_on(bought.and_then(|b| b.piece).as_ref());
        self.people[buyer as usize].recompute_might();
        true
    }

    /// The squad sells one of something, at the first price it's offered.
    pub fn sell(&mut self, npc: PersonId, it: ItemId) -> bool {
        let Some(&(_, price)) = self.sellable(npc).iter().find(|x| x.0 == it) else { return false };
        self.sell_at(npc, it, price)
    }

    /// The squad sells one of something for this price (the one of it that
    /// fetches that, if they carry several).
    pub fn sell_at(&mut self, npc: PersonId, it: ItemId, price: u16) -> bool {
        let Some((town, _)) = self.shelves(npc) else { return false };
        let mut seller = None;
        let order: Vec<PersonId> = self.trader().into_iter().chain(self.squad.members.iter().copied()).collect();
        'find: for m in order {
            if let Some(d) = &self.people[m as usize].detail {
                for (k, e) in d.gear.bag.iter().enumerate() {
                    if e.0 == it && e.1 > 0 && self.offer(npc, it, e.2.as_ref()) == Some(price) {
                        seller = Some((m, k));
                        break 'find;
                    }
                }
            }
        }
        let Some((m, k)) = seller else { return false };
        self.settle_condition(m, self.time);
        let Some((_, piece)) = self.people[m as usize].detail.as_mut().and_then(|d| d.gear.take_entry(k)) else { return false };
        if let Some(d) = self.people[m as usize].detail.as_mut() {
            d.gear.add(items::id("coin"), price);
        }
        self.people[m as usize].recompute_might();
        // Bringing a town what it lacks is a favour.
        if let Some(g) = super::jobs::good_of(items::item(it).key) {
            if self.price_factor(town, g) >= 1.3 {
                self.add_standing(m, town, 0.5);
            }
        }
        let tl = &mut self.society.towns[town as usize];
        match super::jobs::good_of(items::item(it).key) {
            Some(g) => tl.stock[g.index()].base += Self::units_of(g, it),
            None if tl.shelf.len() < super::making::SHELF_CAP => tl.shelf.push(Shelved { item: it, piece }),
            None => tl.stock[Good::Wares.index()].base += (items::item(it).value / Good::Wares.value()).max(0.5),
        }
        self.passed_on(piece.as_ref());
        self.spend_purse(town, -(price as f32));
        true
    }

    /// Things the squad carries that this merchant would buy, with the price.
    pub fn sellable(&self, npc: PersonId) -> Vec<(ItemId, u16)> {
        let mut out: Vec<(ItemId, u16)> = Vec::new();
        for &m in &self.squad.members {
            if let Some(d) = &self.people[m as usize].detail {
                for e in &d.gear.bag {
                    if let Some(p) = self.offer(npc, e.0, e.2.as_ref()) {
                        if !out.contains(&(e.0, p)) {
                            out.push((e.0, p));
                        }
                    }
                }
            }
        }
        out
    }

    // ---- The exchange -----------------------------------------------------------

    /// An exchanger at work.
    pub fn is_exchanging(&self, npc: PersonId) -> bool {
        self.life(npc).job == Job::Exchanger && self.at_work(npc, self.time)
    }

    /// Swap coin for a note (`to_note`) or a note for coin.
    pub fn exchange(&mut self, npc: PersonId, to_note: bool) -> bool {
        if !self.is_exchanging(npc) {
            return false;
        }
        let (coin, note) = (items::id("coin"), items::id("note"));
        let Some(m) = self.trader() else { return false };
        self.settle_condition(m, self.time);
        if to_note {
            if self.squad_count(coin) < NOTE_VALUE + EXCHANGE_FEE {
                return false;
            }
            self.take_from_squad(coin, NOTE_VALUE + EXCHANGE_FEE);
            if let Some(d) = self.people[m as usize].detail.as_mut() {
                d.gear.add(note, 1);
            }
        } else {
            if self.squad_count(note) < 1 {
                return false;
            }
            self.take_from_squad(note, 1);
            if let Some(d) = self.people[m as usize].detail.as_mut() {
                d.gear.add(coin, NOTE_VALUE - EXCHANGE_FEE);
            }
        }
        self.people[m as usize].recompute_might();
        true
    }

    // ---- Caravans ---------------------------------------------------------------

    /// How much of a good a town keeps back before it trades any away.
    pub(super) fn keep_back(&self, town: SettlementId, g: Good) -> f32 {
        let tl = &self.society.towns[town as usize];
        let need: f32 = std::iter::once(tl.shore).chain(tl.stilts).map(|ci| self.society.communities[ci as usize].food.need).sum();
        if g.is_food() {
            need * KEEP_DAYS
        } else {
            // A store's worth of each, by value: few pearls, plenty of rock.
            (need * KEEP_COIN / g.value()).clamp(3.0, 80.0)
        }
    }

    /// Does a caravan set out from town `s` this hour? Decided by a roll
    /// keyed to the town and the hour, from who's free at the top of it.
    pub(super) fn plan_caravan(&mut self, s: SettlementId, h: i64) -> Option<Group> {
        let hod = h.rem_euclid(24);
        if !(7..=14).contains(&hod) {
            return None;
        }
        let t0 = h as f64 * HOUR;
        let free = |w: &World, p: PersonId| !w.people[p as usize].dead && w.busy_until[p as usize] <= t0;
        let town = &self.settlements[s as usize];
        let lead = town.residents.iter().copied().find(|&p| self.society.lives[p as usize].job == Job::Caravaner && free(self, p))?;
        let mut rng = Rng::from_keys(&[self.seed, s as u64, h as u64, 0x4341_5241]);
        if !rng.chance(CARAVAN_CHANCE) {
            return None;
        }
        // What to carry where: the good and market with most to gain between
        // the two towns' own prices, for the length of the walk.
        let mut best: Option<(Good, f32, Vec<f32>, f32)> = None;
        for g in GOODS {
            let spare = self.society.towns[s as usize].stock[g.index()].base - self.keep_back(s, g);
            if spare <= 0.0 || g.items().is_empty() || spare * g.value() < 60.0 {
                continue;
            }
            let home = self.price_factor_at(s, g, t0);
            let load = spare.min(CARGO_PER_HEAD * 2.0) * g.value();
            let w: Vec<f32> = self
                .settlements
                .iter()
                .map(|o| {
                    if o.id == s {
                        return 0.0;
                    }
                    let gain = (self.price_factor_at(o.id, g, t0) - home).max(0.0);
                    gain * load / (1.0 + (o.pos.dist(town.pos) / 6000.0).powi(2))
                })
                .collect();
            let score = w.iter().copied().fold(0.0, f32::max);
            if score > best.as_ref().map(|b| b.3).unwrap_or(0.0) {
                best = Some((g, spare, w, score));
            }
        }
        let (good, spare, w, _) = best?;
        let to = rng.weighted(&w)? as SettlementId;
        let mut members = vec![lead];
        let porters: Vec<PersonId> = town.residents.iter().copied().filter(|&p| self.society.lives[p as usize].job == Job::Labourer && free(self, p)).collect();
        for _ in 0..2 {
            if porters.is_empty() || !rng.chance(0.6) {
                break;
            }
            let p = porters[rng.below(porters.len())];
            if !members.contains(&p) {
                members.push(p);
            }
        }
        let amount = spare.min(CARGO_PER_HEAD * members.len() as f32);
        let speed = members.iter().map(|&m| self.people[m as usize].race.walk_speed()).fold(f32::MAX, f32::min);
        let start = t0 + rng.f64() * HOUR;
        let dest = &self.settlements[to as usize];
        let road = |a: SettlementId, b: SettlementId, from: super::geo::V2, to: super::geo::V2| -> Vec<super::geo::V2> {
            let mut p = self.routes.between(a, b);
            if p.len() < 2 {
                return vec![from, to];
            }
            p[0] = from;
            let last = p.len() - 1;
            p[last] = to;
            p
        };
        let there = Leg::along(road(s, to, town.pos, dest.pos), start, speed, Some(to), &self.terrain);
        let stay = HOUR * (2.0 + rng.f64() * 2.0);
        let back = Leg::along(road(to, s, dest.pos, town.pos), there.arrive + stay, speed, Some(s), &self.terrain);
        let ends = back.arrive;
        self.society.towns[s as usize].stock[good.index()].base -= amount;
        let id = self.next_group;
        self.next_group += 1;
        Some(Group {
            id,
            seed: rng::key(&[self.seed, s as u64, h as u64, 0x4341_5247]),
            members,
            kind: Kind::Journey { home: s },
            legs: vec![there, back],
            speed,
            ends,
            written: 0,
            hostile: false,
            pos: town.pos,
            last_update: start,
            band: 3,
            cargo: Some(Cargo { good, amount, from: s, to, coin: 0.0, delivered: false, robbed: false }),
        })
    }

    /// The next caravan arrival or homecoming, if any: (when, group, which).
    pub(super) fn next_cargo(&self) -> Option<(f64, GroupId, CargoDue)> {
        let mut best: Option<(f64, GroupId, CargoDue)> = None;
        for g in &self.groups {
            let Some(c) = &g.cargo else { continue };
            if self.fighting_groups.contains(&g.id) {
                continue;
            }
            let due = if !c.delivered && !c.robbed {
                g.legs.iter().find(|l| l.dest == Some(c.to)).map(|l| (l.arrive, CargoDue::Arrive))
            } else {
                None
            };
            let due = due.or_else(|| g.ends.is_finite().then_some((g.ends, CargoDue::Home)));
            if let Some((t, what)) = due {
                if best.map(|b| (t, g.id) < (b.0, b.1)).unwrap_or(true) {
                    best = Some((t, g.id, what));
                }
            }
        }
        best
    }

    /// A caravan reaches its market, or gets home.
    pub(super) fn cargo_event(&mut self, gid: GroupId, what: CargoDue, t: f64) {
        let Some(gi) = self.groups.iter().position(|g| g.id == gid) else { return };
        let Some(mut c) = self.groups[gi].cargo.clone() else { return };
        match what {
            CargoDue::Arrive => {
                // Sold to the merchants there, as much as they can pay for;
                // the rest goes home again.
                let each = c.good.value() * self.price_factor_at(c.to, c.good, t) * CARAVAN_PRICE;
                let tl = &mut self.society.towns[c.to as usize];
                let sold = c.amount.min(purse_at(tl, t) / each);
                tl.purse = purse_at(tl, t) - sold * each;
                tl.purse_at = t;
                tl.stock[c.good.index()].base += sold;
                c.coin = sold * each;
                c.amount -= sold;
                c.delivered = true;
                self.groups[gi].cargo = Some(c);
            }
            CargoDue::Home => {
                let tl = &mut self.society.towns[c.from as usize];
                tl.purse = purse_at(tl, t) + c.coin;
                tl.purse_at = t;
                // Whatever never got sold comes back to the store.
                tl.stock[c.good.index()].base += c.amount;
                self.groups[gi].cargo = None;
            }
        }
    }

    /// Bandits who beat a caravan take what it carries.
    pub(super) fn rob_cargo(&mut self, gid: GroupId) {
        if let Some(g) = self.groups.iter_mut().find(|g| g.id == gid) {
            if let Some(c) = g.cargo.as_mut() {
                c.amount = 0.0;
                c.coin = 0.0;
                c.robbed = true;
            }
        }
    }
}

/// The merchants' coin at time `t`.
pub fn purse_at(tl: &super::society::TownLife, t: f64) -> f32 {
    (tl.purse + tl.purse_rate * ((t - tl.purse_at) / HOUR).max(0.0) as f32).min(tl.purse_cap.max(tl.purse))
}

/// For flows of goods: what a stock is in time.
pub fn flow_now(f: &Flow, from: f64, t: f64) -> f32 {
    f.at(from, t)
}
