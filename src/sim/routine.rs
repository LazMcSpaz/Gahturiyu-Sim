//! Daily routines, looked up from the clock.
//!
//! Nobody's day is stepped through. A person's **day plan** — when they
//! wake, walk to work, eat, work, spend the evening and sleep — is a pure
//! function of who they are (job, habits, seed), their community's customs
//! (its rhythm above all), the day number and the clock: seasonal hours
//! slide with the year, bells ring fixed shifts, tide-keepers follow the low
//! water a little later every day, and some keep no fixed hours at all.
//! Where someone is and what they're doing is read off the plan at any
//! moment; the economy reads the same plans to know who worked when.
//!
//! A service is open only while someone who does it is at work right now.

use super::culture::{Evening, Rhythm};
use super::geo::{self, V2};
use super::jobs::{Job, PlaceKind, Service};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::{BuildingKind, SettlementId};
use super::society::{Life, MARKET_EVERY, WEEK};
use super::tide;
use super::world::{World, DAY, HOUR};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doing {
    Asleep,
    /// About the house and yard.
    Home,
    /// Walking from one place to the next.
    Walking,
    Work,
    Meal,
    /// A meal runner's midday round, carrying pots out to the workplaces.
    Run,
    /// Crewing the dawn boat.
    Ferry,
    /// A Stone Tender going from home to home.
    Rounds,
    Evening,
    Market,
    Visiting,
    Garden,
    /// Nothing much: drifters, and anyone with no work today.
    Idle,
}

impl Doing {
    pub fn word(self) -> &'static str {
        match self {
            Doing::Asleep => "asleep",
            Doing::Home => "at home",
            Doing::Walking => "walking",
            Doing::Work => "at work",
            Doing::Meal => "eating",
            Doing::Run => "carrying the midday pots",
            Doing::Ferry => "bringing the catch ashore",
            Doing::Rounds => "tending the homes",
            Doing::Evening => "spending the evening",
            Doing::Market => "at the market",
            Doing::Visiting => "visiting",
            Doing::Garden => "tending the garden",
            Doing::Idle => "idling",
        }
    }
}

/// Where a part of the day is spent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Home,
    Work,
    /// A workplace by index (inn, market, deck, kitchen).
    Place(u16),
    Hearth,
    /// Someone else's home (by building).
    Visit(u16),
    Garden,
    Rounds,
    /// The dawn boat's run from the stilts to the shore and back.
    Boat,
    /// The meal runner's round.
    RunRound,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seg {
    /// Hours into the day this part starts.
    pub from: f32,
    pub doing: Doing,
    pub spot: Spot,
}

/// One person's day, midnight to midnight.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub segs: [Seg; 18],
    pub n: usize,
    pub rest: bool,
    /// The working hours, if they work today.
    pub work: Option<(f32, f32)>,
}

impl Plan {
    fn new() -> Plan {
        Plan { segs: [Seg { from: 0.0, doing: Doing::Asleep, spot: Spot::Home }; 18], n: 0, rest: false, work: None }
    }

    /// Add a part of the day; anything out of order is dropped.
    fn push(&mut self, from: f32, doing: Doing, spot: Spot) {
        if self.n >= self.segs.len() || from >= 24.0 {
            return;
        }
        if self.n > 0 {
            let last = self.segs[self.n - 1];
            if from <= last.from {
                if from == last.from {
                    self.segs[self.n - 1] = Seg { from, doing, spot };
                }
                return;
            }
            if last.doing == doing && last.spot == spot {
                return;
            }
        }
        self.segs[self.n] = Seg { from: from.max(0.0), doing, spot };
        self.n += 1;
    }

    pub fn segs(&self) -> &[Seg] {
        &self.segs[..self.n]
    }

    /// Which part of the day hour `h` falls in.
    pub fn index_at(&self, h: f32) -> usize {
        self.segs().iter().rposition(|s| s.from <= h).unwrap_or(0)
    }

    pub fn end_of(&self, i: usize) -> f32 {
        if i + 1 < self.n {
            self.segs[i + 1].from
        } else {
            24.0
        }
    }

    /// Hours between `a` and `b` (hours of the day) spent doing `what`.
    pub fn hours_of(&self, what: Doing, a: f32, b: f32) -> f32 {
        let mut total = 0.0;
        for i in 0..self.n {
            if self.segs[i].doing == what {
                let (s, e) = (self.segs[i].from.max(a), self.end_of(i).min(b));
                if e > s {
                    total += e - s;
                }
            }
        }
        total
    }
}

/// Length of a meal runner's midday round, hours.
pub const RUN_HOURS: f32 = 1.5;
/// When the dawn boat leaves the stilts and when it's back, hours.
pub const BOAT_OUT: f32 = 5.4;
pub const BOAT_BACK: f32 = 6.4;
/// Usual bedtime, hours (individuals vary).
pub const BED: f32 = 22.0;

fn hours(t: f64) -> f32 {
    (t.rem_euclid(DAY) / HOUR) as f32
}

impl World {
    /// The rhythm someone keeps: their own, if they keep their people's
    /// hours, else their community's.
    pub fn rhythm_of(&self, pid: PersonId) -> Rhythm {
        let l = self.life(pid);
        l.habits.own_rhythm.unwrap_or_else(|| self.community_of(pid).map(|c| c.customs.rhythm).unwrap_or(Rhythm::Irregular))
    }

    fn is_rest_day(&self, pid: PersonId, day: i64, rhythm: Rhythm) -> bool {
        let l = self.life(pid);
        match rhythm {
            // Under the bells, a third of a shift rests together.
            Rhythm::Bells => (day + l.shift as i64 * 3 + (l.rest_day % 3) as i64 * 2).rem_euclid(WEEK) == 0,
            Rhythm::Irregular => Rng::from_keys(&[self.people[pid as usize].seed, day as u64, 0x5245_5354]).chance(0.25),
            _ => (day + l.rest_day as i64).rem_euclid(WEEK) == 0,
        }
    }

    /// When a community eats at midday (and the runners go out), hours.
    pub fn community_meal(&self, ci: u32, day: i64) -> f32 {
        let c = &self.society.communities[ci as usize];
        match c.customs.rhythm {
            Rhythm::Seasonal => 12.0 - tide::season_shift(day),
            Rhythm::Bells => 10.0,
            Rhythm::Tides => (tide::work_low(day) + 0.5).clamp(7.0, 17.0),
            Rhythm::Irregular => 12.5,
        }
    }

    /// Working hours and meal time on a day, by rhythm (before the job's own
    /// hours are applied).
    fn work_hours(&self, pid: PersonId, day: i64, rhythm: Rhythm, jitter: f32, r: &mut Rng) -> (f32, f32, f32) {
        let l = self.life(pid);
        match rhythm {
            Rhythm::Seasonal => {
                let ws = 7.0 - tide::season_shift(day) + jitter * 0.5;
                (ws, ws + 9.5, ws + 5.0)
            }
            Rhythm::Bells => {
                if l.shift == 1 {
                    (14.0, 22.0, 18.0)
                } else {
                    (6.0, 14.0, 10.0)
                }
            }
            Rhythm::Tides => {
                let low = tide::work_low(day);
                let ws = (low - 3.0).max(4.0);
                let we = (low + 4.5).min(21.5).max(ws + 4.0);
                (ws, we, (low + 0.5).clamp(ws + 1.0, we - 1.0))
            }
            Rhythm::Irregular => {
                let ws = 6.5 + r.f32() * 5.0;
                let we = ws + 4.0 + r.f32() * 5.0;
                (ws, we.min(22.5), (ws + we) * 0.5)
            }
        }
    }

    /// Where someone likes to spend the evening, among what their
    /// community has.
    fn evening_spot(&self, pid: PersonId) -> Spot {
        let l = self.life(pid);
        let Some(c) = l.community else { return Spot::Home };
        let comm = &self.society.communities[c as usize];
        let Some(town) = self.people[pid as usize].home else { return Spot::Home };
        let tl = &self.society.towns[town as usize];
        let find = |k: PlaceKind| tl.places.iter().position(|p| p.kind == k && (geo::inland(p.pos) < 0.0) == comm.stilts).map(|i| Spot::Place(i as u16));
        match (l.habits.evening, comm.stilts) {
            (Evening::Home, _) => Spot::Home,
            // Offshore, the deck is the hearth.
            (_, true) => find(PlaceKind::Deck).unwrap_or(Spot::Home),
            (Evening::Inn, false) => find(PlaceKind::Inn).unwrap_or(Spot::Hearth),
            (Evening::Deck, false) => find(PlaceKind::Deck).unwrap_or(Spot::Hearth),
            (Evening::Hearth, false) => Spot::Hearth,
        }
    }

    fn market_spot(&self, town: SettlementId) -> Option<Spot> {
        let tl = &self.society.towns[town as usize];
        tl.places.iter().position(|p| matches!(p.kind, PlaceKind::Market | PlaceKind::Shop(_))).map(|i| Spot::Place(i as u16))
    }

    /// Does this person tend a garden?
    pub fn gardener_of(&self, pid: PersonId) -> Option<u16> {
        let town = self.people[pid as usize].home?;
        self.society.towns[town as usize].gardens.iter().find(|g| g.gardener == Some(pid)).map(|g| g.building)
    }

    /// Someone's plan for a day (day 0 begins at the world's first midnight).
    pub fn day_plan(&self, pid: PersonId, day: i64) -> Plan {
        let mut plan = Plan::new();
        let p = &self.people[pid as usize];
        let l: Life = *self.life(pid);
        let Some(town) = p.home else {
            plan.push(0.0, Doing::Idle, Spot::Home);
            return plan;
        };
        let Some(ci) = l.community else {
            plan.push(0.0, Doing::Idle, Spot::Home);
            return plan;
        };
        let comm = &self.society.communities[ci as usize];
        let tl = &self.society.towns[town as usize];
        let rhythm = self.rhythm_of(pid);
        let mut r = Rng::from_keys(&[p.seed, day as u64, 0x504C_414E]);
        let jitter = r.range(-0.3, 0.3);
        let market = (day + tl.market_day as i64).rem_euclid(MARKET_EVERY) == 0 && !comm.stilts;
        let evening = self.evening_spot(pid);
        let garden = self.gardener_of(pid).is_some();
        let visit = {
            let homes: Vec<u16> = self.settlements[town as usize]
                .buildings
                .iter()
                .enumerate()
                .filter(|(_, b)| (b.kind == BuildingKind::HoraroStilt) == comm.stilts && matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock | BuildingKind::HoraroStilt))
                .map(|(i, _)| i as u16)
                .collect();
            (r.chance(0.3) && !homes.is_empty()).then(|| homes[r.below(homes.len())])
        };
        let bed = (BED + jitter * 2.0).clamp(20.5, 23.6);
        let rest = matches!(l.job, Job::Drifter | Job::None) || self.is_rest_day(pid, day, rhythm);
        plan.rest = rest;
        let crew = comm.stilts && l.job == Job::Fisher && l.shift == 1 && !comm.withdrawn;

        // The night watch keeps its own day.
        if l.job == Job::Guard && l.shift == 1 {
            let worked_last_night = !self.is_rest_day(pid, day - 1, rhythm);
            if worked_last_night {
                plan.push(0.0, Doing::Work, Spot::Work);
                plan.push(2.0, Doing::Meal, Spot::Work);
                plan.push(2.5, Doing::Work, Spot::Work);
                plan.push(6.0, Doing::Walking, Spot::Home);
                plan.push(6.25, Doing::Asleep, Spot::Home);
            } else {
                plan.push(0.0, Doing::Asleep, Spot::Home);
            }
            plan.push(14.0 + jitter, Doing::Home, Spot::Home);
            if rest {
                plan.push(17.0, Doing::Walking, evening);
                plan.push(17.25, Doing::Evening, evening);
                plan.push(bed - 0.25, Doing::Walking, Spot::Home);
                plan.push(bed, Doing::Asleep, Spot::Home);
            } else {
                if garden {
                    plan.push(15.0, Doing::Garden, Spot::Garden);
                    plan.push(16.0, Doing::Home, Spot::Home);
                }
                plan.push(17.6, Doing::Walking, Spot::Work);
                plan.push(18.0, Doing::Work, Spot::Work);
                plan.work = Some((18.0, 30.0));
            }
            return plan;
        }

        if rest {
            let wake = (6.5 + jitter + r.f32()).min(9.0);
            plan.push(0.0, Doing::Asleep, Spot::Home);
            if crew {
                plan.push(BOAT_OUT - 0.3, Doing::Home, Spot::Home);
                plan.push(BOAT_OUT, Doing::Ferry, Spot::Boat);
                plan.push(BOAT_BACK, Doing::Asleep, Spot::Home);
            }
            plan.push(wake, Doing::Home, Spot::Home);
            let out = if l.job == Job::Drifter { Some(Spot::Hearth) } else { None };
            let mut h = wake + 1.5;
            if let Some(s) = out {
                plan.push(h, Doing::Walking, s);
                plan.push(h + 0.25, Doing::Idle, s);
                h += 3.0;
            }
            if market {
                if let Some(m) = self.market_spot(town) {
                    plan.push(h.max(10.0), Doing::Walking, m);
                    plan.push(h.max(10.0) + 0.25, Doing::Market, m);
                    h = h.max(10.0) + 1.75;
                }
            }
            if garden {
                plan.push(h.max(13.0), Doing::Walking, Spot::Garden);
                plan.push(h.max(13.0) + 0.1, Doing::Garden, Spot::Garden);
                h = h.max(13.0) + 1.5;
            } else {
                plan.push(h, Doing::Walking, Spot::Home);
                plan.push(h + 0.25, Doing::Home, Spot::Home);
            }
            if let Some(b) = visit {
                let s = h.max(15.0);
                plan.push(s, Doing::Walking, Spot::Visit(b));
                plan.push(s + 0.25, Doing::Visiting, Spot::Visit(b));
                h = s + 2.5;
            }
            let eve = h.max(18.0);
            plan.push(eve, Doing::Walking, evening);
            plan.push(eve + 0.25, Doing::Evening, evening);
            plan.push(bed - 0.25, Doing::Walking, Spot::Home);
            plan.push(bed, Doing::Asleep, Spot::Home);
            return plan;
        }

        let (mut ws, mut we, mut meal) = self.work_hours(pid, day, rhythm, jitter, &mut r);
        match l.job {
            Job::Innkeeper => (ws, we, meal) = (10.0, 23.0, 16.0),
            Job::Cook => {
                ws -= 1.5;
                we -= 1.0;
                meal = (ws + we) * 0.5 + 1.0;
            }
            Job::Merchant if market => we += 2.0,
            Job::Guard => (ws, we, meal) = (6.0, 18.0, 12.0),
            _ => {}
        }
        ws = ws.max(if crew { BOAT_BACK + 0.25 } else { 3.5 });
        we = we.min(23.4).max(ws + 2.0);
        let wake = if crew { BOAT_OUT - 0.4 } else { (ws - 0.9).max(3.0).min(ws - 0.3) };
        let bed = bed.max(we + 1.2).min(23.8);
        plan.work = Some((ws, we));

        plan.push(0.0, Doing::Asleep, Spot::Home);
        plan.push(wake, Doing::Home, Spot::Home);
        if crew {
            plan.push(BOAT_OUT, Doing::Ferry, Spot::Boat);
            plan.push(BOAT_BACK, Doing::Home, Spot::Home);
        }
        let working = if l.job == Job::StoneTender { (Doing::Rounds, Spot::Rounds) } else { (Doing::Work, Spot::Work) };
        plan.push(ws - 0.25, Doing::Walking, working.1);
        plan.push(ws, working.0, working.1);
        if l.job == Job::Runner {
            let m = self.community_meal(ci, day);
            let (a, b) = ((m - RUN_HOURS * 0.5).max(ws + 0.25), (m + RUN_HOURS * 0.5).min(we - 0.5));
            plan.push(a, Doing::Run, Spot::RunRound);
            plan.push(b, Doing::Meal, Spot::Work);
            plan.push(b + 0.4, Doing::Work, Spot::Work);
        } else {
            let m = meal.clamp(ws + 0.5, we - 0.6);
            plan.push(m, Doing::Meal, Spot::Work);
            plan.push(m + 0.5, working.0, working.1);
        }
        let mut h = we;
        if garden {
            plan.push(h, Doing::Walking, Spot::Garden);
            plan.push(h + 0.25, Doing::Garden, Spot::Garden);
            h += 1.25;
        }
        if market && r.chance(0.5) {
            if let Some(m) = self.market_spot(town) {
                plan.push(h, Doing::Walking, m);
                plan.push(h + 0.25, Doing::Market, m);
                h += 1.25;
            }
        }
        if let Some(b) = visit.filter(|_| r.chance(0.4)) {
            plan.push(h, Doing::Walking, Spot::Visit(b));
            plan.push(h + 0.25, Doing::Visiting, Spot::Visit(b));
            h += 1.5;
        }
        plan.push(h, Doing::Walking, evening);
        plan.push(h + 0.25, Doing::Evening, evening);
        plan.push(bed - 0.25, Doing::Walking, Spot::Home);
        plan.push(bed, Doing::Asleep, Spot::Home);
        plan
    }

    /// Is someone at home in their town (not off on the road, not dead)?
    pub fn is_about(&self, pid: PersonId, t: f64) -> bool {
        let p = &self.people[pid as usize];
        p.home.is_some() && !p.dead && !p.in_squad && !p.bandit && self.busy_until[pid as usize] <= t && self.group_of[pid as usize].is_none()
    }

    /// What someone is doing right now, by their plan (None if they're away
    /// or have no town).
    pub fn doing_now(&self, pid: PersonId) -> Option<(Doing, Spot)> {
        if !self.is_about(pid, self.time) {
            return None;
        }
        let plan = self.day_plan(pid, World::day_of(self.time));
        let s = plan.segs[plan.index_at(hours(self.time))];
        Some((s.doing, s.spot))
    }

    /// At work right now (so whatever they do is open).
    pub fn at_work(&self, pid: PersonId, t: f64) -> bool {
        if !self.is_about(pid, t) || self.fighting.contains_key(&pid) {
            return false;
        }
        let p = &self.people[pid as usize];
        if super::body::knocked_out(&p.wounds.hp_at(&p.stats, t)) {
            return false;
        }
        let plan = self.day_plan(pid, World::day_of(t));
        matches!(plan.segs[plan.index_at(hours(t))].doing, Doing::Work | Doing::Rounds)
    }

    /// Asleep in their own bed right now.
    pub fn is_indoors_asleep(&self, pid: PersonId) -> bool {
        matches!(self.doing_now(pid), Some((Doing::Asleep, _)))
    }

    /// Is this service open in a town right now? (Someone who does it is at work.)
    pub fn service_open(&self, town: SettlementId, svc: Service) -> bool {
        self.settlements[town as usize].residents.iter().any(|&p| self.life(p).job.service() == Some(svc) && self.at_work(p, self.time))
    }

    /// Is a particular workplace open right now?
    pub fn place_open(&self, town: SettlementId, place: u16) -> bool {
        self.settlements[town as usize].residents.iter().any(|&p| self.life(p).place == Some(place) && self.life(p).job.is_post() && self.at_work(p, self.time))
    }

    /// Who does a service in a town, at all (alive and living there).
    pub fn service_workers(&self, town: SettlementId, svc: Service) -> usize {
        self.settlements[town as usize].residents.iter().filter(|&&p| !self.people[p as usize].dead && self.life(p).job.service() == Some(svc)).count()
    }

    // ---- Where people are -------------------------------------------------

    fn spot_pos(&self, pid: PersonId, spot: Spot, asleep: bool, h: f32, day: i64) -> V2 {
        let p = &self.people[pid as usize];
        let town = p.home.unwrap();
        let s = &self.settlements[town as usize];
        let tl = &self.society.towns[town as usize];
        let mut r = Rng::from_keys(&[p.seed, 0x504F_5349]);
        let a = r.f32() * std::f32::consts::TAU;
        let u = r.f32();
        let around = |c: V2, near: f32, far: f32| c.add(V2::new(a.cos(), a.sin()).scale(near + (far - near) * u));
        let home = || -> V2 {
            match p.dwelling.map(|d| &s.buildings[d as usize]) {
                // In bed: indoors.
                Some(b) if asleep => around(b.pos, 0.0, b.size * 0.25),
                Some(b) if b.kind == BuildingKind::HoraroStilt => around(b.pos, b.size * 0.36, b.size * 0.44),
                Some(b) => around(b.pos, b.size * 0.6 + 2.0, b.size * 0.6 + 9.0),
                None => around(s.pos, 5.0, s.radius()),
            }
        };
        let place = |i: u16| -> V2 {
            let w = &tl.places[i as usize];
            // Divers spread out in the water round their platform.
            let spread = if w.kind == PlaceKind::DivePlatform { 1.4 } else { 0.4 };
            around(w.pos, 1.0, w.kind.size() * spread)
        };
        match spot {
            Spot::Home => home(),
            Spot::Work => match self.life(pid).place {
                Some(i) => place(i),
                None => home(),
            },
            Spot::Place(i) => place(i),
            Spot::Hearth => around(s.pos, 4.5, 11.0),
            Spot::Visit(b) => {
                let b = &s.buildings[b as usize];
                around(b.pos, b.size * 0.6 + 1.5, b.size * 0.6 + 5.0)
            }
            Spot::Garden => match p.dwelling.map(|d| &s.buildings[d as usize]) {
                Some(b) => {
                    let away = b.pos.sub(s.pos);
                    let d = away.scale(1.0 / away.len().max(1.0));
                    b.pos.add(d.scale(b.size * 0.6 + 5.0)).add(V2::new(a.cos(), a.sin()).scale(1.5 * u))
                }
                None => home(),
            },
            Spot::Rounds => {
                let homes: Vec<V2> = s.buildings.iter().filter(|b| matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::HoraroStilt)).map(|b| b.pos.add(V2::new(b.size * 0.55, 0.0))).collect();
                if homes.is_empty() {
                    return around(s.pos, 4.0, 12.0);
                }
                // A home every 24 minutes, walking the first few.
                let k = (h / 0.4).floor();
                let f = ((h / 0.4) - k).clamp(0.0, 1.0);
                let start = (p.seed as usize).wrapping_add(day as usize * 7);
                let i = (start + k as usize) % homes.len();
                let j = (i + 1) % homes.len();
                if f < 0.25 {
                    homes[(i + homes.len() - 1) % homes.len()].lerp(homes[i], f / 0.25)
                } else {
                    let _ = j;
                    homes[i]
                }
            }
            Spot::Boat => {
                // Out from the stilts to the dock, unload, and back.
                let from = tl.places.iter().find(|w| w.kind == PlaceKind::Deck && geo::inland(w.pos) < 0.0).or_else(|| tl.places.iter().find(|w| w.kind == PlaceKind::DivePlatform)).map(|w| w.pos).unwrap_or(s.pos);
                // Each boat pulls up at its own spot along the dock.
                let to = tl.places.iter().find(|w| w.kind == PlaceKind::Dock).map(|w| w.pos.add(V2::new(-4.0 - 12.0 * u, 2.8 * if a.sin() > 0.0 { 1.0 } else { -1.0 }))).unwrap_or(s.pos);
                let span = BOAT_BACK - BOAT_OUT;
                let f = ((h - BOAT_OUT) / span).clamp(0.0, 1.0);
                let leg = if f < 0.42 { f / 0.42 } else if f < 0.58 { 1.0 } else { 1.0 - (f - 0.58) / 0.42 };
                from.add(V2::new(0.0, a.sin() * 8.0)).lerp(to, leg)
            }
            Spot::RunRound => {
                // Kitchen → each out-of-town workplace → kitchen.
                let base = self.life(pid).place.map(|i| tl.places[i as usize].pos).unwrap_or(s.pos);
                let mut stops: Vec<V2> = vec![base];
                stops.extend(tl.places.iter().filter(|w| w.kind.is_away() && !w.kind.offshore() && w.kind != PlaceKind::Dock).map(|w| w.pos));
                stops.push(base);
                let plan = self.day_plan(pid, day);
                let i = plan.index_at(h);
                let (s0, s1) = (plan.segs[i].from, plan.end_of(i));
                let f = ((h - s0) / (s1 - s0).max(0.01)).clamp(0.0, 0.999);
                let lens: Vec<f32> = stops.windows(2).map(|w| w[0].dist(w[1])).collect();
                let total: f32 = lens.iter().sum::<f32>().max(0.01);
                let mut d = f * total;
                for (k, &len) in lens.iter().enumerate() {
                    if d <= len {
                        return stops[k].lerp(stops[k + 1], d / len.max(0.01));
                    }
                    d -= len;
                }
                base
            }
        }
    }

    /// Where a resident who is at home in town stands at `t`, by their plan.
    pub(super) fn routine_pos(&self, pid: PersonId, t: f64) -> V2 {
        let day = World::day_of(t);
        let plan = self.day_plan(pid, day);
        let h = hours(t);
        let i = plan.index_at(h);
        let seg = plan.segs[i];
        let p = &self.people[pid as usize];
        let wobble = {
            let phase = (p.seed % 1000) as f32 / 10.0;
            let tt = (t / 240.0) as f32 + phase;
            let drift = match seg.doing {
                Doing::Asleep | Doing::Walking | Doing::Run | Doing::Ferry | Doing::Rounds => 0.0,
                Doing::Work => 1.6,
                _ => 2.2,
            };
            V2::new(tt.sin(), (tt * 0.7).cos()).scale(drift)
        };
        if seg.doing == Doing::Walking {
            // From where the last part was to where the next one is.
            let prev = if i > 0 { plan.segs[i - 1] } else { seg };
            let next = if i + 1 < plan.n { plan.segs[i + 1] } else { seg };
            let a = self.spot_pos(pid, prev.spot, prev.doing == Doing::Asleep, prev.from, day);
            let b = self.spot_pos(pid, next.spot, next.doing == Doing::Asleep, next.from, day);
            let f = ((h - seg.from) / (plan.end_of(i) - seg.from).max(0.01)).clamp(0.0, 1.0);
            return a.lerp(b, f);
        }
        self.spot_pos(pid, seg.spot, seg.doing == Doing::Asleep, h, day).add(wobble)
    }
}
