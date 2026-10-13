//! The tutorial: lessons as data (`docs/tutorial-island.md`). The game's
//! opening is Salt Isle (`island.rs`): the player wakes wrecked on its shore
//! in a storm, finds the shelter hut, cooks, meets Diʻub, and leaves by the
//! caught path or the sneak path. Each lesson is self-contained: which path
//! it belongs to, what it asks, what it teaches, and whether it's switched
//! on (a lesson for a system that isn't built yet stays off, and the
//! tutorial runs with what works).
//!
//! A lesson is **due** when it's the first on the squad's path that isn't
//! done; it's **done** when its condition holds (a pure reading of the
//! world, checked in the squad's own step: the player exception) or when
//! the system it teaches says so (`World::lesson_done`, which the talk,
//! the law and the ship call). What's done is saved (`World::tutorial`).
//! The HUD shows the lesson in the tracked corner; Diʻub's own words for
//! each come from `data/lines/diub.txt` (the second session's).

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::island::Spot;
use super::save::Name;
use super::world::{World, HOUR};

/// Which way through the island a lesson belongs to.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Both,
    Caught,
    Sneak,
}

/// One lesson.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lesson {
    pub key: &'static str,
    pub path: Path,
    /// The tracked corner's title.
    pub title: &'static str,
    /// What to do, in plain words.
    pub task: &'static str,
    pub teaches: &'static str,
    /// Where it points (the tracked corner's distance line).
    pub at: Option<Spot>,
    /// Built: a lesson for a system that isn't in yet stays off.
    pub on: bool,
}

const fn l(key: &'static str, path: Path, title: &'static str, task: &'static str, teaches: &'static str, at: Option<Spot>, on: bool) -> Lesson {
    Lesson { key, path, title, task, teaches, at, on }
}

/// The lessons, in order: the shared opening first, then each path's.
pub const LESSONS: &[Lesson] = &[
    l("wake", Path::Both, "Wake", "You're alive, on a shore, in a storm. Left-click the ground to walk; drag to turn the camera, scroll to zoom.", "moving, the camera, weather", None, true),
    l("shelter", Path::Both, "Shelter", "Get out of the weather. There's a hut up the slope with a fire going.", "shelter, cold", Some(Spot::Hut), true),
    l("cook", Path::Both, "Cook", "There's raw food by the fire. Right-click the fire, Cook here, and eat what you make.", "fire, cooking, eating", Some(Spot::Hut), true),
    l("questions", Path::Both, "The stranger", "Someone's here. Talk to him.", "talking, who you were", Some(Spot::Hut), false),
    l("labour", Path::Caught, "Labour", "Work your shift.", "tools, gathering, carrying", Some(Spot::Quarry), false),
    l("workyard", Path::Caught, "The workyard", "Make what the overseers want.", "crafting, repair, bandages", Some(Spot::Workyard), false),
    l("yard", Path::Caught, "The yard", "Prove yourself.", "fighting, wounds, knockouts", Some(Spot::Yard), false),
    l("bond", Path::Caught, "Your bond", "Learn what you owe.", "bonds, the law", Some(Spot::Hall), false),
    l("bought", Path::Caught, "Bought out", "Someone has paid for you.", "your first faction", Some(Spot::Dock), false),
    l("night_run", Path::Sneak, "Night run", "Get into the compound after dark, unseen.", "sneaking, light, guard routes", Some(Spot::Gate), false),
    l("stores", Path::Sneak, "The stores", "Take what you need.", "looting, inventory, lockpicking", Some(Spot::Stores), false),
    l("gate", Path::Sneak, "The gate", "Pass as one of theirs.", "disguise", Some(Spot::Gate), false),
    l("spotted", Path::Sneak, "Spotted", "Fight or run.", "fighting, fleeing", None, false),
    l("stow", Path::Sneak, "Stow away", "Get aboard.", "escaping; your first faction", Some(Spot::Dock), false),
];

pub fn lesson(key: &str) -> Option<&'static Lesson> {
    LESSONS.iter().find(|x| x.key == key)
}

/// How far from the start counts as having walked.
pub const WALKED: f32 = 20.0;
/// Or this long looking about.
pub const WAKE_AFTER: f64 = 2.0 * 60.0;
/// Within this of the hut's fire is sheltered.
pub const AT_HUT: f32 = 12.0;

/// Where the squad is in the tutorial. Saved.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Tutorial {
    /// Which path they're on, once it's decided (caught or sneaking).
    pub path: Option<Path>,
    /// Lessons done, with when.
    #[serde(with = "done_list")]
    pub done: Vec<(Name, f64)>,
    pub started: f64,
    /// Where they woke.
    pub woke_at: V2,
    /// The island is over: off by ship, or skipped.
    pub over: bool,
}

/// The done list saved by key (a key no longer in the table is dropped).
mod done_list {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[(super::Name, f64)], s: S) -> Result<S::Ok, S::Error> {
        let out: Vec<(String, f64)> = v.iter().map(|(k, t)| (k.to_string(), *t)).collect();
        out.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<(super::Name, f64)>, D::Error> {
        let raw: Vec<(String, f64)> = Vec::deserialize(d)?;
        Ok(raw.into_iter().filter_map(|(k, t)| super::lesson(&k).map(|x| (x.key, t))).collect())
    }
}

impl Tutorial {
    pub fn new(t: f64, at: V2) -> Tutorial {
        Tutorial { path: None, done: Vec::new(), started: t, woke_at: at, over: false }
    }

    pub fn is_done(&self, key: &str) -> bool {
        self.done.iter().any(|(k, _)| *k == key)
    }

    fn fits(&self, x: &Lesson) -> bool {
        x.on && (x.path == Path::Both || self.path == Some(x.path))
    }

    /// The lesson up now: the first on the path that isn't done.
    pub fn now(&self) -> Option<&'static Lesson> {
        if self.over {
            return None;
        }
        LESSONS.iter().find(|x| self.fits(x) && !self.is_done(x.key))
    }
}

impl World {
    pub fn tutorial(&self) -> Option<&Tutorial> {
        self.tutorial.as_ref()
    }

    /// The lesson up now, if the squad is in the tutorial.
    pub fn lesson_now(&self) -> Option<&'static Lesson> {
        self.tutorial.as_ref().and_then(|t| t.now())
    }

    /// Where a lesson points.
    pub fn lesson_at(&self, x: &Lesson) -> Option<V2> {
        Some(self.island()?.spot(x.at?))
    }

    /// A system says a lesson is done (the talk: the questions answered;
    /// the law: made a slave, bought out; the ship: aboard). Also sets the
    /// path when the lesson is one path's.
    pub fn lesson_done(&mut self, key: &str) {
        let t = self.time;
        let Some(tu) = self.tutorial.as_mut() else { return };
        let Some(x) = lesson(key) else { return };
        if tu.is_done(key) {
            return;
        }
        if x.path != Path::Both && tu.path.is_none() {
            tu.path = Some(x.path);
        }
        tu.done.push((x.key, t));
        self.log.push_front((t, format!("Lesson done: {}.", x.title)));
    }

    /// The squad takes the caught path or the sneak path (the law, the
    /// stealth rules). Caught on the sneak path drops onto the caught one.
    pub fn set_path(&mut self, path: Path) {
        if let Some(tu) = self.tutorial.as_mut() {
            tu.path = Some(path);
        }
    }

    /// The island is over: off by ship, or skipped.
    pub fn end_tutorial(&mut self) {
        if let Some(tu) = self.tutorial.as_mut() {
            tu.over = true;
        }
    }

    /// Does the lesson up now hold (the ones read off the world)?
    fn lesson_holds(&self, x: &Lesson) -> bool {
        let Some(tu) = self.tutorial.as_ref() else { return false };
        let positions = || self.squad.members.iter().map(|&m| self.person_pos(m));
        match x.key {
            "wake" => positions().any(|p| p.dist(tu.woke_at) > WALKED) || self.time - tu.started > WAKE_AFTER,
            "shelter" => {
                let Some(hut) = self.island().map(|i| i.spot(Spot::Hut)) else { return false };
                positions().any(|p| p.dist(hut) <= AT_HUT) || self.squad.members.iter().any(|&m| self.building_at(self.person_pos(m)).is_some())
            }
            // A meal's buff on them: they've cooked and eaten.
            "cook" => self.squad.members.iter().any(|&m| self.people[m as usize].cond.as_ref().is_some_and(|c| c.meal.is_some())),
            _ => false,
        }
    }

    /// The squad's step: the lesson up now is checked (cheaply, every so
    /// often), and the next comes up.
    pub(super) fn tutorial_step(&mut self) {
        let Some(x) = self.lesson_now() else { return };
        if self.lesson_holds(x) {
            self.lesson_done(x.key);
        }
    }

    /// Whether the squad is on Salt Isle still (the tutorial's rules apply).
    pub fn on_island(&self) -> bool {
        self.island().is_some_and(|i| i.contains(self.squad.pos)) && self.tutorial.as_ref().is_some_and(|t| !t.over)
    }
}

/// A storm to wake in: the first hour from `from` when the island's shore
/// has weather nobody would work out in, by daylight; or `from` itself if
/// the next days are all fair. A pure reading of the weather (rule 21).
pub fn storm_start(terrain: &super::terrain::Terrain, seed: u64, shore: V2, from: f64) -> f64 {
    for h in 0..(10 * 24) {
        let t = from + h as f64 * HOUR;
        let fx = super::weather::weather_effects(terrain, seed, shore, t);
        if !fx.outdoor_work_ok && super::stealth::daylight(t) > 0.25 {
            return t;
        }
    }
    from
}
