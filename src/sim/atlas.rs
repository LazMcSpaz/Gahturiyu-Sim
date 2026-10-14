//! The atlas: the world's land, read from Laz's map (Laz, 13 Oct 2026:
//! "make the landmass smooth and coherent"; `docs/map/README.md`).
//!
//! The map is a raster, one pixel `PX` metres across. Each pixel has a
//! **class** from the map's legend (sea, coast, plains, forest, bog, rocky
//! ground, snowy mountains, the Overgrowth, fresh water), a **base height**
//! (the class's height, the lowland rising gently inland, all blurred so
//! mountain feet and shores are coherent at the walking scale), a signed
//! **distance to the sea** (negative in it: what `geo::inland` answers now)
//! and a **mountain share**. The seed adds the small relief on top
//! (`terrain.rs`); everything else about the world still comes from it.
//!
//! The layers are read from `assets/world/` when they've been exported
//! (`terrain.png` with the legend's own colours; later `water.png`,
//! `roads.png`, `cliffs.png`, `territory.png`). Until then Laz's clean map
//! (`docs/map/laz-map-2026-10-13.png`) is read by colour: each pixel goes
//! to the nearest legend colour by hue (the hill-shading only lightens or
//! darkens), the legend box is sea, and the dark ink (cliff ticks,
//! outlines) takes its neighbours' class. The older annotated draft is the
//! last resort (sea, snow, rock, the Overgrowth; its tinted lowland all
//! plains). The atlas is built once per process (`atlas()`), ~2 s.
//!
//! Towns are marks on the map (`assets/world/towns.ron`: pixel, kind,
//! founders, holder), placeholders until Laz's list.

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

use super::geo::V2;
use super::race::Race;

/// Metres a pixel: 36 hours of travel from the Imperial capital to the
/// crossroads village (Laz) over about 1 100 pixels of road.
pub const PX: f32 = 150.0;

/// What the ground is, from the legend.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Sea = 0,
    /// Land within `COAST_PX` of the sea.
    Coast = 1,
    Plains = 2,
    Forest = 3,
    Bog = 4,
    Rock = 5,
    Snow = 6,
    Overgrowth = 7,
    /// Rivers and lakes.
    Water = 8,
}

impl Class {
    pub const ALL: [Class; 9] = [Class::Sea, Class::Coast, Class::Plains, Class::Forest, Class::Bog, Class::Rock, Class::Snow, Class::Overgrowth, Class::Water];

    pub fn from_u8(v: u8) -> Class {
        Class::ALL.get(v as usize).copied().unwrap_or(Class::Sea)
    }

    pub fn name(self) -> &'static str {
        match self {
            Class::Sea => "sea",
            Class::Coast => "coast",
            Class::Plains => "rolling plains",
            Class::Forest => "forest",
            Class::Bog => "bog",
            Class::Rock => "rocky ground",
            Class::Snow => "snowy mountains",
            Class::Overgrowth => "the Overgrowth",
            Class::Water => "water",
        }
    }

    /// The class's height before the lowland's rise and the blur, metres.
    fn base_height(self) -> f32 {
        match self {
            Class::Sea => 0.0,
            Class::Coast => 4.0,
            Class::Plains => 25.0,
            Class::Forest => 35.0,
            Class::Bog => 3.0,
            Class::Rock => 260.0,
            Class::Snow => 700.0,
            Class::Overgrowth => 30.0,
            // A river's pixel is 150 m wide: lowland height, so it doesn't
            // cut a trench (rivers as water are the next piece).
            Class::Water => 20.0,
        }
    }

    /// How mountainous the class is, 0..1: the snowy tops are mountain;
    /// rocky ground is upland (its own ground, `terrain.rs`).
    fn mountain(self) -> f32 {
        match self {
            Class::Snow => 1.0,
            Class::Rock => 0.3,
            _ => 0.0,
        }
    }

    pub fn is_land(self) -> bool {
        self != Class::Sea
    }
}

/// Land within this many pixels of the sea is coast.
pub const COAST_PX: usize = 2;
/// Nobody's town stands higher than this, metres (`dry_spot`).
pub const TOWN_HEIGHT: f32 = 240.0;
/// The blur's radius in pixels (two passes): shores and mountain feet
/// become slopes a few hundred metres long.
const BLUR_PX: usize = 4;
/// The lowland rises this much over this far from the sea.
const LOWLAND_RISE: f32 = 40.0;
const LOWLAND_OVER: f32 = 15_000.0;
/// How deep the sea gets, per metre from the shore, and at most.
pub const SEA_DEPTH_PER_M: f32 = 0.02;
pub const SEA_DEPTH_MOST: f32 = 60.0;

/// The sea's colours as the annotated draft paints them.
const SEA_RGB: [[u8; 3]; 2] = [[180, 196, 216], [184, 200, 216]];

/// The clean map's legend colours (sampled from its own legend box) and
/// the class each means. The exported layers use the same.
const LEGEND: [([u8; 3], Class); 9] = [
    ([72, 112, 160], Class::Sea),
    ([164, 186, 112], Class::Plains),
    ([78, 116, 72], Class::Forest),
    ([220, 206, 158], Class::Coast),
    ([122, 128, 92], Class::Bog),
    ([150, 138, 124], Class::Rock),
    ([242, 244, 248], Class::Snow),
    ([64, 82, 70], Class::Overgrowth),
    ([140, 165, 195], Class::Water),
];
/// The clean map's legend box, pixels (over the sea, bottom left): sea.
const LEGEND_BOX: (usize, usize, usize, usize) = (40, 1550, 392, 1904);
/// Ink this dark (cliff ticks, outlines, the legend's text) is no class:
/// it takes its neighbours'.
const INK_SUM: u32 = 230;
const UNKNOWN: u8 = 255;

/// A pixel's share of red, green and blue (its hue, free of the shading).
fn chroma(rgb: [u8; 3]) -> [f32; 3] {
    let s = (rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32).max(1.0);
    [rgb[0] as f32 / s, rgb[1] as f32 / s, rgb[2] as f32 / s]
}

/// A clean-map pixel's class: the nearest legend colour by hue, with a
/// little weight on brightness (the forest and the Overgrowth, snow and
/// light rock). Dark ink is `UNKNOWN`.
fn classify_clean(rgb: [u8; 3]) -> u8 {
    let sum = rgb[0] as u32 + rgb[1] as u32 + rgb[2] as u32;
    if sum < INK_SUM {
        return UNKNOWN;
    }
    let c = chroma(rgb);
    let mut best = (f32::MAX, Class::Sea);
    for (lc, class) in LEGEND {
        let l = chroma(lc);
        let hue = (0..3).map(|i| (c[i] - l[i]).abs()).sum::<f32>();
        let lsum = lc[0] as f32 + lc[1] as f32 + lc[2] as f32;
        let bright = (sum as f32 - lsum).abs() / 765.0 * 0.08;
        let d = hue + bright;
        if d < best.0 {
            best = (d, class);
        }
    }
    best.1 as u8
}

/// Give every `UNKNOWN` pixel the commonest class among its known
/// neighbours, widening the look until one is found.
fn fill_unknown(class: &mut [u8], w: usize, h: usize) {
    for radius in 1..=6usize {
        let before = class.to_vec();
        let mut left = false;
        for j in 0..h {
            for i in 0..w {
                let k = j * w + i;
                if before[k] != UNKNOWN {
                    continue;
                }
                let mut count = [0u32; 9];
                for dj in -(radius as i64)..=radius as i64 {
                    for di in -(radius as i64)..=radius as i64 {
                        let (ii, jj) = (i as i64 + di, j as i64 + dj);
                        if ii >= 0 && jj >= 0 && ii < w as i64 && jj < h as i64 {
                            let c = before[jj as usize * w + ii as usize];
                            if c != UNKNOWN {
                                count[c as usize] += 1;
                            }
                        }
                    }
                }
                if let Some((c, _)) = count.iter().enumerate().filter(|(_, n)| **n > 0).max_by_key(|(_, n)| **n) {
                    class[k] = c as u8;
                } else {
                    left = true;
                }
            }
        }
        if !left {
            return;
        }
    }
    for c in class.iter_mut() {
        if *c == UNKNOWN {
            *c = Class::Sea as u8;
        }
    }
}

pub struct Atlas {
    /// Pixels across and down.
    pub w: usize,
    pub h: usize,
    class: Vec<u8>,
    /// Base height per pixel, metres (blurred).
    base: Vec<f32>,
    /// Signed distance to the sea per pixel, metres: positive inland.
    sea: Vec<f32>,
    /// Mountain share per pixel, 0..1 (blurred).
    mount: Vec<f32>,
    /// Every land pixel, for picking spots.
    land: Vec<u32>,
    /// Per pixel: on the mainland (the biggest piece of land) or not.
    main: Vec<bool>,
    /// Where it came from.
    pub source: String,
}

static ATLAS: OnceLock<Atlas> = OnceLock::new();

/// The world's atlas, built once per process.
pub fn atlas() -> &'static Atlas {
    ATLAS.get_or_init(|| match Atlas::load() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("atlas: {e}; using a small round island instead");
            Atlas::fallback()
        }
    })
}

/// The map files, in the order tried.
const SOURCES: [&str; 3] = ["assets/world/terrain.png", "docs/map/laz-map-2026-10-13.png", "docs/map/laz-map-draft-2026-10-13.png"];

/// What kind of place a mark on the map is (its size and look).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Capital,
    City,
    Town,
    Village,
    Hamlet,
    Outpost,
    Fort,
}

impl Kind {
    /// The settlement's size (the generator's 0.35..2.15 scale).
    pub fn size(self) -> f32 {
        match self {
            Kind::Capital => 2.2,
            Kind::City => 1.8,
            Kind::Town => 1.2,
            Kind::Village => 0.7,
            Kind::Hamlet => 0.45,
            Kind::Outpost => 0.5,
            Kind::Fort => 0.8,
        }
    }
}

/// A town as marked on the map.
#[derive(Deserialize, Clone, Debug)]
pub struct TownMark {
    /// Stable, lower-case: the town's name in data (`roster.rs`: its roster
    /// is `assets/world/people/<key>.ron`). Left out, it's made from the label.
    #[serde(default)]
    pub key: String,
    /// The mark's id on Laz's map page (`marks-export.json`): the key stays
    /// with it (`tools/map/marks_to_towns.py`).
    #[serde(default)]
    pub id: String,
    /// What the map says (not the town's name: that's made by the names).
    pub label: String,
    /// The pixel on the map.
    pub px: (u32, u32),
    pub kind: Kind,
    pub founders: Race,
    /// A faction key (`factions.rs`), or empty for nobody.
    #[serde(default)]
    pub holder: String,
}

impl TownMark {
    /// The town's key (`key`, else made from the label).
    pub fn key(&self) -> String {
        if self.key.is_empty() { super::roster::key_of_label(&self.label) } else { self.key.clone() }
    }

    /// The mark's spot in metres (the pixel's centre).
    pub fn pos(&self) -> V2 {
        V2::new(self.px.0 as f32 * PX + PX / 2.0, self.px.1 as f32 * PX + PX / 2.0)
    }
}

/// The towns marked on the map (`assets/world/towns.ron`).
pub fn town_marks() -> &'static [TownMark] {
    static MARKS: OnceLock<Vec<TownMark>> = OnceLock::new();
    MARKS.get_or_init(|| ron::from_str(include_str!("../../assets/world/towns.ron")).unwrap_or_else(|e| panic!("assets/world/towns.ron: {e}")))
}

/// Look for a map file beside the working directory, its parents, and the
/// running program (the game is launched from its clone; tests from the
/// crate; a screenshot from anywhere).
fn find(rel: &str) -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.clone());
        for a in cwd.ancestors().skip(1).take(3) {
            roots.push(a.to_path_buf());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for a in exe.ancestors().skip(1).take(4) {
            roots.push(a.to_path_buf());
        }
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        roots.push(PathBuf::from(dir));
    }
    roots.into_iter().map(|r| r.join(rel)).find(|p| p.is_file())
}

fn colour_dist(a: [u8; 3], b: [u8; 3]) -> i32 {
    (0..3).map(|i| (a[i] as i32 - b[i] as i32).abs()).sum()
}

/// A draft pixel's class by its colour: the sea's blue-grey, snow's white,
/// rock's warm grey, the Overgrowth's green-grey, the lake's blue; the
/// tinted lowland is plains.
fn classify(rgb: [u8; 3]) -> Class {
    if SEA_RGB.iter().any(|s| colour_dist(rgb, *s) <= 24) {
        return Class::Sea;
    }
    let [r, g, b] = rgb;
    let (lo, hi) = (r.min(g).min(b), r.max(g).max(b));
    let spread = hi - lo;
    if lo >= 225 && spread <= 24 {
        return Class::Snow;
    }
    if spread <= 26 {
        // Grey: the Overgrowth's leans green, rocky ground's leans warm.
        if g >= r + 2 && g >= b + 2 {
            return Class::Overgrowth;
        }
        if hi >= 150 {
            return Class::Rock;
        }
        return Class::Overgrowth;
    }
    if b > r + 20 && b > g + 4 && b >= 180 {
        return Class::Water;
    }
    Class::Plains
}

impl Atlas {
    /// Read the first map file found.
    pub fn load() -> Result<Atlas, String> {
        let (path, rel) = SOURCES.iter().find_map(|rel| find(rel).map(|p| (p, *rel))).ok_or_else(|| format!("no map file: looked for {}", SOURCES.join(", ")))?;
        let img = image::open(&path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgb8();
        let (w, h) = (img.width() as usize, img.height() as usize);
        let mut class = vec![0u8; w * h];
        if rel.contains("draft") {
            for (i, px) in img.pixels().enumerate() {
                class[i] = classify(px.0) as u8;
            }
        } else {
            for (i, px) in img.pixels().enumerate() {
                class[i] = classify_clean(px.0);
            }
            if rel.starts_with("docs/") {
                let (x0, y0, x1, y1) = LEGEND_BOX;
                for j in y0..y1.min(h) {
                    for i in x0..x1.min(w) {
                        class[j * w + i] = Class::Sea as u8;
                    }
                }
            }
            fill_unknown(&mut class, w, h);
        }
        Ok(Atlas::build(w, h, class, rel))
    }

    /// A small round island, for when there's no map file at all.
    pub fn fallback() -> Atlas {
        let (w, h) = (200usize, 200usize);
        let mut class = vec![0u8; w * h];
        for j in 0..h {
            for i in 0..w {
                let (dx, dy) = (i as f32 - 100.0, j as f32 - 100.0);
                let r = (dx * dx + dy * dy).sqrt();
                class[j * w + i] = if r < 70.0 { Class::Plains as u8 } else { Class::Sea as u8 };
            }
        }
        Atlas::build(w, h, class, "a round island")
    }

    /// From the classes: the coast, the distance to the sea, the heights
    /// and the mountain share.
    fn build(w: usize, h: usize, mut class: Vec<u8>, source: &str) -> Atlas {
        let n = w * h;
        // The distance to the sea, in pixels (a chamfer transform, two passes).
        let big = 1e6f32;
        let mut to_sea = vec![big; n];
        let mut to_land = vec![big; n];
        for i in 0..n {
            if class[i] == Class::Sea as u8 {
                to_sea[i] = 0.0;
            } else {
                to_land[i] = 0.0;
            }
        }
        chamfer(&mut to_sea, w, h);
        chamfer(&mut to_land, w, h);
        // Land within reach of the sea is coast.
        for i in 0..n {
            if class[i] != Class::Sea as u8 && class[i] != Class::Snow as u8 && class[i] != Class::Rock as u8 && to_sea[i] <= COAST_PX as f32 {
                class[i] = Class::Coast as u8;
            }
        }
        // (The waterline runs between a sea pixel and a land pixel: half a
        // pixel from either centre, so the distance climbs one metre a metre
        // across it, not two.)
        let mut sea = vec![0.0f32; n];
        for i in 0..n {
            sea[i] = if class[i] == Class::Sea as u8 { -(to_land[i] - 0.5) * PX } else { (to_sea[i] - 0.5) * PX };
        }
        // Heights and the mountain share, then the blur.
        let mut base = vec![0.0f32; n];
        let mut mount = vec![0.0f32; n];
        for i in 0..n {
            let c = Class::from_u8(class[i]);
            let inland = sea[i].max(0.0);
            let rise = if c.is_land() && c != Class::Snow && c != Class::Rock { LOWLAND_RISE * smooth(0.0, LOWLAND_OVER, inland) } else { 0.0 };
            base[i] = c.base_height() + rise;
            mount[i] = c.mountain();
        }
        blur(&mut base, w, h, BLUR_PX);
        blur(&mut base, w, h, BLUR_PX);
        blur(&mut mount, w, h, BLUR_PX);
        blur(&mut mount, w, h, BLUR_PX);
        // The sea stays below the waterline whatever the blur did, land above.
        for i in 0..n {
            if class[i] == Class::Sea as u8 {
                base[i] = -1.0 - (-(sea[i])).min(SEA_DEPTH_MOST / SEA_DEPTH_PER_M) * SEA_DEPTH_PER_M;
            } else {
                base[i] = base[i].max(0.6);
            }
        }
        let land: Vec<u32> = (0..n).filter(|&i| class[i] != Class::Sea as u8).map(|i| i as u32).collect();
        // The pieces of land, and the biggest of them: the mainland. (Towns
        // on the others are offshore: no road reaches them.)
        let mut piece = vec![u32::MAX; n];
        let mut sizes: Vec<usize> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();
        for &start in &land {
            let start = start as usize;
            if piece[start] != u32::MAX {
                continue;
            }
            let id = sizes.len() as u32;
            sizes.push(0);
            piece[start] = id;
            stack.push(start);
            while let Some(k) = stack.pop() {
                sizes[id as usize] += 1;
                let (i, j) = ((k % w) as i64, (k / w) as i64);
                for (di, dj) in [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)] {
                    let (ii, jj) = (i + di, j + dj);
                    if ii < 0 || jj < 0 || ii >= w as i64 || jj >= h as i64 {
                        continue;
                    }
                    let m = jj as usize * w + ii as usize;
                    if class[m] != Class::Sea as u8 && piece[m] == u32::MAX {
                        piece[m] = id;
                        stack.push(m);
                    }
                }
            }
        }
        let biggest = (0..sizes.len()).max_by_key(|&k| sizes[k]).map(|k| k as u32).unwrap_or(u32::MAX);
        let main: Vec<bool> = (0..n).map(|i| piece[i] == biggest).collect();
        Atlas { w, h, class, base, sea, mount, land, main, source: source.to_string() }
    }

    /// The world's extent, metres: the map's width (the world is square,
    /// the sea padding the rest).
    pub fn extent(&self) -> f32 {
        self.w.max(self.h) as f32 * PX
    }

    /// The map's own width and height, metres.
    pub fn size(&self) -> (f32, f32) {
        (self.w as f32 * PX, self.h as f32 * PX)
    }

    fn idx(&self, i: i64, j: i64) -> Option<usize> {
        if i < 0 || j < 0 || i >= self.w as i64 || j >= self.h as i64 {
            None
        } else {
            Some(j as usize * self.w + i as usize)
        }
    }

    /// Is a spot on the mainland, the map's biggest piece of land (an
    /// island's town is offshore: no road reaches it)?
    pub fn on_mainland(&self, p: V2) -> bool {
        let (i, j) = ((p.x / PX).floor() as i64, (p.y / PX).floor() as i64);
        self.idx(i, j).map(|k| self.main[k]).unwrap_or(false)
    }

    /// The class at a spot (off the map: sea).
    pub fn class_at(&self, p: V2) -> Class {
        let (i, j) = ((p.x / PX).floor() as i64, (p.y / PX).floor() as i64);
        self.idx(i, j).map(|k| Class::from_u8(self.class[k])).unwrap_or(Class::Sea)
    }

    fn sample(&self, v: &[f32], p: V2, off: f32) -> f32 {
        // Cell centres at (i + 0.5) * PX.
        let (fx, fy) = (p.x / PX - 0.5, p.y / PX - 0.5);
        let (ix, iy) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - ix, fy - iy);
        let (ix, iy) = (ix as i64, iy as i64);
        let at = |i: i64, j: i64| self.idx(i, j).map(|k| v[k]).unwrap_or(off);
        let a = at(ix, iy);
        let b = at(ix + 1, iy);
        let c = at(ix, iy + 1);
        let d = at(ix + 1, iy + 1);
        let top = a + (b - a) * tx;
        let bot = c + (d - c) * tx;
        top + (bot - top) * ty
    }

    /// The base height at a spot, metres (below 0 in the sea).
    pub fn base_height(&self, p: V2) -> f32 {
        self.sample(&self.base, p, -SEA_DEPTH_MOST)
    }

    /// How far inland a spot is, metres (negative at sea; far negative off the map).
    pub fn inland(&self, p: V2) -> f32 {
        self.sample(&self.sea, p, -self.extent())
    }

    /// How mountainous a spot is, 0..1.
    pub fn mountain(&self, p: V2) -> f32 {
        self.sample(&self.mount, p, 0.0)
    }

    pub fn is_land(&self, p: V2) -> bool {
        self.inland(p) > 0.0
    }

    /// The way to the sea from a spot (unit; zero if none near), from the
    /// distance field's fall.
    pub fn toward_sea(&self, p: V2) -> V2 {
        let e = PX;
        let dx = self.inland(p.add(V2::new(e, 0.0))) - self.inland(p.sub(V2::new(e, 0.0)));
        let dy = self.inland(p.add(V2::new(0.0, e))) - self.inland(p.sub(V2::new(0.0, e)));
        let v = V2::new(-dx, -dy);
        let len = v.len();
        if len < 1e-3 {
            V2::new(0.0, 0.0)
        } else {
            v.scale(1.0 / len)
        }
    }

    /// The nearest spot in the sea from `p`, roughly, this far past the
    /// shore (walking the distance field down).
    pub fn seaward(&self, p: V2, past: f32) -> V2 {
        let mut q = p;
        for _ in 0..200 {
            let d = self.inland(q);
            if d <= -past {
                return q;
            }
            let dir = self.toward_sea(q);
            if dir.len() < 0.5 {
                return q;
            }
            q = q.add(dir.scale((d + past).max(PX * 0.5).min(PX * 4.0)));
        }
        q
    }

    /// The nearest open sea to `p`: a spot at least `past` from any shore,
    /// found by looking round in widening rings (a walk down the distance
    /// field can end in a river, which the map counts as water). `p` itself
    /// if there's none within `reach`.
    pub fn open_sea_near(&self, p: V2, past: f32, reach: f32) -> V2 {
        let mut r = PX * 2.0;
        while r <= reach {
            let n = ((std::f32::consts::TAU * r / PX) as usize).max(16);
            let best = (0..n)
                .map(|k| {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    p.add(V2::new(a.cos() * r, a.sin() * r))
                })
                .map(|c| (self.inland(c), c))
                .filter(|(h, _)| *h <= -past)
                .min_by(|x, y| x.0.total_cmp(&y.0));
            if let Some((_, c)) = best {
                return c;
            }
            r += PX * 2.0;
        }
        p
    }

    /// The nearest spot on land from `p` with this much ground between it
    /// and the sea (walking the distance field up): a mark drawn in the
    /// shallows comes ashore.
    pub fn landward(&self, p: V2, inland: f32) -> V2 {
        let mut q = p;
        for _ in 0..400 {
            let d = self.inland(q);
            if d >= inland {
                return q;
            }
            let dir = self.toward_sea(q);
            if dir.len() < 0.5 {
                // No fall to follow (the open sea): head for the map's middle.
                let mid = V2::new(self.w as f32 * PX / 2.0, self.h as f32 * PX / 2.0);
                let to = mid.sub(q);
                let len = to.len().max(1.0);
                q = q.add(to.scale((PX * 2.0).min(len) / len));
                continue;
            }
            q = q.sub(dir.scale((inland - d).max(PX * 0.5).min(PX * 2.0)));
        }
        q
    }

    /// A spot on the shore nearest `p`, this far inland of the waterline
    /// (where a dock meets dry land): down the distance field in half
    /// steps until the line is crossed, then the crossing is bisected (the
    /// field is bilinear, so it's continuous along the way).
    pub fn shore_spot(&self, p: V2, inland: f32) -> V2 {
        let mut q = p;
        let mut d = self.inland(q);
        for _ in 0..400 {
            if (d - inland).abs() <= 0.5 {
                return q;
            }
            let dir = self.toward_sea(q);
            if dir.len() < 0.5 {
                return q;
            }
            let nq = q.add(dir.scale((d - inland).clamp(-PX, PX) * 0.5));
            let nd = self.inland(nq);
            if (nd - inland) * (d - inland) < 0.0 {
                let (mut lo, mut hi) = (q, nq);
                for _ in 0..30 {
                    let mid = lo.lerp(hi, 0.5);
                    let md = self.inland(mid);
                    if (md - inland).abs() <= 0.5 {
                        return mid;
                    }
                    if (md - inland) * (d - inland) < 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                return lo.lerp(hi, 0.5);
            }
            q = nq;
            d = nd;
        }
        q
    }

    /// The shoreline between two corners (metres), as line segments at
    /// `step` pixels (for the map's crisp line).
    pub fn shoreline_in(&self, a: V2, b: V2, step: usize) -> Vec<(V2, V2)> {
        let step = step.max(1);
        let (i0, i1) = (((a.x / PX).floor().max(0.0) as usize), ((b.x / PX).ceil().max(0.0) as usize).min(self.w));
        let (j0, j1) = (((a.y / PX).floor().max(0.0) as usize), ((b.y / PX).ceil().max(0.0) as usize).min(self.h));
        let mut out = Vec::new();
        let sea = |i: usize, j: usize| self.class[j * self.w + i] == Class::Sea as u8;
        let mut j = j0 - j0 % step;
        while j < j1 {
            let mut i = i0 - i0 % step;
            while i < i1 {
                let here = sea(i, j);
                if i + step < self.w && sea(i + step, j) != here {
                    let x = (i + step) as f32 * PX;
                    out.push((V2::new(x, j as f32 * PX), V2::new(x, (j + step).min(self.h) as f32 * PX)));
                }
                if j + step < self.h && sea(i, j + step) != here {
                    let y = (j + step) as f32 * PX;
                    out.push((V2::new(i as f32 * PX, y), V2::new((i + step).min(self.w) as f32 * PX, y)));
                }
                i += step;
            }
            j += step;
        }
        out
    }

    /// A land pixel picked by `k` (any number), as a spot: for scattering
    /// things over the land without searching the sea.
    pub fn land_spot(&self, k: u64) -> V2 {
        if self.land.is_empty() {
            return V2::new(self.extent() / 2.0, self.extent() / 2.0);
        }
        let i = self.land[(k % self.land.len() as u64) as usize] as usize;
        V2::new((i % self.w) as f32 * PX + PX / 2.0, (i / self.w) as f32 * PX + PX / 2.0)
    }

    /// How many pixels are land.
    pub fn land_pixels(&self) -> usize {
        self.land.len()
    }

    /// The leftmost land at this `y`, metres (the old `coast_x`); the map's
    /// width if none.
    pub fn west_shore_x(&self, y: f32) -> f32 {
        let j = ((y / PX).floor() as i64).clamp(0, self.h as i64 - 1);
        for i in 0..self.w {
            if self.class[j as usize * self.w + i] != Class::Sea as u8 {
                return i as f32 * PX;
            }
        }
        self.w as f32 * PX
    }

    /// The shoreline as line segments between neighbouring pixels of sea
    /// and land, at `step` pixels (for the map's crisp line): each is
    /// (from, to) in metres.
    pub fn shoreline(&self, step: usize) -> Vec<(V2, V2)> {
        let mut out = Vec::new();
        let step = step.max(1);
        for j in (0..self.h).step_by(step) {
            for i in (0..self.w).step_by(step) {
                let here = self.class[j * self.w + i] == Class::Sea as u8;
                if i + step < self.w && (self.class[j * self.w + i + step] == Class::Sea as u8) != here {
                    let x = (i + step) as f32 * PX;
                    out.push((V2::new(x, j as f32 * PX), V2::new(x, (j + step) as f32 * PX)));
                }
                if j + step < self.h && (self.class[(j + step) * self.w + i] == Class::Sea as u8) != here {
                    let y = (j + step) as f32 * PX;
                    out.push((V2::new(i as f32 * PX, y), V2::new((i + step) as f32 * PX, y)));
                }
            }
        }
        out
    }

    /// A course through the sea from `from` to `to`, as the centres of the
    /// pixels it passes (the ends snapped to the nearest open water): the
    /// cheapest walk over pixels at least `keep_off` metres from any
    /// shore (the whole map, since a coast may bulge far out of the way;
    /// it stops as soon as the far end is reached); pixels within
    /// `avoid.1` of `avoid.0` are closed (an island the atlas doesn't
    /// know). Empty when no way is found.
    pub fn sea_path(&self, from: V2, to: V2, keep_off: f32, avoid: Option<(V2, f32)>) -> Vec<V2> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;
        let px = |p: V2| ((p.x / PX).floor() as i64, (p.y / PX).floor() as i64);
        let pad = 40i64;
        let (a, b) = (px(from), px(to));
        let (x0, y0) = (0i64, 0i64);
        let (x1, y1) = (self.w as i64 - 1, self.h as i64 - 1);
        let (ww, hh) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
        let centre = |i: i64, j: i64| V2::new(i as f32 * PX + PX / 2.0, j as f32 * PX + PX / 2.0);
        let open = |i: i64, j: i64| -> bool {
            if i < x0 || j < y0 || i > x1 || j > y1 {
                return false;
            }
            let k = j as usize * self.w + i as usize;
            if self.sea[k] > -keep_off {
                return false;
            }
            match avoid {
                Some((c, r)) => centre(i, j).dist(c) > r,
                None => true,
            }
        };
        // Each end: the nearest open pixel.
        let snap = |p: (i64, i64)| -> Option<(i64, i64)> {
            for r in 0..=pad {
                for dj in -r..=r {
                    for di in -r..=r {
                        if (di.abs() == r || dj.abs() == r) && open(p.0 + di, p.1 + dj) {
                            return Some((p.0 + di, p.1 + dj));
                        }
                    }
                }
            }
            None
        };
        let (Some(s), Some(g)) = (snap(a), snap(b)) else { return Vec::new() };
        let idx = |i: i64, j: i64| (j - y0) as usize * ww + (i - x0) as usize;
        let mut dist = vec![f32::INFINITY; ww * hh];
        let mut parent = vec![u32::MAX; ww * hh];
        let mut heap = BinaryHeap::new();
        dist[idx(s.0, s.1)] = 0.0;
        heap.push(Reverse((0u32, idx(s.0, s.1) as u32)));
        let steps: [(i64, i64, f32); 8] = [(1, 0, 1.0), (-1, 0, 1.0), (0, 1, 1.0), (0, -1, 1.0), (1, 1, 1.4142), (1, -1, 1.4142), (-1, 1, 1.4142), (-1, -1, 1.4142)];
        let goal = idx(g.0, g.1);
        while let Some(Reverse((dbits, u))) = heap.pop() {
            let u = u as usize;
            let du = f32::from_bits(dbits);
            if du > dist[u] {
                continue;
            }
            if u == goal {
                break;
            }
            let (ui, uj) = ((u % ww) as i64 + x0, (u / ww) as i64 + y0);
            for &(di, dj, len) in &steps {
                let (vi, vj) = (ui + di, uj + dj);
                if !open(vi, vj) {
                    continue;
                }
                let v = idx(vi, vj);
                let nd = du + len;
                if nd < dist[v] {
                    dist[v] = nd;
                    parent[v] = u as u32;
                    heap.push(Reverse((nd.to_bits(), v as u32)));
                }
            }
        }
        if parent[goal] == u32::MAX && goal != idx(s.0, s.1) {
            return Vec::new();
        }
        let mut cells = vec![goal];
        while let Some(&last) = cells.last() {
            let p = parent[last];
            if p == u32::MAX {
                break;
            }
            cells.push(p as usize);
        }
        cells.reverse();
        let pts: Vec<V2> = cells.iter().map(|&c| centre((c % ww) as i64 + x0, (c / ww) as i64 + y0)).collect();
        // Drop the points along straight runs.
        let mut out: Vec<V2> = Vec::new();
        for (k, &p) in pts.iter().enumerate() {
            if k > 0 && k + 1 < pts.len() {
                let (a, b) = (pts[k - 1], pts[k + 1]);
                let (d1, d2) = (p.sub(a), b.sub(p));
                if (d1.x * d2.y - d1.y * d2.x).abs() < 1e-3 && d1.x * d2.x + d1.y * d2.y > 0.0 {
                    continue;
                }
            }
            out.push(p);
        }
        out
    }

    /// The nearest spot on dry, habitable land to `p` (not sea, not a river
    /// or lake, not the snow, not above `TOWN_HEIGHT`), within `within`
    /// pixels; `p` itself if there's none.
    pub fn dry_spot(&self, p: V2, within: i64) -> V2 {
        let (i0, j0) = ((p.x / PX).floor() as i64, (p.y / PX).floor() as i64);
        let dry = |i: i64, j: i64| self.idx(i, j).map(|k| self.class[k] != Class::Sea as u8 && self.class[k] != Class::Water as u8 && self.class[k] != Class::Snow as u8 && self.base[k] < TOWN_HEIGHT).unwrap_or(false);
        if dry(i0, j0) {
            return p;
        }
        for r in 1..=within {
            for dj in -r..=r {
                for di in -r..=r {
                    if (di.abs() == r || dj.abs() == r) && dry(i0 + di, j0 + dj) {
                        return V2::new((i0 + di) as f32 * PX + PX / 2.0, (j0 + dj) as f32 * PX + PX / 2.0);
                    }
                }
            }
        }
        p
    }

    /// The heart of the biggest snowfield: the snow pixel farthest from any
    /// other ground (a chamfer distance over the snow), and that distance
    /// in pixels. The map's middle, 0, with no snow at all.
    pub fn snow_heart(&self) -> (V2, f32) {
        let n = self.w * self.h;
        let mut d = vec![1e6f32; n];
        let mut any = false;
        for i in 0..n {
            if self.class[i] != Class::Snow as u8 {
                d[i] = 0.0;
            } else {
                any = true;
            }
        }
        if !any {
            return (V2::new(self.w as f32 * PX / 2.0, self.h as f32 * PX / 2.0), 0.0);
        }
        chamfer(&mut d, self.w, self.h);
        let (mut best, mut at) = (0.0f32, 0usize);
        for i in 0..n {
            if d[i] > best {
                best = d[i];
                at = i;
            }
        }
        (V2::new((at % self.w) as f32 * PX + PX / 2.0, (at / self.w) as f32 * PX + PX / 2.0), best)
    }

    /// How many pixels are of each class (for the tests and `headless`).
    pub fn census(&self) -> [usize; 9] {
        let mut out = [0usize; 9];
        for &c in &self.class {
            out[c as usize] += 1;
        }
        out
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A chamfer distance transform in place: each cell's distance (in pixels,
/// roughly Euclidean) to the nearest zero cell.
fn chamfer(d: &mut [f32], w: usize, h: usize) {
    const A: f32 = 1.0;
    const B: f32 = 1.4142;
    for j in 0..h {
        for i in 0..w {
            let k = j * w + i;
            let mut v = d[k];
            if i > 0 {
                v = v.min(d[k - 1] + A);
            }
            if j > 0 {
                v = v.min(d[k - w] + A);
                if i > 0 {
                    v = v.min(d[k - w - 1] + B);
                }
                if i + 1 < w {
                    v = v.min(d[k - w + 1] + B);
                }
            }
            d[k] = v;
        }
    }
    for j in (0..h).rev() {
        for i in (0..w).rev() {
            let k = j * w + i;
            let mut v = d[k];
            if i + 1 < w {
                v = v.min(d[k + 1] + A);
            }
            if j + 1 < h {
                v = v.min(d[k + w] + A);
                if i + 1 < w {
                    v = v.min(d[k + w + 1] + B);
                }
                if i > 0 {
                    v = v.min(d[k + w - 1] + B);
                }
            }
            d[k] = v;
        }
    }
}

/// A box blur of radius `r` pixels, in place (separable, two passes).
fn blur(v: &mut [f32], w: usize, h: usize, r: usize) {
    let mut tmp = vec![0.0f32; v.len()];
    let span = (2 * r + 1) as f32;
    // Rows.
    for j in 0..h {
        let row = &v[j * w..(j + 1) * w];
        let mut acc: f32 = 0.0;
        for i in 0..=r.min(w - 1) {
            acc += row[i];
        }
        for i in 0..w {
            // The window [i - r, i + r], clamped: keep the sum over the
            // edge by repeating the edge value.
            let lo = i as i64 - r as i64;
            let hi = i + r;
            let mut sum = acc;
            if lo < 0 {
                sum += row[0] * (-lo) as f32;
            }
            if hi >= w {
                sum += row[w - 1] * (hi - (w - 1)) as f32;
            }
            tmp[j * w + i] = sum / span;
            // Slide.
            if hi + 1 < w {
                acc += row[hi + 1];
            }
            if lo >= 0 {
                acc -= row[lo as usize];
            }
        }
    }
    // Columns.
    for i in 0..w {
        let mut acc: f32 = 0.0;
        for j in 0..=r.min(h - 1) {
            acc += tmp[j * w + i];
        }
        for j in 0..h {
            let lo = j as i64 - r as i64;
            let hi = j + r;
            let mut sum = acc;
            if lo < 0 {
                sum += tmp[i] * (-lo) as f32;
            }
            if hi >= h {
                sum += tmp[(h - 1) * w + i] * (hi - (h - 1)) as f32;
            }
            v[j * w + i] = sum / span;
            if hi + 1 < h {
                acc += tmp[(hi + 1) * w + i];
            }
            if lo >= 0 {
                acc -= tmp[lo as usize * w + i];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fallback_island_is_land_in_a_sea() {
        let a = Atlas::fallback();
        let c = V2::new(100.0 * PX, 100.0 * PX);
        assert!(a.is_land(c) && a.inland(c) > 60.0 * PX * 0.9);
        assert!(!a.is_land(V2::new(5.0 * PX, 5.0 * PX)));
        assert!(a.base_height(c) > 0.5 && a.base_height(V2::new(5.0 * PX, 5.0 * PX)) < 0.0);
        assert_eq!(a.class_at(c), Class::Plains);
        assert_eq!(a.class_at(V2::new(31.0 * PX, 100.0 * PX)), Class::Coast);
        let shore = a.seaward(c, 300.0);
        assert!(a.inland(shore) <= -300.0 + 1.0);
    }
}
