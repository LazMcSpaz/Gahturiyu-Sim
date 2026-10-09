//! The world's history and its gossip (Part 5, Stage 3).
//!
//! **Events.** Each town keeps its own record of what happened there — who
//! did what to whom, when, who saw, how serious — separate from the screen
//! log: the last `ORDINARY_KEPT` ordinary events, and serious ones
//! (`SERIOUS` and up) for longer, the last `SERIOUS_KEPT`. Events are
//! numbered once, world-wide.
//!
//! **Who knows what.** Each person knows a few events (`KNOWS_CAP`), the
//! most interesting kept. Those involved and those who saw know at once.
//! Everyone else hears by **gossip**: each dawn, for the evening before,
//! people who were at the same evening spot (inn, hearth, deck, mess hall,
//! a neighbour's) pass on the most interesting thing they know to a few
//! others there — sociable people more often. News goes between towns
//! with **travellers**: a journey or caravan setting out carries its
//! town's notable news, told to a few people where it arrives (delivered at
//! the first dawn after it gets there).
//!
//! **What it changes.** Hearing of a wrong sours the hearer on whoever did
//! it (if anyone knows who), and wrongs of violence or theft at home make
//! people fear for their safety.
//!
//! All of it is settled at dawn or at a journey's departure, on the world's
//! timeline, with rolls keyed to who and what.

use serde::{Deserialize, Serialize};

use super::group::{GroupId, Kind};
use super::memory::{Venues, Who, EVENING, VISIT};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::{World, DAY, HOUR};

// ---- Dials -----------------------------------------------------------------

pub const ORDINARY_KEPT: usize = 200;
pub const SERIOUS_KEPT: usize = 60;
/// Severity from which an event is kept longer.
pub const SERIOUS: f32 = 0.5;
/// Most events a person knows of.
pub const KNOWS_CAP: usize = 8;
/// Most people who see something done in the open.
pub const WITNESSES: usize = 4;
/// Chance someone at an evening spot passes something on (× 0.4 + sociability),
/// and to how many.
pub const GOSSIP_CHANCE: f32 = 0.35;
pub const GOSSIP_REACH: usize = 3;
/// Days over which news grows stale (interest falls to about a third).
pub const NEWS_FADE: f32 = 8.0;
/// News a journey carries per leg, the chance each is told, how serious it
/// must be to travel, and to how many people it's told on arrival.
pub const TIDINGS_PER_LEG: usize = 2;
pub const TIDING_CHANCE: f32 = 0.4;
pub const TIDING_SEVERITY: f32 = 0.3;
pub const TOLD: usize = 3;
/// Most news on the roads at once.
pub const TIDINGS_CAP: usize = 4000;
/// How much hearing of a wrong sours the hearer on its doer (× severity).
pub const OPINION: f32 = 0.2;
/// How much known violence and theft at home weighs on safety (× severity).
pub const DANGER: f32 = 0.8;

/// What was done.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Deed {
    // Everyday dealings.
    Kindness,
    Loan,
    Slight,
    WorkQuarrel,
    DebtQuarrel,
    TradeDispute,
    // A grudge's ladder.
    Avoid,
    PublicDispute,
    Slander,
    Claim,
    Duel,
    Shun,
    Record,
    Sabotage,
    Theft,
    Beating,
    Brawl,
    Feud,
}

/// An event someone knows of.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Known {
    pub id: u32,
    pub town: SettlementId,
}

/// News on the road: told in `town` at the first dawn after `at`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Tiding {
    pub at: f64,
    pub town: SettlementId,
    pub news: Known,
}

impl Deed {
    /// Is it a wrong done to someone?
    pub fn is_wrong(self) -> bool {
        !matches!(self, Deed::Kindness | Deed::Loan | Deed::Avoid)
    }

    /// Does it make the town feel less safe?
    pub fn is_danger(self) -> bool {
        matches!(self, Deed::Theft | Deed::Sabotage | Deed::Beating | Deed::Brawl | Deed::Feud)
    }

    /// How many see it done.
    pub fn seen_by(self) -> usize {
        match self {
            Deed::Kindness | Deed::Loan | Deed::Avoid => 0,
            Deed::Slander | Deed::Sabotage | Deed::Theft | Deed::Beating => 1,
            _ => WITNESSES,
        }
    }

    /// How serious it is, 0..1.
    pub fn severity(self) -> f32 {
        match self {
            Deed::Kindness | Deed::Loan => 0.05,
            Deed::Slight | Deed::WorkQuarrel | Deed::DebtQuarrel | Deed::TradeDispute | Deed::Avoid => 0.1,
            Deed::PublicDispute | Deed::Slander | Deed::Shun | Deed::Record => 0.3,
            Deed::Claim | Deed::Sabotage => 0.4,
            Deed::Theft | Deed::Duel => 0.5,
            Deed::Brawl | Deed::Beating => 0.7,
            Deed::Feud => 0.8,
        }
    }

    /// Words for it: "{actor} {words} {victim}".
    pub fn words(self) -> &'static str {
        match self {
            Deed::Kindness => "did a kindness to",
            Deed::Loan => "lent money to",
            Deed::Slight => "slighted",
            Deed::WorkQuarrel => "quarrelled over work with",
            Deed::DebtQuarrel => "quarrelled over a debt with",
            Deed::TradeDispute => "fell out over a sale with",
            Deed::Avoid => "turned their back on",
            Deed::PublicDispute => "took a dispute to the hall against",
            Deed::Slander => "spread talk about",
            Deed::Claim => "pressed a claim on the labour of",
            Deed::Duel => "fought a duel with",
            Deed::Shun => "had their people shun",
            Deed::Record => "had a wrong written into the record against",
            Deed::Sabotage => "sabotaged the work of",
            Deed::Theft => "stole from",
            Deed::Beating => "had a beating given to",
            Deed::Brawl => "came to blows in the open with",
            Deed::Feud => "is in a feud with",
        }
    }
}

/// Something that happened.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    pub id: u32,
    pub deed: Deed,
    pub actor: Option<PersonId>,
    pub victim: Option<PersonId>,
    pub town: SettlementId,
    pub t: f64,
    pub severity: f32,
    /// Done in secret, and not (yet) found out: no one knows who.
    pub hidden: bool,
    /// Who saw it (besides those involved).
    pub witnesses: Vec<PersonId>,
}

/// Every town's record.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct History {
    pub next: u32,
    pub towns: Vec<Vec<Event>>,
    /// News on the roads.
    pub tidings: Vec<Tiding>,
}

impl World {
    /// Write an event into a town's history, seen by whoever was about (those
    /// the doer or the done-to work or live with). Returns its number.
    pub fn note(&mut self, deed: Deed, actor: Option<PersonId>, victim: Option<PersonId>, town: SettlementId, t: f64, hidden: bool) -> u32 {
        let n = if hidden { 0 } else { deed.seen_by() };
        let mut seen = Vec::new();
        if n > 0 {
            let mut about: Vec<PersonId> = Vec::new();
            for x in [actor, victim].into_iter().flatten() {
                if let Some(l) = self.society.lives.get(x as usize) {
                    if let Some(h) = l.household {
                        about.extend(self.society.households[h as usize].members.iter().copied());
                    }
                    if let Some(pl) = l.place {
                        about.extend(self.settlements[town as usize].residents.iter().copied().filter(|&q| self.society.lives[q as usize].place == Some(pl)));
                    }
                }
            }
            about.retain(|&q| Some(q) != actor && Some(q) != victim && !self.people[q as usize].dead && !self.people[q as usize].in_squad);
            about.sort_unstable();
            about.dedup();
            let mut r = Rng::from_keys(&[self.seed, actor.unwrap_or(u32::MAX) as u64, victim.unwrap_or(u32::MAX) as u64, t.to_bits(), 0x5345_454E]);
            while seen.len() < n && !about.is_empty() {
                seen.push(about.swap_remove(r.below(about.len())));
            }
        }
        self.note_seen(deed, actor, victim, town, t, hidden, seen)
    }

    /// Write an event seen by these witnesses. Those involved and those who
    /// saw know of it from now.
    pub fn note_seen(&mut self, deed: Deed, actor: Option<PersonId>, victim: Option<PersonId>, town: SettlementId, t: f64, hidden: bool, witnesses: Vec<PersonId>) -> u32 {
        let h = &mut self.society.history;
        if h.towns.len() <= town as usize {
            h.towns.resize(town as usize + 1, Vec::new());
        }
        let id = h.next;
        h.next += 1;
        let ev = &mut h.towns[town as usize];
        ev.push(Event { id, deed, actor, victim, town, t, severity: deed.severity(), hidden, witnesses: witnesses.clone() });
        let ordinary = ev.iter().filter(|e| e.severity < SERIOUS).count();
        if ordinary > ORDINARY_KEPT {
            let k = ev.iter().position(|e| e.severity < SERIOUS).unwrap();
            ev.remove(k);
        }
        if ev.len() - ev.iter().filter(|e| e.severity < SERIOUS).count() > SERIOUS_KEPT {
            let k = ev.iter().position(|e| e.severity >= SERIOUS).unwrap();
            ev.remove(k);
        }
        let news = Known { id, town };
        let day = World::day_of(t) as i32;
        // The doer knows; so does the one it was done to.
        for p in actor.into_iter().chain(victim).chain(witnesses) {
            self.learn(p, news, day);
        }
        id
    }

    /// A town's events, oldest first.
    pub fn events(&self, town: SettlementId) -> &[Event] {
        self.society.history.towns.get(town as usize).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// An event someone knows of, if it's still on record.
    pub fn known_event(&self, k: Known) -> Option<&Event> {
        let ev = self.events(k.town);
        ev.binary_search_by_key(&k.id, |e| e.id).ok().map(|i| &ev[i])
    }

    /// An event by number, if it's still on record.
    pub fn event(&self, id: u32) -> Option<&Event> {
        (0..self.society.history.towns.len() as SettlementId).find_map(|town| self.known_event(Known { id, town }))
    }

    /// Who knows of an event (by asking everyone; for tests and the debug view).
    pub fn knowers(&self, id: u32) -> Vec<PersonId> {
        (0..self.society.minds.len() as PersonId).filter(|&p| self.mind(p).knows.iter().any(|k| k.id == id)).collect()
    }

    /// How interesting a piece of news is on `day`: serious and fresh.
    pub fn interest(&self, k: Known, day: i32) -> f32 {
        self.known_event(k).map(|e| e.severity * (-((day as f64 - e.t / DAY).max(0.0) as f32) / NEWS_FADE).exp()).unwrap_or(0.0)
    }

    /// Someone learns of an event. Hearing of a wrong sours them on whoever
    /// did it. Returns whether it was news to them.
    pub fn learn(&mut self, p: PersonId, k: Known, day: i32) -> bool {
        if p as usize >= self.society.minds.len() || self.people[p as usize].dead || self.mind(p).knows.iter().any(|x| x.id == k.id) {
            return false;
        }
        let Some(e) = self.known_event(k) else { return false };
        let (deed, actor, victim, hidden, sev) = (e.deed, e.actor, e.victim, e.hidden, e.severity);
        let mut knows = std::mem::take(&mut self.society.minds[p as usize].knows);
        knows.retain(|x| self.known_event(*x).is_some());
        knows.push(k);
        if knows.len() > KNOWS_CAP {
            let i = (0..knows.len()).min_by(|&a, &b| self.interest(knows[a], day).total_cmp(&self.interest(knows[b], day)).then(knows[a].id.cmp(&knows[b].id))).unwrap();
            knows.remove(i);
        }
        self.society.minds[p as usize].knows = knows;
        if let (Some(a), false) = (actor, hidden) {
            let kin = |x: PersonId| self.society.lives.get(x as usize).and_then(|l| l.household);
            if deed.is_wrong() && a != p && Some(p) != victim && (kin(p).is_none() || kin(p) != kin(a)) {
                // Kin of the wronged take it as their own.
                let own = victim.is_some_and(|v| kin(v).is_some() && kin(v) == kin(p));
                self.remember_as(p, Who::Person(a), deed, -sev * OPINION, day, !own);
            }
        }
        true
    }

    /// How much danger someone knows of at home, 0..1: violence and theft
    /// they've heard of, the fresher the worse.
    pub fn known_danger(&self, p: PersonId, day: i32) -> f32 {
        let Some(home) = self.people[p as usize].home else { return 0.0 };
        let d: f32 = self.mind(p).knows.iter().filter(|k| k.town == home).filter(|k| self.known_event(**k).is_some_and(|e| e.deed.is_danger())).map(|&k| self.interest(k, day)).sum();
        (d * DANGER).min(1.0)
    }

    // ---- At dawn -------------------------------------------------------------

    /// Last evening's talk, among those who were at the same evening spot.
    pub(super) fn gossip(&mut self, v: &Venues, t: f64) {
        let day = World::day_of(t) as i32;
        // Only what had happened by the evening.
        let by = t - 12.0 * HOUR;
        let mut told: Vec<(PersonId, Known)> = Vec::new();
        for ((kind, _), there) in &v.at {
            if !(*kind == EVENING || *kind == VISIT) || there.len() < 2 {
                continue;
            }
            for &p in there {
                let soc = self.people[p as usize].traits.sociability;
                let mut r = Rng::from_keys(&[self.seed, p as u64, day as u64, 0x474F_5353]);
                if !r.chance(GOSSIP_CHANCE * (0.4 + soc)) {
                    continue;
                }
                let best = self.mind(p).knows.iter().copied().filter(|&k| self.known_event(k).is_some_and(|e| e.t <= by)).max_by(|a, b| self.interest(*a, day).total_cmp(&self.interest(*b, day)).then(b.id.cmp(&a.id)));
                let Some(k) = best else { continue };
                for _ in 0..GOSSIP_REACH {
                    let q = there[r.below(there.len())];
                    if q != p {
                        told.push((q, k));
                    }
                }
            }
        }
        for (q, k) in told {
            self.learn(q, k, day);
        }
    }

    /// News that has reached a town is told to a few of its people.
    pub(super) fn hear_tidings(&mut self, town: SettlementId, t: f64) {
        let day = World::day_of(t) as i32;
        let (here, rest): (Vec<Tiding>, Vec<Tiding>) = self.society.history.tidings.iter().partition(|x| x.town == town && x.at <= t);
        if here.is_empty() {
            return;
        }
        self.society.history.tidings = rest;
        let folk: Vec<PersonId> = self.settlements[town as usize].residents.iter().copied().filter(|&p| !self.people[p as usize].dead && !self.people[p as usize].in_squad && self.people[p as usize].home == Some(town)).collect();
        if folk.is_empty() {
            return;
        }
        for x in here {
            let mut r = Rng::from_keys(&[self.seed, x.news.id as u64, town as u64, 0x544F_4C44]);
            for _ in 0..TOLD {
                let p = folk[r.below(folk.len())];
                self.learn(p, x.news, day);
            }
        }
    }

    /// A group has just set out: each leg carries the notable news of the
    /// town it leaves from to the town it reaches.
    pub(super) fn carry_tidings(&mut self, gid: GroupId) {
        let Some(g) = self.group(gid) else { return };
        let Kind::Journey { home } = g.kind else { return };
        let legs: Vec<(SettlementId, f64)> = g.legs.iter().filter_map(|l| l.dest.map(|d| (d, l.arrive))).collect();
        // What's known by the time they set out (not when the world is next looked at).
        let Some(now) = g.legs.first().map(|l| l.depart) else { return };
        let day = World::day_of(now) as i32;
        let mut from = home;
        for (dest, arrive) in legs {
            if dest != from {
                let mut news: Vec<Known> = self.events(from).iter().filter(|e| e.severity >= TIDING_SEVERITY && e.t <= now).map(|e| Known { id: e.id, town: from }).collect();
                news.sort_by(|a, b| self.interest(*b, day).total_cmp(&self.interest(*a, day)).then(a.id.cmp(&b.id)));
                for k in news.into_iter().take(TIDINGS_PER_LEG) {
                    if Rng::from_keys(&[self.seed, gid as u64, k.id as u64, 0x5449_4445]).chance(TIDING_CHANCE) {
                        self.society.history.tidings.push(Tiding { at: arrive, town: dest, news: k });
                    }
                }
            }
            from = dest;
        }
        let n = self.society.history.tidings.len();
        if n > TIDINGS_CAP {
            self.society.history.tidings.drain(..n - TIDINGS_CAP);
        }
    }
}
