//! Smoking (the items design, Part 5 rule 7 and "Smokes"). Haze is a value on
//! the condition timeline beside drunkenness (`Condition::haze`): each smoke
//! a jump, draining over hours at `HAZE_DRAIN`, the moments it falls past a
//! stage solved for (`Event::Sober`), never ticked (rule 8). No hangover.
//!
//! - **Mellow:** easier company; breath comes back a little slower.
//! - **Hazy:** clumsier and worse aim, a little slower on their feet.
//! - **Stupor:** they doze off where they are (a nudge wakes them).
//!
//! Each smoke carries a small **habit** chance: a keyed roll, against a
//! susceptibility rolled per person from the seed like a trait
//! (`susceptibility`). A habit brings **cravings** by the clock since the
//! last smoke: mild after `CRAVE_AFTER` hours (focus and temper worse),
//! stronger after `CRAVE_HARD`; it fades after `HABIT_FADES` days without.
//! The craving's moments are on the condition timeline too.
//!
//! Stand-ins until their places exist (Laz's smoke halls, the Ṭaḍoro water
//! pipe): incense is lit where they stand, and the water-pipe blend is
//! smoked in a pipe.

use super::effects::Lasts;
use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::rng::Rng;
use super::world::{World, DAY, HOUR};

/// Stage lines on the 0..100 scale.
pub const MELLOW: f32 = 15.0;
pub const HAZY: f32 = 40.0;
pub const STUPOR: f32 = 75.0;
/// How fast it drains, points an hour.
pub const HAZE_DRAIN: f32 = 8.0;
/// Mellow and past it, breath comes back at this share.
pub const MELLOW_BREATH: f32 = 0.85;
/// Hazy and past it, pace.
pub const HAZY_PACE: f32 = 0.9;
/// What each stage takes off Agility, and Marksman.
pub const HAZE_CLUMSY: [f32; 4] = [0.0, 0.0, 6.0, 12.0];
pub const HAZE_AIM: [f32; 4] = [0.0, 0.0, 12.0, 20.0];
/// How each stage sits with company (disposition).
pub const HAZE_TALK: [f32; 4] = [0.0, 3.0, 0.0, -5.0];
/// The habit: the chance per smoke for an average person, and the clock.
pub const HABIT_CHANCE: f32 = 0.05;
pub const CRAVE_AFTER: f64 = 24.0;
pub const CRAVE_HARD: f64 = 48.0;
pub const HABIT_FADES: f64 = 5.0;
/// A craving's cost (mild, hard): Intellect and Willpower, and temper.
pub const CRAVE_FOCUS: [f32; 3] = [0.0, 5.0, 10.0];
pub const CRAVE_TALK: [f32; 3] = [0.0, -3.0, -6.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Haze {
    Clear,
    Mellow,
    Hazy,
    Stupor,
}

impl Haze {
    pub fn of(v: f32) -> Haze {
        if v >= STUPOR {
            Haze::Stupor
        } else if v >= HAZY {
            Haze::Hazy
        } else if v >= MELLOW {
            Haze::Mellow
        } else {
            Haze::Clear
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Haze::Clear => "clear-headed",
            Haze::Mellow => "mellow",
            Haze::Hazy => "hazy",
            Haze::Stupor => "in a stupor",
        }
    }
}

/// How prone someone is to a habit: rolled from the seed like a trait,
/// about 0.2 to 1.8 (1 for most).
pub fn susceptibility(seed: u64, pid: PersonId) -> f32 {
    let mut r = Rng::from_keys(&[seed, pid as u64, 0x4841_4249]);
    0.2 + 0.8 * (r.f32() + r.f32())
}

/// Is this one smoked in a pipe?
pub fn in_a_pipe(id: ItemId) -> bool {
    super::items::smoked_in_a_pipe(id)
}

impl World {
    /// Why someone can't smoke this now, if they can't (a pipe blend wants
    /// a pipe).
    pub fn why_cant_smoke(&self, pid: PersonId, it: ItemId) -> Option<String> {
        (in_a_pipe(it) && self.count_of(pid, "pipe") == 0).then(|| format!("The {} wants a pipe to smoke it in.", item(it).name.to_lowercase()))
    }

    /// Smoke one of something. False if it isn't a smoke, they have none,
    /// or a pipe blend and no pipe.
    pub fn smoke(&mut self, pid: PersonId, it: ItemId) -> bool {
        let t = self.time;
        let Kind::Smoke(haze) = item(it).kind else { return false };
        if self.why_cant_smoke(pid, it).is_some() {
            return false;
        }
        if !self.people[pid as usize].detail.as_mut().is_some_and(|d| d.gear.take(it)) {
            return false;
        }
        self.settle_condition(pid, t);
        let suscept = susceptibility(self.seed, pid);
        let roll = Rng::from_keys(&[self.seed, pid as u64, t.to_bits(), 0x5348_4142]).f32();
        let Some(c) = self.people[pid as usize].cond.as_mut() else { return false };
        let was = Haze::of(c.haze);
        c.haze = (c.haze + haze).min(100.0);
        c.last_smoke = t;
        c.haze_seen = 1.0;
        let took_up = !c.habit && roll < HABIT_CHANCE * suscept;
        if took_up {
            c.habit = true;
        }
        let now = Haze::of(c.haze);
        // (New rates from here, and the craving's clock started again.)
        self.settle_condition(pid, t);
        self.smoke_effect(pid, it, t);
        let name = self.name_of(pid);
        let how = if in_a_pipe(it) { "smokes a pipe of" } else { "burns some" };
        self.say(t, format!("{name} {how} {}.", item(it).name.to_lowercase()));
        if now > was {
            self.say(t, format!("{name} is {}.", now.name()));
        }
        if took_up {
            self.say(t, format!("{name} has taken a liking to it: a habit."));
        }
        // In a stupor, they doze off where they are.
        if now == Haze::Stupor && was < Haze::Stupor {
            if let Some(k) = self.squad.index(pid) {
                self.squad.resting[k] = true;
                self.squad.goal[k] = self.squad.at[k];
                self.squad.route[k].clear();
            }
            self.say(t, format!("{name} dozes off."));
        }
        self.people[pid as usize].recompute_might();
        true
    }

    /// A smoke's one small effect: one at a time, the stronger winning
    /// against the same effect from elsewhere.
    fn smoke_effect(&mut self, pid: PersonId, it: ItemId, t: f64) {
        let Some(e) = item(it).effects.first().copied() else { return };
        match e.lasts {
            Lasts::Secs(secs) => {
                let last = self.people[pid as usize].cond.as_ref().and_then(|c| c.smoke_buff);
                if let Some(old) = last.filter(|&d| d != e.does) {
                    self.boons.retain(|b| !(b.pid == pid && b.does == old));
                }
                if self.boon_at(pid, e.does, t) >= e.power {
                    return;
                }
                self.add_boon_at(pid, e.does, e.power, t + secs as f64, t);
                if let Some(c) = self.people[pid as usize].cond.as_mut() {
                    c.smoke_buff = Some(e.does);
                }
            }
            _ => {
                self.apply_effect(pid, &e, 1.0);
            }
        }
    }

    /// Haze has fallen past a stage line, or a craving has turned, at `t`
    /// (settled already): a line in the log.
    pub(super) fn haze_eased(&mut self, pid: PersonId, t: f64) {
        if self.squad.index(pid).is_none() {
            return;
        }
        let Some(c) = self.people[pid as usize].cond.as_ref() else { return };
        if c.haze <= 1e-3 && c.haze_seen > 0.0 {
            let name = self.name_of(pid);
            if let Some(c) = self.people[pid as usize].cond.as_mut() {
                c.haze_seen = 0.0;
            }
            self.say(t, format!("{name}'s head has cleared."));
        }
    }

    /// How someone's haze and cravings sit with company.
    pub fn haze_talk(&self, pid: PersonId) -> f32 {
        self.people[pid as usize].cond.as_ref().map(|c| HAZE_TALK[Haze::of(c.haze) as usize] + CRAVE_TALK[c.craving as usize]).unwrap_or(0.0)
    }

    /// How high someone is now, 0..100 (for the meter).
    pub fn haze_of(&self, pid: PersonId) -> f32 {
        self.people[pid as usize].cond.as_ref().map(|c| c.haze_at(self.time)).unwrap_or(0.0)
    }

    /// How badly someone craves a smoke now: 0 none, 1 mild, 2 hard.
    pub fn craving_of(&self, pid: PersonId) -> u8 {
        self.people[pid as usize].cond.as_ref().map(|c| c.craving_at(self.time)).unwrap_or(0)
    }
}

/// The craving's moments for someone with a habit, from their last smoke:
/// mild, hard, and the habit fading.
pub fn craving_times(last: f64) -> [f64; 3] {
    [last + CRAVE_AFTER * HOUR, last + CRAVE_HARD * HOUR, last + HABIT_FADES * DAY]
}
