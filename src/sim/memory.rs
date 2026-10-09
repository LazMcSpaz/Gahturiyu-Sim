//! Memory and grudges (filled in by Stage 2).

use serde::{Deserialize, Serialize};

use super::person::PersonId;
use super::world::World;

/// Something someone remembers: who wronged or helped them, how, how much.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Memory {
    pub about: PersonId,
    pub amount: f32,
    pub day: i32,
}

/// How one household feels about another.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Feeling {
    pub other: u32,
    pub warmth: f32,
    pub stage: u8,
}

impl World {
    /// How aggrieved someone is, 0..1, from the wrongs they remember.
    pub fn grievance_of(&self, p: PersonId) -> f32 {
        let bad: f32 = self.mind(p).memories.iter().filter(|m| m.amount < 0.0).map(|m| -m.amount).sum();
        (bad / 3.0).min(1.0)
    }
}
