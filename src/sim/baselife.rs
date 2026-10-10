//! Life at an outpost (Part 7, stage 2): squad members left there, their
//! jobs, the base's store and its space, and the rounds of work that fill it.
//!
//! A resident works in rounds. A round starts when they're free and have
//! what it needs (a field to farm, grain and fuel to cook, a shed and the
//! materials for a recipe), takes its inputs from the store at once, and
//! ends at a fixed time on the world's timeline, when its output goes into
//! the store. Nothing is counted step by step, so a week away is the same
//! week as one watched. Builders don't work in rounds: they join the base's
//! builders (`base.rs`), on sites and mending.
//!
//! Residents stay on the condition timeline like the rest of the squad:
//! they eat from the base's store and sleep in its beds.

use serde::{Deserialize, Serialize};

use super::base::{Base, BaseId, Kind};
use super::condition::Shelter;
use super::crafting::{made_piece, success_chance, RECIPES};
use super::geo::V2;
use super::inventory::Entry;
use super::items::{self, item, ItemId, Kind as ItemKind};
use super::materials::Grade;
use super::person::PersonId;
use super::rng::Rng;
use super::world::{World, HOUR};

/// What a resident does at the base.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    Idle,
    /// Builds sites and mends what's run down.
    Builder,
    /// Works a field plot: a harvest of grain a day.
    Farmer,
    /// Bakes grain into flatbread at the hearth kitchen (timber for fuel).
    Cook,
    /// Works a recipe at a work shed, from the store's materials.
    Crafter,
    /// Fetches timber and stone from round about.
    Hauler,
    /// Keeps watch (counts toward the base's defence).
    Guard,
}

pub const JOBS: [Job; 7] = [Job::Idle, Job::Builder, Job::Farmer, Job::Cook, Job::Crafter, Job::Hauler, Job::Guard];

impl Job {
    pub fn name(self) -> &'static str {
        match self {
            Job::Idle => "Idle",
            Job::Builder => "Builder",
            Job::Farmer => "Farmer",
            Job::Cook => "Cook",
            Job::Crafter => "Crafter",
            Job::Hauler => "Hauler",
            Job::Guard => "Guard",
        }
    }
}

/// A squad member living at a base.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Resident {
    pub who: PersonId,
    pub job: Job,
    /// A crafter's recipe (index into `RECIPES`).
    pub recipe: Option<u16>,
    /// The round under way, if any.
    pub cycle: Option<Cycle>,
    /// Rounds worked so far (keys their rolls).
    pub n: u32,
    pub since: f64,
    /// A townsperson hired for wages (None: one of the squad).
    pub hire: Option<Hire>,
}

/// A round of work: when it ends, where, and for a crafter which recipe.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Cycle {
    pub done_at: f64,
    pub at: u32,
    pub recipe: Option<u16>,
}

// ---- The numbers (placeholders to tune) --------------------------------------------

/// Storage a base has before any building adds to it, kg.
pub const STORE_BASE: f32 = 100.0;
/// A field plot: one harvest a day for its farmer.
pub const FARM_HOURS: f64 = 24.0;
pub const FARM_YIELD: u16 = 6;
/// The hearth kitchen: grain and fuel in, flatbread out.
pub const COOK_HOURS: f64 = 2.0;
pub const COOK_IN: (u16, u16) = (2, 1);
pub const COOK_OUT: u16 = 3;
/// A hauler's round and what it brings: timber, then stone.
pub const HAUL_HOURS: f64 = 3.0;
pub const HAUL_YIELD: (u16, u16) = (2, 1);
/// The share of a store a hauler leaves free for food.
pub const HAUL_SPARE: f32 = 0.25;

impl Base {
    /// How much the store can hold, kg.
    pub fn capacity(&self) -> f32 {
        STORE_BASE + self.buildings.iter().filter(|b| b.standing()).map(|b| b.def().storage).sum::<f32>()
    }

    /// What the store holds, kg.
    pub fn load(&self) -> f32 {
        self.store.iter().map(|e| item(e.0).weight * e.1 as f32).sum()
    }

    pub fn room_for(&self, kg: f32) -> bool {
        self.load() + kg <= self.capacity() + 1e-3
    }

    pub fn resident(&self, who: PersonId) -> Option<&Resident> {
        self.residents.iter().find(|r| r.who == who)
    }

    /// Where a resident stands: at their work, or round the fire.
    pub fn spot(&self, who: PersonId) -> Option<V2> {
        let k = self.residents.iter().position(|r| r.who == who)?;
        let r = &self.residents[k];
        if let Some(bl) = r.cycle.as_ref().and_then(|c| self.building(c.at)) {
            let f = V2::new(bl.rot.cos(), bl.rot.sin());
            return Some(bl.at.add(f.scale(bl.def().d * 0.5 + 1.2)).add(V2::new(0.6 * k as f32, 0.0)));
        }
        if let Some(&(_, id, _)) = self.builders.iter().find(|h| h.0 == who) {
            if let Some(bl) = self.building(id) {
                let f = V2::new(bl.rot.cos(), bl.rot.sin());
                return Some(bl.at.add(f.scale(bl.def().d * 0.5 + 1.0)).add(V2::new(0.0, 0.8 * k as f32)));
            }
        }
        let a = k as f32 * 1.3 + 0.4;
        Some(self.at.add(V2::new(a.cos(), a.sin()).scale(3.0)))
    }

    /// A standing building of this kind nobody else is working.
    fn free(&self, kind: Kind, station: Option<super::crafting::Station>, me: PersonId) -> Option<u32> {
        self.buildings
            .iter()
            .filter(|b| b.standing() && b.def().kind == kind && (station.is_none() || b.def().station == station))
            .find(|b| !self.residents.iter().any(|r| r.who != me && r.cycle.as_ref().is_some_and(|c| c.at == b.id)))
            .map(|b| b.id)
    }

    fn take(&mut self, key: &str, n: u16) -> bool {
        let id = items::id(key);
        if self.count_in_store(id) < n {
            return false;
        }
        let mut left = n;
        for e in self.store.iter_mut().filter(|e| e.0 == id && e.2.is_none()) {
            let k = e.1.min(left);
            e.1 -= k;
            left -= k;
        }
        self.store.retain(|e| e.1 > 0);
        true
    }
}

impl World {
    /// The base someone lives at, and their place in its list.
    pub fn resident_of(&self, who: PersonId) -> Option<(BaseId, usize)> {
        self.bases.iter().find_map(|b| b.residents.iter().position(|r| r.who == who).map(|k| (b.id, k)))
    }

    fn base_i(&self, bid: BaseId) -> Option<usize> {
        self.bases.iter().position(|b| b.id == bid)
    }

    /// Leave a squad member at a base they're at: they live and work there,
    /// out of the travelling squad, until picked up.
    pub fn leave_at_base(&mut self, who: PersonId, bid: BaseId) -> Result<(), &'static str> {
        let Some(k) = self.squad.index(who) else { return Err("not in the squad") };
        let Some(i) = self.base_i(bid) else { return Err("no such base") };
        if self.squad.members.len() <= 1 {
            return Err("the squad needs someone to travel");
        }
        if !self.bases[i].present.contains(&who) {
            return Err("they must be at the base");
        }
        if self.fighting.contains_key(&who) || self.carrying(who).is_some() || self.carried_by(who).is_some() || self.crafting.iter().any(|j| j.who == who) || self.squad.inside[k].is_some() {
            return Err("they're busy");
        }
        let t = self.time;
        self.settle_condition(who, t);
        self.set_activity(who, super::condition::Activity::Resting);
        self.squad.remove(k);
        self.gathering.retain(|g| g.0 != who);
        let b = &mut self.bases[i];
        b.present.retain(|&m| m != who);
        b.residents.push(Resident { who, job: Job::Idle, recipe: None, cycle: None, n: 0, since: t, hire: None });
        let (name, bname) = (self.people[who as usize].name().unwrap_or("someone").to_string(), self.bases[i].name.clone());
        self.base_note(bid, t, format!("{name} stays behind at {bname}."), true);
        self.base_changed(bid);
        Ok(())
    }

    /// Take a resident back into the squad. Someone from the squad must be
    /// at the base to collect them.
    pub fn pick_up(&mut self, who: PersonId) -> Result<(), &'static str> {
        let Some((bid, k)) = self.resident_of(who) else { return Err("not living at a base") };
        let i = self.base_i(bid).unwrap();
        if self.bases[i].residents[k].hire.is_some() {
            return self.dismiss(who);
        }
        if self.bases[i].present.is_empty() {
            return Err("nobody from the squad is there to collect them");
        }
        // The travelling squad has its limit however people join it (NM-8).
        if self.squad.members.len() >= super::recruit::MAX_SQUAD {
            return Err("the squad is full");
        }
        let t = self.time;
        let at = self.bases[i].spot(who).unwrap_or(self.bases[i].at);
        // A round under way is dropped; what went into it goes back.
        self.drop_round(i, k);
        self.bases[i].residents.remove(k);
        self.bases[i].builders.retain(|h| h.0 != who);
        self.settle_condition(who, t);
        self.squad.add(who, at);
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        let bname = self.bases[i].name.clone();
        self.base_note(bid, t, format!("{name} rejoins the squad at {bname}."), true);
        self.base_changed(bid);
        Ok(())
    }

    /// Let a hired hand go: they're paid what's owed (if there's coin) and
    /// go home without bad blood.
    pub fn dismiss(&mut self, who: PersonId) -> Result<(), &'static str> {
        let Some((bid, k)) = self.resident_of(who) else { return Err("not living at a base") };
        let i = self.base_i(bid).unwrap();
        let Some(h) = self.bases[i].residents[k].hire.clone() else { return Err("one of the squad") };
        let coin = items::id("coin");
        // What's owed from earlier dawns, and the part of today's wage for
        // the hours since the last dawn (or since they got here): NM-6.
        let t = self.time;
        let dawn = super::society::DAWN as f64 * HOUR;
        let last_dawn = ((t - dawn) / super::world::DAY).floor() * super::world::DAY + dawn;
        let worked = ((t - last_dawn.max(h.arrives)) / super::world::DAY).clamp(0.0, 1.0) as f32;
        let due = h.owed + (h.wage as f32 * worked).round() as u16;
        let pay = self.squad_count(coin).min(due);
        self.take_from_squad(coin, pay);
        if let Some(hh) = h.household {
            self.society.households[hh as usize].purse.coin += pay as f32;
        }
        let t = self.time;
        self.hand_leaves(i, who, t, "let go");
        self.base_changed(bid);
        Ok(())
    }

    /// Set a resident's job.
    pub fn set_base_job(&mut self, who: PersonId, job: Job) {
        let Some((bid, k)) = self.resident_of(who) else { return };
        let i = self.base_i(bid).unwrap();
        let t = self.time;
        self.bases[i].settle_now(t);
        if self.bases[i].residents[k].job == job {
            return;
        }
        // The round under way is dropped, and what went into it goes back
        // to the store: changing someone's job costs nothing (NM-7).
        self.drop_round(i, k);
        let r = &mut self.bases[i].residents[k];
        r.job = job;
        if job != Job::Crafter {
            r.recipe = None;
        }
        self.base_changed(bid);
    }

    /// Drop resident `k`'s round at base `i`, if one is under way: the grain
    /// and timber of a bake, or a recipe's parts, go back into the store.
    fn drop_round(&mut self, i: usize, k: usize) {
        let Some(c) = self.bases[i].residents[k].cycle.take() else { return };
        match self.bases[i].residents[k].job {
            Job::Cook => {
                self.bases[i].add_to_store(items::id("grain"), COOK_IN.0);
                self.bases[i].add_to_store(items::id("timber"), COOK_IN.1);
            }
            Job::Crafter => {
                if let Some(ri) = c.recipe {
                    for &(key, n) in RECIPES[ri as usize].inputs {
                        self.bases[i].add_to_store(items::id(key), n);
                    }
                }
            }
            _ => {}
        }
    }

    /// The recipes a crafter could work at this base's sheds.
    pub fn base_recipes(&self, who: PersonId) -> Vec<u16> {
        let Some((bid, _)) = self.resident_of(who) else { return Vec::new() };
        let b = self.base(bid).unwrap();
        RECIPES
            .iter()
            .enumerate()
            .filter(|(_, rc)| self.knows_craft(who, rc.craft()) && rc.skill != super::stats::Skill::Tending && b.buildings.iter().any(|bl| bl.standing() && bl.def().station == Some(rc.station)))
            .map(|(k, _)| k as u16)
            .collect()
    }

    /// Set a crafter's recipe.
    pub fn set_base_recipe(&mut self, who: PersonId, recipe: Option<u16>) {
        let Some((bid, k)) = self.resident_of(who) else { return };
        let i = self.base_i(bid).unwrap();
        if self.bases[i].residents[k].recipe == recipe {
            return;
        }
        self.bases[i].residents[k].recipe = recipe;
        if self.bases[i].residents[k].cycle.is_none() {
            self.base_changed(bid);
        }
    }

    /// Seal a building's thatch with pitch from the store: it stops rotting.
    pub fn seal_building(&mut self, bid: BaseId, id: u32) -> Result<(), &'static str> {
        let Some(i) = self.base_i(bid) else { return Err("no such base") };
        let Some(k) = self.bases[i].buildings.iter().position(|b| b.id == id) else { return Err("no such building") };
        if !self.bases[i].buildings[k].rots() {
            return Err("nothing to seal");
        }
        let t = self.time;
        self.bases[i].settle_now(t);
        if !self.bases[i].take("pitch", 1) {
            return Err("no pitch in the store");
        }
        self.bases[i].buildings[k].sealed = true;
        self.base_changed(bid);
        Ok(())
    }

    // ---- Rounds of work ----------------------------------------------------------

    /// Start resident `r`'s next round at base `i`, if they can.
    pub(super) fn start_cycle(&mut self, i: usize, r: usize, t: f64) {
        let res = self.bases[i].residents[r].clone();
        let who = res.who;
        if self.people[who as usize].dead || self.on_the_way(i, r, t) {
            return;
        }
        let b = &self.bases[i];
        let plan: Option<(f64, u32, Option<u16>)> = match res.job {
            Job::Idle | Job::Builder | Job::Guard => None,
            Job::Farmer => b.free(Kind::Field, None, who).map(|id| (FARM_HOURS, id, None)),
            Job::Cook => {
                // Room for the bread once the grain and timber have come out
                // (a bake usually frees room: NM-5).
                let freed = item(items::id("grain")).weight * COOK_IN.0 as f32 + item(items::id("timber")).weight * COOK_IN.1 as f32;
                let room = b.room_for(item(items::id("flatbread")).weight * COOK_OUT as f32 - freed);
                match b.free(Kind::Kitchen, None, who) {
                    Some(id) if room && b.count_in_store(items::id("grain")) >= COOK_IN.0 && b.count_in_store(items::id("timber")) >= COOK_IN.1 => Some((COOK_HOURS, id, None)),
                    _ => None,
                }
            }
            Job::Hauler => {
                // A hauler stops short of a full store, so there's room for
                // the harvest and the baking (NM-5).
                let kg = item(items::id("timber")).weight * HAUL_YIELD.0 as f32 + item(items::id("rock")).weight * HAUL_YIELD.1 as f32;
                b.room_for(kg + b.capacity() * HAUL_SPARE).then_some((HAUL_HOURS, u32::MAX, None))
            }
            Job::Crafter => res.recipe.and_then(|ri| {
                let rc = &RECIPES[ri as usize];
                let has = rc.inputs.iter().all(|&(k, n)| b.count_in_store(items::id(k)) >= n);
                let out = rc.item(Grade::Common);
                let room = b.room_for(item(out).weight * rc.makes as f32);
                let shed = b.free(Kind::Shed, Some(rc.station), who);
                match (has && room && self.knows_craft(who, rc.craft()), shed) {
                    (true, Some(id)) => Some((rc.time / HOUR, id, Some(ri))),
                    _ => None,
                }
            }),
        };
        let Some((hours, at, recipe)) = plan else { return };
        // Inputs go in now.
        let b = &mut self.bases[i];
        match res.job {
            Job::Cook => {
                b.take("grain", COOK_IN.0);
                b.take("timber", COOK_IN.1);
            }
            Job::Crafter => {
                let rc = &RECIPES[recipe.unwrap() as usize];
                for &(k, n) in rc.inputs {
                    b.take(k, n);
                }
            }
            _ => {}
        }
        let r = &mut b.residents[r];
        r.cycle = Some(Cycle { done_at: t + hours * HOUR, at, recipe });
    }

    /// Resident `who`'s round at base `i` is over: its output goes in.
    pub(super) fn finish_cycle(&mut self, i: usize, who: PersonId, t: f64) {
        let Some(k) = self.bases[i].residents.iter().position(|r| r.who == who) else { return };
        let Some(c) = self.bases[i].residents[k].cycle.take() else { return };
        self.bases[i].residents[k].n += 1;
        let n = self.bases[i].residents[k].n;
        let job = self.bases[i].residents[k].job;
        let bid = self.bases[i].id;
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        match job {
            Job::Farmer => {
                self.bases[i].add_to_store(items::id("grain"), FARM_YIELD);
                self.base_note(bid, t, format!("{name} brings in {FARM_YIELD} grain."), false);
            }
            Job::Cook => {
                self.bases[i].add_to_store(items::id("flatbread"), COOK_OUT);
            }
            Job::Hauler => {
                self.bases[i].add_to_store(items::id("timber"), HAUL_YIELD.0);
                self.bases[i].add_to_store(items::id("rock"), HAUL_YIELD.1);
            }
            Job::Crafter => {
                let Some(ri) = c.recipe else { return };
                let rc = &RECIPES[ri as usize];
                let skill = self.work_skill(who, rc.craft()).max(self.people[who as usize].effective_stats().skill(rc.skill));
                let mut roll = Rng::from_keys(&[self.seed, who as u64, bid as u64, n as u64, 0x4241_5345]);
                let ok = roll.f32() < success_chance(skill, rc.difficulty);
                let grade = Grade::from(skill, 1.0, roll.f32());
                self.people[who as usize].stats.exercise(rc.skill, if ok { 2.0 } else { 0.8 });
                if ok {
                    let id = rc.item(grade);
                    let town = None;
                    if rc.is_piece() {
                        let pc = made_piece(id, who, town, skill, rc.seals(), t);
                        self.bases[i].store.push(Entry(id, 1, Some(pc)));
                    } else {
                        self.bases[i].add_to_store(id, rc.makes);
                    }
                    self.base_note(bid, t, format!("{name} makes {}.", item(id).name.to_lowercase()), false);
                } else {
                    for &(key, m) in rc.inputs {
                        self.bases[i].add_to_store(items::id(key), m / 2);
                    }
                    self.base_note(bid, t, format!("{name} botches the {}.", item(rc.item(Grade::Common)).name.to_lowercase()), false);
                }
            }
            Job::Idle | Job::Builder | Job::Guard => {}
        }
    }

    // ---- Living there --------------------------------------------------------------

    /// The best thing in a resident's base store to eat.
    pub(super) fn base_food_for(&self, who: PersonId, hunger: f32) -> Option<ItemId> {
        let (bid, _) = self.resident_of(who)?;
        let b = self.base(bid)?;
        let foods: Vec<(ItemId, f32)> = b.store.iter().filter_map(|e| if let ItemKind::Food(n) = item(e.0).kind { Some((e.0, n)) } else { None }).collect();
        foods.iter().filter(|f| f.1 <= hunger).max_by(|a, b| a.1.total_cmp(&b.1)).or_else(|| foods.iter().min_by(|a, b| a.1.total_cmp(&b.1))).map(|f| f.0)
    }

    /// Take one of `it` from a resident's base store (for a meal).
    pub(super) fn base_take_food(&mut self, who: PersonId, it: ItemId) -> bool {
        let Some((bid, _)) = self.resident_of(who) else { return false };
        let i = self.base_i(bid).unwrap();
        let key = item(it).key;
        self.bases[i].take(key, 1)
    }

    /// Where someone at a base sleeps: a bed (huts and longhouses indoors,
    /// a lean-to as good as a tent), residents first.
    pub(super) fn base_bed(&self, who: PersonId) -> Option<Shelter> {
        let b = self.bases.iter().find(|b| b.residents.iter().any(|r| r.who == who) || b.present.contains(&who))?;
        let order: Vec<PersonId> = b.residents.iter().map(|r| r.who).chain(b.present.iter().copied()).collect();
        let place = order.iter().position(|&m| m == who)?;
        let mut beds: Vec<Shelter> = Vec::new();
        for bl in b.buildings.iter().filter(|x| x.standing()) {
            let d = bl.def();
            let s = if d.key == "lean_to" { Shelter::Tent } else { Shelter::Indoors };
            for _ in 0..d.beds {
                beds.push(s);
            }
        }
        beds.sort_by_key(|s| if *s == Shelter::Indoors { 0 } else { 1 });
        beds.get(place).copied()
    }

    /// Every resident of every base.
    pub fn all_residents(&self) -> Vec<PersonId> {
        self.bases.iter().flat_map(|b| b.residents.iter().map(|r| r.who)).collect()
    }
}

impl Base {
    /// Settle at `t` (labour and health) without re-planning.
    pub(super) fn settle_now(&mut self, t: f64) {
        // (The same settling the base does on any change.)
        self.settle_pub(t);
    }

    /// Building ids by kind that stand.
    pub fn standing_of(&self, kind: Kind) -> usize {
        self.buildings.iter().filter(|b| b.standing() && b.def().kind == kind).count()
    }
}

/// A building's name for a resident's round.
pub fn round_place(b: &Base, c: &Cycle) -> String {
    match b.building(c.at) {
        Some(bl) => bl.def().name.to_lowercase(),
        None => "round about".to_string(),
    }
}

// ---- Hired hands (stage 3) -----------------------------------------------------------

/// What a townsperson hired to live and work at a base keeps.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Hire {
    /// Coin a day, paid at dawn.
    pub wage: u16,
    /// 0..1: below `QUIT_AT` they leave.
    pub loyalty: f32,
    /// Where they came from, and what they were there (they go back to it).
    pub town: super::settlement::SettlementId,
    pub household: Option<u32>,
    pub community: Option<u32>,
    pub trade: super::jobs::Job,
    /// When they reach the base (they walk from town).
    pub arrives: f64,
    /// Wages owed and not yet paid.
    pub owed: u16,
}

/// Loyalty a hand starts with, and what moves it at each dawn.
pub const LOYALTY_START: f32 = 0.6;
pub const LOYALTY_CONTENT: f32 = 0.03;
pub const LOYALTY_UNPAID: f32 = 0.15;
pub const LOYALTY_UNFED: f32 = 0.10;
pub const LOYALTY_NO_BED: f32 = 0.04;
/// Below this they quit at dawn.
pub const QUIT_AT: f32 = 0.2;
/// What surviving a raid at the base (or being protected) adds (stage 4).
pub const LOYALTY_HELD: f32 = 0.15;
/// Nourishment a hand needs a day (from the store's food).
pub const HAND_FOOD: f32 = 50.0;
/// Walking pace of a hand going to a base, metres an hour.
pub const HAND_WALK: f32 = 4000.0;
/// The most a thief takes on the way out, kg.
pub const THEFT_KG: f32 = 20.0;

impl World {
    /// Someone's trade: their job, or for a hired hand the one they left.
    pub fn trade_of(&self, who: PersonId) -> super::jobs::Job {
        let j = self.society.lives.get(who as usize).map(|l| l.job).unwrap_or(super::jobs::Job::None);
        if j != super::jobs::Job::None {
            return j;
        }
        self.bases.iter().flat_map(|b| b.residents.iter()).find(|r| r.who == who).and_then(|r| r.hire.as_ref()).map(|h| h.trade).unwrap_or(j)
    }

    /// How good someone is at a building skill: their own, or a hired
    /// carpenter's or mason's years at it.
    pub fn build_skill(&self, who: PersonId, s: super::stats::Skill) -> f32 {
        match super::materials::Craft::of_skill(s) {
            Some(c) => self.work_skill(who, c),
            None => self.people[who as usize].stats.skill(s),
        }
    }

    /// What this townsperson would ask a day to work at the squad's base,
    /// if they'd come at all.
    pub fn hire_terms(&self, npc: PersonId) -> Option<u16> {
        let p = &self.people[npc as usize];
        if p.dead || p.in_squad || p.bandit || self.group_of[npc as usize].is_some() || self.bases.is_empty() || self.resident_of(npc).is_some() {
            return None;
        }
        let life = self.society.lives.get(npc as usize)?;
        life.community?;
        let m = self.mind(npc);
        use super::jobs::Job as J;
        let loose = matches!(life.job, J::Labourer | J::Drifter | J::None) || m.work == super::lives::Work::Jobless;
        let builders = matches!(life.job, J::Carpenter | J::Mason);
        let wants = m.needs()[0] > 0.4 || p.traits.wanderlust > 0.6;
        if !(loose || builders || wants) || matches!(m.work, super::lives::Work::Bonded | super::lives::Work::Injured) {
            return None;
        }
        // More than their work at home pays: they give up home and household.
        Some(((life.job.pay().max(0.7) * 14.0).ceil() as u16).max(8))
    }

    /// The squad's base nearest a point.
    fn nearest_base(&self, p: V2) -> Option<BaseId> {
        self.bases.iter().min_by(|a, b| a.at.dist(p).total_cmp(&b.at.dist(p))).map(|b| b.id)
    }

    /// Hire a townsperson to live and work at the squad's nearest base. They
    /// leave their post, home and household and walk out there.
    pub fn hire(&mut self, npc: PersonId) -> Result<BaseId, &'static str> {
        let wage = self.hire_terms(npc).ok_or("they won't come")?;
        let at = self.person_pos(npc);
        let bid = self.nearest_base(at).ok_or("you've no base")?;
        let i = self.base_i(bid).unwrap();
        let t = self.time;
        let town = self.people[npc as usize].home.unwrap_or(0);
        self.people[npc as usize].ensure_detail();
        let life = &mut self.society.lives[npc as usize];
        let (trade, household, community) = (life.job, life.household, life.community);
        if trade.is_post() {
            self.society.minds[npc as usize].lost = Some((trade, super::lives::Loss::Away));
        }
        life.job = super::jobs::Job::None;
        life.place = None;
        life.community = None;
        life.household = None;
        if let Some(h) = household {
            self.society.households[h as usize].members.retain(|&m| m != npc);
        }
        self.settlements[town as usize].residents.retain(|&m| m != npc);
        self.refresh_container_owners();
        let arrives = t + (at.dist(self.bases[i].at) / HAND_WALK) as f64 * HOUR;
        // With no hands until now, no dawn has been tallied here: the first
        // is the next one, not every dawn since the base was founded (NM-1).
        if !self.bases[i].residents.iter().any(|r| r.hire.is_some()) {
            let last = ((t - super::society::DAWN as f64 * HOUR) / super::world::DAY).floor() as i64;
            self.bases[i].dawn_done = self.bases[i].dawn_done.max(last);
        }
        self.bases[i].residents.push(Resident { who: npc, job: Job::Idle, recipe: None, cycle: None, n: 0, since: t, hire: Some(Hire { wage, loyalty: LOYALTY_START, town, household, community, trade, arrives, owed: 0 }) });
        let name = self.people[npc as usize].name().unwrap_or("someone").to_string();
        let bname = self.bases[i].name.clone();
        self.base_note(bid, t, format!("{name} is hired at {wage} coin a day, and sets out for {bname}."), true);
        self.base_changed(bid);
        Ok(bid)
    }

    /// Is this resident a hired hand still on the road?
    fn on_the_way(&self, i: usize, r: usize, t: f64) -> bool {
        self.bases[i].residents[r].hire.as_ref().is_some_and(|h| h.arrives > t)
    }

    /// A hand goes home: back to their town, household and community, as a
    /// labourer (their old post has likely been filled).
    fn hand_leaves(&mut self, i: usize, who: PersonId, t: f64, why: &str) {
        let Some(k) = self.bases[i].residents.iter().position(|r| r.who == who) else { return };
        self.drop_round(i, k);
        let r = self.bases[i].residents.remove(k);
        self.bases[i].builders.retain(|h| h.0 != who);
        let Some(h) = r.hire else { return };
        let life = &mut self.society.lives[who as usize];
        life.job = super::jobs::Job::Labourer;
        life.community = h.community;
        life.household = h.household;
        if let Some(hh) = h.household {
            if !self.society.households[hh as usize].members.contains(&who) {
                self.society.households[hh as usize].members.push(who);
            }
        }
        if !self.settlements[h.town as usize].residents.contains(&who) {
            self.settlements[h.town as usize].residents.push(who);
        }
        self.refresh_container_owners();
        let bid = self.bases[i].id;
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        self.base_note(bid, t, format!("{name} quits and goes home: {why}."), true);
    }

    /// Hands who live through a raid at the base, or see the squad stand by
    /// them, think better of it (stage 4 calls this).
    #[allow(dead_code)]
    pub(super) fn hands_held(&mut self, bid: BaseId, by: f32) {
        let Some(i) = self.base_i(bid) else { return };
        for r in &mut self.bases[i].residents {
            if let Some(h) = r.hire.as_mut() {
                h.loyalty = (h.loyalty + by).min(1.0);
            }
        }
    }

    /// Dawn at a base: wages, food and beds for the hired hands, loyalty,
    /// and anyone who's had enough. On the world's timeline, at the dawn's `t`.
    pub(super) fn base_dawn(&mut self, i: usize, t: f64) {
        let bid = self.bases[i].id;
        self.bases[i].dawn_done = (t / super::world::DAY).floor() as i64;
        // Beds go to the squad's own first.
        let beds = self.bases[i].beds() as usize;
        let own = self.bases[i].residents.iter().filter(|r| r.hire.is_none()).count();
        let mut bed_left = beds.saturating_sub(own);
        let hands: Vec<PersonId> = self.bases[i].residents.iter().filter(|r| r.hire.as_ref().is_some_and(|h| h.arrives <= t)).map(|r| r.who).collect();
        let coin = items::id("coin");
        let mut quits: Vec<(PersonId, &'static str)> = Vec::new();
        for who in hands {
            let k = self.bases[i].residents.iter().position(|r| r.who == who).unwrap();
            let wage = self.bases[i].residents[k].hire.as_ref().unwrap().wage;
            let owed = self.bases[i].residents[k].hire.as_ref().unwrap().owed + wage;
            // Pay: from the base's store, then the squad's purse.
            let from_store = self.bases[i].count_in_store(coin).min(owed);
            if from_store > 0 {
                self.bases[i].take("coin", from_store);
            }
            let from_squad = self.squad_count(coin).min(owed - from_store);
            if from_squad > 0 {
                self.take_from_squad(coin, from_squad);
            }
            let unpaid = owed - from_store - from_squad;
            // The pay goes home to their household.
            if let Some(hh) = self.bases[i].residents[k].hire.as_ref().unwrap().household {
                self.society.households[hh as usize].purse.coin += (from_store + from_squad) as f32;
            }
            // Food: two meals' worth from the store.
            let mut fed = 0.0f32;
            while fed < HAND_FOOD {
                let pick = self.bases[i].store.iter().filter_map(|e| if let ItemKind::Food(n) = item(e.0).kind { Some((e.0, n)) } else { None }).max_by(|a, b| a.1.total_cmp(&b.1));
                let Some((it, n)) = pick else { break };
                self.bases[i].take(item(it).key, 1);
                fed += n;
            }
            let bed = bed_left > 0;
            bed_left = bed_left.saturating_sub(1);
            let h = self.bases[i].residents[k].hire.as_mut().unwrap();
            h.owed = unpaid;
            let mut change = 0.0;
            if unpaid > 0 {
                change -= LOYALTY_UNPAID;
            }
            if fed < HAND_FOOD {
                change -= LOYALTY_UNFED;
            }
            if !bed {
                change -= LOYALTY_NO_BED;
            }
            if change == 0.0 {
                change = LOYALTY_CONTENT;
            }
            h.loyalty = (h.loyalty + change).clamp(0.0, 1.0);
            if h.loyalty < QUIT_AT {
                quits.push((who, if unpaid > 0 { "not paid" } else if fed < HAND_FOOD { "not fed" } else { "nowhere to sleep" }));
            }
        }
        for (who, why) in quits {
            // The less honourable help themselves on the way out.
            let honour = self.life(who).habits.honour;
            let day = (t / super::world::DAY) as u64;
            let mut roll = Rng::from_keys(&[self.seed, who as u64, bid as u64, day, 0x5155_4954]);
            let town = self.bases[i].residents.iter().find(|r| r.who == who).and_then(|r| r.hire.as_ref()).map(|h| h.town).unwrap_or(0);
            if roll.f32() < (0.5 - honour).max(0.0) * 1.5 {
                let took = self.hand_steals(i, who);
                if !took.is_empty() {
                    let name = self.people[who as usize].name().unwrap_or("someone").to_string();
                    self.base_note(bid, t, format!("{name} took {} on the way out.", took.join(", ")), true);
                    let victim = self.squad.members.first().copied();
                    self.note(super::history::Deed::Theft, Some(who), victim, town, t, false);
                }
            }
            let victim = self.squad.members.first().copied();
            self.note(super::history::Deed::WorkQuarrel, Some(who), victim, town, t, false);
            self.hand_leaves(i, who, t, why);
        }
    }

    /// A hand quitting in bad blood takes the dearest things they can carry
    /// from the store. What was taken, by name.
    fn hand_steals(&mut self, i: usize, who: PersonId) -> Vec<String> {
        let mut took = Vec::new();
        let mut kg = 0.0;
        loop {
            let pick = self.bases[i].store.iter().enumerate().filter(|(_, e)| kg + item(e.0).weight <= THEFT_KG).max_by(|a, b| item(a.1 .0).value.total_cmp(&item(b.1 .0).value)).map(|(k, _)| k);
            let Some(k) = pick else { break };
            let e = self.bases[i].store[k];
            kg += item(e.0).weight;
            if e.1 > 1 {
                self.bases[i].store[k].1 -= 1;
            } else {
                self.bases[i].store.remove(k);
            }
            if let Some(d) = self.people[who as usize].detail.as_mut() {
                match e.2 {
                    Some(pc) => d.gear.add_piece(e.0, pc),
                    None => d.gear.add(e.0, 1),
                }
            }
            let name = item(e.0).name.to_lowercase();
            if !took.contains(&name) {
                took.push(name);
            }
            if took.len() > 8 {
                break;
            }
        }
        took
    }
}
