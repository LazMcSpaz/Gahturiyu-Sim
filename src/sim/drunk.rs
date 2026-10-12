//! Drinking (the items design, Part 5 rule 5 and "Alcohol"). Drunkenness is
//! a value on the condition timeline (`Condition::drunk`): each drink a
//! quick jump, draining over hours at `DRUNK_DRAIN`, the moments it falls
//! past a stage solved for (`Event::Sober`), never ticked (rule 8).
//!
//! - **Tipsy:** a little better company (talk).
//! - **Drunk:** clumsier (Agility down; aim and sneaking worse), and talk goes
//!   worse.
//! - **Very drunk:** worse still, stumbling (slower), and a keyed chance to
//!   pass out on reaching it.
//! - Sobering up from drunk leaves a **hangover** for `HANGOVER_HOURS`:
//!   breath back slower, a little clumsy.
//!
//! The stages act through the condition (`Person::weaken`, `pace_factor`,
//! `stamina_rate`, `World::drunk_talk`), read as of the last settle, so a
//! fight reads them too. A drink's own small effect is one at a time, the
//! newest replacing the last, the stronger winning against the same effect
//! from elsewhere (a potion, a meal).

use super::effects::Lasts;
use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::rng::Rng;
use super::world::{World, HOUR};

/// Stage lines on the 0..100 scale.
pub const TIPSY: f32 = 15.0;
pub const DRUNK: f32 = 40.0;
pub const VERY_DRUNK: f32 = 70.0;
/// How fast it drains, points an hour.
pub const DRUNK_DRAIN: f32 = 6.0;
/// The hangover after being drunk: how long, and breath coming back slower.
pub const HANGOVER_HOURS: f64 = 6.0;
pub const HANGOVER_BREATH: f32 = 0.7;
/// Very drunk, they stumble along at this share of their pace.
pub const STUMBLE_PACE: f32 = 0.8;
/// The chance of passing out on reaching very drunk.
pub const PASS_OUT: f32 = 0.35;
/// What each stage takes off: Agility, and Sneak and Marksman.
pub const DRUNK_CLUMSY: [f32; 4] = [0.0, 0.0, 8.0, 18.0];
pub const DRUNK_SKILLS: [f32; 4] = [0.0, 0.0, 10.0, 25.0];
pub const HUNGOVER_CLUMSY: f32 = 4.0;
/// How each stage sits with whoever they talk to (disposition).
pub const DRUNK_TALK: [f32; 4] = [0.0, 4.0, -4.0, -10.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Drunk {
    Sober,
    Tipsy,
    Drunk,
    VeryDrunk,
}

impl Drunk {
    pub fn of(v: f32) -> Drunk {
        if v >= VERY_DRUNK {
            Drunk::VeryDrunk
        } else if v >= DRUNK {
            Drunk::Drunk
        } else if v >= TIPSY {
            Drunk::Tipsy
        } else {
            Drunk::Sober
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Drunk::Sober => "sober",
            Drunk::Tipsy => "tipsy",
            Drunk::Drunk => "drunk",
            Drunk::VeryDrunk => "very drunk",
        }
    }
}

impl World {
    /// Drink a bottle of something. False if it isn't a drink, or they have
    /// none.
    pub fn drink(&mut self, pid: PersonId, it: ItemId) -> bool {
        let t = self.time;
        let Kind::Drink(strength) = item(it).kind else { return false };
        let Some(fresh) = self.take_food(pid, it, t) else { return false };
        self.settle_condition(pid, t);
        let (quench, tox) = super::items::drink_sides(it);
        let Some(c) = self.people[pid as usize].cond.as_mut() else { return false };
        let was = Drunk::of(c.drunk);
        c.drunk = (c.drunk + strength).min(100.0);
        let now = Drunk::of(c.drunk);
        if now >= Drunk::Drunk {
            c.was_drunk = true;
        }
        c.thirst = (c.thirst - quench).clamp(0.0, 100.0);
        // (New rates from here: the next stage line, the thirst.)
        self.settle_condition(pid, t);
        if tox > 0.0 {
            self.add_toxicity(pid, tox, t);
        }
        let name = self.name_of(pid);
        let what = item(it).name.to_lowercase();
        if fresh == super::spoil::Fresh::Spoiled {
            // Gone off (ale and cider spoil): sick, and nothing more.
            self.take_dose_at(pid, &super::condition::SPOILED_NAUSEA, t);
            self.say(t, format!("{name} drinks some soured {what} and feels sick."));
        } else {
            self.drink_effect(pid, it, t);
            self.say(t, format!("{name} drinks some {what}."));
        }
        if now > was {
            self.say(t, format!("{name} is {}.", now.name()));
        }
        // Reaching very drunk, they may pass out where they stand.
        if now == Drunk::VeryDrunk && was < Drunk::VeryDrunk {
            let roll = Rng::from_keys(&[self.seed, pid as u64, t.to_bits(), 0x5041_5353]).f32();
            if roll < PASS_OUT {
                if let Some(k) = self.squad.index(pid) {
                    self.squad.resting[k] = true;
                    self.squad.goal[k] = self.squad.at[k];
                    self.squad.route[k].clear();
                }
                self.say(t, format!("{name} passes out."));
            }
        }
        self.people[pid as usize].recompute_might();
        true
    }

    /// A drink's one small effect: one at a time (the newest replacing the
    /// last), and against the same effect from elsewhere the stronger wins.
    fn drink_effect(&mut self, pid: PersonId, it: ItemId, t: f64) {
        let Some(e) = item(it).effects.first().copied() else { return };
        let grade = super::items::info(it).grade;
        let power = e.power * grade.power();
        match e.lasts {
            Lasts::Secs(secs) => {
                let last = self.people[pid as usize].cond.as_ref().and_then(|c| c.drink_buff);
                if let Some(old) = last.filter(|&d| d != e.does) {
                    self.boons.retain(|b| !(b.pid == pid && b.does == old));
                }
                if self.boon_at(pid, e.does, t) >= power {
                    return;
                }
                self.add_boon_at(pid, e.does, power, t + secs as f64, t);
                if let Some(c) = self.people[pid as usize].cond.as_mut() {
                    c.drink_buff = Some(e.does);
                }
            }
            _ => {
                let e = super::effects::Effect { power, ..e };
                self.apply_effect(pid, &e, 1.0);
            }
        }
    }

    /// Drunkenness has fallen past a stage line at `t` (settled already): a
    /// line in the log, and sobered up from drunk, the hangover.
    pub(super) fn sobered(&mut self, pid: PersonId, t: f64) {
        let Some(c) = self.people[pid as usize].cond.as_mut() else { return };
        let hangover = c.drunk <= 1e-3 && c.was_drunk;
        if hangover {
            c.was_drunk = false;
            c.hangover_until = t + HANGOVER_HOURS * HOUR;
        }
        let now = Drunk::of(c.drunk);
        if hangover {
            // (Breath slower from now: a new piece.)
            self.settle_condition(pid, t);
        }
        if self.squad.index(pid).is_none() {
            return;
        }
        let name = self.name_of(pid);
        if hangover {
            self.say(t, format!("{name} has sobered up, with a sore head."));
        } else if now == Drunk::Sober {
            self.say(t, format!("{name} has sobered up."));
        }
    }

    /// How someone's drinking sits with whoever they're talking to.
    pub fn drunk_talk(&self, pid: PersonId) -> f32 {
        self.people[pid as usize].cond.as_ref().map(|c| DRUNK_TALK[Drunk::of(c.drunk) as usize]).unwrap_or(0.0)
    }

    /// How drunk someone is now, 0..100 (for the meter).
    pub fn drunk_of(&self, pid: PersonId) -> f32 {
        self.people[pid as usize].cond.as_ref().map(|c| c.drunk_at(self.time)).unwrap_or(0.0)
    }

    /// Hungover now?
    pub fn hungover(&self, pid: PersonId) -> bool {
        self.people[pid as usize].cond.as_ref().is_some_and(|c| self.time < c.hangover_until)
    }
}
