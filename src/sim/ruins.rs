//! Ruins and lairs (the playable-MVP list): a handful of places off the
//! roads worth the walk. A ruin is held by a hard band of wardens (bandits
//! who've dug in; they watch like a camp and rob like one); a lair is the
//! home of one of the great beasts (a Cragmaw, a Chasm Lurker) with what's
//! left of those who came before. Each has a cache lying in it: coin and a
//! few good things.
//!
//! Placed once from the seed at world-making; the caches are containers out
//! in the wild (`containers::WILD`): a ruin's is a locked chest and a crate
//! with something to read, a lair's an old chest. They're nobody's, so
//! taking is no crime, but the wardens hear the lid. The wardens are an
//! ordinary band. Placeholder English names.

use serde::{Deserialize, Serialize};

use super::animals::Sp;
use super::geo::{self, V2};
use super::group::GroupId;
use super::items;
use super::rng::Rng;
use super::containers::{Container, Owner, WILD};
use super::inventory::Entry;
use super::layout::Holder;
use super::world::World;

/// How many warded ruins, and lairs at most.
pub const RUINS: usize = 5;
pub const LAIRS: usize = 4;
/// Ruins keep this far from towns and roads, metres.
pub const FROM_TOWNS: f32 = 1500.0;
pub const FROM_ROADS: f32 = 400.0;
/// How close the squad must come to find one.
pub const FIND_AT: f32 = 150.0;
/// Wardens are this much better at their skills than a roadside bandit.
pub const WARDEN_EDGE: f32 = 18.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum RuinKind {
    Ruin,
    /// A great beast's home: the herd.
    Lair(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ruin {
    pub id: u32,
    pub pos: V2,
    pub kind: RuinKind,
    /// The wardens' band (ruins).
    pub guards: Option<GroupId>,
    /// The squad has come upon it.
    pub found: bool,
}

/// What a ruin's cache may hold besides coin (keys; two are picked).
const RUIN_GOODS: &[&str] = &["gold_ring", "pearl_necklace", "greater_healing", "mana_tonic", "scroll_fireball", "scroll_lightning", "notes_paralyze", "manual_smithing", "manual_alchemy", "longsword", "scale_hauberk", "iron_helm", "kite_shield", "large_pack", "crossbow"];
/// What lies in a lair, left by those who came before (one is picked).
const LAIR_GOODS: &[&str] = &["longsword", "glaive", "war_pick", "scale_greaves", "iron_helm", "healing_draught", "greater_healing", "gold_ring"];
/// Something to read in a ruin's crate (one is picked).
const READING: &[&str] = &["notes_paralyze", "manual_smithing", "manual_alchemy", "scroll_fireball", "scroll_lightning"];
/// And something the wardens live on (one is picked, with how many).
const SUPPLIES: &[(&str, u16)] = &[("flatbread", 4), ("dried_fish", 3), ("torch", 2), ("healing_draught", 1)];
/// How hard a ruin's chest is to pick (between these).
pub const CHEST_LOCK: (f32, f32) = (30.0, 60.0);

/// Distance from a point to the nearest road.
fn road_dist(w: &World, p: V2) -> f32 {
    let mut best = f32::MAX;
    for road in &w.routes.roads {
        for s in road.windows(2) {
            let (a, b) = (s[0], s[1]);
            let d = b.sub(a);
            let l2 = d.x * d.x + d.y * d.y;
            let t = if l2 < 1e-6 { 0.0 } else { ((p.sub(a).x * d.x + p.sub(a).y * d.y) / l2).clamp(0.0, 1.0) };
            best = best.min(a.add(d.scale(t)).dist(p));
        }
    }
    best
}

impl Ruin {
    pub fn name(&self, w: &World) -> String {
        match self.kind {
            RuinKind::Ruin => "Old ruin".to_string(),
            RuinKind::Lair(h) => format!("{} lair", w.animals.herds.get(h as usize).map(|h| h.def().name).unwrap_or("Beast")),
        }
    }
}

impl World {
    /// Place the ruins and lairs (world-making, after the animals).
    pub(super) fn place_ruins(&mut self) {
        let mut r = Rng::from_keys(&[self.seed, 0x5255_494E]);
        let size = geo::WORLD_SIZE;
        let mut out: Vec<Ruin> = Vec::new();
        let mut tries = 0;
        while out.len() < RUINS && tries < 6000 {
            tries += 1;
            let at = V2::new(r.range(400.0, size - 400.0), r.range(400.0, size - 400.0));
            let ok = geo::is_land(at)
                && !self.terrain.is_sea(at)
                && geo::inland(at) > 200.0
                && self.terrain.slope(at) < 0.25
                && self.settlements.iter().all(|s| s.pos.dist(at) > s.radius() + FROM_TOWNS)
                && at.dist(self.squad.pos) > 2000.0
                && out.iter().all(|o| o.pos.dist(at) > 2500.0)
                && self.camps.iter().all(|c| c.pos.dist(at) > 1000.0)
                && road_dist(self, at) > FROM_ROADS;
            if !ok {
                continue;
            }
            let n = 3 + r.below(3);
            let mage = r.chance(0.5);
            let first = self.people.len();
            let gid = self.add_bandits(at, n, mage, false);
            // Dug in for a long while: harder than the roadside sort.
            for p in first..self.people.len() {
                let pp = &mut self.people[p];
                for s in super::stats::SKILLS {
                    let v = pp.stats.skill(s);
                    if v >= 15.0 {
                        pp.stats.set_skill(s, v + WARDEN_EDGE);
                    }
                }
                pp.recompute_might();
            }
            // Everyone has a line in the society's lists (the camps' bandits
            // got theirs when society was founded; these come after).
            while self.society.lives.len() < self.people.len() {
                let p = &self.people[self.society.lives.len()];
                let l = super::society::Life::new(p.race, p.seed);
                self.society.lives.push(l);
            }
            while self.society.minds.len() < self.people.len() {
                self.society.minds.push(Default::default());
            }
            // (The camp's first look at the roads found its raids; wardens don't raid.)
            self.encounters.retain(|e| e.camp != gid);
            out.push(Ruin { id: out.len() as u32, pos: at, kind: RuinKind::Ruin, guards: Some(gid), found: false });
        }
        // Lairs: the great beasts' homes.
        let mut beasts: Vec<(u32, V2)> = Vec::new();
        for sp in [Sp::Cragmaw, Sp::ChasmLurker] {
            beasts.extend(self.animals.herds.iter().filter(|h| h.sp == sp).map(|h| (h.id, h.home)));
        }
        for (h, home) in beasts.into_iter().take(LAIRS) {
            out.push(Ruin { id: out.len() as u32, pos: home, kind: RuinKind::Lair(h), guards: None, found: false });
        }
        // The caches: a ruin's in a locked chest the wardens keep, with a
        // crate beside it holding something to read; a lair's in an old
        // chest whose owner won't be back for it.
        for ru in &out {
            let mut rr = Rng::from_keys(&[self.seed, ru.id as u64, 0x4341_4348]);
            let (coin, goods, picks) = match ru.kind {
                RuinKind::Ruin => (60 + rr.below(120) as u16, RUIN_GOODS, 2),
                RuinKind::Lair(_) => (30 + rr.below(90) as u16, LAIR_GOODS, 1),
            };
            let mut chest: Vec<Entry> = vec![Entry(items::id("coin"), coin, None)];
            for _ in 0..picks {
                let it = items::id(goods[rr.below(goods.len())]);
                match chest.iter_mut().find(|e| e.0 == it) {
                    Some(e) => e.1 += 1,
                    None => chest.push(Entry(it, 1, None)),
                }
            }
            let rot = rr.f32() * std::f32::consts::TAU;
            let at = |k: f32| ru.pos.add(V2::new(rot.cos(), rot.sin()).scale(1.6 * k));
            let lock = match ru.kind {
                RuinKind::Ruin => CHEST_LOCK.0 + rr.f32() * (CHEST_LOCK.1 - CHEST_LOCK.0),
                RuinKind::Lair(_) => 0.0,
            };
            let id = (WILD, ru.id as u16, 0);
            self.containers.insert(id, Container { id, what: Holder::Chest, pos: at(1.0), rot, items: chest, lock, picked: false, owner: Owner::Nobody, taken: 0, ours: Vec::new() });
            if ru.kind == RuinKind::Ruin {
                let mut crate_: Vec<Entry> = vec![Entry(items::id(READING[rr.below(READING.len())]), 1, None)];
                let (k, n) = SUPPLIES[rr.below(SUPPLIES.len())];
                crate_.push(Entry(items::id(k), n, None));
                let id = (WILD, ru.id as u16, 1);
                self.containers.insert(id, Container { id, what: Holder::Crate, pos: at(-1.0), rot: rot + 0.4, items: crate_, lock: 0.0, picked: false, owner: Owner::Nobody, taken: 0, ours: Vec::new() });
            }
        }
        self.ruins = out;
    }

    /// Note ruins the squad comes upon (the squad's step).
    pub(super) fn find_ruins(&mut self) {
        let here = self.squad.pos;
        let t = self.time;
        let mut found = Vec::new();
        for ru in self.ruins.iter_mut().filter(|r| !r.found) {
            if ru.pos.dist(here) <= FIND_AT {
                ru.found = true;
                found.push(ru.clone());
            }
        }
        for ru in found {
            let what = match ru.kind {
                RuinKind::Ruin => {
                    let held = ru.guards.and_then(|g| self.group(g)).is_some_and(|g| g.members.iter().any(|&m| !self.people[m as usize].dead));
                    if held {
                        "You come upon an old ruin. Someone has dug in here — and something glints inside.".to_string()
                    } else {
                        "You come upon an old ruin, empty now.".to_string()
                    }
                }
                RuinKind::Lair(_) => format!("You come upon a {}. Bones, and the gleam of things their owners won't need.", ru.name(self).to_lowercase()),
            };
            self.alerts.push(what.clone());
            self.log.push_front((t, what));
            self.log.truncate(14);
        }
    }

    /// Is this band a ruin's wardens?
    pub fn is_warden(&self, g: GroupId) -> bool {
        self.ruins.iter().any(|r| r.guards == Some(g))
    }

    /// Is a ruin's guard (wardens or beast) still there?
    pub fn ruin_held(&self, id: u32) -> bool {
        let Some(ru) = self.ruins.get(id as usize) else { return false };
        match ru.kind {
            RuinKind::Ruin => ru.guards.and_then(|g| self.group(g)).is_some_and(|g| g.members.iter().any(|&m| !self.people[m as usize].dead)),
            RuinKind::Lair(h) => self.animals.herds.get(h as usize).is_some_and(|h| h.alive(self.time) > 0),
        }
    }

    /// How many things are still in a ruin's cache (its chest and crate).
    pub fn ruin_cache(&self, id: u32) -> usize {
        self.ruin_containers(id).map(|c| c.items.len()).sum()
    }

    /// A ruin's (or lair's) cache: its chest, and a ruin's crate.
    pub fn ruin_containers(&self, id: u32) -> impl Iterator<Item = &Container> {
        self.containers.range((WILD, id as u16, 0)..=(WILD, id as u16, u8::MAX)).map(|(_, c)| c)
    }
}

/// Every key the caches can hold exists.
pub fn check_tables() -> Result<(), String> {
    for k in RUIN_GOODS.iter().chain(LAIR_GOODS).chain(READING).chain(SUPPLIES.iter().map(|s| &s.0)).chain(super::containers::STASH_GOODS) {
        if !items::catalogue_has(k) {
            return Err(k.to_string());
        }
    }
    Ok(())
}
