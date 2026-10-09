//! A town's crafters at work, what the land gives, what things cost, and the
//! squad's dealings with crafters: orders, lessons and manuals.
//!
//! **Sources.** What the land round a town offers (ore in the mountains,
//! sand on the plateau, reed and shell on the coast...) is read off the
//! terrain once, as the world is made, and stored with the town. Gatherers
//! bring in their share of it every hour they work (`Job::gathers`).
//!
//! **Crafters** work like everyone else: at the top of each hour the hours
//! they'll put in come from their day plan. When a crafter has nothing in
//! hand they choose something to make from what the town has in store and
//! what it lacks (one roll, keyed to them and the hour); the materials go in
//! then, and it's done once their hours add up (`Recipe::npc_hours`).
//! Materials go back into the store; weapons, armour, scrolls and the like go
//! on the town's shelf, graded by the maker's skill and their station, and
//! marked with who made them and where.
//!
//! **Prices** follow each town's own store: a good it has little of costs
//! more there (`price_factor`), worked out when asked. Grade is in an item's
//! worth already; a stamped maker's mark adds to it, more as the maker
//! becomes known.
//!
//! **Orders.** Grown pieces are ordered from a Tender: a deposit now, the
//! rest when it's ready, a set number of days later. It only grows while the
//! Tender is there to tend it; if they die, the order and the deposit are
//! lost.

use serde::{Deserialize, Serialize};

use super::crafting::{made_piece, Recipe, RECIPES};
use super::culture::Teaching;
use super::items::{self, item, ItemId, Kind};
use super::jobs::{good_of, good_of_material, Good, Job, GOODS, N_GOODS};
use super::materials::{Craft, Grade, Piece};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::stats::Skill;
use super::world::{World, DAY, HOUR};

/// Most made things a town's shelf holds, and of any one kind.
pub const SHELF_CAP: usize = 30;
pub const SHELF_EACH: usize = 3;
/// A stamped maker's mark multiplies an item's worth by this...
pub const MARK_WORTH: f32 = 1.15;
/// ...plus this for each piece of theirs sold, up to `RENOWN_MAX`.
pub const RENOWN_WORTH: f32 = 0.01;
pub const RENOWN_MAX: f32 = 25.0;
/// A town's price for a good: its worth times this factor, from its store.
pub const PRICE_RANGE: (f32, f32) = (0.5, 2.5);
/// Charcoal burning: timber an hour, and what comes of each unit.
pub const BURN_PER_HOUR: f32 = 1.0;
pub const CHARCOAL_PER_TIMBER: f32 = 0.6;
pub const ASH_PER_TIMBER: f32 = 0.15;
/// Ash from a forge, per thing made.
pub const FORGE_ASH: f32 = 0.3;
/// A lesson's base price, coin.
pub const LESSON_PRICE: f32 = 30.0;
/// Teachers teach up to this skill; past it, practice.
pub const LESSON_CAP: f32 = 60.0;
/// A manual: skill it gives, hours to study it, and the most it takes you to.
pub const MANUAL_GAIN: f32 = 8.0;
pub const MANUAL_HOURS: f64 = 8.0;
pub const MANUAL_CAP: f32 = 30.0;
/// How far out quarriers go for ore and rare veins, metres.
pub const VEIN_REACH: f32 = 3200.0;
/// Made things the townsfolk buy off the shelf each dawn (the oldest first).
pub const LOCALS_BUY: usize = 3;
/// Share of an order paid up front.
pub const DEPOSIT: f32 = 0.5;

/// A crafter's work in hand.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Making {
    pub who: PersonId,
    pub recipe: u16,
    /// Hours put in, settled on the hour.
    pub done: f32,
    /// Hours they're putting in this hour.
    pub rate: f32,
    /// The hour they took it up (keys the grade roll).
    pub hour: i64,
}

/// A made thing on a town's shelf.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Shelved {
    pub item: ItemId,
    pub piece: Option<Piece>,
}

/// A grown piece on order.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Order {
    pub tender: PersonId,
    pub recipe: u16,
    pub town: SettlementId,
    pub ready_at: f64,
    pub deposit: u16,
    /// Still to pay when it's collected.
    pub rest: u16,
    pub grade: Grade,
}

/// A craft being learned.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Lesson {
    pub who: PersonId,
    pub craft: Craft,
    pub gain: f32,
    /// The most this lesson takes their skill to.
    pub cap: f32,
    pub done_at: f64,
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Does this trade take on this recipe? (A tanner tans; a tailor works cloth.)
fn job_takes(job: Job, rc: &Recipe) -> bool {
    let uses = |k: &str| rc.inputs.iter().any(|i| i.0 == k);
    match job {
        Job::Tanner => rc.output == "leather",
        Job::Leatherworker => rc.output != "leather" && uses("leather"),
        Job::Tailor => rc.output == "cloth" || uses("cloth") || uses("fibre"),
        Job::Woodworker => uses("timber") || uses("rock"),
        _ => true,
    }
}

impl World {
    // ---- Sources --------------------------------------------------------------

    /// What the land within reach of a town offers, by good, 0..1.
    pub(super) fn town_sources(&self, town: SettlementId) -> Vec<f32> {
        let s = &self.settlements[town as usize];
        let (mut n, mut mount, mut plat, mut edge, mut hills, mut low) = (0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0);
        for j in -3..=3 {
            for i in -3..=3 {
                let p = s.pos.add(super::geo::V2::new(i as f32 * 500.0, j as f32 * 500.0));
                if p.dist(s.pos) > 1600.0 || !super::geo::is_land(p) {
                    continue;
                }
                let (m, pl, h) = (self.terrain.mountains(p), self.terrain.plateau(p), self.terrain.height(p));
                n += 1.0;
                mount += m;
                plat += pl;
                edge += 4.0 * pl * (1.0 - pl);
                hills += smooth(60.0, 180.0, h) * (1.0 - pl);
                low += ((h < 45.0) && m < 0.15 && pl < 0.15) as u8 as f32;
            }
        }
        let n = n.max(1.0);
        let (mount, plat, edge, hills, low) = (mount / n, plat / n, edge / n, hills / n, low / n);
        // Quarriers go further for ore and veins: the high ground within a day's round trip.
        let (mut nf, mut far) = (0.0f32, 0.0f32);
        for j in -4..=4 {
            for i in -4..=4 {
                let p = s.pos.add(super::geo::V2::new(i as f32 * 750.0, j as f32 * 750.0));
                if p.dist(s.pos) > VEIN_REACH || !super::geo::is_land(p) {
                    continue;
                }
                nf += 1.0;
                far += self.terrain.mountains(p).max(4.0 * self.terrain.plateau(p) * (1.0 - self.terrain.plateau(p)) * 0.5);
            }
        }
        let far = far / nf.max(1.0);
        let sea = if s.coastal { 1.0 } else { 0.0 };
        let mut r = Rng::from_keys(&[self.seed, town as u64, 0x534F_5552]);
        let mut src = vec![0.0f32; N_GOODS];
        let mut set = |g: Good, x: f32| src[g.index()] = x.clamp(0.0, 1.0);
        set(Good::Timber, (1.0 - 0.7 * plat - 0.5 * mount).max(0.15));
        set(Good::Rock, mount + hills * 0.7 + plat * 0.3);
        set(Good::Ore, mount + edge * 0.4 + far * 1.5 - 0.05);
        set(Good::Clay, low);
        set(Good::Sand, plat + sea * 0.25);
        // Rare veins.
        let veins = r.f32();
        set(Good::Gold, if veins < 0.3 && (far > 0.15 || plat > 0.3) { 0.6 } else { 0.0 });
        set(Good::EdgeSeed, if far > 0.2 { (far - 0.2) * 1.2 } else { 0.0 });
        // The coast: reed, resin from the shore woods, shellbeds, the seabed.
        set(Good::Seareed, sea);
        set(Good::Pitch, sea * 0.6);
        set(Good::Nacre, sea);
        set(Good::Pearl, sea * 0.5);
        set(Good::Salvage, sea);
        set(Good::Fishskin, sea);
        // Tentsilk cocoons in the hills (a stand-in until there's a bestiary).
        set(Good::Tentsilk, hills + mount * 0.5);
        for g in [Good::Grain, Good::Game, Good::Hides, Good::Herbs, Good::Fibre] {
            set(g, 1.0);
        }
        src
    }

    /// What the land round a town offers of a good, 0..1.
    pub fn source(&self, town: SettlementId, g: Good) -> f32 {
        self.society.towns[town as usize].sources.get(g.index()).copied().unwrap_or(0.0)
    }

    // ---- Prices ---------------------------------------------------------------

    /// How much of a good a town would like to have in store.
    pub fn want_of(&self, town: SettlementId, g: Good) -> f32 {
        if g.is_food() {
            // Food is wanted all together; any one kind is a share of it.
            self.keep_back(town, g) / super::jobs::FOODS.len() as f32
        } else {
            self.keep_back(town, g)
        }
    }

    /// A town's price for a good, as a share of its worth: dear where it's
    /// scarce, cheap where it's plentiful.
    pub fn price_factor(&self, town: SettlementId, g: Good) -> f32 {
        self.price_factor_at(town, g, self.time)
    }

    /// The same, at a moment on the world's timeline.
    pub fn price_factor_at(&self, town: SettlementId, g: Good, t: f64) -> f32 {
        let want = self.want_of(town, g).max(1.0);
        let have = self.society.towns[town as usize].stock[g.index()].at(self.society.rates_from, t).max(0.0);
        (want / (have + want * 0.25)).sqrt().clamp(PRICE_RANGE.0, PRICE_RANGE.1)
    }

    /// What one of something is worth in a town: its worth (grade in it
    /// already) at the town's price, more for a stamped mark, less for wear.
    pub fn worth_in(&self, town: SettlementId, it: ItemId, piece: Option<&Piece>) -> f32 {
        self.worth_at(town, it, piece, self.time)
    }

    /// The same, at a moment on the world's timeline.
    pub fn worth_at(&self, town: SettlementId, it: ItemId, piece: Option<&Piece>, t: f64) -> f32 {
        let def = item(it);
        let mut v = def.value;
        if let Some(g) = good_of(def.key) {
            v *= self.price_factor_at(town, g, t);
        } else if let Some(g) = good_of_material(items::info(it).main) {
            // Made pieces: mostly the work, partly the material.
            v *= self.price_factor_at(town, g, t).sqrt();
        }
        if let Some(pc) = piece {
            if let Some(m) = pc.mark.filter(|m| m.stamped) {
                v *= MARK_WORTH + RENOWN_WORTH * self.renown(m.maker).min(RENOWN_MAX);
            }
            let most = items::max_durability(it);
            if most > 0.0 {
                let left = pc.left_at(items::info(it).main.def().rots, t);
                v *= 0.3 + 0.7 * (left / most).clamp(0.0, 1.0);
            }
        }
        v
    }

    /// How known a maker is.
    pub fn renown(&self, maker: PersonId) -> f32 {
        self.society.renown.get(&maker).copied().unwrap_or(0.0)
    }

    /// A stamped piece changes hands: its maker becomes better known.
    pub(super) fn passed_on(&mut self, piece: Option<&Piece>) {
        if let Some(m) = piece.and_then(|p| p.mark).filter(|m| m.stamped) {
            *self.society.renown.entry(m.maker).or_insert(0.0) += 1.0;
        }
    }

    // ---- Crafters at work -----------------------------------------------------

    /// How good someone is at their trade: their own skill, or what years at
    /// it have made of them (rolled once from who they are).
    pub fn work_skill(&self, pid: PersonId, craft: Craft) -> f32 {
        let own = self.people[pid as usize].stats.skill(craft.skill());
        if self.life(pid).job.craft() == Some(craft) && !self.people[pid as usize].in_squad {
            own.max(25.0 + 45.0 * Rng::from_keys(&[self.seed, pid as u64, craft.index() as u64, 0x534B_494C]).f32())
        } else {
            own
        }
    }

    /// Stock units one of an item counts as in its good.
    pub fn units_of(good: Good, it: ItemId) -> f32 {
        let u = item(it).value / good.value();
        // Wares are counted by worth (a handful of sling stones is no torch);
        // materials one to a unit or more.
        if good == Good::Wares {
            u.max(0.01)
        } else {
            u.max(1.0).ceil()
        }
    }

    /// Has the town the materials for a recipe in store?
    fn has_inputs(&self, town: SettlementId, rc: &Recipe) -> bool {
        let st = &self.society.towns[town as usize].stock;
        rc.inputs.iter().all(|&(k, n)| good_of(k).map(|g| st[g.index()].base >= n as f32 * Self::units_of(g, items::id(k))).unwrap_or(false))
    }

    /// Something for a crafter to take up, chosen from what the town has and
    /// lacks, by one roll keyed to them and the hour. Its materials go in now.
    fn choose_work(&mut self, town: SettlementId, who: PersonId, job: Job, h: i64) -> Option<usize> {
        let craft = job.craft()?;
        let tl = &self.society.towns[town as usize];
        let mut picks: Vec<(usize, f32)> = Vec::new();
        for (ri, rc) in RECIPES.iter().enumerate() {
            if rc.skill != craft.skill() || rc.is_grown_piece() || !job_takes(job, rc) || !self.has_inputs(town, rc) {
                continue;
            }
            let w = match good_of(rc.output) {
                Some(g) => {
                    let want = self.want_of(town, g);
                    let have = tl.stock[g.index()].base;
                    if have >= want {
                        continue;
                    }
                    1.0 + (want - have) / want.max(1.0)
                }
                None => {
                    let form = items::info(rc.item(Grade::Common)).form;
                    let same = tl.shelf.iter().filter(|s| items::info(s.item).form == form).count() + tl.making.iter().filter(|m| RECIPES[m.recipe as usize].output == rc.output).count();
                    if tl.shelf.len() + tl.making.len() >= SHELF_CAP || same >= SHELF_EACH {
                        continue;
                    }
                    1.0
                }
            };
            picks.push((ri, w));
        }
        let mut r = Rng::from_keys(&[self.seed, who as u64, h as u64, 0x574F_524B]);
        let k = r.weighted(&picks.iter().map(|p| p.1).collect::<Vec<_>>())?;
        let ri = picks[k].0;
        let st = &mut self.society.towns[town as usize].stock;
        for &(key, n) in RECIPES[ri].inputs {
            if let Some(g) = good_of(key) {
                st[g.index()].base -= n as f32 * Self::units_of(g, items::id(key));
            }
        }
        Some(ri)
    }

    /// This hour's work for a town's crafters: what each is making, and how
    /// many hours they'll put in.
    pub(super) fn plan_making(&mut self, town: SettlementId, h: i64, crafters: &[(PersonId, Job, f32)]) {
        for m in &mut self.society.towns[town as usize].making {
            m.rate = 0.0;
        }
        for &(p, job, worked) in crafters {
            if worked <= 0.0 {
                continue;
            }
            let have = self.society.towns[town as usize].making.iter().position(|m| m.who == p);
            let k = match have {
                Some(k) => k,
                None => {
                    let Some(ri) = self.choose_work(town, p, job, h) else { continue };
                    let ms = &mut self.society.towns[town as usize].making;
                    ms.push(Making { who: p, recipe: ri as u16, done: 0.0, rate: 0.0, hour: h });
                    ms.len() - 1
                }
            };
            self.society.towns[town as usize].making[k].rate = worked;
            if let Some(m) = self.society.minds.get_mut(p as usize) {
                m.had_work = true;
            }
        }
    }

    /// The hour's work settled: anything finished goes to the store or the shelf.
    pub(super) fn settle_making(&mut self, town: SettlementId, t: f64) {
        let mut done = Vec::new();
        let ms = &mut self.society.towns[town as usize].making;
        for (k, m) in ms.iter_mut().enumerate() {
            m.done += m.rate;
            m.rate = 0.0;
            if m.done >= RECIPES[m.recipe as usize].npc_hours() {
                done.push(k);
            }
        }
        let finished: Vec<Making> = done.into_iter().rev().map(|k| ms.remove(k)).collect();
        for m in finished.into_iter().rev() {
            self.finish_making(town, m, t);
        }
    }

    fn finish_making(&mut self, town: SettlementId, m: Making, t: f64) {
        let rc = &RECIPES[m.recipe as usize];
        let skill = self.work_skill(m.who, rc.craft());
        let roll = Rng::from_keys(&[self.seed, m.who as u64, m.hour as u64, 0x4752_4144]).f32();
        let grade = Grade::from(skill, 1.0, roll);
        let id = rc.item(grade);
        let tl = &mut self.society.towns[town as usize];
        if rc.station == super::crafting::Station::Forge {
            tl.stock[Good::Ash.index()].base += FORGE_ASH;
        }
        match good_of(rc.output) {
            Some(g) => tl.stock[g.index()].base += rc.makes as f32 * Self::units_of(g, id),
            None => {
                let piece = rc.is_piece().then(|| made_piece(id, m.who, Some(town), skill, rc.seals(), t));
                for _ in 0..rc.makes {
                    if tl.shelf.len() < SHELF_CAP {
                        tl.shelf.push(Shelved { item: id, piece });
                    }
                }
            }
        }
    }

    /// At dawn: work left by crafters who are gone or have changed trade is dropped.
    pub(super) fn tidy_making(&mut self, town: SettlementId) {
        let ms = std::mem::take(&mut self.society.towns[town as usize].making);
        let keep = ms.into_iter().filter(|m| !self.people[m.who as usize].dead && self.people[m.who as usize].home == Some(town) && self.life(m.who).job.craft() == Some(RECIPES[m.recipe as usize].craft())).collect();
        self.society.towns[town as usize].making = keep;
    }

    // ---- The shelf --------------------------------------------------------------

    /// At dawn: the townsfolk buy the oldest few things off the shelf (the
    /// merchants take the coin; stamped work spreads its maker's name).
    pub(super) fn locals_buy(&mut self, town: SettlementId, t: f64) {
        // Reed that's rotted through on the shelf is thrown out.
        self.society.towns[town as usize].shelf.retain(|s| s.piece.map(|p| p.left_at(items::info(s.item).main.def().rots, t) > 0.0).unwrap_or(true));
        let n = LOCALS_BUY.min(self.society.towns[town as usize].shelf.len());
        let sold: Vec<Shelved> = self.society.towns[town as usize].shelf.drain(..n).collect();
        let coin: f32 = sold.iter().map(|s| self.worth_at(town, s.item, s.piece.as_ref(), t)).sum();
        for s in &sold {
            self.passed_on(s.piece.as_ref());
        }
        let tl = &mut self.society.towns[town as usize];
        let now = super::economy::purse_at(tl, t);
        tl.purse = (now + coin).min(tl.purse_cap.max(now));
        tl.purse_at = t;
    }

    /// Made things on a town's shelf: (item, how many, price, where on the shelf).
    pub fn shelf_for_sale(&self, town: SettlementId) -> Vec<(ItemId, u16, u16, usize)> {
        let mut out: Vec<(ItemId, u16, u16, usize)> = Vec::new();
        for (k, s) in self.society.towns[town as usize].shelf.iter().enumerate() {
            let price = (self.worth_in(town, s.item, s.piece.as_ref()) * super::economy::BUY_MARKUP).ceil().max(1.0) as u16;
            match out.iter_mut().find(|o| o.0 == s.item && o.2 == price) {
                Some(o) => o.1 += 1,
                None => out.push((s.item, 1, price, k)),
            }
        }
        out
    }

    // ---- Orders -----------------------------------------------------------------

    /// A Tender at work: what they'd grow to order, and the price.
    pub fn order_options(&self, tender: PersonId) -> Vec<(usize, u16)> {
        let Some(town) = self.people[tender as usize].home else { return vec![] };
        if self.life(tender).job != Job::StoneTender || !self.at_work(tender, self.time) {
            return vec![];
        }
        RECIPES
            .iter()
            .enumerate()
            .filter(|(_, rc)| rc.is_grown_piece() && self.has_inputs(town, rc))
            .map(|(ri, rc)| (ri, item(rc.item(Grade::Common)).value.ceil() as u16))
            .collect()
    }

    /// Order a grown piece: the deposit now, the feedstock from the town's
    /// store, ready a set time from now if it's tended.
    pub fn place_order(&mut self, tender: PersonId, ri: usize) -> Result<usize, &'static str> {
        let Some(&(_, price)) = self.order_options(tender).iter().find(|o| o.0 == ri) else { return Err("they can't grow that now") };
        let town = self.people[tender as usize].home.ok_or("no town")?;
        let deposit = (price as f32 * DEPOSIT).ceil() as u16;
        if self.squad_count(items::id("coin")) < deposit {
            return Err("not enough coin");
        }
        self.take_from_squad(items::id("coin"), deposit);
        let rc = &RECIPES[ri];
        let st = &mut self.society.towns[town as usize].stock;
        for &(k, n) in rc.inputs {
            if let Some(g) = good_of(k) {
                st[g.index()].base -= n as f32 * Self::units_of(g, items::id(k));
            }
        }
        let skill = self.work_skill(tender, Craft::Tending);
        let roll = Rng::from_keys(&[self.seed, tender as u64, ri as u64, (self.time / HOUR) as u64, 0x4F52_4452]).f32();
        self.orders.push(Order { tender, recipe: ri as u16, town, ready_at: self.time + rc.time, deposit, rest: price - deposit, grade: Grade::from(skill, 1.0, roll) });
        Ok(self.orders.len() - 1)
    }

    /// The squad's orders with this Tender.
    pub fn orders_with(&self, tender: PersonId) -> Vec<usize> {
        (0..self.orders.len()).filter(|&k| self.orders[k].tender == tender).collect()
    }

    pub fn order_ready(&self, k: usize) -> bool {
        self.orders.get(k).map(|o| self.time >= o.ready_at).unwrap_or(false)
    }

    /// Collect a finished order (the Tender at work), paying the rest.
    pub fn collect_order(&mut self, k: usize) -> Result<ItemId, &'static str> {
        let o = *self.orders.get(k).ok_or("no such order")?;
        if !self.order_ready(k) {
            return Err("it isn't ready");
        }
        if !self.at_work(o.tender, self.time) {
            return Err("the Tender isn't at work");
        }
        if self.squad_count(items::id("coin")) < o.rest {
            return Err("not enough coin");
        }
        let who = self.talk.as_ref().map(|c| c.with).or_else(|| self.squad.members.first().copied()).ok_or("no one to take it")?;
        self.take_from_squad(items::id("coin"), o.rest);
        let rc = &RECIPES[o.recipe as usize];
        let id = rc.item(o.grade);
        let skill = self.work_skill(o.tender, Craft::Tending);
        let piece = made_piece(id, o.tender, Some(o.town), skill, false, self.time);
        self.settle_condition(who, self.time);
        if let Some(d) = self.people[who as usize].detail.as_mut() {
            d.gear.add_piece(id, piece);
        }
        self.people[who as usize].recompute_might();
        self.orders.remove(k);
        Ok(id)
    }

    /// At dawn: an order whose Tender has died is lost with its deposit; one
    /// whose Tender is away or down grew no further yesterday.
    pub(super) fn check_orders(&mut self, t: f64) {
        let mut k = 0;
        while k < self.orders.len() {
            let o = self.orders[k];
            let p = &self.people[o.tender as usize];
            // A Tender who's moved takes the bed with them.
            if let Some(h) = p.home.filter(|&h| h != o.town && !p.dead) {
                self.orders[k].town = h;
                let what = item(RECIPES[o.recipe as usize].item(o.grade)).name.to_lowercase();
                let to = self.settlements[h as usize].name.clone();
                self.log.push_front((t, format!("The Tender growing your {what} has moved to {to}; collect it there.")));
                self.log.truncate(14);
            }
            let p = &self.people[o.tender as usize];
            if p.dead || p.home.is_none() {
                let what = item(RECIPES[o.recipe as usize].item(o.grade)).name.to_lowercase();
                self.log.push_front((t, format!("The Tender growing your {what} has died. The order is lost, and the deposit with it.")));
                self.log.truncate(14);
                self.orders.remove(k);
                continue;
            }
            let away = self.busy_until[o.tender as usize] > t || super::body::knocked_out(&p.wounds.hp_at(&p.stats, t));
            if away && o.ready_at > t {
                self.orders[k].ready_at += DAY;
            }
            k += 1;
        }
    }

    // ---- Learning a craft -------------------------------------------------------

    /// A crafter at work offers to teach their trade: (craft, price, how they teach).
    pub fn craft_lesson(&self, teacher: PersonId, learner: PersonId) -> Option<(Craft, u16, Teaching)> {
        let craft = self.life(teacher).job.craft()?;
        if self.lessons.iter().any(|l| l.who == learner) || !self.at_work(teacher, self.time) || self.people[learner as usize].stats.skill(craft.skill()) >= LESSON_CAP.min(self.work_skill(teacher, craft)) {
            return None;
        }
        let style = self.life(teacher).habits.teaching;
        let have = self.people[learner as usize].stats.skill(craft.skill());
        Some((craft, (LESSON_PRICE * style.lesson().2 * (1.0 + have / 40.0)).ceil() as u16, style))
    }

    /// Pay for a lesson. The skill (and the craft, if it's new) comes when it's over.
    pub fn take_craft_lesson(&mut self, teacher: PersonId, learner: PersonId) -> Result<Craft, &'static str> {
        let (craft, price, style) = self.craft_lesson(teacher, learner).ok_or("they've nothing to teach you")?;
        if self.lessons.iter().any(|l| l.who == learner) {
            return Err("already learning");
        }
        if self.squad_count(items::id("coin")) < price {
            return Err("not enough coin");
        }
        self.take_from_squad(items::id("coin"), price);
        let (gain, hours, _) = style.lesson();
        let cap = LESSON_CAP.min(self.work_skill(teacher, craft));
        self.lessons.push(Lesson { who: learner, craft, gain, cap, done_at: self.time + hours * HOUR });
        Ok(craft)
    }

    /// Study a manual (kept afterwards): slower than a teacher, and it only
    /// takes you so far.
    pub fn study(&mut self, who: PersonId, it: ItemId) -> bool {
        let Kind::Manual(skill) = item(it).kind else { return false };
        let Some(craft) = Craft::of_skill(skill) else { return false };
        if self.lessons.iter().any(|l| l.who == who) || (self.knows_craft(who, craft) && self.people[who as usize].stats.skill(skill) >= MANUAL_CAP) {
            return false;
        }
        self.lessons.push(Lesson { who, craft, gain: MANUAL_GAIN, cap: MANUAL_CAP, done_at: self.time + MANUAL_HOURS * HOUR });
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        self.log.push_front((self.time, format!("{name} settles down to study the {}.", item(it).name.to_lowercase())));
        self.log.truncate(14);
        true
    }

    /// Lessons that are over.
    pub(super) fn do_lessons(&mut self) {
        let t = self.time;
        let mut k = 0;
        while k < self.lessons.len() {
            let l = self.lessons[k];
            if t < l.done_at {
                k += 1;
                continue;
            }
            self.lessons.remove(k);
            let p = &mut self.people[l.who as usize];
            let s: Skill = l.craft.skill();
            let now = p.stats.skill(s);
            p.stats.set_skill(s, (now + l.gain).min(l.cap.max(now)));
            if let Some(d) = p.detail.as_mut() {
                if !d.crafts.contains(&s) {
                    d.crafts.push(s);
                }
            }
            let name = p.name().unwrap_or("someone").to_string();
            p.recompute_might();
            self.log.push_front((l.done_at, format!("{name} has learned more of {}.", l.craft.name())));
            self.log.truncate(14);
        }
    }

    /// How far along someone's lesson is, 0..1.
    pub fn lesson_progress(&self, who: PersonId) -> Option<(Craft, f32)> {
        let l = self.lessons.iter().find(|l| l.who == who)?;
        let left = ((l.done_at - self.time) / HOUR).max(0.0) as f32;
        Some((l.craft, left))
    }

    // ---- Reading the books ------------------------------------------------------

    /// Goods a town lacks most, at its own prices (for the town panel).
    pub fn dearest(&self, town: SettlementId, n: usize) -> Vec<(Good, f32)> {
        let mut v: Vec<(Good, f32)> = GOODS.iter().map(|&g| (g, self.price_factor(town, g))).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }
}
