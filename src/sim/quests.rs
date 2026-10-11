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
use super::settlement::SettlementId;
use super::chances::Chance;
use super::world::World;

/// Squad members this near the one talking are with them: their packs and
/// purses count for what's bought, sold, paid or handed in, metres.
pub const AT_HAND: f32 = 25.0;

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

    /// Whose packs count right now. With a talk open: the one talking and
    /// the squad members standing with them (within `AT_HAND`), the talker
    /// first, so nothing is sold, paid or handed in out of a pack that isn't
    /// there (RG-1 = RG-20 = NM-55). With no talk open: the whole squad.
    pub fn at_hand(&self) -> Vec<PersonId> {
        match self.talk.as_ref().and_then(|c| self.squad.index(c.with)) {
            Some(k) => self.near_member(k),
            None => self.squad.members.clone(),
        }
    }

    /// Member `k` and the squad members within `AT_HAND` of them, `k` first.
    pub fn near_member(&self, k: usize) -> Vec<PersonId> {
        let at = self.member_pos(k);
        let mut out = vec![self.squad.members[k]];
        out.extend((0..self.squad.members.len()).filter(|&j| j != k && self.member_pos(j).dist(at) <= AT_HAND).map(|j| self.squad.members[j]));
        out
    }

    /// How many of an item the squad has at hand (`at_hand`). Money is coin
    /// and notes together: a note pays as fifty coin anywhere coin is asked
    /// for (NM-27).
    pub fn squad_count(&self, it: ItemId) -> u16 {
        self.count_among(&self.at_hand(), it)
    }

    /// The same, among these members.
    pub fn count_among(&self, who: &[PersonId], it: ItemId) -> u16 {
        let n = self.has_among(who, it);
        if it == items::id("coin") {
            n.saturating_add(self.has_among(who, items::id("note")).saturating_mul(super::economy::NOTE_VALUE))
        } else {
            n
        }
    }

    /// How many of exactly this the squad has at hand (coin without the notes).
    pub fn squad_has(&self, it: ItemId) -> u16 {
        self.has_among(&self.at_hand(), it)
    }

    fn has_among(&self, who: &[PersonId], it: ItemId) -> u16 {
        who.iter().filter_map(|&m| self.people[m as usize].detail.as_ref()).map(|d| d.gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum::<u16>()).fold(0u16, |a, b| a.saturating_add(b))
    }

    /// Take `n` of an item from whoever at hand has them. Short of coin, a
    /// note is broken: the change stays with whoever held it.
    pub(super) fn take_from_squad(&mut self, it: ItemId, n: u16) {
        let who = self.at_hand();
        self.take_among(&who, it, n);
    }

    /// The same, from these members.
    pub(super) fn take_among(&mut self, who: &[PersonId], it: ItemId, n: u16) {
        let mut n = self.take_plain(who, it, n);
        if it != items::id("coin") {
            return;
        }
        let note = items::id("note");
        while n > 0 {
            let Some(m) = who.iter().copied().find(|&m| self.people[m as usize].detail.as_mut().is_some_and(|d| d.gear.take(note))) else { break };
            if let Some(d) = self.people[m as usize].detail.as_mut() {
                d.gear.add(it, super::economy::NOTE_VALUE);
            }
            n = self.take_plain(who, it, n);
        }
    }

    /// Take up to `n` of exactly this item; returns how many are still to find.
    fn take_plain(&mut self, who: &[PersonId], it: ItemId, mut n: u16) -> u16 {
        for &m in who {
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
        // Paid from the giver's purse, as far as it goes (U-10).
        let coin = match self.quests[qi].opp.and_then(|id| self.opportunity(id)) {
            Some(o) if !o.favour => coin.min(self.giver_can_pay(o) as u16),
            _ => coin,
        };
        self.quests[qi].coin = coin;
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
        // A favour done for a town raises your standing there. (A job from
        // the board does that in `finish_opp`, and only if lawful: U-9.)
        if self.quests[qi].opp.is_none() {
            if let Some(town) = self.people[giver as usize].home {
                self.add_standing(who, town, 5.0);
            }
        }
    }

    /// One line saying what a job needs now, for the journal.
    pub fn quest_line(&self, q: &Quest) -> String {
        let giver = self.known_as(q.giver);
        let home = self.people[q.giver as usize].home;
        let town = home.map(|h| format!("{}{}", self.settlements[h as usize].name, self.how_far(h))).unwrap_or_else(|| "?".into());
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
                let place = self.people[*to as usize].home.map(|h| format!("{}{}", self.settlements[h as usize].name, self.how_far(h))).unwrap_or_else(|| "?".into());
                format!("Take {giver}'s letter to {} in {place}.", self.known_as(*to))
            }
            (QuestKind::Job { opp }, _) => match self.opportunity(*opp) {
                Some(o) => format!("For {giver} of {town}: {}.", self.opp_line(o)),
                None => format!("A job for {giver} of {town}."),
            },
        }
    }

    /// Someone's name, with their trade when another in their town shares it.
    pub fn known_as(&self, p: PersonId) -> String {
        let me = &self.people[p as usize];
        let name = me.name().unwrap_or("someone").to_string();
        let Some(h) = me.home else { return name };
        let twin = self.settlements[h as usize].residents.iter().any(|&o| o != p && self.people[o as usize].name() == me.name());
        match self.society.lives.get(p as usize) {
            Some(l) if twin => format!("{name} the {}", l.job.title(me.seed).to_lowercase()),
            _ => name,
        }
    }

    /// How far off a town is, if the squad isn't in it: " (3.2 km north)".
    pub fn how_far(&self, town: SettlementId) -> String {
        let s = &self.settlements[town as usize];
        let v = s.pos.sub(self.squad.pos);
        if v.len() <= s.radius() {
            return String::new();
        }
        format!(" ({:.1} km {})", v.len() / 1000.0, compass(v))
    }
}

/// "north-east" and the like, for a direction on the map (y grows south).
pub fn compass(v: V2) -> &'static str {
    let a = (-v.y).atan2(v.x).to_degrees();
    let a = (a + 360.0) % 360.0;
    ["east", "north-east", "north", "north-west", "west", "south-west", "south", "south-east"][((a + 22.5) / 45.0) as usize % 8]
}
