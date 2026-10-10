//! Containers: chests, crates, cupboards and barrels inside buildings
//! (playable-MVP step 6).
//!
//! Each building's variant (`layout.rs`) says where its containers stand.
//! The first time anyone in the squad steps in, each one is filled from one
//! keyed roll (world seed + town + building + slot), so the same world
//! always has the same things in the same chest, whenever it's first
//! opened. From then on a container is saved state (`World::containers`):
//! what's taken stays taken.
//!
//! **Whose it is.** A container belongs to the household living there, or,
//! where nobody does (workshops, halls, shrines), to the town. Taking from it
//! is theft: each thing taken is one keyed roll against the town's eyes,
//! with the same light-and-sneaking rules as anywhere (`witnessed`), and a
//! seen theft is a crime in that town (bounty, or arrest if the watch is on
//! hand). Opening and looking is free.
//!
//! **Locks.** Most chests and some cupboards are locked, day and night.
//! They're picked with the door rules (`order_pick_container`, `do_picking`):
//! a picked container stays open for good.
//!
//! Going through one uses the loot panel (`loot.rs`'s `Source::Chest`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::buildings::{Door, DoorId};
use super::geo::V2;
use super::inventory::Entry;
use super::items;
use super::layout::{Holder, Use};
use super::loot::{Looting, Source};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::World;

/// A container: its town, building, and slot in the building's layout.
pub type ContainerId = (SettlementId, u16, u8);

/// Whose things they are.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Owner {
    /// A household (index into `Society::households`).
    Household(u32),
    Town(SettlementId),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Container {
    pub id: ContainerId,
    pub what: Holder,
    pub pos: V2,
    /// Facing, radians.
    pub rot: f32,
    pub items: Vec<Entry>,
    /// 0 = no lock; otherwise how hard to pick, 1..100.
    pub lock: f32,
    pub picked: bool,
    pub owner: Owner,
    /// Things taken from it so far (keys the witness rolls).
    pub taken: u32,
}

/// All the containers laid out so far.
pub type Containers = BTreeMap<ContainerId, Container>;

const FOOD: &[&str] = &["flatbread", "dried_fish", "salted_meat", "grain", "mussels", "kelp_frond", "wild_berries"];
const STUFF: &[&str] = &["timber", "hide", "leather", "fibre", "clay", "seareed", "salt_crystal", "cloth"];
const SMITH: &[&str] = &["iron_ingot", "iron_ore", "charcoal", "bronze_ingot", "iron_ore"];
const TENDER: &[&str] = &["rock", "sand", "clay", "hearthclay", "ringstone"];
const POTTER: &[&str] = &["clay", "hearthclay", "charcoal", "sand"];
const CLOTHES: &[&str] = &["cloth_shirt", "trousers", "padded_jacket", "wraps", "boots", "leather_cap", "knife", "reed_paper", "squid_ink", "healing_draught", "torch", "lockpick"];
const SHOP: &[&str] = &["cloth_shirt", "trousers", "boots", "leather_gloves", "small_pack", "torch", "healing_draught", "knife", "hatchet", "reed_paper"];
const BETTER: &[&str] = &["short_sword", "hide_coat", "war_pick", "spear", "buckler", "hide_leggings", "iron_helm", "gold_ring", "pearl"];
const RARE: &[&str] = &["ring_swiftness", "ring_might", "amulet_wellspring", "amulet_clear_mind", "ring_hearth", "seers_hood", "striders_boots", "duelists_gloves", "pearl_necklace"];

/// What a container holds when first opened, and its lock. One keyed roll
/// per container: world seed, town, building, slot.
pub fn stock(seed: u64, id: ContainerId, what: Holder, use_: Use, variant: &str) -> (Vec<Entry>, f32) {
    let mut r = Rng::from_keys(&[seed, id.0 as u64, id.1 as u64, id.2 as u64, 0x434F_4E54]);
    let mut out: Vec<Entry> = Vec::new();
    let add = |out: &mut Vec<Entry>, key: &str, n: u16| {
        let it = items::id(key);
        match out.iter_mut().find(|e| e.0 == it) {
            Some(e) => e.1 += n,
            None => out.push(Entry(it, n, None)),
        }
    };
    let trade: &[&str] = if variant.contains("forge") || variant.contains("workyard") {
        if r.chance(0.5) { SMITH } else { POTTER }
    } else if variant.contains("tender") {
        TENDER
    } else {
        STUFF
    };
    let lock = match what {
        Holder::Barrel => {
            for _ in 0..1 + r.below(2) {
                let k = *r.pick(FOOD);
                add(&mut out, k, 2 + r.below(7) as u16);
            }
            0.0
        }
        Holder::Crate => {
            for _ in 0..1 + r.below(3) {
                let k = *r.pick(trade);
                add(&mut out, k, 1 + r.below(6) as u16);
            }
            0.0
        }
        Holder::Cupboard => {
            let list = if use_ == Use::Shop { SHOP } else { CLOTHES };
            for _ in 0..1 + r.below(if use_ == Use::Shop { 4 } else { 3 }) {
                let k = *r.pick(list);
                add(&mut out, k, 1);
            }
            if r.chance(0.2) { 10.0 + r.f32() * 30.0 } else { 0.0 }
        }
        Holder::Chest => {
            add(&mut out, "coin", 5 + r.below(if use_ == Use::Shop { 90 } else { 45 }) as u16);
            for _ in 0..r.below(3) {
                let k = if r.chance(0.08) {
                    *r.pick(RARE)
                } else if r.chance(0.35) {
                    *r.pick(BETTER)
                } else {
                    *r.pick(CLOTHES)
                };
                add(&mut out, k, 1);
            }
            if use_ == Use::Shrine {
                let texts: Vec<&str> = items::ITEMS.iter().filter(|d| matches!(d.kind, items::Kind::Text(_) | items::Kind::Notes(_))).map(|d| d.key).collect();
                if !texts.is_empty() {
                    add(&mut out, r.pick(&texts), 1);
                }
                70.0
            } else if r.chance(0.75) {
                15.0 + r.f32() * 55.0
            } else {
                0.0
            }
        }
    };
    (out, lock)
}

impl World {
    /// Where a building's containers stand: slot, kind, place, facing.
    pub fn container_spots(&self, d: &Door) -> Vec<(u8, Holder, V2, f32)> {
        d.variant().holders.iter().enumerate().map(|(k, s)| (k as u8, s.what, d.to_world(s.at), d.rot + s.rot)).collect()
    }

    /// Whose a building's things are: the household living there, or the town.
    pub fn belongs_to(&self, door: DoorId) -> Owner {
        let town = door.0;
        self.society
            .households
            .iter()
            .position(|h| h.home == Some(door.1) && self.society.communities.get(h.community as usize).is_some_and(|c| c.town == town) && !h.members.is_empty())
            .map(|i| Owner::Household(i as u32))
            .unwrap_or(Owner::Town(town))
    }

    /// Fill a building's containers the first time it's entered (those
    /// already laid out are left as they are).
    pub(super) fn stock_building(&mut self, d: Door) {
        let v = d.variant();
        let owner = self.belongs_to(d.id);
        for (slot, what, pos, rot) in self.container_spots(&d) {
            let id = (d.id.0, d.id.1, slot);
            if self.containers.contains_key(&id) {
                continue;
            }
            let (items, lock) = stock(self.seed, id, what, v.use_, v.key);
            self.containers.insert(id, Container { id, what, pos, rot, items, lock, picked: false, owner, taken: 0 });
        }
    }

    pub fn container(&self, id: ContainerId) -> Option<&Container> {
        self.containers.get(&id)
    }

    /// The containers laid out in a building.
    pub fn containers_in(&self, door: DoorId) -> impl Iterator<Item = &Container> {
        self.containers.range((door.0, door.1, 0)..=(door.0, door.1, u8::MAX)).map(|(_, c)| c)
    }

    pub fn container_locked(&self, id: ContainerId) -> bool {
        self.containers.get(&id).is_some_and(|c| c.lock > 0.0 && !c.picked)
    }

    /// Where someone stands to open a container: a step in from it, toward
    /// the middle of the room.
    pub fn container_stand(&self, id: ContainerId) -> Option<V2> {
        let c = self.containers.get(&id)?;
        let d = self.door((id.0, id.1))?;
        let to = d.centre.sub(c.pos);
        let len = to.len();
        let dir = if len > 0.5 { to.scale(1.0 / len) } else { V2::new(c.rot.cos(), c.rot.sin()) };
        Some(c.pos.add(dir.scale(0.9)))
    }

    /// Send a squad member to open a container and go through it. Locked
    /// ones have to be picked first.
    pub fn order_search(&mut self, who: PersonId, id: ContainerId) -> bool {
        if self.container_locked(id) {
            return false;
        }
        let (Some(k), Some(stand)) = (self.squad.index(who), self.container_stand(id)) else { return false };
        let (path, locked) = self.route(self.member_pos(k), stand);
        if locked.is_some() {
            self.log.push_front((self.time, "The door is locked.".to_string()));
            self.log.truncate(14);
            return false;
        }
        self.squad.goal[k] = *path.last().unwrap_or(&stand);
        self.squad.route[k] = path;
        self.pickups.retain(|p| p.who != who);
        self.picking.retain(|p| p.who != who);
        self.looting.retain(|l| l.who != who);
        self.looting.push(Looting { who, from: Source::Chest(id) });
        true
    }

    /// Send a squad member to pick a container's lock.
    pub fn order_pick_container(&mut self, who: PersonId, id: ContainerId) -> bool {
        if !self.container_locked(id) {
            return false;
        }
        let (Some(k), Some(stand)) = (self.squad.index(who), self.container_stand(id)) else { return false };
        let (path, locked) = self.route(self.member_pos(k), stand);
        if locked.is_some() {
            return false;
        }
        self.squad.goal[k] = *path.last().unwrap_or(&stand);
        self.squad.route[k] = path;
        self.pickups.retain(|p| p.who != who);
        self.looting.retain(|l| l.who != who);
        self.picking.retain(|p| p.who != who);
        self.picking.push(super::buildings::Picking { who, door: (id.0, id.1), tries: 0, next: None, holder: Some(id.2) });
        true
    }

    /// The container this member is standing at and going through, if any.
    pub fn searching_now(&self, who: PersonId) -> Option<ContainerId> {
        match self.source_now(who)? {
            Source::Chest(c) => Some(c),
            Source::Body(_) => None,
        }
    }

    /// Note a thing taken from a container: one keyed roll against the
    /// town's eyes; seen, it's theft.
    pub(super) fn took_from(&mut self, who: PersonId, id: ContainerId, worth: f32) {
        let Some(c) = self.containers.get_mut(&id) else { return };
        c.taken += 1;
        let (n, at, what) = (c.taken, c.pos, c.what);
        let mut r = Rng::from_keys(&[self.seed, who as u64, id.0 as u64, id.1 as u64, id.2 as u64, n as u64, 0x5448_4546]);
        if self.witnessed(who, at, id.0, &mut r) {
            let name = self.people[who as usize].name().unwrap_or("someone").to_string();
            self.crime(who, id.0, worth * 0.5 + 10.0, format!("{name} is seen stealing from a {}!", what.name()));
        }
    }
}
