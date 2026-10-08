//! Making things: potions, scrolls, weapons and armour.
//!
//! Four crafts, each with its own skill and place to work:
//!
//! | Craft | Skill | Where |
//! |---|---|---|
//! | Potions | Alchemy | anywhere with a mortar and pestle, or at an alchemy table |
//! | Scrolls | Inscription | a scribe's desk |
//! | Weapons (and smelting) | Smithing | a forge |
//! | Armour (and tanning) | Armoring | an armourer's bench (a forge for iron pieces) |
//!
//! Every town has the four stations round its hearth. A recipe takes its
//! materials up front and some game time at the station; then a keyed roll
//! against skill and difficulty decides whether it worked. A botched job
//! gives half the materials back. Either way the skill improves (more for a
//! success).
//!
//! Materials come from the land — kelp on the coast, emberroot and salt on
//! the Qotiro plateau, ore and storm glass in the mountains, deadwood
//! anywhere — and from what people keep at home. A picked spot grows back
//! after a day.

use super::geo::{self, V2};
use super::items::{self, item, ItemId, Kind};
use super::person::PersonId;
use super::rng::Rng;
use super::stats::Skill;
use super::world::{World, DAY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Station {
    Forge,
    Bench,
    Desk,
    AlchemyTable,
}

impl Station {
    pub fn name(self) -> &'static str {
        match self {
            Station::Forge => "Forge",
            Station::Bench => "Armourer's bench",
            Station::Desk => "Scribe's desk",
            Station::AlchemyTable => "Alchemy table",
        }
    }
}

pub const STATIONS: [Station; 4] = [Station::Forge, Station::Bench, Station::Desk, Station::AlchemyTable];

/// How close to a station you must stand to use it, metres.
pub const AT_STATION: f32 = 3.0;

pub struct Recipe {
    pub output: &'static str,
    pub makes: u16,
    pub inputs: &'static [(&'static str, u16)],
    pub skill: Skill,
    /// Skill at which it works about four times in five.
    pub difficulty: f32,
    pub station: Station,
    /// Game seconds at the station.
    pub time: f64,
}

const fn r(output: &'static str, makes: u16, inputs: &'static [(&'static str, u16)], skill: Skill, difficulty: f32, station: Station, time: f64) -> Recipe {
    Recipe { output, makes, inputs, skill, difficulty, station, time }
}

pub static RECIPES: &[Recipe] = &[
    // Alchemy: a mortar and pestle in your pack does as well as the table.
    r("healing_draught", 1, &[("kelp_frond", 2), ("ash_moss", 1)], Skill::Alchemy, 15.0, Station::AlchemyTable, 40.0),
    r("mana_tonic", 1, &[("ghostcap", 2), ("salt_crystal", 1)], Skill::Alchemy, 25.0, Station::AlchemyTable, 40.0),
    r("greater_healing", 1, &[("kelp_frond", 2), ("ghostcap", 1), ("salt_crystal", 1)], Skill::Alchemy, 45.0, Station::AlchemyTable, 60.0),
    // Inscription.
    r("scroll_heal", 1, &[("reed_paper", 1), ("squid_ink", 1), ("kelp_frond", 1)], Skill::Inscription, 20.0, Station::Desk, 60.0),
    r("scroll_lightning", 1, &[("reed_paper", 1), ("squid_ink", 1), ("storm_glass", 1)], Skill::Inscription, 35.0, Station::Desk, 60.0),
    r("scroll_paralyze", 1, &[("reed_paper", 1), ("squid_ink", 1), ("ghostcap", 2)], Skill::Inscription, 40.0, Station::Desk, 60.0),
    r("scroll_fireball", 1, &[("reed_paper", 1), ("squid_ink", 1), ("emberroot", 2)], Skill::Inscription, 45.0, Station::Desk, 60.0),
    // Smithing.
    r("iron_ingot", 1, &[("iron_ore", 2)], Skill::Smithing, 5.0, Station::Forge, 60.0),
    r("knife", 1, &[("iron_ingot", 1), ("leather", 1)], Skill::Smithing, 10.0, Station::Forge, 90.0),
    r("spear", 1, &[("iron_ingot", 1), ("timber", 2)], Skill::Smithing, 20.0, Station::Forge, 120.0),
    r("short_sword", 1, &[("iron_ingot", 2), ("leather", 1)], Skill::Smithing, 30.0, Station::Forge, 150.0),
    r("war_pick", 1, &[("iron_ingot", 3), ("timber", 1)], Skill::Smithing, 40.0, Station::Forge, 180.0),
    r("longsword", 1, &[("iron_ingot", 3), ("leather", 1)], Skill::Smithing, 50.0, Station::Forge, 210.0),
    // Armoring.
    r("leather", 1, &[("hide", 2)], Skill::Armoring, 5.0, Station::Bench, 60.0),
    r("leather_cap", 1, &[("leather", 1)], Skill::Armoring, 8.0, Station::Bench, 60.0),
    r("leather_gloves", 1, &[("leather", 1)], Skill::Armoring, 10.0, Station::Bench, 60.0),
    r("boots", 1, &[("leather", 2)], Skill::Armoring, 10.0, Station::Bench, 90.0),
    r("hide_coat", 1, &[("hide", 4)], Skill::Armoring, 20.0, Station::Bench, 120.0),
    r("buckler", 1, &[("iron_ingot", 1), ("timber", 2)], Skill::Armoring, 25.0, Station::Bench, 120.0),
    r("iron_helm", 1, &[("iron_ingot", 2), ("leather", 1)], Skill::Armoring, 35.0, Station::Forge, 150.0),
    r("scale_hauberk", 1, &[("iron_ingot", 6), ("leather", 2)], Skill::Armoring, 55.0, Station::Forge, 300.0),
];

/// Chance a job comes out right.
pub fn success_chance(skill: f32, difficulty: f32) -> f32 {
    (0.8 + (skill - difficulty) * 0.02).clamp(0.05, 0.98)
}

/// Someone at work at a station.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Job {
    pub who: PersonId,
    pub recipe: usize,
    pub done_at: f64,
    /// Which job this is for them (keys the roll).
    pub n: u64,
}

/// A spot where something can be gathered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub id: u32,
    pub pos: V2,
    pub item: ItemId,
    pub amount: u16,
    /// When it was last picked (it grows back a day later).
    pub picked_at: Option<f64>,
}

impl Node {
    pub fn ready(&self, t: f64) -> bool {
        self.picked_at.map(|p| t - p >= DAY).unwrap_or(true)
    }
}

/// Why a recipe can't be made right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cannot {
    Missing(&'static str, u16),
    NoStation(Station),
    Busy,
}

impl World {
    // ---- Setting up ---------------------------------------------------------

    /// Workshops round every town's hearth, and things to gather across the land.
    pub(super) fn place_crafting(&mut self) {
        for s in &self.settlements {
            for (k, st) in STATIONS.iter().enumerate() {
                let a = k as f32 / 4.0 * std::f32::consts::TAU + 0.4;
                let p = s.pos.add(V2::new(a.cos(), a.sin()).scale(8.5));
                self.stations.push((p, *st));
            }
        }
        let mut r = Rng::from_keys(&[self.seed, 0x4E4F_4445]);
        let size = geo::WORLD_SIZE;
        let mut id = 0;
        for _ in 0..9000 {
            if self.nodes.len() >= 700 {
                break;
            }
            let p = V2::new(r.range(300.0, size - 300.0), r.range(300.0, size - 300.0));
            let inland = geo::inland(p);
            let (h, mt, pl) = (self.terrain.height(p), self.terrain.mountains(p), self.terrain.plateau(p));
            let roll = r.f32();
            let key = if (-40.0..15.0).contains(&inland) {
                if roll < 0.7 { "kelp_frond" } else { "salt_crystal" }
            } else if inland < 0.0 {
                continue;
            } else if mt > 0.45 && h > 300.0 {
                if roll < 0.3 { "storm_glass" } else { "iron_ore" }
            } else if mt > 0.2 {
                if roll < 0.55 { "iron_ore" } else { "ash_moss" }
            } else if pl > 0.5 {
                if roll < 0.6 { "emberroot" } else { "salt_crystal" }
            } else if roll < 0.35 {
                "timber"
            } else if roll < 0.6 {
                "ghostcap"
            } else if roll < 0.75 {
                "ash_moss"
            } else {
                continue;
            };
            // Not in the middle of town.
            if self.settlements.iter().any(|s| s.pos.dist(p) < s.reach) {
                continue;
            }
            let amount = 1 + r.below(3) as u16;
            self.nodes.push(Node { id, pos: p, item: items::id(key), amount, picked_at: None });
            id += 1;
        }
    }

    // ---- Gathering ----------------------------------------------------------

    /// Send someone to gather from a spot.
    pub fn order_gather(&mut self, who: PersonId, node: u32) -> bool {
        let Some(n) = self.nodes.iter().find(|n| n.id == node).copied() else { return false };
        let Some(k) = self.squad.index(who) else { return false };
        let (path, _) = self.route(self.member_pos(k), n.pos);
        self.squad.goal[k] = *path.last().unwrap_or(&n.pos);
        self.squad.route[k] = path;
        self.gathering.retain(|g| g.0 != who);
        self.gathering.push((who, node));
        true
    }

    pub(super) fn do_gathering(&mut self) {
        let mut done = Vec::new();
        for (i, &(who, node)) in self.gathering.iter().enumerate() {
            let (Some(k), Some(ni)) = (self.squad.index(who), self.nodes.iter().position(|n| n.id == node)) else {
                done.push(i);
                continue;
            };
            if self.squad.at[k].dist(self.nodes[ni].pos) > super::squad::REACH {
                continue;
            }
            done.push(i);
            if !self.nodes[ni].ready(self.time) {
                continue;
            }
            let n = self.nodes[ni];
            self.nodes[ni].picked_at = Some(self.time);
            let name = self.people[who as usize].name().unwrap_or("someone").to_string();
            if let Some(d) = self.people[who as usize].detail.as_mut() {
                d.gear.add(n.item, n.amount);
            }
            self.people[who as usize].recompute_might();
            self.log.push_front((self.time, format!("{name} gathers {} × {}.", n.amount, item(n.item).name.to_lowercase())));
            self.log.truncate(14);
        }
        for i in done.into_iter().rev() {
            self.gathering.remove(i);
        }
    }

    // ---- Crafting -----------------------------------------------------------

    /// The nearest station of a kind, if within reach of `p`.
    pub fn station_near(&self, p: V2, kind: Station) -> Option<V2> {
        self.stations.iter().filter(|(_, k)| *k == kind).map(|(s, _)| *s).find(|s| s.dist(p) <= AT_STATION)
    }

    pub fn count_of(&self, who: PersonId, key: &str) -> u16 {
        let id = items::id(key);
        self.people[who as usize].detail.as_ref().map(|d| d.gear.bag.iter().filter(|e| e.0 == id).map(|e| e.1).sum()).unwrap_or(0)
    }

    /// Can this person make recipe `ri` where they stand?
    pub fn can_craft(&self, who: PersonId, ri: usize) -> Result<(), Cannot> {
        let rc = &RECIPES[ri];
        if self.crafting.iter().any(|j| j.who == who) || self.fighting.contains_key(&who) {
            return Err(Cannot::Busy);
        }
        for &(k, n) in rc.inputs {
            if self.count_of(who, k) < n {
                return Err(Cannot::Missing(k, n));
            }
        }
        let at = self.person_pos(who);
        let portable = rc.station == Station::AlchemyTable && self.count_of(who, "mortar_and_pestle") > 0;
        if !portable && self.station_near(at, rc.station).is_none() {
            return Err(Cannot::NoStation(rc.station));
        }
        Ok(())
    }

    /// Start making something: materials go in now.
    pub fn start_craft(&mut self, who: PersonId, ri: usize) -> Result<(), Cannot> {
        self.can_craft(who, ri)?;
        let rc = &RECIPES[ri];
        if let Some(d) = self.people[who as usize].detail.as_mut() {
            for &(k, n) in rc.inputs {
                for _ in 0..n {
                    d.gear.take(items::id(k));
                }
            }
        }
        self.people[who as usize].recompute_might();
        let n = self.crafted_count.entry(who).or_insert(0);
        *n += 1;
        let job = Job { who, recipe: ri, done_at: self.time + rc.time, n: *n };
        self.crafting.push(job);
        Ok(())
    }

    pub(super) fn do_crafting(&mut self) {
        let mut done = Vec::new();
        for (i, j) in self.crafting.iter().enumerate() {
            if self.time >= j.done_at {
                done.push((i, *j));
            }
        }
        for &(i, _) in done.iter().rev() {
            self.crafting.remove(i);
        }
        for (_, j) in done {
            let rc = &RECIPES[j.recipe];
            let p = &self.people[j.who as usize];
            let skill = p.effective_stats().skill(rc.skill);
            let roll = Rng::from_keys(&[self.seed, j.who as u64, j.n, 0x4352_4146]).f32();
            let name = p.name().unwrap_or("someone").to_string();
            let ok = roll < success_chance(skill, rc.difficulty);
            let p = &mut self.people[j.who as usize];
            p.stats.exercise(rc.skill, if ok { 2.0 } else { 0.8 });
            let line = if ok {
                if let Some(d) = p.detail.as_mut() {
                    d.gear.add(items::id(rc.output), rc.makes);
                }
                format!("{name} makes {}.", item(items::id(rc.output)).name.to_lowercase())
            } else {
                if let Some(d) = p.detail.as_mut() {
                    for &(k, n) in rc.inputs {
                        if n / 2 > 0 {
                            d.gear.add(items::id(k), n / 2);
                        }
                    }
                }
                format!("{name} botches the {}.", item(items::id(rc.output)).name.to_lowercase())
            };
            p.recompute_might();
            self.log.push_front((j.done_at, line));
            self.log.truncate(14);
        }
    }

    /// How far along someone's job is, 0..1.
    pub fn craft_progress(&self, who: PersonId) -> Option<f32> {
        let j = self.crafting.iter().find(|j| j.who == who)?;
        let total = RECIPES[j.recipe].time;
        Some((1.0 - (j.done_at - self.time) / total).clamp(0.0, 1.0) as f32)
    }

    // ---- Using things -------------------------------------------------------

    /// Drink a potion or read a scroll outside a fight. Returns false if it
    /// can't be used now (attack scrolls are for fights).
    pub fn use_item(&mut self, who: PersonId, it: ItemId) -> bool {
        let t = self.time;
        let p = &self.people[who as usize];
        let Some(d) = p.detail.as_ref() else { return false };
        if !d.gear.bag.iter().any(|e| e.0 == it) || self.fighting.contains_key(&who) {
            return false;
        }
        let (heal, mana) = match item(it).kind {
            Kind::Potion(pd) => (pd.heal, pd.mana),
            Kind::Scroll(super::magic::Spell::Heal) => (super::magic::Spell::Heal.def().magnitude, 0.0),
            _ => return false,
        };
        let name = p.name().unwrap_or("someone").to_string();
        let p = &mut self.people[who as usize];
        p.detail.as_mut().unwrap().gear.take(it);
        if heal > 0.0 {
            let base = p.stats.clone();
            let mut hp = p.wounds.hp_at(&base, t);
            mend(&mut hp, &base, heal);
            p.wounds.set(&base, &hp, t);
        }
        if mana > 0.0 {
            let m = (p.mana_at(t) + mana).min(p.max_mana());
            p.set_mana(m, t);
        }
        p.recompute_might();
        self.log.push_front((t, format!("{name} uses the {}.", item(it).name.to_lowercase())));
        self.log.truncate(14);
        true
    }
}

/// Spread `amount` of healing over a body, worst-hurt vital parts first.
pub fn mend(hp: &mut [f32; 6], stats: &super::stats::Stats, mut amount: f32) {
    use super::body::PARTS;
    while amount > 0.01 {
        // The part furthest below its maximum, vital parts weighted up.
        let Some((i, gap)) = PARTS
            .iter()
            .enumerate()
            .map(|(i, part)| (i, (stats.max_hp(*part) - hp[i]) * if part.vital() { 1.5 } else { 1.0 }))
            .filter(|(_, g)| *g > 0.01)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            break;
        };
        let room = stats.max_hp(PARTS[i]) - hp[i];
        let give = amount.min(room).min(gap.max(5.0));
        hp[i] += give;
        amount -= give;
    }
}
