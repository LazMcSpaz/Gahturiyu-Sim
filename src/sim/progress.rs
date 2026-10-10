//! Progress you can see (the playable-MVP list): when a squad member's skill
//! or attribute reaches a new whole level, the log says so and their card
//! shows it for a while. Kenshi's quiet "+1" is half the fun of the grind.
//!
//! Reads the stats; never changes them. Each member's last-announced levels
//! are kept (and saved), so a load doesn't announce everything again.

use serde::{Deserialize, Serialize};

use super::person::PersonId;
use super::stats::{ATTRS, N_SKILLS, SKILLS};
use super::world::World;

/// How long a level-up stays on a member's card, seconds of game time.
pub const SHOW_FOR: f64 = 20.0 * 60.0;

/// A member's levels as last announced, and their latest rise.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Levels {
    pub who: PersonId,
    pub skills: Vec<u8>,
    pub attrs: Vec<u8>,
    /// When, what (a skill or attribute's name) and to what.
    pub last: Option<(f64, String, u8)>,
}

impl World {
    fn levels_now(&self, who: PersonId) -> (Vec<u8>, Vec<u8>) {
        let s = &self.people[who as usize].stats;
        let skills = SKILLS.iter().map(|&k| s.skill(k).floor().clamp(0.0, 255.0) as u8).collect();
        let attrs = ATTRS.iter().map(|&a| s.attr(a).floor().clamp(0.0, 255.0) as u8).collect();
        (skills, attrs)
    }

    /// Announce any new whole levels in the travelling squad (once a step).
    pub(super) fn note_progress(&mut self) {
        let t = self.time;
        for who in self.squad.members.clone() {
            let (skills, attrs) = self.levels_now(who);
            let Some(k) = self.levels.iter().position(|l| l.who == who) else {
                // First seen (a new recruit, or an old save): nothing to say.
                self.levels.push(Levels { who, skills, attrs, last: None });
                continue;
            };
            let old = &self.levels[k];
            let mut ups: Vec<(String, u8)> = Vec::new();
            for i in 0..N_SKILLS.min(skills.len()).min(old.skills.len()) {
                if skills[i] > old.skills[i] {
                    ups.push((SKILLS[i].name().to_string(), skills[i]));
                }
            }
            for i in 0..ATTRS.len().min(attrs.len()).min(old.attrs.len()) {
                if attrs[i] > old.attrs[i] {
                    ups.push((ATTRS[i].name().to_string(), attrs[i]));
                }
            }
            let l = &mut self.levels[k];
            l.skills = skills;
            l.attrs = attrs;
            if ups.is_empty() {
                continue;
            }
            l.last = ups.last().map(|u| (t, u.0.clone(), u.1));
            let name = self.people[who as usize].name().unwrap_or("someone").to_string();
            let list: Vec<String> = ups.iter().map(|(n, v)| format!("{n} {v}")).collect();
            self.log.push_front((t, format!("{name} improves: {} ↑", list.join(", "))));
            self.log.truncate(14);
        }
    }

    /// A member's latest level-up, while it's fresh (for their card).
    pub fn fresh_level_up(&self, who: PersonId) -> Option<(&str, u8)> {
        let l = self.levels.iter().find(|l| l.who == who)?;
        let (at, what, v) = l.last.as_ref()?;
        (self.time - at < SHOW_FOR).then_some((what.as_str(), *v))
    }
}
