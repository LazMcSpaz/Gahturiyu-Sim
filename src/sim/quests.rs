//! Basic quests: work people offer, and how it's tracked.
//!
//! Three kinds to start:
//!
//! - **Clear a camp.** Someone wants the nearest bandit camp dealt with.
//!   Done when your squad wins a fight against it (or it's wiped out).
//! - **Fetch.** Bring them a few of something (materials, a potion).
//!   Hand it over by talking to them with it in anyone's pack.
//! - **Deliver.** Carry a sealed letter to someone in another town.
//!   Done when you talk to them with the letter.
//!
//! Who offers what is decided from their seed and their town's situation,
//! so the same person always has the same request. Each person offers at
//! most one job. Rewards are coin, sometimes with something useful.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::group::GroupId;
use super::items::{self, ItemId};
use super::person::PersonId;
use super::rng::Rng;
use super::world::World;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QuestKind {
    ClearCamp { camp: GroupId, at: V2 },
    Fetch { item: ItemId, count: u16 },
    Deliver { to: PersonId },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Taken on, not done yet.
    Active,
    /// Done; go back to whoever asked.
    Report,
    /// Rewarded and over.
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Quest {
    pub id: u32,
    pub giver: PersonId,
    pub kind: QuestKind,
    pub stage: Stage,
    pub coin: u16,
    pub bonus: Option<ItemId>,
}

impl World {
    /// The job this person would offer, if any. Pure: the same person always
    /// asks for the same thing (unless the world has changed under them —
    /// a camp gone, say).
    pub fn quest_offer(&self, npc: PersonId) -> Option<(QuestKind, u16, Option<ItemId>)> {
        let p = &self.people[npc as usize];
        if p.bandit || p.in_squad || self.quests.iter().any(|q| q.giver == npc) {
            return None;
        }
        let home = p.home?;
        let town = &self.settlements[home as usize];
        let mut r = Rng::from_keys(&[p.seed, 0x5155_4553]);
        if !r.chance(0.35 + p.traits.sociability * 0.3) {
            return None;
        }
        let roll = r.f32();
        // A camp near enough to trouble them?
        let camp = self.camps.iter().filter(|c| c.pos.dist(town.pos) < 4000.0).min_by(|a, b| a.pos.dist(town.pos).total_cmp(&b.pos.dist(town.pos)));
        if let (Some(c), true) = (camp, roll < 0.35) {
            let bonus = [items::id("ring_hearth"), items::id("amulet_clear_mind"), items::id("greater_healing")][r.below(3)];
            return Some((QuestKind::ClearCamp { camp: c.group, at: c.pos }, 120 + r.below(80) as u16, Some(bonus)));
        }
        if roll < 0.7 {
            let wants: &[(&str, u16)] = &[("kelp_frond", 4), ("iron_ore", 3), ("ash_moss", 3), ("healing_draught", 1), ("emberroot", 2), ("hide", 2), ("timber", 2)];
            let (k, n) = wants[r.below(wants.len())];
            let coin = (items::item(items::id(k)).value * n as f32 * 2.5) as u16 + 10;
            return Some((QuestKind::Fetch { item: items::id(k), count: n }, coin, None));
        }
        // A letter for someone elsewhere.
        let others: Vec<usize> = (0..self.settlements.len()).filter(|&s| s != home as usize && !self.settlements[s].residents.is_empty()).collect();
        if others.is_empty() {
            return None;
        }
        let s = &self.settlements[others[r.below(others.len())]];
        let to = s.residents[r.below(s.residents.len())];
        let far = s.pos.dist(town.pos);
        Some((QuestKind::Deliver { to }, (20.0 + far / 150.0) as u16, None))
    }

    /// Take on someone's job. A delivery hands `who` the letter.
    pub fn accept_quest(&mut self, npc: PersonId, who: PersonId) -> Option<u32> {
        let (kind, coin, bonus) = self.quest_offer(npc)?;
        let id = self.quests.len() as u32;
        if let QuestKind::Deliver { to } = kind {
            self.people[to as usize].ensure_detail();
            if let Some(d) = self.people[who as usize].detail.as_mut() {
                d.gear.add(items::id("sealed_letter"), 1);
            }
        }
        self.quests.push(Quest { id, giver: npc, kind, stage: Stage::Active, coin, bonus });
        Some(id)
    }

    /// Notice jobs that have been done out in the world.
    pub(super) fn update_quests(&mut self) {
        for i in 0..self.quests.len() {
            let q = &self.quests[i];
            if q.stage != Stage::Active {
                continue;
            }
            if let QuestKind::ClearCamp { camp, .. } = q.kind {
                let gone = !self.camps.iter().any(|c| c.group == camp);
                if gone || self.beaten_camps.contains(&camp) {
                    self.quests[i].stage = Stage::Report;
                    let line = "The camp is broken. Time to report back.".to_string();
                    self.log.push_front((self.time, line));
                    self.log.truncate(14);
                }
            }
        }
    }

    /// How many of an item the whole squad carries.
    pub fn squad_count(&self, it: ItemId) -> u16 {
        self.squad.members.iter().filter_map(|&m| self.people[m as usize].detail.as_ref()).map(|d| d.gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum::<u16>()).sum()
    }

    /// Take `n` of an item from whoever in the squad has them.
    pub(super) fn take_from_squad(&mut self, it: ItemId, mut n: u16) {
        for m in self.squad.members.clone() {
            while n > 0 {
                let took = self.people[m as usize].detail.as_mut().map(|d| d.gear.take(it)).unwrap_or(false);
                if !took {
                    break;
                }
                n -= 1;
            }
            self.people[m as usize].recompute_might();
        }
    }

    /// Pay out a finished job to `who`.
    pub(super) fn reward(&mut self, qi: usize, who: PersonId) {
        let (coin, bonus) = (self.quests[qi].coin, self.quests[qi].bonus);
        if let Some(d) = self.people[who as usize].detail.as_mut() {
            d.gear.add(items::id("coin"), coin);
            if let Some(b) = bonus {
                d.gear.add(b, 1);
            }
        }
        self.people[who as usize].recompute_might();
        self.quests[qi].stage = Stage::Done;
        let giver = self.quests[qi].giver;
        *self.regard.entry(giver).or_insert(0.0) += 20.0;
        // A favour done for a town raises your standing there.
        if let Some(town) = self.people[giver as usize].home {
            self.add_standing(who, town, 5.0);
        }
    }

    /// One line saying what a job needs now, for the journal.
    pub fn quest_line(&self, q: &Quest) -> String {
        let giver = self.people[q.giver as usize].name().unwrap_or("someone");
        let town = self.people[q.giver as usize].home.map(|h| self.settlements[h as usize].name.as_str()).unwrap_or("?");
        match (&q.kind, q.stage) {
            (_, Stage::Done) => format!("Done: a job for {giver} of {town}."),
            (_, Stage::Report) => format!("Report back to {giver} in {town}."),
            (QuestKind::ClearCamp { at, .. }, _) => {
                let d = at.dist(self.squad.pos);
                let v = at.sub(self.squad.pos);
                format!("Break the bandit camp {:.1} km {} for {giver} of {town}.", d / 1000.0, compass(v))
            }
            (QuestKind::Fetch { item, count }, _) => format!(
                "Bring {giver} in {town} {count} × {} (you have {}).",
                items::item(*item).name.to_lowercase(),
                self.squad_count(*item)
            ),
            (QuestKind::Deliver { to }, _) => {
                let p = &self.people[*to as usize];
                let place = p.home.map(|h| self.settlements[h as usize].name.as_str()).unwrap_or("?");
                format!("Take {giver}'s letter to {} in {place}.", p.name().unwrap_or("someone"))
            }
        }
    }
}

/// "north-east" and the like, for a direction on the map (y grows south).
pub fn compass(v: V2) -> &'static str {
    let a = (-v.y).atan2(v.x).to_degrees();
    let a = (a + 360.0) % 360.0;
    ["east", "north-east", "north", "north-west", "west", "south-west", "south", "south-east"][((a + 22.5) / 45.0) as usize % 8]
}
