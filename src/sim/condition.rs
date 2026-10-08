//! How your squad is holding up: hunger (and, later in this file's life,
//! tiredness and stamina), and what it does to them.
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
//! Only your squad has a condition. Everyone else in the world gets by.

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
    pub activity: Activity,
    /// Load (share of capacity) during this piece.
    pub load: f32,
    /// Carrying wounds during this piece.
    pub wounded: bool,
}

impl Condition {
    pub fn new(t: f64) -> Condition {
        Condition { at: t, hunger: 10.0, activity: Activity::Resting, load: 0.0, wounded: false }
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
        self.at = t;
    }

    pub fn stage(&self) -> HungerStage {
        stage_of(self.hunger)
    }

    /// Multiplier on attributes from how they're holding up.
    pub fn attr_factor(&self) -> f32 {
        if self.stage() >= HungerStage::Weak {
            WEAK_FACTOR
        } else {
            1.0
        }
    }

    /// Healing per hour for this piece.
    pub fn heal_rate(&self) -> f32 {
        HEAL_PER_HOUR
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
        p.kit().load(&p.stats)
    }

    /// Settle someone's condition and wounds at `t` and start a new piece.
    fn settle(&mut self, pid: PersonId, t: f64) {
        let load = self.load_of(pid);
        let p = &mut self.people[pid as usize];
        let Some(c) = p.cond.as_mut() else { return };
        c.settle(t);
        let lost = p.wounds.lost_at(t);
        p.wounds.lost = lost;
        p.wounds.at = t;
        c.load = load;
        c.wounded = lost.iter().any(|&l| l > 0.01);
        let c = c.clone();
        let stats = p.stats.clone();
        apply_to_wounds(&c, &mut p.wounds, &stats);
    }

    /// Change what a member is doing (from now). No-op if nothing changed.
    pub(super) fn set_activity(&mut self, pid: PersonId, a: Activity) {
        let load = self.load_of(pid);
        let t = self.time;
        let Some(c) = self.people[pid as usize].cond.as_ref() else { return };
        if c.activity == a && (c.load - load).abs() < 0.05 {
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
                // Crossing into the next stage of hunger.
                for level in [HUNGRY, WEAK, STARVING] {
                    consider(c.hunger_reaches(level), Event::Stage);
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Event {
    Stage,
    Eat,
}

/// Knocked out from wasting (hunger) — used by tests and the window.
pub fn collapsed(hp: &[f32; 6]) -> bool {
    body::knocked_out(hp)
}
