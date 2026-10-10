//! Your squad: each member walks on their own feet, Kenshi-style.
//!
//! Every member has their own position and destination. An order to "the
//! squad" sends everyone to a spot in loose formation; an order to selected
//! members moves only them. Each walks at their own pace — slowed by hills,
//! injured legs and an overloaded pack — so the squad strings out on a long
//! climb. The squad's centre (the middle of everyone still standing) is what
//! the bands are measured from.
//!
//! Also here: what members do with things — equipping, dropping, picking up.

use serde::{Deserialize, Serialize};

use super::body;
use super::condition::Activity;
use super::buildings::DoorId;
use super::geo::{self, V2};
use super::inventory;
use super::items::{item, ItemId, Slot};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::terrain::walk_factor;
use super::world::World;

/// Walking pace on flat ground for a fit, unburdened member, m/s.
pub const SQUAD_SPEED: f32 = 1.5;
/// How close someone must be to pick something up, metres.
pub const REACH: f32 = 1.8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Squad {
    pub members: Vec<PersonId>,
    /// Where each member is (parallel to `members`).
    pub at: Vec<V2>,
    /// Where each member is heading.
    pub goal: Vec<V2>,
    /// The middle of everyone still standing.
    pub pos: V2,
    /// The last place the whole squad was sent (for drawing).
    pub target: V2,
    /// Who's sneaking (parallel to `members`).
    pub sneaking: Vec<bool>,
    /// Each member's way to their goal: corners round buildings, doors.
    pub route: Vec<Vec<V2>>,
    /// The building each member is in, if any.
    pub inside: Vec<Option<DoorId>>,
    /// Ordered to rest, or bedded down for the night (they sleep while stopped).
    pub resting: Vec<bool>,
    /// Got up during the night by order: they won't bed down again on their
    /// own until this time.
    pub kept_up: Vec<f64>,
}

/// Loose formation: the first stands on the spot, the rest in a spiral.
pub fn formation(k: usize) -> V2 {
    if k == 0 {
        return V2::default();
    }
    let k = k as f32;
    V2::new((k * 2.4).cos(), (k * 2.4).sin()).scale(1.8 + k * 0.6)
}

impl Squad {
    pub fn new(members: Vec<PersonId>, centre: V2) -> Squad {
        let at: Vec<V2> = (0..members.len()).map(|k| centre.add(formation(k))).collect();
        let n = members.len();
        Squad { goal: at.clone(), at, members, pos: centre, target: centre, sneaking: vec![false; n], route: vec![Vec::new(); n], inside: vec![None; n], resting: vec![false; n], kept_up: vec![0.0; n] }
    }

    /// Take member `k` out of the travelling squad (left at a base).
    pub fn remove(&mut self, k: usize) {
        self.members.remove(k);
        self.at.remove(k);
        self.goal.remove(k);
        self.sneaking.remove(k);
        self.route.remove(k);
        self.inside.remove(k);
        self.resting.remove(k);
        self.kept_up.remove(k);
    }

    /// Bring someone into the squad, standing at `at`.
    pub fn add(&mut self, pid: PersonId, at: V2) {
        self.members.push(pid);
        self.at.push(at);
        self.goal.push(at);
        self.sneaking.push(false);
        self.route.push(Vec::new());
        self.inside.push(None);
        self.resting.push(false);
        self.kept_up.push(0.0);
    }

    pub fn index(&self, pid: PersonId) -> Option<usize> {
        self.members.iter().position(|&m| m == pid)
    }

    /// Drop members for whom `keep` is false, keeping the parallel lists in step.
    pub fn retain(&mut self, keep: impl Fn(PersonId) -> bool) {
        let mut k = 0;
        while k < self.members.len() {
            if keep(self.members[k]) {
                k += 1;
            } else {
                self.members.remove(k);
                self.at.remove(k);
                self.goal.remove(k);
                self.sneaking.remove(k);
                self.route.remove(k);
                self.inside.remove(k);
                self.resting.remove(k);
                self.kept_up.remove(k);
            }
        }
    }
}

/// What someone has asked a squad member to do when they get there.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub who: PersonId,
    pub thing: u32,
}

/// Something lying on the ground.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GroundItem {
    pub id: u32,
    pub item: ItemId,
    pub count: u16,
    pub pos: V2,
    /// Whose it is, if anyone's: taking it is theft.
    pub owner: Option<SettlementId>,
    /// A made piece's own wear and maker's mark.
    #[serde(default)]
    pub piece: Option<super::materials::Piece>,
}

impl World {
    /// Put the whole squad somewhere instantly (tests, debugging).
    pub fn teleport_squad(&mut self, p: V2) {
        let n = self.squad.members.len();
        self.squad.at = (0..n).map(|k| p.add(formation(k))).collect();
        self.squad.goal = self.squad.at.clone();
        self.squad.route = vec![Vec::new(); n];
        self.squad.pos = p;
        self.squad.target = p;
        self.bands.update(p);
        // Everything gets re-banded from the new spot straight away.
        let t = self.time;
        for g in &mut self.groups {
            g.pos = g.position_at(t);
            g.band = self.bands.band_at(g.pos);
            g.last_update = t;
        }
    }

    /// Send the whole squad somewhere, in formation.
    pub fn order_squad(&mut self, target: V2) {
        let members = self.squad.members.clone();
        self.order_members(&members, target);
        self.squad.target = geo::clamp_to_world(target, 50.0);
    }

    /// Send some members somewhere, in formation among themselves. Mid-fight,
    /// they stop fighting until they get there.
    pub fn order_members(&mut self, who: &[PersonId], target: V2) {
        let target = geo::clamp_to_world(target, 50.0);
        let mut blocked = None;
        for (n, &pid) in who.iter().enumerate() {
            if let Some(k) = self.squad.index(pid) {
                // Indoors there's no room for a formation.
                let spot = if self.building_at(target).is_some() { target.add(formation(n).scale(0.3)) } else { target.add(formation(n)) };
                let (path, locked) = self.travel(self.member_pos(k), spot);
                blocked = blocked.or(locked);
                self.squad.goal[k] = *path.last().unwrap_or(&spot);
                self.squad.route[k] = path;
            }
            self.pickups.retain(|p| p.who != pid);
            self.picking.retain(|p| p.who != pid);
            self.gathering.retain(|g| g.0 != pid);
            self.stop_work(pid);
            if let Some(k) = self.squad.index(pid) {
                self.squad.resting[k] = false;
            }
            self.want_carry.retain(|w| w.0 != pid);
            if self.want_talk.map(|w| w.0 == pid).unwrap_or(false) {
                self.want_talk = None;
            }
        }
        if blocked.is_some() {
            self.log.push_front((self.time, "The door is locked.".to_string()));
            self.log.truncate(14);
        }
        self.order_in_battle(who, target);
    }

    /// How fast a member can walk right now, before slope.
    pub fn member_speed(&self, pid: PersonId) -> f32 {
        let p = &self.people[pid as usize];
        let gear = p.kit();
        let stats = inventory::effective(&p.stats, &gear);
        let hp = p.wounds.hp_at(&p.stats, self.time);
        let bonus = gear.worn(super::effects::Does::MoveSpeed) + self.boon(pid, super::effects::Does::MoveSpeed) + self.boon(pid, super::effects::Does::Haste);
        let sneak = if self.is_sneaking(pid) { super::stealth::SNEAK_PACE } else { 1.0 };
        let worn = p.cond.as_ref().map(|c| c.pace_factor(self.time)).unwrap_or(1.0);
        // Their own kit's weight as usual; a body they're carrying has its own pace.
        let load = self.kit_weight_at(pid, self.time) / self.capacity_at(pid, self.time).max(1.0);
        worn * sneak * SQUAD_SPEED * stats.move_factor() * body::leg_factor(&hp) * inventory::encumbrance_factor(load) * self.carry_pace(pid) * (1.0 + bonus)
    }

    /// Walk everyone a step toward their goal (members in a fight are moved by the fight).
    pub(super) fn walk_squad(&mut self, dt: f64) {
        // What each member is doing from now on (for hunger and the rest).
        for k in 0..self.squad.members.len() {
            let pid = self.squad.members[k];
            let p = &self.people[pid as usize];
            let down = p.dead || body::knocked_out(&p.wounds.hp_at(&p.stats, self.time));
            let a = if self.fighting.contains_key(&pid) {
                Activity::Fighting
            } else if !down && self.squad.at[k].dist(self.squad.goal[k]) > 1e-3 {
                Activity::Walking
            } else if self.squad.resting[k] || down {
                // Ordered to rest, or out cold: either way, they're asleep.
                Activity::Sleeping
            } else {
                Activity::Resting
            };
            self.set_activity(pid, a);
        }
        for k in 0..self.squad.members.len() {
            let pid = self.squad.members[k];
            if self.fighting.contains_key(&pid) {
                continue;
            }
            let p = &self.people[pid as usize];
            if p.dead || body::knocked_out(&p.wounds.hp_at(&p.stats, self.time)) {
                continue;
            }
            let at = self.squad.at[k];
            // Next corner or door on the way, else the goal itself.
            while let Some(&w) = self.squad.route[k].first() {
                if at.dist(w) < 0.05 && self.squad.route[k].len() > 1 {
                    self.squad.route[k].remove(0);
                } else {
                    break;
                }
            }
            let goal = self.squad.route[k].first().copied().unwrap_or(self.squad.goal[k]);
            let to_go = goal.sub(at);
            let d = to_go.len();
            if d < 1e-3 {
                self.squad.route[k].clear();
                continue;
            }
            let dir = to_go.scale(1.0 / d);
            let ahead = at.add(dir.scale(3.0));
            let grade = (self.terrain.height(ahead) - self.terrain.height(at)) / 3.0;
            let ground = self.terrain.ground(at).pace();
            let stride = (self.member_speed(pid) as f64 * walk_factor(grade) as f64 * ground as f64 * dt) as f32;
            let next = if d <= stride { goal } else { at.add(dir.scale(stride)) };
            if !self.terrain.is_sea(next) || self.building_at(next).is_some() {
                let rise = self.terrain.height(next) - self.terrain.height(at);
                self.climb(pid, rise);
                self.squad.at[k] = next;
            } else {
                self.squad.goal[k] = at;
                self.squad.route[k].clear();
            }
        }
        self.recentre_squad();
        self.update_indoors();
        self.do_pickups();
        self.tidy_looting();
        self.do_picking();
        self.do_gathering();
        self.do_labour();
        self.find_ruins();
        self.do_crafting();
        self.do_lessons();
        self.check_runaways();
        self.do_carrying();
        self.try_open_talk();
    }

    pub(super) fn recentre_squad(&mut self) {
        let standing: Vec<V2> = (0..self.squad.members.len())
            .filter(|&k| !self.people[self.squad.members[k] as usize].dead)
            .map(|k| self.member_pos(k))
            .collect();
        if !standing.is_empty() {
            self.squad.pos = standing.iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / standing.len() as f32);
        }
    }

    /// Where squad member number `k` is: their fight position if fighting.
    pub fn member_pos(&self, k: usize) -> V2 {
        let pid = self.squad.members[k];
        self.fighter_pos(pid).unwrap_or(self.squad.at[k])
    }

    // ---- Things --------------------------------------------------------------

    /// Equip something from a person's pack. Changes their might.
    pub fn equip(&mut self, pid: PersonId, it: ItemId) -> bool {
        self.equip_from(pid, it, None)
    }

    /// Put on a particular thing from the pack (the `k`th entry): for one
    /// made piece among several of the same kind.
    pub fn equip_entry(&mut self, pid: PersonId, k: usize) -> bool {
        let Some(it) = self.people[pid as usize].detail.as_ref().and_then(|d| d.gear.bag.get(k)).map(|e| e.0) else { return false };
        self.equip_from(pid, it, Some(k))
    }

    fn equip_from(&mut self, pid: PersonId, it: ItemId, entry: Option<usize>) -> bool {
        let p = &self.people[pid as usize];
        // Without a left arm there's no holding a shield or a two-handed weapon.
        let def = item(it);
        let one_armed = p.wounds.missing[body::Part::LeftArm as usize] || p.wounds.missing[body::Part::RightArm as usize];
        if one_armed && (def.slot == Slot::OffHand || def.weapon().map(|w| w.two_handed).unwrap_or(false)) {
            return false;
        }
        // Whatever takes the off hand puts out a burning torch (thrown away).
        let displaces_torch = def.slot == Slot::OffHand || def.weapon().map(|w| w.two_handed).unwrap_or(false);
        if displaces_torch && self.torch_in_hand(pid).is_some() && (self.torches.contains_key(&pid) || self.torch_left.contains_key(&pid)) {
            self.unequip(pid, Slot::OffHand);
        }
        let p = &mut self.people[pid as usize];
        let Some(d) = p.detail.as_mut() else { return false };
        // (A torch put away above may have moved the pack's entries.)
        let entry = entry.filter(|&k| d.gear.bag.get(k).map(|e| e.0 == it).unwrap_or(false));
        let done = match entry {
            Some(k) => d.gear.equip_entry(k),
            None => d.gear.equip(it),
        };
        if done.is_err() {
            return false;
        }
        p.recompute_might();
        true
    }

    pub fn unequip(&mut self, pid: PersonId, slot: Slot) -> bool {
        if slot == Slot::OffHand && self.torch_in_hand(pid).is_some() && (self.torches.contains_key(&pid) || self.torch_left.contains_key(&pid)) {
            // A lit or part-burnt torch is thrown away, not packed.
            self.torch_left_hand(pid);
            let p = &mut self.people[pid as usize];
            if let Some(d) = p.detail.as_mut() {
                d.gear.discard(Slot::OffHand);
            }
            p.recompute_might();
            return true;
        }
        let p = &mut self.people[pid as usize];
        let Some(d) = p.detail.as_mut() else { return false };
        if d.gear.unequip(slot).is_none() {
            return false;
        }
        p.recompute_might();
        true
    }

    /// Drop one of something from a person's pack onto the ground at their feet.
    pub fn drop_item(&mut self, pid: PersonId, it: ItemId) -> bool {
        let Some(k) = self.people[pid as usize].detail.as_ref().and_then(|d| d.gear.bag.iter().position(|e| e.0 == it && e.2.is_none()).or_else(|| d.gear.bag.iter().position(|e| e.0 == it))) else {
            return false;
        };
        self.drop_entry(pid, k)
    }

    /// Put down one of the `k`th thing in someone's pack (that very piece).
    pub fn drop_entry(&mut self, pid: PersonId, k: usize) -> bool {
        let pos = self.person_pos(pid);
        if self.squad.index(pid).is_some() {
            self.settle_condition(pid, self.time);
        }
        let p = &mut self.people[pid as usize];
        let Some(d) = p.detail.as_mut() else { return false };
        let Some((it, piece)) = d.gear.take_entry(k) else {
            return false;
        };
        p.recompute_might();
        let jitter = V2::new(((self.next_ground_id * 37) % 7) as f32 * 0.15 - 0.45, ((self.next_ground_id * 53) % 5) as f32 * 0.2 - 0.4);
        let g = self.put_on_ground(it, 1, pos.add(jitter));
        if let Some(x) = self.ground.iter_mut().find(|x| x.id == g) {
            x.piece = piece;
        }
        true
    }

    pub fn put_on_ground(&mut self, it: ItemId, count: u16, pos: V2) -> u32 {
        let id = self.next_ground_id;
        self.next_ground_id += 1;
        self.ground.push(GroundItem { id, item: it, count, pos, owner: None, piece: None });
        id
    }

    /// Send a member to pick something up; they walk over and take it.
    pub fn order_pickup(&mut self, who: PersonId, thing: u32) -> bool {
        let Some(g) = self.ground.iter().find(|g| g.id == thing) else { return false };
        let pos = g.pos;
        if let Some(k) = self.squad.index(who) {
            let (path, locked) = self.route(self.member_pos(k), pos);
            if locked.is_some() {
                self.log.push_front((self.time, "The door is locked.".to_string()));
            }
            self.squad.goal[k] = *path.last().unwrap_or(&pos);
            self.squad.route[k] = path;
        } else {
            return false;
        }
        self.pickups.retain(|p| p.who != who);
        self.pickups.push(Pickup { who, thing });
        self.do_pickups();
        true
    }

    /// Anyone standing by what they were sent for picks it up.
    pub(super) fn do_pickups(&mut self) {
        let mut done = Vec::new();
        for (n, pk) in self.pickups.clone().iter().enumerate() {
            let Some(gi) = self.ground.iter().position(|g| g.id == pk.thing) else {
                done.push(n);
                continue;
            };
            let Some(k) = self.squad.index(pk.who) else {
                done.push(n);
                continue;
            };
            if self.squad.at[k].dist(self.ground[gi].pos) <= REACH {
                let g = self.ground.remove(gi);
                let name = self.people[pk.who as usize].name().unwrap_or("someone").to_string();
                if let Some(town) = g.owner {
                    let mut r = Rng::from_keys(&[self.seed, pk.who as u64, g.id as u64, 0x5448_4546]);
                    if self.witnessed(pk.who, g.pos, town, &mut r) {
                        self.crime(pk.who, town, item(g.item).value * 0.5 + 10.0, format!("{name} is seen stealing!"));
                    }
                }
                if let Some(d) = self.people[pk.who as usize].detail.as_mut() {
                    match g.piece {
                        Some(piece) => d.gear.add_piece(g.item, piece),
                        None => d.gear.add(g.item, g.count),
                    }
                }
                self.people[pk.who as usize].recompute_might();
                self.log.push_front((self.time, format!("{name} picks up {}.", item(g.item).name.to_lowercase())));
                done.push(n);
            }
        }
        for n in done.into_iter().rev() {
            self.pickups.remove(n);
        }
    }

    /// Everything someone had falls where they died.
    pub fn drop_everything(&mut self, pid: PersonId, at: V2) {
        let Some(d) = self.people[pid as usize].detail.as_mut() else { return };
        let mut stuff: Vec<(ItemId, u16, Option<super::materials::Piece>)> = super::items::SLOTS.iter().filter_map(|&s| d.gear.in_slot(s).map(|i| (i, 1, d.gear.piece(s).copied()))).collect();
        stuff.extend(d.gear.bag.iter().map(|e| (e.0, e.1, e.2)));
        d.gear = inventory::Gear::default();
        for (n, (it, c, piece)) in stuff.into_iter().enumerate() {
            let off = V2::new((n as f32 * 2.1).cos(), (n as f32 * 2.1).sin()).scale(0.5 + n as f32 * 0.12);
            let g = self.put_on_ground(it, c, at.add(off));
            if let Some(x) = self.ground.iter_mut().find(|x| x.id == g) {
                x.piece = piece;
            }
        }
    }
}
