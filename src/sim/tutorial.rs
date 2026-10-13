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
    l("salvage", Path::Both, "Salvage", "What the sea left of your things lies at the water's edge below the wreck. Click the crate and take what's there.", "looting, your pack", Some(Spot::Wreck), true),
    l("shelter", Path::Both, "Shelter", "Get out of the weather. There's a hut up the slope with a fire going.", "shelter, cold", Some(Spot::Hut), true),
    l("cook", Path::Both, "Cook", "There's raw food by the fire: click it to pick it up. Then right-click the fire, Cook here, make something, and eat it (double-click it in the pack).", "fire, cooking, eating", Some(Spot::Hut), true),
    l("questions", Path::Both, "The stranger", "Someone's here. Talk to him.", "talking, who you were", Some(Spot::Hut), true),
    l("labour", Path::Caught, "Labour", "You're the lord's now. By day the overseers walk you to the woodlot or the quarry: cut and dig (it goes on his pile), and you're walked back at dusk. You take no orders while bound.", "tools, gathering, carrying", Some(Spot::Woodlot), true),
    l("workyard", Path::Caught, "The workyard", "Every third day you're put to the workyard. Press Craft there: mend something, or make a bandage (two cloth) for the sick house.", "crafting, repair, bandages", Some(Spot::Workyard), true),
    l("yard", Path::Caught, "The yard", "Prove yourself.", "fighting, wounds, knockouts", Some(Spot::Yard), false),
    l("bond", Path::Caught, "Your bond", "Learn what you owe.", "bonds, the law", Some(Spot::Hall), false),
    l("bought", Path::Caught, "Bought out", "Someone has paid for you.", "your first faction", Some(Spot::Dock), false),
    l("night_run", Path::Both, "The compound", "The compound has what you need. Its gate is watched: go in by night, sneaking (the Sneak order), and keep out of the watch's sight. (Walk in by day and you'll be taken for a runaway.)", "sneaking, light, guard routes", Some(Spot::Gate), true),
    l("stores", Path::Sneak, "The stores", "The hall and the steward's house keep chests and crates. Go in, go through one (click it) and take what you need; a lock wants a lockpick.", "looting, inventory, lockpicking", Some(Spot::Hall), true),
    l("gate", Path::Sneak, "The gate", "By day, pass the gate as one of theirs: put on a labourer's smock (from the stores, or the Workbench) and walk through. The watch may look you over.", "disguise", Some(Spot::Gate), true),
    l("spotted", Path::Sneak, "Spotted", "If the watch catches on, fight or run: the open ground north of the compound is yours, the briar theirs to avoid.", "fighting, fleeing", None, true),
    l("stow", Path::Sneak, "Stow away", "The ship lies at the dock in the morning. Sneak aboard: right-click the dock while sneaking, Board the ship.", "escaping; your first faction", Some(Spot::Dock), true),
    l("sail", Path::Both, "The ship", "The ferry lies at the dock in the morning. Right-click the dock to board; it sails when you're all aboard.", "leaving the island", Some(Spot::Dock), true),
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
    /// What they've done on the island (the caught path's buyer reads it).
    #[serde(default)]
    pub deeds: Deeds,
}

/// Work done on the island, counted as it happens: for the lessons, and for
/// who buys a debtor out (the Lantern Company for fighters, the Writers for
/// makers: the second session's `bought`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Deeds {
    /// Units cut or dug on a shift.
    pub laboured: u32,
    /// Things made in the compound.
    pub made: u32,
    /// Fights won on the island.
    pub fights_won: u32,
}

/// A deed on the island.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deed {
    Labour,
    Made,
    FightWon,
}

/// Units of a shift's work that satisfy the labour lesson.
pub const SHIFT_UNITS: u32 = 3;
/// A shift's hours of the day (from, to).
pub const SHIFT_HOURS: (f64, f64) = (7.0, 17.0);

/// Where a bound squad member is put on the island.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shift {
    /// A deposit to work: its id and where it is.
    Face(u32, V2),
    /// The workyard: making, at the player's choice.
    Workyard(V2),
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
        Tutorial { path: None, done: Vec::new(), started: t, woke_at: at, over: false, deeds: Deeds::default() }
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
    /// path when the lesson is one path's, and the night run (a sneak into
    /// the compound) sets the sneak path.
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
        if key == "night_run" && tu.path.is_none() {
            tu.path = Some(Path::Sneak);
        }
        tu.done.push((x.key, t));
        self.log.push_front((t, format!("Lesson done: {}.", x.title)));
        // Stowed away without a fight: never spotted, well done.
        if key == "stow" {
            self.lesson_done("spotted");
        }
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
            // Shifts worked, things made (`Deeds`, counted as they happen).
            "labour" => self.tutorial.as_ref().is_some_and(|t| t.deeds.laboured >= SHIFT_UNITS),
            "workyard" => self.tutorial.as_ref().is_some_and(|t| t.deeds.made >= 1),
            // Inside the palisade after dark, sneaking: the night run. (By
            // day through the gate, the law takes them: the caught path.)
            "night_run" => {
                let Some(isle) = self.island() else { return false };
                let dark = super::stealth::daylight(self.time) < 0.3;
                dark && self.squad.members.iter().enumerate().any(|(k, &m)| self.squad.sneaking[k] && isle.in_compound(self.person_pos(m)))
            }
            // Something of the compound's gone through.
            "stores" => {
                let Some(t) = self.island().map(|i| i.settlement) else { return false };
                self.containers.range((t, 0, 0)..=(t, u16::MAX, u8::MAX)).any(|(_, c)| c.taken > 0)
            }
            // At the gate by day, passing as one of theirs.
            "gate" => {
                let Some(isle) = self.island() else { return false };
                let gate = isle.gate();
                super::stealth::daylight(self.time) >= 0.3 && self.squad.members.iter().any(|&m| self.person_pos(m).dist(gate.at) <= gate.width && self.disguise_of(m).is_some())
            }
            // A fight with the compound's people, or its watch after them.
            "spotted" => {
                let Some(t) = self.island().map(|i| i.settlement) else { return false };
                self.pursuits.iter().any(|p| p.town == t) || self.squad_battle().is_some_and(|b| b.fighters.iter().any(|f| f.is_person() && !self.people[f.pid as usize].in_squad && self.people[f.pid as usize].home == Some(t)))
            }
            // The wreck's crate gone through (`island::WRECK_CRATE`).
            "salvage" => self.containers.get(&super::island::WRECK_CRATE).is_some_and(|c| c.taken > 0 || c.items.is_empty()),
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

    /// Skip the island (a later playthrough): the squad stands on the
    /// mainland port's dock with a traveller's kit, the island behind them.
    pub fn skip_island(&mut self) {
        if !self.on_island() {
            return;
        }
        let t = self.time;
        for &m in &self.squad.members.clone() {
            self.people[m as usize].ensure_detail();
            let d = self.people[m as usize].detail.as_mut().unwrap();
            for key in super::island::BASIC_WEAR {
                let id = super::items::id(key);
                d.gear.add_piece(id, super::wear::fresh(id, t));
                let _ = d.gear.equip(id);
            }
            for (key, n) in super::island::BASIC_PACK {
                d.gear.add(super::items::id(key), n);
            }
            self.people[m as usize].recompute_might();
        }
        if let Some((_, dock)) = self.ferry_port() {
            let at = self.free_spot(dock.add(V2::new(4.0, 0.0)));
            self.teleport_squad(at);
        }
        self.end_tutorial();
        self.log.push_front((t, "The ferry puts you ashore on the mainland.".to_string()));
    }

    /// Count a deed on the island (nothing off it, or after it's over).
    pub fn island_deed(&mut self, d: Deed) {
        if !self.on_island() {
            return;
        }
        if let Some(tu) = self.tutorial.as_mut() {
            match d {
                Deed::Labour => tu.deeds.laboured += 1,
                Deed::Made => tu.deeds.made += 1,
                Deed::FightWon => tu.deeds.fights_won += 1,
            }
        }
    }

    /// What they've done on the island so far.
    pub fn island_deeds(&self) -> Option<Deeds> {
        self.tutorial.as_ref().map(|t| t.deeds)
    }

    /// The bound squad member's shift on the island, by the day and the
    /// hour: a deposit to work (the woodlot and the quarry, turn about), the
    /// workyard every third day, or none (off shift). Pure: the overseers'
    /// rota is the clock.
    pub fn island_shift(&self, t: f64) -> Option<Shift> {
        let isle = self.island()?;
        let h = (t / HOUR).rem_euclid(24.0);
        if !(SHIFT_HOURS.0..SHIFT_HOURS.1).contains(&h) {
            return None;
        }
        let day = World::day_of(t);
        if day % 3 == 2 {
            return Some(Shift::Workyard(isle.spot(Spot::Workyard)));
        }
        let faces: Vec<&super::labour::Deposit> = self.deposits.iter().filter(|d| d.town == isle.settlement).collect();
        if faces.is_empty() {
            return Some(Shift::Workyard(isle.spot(Spot::Workyard)));
        }
        let d = faces[(day.rem_euclid(faces.len() as i64)) as usize];
        Some(Shift::Face(d.id, d.pos))
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
