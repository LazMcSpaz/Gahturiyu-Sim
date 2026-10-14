//! The map site's close-up pictures: the game's own land photographed from
//! straight above, a tile at a time (Laz, 14 Oct: "generate the map in game
//! with no cities and such, then use a high resolution image of that").
//! Drawing only: nothing here changes the world.
//!
//! `GAHT_MAPRENDER=<dir>` runs the window in this mode and quits when done:
//! an orthographic camera looking straight down, a fixed sun from the
//! north-west, no fog, no towns, people, animals or panels; for each tile a
//! ground mesh one cell every `mpp × 1.5` metres (the game's own ground
//! colours, sea, roads and rocks) and the land's own trees and bushes
//! (`sim/plants.rs`, the game's tree meshes), then a screenshot.
//!
//! - `GAHT_MAPRENDER_LEVELS=land,marks` (the default, both): `land` covers the
//!   whole map at `GAHT_MAPRENDER_M` metres a pixel (4), skipping open sea;
//!   `marks` is one tile centred on every mark at `GAHT_MAPRENDER_NEAR` (1).
//! - `GAHT_MAPRENDER_AT=x,y` one tile there instead (at `GAHT_MAPRENDER_M`).
//! - `GAHT_MAPRENDER_LIMIT=n` stops after n tiles (to try it out).
//!
//! A mark's tile leaves its place's area (`grounds::area_of`) without plants:
//! in a built place only the trees Laz stamps grow, and the site draws them.
//!
//! Writes `<dir>/<name>.png` and `<dir>/index.json`: per tile its name, level,
//! metres a pixel and the world box it shows (x0, y0, x1, y1, north up).

use std::path::PathBuf;

use bevy::camera::ScalingMode;
use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PrimaryWindow;
use gahturiyu_sim::sim::{atlas, geo::V2, grounds, plants, rng::Rng, terrain::Terrain};

use super::app::{Game, MainCamera};
use super::cam::to3;
use super::foliage::Foliage;
use super::light::{Moon, Sun};
use super::mesh::Builder;
use super::palette;
use super::scene::Mats;

/// Frames to wait after a tile is laid out before it's photographed (meshes
/// uploaded, shadows drawn).
const SETTLE_FRAMES: u32 = 6;
/// The light: the sun from the north-west, this high, as the site shades.
const SUN_FROM: Vec3 = Vec3::new(0.55, -1.0, 0.5);

#[derive(Clone, Debug)]
struct Tile {
    name: String,
    level: &'static str,
    centre: V2,
    mpp: f32,
    /// A mark's place: no plants drawn inside (only what Laz stamps grows
    /// in a built place, and the site draws those itself).
    hole: Option<(f32, f32, f32, f32)>,
}

#[derive(Resource)]
pub struct MapRender {
    dir: PathBuf,
    levels: Vec<String>,
    land_mpp: f32,
    near_mpp: f32,
    at: Option<V2>,
    limit: Option<usize>,
    tiles: Option<Vec<Tile>>,
    next: usize,
    frame: u32,
    /// The tile laid out now, and the picture being written.
    laid: Option<(Tile, f32, f32)>,
    shot: Option<(PathBuf, u32)>,
    entities: Vec<Entity>,
    index: Vec<serde_json::Value>,
}

/// Whether the window is drawing the game as usual (not the map render):
/// the run condition for the systems the render replaces.
pub fn off(mr: Option<Res<MapRender>>) -> bool {
    mr.is_none()
}

impl MapRender {
    pub fn from_env() -> Option<MapRender> {
        let dir = PathBuf::from(std::env::var("GAHT_MAPRENDER").ok()?);
        let num = |k: &str, d: f32| std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d);
        let at = std::env::var("GAHT_MAPRENDER_AT").ok().and_then(|v| {
            let mut it = v.split(',').map(|s| s.trim().parse::<f32>());
            Some(V2::new(it.next()?.ok()?, it.next()?.ok()?))
        });
        Some(MapRender {
            dir,
            levels: std::env::var("GAHT_MAPRENDER_LEVELS").unwrap_or_else(|_| "land,marks".into()).split(',').map(|s| s.trim().to_string()).collect(),
            land_mpp: num("GAHT_MAPRENDER_M", 4.0),
            near_mpp: num("GAHT_MAPRENDER_NEAR", 1.0),
            at,
            limit: std::env::var("GAHT_MAPRENDER_LIMIT").ok().and_then(|v| v.parse().ok()),
            tiles: None,
            next: 0,
            frame: 0,
            laid: None,
            shot: None,
            entities: Vec::new(),
            index: Vec::new(),
        })
    }

    /// The tiles to take, for a picture `pw` × `ph` pixels.
    fn plan(&self, pw: f32, ph: f32) -> Vec<Tile> {
        let mut out = Vec::new();
        if let Some(c) = self.at {
            out.push(Tile { name: format!("at_{}_{}", c.x as i64, c.y as i64), level: "at", centre: c, mpp: self.land_mpp, hole: None });
            return out;
        }
        let a = atlas::atlas();
        if self.levels.iter().any(|l| l == "land") {
            let (tw, th) = (pw * self.land_mpp, ph * self.land_mpp);
            let (mw, mh) = (a.w as f32 * atlas::PX, a.h as f32 * atlas::PX);
            let (nx, ny) = ((mw / tw).ceil() as i32, (mh / th).ceil() as i32);
            for j in 0..ny {
                for i in 0..nx {
                    let c = V2::new((i as f32 + 0.5) * tw, (j as f32 + 0.5) * th);
                    // (Open sea all over: the site draws the map's sea.)
                    let reach = tw.max(th);
                    let probe = (0..5).flat_map(|v| (0..5).map(move |u| (u as f32 / 4.0 - 0.5, v as f32 / 4.0 - 0.5)));
                    if probe.clone().all(|(u, v)| a.inland(V2::new(c.x + u * tw, c.y + v * th)) < -reach * 0.25) {
                        continue;
                    }
                    out.push(Tile { name: format!("land_{i}_{j}"), level: "land", centre: c, mpp: self.land_mpp, hole: None });
                }
            }
        }
        if self.levels.iter().any(|l| l == "marks") {
            for (key, _, _, kind, c) in grounds::marks() {
                // (The place's grid as the builder has it: from its origin cell's corner.)
                let (gw, gh) = grounds::area_of(&kind);
                let (px0, py0) = (c.x - (gw / 2) as f32 * grounds::CELL, c.y - (gh / 2) as f32 * grounds::CELL);
                let hole = Some((px0, py0, px0 + gw as f32 * grounds::CELL, py0 + gh as f32 * grounds::CELL));
                out.push(Tile { name: format!("mark_{key}"), level: "marks", centre: c, mpp: self.near_mpp, hole });
            }
        }
        if let Some(n) = self.limit {
            out.truncate(n);
        }
        out
    }
}

/// The ground under a tile, as the window draws the land (`scene::ground_patch`,
/// a cell every `cell` metres): land coloured by the game's palette, sea flat
/// at the waterline, the roads laid over it.
fn ground(t: &Terrain, roads: &[Vec<V2>], x0: f32, y0: f32, x1: f32, y1: f32, cell: f32) -> Builder {
    let (nx, ny) = (((x1 - x0) / cell).ceil() as usize, ((y1 - y0) / cell).ceil() as usize);
    let vert = |p: V2| -> (Vec3, Vec3, [f32; 4]) {
        let h = t.height(p);
        if h >= 0.0 {
            let h = h.max(0.3);
            let (nx, ny, nz) = t.normal(p, cell.max(4.0));
            (to3(p, h), Vec3::new(nx, ny, nz), palette::lin(palette::ground(t, p, h, ny)))
        } else {
            (to3(p, 0.0), Vec3::Y, palette::lin(palette::sea(p)))
        }
    };
    // Each corner worked out once, then the quads.
    let mut vs = Vec::with_capacity((nx + 1) * (ny + 1));
    for j in 0..=ny {
        for i in 0..=nx {
            vs.push(vert(V2::new(x0 + i as f32 * cell, y0 + j as f32 * cell)));
        }
    }
    let mut b = Builder::new();
    for j in 0..ny {
        for i in 0..nx {
            let k = [j * (nx + 1) + i, j * (nx + 1) + i + 1, (j + 1) * (nx + 1) + i + 1, (j + 1) * (nx + 1) + i];
            b.quad_lin(k.map(|k| vs[k].0), k.map(|k| vs[k].1), k.map(|k| vs[k].2));
        }
    }
    // The roads: a band laid on the ground, a piece every few metres.
    let width = 4.0f32.max(cell * 1.5);
    let inside = |p: V2| p.x > x0 - 50.0 && p.x < x1 + 50.0 && p.y > y0 - 50.0 && p.y < y1 + 50.0;
    for road in roads {
        for seg in road.windows(2) {
            let (a, c) = (seg[0], seg[1]);
            if !inside(a) && !inside(c) {
                continue;
            }
            let len = a.dist(c);
            if len < 0.01 {
                continue;
            }
            let dir = c.sub(a).scale(1.0 / len);
            let side = V2::new(-dir.y, dir.x).scale(width / 2.0);
            let n = (len / (cell * 2.0).max(3.0)).ceil().max(1.0) as usize;
            for s in 0..n {
                let (p, q) = (a.lerp(c, s as f32 / n as f32), a.lerp(c, (s + 1) as f32 / n as f32));
                let up = |v: V2| to3(v, t.surface(v) + 0.25);
                b.quad([up(p.sub(side)), up(p.add(side)), up(q.add(side)), up(q.sub(side))], Vec3::Y, palette::ROAD);
            }
        }
    }
    b
}

/// The map render, once a frame: lay the next tile out, wait, photograph it,
/// go on; write the index and quit at the end.
#[allow(clippy::too_many_arguments)]
pub fn step(
    mut commands: Commands,
    mr: Option<ResMut<MapRender>>,
    game: Res<Game>,
    foliage: Res<Foliage>,
    mats: Res<Mats>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut cam: Query<(Entity, &mut Camera, &mut Transform, &mut Projection), (With<MainCamera>, Without<Sun>, Without<Moon>)>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), (With<Sun>, Without<Moon>, Without<MainCamera>)>,
    mut moon: Query<&mut DirectionalLight, (With<Moon>, Without<Sun>)>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(mut mr) = mr else { return };
    let mr = &mut *mr;
    mr.frame += 1;
    let Ok(win) = window.single() else { return };
    let (pw, ph) = (win.physical_width() as f32, win.physical_height() as f32);
    if mr.tiles.is_none() {
        // (The window settles to its size over the first frames.)
        if mr.frame < 3 {
            return;
        }
        let tiles = mr.plan(pw, ph);
        println!("map render: {} tiles of {pw} x {ph} px to {}", tiles.len(), mr.dir.display());
        let _ = std::fs::create_dir_all(&mr.dir);
        mr.tiles = Some(tiles);
    }
    let Ok((cam_e, mut c, mut tf, mut proj)) = cam.single_mut() else { return };
    c.is_active = true;
    // The light: a fixed sun, no moon, the daytime sky.
    if let Ok((mut l, mut st)) = sun.single_mut() {
        l.illuminance = 10_000.0;
        *st = Transform::default().looking_to(SUN_FROM, Vec3::Y);
    }
    if let Ok(mut m) = moon.single_mut() {
        m.illuminance = 0.0;
    }
    ambient.brightness = 350.0;
    clear.0 = palette::bevy(palette::sea(V2::new(0.0, 0.0)));

    // A picture being written: wait for it, then on to the next tile.
    if let Some((path, at)) = mr.shot.clone() {
        let done = std::fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false);
        if !(done && mr.frame > at + 2) && mr.frame < at + 600 {
            return;
        }
        if let Some((t, w, h)) = mr.laid.take() {
            mr.index.push(serde_json::json!({
                "name": t.name, "level": t.level, "metres_a_pixel": t.mpp, "place": t.hole.map(|(a, b, c, d)| [a, b, c, d]),
                "x0": t.centre.x - w / 2.0, "y0": t.centre.y - h / 2.0, "x1": t.centre.x + w / 2.0, "y1": t.centre.y + h / 2.0,
                "w_px": pw, "h_px": ph,
            }));
            println!("map render: {} ({}/{})", t.name, mr.next, mr.tiles.as_ref().map(|t| t.len()).unwrap_or(0));
        }
        mr.shot = None;
    }
    let tiles = mr.tiles.clone().unwrap_or_default();
    // Laid out and settled: photograph it.
    if let Some((t, _, _)) = &mr.laid {
        if mr.frame >= SETTLE_FRAMES {
            let path = mr.dir.join(format!("{}.png", t.name));
            let _ = std::fs::remove_file(&path);
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()));
            mr.shot = Some((path, mr.frame));
        }
        return;
    }
    // Everything taken: the index, and quit.
    if mr.next >= tiles.len() {
        let index = serde_json::json!({ "seed": game.world.seed, "sun_from": [SUN_FROM.x, SUN_FROM.y, SUN_FROM.z], "tiles": mr.index });
        let _ = std::fs::write(mr.dir.join("index.json"), serde_json::to_string_pretty(&index).unwrap_or_default());
        println!("map render: done, {} tiles", mr.index.len());
        exit.write(AppExit::Success);
        return;
    }
    // Lay the next tile out.
    for e in mr.entities.drain(..) {
        commands.entity(e).despawn();
    }
    let t = tiles[mr.next].clone();
    mr.next += 1;
    let (w, h) = (pw * t.mpp, ph * t.mpp);
    let (x0, y0, x1, y1) = (t.centre.x - w / 2.0, t.centre.y - h / 2.0, t.centre.x + w / 2.0, t.centre.y + h / 2.0);
    let world = &game.world;
    let ter = &world.terrain;
    let cell = (t.mpp * 1.5).max(1.5);
    // (A cell's margin round the edge, so nothing's missing at the border.)
    let g = ground(ter, &world.routes.roads, x0 - cell, y0 - cell, x1 + cell, y1 + cell, cell);
    let mut top = 0.0f32;
    for j in 0..=8 {
        for i in 0..=8 {
            top = top.max(ter.height(V2::new(x0 + w * i as f32 / 8.0, y0 + h * j as f32 / 8.0)));
        }
    }
    let ground_e = commands.spawn((Mesh3d(meshes.add(g.mesh())), MeshMaterial3d(mats.lit.clone()), Transform::default(), bevy::camera::visibility::NoFrustumCulling)).id();
    mr.entities.push(ground_e);
    // The map's own rocks.
    let mut rb = Builder::new();
    for k in ter.edits.rocks.iter().chain(ter.authored.rocks.iter()).filter(|k| k.pos.x >= x0 && k.pos.x < x1 && k.pos.y >= y0 && k.pos.y < y1) {
        super::scene::rock(&mut rb, k, ter.surface(k.pos), ter.slope(k.pos));
    }
    if !rb.is_empty() {
        mr.entities.push(commands.spawn((Mesh3d(meshes.add(rb.mesh())), MeshMaterial3d(mats.lit.clone()), Transform::default())).id());
    }
    // The land's own trees and bushes (full shapes close up, the low ones from afar).
    let (trees, blobs, bushes, bush_simple, mat) = foliage.parts();
    let close = t.mpp <= 2.0;
    let turn = |p: V2| Rng::from_keys(&[(p.x * 10.0) as i64 as u64, (p.y * 10.0) as i64 as u64, 0x7AE5]).f32() * std::f32::consts::TAU;
    let in_hole = |p: V2| t.hole.is_some_and(|(a, b, c, d)| p.x >= a && p.x < c && p.y >= b && p.y < d);
    for (p, kind, s) in plants::land_trees_in(world, x0, y0, x1, y1).into_iter().filter(|q| !in_hole(q.0)) {
        let mesh = if close { &trees[kind as usize] } else { &blobs[kind as usize] };
        let tf = Transform::from_translation(to3(p, ter.surface(p) - 0.05)).with_rotation(Quat::from_rotation_y(turn(p))).with_scale(Vec3::splat(s));
        mr.entities.push(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), tf)).id());
    }
    for (p, s) in plants::land_bushes_in(world, x0, y0, x1, y1).into_iter().filter(|q| !in_hole(q.0)) {
        let v = (turn(p) * 10.0) as usize % bushes.len().max(1);
        let mesh = if close { &bushes[v] } else { &bush_simple[v] };
        let tf = Transform::from_translation(to3(p, ter.surface(p) - 0.05)).with_rotation(Quat::from_rotation_y(turn(p))).with_scale(Vec3::splat(s));
        mr.entities.push(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), tf)).id());
    }
    // The camera: straight down over the tile, north up.
    let eye = top + 300.0;
    *tf = Transform::from_translation(to3(t.centre, eye)).looking_at(to3(t.centre, 0.0), Vec3::NEG_Z);
    *proj = Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::Fixed { width: w, height: h }, near: 1.0, far: eye + 800.0, ..OrthographicProjection::default_3d() });
    commands.entity(cam_e).remove::<bevy::post_process::bloom::Bloom>();
    commands.entity(cam_e).remove::<bevy::pbr::DistanceFog>();
    // Shadows over the whole tile (one cascade, as deep as the camera sees).
    let cascades = CascadeShadowConfigBuilder { num_cascades: 1, minimum_distance: 1.0, first_cascade_far_bound: eye + 800.0, maximum_distance: eye + 800.0, overlap_proportion: 0.0 }.build();
    commands.queue(move |w: &mut World| {
        let mut q = w.query_filtered::<Entity, With<Sun>>();
        let suns: Vec<Entity> = q.iter(w).collect();
        for s in suns {
            w.entity_mut(s).insert(cascades.clone());
        }
    });
    mr.laid = Some((t, w, h));
    mr.frame = 0;
}
