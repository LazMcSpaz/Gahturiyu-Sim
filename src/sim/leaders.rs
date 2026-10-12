//! Faction leaders as quest-givers (`docs/leaders.md`: "every leader here is
//! a quest-giver"; F6).
//!
//! **Seats.** A leader is an ordinary person the world has seated as one of
//! `factions::LEADERS` (`World::seats`, saved). Nobody is seated until Laz's
//! map gives the factions their towns; then the map's placing seats them
//! once, at world creation, and tests seat someone by hand (`seat_leader`).
//! A seated leader goes by their leader's name, belongs to their faction
//! (`factions_of`), and speaks its lines.
//!
//! **Their jobs** are data (`TASKS`): each leader asks for things that serve
//! their goals, one at a time, in order, each once. A job is one of the
//! journal's own kinds (break a camp, bring something, carry a letter to
//! another leader), so it's tracked, handed in and paid like any other; on
//! top of the coin it counts as a deed with the leader's faction, which
//! ripples to that faction's friends and enemies. A job may want the squad
//! to stand well with the faction first; until then the leader says so
//! (`not_yet`). What they say is `data/lines/leaders.txt`.
//!
//! The jobs in `TASKS` are Claude's first drafts from each leader's goals,
//! for Laz to change: a line of data each.

use serde::{Deserialize, Serialize};

use super::factions::{self, FactionId, LEADERS};
use super::group::GroupId;
use super::items::{self, ItemId};
use super::person::PersonId;
use super::quests::{Quest, QuestKind, Stage};
use super::save::Name;
use super::world::World;

/// Someone seated as one of the leaders.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Seat {
    /// The leader's given name, as in `LEADERS`.
    #[serde(with = "super::save::name")]
    pub leader: Name,
    pub who: PersonId,
}

/// What a leader's job asks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ask {
    /// Break a camp: the Binders' (`slavers`) or a bandits'. The nearest to
    /// the leader that's standing.
    BreakCamp { slavers: bool },
    /// Bring them this many of a thing.
    Fetch { item: &'static str, count: u16 },
    /// Carry a sealed letter to another leader (only once they're seated).
    Letter { to: &'static str },
}

impl Ask {
    /// The tag lines use for the kind of job.
    pub fn tag(self) -> &'static str {
        match self {
            Ask::BreakCamp { slavers: true } => "ask=slavers",
            Ask::BreakCamp { slavers: false } => "ask=camp",
            Ask::Fetch { .. } => "ask=fetch",
            Ask::Letter { .. } => "ask=letter",
        }
    }
}

/// One job a leader gives.
#[derive(Clone, Copy, Debug)]
pub struct Task {
    /// Its key: the lines' `task=` tag, and what the journal remembers.
    pub key: &'static str,
    /// The leader who asks (given name, as in `LEADERS`).
    pub leader: &'static str,
    pub ask: Ask,
    pub coin: u16,
    /// Standing it earns with the leader's faction.
    pub deed: f32,
    /// Standing with the leader's faction they want first.
    pub needs: f32,
}

const fn task(key: &'static str, leader: &'static str, ask: Ask, coin: u16, deed: f32, needs: f32) -> Task {
    Task { key, leader, ask, coin, deed, needs }
}

use Ask::{BreakCamp, Fetch, Letter};

/// Standing a leader wants before trusting the squad with a weightier job.
pub const TRUSTED: f32 = factions::FRIENDLY;

/// The leaders' jobs, each leader's in the order they're given. Claude's
/// first drafts from `docs/leaders.md` (Laz to change).
pub static TASKS: &[Task] = &[
    // Open Council: free the bonded.
    task("break_binders", "Waarua", BreakCamp { slavers: true }, 120, 10.0, 0.0),
    // Tide Houses: bring the Far Sails home.
    task("word_to_the_fleet", "Niralea", Letter { to: "Nemu" }, 40, 6.0, 0.0),
    // Far Sails: never go back (his answer).
    task("answer_the_shore", "Nemu", Letter { to: "Niralea" }, 40, 6.0, 0.0),
    // Binders: a title from the lords.
    task("terms_for_a_lord", "Paqax", Letter { to: "Kotqot" }, 60, 6.0, 0.0),
    // Sharp Hands: their quiet Binder ties.
    task("quiet_word", "Feyesh", Letter { to: "Paqax" }, 70, 6.0, 0.0),
    // Road Crews: safe roads.
    task("clear_the_road", "Doruḍoqu", BreakCamp { slavers: false }, 80, 8.0, 0.0),
    // Lantern Company: a bounty.
    task("camp_bounty", "Lamawina", BreakCamp { slavers: false }, 100, 6.0, 0.0),
    // Last Wave: the war-chief tests outsiders; past him, the high
    // priestess asks for food for the enclave.
    task("test_of_nerve", "Totkit", BreakCamp { slavers: false }, 50, TRUSTED, 0.0),
    task("bread_for_the_enclave", "Textrim", Fetch { item: "flatbread", count: 10 }, 60, 8.0, TRUSTED),
    // Wardens: make the powers listen; supplies for the wall.
    task("bandages_for_the_wall", "Hoqa", Fetch { item: "bandage", count: 6 }, 30, 8.0, 0.0),
    task("plea_to_the_general", "Haʻo", Letter { to: "Haʻahoqo" }, 30, 6.0, 0.0),
    // Holy House: the priestess feeds the poor.
    task("bread_for_the_poor", "Aloqi", Fetch { item: "flatbread", count: 6 }, 20, 8.0, 0.0),
];

/// A leader by given name.
pub fn leader_index(name: &str) -> Option<usize> {
    LEADERS.iter().position(|l| l.name == name)
}

/// A job by key.
pub fn task_by_key(key: &str) -> Option<&'static Task> {
    TASKS.iter().find(|t| t.key == key)
}

/// The faction a job is done for: its leader's.
pub fn task_faction(t: &Task) -> FactionId {
    factions::id(LEADERS[leader_index(t.leader).expect("a task's leader is in LEADERS")].faction)
}

/// A leader's job, worked out for now: what it asks and pays.
#[derive(Clone, Debug)]
pub struct Offer {
    pub task: &'static Task,
    pub kind: QuestKind,
}

impl World {
    /// Seat someone as a leader (the map's placing at world creation; tests
    /// by hand). They go by the leader's name from now on. Anyone already in
    /// that seat leaves it.
    pub fn seat_leader(&mut self, name: &str, who: PersonId) -> bool {
        let Some(k) = leader_index(name) else { return false };
        if self.people[who as usize].in_squad {
            return false;
        }
        let leader = LEADERS[k].name;
        self.seats.retain(|s| s.leader != leader && s.who != who);
        self.seats.push(Seat { leader, who });
        let p = &mut self.people[who as usize];
        p.ensure_detail();
        if let Some(d) = p.detail.as_mut() {
            d.name = leader.to_string();
        }
        true
    }

    /// The leader someone is seated as.
    pub fn leader_of(&self, who: PersonId) -> Option<&'static factions::Leader> {
        let s = self.seats.iter().find(|s| s.who == who)?;
        leader_index(s.leader).map(|k| &LEADERS[k])
    }

    /// Who sits as this leader, if anyone (and they're alive).
    pub fn leader_seated(&self, name: &str) -> Option<PersonId> {
        self.seats.iter().find(|s| s.leader == name).map(|s| s.who).filter(|&p| !self.people[p as usize].dead)
    }

    /// A leader's full style: "Waarua of the Harbour".
    pub fn leader_style(&self, who: PersonId) -> Option<String> {
        self.leader_of(who).map(|l| format!("{} {}", l.name, l.byname))
    }

    /// Has the squad already taken this job (done or not)?
    fn task_taken(&self, key: &str) -> bool {
        self.quests.iter().any(|q| q.task == Some(key))
    }

    /// The camp a job to break one points at: the nearest standing camp of
    /// the kind to the leader.
    fn camp_for(&self, who: PersonId, slavers: bool) -> Option<(GroupId, super::geo::V2)> {
        let at = self.person_pos(who);
        self.camps
            .iter()
            .filter(|c| c.slavers == slavers && !self.is_warden(c.group) && !self.beaten_camps.contains(&c.group))
            .min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at)))
            .map(|c| (c.group, c.pos))
    }

    /// The next job of this leader's the squad hasn't taken, whether or not
    /// they'd give it yet.
    fn next_task(&self, who: PersonId) -> Option<&'static Task> {
        let l = self.leader_of(who)?;
        TASKS.iter().filter(|t| t.leader == l.name).find(|t| !self.task_taken(t.key) && self.task_kind(who, t).is_some())
    }

    /// What a job asks for now, if it can be done at all (a camp to break,
    /// a leader to write to).
    fn task_kind(&self, who: PersonId, t: &Task) -> Option<QuestKind> {
        Some(match t.ask {
            Ask::BreakCamp { slavers } => {
                let (camp, at) = self.camp_for(who, slavers)?;
                QuestKind::ClearCamp { camp, at }
            }
            Ask::Fetch { item, count } => QuestKind::Fetch { item: items::id(item), count },
            Ask::Letter { to } => QuestKind::Deliver { to: self.leader_seated(to).filter(|&p| p != who)? },
        })
    }

    /// The job a leader would give the squad now.
    pub fn leader_offer(&self, who: PersonId) -> Option<Offer> {
        if self.people[who as usize].dead {
            return None;
        }
        let t = self.next_task(who)?;
        if self.standing_with(task_faction(t)) < t.needs {
            return None;
        }
        Some(Offer { task: t, kind: self.task_kind(who, t)? })
    }

    /// The job a leader has but won't give the squad yet (not trusted
    /// enough).
    pub fn leader_holds_back(&self, who: PersonId) -> Option<&'static Task> {
        let t = self.next_task(who)?;
        (self.standing_with(task_faction(t)) < t.needs).then_some(t)
    }

    /// Take on a leader's job, `member` doing it. A letter goes into their
    /// pack. Returns the journal entry.
    pub fn accept_leader_task(&mut self, leader: PersonId, member: PersonId) -> Option<u32> {
        let o = self.leader_offer(leader)?;
        if !self.people[member as usize].in_squad {
            return None;
        }
        match o.kind {
            QuestKind::Deliver { to } => {
                self.people[to as usize].ensure_detail();
                if let Some(d) = self.people[member as usize].detail.as_mut() {
                    d.gear.add(items::id("sealed_letter"), 1);
                }
                self.people[member as usize].recompute_might();
            }
            QuestKind::ClearCamp { camp, .. } => self.hear_of_camp(camp),
            _ => {}
        }
        let id = self.quests.len() as u32;
        self.quests.push(Quest { id, giver: leader, kind: o.kind, stage: Stage::Active, coin: o.task.coin, bonus: None, opp: None, task: Some(o.task.key) });
        Some(id)
    }

    /// A leader's job handed in: the deed with their faction.
    pub(super) fn leader_task_done(&mut self, key: &str) {
        if let Some(t) = task_by_key(key) {
            self.deed(task_faction(t), t.deed);
        }
    }

    /// The tags and slot values a leader's lines get for a job.
    pub(super) fn task_facts(&self, who: PersonId, t: &Task, kind: Option<&QuestKind>, coin: u16) -> (Vec<String>, Vec<(&'static str, String)>) {
        let mut tags = vec![format!("task={}", t.key), t.ask.tag().to_string()];
        if let Some(l) = self.leader_of(who) {
            tags.push(format!("leader={}", l.name));
        }
        let mut values: Vec<(&'static str, String)> = vec![("coin", coin.to_string())];
        let from = self.person_pos(who);
        match kind {
            Some(QuestKind::ClearCamp { at, .. }) => {
                let v = at.sub(from);
                values.push(("walk", super::talk::walk_words(v.len()).to_string()));
                values.push(("dir", super::quests::compass(v).to_string()));
            }
            Some(QuestKind::Fetch { item, count }) => {
                values.push(("count", count.to_string()));
                values.push(("item", items::item(*item).name.to_lowercase()));
            }
            Some(QuestKind::Deliver { to }) => {
                values.push(("to", self.people[*to as usize].name().unwrap_or("them").to_string()));
                let place = self.people[*to as usize].home.map(|h| self.settlements[h as usize].name.clone()).unwrap_or_else(|| "their hall".into());
                values.push(("place", place));
            }
            _ => {}
        }
        (tags, values)
    }
}

/// The item a fetch job wants (for tests and the window).
pub fn fetch_item(t: &Task) -> Option<ItemId> {
    match t.ask {
        Ask::Fetch { item, .. } => Some(items::id(item)),
        _ => None,
    }
}
