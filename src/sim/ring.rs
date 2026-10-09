//! The criminal ring (Part 5, Stage 4).
//!
//! A larger town (`RING_POP` people and up) has one ring: real people — the
//! bold, the low in honour, the desperate it takes in — and a leader. Once a
//! day it reads its town and picks a move:
//!
//! - **make an example** of an outside thief when too many thefts on its turf
//!   aren't its own (a beating, turning them in to the watch to keep the
//!   watch sweet, or a quiet killing paid for — an opportunity);
//! - **lie low** when the watch is too interested;
//! - **bribe** a guard when the treasury isn't paying them;
//! - **recruit** someone desperate;
//! - **lean on** a merchant for money;
//! - **fence** stolen things (their makers' marks lead back to the ring).
//!
//! The move is a storyline like anyone's, so the storyteller decides whether
//! it plays out today, and it writes events and memories like anything else.
//! Members and the leader can be found out, killed or replaced (a dead
//! leader's place goes to the boldest member).

use serde::{Deserialize, Serialize};

use super::chances::{Chance, Giver};
use super::history::Deed;
use super::jobs::Job;
use super::lives::{Need, Work};
use super::memory::Who;
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::{World, DAY};

// ---- Dials -----------------------------------------------------------------

/// Towns this big have a ring.
pub const RING_POP: usize = 150;
/// Members it starts with, and the most it takes.
pub const RING_START: usize = 3;
pub const RING_MAX: usize = 8;
/// Honour below which someone would join.
pub const RING_HONOUR: f32 = 0.4;
/// Days back outside thefts count, and how many before it acts.
pub const FREELANCE_DAYS: f64 = 7.0;
pub const FREELANCE_LIMIT: usize = 3;
/// Heat: added by an arrest of a member or a leaning, cooling each day by
/// this factor; above `HEAT_HIGH` it lies low.
pub const HEAT_ARREST: f32 = 1.0;
pub const HEAT_LEAN: f32 = 0.3;
pub const HEAT_COOL: f32 = 0.85;
pub const HEAT_HIGH: f32 = 2.5;
/// A merchant leaned on pays this many days of their household's costs.
pub const EXTORT_DAYS: f32 = 2.0;
/// A guard's bribe, coin.
pub const BRIBE: f32 = 20.0;
/// What fencing a stolen thing brings, as a share of its worth (to the ring,
/// and to the thief).
pub const FENCE_SHARE: f32 = 0.3;
/// A killing the ring pays for, coin.
pub const HIRE: f32 = 60.0;
/// Drive of the ring's moves (an example is pressing; the rest routine).
pub const EXAMPLE_DRIVE: f32 = 0.95;
pub const MOVE_DRIVE: f32 = 0.55;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Ring {
    pub town: SettlementId,
    pub leader: Option<PersonId>,
    pub members: Vec<PersonId>,
    pub purse: f32,
    pub heat: f32,
    /// Guards in its pay.
    pub bribed: Vec<PersonId>,
    /// Households paying it.
    pub paying: Vec<u32>,
    /// The squad knows who leads it.
    pub found: bool,
    /// Outside thefts before this count no more (after an example).
    pub reckoned: f64,
    pub last: Option<(Move, i32)>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Example {
    Beat,
    TurnIn,
    Hire,
}

/// A ring's move for the day.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Example { thief: PersonId, how: Example },
    LieLow,
    Bribe { guard: PersonId },
    Recruit { who: PersonId },
    Extort { household: u32 },
    Fence,
}

impl Move {
    pub fn words(self) -> &'static str {
        match self {
            Move::Example { how: Example::Beat, .. } => "has a thief beaten",
            Move::Example { how: Example::TurnIn, .. } => "turns a thief in",
            Move::Example { how: Example::Hire, .. } => "wants a thief dead",
            Move::LieLow => "lies low",
            Move::Bribe { .. } => "bribes a guard",
            Move::Recruit { .. } => "takes someone in",
            Move::Extort { .. } => "leans on a merchant",
            Move::Fence => "fences stolen goods",
        }
    }
}

impl World {
    pub fn ring(&self, town: SettlementId) -> Option<&Ring> {
        self.society.rings.iter().find(|r| r.town == town)
    }

    fn ring_mut(&mut self, town: SettlementId) -> Option<&mut Ring> {
        self.society.rings.iter_mut().find(|r| r.town == town)
    }

    /// Would this person join a ring?
    fn ring_material(&self, p: PersonId, town: SettlementId) -> bool {
        let pp = &self.people[p as usize];
        !pp.dead && !pp.in_squad && !pp.bandit && pp.home == Some(town) && (p as usize) < self.society.minds.len() && self.society.lives[p as usize].habits.honour < RING_HONOUR && !self.holds_office(p) && self.society.lives[p as usize].job != Job::Guard
    }

    /// At dawn: a big enough town gets its ring; the dead and gone leave it;
    /// a lost leader is replaced; the heat cools.
    pub(super) fn dawn_ring(&mut self, town: SettlementId, t: f64) {
        let pop = self.settlements[town as usize].residents.len();
        if self.ring(town).is_none() && pop >= RING_POP {
            let mut folk: Vec<PersonId> = self.settlements[town as usize].residents.iter().copied().filter(|&p| self.ring_material(p, town) && self.people[p as usize].traits.boldness > 0.5).collect();
            let rank = |w: &World, p: PersonId| w.people[p as usize].traits.boldness + 1.0 - w.society.lives[p as usize].habits.honour;
            folk.sort_by(|&a, &b| rank(self, b).total_cmp(&rank(self, a)).then(a.cmp(&b)));
            folk.truncate(RING_START);
            if !folk.is_empty() {
                self.society.rings.push(Ring { town, leader: Some(folk[0]), members: folk, purse: 30.0, heat: 0.0, bribed: Vec::new(), paying: Vec::new(), found: false, reckoned: t, last: None });
            }
        }
        let gone: Vec<PersonId> = self.ring(town).map(|r| r.members.iter().copied().filter(|&m| self.people[m as usize].dead || self.people[m as usize].in_squad || self.people[m as usize].home != Some(town)).collect()).unwrap_or_default();
        let boldest = |w: &World, ms: &[PersonId]| ms.iter().copied().max_by(|&a, &b| w.people[a as usize].traits.boldness.total_cmp(&w.people[b as usize].traits.boldness).then(b.cmp(&a)));
        let dead_guards: Vec<PersonId> = self.ring(town).map(|r| r.bribed.iter().copied().filter(|&g| self.people[g as usize].dead || self.society.lives[g as usize].job != Job::Guard).collect()).unwrap_or_default();
        let Some(r) = self.ring(town).cloned() else { return };
        let mut members = r.members.clone();
        members.retain(|m| !gone.contains(m));
        let leader = match r.leader {
            Some(l) if members.contains(&l) => Some(l),
            _ => boldest(self, &members),
        };
        let r = self.ring_mut(town).unwrap();
        r.members = members;
        r.leader = leader;
        r.heat *= HEAT_COOL;
        r.bribed.retain(|g| !dead_guards.contains(g));
    }

    /// Outside thefts on the ring's turf since it last reckoned with them,
    /// by thief (the ring hears of them all, seen or not).
    pub fn freelance_thefts(&self, town: SettlementId, t: f64) -> Vec<(PersonId, usize)> {
        let Some(r) = self.ring(town) else { return Vec::new() };
        let from = (t - FREELANCE_DAYS * DAY).max(r.reckoned);
        let mut n: std::collections::BTreeMap<PersonId, usize> = Default::default();
        for e in self.events(town).iter().filter(|e| e.t > from && e.t <= t && matches!(e.deed, Deed::Theft | Deed::Con)) {
            if let Some(a) = e.actor.filter(|a| !r.members.contains(a)) {
                *n.entry(a).or_default() += 1;
            }
        }
        n.into_iter().collect()
    }

    /// The ring's move for the day, if any: (its leader, the move, its drive).
    pub(super) fn ring_move(&self, town: SettlementId, t: f64) -> Option<(PersonId, Move, f32)> {
        let r = self.ring(town)?;
        let leader = r.leader?;
        if self.people[leader as usize].dead || self.in_story(leader) {
            return None;
        }
        let day = World::day_of(t);
        let mut rng = Rng::from_keys(&[self.seed, town as u64, day as u64, 0x5249_4E47]);
        let lt = self.people[leader as usize].traits;
        let honour = self.society.lives[leader as usize].habits.honour;
        // Too many thieves not its own: make an example of the busiest.
        let free = self.freelance_thefts(town, t);
        if free.iter().map(|x| x.1).sum::<usize>() >= FREELANCE_LIMIT {
            let thief = free.iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))).unwrap().0;
            if !self.people[thief as usize].dead {
                let w = [lt.boldness, honour + r.heat * 0.2, if r.purse >= HIRE { (1.0 - honour) * lt.boldness } else { 0.0 }];
                let how = [Example::Beat, Example::TurnIn, Example::Hire][rng.weighted(&w).unwrap_or(0)];
                return Some((leader, Move::Example { thief, how }, EXAMPLE_DRIVE));
            }
        }
        if r.heat > HEAT_HIGH {
            return Some((leader, Move::LieLow, MOVE_DRIVE));
        }
        let folk = &self.settlements[town as usize].residents;
        let tl = &self.society.towns[town as usize];
        let guard = folk.iter().copied().find(|&g| self.society.lives[g as usize].job == Job::Guard && !self.people[g as usize].dead && !r.bribed.contains(&g));
        let desperate = folk.iter().copied().find(|&p| !r.members.contains(&p) && self.ring_material(p, town) && !self.in_story(p) && (self.mind(p).work == Work::Jobless || self.mind(p).needs()[Need::Money as usize] > 0.5));
        let own: Vec<u32> = r.members.iter().filter_map(|&m| self.society.lives[m as usize].household).collect();
        let merchant = folk.iter().copied().filter(|&p| self.society.lives[p as usize].job == Job::Merchant && !self.people[p as usize].dead).filter_map(|p| self.society.lives[p as usize].household).find(|h| !r.paying.contains(h) && !own.contains(h));
        let loot = self.society.stolen.iter().any(|s| s.town == town && !s.fenced && !s.recovered);
        let moves = [
            (guard.filter(|_| tl.owed > 0.0 && r.purse >= BRIBE).map(|g| Move::Bribe { guard: g }), 2.0),
            (desperate.filter(|_| r.members.len() < RING_MAX).map(|p| Move::Recruit { who: p }), 0.5),
            (merchant.map(|h| Move::Extort { household: h }), 1.5 * lt.boldness),
            (loot.then_some(Move::Fence), 1.0),
            (None, 1.5),
        ];
        // (The last is doing nothing today.)
        let w: Vec<f32> = moves.iter().enumerate().map(|(i, m)| if i == moves.len() - 1 || m.0.is_some() { m.1 } else { 0.0 }).collect();
        let k = rng.weighted(&w)?;
        moves[k].0.map(|mv| (leader, mv, MOVE_DRIVE))
    }

    /// The ring plays its move out. Returns an opportunity it now waits on.
    pub(super) fn ring_act(&mut self, town: SettlementId, mv: Move, t: f64) -> Option<u32> {
        let day = World::day_of(t) as i32;
        let r = self.ring(town)?.clone();
        let leader = r.leader?;
        let mut rng = Rng::from_keys(&[self.seed, town as u64, day as u64, 0x5241_4354]);
        let mut waits = None;
        match mv {
            Move::Example { thief, how } => {
                match how {
                    Example::Beat => {
                        let hand = r.members.iter().copied().filter(|&m| m != leader && !self.people[m as usize].dead).nth(0).unwrap_or(leader);
                        let pp = &mut self.people[thief as usize];
                        let base = pp.stats.clone();
                        let mut hp = pp.wounds.hp_at(&base, t);
                        for (k, x) in hp.iter_mut().enumerate() {
                            *x = x.min(base.max_hp(super::body::PARTS[k]) * super::memory::BEATEN_TO);
                        }
                        pp.wounds.set(&base, &hp, t);
                        let seen = rng.chance(super::memory::FOUND_OUT);
                        self.note(Deed::Beating, Some(hand), Some(thief), town, t, !seen);
                        self.remember(thief, Who::Ring(town), Deed::Beating, -0.6, day);
                    }
                    Example::TurnIn => {
                        self.note(Deed::TurnedIn, Some(leader), Some(thief), town, t, true);
                        let victim = self.events(town).iter().rev().find(|e| e.actor == Some(thief) && e.deed == Deed::Theft).and_then(|e| e.victim);
                        let guard = self.settlements[town as usize].residents.iter().copied().find(|&g| self.society.lives[g as usize].job == Job::Guard && !self.people[g as usize].dead);
                        self.note(Deed::Arrest, guard, Some(thief), town, t, false);
                        if let Some(v) = victim {
                            let mut jr = Rng::from_keys(&[self.seed, thief as u64, day as u64, 0x5455_524E]);
                            self.public_dispute(thief, v, town, t, &mut jr);
                        }
                        if let Some(x) = self.ring_mut(town) {
                            x.heat = (x.heat - 1.0).max(0.0);
                        }
                    }
                    Example::Hire => {
                        let mut o = self.opp(Chance::Kill, leader, town);
                        o.giver = Giver::Ring(town);
                        o.target = Some(thief);
                        o.open = false;
                        o.reward = HIRE.min(r.purse) as u16;
                        waits = self.post_opp(o, t);
                    }
                }
                if let Some(x) = self.ring_mut(town) {
                    x.reckoned = t;
                }
            }
            Move::LieLow => {
                if let Some(x) = self.ring_mut(town) {
                    x.heat *= 0.5;
                }
            }
            Move::Bribe { guard } => {
                if let Some(gh) = self.society.lives[guard as usize].household {
                    self.society.households[gh as usize].purse.coin += BRIBE;
                }
                self.note(Deed::Bribe, Some(leader), Some(guard), town, t, true);
                let x = self.ring_mut(town)?;
                x.purse -= BRIBE;
                x.bribed.push(guard);
            }
            Move::Recruit { who } => {
                self.note(Deed::Recruited, Some(leader), Some(who), town, t, false);
                self.remember(who, Who::Ring(town), Deed::Recruited, 0.3, day);
                self.ring_mut(town)?.members.push(who);
            }
            Move::Extort { household } => {
                let take = (self.daily_cost(household) * EXTORT_DAYS).min(self.society.households[household as usize].purse.coin.max(0.0));
                self.society.households[household as usize].purse.coin -= take;
                let victim = self.society.households[household as usize].members.iter().copied().find(|&m| self.society.lives[m as usize].job == Job::Merchant).or_else(|| self.society.households[household as usize].members.first().copied());
                self.note(Deed::Extortion, Some(leader), victim, town, t, false);
                if let Some(v) = victim {
                    self.remember(v, Who::Ring(town), Deed::Extortion, -0.4, day);
                }
                let x = self.ring_mut(town)?;
                x.purse += take;
                x.heat += HEAT_LEAN;
                x.paying.push(household);
            }
            Move::Fence => {
                let free: Vec<PersonId> = self.freelance_thefts(town, t).into_iter().map(|x| x.0).collect();
                let mut got = 0.0;
                for k in 0..self.society.stolen.len() {
                    let s = self.society.stolen[k];
                    if s.town != town || s.fenced || s.recovered || !(r.members.contains(&s.thief) || free.contains(&s.thief)) {
                        continue;
                    }
                    let worth = super::items::item(s.item).value * FENCE_SHARE;
                    self.society.stolen[k].fenced = true;
                    got += worth;
                    if let Some(th) = self.society.lives[s.thief as usize].household {
                        self.society.households[th as usize].purse.coin += worth;
                    }
                    self.note(Deed::Fenced, Some(leader), Some(s.thief), town, t, true);
                }
                self.ring_mut(town)?.purse += got;
            }
        }
        if let Some(x) = self.ring_mut(town) {
            x.last = Some((mv, day));
        }
        waits
    }
}
