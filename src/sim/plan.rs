//! Town plans: a town laid out by hand from one of Laz's drawings
//! (`docs/town-plans.md`). A plan is a file in `assets/world/plans/<town
//! key>.ron`, transcribed from the sketch: every building with its place,
//! facing, size and who it houses; the workplaces drawn outside or in a
//! building; the pens; and, for each thing the game can't yet show as
//! drawn, a note of what was asked (`wants`, `later`) so the shape can be
//! made afterwards without going back to the picture.
//!
//! A town with a plan is laid out from it instead of spiralled from the
//! hearth (`Settlement::build_from_plan`); its workplaces stand where the
//! plan puts them, and any it needs but the plan doesn't give are spiralled
//! in after (`society::lay_out`); its pens are the plan's
//! (`animals/place.rs`). The hearth is the plan's origin: building 0, as in
//! every town, so a roster's `home: Some(n)` counts the plan's buildings
//! from 1. Plans are kept in the world (`World::plans`, saved) because a
//! town is laid out again when its customs shift.
//!
//! Places are metres east and south of the town's centre; facings are
//! degrees, 0 east, 90 south, 180 west, 270 north (the simulation's own
//! convention, y growing southward, in degrees so the file reads easily).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::geo::V2;
use super::jobs::PlaceKind;
use super::race::Race;
use super::rng::{self, Rng};
use super::settlement::{Building, BuildingKind, Settlement, SettlementId};
use super::society::Workplace;
use super::world::World;

/// Where the plans live, relative to the game's folder.
pub const PLANS_DIR: &str = "assets/world/plans";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct Plan {
    /// The town's key (`assets/world/towns.ron`).
    pub town: String,
    /// What the drawing is headed.
    pub title: String,
    /// Laz's margin notes, kept as written.
    pub notes: String,
    /// The drawing it was read from (a path under `docs/`), and the reading:
    /// metres per pixel and the pixel taken as the centre.
    pub sketch: String,
    pub scale: f32,
    pub origin: (f32, f32),
    pub buildings: Vec<PlanBuilding>,
    pub places: Vec<PlanPlace>,
    pub pens: Vec<PlanPen>,
    /// What's drawn that nothing stands for yet.
    pub later: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PlanBuilding {
    pub label: String,
    pub kind: BuildingKind,
    /// A variant key (`layout::VARIANTS`); None: chosen from who lives
    /// there, as in any town.
    pub variant: Option<String>,
    pub at: (f32, f32),
    pub facing: f32,
    /// Metres across.
    pub size: f32,
    /// Sleeping places; 0 for a building nobody lives in.
    pub room_for: u16,
    /// What the drawing asks for beyond what the variant gives.
    pub wants: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlanPlace {
    pub label: String,
    pub kind: PlaceKind,
    pub at: (f32, f32),
    /// None: toward the centre.
    #[serde(default)]
    pub facing: Option<f32>,
    #[serde(default)]
    pub wants: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlanPen {
    pub label: String,
    /// A species key (`animals::SPECIES`).
    pub species: String,
    pub at: (f32, f32),
    /// How many it holds.
    pub limit: f32,
}

impl Default for PlanBuilding {
    fn default() -> Self {
        PlanBuilding { label: String::new(), kind: BuildingKind::QotiroBlock, variant: None, at: (0.0, 0.0), facing: 0.0, size: 12.0, room_for: 0, wants: String::new() }
    }
}

impl PlanPen {
    /// The species it holds, if the key names one.
    pub fn species(&self) -> Option<super::animals::Sp> {
        super::animals::species(&self.species)
    }
}

impl Plan {
    pub fn from_ron(text: &str) -> Result<Plan, String> {
        ron::from_str(text).map_err(|e| e.to_string())
    }

    pub fn to_ron(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap_or_default()
    }

    pub fn load(path: &Path) -> Result<Plan, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Plan::from_ron(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// A plan's offset as a vector.
    fn off(at: (f32, f32)) -> V2 {
        V2::new(at.0, at.1)
    }

    /// Sleeping places in all.
    pub fn room(&self) -> usize {
        self.buildings.iter().map(|b| b.room_for as usize).sum()
    }
}

/// Every plan in the folder, by town key.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Plans {
    pub towns: BTreeMap<String, Plan>,
}

impl Plans {
    /// The plans in `assets/world/plans/` (none there: none at all). A file
    /// that won't read is said on the console and skipped.
    pub fn load() -> Plans {
        match plans_dir() {
            Some(dir) => Plans::load_from(&dir),
            None => Plans::default(),
        }
    }

    pub fn load_from(dir: &Path) -> Plans {
        let mut out = Plans::default();
        let Ok(entries) = std::fs::read_dir(dir) else { return out };
        let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ron")).collect();
        files.sort();
        for path in files {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            match Plan::load(&path) {
                Ok(mut p) => {
                    if p.town.is_empty() {
                        p.town = stem.clone();
                    }
                    out.towns.insert(stem, p);
                }
                Err(e) => eprintln!("plan skipped: {e}"),
            }
        }
        out
    }

    pub fn for_town(&self, key: &str) -> Option<&Plan> {
        self.towns.get(key).filter(|p| !p.buildings.is_empty())
    }
}

/// The plans folder, if there is one near the game or the source.
pub fn plans_dir() -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.extend(cwd.ancestors().take(4).map(Path::to_path_buf));
    }
    if let Ok(exe) = std::env::current_exe() {
        roots.extend(exe.ancestors().skip(1).take(4).map(Path::to_path_buf));
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(dir));
    }
    roots.into_iter().map(|r| r.join(PLANS_DIR)).find(|p| p.is_dir())
}

impl Settlement {
    /// Lay the town out from its plan: the hearth at the centre, then the
    /// plan's buildings as drawn. Returns each resident's home (in
    /// `residents` order): the plan's homes fill in order, each to its
    /// `room_for`, and go round again if more people come than it has room
    /// for (a roster sets the real head count).
    pub fn build_from_plan(&mut self, plan: &Plan, races: &[Race], seed: u64) -> Vec<u16> {
        let mut rng = Rng::from_keys(&[seed, self.id as u64, 0x504C_414E]);
        let mut b: Vec<Building> = vec![Building { pos: self.pos, kind: BuildingKind::Hearth, size: 6.0, rot: 0.0, seed: rng.next_u64() }];
        for pb in &plan.buildings {
            b.push(Building { pos: self.pos.add(Plan::off(pb.at)), kind: pb.kind, size: pb.size, rot: pb.facing.to_radians(), seed: rng.next_u64() });
        }
        self.reach = b.iter().map(|o| o.pos.dist(self.pos) + o.size).fold(self.radius(), f32::max);
        let homes: Vec<(u16, u16)> = plan.buildings.iter().enumerate().filter(|(_, pb)| pb.room_for > 0).map(|(i, pb)| (i as u16 + 1, pb.room_for)).collect();
        let mut out = Vec::with_capacity(races.len());
        let (mut hi, mut filled) = (0usize, 0u16);
        for _ in races {
            if homes.is_empty() {
                out.push(0);
                continue;
            }
            if filled >= homes[hi % homes.len()].1 {
                hi += 1;
                filled = 0;
            }
            out.push(homes[hi % homes.len()].0);
            filled += 1;
        }
        self.buildings = b;
        out
    }
}

impl World {
    /// The plan a town was laid out from, if any.
    pub fn plan_of(&self, t: SettlementId) -> Option<&Plan> {
        self.plans.iter().find(|(id, _)| *id == t).map(|(_, p)| p)
    }

    /// The workplaces a town's plan puts down, at their drawn places.
    pub(super) fn plan_places(&self, t: SettlementId) -> Vec<Workplace> {
        let Some(plan) = self.plan_of(t) else { return Vec::new() };
        let c = self.settlements[t as usize].pos;
        plan.places
            .iter()
            .enumerate()
            .map(|(i, pp)| {
                let pos = c.add(Plan::off(pp.at));
                let rot = pp.facing.map(f32::to_radians).unwrap_or_else(|| (c.y - pos.y).atan2(c.x - pos.x));
                Workplace { kind: pp.kind, pos, rot, seed: rng::key(&[self.seed, t as u64, i as u64, 0x504C_4143]) }
            })
            .collect()
    }

    /// The variants a town's plan names, over those chosen from the residents.
    pub(super) fn plan_styles(&mut self, t: SettlementId) {
        let Some(plan) = self.plan_of(t).cloned() else { return };
        let s = &mut self.settlements[t as usize];
        for (i, pb) in plan.buildings.iter().enumerate() {
            let Some(key) = pb.variant.as_deref() else { continue };
            let Some(v) = super::layout::VARIANTS.iter().find(|v| v.key == key) else {
                eprintln!("plan {}: no variant {key:?} for {:?}", plan.town, pb.label);
                continue;
            };
            if let Some(slot) = s.styles.get_mut(i + 1) {
                *slot = Some(v.key);
            }
        }
    }
}
