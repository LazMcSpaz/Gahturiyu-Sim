//! The world's history (Part 5): what happened, to whom, where and when.
//!
//! Each town keeps its own record of events, separate from the screen log:
//! the last `ORDINARY_KEPT` ordinary ones, and serious ones (`SERIOUS`
//! and up) for longer, the last `SERIOUS_KEPT`. Events are numbered once,
//! world-wide, so people can know them by number (Stage 3).

use serde::{Deserialize, Serialize};

use super::person::PersonId;
use super::settlement::SettlementId;
use super::world::World;

// ---- Dials -----------------------------------------------------------------

pub const ORDINARY_KEPT: usize = 200;
pub const SERIOUS_KEPT: usize = 60;
/// Severity from which an event is kept longer.
pub const SERIOUS: f32 = 0.5;

/// What was done.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Deed {
    // Everyday dealings.
    Kindness,
    Loan,
    Slight,
    WorkQuarrel,
    DebtQuarrel,
    TradeDispute,
    // A grudge's ladder.
    Avoid,
    PublicDispute,
    Slander,
    Claim,
    Duel,
    Shun,
    Record,
    Sabotage,
    Theft,
    Beating,
    Brawl,
    Feud,
}

impl Deed {
    /// How serious it is, 0..1.
    pub fn severity(self) -> f32 {
        match self {
            Deed::Kindness | Deed::Loan => 0.05,
            Deed::Slight | Deed::WorkQuarrel | Deed::DebtQuarrel | Deed::TradeDispute | Deed::Avoid => 0.1,
            Deed::PublicDispute | Deed::Slander | Deed::Shun | Deed::Record => 0.3,
            Deed::Claim | Deed::Sabotage => 0.4,
            Deed::Theft | Deed::Duel => 0.5,
            Deed::Brawl | Deed::Beating => 0.7,
            Deed::Feud => 0.8,
        }
    }

    /// Words for it: "{actor} {words} {victim}".
    pub fn words(self) -> &'static str {
        match self {
            Deed::Kindness => "did a kindness to",
            Deed::Loan => "lent money to",
            Deed::Slight => "slighted",
            Deed::WorkQuarrel => "quarrelled over work with",
            Deed::DebtQuarrel => "quarrelled over a debt with",
            Deed::TradeDispute => "fell out over a sale with",
            Deed::Avoid => "turned their back on",
            Deed::PublicDispute => "took a dispute to the hall against",
            Deed::Slander => "spread talk about",
            Deed::Claim => "pressed a claim on the labour of",
            Deed::Duel => "fought a duel with",
            Deed::Shun => "had their people shun",
            Deed::Record => "had a wrong written into the record against",
            Deed::Sabotage => "sabotaged the work of",
            Deed::Theft => "stole from",
            Deed::Beating => "had a beating given to",
            Deed::Brawl => "came to blows in the open with",
            Deed::Feud => "is in a feud with",
        }
    }
}

/// Something that happened.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    pub id: u32,
    pub deed: Deed,
    pub actor: Option<PersonId>,
    pub victim: Option<PersonId>,
    pub town: SettlementId,
    pub t: f64,
    pub severity: f32,
    /// Done in secret, and not (yet) found out: no one knows who.
    pub hidden: bool,
}

/// Every town's record.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct History {
    pub next: u32,
    pub towns: Vec<Vec<Event>>,
}

impl World {
    /// Write an event into a town's history. Returns its number.
    pub fn note(&mut self, deed: Deed, actor: Option<PersonId>, victim: Option<PersonId>, town: SettlementId, t: f64, hidden: bool) -> u32 {
        let h = &mut self.society.history;
        if h.towns.len() <= town as usize {
            h.towns.resize(town as usize + 1, Vec::new());
        }
        let id = h.next;
        h.next += 1;
        let ev = &mut h.towns[town as usize];
        ev.push(Event { id, deed, actor, victim, town, t, severity: deed.severity(), hidden });
        let ordinary = ev.iter().filter(|e| e.severity < SERIOUS).count();
        if ordinary > ORDINARY_KEPT {
            let k = ev.iter().position(|e| e.severity < SERIOUS).unwrap();
            ev.remove(k);
        }
        if ev.len() - ev.iter().filter(|e| e.severity < SERIOUS).count() > SERIOUS_KEPT {
            let k = ev.iter().position(|e| e.severity >= SERIOUS).unwrap();
            ev.remove(k);
        }
        id
    }

    /// A town's events, oldest first.
    pub fn events(&self, town: SettlementId) -> &[Event] {
        self.society.history.towns.get(town as usize).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// An event by number, if it's still on record.
    pub fn event(&self, id: u32) -> Option<&Event> {
        self.society.history.towns.iter().flat_map(|v| v.iter()).find(|e| e.id == id)
    }

    /// How much danger someone knows of nearby, 0..1 (Stage 3).
    pub fn known_danger(&self, _p: PersonId) -> f32 {
        0.0
    }
}
