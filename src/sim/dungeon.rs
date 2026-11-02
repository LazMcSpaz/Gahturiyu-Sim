//! Sites from the story-led dungeon generator (Laz, 2 Nov 2026).
//!
//! The generator is another program (its agent and workspace are separate:
//! the interface is agreed in its `ENGINE-INTEGRATION.md`). It exports a
//! `LayoutResult` JSON file: a site of one-metre cells with their heights,
//! water and decks, and its doors, ladders, plaques, chest and exit. This
//! reads that file as given (nothing generated here, nothing renamed) and
//! makes it a place the squad can be in.
//!
//! **Where.** No site has a mouth onto the overworld yet (the generator
//! doesn't export one), so a site stands in its own pocket east of the map
//! (`POCKET_*`), its local metres translated (`Site::origin`, `base_h`). A
//! squad member goes in by a portal (`World::enter_site`) to the site's
//! spawn, and out by its Exit.
//!
//! **Ways.** The overworld's way-finder (`walk.rs`) has one level; a site has
//! several (floors at many heights, decks over floors and pits, water). So
//! inside a site the way is found over its cells: a node is a cell and a
//! level (`Lv::Low`: the floor, or the water's surface; `Lv::Deck`), and the
//! squad member walks, climbs (ladders) and swims (water) along it. Laz:
//! "Build ladders and swimming." Rules taken from the generator's own
//! movement so the two agree: a step up or down is at most `STEP`; a ladder
//! joins its cell's floor and deck; a swimmer can climb out onto a bank at
//! most `STEP` above the surface; a closed door fills its doorway cells
//! (walkers open it on the way, at `DOOR_SECS`); a pit has no floor (its deck,
//! if any, is a bridge). A walker is one character (Laz: "parties of a
//! single character" for testing): no width beyond a cell.
//!
//! **Rules kept.** A way is a pure function of the site and its doors.
//! Walking inside a site is the squad's own stepping (the player exception),
//! with climbing paid for by the metre (`World::climb`). The site and who is
//! in it are saved.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};

use serde::{Deserialize, Serialize};

use super::geo::{V2, WORLD_SIZE};
use super::person::PersonId;
use super::world::World;

/// Export versions read: 0 (no `schema_version`, the legacy files) and 1.
pub const SCHEMAS: [u32; 2] = [0, 1];
/// The most a walker steps up or down between neighbouring cells, metres
/// (the generator's own rule).
pub const STEP: f32 = 0.23;
/// Seconds per metre climbed on a ladder (the generator takes 1.2 s for 2.4 m).
pub const CLIMB_SECS_PER_M: f32 = 0.5;
/// A swimmer's pace as a share of walking (the generator's 2.7 against 5.0).
pub const SWIM_PACE: f32 = 0.54;
/// The furthest a walker drops from a ledge into water, metres.
pub const DROP_INTO_WATER: f32 = 2.5;
/// Seconds to open a closed door on the way through.
pub const DOOR_SECS: f32 = 1.0;
/// How far from a thing a walker can use it (read a plaque, open a chest),
/// metres across, and up or down.
pub const USE_REACH: f32 = 1.6;
pub const USE_RISE: f32 = 1.3;
/// A jump (the generator's own numbers, so its jumpable gaps agree): up at
/// this speed, pulled down at `GRAVITY`; about a metre up, two across.
pub const JUMP: f32 = 6.2;
pub const GRAVITY: f32 = 18.0;
/// A jump made while moving carries at least this fast (the generator's
/// running pace), so the gaps it calls jumpable are.
pub const JUMP_CARRY: f32 = 5.0;
/// A dodge backwards: this far, this fast, for this share of the most
/// stamina they can have.
pub const DODGE_M: f32 = 1.6;
pub const DODGE_SPEED: f32 = 6.0;
pub const DODGE_STAMINA: f32 = 0.04;
/// A squad mate following the one you control finds their way again at
/// most this often (seconds); how far behind they keep is their setting
/// (`behaviour::Follow::gap`). Holding a spot, they go back once this far off it.
pub const FOLLOW_REPLAN: f64 = 0.6;
pub const HOLD_SLACK: f32 = 0.8;
/// Sprinting (Shift): this much faster, for this share of their most
/// stamina a metre (by the metre, like climbing), not below this share left.
pub const SPRINT: f32 = 1.8;
pub const SPRINT_PER_M: f32 = 0.004;
pub const SPRINT_MIN: f32 = 0.05;
/// How far a character sees in a site, metres (Laz); walls and closed doors
/// block it.
pub const SIGHT: f32 = 40.0;
/// Where sites stand: a pocket east of the map, one every `POCKET_STEP`
/// metres, with their datum `POCKET_H` up (clear of any sea drawn there).
pub const POCKET_X: f32 = WORLD_SIZE + 5000.0;
pub const POCKET_Y: f32 = 5000.0;
pub const POCKET_STEP: f32 = 1000.0;
pub const POCKET_H: f32 = 300.0;

// ---- The file, as the generator writes it ----------------------------------------

#[derive(Deserialize)]
struct FileResult {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    contract_failure: Option<serde_json::Value>,
    #[serde(default)]
    geometry_error: Option<serde_json::Value>,
    #[serde(default)]
    unmet: Vec<serde_json::Value>,
    #[serde(default)]
    placed: Vec<serde_json::Value>,
    dungeon: FileDungeon,
}

#[derive(Deserialize)]
struct FileDungeon {
    width: usize,
    height: usize,
    #[serde(default)]
    title: String,
    #[serde(default)]
    culture: String,
    #[serde(default)]
    site_kind: String,
    #[serde(default)]
    relic: String,
    tiles: Vec<FileTile>,
    objects: Vec<FileObject>,
    #[serde(default)]
    rooms: Vec<FileRoom>,
    spawn: [f32; 2],
    #[serde(default)]
    clearance: Option<FileClearance>,
    #[serde(default)]
    archways: Vec<FileArch>,
}

#[derive(Deserialize)]
struct FileTile {
    terrain: String,
    /// Natural rock (a cavern) rather than built (a hall).
    #[serde(default)]
    natural: bool,
    floor: f32,
    #[serde(default)]
    water_surface: Option<f32>,
    #[serde(default)]
    deck: Option<f32>,
    #[serde(default)]
    low_arch: bool,
}

#[derive(Deserialize)]
struct FileObject {
    kind: String,
    #[serde(default)]
    door_style: Option<String>,
    #[serde(default)]
    door_axis: Option<[i32; 2]>,
    x: f32,
    y: f32,
    z: f32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    open: bool,
    #[serde(default)]
    reward: Option<String>,
}

#[derive(Deserialize)]
struct FileRoom {
    name: String,
    #[serde(default)]
    original_purpose: String,
    center: [i32; 2],
}

#[derive(Deserialize)]
struct FileClearance {
    work_marker: usize,
    from: [i32; 2],
    to: [i32; 2],
    blocked_tiles: Vec<[i32; 2]>,
    restored: bool,
    #[serde(default)]
    source_event: String,
}

#[derive(Deserialize)]
struct FileArch {
    center: [f32; 2],
    axis: [i32; 2],
    span: i32,
    z: f32,
}

// ---- The site, as the game keeps it ----------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    Wall,
    Floor,
    Water,
    Pit,
    Rubble,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ground: Ground,
    pub floor: f32,
    pub water: Option<f32>,
    pub deck: Option<f32>,
    pub low_arch: bool,
    /// Natural rock (a cavern), not built (a hall): drawn in its own colours.
    #[serde(default)]
    pub natural: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjKind {
    Door,
    Plaque,
    Chest,
    Ladder,
    Memorial,
    Exit,
    /// A kind this reader doesn't know: kept, drawn as a marker.
    Other,
}

/// One of the site's things, under its index in the file (`index`), which
/// the generator's requirements point at.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Obj {
    pub index: u16,
    pub kind: ObjKind,
    /// The door style's name as written ("Timber", "CeremonialIron"…).
    pub style: String,
    /// Along the doorway (the generator's `door_axis`: across the opening).
    pub axis: [i8; 2],
    /// Doorway cells (ceremonial doors are two).
    pub cells: u8,
    /// Clear opening, metres wide and high (the style's).
    pub opening: [f32; 2],
    /// Site metres, and height in the site's datum.
    pub at: V2,
    pub z: f32,
    pub name: String,
    pub text: String,
    pub open: bool,
    pub reward: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Room {
    pub name: String,
    pub purpose: String,
    pub centre: [i32; 2],
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Clearance {
    pub marker: u16,
    pub from: [i32; 2],
    pub to: [i32; 2],
    pub blocked: Vec<[i32; 2]>,
    pub restored: bool,
    pub source_event: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Arch {
    pub centre: V2,
    pub axis: [i8; 2],
    pub span: u8,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Site {
    pub id: u32,
    /// The file it was read from, and its export version.
    pub source: String,
    pub schema: u32,
    /// The generator's own verdict, kept as given: "complete", "partial"…,
    /// and its contract failure, geometry error and unmet requirements.
    pub status: String,
    pub contract_failure: Option<String>,
    pub geometry_error: Option<String>,
    pub unmet: u32,
    pub placed: u32,
    pub title: String,
    pub culture: String,
    pub kind: String,
    pub relic: String,
    pub w: u16,
    pub h: u16,
    pub cells: Vec<Cell>,
    pub objects: Vec<Obj>,
    pub rooms: Vec<Room>,
    pub arches: Vec<Arch>,
    pub clearance: Option<Clearance>,
    /// Where the generator puts arrivals, and its height.
    pub spawn: V2,
    pub spawn_h: f32,
    /// World metres of the site's (0, 0), and world height of its datum.
    pub origin: V2,
    pub base_h: f32,
    /// Where on the overworld its portal stands, and where those who leave
    /// come out.
    pub portal: V2,
    /// What the import couldn't represent, one line each (shown, not hidden).
    pub notes: Vec<String>,
    /// Cells the squad has seen (by index), kept.
    #[serde(default)]
    pub seen: Vec<bool>,
}

/// Which level of a cell: its floor (or the water's surface), or its deck.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lv {
    Low,
    Deck,
}

/// A place to be in a site: a cell and a level.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node {
    pub x: u16,
    pub y: u16,
    pub lv: Lv,
}

/// How a node is crossed.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    Walk,
    Swim,
    Climb,
}

/// The way through a site: the nodes in turn after the start.
#[derive(Clone, Debug, PartialEq)]
pub struct Way {
    pub nodes: Vec<Node>,
    /// Seconds at a walker's full pace (walking 1 m/s), climbs and doors included.
    pub cost: f32,
}

#[derive(Debug)]
pub enum ImportError {
    Read(String),
    Parse(String),
    Schema(u32),
    Shape(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImportError::Read(e) => write!(f, "couldn't read the file: {e}"),
            ImportError::Parse(e) => write!(f, "not a LayoutResult: {e}"),
            ImportError::Schema(v) => write!(f, "export version {v} isn't one this reads ({:?})", SCHEMAS),
            ImportError::Shape(e) => write!(f, "the site doesn't hold together: {e}"),
        }
    }
}

fn door_opening(style: &str) -> [f32; 2] {
    match style {
        "Service" => [0.74, 1.95],
        "Ironbound" => [0.90, 2.60],
        "Ceremonial" | "CeremonialIron" => [1.90, 3.40],
        _ => [0.90, 2.20],
    }
}

fn short(v: &Option<serde_json::Value>) -> Option<String> {
    v.as_ref().filter(|v| !v.is_null()).map(|v| v.to_string())
}

impl Site {
    /// Read an exported `LayoutResult` (`path`): the site as given, standing
    /// at the origin until placed.
    pub fn load(path: &str) -> Result<Site, ImportError> {
        let text = std::fs::read_to_string(path).map_err(|e| ImportError::Read(e.to_string()))?;
        Site::from_json(&text, path)
    }

    pub fn from_json(text: &str, source: &str) -> Result<Site, ImportError> {
        let f: FileResult = serde_json::from_str(text).map_err(|e| ImportError::Parse(e.to_string()))?;
        let schema = f.schema_version.unwrap_or(0);
        if !SCHEMAS.contains(&schema) {
            return Err(ImportError::Schema(schema));
        }
        let d = f.dungeon;
        if d.width == 0 || d.height == 0 || d.width > u16::MAX as usize || d.height > u16::MAX as usize || d.tiles.len() != d.width * d.height {
            return Err(ImportError::Shape(format!("{} × {} cells but {} tiles", d.width, d.height, d.tiles.len())));
        }
        let mut notes = Vec::new();
        let cells: Vec<Cell> = d
            .tiles
            .iter()
            .map(|t| Cell {
                ground: match t.terrain.as_str() {
                    "Wall" => Ground::Wall,
                    "Floor" => Ground::Floor,
                    "Water" => Ground::Water,
                    "Pit" => Ground::Pit,
                    "Rubble" => Ground::Rubble,
                    _ => Ground::Wall,
                },
                floor: t.floor,
                water: t.water_surface,
                deck: t.deck,
                low_arch: t.low_arch,
                natural: t.natural,
            })
            .collect();
        let unknown = d.tiles.iter().filter(|t| !matches!(t.terrain.as_str(), "Wall" | "Floor" | "Water" | "Pit" | "Rubble")).count();
        if unknown > 0 {
            notes.push(format!("{unknown} cells of an unknown terrain, taken as wall"));
        }
        let arched = cells.iter().filter(|c| c.low_arch).count();
        if arched > 0 {
            notes.push(format!("{arched} low-arch cells: diving under them isn't supported (they're swum at the surface)"));
        }
        let mut objects = Vec::new();
        for (i, o) in d.objects.iter().enumerate() {
            let kind = match o.kind.as_str() {
                "Door" => ObjKind::Door,
                "Plaque" => ObjKind::Plaque,
                "Chest" => ObjKind::Chest,
                "Ladder" => ObjKind::Ladder,
                "Memorial" => ObjKind::Memorial,
                "Exit" => ObjKind::Exit,
                other => {
                    notes.push(format!("object {i}: kind {other:?} isn't known here; kept as a marker"));
                    ObjKind::Other
                }
            };
            let style = o.door_style.clone().unwrap_or_else(|| "Timber".into());
            let cells_n = if kind == ObjKind::Door && style.starts_with("Ceremonial") { 2 } else { 1 };
            // The generator's default axis: across the doorway runs x when
            // the cell to the west is wall.
            let axis = o.door_axis.unwrap_or_else(|| {
                let (cx, cy) = (o.x as i32 - 1, o.y as i32);
                let wall_west = cx >= 0 && (cx as usize) < d.width && cy >= 0 && (cy as usize) < d.height && d.tiles[cy as usize * d.width + cx as usize].terrain == "Wall";
                if wall_west {
                    [1, 0]
                } else {
                    [0, 1]
                }
            });
            objects.push(Obj {
                index: i as u16,
                kind,
                opening: door_opening(&style),
                style,
                axis: [axis[0].signum() as i8, axis[1].signum() as i8],
                cells: cells_n,
                at: V2::new(o.x, o.y),
                z: o.z,
                name: o.name.clone(),
                text: o.text.clone(),
                open: o.open,
                reward: o.reward.clone(),
            });
        }
        let mut site = Site {
            id: 0,
            source: source.to_string(),
            schema,
            status: f.status,
            contract_failure: short(&f.contract_failure),
            geometry_error: short(&f.geometry_error),
            unmet: f.unmet.len() as u32,
            placed: f.placed.len() as u32,
            title: d.title,
            culture: d.culture,
            kind: d.site_kind,
            relic: d.relic,
            w: d.width as u16,
            h: d.height as u16,
            cells,
            objects,
            rooms: d.rooms.iter().map(|r| Room { name: r.name.clone(), purpose: r.original_purpose.clone(), centre: r.center }).collect(),
            arches: d.archways.iter().map(|a| Arch { centre: V2::new(a.center[0], a.center[1]), axis: [a.axis[0].signum() as i8, a.axis[1].signum() as i8], span: a.span.clamp(0, 255) as u8, z: a.z }).collect(),
            clearance: d.clearance.as_ref().map(|c| Clearance { marker: c.work_marker as u16, from: c.from, to: c.to, blocked: c.blocked_tiles.clone(), restored: c.restored, source_event: c.source_event.clone() }),
            spawn: V2::new(d.spawn[0], d.spawn[1]),
            spawn_h: 0.0,
            origin: V2::default(),
            base_h: 0.0,
            portal: V2::default(),
            notes,
            seen: Vec::new(),
        };
        site.seen = vec![false; site.cells.len()];
        // Arrivals stand on the spawn cell's deck if it has one, else its floor.
        let Some(sc) = site.cell_at(site.spawn) else {
            return Err(ImportError::Shape(format!("spawn ({}, {}) is outside the site", site.spawn.x, site.spawn.y)));
        };
        site.spawn_h = sc.deck.unwrap_or(sc.floor);
        // A ladder needs both its ends: a missing deck is an import error,
        // not something to guess (the generator's ruling).
        for o in site.objects.iter().filter(|o| o.kind == ObjKind::Ladder) {
            match site.cell_at(o.at) {
                Some(c) if c.deck.is_some() => {}
                _ => return Err(ImportError::Shape(format!("ladder {} at ({}, {}) has no deck in its cell", o.index, o.at.x, o.at.y))),
            }
        }
        // Openings our walker (0.6 m wide) doesn't fit, said rather than assumed.
        for o in site.objects.iter().filter(|o| o.kind == ObjKind::Door) {
            if o.opening[0] < 2.0 * super::solid::BODY {
                site.notes.push(format!("door {} ({}) is {:.2} m clear: narrower than the engine's walker", o.index, o.style, o.opening[0]));
            }
        }
        Ok(site)
    }

    // ---- Cells --------------------------------------------------------------------

    pub fn cell(&self, x: i32, y: i32) -> Option<&Cell> {
        (x >= 0 && y >= 0 && (x as usize) < self.w as usize && (y as usize) < self.h as usize).then(|| &self.cells[y as usize * self.w as usize + x as usize])
    }

    /// The cell a point of the site (in its metres) is in.
    pub fn cell_at(&self, p: V2) -> Option<&Cell> {
        self.cell(p.x.floor() as i32, p.y.floor() as i32)
    }

    pub fn centre(n: Node) -> V2 {
        V2::new(n.x as f32 + 0.5, n.y as f32 + 0.5)
    }

    /// Site metres ↔ world metres.
    pub fn to_world(&self, p: V2) -> V2 {
        self.origin.add(p)
    }
    pub fn to_local(&self, p: V2) -> V2 {
        p.sub(self.origin)
    }
    pub fn contains_world(&self, p: V2) -> bool {
        let l = self.to_local(p);
        l.x >= 0.0 && l.y >= 0.0 && l.x < self.w as f32 && l.y < self.h as f32
    }

    /// Does a closed door fill this cell (the generator's `covers_cell`)?
    pub fn closed_door_in(&self, x: i32, y: i32) -> Option<&Obj> {
        self.objects.iter().find(|o| o.kind == ObjKind::Door && !o.open && door_covers(o, x, y))
    }

    pub fn ladder_in(&self, x: i32, y: i32) -> Option<&Obj> {
        self.objects.iter().find(|o| o.kind == ObjKind::Ladder && o.at.x.floor() as i32 == x && o.at.y.floor() as i32 == y)
    }

    /// The height one stands (or floats) at on a node, and how it's crossed;
    /// None if there's no such place (a wall; a pit's floor; no deck).
    pub fn level(&self, n: Node) -> Option<(f32, How)> {
        let c = self.cell(n.x as i32, n.y as i32)?;
        match n.lv {
            Lv::Deck => c.deck.map(|d| (d, How::Walk)),
            Lv::Low => match c.ground {
                Ground::Wall | Ground::Pit => None,
                _ => match c.water {
                    Some(s) if s > c.floor => Some((s, How::Swim)),
                    _ if c.ground == Ground::Water => None,
                    _ => Some((c.floor, How::Walk)),
                },
            },
        }
    }

    /// The nodes of a cell, lowest first.
    pub fn nodes_in(&self, x: i32, y: i32) -> Vec<Node> {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return Vec::new();
        }
        [Lv::Low, Lv::Deck].into_iter().map(|lv| Node { x: x as u16, y: y as u16, lv }).filter(|&n| self.level(n).is_some()).collect()
    }

    /// Can a walker go straight from `a` to its neighbour `b` (the step,
    /// not counting doors)?
    fn step_ok(&self, a: Node, b: Node) -> bool {
        let (Some((ha, ka)), Some((hb, kb))) = (self.level(a), self.level(b)) else { return false };
        match (ka, kb) {
            (How::Swim, How::Walk) => hb - ha <= STEP && ha - hb <= STEP,
            (How::Walk, How::Swim) => hb <= ha + STEP && ha - hb <= DROP_INTO_WATER,
            _ => (hb - ha).abs() <= STEP,
        }
    }

    /// The neighbours of a node a walker can go to, with what each costs
    /// (seconds at walking pace 1 m/s).
    pub fn neighbours(&self, a: Node) -> Vec<(Node, f32)> {
        let mut out = Vec::new();
        let Some((ha, ka)) = self.level(a) else { return out };
        let (x, y) = (a.x as i32, a.y as i32);
        // Up or down a ladder in this cell.
        if self.ladder_in(x, y).is_some() {
            let other = Node { lv: if a.lv == Lv::Low { Lv::Deck } else { Lv::Low }, ..a };
            if let Some((hb, _)) = self.level(other) {
                out.push((other, (hb - ha).abs() * CLIMB_SECS_PER_M));
            }
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            for b in self.nodes_in(nx, ny) {
                if !self.step_ok(a, b) {
                    continue;
                }
                // Diagonally only where both cells beside the corner can be
                // stepped through too (no cutting a wall's corner).
                if dx != 0 && dy != 0 {
                    let side = |sx: i32, sy: i32| self.nodes_in(sx, sy).into_iter().any(|m| self.step_ok(a, m) && self.step_ok(m, b) && self.closed_door_in(sx, sy).is_none());
                    if !(side(x + dx, y) && side(x, y + dy)) {
                        continue;
                    }
                    // Nor through a doorway at a slant.
                    if self.closed_door_in(nx, ny).is_some() || self.is_doorway(nx, ny) || self.is_doorway(x, y) {
                        continue;
                    }
                }
                let (_, kb) = self.level(b).unwrap();
                let pace = if ka == How::Swim || kb == How::Swim { SWIM_PACE } else { 1.0 };
                let len = if dx != 0 && dy != 0 { std::f32::consts::SQRT_2 } else { 1.0 };
                let door = if self.closed_door_in(nx, ny).is_some() { DOOR_SECS } else { 0.0 };
                out.push((b, len / pace + door));
            }
        }
        out
    }

    fn is_doorway(&self, x: i32, y: i32) -> bool {
        self.objects.iter().any(|o| o.kind == ObjKind::Door && door_covers(o, x, y))
    }

    /// The cheapest way from `from` to `to` (A*), or None if none leads there.
    pub fn route(&self, from: Node, to: Node) -> Option<Way> {
        if self.level(from).is_none() || self.level(to).is_none() {
            return None;
        }
        let guess = |n: Node| {
            let (dx, dy) = ((n.x as f32 - to.x as f32).abs(), (n.y as f32 - to.y as f32).abs());
            dx.max(dy) + (std::f32::consts::SQRT_2 - 1.0) * dx.min(dy)
        };
        let mut best: BTreeMap<Node, f32> = BTreeMap::new();
        let mut came: BTreeMap<Node, Node> = BTreeMap::new();
        let mut open = BinaryHeap::new();
        best.insert(from, 0.0);
        open.push(Open { f: guess(from), g: 0.0, n: from });
        while let Some(Open { g, n, .. }) = open.pop() {
            if n == to {
                let mut nodes = vec![to];
                let mut at = to;
                while let Some(&p) = came.get(&at) {
                    if p == from {
                        break;
                    }
                    nodes.push(p);
                    at = p;
                }
                nodes.reverse();
                if from == to {
                    nodes.clear();
                }
                return Some(Way { nodes, cost: g });
            }
            if g > *best.get(&n).unwrap_or(&f32::INFINITY) {
                continue;
            }
            for (m, c) in self.neighbours(n) {
                let ng = g + c;
                if ng < *best.get(&m).unwrap_or(&f32::INFINITY) {
                    best.insert(m, ng);
                    came.insert(m, n);
                    open.push(Open { f: ng + guess(m), g: ng, n: m });
                }
            }
        }
        None
    }

    /// Every node a walker at `from` can reach, with its cost.
    pub fn reach(&self, from: Node) -> BTreeMap<Node, f32> {
        let mut best: BTreeMap<Node, f32> = BTreeMap::new();
        if self.level(from).is_none() {
            return best;
        }
        let mut open = BinaryHeap::new();
        best.insert(from, 0.0);
        open.push(Open { f: 0.0, g: 0.0, n: from });
        while let Some(Open { g, n, .. }) = open.pop() {
            if g > *best.get(&n).unwrap_or(&f32::INFINITY) {
                continue;
            }
            for (m, c) in self.neighbours(n) {
                let ng = g + c;
                if ng < *best.get(&m).unwrap_or(&f32::INFINITY) {
                    best.insert(m, ng);
                    open.push(Open { f: ng, g: ng, n: m });
                }
            }
        }
        best
    }

    /// The node for a spot in the site: its own cell's level nearest `h`, or
    /// the nearest cell with somewhere to stand.
    pub fn node_near(&self, p: V2, h: f32) -> Option<Node> {
        let (cx, cy) = (p.x.floor() as i32, p.y.floor() as i32);
        for r in 0..12i32 {
            let mut best: Option<(f32, Node)> = None;
            for y in cy - r..=cy + r {
                for x in cx - r..=cx + r {
                    if (x - cx).abs().max((y - cy).abs()) != r {
                        continue;
                    }
                    for n in self.nodes_in(x, y) {
                        let (nh, _) = self.level(n).unwrap();
                        let d = Site::centre(n).dist(p) + (nh - h).abs() * 2.0;
                        if best.map(|b| d < b.0).unwrap_or(true) {
                            best = Some((d, n));
                        }
                    }
                }
            }
            if let Some((_, n)) = best {
                return Some(n);
            }
        }
        None
    }

    /// Where a walker stands to use object `i` (by its index in the file):
    /// the cheapest node of `reach` (what a walker can reach, `Site::reach`) within reach of it (`USE_REACH` across,
    /// `USE_RISE` up or down). A door, a ladder or an exit is used from its
    /// own cell.
    pub fn approach(&self, i: u16, reach: &BTreeMap<Node, f32>) -> Option<(Node, f32)> {
        let o = self.objects.iter().find(|o| o.index == i)?;
        let mut best: Option<(Node, f32)> = None;
        for (&n, &c) in reach {
            let (h, _) = self.level(n).unwrap();
            let close = Site::centre(n).dist(o.at) <= USE_REACH && (h - o.z).abs() <= USE_RISE;
            if close && best.map(|b| c < b.1).unwrap_or(true) {
                best = Some((n, c));
            }
        }
        best
    }

    /// The node arrivals stand on.
    pub fn spawn_node(&self) -> Option<Node> {
        let (x, y) = (self.spawn.x.floor() as i32, self.spawn.y.floor() as i32);
        let c = self.cell(x, y)?;
        let lv = if c.deck.is_some() { Lv::Deck } else { Lv::Low };
        let n = Node { x: x as u16, y: y as u16, lv };
        self.level(n).map(|_| n)
    }

    /// Where a body (`solid::BODY` each side) at `p` would stand, coming
    /// from node `from`: the node of `p`'s cell it can step to (`from`
    /// itself in its own cell), so long as every cell the body reaches into
    /// can be stepped to as well. None: it can't be there.
    pub fn stand_at(&self, from: Node, p: V2) -> Option<Node> {
        let (cx, cy) = (p.x.floor() as i32, p.y.floor() as i32);
        let here = if cx == from.x as i32 && cy == from.y as i32 {
            from
        } else {
            if (cx - from.x as i32).abs() > 1 || (cy - from.y as i32).abs() > 1 {
                return None;
            }
            let mut ok: Vec<Node> = self.nodes_in(cx, cy).into_iter().filter(|&n| self.step_ok(from, n)).collect();
            // On a deck or a floor, the same kind of level first.
            ok.sort_by_key(|n| (n.lv != from.lv) as u8);
            *ok.first()?
        };
        let r = super::solid::BODY * 0.9;
        for (ox, oy) in [(r, 0.0), (-r, 0.0), (0.0, r), (0.0, -r), (r * 0.7, r * 0.7), (-r * 0.7, r * 0.7), (r * 0.7, -r * 0.7), (-r * 0.7, -r * 0.7)] {
            let (qx, qy) = ((p.x + ox).floor() as i32, (p.y + oy).floor() as i32);
            if (qx, qy) == (cx, cy) || (qx, qy) == (from.x as i32, from.y as i32) {
                continue;
            }
            if !self.nodes_in(qx, qy).into_iter().any(|m| self.step_ok(here, m)) {
                return None;
            }
        }
        Some(here)
    }

    /// What a body in the air at `p` with its feet at `z` would land on in
    /// that cell: the highest level at or below its feet (`was`: their
    /// height a moment ago). None: nothing (a pit's open floor).
    pub fn landing(&self, p: V2, was: f32) -> Option<(Node, f32)> {
        let (cx, cy) = (p.x.floor() as i32, p.y.floor() as i32);
        self.nodes_in(cx, cy).into_iter().filter_map(|n| self.level(n).map(|l| (n, l.0))).filter(|&(_, h)| h <= was + 0.05).max_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Can a body in the air with its feet at `z` be at `p`: no wall or
    /// closed door in reach, and nothing there standing higher than a step
    /// above its feet?
    pub fn air_ok(&self, p: V2, z: f32) -> bool {
        let r = super::solid::BODY * 0.9;
        for (ox, oy) in [(0.0, 0.0), (r, 0.0), (-r, 0.0), (0.0, r), (0.0, -r), (r * 0.7, r * 0.7), (-r * 0.7, r * 0.7), (r * 0.7, -r * 0.7), (-r * 0.7, -r * 0.7)] {
            let (x, y) = ((p.x + ox).floor() as i32, (p.y + oy).floor() as i32);
            let Some(c) = self.cell(x, y) else { return false };
            if c.ground == Ground::Wall || self.closed_door_in(x, y).is_some() {
                return false;
            }
            let lowest = self.nodes_in(x, y).into_iter().filter_map(|n| self.level(n)).map(|l| l.0).fold(f32::INFINITY, f32::min);
            if lowest.is_finite() && lowest > z + STEP {
                return false;
            }
        }
        true
    }

    /// The lowest floor in the site (a fall below it is caught).
    pub fn lowest(&self) -> f32 {
        self.cells.iter().map(|c| c.floor).fold(0.0, f32::min)
    }

    /// Does this cell stop sight: a wall, or a closed door's doorway?
    fn opaque(&self, x: i32, y: i32) -> bool {
        match self.cell(x, y) {
            None => true,
            Some(c) => c.ground == Ground::Wall || self.closed_door_in(x, y).is_some(),
        }
    }

    /// The cells seen from `from` (site metres) out to `range`: rays every
    /// quarter of a degree, each stopping at the first wall or closed door
    /// (which is itself seen). Heights hide nothing (a ledge or a deck is
    /// seen over).
    pub fn sight(&self, from: V2, range: f32) -> Vec<bool> {
        let mut seen = vec![false; self.cells.len()];
        let w = self.w as usize;
        let h = self.h as usize;
        let mark = |seen: &mut Vec<bool>, x: i32, y: i32| {
            if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                seen[y as usize * w + x as usize] = true;
            }
        };
        mark(&mut seen, from.x.floor() as i32, from.y.floor() as i32);
        const RAYS: usize = 1440;
        const STEP: f32 = 0.2;
        for k in 0..RAYS {
            let a = k as f32 / RAYS as f32 * std::f32::consts::TAU;
            let (dx, dy) = (a.cos(), a.sin());
            let mut t = STEP;
            while t <= range {
                let (x, y) = ((from.x + dx * t).floor() as i32, (from.y + dy * t).floor() as i32);
                mark(&mut seen, x, y);
                if self.opaque(x, y) {
                    break;
                }
                t += STEP;
            }
        }
        seen
    }

    /// Fixture 5's clearance (the generator's rule): exactly the listed
    /// debris becomes floor at 0; nothing else changes.
    pub fn clear_route(&mut self) {
        let Some(c) = self.clearance.clone() else { return };
        for [x, y] in c.blocked {
            if x >= 0 && y >= 0 && (x as usize) < self.w as usize && (y as usize) < self.h as usize {
                let i = y as usize * self.w as usize + x as usize;
                self.cells[i].ground = Ground::Floor;
                self.cells[i].floor = 0.0;
            }
        }
        if let Some(c) = self.clearance.as_mut() {
            c.restored = true;
        }
    }
}

/// The generator's `covers_cell` for a door: the cells its doorway fills.
fn door_covers(o: &Obj, x: i32, y: i32) -> bool {
    let (ax, ay) = (o.axis[0] as f32, o.axis[1] as f32);
    let (dx, dy) = (x as f32 + 0.5 - o.at.x, y as f32 + 0.5 - o.at.y);
    (dx * ax + dy * ay).abs() < o.cells as f32 / 2.0 && (-dx * ay + dy * ax).abs() < 0.51
}

#[derive(PartialEq)]
struct Open {
    f: f32,
    g: f32,
    n: Node,
}
impl Eq for Open {}
impl Ord for Open {
    fn cmp(&self, o: &Self) -> Ordering {
        // Least first; ties by node, so the way found never depends on the heap.
        o.f.total_cmp(&self.f).then_with(|| o.n.cmp(&self.n))
    }
}
impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

// ---- Who is in a site -------------------------------------------------------------

/// A squad member in a site: where, and the way they're on.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Inside {
    pub site: u32,
    /// The node they're on (or leaving), and their height now.
    pub node: Node,
    pub h: f32,
    /// The nodes still to go.
    pub way: Vec<Node>,
    /// Up or down a ladder: from height, to height, how far (0 to 1).
    pub climbing: Option<(f32, f32, f32)>,
    /// Swimming now.
    pub swimming: bool,
    /// The thing (by its index in the file) they're on their way to use.
    #[serde(default)]
    pub using: Option<u16>,
    /// Steered by the keys (WASD): the way pressed, in site metres. Not
    /// saved (it's the keys held this moment).
    #[serde(skip)]
    pub steer: Option<V2>,
    /// Sprinting while steered (Shift held). Not saved, like the keys.
    #[serde(skip)]
    pub sprint: bool,
    /// In the air (a jump, or a fall): rising speed now, metres a second.
    #[serde(default)]
    pub air: Option<f32>,
    /// Dodging: which way, and how far still to go.
    #[serde(default)]
    pub dodge: Option<(V2, f32)>,
    /// The last place they stood on dry floor (a fall into a pit puts them
    /// back there, as the generator does).
    #[serde(default)]
    pub safe: Option<Node>,
    /// When a follower last found their way after the one controlled.
    #[serde(default)]
    pub planned: f64,
}

/// Every site and who's in one (saved).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Sites {
    pub sites: Vec<Site>,
    pub inside: BTreeMap<PersonId, Inside>,
    /// The squad member the player controls in a site (third person,
    /// `docs/third-person.md`); the others in the same site follow them.
    #[serde(default)]
    pub lead: Option<PersonId>,
    /// What the squad sees in each site now (by site: the cells looked from,
    /// and the cells seen). Worked out again when someone moves a cell; not
    /// saved.
    #[serde(skip)]
    pub sight: BTreeMap<u32, (Vec<(u16, u16)>, Vec<bool>)>,
}

impl Sites {
    /// The cells of a site in the squad's sight now (None: not worked out yet).
    pub fn sees(&self, site: u32) -> Option<&Vec<bool>> {
        self.sight.get(&site).map(|s| &s.1)
    }

    pub fn get(&self, id: u32) -> Option<&Site> {
        self.sites.iter().find(|s| s.id == id)
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Site> {
        self.sites.iter_mut().find(|s| s.id == id)
    }
}

impl World {
    /// Read a site's file and stand it in its pocket, with its portal at
    /// `portal` on the overworld. Its id.
    pub fn import_site(&mut self, path: &str, portal: V2) -> Result<u32, ImportError> {
        Ok(self.add_site(Site::load(path)?, portal))
    }

    /// Stand a site already read in its pocket, its portal at `portal`. Its id.
    pub fn add_site(&mut self, mut site: Site, portal: V2) -> u32 {
        let id = self.sites.sites.iter().map(|s| s.id + 1).max().unwrap_or(0);
        site.id = id;
        site.origin = V2::new(POCKET_X + id as f32 * POCKET_STEP, POCKET_Y);
        site.base_h = POCKET_H;
        site.portal = portal;
        self.sites.sites.push(site);
        id
    }

    /// The site a squad member is in, if any.
    pub fn site_of(&self, pid: PersonId) -> Option<&Inside> {
        self.sites.inside.get(&pid)
    }

    /// The world height a squad member in a site stands at.
    pub fn site_height(&self, pid: PersonId) -> Option<f32> {
        let i = self.sites.inside.get(&pid)?;
        let s = self.sites.get(i.site)?;
        Some(s.base_h + i.h)
    }

    /// A squad member goes through the portal: they stand at the spawn.
    pub fn enter_site(&mut self, pid: PersonId, site: u32) -> bool {
        let Some(k) = self.squad.index(pid) else { return false };
        let Some(s) = self.sites.get(site) else { return false };
        let Some(n) = s.spawn_node() else { return false };
        let at = s.to_world(s.spawn);
        let h = s.level(n).map(|l| l.0).unwrap_or(s.spawn_h);
        self.squad.at[k] = at;
        self.squad.goal[k] = at;
        self.squad.route[k].clear();
        self.squad.inside[k] = None;
        self.sites.inside.insert(pid, Inside { site, node: n, h, way: Vec::new(), climbing: None, swimming: false, using: None, steer: None, sprint: false, air: None, dodge: None, safe: Some(n), planned: 0.0 });
        self.recentre_squad();
        self.update_sight(site);
        true
    }

    /// A squad member leaves a site: back out beside its portal.
    pub fn leave_site(&mut self, pid: PersonId) {
        let Some(i) = self.sites.inside.remove(&pid) else { return };
        let Some(k) = self.squad.index(pid) else { return };
        let out = self.sites.get(i.site).map(|s| s.portal).unwrap_or(self.squad.pos);
        let spot = self.free_spot(out.add(super::squad::formation(k + 1)));
        self.squad.at[k] = spot;
        self.squad.goal[k] = spot;
        self.squad.route[k].clear();
        self.recentre_squad();
    }

    /// Send a squad member in a site to a spot of it (world metres), at the
    /// level nearest `h` (world height; None: their own). False if there's
    /// no way there (they go nowhere).
    pub fn order_in_site(&mut self, pid: PersonId, target: V2, h: Option<f32>) -> bool {
        let Some(i) = self.sites.inside.get(&pid).cloned() else { return false };
        let Some(s) = self.sites.get(i.site) else { return false };
        let local = s.to_local(target);
        let want = h.map(|h| h - s.base_h).unwrap_or(i.h);
        let Some(to) = s.node_near(local, want) else { return false };
        // A walker mid-step finishes from the node they're on.
        let from = i.node;
        let Some(way) = s.route(from, to) else {
            let line = "There's no way through to there.".to_string();
            if self.log.front().map(|l| l.1 != line).unwrap_or(true) {
                self.log.push_front((self.time, line));
                self.log.truncate(14);
            }
            return false;
        };
        let points: Vec<V2> = way.nodes.iter().map(|&n| s.to_world(Site::centre(n))).collect();
        let goal = points.last().copied().unwrap_or(s.to_world(Site::centre(from)));
        if let Some(k) = self.squad.index(pid) {
            self.squad.goal[k] = goal;
            self.squad.route[k] = points;
        }
        if let Some(i) = self.sites.inside.get_mut(&pid) {
            i.way = way.nodes;
            i.using = None;
        }
        true
    }

    /// The thing of a site nearest a spot of it (world metres, world height),
    /// if one is within a click's slack.
    pub fn site_object_near(&self, site: u32, p: V2, h: f32) -> Option<u16> {
        let s = self.sites.get(site)?;
        let l = s.to_local(p);
        let hz = h - s.base_h;
        s.objects
            .iter()
            .filter(|o| o.at.dist(l) <= 1.2 && (o.z - hz).abs() <= 2.0)
            .min_by(|a, b| a.at.dist(l).total_cmp(&b.at.dist(l)))
            .map(|o| o.index)
    }

    /// Send a squad member in a site to use thing `i` (by its index in the
    /// file): to the nearest spot within reach of it, and then use it
    /// (`use_site_object`). False if no way leads there.
    pub fn order_use_in_site(&mut self, pid: PersonId, i: u16) -> bool {
        let Some(inside) = self.sites.inside.get(&pid).cloned() else { return false };
        let Some(s) = self.sites.get(inside.site) else { return false };
        let reach = s.reach(inside.node);
        // A ladder used is climbed: to whichever of its ends is further from
        // where they stand now.
        let ladder = s.objects.iter().find(|o| o.index == i && o.kind == ObjKind::Ladder).map(|o| (o.at.x.floor() as u16, o.at.y.floor() as u16));
        let far_end = ladder.and_then(|(x, y)| {
            let h = |n: &Node| (s.level(*n).map(|l| l.0).unwrap_or(0.0) - inside.h).abs();
            [Lv::Low, Lv::Deck].into_iter().map(|lv| Node { x, y, lv }).filter(|n| reach.contains_key(n)).max_by(|a, b| h(a).total_cmp(&h(b)))
        });
        let found = match far_end {
            Some(n) => Some((n, 0.0)),
            None => s.approach(i, &reach),
        };
        let Some((at, _)) = found else {
            let line = "There's no way through to there.".to_string();
            if self.log.front().map(|l| l.1 != line).unwrap_or(true) {
                self.log.push_front((self.time, line));
                self.log.truncate(14);
            }
            return false;
        };
        let way = s.route(inside.node, at).map(|w| w.nodes).unwrap_or_default();
        let points: Vec<V2> = way.iter().map(|&n| s.to_world(Site::centre(n))).collect();
        let goal = points.last().copied().unwrap_or(s.to_world(Site::centre(at)));
        if let Some(k) = self.squad.index(pid) {
            self.squad.goal[k] = goal;
            self.squad.route[k] = points;
        }
        let now = way.is_empty() && far_end.is_none();
        if let Some(m) = self.sites.inside.get_mut(&pid) {
            m.way = way;
            m.using = if far_end.is_some() { None } else { Some(i) };
            m.steer = None;
        }
        if now {
            self.use_site_object(pid);
        }
        true
    }

    /// A squad member in reach of the thing they set out to use uses it: the
    /// exit takes them out; a plaque is read; a chest opened (what it holds is
    /// named: what it gives isn't agreed with the generator yet); a door opened
    /// or shut.
    pub(super) fn use_site_object(&mut self, pid: PersonId) {
        let Some(inside) = self.sites.inside.get_mut(&pid) else { return };
        let Some(i) = inside.using.take() else { return };
        let site = inside.site;
        let Some(s) = self.sites.get_mut(site) else { return };
        // The work marker of a blocked route (fixture 5): the listed debris
        // is cleared, as in the generator.
        if s.clearance.as_ref().is_some_and(|c| c.marker == i && !c.restored) {
            s.clear_route();
            self.log.push_front((self.time, "The fallen stone is cleared from the dry bypass.".to_string()));
            self.log.truncate(14);
            return;
        }
        let Some(o) = s.objects.iter_mut().find(|o| o.index == i) else { return };
        let line = match o.kind {
            ObjKind::Exit => {
                let line = format!("Out of {}. ({})", s.kind.to_lowercase(), o.name);
                self.leave_site(pid);
                line
            }
            ObjKind::Chest => {
                if o.open {
                    format!("{} is empty.", o.name)
                } else {
                    o.open = true;
                    match &o.reward {
                        Some(r) => format!("Found: {r}. {}", o.text),
                        None => format!("{}: {}", o.name, o.text),
                    }
                }
            }
            ObjKind::Door => {
                o.open = !o.open;
                format!("{} {}.", o.name, if o.open { "opened" } else { "shut" })
            }
            _ => format!("{}: {}", o.name, o.text),
        };
        self.log.push_front((self.time, line));
        self.log.truncate(14);
    }

    /// What the squad in site `id` sees now, worked out again if anyone of
    /// them has moved a cell; what's seen is kept as seen.
    pub(super) fn update_sight(&mut self, id: u32) {
        let Some(s) = self.sites.get(id) else { return };
        let mut from: Vec<(u16, u16)> = Vec::new();
        for (&pid, i) in &self.sites.inside {
            if i.site != id {
                continue;
            }
            if let Some(k) = self.squad.index(pid) {
                let l = s.to_local(self.squad.at[k]);
                from.push((l.x.floor().max(0.0) as u16, l.y.floor().max(0.0) as u16));
            }
        }
        if self.sites.sight.get(&id).is_some_and(|c| c.0 == from) {
            return;
        }
        let mut now = vec![false; s.cells.len()];
        for f in &from {
            // From the middle of the cell, so one cell always sees the same.
            let eye = V2::new(f.0 as f32 + 0.5, f.1 as f32 + 0.5);
            for (a, b) in now.iter_mut().zip(s.sight(eye, SIGHT)) {
                *a |= b;
            }
        }
        if let Some(s) = self.sites.get_mut(id) {
            if s.seen.len() != now.len() {
                s.seen = vec![false; now.len()];
            }
            for (a, b) in s.seen.iter_mut().zip(&now) {
                *a |= *b;
            }
        }
        self.sites.sight.insert(id, (from, now));
    }

    /// Steer a squad member in a site by the keys: `dir` the way pressed
    /// (world metres, any length; None: let go). Any way they were walking or
    /// thing they were going to use is dropped.
    pub fn steer_in_site(&mut self, pid: PersonId, dir: Option<V2>) {
        let Some(i) = self.sites.inside.get_mut(&pid) else { return };
        let dir = dir.filter(|d| d.len() > 1e-4).map(|d| d.scale(1.0 / d.len()));
        if dir.is_some() && i.climbing.is_none() {
            i.way.clear();
            i.using = None;
            if let Some(k) = self.squad.index(pid) {
                self.squad.route[k].clear();
                self.squad.goal[k] = self.squad.at[k];
            }
        }
        i.steer = dir;
    }

    /// Sprint (Shift) while steered, or stop.
    pub fn sprint_in_site(&mut self, pid: PersonId, on: bool) {
        if let Some(i) = self.sites.inside.get_mut(&pid) {
            i.sprint = on;
        }
    }

    /// A squad member in a site jumps (on firm footing: not swimming,
    /// climbing or already in the air).
    pub fn jump_in_site(&mut self, pid: PersonId) {
        let Some(i) = self.sites.inside.get_mut(&pid) else { return };
        if i.air.is_none() && i.climbing.is_none() && !i.swimming {
            i.air = Some(JUMP);
            i.way.clear();
            i.using = None;
        }
    }

    /// A squad member in a site dodges back from `facing` (world metres, the
    /// way they face), for stamina.
    pub fn dodge_in_site(&mut self, pid: PersonId, facing: V2) {
        let Some(i) = self.sites.inside.get_mut(&pid) else { return };
        if i.air.is_some() || i.climbing.is_some() || i.swimming || i.dodge.is_some() || facing.len() < 1e-4 {
            return;
        }
        let back = facing.scale(-1.0 / facing.len());
        i.dodge = Some((back, DODGE_M));
        i.way.clear();
        i.using = None;
        let t = self.time;
        self.settle_condition(pid, t);
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.stamina = (c.stamina - c.max_stamina * DODGE_STAMINA).max(0.0);
        }
    }

    /// Is thing `i` of the site within a squad member's reach where they stand?
    pub fn site_in_reach(&self, pid: PersonId, i: u16) -> bool {
        let Some(m) = self.sites.inside.get(&pid) else { return false };
        let Some(s) = self.sites.get(m.site) else { return false };
        let Some(k) = self.squad.index(pid) else { return false };
        let Some(o) = s.objects.iter().find(|o| o.index == i) else { return false };
        let at = s.to_local(self.squad.at[k]);
        at.dist(o.at) <= USE_REACH + 0.3 && (o.z - m.h).abs() <= USE_RISE + if o.kind == ObjKind::Ladder { 2.6 } else { 0.0 }
    }

    /// What using thing `i` would do, in a word or two ("Open the timber
    /// door", "Climb", "Read", "Leave").
    pub fn site_use_label(&self, site: u32, i: u16) -> Option<String> {
        let s = self.sites.get(site)?;
        let o = s.objects.iter().find(|o| o.index == i)?;
        if s.clearance.as_ref().is_some_and(|c| c.marker == i && !c.restored) {
            return Some(format!("Clear the way ({})", o.name));
        }
        Some(match o.kind {
            ObjKind::Door => format!("{} {}", if o.open { "Shut" } else { "Open" }, o.name),
            ObjKind::Ladder => "Climb the ladder".to_string(),
            ObjKind::Chest => {
                if o.open {
                    format!("{} (empty)", o.name)
                } else {
                    format!("Open {}", o.name)
                }
            }
            ObjKind::Exit => format!("Leave: {}", o.name),
            _ => format!("Read: {}", o.name),
        })
    }

    /// Use thing `i` now, from where they stand (third person's E): a ladder
    /// is climbed, anything else used at once. False if it's out of reach.
    pub fn use_in_reach(&mut self, pid: PersonId, i: u16) -> bool {
        if !self.site_in_reach(pid, i) {
            return false;
        }
        let ladder = self.sites.inside.get(&pid).and_then(|m| self.sites.get(m.site)).is_some_and(|s| s.objects.iter().any(|o| o.index == i && o.kind == ObjKind::Ladder));
        if ladder {
            return self.order_use_in_site(pid, i);
        }
        if let Some(m) = self.sites.inside.get_mut(&pid) {
            m.using = Some(i);
            m.way.clear();
            m.steer = None;
        }
        self.use_site_object(pid);
        true
    }

    /// A squad mate in a site who isn't the one controlled keeps up with
    /// the one who is: once they're further off than their setting allows
    /// (`behaviour::Follow::gap`), a way to just behind them, found again
    /// now and then; or, holding a spot, back to it.
    fn follow_lead(&mut self, pid: PersonId) {
        let Some(lead) = self.sites.lead.filter(|&l| l != pid) else { return };
        let (Some(me), Some(them)) = (self.sites.inside.get(&pid).cloned(), self.sites.inside.get(&lead).cloned()) else { return };
        if me.site != them.site || me.steer.is_some() || me.climbing.is_some() || me.air.is_some() || me.using.is_some() || self.time - me.planned < FOLLOW_REPLAN {
            return;
        }
        let (Some(km), Some(kl)) = (self.squad.index(pid), self.squad.index(lead)) else { return };
        let b = self.behaviour(pid);
        let Some(s) = self.sites.get(me.site) else { return };
        // Holding a spot in this site: back to it once they're off it.
        let (target, gap_ok, stop) = match (b.follow, b.hold) {
            (super::behaviour::Follow::Hold, Some((site, spot, h))) if site == me.site => {
                let Some(n) = s.node_near(s.to_local(spot), h) else { return };
                let off = self.squad.at[km].dist(spot) + (me.h - h).abs();
                (n, off <= HOLD_SLACK || me.node == n, 0.0)
            }
            (super::behaviour::Follow::Hold, _) => return,
            (f, _) => {
                let gap = self.squad.at[km].dist(self.squad.at[kl]) + (me.h - them.h).abs();
                // Caught up, they stand off at about half their gap.
                (them.node, gap <= f.gap(), f.gap() * 0.5)
            }
        };
        if gap_ok {
            return;
        }
        let Some(mut way) = s.route(me.node, target).map(|w| w.nodes) else { return };
        // Following, stop that far short of them (by the way's own length).
        if stop > 0.0 {
            let end = Site::centre(target);
            while way.len() > 1 && Site::centre(way[way.len() - 2]).dist(end) <= stop {
                way.pop();
            }
            if way.last().is_some_and(|&n| Site::centre(n).dist(end) < stop.min(1.0)) {
                way.pop();
            }
        }
        let points: Vec<V2> = way.iter().map(|&n| s.to_world(Site::centre(n))).collect();
        if let Some(m) = self.sites.inside.get_mut(&pid) {
            m.way = way;
            m.planned = self.time;
        }
        if let Some(last) = points.last().copied() {
            self.squad.goal[km] = last;
            self.squad.route[km] = points;
        }
    }

    /// A squad member in a site moves along their way for `dt` seconds:
    /// walking, swimming, climbing; a closed door on the way opens as they
    /// reach it. (The squad's own stepping, like walking outside.)
    pub(super) fn walk_in_site(&mut self, k: usize, dt: f64) {
        let pid = self.squad.members[k];
        self.follow_lead(pid);
        let Some(mut i) = self.sites.inside.get(&pid).cloned() else { return };
        // Steered by hand they go at their own pace; sent, with the others.
        let free = i.steer.is_some() || i.air.is_some() || i.dodge.is_some();
        let speed = if free { self.member_speed(pid) } else { self.march_speed(k) }.max(0.1);
        let Some(site) = self.sites.get(i.site) else { return };
        let mut left = dt as f32;
        let mut at = site.to_local(self.squad.at[k]);
        let mut opened: Vec<u16> = Vec::new();
        let mut rise = 0.0f32;
        let mut sprinted = 0.0f32;
        let sprint_ok = self.stamina_of(pid).unwrap_or(1.0) > SPRINT_MIN;
        while left > 1e-6 {
            if let Some((from, to, done)) = i.climbing {
                let span = (to - from).abs().max(1e-3);
                let secs = span * CLIMB_SECS_PER_M / (speed / super::squad::SQUAD_SPEED).max(0.1);
                let step = (left / secs).min(1.0 - done);
                let now = done + step;
                left -= step * secs;
                i.h = from + (to - from) * now;
                if to > from {
                    rise += (to - from) * step;
                }
                if now >= 1.0 - 1e-6 {
                    i.climbing = None;
                    i.h = to;
                } else {
                    i.climbing = Some((from, to, now));
                }
                continue;
            }
            if i.steer.is_some() || i.air.is_some() || i.dodge.is_some() {
                // Moved by hand: the keys' way (or a dodge), and in the air
                // under gravity, in slices of at most 20 ms.
                let sub = left.min(0.02);
                left -= sub;
                let pace = if i.swimming { SWIM_PACE } else { 1.0 };
                let sprinting = i.sprint && i.steer.is_some() && !i.swimming && i.air.is_none() && sprint_ok;
                let run = if sprinting { SPRINT } else { 1.0 };
                let v = match i.dodge {
                    Some((d, _)) => d.scale(DODGE_SPEED),
                    None => i.steer.map(|d| d.scale(if i.air.is_some() { (speed * pace).max(JUMP_CARRY) } else { speed * pace * run })).unwrap_or_default(),
                };
                let tries = [v, V2::new(v.x, 0.0), V2::new(0.0, v.y)];
                if let Some(vz) = i.air {
                    let vz = vz - GRAVITY * sub;
                    let z = i.h + vz * sub;
                    for d in tries {
                        if d.len() < 1e-4 {
                            continue;
                        }
                        let to = at.add(d.scale(sub));
                        if site.air_ok(to, i.h.min(z)) {
                            at = to;
                            break;
                        }
                    }
                    match site.landing(at, i.h).filter(|&(_, lv)| vz <= 0.0 && z <= lv) {
                        Some((n, lv)) => {
                            i.node = n;
                            i.h = lv;
                            i.air = None;
                            i.swimming = site.level(n).is_some_and(|l| l.1 == How::Swim);
                        }
                        None => {
                            i.h = z;
                            i.air = Some(vz);
                            // A fall past the bottom: back to where they last stood.
                            if z < site.lowest() - 1.5 {
                                let back = i.safe.unwrap_or(i.node);
                                at = Site::centre(back);
                                i.node = back;
                                i.h = site.level(back).map(|l| l.0).unwrap_or(i.h);
                                i.air = None;
                                i.dodge = None;
                            }
                        }
                    }
                } else {
                    let mut moved = false;
                    for d in tries {
                        if d.len() < 1e-4 {
                            continue;
                        }
                        let to = at.add(d.scale(sub));
                        if let Some(n) = site.stand_at(i.node, to) {
                            if let Some(dr) = site.closed_door_in(n.x as i32, n.y as i32) {
                                if !opened.contains(&dr.index) {
                                    opened.push(dr.index);
                                }
                            }
                            let (nh, how) = site.level(n).unwrap();
                            if nh > i.h {
                                rise += nh - i.h;
                            }
                            if sprinting {
                                sprinted += to.dist(at);
                            }
                            at = to;
                            i.node = n;
                            i.h = nh;
                            i.swimming = how == How::Swim;
                            if how == How::Walk && n.lv == Lv::Low {
                                i.safe = Some(n);
                            }
                            moved = true;
                            break;
                        }
                    }
                    if !moved && i.dodge.is_none() && i.steer.is_some() && v.len() > 1e-4 {
                        // Pressed against something: stand still this slice.
                    }
                    if !moved {
                        i.dodge = None;
                    }
                }
                if let Some((d, rest)) = i.dodge {
                    let rest = rest - DODGE_SPEED * sub;
                    i.dodge = (rest > 0.0).then_some((d, rest));
                }
                // Nothing to do this slice and on the ground: done.
                if i.air.is_none() && i.dodge.is_none() && v.len() < 1e-4 {
                    break;
                }
                continue;
            }
            let Some(&next) = i.way.first() else { break };
            let Some((nh, how)) = site.level(next) else {
                i.way.clear();
                break;
            };
            if next.x == i.node.x && next.y == i.node.y {
                // The ladder in this cell.
                i.climbing = Some((i.h, nh, 0.0));
                i.node = next;
                i.way.remove(0);
                continue;
            }
            if let Some(d) = site.closed_door_in(next.x as i32, next.y as i32) {
                if !opened.contains(&d.index) {
                    opened.push(d.index);
                }
            }
            let target = Site::centre(next);
            let pace = if how == How::Swim || i.swimming { SWIM_PACE } else { 1.0 };
            let v = speed * pace;
            let d = at.dist(target);
            let can = v * left;
            if can >= d {
                left -= d / v;
                at = target;
                if nh > i.h {
                    rise += nh - i.h;
                }
                i.h = nh;
                i.node = next;
                i.swimming = how == How::Swim;
                i.way.remove(0);
            } else {
                let t = can / d.max(1e-6);
                at = at.add(target.sub(at).scale(t));
                // Heights change over the step (a bank, a stair).
                let (h0, _) = site.level(i.node).unwrap_or((i.h, How::Walk));
                let seg = Site::centre(i.node).dist(target).max(1e-3);
                i.h = h0 + (nh - h0) * (1.0 - target.dist(at) / seg).clamp(0.0, 1.0);
                left = 0.0;
            }
        }
        let world_at = site.to_world(at);
        let done = i.way.is_empty() && i.climbing.is_none() && i.steer.is_none() && i.air.is_none() && i.dodge.is_none();
        let site_id = i.site;
        self.sites.inside.insert(pid, i);
        if let Some(s) = self.sites.get_mut(site_id) {
            for o in s.objects.iter_mut().filter(|o| opened.contains(&o.index)) {
                o.open = true;
            }
        }
        self.squad.at[k] = world_at;
        if done {
            self.squad.goal[k] = world_at;
            self.squad.route[k].clear();
            if self.sites.inside.get(&pid).is_some_and(|i| i.using.is_some()) {
                self.use_site_object(pid);
            }
        } else if let Some(r) = self.squad.route[k].first().copied() {
            if r.dist(world_at) < 0.05 {
                self.squad.route[k].remove(0);
            }
        }
        if rise > 0.0 {
            self.climb(pid, rise);
        }
        if sprinted > 0.0 {
            // By the metre, like climbing.
            if let Some(c) = self.people[pid as usize].cond.as_mut() {
                c.stamina = (c.stamina - c.max_stamina * SPRINT_PER_M * sprinted).max(0.0);
            }
        }
        self.update_sight(site_id);
    }
}

// ---- Making a new one with the generator's own program ----------------------------

/// Where the generator's outputs live on Laz's PC (its `outputs` folder);
/// `GAHT_UNDERSTORY` points elsewhere.
pub const UNDERSTORY: &str = r"C:\Users\Laz PC\Documents\Codex\2026-10-28\files-mentioned-by-the-user-grinblat2017subverting\outputs";
/// Where the engine keeps what it has the generator make (its own folder;
/// nothing is written into the generator's).
pub const MADE: &str = "dungeons-made";

pub fn understory_root() -> std::path::PathBuf {
    std::env::var("GAHT_UNDERSTORY").map(Into::into).unwrap_or_else(|_| UNDERSTORY.into())
}

/// The generator's audited histories (its `story-to-map/batch/report.json`):
/// each story's seed and its verdict there ("complete": an accepted mission).
pub fn understory_stories() -> Result<Vec<(u64, String, String)>, String> {
    let path = understory_root().join("story-to-map").join("batch").join("report.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let rows = v["layouts"].as_array().ok_or("no layouts in the report")?;
    Ok(rows
        .iter()
        .filter_map(|r| Some((r["story"].as_u64()?, r["status"].as_str().unwrap_or("").to_string(), format!("{} ({})", r["site"].as_str().unwrap_or("?"), r["culture"].as_str().unwrap_or("?")))))
        .collect())
}

/// Have the generator make story `story`'s site at layout seed `layout`:
/// its history record is copied out of `batch-inputs.jsonl` into the engine's
/// own folder, and its checker (`understory-general-check.exe`, the same call
/// its "Play Random Story" launcher makes) writes the `LayoutResult` there.
/// The file made, whatever the generator's verdict (that's in the file).
pub fn make_with_understory(story: u64, layout: u32) -> Result<std::path::PathBuf, String> {
    let root = understory_root();
    let inputs = root.join("story-to-map").join("batch-inputs.jsonl");
    let text = std::fs::read_to_string(&inputs).map_err(|e| format!("{}: {e}", inputs.display()))?;
    let record = text
        .lines()
        .find(|l| serde_json::from_str::<serde_json::Value>(l).ok().and_then(|v| v["seed"].as_u64()) == Some(story))
        .ok_or_else(|| format!("story {story} isn't among the generator's histories"))?;
    let out = std::path::PathBuf::from(MADE).join(format!("story-{story}"));
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let rec = out.join("record.json");
    std::fs::write(&rec, record).map_err(|e| e.to_string())?;
    let exe = root.join("understory-general-check.exe");
    let run = std::process::Command::new(&exe)
        .arg("--file")
        .arg(&rec)
        .arg("--count")
        .arg((layout + 1).to_string())
        .arg("--out")
        .arg(&out)
        .output()
        .map_err(|e| format!("couldn't run {}: {e}", exe.display()))?;
    let made = out.join(format!("story-{story}-{layout}.json"));
    if made.exists() {
        Ok(made)
    } else {
        Err(format!("the generator made nothing: {}{}", String::from_utf8_lossy(&run.stderr), String::from_utf8_lossy(&run.stdout)).chars().take(400).collect())
    }
}
