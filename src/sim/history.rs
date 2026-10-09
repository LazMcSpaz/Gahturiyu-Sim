//! The world's history and gossip (filled in by Stage 3).

use super::person::PersonId;
use super::world::World;

impl World {
    /// How much danger someone knows of nearby, 0..1.
    pub fn known_danger(&self, _p: PersonId) -> f32 {
        0.0
    }
}
