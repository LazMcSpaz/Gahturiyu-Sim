//! Who acts, and what they do (Part 5, Stage 4).
//!
//! **Three layers, and a storyteller.** Everyone carries *state* — needs,
//! purse, memories, grudges — and most of it settles quietly at dawn
//! (borrowing, labouring, a grudge fading). A few people have a strong
//! enough **drive** (their top need or grudge, weighted by temper) to act.
//! Each dawn, each town's **storyteller** looks at those few and promotes
//! only some to live **storylines**: no more than the town's cap (by its
//! size and its **drama** level), favouring **prominent** people (office
//! holders, wealthy merchants, heads of big households, the ring), not too
//! many of one kind at once, and not the same kind again too soon. The rest
//! simmer, or let it go. Nothing here reads where the squad is.
//!
//! **A storyline** is planned at dawn and acted out at a set hour on the
//! world's timeline (a theft at night, a con on market hours, a grudge's
//! next rung, a ring's move), and its consequences are written there and
//! then: events, memories, arrests. One that needs outside help makes an
//! **opportunity** (`chances.rs`) and waits on it.
//!
//! **Townsfolk crime.** The worried, hungry, low-honour and bold steal (from
//! a merchant's shelf or a neighbour's purse), swindle, or stop paying a
//! debt. Who sees comes from who's about at that hour — the place's workers
//! on shift, the household awake, the watch on duty — and how light it is.
//! A watch on duty that sees (or is told) arrests, and the thief is judged
//! by custom; a squad member guarding the place catches them outright.
//! Stolen made things keep their maker's mark, so they can be traced.

use serde::{Deserialize, Serialize};

use super::chances::{Chance, OppState};
use super::history::Deed;
use super::jobs::Job;
use super::lives::{Creditor, Need, Work};
use super::memory::{Who, RUNGS};
use super::person::PersonId;
use super::ring::Move;
use super::rng::Rng;
use super::routine::Doing;
use super::settlement::SettlementId;
use super::world::{World, DAY, HOUR};

// ---- Dials -----------------------------------------------------------------

/// Where every town's drama starts (0 calm .. 1 eventful); 0.5 is "normal".
pub const DRAMA_START: f32 = 0.35;
pub const DRAMA_NORMAL: f32 = 0.5;
/// Live storylines a town can hold at normal drama: this, plus one per
/// `CAP_PEOPLE` residents.
pub const CAP_BASE: f32 = 1.5;
pub const CAP_PEOPLE: f32 = 60.0;
/// Drive needed to be a candidate at normal drama (higher when calmer).
pub const DRIVE_AT: f32 = 0.35;
pub const DRIVE_PER_DRAMA: f32 = 0.5;
/// How much prominence lifts a candidate's chance of being picked.
pub const PROMINENCE: f32 = 0.6;
/// Most live storylines of one kind in a town.
pub const VARIETY: usize = 2;
/// Days before a town has another storyline of the same kind start: theft
/// or swindle, grudge, ring, debt-dodging, each kind of asking.
pub const COOLDOWN: [i64; 5] = [2, 3, 1, 2, 2];
/// Crime: honour below, boldness above.
pub const CRIME_HONOUR: f32 = 0.45;
pub const CRIME_BOLD: f32 = 0.35;
/// A theft takes this many days of the victim household's costs in coin; a
/// con this many.
pub const THEFT_DAYS: f32 = 2.5;
pub const CON_DAYS: f32 = 2.0;
/// How likely someone about is to see a thing done: at least this, plus this
/// much more in full light.
pub const SEE_BASE: f32 = 0.1;
pub const SEE_LIGHT: f32 = 0.6;
/// The light lamps and windows give a town at night.
pub const TOWN_LAMPS: f32 = 0.2;
/// Honour from which a witness tells the watch.
pub const REPORTS_AT: f32 = 0.4;
/// Days back a theft or swindle counts toward feeling robbed.
pub const ROBBED_DAYS: f64 = 10.0;
/// Most stolen things remembered world-wide.
pub const STOLEN_CAP: usize = 400;

// ---- State -----------------------------------------------------------------

/// What a storyline is about.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Plot {
    /// Steal from a merchant's shelf (the merchant) or a household.
    Steal { merchant: Option<PersonId>, household: Option<u32> },
    /// Swindle a household out of coin.
    Con { household: u32 },
    /// Stop paying what's owed to a household.
    DodgeDebt { creditor: u32 },
    /// Take a grudge against a household one rung further.
    Grudge { other: u32 },
    /// Ask an outsider to do something.
    Ask { chance: Chance, target: Option<PersonId>, event: Option<u32> },
    /// A ring's move.
    Ring { mv: Move },
}

impl Plot {
    /// Which kind, for variety and cooldowns: theft, grudge, ring, debt,
    /// swindle, and each kind of asking for help apart.
    pub fn kind(self) -> usize {
        match self {
            Plot::Steal { .. } => 0,
            Plot::Grudge { .. } => 1,
            Plot::Ring { .. } => 2,
            Plot::DodgeDebt { .. } => 3,
            Plot::Con { .. } => 4,
            Plot::Ask { chance, .. } => 5 + chance.tag() as usize,
        }
    }

    /// Days before a town has another storyline of this kind start.
    pub fn cooldown(self) -> i64 {
        match self {
            Plot::Steal { .. } | Plot::Con { .. } => COOLDOWN[0],
            Plot::Grudge { .. } => COOLDOWN[1],
            Plot::Ring { .. } => COOLDOWN[2],
            Plot::DodgeDebt { .. } => COOLDOWN[3],
            Plot::Ask { .. } => COOLDOWN[4],
        }
    }

    pub fn words(self) -> &'static str {
        match self {
            Plot::Steal { .. } => "means to steal",
            Plot::Con { .. } => "means to swindle someone",
            Plot::DodgeDebt { .. } => "means to stop paying a debt",
            Plot::Grudge { .. } => "nurses a grudge",
            Plot::Ask { .. } => "wants help",
            Plot::Ring { .. } => "the ring is moving",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Story {
    pub id: u32,
    pub town: SettlementId,
    pub who: PersonId,
    pub plot: Plot,
    /// The hour (counted from the world's start) it's acted out.
    pub act_hour: i64,
    pub started: f64,
    /// Acted out, and waiting on this opportunity.
    pub waiting: Option<u32>,
}

/// Someone the storyteller is looking at.
struct Candidate {
    who: PersonId,
    plot: Plot,
    drive: f32,
    prominence: f32,
}

impl World {
    // ---- Measures ------------------------------------------------------------

    /// A town's drama level.
    pub fn drama(&self, town: SettlementId) -> f32 {
        self.society.towns[town as usize].drama
    }

    /// How many live storylines a town can hold.
    pub fn story_cap(&self, town: SettlementId) -> usize {
        let pop = self.settlements[town as usize].residents.len() as f32;
        ((CAP_BASE + pop / CAP_PEOPLE) * self.drama(town) / DRAMA_NORMAL).round().max(0.0) as usize
    }

    /// The drive a candidate needs.
    pub fn drive_needed(&self, town: SettlementId) -> f32 {
        DRIVE_AT + (DRAMA_NORMAL - self.drama(town)) * DRIVE_PER_DRAMA
    }

    /// How much someone counts for in their town, 0..~2.5.
    pub fn prominence(&self, p: PersonId) -> f32 {
        let mut x = 0.0;
        if self.holds_office(p) {
            x += 1.0;
        }
        let l = self.society.lives[p as usize];
        if matches!(l.job, Job::Merchant | Job::Exchanger) {
            x += 0.5;
        }
        if let Some(h) = l.household {
            let hh = &self.society.households[h as usize];
            if hh.purse.coin > self.daily_cost(h) * 15.0 {
                x += 0.3;
            }
            if hh.members.len() >= 4 && hh.members.len() <= 8 && self.eldest_of(h) == Some(p) {
                x += 0.3;
            }
        }
        for r in &self.society.rings {
            if r.leader == Some(p) {
                x += 1.0;
            } else if r.members.contains(&p) {
                x += 0.5;
            }
        }
        x
    }

    fn eldest_of(&self, h: u32) -> Option<PersonId> {
        self.society.households[h as usize].members.iter().copied().filter(|&m| !self.people[m as usize].dead).max_by_key(|&m| (self.society.lives[m as usize].age, std::cmp::Reverse(m)))
    }

    /// Live storylines in a town: planned and not yet played out. (Those
    /// waiting on an outsider are simmering, and don't count against the cap;
    /// the opportunities they wait on have their own, `chances::OPEN_CAP`.)
    pub fn stories_in(&self, town: SettlementId) -> Vec<&Story> {
        self.society.stories.iter().filter(|s| s.town == town && s.waiting.is_none()).collect()
    }

    /// Is this person in a live storyline?
    pub fn in_story(&self, p: PersonId) -> bool {
        self.society.stories.iter().any(|s| s.who == p)
    }

    // ---- The storyteller, at dawn -------------------------------------------------

    /// Who might act today: the few with a strong drive.
    fn story_candidates(&self, town: SettlementId, t: f64) -> Vec<Candidate> {
        let day = World::day_of(t);
        let need = self.drive_needed(town);
        let mut out = Vec::new();
        let folk: Vec<PersonId> = self.settlements[town as usize].residents.iter().copied().filter(|&p| self.free_to_act(p, town, t)).collect();
        // Thefts and swindles of the last days, by who suffered them.
        let mut robbed: std::collections::BTreeMap<PersonId, (u32, u32, bool)> = Default::default();
        // Hidden harm done to a household: its people want to know who.
        let mut harmed: std::collections::BTreeMap<u32, u32> = Default::default();
        for e in self.events(town).iter().filter(|e| e.t > t - ROBBED_DAYS * DAY) {
            if let (Some(v), true) = (e.victim, matches!(e.deed, Deed::Theft | Deed::Con | Deed::Extortion)) {
                let x = robbed.entry(v).or_insert((0, e.id, e.hidden));
                x.0 += 1;
                x.1 = e.id;
                x.2 = e.hidden;
            }
            if let (Some(v), true) = (e.victim, e.hidden && matches!(e.deed, Deed::Killing | Deed::Beating | Deed::Sabotage)) {
                if let Some(h) = self.society.lives.get(v as usize).and_then(|l| l.household) {
                    harmed.insert(h, e.id);
                }
            }
        }
        let has_merchant = |w: &World| folk.iter().any(|&q| w.society.lives[q as usize].job == Job::Merchant);
        let merchants: Vec<PersonId> = folk.iter().copied().filter(|&q| self.society.lives[q as usize].job == Job::Merchant).collect();
        for &p in &folk {
            let l = self.society.lives[p as usize];
            let Some(h) = l.household else { continue };
            let m = self.mind(p);
            let tr = self.people[p as usize].traits;
            let honour = l.habits.honour;
            let asks = 0.5 + l.habits.asks;
            let mut r = Rng::from_keys(&[self.seed, p as u64, day as u64, 0x4341_4E44]);
            // Crime.
            let want = m.needs[Need::Money as usize].max(m.needs[Need::Hunger as usize]);
            if honour < CRIME_HONOUR && tr.boldness >= CRIME_BOLD && want > 0.2 {
                let drive = want * (0.5 + tr.boldness) * (1.2 - honour);
                let owes = self.society.households[h as usize].purse.debts.iter().find_map(|d| match d.to {
                    Creditor::Household(o) if !d.dodged => Some(o),
                    _ => None,
                });
                let plot = if let (Some(o), true) = (owes, r.chance(0.3)) {
                    Some(Plot::DodgeDebt { creditor: o })
                } else if tr.sociability > 0.6 && r.chance(0.4) {
                    self.mark_for(h, &mut r).map(|o| Plot::Con { household: o })
                } else if has_merchant(self) && r.chance(0.5) {
                    Some(Plot::Steal { merchant: Some(merchants[r.below(merchants.len())]), household: None })
                } else {
                    self.mark_for(h, &mut r).map(|o| Plot::Steal { merchant: None, household: Some(o) })
                };
                if let Some(plot) = plot {
                    out.push(Candidate { who: p, plot, drive, prominence: 0.0 });
                }
            }
            // Asking for help.
            if let Some(&(n, ev, hidden)) = robbed.get(&p) {
                let trade = l.job.craft().is_some() || matches!(l.job, Job::Merchant | Job::Innkeeper);
                let stolen = self.society.stolen.iter().find(|s| s.event == ev && !s.recovered).map(|s| s.thief);
                let chance = if trade && (n >= 2 || m.needs[Need::Safety as usize] > 0.4) {
                    Some((Chance::Guard, None))
                } else if let Some(thief) = stolen {
                    Some((Chance::Recover, Some(thief)))
                } else if hidden {
                    Some((Chance::FindOut, None))
                } else {
                    None
                };
                if let Some((chance, target)) = chance {
                    out.push(Candidate { who: p, plot: Plot::Ask { chance, target, event: Some(ev) }, drive: (0.45 + 0.15 * n as f32) * asks, prominence: 0.0 });
                }
            }
            if let Some(&ev) = harmed.get(&h) {
                if !robbed.contains_key(&p) {
                    out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::FindOut, target: None, event: Some(ev) }, drive: 0.7 * asks, prominence: 0.0 });
                }
            }
            // A household that won't pay what it owes them.
            if let Some((debtor, amount)) = self.dodged_debt(h) {
                let cost = self.daily_cost(h).max(1.0);
                out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::CollectDebt, target: self.eldest_of(debtor), event: None }, drive: (amount / (cost * 5.0)).min(1.0) * asks, prominence: 0.0 });
            }
            // Someone who ran from the household's bond.
            if let Some(&(run, ..)) = self.society.runaways.iter().find(|x| self.society.lives.get(x.1 as usize).and_then(|l| l.household) == Some(h)) {
                out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::Runaway, target: Some(run), event: None }, drive: 0.6 * asks, prominence: 0.0 });
            }
            // A low-honour grudge that's gone far wants someone hurt.
            if honour < 0.35 {
                if let Some(f) = self.society.households[h as usize].feelings.iter().find(|f| f.stage >= 3 && f.warmth < RUNGS[2]) {
                    let target = self.eldest_of(f.other);
                    let chance = if f.stage >= 4 && honour < 0.2 { Chance::Kill } else { Chance::Intimidate };
                    out.push(Candidate { who: p, plot: Plot::Ask { chance, target, event: None }, drive: -f.warmth * (0.5 + tr.boldness * 0.5), prominence: 0.0 });
                }
            }
            // Bandits nearby.
            if let Some(c) = self.near_camp(town) {
                let d = c.1;
                let fear = m.needs[Need::Safety as usize] * 0.5 + (1.0 - d / 4000.0).max(0.0) * 0.4;
                if r.chance(0.05) {
                    out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::ClearCamp { camp: c.0, at: c.2 }, target: None, event: None }, drive: fear * asks, prominence: 0.0 });
                }
            }
            // A crafter with nothing to work on wants their goods.
            if m.idle_days >= 1 && l.job.craft().is_some() {
                if let Some((g, f)) = self.dearest(town, 1).first().copied() {
                    if f > 1.3 {
                        let item = super::items::id(g.items()[0]);
                        out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::Fetch { item, count: 3 }, target: None, event: None }, drive: (0.2 + 0.1 * m.idle_days as f32) * asks, prominence: 0.0 });
                    }
                }
            }
            // A letter to someone far away, now and then.
            if tr.sociability > 0.55 && Rng::from_keys(&[self.seed, p as u64, (day / 7) as u64, 0x4C45_5454]).chance(0.04) {
                if let Some(to) = self.kin_elsewhere(p) {
                    out.push(Candidate { who: p, plot: Plot::Ask { chance: Chance::Deliver { to }, target: None, event: None }, drive: 0.5 * asks, prominence: 0.0 });
                }
            }
        }
        // Grudges ready for their next rung.
        for (who, other, drive) in self.grudge_candidates(town, t) {
            if self.free_to_act(who, town, t) {
                out.push(Candidate { who, plot: Plot::Grudge { other }, drive, prominence: 0.0 });
            }
        }
        // The ring's move for the day.
        if let Some((who, mv, drive)) = self.ring_move(town, t) {
            out.push(Candidate { who, plot: Plot::Ring { mv }, drive, prominence: 0.0 });
        }
        out.retain(|c| c.drive >= need);
        for c in &mut out {
            c.prominence = self.prominence(c.who);
        }
        out
    }

    /// Is someone free to start something?
    fn free_to_act(&self, p: PersonId, town: SettlementId, t: f64) -> bool {
        let pp = &self.people[p as usize];
        !pp.dead && !pp.in_squad && !pp.bandit && pp.home == Some(town) && (p as usize) < self.society.minds.len() && !matches!(self.mind(p).work, Work::Bonded | Work::Injured | Work::Away) && self.busy_until[p as usize] <= t && !self.in_story(p)
    }

    /// A household worth robbing or swindling, from the thief's community.
    fn mark_for(&self, h: u32, r: &mut Rng) -> Option<u32> {
        let ci = self.society.households[h as usize].community;
        let mut rich: Vec<(u32, f32)> = (0..self.society.households.len() as u32).filter(|&o| o != h && self.society.households[o as usize].community == ci).map(|o| (o, self.society.households[o as usize].purse.coin)).filter(|x| x.1 > 20.0).collect();
        if rich.is_empty() {
            return None;
        }
        rich.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        rich.truncate(8);
        Some(rich[r.below(rich.len())].0)
    }

    /// A debt a household is owed and isn't being paid: (debtor, amount).
    pub fn dodged_debt(&self, h: u32) -> Option<(u32, f32)> {
        let ci = self.society.households[h as usize].community;
        (0..self.society.households.len() as u32).filter(|&o| self.society.households[o as usize].community == ci).find_map(|o| self.society.households[o as usize].purse.debts.iter().find(|d| d.dodged && d.to == Creditor::Household(h)).map(|d| (o, d.amount)))
    }

    /// The nearest bandit camp within reach of a town: (camp, distance, where).
    fn near_camp(&self, town: SettlementId) -> Option<(super::group::GroupId, f32, super::geo::V2)> {
        let at = self.settlements[town as usize].pos;
        self.camps.iter().map(|c| (c.group, c.pos.dist(at), c.pos)).filter(|c| c.1 < 4000.0).min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Someone in another town this person would write to (a fixed pick
    /// from their seed).
    fn kin_elsewhere(&self, p: PersonId) -> Option<PersonId> {
        let home = self.people[p as usize].home?;
        let mut r = Rng::from_keys(&[self.people[p as usize].seed, 0x4B49_4E53]);
        let others: Vec<usize> = (0..self.settlements.len()).filter(|&s| s != home as usize && !self.settlements[s].residents.is_empty()).collect();
        let s = &self.settlements[*others.get(r.below(others.len().max(1)))?];
        let to = s.residents[r.below(s.residents.len())];
        (!self.people[to as usize].dead).then_some(to)
    }

    /// Each town's storyteller picks today's storylines.
    pub(super) fn dawn_stories(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        // Stories waiting on an opportunity end when it does.
        self.society.stories.retain(|s| match s.waiting {
            Some(id) => self.society.opps.iter().any(|o| o.id == id && matches!(o.state, OppState::Open | OppState::Taken)),
            None => true,
        });
        let cap = self.story_cap(town);
        let mut cands = self.story_candidates(town, t);
        // Most driven first, prominence lifting; a keyed jitter breaks ties.
        let score = |c: &Candidate| {
            let j = Rng::from_keys(&[self.seed, c.who as u64, day as u64, 0x5354_4F52]).range(0.9, 1.1);
            c.drive * (1.0 + PROMINENCE * c.prominence) * j
        };
        let mut scored: Vec<(f32, Candidate)> = cands.drain(..).map(|c| (score(&c), c)).collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.who.cmp(&b.1.who)));
        for (_, c) in scored {
            let live: Vec<Plot> = self.stories_in(town).iter().map(|s| s.plot).collect();
            if live.len() >= cap {
                break;
            }
            let k = c.plot.kind();
            if live.iter().filter(|p| p.kind() == k).count() >= VARIETY || self.in_story(c.who) {
                continue;
            }
            let cool = &self.society.towns[town as usize].cooldowns;
            if cool.iter().any(|&(kk, d)| kk as usize == k && day - (d as i64) < c.plot.cooldown()) {
                continue;
            }
            // One ask of a kind per person at a time.
            if let Plot::Ask { chance, .. } = c.plot {
                if self.society.opps.iter().any(|o| o.asker == c.who && o.kind.tag() == chance.tag() && matches!(o.state, OppState::Open | OppState::Taken)) {
                    continue;
                }
            }
            self.start_story(town, c.who, c.plot, t);
        }
    }

    /// The hour (from the start of the world) this storyline will be acted out.
    fn act_hour(&self, who: PersonId, plot: Plot, t: f64) -> i64 {
        let dawn = (t / HOUR).round() as i64;
        let mut r = Rng::from_keys(&[self.seed, who as u64, dawn as u64, 0x4143_5448]);
        let night = 16 + r.below(5) as i64; // 22:00 to 02:00
        let day = 4 + r.below(6) as i64; // 10:00 to 15:00
        dawn + match plot {
            Plot::Steal { .. } => night,
            Plot::Con { .. } => day,
            Plot::DodgeDebt { .. } | Plot::Ask { .. } => 1,
            Plot::Grudge { .. } => {
                if self.society.lives[who as usize].habits.honour >= super::memory::OPEN_AT {
                    day
                } else {
                    night
                }
            }
            Plot::Ring { mv } => match mv {
                Move::Example { .. } => night,
                _ => 8 + r.below(6) as i64,
            },
        }
    }

    pub fn start_story(&mut self, town: SettlementId, who: PersonId, plot: Plot, t: f64) {
        let id = self.society.next_story;
        self.society.next_story += 1;
        let act_hour = self.act_hour(who, plot, t);
        self.society.stories.push(Story { id, town, who, plot, act_hour, started: t, waiting: None });
        let day = World::day_of(t) as i32;
        let cool = &mut self.society.towns[town as usize].cooldowns;
        let k = plot.kind() as u8;
        cool.retain(|c| c.0 != k);
        cool.push((k, day));
    }

    // ---- Acting them out, on the hour ----------------------------------------

    /// Storylines due this hour are acted out, in the order they were started.
    pub(super) fn story_hour(&mut self, h: i64, t: f64) {
        let due: Vec<Story> = self.society.stories.iter().filter(|s| s.act_hour == h && s.waiting.is_none()).copied().collect();
        for s in due {
            let waiting = if self.people[s.who as usize].dead || self.people[s.who as usize].in_squad { None } else { self.act_out(&s, t) };
            match waiting {
                Some(opp) => {
                    if let Some(x) = self.society.stories.iter_mut().find(|x| x.id == s.id) {
                        x.waiting = Some(opp);
                    }
                }
                None => self.society.stories.retain(|x| x.id != s.id),
            }
        }
    }

    /// Act a storyline out. Returns the opportunity it now waits on, if any.
    fn act_out(&mut self, s: &Story, t: f64) -> Option<u32> {
        match s.plot {
            Plot::Steal { merchant, household } => {
                self.steal(s.who, merchant, household, s.town, t);
                None
            }
            Plot::Con { household } => {
                self.swindle(s.who, household, s.town, t);
                None
            }
            Plot::DodgeDebt { creditor } => {
                self.dodge_debt(s.who, creditor, s.town, t);
                None
            }
            Plot::Grudge { other } => {
                if let Some(h) = self.society.lives[s.who as usize].household {
                    self.grudge_act(h, other, s.town, t);
                }
                None
            }
            Plot::Ask { chance, target, event } => self.ask_for_help(s.who, chance, target, event, s.town, t),
            Plot::Ring { mv } => self.ring_act(s.town, mv, t),
        }
    }

    // ---- Townsfolk crime -------------------------------------------------------

    /// Who's about at a place and hour and sees what's done: the place's
    /// workers on shift, the household (if awake), and the watch on duty;
    /// each sees by the light.
    fn see_it(&self, town: SettlementId, place: Option<u16>, household: Option<u32>, thief: PersonId, t: f64) -> Vec<PersonId> {
        let day = World::day_of(t);
        let h0 = (t.rem_euclid(DAY) / HOUR) as f32;
        let light = (super::stealth::daylight(t) + TOWN_LAMPS).min(1.0);
        let mut about: Vec<PersonId> = Vec::new();
        for &q in &self.settlements[town as usize].residents {
            if q == thief || self.people[q as usize].dead || self.people[q as usize].in_squad || self.busy_until[q as usize] > t {
                continue;
            }
            let l = self.society.lives[q as usize];
            let plan = self.day_plan(q, day);
            let doing = plan.segs()[plan.index_at(h0)].doing;
            let at_place = place.is_some() && l.place == place && doing == Doing::Work;
            let at_home = household.is_some() && l.household == household && !matches!(doing, Doing::Asleep);
            let on_watch = l.job == Job::Guard && doing == Doing::Work;
            if at_place || at_home || on_watch && Rng::from_keys(&[self.seed, q as u64, t.to_bits(), 0x5741_5443]).chance(0.3) {
                about.push(q);
            }
        }
        about.into_iter().filter(|&q| Rng::from_keys(&[self.seed, q as u64, thief as u64, t.to_bits(), 0x5345_4553]).chance(SEE_BASE + SEE_LIGHT * light)).collect()
    }

    /// Is this a guard on duty at `t`, by their day's plan?
    fn on_duty(&self, q: PersonId, t: f64) -> bool {
        if self.society.lives[q as usize].job != Job::Guard || self.people[q as usize].dead || self.busy_until[q as usize] > t {
            return false;
        }
        let plan = self.day_plan(q, World::day_of(t));
        plan.segs()[plan.index_at((t.rem_euclid(DAY) / HOUR) as f32)].doing == Doing::Work
    }

    /// A theft from a merchant's shelf or a household's purse.
    pub fn steal(&mut self, thief: PersonId, merchant: Option<PersonId>, household: Option<u32>, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        let (victim, place, vh) = match (merchant, household) {
            (Some(m), _) => (m, self.society.lives[m as usize].place, self.society.lives[m as usize].household),
            (None, Some(h)) => match self.eldest_of(h) {
                Some(v) => (v, None, Some(h)),
                None => return,
            },
            _ => return,
        };
        if self.people[victim as usize].dead {
            return;
        }
        // A squad member on guard there catches them outright.
        if let Some(guard) = place.and_then(|pl| self.on_watch(town, pl)) {
            let ev = self.note_seen(Deed::Caught, Some(guard), Some(thief), town, t, false, vec![victim]);
            let _ = ev;
            self.remember(victim, Who::Person(guard), Deed::Caught, 0.4, day);
            self.add_standing(guard, town, 2.0);
            let name = self.name_of(guard);
            self.say(t, format!("{name} catches {} trying to rob {}.", self.name_of(thief), self.name_of(victim)));
            self.judge_thief(thief, victim, town, t);
            return;
        }
        let seen = self.see_it(town, place, vh, thief, t);
        // What's taken: a made thing off the shelf, or coin.
        let mut item = None;
        if merchant.is_some() {
            let shelf = &mut self.society.towns[town as usize].shelf;
            if !shelf.is_empty() {
                let k = Rng::from_keys(&[self.seed, thief as u64, t.to_bits(), 0x5348_4C46]).below(shelf.len());
                item = Some(shelf.remove(k));
            }
        }
        let th = self.society.lives[thief as usize].household;
        if item.is_none() {
            if let (Some(vh), Some(th)) = (vh, th) {
                let take = (self.daily_cost(vh) * THEFT_DAYS).min(self.society.households[vh as usize].purse.coin.max(0.0));
                self.society.households[vh as usize].purse.coin -= take;
                self.society.households[th as usize].purse.coin += take;
            }
        }
        let hidden = seen.is_empty();
        let ev = self.note_seen(Deed::Theft, Some(thief), Some(victim), town, t, hidden, seen.clone());
        if let Some(sh) = item {
            self.society.stolen.push(super::chances::Stolen { item: sh.item, piece: sh.piece, from: Who::Person(victim), thief, town, event: ev, day, fenced: false, recovered: false });
            let n = self.society.stolen.len();
            if n > STOLEN_CAP {
                self.society.stolen.drain(..n - STOLEN_CAP);
            }
        }
        self.remember(victim, if hidden { Who::Someone } else { Who::Person(thief) }, Deed::Theft, -0.4, day);
        // The watch: a guard who saw, or a witness who tells one on duty.
        let watch_paid = self.society.towns[town as usize].owed <= 0.0;
        let guard = seen.iter().copied().find(|&q| self.society.lives[q as usize].job == Job::Guard);
        let told = seen.iter().any(|&q| self.society.lives[q as usize].habits.honour >= REPORTS_AT);
        let on_duty = self.settlements[town as usize].residents.iter().any(|&q| self.on_duty(q, t));
        let bribed = self.society.rings.iter().any(|r| r.town == town && r.members.contains(&thief) && !r.bribed.is_empty());
        if (guard.is_some() || (told && on_duty)) && (watch_paid || guard.is_some()) && !bribed {
            if let Some(sh) = item {
                // The goods go back.
                self.society.towns[town as usize].shelf.push(sh);
                if let Some(s) = self.society.stolen.iter_mut().find(|s| s.event == ev) {
                    s.recovered = true;
                }
            }
            let by = guard.or_else(|| self.settlements[town as usize].residents.iter().copied().find(|&q| self.on_duty(q, t)));
            self.note(Deed::Arrest, by, Some(thief), town, t, false);
            self.judge_thief(thief, victim, town, t);
        }
    }

    /// A thief caught: judged by the wronged party's custom, and remembered.
    fn judge_thief(&mut self, thief: PersonId, victim: PersonId, town: SettlementId, t: f64) {
        let day = World::day_of(t);
        let mut r = Rng::from_keys(&[self.seed, thief as u64, victim as u64, day as u64, 0x4A55_4447]);
        self.public_dispute(thief, victim, town, t, &mut r);
        if let Some(ring) = self.society.rings.iter_mut().find(|x| x.town == town && x.members.contains(&thief)) {
            ring.heat += super::ring::HEAT_ARREST;
        }
        self.remember(thief, Who::Person(victim), Deed::Arrest, -0.3, day as i32);
    }

    /// A swindle: coin talked out of a household. They find out later who.
    fn swindle(&mut self, who: PersonId, household: u32, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        let Some(th) = self.society.lives[who as usize].household else { return };
        let Some(victim) = self.eldest_of(household) else { return };
        let take = (self.daily_cost(household) * CON_DAYS).min(self.society.households[household as usize].purse.coin.max(0.0));
        self.society.households[household as usize].purse.coin -= take;
        self.society.households[th as usize].purse.coin += take;
        let mut r = Rng::from_keys(&[self.seed, who as u64, household as u64, day as u64, 0x434F_4E53]);
        let knows = r.chance(0.5);
        self.note(Deed::Con, Some(who), Some(victim), town, t, !knows);
        self.remember(victim, if knows { Who::Person(who) } else { Who::Someone }, Deed::Con, -0.35, day);
        // The honourable take it to a hearing.
        if knows && self.society.lives[victim as usize].habits.honour > 0.6 {
            self.public_dispute(who, victim, town, t, &mut r);
        }
    }

    /// A household stops paying what it owes another.
    fn dodge_debt(&mut self, who: PersonId, creditor: u32, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        let Some(h) = self.society.lives[who as usize].household else { return };
        let mut any = false;
        for d in &mut self.society.households[h as usize].purse.debts {
            if d.to == Creditor::Household(creditor) {
                d.dodged = true;
                any = true;
            }
        }
        if !any {
            return;
        }
        if let Some(c) = self.eldest_of(creditor) {
            self.note(Deed::DebtDodge, Some(who), Some(c), town, t, false);
            self.remember(c, Who::Person(who), Deed::DebtDodge, -0.4, day);
        }
    }

    /// Someone asks for help: an opportunity, if they can pay or owe.
    fn ask_for_help(&mut self, who: PersonId, chance: Chance, target: Option<PersonId>, event: Option<u32>, town: SettlementId, t: f64) -> Option<u32> {
        let h = self.society.lives[who as usize].household?;
        let cost = self.daily_cost(h).max(1.0);
        let mut o = self.opp(chance, who, town);
        o.target = target;
        o.event = event;
        let prosperity = 0.5 + 0.5 * self.society.towns[town as usize].prosperity.min(1.2);
        let want = match chance {
            Chance::Guard => {
                o.place = self.society.lives[who as usize].place;
                o.place?;
                o.amount = (Job::Guard.pay() * 9.0 * prosperity).round();
                o.amount * super::chances::GUARD_DAYS as f32
            }
            Chance::Recover => {
                let s = self.society.stolen.iter().find(|s| Some(s.event) == event && !s.recovered)?;
                o.item = Some(s.item);
                o.target = Some(s.thief);
                (super::items::item(s.item).value * super::chances::REWARD_SHARE).max(super::chances::REWARD_MIN as f32)
            }
            Chance::FindOut => cost * 3.0,
            Chance::CollectDebt => {
                let (_, amount) = self.dodged_debt(h)?;
                o.amount = amount;
                amount * super::chances::REWARD_SHARE
            }
            Chance::Intimidate => {
                o.open = false;
                cost * super::chances::INTIMIDATE_DAYS
            }
            Chance::Kill => {
                o.open = false;
                cost * super::chances::KILL_DAYS
            }
            Chance::Runaway => {
                let (_, _, _, days) = *self.society.runaways.iter().find(|x| Some(x.0) == target)?;
                o.amount = days;
                cost * 3.0
            }
            Chance::ClearCamp { .. } => 120.0 + cost * 10.0,
            Chance::Fetch { item, count } => super::items::item(item).value * count as f32 * 2.5 + 10.0,
            Chance::Deliver { to } => {
                let far = self.settlements[self.people[to as usize].home? as usize].pos.dist(self.settlements[town as usize].pos);
                20.0 + far / 150.0
            }
            Chance::Escort | Chance::Champion => return None,
        };
        let (reward, favour) = self.can_reward(who, want)?;
        o.reward = reward;
        o.favour = favour;
        self.post_opp(o, t)
    }
}
