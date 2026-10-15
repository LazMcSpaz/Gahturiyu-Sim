//! The land under each of Laz's marks as the game makes it, for the place
//! builder (`docs/place-builder.md`): `headless grounds <dir> [seed]` writes
//! `<dir>/<key>.json` per mark and `<dir>/index.json` (mark id → key), and
//! the builder draws a place over it, so what Laz lays out sits on the land
//! he'll walk on.
//!
//! Each file holds the place's own grid (`CELL` metres a cell, its area by
//! the mark's kind, `AREAS`, centred on the town's spot, `worldgen::town_spot`)
//! and a ring as many cells round it, `AROUND_SCALE` times coarser: a letter of
//! ground a cell, the height a cell (decimetres from the centre's, as
//! little-endian `i16`s in base64), and in the place the trees and bushes the
//! land itself grows there (`plants.rs`, at full density, as no plan says
//! otherwise). The land's broad shape is Laz's map and the same in every
//! world; its small relief, scrub patches and woods follow the seed, which
//! the file names.

use serde::{Deserialize, Serialize};

use super::atlas::{self, Class};
use super::geo::V2;
use super::plants;
use super::terrain::Ground as G;
use super::world::World;

/// The place builder's cells, metres.
pub const CELL: f32 = 2.0;
/// The ring round a place has as many cells as the place, each this many
/// times bigger (so it reaches this many times as far).
pub const AROUND_SCALE: f32 = 4.0;
/// A place's area by its mark's kind, metres across and down (Laz, 14 Oct):
/// a hamlet's yards to a capital's quarters. Anything else marked (a ruin,
/// a lookout, a mine...) gets `OTHER_AREA`.
pub const AREAS: &[(&str, f32, f32)] = &[
    ("hamlet", 120.0, 90.0),
    ("outpost", 160.0, 120.0),
    ("village", 200.0, 150.0),
    ("town", 320.0, 240.0),
    ("fort", 320.0, 240.0),
    ("city", 450.0, 340.0),
    ("capital", 600.0, 450.0),
];
pub const OTHER_AREA: (f32, f32) = (160.0, 120.0);

/// The place's grid for a mark of this kind: cells across and down.
pub fn area_of(kind: &str) -> (usize, usize) {
    let (w, h) = AREAS.iter().find(|a| a.0 == kind).map(|a| (a.1, a.2)).unwrap_or(OTHER_AREA);
    ((w / CELL).round() as usize, (h / CELL).round() as usize)
}

/// A mark that isn't a town (`assets/world/landmarks.ron`).
#[derive(Deserialize, Clone, Debug)]
pub struct Landmark {
    pub key: String,
    #[serde(default)]
    pub id: String,
    pub label: String,
    pub px: (u32, u32),
    pub kind: String,
    #[serde(default)]
    pub holder: String,
    #[serde(default)]
    pub notes: String,
}

impl Landmark {
    pub fn pos(&self) -> V2 {
        V2::new(self.px.0 as f32 * atlas::PX + atlas::PX / 2.0, self.px.1 as f32 * atlas::PX + atlas::PX / 2.0)
    }
}

pub fn landmarks() -> &'static [Landmark] {
    static MARKS: std::sync::OnceLock<Vec<Landmark>> = std::sync::OnceLock::new();
    MARKS.get_or_init(|| ron::from_str(include_str!("../../assets/world/landmarks.ron")).unwrap_or_else(|e| panic!("assets/world/landmarks.ron: {e}")))
}

/// Every mark the builder can open: key, mark id, label, kind (the page's
/// word), and the centre a place there is laid round.
pub fn marks() -> Vec<(String, String, String, String, V2)> {
    let mut out: Vec<(String, String, String, String, V2)> = atlas::town_marks().iter().map(|m| (m.key(), m.id.clone(), m.label.clone(), format!("{:?}", m.kind).to_lowercase(), super::worldgen::town_spot(m))).collect();
    out.extend(landmarks().iter().map(|l| (l.key.clone(), l.id.clone(), l.label.clone(), l.kind.clone(), l.pos())));
    out
}

/// One grid of the land: a ground letter and a height a cell.
#[derive(Serialize, Clone, Debug)]
pub struct Grid {
    pub cell: f32,
    pub size: (usize, usize),
    /// The cell at the centre (the centre is its top-left corner, as in the
    /// place file).
    pub origin: (usize, usize),
    /// A letter a cell, row-major from the top left: `g` grass, `s` scrub,
    /// `n` sand, `r` rock, `d` dirt, `m` mud, `v` gravel, `w` snow, `R` road,
    /// `q` river or lake, `~` sea; in the ring, `f` for wooded grass or scrub.
    pub ground: String,
    /// Decimetres above the centre's height a cell, `i16` little-endian, base64.
    pub height: String,
}

/// The land at a mark.
#[derive(Serialize, Clone, Debug)]
pub struct LandAt {
    pub format: u32,
    pub key: String,
    pub id: String,
    pub label: String,
    pub kind: String,
    /// The world seed the small relief, scrub and woods are from.
    pub seed: u64,
    /// The centre in world metres, and its height (m).
    pub centre: (f32, f32),
    pub h0: f32,
    pub place: Grid,
    pub around: Grid,
    /// The land's own trees in the place: x, y from the centre (m), kind
    /// (0 broadleaf, 1 conifer), scale.
    pub trees: Vec<(f32, f32, u8, f32)>,
    /// Its bushes: x, y, scale.
    pub bushes: Vec<(f32, f32, f32)>,
    /// Rocks already on the land there (the map's own edits): x, y, size.
    pub rocks: Vec<(f32, f32, f32)>,
}

/// The ground letter at a spot (see `Grid::ground`).
pub fn letter(w: &World, p: V2) -> char {
    let t = &w.terrain;
    if t.is_sea(p) {
        return '~';
    }
    if atlas::atlas().class_at(p) == Class::Water {
        return 'q';
    }
    match t.ground(p) {
        G::Road => 'R',
        G::Grass => 'g',
        G::Scrub => 's',
        G::Rock => 'r',
        G::Sand => 'n',
        G::Dirt => 'd',
        G::Mud => 'm',
        G::Gravel => 'v',
        G::Snow => 'w',
    }
}

fn grid(w: &World, c: V2, h0: f32, size: (usize, usize), cell: f32, woods: bool) -> Grid {
    let (gw, gh) = size;
    let origin = (gw / 2, gh / 2);
    let mut ground = String::with_capacity(gw * gh);
    let mut height: Vec<u8> = Vec::with_capacity(gw * gh * 2);
    for j in 0..gh {
        for i in 0..gw {
            let p = c.add(V2::new((i as f32 - origin.0 as f32 + 0.5) * cell, (j as f32 - origin.1 as f32 + 0.5) * cell));
            let mut l = letter(w, p);
            if woods && matches!(l, 'g' | 's') && plants::wood(w, p) > 0.45 && plants::spot(w, p).is_some_and(|s| s.h < plants::TREE_LINE) {
                l = 'f';
            }
            ground.push(l);
            let dm = ((w.terrain.height(p) - h0) * 10.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            height.extend_from_slice(&dm.to_le_bytes());
        }
    }
    Grid { cell, size, origin, ground, height: base64(&height) }
}

/// The land at one mark's centre.
pub fn land_at(w: &World, key: &str, id: &str, label: &str, kind: &str, c: V2) -> LandAt {
    let h0 = w.terrain.height(c);
    let size = area_of(kind);
    // (The box the grid covers: from the origin cell's corner, as the place file reads it.)
    let (x0, y0) = (c.x - (size.0 / 2) as f32 * CELL, c.y - (size.1 / 2) as f32 * CELL);
    let (x1, y1) = (x0 + size.0 as f32 * CELL, y0 + size.1 as f32 * CELL);
    let r1 = |v: f32| (v * 10.0).round() / 10.0;
    let r2 = |v: f32| (v * 100.0).round() / 100.0;
    let trees = plants::land_trees_in(w, x0, y0, x1, y1).into_iter().map(|(p, k, s)| (r1(p.x - c.x), r1(p.y - c.y), k, r2(s))).collect();
    let bushes = plants::land_bushes_in(w, x0, y0, x1, y1).into_iter().map(|(p, s)| (r1(p.x - c.x), r1(p.y - c.y), r2(s))).collect();
    let rocks = w.terrain.edits.rocks.iter().chain(w.terrain.authored.rocks.iter()).filter(|r| r.pos.x >= x0 && r.pos.x < x1 && r.pos.y >= y0 && r.pos.y < y1).map(|r| (r1(r.pos.x - c.x), r1(r.pos.y - c.y), r2(r.size))).collect();
    LandAt {
        format: 1,
        key: key.to_string(),
        id: id.to_string(),
        label: label.to_string(),
        kind: kind.to_string(),
        seed: w.seed,
        centre: (r1(c.x), r1(c.y)),
        h0: r1(h0),
        place: grid(w, c, h0, size, CELL, false),
        around: grid(w, c, h0, size, CELL * AROUND_SCALE, true),
        trees,
        bushes,
        rocks,
    }
}

/// The world the files are read from: the seed's, with every town laid out
/// by hand standing where it will (its buildings kept), but none of the
/// places' own ground or height on the land, so the builder shows the land
/// as it is before Laz's paint.
pub fn world_for(seed: u64) -> World {
    let mut plans = super::plan::Plans::load();
    for p in plans.towns.values_mut() {
        p.land = None;
    }
    super::worldgen::generate_authored_with(seed, &super::roster::Rosters::load(), &plans, super::mapedit::MapEdits::world())
}

/// Write every mark's file and the index into `dir`. Returns how many.
pub fn write_all(w: &World, dir: &std::path::Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    let mut index = serde_json::Map::new();
    let all = marks();
    // (A mark gone or renamed leaves no stale file behind.)
    for e in std::fs::read_dir(dir)?.flatten() {
        if e.path().extension().is_some_and(|x| x == "json") {
            std::fs::remove_file(e.path())?;
        }
    }
    for (key, id, label, kind, c) in &all {
        let land = land_at(w, key, id, label, kind, *c);
        std::fs::write(dir.join(format!("{key}.json")), serde_json::to_string(&land).expect("it writes"))?;
        index.insert(id.clone(), serde_json::json!({ "key": key, "label": label, "kind": kind }));
    }
    std::fs::write(dir.join("index.json"), serde_json::to_string_pretty(&serde_json::Value::Object(index)).expect("it writes"))?;
    Ok(all.len())
}

/// Standard base64 (with padding).
pub fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        out.push(A[(n >> 18) as usize & 63] as char);
        out.push(A[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 { A[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if c.len() > 2 { A[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Everything the map site (`tools/map/`, served by GitHub Pages) needs
/// beside its page, refreshed in one go from the repo at `root`: the
/// catalogue, every mark's land (`grounds/`), and copies of Laz's marks and
/// places (`data/`). Rosters aren't copied (the page keeps its own), but each
/// is read, so a bad one is said here. The same files every time it's run on
/// the same repo. Returns what it did, and any problem found, line by line.
pub fn write_site(root: &std::path::Path, seed: u64) -> std::io::Result<Vec<String>> {
    let site = root.join("tools/map");
    let mut said = Vec::new();
    std::fs::write(site.join("catalogue.json"), super::catalogue::catalogue_json())?;
    std::fs::write(site.join("signs.json"), super::catalogue::signs_json())?;
    said.push(format!("catalogue.json: {} things", super::catalogue::catalogue().len()));
    let w = world_for(seed);
    let n = write_all(&w, &site.join("grounds"))?;
    said.push(format!("grounds/: {n} marks' land (world {seed})"));
    // Laz's own files, as they are: the marks, and a place a file.
    let data = site.join("data");
    let places = data.join("places");
    std::fs::create_dir_all(&places)?;
    std::fs::copy(root.join("assets/world/marks-export.json"), data.join("marks-export.json"))?;
    for e in std::fs::read_dir(&places)?.flatten() {
        if e.path().extension().is_some_and(|x| x == "json") {
            std::fs::remove_file(e.path())?;
        }
    }
    let mut copied = 0;
    if let Ok(dir) = std::fs::read_dir(root.join("assets/world/places")) {
        let mut files: Vec<std::path::PathBuf> = dir.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
        files.sort();
        for f in files {
            std::fs::copy(&f, places.join(f.file_name().expect("a file")))?;
            copied += 1;
        }
    }
    said.push(format!("data/: marks-export.json and {copied} places"));
    // Every roster reads, and is for a town on the map.
    let keys: Vec<String> = marks().into_iter().map(|m| m.0).collect();
    if let Ok(dir) = std::fs::read_dir(root.join("assets/world/people")) {
        let mut files: Vec<std::path::PathBuf> = dir.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "ron")).collect();
        files.sort();
        for f in files {
            match super::roster::Roster::load(&f) {
                Ok(r) if r.town != "wanderers" && !keys.contains(&r.town) => said.push(format!("PROBLEM {}: town \"{}\" isn't a mark's key", f.display(), r.town)),
                Ok(r) => said.push(format!("roster {}: {} characters, {} crowd rows", r.town, r.characters.len(), r.crowd_rows.len())),
                Err(e) => said.push(format!("PROBLEM {e}")),
            }
        }
    }
    Ok(said)
}

// ---- The land everywhere, as pictures (the map site's zoomed-in view) -------

/// Metres a pixel of the land tiles, and pixels a tile side.
pub const TILE_M: f32 = 8.0;
pub const TILE_PX: u32 = 256;

/// The colours the site draws the land in (the page's `NAT_COL`), by ground letter.
fn colour_of(l: char) -> [u8; 3] {
    match l {
        'g' => [0x6b, 0x8a, 0x46],
        's' => [0x8a, 0x8f, 0x4e],
        'n' => [0xc9, 0xb9, 0x8a],
        'r' => [0x7b, 0x77, 0x72],
        'd' => [0x8c, 0x73, 0x52],
        'm' => [0x5f, 0x4d, 0x3a],
        'v' => [0x8f, 0x8d, 0x85],
        'w' => [0xe9, 0xec, 0xef],
        'R' => [0x8c, 0x6a, 0x45],
        'q' => [0x5b, 0x84, 0xb8],
        '~' => [0x3f, 0x64, 0x96],
        'f' => [0x4f, 0x6f, 0x38],
        _ => [0x6b, 0x8a, 0x46],
    }
}

/// One tile of the land as the game makes it (`TILE_PX` square at `TILE_M`
/// a pixel; tile (i, j) starts at (i, j) × the tile's metres), shaded from
/// the north-west as the site shades a place; None if it's all open sea.
pub fn land_tile(w: &World, i: u32, j: u32) -> Option<image::RgbImage> {
    let n = TILE_PX as usize;
    let side = TILE_PX as f32 * TILE_M;
    let (x0, y0) = (i as f32 * side, j as f32 * side);
    let a = atlas::atlas();
    // Quick look: open sea all over (every corner and the middle well out).
    let probe = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0), (0.5, 0.5), (0.5, 0.0), (0.0, 0.5), (1.0, 0.5), (0.5, 1.0)];
    if probe.iter().all(|(u, v)| a.inland(V2::new(x0 + u * side, y0 + v * side)) < -side) && super::geo::isle_inland(V2::new(x0 + side / 2.0, y0 + side / 2.0)) < -side {
        return None;
    }
    // Heights with a pixel's border, for the shading.
    let at = |px: i64, py: i64| V2::new(x0 + (px as f32 + 0.5) * TILE_M, y0 + (py as f32 + 0.5) * TILE_M);
    let hn = n + 2;
    let mut h = vec![0.0f32; hn * hn];
    for py in 0..hn {
        for px in 0..hn {
            h[py * hn + px] = w.terrain.surface(at(px as i64 - 1, py as i64 - 1));
        }
    }
    let mut img = image::RgbImage::new(TILE_PX, TILE_PX);
    let mut any_land = false;
    for py in 0..n {
        for px in 0..n {
            let p = at(px as i64, py as i64);
            let mut l = letter(w, p);
            if matches!(l, 'g' | 's') && plants::wood(w, p) > 0.45 && plants::spot(w, p).is_some_and(|s| s.h < plants::TREE_LINE) {
                l = 'f';
            }
            any_land |= l != '~';
            let k = (py + 1) * hn + px + 1;
            let d = ((h[k + 1] - h[k - 1]) + (h[k + hn] - h[k - hn])) / (2.0 * TILE_M);
            // (In twentieths: the picture compresses several times smaller, and looks the same.)
            let f = if l == '~' { 1.0 } else { ((1.0 - d * 1.2).clamp(0.6, 1.35) * 20.0).round() / 20.0 };
            let c = colour_of(l);
            img.put_pixel(px as u32, py as u32, image::Rgb([(c[0] as f32 * f).min(255.0) as u8, (c[1] as f32 * f).min(255.0) as u8, (c[2] as f32 * f).min(255.0) as u8]));
        }
    }
    any_land.then_some(img)
}

/// Every tile with land in it, written to `dir/<i>_<j>.png`, and
/// `dir/index.json` (the metres a pixel and a tile's pixels, and the tiles
/// there are). Worked on all the machine's threads. Returns the count.
pub fn write_tiles(w: &World, dir: &std::path::Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    for e in std::fs::read_dir(dir)?.flatten() {
        if e.path().extension().is_some_and(|x| x == "png" || x == "json") {
            std::fs::remove_file(e.path())?;
        }
    }
    let side = TILE_PX as f32 * TILE_M;
    let (nx, ny) = ((super::geo::WORLD_SIZE / side).ceil() as u32, ((1932.0 * atlas::PX) / side).ceil() as u32);
    let cells: Vec<(u32, u32)> = (0..ny).flat_map(|j| (0..nx).map(move |i| (i, j))).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 16);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let done: std::sync::Mutex<Vec<(u32, u32)>> = std::sync::Mutex::new(Vec::new());
    let failed: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&(i, j)) = cells.get(k) else { break };
                if let Some(img) = land_tile(w, i, j) {
                    match save_png(&img, &dir.join(format!("{i}_{j}.png"))) {
                        Ok(()) => done.lock().unwrap().push((i, j)),
                        Err(e) => *failed.lock().unwrap() = Some(e.to_string()),
                    }
                }
            });
        }
    });
    if let Some(e) = failed.into_inner().unwrap() {
        return Err(std::io::Error::other(e));
    }
    let mut tiles = done.into_inner().unwrap();
    tiles.sort();
    let index = serde_json::json!({ "metres_a_pixel": TILE_M, "tile_px": TILE_PX, "seed": w.seed, "tiles": tiles });
    std::fs::write(dir.join("index.json"), serde_json::to_string(&index).expect("it writes"))?;
    Ok(tiles.len())
}

/// A picture as a PNG squeezed as small as the encoder goes (the site's
/// tiles go up a slow line).
fn save_png(img: &image::RgbImage, path: &std::path::Path) -> Result<(), String> {
    use image::ImageEncoder;
    let f = std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?);
    let enc = image::codecs::png::PngEncoder::new_with_quality(f, image::codecs::png::CompressionType::Best, image::codecs::png::FilterType::Adaptive);
    enc.write_image(img.as_raw(), img.width(), img.height(), image::ExtendedColorType::Rgb8).map_err(|e| e.to_string())
}

/// The mark whose place holds a spot (its area by kind round its centre;
/// the nearest centre if two overlap): key, label, kind, centre.
pub fn place_at(p: V2) -> Option<(String, String, String, V2)> {
    marks()
        .into_iter()
        .filter(|(_, _, _, kind, c)| {
            let (gw, gh) = area_of(kind);
            let (x0, y0) = (c.x - (gw / 2) as f32 * CELL, c.y - (gh / 2) as f32 * CELL);
            p.x >= x0 && p.y >= y0 && p.x < x0 + gw as f32 * CELL && p.y < y0 + gh as f32 * CELL
        })
        .min_by(|a, b| a.4.dist(p).total_cmp(&b.4.dist(p)))
        .map(|(key, _, label, kind, c)| (key, label, kind, c))
}
