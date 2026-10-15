//! Place files: a place as Laz builds it in the place builder (the map
//! page's builder mode, `docs/place-builder.md`), read into the game.
//!
//! A place file is `assets/world/places/<town key>.json`: the ground and
//! height painted round the town, the trees and rocks stamped where he put
//! them, the roads, and the things (buildings, workplaces, pens, props) by
//! their catalogue keys (`catalogue.rs`). It's turned into a town plan
//! (`plan.rs`), so everything a drawn town does a built one does too: the
//! buildings stand as placed (`Settlement::build_from_plan`), their
//! variants are the ones picked (`plan_styles`), the workplaces where they
//! were put (`plan_places`), the pens as drawn. What a plan read from a
//! sketch never had is the land (`PlaceLand`, kept on the plan):
//!
//! - **ground and height** go under the town as land layers
//!   (`Terrain::authored`, as a forged town's land does): each paint letter
//!   a texture at full strength; each height letter so many metres above or
//!   below the land's own height at the centre;
//! - **rocks** (stamped or placed) become rocks of the land, solid as any;
//! - **trees and bushes** are drawn exactly where they were stamped, and no
//!   others grow inside the built area (`World::plan_woods_at`,
//!   `World::place_stamps_in`);
//! - **roads** are painted on the ground (dirt, or gravel for cobbles) and
//!   kept as ways.
//!
//! Anything the game can't stand up yet (a base building in a town, a prop
//! marked `missing` in the catalogue, painted water) goes to the plan's
//! `later`, with where it was put, so nothing Laz placed is silently lost.
//!
//! Coordinates are metres from the place's centre, x east, y south;
//! rotations degrees clockwise seen from above, 0 = the door looks east
//! (the sim's own facing).

use serde::{Deserialize, Serialize};

use super::catalogue::{self, workplace_of};
use super::geo::V2;
use super::mapedit::{MapEdits, Rock, EDIT_CELL, TEXTURES};
use super::plan::{Plan, PlanBuilding, PlanPen, PlanPlace};
use super::terrain::Terrain;

/// Where the place files live, relative to the game's folder.
pub const PLACES_DIR: &str = "assets/world/places";

fn one() -> f32 {
    1.0
}

/// A stamped tree, bush or rock.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stamp {
    pub k: String,
    pub x: f32,
    pub y: f32,
    #[serde(default = "one")]
    pub s: f32,
}

/// A road or path: its kind (`road`, `path`, `cobbles`, `stairs`), width
/// and points.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Road {
    pub k: String,
    #[serde(default = "one")]
    pub w: f32,
    pub pts: Vec<(f32, f32)>,
}

/// A thing placed: a catalogue key, where, which way, how big.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Thing {
    #[serde(default)]
    pub id: String,
    pub k: String,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub rot: f32,
    #[serde(default = "one")]
    pub s: f32,
    #[serde(default)]
    pub label: String,
    /// The beds the builder worked out (editable there); None: the
    /// catalogue's.
    #[serde(default)]
    pub room: Option<u16>,
    #[serde(default)]
    pub note: String,
    /// What it's used as, by its sign: a trade or service key (`uses`), or
    /// empty for the building's own (a home, or the trade its kind keeps).
    #[serde(default, rename = "use", skip_serializing_if = "String::is_empty")]
    pub use_: String,
}

/// The file as the builder writes it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct PlaceFile {
    pub format: u32,
    pub key: String,
    pub title: String,
    pub notes: String,
    /// Metres across a raster cell.
    pub cell: f32,
    /// Cells across and down.
    pub size: (u32, u32),
    /// The cell (fractional) at the place's centre.
    pub origin: (f32, f32),
    pub ground: String,
    pub height: String,
    pub foliage: Vec<Stamp>,
    pub roads: Vec<Road>,
    pub things: Vec<Thing>,
    pub saved: String,
}

/// What a built place lays on the land, kept on its plan (saved with the
/// world, as plans are).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct PlaceLand {
    pub cell: f32,
    pub size: (u32, u32),
    pub origin: (f32, f32),
    pub ground: String,
    pub height: String,
    /// Trees and bushes, drawn where stamped.
    pub plants: Vec<Stamp>,
    /// Rocks: stamped ones and placed ones (catalogue `rock_*`), as rocks of the land.
    pub rocks: Vec<Stamp>,
    pub roads: Vec<Road>,
}

/// A paint letter's texture (index into `TEXTURES`), or None (`.`, or water,
/// which the game can't paint yet).
pub fn texture_of(letter: char) -> Option<usize> {
    let name = match letter {
        'g' => "Grass",
        's' => "Scrub",
        'd' => "Dirt",
        'n' => "Sand",
        'm' => "Mud",
        'v' => "Gravel",
        'r' => "Rock",
        'w' => "Snow",
        _ => return None,
    };
    TEXTURES.iter().position(|t| t.name == name)
}

/// A height letter's metres from the land's own height at the centre
/// (`A`..`Z` -26..-1, `a`..`z` 0..25), or None (`.`).
pub fn height_of(letter: char) -> Option<f32> {
    match letter {
        'A'..='Z' => Some(letter as i32 as f32 - 'A' as i32 as f32 - 26.0),
        'a'..='z' => Some(letter as i32 as f32 - 'a' as i32 as f32),
        _ => None,
    }
}

/// A rock's kind (`mapedit::ROCKS`) and size for a rock stamp or thing.
fn rock_of(key: &str, s: f32) -> Option<(u8, f32)> {
    Some(match key {
        "rock_small" => (0, 1.0 * s),
        "rock_large" => (0, 2.5 * s),
        "rock_boulder" => (0, 2.0 * s),
        "rock_slab" => (1, 2.0 * s),
        "rock_pillar" => (2, 2.0 * s),
        "rock_scree" => (3, 2.0 * s),
        "rock_outcrop" => (4, 2.0 * s),
        _ => return None,
    })
}

impl PlaceFile {
    pub fn from_json(text: &str) -> Result<PlaceFile, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a place file writes")
    }

    pub fn load(path: &std::path::Path) -> Result<PlaceFile, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        PlaceFile::from_json(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The town plan the place makes: its buildings, workplaces and pens as
    /// placed, its land kept for laying under the town, and anything the
    /// game can't stand up yet listed in `later`.
    pub fn to_plan(&self) -> Plan {
        let mut plan = Plan { town: self.key.clone(), title: self.title.clone(), notes: self.notes.clone(), scale: 1.0, ..Default::default() };
        let mut land = PlaceLand { cell: self.cell, size: self.size, origin: self.origin, ground: self.ground.clone(), height: self.height.clone(), roads: self.roads.clone(), ..Default::default() };
        let place = |t: &Thing| if t.label.is_empty() { format!("{} at ({:.0}, {:.0})", t.k, t.x, t.y) } else { format!("{} ({}) at ({:.0}, {:.0})", t.label, t.k, t.x, t.y) };
        for st in &self.foliage {
            if rock_of(&st.k, st.s).is_some() {
                land.rocks.push(st.clone());
            } else {
                land.plants.push(st.clone());
            }
        }
        for t in &self.things {
            let Some(e) = catalogue::find(&t.k) else {
                plan.later.push(format!("{}: not in the catalogue", place(t)));
                continue;
            };
            if e.missing {
                plan.later.push(format!("{}: {}", place(t), e.blurb));
                continue;
            }
            match e.group {
                "building" => {
                    let v = super::layout::by_key(&t.k).expect("a catalogue building is a variant");
                    let (usual, _) = super::layout::example_of(v);
                    plan.buildings.push(PlanBuilding { label: t.label.clone(), kind: v.kind, variant: Some(t.k.clone()), at: (t.x, t.y), facing: t.rot, size: usual * t.s, room_for: t.room.unwrap_or(e.beds), wants: t.note.clone() });
                }
                "workplace" => {
                    let kind = workplace_of(&t.k).expect("a catalogue workplace is a place kind");
                    plan.places.push(PlanPlace { label: t.label.clone(), kind, at: (t.x, t.y), facing: Some(t.rot), wants: t.note.clone() });
                }
                "pen" => {
                    let species = e.kind.clone();
                    let limit = (e.limit.unwrap_or(4.0) * t.s * t.s).round().max(1.0);
                    plan.pens.push(PlanPen { label: t.label.clone(), species, at: (t.x, t.y), limit });
                }
                "prop" if rock_of(&t.k, t.s).is_some() => land.rocks.push(Stamp { k: t.k.clone(), x: t.x, y: t.y, s: t.s }),
                "foliage" if rock_of(&t.k, t.s).is_some() => land.rocks.push(Stamp { k: t.k.clone(), x: t.x, y: t.y, s: t.s }),
                "foliage" => land.plants.push(Stamp { k: t.k.clone(), x: t.x, y: t.y, s: t.s }),
                "base" => plan.later.push(format!("{}: a squad base building; towns can't have those yet", place(t))),
                _ => plan.later.push(format!("{}: the game can't stand this up in a town yet", place(t))),
            }
        }
        if self.ground.contains('~') {
            plan.later.push("water painted: the game has no fresh-water ground yet (it's left as the land's own)".into());
        }
        plan.land = Some(land);
        plan
    }

    /// Beds in the place's housing, as the builder counts them.
    pub fn capacity(&self) -> u32 {
        self.things.iter().filter_map(|t| catalogue::find(&t.k).filter(|e| e.group == "building").map(|e| t.room.unwrap_or(e.beds) as u32)).sum()
    }

    /// The roles its workplaces and workshops give (job names, sorted, once each).
    pub fn roles(&self) -> Vec<String> {
        let mut out: Vec<String> = self.things.iter().filter_map(|t| catalogue::find(&t.k)).filter(|e| !e.missing && matches!(e.group, "building" | "workplace")).flat_map(|e| e.jobs).collect();
        out.sort();
        out.dedup();
        out
    }
}

impl PlaceLand {
    /// The raster cell (column, row) under an offset from the centre, if any.
    fn cell_at(&self, off: V2) -> Option<usize> {
        if self.cell <= 0.0 {
            return None;
        }
        let i = (off.x / self.cell + self.origin.0).floor();
        let j = (off.y / self.cell + self.origin.1).floor();
        if i < 0.0 || j < 0.0 || i >= self.size.0 as f32 || j >= self.size.1 as f32 {
            return None;
        }
        Some(j as usize * self.size.0 as usize + i as usize)
    }

    /// Is an offset from the centre inside the built area?
    pub fn covers(&self, off: V2) -> bool {
        self.cell_at(off).is_some()
    }

    /// Lay the place's land under a town centred at `centre`: paint and
    /// height (over the land as `terrain` makes it, before any authored
    /// layer), rocks, and the roads painted on.
    pub fn lay(&self, centre: V2, terrain: &Terrain, into: &mut MapEdits) {
        if self.cell > 0.0 {
            let ground: Vec<char> = self.ground.chars().collect();
            let height: Vec<char> = self.height.chars().collect();
            let (w, h) = (self.size.0 as f32 * self.cell, self.size.1 as f32 * self.cell);
            let x0 = centre.x - self.origin.0 * self.cell;
            let y0 = centre.y - self.origin.1 * self.cell;
            let h0 = terrain.height(centre);
            let (i0, i1) = ((x0 / EDIT_CELL).floor() as i64, ((x0 + w) / EDIT_CELL).floor() as i64);
            let (j0, j1) = ((y0 / EDIT_CELL).floor() as i64, ((y0 + h) / EDIT_CELL).floor() as i64);
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let q = V2::new((i as f32 + 0.5) * EDIT_CELL, (j as f32 + 0.5) * EDIT_CELL);
                    let Some(k) = self.cell_at(q.sub(centre)) else { continue };
                    if let Some(t) = ground.get(k).copied().and_then(texture_of) {
                        into.paint.set(i, j, ((t as u8 + 1) << 4) | 15);
                    }
                    if let Some(dh) = height.get(k).copied().and_then(height_of) {
                        into.height.set(i, j, h0 + dh - terrain.height(q));
                    }
                }
            }
        }
        for (n, r) in self.rocks.iter().enumerate() {
            let Some((kind, size)) = rock_of(&r.k, r.s) else { continue };
            let pos = centre.add(V2::new(r.x, r.y));
            let seed = super::rng::key(&[super::roster::text_key(&r.k), n as u64, (pos.x * 10.0) as i64 as u64, (pos.y * 10.0) as i64 as u64]) as u32;
            into.rocks.push(Rock { pos, kind, size, rot: (seed % 628) as f32 / 100.0, seed });
        }
        for road in &self.roads {
            let tex = match road.k.as_str() {
                "cobbles" => "Gravel",
                "stairs" => "Rock",
                _ => "Dirt",
            };
            let Some(t) = TEXTURES.iter().position(|x| x.name == tex) else { continue };
            let paint = ((t as u8 + 1) << 4) | 15;
            for seg in road.pts.windows(2) {
                let (a, b) = (centre.add(V2::new(seg[0].0, seg[0].1)), centre.add(V2::new(seg[1].0, seg[1].1)));
                let steps = (a.dist(b) / (EDIT_CELL * 0.5)).ceil().max(1.0) as usize;
                for s in 0..=steps {
                    let p = a.lerp(b, s as f32 / steps as f32);
                    into.paint.set((p.x / EDIT_CELL).floor() as i64, (p.y / EDIT_CELL).floor() as i64, paint);
                }
            }
        }
    }

    /// The trees and bushes stamped, at world positions, for a centre.
    pub fn plants_at(&self, centre: V2) -> impl Iterator<Item = (V2, &str, f32)> + '_ {
        self.plants.iter().map(move |s| (centre.add(V2::new(s.x, s.y)), s.k.as_str(), s.s))
    }
}

/// Every place file in the folder, as plans by town key.
pub fn load_places(dir: &std::path::Path) -> Vec<(String, Plan)> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<std::path::PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json")).collect();
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        match PlaceFile::load(&path) {
            Ok(mut f) => {
                if f.key.is_empty() {
                    f.key = stem.clone();
                }
                out.push((stem, f.to_plan()));
            }
            Err(e) => eprintln!("place skipped: {e}"),
        }
    }
    out
}

/// The folder the game reads place files from (beside the plans), if found.
pub fn places_dir() -> Option<std::path::PathBuf> {
    super::plan::plans_dir().and_then(|d| d.parent().map(|p| p.join("places")))
}

/// Now, as the builder writes `saved` (UTC, to the second).
pub fn now_stamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

impl PlaceFile {
    /// A new, empty place for a mark: its area's grid (2 m cells, the
    /// origin at the middle cell), nothing painted, set or placed.
    pub fn blank(key: &str, title: &str, size: (usize, usize)) -> PlaceFile {
        let n = size.0 * size.1;
        PlaceFile {
            format: 1,
            key: key.to_string(),
            title: title.to_string(),
            cell: super::grounds::CELL,
            size: (size.0 as u32, size.1 as u32),
            origin: ((size.0 / 2) as f32, (size.1 / 2) as f32),
            ground: ".".repeat(n),
            height: ".".repeat(n),
            ..Default::default()
        }
    }

    /// The place file for a key, as saved in the game's places folder, if any.
    pub fn for_key(key: &str) -> Option<PlaceFile> {
        let path = places_dir()?.join(format!("{key}.json"));
        PlaceFile::load(&path).ok()
    }

    /// Write it to the game's places folder (`<key>.json`), stamped now.
    /// Returns where.
    pub fn save(&mut self) -> Result<std::path::PathBuf, String> {
        let dir = places_dir().ok_or("no places folder near the game")?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        self.saved = now_stamp();
        let path = dir.join(format!("{}.json", self.key));
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }

    /// The box the place's grid covers round a centre: (x0, y0, x1, y1).
    pub fn bounds(&self, centre: V2) -> (f32, f32, f32, f32) {
        let (x0, y0) = (centre.x - self.origin.0 * self.cell, centre.y - self.origin.1 * self.cell);
        (x0, y0, x0 + self.size.0 as f32 * self.cell, y0 + self.size.1 as f32 * self.cell)
    }
}
