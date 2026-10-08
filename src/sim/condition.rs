//! How your squad is holding up: hunger, stamina and tiredness, and what
//! they do to them.
//!
//! **Worked out from the clock, in pieces.** A member's condition is stored
//! as values at one moment plus what they're doing (resting, walking...).
//! While nothing changes, everything moves at a steady rate, so the value at
//! any later time is a straight-line sum. Whenever something does change —
//! they start walking, eat, cross into a new stage of hunger — the values are
//! "settled" at that exact moment and a new piece begins. Stage changes and
//! meals happen at the moment the numbers say, not when the world happens to
//! be stepped, so a fine and a coarse step give the same result.
//!
//! **Hunger** runs from 0 (full) to 100 (starving). It rises faster walking
//! than resting, faster with a heavy load, faster when wounded. Members eat
//! from their own pack when it reaches `EAT_AT`. Stages:
//!
//! | Hunger | Stage | Effect |
//! |---|---|---|
//! | under 40 | fed | none |
//! | 40–65 | hungry | wounds heal at half speed |
//! | 65–85 | weak | also attributes down 15% |
//! | 85+ | starving | no healing at all; the torso slowly wastes, until they collapse (knocked out, never killed) |
//!
//! **Stamina** (the same pool fights draw on) drains slowly while walking —
//! faster with a heavy load, and by the metre on every climb — and comes back
//! quickly standing still. A fight starts with whatever stamina they have.
//!
//! **Tiredness** runs from 0 (fresh) to 100 and builds over hours awake. Only
//! sleep brings it down: slowly in the open, faster in a tent, fastest
//! indoors. Members sleep when ordered to rest and stopped; moving wakes them.
//! Very tired members (75+) move slower and fight worse.
//!
//! **Healing** follows all this: asleep and fed heals fastest (a tent or a
//! roof helps), standing about is ordinary, marching heals little, and hunger
//! cuts it further — a starving marcher heals nothing at all.
//!
//! Only your squad has a condition. Everyone else in the world gets by, and
//! heals at the plain constant rate.

use super::body::{self, Part, Wounds, HEAL_PER_HOUR};
use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::stats::Stats;
use super::world::{World, HOUR};

/// Hunger gained per game hour, resting.
pub const HUNGER_PER_HOUR: f32 = 1.6;
/// Multipliers on hunger: walking (or fighting), sleeping, wounded.
pub const HUNGER_WALKING: f32 = 1.6;
pub const HUNGER_SLEEPING: f32 = 0.7;
pub const HUNGER_WOUNDED: f32 = 1.25;
/// Extra hunger per unit of load above half capacity.
pub const HUNGER_LOAD: f32 = 0.6;
/// Members eat when hunger reaches this, if they have food.
pub const EAT_AT: f32 = 35.0;
/// Stage thresholds.
pub const HUNGRY: f32 = 40.0;
pub const WEAK: f32 = 65.0;
pub const STARVING: f32 = 85.0;
/// Attribute multiplier when weak from hunger.
pub const WEAK_FACTOR: f32 = 0.85;
/// Torso health lost per hour while starving.
pub const STARVE_DRAIN: f32 = 3.0;

/// Stamina used per hour walking on the flat, unburdened.
pub const STAMINA_WALK: f32 = 12.0;
/// Stamina used per metre climbed (times the load surcharge).
pub const STAMINA_CLIMB: f32 = 0.25;
/// Stamina regained per hour standing still or asleep.
pub const STAMINA_REST: f32 = 300.0;
/// Extra stamina use per unit of load above half capacity.
pub const STAMINA_LOAD: f32 = 1.5;
/// Below this share of stamina, walkers slow down.
pub const WINDED: f32 = 0.1;

/// Tiredness gained per hour awake (walking and fighting tire more).
pub const TIRED_PER_HOUR: f32 = 5.0;
/// Tiredness slept off per hour: in the open, in a tent, indoors.
pub const SLEEP_OPEN: f32 = 9.0;
pub const SLEEP_TENT: f32 = 15.0;
pub const SLEEP_INDOORS: f32 = 22.0;
/// Very tired from here: slower and weaker.
pub const EXHAUSTED: f32 = 75.0;
/// Pace and attribute multipliers when very tired.
pub const EXHAUSTED_PACE: f32 = 0.8;
pub const EXHAUSTED_FACTOR: f32 = 0.85;

/// Healing speed (times `HEAL_PER_HOUR`) by what they're doing. Fed members
/// asleep indoors heal fastest; walking heals little.
pub const HEAL_SLEEP_OPEN: f32 = 1.5;
pub const HEAL_SLEEP_TENT: f32 = 1.75;
pub const HEAL_SLEEP_INDOORS: f32 = 2.0;
pub const HEAL_RESTING: f32 = 1.0;
pub const HEAL_WALKING: f32 = 0.3;

/// Where someone is sleeping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shelter {
    Open,
    Tent,
    Indoors,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Resting,
    Walking,
    Sleeping,
    Fighting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerStage {
    Fed,
    Hungry,
    Weak,
    Starving,
}

pub fn stage_of(hunger: f32) -> HungerStage {
    if hunger >= STARVING {
        HungerStage::Starving
    } else if hunger >= WEAK {
        HungerStage::Weak
    } else if hunger >= HUNGRY {
        HungerStage::Hungry
    } else {
        HungerStage::Fed
    }
}

/// A squad member's condition, as of `at`.
#[derive(Clone, Debug, PartialEq)]
pub struct Condition {
    pub at: f64,
    pub hunger: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub tired: f32,
    pub activity: Activity,
    /// Where they sleep, if asleep.
    pub shelter: Shelter,
    /// Load (share of capacity) during this piece.
    pub load: f32,
    /// Carrying wounds during this piece.
    pub wounded: bool,
}

impl Condition {
    pub fn new(t: f64) -> Condition {
        Condition { at: t, hunger: 10.0, stamina: 100.0, max_stamina: 100.0, tired: 10.0, activity: Activity::Resting, shelter: Shelter::Open, load: 0.0, wounded: false }
    }

    fn load_surcharge(&self, k: f32) -> f32 {
        1.0 + k * (self.load - 0.5).clamp(0.0, 1.5)
    }

    /// Stamina change per hour in this piece (negative while walking).
    pub fn stamina_rate(&self) -> f32 {
        match self.activity {
            Activity::Walking => -STAMINA_WALK * self.load_surcharge(STAMINA_LOAD),
            Activity::Resting | Activity::Sleeping => STAMINA_REST,
            Activity::Fighting => 0.0, // the fight keeps its own count
        }
    }

    pub fn stamina_at(&self, t: f64) -> f32 {
        (self.stamina + self.stamina_rate() * hours(self.at, t)).clamp(0.0, self.max_stamina)
    }

    /// Stamina for a climb of `metres` (by the metre, not the clock).
    pub fn climb_cost(&self, metres: f32) -> f32 {
        metres.max(0.0) * STAMINA_CLIMB * self.load_surcharge(STAMINA_LOAD)
    }

    /// Tiredness change per hour in this piece.
    pub fn tired_rate(&self) -> f32 {
        match self.activity {
            Activity::Sleeping => -match self.shelter {
                Shelter::Open => SLEEP_OPEN,
                Shelter::Tent => SLEEP_TENT,
                Shelter::Indoors => SLEEP_INDOORS,
            },
            Activity::Resting => TIRED_PER_HOUR,
            Activity::Walking => TIRED_PER_HOUR * 1.3,
            Activity::Fighting => TIRED_PER_HOUR * 1.5,
        }
    }

    pub fn tired_at(&self, t: f64) -> f32 {
        (self.tired + self.tired_rate() * hours(self.at, t)).clamp(0.0, 100.0)
    }

    /// When tiredness next reaches `level` going up (for the stage change).
    pub fn tired_reaches(&self, level: f32) -> Option<f64> {
        let r = self.tired_rate();
        if self.tired >= level || r <= 0.0 {
            return None;
        }
        Some(self.at + ((level - self.tired) / r) as f64 * HOUR)
    }

    /// When sleep brings tiredness down past `level`.
    pub fn rested_by(&self, level: f32) -> Option<f64> {
        let r = self.tired_rate();
        if self.tired <= level || r >= 0.0 {
            return None;
        }
        Some(self.at + ((self.tired - level) / -r) as f64 * HOUR)
    }

    pub fn exhausted(&self) -> bool {
        self.tired >= EXHAUSTED
    }

    /// Hunger gained per hour in this piece.
    pub fn hunger_rate(&self) -> f32 {
        let mut r = HUNGER_PER_HOUR;
        r *= match self.activity {
            Activity::Walking | Activity::Fighting => HUNGER_WALKING,
            Activity::Sleeping => HUNGER_SLEEPING,
            Activity::Resting => 1.0,
        };
        r *= 1.0 + HUNGER_LOAD * (self.load - 0.5).clamp(0.0, 1.5);
        if self.wounded {
            r *= HUNGER_WOUNDED;
        }
        r
    }

    pub fn hunger_at(&self, t: f64) -> f32 {
        (self.hunger + self.hunger_rate() * hours(self.at, t)).clamp(0.0, 100.0)
    }

    /// When hunger next reaches `level` in this piece (if it's still below).
    pub fn hunger_reaches(&self, level: f32) -> Option<f64> {
        if self.hunger >= level {
            return None;
        }
        let r = self.hunger_rate();
        (r > 0.0).then(|| self.at + ((level - self.hunger) / r) as f64 * HOUR)
    }

    /// Bring the values forward to `t` and start a new piece there.
    pub fn settle(&mut self, t: f64) {
        self.hunger = self.hunger_at(t);
        self.stamina = self.stamina_at(t);
        self.tired = self.tired_at(t);
        self.at = t;
    }

    pub fn stage(&self) -> HungerStage {
        stage_of(self.hunger)
    }

    /// Multiplier on attributes from how they're holding up.
    pub fn attr_factor(&self) -> f32 {
        let mut f = 1.0;
        if self.stage() >= HungerStage::Weak {
            f *= WEAK_FACTOR;
        }
        if self.exhausted() {
            f *= EXHAUSTED_FACTOR;
        }
        f
    }

    /// Multiplier on walking pace from tiredness and lack of breath.
    pub fn pace_factor(&self, t: f64) -> f32 {
        let mut f = if self.exhausted() { EXHAUSTED_PACE } else { 1.0 };
        if self.stamina_at(t) < self.max_stamina * WINDED {
            f *= 0.7;
        }
        f
    }

    /// Healing per hour for this piece: how well they're resting, times how
    /// well they're fed. (People outside the squad always heal at the plain
    /// `HEAL_PER_HOUR` — see `body::Wounds`.)
    pub fn heal_rate(&self) -> f32 {
        let rest = match self.activity {
            Activity::Sleeping => match self.shelter {
                Shelter::Open => HEAL_SLEEP_OPEN,
                Shelter::Tent => HEAL_SLEEP_TENT,
                Shelter::Indoors => HEAL_SLEEP_INDOORS,
            },
            Activity::Resting => HEAL_RESTING,
            Activity::Walking => HEAL_WALKING,
            Activity::Fighting => 0.0,
        };
        HEAL_PER_HOUR
            * rest
            * match self.stage() {
                HungerStage::Fed => 1.0,
                HungerStage::Hungry => 0.5,
                HungerStage::Weak => 0.25,
                HungerStage::Starving => 0.0,
            }
    }
}

fn hours(from: f64, to: f64) -> f32 {
    ((to - from).max(0.0) / HOUR) as f32
}

/// Apply a condition's healing (and wasting) to someone's wounds from now on.
fn apply_to_wounds(c: &Condition, w: &mut Wounds, stats: &Stats) {
    w.rate = c.heal_rate();
    if c.stage() == HungerStage::Starving {
        w.drain = STARVE_DRAIN;
        // Wasting knocks you out but never kills: it stops just past zero.
        w.drain_cap = stats.max_hp(Part::Torso) + 1.0;
    } else {
        w.drain = 0.0;
    }
}

impl World {
    /// The squad member's load right now, as a share of what they can carry.
    pub fn load_of(&self, pid: PersonId) -> f32 {
        let p = &self.people[pid as usize];
        let gear = p.kit();
        (gear.weight() + self.burden_weight(pid)) / gear.capacity(&p.stats).max(1.0)
    }

    /// Settle someone's condition and wounds at `t` and start a new piece.
    fn settle(&mut self, pid: PersonId, t: f64) {
        let load = self.load_of(pid);
        let shelter = self.shelter_of(pid);
        let max_stamina = {
            let p = &self.people[pid as usize];
            super::inventory::effective(&p.stats, &p.kit()).max_fatigue()
        };
        let p = &mut self.people[pid as usize];
        let Some(c) = p.cond.as_mut() else { return };
        c.settle(t);
        c.shelter = shelter;
        c.max_stamina = max_stamina;
        c.stamina = c.stamina.min(c.max_stamina);
        let lost = p.wounds.lost_at(t);
        p.wounds.lost = lost;
        p.wounds.at = t;
        c.load = load;
        c.wounded = lost.iter().any(|&l| l > 0.01);
        let c = c.clone();
        let stats = p.stats.clone();
        apply_to_wounds(&c, &mut p.wounds, &stats);
    }

    /// Where a member would sleep right now: indoors, in a tent someone in
    /// the squad is carrying nearby, or in the open.
    pub fn shelter_of(&self, pid: PersonId) -> Shelter {
        let Some(k) = self.squad.index(pid) else { return Shelter::Open };
        if self.squad.inside[k].is_some() {
            return Shelter::Indoors;
        }
        let tent = super::items::id("tent");
        let here = self.squad.at[k];
        let has_tent = (0..self.squad.members.len()).any(|j| {
            self.squad.at[j].dist(here) < 15.0 && self.people[self.squad.members[j] as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == tent)).unwrap_or(false)
        });
        if has_tent {
            Shelter::Tent
        } else {
            Shelter::Open
        }
    }

    /// Change what a member is doing (from now). No-op if nothing changed.
    pub(super) fn set_activity(&mut self, pid: PersonId, a: Activity) {
        let load = self.load_of(pid);
        let t = self.time;
        let shelter = if a == Activity::Sleeping { self.shelter_of(pid) } else { Shelter::Open };
        let Some(c) = self.people[pid as usize].cond.as_ref() else { return };
        if c.activity == a && (c.load - load).abs() < 0.05 && (a != Activity::Sleeping || c.shelter == shelter) {
            return;
        }
        self.settle(pid, t);
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.activity = a;
        }
    }

    /// The best thing in someone's pack to eat now.
    fn food_for(&self, pid: PersonId, hunger: f32) -> Option<ItemId> {
        let d = self.people[pid as usize].detail.as_ref()?;
        let foods: Vec<(ItemId, f32)> = d.gear.bag.iter().filter_map(|&(i, _)| if let Kind::Food(n) = item(i).kind { Some((i, n)) } else { None }).collect();
        // The biggest meal that won't be wasted, else the smallest one there is.
        foods
            .iter()
            .filter(|f| f.1 <= hunger)
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .or_else(|| foods.iter().min_by(|a, b| a.1.total_cmp(&b.1)))
            .map(|f| f.0)
    }

    /// Eat something now (or at `t`): hunger drops by its nourishment.
    pub fn eat(&mut self, pid: PersonId, it: ItemId, t: f64) -> bool {
        let Kind::Food(n) = item(it).kind else { return false };
        if !self.people[pid as usize].detail.as_mut().map(|d| d.gear.take(it)).unwrap_or(false) {
            return false;
        }
        self.settle(pid, t);
        let p = &mut self.people[pid as usize];
        if let Some(c) = p.cond.as_mut() {
            c.hunger = (c.hunger - n).max(0.0);
        }
        let c = p.cond.clone();
        let stats = p.stats.clone();
        if let Some(c) = c {
            apply_to_wounds(&c, &mut p.wounds, &stats);
        }
        p.recompute_might();
        let name = p.name().unwrap_or("someone").to_string();
        self.log.push_front((t, format!("{name} eats some {}.", item(it).name.to_lowercase())));
        self.log.truncate(14);
        true
    }

    /// Run every squad member's condition forward to now, handling each
    /// stage change and meal at the moment it falls due.
    pub(super) fn update_conditions(&mut self) {
        let now = self.time;
        for pid in self.squad.members.clone() {
            // A fight rewrote their wounds: start a fresh piece from then.
            let (Some(c), w) = (self.people[pid as usize].cond.clone(), self.people[pid as usize].wounds) else { continue };
            if w.at > c.at && w.at <= now {
                self.settle(pid, w.at);
            }
            for _ in 0..16 {
                let p = &self.people[pid as usize];
                let Some(c) = p.cond.clone() else { break };
                if p.dead {
                    break;
                }
                let mut next: Option<(f64, Event)> = None;
                let mut consider = |t: Option<f64>, e: Event| {
                    if let Some(t) = t {
                        if t <= now && next.map(|n| t < n.0).unwrap_or(true) {
                            next = Some((t.max(c.at), e));
                        }
                    }
                };
                // Crossing into the next stage of hunger, or of tiredness.
                for level in [HUNGRY, WEAK, STARVING] {
                    consider(c.hunger_reaches(level), Event::Stage);
                }
                consider(c.tired_reaches(EXHAUSTED), Event::Stage);
                consider(c.rested_by(EXHAUSTED), Event::Stage);
                // Fully rested: wake up (unless they're out cold then).
                if c.activity == Activity::Sleeping {
                    if let Some(tw) = c.rested_by(0.5) {
                        if !body::knocked_out(&p.wounds.hp_at(&p.stats, tw)) {
                            consider(Some(tw), Event::Wake);
                        }
                    }
                }
                // Time to eat, if there's food.
                let hungry_now = c.hunger_at(now) >= EAT_AT;
                if hungry_now && self.food_for(pid, c.hunger_at(now)).is_some() {
                    consider(Some(c.hunger_reaches(EAT_AT).unwrap_or(c.at)), Event::Eat);
                }
                // Wounds all healed: hunger stops counting them.
                if c.wounded {
                    let lost = p.wounds.lost;
                    let worst = lost.iter().cloned().fold(0.0f32, f32::max);
                    if p.wounds.rate > 0.0 && p.wounds.drain == 0.0 {
                        consider(Some(p.wounds.at + (worst / p.wounds.rate) as f64 * HOUR), Event::Stage);
                    }
                }
                let Some((t, e)) = next else { break };
                match e {
                    Event::Stage => self.settle(pid, t),
                    Event::Wake => {
                        self.settle(pid, t);
                        if let Some(c) = self.people[pid as usize].cond.as_mut() {
                            c.activity = Activity::Resting;
                        }
                        if let Some(k) = self.squad.index(pid) {
                            self.squad.resting[k] = false;
                        }
                        let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
                        self.log.push_front((t, format!("{name} wakes, rested.")));
                        self.log.truncate(14);
                    }
                    Event::Eat => {
                        let h = c.hunger_at(t);
                        if let Some(f) = self.food_for(pid, h) {
                            self.eat(pid, f, t);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// For the window: hunger 0..100 now.
    pub fn hunger_of(&self, pid: PersonId) -> Option<f32> {
        self.people[pid as usize].cond.as_ref().map(|c| c.hunger_at(self.time))
    }

    /// Tiredness 0..100 now.
    pub fn tired_of(&self, pid: PersonId) -> Option<f32> {
        self.people[pid as usize].cond.as_ref().map(|c| c.tired_at(self.time))
    }

    /// Stamina now, as a share of the most they can have.
    pub fn stamina_of(&self, pid: PersonId) -> Option<f32> {
        if let Some(f) = self.fighter(pid) {
            return Some(f.fatigue / f.max_fatigue.max(1.0));
        }
        self.people[pid as usize].cond.as_ref().map(|c| c.stamina_at(self.time) / c.max_stamina.max(1.0))
    }

    pub fn is_asleep(&self, pid: PersonId) -> bool {
        self.people[pid as usize].cond.as_ref().map(|c| c.activity == Activity::Sleeping).unwrap_or(false)
    }

    /// Order some members to rest: they stop where they are and sleep. A
    /// second order (or any order to move) gets them up.
    pub fn order_rest(&mut self, who: &[PersonId]) {
        let all_resting = who.iter().all(|&m| self.squad.index(m).map(|k| self.squad.resting[k]).unwrap_or(true));
        for &m in who {
            let Some(k) = self.squad.index(m) else { continue };
            if all_resting {
                self.squad.resting[k] = false;
            } else if !self.fighting.contains_key(&m) {
                self.squad.resting[k] = true;
                self.squad.goal[k] = self.squad.at[k];
                self.squad.route[k].clear();
            }
        }
    }

    /// A walker climbing: stamina by the metre.
    pub(super) fn climb(&mut self, pid: PersonId, metres: f32) {
        if metres <= 0.0 {
            return;
        }
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            let cost = c.climb_cost(metres);
            c.stamina = (c.stamina - cost).max(0.0);
        }
    }

    /// After a fight: carry on with the stamina it left them.
    pub(super) fn after_fight(&mut self, pid: PersonId, t: f64, stamina: f32) {
        if self.people[pid as usize].cond.is_none() {
            return;
        }
        self.settle(pid, t);
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.stamina = stamina.clamp(0.0, c.max_stamina);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Event {
    Stage,
    Eat,
    Wake,
}

/// Knocked out from wasting (hunger) — used by tests and the window.
pub fn collapsed(hp: &[f32; 6]) -> bool {
    body::knocked_out(hp)
}
