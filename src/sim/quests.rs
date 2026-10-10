//! The squad's journal: jobs taken on, and how they're tracked.
//!
//! Jobs come from people's lives: each is an opportunity (`chances.rs`)
//! made by a storyline (`stories.rs`) — a crafter short of goods wants them
//! fetched, someone with kin far off has a letter, a town troubled by a camp
//! wants it broken, the robbed want a guard or their things back, and so on.
//! The three older kinds keep their own tracking here:
//!
//! - **Clear a camp.** Done when your squad wins a fight against it (or it's
//!   wiped out).
//! - **Fetch.** Hand it over by talking to them with it in anyone's pack.
//! - **Deliver.** Carry a sealed letter; done when you talk to them with it.
//!
//! Everything else is a `Job` tracked on its opportunity. Rewards are coin
//! from the giver's purse, or a favour owed.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::group::GroupId;
use super::items::{self, ItemId};
use super::person::PersonId;
use super::chances::Chance;
use super::world::World;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum QuestKind {
    ClearCamp { camp: GroupId, at: V2 },
    Fetch { item: ItemId, count: u16 },
    Deliver { to: PersonId },
    /// Any other job, as an opportunity (`chances.rs`).
    Job { opp: u32 },
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
    /// The opportunity it came from.
    #[serde(default)]
    pub opp: Option<u32>,
}

impl World {
    /// The job this person would offer the squad, if any: their open
    /// opportunity (`chances.rs`), made from what's going on in their life.
    pub fn quest_offer(&self, npc: PersonId) -> Option<(QuestKind, u16, Option<ItemId>)> {
        let o = self.open_offer(npc)?;
        let kind = match o.kind {
            Chance::ClearCamp { camp, at } => QuestKind::ClearCamp { camp, at },
            Chance::Fetch { item, count } => QuestKind::Fetch { item, count },
            Chance::Deliver { to } => QuestKind::Deliver { to },
            _ => QuestKind::Job { opp: o.id },
        };
        Some((kind, o.reward, None))
    }

    /// Take on someone's job. A delivery hands `who` the letter.
    pub fn accept_quest(&mut self, npc: PersonId, who: PersonId) -> Option<u32> {
        let id = self.open_offer(npc)?.id;
        self.take_opportunity(id, who)
    }

    /// Notice jobs that have been done out in the world.
    pub(super) fn update_quests(&mut self) {
        // A camp stays beaten only until its gang has rested and is back on
        // its feet (the camp's ready dawn); then it has to be beaten again.
        let t = self.time;
        let camps = &self.camps;
        self.beaten_camps.retain(|g| camps.iter().any(|c| c.group == *g && c.ready_at > t));
        for i in 0..self.quests.len() {
            let q = &self.quests[i];
            if q.stage != Stage::Active {
                continue;
            }
            if let QuestKind::ClearCamp { camp, .. } = q.kind {
                let gone = !self.camps.iter().any(|c| c.group == camp);
                if gone || self.beaten_camps.contains(&camp) {
                    self.quests[i].stage = Stage::Report;
                    if let Some(id) = self.quests[i].opp {
                        if let Some(o) = self.society.opps.iter_mut().find(|o| o.id == id) {
                            o.done = true;
                        }
                    }
                    let line = "The camp is broken. Time to report back.".to_string();
                    self.log.push_front((self.time, line));
                    self.log.truncate(14);
                }
            }
        }
    }

    /// How many of an item the whole squad carries. Money is coin and notes
    /// together: a note pays as fifty coin anywhere coin is asked for (NM-27).
    pub fn squad_count(&self, it: ItemId) -> u16 {
        let n = self.squad_has(it);
        if it == items::id("coin") {
            n.saturating_add(self.squad_has(items::id("note")).saturating_mul(super::economy::NOTE_VALUE))
        } else {
            n
        }
    }

    /// How many of exactly this the squad carries (coin without the notes).
    pub fn squad_has(&self, it: ItemId) -> u16 {
        self.squad.members.iter().filter_map(|&m| self.people[m as usize].detail.as_ref()).map(|d| d.gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum::<u16>()).sum()
    }

    /// Take `n` of an item from whoever in the squad has them. Short of
    /// coin, a note is broken: the change stays with whoever held it.
    pub(super) fn take_from_squad(&mut self, it: ItemId, n: u16) {
        let mut n = self.take_plain(it, n);
        if it != items::id("coin") {
            return;
        }
        let note = items::id("note");
        while n > 0 {
            let Some(m) = self.squad.members.clone().into_iter().find(|&m| self.people[m as usize].detail.as_mut().is_some_and(|d| d.gear.take(note))) else { break };
            if let Some(d) = self.people[m as usize].detail.as_mut() {
                d.gear.add(it, super::economy::NOTE_VALUE);
            }
            n = self.take_plain(it, n);
        }
    }

    /// Take up to `n` of exactly this item; returns how many are still to find.
    fn take_plain(&mut self, it: ItemId, mut n: u16) -> u16 {
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
        n
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
        if let Some(opp) = self.quests[qi].opp {
            let t = self.time;
            self.finish_opp(opp, who, t);
        }
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
            (QuestKind::Job { opp }, _) => match self.opportunity(*opp) {
                Some(o) => format!("For {giver} of {town}: {}.", self.opp_line(o)),
                None => format!("A job for {giver} of {town}."),
            },
        }
    }
}

/// "north-east" and the like, for a direction on the map (y grows south).
pub fn compass(v: V2) -> &'static str {
    let a = (-v.y).atan2(v.x).to_degrees();
    let a = (a + 360.0) % 360.0;
    ["east", "north-east", "north", "north-west", "west", "south-west", "south", "south-east"][((a + 22.5) / 45.0) as usize % 8]
}
