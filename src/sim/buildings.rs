//! Going indoors: doors, locks, lockpicking, and what's inside.
//!
//! Every enterable building has one door, worked out from the building
//! itself (so it needs no storing). Buildings are solid: squad members walk
//! around them, and in and out only through the door. Inside, the building
//! is the same place on the map; the window cuts its walls and roof away so
//! you can see in.
//!
//! **Locks.** Most homes have a lock, from flimsy to stout. Doors are locked
//! from 20:00 to 06:00 and open by day. A squad member with a lockpick can
//! try to pick one: every few seconds an attempt, its chance set by Security
//! and Agility against the lock; a failed attempt may snap the pick. Picking
//! trains Security. A picked door stays open until the next night.
//!
//! **Being seen.** Townsfolk nearby may see someone picking a lock or taking
//! what isn't theirs, judged by the same light-and-sneaking rules as bandits'
//! lookouts. If they do, the attempt stops and you gain a bounty in that town.
//!
//! **Insides.** The first time anyone in the squad steps into a building, its
//! belongings are laid out from the building's seed: a few things on the
//! shelves and in the chest. They're owned; taking them is theft.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items;
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::{Building, BuildingKind, Settlement, SettlementId};
use super::stats::{Attr, Skill};
use super::stealth;
use super::world::{World, DAY, HOUR};

/// A door: the town, and the building's index in it.
pub type DoorId = (SettlementId, u16);

/// Trips longer than this (metres) consider the roads.
pub const ROAD_TRIP: f32 = 600.0;
/// Seconds per lockpicking attempt.
pub const PICK_TIME: f64 = 4.0;
/// How close to the door you must stand to work the lock, metres.
pub const AT_DOOR: f32 = 1.5;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Door {
    pub id: DoorId,
    /// Just outside the door.
    pub outside: V2,
    /// Just inside it.
    pub inside: V2,
    /// The building's middle and how far its walls reach (a circle).
    pub centre: V2,
    pub radius: f32,
    /// 0 = no lock; otherwise how hard to pick, 1..100.
    pub lock: f32,
}

/// The door of a building, if it has one you can walk through. Stilt homes
/// are reached by swimming and have none (yet); hearths aren't buildings.
pub fn door_of(s: &Settlement, i: u16) -> Option<Door> {
    let b = s.buildings.get(i as usize)?;
    let (radius, front) = match b.kind {
        BuildingKind::RoduroHome => (b.size * 0.5, b.size * 0.5),
        BuildingKind::QotiroBlock | BuildingKind::QotiroHall => (b.size * 0.48, b.size * 0.4),
        BuildingKind::QotiroTemple => (b.size * 0.5, b.size * 0.5),
        BuildingKind::HoraroStilt | BuildingKind::Hearth => return None,
    };
    let dir = V2::new(b.rot.cos(), b.rot.sin());
    let side = V2::new(-dir.y, dir.x);
    // Roduro doors sit a little to one side of the window.
    let shift = if b.kind == BuildingKind::RoduroHome { 1.4 } else { 0.0 };
    let face = b.pos.add(dir.scale(front)).add(side.scale(shift));
    Some(Door { id: (s.id, i), outside: face.add(dir.scale(1.2)), inside: face.sub(dir.scale(1.6)), centre: b.pos, radius, lock: lock_of(b) })
}

fn lock_of(b: &Building) -> f32 {
    let mut r = Rng::from_keys(&[b.seed, 0x4C4F_434B]);
    match b.kind {
        BuildingKind::RoduroHome if r.chance(0.7) => 10.0 + r.f32() * 50.0,
        BuildingKind::QotiroBlock | BuildingKind::QotiroHall => 25.0 + r.f32() * 40.0,
        BuildingKind::QotiroTemple => 70.0,
        _ => 0.0,
    }
}

/// Which night it is (20:00 one day to 06:00 the next share a number).
pub fn night_of(t: f64) -> i64 {
    ((t + 4.0 * HOUR) / DAY).floor() as i64
}

pub fn is_night(t: f64) -> bool {
    let h = t.rem_euclid(DAY) / HOUR;
    !(6.0..20.0).contains(&h)
}

/// Someone at work on a lock.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Picking {
    pub who: PersonId,
    pub door: DoorId,
    /// Attempts made so far.
    pub tries: u32,
    /// When the next attempt finishes (once they're at the door).
    pub next: Option<f64>,
}

impl World {
    pub fn door(&self, id: DoorId) -> Option<Door> {
        door_of(self.settlements.get(id.0 as usize)?, id.1)
    }

    /// Doors of every building near `p`.
    pub fn doors_near(&self, p: V2, within: f32) -> Vec<Door> {
        let mut out = Vec::new();
        for s in self.settlements.iter().filter(|s| s.pos.dist(p) < s.reach + within) {
            for i in 0..s.buildings.len() as u16 {
                if let Some(d) = door_of(s, i) {
                    if d.centre.dist(p) < d.radius + within {
                        out.push(d);
                    }
                }
            }
        }
        out
    }

    /// The building whose walls enclose `p`, if any.
    pub fn building_at(&self, p: V2) -> Option<Door> {
        self.doors_near(p, 0.0).into_iter().find(|d| d.centre.dist(p) < d.radius)
    }

    pub fn is_locked(&self, id: DoorId) -> bool {
        let Some(d) = self.door(id) else { return false };
        d.lock > 0.0 && is_night(self.time) && self.picked.get(&id) != Some(&night_of(self.time))
    }

    /// The way from `a` to `b` on foot: around buildings, and through doors
    /// to get in or out. Ends at `b`, or at a locked door (which it names).
    pub fn route(&self, a: V2, b: V2) -> (Vec<V2>, Option<DoorId>) {
        let from_in = self.building_at(a);
        let to_in = self.building_at(b);
        if let (Some(x), Some(y)) = (from_in, to_in) {
            if x.id == y.id {
                return (vec![b], None);
            }
        }
        let mut out = Vec::new();
        let mut cur = a;
        if let Some(d) = from_in {
            out.push(d.inside);
            out.push(d.outside);
            cur = d.outside;
        }
        let dest = to_in.map(|d| d.outside).unwrap_or(b);
        self.detour(cur, dest, &mut out, 0);
        out.push(dest);
        if let Some(d) = to_in {
            if self.is_locked(d.id) {
                return (out, Some(d.id));
            }
            out.push(d.inside);
            out.push(b);
        }
        (out, None)
    }

    /// The way to go a long way: along the roads if that's quicker overall
    /// than striking out across country, else straight (round buildings and
    /// through doors as usual). Short trips always go straight.
    pub fn travel(&self, a: V2, b: V2) -> (Vec<V2>, Option<DoorId>) {
        if a.dist(b) < ROAD_TRIP {
            return self.route(a, b);
        }
        let (Some(na), Some(nb)) = (self.routes.nearest_node(a), self.routes.nearest_node(b)) else { return self.route(a, b) };
        let cross = self.overland_effort(a, b);
        let Some((road, along)) = self.routes.along_roads(na, nb) else { return self.route(a, b) };
        let (ra, rb) = (self.routes.nodes[na as usize], self.routes.nodes[nb as usize]);
        let by_road = self.overland_effort(a, ra) + along + self.overland_effort(rb, b);
        if by_road >= cross {
            return self.route(a, b);
        }
        let (mut path, _) = self.route(a, ra);
        path.extend(road.iter().skip(1).copied());
        let (tail, locked) = self.route(rb, b);
        path.extend(tail);
        (path, locked)
    }

    /// Walking effort straight across the land; the sea can't be crossed.
    fn overland_effort(&self, a: V2, b: V2) -> f32 {
        let n = ((a.dist(b) / 50.0).ceil() as usize).max(1);
        let mut total = 0.0;
        for k in 0..n {
            let (p, q) = (a.lerp(b, k as f32 / n as f32), a.lerp(b, (k + 1) as f32 / n as f32));
            if !super::geo::is_land(q) {
                return f32::INFINITY;
            }
            total += self.terrain.effort(p, q);
        }
        total
    }

    /// Add corner points so the line `p`→`q` goes round buildings.
    fn detour(&self, p: V2, q: V2, out: &mut Vec<V2>, depth: u8) {
        if depth > 4 || p.dist(q) < 0.5 {
            return;
        }
        let mid = p.lerp(q, 0.5);
        let hit = self
            .doors_near(mid, p.dist(q) * 0.5 + 2.0)
            .into_iter()
            .filter(|d| d.centre.dist(p) > d.radius + 0.3 && d.centre.dist(q) > d.radius + 0.3)
            .filter(|d| seg_dist(p, q, d.centre) < d.radius + 0.8)
            .min_by(|x, y| x.centre.dist(p).total_cmp(&y.centre.dist(p)));
        let Some(d) = hit else { return };
        let dir = q.sub(p);
        let len = dir.len().max(1e-3);
        let n = V2::new(-dir.y / len, dir.x / len);
        let side = if d.centre.sub(p).x * n.x + d.centre.sub(p).y * n.y > 0.0 { -1.0 } else { 1.0 };
        let corner = d.centre.add(n.scale(side * (d.radius + 1.8)));
        self.detour(p, corner, out, depth + 1);
        out.push(corner);
        self.detour(corner, q, out, depth + 1);
    }

    // ---- Lockpicking --------------------------------------------------------

    /// Send someone to pick a lock.
    pub fn order_pick(&mut self, who: PersonId, door: DoorId) -> bool {
        let Some(d) = self.door(door) else { return false };
        if !self.is_locked(door) {
            return false;
        }
        let Some(k) = self.squad.index(who) else { return false };
        let from = self.member_pos(k);
        let (mut path, _) = self.route(from, d.outside);
        if path.is_empty() {
            path.push(d.outside);
        }
        self.squad.goal[k] = d.outside;
        self.squad.route[k] = path;
        self.pickups.retain(|p| p.who != who);
        self.picking.retain(|p| p.who != who);
        self.picking.push(Picking { who, door, tries: 0, next: None });
        true
    }

    /// Chance one attempt opens the lock.
    pub fn pick_chance(&self, who: PersonId, lock: f32) -> f32 {
        let s = self.people[who as usize].effective_stats();
        (0.35 + (s.skill(Skill::Security) + s.attr(Attr::Agility) * 0.2 - lock) * 0.018).clamp(0.03, 0.95)
    }

    pub(super) fn do_picking(&mut self) {
        let lockpick = items::id("lockpick");
        let mut done = Vec::new();
        for n in 0..self.picking.len() {
            let pk = self.picking[n];
            let (Some(k), Some(d)) = (self.squad.index(pk.who), self.door(pk.door)) else {
                done.push(n);
                continue;
            };
            if !self.is_locked(pk.door) {
                done.push(n);
                continue;
            }
            if self.squad.at[k].dist(d.outside) > AT_DOOR || self.fighting.contains_key(&pk.who) {
                continue;
            }
            let name = self.people[pk.who as usize].name().unwrap_or("someone").to_string();
            let has_pick = |w: &World| w.people[pk.who as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == lockpick)).unwrap_or(false);
            if !has_pick(self) {
                self.log.push_front((self.time, format!("{name} has no lockpicks.")));
                done.push(n);
                continue;
            }
            let mut next = pk.next.unwrap_or(self.time + PICK_TIME);
            let mut tries = pk.tries;
            let mut finished = false;
            while next <= self.time && !finished {
                tries += 1;
                let mut r = Rng::from_keys(&[self.seed, pk.who as u64, pk.door.0 as u64, pk.door.1 as u64, tries as u64, 0x5049_434B]);
                let (r_ok, r_break) = (r.f32(), r.f32());
                self.people[pk.who as usize].stats.exercise(Skill::Security, 1.0);
                if self.witnessed(pk.who, d.outside, pk.door.0, &mut r) {
                    self.crime(pk.door.0, 40.0, format!("{name} is seen picking a lock!"));
                    finished = true;
                } else if r_ok < self.pick_chance(pk.who, d.lock) {
                    self.picked.insert(pk.door, night_of(self.time));
                    self.log.push_front((next, format!("{name} picks the lock.")));
                    finished = true;
                } else if r_break < 0.3 {
                    if let Some(dd) = self.people[pk.who as usize].detail.as_mut() {
                        dd.gear.take(lockpick);
                    }
                    self.log.push_front((next, format!("{name}'s lockpick snaps.")));
                    if !has_pick(self) {
                        finished = true;
                    }
                }
                next += PICK_TIME;
            }
            self.log.truncate(14);
            if finished {
                done.push(n);
            } else {
                self.picking[n].tries = tries;
                self.picking[n].next = Some(next);
            }
        }
        for n in done.into_iter().rev() {
            self.picking.remove(n);
        }
    }

    // ---- Witnesses and bounties ---------------------------------------------

    /// Does anyone in town see this squad member at `at`? Uses the same
    /// light and sneaking rules as lookouts, with one keyed roll.
    pub(super) fn witnessed(&self, who: PersonId, at: V2, town: SettlementId, r: &mut Rng) -> bool {
        let roll = r.f32();
        let sight = stealth::SIGHT * 0.6 * self.visibility_of(who);
        let closest = self
            .residents_in_band1(town)
            .into_iter()
            .map(|p| self.person_pos(p).dist(at))
            .fold(f32::MAX, f32::min);
        if closest > sight {
            return false;
        }
        roll < 0.6 * (1.0 - closest / sight.max(0.1)) + 0.2
    }

    pub(super) fn crime(&mut self, town: SettlementId, amount: f32, line: String) {
        *self.bounty.entry(town).or_insert(0.0) += amount;
        self.crime_known(town);
        let total = self.bounty[&town];
        self.log.push_front((self.time, format!("{line} Bounty in {}: {total:.0}.", self.settlements[town as usize].name)));
        self.log.truncate(14);
        self.alerts.push(line);
    }

    // ---- What's inside ------------------------------------------------------

    /// Lay out a building's belongings the first time the squad goes in.
    pub(super) fn furnish(&mut self, d: Door) {
        if !self.furnished.insert(d.id) {
            return;
        }
        let b = self.settlements[d.id.0 as usize].buildings[d.id.1 as usize].clone();
        let mut r = Rng::from_keys(&[b.seed, 0x4655_524E]);
        let common: &[&str] = &[
            "cloth_shirt", "trousers", "knife", "club", "leather_cap", "boots", "small_pack", "lockpick", "padded_jacket", "leather_gloves", "hide", "reed_paper", "squid_ink", "timber",
            "kelp_frond", "healing_draught", "iron_ingot", "leather", "flatbread", "dried_fish", "salted_meat", "flatbread",
        ];
        let better: &[&str] = &["short_sword", "hide_coat", "war_pick", "spear", "buckler", "hide_leggings", "iron_helm"];
        let rare: &[&str] = &["ring_swiftness", "ring_might", "amulet_wellspring", "amulet_clear_mind", "ring_hearth", "seers_hood", "striders_boots", "duelists_gloves"];
        let n = 1 + r.below(3) + if b.kind == BuildingKind::QotiroTemple { 3 } else { 0 };
        let back = d.centre.sub(d.inside);
        for k in 0..n {
            let key = if r.chance(0.06) {
                *r.pick(rare)
            } else if r.chance(0.3) {
                *r.pick(better)
            } else {
                *r.pick(common)
            };
            let a = (k as f32 - n as f32 / 2.0) * 0.7;
            let spot = d.centre.add(V2::new(back.x * a.cos() - back.y * a.sin(), back.x * a.sin() + back.y * a.cos()).scale(0.55));
            let id = self.put_on_ground(items::id(key), 1, spot);
            if let Some(g) = self.ground.iter_mut().find(|g| g.id == id) {
                g.owner = Some(d.id.0);
            }
        }
    }

    /// Note who has stepped indoors, and furnish places on first visit.
    pub(super) fn update_indoors(&mut self) {
        for k in 0..self.squad.members.len() {
            let p = self.squad.at[k];
            let inside = self.building_at(p);
            self.squad.inside[k] = inside.map(|d| d.id);
            if let Some(d) = inside {
                self.furnish(d);
            }
        }
    }

    /// The building any squad member is in (the window cuts these open).
    pub fn occupied(&self) -> Vec<DoorId> {
        let mut v: Vec<DoorId> = self.squad.inside.iter().flatten().copied().collect();
        v.sort();
        v.dedup();
        v
    }
}

/// Distance from `c` to the segment `a`–`b`.
fn seg_dist(a: V2, b: V2, c: V2) -> f32 {
    let ab = b.sub(a);
    let l2 = ab.x * ab.x + ab.y * ab.y;
    if l2 < 1e-6 {
        return a.dist(c);
    }
    let t = (((c.x - a.x) * ab.x + (c.y - a.y) * ab.y) / l2).clamp(0.0, 1.0);
    a.lerp(b, t).dist(c)
}

