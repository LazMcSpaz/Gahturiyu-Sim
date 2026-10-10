//! The money grind (the playable-MVP list). Kenshi's first fortune is dug,
//! chopped and hunted:
//!
//! - **Deposits** at every town's woodlot and mine: a squad member sent there
//!   works steadily, a unit at a time, until their pack is full or the face
//!   is worked out. A deposit refills by the clock (its amount is stored at
//!   one moment and grows at a steady rate; never ticked).
//! - **Hunting** by order: click a wild animal and the selected walk over and
//!   set about its herd once in reach (`World::hunt`).
//! - **Butchering**: click a carcass and someone cuts it up: meat to eat,
//!   hides and the rest to sell (`World::butcher`, through `YIELD_ITEMS`).
//!
//! All of it is the squad's own doing, so it runs in the squad's step (the
//! player exception), but every unit falls due at a fixed time
//! (`Labour::next`), so how often the squad is stepped never changes what
//! comes of it.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items::{self, item, ItemId};
use super::jobs::{Good, PlaceKind};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::stats::{Attr, Skill};
use super::squad::REACH;
use super::world::{World, DAY, HOUR};

/// A face to work: a town's woodlot or mine.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Deposit {
    pub id: u32,
    pub town: SettlementId,
    pub pos: V2,
    pub item: ItemId,
    /// The most it holds.
    pub cap: u16,
    /// What was left at `at`.
    pub left: f32,
    pub at: f64,
}

/// What a kind of deposit gives and how hard it is to work.
pub struct Face {
    pub key: &'static str,
    pub name: &'static str,
    pub cap: u16,
    /// Minutes a unit takes an ordinary worker.
    pub minutes: f32,
    /// Share of `cap` that grows back each day.
    pub refill: f32,
}

/// The kinds of face (data; a new kind is a line here and a place for it).
pub const FACES: &[Face] = &[
    Face { key: "timber", name: "Woodlot", cap: 80, minutes: 5.0, refill: 0.6 },
    Face { key: "iron_ore", name: "Iron seam", cap: 60, minutes: 6.0, refill: 0.5 },
    Face { key: "gold_nugget", name: "Gold seam", cap: 6, minutes: 30.0, refill: 0.25 },
];

/// How much of a good the land round a town must offer for a mine to be dug.
pub const ORE_AT: f32 = 0.1;
pub const GOLD_AT: f32 = 0.5;

/// What a carcass's parts become: (yield name, item, items per unit of yield).
pub const YIELD_ITEMS: &[(&str, &str, f32)] = &[
    ("meat", "raw_meat", 1.0),
    ("small_meat", "raw_meat", 1.0),
    ("hide", "hide", 1.0),
    ("light_hide", "light_hide", 1.0),
    ("thick_hide", "thick_hide", 1.0),
    ("tough_skin", "tough_skin", 1.0),
    ("shell", "shell", 1.0),
    ("horn_plate", "horn_plate", 1.0),
    ("teeth", "teeth", 1.0),
    ("oil", "animal_oil", 0.5),
    ("egg", "egg", 1.0),
    ("lurker_trophy", "lurker_trophy", 1.0),
    ("cragmaw_trophy", "cragmaw_trophy", 1.0),
    ("silk_gland", "silk_gland", 1.0),
    ("briar_heart", "briar_heart", 1.0),
];

/// Minutes to cut up a carcass, per unit of the animal's size.
pub const BUTCHER_MINUTES: f32 = 15.0;
/// How far the hunted herd may move before the hunters' way is found again.
pub const CHASE_SLACK: f32 = 15.0;

/// Someone at work at a deposit. `next`: when their next unit is done
/// (set once they're there).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Labour {
    pub who: PersonId,
    pub deposit: u32,
    pub next: Option<f64>,
}

/// Someone cutting up a carcass. `done`: when they finish (set once there).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Butchering {
    pub who: PersonId,
    pub carcass: u32,
    pub done: Option<f64>,
}

/// Some of the squad going after a herd: `aim` is where they were sent.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Chase {
    pub who: Vec<PersonId>,
    pub herd: u32,
    pub aim: V2,
}

pub fn face_of(it: ItemId) -> Option<&'static Face> {
    FACES.iter().find(|f| items::id(f.key) == it)
}

impl Deposit {
    pub fn face(&self) -> &'static Face {
        face_of(self.item).expect("a deposit's item has a face")
    }

    /// How much is there at `t` (it grows back at a steady rate).
    pub fn left_at(&self, t: f64) -> f32 {
        let f = self.face();
        (self.left + f.refill * f.cap as f32 * ((t - self.at).max(0.0) / DAY) as f32).min(f.cap as f32)
    }
}

impl World {
    // ---- Setting up ---------------------------------------------------------

    /// A face at every town's woodlot, and at its mine where the land has ore
    /// (and a gold seam where it has gold). From the workplaces, so the same
    /// world always gets the same.
    pub(super) fn place_deposits(&mut self) {
        let mut out = Vec::new();
        for (town, tl) in self.society.towns.iter().enumerate() {
            for wp in &tl.places {
                let off = |k: f32| wp.pos.add(V2::new((wp.rot + k).cos(), (wp.rot + k).sin()).scale(7.0));
                let mut put = |key: &str, at: V2| {
                    let f = face_of(items::id(key)).unwrap();
                    out.push(Deposit { id: out.len() as u32, town: town as SettlementId, pos: at, item: items::id(key), cap: f.cap, left: f.cap as f32, at: 0.0 });
                };
                match wp.kind {
                    PlaceKind::Woodlot => put("timber", off(1.6)),
                    PlaceKind::Quarry => {
                        if tl.sources[Good::Ore.index()] >= ORE_AT {
                            put("iron_ore", off(0.0));
                        }
                        if tl.sources[Good::Gold.index()] >= GOLD_AT {
                            put("gold_nugget", off(3.1));
                        }
                    }
                    _ => {}
                }
            }
        }
        self.deposits = out;
    }

    pub fn deposit(&self, id: u32) -> Option<&Deposit> {
        self.deposits.get(id as usize)
    }

    /// Stop whatever grind work someone was doing (a new order).
    pub(super) fn stop_work(&mut self, who: PersonId) {
        self.labour.retain(|l| l.who != who);
        self.butchering.retain(|b| b.who != who);
        for c in &mut self.chases {
            c.who.retain(|&m| m != who);
        }
        self.chases.retain(|c| !c.who.is_empty());
    }

    /// Send a member to a spot: by the way round buildings, or straight
    /// there if that way stops short of it.
    pub(super) fn send(&mut self, who: PersonId, to: V2) -> bool {
        let Some(k) = self.squad.index(who) else { return false };
        let (mut path, _) = self.route(self.member_pos(k), to);
        if path.last().is_some_and(|e| e.dist(to) > REACH) {
            path.push(to);
        }
        self.squad.goal[k] = *path.last().unwrap_or(&to);
        self.squad.route[k] = path;
        self.squad.resting[k] = false;
        true
    }

    /// Send someone to work a deposit until their pack is full.
    pub fn order_labour(&mut self, who: PersonId, deposit: u32) -> bool {
        let Some(d) = self.deposit(deposit).copied() else { return false };
        if self.squad.index(who).is_none() {
            return false;
        }
        self.stop_work(who);
        self.gathering.retain(|g| g.0 != who);
        // Side by side, not on top of each other.
        let n = self.labour.iter().filter(|l| l.deposit == deposit).count() as f32;
        let a = n * 2.4 + deposit as f32;
        let spot = if n == 0.0 { d.pos } else { d.pos.add(V2::new(a.cos(), a.sin()).scale(1.6)) };
        self.send(who, spot);
        self.labour.push(Labour { who, deposit, next: None });
        true
    }

    /// Minutes a unit takes this worker: the strong and fit work faster.
    pub fn unit_minutes(&self, who: PersonId, deposit: u32) -> f32 {
        let Some(d) = self.deposit(deposit) else { return f32::MAX };
        let s = &self.people[who as usize].stats;
        let knack = 0.6 + s.attr(Attr::Strength) / 100.0 + s.skill(Skill::Athletics) / 150.0;
        d.face().minutes / knack.max(0.3)
    }

    /// Is someone at a deposit, working it?
    pub fn labouring(&self, who: PersonId) -> Option<u32> {
        self.labour.iter().find(|l| l.who == who && l.next.is_some()).map(|l| l.deposit)
    }

    /// Would one more of this fit in their pack?
    fn room_for(&self, who: PersonId, it: ItemId, t: f64) -> bool {
        self.kit_weight_at(who, t) + self.burden_weight(who) + item(it).weight <= self.capacity_at(who, t)
    }

    /// Settle someone's condition at `t`, or at the start of their current
    /// piece if that's later (a change can't go back before it).
    fn settle_at(&mut self, who: PersonId, t: f64) {
        let from = self.people[who as usize].cond.as_ref().map(|c| c.at).unwrap_or(t);
        self.settle_condition(who, t.max(from));
    }

    fn work_note(&mut self, t: f64, line: String) {
        self.log.push_front((t, line));
        self.log.truncate(14);
    }

    /// Work the deposits, butcher the carcasses and chase the herds the squad
    /// was sent to (the squad's step).
    pub(super) fn do_labour(&mut self) {
        let now = self.time;
        let mut k = 0;
        while k < self.labour.len() {
            let l = self.labour[k];
            let (Some(i), Some(d)) = (self.squad.index(l.who), self.deposit(l.deposit).copied()) else {
                self.labour.remove(k);
                continue;
            };
            let busy = self.fighting.contains_key(&l.who) || self.is_down(l.who);
            if busy || self.squad.at[i].dist(d.pos) > REACH * 2.0 {
                // Not there yet (or knocked off it): the clock starts on arrival.
                if l.next.is_some() && busy {
                    self.labour.remove(k);
                    continue;
                }
                k += 1;
                continue;
            }
            let mut next = l.next.unwrap_or(now + self.unit_minutes(l.who, l.deposit) as f64 * 60.0);
            let mut done: Option<&'static str> = None;
            let mut got = 0u16;
            while next <= now {
                if !self.room_for(l.who, d.item, next) {
                    done = Some("their pack is full");
                    break;
                }
                let dd = &mut self.deposits[l.deposit as usize];
                let left = dd.left_at(next);
                if left < 1.0 {
                    done = Some("it's worked out for now");
                    break;
                }
                dd.left = left - 1.0;
                dd.at = next;
                self.settle_at(l.who, next);
                if let Some(det) = self.people[l.who as usize].detail.as_mut() {
                    det.gear.add(d.item, 1);
                }
                self.people[l.who as usize].stats.exercise(Skill::Athletics, 0.25);
                self.people[l.who as usize].recompute_might();
                got += 1;
                next += self.unit_minutes(l.who, l.deposit) as f64 * 60.0;
            }
            let _ = got;
            match done {
                Some(why) => {
                    let name = self.people[l.who as usize].name().unwrap_or("someone").to_string();
                    let n = self.count_of(l.who, item(d.item).key);
                    self.work_note(now, format!("{name} stops at the {}: {why} ({n} {} carried).", d.face().name.to_lowercase(), item(d.item).name.to_lowercase()));
                    self.labour.remove(k);
                }
                None => {
                    self.labour[k].next = Some(next);
                    k += 1;
                }
            }
        }
        self.do_butchering(now);
        self.do_chases(now);
    }

    // ---- Butchering -----------------------------------------------------------

    /// Send someone to cut up a carcass.
    pub fn order_butcher(&mut self, who: PersonId, carcass: u32) -> bool {
        let Some(c) = self.animals.carcasses.iter().find(|c| c.id == carcass && c.gone_at > self.time) else { return false };
        let at = c.pos;
        if self.squad.index(who).is_none() || c.sp.def().yields.is_empty() {
            return false;
        }
        self.stop_work(who);
        self.send(who, at);
        self.butchering.push(Butchering { who, carcass, done: None });
        true
    }

    /// Is someone cutting up a carcass?
    pub fn butchering_now(&self, who: PersonId) -> bool {
        self.butchering.iter().any(|b| b.who == who && b.done.is_some())
    }

    fn do_butchering(&mut self, now: f64) {
        let mut k = 0;
        while k < self.butchering.len() {
            let b = self.butchering[k];
            let Some(c) = self.animals.carcasses.iter().find(|c| c.id == b.carcass && c.gone_at > now).cloned() else {
                self.butchering.remove(k);
                continue;
            };
            let Some(i) = self.squad.index(b.who) else {
                self.butchering.remove(k);
                continue;
            };
            if self.squad.at[i].dist(c.pos) > REACH * 1.5 || self.fighting.contains_key(&b.who) {
                k += 1;
                continue;
            }
            let done = b.done.unwrap_or(now + (BUTCHER_MINUTES * c.size.clamp(0.3, 4.0)) as f64 * 60.0);
            if done > now {
                self.butchering[k].done = Some(done);
                k += 1;
                continue;
            }
            self.butchering.remove(k);
            let parts = self.butcher(c.id);
            let mut r = Rng::from_keys(&[self.seed, c.key, 0x4255_5443]);
            let mut got: Vec<String> = Vec::new();
            for (name, amount) in parts {
                let Some(&(_, key, per)) = YIELD_ITEMS.iter().find(|y| y.0 == name) else { continue };
                let a = amount * per;
                let n = a.floor() as u16 + u16::from(r.f32() < a.fract());
                if n == 0 {
                    continue;
                }
                if let Some(d) = self.people[b.who as usize].detail.as_mut() {
                    d.gear.add(items::id(key), n);
                }
                got.push(format!("{n} × {}", item(items::id(key)).name.to_lowercase()));
            }
            self.settle_at(b.who, done);
            self.people[b.who as usize].recompute_might();
            let name = self.people[b.who as usize].name().unwrap_or("someone").to_string();
            let what = if got.is_empty() { "nothing worth keeping".to_string() } else { got.join(", ") };
            self.work_note(now, format!("{name} cuts up the {}: {what}.", c.sp.def().name.to_lowercase()));
        }
    }

    // ---- Hunting ----------------------------------------------------------------

    /// Send some of the squad after a wild herd: they walk toward it and set
    /// about it once in reach.
    pub fn order_hunt(&mut self, who: &[PersonId], herd: u32) -> bool {
        let t = self.time;
        let Some(h) = self.animals.herds.get(herd as usize) else { return false };
        if h.alive(t) == 0 || who.is_empty() || self.squad_battle().is_some() {
            return false;
        }
        if self.hunt(who, herd) {
            return true;
        }
        let aim = self.herd_pos(herd, t);
        for &m in who {
            self.stop_work(m);
            self.send(m, aim);
        }
        self.chases.push(Chase { who: who.to_vec(), herd, aim });
        let name = self.animals.herds[herd as usize].def().name;
        self.work_note(t, format!("You go after the {name}s."));
        true
    }

    fn do_chases(&mut self, now: f64) {
        let mut k = 0;
        while k < self.chases.len() {
            let c = self.chases[k].clone();
            let who: Vec<PersonId> = c.who.iter().copied().filter(|&m| self.squad.index(m).is_some()).collect();
            let alive = self.animals.herds.get(c.herd as usize).is_some_and(|h| h.alive(now) > 0);
            if who.is_empty() || !alive || self.squad_battle().is_some() {
                self.chases.remove(k);
                continue;
            }
            if self.hunt(&who, c.herd) {
                self.chases.remove(k);
                continue;
            }
            let at = self.herd_pos(c.herd, now);
            if at.dist(c.aim) > CHASE_SLACK {
                for &m in &who {
                    self.send(m, at);
                }
                self.chases[k].aim = at;
            }
            k += 1;
        }
    }

    /// Hours a full day's refill takes, for the hover (drawing only).
    pub fn deposit_full_in(&self, id: u32) -> Option<f32> {
        let d = self.deposit(id)?;
        let f = d.face();
        let left = d.left_at(self.time);
        (left < f.cap as f32).then(|| (f.cap as f32 - left) / (f.refill * f.cap as f32) * 24.0)
    }
}

/// Every yield an animal can give has an item, and every face's item exists.
pub fn check_tables() -> Result<(), String> {
    for f in FACES {
        if !items::catalogue_has(f.key) {
            return Err(format!("face item {}", f.key));
        }
    }
    for (_, key, _) in YIELD_ITEMS {
        if !items::catalogue_has(key) {
            return Err(format!("yield item {key}"));
        }
    }
    let _ = HOUR;
    Ok(())
}
