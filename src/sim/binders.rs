//! The Binders' slave raids (docs/factions.md: "the main human predator on
//! the roads. Slave raids, people to free, and the way a squad member can
//! end up enslaved"; F6).
//!
//! **Camps.** A few bands of slavers, mostly Qotiro, camp by the roads like
//! bandits (`Camp::slavers`), and are found and fought the same way: their
//! ambushes come from the schedules when legs are written, and a raid away
//! from the squad is fought out with the full rules at once (rule 7). They
//! are placed after everything else in the world, so nobody else's dice or
//! numbering move.
//!
//! **Travellers they beat** are taken and sold into bondage through the
//! law's bonds (`law::bond`, `sell_bond`), to a town where slavery is
//! allowed; like any bound townsfolk they keep their work, and what they
//! earn goes to whoever holds them.
//!
//! **A squad they beat** isn't robbed: the downed are carried off to the
//! camp (`Captive`), take no orders, and are kept by the fire. Each dawn a
//! captive may slip away (a keyed roll, better the sneakier and quicker
//! they are); after `CAPTIVE_DAYS` unsold they're sold on, into a slave's
//! bond in the nearest town that allows it, and the law's own rules (buying
//! out, running) take over. Beating the band frees everyone held there.
//!
//! Not yet: selling to the Free Lords themselves (the lords have no seats
//! until Laz's map), Binder kit of their own (saps and nets).

use serde::{Deserialize, Serialize};

use super::combat::{Battle, SQUAD_SIDE};
use super::culture::Slavery;
use super::geo::{self, V2};
use super::group::GroupId;
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::{World, DAY};

/// Slavers' camps across the world at the start.
pub const BINDER_CAMPS: usize = 3;
/// Days a captive is held before the Binders sell them on.
pub const CAPTIVE_DAYS: f64 = 3.0;
/// A captive's chance of slipping away each dawn, before skill: and what
/// Sneak and Agility add (per point).
pub const ESCAPE_BASE: f32 = 0.08;
pub const ESCAPE_SNEAK: f32 = 1.0 / 400.0;
pub const ESCAPE_AGILITY: f32 = 1.0 / 600.0;
/// What beating a slavers' band does to standing with the Binders.
pub const BEATEN_DEED: f32 = -12.0;

/// A squad member held at a slavers' camp.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Captive {
    pub who: PersonId,
    pub camp: GroupId,
    pub since: f64,
}

impl World {
    /// Is this a slavers' camp?
    pub fn is_slaver_camp(&self, gid: GroupId) -> bool {
        self.camps.iter().any(|c| c.group == gid && c.slavers)
    }

    /// Set up the Binders' camps: beside the roads, clear of the towns, of
    /// the squad's start and of the other camps.
    pub(super) fn place_binders(&mut self) {
        let mut r = Rng::from_keys(&[self.seed, 0xB1_4D_E5]);
        let mut tries = 0;
        let mut placed = 0;
        while placed < BINDER_CAMPS && tries < 4000 && !self.routes.roads.is_empty() {
            tries += 1;
            let road = &self.routes.roads[r.below(self.routes.roads.len())];
            if road.len() < 2 {
                continue;
            }
            let i = r.below(road.len() - 1);
            let (a, b) = (road[i], road[i + 1]);
            let d = b.sub(a);
            let len = d.len().max(1e-3);
            let side = if r.chance(0.5) { 1.0 } else { -1.0 };
            let off = V2::new(-d.y / len, d.x / len).scale(side * r.range(50.0, 90.0));
            let at = a.lerp(b, r.f32()).add(off);
            let ok = geo::is_land(at)
                && geo::inland(at) > 150.0
                && self.terrain.slope(at) < 0.2
                && self.settlements.iter().all(|s| s.pos.dist(at) > s.radius() + 900.0)
                && at.dist(self.squad.pos) > 1800.0
                && self.camps.iter().all(|c| c.pos.dist(at) > 1500.0);
            if !ok {
                continue;
            }
            let n = 4 + r.below(3);
            self.add_band(at, n, false, false, true);
            placed += 1;
        }
        // Everyone has a line in the society's lists (these come after its
        // founding, like the ruins' wardens).
        while self.society.lives.len() < self.people.len() {
            let p = &self.people[self.society.lives.len()];
            let l = super::society::Life::new(p.race, p.seed);
            self.society.lives.push(l);
        }
        while self.society.minds.len() < self.people.len() {
            self.society.minds.push(Default::default());
        }
    }

    /// A slavers' band at `at`, met already (for tests and scenes).
    pub fn spawn_binders(&mut self, at: V2, count: usize) -> GroupId {
        self.add_band(at, count, false, true, true)
    }

    /// The slavers' camp holding someone, if they're held.
    pub fn captive_of(&self, pid: PersonId) -> Option<&Captive> {
        self.captives.iter().find(|c| c.who == pid)
    }

    /// Where a captive is kept: by the camp's fire, each in their place.
    fn held_spot(&self, camp: GroupId, k: usize) -> Option<V2> {
        let c = self.camps.iter().find(|c| c.group == camp)?;
        let a = 0.9 + k as f32 * 1.7;
        Some(self.free_spot(c.pos.add(V2::new(a.cos(), a.sin()).scale(5.0))))
    }

    /// The squad beaten by slavers: the downed are carried off to their
    /// camp, not robbed. True if it was slavers (so no robbing).
    pub(super) fn take_captives(&mut self, b: &Battle) -> bool {
        let Some(won) = b.winner() else { return false };
        if won == SQUAD_SIDE {
            return false;
        }
        let Some(camp) = b.fighters.iter().filter(|f| f.home == won && f.is_person()).filter_map(|f| self.group_of[f.pid as usize]).find(|&g| self.is_slaver_camp(g)) else { return false };
        let t = b.time;
        let taken: Vec<PersonId> = b.fighters.iter().filter(|f| f.home == SQUAD_SIDE && f.is_person() && !f.dead && self.squad.index(f.pid).is_some() && self.is_down(f.pid)).map(|f| f.pid).collect();
        for (n, &who) in taken.iter().enumerate() {
            self.captives.retain(|c| c.who != who);
            self.captives.push(Captive { who, camp, since: t });
            if let (Some(k), Some(spot)) = (self.squad.index(who), self.held_spot(camp, n)) {
                self.squad.at[k] = spot;
                self.squad.goal[k] = spot;
                self.squad.route[k].clear();
                self.squad.resting[k] = false;
            }
            let name = self.name_of(who);
            let line = format!("The Binders carry {name} off to their camp.");
            self.alerts.push(line.clone());
            self.say(t, line);
        }
        true
    }

    /// Captives are kept by the fire (the squad's step, like the bound).
    pub(super) fn hold_captives(&mut self) {
        for (n, c) in self.captives.clone().into_iter().enumerate() {
            let Some(k) = self.squad.index(c.who) else { continue };
            if self.fighting.contains_key(&c.who) || self.is_down(c.who) {
                continue;
            }
            let Some(spot) = self.held_spot(c.camp, n) else { continue };
            if self.squad.goal[k].dist(spot) > 2.0 {
                let (path, _) = self.route(self.member_pos(k), spot);
                self.squad.goal[k] = *path.last().unwrap_or(&spot);
                self.squad.route[k] = path;
                self.squad.resting[k] = false;
            }
        }
    }

    /// Everyone held at this camp goes free.
    pub(super) fn free_captives(&mut self, camp: GroupId, t: f64) {
        let freed: Vec<PersonId> = self.captives.iter().filter(|c| c.camp == camp).map(|c| c.who).collect();
        self.captives.retain(|c| c.camp != camp);
        for who in freed {
            let name = self.name_of(who);
            self.say(t, format!("{name} is free of the Binders."));
        }
    }

    /// At dawn: a captive may slip away; one held long enough is sold on.
    pub(super) fn captives_at_dawn(&mut self, t: f64) {
        let day = (t / DAY).floor() as u64;
        for c in self.captives.clone() {
            // The band's gone (beaten, or broken up): nobody's holding them.
            if !self.camps.iter().any(|x| x.group == c.camp) {
                self.free_captives(c.camp, t);
                continue;
            }
            let s = self.people[c.who as usize].effective_stats();
            let chance = ESCAPE_BASE + s.skill(super::stats::Skill::Sneak) * ESCAPE_SNEAK + s.attr(super::stats::Attr::Agility) * ESCAPE_AGILITY;
            let roll = Rng::from_keys(&[self.seed, c.who as u64, day, 0x4553_4350]).f32();
            if roll < chance {
                self.captives.retain(|x| x.who != c.who);
                let name = self.name_of(c.who);
                let line = format!("{name} slips away from the Binders' camp in the night.");
                self.alerts.push(line.clone());
                self.say(t, line);
                continue;
            }
            if t - c.since >= CAPTIVE_DAYS * DAY {
                let at = self.person_pos(c.who);
                let Some(town) = self.slave_market(at) else {
                    // Nowhere on the mainland buys slaves: no use keeping them.
                    self.captives.retain(|x| x.who != c.who);
                    let line = format!("With no buyer to be found, the Binders turn {} loose.", self.name_of(c.who));
                    self.alerts.push(line.clone());
                    self.say(t, line);
                    continue;
                };
                self.captives.retain(|x| x.who != c.who);
                self.sell_into_slavery(c.who, town, t);
                let name = self.name_of(c.who);
                let line = format!("The Binders sell {name} into slavery in {}.", self.settlements[town as usize].name);
                self.alerts.push(line.clone());
                self.say(t, line);
            }
        }
    }

    /// Travellers beaten by slavers on the road are taken and sold, each
    /// into a bond in the nearest town where slavery is allowed.
    pub(super) fn enslave_travellers(&mut self, b: &Battle, side: super::combat::Side, t: f64) {
        let at = b.fighters.iter().map(|f| f.pos).next().unwrap_or_default();
        let Some(town) = self.slave_market(at) else { return };
        let taken: Vec<PersonId> = b.fighters.iter().filter(|f| f.home == side && f.is_person() && !f.dead).map(|f| f.pid).filter(|&p| !self.people[p as usize].in_squad && !self.people[p as usize].dead).collect();
        for who in taken {
            if self.is_bonded(who, t) {
                continue;
            }
            self.sell_into_slavery(who, town, t);
        }
    }

    /// The nearest town where a community allows slavery. Not Salt Isle:
    /// nothing from the mainland reaches it (no road, no caravan), and its
    /// bound are its own (`island::DEBTORS`).
    pub fn slave_market(&self, at: V2) -> Option<SettlementId> {
        (0..self.settlements.len() as SettlementId)
            .filter(|&s| !self.settlements[s as usize].island)
            .filter(|&s| {
                let tl = &self.society.towns[s as usize];
                std::iter::once(tl.shore).chain(tl.stilts).any(|ci| self.society.communities.get(ci as usize).is_some_and(|c| c.customs.slavery == Slavery::Allowed))
            })
            .min_by(|&a, &b| self.settlements[a as usize].pos.dist(at).total_cmp(&self.settlements[b as usize].pos.dist(at)))
    }

    /// Bind someone in `town` and sell the bond on into slavery; where the
    /// town's law won't let it be sold, it stays a long term bond.
    fn sell_into_slavery(&mut self, who: PersonId, town: SettlementId, t: f64) {
        self.bond(who, town, None, t, t + super::law::BOND_MAX_DAYS as f64 * DAY);
        if let Some(k) = self.bonds.iter().rposition(|b| b.who == who) {
            let _ = self.sell_bond(k, t);
        }
    }
}
