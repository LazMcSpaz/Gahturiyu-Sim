//! Settlements. Every town is mixed; the founding race only tilts who lives
//! there. Any town on the coast draws a Horaro community that lives on stilts
//! over the water just off its shore, so coastal towns of every race end up
//! depending on them.

use super::geo::V2;
use super::person::PersonId;
use super::race::Race;

pub type SettlementId = u16;

#[derive(Clone, Debug)]
pub struct Settlement {
    pub id: SettlementId,
    pub name: String,
    pub pos: V2,
    /// Who built it. Shapes the name and who it attracts, nothing more.
    pub founders: Race,
    pub coastal: bool,
    /// Where the Horaro stilt houses stand, just offshore. Coastal towns only.
    pub stilts: Option<V2>,
    /// How strongly the town draws people: a few cities, many villages.
    pub size: f32,
    /// Everyone whose home this is, whether or not they are in right now.
    pub residents: Vec<PersonId>,
}

impl Settlement {
    /// Rough footprint in metres, for drawing and for milling residents.
    pub fn radius(&self) -> f32 {
        40.0 + (self.residents.len() as f32).sqrt() * 9.0
    }
}
