//! The world, and the loop that moves it forward.

use std::collections::{HashMap, VecDeque};

use super::bands::{BandMap, REFRESH};
use super::geo::{self, V2};
use super::group::{Group, GroupId, Kind, Leg};
use super::person::{Person, PersonId};
use super::rng::{self, Rng};
use super::settlement::{BuildingKind, Settlement, SettlementId};

/// Departure pressure per unit of (wanderlust squared) per game hour. The main
/// dial for how busy the roads are.
pub const DEPARTURE_RATE: f32 = 0.014;
/// Share of the daytime departure rate that still happens at night.
pub const NIGHT_FACTOR: f32 = 0.12;
/// The squad's walking pace, metres per second.
pub const SQUAD_SPEED: f32 = 1.5;
/// A group that leaves band 1 and comes back within this many game seconds is
/// not announced again.
const ANNOUNCE_GAP: f64 = 3600.0;
/// How often (game seconds) townsfolk are checked for coming into band 1.
const TOWN_LOOK_EVERY: f64 = 3.0;
/// How many lines of the event log to keep.
const LOG_LEN: usize = 14;

pub const HOUR: f64 = 3600.0;
pub const DAY: f64 = 24.0 * HOUR;

#[derive(Clone, Debug)]
pub struct Squad {
    pub members: Vec<PersonId>,
    pub pos: V2,
    pub target: V2,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// People whose name and gear have been built so far.
    pub detailed: usize,
    /// Groups refreshed during the last step, by band.
    pub refreshed: [usize; 4],
    /// Groups currently in each band.
    pub in_band: [usize; 4],
    pub journeys_started: usize,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u64,
    /// Game seconds since the world began. Day 1 starts at 06:00.
    pub time: f64,
    pub people: Vec<Person>,
    /// When each person is next free to set out. Read off schedules, so it is
    /// correct no matter how coarsely their group is being simulated.
    pub busy_until: Vec<f64>,
    pub group_of: Vec<Option<GroupId>>,
    pub settlements: Vec<Settlement>,
    pub groups: Vec<Group>,
    group_index: HashMap<GroupId, usize>,
    pub next_group: GroupId,
    pub squad: Squad,
    pub bands: BandMap,
    /// The last whole game-hour whose departures have been decided.
    pub hour_done: i64,
    pub log: VecDeque<(f64, String)>,
    pub stats: Stats,
    /// Last time each group was in band 1, so one that hovers on the edge of
    /// the band is not announced over and over.
    in_view_groups: HashMap<GroupId, f64>,
    in_view_towns: Vec<bool>,
    last_town_look: f64,
    looked_once: bool,
}

impl World {
    pub fn assemble(
        seed: u64,
        people: Vec<Person>,
        settlements: Vec<Settlement>,
        groups: Vec<Group>,
        squad: Squad,
        start_time: f64,
    ) -> World {
        let n = people.len();
        let mut w = World {
            seed,
            time: start_time,
            busy_until: vec![0.0; n],
            group_of: vec![None; n],
            next_group: groups.iter().map(|g| g.id + 1).max().unwrap_or(0),
            in_view_towns: vec![false; settlements.len()],
            bands: BandMap::new(squad.pos),
            people,
            settlements,
            groups: Vec::new(),
            group_index: HashMap::new(),
            squad,
            hour_done: (start_time / HOUR).floor() as i64,
            log: VecDeque::new(),
            stats: Stats::default(),
            in_view_groups: HashMap::new(),
            last_town_look: 0.0,
            looked_once: false,
        };
        for &m in &w.squad.members.clone() {
            w.busy_until[m as usize] = f64::INFINITY;
        }
        for g in groups {
            w.add_group(g);
        }
        w.stats.detailed = w.people.iter().filter(|p| p.detail.is_some()).count();
        w.observe();
        w
    }

    fn add_group(&mut self, mut g: Group) {
        g.extend_to(self.time);
        g.pos = g.position_at(self.time);
        g.last_update = self.time;
        g.band = self.bands.band_at(g.pos);
        for &m in &g.members {
            self.busy_until[m as usize] = g.ends;
            self.group_of[m as usize] = Some(g.id);
        }
        self.group_index.insert(g.id, self.groups.len());
        self.groups.push(g);
    }

    pub fn group(&self, id: GroupId) -> Option<&Group> {
        self.group_index.get(&id).map(|&i| &self.groups[i])
    }

    pub fn order_squad(&mut self, target: V2) {
        self.squad.target = geo::clamp_to_world(target, 50.0);
    }

    /// Advance the world by `dt` game seconds.
    pub fn step(&mut self, dt: f64) {
        // 1. The squad walks. It is always fully simulated.
        let to_go = self.squad.target.sub(self.squad.pos);
        let d = to_go.len();
        let stride = (SQUAD_SPEED as f64 * dt) as f32;
        self.squad.pos = if d <= stride { self.squad.target } else { self.squad.pos.add(to_go.scale(stride / d)) };
        self.bands.update(self.squad.pos);

        self.time += dt;

        // 2. Departures, decided one game-hour at a time, in order. Keyed by
        //    settlement and hour, so they never depend on the step size.
        let now_hour = (self.time / HOUR).floor() as i64;
        while self.hour_done < now_hour {
            self.hour_done += 1;
            for s in 0..self.settlements.len() {
                if let Some(g) = self.plan_departure(s as SettlementId, self.hour_done) {
                    self.add_group(g);
                    self.stats.journeys_started += 1;
                }
            }
        }

        // 3. Refresh groups, each at its band's rate.
        self.stats.refreshed = [0; 4];
        let t = self.time;
        for g in &mut self.groups {
            if t - g.last_update >= REFRESH[g.band as usize] {
                g.extend_to(t);
                g.pos = g.position_at(t);
                g.last_update = t;
                g.band = self.bands.band_at(g.pos);
                self.stats.refreshed[g.band as usize] += 1;
            }
        }

        // 4. Journeys that are over dissolve; their people are home.
        //    Decided by the schedule, not by whether anyone looked.
        if self.groups.iter().any(|g| t >= g.ends) {
            for g in self.groups.iter().filter(|g| t >= g.ends) {
                for &m in &g.members {
                    // They may already have set out again with a newer group
                    // inside this same step; only clear this group's claim.
                    if self.group_of[m as usize] == Some(g.id) {
                        self.group_of[m as usize] = None;
                    }
                }
            }
            self.groups.retain(|g| t < g.ends);
            self.group_index = self.groups.iter().enumerate().map(|(i, g)| (g.id, i)).collect();
        }

        self.observe();
    }

    /// Whatever is close enough to matter gets its details built, and stays
    /// built. Also notes what has just come into view.
    fn observe(&mut self) {
        self.stats.in_band = [0; 4];
        let t = self.time;
        let mut made = 0;
        let mut entered: Vec<GroupId> = Vec::new();
        for g in &self.groups {
            self.stats.in_band[g.band as usize] += 1;
            if g.band == 1 {
                for &m in &g.members {
                    if self.people[m as usize].ensure_detail() {
                        made += 1;
                    }
                }
                let fresh = self.in_view_groups.get(&g.id).map(|&last| t - last > ANNOUNCE_GAP).unwrap_or(true);
                if fresh {
                    entered.push(g.id);
                }
                self.in_view_groups.insert(g.id, t);
            }
        }
        self.in_view_groups.retain(|_, last| t - *last <= ANNOUNCE_GAP);
        for id in entered {
            let line = self.describe_group(id);
            self.push_log(line);
        }

        // Townsfolk are banded one by one, by where each of them is standing.
        // Checked a few times a game-minute: who gets *named* never changes
        // what happens, so this needs no finer timing.
        if t - self.last_town_look < TOWN_LOOK_EVERY && !self.in_view_towns.is_empty() && self.looked_once {
            self.stats.detailed += made;
            return;
        }
        self.last_town_look = t;
        self.looked_once = true;
        for s in 0..self.settlements.len() {
            let near = self.residents_in_band1(s as SettlementId);
            for &p in &near {
                if self.people[p as usize].ensure_detail() {
                    made += 1;
                }
            }
            let any = !near.is_empty();
            if any && !self.in_view_towns[s] {
                let line = format!("{} comes into view.", self.settlements[s].name);
                self.push_log(line);
            }
            self.in_view_towns[s] = any;
        }
        self.stats.detailed += made;
    }

    fn push_log(&mut self, line: String) {
        self.log.push_front((self.time, line));
        self.log.truncate(LOG_LEN);
    }

    pub fn describe_group(&self, id: GroupId) -> String {
        let Some(g) = self.group(id) else { return String::new() };
        let lead = &self.people[g.members[0] as usize];
        let who = match lead.name() {
            Some(n) if g.members.len() == 1 => format!("{} ({})", n, lead.race.name()),
            Some(n) => format!("{} and {} other{}", n, g.members.len() - 1, if g.members.len() == 2 { "" } else { "s" }),
            None => format!("{} travellers", g.members.len()),
        };
        match g.kind {
            Kind::Wanderer { .. } => format!("{who}, wandering, crosses your path."),
            Kind::Journey { home } => {
                let going = g
                    .current_leg(self.time)
                    .and_then(|l| l.dest)
                    .map(|d| {
                        if d == home {
                            format!("heading home to {}", self.settlements[d as usize].name)
                        } else {
                            format!("bound for {}", self.settlements[d as usize].name)
                        }
                    })
                    .unwrap_or_default();
                format!("{who}, {going}.")
            }
        }
    }

    /// Residents of a town who are at home and inside band 1 right now —
    /// the ones who exist as individuals at this moment.
    pub fn residents_in_band1(&self, s: SettlementId) -> Vec<PersonId> {
        let town = &self.settlements[s as usize];
        if !self.bands.touches_band1(town.pos, town.reach + 20.0) {
            return Vec::new();
        }
        town.residents
            .iter()
            .copied()
            .filter(|&p| self.busy_until[p as usize] <= self.time && self.group_of[p as usize].is_none())
            .filter(|&p| self.bands.band_at(self.person_pos(p)) == 1)
            .collect()
    }

    /// Where a person stands right now, to the metre. Only meaningful for
    /// people close enough to be drawn individually.
    pub fn person_pos(&self, pid: PersonId) -> V2 {
        let p = &self.people[pid as usize];
        let mut r = Rng::from_keys(&[p.seed, 0x504F_5349]);
        if p.in_squad {
            let i = self.squad.members.iter().position(|&m| m == pid).unwrap_or(0) as f32;
            return self.squad.pos.add(V2::new((i * 2.4).cos(), (i * 2.4).sin()).scale(3.0 + i));
        }
        if let Some(gid) = self.group_of[pid as usize] {
            if let Some(g) = self.group(gid) {
                let i = g.members.iter().position(|&m| m == pid).unwrap_or(0) as f32;
                let spread = if g.is_moving(self.time) { 2.5 } else { 7.0 };
                return g.pos.add(V2::new((i * 2.1 + 0.5).cos(), (i * 2.1 + 0.5).sin()).scale(spread * (0.6 + i * 0.35)));
            }
        }
        if let Some(h) = p.home {
            let s = &self.settlements[h as usize];
            // Around their own front door: on the deck for a stilt home, in
            // the yard for anything on land.
            let (centre, near, far, drift) = match p.dwelling.map(|d| &s.buildings[d as usize]) {
                Some(b) if b.kind == BuildingKind::HoraroStilt => (b.pos, b.size * 0.36, b.size * 0.44, 0.3),
                Some(b) => (b.pos, b.size * 0.6 + 2.0, b.size * 0.6 + 12.0, 3.0),
                None => (s.pos, 5.0, s.radius(), 5.0),
            };
            let a = r.f32() * std::f32::consts::TAU;
            let rad = r.range(near, far);
            // A slow drift, so a town is not a frozen photograph.
            let phase = r.f32() * 100.0;
            let tt = (self.time / 240.0) as f32 + phase;
            let wobble = V2::new(tt.sin(), (tt * 0.7).cos()).scale(drift);
            return centre.add(V2::new(a.cos(), a.sin()).scale(rad)).add(wobble);
        }
        V2::default()
    }

    /// Decide whether anyone sets out from settlement `s` during hour `h`,
    /// and if so, write their whole journey down.
    fn plan_departure(&mut self, s: SettlementId, h: i64) -> Option<Group> {
        let mut rng = Rng::from_keys(&[self.seed, s as u64, h as u64, 0x4445_5054]);
        let start = h as f64 * HOUR + rng.f64() * HOUR;
        let hour_of_day = h.rem_euclid(24);
        let day = (6..=20).contains(&hour_of_day);

        let town = &self.settlements[s as usize];
        let home_pos = town.pos;
        let free: Vec<PersonId> = town.residents.iter().copied().filter(|&p| self.busy_until[p as usize] <= start).collect();
        if free.is_empty() {
            return None;
        }
        let pull: Vec<f32> = free.iter().map(|&p| self.people[p as usize].traits.wanderlust.powi(2)).collect();
        let pressure: f32 = pull.iter().sum::<f32>() * DEPARTURE_RATE * if day { 1.0 } else { NIGHT_FACTOR };
        if !rng.chance(1.0 - (-pressure).exp()) {
            return None;
        }

        // Who leads, and who comes along.
        let li = rng.weighted(&pull)?;
        let leader = free[li];
        let lt = self.people[leader as usize].traits;
        let mut members = vec![leader];
        let extra = ((rng.f32() * (1.0 + lt.sociability * 4.0)) as usize).min(5);
        let mut pool: Vec<PersonId> = free.iter().copied().filter(|&p| p != leader).collect();
        for _ in 0..extra {
            let w: Vec<f32> = pool
                .iter()
                .map(|&p| {
                    let t = self.people[p as usize].traits;
                    t.sociability * (0.3 + t.wanderlust)
                })
                .collect();
            let Some(i) = rng.weighted(&w) else { break };
            members.push(pool.swap_remove(i));
        }

        // Where to. The restless go further.
        let reach = 2500.0 + 9000.0 * lt.wanderlust;
        let pick_dest = |rng: &mut Rng, exclude: &[SettlementId]| -> Option<SettlementId> {
            let w: Vec<f32> = self
                .settlements
                .iter()
                .map(|o| {
                    if exclude.contains(&o.id) {
                        0.0
                    } else {
                        let d = o.pos.dist(town.pos);
                        o.size.sqrt() / (1.0 + (d / reach).powi(2))
                    }
                })
                .collect();
            rng.weighted(&w).map(|i| i as SettlementId)
        };
        let mut stops = vec![pick_dest(&mut rng, &[s])?];
        if rng.chance(lt.wanderlust * 0.6) {
            if let Some(second) = pick_dest(&mut rng, &[s, stops[0]]) {
                stops.push(second);
            }
        }

        let speed = members.iter().map(|&m| self.people[m as usize].race.walk_speed()).fold(f32::MAX, f32::min);
        let mut legs = Vec::new();
        let mut at = town.pos;
        let mut clock = start;
        for &d in &stops {
            let o = &self.settlements[d as usize];
            let ang = rng.f32() * std::f32::consts::TAU;
            let to = o.pos.add(V2::new(ang.cos(), ang.sin()).scale(o.radius() * rng.range(0.5, 1.1)));
            let arrive = clock + at.dist(to) as f64 / speed as f64;
            legs.push(Leg { from: at, to, depart: clock, arrive, dest: Some(d) });
            let stay = HOUR * (2.0 + rng.f64() * (4.0 + 18.0 * (1.0 - lt.wanderlust as f64)));
            clock = arrive + stay;
            at = to;
        }
        let arrive = clock + at.dist(town.pos) as f64 / speed as f64;
        legs.push(Leg { from: at, to: town.pos, depart: clock, arrive, dest: Some(s) });

        let id = self.next_group;
        self.next_group += 1;
        Some(Group {
            id,
            seed: rng::key(&[self.seed, s as u64, h as u64, 0x4752_5550]),
            members,
            kind: Kind::Journey { home: s },
            legs,
            speed,
            ends: arrive,
            written: 0,
            pos: home_pos,
            last_update: start,
            band: 3,
        })
    }

    /// Clock text, e.g. "Day 3, 14:05". Day 1 begins at midnight.
    pub fn clock(&self) -> String {
        let day = (self.time / DAY).floor() as i64 + 1;
        let secs = self.time.rem_euclid(DAY);
        format!("Day {}, {:02}:{:02}", day, (secs / HOUR) as i64, ((secs % HOUR) / 60.0) as i64)
    }

    pub fn on_road(&self) -> usize {
        self.groups.iter().map(|g| g.members.len()).sum()
    }
}
