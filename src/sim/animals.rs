//! Animals: what lives in the world besides people — wild herds and the
//! things that hunt them, livestock, silk colonies, carrion birds, fish —
//! and how they behave, fight, multiply and what they yield.
//!
//! The same promises as the rest of the simulation, kept the same ways:
//!
//! - **Nothing is ticked per animal.** A herd's place comes from the clock;
//!   its numbers, a pen's hunger and produce, a colony's cocoons are each
//!   one value at one moment plus a rule for any later one. Thousands of
//!   animals cost nothing while nobody is looking.
//! - **What happens never depends on who is watching.** Herds and packs
//!   exist everywhere as small records, whether the squad is near or not;
//!   being near only decides how they are drawn. Far off, wildlife is read
//!   as numbers per region (the sum of the herds there); nearer, as herds on
//!   their rounds; close up, as individual animals whose looks come from
//!   the herd's seed and never change.
//! - **Randomness is keyed** by what is decided (this herd, this hour, these
//!   travellers), never drawn from a running source.
//! - **Attacks run on the world's one timeline.** Each hour, every hunter's
//!   round for that hour is checked against every traveller's schedule; the
//!   moment a pack picks travellers up, and the moment it strikes, are
//!   events handled in time order with bandit ambushes and caravans. The
//!   fight is fought at once, blow by blow, under the ordinary fight rules,
//!   and its end is handled by the same code that ends a roadside ambush.
//!   So a pack's attack on the far side of the world goes exactly as it
//!   would have beside you.
//!
//! What society will want from animals later (herders, hunting, crime,
//! trade goods) is exposed as plain functions and left there: owners are
//! opaque numbers, yields are names and amounts.
//!
//! Files: `species` (the data table), `region` (country, the Overgrowth
//! stand-in, fish, dive danger), `herd` (numbers, rounds, individuals),
//! `attack` (the timeline, fights, the squad's run-ins, hunting),
//! `livestock` (pens, pack animals, tamed hounds), `hooks` (grazing,
//! Briarbacks, silk, bodies and Bonepickers, carcasses, taming), `place`
//! (stocking a new world).

mod attack;
mod herd;
mod hooks;
mod livestock;
mod place;
mod region;
mod species;

use serde::{Deserialize, Serialize};

pub use attack::*;
pub use herd::*;
pub use hooks::*;
pub use livestock::*;
pub use region::*;
pub use species::*;

use super::combat::Side;
use super::geo::V2;
use super::group::GroupId;
use super::person::PersonId;
use super::world::{World, HOUR};

/// Fights with animals in them are numbered from here up, apart from the
/// world's other fights.
pub const FIRST_BATTLE: u32 = 0x8000_0000;
/// The side wild animals fight on.
pub const ANIMAL_SIDE: Side = 3;
/// The side travellers fight on when animals attack them. (The same number
/// `encounters.rs` gives ambushed travellers, so the code that ends a
/// roadside fight knows who they are.)
pub const TRAVELLER_SIDE: Side = 2;
/// What stands in for the bandits' group in the record of a far-off fight
/// with animals: no camp has this id, so the camp side of ending the fight
/// finds nothing to do.
pub const NO_CAMP: GroupId = GroupId::MAX;
/// The hour of the day when herds, pens and colonies take stock.
pub const STOCK_HOUR: i64 = 6;
/// How often (game seconds) the animals round the squad are looked over:
/// who runs from it, who comes for it.
pub const LOOK_EVERY: f64 = 2.0;
/// Herds with their home within this of the squad are the ones looked over.
pub const NEAR: f32 = 1500.0;
/// How many attacks the record keeps.
pub const ATTACK_LOG: usize = 48;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct AnimalStats {
    /// Times animals have attacked people, anywhere.
    pub attacks: u32,
    /// Times a hunter picked travellers up and thought better of it.
    pub left_alone: u32,
    pub people_downed: u32,
    pub people_killed: u32,
    pub animals_killed: u32,
    pub bodies_cleared: u32,
    /// Animals seen up close so far (nothing depends on this).
    pub met: u32,
    pub tamed: u32,
}

/// One attack by animals on people, for the record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AttackRecord {
    pub t: f64,
    pub herd: u32,
    pub sp: Sp,
    /// The travellers attacked (`None`: your squad).
    pub victim: Option<GroupId>,
    pub at: V2,
    pub pack: u8,
    pub people: u8,
    pub pack_might: f32,
    pub their_might: f32,
    pub battle: u32,
    /// Filled in when the fight is over.
    pub over: bool,
    pub animals_won: bool,
    /// Your squad started it (a hunt), not the animals.
    pub by_squad: bool,
    pub people_killed: u8,
    pub animals_lost: u8,
}

/// Something of the animals' waiting for its moment on the timeline.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Due {
    pub t: f64,
    pub what: What,
    pub herd: u32,
    pub victim: GroupId,
    pub battle: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum What {
    /// A hunter's round brings it within reach of travellers: it decides.
    Notice,
    /// A pack that has been shadowing travellers closes.
    Strike,
    /// A far-off fight, already fought, is over for the animals in it.
    FrayEnd,
}

/// Everything about animals that the world stores. All of it is saved.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Animals {
    pub regions: Vec<Region>,
    pub herds: Vec<Herd>,
    pub pens: Vec<Pen>,
    pub led: Vec<Led>,
    pub hounds: Vec<Hound>,
    pub colonies: Vec<Colony>,
    pub carcasses: Vec<Carcass>,
    pub fish: Vec<FishStock>,
    /// Bodies Bonepickers have cleared lately: (who, when).
    pub cleared: Vec<(PersonId, f64)>,

    // --- The timeline ---------------------------------------------------------
    /// The last whole game-hour whose rounds have been checked.
    pub hour_done: i64,
    pub pending: Vec<Due>,
    /// Fights going on with animals in them.
    pub frays: Vec<Fray>,
    pub next_battle: u32,
    pub next_carcass: u32,
    pub next_hound: u32,
    pub tame_tries: u32,
    pub attacks: Vec<AttackRecord>,
    pub stats: AnimalStats,

    // --- Round the squad (drawing, and the squad's own run-ins) ---------------
    /// Herds whose home is near the squad, and where the squad was when
    /// that list was made.
    pub near: Vec<u32>,
    pub near_at: V2,
    pub looked_at: f64,
    pub stepped_at: f64,
}

impl Due {
    fn order(&self) -> (What, u32, GroupId, u32) {
        (self.what, self.herd, self.victim, self.battle)
    }
}

/// What the animals have due next.
enum Next {
    Due(usize),
    Body(PersonId),
}

impl World {
    /// The animals' part of the world's timeline. Called from the top of
    /// the event loop in `World::step`; returns true if it did something
    /// (so the loop looks again from the top).
    ///
    /// In order: the hour's stock-take and the check of hunters' rounds
    /// against travellers' schedules, once per game-hour, straight after
    /// that hour's departures are planned; then whichever animal event is
    /// due, if nothing else on the timeline comes before it; then, once per
    /// step, the animals round the squad.
    pub(super) fn animal_events(&mut self, hour_t: f64) -> bool {
        if self.animals.regions.is_empty() {
            return false;
        }
        if self.animals.hour_done < self.hour_done {
            let h = self.animals.hour_done + 1;
            self.animals.hour_done = h;
            self.animals_hour(h);
            return true;
        }
        if let Some((t, next)) = self.next_animal_event() {
            if t <= self.time && t < hour_t {
                let other = [self.next_event().map(|e| e.0), self.next_cargo().map(|c| c.0)].into_iter().flatten().fold(f64::INFINITY, f64::min);
                // The animals' side of a fight's ending comes straight after
                // the world's own ending of that fight at the same instant;
                // anything else of theirs goes first on a tie.
                let after = matches!(next, Next::Due(i) if self.animals.pending[i].what == What::FrayEnd);
                if t < other || (t == other && !after) {
                    match next {
                        Next::Due(i) => {
                            let d = self.animals.pending.remove(i);
                            match d.what {
                                What::Notice => self.pack_notices(d.herd, d.victim, d.t),
                                What::Strike => self.pack_strikes(d.herd, d.victim, d.t),
                                What::FrayEnd => self.far_fray_ends(d.battle, d.t),
                            }
                        }
                        Next::Body(pid) => self.clear_body(pid, t),
                    }
                    return true;
                }
            }
        }
        if self.animals.stepped_at != self.time {
            self.animals.stepped_at = self.time;
            self.animals_by_the_squad();
        }
        false
    }

    fn next_animal_event(&self) -> Option<(f64, Next)> {
        let due = self.animals.pending.iter().enumerate().min_by(|(_, a), (_, b)| a.t.total_cmp(&b.t).then(a.order().cmp(&b.order()))).map(|(i, d)| (d.t, Next::Due(i)));
        let body = self.next_body_cleared().map(|(t, pid)| (t, Next::Body(pid)));
        match (due, body) {
            (Some(d), Some(b)) => Some(if b.0 < d.0 { b } else { d }),
            (a, b) => a.or(b),
        }
    }

    /// The top of game-hour `h`.
    fn animals_hour(&mut self, h: i64) {
        let t = h as f64 * HOUR;
        if h.rem_euclid(24) == STOCK_HOUR {
            self.settle_herds(t);
            self.settle_pens(t);
            self.settle_colonies(t);
            self.settle_hounds(t);
        }
        self.animals.carcasses.retain(|c| c.gone_at > t);
        self.watch_the_roads(h);
        // Whoever is near the squad gets looked up afresh.
        self.animals.near_at = V2::new(-1e9, -1e9);
    }

    /// How many animals there are, wild and kept, for the readout.
    pub fn animal_count(&self) -> (usize, usize) {
        let t = self.time;
        let wild = self.animals.herds.iter().map(|h| h.alive(t)).sum();
        let kept = self.animals.pens.iter().map(|p| p.now(t).count.floor() as usize).sum();
        (wild, kept)
    }
}
