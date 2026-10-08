//! Settlements. Every town is mixed; the founding race only tilts who lives
//! there. Any town on the coast draws a Horaro community that lives on stilts
//! over the water just off its shore, so coastal towns of every race end up
//! depending on them.
//!
//! Buildings follow the project's `architecture.md`: Roduro grow their homes
//! from living stone, Horaro weave domes on Roduro-grown stone stilts, Qotiro
//! quarry stepped blocks (and keep one small temple wherever enough of them
//! live away from home). Ṭaḍoro build nothing; they lodge with others.

use serde::{Deserialize, Serialize};

use super::geo::{self, V2};
use super::person::PersonId;
use super::race::Race;
use super::rng::Rng;

pub type SettlementId = u16;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    /// Grown dark stone, rounded and banded. One household.
    RoduroHome,
    /// Woven dome on stone pillars just offshore, with a deck. One household.
    HoraroStilt,
    /// Quarried, stepped, many small windows. Several households.
    QotiroBlock,
    /// The stepped temple-fortress at the heart of a Qotiro town.
    QotiroTemple,
    /// A compact two-tier hall with a temple crown, for Qotiro living away from home.
    QotiroHall,
    /// The shared fire pit every settlement gathers around.
    Hearth,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Building {
    pub pos: V2,
    pub kind: BuildingKind,
    /// Footprint width in metres.
    pub size: f32,
    /// Facing, radians.
    pub rot: f32,
    pub seed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub buildings: Vec<Building>,
    /// Distance from `pos` that covers every building, stilts included.
    pub reach: f32,
}

impl Settlement {
    /// Rough footprint of the land part of town, in metres.
    pub fn radius(&self) -> f32 {
        40.0 + (self.residents.len() as f32).sqrt() * 9.0
    }

    /// Lay the town out from who lives there. Returns, for each resident (in
    /// `residents` order), the index of the building they call home.
    pub fn build(&mut self, races: &[Race], seed: u64) -> Vec<u16> {
        let mut rng = Rng::from_keys(&[seed, self.id as u64, 0x4255_494C]);
        let count = |r: Race| races.iter().filter(|&&x| x == r).count();
        let horaro = if self.stilts.is_some() { count(Race::Horaro) } else { 0 };
        let qotiro = count(Race::Qotiro);
        let landfolk = races.len() - horaro;
        let qotiro_town = self.founders == Race::Qotiro;

        let mut b: Vec<Building> = vec![Building { pos: self.pos, kind: BuildingKind::Hearth, size: 6.0, rot: 0.0, seed: rng.next_u64() }];
        let mut land: Vec<(BuildingKind, f32)> = Vec::new();
        if qotiro_town {
            land.push((BuildingKind::QotiroTemple, 46.0 + (landfolk as f32).sqrt() * 1.5));
            for _ in 0..landfolk.div_ceil(12) {
                land.push((BuildingKind::QotiroBlock, rng.range(14.0, 20.0)));
            }
        } else {
            if qotiro >= 12 {
                land.push((BuildingKind::QotiroHall, 16.0));
            }
            for _ in 0..landfolk.div_ceil(5) {
                land.push((BuildingKind::RoduroHome, rng.range(7.0, 12.5)));
            }
        }

        // Land buildings spiral out from the hearth, biggest first, and stay
        // on dry ground.
        let mut ring = 14.0f32;
        let mut ang = rng.f32() * std::f32::consts::TAU;
        for (kind, size) in land {
            let mut pos = self.pos;
            for _ in 0..400 {
                ang += 2.399_963 / (1.0 + ring / 60.0);
                ring += size * 0.11;
                let r = ring + size * 0.5 + rng.range(-3.0, 3.0);
                let cand = self.pos.add(V2::new(ang.cos(), ang.sin()).scale(r));
                let clear = b.iter().all(|o| o.pos.dist(cand) > (o.size + size) * 0.62 + 3.0);
                if clear && geo::inland(cand) > size {
                    pos = cand;
                    break;
                }
            }
            let rot = (self.pos.y - pos.y).atan2(self.pos.x - pos.x) + rng.range(-0.4, 0.4);
            b.push(Building { pos, kind, size, rot, seed: rng.next_u64() });
        }
        let land_count = b.len();

        // Stilt homes, clustered in the shallows off the shore.
        if let Some(st) = self.stilts {
            let n = horaro.div_ceil(4);
            let mut ring = 6.0f32;
            let mut ang = rng.f32() * std::f32::consts::TAU;
            for _ in 0..n {
                let size = rng.range(7.0, 10.0);
                let mut pos = st;
                for _ in 0..400 {
                    ang += 2.399_963 / (1.0 + ring / 40.0);
                    ring += 1.1;
                    let cand = st.add(V2::new(ang.cos(), ang.sin()).scale(ring + rng.range(-2.0, 2.0)));
                    let clear = b[land_count..].iter().all(|o| o.pos.dist(cand) > (o.size + size) * 0.75 + 4.0);
                    // In the water, but not far out.
                    let off = -geo::inland(cand);
                    if clear && off > 25.0 && off < 260.0 {
                        pos = cand;
                        break;
                    }
                }
                b.push(Building { pos, kind: BuildingKind::HoraroStilt, size, rot: rng.f32() * 6.28, seed: rng.next_u64() });
            }
        }

        self.reach = b.iter().map(|o| o.pos.dist(self.pos) + o.size).fold(self.radius(), f32::max);

        // Who lives where. Horaro on the stilts, everyone else on land; each
        // household fills before the next.
        let land_homes: Vec<u16> = (1..land_count as u16).collect();
        let stilt_homes: Vec<u16> = (land_count as u16..b.len() as u16).collect();
        let mut out = Vec::with_capacity(races.len());
        let (mut li, mut si) = (rng.below(land_homes.len().max(1)), 0usize);
        for &r in races {
            let home = if r == Race::Horaro && !stilt_homes.is_empty() {
                si += 1;
                stilt_homes[(si / 4) % stilt_homes.len()]
            } else if !land_homes.is_empty() {
                li += 1;
                let per = if qotiro_town { 12 } else { 5 };
                land_homes[(li / per) % land_homes.len()]
            } else {
                0
            };
            out.push(home);
        }
        self.buildings = b;
        out
    }
}
