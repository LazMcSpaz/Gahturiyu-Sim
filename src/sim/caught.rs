//! The caught path on Salt Isle (`docs/tutorial-island.md`, Path A: "you're
//! made a slave").
//!
//! - **Taken.** A squad member in the compound by day, plainly (no
//!   disguise), or caught by the watch anywhere on the island, is taken for
//!   a runaway (`taken`): bound to the lord's steward as a slave, for life,
//!   and stood before Ḍeshu in the overseers' hall. He decides; the Binders
//!   defer. The tutorial goes onto the caught path (`set_path`); caught on
//!   the sneak path, that drops them onto it.
//! - **The bond** (the "bond" lesson): asked of the lord or the steward,
//!   "What do I owe?" says how it stands.
//! - **The yard** (the "yard" lesson): a bound member who walks into the yard
//!   by day is put to a bout with one of the debtors, to knockout (a duel:
//!   nobody dies of it, `law.rs`). Fought, the lesson's done; won, it counts.
//! - **Bought out** (the "bought" lesson): when it's up, someone has paid
//!   for them: the Lantern Company if they fought most, the Writers if they
//!   worked with their head (what the island saw: `tutorial::Deeds`, fights
//!   won against things made; Diʻub's answers break a tie, `Strangers::lean`). The bond ends, they're counted
//!   among the buyer's people, and they owe it a while's work (`owes`).
//!   The lesson's done at the dock, where the buyer's ship comes in.
//!
//! **The sneak path's end** (`stowed_away`, called by the ship when it sails
//! with them stowed away): Diʻub goes to work for the Night Market and
//! brings them in with him (his line, `data/lines/diub.txt` `night_market`):
//! their first faction, as the Lantern Company or the Writers are on the
//! caught path.
//!
//! The squad's own (the player exception): checked in the squad's step.

use serde::{Deserialize, Serialize};

use super::factions::{self, FactionId};
use super::island::Spot;
use super::person::PersonId;
use super::world::World;

/// Within this of the yard's middle is in the yard.
pub const YARD_REACH: f32 = 14.0;
/// Within this of the dock is at it.
pub const AT_DOCK: f32 = 25.0;
/// What a buyer paid, in coin: what's owed them in work.
pub const BOUGHT_FOR: u16 = 150;
/// How light it must be for the compound to see you plainly.
const DAY: f32 = 0.3;

/// How the caught path stands. Saved.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Caught {
    /// Who was taken, and when.
    pub taken: Option<(PersonId, f64)>,
    /// Who bought them out, and what's owed in work.
    pub owes: Option<(FactionId, u16)>,
    /// Stowed away with Diʻub on the sneak path: he's taken them into the
    /// Night Market.
    #[serde(default)]
    pub night_market: bool,
}

impl World {
    /// The squad's step: taken by day in the compound; the yard's bout; the
    /// buyer's coming; at the dock.
    pub(super) fn caught_step(&mut self) {
        if !self.on_island() {
            return;
        }
        let Some(isle) = self.island().cloned() else { return };
        let t = self.time;
        let day = super::stealth::daylight(t) >= DAY;
        // Taken: in the compound by day, plainly.
        if day && self.caught.taken.is_none() {
            let seen = self.squad.members.iter().copied().find(|&m| isle.in_compound(self.person_pos(m)) && self.disguise_of(m).is_none() && !self.is_bonded(m, t) && !self.is_down(m) && self.captive_of(m).is_none());
            if let Some(m) = seen {
                self.taken(m);
                return;
            }
        }
        let Some((who, _)) = self.caught.taken else { return };
        let now = self.lesson_now().map(|x| x.key);
        // The yard: a bout, once.
        if now == Some("yard") && day && self.squad_battle().is_none() && self.is_bonded(who, t) && self.person_pos(who).dist(isle.spot(Spot::Yard)) <= YARD_REACH {
            self.yard_bout(who);
        }
        // Bought out, then at the dock.
        if now == Some("bought") {
            if self.caught.owes.is_none() {
                self.bought_out(who);
            } else if self.squad.members.iter().any(|&m| self.person_pos(m).dist(isle.spot(Spot::Dock)) <= AT_DOCK) {
                self.lesson_done("bought");
            }
        }
    }

    /// Taken for a runaway: bound to the steward as a slave, and stood
    /// before the lord. (The watch's catch on the island comes here too:
    /// `pursuit.rs`.)
    pub fn taken(&mut self, who: PersonId) {
        let Some(isle) = self.island().cloned() else { return };
        let t = self.time;
        self.caught.taken = Some((who, t));
        self.set_path(super::tutorial::Path::Caught);
        // (Into the compound they went: by the front, this way.)
        self.lesson_done("night_run");
        self.bond(who, isle.settlement, Some(isle.steward), t, f64::INFINITY);
        if let Some(b) = self.bonds.iter_mut().rev().find(|b| b.who == who) {
            b.slave = true;
        }
        let name = self.name_of(who);
        let line = format!("{name} is taken for a runaway and brought before the lord.");
        self.alerts.push(line.clone());
        self.say(t, line);
        // Before Ḍeshu in the hall.
        let Some(lord) = self.strangers.lord else { return };
        let before = self.free_spot(self.person_pos(lord).add(super::geo::V2::new(2.0, 0.0)));
        if let Some(k) = self.squad.index(who) {
            self.squad.at[k] = before;
            self.squad.goal[k] = before;
            self.squad.route[k].clear();
        }
        self.end_talk();
        // The overseer would speak; the lord doesn't let him.
        let defer = self.say_from(lord, who, "isle", &["defer"], &[], &[]).text;
        if !defer.is_empty() {
            self.say(t, defer);
        }
        let said = self.say_from(lord, who, "isle", &["judgment"], &[], &[]);
        // (Brought in at night, they're judged where he lies: his word is
        // in the news, and they're asked nothing.)
        if self.is_indoors_asleep(lord) {
            self.say(t, said.text);
            return;
        }
        self.met.insert((lord, who));
        self.talk = Some(super::dialogue::Conversation::opened(who, lord, said.text, said.pieces));
    }

    /// Is this squad member bound on the island?
    pub fn bound_on_island(&self, who: PersonId) -> bool {
        self.caught.taken.is_some_and(|(m, _)| m == who) && self.is_bonded(who, self.time)
    }

    /// "What do I owe?": the lord or the steward says how the bond stands.
    pub(super) fn what_is_owed(&mut self, npc: PersonId, who: PersonId) -> String {
        let said = self.say_from(npc, who, "isle", &["owe"], &[], &[]).text;
        self.lesson_done("bond");
        said
    }

    /// Will this person say what the squad member owes? (The lord, or the
    /// steward who holds the bond.)
    pub fn says_what_is_owed(&self, npc: PersonId, who: PersonId) -> bool {
        self.bound_on_island(who) && (self.is_lord(npc) || self.island().is_some_and(|i| i.steward == npc))
    }

    /// A bout in the yard: one of the debtors, to knockout.
    fn yard_bout(&mut self, who: PersonId) {
        let Some(isle) = self.island().cloned() else { return };
        let t = self.time;
        let at = isle.spot(Spot::Yard);
        let foe = self.settlements[isle.settlement as usize]
            .residents
            .iter()
            .copied()
            .filter(|&p| !self.people[p as usize].dead && !self.fighting.contains_key(&p) && !self.keeps_home(p) && self.bond_of(p).is_some_and(|b| b.slave) && self.health_share(p, t) > 0.6)
            .min_by(|&a, &b| self.person_pos(a).dist(at).total_cmp(&self.person_pos(b).dist(at)));
        let Some(foe) = foe else { return };
        let id = self.begin_duel(who, foe, at);
        self.duels.push(super::law::Duel { battle: id, accused: who, town: isle.settlement, fine: 0.0, yard: true });
        let (a, b) = (self.name_of(who), self.name_of(foe));
        self.say(t, format!("The overseers call {a} into the yard against {b}."));
    }

    /// A yard bout is over (`law::duel_over`): fought, the lesson's done.
    pub(super) fn yard_bout_over(&mut self, who: PersonId, won: bool, t: f64) {
        let name = self.name_of(who);
        // (A win is counted with any fight won on the island: `fights.rs`.)
        if won {
            self.say(t, format!("{name} wins the bout. The overseers take note."));
        } else {
            self.say(t, format!("{name} goes down in the yard. The overseers take note all the same."));
        }
        self.lesson_done("yard");
    }

    /// Who would buy them out now: the Lantern Company for a fighter, the
    /// Writers for a head (labour leans to neither).
    pub fn buyer(&self) -> FactionId {
        let d = self.island_deeds().unwrap_or_default();
        let fight = d.fights_won as i32 * 2;
        let head = d.made as i32;
        let lean = self.strangers.lean();
        let lantern = fight > head || (fight == head && lean >= 0);
        factions::id(if lantern { "lantern_company" } else { "writers" })
    }

    /// Bought out: the bond ends, they're counted among the buyer's people,
    /// and they owe it a while's work.
    fn bought_out(&mut self, who: PersonId) {
        let t = self.time;
        let f = self.buyer();
        self.bonds.retain(|b| b.who != who);
        self.caught.owes = Some((f, BOUGHT_FOR));
        let _ = self.join_guild(f);
        let Some(lord) = self.strangers.lord else { return };
        let tag = format!("buyer={}", factions::def(f).key);
        let said = self.say_from(lord, who, "isle", &["bought"], &[tag], &[]).text;
        let name = self.name_of(who);
        let line = format!("{} has paid {name}'s bond. Their ship comes in at the dock.", factions::def(f).name);
        self.alerts.push(line.clone());
        self.say(t, line);
        if !said.is_empty() {
            self.say(t, said);
        }
    }

    /// Off the island stowed away (the sneak path): Diʻub's word, and the
    /// Night Market takes them in. Once.
    pub fn stowed_away(&mut self) {
        if self.caught.night_market {
            return;
        }
        self.caught.night_market = true;
        let t = self.time;
        if let Some(line) = self.diub_line("night_market") {
            let said = format!("{}: “{line}”", super::diub::DIUB);
            self.alerts.push(said.clone());
            self.say(t, said);
        }
        let _ = self.join_guild(factions::id("night_market"));
    }
}
