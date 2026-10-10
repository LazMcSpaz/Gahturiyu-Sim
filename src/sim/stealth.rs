//! Being seen and heard.
//!
//! Anyone watching (for now: bandit camps) keeps a suspicion meter for each
//! of your squad. It fills while they can see or hear that person and drains
//! slowly when they can't; full means noticed. Bandits attack whoever they
//! notice.
//!
//! - **Seeing** reaches further in good light and against people who stand
//!   out. Light comes from the sun (by hour), from campfires, from lit
//!   windows in town at night, and from torches (see `torch`). Sneaking makes
//!   you a smaller, slower shape; Sneak skill and Agility make that better.
//!   A lit torch undoes most of that, and at night it can be seen from
//!   `torch::TORCH_SEEN` away.
//! - **Hearing** depends on how much noise you make: moving is louder than
//!   standing, a fight is loudest, and armour clanks by its weight. Sneaking
//!   softens your step, and Sneak skill softens it more. Town bustle covers
//!   noise.
//! - **Watchers** are sharper with better Intellect and Willpower.
//!
//! Sneaking slows you to under half pace. Moving unseen near watchers trains
//! Sneak, and hitting someone who hasn't noticed you is a sneak attack (see
//! `combat::SNEAK_ATTACK`).
//!
//! The meters run in fixed quarter-second ticks of game time, so how often
//! the world is stepped doesn't change when someone is noticed.

use super::geo::V2;
use super::group::GroupId;
use super::items::{item, Kind, Slot, SLOTS};
use super::person::PersonId;
use super::stats::{Attr, Skill};
use super::world::{World, DAY, HOUR};

/// Seconds of game time per detection tick.
pub const TICK: f64 = 0.25;
/// Furthest anyone can see a person standing in full daylight, metres.
pub const SIGHT: f32 = 75.0;
/// How far a normal walking pace can be heard, metres.
pub const HEARING: f32 = 30.0;
/// Bump into someone and they notice, whatever.
pub const TOUCH: f32 = 3.0;
/// Sneaking within this many metres of watchers, unnoticed, trains Sneak.
pub const CREEP: f32 = 45.0;
/// Pace while sneaking, as a share of normal.
pub const SNEAK_PACE: f32 = 0.45;

/// Sunlight at time `t`: full by day, dim at dawn and dusk, moonlight at night.
pub fn daylight(t: f64) -> f32 {
    let h = (t.rem_euclid(DAY) / HOUR) as f32;
    let night = 0.12;
    if (7.0..19.0).contains(&h) {
        1.0
    } else if (5.0..7.0).contains(&h) {
        night + (1.0 - night) * (h - 5.0) / 2.0
    } else if (19.0..21.0).contains(&h) {
        1.0 - (1.0 - night) * (h - 19.0) / 2.0
    } else {
        night
    }
}

/// A word for a light level.
pub fn light_word(l: f32) -> &'static str {
    match l {
        l if l >= 0.85 => "bright",
        l if l >= 0.5 => "dim",
        l if l >= 0.25 => "gloomy",
        _ => "dark",
    }
}

impl World {
    /// How well lit a spot is, 0..1.
    pub fn light_at(&self, p: V2) -> f32 {
        if daylight(self.time) >= 0.99 {
            return 1.0;
        }
        // Campfires, lit windows (town is never quite dark) and torches.
        super::torch::light_from(self.time, &self.lights(), p)
    }

    /// Is someone on the squad sneaking?
    pub fn is_sneaking(&self, pid: PersonId) -> bool {
        self.squad.index(pid).map(|k| self.squad.sneaking[k]).unwrap_or(false)
    }

    pub fn set_sneaking(&mut self, pid: PersonId, on: bool) {
        if let Some(k) = self.squad.index(pid) {
            self.squad.sneaking[k] = on;
        }
    }

    /// How much noise a squad member is making, 0 (silent) and up; 1 is an
    /// ordinary walk in light clothes.
    pub fn noise_of(&self, pid: PersonId) -> f32 {
        let p = &self.people[pid as usize];
        let Some(k) = self.squad.index(pid) else { return 1.0 };
        let moving = self.squad.at[k].dist(self.squad.goal[k]) > 0.3;
        let mut n = if self.fighting.contains_key(&pid) {
            2.5
        } else if moving {
            1.0
        } else {
            0.25
        };
        // Armour clanks by its weight.
        let gear = p.kit();
        let worn: f32 = SLOTS.iter().filter_map(|&s| gear.in_slot(s)).filter(|&i| matches!(item(i).kind, Kind::Armor(_)) || item(i).slot == Slot::OffHand).map(|i| item(i).weight).sum();
        n += worn * 0.035;
        if self.squad.sneaking[k] {
            let skill = p.effective_stats().skill(Skill::Sneak);
            n *= 0.4 * (1.0 - skill / 160.0);
        }
        // Silent step.
        n *= (1.0 - self.boon(pid, super::effects::Does::Silent)).max(0.0);
        n
    }

    /// How easy a squad member is to see, 0..1, before distance.
    pub fn visibility_of(&self, pid: PersonId) -> f32 {
        let Some(k) = self.squad.index(pid) else { return 1.0 };
        let at = self.member_pos(k);
        let mut v = self.light_at(at);
        if self.squad.sneaking[k] {
            let s = self.people[pid as usize].effective_stats();
            let hidden = (0.55 - s.skill(Skill::Sneak) / 250.0 - s.attr(Attr::Agility) / 700.0).max(0.08);
            // There's no creeping about with a lit torch.
            v *= if self.torch_lit(pid) { hidden.max(super::torch::TORCH_SNEAK) } else { hidden };
        }
        let moving = self.squad.at[k].dist(self.squad.goal[k]) > 0.3;
        if !moving {
            v *= 0.75;
        }
        // Hidden by a spell.
        v *= (1.0 - self.boon(pid, super::effects::Does::Hide)).max(0.0);
        v
    }

    /// How sharp a group's lookouts are: the best of them, about 0.7..1.3.
    fn alertness(&self, gid: GroupId) -> f32 {
        let Some(g) = self.group(gid) else { return 1.0 };
        g.members
            .iter()
            .filter(|&&m| !self.people[m as usize].dead)
            .map(|&m| {
                let s = &self.people[m as usize].stats;
                0.7 + (s.attr(Attr::Intellect) + s.attr(Attr::Willpower)) / 2.0 / 160.0
            })
            .fold(0.0, f32::max)
    }

    /// How fast a watcher's suspicion of someone rises per second, right now.
    /// Also returns whether they're within the watcher's senses at all.
    pub fn detect_rate(&self, watcher: GroupId, from: V2, pid: PersonId) -> (f32, bool) {
        let Some(k) = self.squad.index(pid) else { return (0.0, false) };
        let at = self.member_pos(k);
        let d = at.dist(from);
        if d < TOUCH {
            return (100.0, true);
        }
        // Inside a veil, lookouts see and hear nothing.
        if self.wards_at(super::effects::Does::Veil, at).next().is_some() {
            return (0.0, d < CREEP);
        }
        let sharp = self.alertness(watcher);
        let mut sight = SIGHT * self.visibility_of(pid) * sharp;
        // A torch in the dark is a beacon.
        if self.torch_lit(pid) {
            sight = sight.max(super::torch::TORCH_SEEN * (1.0 - daylight(self.time)) * sharp);
        }
        let in_town = self.settlements.iter().any(|s| s.pos.dist(at) < s.reach);
        let hearing = HEARING * self.noise_of(pid) * sharp * if in_town { 0.6 } else { 1.0 };
        let mut rate = 0.0;
        if d < sight {
            rate += 0.3 + 1.5 * (1.0 - d / sight);
        }
        if d < hearing {
            rate += 0.2 + 1.0 * (1.0 - d / hearing);
        }
        (rate, d < CREEP)
    }

    /// Run the watchers' meters up to now. Returns the groups that noticed
    /// someone, with the moment they did.
    pub(super) fn update_watchers(&mut self) -> Vec<(GroupId, f64)> {
        let first = (self.watch_done / TICK).floor() as i64 + 1;
        let last = (self.time / TICK).floor() as i64;
        self.watch_done = self.time;
        if last < first {
            return Vec::new();
        }
        // Who's watching: camps close enough to matter and not busy fighting.
        // A camp licking its wounds after a fight doesn't go looking for
        // another, but still sees who walks into its own ground, unless the
        // squad beat it (then the squad holds the field till they've rested).
        let watchers: Vec<(GroupId, V2, Option<V2>)> = self
            .camps
            .iter()
            .filter(|c| c.ready_at <= self.time || !self.beaten_camps.contains(&c.group))
            .filter_map(|c| self.group(c.group).map(|g| (g, c.pos, c.ready_at > self.time)))
            .filter(|(g, _, _)| g.band == 1 && !self.fighting_groups.contains(&g.id) && g.members.iter().any(|m| !self.people[*m as usize].dead && !self.fighting.contains_key(m)))
            .map(|(g, home, resting)| (g.id, g.position_at(self.time), resting.then_some(home)))
            .collect();
        let members: Vec<PersonId> = self.squad.members.iter().copied().filter(|m| !self.people[*m as usize].dead && !self.fighting.contains_key(m)).collect();
        // Rates don't change much within one step; work them out once.
        let ticks = (last - first + 1).min(400) as f32;
        let mut noticed = Vec::new();
        for &(gid, from, resting) in &watchers {
            let mut when = None;
            for &m in &members {
                let (rate, near) = match resting {
                    Some(home) if self.person_pos(m).dist(home) > super::encounters::GUARD_RING => (0.0, false),
                    _ => self.detect_rate(gid, from, m),
                };
                let meter = self.suspicion.entry((gid, m)).or_insert(0.0);
                let before = *meter;
                if rate > 0.0 {
                    *meter += rate * TICK as f32 * ticks;
                } else {
                    *meter = (*meter - 0.15 * TICK as f32 * ticks).max(0.0);
                }
                if before < 1.0 && *meter >= 1.0 {
                    // The tick it filled on.
                    let need = ((1.0 - before) / (rate * TICK as f32)).ceil().max(1.0) as i64;
                    let t = (first + need - 1) as f64 * TICK;
                    when = Some(when.map_or(t, |w: f64| w.min(t)));
                }
                if near && *meter < 1.0 && self.is_sneaking(m) {
                    self.people[m as usize].stats.exercise(Skill::Sneak, 0.05 * TICK as f32 * ticks);
                }
            }
            if let Some(t) = when {
                noticed.push((gid, t));
            }
        }
        self.suspicion.retain(|_, v| *v > 0.0);
        noticed
    }

    /// The highest suspicion anyone has of this squad member, 0..1.
    pub fn suspicion_of(&self, pid: PersonId) -> f32 {
        self.suspicion.iter().filter(|((_, m), _)| *m == pid).map(|(_, v)| *v).fold(0.0, f32::max).min(1.0)
    }

    /// Has this group noticed anyone in the squad?
    pub fn has_noticed(&self, gid: GroupId) -> bool {
        self.suspicion.iter().any(|((g, _), v)| *g == gid && *v >= 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_is_brighter_than_night() {
        let noon = 12.0 * HOUR;
        let dusk = 20.0 * HOUR;
        let midnight = DAY;
        assert!(daylight(noon) > daylight(dusk));
        assert!(daylight(dusk) > daylight(midnight));
    }
}
