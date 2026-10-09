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
        b.residents.push(Resident { who, job: Job::Idle, recipe: None, cycle: None, n: 0, since: t });
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
        if self.bases[i].present.is_empty() {
            return Err("nobody from the squad is there to collect them");
        }
        let t = self.time;
        let at = self.bases[i].spot(who).unwrap_or(self.bases[i].at);
        // A round under way is dropped; its inputs are lost with it.
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

    /// Set a resident's job.
    pub fn set_base_job(&mut self, who: PersonId, job: Job) {
        let Some((bid, k)) = self.resident_of(who) else { return };
        let i = self.base_i(bid).unwrap();
        let t = self.time;
        self.bases[i].settle_now(t);
        let r = &mut self.bases[i].residents[k];
        if r.job == job {
            return;
        }
        r.job = job;
        r.cycle = None;
        if job != Job::Crafter {
            r.recipe = None;
        }
        self.base_changed(bid);
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
        if self.people[who as usize].dead {
            return;
        }
        let b = &self.bases[i];
        let plan: Option<(f64, u32, Option<u16>)> = match res.job {
            Job::Idle | Job::Builder | Job::Guard => None,
            Job::Farmer => b.free(Kind::Field, None, who).map(|id| (FARM_HOURS, id, None)),
            Job::Cook => {
                let room = b.room_for(item(items::id("flatbread")).weight * COOK_OUT as f32);
                match b.free(Kind::Kitchen, None, who) {
                    Some(id) if room && b.count_in_store(items::id("grain")) >= COOK_IN.0 && b.count_in_store(items::id("timber")) >= COOK_IN.1 => Some((COOK_HOURS, id, None)),
                    _ => None,
                }
            }
            Job::Hauler => {
                let kg = item(items::id("timber")).weight * HAUL_YIELD.0 as f32 + item(items::id("rock")).weight * HAUL_YIELD.1 as f32;
                b.room_for(kg).then_some((HAUL_HOURS, u32::MAX, None))
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
                let skill = self.people[who as usize].effective_stats().skill(rc.skill);
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
