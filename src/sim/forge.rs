//! A town authored by the town forge (`src/bin/town_forge`): its land as the
//! editor's layers, and what each approved step placed on it. The forge
//! writes `assets/towns/<name>/`; the game reads it when a world is made.
//! Nothing here changes what happens yet: the homes are scenery until the
//! town becomes a settlement (a later step).

use super::geo::V2;
use super::mapedit::MapEdits;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A home as a step file records it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Home {
    pub at: V2,
    /// Facing (radians, the simulation's convention): the door looks this way.
    pub rot: f32,
    /// The model to draw it with (`view/models.rs` names).
    pub model: String,
    pub eldest: bool,
    /// Why it's here (for the notes).
    pub why: String,
}

/// Step 3's record: the founders.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Founding {
    pub landing: V2,
    pub spring: V2,
    pub eldest_site: V2,
    pub homes: Vec<Home>,
    /// Footpaths, each a line of points.
    pub paths: Vec<Vec<V2>>,
    /// The way in from the world's road, as far as the first home.
    pub approach: Vec<V2>,
}

/// Everything the game keeps of a forged town.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Town {
    pub name: String,
    pub founding: Founding,
}

impl Town {
    /// Roughly where the town is (for cameras and the fine ground).
    pub fn centre(&self) -> Option<V2> {
        let f = &self.founding;
        (!f.homes.is_empty()).then(|| f.landing.lerp(f.eldest_site, 0.5))
    }
}

/// What the forge has written so far, as loaded.
pub struct Forge {
    pub land: MapEdits,
    pub town: Town,
}

pub const LAND_FILE: &str = "land.gmap";
pub const FOUNDING_FILE: &str = "step_3_founding.ron";

/// Read a forged town from `assets/towns/<name>/`, if its land is for this
/// world seed. Steps not yet written are simply absent.
pub fn load(dir: &Path, world_seed: u64) -> Result<Option<Forge>, String> {
    let land_path = dir.join(LAND_FILE);
    if !land_path.is_file() {
        return Ok(None);
    }
    let (seed, land) = MapEdits::load_from(&land_path)?;
    if seed != world_seed {
        return Ok(None);
    }
    let founding = match std::fs::read_to_string(dir.join(FOUNDING_FILE)) {
        Ok(text) => ron::from_str(&text).map_err(|e| format!("{}: {e}", FOUNDING_FILE))?,
        Err(_) => Founding::default(),
    };
    let name = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "forge".into());
    Ok(Some(Forge { land, town: Town { name, founding } }))
}
