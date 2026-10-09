//! Hand edits to the land: the map's authored layer.
//!
//! The land is made from the world seed; on top of that sit whatever edits
//! have been made by hand in the land editor (F10 in the window). They're
//! kept in fine layers, 5 m a cell, stored only where something was changed:
//!
//! - **height**: metres added to the land (raise, lower, smooth, flatten...);
//! - **paint**: a ground texture and how strongly it's laid on (grass, sand,
//!   rock, snow...). Painted ground counts as that ground for walking pace;
//! - **plants**: how much grass, brush and trees grow (less, none, or more
//!   than the land would have on its own). Drawing only;
//! - **rocks**: boulders, slabs, pillars, scree and outcrops placed by hand.
//!   Drawing only.
//!
//! Edits are part of the map, not of a game in progress: they're saved to a
//! map file (`maps/seed-N.gmap`) that a new game with that seed starts from,
//! and carried in every save of a game made from it. Towns are placed from
//! the land as the seed made it, so editing never moves a town; the roads are
//! found again over the edited land.

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

use super::geo::{V2, WORLD_SIZE};
use super::rng::{self, Rng};
use super::terrain::{Ground, Terrain};

// ---- Dials -----------------------------------------------------------------

/// Edit cell size, metres, and cells across a stored chunk.
pub const EDIT_CELL: f32 = 5.0;
pub const CHUNK: usize = 64;
const CHUNK_M: f32 = EDIT_CELL * CHUNK as f32;
const CHUNKS: usize = (WORLD_SIZE / CHUNK_M) as usize + 2;
/// How many strokes can be undone.
pub const UNDO_DEPTH: usize = 40;
/// Most a brush raises or lowers per second at full strength, metres.
pub const SCULPT_RATE: f32 = 12.0;
/// How strongly each plant kind can be added (chance per spot, on top of
/// what the land grows on its own).
pub const PLANT_ADD: [f32; 3] = [0.8, 0.35, 0.9];
/// Rocks placed per second per 100 m² of brush at full strength.
pub const ROCK_RATE: f32 = 0.6;

/// The paintable ground textures (index 1..; 0 is "not painted").
#[derive(Clone, Copy, Debug)]
pub struct Texture {
    pub name: &'static str,
    /// What it counts as underfoot.
    pub ground: Ground,
}

pub const TEXTURES: [Texture; 13] = [
    Texture { name: "Grass", ground: Ground::Grass },
    Texture { name: "Dry grass", ground: Ground::Grass },
    Texture { name: "Heath", ground: Ground::Scrub },
    Texture { name: "Scrub", ground: Ground::Scrub },
    Texture { name: "Dirt", ground: Ground::Dirt },
    Texture { name: "Mud", ground: Ground::Mud },
    Texture { name: "Sand", ground: Ground::Sand },
    Texture { name: "Shingle", ground: Ground::Sand },
    Texture { name: "Gravel", ground: Ground::Gravel },
    Texture { name: "Rock", ground: Ground::Rock },
    Texture { name: "Sandstone", ground: Ground::Rock },
    Texture { name: "Snow", ground: Ground::Snow },
    Texture { name: "Moss", ground: Ground::Grass },
];

/// The kinds of rock that can be placed.
pub const ROCKS: [&str; 5] = ["Boulder", "Slab", "Pillar", "Scree", "Outcrop"];
/// Plant kinds for the plant brushes.
pub const PLANTS: [&str; 3] = ["Grass", "Brush", "Trees"];

// ---- Layers ----------------------------------------------------------------

/// A world-sized grid of values at `EDIT_CELL`, stored only where changed.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer<T: Copy + PartialEq> {
    fill: T,
    chunks: Vec<Option<Box<[T]>>>,
    /// Whether any chunk holds something (a quick out for the common case).
    any: bool,
}

impl<T: Copy + PartialEq> Layer<T> {
    pub fn new(fill: T) -> Self {
        Layer { fill, chunks: vec![None; CHUNKS * CHUNKS], any: false }
    }

    fn recount(&mut self) {
        self.any = self.chunks.iter().any(|c| c.is_some());
    }

    fn slot(i: i64, j: i64) -> Option<(usize, usize)> {
        if i < 0 || j < 0 {
            return None;
        }
        let (ci, cj) = (i as usize / CHUNK, j as usize / CHUNK);
        if ci >= CHUNKS || cj >= CHUNKS {
            return None;
        }
        Some((cj * CHUNKS + ci, (j as usize % CHUNK) * CHUNK + i as usize % CHUNK))
    }

    /// The value at cell (i, j).
    pub fn get(&self, i: i64, j: i64) -> T {
        if !self.any {
            return self.fill;
        }
        match Self::slot(i, j) {
            Some((c, k)) => self.chunks[c].as_ref().map(|ch| ch[k]).unwrap_or(self.fill),
            None => self.fill,
        }
    }

    fn set_raw(&mut self, i: i64, j: i64, v: T) {
        let Some((c, k)) = Self::slot(i, j) else { return };
        let fill = self.fill;
        let ch = self.chunks[c].get_or_insert_with(|| vec![fill; CHUNK * CHUNK].into_boxed_slice());
        ch[k] = v;
        self.any = true;
    }

    /// Does anything differ from untouched?
    pub fn is_empty(&self) -> bool {
        !self.any
    }

    /// Chunks that hold something.
    pub fn used(&self) -> usize {
        self.chunks.iter().filter(|c| c.is_some()).count()
    }

    /// Drop chunks that went back to untouched.
    fn tidy(&mut self) {
        let fill = self.fill;
        for c in &mut self.chunks {
            if c.as_ref().is_some_and(|ch| ch.iter().all(|&v| v == fill)) {
                *c = None;
            }
        }
        self.recount();
    }
}

impl Layer<f32> {
    /// Bilinear value at a point.
    pub fn at(&self, p: V2) -> f32 {
        if !self.any {
            return self.fill;
        }
        let (fx, fy) = (p.x / EDIT_CELL, p.y / EDIT_CELL);
        let (i, j) = (fx.floor(), fy.floor());
        let (u, v) = (fx - i, fy - j);
        let (i, j) = (i as i64, j as i64);
        // A quick out where nothing is stored.
        if let Some((c, _)) = Self::slot(i, j) {
            if self.chunks[c].is_none() && (u < 0.999 && v < 0.999) && (i as usize % CHUNK) < CHUNK - 1 && (j as usize % CHUNK) < CHUNK - 1 {
                return self.fill;
            }
        }
        let a = self.get(i, j);
        let b = self.get(i + 1, j);
        let c = self.get(i, j + 1);
        let d = self.get(i + 1, j + 1);
        let top = a + (b - a) * u;
        let bot = c + (d - c) * u;
        top + (bot - top) * v
    }
}

// Saved as the chunks that hold something.
impl<T: Copy + PartialEq + Serialize> Serialize for Layer<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let used: Vec<(u32, &[T])> = self.chunks.iter().enumerate().filter_map(|(k, c)| c.as_ref().map(|c| (k as u32, &c[..]))).collect();
        (self.fill, used).serialize(s)
    }
}

impl<'de, T: Copy + PartialEq + Deserialize<'de>> Deserialize<'de> for Layer<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (fill, used): (T, Vec<(u32, Vec<T>)>) = Deserialize::deserialize(d)?;
        let mut l = Layer::new(fill);
        for (k, c) in used {
            if (k as usize) < l.chunks.len() && c.len() == CHUNK * CHUNK {
                l.chunks[k as usize] = Some(c.into_boxed_slice());
            }
        }
        l.recount();
        Ok(l)
    }
}

/// A rock placed by hand.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Rock {
    pub pos: V2,
    /// Index into `ROCKS`.
    pub kind: u8,
    /// Rough size, metres.
    pub size: f32,
    pub rot: f32,
    /// Shapes each rock (fixed when it's placed).
    pub seed: u32,
}

/// Untouched plants: the land decides.
pub const PLANTS_NATURAL: [u8; 3] = [128, 128, 128];

/// Everything edited by hand.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MapEdits {
    pub height: Layer<f32>,
    /// Texture (high four bits, index into `TEXTURES` + 1) and how strongly
    /// (low four bits, 0..15).
    pub paint: Layer<u8>,
    /// Grass, brush, trees: 0 none, 128 as the land would have it, 255 as
    /// much as can grow.
    pub plants: Layer<[u8; 3]>,
    pub rocks: Vec<Rock>,
    /// Bumped on every change (the window rebuilds what it drew).
    #[serde(skip)]
    pub version: u32,
    #[serde(skip)]
    undo: Vec<Snapshot>,
    #[serde(skip)]
    redo: Vec<Snapshot>,
    #[serde(skip)]
    stroke: Option<Snapshot>,
}

impl Default for MapEdits {
    fn default() -> Self {
        MapEdits { height: Layer::new(0.0), paint: Layer::new(0), plants: Layer::new(PLANTS_NATURAL), rocks: Vec::new(), version: 0, undo: Vec::new(), redo: Vec::new(), stroke: None }
    }
}

/// Chunks as they were before a stroke (to put back on undo).
#[derive(Clone, Debug, Default, PartialEq)]
struct Snapshot {
    height: Vec<(usize, Option<Box<[f32]>>)>,
    paint: Vec<(usize, Option<Box<[u8]>>)>,
    plants: Vec<(usize, Option<Box<[[u8; 3]]>>)>,
    rocks: Option<Vec<Rock>>,
}

/// What a brush does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Brush {
    Raise,
    Lower,
    /// Even out bumps.
    Smooth,
    /// Add bumps.
    Roughen,
    /// Bring the land toward the height under the brush where the stroke began.
    Flatten,
    /// Bring the land toward a chosen height.
    SetHeight(f32),
    /// Cut the land into steps this many metres tall.
    Terrace(f32),
    /// Take the land back toward how the seed made it.
    Restore,
    /// Lay a ground texture (index into `TEXTURES`).
    Paint(u8),
    /// Take painted ground back toward the land's own.
    Unpaint,
    /// More of a plant kind (index into `PLANTS`); `less` for fewer.
    Plants { kind: u8, less: bool },
    /// Plants back to how the land has them.
    NaturalPlants,
    /// Scatter rocks of a kind.
    Rocks(u8),
    /// Take rocks away.
    ClearRocks,
}

impl Brush {
    pub fn name(self) -> String {
        match self {
            Brush::Raise => "Raise".into(),
            Brush::Lower => "Lower".into(),
            Brush::Smooth => "Smooth".into(),
            Brush::Roughen => "Roughen".into(),
            Brush::Flatten => "Flatten".into(),
            Brush::SetHeight(h) => format!("Set height ({h:.0} m)"),
            Brush::Terrace(s) => format!("Terrace ({s:.0} m steps)"),
            Brush::Restore => "Restore the land".into(),
            Brush::Paint(k) => format!("Paint {}", TEXTURES[k as usize].name.to_lowercase()),
            Brush::Unpaint => "Unpaint".into(),
            Brush::Plants { kind, less } => format!("{} {}", if less { "Fewer" } else { "More" }, PLANTS[kind as usize].to_lowercase()),
            Brush::NaturalPlants => "Natural plants".into(),
            Brush::Rocks(k) => format!("Place {}s", ROCKS[k as usize].to_lowercase()),
            Brush::ClearRocks => "Clear rocks".into(),
        }
    }

    /// Does it change the land's shape (and so the roads)?
    pub fn shapes(self) -> bool {
        matches!(self, Brush::Raise | Brush::Lower | Brush::Smooth | Brush::Roughen | Brush::Flatten | Brush::SetHeight(_) | Brush::Terrace(_) | Brush::Restore)
    }
}

/// One dab of a brush.
#[derive(Clone, Copy, Debug)]
pub struct Dab {
    pub brush: Brush,
    pub at: V2,
    /// Metres.
    pub radius: f32,
    /// 0..1.
    pub strength: f32,
    /// 0 a hard edge .. 1 fading all the way from the middle.
    pub falloff: f32,
    /// Seconds this dab stands for (brushes work at a rate).
    pub dt: f32,
    /// For Flatten: the height to bring the land to.
    pub target: f32,
}

/// The area a change touched: (min, max) corners.
pub type Dirty = (V2, V2);

fn hash01(seed: u64, i: i64, j: i64) -> f32 {
    (rng::key(&[seed, i as u64, j as u64]) >> 40) as f32 / (1u64 << 24) as f32
}

impl MapEdits {
    pub fn is_empty(&self) -> bool {
        self.height.is_empty() && self.paint.is_empty() && self.plants.is_empty() && self.rocks.is_empty()
    }

    /// The painted texture at a point and how strongly (0..1), if any.
    pub fn paint_at(&self, p: V2) -> Option<(u8, f32)> {
        let v = self.paint.get((p.x / EDIT_CELL).round() as i64, (p.y / EDIT_CELL).round() as i64);
        (v & 15 > 0 && v >> 4 > 0).then(|| ((v >> 4) - 1, (v & 15) as f32 / 15.0))
    }

    /// Plant factors at a point: (multiplier on what grows, chance added), by kind.
    pub fn plants_at(&self, p: V2) -> [(f32, f32); 3] {
        let v = self.plants.get((p.x / EDIT_CELL).floor() as i64, (p.y / EDIT_CELL).floor() as i64);
        let mut out = [(1.0, 0.0); 3];
        for k in 0..3 {
            let x = v[k] as f32;
            out[k] = if x <= 128.0 { (x / 128.0, 0.0) } else { (1.0, (x - 128.0) / 127.0 * PLANT_ADD[k]) };
        }
        out
    }

    // ---- Strokes and undo -------------------------------------------------------

    /// A stroke begins: what it changes can be undone as one.
    pub fn begin_stroke(&mut self) {
        self.stroke = Some(Snapshot::default());
    }

    /// The stroke is over.
    pub fn end_stroke(&mut self) {
        if let Some(s) = self.stroke.take() {
            if s != Snapshot::default() {
                self.undo.push(s);
                if self.undo.len() > UNDO_DEPTH {
                    self.undo.remove(0);
                }
                self.redo.clear();
            }
        }
        self.height.tidy();
        self.paint.tidy();
        self.plants.tidy();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn swap(&mut self, s: Snapshot) -> Snapshot {
        let mut back = Snapshot::default();
        for (k, c) in s.height {
            back.height.push((k, std::mem::replace(&mut self.height.chunks[k], c)));
        }
        for (k, c) in s.paint {
            back.paint.push((k, std::mem::replace(&mut self.paint.chunks[k], c)));
        }
        for (k, c) in s.plants {
            back.plants.push((k, std::mem::replace(&mut self.plants.chunks[k], c)));
        }
        if let Some(r) = s.rocks {
            back.rocks = Some(std::mem::replace(&mut self.rocks, r));
        }
        self.height.recount();
        self.paint.recount();
        self.plants.recount();
        self.version = self.version.wrapping_add(1);
        back
    }

    /// Take back the last stroke. True if there was one.
    pub fn undo(&mut self) -> bool {
        let Some(s) = self.undo.pop() else { return false };
        let back = self.swap(s);
        self.redo.push(back);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(s) = self.redo.pop() else { return false };
        let back = self.swap(s);
        self.undo.push(back);
        true
    }

    /// Before a chunk of a layer is changed in a stroke, keep how it was.
    fn keep_height(&mut self, c: usize) {
        if let Some(s) = self.stroke.as_mut() {
            if !s.height.iter().any(|x| x.0 == c) {
                s.height.push((c, self.height.chunks[c].clone()));
            }
        }
    }
    fn keep_paint(&mut self, c: usize) {
        if let Some(s) = self.stroke.as_mut() {
            if !s.paint.iter().any(|x| x.0 == c) {
                s.paint.push((c, self.paint.chunks[c].clone()));
            }
        }
    }
    fn keep_plants(&mut self, c: usize) {
        if let Some(s) = self.stroke.as_mut() {
            if !s.plants.iter().any(|x| x.0 == c) {
                s.plants.push((c, self.plants.chunks[c].clone()));
            }
        }
    }
    fn keep_rocks(&mut self) {
        let rocks = self.rocks.clone();
        if let Some(s) = self.stroke.as_mut() {
            if s.rocks.is_none() {
                s.rocks = Some(rocks);
            }
        }
    }

    /// Clear every edit (can be undone).
    pub fn clear_all(&mut self) {
        self.begin_stroke();
        for c in 0..self.height.chunks.len() {
            if self.height.chunks[c].is_some() {
                self.keep_height(c);
                self.height.chunks[c] = None;
            }
            if self.paint.chunks[c].is_some() {
                self.keep_paint(c);
                self.paint.chunks[c] = None;
            }
            if self.plants.chunks[c].is_some() {
                self.keep_plants(c);
                self.plants.chunks[c] = None;
            }
        }
        if !self.rocks.is_empty() {
            self.keep_rocks();
            self.rocks.clear();
        }
        self.height.recount();
        self.paint.recount();
        self.plants.recount();
        self.end_stroke();
        self.version = self.version.wrapping_add(1);
    }

    // ---- Files -------------------------------------------------------------------

    /// The map file for a seed.
    pub fn path_for(seed: u64) -> std::path::PathBuf {
        std::path::PathBuf::from("maps").join(format!("seed-{seed}.gmap"))
    }

    pub fn to_bytes(&self, seed: u64) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAP_MAGIC);
        out.extend_from_slice(&MAP_FORMAT.to_le_bytes());
        out.extend_from_slice(&seed.to_le_bytes());
        bincode::serialize_into(&mut out, self).expect("edits always serialise");
        out
    }

    /// Edits back from a map file's bytes, and the seed they were made for.
    pub fn from_bytes(b: &[u8]) -> Result<(u64, MapEdits), String> {
        if b.len() < 16 || &b[..4] != MAP_MAGIC {
            return Err("not a map file".into());
        }
        let v = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
        if v != MAP_FORMAT {
            return Err(format!("a map from another version (format {v}, this is {MAP_FORMAT})"));
        }
        let seed = u64::from_le_bytes(b[8..16].try_into().unwrap());
        let e: MapEdits = bincode::deserialize(&b[16..]).map_err(|e| e.to_string())?;
        Ok((seed, e))
    }

    pub fn save_to(&self, seed: u64, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::File::create(&tmp)?.write_all(&self.to_bytes(seed))?;
        std::fs::rename(tmp, path)
    }

    pub fn load_from(path: &std::path::Path) -> Result<(u64, MapEdits), String> {
        let mut b = Vec::new();
        std::fs::File::open(path).and_then(|mut f| f.read_to_end(&mut b)).map_err(|e| e.to_string())?;
        MapEdits::from_bytes(&b)
    }
}

const MAP_MAGIC: &[u8; 4] = b"GMAP";
/// Bumped whenever a map file's shape changes.
pub const MAP_FORMAT: u32 = 1;

// ---- Brushes ----------------------------------------------------------------

impl Terrain {
    /// Apply a dab. Returns the area it changed, if anything.
    pub fn dab(&mut self, d: Dab) -> Option<Dirty> {
        let r = d.radius.max(EDIT_CELL);
        let (lo, hi) = (V2::new(d.at.x - r, d.at.y - r), V2::new(d.at.x + r, d.at.y + r));
        let cells = |v: f32| (v / EDIT_CELL).floor() as i64;
        let (i0, i1, j0, j1) = (cells(lo.x), cells(hi.x) + 1, cells(lo.y), cells(hi.y) + 1);
        // How much a cell feels the brush, 0..1.
        let weight = |q: V2| -> f32 {
            let x = q.dist(d.at) / r;
            if x >= 1.0 {
                return 0.0;
            }
            let hard = 1.0 - d.falloff.clamp(0.0, 1.0);
            if x <= hard {
                1.0
            } else {
                let t = (x - hard) / (1.0 - hard).max(1e-3);
                1.0 - t * t * (3.0 - 2.0 * t)
            }
        };
        let rate = (d.strength.clamp(0.0, 1.0) * d.dt).min(1.0);
        match d.brush {
            Brush::Rocks(kind) => {
                self.edits.keep_rocks();
                let area = std::f32::consts::PI * r * r / 100.0;
                let want = area * ROCK_RATE * rate;
                let mut g = Rng::from_keys(&[self.seed(), d.at.x.to_bits() as u64, d.at.y.to_bits() as u64, self.edits.version as u64, 0x524F_434B]);
                let mut n = want.floor() as usize + g.chance(want.fract()) as usize;
                let mut placed = false;
                while n > 0 {
                    n -= 1;
                    let a = g.f32() * std::f32::consts::TAU;
                    let q = d.at.add(V2::new(a.cos(), a.sin()).scale(r * g.f32().sqrt()));
                    if !super::geo::is_land(q) || g.f32() > weight(q) {
                        continue;
                    }
                    let size = match kind {
                        0 => g.range(1.0, 3.0),
                        1 => g.range(1.5, 4.0),
                        2 => g.range(1.0, 2.0),
                        3 => g.range(0.3, 0.9),
                        _ => g.range(4.0, 9.0),
                    };
                    self.edits.rocks.push(Rock { pos: q, kind, size, rot: g.f32() * std::f32::consts::TAU, seed: g.next_u64() as u32 });
                    placed = true;
                }
                self.edits.version = self.edits.version.wrapping_add(1);
                return placed.then_some((lo, hi));
            }
            Brush::ClearRocks => {
                let before = self.edits.rocks.len();
                if self.edits.rocks.iter().any(|k| weight(k.pos) > 0.0) {
                    self.edits.keep_rocks();
                }
                let mut g = Rng::from_keys(&[self.seed(), d.at.x.to_bits() as u64, self.edits.version as u64, 0x434C_5252]);
                self.edits.rocks.retain(|k| !(weight(k.pos) > 0.0 && g.f32() < (rate * 4.0 * weight(k.pos)).min(1.0)));
                self.edits.version = self.edits.version.wrapping_add(1);
                return (self.edits.rocks.len() != before).then_some((lo, hi));
            }
            _ => {}
        }
        // Keep every touched chunk for undo first.
        let mut chunks: Vec<usize> = Vec::new();
        for cj in (j0.max(0) as usize / CHUNK)..=((j1.max(0) as usize / CHUNK).min(CHUNKS - 1)) {
            for ci in (i0.max(0) as usize / CHUNK)..=((i1.max(0) as usize / CHUNK).min(CHUNKS - 1)) {
                chunks.push(cj * CHUNKS + ci);
            }
        }
        for &c in &chunks {
            match d.brush {
                Brush::Paint(_) | Brush::Unpaint => self.edits.keep_paint(c),
                Brush::Plants { .. } | Brush::NaturalPlants => self.edits.keep_plants(c),
                _ => self.edits.keep_height(c),
            }
        }
        // Heights are worked out from the land as it was before this dab.
        let total = |t: &Terrain, i: i64, j: i64| t.base_at(i, j) + t.edits.height.get(i, j);
        let mut changes: Vec<(i64, i64, f32)> = Vec::new();
        for j in j0..=j1 {
            for i in i0..=i1 {
                let q = V2::new(i as f32 * EDIT_CELL, j as f32 * EDIT_CELL);
                let w = weight(q);
                if w <= 0.0 {
                    continue;
                }
                let k = rate * w;
                let here = total(self, i, j);
                let delta = self.edits.height.get(i, j);
                let new = match d.brush {
                    Brush::Raise => delta + SCULPT_RATE * d.strength * d.dt * w,
                    Brush::Lower => delta - SCULPT_RATE * d.strength * d.dt * w,
                    Brush::Smooth => {
                        let s = 2;
                        let mut sum = 0.0;
                        let mut n = 0.0;
                        for dj in -s..=s {
                            for di in -s..=s {
                                sum += total(self, i + di, j + dj);
                                n += 1.0;
                            }
                        }
                        delta + (sum / n - here) * (k * 4.0).min(1.0)
                    }
                    Brush::Roughen => {
                        let n = hash01(self.seed() ^ 0x5255_4748, i, j) - 0.5 + (hash01(self.seed() ^ 0x5255_4749, i / 4, j / 4) - 0.5) * 1.5;
                        delta + n * SCULPT_RATE * 0.6 * d.strength * d.dt * w
                    }
                    Brush::Flatten | Brush::SetHeight(_) => {
                        let target = if let Brush::SetHeight(h) = d.brush { h } else { d.target };
                        delta + (target - here) * (k * 3.0).min(1.0)
                    }
                    Brush::Terrace(step) => {
                        let step = step.max(0.5);
                        let target = (here / step).floor() * step + step * smooth_step((here / step).fract(), 0.85, 1.0);
                        delta + (target - here) * (k * 3.0).min(1.0)
                    }
                    Brush::Restore => delta * (1.0 - (k * 3.0).min(1.0)),
                    Brush::Paint(tex) => {
                        let v = self.edits.paint.get(i, j);
                        let (old_t, old_w) = (v >> 4, (v & 15) as f32);
                        let gain = (k * 30.0).max(1.0);
                        let w2 = if old_t == tex + 1 { (old_w + gain).min(15.0) } else if old_w > gain { old_w - gain } else { gain.min(15.0) };
                        let t2 = if old_t == tex + 1 || old_w <= gain { tex + 1 } else { old_t };
                        self.edits.paint.set_raw(i, j, (t2 << 4) | (w2.round() as u8).min(15));
                        continue;
                    }
                    Brush::Unpaint => {
                        let v = self.edits.paint.get(i, j);
                        let w2 = ((v & 15) as f32 - (k * 30.0).max(1.0)).max(0.0) as u8;
                        self.edits.paint.set_raw(i, j, if w2 == 0 { 0 } else { (v & 0xF0) | w2 });
                        continue;
                    }
                    Brush::Plants { kind, less } => {
                        let mut v = self.edits.plants.get(i, j);
                        let step = (k * 400.0).max(1.0);
                        let x = v[kind as usize] as f32 + if less { -step } else { step };
                        v[kind as usize] = x.clamp(0.0, 255.0) as u8;
                        self.edits.plants.set_raw(i, j, v);
                        continue;
                    }
                    Brush::NaturalPlants => {
                        let mut v = self.edits.plants.get(i, j);
                        for x in v.iter_mut() {
                            let f = *x as f32 + (128.0 - *x as f32) * (k * 3.0).min(1.0);
                            *x = if (f - 128.0).abs() < 2.0 { 128 } else { f.round() as u8 };
                        }
                        self.edits.plants.set_raw(i, j, v);
                        continue;
                    }
                    Brush::Rocks(_) | Brush::ClearRocks => continue,
                };
                changes.push((i, j, new));
            }
        }
        for (i, j, v) in changes {
            self.edits.height.set_raw(i, j, if v.abs() < 1e-3 { 0.0 } else { v });
        }
        self.edits.version = self.edits.version.wrapping_add(1);
        Some((lo, hi))
    }

    /// The seed's own land at an edit cell's corner.
    fn base_at(&self, i: i64, j: i64) -> f32 {
        self.base_height(V2::new(i as f32 * EDIT_CELL, j as f32 * EDIT_CELL))
    }

    /// A ramp or path: the land along a line from `a` to `b` is brought to a
    /// straight slope between their heights, `width` across.
    pub fn ramp(&mut self, a: V2, b: V2, width: f32, falloff: f32) -> Option<Dirty> {
        let (ha, hb) = (self.height(a), self.height(b));
        let len = a.dist(b).max(1.0);
        let r = width * 0.5;
        let lo = V2::new(a.x.min(b.x) - r, a.y.min(b.y) - r);
        let hi = V2::new(a.x.max(b.x) + r, a.y.max(b.y) + r);
        let cells = |v: f32| (v / EDIT_CELL).floor() as i64;
        let (i0, i1, j0, j1) = (cells(lo.x), cells(hi.x) + 1, cells(lo.y), cells(hi.y) + 1);
        for cj in (j0.max(0) as usize / CHUNK)..=((j1.max(0) as usize / CHUNK).min(CHUNKS - 1)) {
            for ci in (i0.max(0) as usize / CHUNK)..=((i1.max(0) as usize / CHUNK).min(CHUNKS - 1)) {
                self.edits.keep_height(cj * CHUNKS + ci);
            }
        }
        let dir = b.sub(a).scale(1.0 / len);
        let mut changes = Vec::new();
        for j in j0..=j1 {
            for i in i0..=i1 {
                let q = V2::new(i as f32 * EDIT_CELL, j as f32 * EDIT_CELL);
                let qa = q.sub(a);
                let along = ((qa.x * dir.x + qa.y * dir.y) / len).clamp(0.0, 1.0);
                let off = super::terrain::seg_dist(a, b, q);
                if off > r {
                    continue;
                }
                let hard = 1.0 - falloff.clamp(0.0, 1.0);
                let x = off / r;
                let w = if x <= hard { 1.0 } else { 1.0 - smooth_step((x - hard) / (1.0 - hard).max(1e-3), 0.0, 1.0) };
                let target = ha + (hb - ha) * along;
                let here = self.base_at(i, j) + self.edits.height.get(i, j);
                changes.push((i, j, self.edits.height.get(i, j) + (target - here) * w));
            }
        }
        for (i, j, v) in changes {
            self.edits.height.set_raw(i, j, v);
        }
        self.edits.version = self.edits.version.wrapping_add(1);
        Some((lo, hi))
    }
}

fn smooth_step(x: f32, a: f32, b: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl super::world::World {
    /// After the land's shape or ground has been edited: find the roads again
    /// over it (towns, people and journeys under way stay as they are).
    pub fn refit_land(&mut self) {
        let t = std::mem::replace(&mut self.terrain, Terrain::empty());
        let (t, r) = super::worldgen::land(t, &self.settlements);
        self.terrain = t;
        self.routes = r;
    }
}
