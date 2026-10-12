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
//! **Thirst** (the items design, 0.6: light) runs from 0 to 100 beside
//! hunger: faster marching and fighting, slower asleep, and rain on someone
//! out in it brings it down. Members drink when it reaches `DRINK_AT`: from a
//! town's wells or a base's well if they're there (and they fill their
//! waterskins), else from a waterskin (`SKIN_DRINKS` drinks each; an open
//! skin catches rain). Thirsty (50) they get their breath back slower;
//! parched (75) their attributes are down; at 100 they collapse, knocked
//! out, and it never kills. The rain is read once an hour, at the top of
//! the hour, for anyone out in it.
//!
//! **Toxicity** (Part 4, the alchemist's meter) runs from 0 to 100: every
//! potion drunk adds some (`toxic.rs`), nausea adds more, and it drains away
//! at `TOX_DRAIN` an hour. Queasy (30) breath comes back slower; sick (60)
//! attributes are down, and each draught may come straight back up; at 90
//! they collapse, knocked out until it's down to `TOX_WAKE`. It never
//! kills. Only a dose raises it, so only the falls are on the timeline.
//!
//! Only your squad has a condition. Everyone else in the world gets by, and
//! heals at the plain constant rate.
//!
//! Bedding down: a squad member left standing idle at night (`BEDTIME` to
//! `RISE`) and tired past `BED_TIRED` goes to sleep by themselves, at the
//! moment all three are true (solved, not checked each step). Getting them
//! up with N at night keeps them up until morning.

use serde::{Deserialize, Serialize};

use super::body::{self, Part, Wounds, HEAL_PER_HOUR};
use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::stats::Stats;
use super::world::{World, DAY, HOUR};

/// Hunger gained per game hour, resting.
pub const HUNGER_PER_HOUR: f32 = 1.6;
/// Multipliers on hunger: walking (or fighting), sleeping, wounded.
pub const HUNGER_WALKING: f32 = 1.6;
pub const HUNGER_SLEEPING: f32 = 0.7;
pub const HUNGER_WOUNDED: f32 = 1.25;
/// Extra hunger per unit of load above half capacity.
pub const HUNGER_LOAD: f32 = 0.6;
/// A squadmate this close shares their food with one who has none, metres.
pub const SHARE_REACH: f32 = 40.0;
/// Members eat when hunger reaches this, if they have food.
pub const EAT_AT: f32 = 35.0;
/// What spoiled food does (0.6): the stomach turns, a little toxicity.
pub const SPOILED_NAUSEA: super::effects::Effect = super::effects::lasting(super::effects::Does::Nausea, 0.5, 40.0, super::effects::Reach::Caster);
/// Stage thresholds.
pub const HUNGRY: f32 = 40.0;
pub const WEAK: f32 = 65.0;
pub const STARVING: f32 = 85.0;
/// Attribute multiplier when weak from hunger.
pub const WEAK_FACTOR: f32 = 0.85;
/// Torso health lost per hour while starving.
pub const STARVE_DRAIN: f32 = 2.0;

/// Thirst gained per hour, resting (0.6: water is easy to find, but it
/// matters on long marches and when fighting).
pub const THIRST_PER_HOUR: f32 = 2.2;
/// Multipliers on thirst: marching, fighting, asleep.
pub const THIRST_WALKING: f32 = 1.8;
pub const THIRST_FIGHTING: f32 = 3.0;
pub const THIRST_SLEEPING: f32 = 0.5;
/// Thirst taken off per hour by rain on someone out in it (a downpour, 1).
pub const RAIN_QUENCH: f32 = 6.0;
/// They drink when thirst reaches this.
pub const DRINK_AT: f32 = 40.0;
/// Stages: thirsty, parched, collapse.
pub const THIRSTY: f32 = 50.0;
pub const PARCHED: f32 = 75.0;
pub const COLLAPSE: f32 = 99.9;
/// Breath comes back this much slower thirsty; attributes this much lower
/// parched.
pub const THIRSTY_BREATH: f32 = 0.7;
pub const PARCHED_FACTOR: f32 = 0.85;
/// Drinks a waterskin holds, thirst one takes off, and how fast an open
/// skin fills in rain (drinks an hour, a downpour).
pub const SKIN_DRINKS: f32 = 4.0;
pub const SKIN_QUENCH: f32 = 45.0;
pub const SKIN_CATCH: f32 = 2.0;
/// Torso health lost per hour collapsed from thirst: down to knocked out
/// within the hour or two, never further. Rain or a drink brings them round.
pub const THIRST_DRAIN: f32 = 60.0;

/// Toxicity drained per hour.
pub const TOX_DRAIN: f32 = 5.0;
/// Stages: queasy, sick, collapse; and where someone collapsed comes round.
pub const QUEASY: f32 = 30.0;
pub const SICK: f32 = 60.0;
pub const TOX_COLLAPSE: f32 = 90.0;
pub const TOX_WAKE: f32 = 70.0;
/// Breath comes back this much slower queasy; attributes this much lower sick.
pub const QUEASY_BREATH: f32 = 0.75;
pub const SICK_FACTOR: f32 = 0.85;
/// Torso health lost per hour collapsed from toxicity (down to knocked out,
/// never further).
pub const TOX_KO_DRAIN: f32 = 240.0;

/// Stamina used per hour walking on the flat, unburdened.
pub const STAMINA_WALK: f32 = 12.0;
/// Stamina used per metre climbed (times the load surcharge).
pub const STAMINA_CLIMB: f32 = 0.25;
/// Stamina regained per hour standing still or asleep: about an hour to
/// get it all back (Laz, B5).
pub const STAMINA_REST: f32 = 100.0;
/// Extra stamina use per unit of load above half capacity.
pub const STAMINA_LOAD: f32 = 1.5;
/// Below this share of stamina they're winded: slower on their feet, down
/// to `WINDED_PACE` when it's all gone, and weaker at what they do.
pub const WINDED: f32 = 0.3;
pub const WINDED_PACE: f32 = 0.5;
/// Attribute multiplier while winded.
pub const WINDED_FACTOR: f32 = 0.85;

/// Tiredness gained per hour awake (walking and fighting tire more).
pub const TIRED_PER_HOUR: f32 = 5.0;
/// Standing about tires slower: an idle day in town is no march.
pub const STAND_TIRED: f32 = 3.5;
/// Tiredness slept off per hour: in the open, in a tent, indoors.
pub const SLEEP_OPEN: f32 = 9.0;
pub const SLEEP_TENT: f32 = 15.0;
pub const SLEEP_INDOORS: f32 = 22.0;
/// Very tired from here: slower and weaker.
pub const EXHAUSTED: f32 = 75.0;
/// Pace and attribute multipliers when very tired.
pub const EXHAUSTED_PACE: f32 = 0.8;
pub const EXHAUSTED_FACTOR: f32 = 0.85;

/// Healing speed by what they're doing and how fed: every healing rate is
/// in `body.rs` (P25), named here as before.
pub use super::body::{HEAL_RESTING, HEAL_SLEEP_BED, HEAL_SLEEP_INDOORS, HEAL_SLEEP_OPEN, HEAL_SLEEP_TENT, HEAL_WALKING};
/// Lain down with nothing to sleep off, a member is up again after this long.
pub const NAP_HOURS: f64 = 0.33;

/// Where someone is sleeping.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shelter {
    Open,
    Tent,
    Indoors,
    /// A bed paid for at an inn (`care.rs`).
    Bed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    Resting,
    Walking,
    Sleeping,
    Fighting,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    /// Holding a ritual ready (it drains stamina).
    #[serde(default)]
    pub holding: bool,
    /// Sustained by a spell: no hunger, and no tiredness building up.
    #[serde(default)]
    pub sustained: bool,
    /// Thirst, 0 (slaked) .. 100 (collapsed).
    #[serde(default)]
    pub thirst: f32,
    /// Drinks of water carried in their waterskins.
    #[serde(default)]
    pub water: f32,
    /// The rain on them this piece (0 under a roof), read at its start.
    #[serde(default)]
    pub rain: f32,
    /// Toxicity, 0..100 (`toxic.rs`).
    #[serde(default)]
    pub tox: f32,
    /// Collapsed from toxicity, until it's down to `TOX_WAKE`.
    #[serde(default)]
    pub tox_out: bool,
}

impl Condition {
    pub fn new(t: f64) -> Condition {
        Condition { at: t, hunger: 10.0, stamina: 100.0, max_stamina: 100.0, tired: 10.0, activity: Activity::Resting, shelter: Shelter::Open, load: 0.0, wounded: false, holding: false, sustained: false, thirst: 10.0, water: 0.0, rain: 0.0, tox: 0.0, tox_out: false }
    }

    fn load_surcharge(&self, k: f32) -> f32 {
        1.0 + k * (self.load - 0.5).clamp(0.0, 1.5)
    }

    /// Stamina change per hour in this piece (negative while walking).
    pub fn stamina_rate(&self) -> f32 {
        let hold = if self.holding { super::casting::STAMINA_HOLD } else { 0.0 };
        // (Thirsty, breath comes back slower: read at the last settle, the
        // stage line being a step on the timeline.)
        let dry = if self.thirst >= THIRSTY { THIRSTY_BREATH } else { 1.0 } * if self.tox >= QUEASY { QUEASY_BREATH } else { 1.0 };
        match self.activity {
            Activity::Walking => -STAMINA_WALK * self.load_surcharge(STAMINA_LOAD) - hold,
            Activity::Resting | Activity::Sleeping => STAMINA_REST * dry - hold,
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
        if self.sustained && self.activity != Activity::Sleeping {
            return 0.0;
        }
        match self.activity {
            Activity::Sleeping => -match self.shelter {
                Shelter::Open => SLEEP_OPEN,
                Shelter::Tent => SLEEP_TENT,
                Shelter::Indoors | Shelter::Bed => SLEEP_INDOORS,
            },
            Activity::Resting => STAND_TIRED,
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
        if self.sustained {
            return 0.0;
        }
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

    /// Thirst gained per hour in this piece (less, or falling, in rain).
    pub fn thirst_rate(&self) -> f32 {
        if self.sustained {
            return -self.rain * RAIN_QUENCH;
        }
        let r = THIRST_PER_HOUR
            * match self.activity {
                Activity::Walking => THIRST_WALKING,
                Activity::Fighting => THIRST_FIGHTING,
                Activity::Sleeping => THIRST_SLEEPING,
                Activity::Resting => 1.0,
            }
            * (1.0 + HUNGER_LOAD * (self.load - 0.5).clamp(0.0, 1.5));
        r - self.rain * RAIN_QUENCH
    }

    pub fn thirst_at(&self, t: f64) -> f32 {
        (self.thirst + self.thirst_rate() * hours(self.at, t)).clamp(0.0, 100.0)
    }

    /// When thirst crosses `level` in this piece, either way.
    pub fn thirst_crosses(&self, level: f32) -> Option<f64> {
        let r = self.thirst_rate();
        let gap = level - self.thirst;
        if r == 0.0 || gap.abs() < 1e-4 || gap.signum() != r.signum() {
            return None;
        }
        Some(self.at + (gap / r) as f64 * HOUR)
    }

    /// Toxicity at `t` (it only drains between doses).
    pub fn tox_at(&self, t: f64) -> f32 {
        (self.tox - TOX_DRAIN * hours(self.at, t)).max(0.0)
    }

    /// When toxicity falls to `level` in this piece.
    pub fn tox_falls_to(&self, level: f32) -> Option<f64> {
        (self.tox > level + 1e-4).then(|| self.at + ((self.tox - level) / TOX_DRAIN) as f64 * HOUR)
    }

    /// Water caught in open skins by `t` (they hold `cap` drinks).
    pub fn water_at(&self, t: f64, cap: f32) -> f32 {
        (self.water + self.rain * SKIN_CATCH * hours(self.at, t)).min(cap.max(self.water))
    }

    /// Bring the values forward to `t` and start a new piece there.
    pub fn settle(&mut self, t: f64) {
        self.thirst = self.thirst_at(t);
        self.tox = self.tox_at(t);
        if self.tox_out && self.tox <= TOX_WAKE + 1e-3 {
            self.tox_out = false;
        }
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
        // (Read at the last settle; crossing `WINDED` is a step on the
        // timeline, so the stored value is on the right side of it.)
        if self.winded() {
            f *= WINDED_FACTOR;
        }
        if self.stage() >= HungerStage::Weak {
            f *= WEAK_FACTOR;
        }
        if self.thirst >= PARCHED {
            f *= PARCHED_FACTOR;
        }
        if self.tox >= SICK {
            f *= SICK_FACTOR;
        }
        if self.exhausted() {
            f *= EXHAUSTED_FACTOR;
        }
        f
    }

    /// Multiplier on walking pace from tiredness and lack of breath.
    pub fn pace_factor(&self, t: f64) -> f32 {
        let mut f = if self.exhausted() { EXHAUSTED_PACE } else { 1.0 };
        // Slower as breath runs out, to half pace with none left.
        let share = self.stamina_at(t) / self.max_stamina.max(1.0);
        if share < WINDED {
            f *= WINDED_PACE + (1.0 - WINDED_PACE) * share / WINDED;
        }
        f
    }

    /// Out of breath (as of the last settle).
    pub fn winded(&self) -> bool {
        self.stamina < self.max_stamina * WINDED - 1e-3
    }

    /// When stamina next crosses the winded line, either way.
    pub fn winded_turns(&self) -> Option<f64> {
        let line = self.max_stamina * WINDED;
        let r = self.stamina_rate();
        let gap = line - self.stamina;
        if r == 0.0 || gap.abs() < 1e-3 || gap.signum() != r.signum() {
            return None;
        }
        Some(self.at + (gap / r) as f64 * HOUR)
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
                Shelter::Bed => HEAL_SLEEP_BED,
            },
            Activity::Resting => HEAL_RESTING,
            Activity::Walking => HEAL_WALKING,
            Activity::Fighting => 0.0,
        };
        HEAL_PER_HOUR
            * rest
            * match self.stage() {
                HungerStage::Fed => body::HEAL_FED,
                HungerStage::Hungry => body::HEAL_HUNGRY,
                HungerStage::Weak => body::HEAL_WEAK,
                HungerStage::Starving => body::HEAL_STARVING,
            }
    }
}

fn hours(from: f64, to: f64) -> f32 {
    ((to - from).max(0.0) / HOUR) as f32
}

/// Apply a condition's healing (and wasting) to someone's wounds from now on.
fn apply_to_wounds(c: &Condition, w: &mut Wounds, stats: &Stats, can_die: bool) {
    w.rate = c.heal_rate();
    // Collapsed from thirst, or from what they've drunk: the torso wastes to
    // knocked out, never further.
    if (c.thirst >= COLLAPSE || c.tox_out) && c.stage() != HungerStage::Starving {
        w.drain = if c.tox_out { TOX_KO_DRAIN } else { THIRST_DRAIN };
        w.drain_cap = stats.max_hp(Part::Torso) + 1.0;
        w.rally = 0.0;
        return;
    }
    if c.stage() == HungerStage::Starving {
        w.drain = STARVE_DRAIN;
        // Wasting knocks a squad member out, then kills them (Laz, B2),
        // slowly enough to be seen coming; anyone else it stops just past
        // knocked out.
        w.drain_cap = if can_die { 2.0 * stats.max_hp(Part::Torso) } else { stats.max_hp(Part::Torso) + 1.0 };
        w.rally = 0.0;
    } else {
        w.drain = 0.0;
        // Knocked down (not starved), they come round soon.
        w.rally = if w.rate > 0.0 { body::RALLY_PER_HOUR } else { 0.0 };
        w.rally_to = [Part::Head, Part::Torso].map(|p| stats.max_hp(p) * (1.0 - body::RALLY_WAKE));
    }
}

impl World {
    /// The squad member's load right now, as a share of what they can carry.
    pub fn load_of(&self, pid: PersonId) -> f32 {
        self.load_at(pid, self.time)
    }

    /// Their pack and gear alone (not someone they're carrying), as a share
    /// of what they can carry.
    pub fn pack_load_of(&self, pid: PersonId) -> f32 {
        self.kit_weight_at(pid, self.time) / self.capacity_at(pid, self.time).max(1.0)
    }

    /// The same at time `t` (spells on them count while they last).
    pub fn load_at(&self, pid: PersonId, t: f64) -> f32 {
        (self.kit_weight_at(pid, t) + self.burden_weight(pid)) / self.capacity_at(pid, t).max(1.0)
    }

    /// Settle someone's condition and wounds at `t` and start a new piece
    /// (for other systems that change something mid-piece).
    pub(super) fn settle_condition(&mut self, pid: PersonId, t: f64) {
        self.settle(pid, t);
    }

    /// Settle someone's condition and wounds at `t` and start a new piece.
    fn settle(&mut self, pid: PersonId, t: f64) {
        let load = self.load_at(pid, t);
        let sustained = self.boon_at(pid, super::effects::Does::Sustain, t) > 0.0;
        let shelter = self.shelter_of(pid);
        let max_stamina = {
            let p = &self.people[pid as usize];
            super::inventory::effective(&p.stats, &p.kit()).max_fatigue()
        };
        let rain = self.rain_on(pid, t);
        let cap = self.skins_of(pid) * SKIN_DRINKS;
        let p = &mut self.people[pid as usize];
        let Some(c) = p.cond.as_mut() else { return };
        c.water = c.water_at(t, cap).min(cap);
        c.settle(t);
        c.rain = rain;
        c.shelter = shelter;
        // Full stays full when the pool's size changes (a fresh member's
        // placeholder pool, or new gear).
        let was_full = c.stamina >= c.max_stamina - 1e-3 && (max_stamina - c.max_stamina).abs() > 1e-3;
        c.max_stamina = max_stamina;
        c.stamina = if was_full { c.max_stamina } else { c.stamina.min(c.max_stamina) };
        let lost = p.wounds.lost_at(t);
        p.wounds.lost = lost;
        p.wounds.at = t;
        c.load = load;
        c.sustained = sustained;
        c.wounded = lost.iter().any(|&l| l > 0.01);
        let c = c.clone();
        let stats = p.stats.clone();
        apply_to_wounds(&c, &mut p.wounds, &stats, p.in_squad);
    }

    /// After what they're doing changed (just after a settle): their wounds
    /// mend at the new rate from here on.
    fn rate_wounds(&mut self, pid: PersonId) {
        let p = &mut self.people[pid as usize];
        let Some(c) = p.cond.clone() else { return };
        let stats = p.stats.clone();
        apply_to_wounds(&c, &mut p.wounds, &stats, p.in_squad);
    }

    /// Where a member would sleep right now: indoors, in a tent someone in
    /// the squad is carrying nearby, or in the open.
    pub fn shelter_of(&self, pid: PersonId) -> Shelter {
        let Some(k) = self.squad.index(pid) else { return self.base_bed(pid).unwrap_or(Shelter::Open) };
        if self.in_rented_bed(pid, self.squad.at[k]) {
            return Shelter::Bed;
        }
        if self.squad.inside[k].is_some() {
            return Shelter::Indoors;
        }
        if let Some(bed) = self.base_bed(pid) {
            return bed;
        }
        // In a pitched tent, or out in the open (on a bedroll).
        match self.bedding_at(self.squad.at[k]) {
            Some(b) if b.kind == super::bedding::BedKind::Tent => Shelter::Tent,
            _ => Shelter::Open,
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
        self.rate_wounds(pid);
        if a == Activity::Sleeping {
            self.drop_held_on_sleep(pid, t);
            self.torch_out_for_sleep(pid, t);
        }
    }

    /// The best thing in someone's pack to eat now.
    fn food_for(&self, pid: PersonId, hunger: f32) -> Option<ItemId> {
        // What's going off is eaten first (0.6).
        if let Some(it) = self.food_going_off(pid, self.time) {
            return Some(it);
        }
        let foods: Vec<(ItemId, f32)> = self.people[pid as usize].detail.as_ref().map(|d| d.gear.bag.iter().filter_map(|e| if let Kind::Food(n) = item(e.0).kind { Some((e.0, n)) } else { None }).collect()).unwrap_or_default();
        // The biggest meal that won't be wasted, else the smallest one there is.
        let own = foods.iter().filter(|f| f.1 <= hunger).max_by(|a, b| a.1.total_cmp(&b.1)).or_else(|| foods.iter().min_by(|a, b| a.1.total_cmp(&b.1))).map(|f| f.0);
        // Someone living at a base eats what they carry first, then from
        // its store (NM-4).
        if own.is_none() && self.resident_of(pid).is_some() {
            return self.base_food_for(pid, hunger);
        }
        // A town feeds those it holds to work (NM-34).
        own.or_else(|| self.bound_food_for(pid)).or_else(|| self.shared_food_for(pid, hunger).map(|(_, it)| it))
    }

    /// With nothing of their own, a squadmate close by shares theirs (RG-21):
    /// (who, what). The nearest with food, within `SHARE_REACH`.
    fn shared_food_for(&self, pid: PersonId, hunger: f32) -> Option<(PersonId, ItemId)> {
        let k = self.squad.index(pid)?;
        let here = self.member_pos(k);
        let (_, giver) = self
            .squad
            .members
            .iter()
            .enumerate()
            .filter(|&(j, &m)| m != pid && !self.people[m as usize].dead && self.member_pos(j).dist(here) <= SHARE_REACH)
            .filter(|&(_, &m)| self.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| matches!(item(e.0).kind, Kind::Food(_)))))
            .map(|(j, &m)| (self.member_pos(j).dist(here), m))
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))?;
        let foods: Vec<(ItemId, f32)> = self.people[giver as usize].detail.as_ref()?.gear.bag.iter().filter_map(|e| if let Kind::Food(n) = item(e.0).kind { Some((e.0, n)) } else { None }).collect();
        let it = foods.iter().filter(|f| f.1 <= hunger).max_by(|a, b| a.1.total_cmp(&b.1)).or_else(|| foods.iter().min_by(|a, b| a.1.total_cmp(&b.1)))?.0;
        Some((giver, it))
    }

    /// Eat something now (or at `t`): hunger drops by its nourishment.
    pub fn eat(&mut self, pid: PersonId, it: ItemId, t: f64) -> bool {
        let Kind::Food(n) = item(it).kind else { return false };
        // From their own pack (what's going off first: `spoil.rs`); a base's
        // resident with none of it, from the store.
        let own = self.take_food(pid, it, t);
        let had = own.is_some() || (self.resident_of(pid).is_some() && self.base_take_food(pid, it)) || self.bound_take_food(pid, it);
        // Stale fills less; spoiled fills little and turns the stomach.
        let fresh = own.unwrap_or(super::spoil::Fresh::Fresh);
        let n = n * super::spoil::FILLS[fresh as usize];
        // Or a squadmate close by hands some over.
        let mut shared_by = None;
        if !had {
            let hunger = self.people[pid as usize].cond.as_ref().map(|c| c.hunger_at(t)).unwrap_or(0.0);
            if let Some((giver, _)) = self.shared_food_for(pid, hunger).filter(|&(_, x)| x == it) {
                if self.people[giver as usize].detail.as_mut().is_some_and(|d| d.gear.take(it)) {
                    self.people[giver as usize].recompute_might();
                    shared_by = Some(giver);
                }
            }
        }
        if !had && shared_by.is_none() {
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
            apply_to_wounds(&c, &mut p.wounds, &stats, p.in_squad);
        }
        p.recompute_might();
        // Spoiled food turns the stomach: Nausea, with a little toxicity.
        if own.is_some() && fresh == super::spoil::Fresh::Spoiled {
            self.take_dose_at(pid, &SPOILED_NAUSEA, t);
        }
        let p = &self.people[pid as usize];
        let name = p.name().unwrap_or("someone").to_string();
        let line = match (shared_by, fresh) {
            (Some(g), _) => format!("{} shares some {} with {name}.", self.people[g as usize].name().unwrap_or("Someone"), item(it).name.to_lowercase()),
            (None, super::spoil::Fresh::Spoiled) => format!("{name} eats some spoiled {} and feels sick.", item(it).name.to_lowercase()),
            (None, super::spoil::Fresh::Stale) => format!("{name} eats some stale {}.", item(it).name.to_lowercase()),
            (None, super::spoil::Fresh::Fresh) => format!("{name} eats some {}.", item(it).name.to_lowercase()),
        };
        // (Those left at a base eat unremarked.)
        if self.squad.index(pid).is_some() {
            self.log.push_front((t, line));
            self.log.truncate(14);
        }
        true
    }

    /// Run every squad member's condition forward to now, handling each
    /// stage change and meal at the moment it falls due.
    pub(super) fn update_conditions(&mut self) {
        let now = self.time;
        let mut everyone = self.squad.members.clone();
        everyone.extend(self.all_residents());
        for pid in everyone {
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
                // Thirst crossing a stage, either way; and for anyone out of
                // doors, the top of the hour (the rain is read again).
                for level in [THIRSTY, PARCHED, COLLAPSE] {
                    consider(c.thirst_crosses(level), Event::Stage);
                }
                // Toxicity falling past a stage (or far enough to come round).
                for level in [QUEASY, SICK, TOX_WAKE] {
                    consider(c.tox_falls_to(level), Event::Stage);
                }
                if self.shelter_of(pid) == Shelter::Open {
                    consider(Some(((c.at / HOUR).floor() + 1.0) * HOUR), Event::Stage);
                }
                // Time to drink, if there's water.
                if self.water_for(pid, &c) {
                    let due = if c.thirst >= DRINK_AT { Some(c.at) } else { c.thirst_crosses(DRINK_AT).filter(|_| c.thirst_rate() > 0.0) };
                    consider(due, Event::Drink);
                }
                consider(c.winded_turns(), Event::Stage);
                consider(c.rested_by(EXHAUSTED), Event::Stage);
                // Fully rested: wake up (unless they're out cold then).
                if c.activity == Activity::Sleeping {
                    // Lain down with nothing to sleep off: up after a short nap,
                    // or once their wounds have mended if they're hurt.
                    let tw = c.rested_by(0.5).or_else(|| {
                        (c.tired <= 0.5).then(|| match p.wounds.healed_in() {
                            Some(h) if c.wounded => p.wounds.at + h as f64 * HOUR + 1.0,
                            _ => c.at + NAP_HOURS * HOUR,
                        })
                    });
                    if let Some(tw) = tw {
                        if !body::knocked_out(&p.wounds.hp_at(&p.stats, tw)) {
                            consider(Some(tw), Event::Wake);
                        }
                    }
                }
                // A ritual finishing.
                if let Some(tr) = self.ritual_due(pid) {
                    consider(Some(tr), Event::Ritual);
                }
                // A spell on them running out (it may have changed their load
                // or needs).
                for te in self.boon_ends(pid) {
                    if te > c.at {
                        consider(Some(te), Event::Stage);
                    }
                }
                // Standing idle at night and tired: bed down.
                if let Some(tb) = self.bed_time(pid, &c) {
                    consider(Some(tb), Event::Bed);
                }
                // Time to eat, if there's food.
                let hungry_now = c.hunger_at(now) >= EAT_AT;
                if hungry_now && self.food_for(pid, c.hunger_at(now)).is_some() {
                    consider(Some(c.hunger_reaches(EAT_AT).unwrap_or(c.at)), Event::Eat);
                }
                // Bled to death, when an open bleed runs its course.
                if p.in_squad && p.wounds.bleed > 0.0 {
                    if let Some(h) = p.wounds.torso_reaches(2.0 * p.stats.max_hp(Part::Torso)) {
                        consider(Some(p.wounds.at + h as f64 * HOUR), Event::BledOut);
                    }
                }
                // Starved to death, when the wasting runs its course (B2).
                if p.in_squad && p.wounds.drain > 0.0 {
                    let cap = 2.0 * p.stats.max_hp(Part::Torso);
                    if p.wounds.drain_cap >= cap - 1e-3 {
                        let left = (cap - p.wounds.lost[Part::Torso as usize]).max(0.0);
                        consider(Some(p.wounds.at + (left / p.wounds.drain) as f64 * HOUR), Event::Starved);
                    }
                }
                // Wounds all healed: hunger stops counting them.
                if c.wounded {
                    if let Some(hrs) = p.wounds.healed_in() {
                        consider(Some(p.wounds.at + hrs as f64 * HOUR), Event::Stage);
                    }
                }
                let Some((t, e)) = next else { break };
                match e {
                    Event::Stage => {
                        let was = c.stage();
                        let was_dry = c.thirst_at(t);
                        self.settle(pid, t);
                        // A line when someone turns thirsty, or parched.
                        if self.squad.index(pid).is_some() {
                            let line = match was_dry {
                                d if d >= PARCHED - 1e-3 && c.thirst < PARCHED - 1e-3 => Some("is parched"),
                                d if d >= THIRSTY - 1e-3 && c.thirst < THIRSTY - 1e-3 => Some("is thirsty"),
                                _ => None,
                            };
                            if let Some(how) = line {
                                let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
                                self.log.push_front((t, format!("{name} {how}.")));
                                self.log.truncate(14);
                            }
                        }
                        // One line when someone starts starving, before
                        // anyone goes down.
                        let now_starving = self.people[pid as usize].cond.as_ref().is_some_and(|c| c.stage() == HungerStage::Starving);
                        if was != HungerStage::Starving && now_starving && self.squad.index(pid).is_some() {
                            let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
                            self.log.push_front((t, format!("{name} is starving.")));
                            self.log.truncate(14);
                        }
                    }
                    Event::Starved => {
                        self.settle(pid, t);
                        let p = &self.people[pid as usize];
                        if body::dead(&p.wounds.hp_at(&p.stats, t), &p.stats) {
                            self.died_out_of_a_fight(pid, t, "has starved to death");
                            break;
                        }
                    }
                    Event::BledOut => {
                        self.settle(pid, t);
                        let p = &self.people[pid as usize];
                        if body::dead(&p.wounds.hp_at(&p.stats, t), &p.stats) {
                            self.died_out_of_a_fight(pid, t, "has bled to death");
                            break;
                        }
                    }
                    Event::Wake => {
                        self.settle(pid, t);
                        if let Some(c) = self.people[pid as usize].cond.as_mut() {
                            c.activity = Activity::Resting;
                        }
                        self.rate_wounds(pid);
                        if let Some(k) = self.squad.index(pid) {
                            self.squad.resting[k] = false;
                        }
                        // (Those left at a base wake and sleep unremarked.)
                        if self.squad.index(pid).is_some() {
                            let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
                            self.log.push_front((t, format!("{name} wakes, rested.")));
                            self.log.truncate(14);
                        }
                    }
                    Event::Bed => {
                        self.settle(pid, t);
                        if let Some(k) = self.squad.index(pid) {
                            self.squad.resting[k] = true;
                        }
                        if let Some(c) = self.people[pid as usize].cond.as_mut() {
                            c.activity = Activity::Sleeping;
                        }
                        self.rate_wounds(pid);
                        if self.squad.index(pid).is_some() {
                            let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
                            self.log.push_front((t, format!("{name} beds down for the night.")));
                            self.log.truncate(14);
                        }
                        self.drop_held_on_sleep(pid, t);
                        self.torch_out_for_sleep(pid, t);
                    }
                    Event::Ritual => self.ritual_done(pid),
                    Event::Drink => {
                        if !self.drink_water(pid, t) {
                            break;
                        }
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

    /// When a squad member standing idle would bed down by themselves: the
    /// first moment that is night, after they're tired enough and after any
    /// order keeping them up. None if they're busy.
    fn bed_time(&self, pid: PersonId, c: &Condition) -> Option<f64> {
        // Residents of a base bed down when they're tired and it's night.
        if self.squad.index(pid).is_none() && self.resident_of(pid).is_some() {
            if c.activity != Activity::Resting {
                return None;
            }
            let tired = if c.tired_at(c.at) >= BED_TIRED { c.at } else { c.tired_reaches(BED_TIRED)?.max(c.at) };
            return Some(night_from(tired));
        }
        let k = self.squad.index(pid)?;
        if c.activity != Activity::Resting || self.squad.resting[k] || self.fighting.contains_key(&pid) {
            return None;
        }
        let busy = self.carrying(pid).is_some()
            || self.carried_by(pid).is_some()
            || self.want_carry.iter().any(|w| w.0 == pid)
            || self.crafting.iter().any(|j| j.who == pid)
            || self.rituals.iter().any(|j| j.who == pid)
            || self.picking.iter().any(|p| p.who == pid)
            || self.gathering.iter().any(|g| g.0 == pid)
            || self.pickups.iter().any(|p| p.who == pid)
            || self.want_talk.map(|w| w.0 == pid).unwrap_or(false)
            || self.talk.as_ref().map(|t| t.with == pid).unwrap_or(false);
        // Nobody beds down by themselves while creeping about or with a camp
        // watching them.
        if busy || self.is_sneaking(pid) || self.suspicion.iter().any(|(&(_, m), &v)| m == pid && v > 0.0) {
            return None;
        }
        // Only where there's something to sleep in (Laz, 11 Oct: nobody
        // beds down on the bare ground; they lie down in a bed, a bedroll
        // or a tent).
        if !self.sleep_place_at(self.squad.at[k]) && self.base_bed(pid).is_none() {
            return None;
        }
        let start = c.at.max(self.squad.kept_up[k]);
        let tired = if c.tired_at(start) >= BED_TIRED { start } else { c.tired_reaches(BED_TIRED)?.max(start) };
        Some(night_from(tired))
    }

    /// For the window: hunger 0..100 now.
    pub fn hunger_of(&self, pid: PersonId) -> Option<f32> {
        self.people[pid as usize].cond.as_ref().map(|c| c.hunger_at(self.time))
    }

    /// Thirst 0..100 now, and the drinks of water they carry.
    pub fn thirst_of(&self, pid: PersonId) -> Option<(f32, f32)> {
        let cap = self.skins_of(pid) * SKIN_DRINKS;
        self.people[pid as usize].cond.as_ref().map(|c| (c.thirst_at(self.time), c.water_at(self.time, cap)))
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

    /// Send a member to a bed, bedroll or tent to sleep there: they walk
    /// over and bed down when they arrive. Nowhere to sleep at `at`: nothing.
    pub fn order_sleep_at(&mut self, who: PersonId, at: super::geo::V2) {
        if !self.sleep_place_at(at) {
            let name = self.name_of(who);
            self.say(self.time, format!("{name} has nothing to sleep on there."));
            return;
        }
        self.order_members(&[who], at);
        if let Some(k) = self.squad.index(who) {
            if self.free_to_order(who) {
                self.squad.resting[k] = true;
            }
        }
    }

    /// Order some members to bed down: each lies down where they stand if
    /// there's a bed, bedroll or tent there, else lays out their own bedroll
    /// at their feet and lies down in it, else goes to a free one close by.
    /// With none of these, they say so. (Getting up is `order_wake`; any order to move
    /// gets them up too.)
    pub fn order_rest(&mut self, who: &[PersonId]) {
        for &m in who {
            let Some(k) = self.squad.index(m) else { continue };
            if self.fighting.contains_key(&m) {
                continue;
            }
            let here = self.squad.at[k];
            let free = self
                .beds
                .iter()
                .filter(|b| b.ready(self.time) && b.pos.dist(here) <= 3.0)
                .find(|b| {
                    let room = if b.kind == super::bedding::BedKind::Tent { super::bedding::IN_TENT } else { 1 };
                    let reach = if b.kind == super::bedding::BedKind::Tent { super::bedding::TENT_REACH } else { super::bedding::BED_REACH };
                    self.squad.members.iter().enumerate().filter(|(j, &o)| o != m && self.squad.resting[*j] && self.squad.at[*j].dist(b.pos) <= reach).count() < room
                })
                .map(|b| b.id);
            // Lying where they stand if they can, else on their own bedroll
            // at their feet, else on a free one close by.
            if self.sleep_place_at(here) {
                self.squad.resting[k] = true;
                self.squad.goal[k] = here;
                self.squad.route[k].clear();
            } else if self.order_lay(m, super::bedding::BedKind::Bedroll, here, true).is_ok() {
            } else if let Some(b) = free {
                let _ = self.order_sleep_in(m, b);
            } else {
                let name = self.name_of(m);
                self.say(self.time, format!("{name} has no bedroll and nothing to sleep on here."));
            }
        }
    }

    /// Get some members up from resting. Got up at night, they stay up
    /// until morning.
    pub fn order_wake(&mut self, who: &[PersonId]) {
        for &m in who {
            let Some(k) = self.squad.index(m) else { continue };
            if self.squad.resting[k] {
                self.squad.resting[k] = false;
                if is_night(self.time) {
                    self.squad.kept_up[k] = next_rise(self.time);
                }
            }
        }
    }

    /// Is anyone of these resting (or asleep on a rest order)?
    pub fn any_resting(&self, who: &[PersonId]) -> bool {
        who.iter().any(|&m| self.squad.index(m).is_some_and(|k| self.squad.resting[k]))
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
    /// Felt casting adds to tiredness too.
    /// A squad member starved or bled to death: they fall where they stand,
    /// their things with them, and they're gone from the squad. `how` ends
    /// the log line ("has starved to death").
    fn died_out_of_a_fight(&mut self, pid: PersonId, t: f64, how: &str) {
        let Some(k) = self.squad.index(pid) else { return };
        let at = self.member_pos(k);
        let race = self.people[pid as usize].race;
        self.quit_work(pid);
        self.people[pid as usize].dead = true;
        self.busy_until[pid as usize] = f64::INFINITY;
        self.corpses.push((at, race, t, pid));
        if self.people[pid as usize].ensure_detail() {
            self.stats.detailed += 1;
        }
        self.drop_everything(pid, at);
        self.carried.retain(|&p, &mut c| p != pid && c != pid);
        self.looting.retain(|l| l.who != pid);
        self.picking.retain(|p| p.who != pid);
        self.pickups.retain(|p| p.who != pid);
        self.giving.retain(|g| g.from != pid && g.to != pid);
        self.boons.retain(|b| b.pid != pid);
        self.torches.remove(&pid);
        if self.held.contains_key(&pid) {
            self.set_holding(pid, t, None);
        }
        self.squad.retain(|m| m != pid);
        self.recentre_squad();
        self.people[pid as usize].recompute_might();
        let name = self.people[pid as usize].name().unwrap_or("someone").to_string();
        self.log.push_front((t, format!("{name} {how}.")));
        self.log.truncate(14);
    }

    pub(super) fn after_fight(&mut self, pid: PersonId, t: f64, stamina: f32, tire: f32, tox: f32) {
        if self.people[pid as usize].cond.is_none() {
            return;
        }
        self.settle(pid, t);
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.stamina = stamina.clamp(0.0, c.max_stamina);
            c.tired = (c.tired + tire).min(100.0);
            // What they drank in the fight (it didn't drain while they fought).
            if tox > c.tox + 1e-3 {
                c.tox = tox.min(100.0);
                c.tox_out |= c.tox >= TOX_COLLAPSE;
            }
        }
        if self.people[pid as usize].cond.as_ref().is_some_and(|c| c.tox_out) {
            self.settle(pid, t);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Event {
    Stage,
    Drink,
    Starved,
    BledOut,
    Eat,
    Wake,
    Bed,
    Ritual,
}

/// Squad members left standing idle bed down by themselves at night: from
/// `BEDTIME` until `RISE` (hours of the day), once at least this tired.
pub const BEDTIME: f64 = 22.0;
pub const RISE: f64 = 6.0;
pub const BED_TIRED: f32 = 35.0;

pub(super) fn is_night(t: f64) -> bool {
    let h = t.rem_euclid(DAY) / HOUR;
    h >= BEDTIME || h < RISE
}

/// The first moment at or after `t` that falls in the night.
fn night_from(t: f64) -> f64 {
    if is_night(t) {
        return t;
    }
    let day0 = (t / DAY).floor() * DAY;
    day0 + BEDTIME * HOUR
}

/// The next morning's `RISE` after `t`.
pub fn next_rise(t: f64) -> f64 {
    let day0 = (t / DAY).floor() * DAY;
    let r = day0 + RISE * HOUR;
    if r > t {
        r
    } else {
        r + DAY
    }
}

/// Knocked out from wasting (hunger) — used by tests and the window.
pub fn collapsed(hp: &[f32; 6]) -> bool {
    body::knocked_out(hp)
}
