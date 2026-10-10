//! Bandit camps and the travellers who pass them — fights that happen
//! whether or not anyone is watching.
//!
//! How this stays honest across the bands:
//!
//! - **When** an ambush happens is worked out from the schedules, not by
//!   looking around each step. Every leg a traveller will walk is checked,
//!   as soon as it is written, against every camp: the exact moment the path
//!   first comes within sight of the camp is a line meeting a circle. Legs
//!   are written a couple of hours ahead, so every ambush is known before it
//!   happens.
//! - **Whether** the bandits attack is a strength-against-strength call: the
//!   camp's combined might against the travellers', with a dice roll keyed to
//!   that camp, those travellers and that moment.
//! - **How it goes** is decided by the very same fight rules used beside you:
//!   the whole fight is run, tick by tick, the moment it starts. That takes
//!   well under a millisecond, so there's no need for a cruder far-away version
//!   that could disagree with the close-up one. If you're near, you watch a
//!   copy play out in real time; the outcome is already fixed and identical.
//! - **What follows** is written back to the people and the schedules at the
//!   moment the fight ends: wounds (which heal on their own clock), deaths,
//!   belongings dropped by the dead, and the survivors' new plans — robbed
//!   travellers wait until they can stand, then turn for home.
//!
//! Hours, ambushes and fight endings are handled strictly in time order, so
//! however coarsely the world is stepped, events happen in the same order.

use serde::{Deserialize, Serialize};

use super::body;
use super::combat::{Battle, Fighter, Side};
use super::geo::{self, V2};
use super::group::{GroupId, Kind, Leg};
use super::names;
use super::person::PersonId;
use super::rng::{self, Rng};
use super::settlement::SettlementId;
use super::squad::formation;
use super::world::{World, HOUR};

/// How far a camp's lookouts see travellers coming, metres.
pub const CAMP_SIGHT: f32 = 110.0;
/// How far ahead travellers' legs are written (and checked), game seconds.
pub const LOOKAHEAD: f64 = 2.0 * HOUR;
/// How long a camp rests after a fight before it tries again.
pub const CAMP_REST: f64 = 4.0 * HOUR;
/// Bandits camped across the world at the start.
pub const CAMPS: usize = 10;

const BANDIT_SIDE: Side = 1;
const TRAVELLER_SIDE: Side = 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camp {
    pub group: GroupId,
    pub pos: V2,
    /// Not looking for trouble before this time.
    pub ready_at: f64,
    /// The campfire is out until this time (doused by magic).
    #[serde(default)]
    pub doused_until: f64,
}

/// When a camp first sees someone walking this leg: within `CAMP_SIGHT`, or
/// after dark (when travellers carry torches) within `torch::TORCH_SEEN`.
pub fn camp_sees(leg: &Leg, camp: V2) -> Option<f64> {
    use super::torch::{TORCH_SEEN, TRAVEL_TORCH_DARK};
    let near = leg.first_within(camp, CAMP_SIGHT);
    let far = leg.first_within(camp, TORCH_SEEN).filter(|&t| t >= leg.depart && t < leg.arrive && super::stealth::daylight(t) < TRAVEL_TORCH_DARK);
    match (near, far) {
        (Some(n), Some(f)) => Some(n.min(f)),
        (n, f) => n.or(f),
    }
}

/// A traveller group coming within sight of a camp, worked out in advance.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Encounter {
    pub t: f64,
    pub camp: GroupId,
    pub victim: GroupId,
}

/// A fight away from the squad, already run to its end.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NpcFight {
    pub id: u32,
    pub ends: f64,
    pub result: Battle,
    pub camp: GroupId,
    pub victim: GroupId,
    /// The victim's legs from the interrupted one onwards, as they were.
    pub was: Vec<Leg>,
    /// Legs the victim had planned ahead that the fight threw away.
    pub dropped: u64,
    /// Where on the interrupted leg's path the fight broke out.
    pub cut: usize,
}

impl World {
    // ---- Placing camps ------------------------------------------------------

    /// Set up the world's bandit camps: beside the roads, away from the towns
    /// and from where the squad starts.
    pub(super) fn place_camps(&mut self) {
        let mut r = Rng::from_keys(&[self.seed, 0xCA4B5]);
        let mut placed: Vec<V2> = Vec::new();
        let mut tries = 0;
        while placed.len() < CAMPS && tries < 4000 && !self.routes.roads.is_empty() {
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
            let off = V2::new(-d.y / len, d.x / len).scale(side * r.range(45.0, 90.0));
            let at = a.lerp(b, r.f32()).add(off);
            let clear_of_towns = self.settlements.iter().all(|s| s.pos.dist(at) > s.radius() + 900.0);
            let ok = geo::is_land(at)
                && geo::inland(at) > 150.0
                && self.terrain.slope(at) < 0.2
                && clear_of_towns
                && at.dist(self.squad.pos) > 1500.0
                && placed.iter().all(|p| p.dist(at) > 1500.0);
            if !ok {
                continue;
            }
            let n = 3 + r.below(4);
            let mage = r.chance(0.4);
            self.add_bandits(at, n, mage, false);
            placed.push(at);
        }
    }

    // ---- Finding encounters -------------------------------------------------

    /// Check legs `from..` of a group against every camp.
    pub(super) fn scan_legs(&mut self, gid: GroupId, from: usize) {
        let Some(g) = self.group(gid) else { return };
        if g.hostile {
            return;
        }
        let mut found = Vec::new();
        for leg in &g.legs[from.min(g.legs.len())..] {
            // (A ruin's wardens keep to their ruin: they guard, they don't raid.)
            for c in self.camps.iter().filter(|c| !self.is_warden(c.group)) {
                if let Some(t) = camp_sees(leg, c.pos) {
                    found.push(Encounter { t, camp: c.group, victim: gid });
                }
            }
        }
        self.encounters.extend(found);
    }

    /// A new camp checks every leg already planned.
    pub(super) fn scan_camp(&mut self, ci: usize) {
        let c = self.camps[ci].clone();
        if self.is_warden(c.group) {
            return;
        }
        let mut found = Vec::new();
        for g in self.groups.iter().filter(|g| !g.hostile) {
            for leg in &g.legs {
                if let Some(t) = camp_sees(leg, c.pos) {
                    if t >= self.time {
                        found.push(Encounter { t, camp: c.group, victim: g.id });
                    }
                }
            }
        }
        self.encounters.extend(found);
    }

    /// Write every wanderer's legs far enough ahead, check the new ones, and
    /// forget old ones.
    pub(super) fn plan_ahead(&mut self) {
        let until = self.time + LOOKAHEAD;
        let now = self.time;
        let mut fresh: Vec<(GroupId, usize)> = Vec::new();
        for g in &mut self.groups {
            if self.fighting_groups.contains(&g.id) {
                continue;
            }
            let before = g.legs.len();
            let added = g.extend_to(until, &self.terrain);
            if added > 0 {
                fresh.push((g.id, before));
            }
        }
        for (gid, from) in fresh {
            self.scan_legs(gid, from);
        }
        for g in &mut self.groups {
            g.trim(now);
        }
    }

    // ---- The timeline -------------------------------------------------------

    /// The earliest pending ambush or fight ending, if any is due by `t`.
    pub(super) fn next_event(&self) -> Option<(f64, bool, usize)> {
        let enc = self
            .encounters
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.t.total_cmp(&b.t).then(a.camp.cmp(&b.camp)).then(a.victim.cmp(&b.victim)))
            .map(|(i, e)| (e.t, false, i));
        let end = self.npc_fights.iter().enumerate().min_by(|(_, a), (_, b)| a.ends.total_cmp(&b.ends).then(a.id.cmp(&b.id))).map(|(i, f)| (f.ends, true, i));
        match (enc, end) {
            (Some(e), Some(f)) => Some(if f.0 <= e.0 { f } else { e }),
            (a, b) => a.or(b),
        }
    }

    pub(super) fn run_event(&mut self, is_end: bool, i: usize) {
        if is_end {
            let f = self.npc_fights.remove(i);
            self.finish_npc_fight(f);
        } else {
            let e = self.encounters.remove(i);
            self.ambush(e);
        }
    }

    fn fit(&self, pid: PersonId, t: f64) -> bool {
        let p = &self.people[pid as usize];
        !p.dead && !self.fighting.contains_key(&pid) && !body::knocked_out(&p.wounds.hp_at(&p.stats, t))
    }

    // ---- An ambush ----------------------------------------------------------

    fn ambush(&mut self, e: Encounter) {
        let t = e.t;
        let Some(ci) = self.camps.iter().position(|c| c.group == e.camp) else { return };
        let camp = self.camps[ci].clone();
        if camp.ready_at > t {
            return;
        }
        let (Some(cg), Some(vg)) = (self.group(e.camp), self.group(e.victim)) else { return };
        if self.fighting_groups.contains(&vg.id) || t >= vg.ends {
            return;
        }
        let at = vg.position_at(t);
        let sight = if self.group_torch_lit(vg, t) { super::torch::TORCH_SEEN } else { CAMP_SIGHT };
        if at.dist(camp.pos) > sight + 1.0 {
            return; // the plan this was found on has since changed
        }
        let bandits: Vec<PersonId> = cg.members.iter().copied().filter(|&m| self.fit(m, t)).collect();
        let victims: Vec<PersonId> = vg.members.iter().copied().filter(|&m| self.fit(m, t)).collect();
        if bandits.is_empty() || victims.is_empty() {
            return;
        }

        // Strength against strength, with a touch of luck: bandits only jump
        // people they think they can take.
        let might = |who: &[PersonId]| who.iter().map(|&m| self.people[m as usize].might).sum::<f32>();
        let (mb, mv) = (might(&bandits), might(&victims));
        let roll = Rng::from_keys(&[self.seed, e.camp as u64, e.victim as u64, t.to_bits(), 0xA3B0]).f32();
        if mb * (0.7 + 0.6 * roll) < mv {
            return;
        }

        // The fight, with everyone standing where they are.
        let id = self.next_battle;
        self.next_battle += 1;
        let seed = rng::key(&[self.seed, e.camp as u64, e.victim as u64, t.to_bits(), 0xBA77]);
        let mut fighters = Vec::new();
        let mut names = Vec::new();
        for (k, &m) in bandits.iter().enumerate() {
            fighters.push(Fighter::from_person(&self.people[m as usize], BANDIT_SIDE, camp.pos.add(formation(k)), t));
            names.push(self.name_of(m));
        }
        // Travellers caught at night have their torch lit (the leader carries it).
        let lit = super::stealth::daylight(t) < super::torch::TRAVEL_TORCH_DARK;
        for (k, &m) in victims.iter().enumerate() {
            let mut f = Fighter::from_person(&self.people[m as usize], TRAVELLER_SIDE, at.add(formation(k)), t);
            f.torch = lit && k == 0;
            fighters.push(f);
            names.push(self.name_of(m));
        }
        let mut b = Battle::new(id, seed, t, fighters, names);
        b.lights = Some(self.fixed_lights_at(t));
        for &m in bandits.iter().chain(&victims) {
            self.fighting.insert(m, id);
        }
        let mut result = b.clone();
        while !result.over {
            result.tick();
        }

        // The travellers' plans stop here for now.
        let gi = self.groups.iter().position(|g| g.id == e.victim).unwrap();
        let g = &mut self.groups[gi];
        let li = g.legs.iter().rposition(|l| t >= l.depart).unwrap_or(0);
        let was: Vec<Leg> = g.legs[li..].to_vec();
        let (cut, k) = if t < g.legs[li].arrive { g.legs[li].cut_at(t) } else { (Leg::stay(at, g.legs[li].arrive, t), 0) };
        let dropped = if matches!(g.kind, Kind::Wanderer { .. }) { (g.legs.len() - li - 1) as u64 } else { 0 };
        g.legs.truncate(li);
        if t < was[0].arrive {
            g.legs.push(cut);
        } else {
            g.legs.push(was[0].clone());
        }
        g.legs.push(Leg::stay(at, t, f64::INFINITY));
        g.written -= dropped;
        g.ends = f64::INFINITY;
        for &m in &g.members.clone() {
            self.busy_until[m as usize] = f64::INFINITY;
        }
        self.encounters.retain(|x| x.victim != e.victim || x.t <= t);
        self.fighting_groups.insert(e.victim);
        self.fighting_groups.insert(e.camp);
        self.camps[ci].ready_at = f64::INFINITY;
        self.stats.ambushes += 1;

        if self.bands.band_at(at) <= 2 {
            let line = format!("Bandits fall on travellers {:.0} m away.", at.dist(self.squad.pos));
            self.log.push_front((t, line));
            self.log.truncate(14);
        }

        // Close by, a copy plays out in real time; it ends exactly as the
        // result already says.
        let mut show = b;
        show.advance_to(self.time);
        self.battles.push(show);
        self.npc_fights.push(NpcFight { id, ends: result.time, result, camp: e.camp, victim: e.victim, was, dropped, cut: k });
    }

    pub fn name_of(&self, pid: PersonId) -> String {
        let p = &self.people[pid as usize];
        p.name().map(|s| s.to_string()).unwrap_or_else(|| names::person_name(p.race, p.seed))
    }

    // ---- After a fight ------------------------------------------------------

    fn finish_npc_fight(&mut self, f: NpcFight) {
        let t = f.ends;
        self.battles.retain(|b| b.id != f.id);
        self.fighting_groups.remove(&f.victim);
        self.fighting_groups.remove(&f.camp);
        let killed = self.write_back(&f.result);
        // Beaten travellers lose what they were carrying.
        if !f.result.fighters.iter().any(|x| x.side == TRAVELLER_SIDE && x.active()) {
            self.rob_cargo(f.victim);
        }

        // When someone knocked down can stand again.
        let up_at = |w: &World, m: PersonId| -> f64 {
            let p = &w.people[m as usize];
            let hp = p.wounds.hp_at(&p.stats, t);
            let need = [body::Part::Head, body::Part::Torso]
                .iter()
                .map(|&part| (1.0 - hp[part as usize]).max(0.0))
                .fold(0.0f32, f32::max);
            t + need as f64 / body::HEAL_PER_HOUR as f64 * HOUR
        };
        let centre = |side: Side, b: &Battle, w: &World| -> Option<V2> {
            let alive: Vec<V2> = b.fighters.iter().filter(|x| x.side == side && x.is_person() && !w.people[x.pid as usize].dead).map(|x| x.pos).collect();
            (!alive.is_empty()).then(|| alive.iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / alive.len() as f32))
        };

        // The travellers: bury the dead, wait for the fallen, go on.
        if let Some(gi) = self.groups.iter().position(|g| g.id == f.victim) {
            let members = self.groups[gi].members.clone();
            let (alive, dead): (Vec<PersonId>, Vec<PersonId>) = members.iter().partition(|&&m| !self.people[m as usize].dead);
            for &m in &dead {
                if self.group_of[m as usize] == Some(f.victim) {
                    self.group_of[m as usize] = None;
                }
            }
            let c = centre(TRAVELLER_SIDE, &f.result, self).unwrap_or(f.was[0].from);
            let resume = alive.iter().map(|&m| up_at(self, m)).fold(t, f64::max) + 600.0;
            let g = &mut self.groups[gi];
            g.members = alive.clone();
            let n = g.legs.len();
            if alive.is_empty() {
                g.legs[n - 1] = Leg::stay(c, t, t);
                g.ends = t;
            } else {
                g.legs[n - 1] = Leg::stay(c, t, resume);
                let speed = g.speed;
                let kind = g.kind.clone();
                let from_leg = n;
                let mut more = Vec::new();
                if let Kind::Journey { home } = kind {
                    more = self.way_home(&f, home, c, resume, speed);
                }
                let g = &mut self.groups[gi];
                g.legs.extend(more);
                if let Kind::Journey { .. } = g.kind {
                    g.ends = g.legs.last().unwrap().arrive;
                } else {
                    g.extend_to(self.time.max(resume) + LOOKAHEAD, &self.terrain);
                }
                let ends = g.ends;
                for &m in &alive {
                    self.busy_until[m as usize] = ends;
                }
                self.scan_legs(f.victim, from_leg);
            }
        }

        // The bandits: back to camp, and rest a while.
        if let Some(ci) = self.camps.iter().position(|c| c.group == f.camp) {
            let camp_pos = self.camps[ci].pos;
            if let Some(gi) = self.groups.iter().position(|g| g.id == f.camp) {
                let alive: Vec<PersonId> = self.groups[gi].members.iter().copied().filter(|&m| !self.people[m as usize].dead).collect();
                if alive.is_empty() {
                    self.groups[gi].ends = t;
                    self.groups[gi].members.clear();
                    self.camps.remove(ci);
                } else {
                    let c = centre(BANDIT_SIDE, &f.result, self).unwrap_or(camp_pos);
                    let back = alive.iter().map(|&m| up_at(self, m)).fold(t, f64::max);
                    let speed = self.groups[gi].speed;
                    let walk = Leg::straight(c, camp_pos, back, speed, None, &self.terrain);
                    let home = walk.arrive;
                    let g = &mut self.groups[gi];
                    g.members = alive;
                    g.legs = vec![walk, Leg::stay(camp_pos, home, f64::INFINITY)];
                    self.camps[ci].ready_at = home + CAMP_REST;
                }
            }
        }

        if self.bands.band_at(f.result.fighters.first().map(|x| x.pos).unwrap_or_default()) <= 2 {
            let line = if killed > 0 { format!("The roadside fight is over. {killed} dead.") } else { "The roadside fight is over.".to_string() };
            self.log.push_front((t, line));
            self.log.truncate(14);
        }
    }

    /// Robbed travellers give up on where they were going and head home: on
    /// along the road if they were already homeward, else back the way they
    /// came and then home by road.
    fn way_home(&self, f: &NpcFight, home: SettlementId, from: V2, at: f64, speed: f32) -> Vec<Leg> {
        let leg = &f.was[0];
        let home_pos = self.settlements[home as usize].pos;
        if leg.dest == Some(home) {
            let mut path = vec![from];
            path.extend_from_slice(&leg.path[f.cut.min(leg.path.len())..]);
            if path.len() < 2 {
                path.push(home_pos);
            }
            return vec![Leg::along(path, at, speed, Some(home), &self.terrain)];
        }
        let mut back = vec![from];
        back.extend(leg.path[..f.cut.min(leg.path.len())].iter().rev());
        if back.len() < 2 {
            back.push(leg.from);
        }
        // Which town that path started from.
        let start_town = self
            .settlements
            .iter()
            .min_by(|a, b| a.pos.dist(leg.from).total_cmp(&b.pos.dist(leg.from)))
            .map(|s| s.id)
            .unwrap_or(home);
        let first = Leg::along(back, at, speed, if start_town == home { Some(home) } else { None }, &self.terrain);
        if start_town == home {
            return vec![first];
        }
        let mut road = self.routes.between(start_town, home);
        if road.len() < 2 {
            road = vec![first.to, home_pos];
        }
        road[0] = first.to;
        let last = road.len() - 1;
        road[last] = home_pos;
        let second = Leg::along(road, first.arrive, speed, Some(home), &self.terrain);
        vec![first, second]
    }
}
