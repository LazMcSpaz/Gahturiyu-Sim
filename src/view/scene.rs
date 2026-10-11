//! The 3D view: land, roads, towns, people and what they're doing, built from
//! the simulation so the view can never disagree with it.
//!
//! Three kinds of mesh, kept for different lengths of time:
//! - the ground and roads, rebuilt only when the camera moves a whole grid
//!   cell or zooms (`Ground`);
//! - each town near the camera, rebuilt only when someone goes in or out of
//!   one of its buildings (the walls are cut away round whoever is inside);
//! - everything that moves (people, fights, things lying about, rings),
//!   rebuilt every frame.
//!
//! Buildings are drawn at true size; people are true size up close and scaled
//! up as the camera pulls back, so they stay visible the way units do in a
//! strategy game. Past a few hundred metres people are a plain shape, and
//! groups beyond band 2 are a single marker.

use std::collections::HashMap;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    body,
    buildings::{door_of, Door},
    combat::{FxKind, SQUAD_SIDE},
    crafting::Station,
    geo::{self, V2},
    group::Kind as GroupKind,
    items,
    effects::{Does, Summon},
    person::PersonId,
    routine::Doing,
    society::Workplace,
    jobs::{PlaceKind, Shelf},
    race::Race,
    rng,
    settlement::{Building, BuildingKind},
    terrain::Terrain,
    World,
};

use super::app::{Game, Hover, View};
use super::cam::to3;
use super::mesh::Builder;
use super::models::Models;
use super::palette::{self, race_color, Rgb};

/// Height of a Horaro stilt-home deck above the water.
const DECK: f32 = 2.4;
/// Cells across the fine ground patch around the camera target.
const GROUND_CELLS: usize = 90;
/// In the land editor: how far round the camera the finer patch reaches, and
/// cells across it.
const EDIT_REACH: f32 = 300.0;
const EDIT_CELLS: usize = 150;
/// Cells across the coarse ring that carries the far land to the horizon.
const FAR_CELLS: usize = 72;
/// Beyond this distance from the camera, a person is a plain shape.
pub const PERSON_SIMPLE: f32 = 260.0;
/// How fast an arrow is drawn flying, m/s.
pub const ARROW_SPEED: f32 = 45.0;
/// How long a missed arrow lies on the ground, seconds.
pub const ARROW_LIES: f32 = 20.0;

/// Materials: lit (sun, moon, fires), glowing (windows, embers, flames:
/// shown at their own colour whatever the light), and flat markings on the
/// ground (rings, order lines: unlit, so they read at night).
#[derive(Resource)]
pub struct Mats {
    pub lit: Handle<StandardMaterial>,
    pub glow: Handle<StandardMaterial>,
    pub flat: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct GroundMesh;
#[derive(Component)]
pub struct Dynamic;

#[derive(Resource, Default)]
pub struct Scene3d {
    ground_key: Option<(i64, i64, u32, u32)>,
    ground: Option<(Handle<Mesh>, Handle<Mesh>)>,
    pub grid: Grid,
    dynamic: Option<(Handle<Mesh>, Handle<Mesh>, Handle<Mesh>)>,
    towns: HashMap<u16, Town>,
    /// Triangles in our own meshes (for the readout): what moves, the
    /// ground and roads, and the towns.
    pub triangles: usize,
    pub ground_triangles: usize,
    pub town_triangles: usize,
    /// Which loaded save the caches were built for (see `Game::loads`).
    loads: u32,
    /// Which version of the land's hand edits the ground and towns were
    /// built for, and the rocks drawn (with what they were built for).
    edits: u32,
    towns_edits: u32,
    rocks: Option<Handle<Mesh>>,
    rocks_key: Option<((i64, i64, u32, u32), u32, (i64, i64))>,
    /// The forged town's homes, and which models and ground they were drawn with.
    forge: Vec<Entity>,
    forge_key: Option<(u32, (i64, i64, u32, u32))>,
}

struct Town {
    entities: Vec<Entity>,
    occupied: Vec<u16>,
    /// The workplaces as they were laid out when drawn.
    layout: u64,
    /// Which set of loaded models it was drawn with.
    with_models: u32,
    triangles: usize,
    /// Each building's door (None for none) and floor level as drawn
    /// (`interiors::floor_height`), worked out when the town is rebuilt, so
    /// people and things inside stand on it without asking the sim each frame.
    doors: Vec<Option<Door>>,
    floors: Vec<f32>,
}

/// Where the ground mesh's vertices are, so things laid on the ground can
/// follow the drawn surface exactly rather than the finer true terrain (which
/// would leave them buried between vertices on steep slopes).
#[derive(Default, Clone, Copy)]
pub struct Grid {
    centre: V2,
    half_fine: f32,
    fine: f32,
    coarse: f32,
    far: f32,
}

impl Grid {
    /// Height of the drawn ground at `p`, matching how each cell is split
    /// into two triangles.
    pub fn height(&self, t: &Terrain, p: V2) -> f32 {
        if self.fine <= 0.0 {
            return t.surface(p);
        }
        let inside = (p.x - self.centre.x).abs() < self.half_fine && (p.y - self.centre.y).abs() < self.half_fine;
        let (half, step) = if inside { (self.half_fine, self.fine) } else { (self.far, self.coarse) };
        let (x0, y0) = (self.centre.x - half, self.centre.y - half);
        let (fx, fy) = ((p.x - x0) / step, (p.y - y0) / step);
        let (i, j) = (fx.floor(), fy.floor());
        let (u, v) = (fx - i, fy - j);
        let corner = |di: f32, dj: f32| {
            let q = V2::new(x0 + (i + di) * step, y0 + (j + dj) * step);
            t.height(q).max(0.0)
        };
        let (a, b, c, d) = (corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0));
        let h = if u >= v { a + (b - a) * u + (c - b) * v } else { a + (c - d) * u + (d - a) * v };
        if t.is_sea(p) {
            0.0
        } else {
            h.max(0.3)
        }
    }
}

pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let lit = materials.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.92, reflectance: 0.2, cull_mode: None, double_sided: false, ..default() });
    let glow = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, cull_mode: None, double_sided: false, ..default() });
    let flat = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, cull_mode: None, double_sided: false, depth_bias: 50.0, ..default() });
    commands.insert_resource(Mats { lit, glow, flat });
}

fn spawn_mesh(commands: &mut Commands, meshes: &mut Assets<Mesh>, mat: &Handle<StandardMaterial>, b: Builder, marker: impl Bundle) -> (Entity, Handle<Mesh>) {
    let h = meshes.add(b.mesh());
    let e = commands.spawn((Mesh3d(h.clone()), MeshMaterial3d(mat.clone()), Transform::default(), NoFrustumCulling, marker)).id();
    (e, h)
}

/// Build what the 3D view shows this frame, and where things are on screen
/// (for hovering, labels and health bars).
#[allow(clippy::too_many_arguments)]
pub fn update(mut commands: Commands, mut game: ResMut<Game>, mut scene: ResMut<Scene3d>, mut meshes: ResMut<Assets<Mesh>>, mats: Res<Mats>, models: Res<Models>, mut vis: Query<&mut Visibility, Or<(With<GroundMesh>, With<Dynamic>)>>) {
    let game = &mut *game;
    let scene = &mut *scene;
    if scene.loads != game.loads {
        // A save was loaded: the land and towns may be another world's.
        scene.loads = game.loads;
        scene.ground_key = None;
        scene.rocks_key = None;
        scene.forge_key = None;
        for (_, t) in scene.towns.drain() {
            for e in t.entities {
                commands.entity(e).despawn();
            }
        }
    }
    // The land was edited: the ground now; the towns on it once the stroke is done.
    let ev = game.world.terrain.edits.ground_v;
    if scene.edits != ev {
        scene.edits = ev;
        scene.ground_key = None;
    }
    if scene.towns_edits != ev && !game.editor.stroking {
        scene.towns_edits = ev;
        for (_, t) in scene.towns.drain() {
            for e in t.entities {
                commands.entity(e).despawn();
            }
        }
    }
    game.picks.clear();
    game.labels.clear();
    game.bars.clear();
    let show = game.view == View::Scene;
    for mut v in &mut vis {
        *v = if show { Visibility::Inherited } else { Visibility::Hidden };
    }
    for t in scene.towns.values() {
        for &e in &t.entities {
            if let Ok(mut v) = vis.get_mut(e) {
                *v = if show { Visibility::Inherited } else { Visibility::Hidden };
            }
        }
    }
    if !show {
        // Towns hold their own entities; hide them by despawning (cheap to rebuild).
        for (_, t) in scene.towns.drain() {
            for e in t.entities {
                commands.entity(e).despawn();
            }
        }
        return;
    }
    let w = &game.world;
    let oc = &game.orbit;
    let radius = oc.draw_radius();
    let far = oc.far_radius();
    let t = &w.terrain;
    let mut tris = 0usize;

    // ---- The land, cached ---------------------------------------------------
    let coarse = far * 2.0 / FAR_CELLS as f32;
    // While the land is being edited, a finer patch round the camera, so
    // small brush strokes show.
    // (And round a forged town, whose cliffs need it.)
    let editing = game.editor.on || w.forge.as_ref().and_then(|f| f.centre()).is_some_and(|c| c.dist(oc.target) < 900.0);
    let key = ((oc.target.x / coarse).round() as i64, (oc.target.y / coarse).round() as i64, radius.to_bits() ^ editing as u32 ^ palette::snow_step().wrapping_mul(0x9E37), (oc.dist * oc.pitch.sin() / 25.0).round() as u32);
    if scene.ground_key != Some(key) {
        let centre = V2::new(key.0 as f32 * coarse, key.1 as f32 * coarse);
        // Fine patch: a whole number of coarse cells, so its edge meets the ring.
        let reach = if editing { radius.min(EDIT_REACH) } else { radius };
        let half_fine = ((reach / coarse).ceil().max(1.0)) * coarse;
        let cells = if editing { EDIT_CELLS } else { GROUND_CELLS };
        let fine = half_fine * 2.0 / cells as f32;
        let mut g = Builder::new();
        ground_patch(&mut g, t, centre, half_fine, fine, None);
        ground_patch(&mut g, t, centre, far, coarse, Some(half_fine));
        scene.grid = Grid { centre, half_fine, fine, coarse, far };
        // Roads, laid over the land.
        let grid = scene.grid;
        let on_ground = |p: V2| grid.height(t, p);
        let mut r = Builder::new();
        let lw = 5.0f32.max(oc.dist / 90.0);
        // Never across a building's floor: the ribbon is cut away inside
        // every town building's outline (the floor is barely above the ground).
        let outlines = doors_around(w, oc.target, radius * 1.5 + 200.0);
        let roads = Clip { ground: &on_ground, outlines: &outlines };
        for road in &w.routes.roads {
            for seg in road.windows(2) {
                if seg[0].dist(oc.target) > radius * 1.5 {
                    continue;
                }
                let piece = (radius / 45.0).max(10.0);
                draped_ribbon_clipped(&mut r, &roads, seg[0], seg[1], lw, 0.25, palette::ROAD, piece);
            }
        }
        g.append(r);
        scene.ground_triangles = g.triangles();
        match &scene.ground {
            Some((h, _)) => {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = g.mesh();
                }
            }
            None => {
                let (_, h) = spawn_mesh(&mut commands, &mut meshes, &mats.lit, g, GroundMesh);
                scene.ground = Some((h.clone(), h));
            }
        }
        scene.ground_key = Some(key);
    }
    let grid = scene.grid;
    let on_ground = |p: V2| grid.height(t, p);
    let eye = oc.eye();

    // ---- Towns, cached ------------------------------------------------------
    let open = w.occupied();
    // Screenshots can show every building cut open (`GAHT_CUTAWAY`).
    let cutaway = game.cutaway;
    let near: Vec<u16> = w.settlements.iter().filter(|s| s.pos.dist(oc.target) <= radius + s.reach).map(|s| s.id).collect();
    scene.towns.retain(|id, town| {
        let keep = near.contains(id);
        if !keep {
            for &e in &town.entities {
                commands.entity(e).despawn();
            }
        }
        keep
    });
    for &sid in &near {
        let s = &w.settlements[sid as usize];
        let occupied: Vec<u16> = (0..s.buildings.len() as u16).filter(|i| cutaway || open.contains(&(sid, *i))).collect();
        let layout = w.society.towns.get(sid as usize).map(|tl| tl.places.iter().fold(tl.places.len() as u64, |h, p| h.rotate_left(5) ^ p.seed)).unwrap_or(0);
        let fresh = scene.towns.get(&sid).map(|tw| tw.occupied != occupied || tw.with_models != models.generation || tw.layout != layout).unwrap_or(true);
        if fresh {
            if let Some(old) = scene.towns.remove(&sid) {
                for e in old.entities {
                    commands.entity(e).despawn();
                }
            }
            let mut lit = Builder::new();
            let mut glow = Builder::new();
            let mut ents = Vec::new();
            for (i, bd) in s.buildings.iter().enumerate() {
                match door_of(s, i as u16) {
                    Some(d) if occupied.contains(&(i as u16)) => super::interiors::interior(&mut lit, &mut glow, w, &d, &on_ground),
                    _ => {
                        let model = match bd.kind {
                            BuildingKind::RoduroHome => models.roduro_kind(bd.seed, bd.size),
                            BuildingKind::HoraroStilt => models.stilt_kind(bd.seed),
                            _ => None,
                        };
                        match (bd.kind, model) {
                            (BuildingKind::RoduroHome, Some(name)) => {
                                let ground = on_ground(bd.pos);
                                let sink = (t.slope(bd.pos) * bd.size * 0.6).min(4.0);
                                // The models are in metres: a cottage is a cottage, a great house is bigger.
                                ents.extend(models.spawn(&mut commands, name, to3(bd.pos, ground - sink), bd.rot, 0.0));
                                window_glow(&mut glow, bd, ground, bd.size * 0.5);
                            }
                            (BuildingKind::HoraroStilt, Some(name)) => {
                                // The kit is in metres already; stood on the water line.
                                ents.extend(models.spawn_assembly(&mut commands, name, to3(bd.pos, 0.0), bd.rot, 1.0));
                            }
                            // Placeholder shapes by variant (the hearth its own way).
                            _ => {
                                if !super::interiors::exterior(&mut lit, &mut glow, t, s, i as u16, &on_ground) {
                                    building(&mut lit, &mut glow, t, bd, &on_ground);
                                }
                            }
                        }
                    }
                }
            }
            if let Some(tl) = w.society.towns.get(sid as usize) {
                for wp in &tl.places {
                    workplace(&mut lit, &mut glow, t, w, wp, &on_ground);
                }
            }
            let tris = lit.triangles() + glow.triangles();
            ents.push(spawn_mesh(&mut commands, &mut meshes, &mats.lit, lit, ()).0);
            ents.push(spawn_mesh(&mut commands, &mut meshes, &mats.glow, glow, ()).0);
            let doors: Vec<Option<Door>> = (0..s.buildings.len() as u16).map(|i| door_of(s, i)).collect();
            let floors = doors.iter().map(|d| d.map(|d| super::interiors::floor_height(&d, &on_ground).0).unwrap_or(f32::MIN)).collect();
            scene.towns.insert(sid, Town { entities: ents, occupied, layout, with_models: models.generation, triangles: tris, doors, floors });
        }
        for (i, _) in s.buildings.iter().enumerate() {
            if let Some(d) = door_of(s, i as u16) {
                if d.outside.dist(oc.target) < 160.0 {
                    game.picks.push((to3(d.outside, on_ground(d.outside) + 1.2), 0.0, Hover::Door((sid, i as u16))));
                }
            }
        }
        if s.pos.dist(oc.target) < radius * 1.1 {
            game.labels.push((to3(s.pos, on_ground(s.pos) + 28.0 + s.radius() * 0.08), super::lexicon::town(w, s.id)));
            game.picks.push((to3(s.pos, on_ground(s.pos) + 28.0 + s.radius() * 0.08), 20.0, Hover::Town(sid)));
        }
    }

    // ---- The forged town's homes, cached ----------------------------------------
    // Real models in metres, stood on the drawn ground; redrawn when a model
    // loads or the ground under them is rebuilt.
    let forge_key = (models.generation, scene.ground_key.unwrap_or_default());
    if scene.forge_key != Some(forge_key) {
        scene.forge_key = Some(forge_key);
        for e in scene.forge.drain(..) {
            commands.entity(e).despawn();
        }
        if let Some(town) = &w.forge {
            let grid = scene.grid;
            let on_ground = |p: V2| grid.height(t, p);
            // With the final models off (or one missing), placeholder shapes.
            let (mut hb, mut hg) = (Builder::new(), Builder::new());
            for h in &town.founding.homes {
                if super::models::use_final_models() && models.has(&h.model) {
                    let ground = grid.height(t, h.at);
                    let sink = (t.slope(h.at) * 6.0).min(2.0);
                    scene.forge.extend(models.spawn(&mut commands, &h.model, to3(h.at, ground - sink), h.rot, 0.0));
                } else {
                    super::interiors::forge_home(&mut hb, &mut hg, t, h.at, h.rot, h.eldest, &on_ground);
                }
            }
            if !hb.is_empty() {
                scene.forge.push(spawn_mesh(&mut commands, &mut meshes, &mats.lit, hb, GroundMesh).0);
                scene.forge.push(spawn_mesh(&mut commands, &mut meshes, &mats.glow, hg, GroundMesh).0);
            }
            // The ways: lanes laid on the drawn ground, stairs cut as treads,
            // slab bridges.
            let mut wb = Builder::new();
            let outlines = town.centre().map(|c| doors_around(w, c, 1500.0)).unwrap_or_default();
            let clip = Clip { ground: &on_ground, outlines: &outlines };
            for way in &town.ways.ways {
                forge_way(&mut wb, &clip, way);
            }
            if !wb.is_empty() {
                let (e, _) = spawn_mesh(&mut commands, &mut meshes, &mats.lit, wb, GroundMesh);
                scene.forge.push(e);
            }
        }
    }

    // ---- Rocks placed by hand, cached ----------------------------------------
    // (Rebuilt when the rocks change, the ground under them is rebuilt, or
    // the camera moves on.)
    let rock_key = (scene.ground_key.unwrap_or_default(), w.terrain.edits.rocks_v, ((oc.target.x / 100.0).round() as i64, (oc.target.y / 100.0).round() as i64));
    if scene.rocks_key != Some(rock_key) && !(game.editor.stroking && scene.rocks_key.is_some_and(|k| k.1 == rock_key.1)) {
        scene.rocks_key = Some(rock_key);
        let mut rb = Builder::new();
        for k in w.terrain.edits.rocks.iter().chain(w.terrain.authored.rocks.iter()).filter(|k| k.pos.dist(oc.target) < radius * 1.3 + 50.0) {
            rock(&mut rb, k, on_ground(k.pos), t.slope(k.pos));
        }
        match &scene.rocks {
            Some(h) => {
                if let Some(mut m) = meshes.get_mut(h) {
                    *m = rb.mesh();
                }
            }
            None => {
                let (_, h) = spawn_mesh(&mut commands, &mut meshes, &mats.lit, rb, GroundMesh);
                scene.rocks = Some(h);
            }
        }
    }

    // ---- Everything that moves, every frame ---------------------------------
    // Inside a building, people and things stand on its floor (cached with
    // the town), not the ground under it.
    // The building enclosing a point (as `World::building_at` finds it) and
    // its floor, from the doors cached with each town drawn; worked out on
    // the spot only for a town not drawn.
    let towns = &scene.towns;
    let inside = |p: V2| -> Option<(Door, f32)> {
        for s in w.settlements.iter().filter(|s| s.pos.dist(p) < s.reach) {
            match towns.get(&s.id) {
                Some(tw) => {
                    if let Some((d, &f)) = tw.doors.iter().zip(&tw.floors).find_map(|(d, f)| d.filter(|d| d.contains(p)).map(|d| (d, f))) {
                        return Some((d, f));
                    }
                }
                None => {
                    if let Some(d) = (0..s.buildings.len() as u16).filter_map(|i| door_of(s, i)).find(|d| d.contains(p)) {
                        return Some((d, super::interiors::floor_height(&d, &on_ground).0));
                    }
                }
            }
        }
        None
    };
    let floor_at = |p: V2| inside(p).map(|(_, f)| f + super::interiors::FLOOR_TOP);
    let stand = |p: V2| floor_at(p).unwrap_or_else(|| on_ground(p));
    let mut b = Builder::new();
    let mut gl = Builder::new();
    let mut fl = Builder::new();
    let k = (oc.dist / 220.0).max(1.0);
    let mut heads: Vec<(Vec3, PersonId)> = Vec::new();
    for s in &w.settlements {
        for pid in w.residents_in_band1(s.id) {
            // Asleep indoors: out of sight.
            if w.is_indoors_asleep(pid) {
                continue;
            }
            heads.push(person(&mut b, &mut gl, &mut fl, w, pid, k, eye, &floor_at, &on_ground));
        }
    }
    // The watch on a chase: a red ring under the guard, a fainter one under
    // whoever they're after.
    for p in &w.pursuits {
        if let Some(g) = p.pos {
            ring_widened(&mut fl, &on_ground, g, 0.85 * k, 0.09 * k, 18, [0.95, 0.18, 0.12], eye, 0.004);
            let c = w.person_pos(p.culprit);
            ring_widened(&mut fl, &on_ground, c, 1.25 * k, 0.05 * k, 22, [0.85, 0.25, 0.15], eye, 0.004);
        }
    }
    // The squad's own tent stands while someone sleeps under it (it lives in
    // a pack; this is drawing only: `condition::Shelter` says who has it).
    {
        use gahturiyu_sim::sim::condition::{Activity, Shelter};
        let tent_id = gahturiyu_sim::sim::items::id("tent");
        for (j, &m) in w.squad.members.iter().enumerate() {
            let at = w.squad.at[j];
            if at.dist(oc.target) > radius || !w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == tent_id)) {
                continue;
            }
            let under = w.squad.members.iter().enumerate().any(|(i, &o)| w.squad.at[i].dist(at) < 15.0 && w.people[o as usize].cond.as_ref().is_some_and(|c| c.activity == Activity::Sleeping && c.shelter == Shelter::Tent));
            if under {
                let spot = at.add(V2::new(2.6, 1.6));
                tent(&mut b, to3(spot, on_ground(spot)), k, w.people[m as usize].seed);
            }
        }
    }
    for g in &w.groups {
        // Travellers in town at night have found beds indoors.
        if g.pos.dist(oc.target) > radius || w.lodging(g) {
            continue;
        }
        let lead = w.people[g.members[0] as usize].race;
        match g.band {
            1 => {
                if matches!(g.kind, GroupKind::Wanderer { .. }) && !g.is_moving(w.time) && !g.hostile {
                    let r = w.people[g.members[0] as usize].seed;
                    let at = g.pos.add(V2::new(3.5, 2.0));
                    tent(&mut b, to3(at, on_ground(at)), k, r);
                }
                for &m in &g.members {
                    heads.push(person(&mut b, &mut gl, &mut fl, w, m, k, eye, &floor_at, &on_ground));
                }
                // The leader's torch, after dark.
                if w.group_torch_lit(g, w.time) {
                    let f = torch_flame(w, g.members[0], k, &stand);
                    b.stick(f - vec3(0.0, 0.55 * k, 0.0), f, 0.06 * k, palette::TIMBER);
                    gl.column(f - vec3(0.0, 0.05, 0.0), 0.11 * k.min(3.0), 0.01, 0.32 * k.min(3.0), 6, palette::EMBER);
                }
            }
            2 => {
                // A plain shape for each traveller, round where the group is.
                let n = g.members.len();
                for (j, &m) in g.members.iter().enumerate() {
                    let a = j as f32 / n.max(1) as f32 * std::f32::consts::TAU;
                    let off = if n > 1 { V2::new(a.cos(), a.sin()).scale(1.2 * k) } else { V2::default() };
                    let at = g.pos.add(off);
                    let r = w.people[m as usize].race;
                    simple_person(&mut b, to3(at, stand(at) - 0.1), r, k);
                }
                game.picks.push((to3(g.pos, on_ground(g.pos) + 2.0 * k), 2.0, Hover::Group(g.id)));
                if w.group_torch_lit(g, w.time) {
                    gl.column(to3(g.pos, on_ground(g.pos) + 2.2 * k), 0.25 * k, 0.02, 0.6 * k, 6, palette::EMBER);
                }
            }
            _ => {
                let mk = (oc.dist / 120.0).max(2.0);
                let base = to3(g.pos, on_ground(g.pos));
                let n = g.members.len() as f32;
                b.column(base, 0.25 * mk, 0.25 * mk, 3.0 * mk, 4, palette::scale(race_color(lead), 0.6));
                b.column(base + vec3(0.0, 3.0 * mk, 0.0), (0.6 + 0.12 * n) * mk, 0.0, 1.3 * mk, 4, race_color(lead));
                game.picks.push((base + vec3(0.0, 3.6 * mk, 0.0), 2.0, Hover::Group(g.id)));
            }
        }
    }
    for &m in &w.squad.members {
        heads.push(person(&mut b, &mut gl, &mut fl, w, m, k, eye, &floor_at, &on_ground));
    }
    // Squad members living at a base.
    for m in w.all_residents() {
        let here = w.resident_of(m).and_then(|(b, _)| w.base(b)).is_some_and(|b| b.arrived(m));
        if here && w.person_pos(m).dist(oc.target) < radius {
            heads.push(person(&mut b, &mut gl, &mut fl, w, m, k, eye, &floor_at, &on_ground));
        }
    }
    // Strangers being carried, or set down somewhere by the squad.
    for &pid in w.carried.keys().chain(w.set_down.keys()) {
        if !w.people[pid as usize].in_squad && !w.people[pid as usize].dead {
            heads.push(person(&mut b, &mut gl, &mut fl, w, pid, k, eye, &floor_at, &on_ground));
        }
    }
    // The fallen.
    for &(at, race, _, pid) in &w.corpses {
        if w.carried_by(pid).is_some() {
            heads.push(person(&mut b, &mut gl, &mut fl, w, pid, k, eye, &floor_at, &on_ground));
            continue;
        }
        if at.dist(oc.target) < radius {
            let floor = stand(at);
            heads.push((to3(at, floor + 0.6), pid));
            let base = to3(at, floor - 0.1);
            b.block(base, 1.6 * k, 0.6 * k, 0.35 * k, at.x * 0.37, [0.35, 0.12, 0.10]);
            b.block(base + vec3(0.0, 0.3 * k, 0.0), 1.2 * k, 0.4 * k, 0.15 * k, at.x * 0.37, palette::scale(race_color(race), 0.5));
        }
    }
    // Bandit camps: rough hide lean-tos round a fire.
    for c in &w.camps {
        if c.pos.dist(oc.target) > radius {
            continue;
        }
        let kk = k.min(4.0);
        let fire = to3(c.pos, on_ground(c.pos));
        gl.column(fire - vec3(0.0, 0.1, 0.0), 0.9 * kk, 0.2 * kk, 0.7 * kk, 6, palette::EMBER);
        for j in 0..3 {
            let a = j as f32 * 2.1 + c.group as f32;
            let at = c.pos.add(V2::new(a.cos(), a.sin()).scale(7.0 * kk));
            b.block(to3(at, on_ground(at) - 0.1), 3.2 * kk, 2.4 * kk, 1.6 * kk, a, palette::CAMP_HIDE);
        }
    }
    // Ruins (broken walls round a floor) and lairs (a rock arch and bones).
    for ru in &w.ruins {
        if ru.pos.dist(oc.target) > radius.min(900.0) {
            continue;
        }
        let kk = k.min(4.0);
        let base = to3(ru.pos, on_ground(ru.pos));
        let rr = |i: u64| ((ru.id as u64 * 977 + i * 131) % 100) as f32 / 100.0;
        match ru.kind {
            gahturiyu_sim::sim::ruins::RuinKind::Ruin => {
                b.block(base - vec3(0.0, 0.05, 0.0), 16.0 * kk, 0.15 * kk, 12.0 * kk, 0.3, [0.52, 0.5, 0.46]);
                for i in 0..14 {
                    let a = i as f32 / 14.0 * std::f32::consts::TAU;
                    let at = ru.pos.add(V2::new(a.cos() * 9.0, a.sin() * 7.0).scale(kk));
                    let tall = 0.4 + rr(i) * 3.2;
                    if rr(i + 40) < 0.25 {
                        continue; // a gap in the wall
                    }
                    b.block(to3(at, on_ground(at)), 3.6 * kk, tall * kk, 0.9 * kk, a + std::f32::consts::FRAC_PI_2, [0.6, 0.58, 0.53]);
                }
                // Two columns, one standing, one fallen.
                b.column(base + vec3(2.0, 0.0, 1.0) * kk, 0.55 * kk, 0.5 * kk, 4.2 * kk, 8, [0.68, 0.66, 0.6]);
                let f0 = base + vec3(-3.0, 0.4, -1.5) * kk;
                b.stick(f0, f0 + vec3(3.8, 0.0, 1.2) * kk, 0.5 * kk, [0.68, 0.66, 0.6]);
            }
            gahturiyu_sim::sim::ruins::RuinKind::Lair(_) => {
                // A rock arch over a dark mouth.
                b.dome(base + vec3(0.0, 0.0, -3.0) * kk, 6.0 * kk, 4.5 * kk, 4.0 * kk, 0.3, 0.1, ru.id as u64, [0.38, 0.35, 0.33]);
                b.block(base + vec3(0.0, 0.0, -0.4) * kk, 2.6 * kk, 2.2 * kk, 0.3 * kk, 0.0, [0.08, 0.07, 0.07]);
                for i in 0..10 {
                    let a = rr(i) * std::f32::consts::TAU;
                    let at = ru.pos.add(V2::new(a.cos(), a.sin()).scale((2.0 + rr(i + 9) * 6.0) * kk));
                    let p0 = to3(at, on_ground(at) + 0.1);
                    b.stick(p0, p0 + vec3(a.sin(), 0.0, -a.cos()) * (1.1 * kk), 0.14 * kk, [0.7, 0.62, 0.45]);
                }
            }
        }
        game.picks.push((base + vec3(0.0, 2.5 * kk, 0.0), 6.0, Hover::Ruin(ru.id)));
    }
    // Standing torches.
    for st in &w.standing {
        if st.pos.dist(oc.target) > radius || !st.burning(w.time) {
            continue;
        }
        let base = to3(st.pos, stand(st.pos));
        b.column(base, 0.07, 0.05, 1.8, 5, palette::TIMBER);
        gl.column(base + vec3(0.0, 1.8, 0.0), 0.16, 0.02, 0.45, 6, palette::EMBER);
    }
    // Workshops round the hearths, and things to gather.
    for (i, &(p, st)) in w.stations.iter().enumerate() {
        if p.dist(oc.target) > radius.min(500.0) {
            continue;
        }
        let base = to3(p, on_ground(p));
        let a = p.x * 0.13;
        match st {
            Station::Forge => {
                b.block(base, 2.0, 1.6, 1.0, a, palette::STONE);
                gl.column(base + vec3(0.0, 1.0, 0.0), 0.5, 0.3, 0.3, 6, palette::EMBER);
                b.column(base + vec3(0.8, 0.0, 0.8), 0.25, 0.25, 0.8, 5, [0.25, 0.25, 0.27]);
            }
            Station::Bench => {
                b.block(base, 2.2, 0.9, 0.9, a, palette::TIMBER);
                b.block(base + vec3(0.0, 0.9, 0.0), 0.8, 0.5, 0.15, a, [0.45, 0.3, 0.2]);
            }
            Station::Desk => {
                b.block(base, 1.4, 0.8, 0.8, a, palette::TIMBER);
                b.block(base + vec3(0.0, 0.8, 0.0), 0.6, 0.4, 0.02, a, [0.9, 0.86, 0.72]);
            }
            Station::AlchemyTable => {
                b.block(base, 1.6, 0.9, 0.85, a, palette::TIMBER);
                b.column(base + vec3(0.3, 0.85, 0.0), 0.15, 0.08, 0.35, 6, [0.4, 0.8, 0.6]);
                b.column(base + vec3(-0.3, 0.85, 0.1), 0.12, 0.05, 0.3, 6, [0.85, 0.3, 0.35]);
            }
            Station::Loom => {
                for dx in [-0.9f32, 0.9] {
                    b.stick(base + vec3(dx, 0.0, 0.0), base + vec3(dx, 1.8, 0.0), 0.1, palette::TIMBER);
                }
                b.block(base + vec3(0.0, 0.5, 0.0), 1.7, 0.08, 1.1, 0.0, [0.42, 0.48, 0.30]);
                b.column(base + vec3(0.0, 0.0, 1.2), 0.5, 0.5, 0.35, 8, [0.12, 0.10, 0.08]);
            }
            Station::Workbench => {
                b.block(base, 2.0, 0.9, 0.85, a, palette::TIMBER);
                b.block(base + vec3(0.4, 0.85, 0.0), 0.7, 0.5, 0.06, a, [0.55, 0.38, 0.24]);
            }
            Station::GrowerBed => {
                b.block(base, 2.4, 1.6, 0.35, a, palette::STONE);
                b.dome(base + vec3(0.0, 0.35, 0.0), 0.6, 0.5, 0.5, 0.1, 0.12, 7, palette::STONE);
            }
        }
        game.picks.push((base + vec3(0.0, 1.3, 0.0), 2.0, Hover::Station(i)));
    }
    for n in &w.nodes {
        if n.pos.dist(oc.target) > radius.min(400.0) || !n.ready(w.time) {
            continue;
        }
        let base = to3(n.pos, on_ground(n.pos));
        let col = super::squadui::ground_color(n.item);
        let key = items::item(n.item).key;
        let kk = k.min(4.0);
        match key {
            "iron_ore" | "storm_glass" | "salt_crystal" => b.dome(
                base,
                0.9 * kk,
                0.8 * kk,
                0.7 * kk,
                0.2,
                0.0,
                n.id as u64,
                if key == "iron_ore" {
                    [0.45, 0.35, 0.3]
                } else if key == "storm_glass" {
                    [0.6, 0.75, 0.95]
                } else {
                    [0.92, 0.9, 0.85]
                },
            ),
            "timber" => b.block(base, 2.4 * kk, 0.4 * kk, 0.4 * kk, n.id as f32, palette::TIMBER),
            "emberroot" => b.column(base, 0.35 * kk, 0.05, 0.9 * kk, 5, [0.9, 0.45, 0.15]),
            "ghostcap" => {
                b.column(base, 0.08 * kk, 0.08 * kk, 0.3 * kk, 4, [0.85, 0.85, 0.8]);
                b.column(base + vec3(0.0, 0.3 * kk, 0.0), 0.3 * kk, 0.05, 0.15 * kk, 8, [0.8, 0.82, 0.9]);
            }
            "kelp_frond" => b.column(to3(n.pos, on_ground(n.pos).max(0.0)), 0.4 * kk, 0.1, 0.5 * kk, 5, [0.25, 0.45, 0.25]),
            _ => b.column(base, 0.6 * kk, 0.3 * kk, 0.2 * kk, 6, col),
        }
        game.picks.push((base + vec3(0.0, 0.8 * kk, 0.0), 2.0, Hover::Node(n.id)));
    }
    // Woodlots and mines the squad can work: the pile shows what's left.
    for d in &w.deposits {
        if d.pos.dist(oc.target) > radius.min(500.0) {
            continue;
        }
        let base = to3(d.pos, on_ground(d.pos));
        let kk = k.min(4.0);
        let cap = d.face().cap as f32;
        let full = (d.left_at(w.time) / cap).clamp(0.0, 1.0);
        let key = items::item(d.item).key;
        let n = (full * 6.0).ceil() as usize;
        match key {
            "timber" => {
                // Stacked logs, a stump and an axe-post.
                for i in 0..n {
                    let (row, col) = (i / 3, i % 3);
                    let p = base + vec3(0.0, (0.25 + row as f32 * 0.45) * kk, (col as f32 - 1.0 + row as f32 * 0.5) * 0.48 * kk);
                    b.stick(p - vec3(1.3 * kk, 0.0, 0.0), p + vec3(1.3 * kk, 0.0, 0.0), 0.22 * kk, palette::TIMBER);
                }
                b.column(base + vec3(2.2 * kk, 0.0, 0.8 * kk), 0.35 * kk, 0.35 * kk, 0.5 * kk, 7, [0.45, 0.33, 0.2]);
            }
            _ => {
                let col = if key == "gold_nugget" { [0.85, 0.7, 0.25] } else { [0.45, 0.32, 0.26] };
                // A cut face and the chunks dug from it.
                b.block(base + vec3(0.0, 0.0, -1.4 * kk), 2.6 * kk, 1.8 * kk, 0.8 * kk, 0.0, palette::STONE);
                for i in 0..n {
                    let a = i as f32 * 1.9 + d.id as f32;
                    let p = base + vec3(a.cos(), 0.0, a.sin() * 0.6 + 0.4) * (0.5 + 0.18 * i as f32) * kk;
                    b.dome(p, 0.45 * kk, 0.4 * kk, 0.35 * kk, 0.2, 0.0, d.id as u64 * 7 + i as u64, col);
                }
                if key == "gold_nugget" && n > 0 {
                    gl.dome(base + vec3(0.0, 0.35 * kk, 0.3 * kk), 0.12 * kk, 0.12 * kk, 0.1 * kk, 0.0, 0.0, 3, [1.0, 0.85, 0.35]);
                }
            }
        }
        // A marker post, so it reads as something to click.
        b.stick(base + vec3(-1.6 * kk, 0.0, 1.2 * kk), base + vec3(-1.6 * kk, 1.6 * kk, 1.2 * kk), 0.09 * kk, palette::TIMBER);
        gl.block(base + vec3(-1.6 * kk, 1.6 * kk, 1.2 * kk), 0.35 * kk, 0.22 * kk, 0.05 * kk, 0.0, if key == "timber" { [0.55, 0.85, 0.45] } else { [0.95, 0.75, 0.35] });
        game.picks.push((base + vec3(0.0, 0.8 * kk, 0.0), 3.0, Hover::Deposit(d.id)));
    }
    // Things lying about.
    for g in &w.ground {
        if g.pos.dist(oc.target) < radius.min(600.0) {
            // Indoors: on the floor, or on the shelves or a table it lies on.
            let indoors = inside(g.pos);
            let on = indoors.and_then(|(d, _)| gahturiyu_sim::sim::layout::resting(&d, g.pos));
            let floor = indoors.map_or_else(|| on_ground(g.pos), |(_, f)| f + super::interiors::FLOOR_TOP);
            let base = to3(g.pos, floor + on.map_or(0.0, |o| o.0));
            let kk = k.min(6.0);
            b.block(base, 0.55 * kk, 0.35 * kk, 0.22 * kk, on.map_or(g.id as f32 * 1.7, |o| o.1), super::squadui::ground_color(g.item));
            game.picks.push((base + vec3(0.0, 0.3 * kk, 0.0), 4.0, Hover::Item(g.id)));
        }
    }
    // Spell effects, briefly; arrows in flight, and misses lying where they fell.
    for battle in &w.battles {
        for fx in &battle.fx {
            let age = (w.time - fx.at) as f32;
            match fx.kind {
                FxKind::Fireball { at, radius } if (0.0..0.8).contains(&age) => {
                    let r = radius * (0.4 + age * 1.2);
                    gl.dome(to3(at, on_ground(at)), r, r, r * 0.8, 0.2, 0.0, 7, [1.0, 0.55 - age * 0.4, 0.15]);
                }
                FxKind::Bolt { from, to } if (0.0..0.8).contains(&age) => {
                    let (a, c) = (to3(from, on_ground(from) + 1.5), to3(to, on_ground(to) + 1.2));
                    gl.stick(a, c, 0.25, [0.85, 0.9, 1.0]);
                }
                FxKind::Fizzle { at } if (0.0..0.8).contains(&age) => {
                    gl.patch(to3(at, on_ground(at) + 2.2), 0.8, 0.8, 0.0, [0.6, 0.6, 0.7]);
                }
                FxKind::Arrow { from, to, hit } => arrow(&mut b, &on_ground, from, to, hit, age, k),
                _ => {}
            }
        }
    }

    // The caches in ruins and lairs, likewise.
    super::interiors::wild_containers(&mut b, &mut gl, w, &on_ground, oc.target, radius.min(400.0), &mut game.picks);
    // Containers in buildings someone is in: drawn every frame, so a lid
    // opens while it's gone through and a picked lock loses its plate.
    for &sid in &near {
        let s = &w.settlements[sid as usize];
        for i in 0..s.buildings.len() as u16 {
            if cutaway || open.contains(&(sid, i)) {
                if let Some(d) = door_of(s, i) {
                    super::interiors::containers(&mut b, &mut gl, w, &d, &on_ground, &mut game.picks);
                }
            }
        }
    }

    // ---- Magic on show ---------------------------------------------------
    magic_scene(w, &mut b, &mut gl, &mut fl, &on_ground, oc.target, radius, k, eye, &mut game.bars);

    // Flames on lit torches.
    for &m in &w.squad.members {
        if w.torch_lit(m) && w.fighter(m).map(|f| !f.ko).unwrap_or(true) {
            let f = torch_flame(w, m, k, &stand);
            gl.column(f - vec3(0.0, 0.05, 0.0), 0.11 * k.min(3.0), 0.01, 0.32 * k.min(3.0), 6, palette::EMBER);
        }
    }

    // Rings and the order lines, laid on the land. At night each member
    // gets a faint ring too, so the squad can always be made out.
    let night = 1.0 - gahturiyu_sim::sim::stealth::daylight(w.time);
    let rw = (oc.dist / 260.0).max(0.12);
    let sq = w.squad.pos;
    for (i, &m) in w.squad.members.iter().enumerate() {
        let at = w.member_pos(i);
        let picked = game.sel.shows(w, m);
        let face = super::cues::facing(w, m, at);
        // Indoors, laid on the floor.
        let inner = floor_at(at);
        let ring_ground = |p: V2| inner.unwrap_or_else(|| on_ground(p));
        if picked {
            facing_ring(&mut fl, &ring_ground, at, 1.2 * k, rw * 0.8, face, palette::GOLD, eye, false);
        } else {
            // Barely there: enough to see which way they face.
            let col = palette::scale(palette::race_color(w.people[m as usize].race), 0.3 + 0.25 * night);
            facing_ring(&mut fl, &ring_ground, at, 1.0 * k, 0.04 * k, face, col, eye, true);
        }
        let goal = w.squad.goal[i];
        if w.fighter(m).is_none() && at.dist(goal) > 1.5 {
            let col = if picked { palette::GOLD } else { [0.95, 0.95, 0.95] };
            draped_ribbon(&mut fl, &stand, at, goal, rw * 0.5, 0.4, col, 10.0);
            let inner = floor_at(goal);
            draped_ring(&mut fl, &|p: V2| inner.unwrap_or_else(|| on_ground(p)), goal, 0.8 * k, rw * 0.6, 12, col, eye);
        }
    }
    // Aiming a spell: how far it reaches, and who's in its sights.
    if let Some((who, s)) = game.aim {
        if let Some(ki) = w.squad.index(who) {
            let at = w.member_pos(ki);
            let range = s.def().range.max(2.0);
            draped_ring(&mut fl, &on_ground, at, range, rw * 0.7, 96, super::squadui::RITUAL, eye);
            if let Some(Hover::Person(p)) = game.hover {
                let tp = w.person_pos(p);
                let col = if tp.dist(at) <= range { super::squadui::RITUAL } else { [0.95, 0.45, 0.30] };
                draped_ring(&mut fl, &on_ground, tp, 1.3 * k, rw * 0.9, 24, col, eye);
            }
        }
    }
    if game.rings {
        draped_ring(&mut fl, &on_ground, sq, BAND1_RADIUS, rw * 2.0, 160, [0.92, 0.94, 0.95], eye);
        draped_ring(&mut fl, &on_ground, sq, BAND2_RADIUS, rw * 3.0, 320, [0.80, 0.84, 0.86], eye);
    }
    super::animals::draw(w, &mut b, &mut gl, &on_ground, oc.target, radius, k, eye);
    super::baseui::draw(w, &mut b, &mut gl, &mut fl, &on_ground, oc.target, radius, game.placing.as_ref());
    tris += b.triangles() + gl.triangles() + fl.triangles();
    match &scene.dynamic {
        Some((hb, hg, hf)) => {
            for (h, m) in [(hb, b), (hg, gl), (hf, fl)] {
                if let Some(mut x) = meshes.get_mut(h) {
                    *x = m.mesh();
                }
            }
        }
        None => {
            let (_, hb) = spawn_mesh(&mut commands, &mut meshes, &mats.lit, b, Dynamic);
            let (_, hg) = spawn_mesh(&mut commands, &mut meshes, &mats.glow, gl, Dynamic);
            let (_, hf) = spawn_mesh(&mut commands, &mut meshes, &mats.flat, fl, Dynamic);
            scene.dynamic = Some((hb, hg, hf));
        }
    }
    scene.triangles = tris;
    scene.town_triangles = scene.towns.values().map(|t| t.triangles).sum();

    // Heads: where to hover people and hang their health bars.
    for (p, id) in heads {
        game.picks.push((p, 2.0, Hover::Person(id)));
        if let Some((vit, mana, down)) = super::hud::bar_for(w, id) {
            if p.distance(eye) < 400.0 {
                game.bars.push((p, vit, mana, down));
            }
        }
    }
}

/// An arrow: flying from `from` to `to` (a short arc), then lying there for a
/// while if it missed.
fn arrow(b: &mut Builder, on_ground: &dyn Fn(V2) -> f32, from: V2, to: V2, hit: bool, age: f32, k: f32) {
    let len = from.dist(to).max(0.1);
    let fly = len / ARROW_SPEED;
    if age < 0.0 || age > fly + if hit { 0.0 } else { ARROW_LIES } {
        return;
    }
    let kk = k.min(3.0);
    let shaft = 0.85 * kk;
    let height = |u: f32| {
        let p = from.lerp(to, u);
        let lift = if hit { 1.3 } else { 1.4 * (1.0 - u) + 0.1 };
        on_ground(p) + lift + 4.0 * len * 0.04 * u * (1.0 - u)
    };
    let u = (age / fly).min(1.0);
    let dir = to.sub(from).scale(1.0 / len);
    let (tip, tail) = if u < 1.0 {
        // In the air, pointing along its flight.
        let p = from.lerp(to, u);
        let du = (0.02f32).min(1.0 - u).max(0.001);
        let ahead = from.lerp(to, u + du);
        let d3 = (to3(ahead, height(u + du)) - to3(p, height(u))).normalize_or_zero();
        let tip = to3(p, height(u));
        (tip, tip - d3 * shaft)
    } else {
        // Stuck in the ground at a slant.
        let base = to3(to, on_ground(to));
        let back = vec3(-dir.x, 0.0, -dir.y);
        (base, base + (back * 0.8 + Vec3::Y * 0.5).normalize() * shaft)
    };
    b.stick(tail, tip, 0.05 * kk, [0.55, 0.42, 0.28]);
    // Fletching, pale, at the tail.
    let d = (tip - tail).normalize_or_zero();
    b.stick(tail, tail + d * 0.18 * kk, 0.14 * kk, [0.9, 0.9, 0.85]);
}

/// A flat strip laid over the land from `a` to `c`, cut into pieces no longer
/// than `piece` metres so it follows the ground instead of cutting through it.
#[allow(clippy::too_many_arguments)]
fn draped_ribbon(b: &mut Builder, ground: &dyn Fn(V2) -> f32, a: V2, c: V2, width: f32, lift: f32, col: Rgb, piece: f32) {
    let len = a.dist(c);
    if len < 1e-3 {
        return;
    }
    let d = c.sub(a).scale(1.0 / len);
    let side = V2::new(-d.y, d.x).scale(width * 0.5);
    let n = ((len / piece).ceil() as usize).max(1);
    let lc = palette::lin(col);
    for i in 0..n {
        let p0 = a.lerp(c, i as f32 / n as f32);
        let p1 = a.lerp(c, (i + 1) as f32 / n as f32);
        let q = [p0.sub(side), p0.add(side), p1.add(side), p1.sub(side)];
        let v = q.map(|p| to3(p, ground(p) + lift));
        b.quad_lin(v, [Vec3::Y; 4], [lc; 4]);
    }
}

/// The ground a strip is laid on, and the building outlines it is cut away
/// inside (roads and ways never run across a floor).
struct Clip<'a> {
    ground: &'a dyn Fn(V2) -> f32,
    outlines: &'a [Door],
}

/// How fine a strip is cut where it meets a building's outline, metres.
const CLIP_CELL: f32 = 0.4;

/// The doors of every town building within `reach` of `c`.
fn doors_around(w: &World, c: V2, reach: f32) -> Vec<Door> {
    w.settlements.iter().filter(|s| s.pos.dist(c) < reach + s.reach).flat_map(|s| (0..s.buildings.len() as u16).filter_map(move |i| door_of(s, i))).collect()
}

impl Clip<'_> {
    /// Lay the quad `q` (corners in order round it) `lift` over the ground;
    /// where it meets a building, as small cells, leaving out any cell that
    /// touches the inside of an outline.
    fn quad(&self, b: &mut Builder, q: [V2; 4], lift: f32, lc: [f32; 4]) {
        let mid = q[0].add(q[1]).add(q[2]).add(q[3]).scale(0.25);
        let reach = q.iter().map(|p| p.dist(mid)).fold(0.0, f32::max);
        let near: Vec<&Door> = self.outlines.iter().filter(|d| d.centre.dist(mid) < d.radius + reach).collect();
        let lay = |b: &mut Builder, c: [V2; 4]| b.quad_lin(c.map(|p| to3(p, (self.ground)(p) + lift)), [Vec3::Y; 4], [lc; 4]);
        if near.is_empty() {
            lay(b, q);
            return;
        }
        // q[0]→q[1] across, q[0]→q[3] along.
        let nu = ((q[0].dist(q[1]) / CLIP_CELL).ceil() as usize).max(1);
        let nv = ((q[0].dist(q[3]) / CLIP_CELL).ceil() as usize).max(1);
        let at = |u: f32, v: f32| q[0].lerp(q[1], u).lerp(q[3].lerp(q[2], u), v);
        for j in 0..nv {
            for i in 0..nu {
                let (u0, u1, v0, v1) = (i as f32 / nu as f32, (i + 1) as f32 / nu as f32, j as f32 / nv as f32, (j + 1) as f32 / nv as f32);
                let c = [at(u0, v0), at(u1, v0), at(u1, v1), at(u0, v1)];
                let centre = at((u0 + u1) * 0.5, (v0 + v1) * 0.5);
                if near.iter().any(|d| c.iter().chain(std::iter::once(&centre)).any(|&p| d.contains(p))) {
                    continue;
                }
                lay(b, c);
            }
        }
    }
}

/// `draped_ribbon`, cut away inside building outlines.
#[allow(clippy::too_many_arguments)]
fn draped_ribbon_clipped(b: &mut Builder, clip: &Clip, a: V2, c: V2, width: f32, lift: f32, col: Rgb, piece: f32) {
    let len = a.dist(c);
    if len < 1e-3 {
        return;
    }
    let d = c.sub(a).scale(1.0 / len);
    let side = V2::new(-d.y, d.x).scale(width * 0.5);
    let n = ((len / piece).ceil() as usize).max(1);
    let lc = palette::lin(col);
    for i in 0..n {
        let p0 = a.lerp(c, i as f32 / n as f32);
        let p1 = a.lerp(c, (i + 1) as f32 / n as f32);
        clip.quad(b, [p0.sub(side), p0.add(side), p1.add(side), p1.sub(side)], lift, lc);
    }
}

/// A strip laid over the land along a line of points, with mitred corners
/// so it reads as one way rather than a row of plates. Each stretch is cut
/// into pieces no longer than `piece` metres so it follows the ground.
fn draped_strip(b: &mut Builder, clip: &Clip, pts: &[V2], width: f32, lift: f32, col: Rgb, piece: f32) {
    let pts: Vec<V2> = pts.iter().copied().fold(Vec::new(), |mut v: Vec<V2>, p| {
        if v.last().is_none_or(|q| q.dist(p) > 0.05) {
            v.push(p);
        }
        v
    });
    if pts.len() < 2 {
        return;
    }
    let dir = |i: usize| pts[i + 1].sub(pts[i]).scale(1.0 / pts[i + 1].dist(pts[i]));
    let n = pts.len();
    // The side offset at each point: perpendicular to the mean of its two
    // segments, lengthened so the strip keeps its width round the corner.
    let mut side = Vec::with_capacity(n);
    for i in 0..n {
        let d = if i == 0 {
            dir(0)
        } else if i == n - 1 {
            dir(n - 2)
        } else {
            let m = dir(i - 1).add(dir(i));
            let l = m.len();
            if l < 1e-3 { dir(i) } else { m.scale(1.0 / l) }
        };
        let perp = V2::new(-d.y, d.x);
        let dot = if i == 0 || i == n - 1 { 1.0 } else { { let q = V2::new(-dir(i).y, dir(i).x); (perp.x * q.x + perp.y * q.y).max(0.5) } };
        side.push(perp.scale(width * 0.5 / dot));
    }
    let lc = palette::lin(col);
    for i in 0..n - 1 {
        let (l0, r0, l1, r1) = (pts[i].sub(side[i]), pts[i].add(side[i]), pts[i + 1].sub(side[i + 1]), pts[i + 1].add(side[i + 1]));
        let k = ((pts[i].dist(pts[i + 1]) / piece).ceil() as usize).max(1);
        for j in 0..k {
            let (t0, t1) = (j as f32 / k as f32, (j + 1) as f32 / k as f32);
            clip.quad(b, [l0.lerp(l1, t0), r0.lerp(r1, t0), r0.lerp(r1, t1), l0.lerp(l1, t1)], lift, lc);
        }
    }
}

/// One stretch of a forged town's way, on the drawn ground.
fn forge_way(b: &mut Builder, clip: &Clip, way: &gahturiyu_sim::sim::forge::Way) {
    let ground = clip.ground;
    use gahturiyu_sim::sim::forge::WayKind;
    match way.kind {
        WayKind::Cobbles | WayKind::Dirt => {
            let col = if way.kind == WayKind::Cobbles { palette::COBBLES } else { palette::PATH };
            draped_strip(b, clip, &way.pts, way.width, 0.12, col, 1.5);
        }
        WayKind::Stairs => {
            // Treads of RISER rise, each a flat slab with a riser face below it,
            // up (or down) the true ground.
            const RISER: f32 = 0.18;
            let (top, face) = (palette::lin(palette::STAIR), palette::lin(palette::STAIR_RISER));
            for seg in way.pts.windows(2) {
                let (a, c) = (seg[0], seg[1]);
                let len = a.dist(c);
                if len < 0.2 {
                    continue;
                }
                let (za, zc) = (ground(a), ground(c));
                let rise = zc - za;
                let n = ((rise.abs() / RISER).ceil() as usize).max(1);
                let d = c.sub(a).scale(1.0 / len);
                let side = V2::new(-d.y, d.x).scale(way.width * 0.5);
                for i in 0..n {
                    let (t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                    let (p0, p1) = (a.lerp(c, t0), a.lerp(c, t1));
                    // The tread sits at the higher of its two ends' step heights.
                    let z = za + rise * if rise >= 0.0 { t1 } else { t0 } + 0.06;
                    let zlow = z - RISER;
                    let q = [p0.sub(side), p0.add(side), p1.add(side), p1.sub(side)];
                    b.quad_lin(q.map(|p| to3(p, z)), [Vec3::Y; 4], [top; 4]);
                    // The riser: at the uphill edge when climbing, downhill edge when descending.
                    let (e0, e1) = if rise >= 0.0 { (q[3], q[2]) } else { (q[1], q[0]) };
                    let nrm = if rise >= 0.0 { to3(d.scale(-1.0), 0.0) } else { to3(d, 0.0) };
                    b.quad_lin([to3(e0, zlow), to3(e1, zlow), to3(e1, z), to3(e0, z)], [nrm; 4], [face; 4]);
                }
            }
        }
        WayKind::Bridge => {
            let (a, c) = (way.pts[0], *way.pts.last().unwrap());
            let len = a.dist(c);
            if len < 0.2 {
                return;
            }
            let d = c.sub(a).scale(1.0 / len);
            let side = V2::new(-d.y, d.x).scale(way.width * 0.5);
            let (za, zc) = (ground(a) + 0.35, ground(c) + 0.35);
            let (top, face) = (palette::lin(palette::STAIR), palette::lin(palette::STAIR_RISER));
            let q = [a.sub(side), a.add(side), c.add(side), c.sub(side)];
            let zq = [za, za, zc, zc];
            // Deck, its two sides and its underside, and a low parapet each side.
            b.quad_lin([to3(q[0], zq[0]), to3(q[1], zq[1]), to3(q[2], zq[2]), to3(q[3], zq[3])], [Vec3::Y; 4], [top; 4]);
            b.quad_lin([to3(q[0], zq[0] - 0.5), to3(q[1], zq[1] - 0.5), to3(q[2], zq[2] - 0.5), to3(q[3], zq[3] - 0.5)], [Vec3::NEG_Y; 4], [face; 4]);
            for (i, j) in [(0, 3), (1, 2)] {
                let nrm = to3(q[i].sub(a).scale(1.0 / way.width.max(0.1)), 0.0);
                b.quad_lin([to3(q[i], zq[i] - 0.5), to3(q[j], zq[j] - 0.5), to3(q[j], zq[j]), to3(q[i], zq[i])], [nrm; 4], [face; 4]);
                let inner = [q[i].sub(nrm_v2(nrm).scale(0.3)), q[j].sub(nrm_v2(nrm).scale(0.3))];
                b.quad_lin([to3(q[i], zq[i]), to3(q[j], zq[j]), to3(q[j], zq[j] + 0.6), to3(q[i], zq[i] + 0.6)], [nrm; 4], [face; 4]);
                b.quad_lin([to3(inner[0], zq[i] + 0.6), to3(inner[1], zq[j] + 0.6), to3(q[j], zq[j] + 0.6), to3(q[i], zq[i] + 0.6)], [Vec3::Y; 4], [top; 4]);
            }
        }
    }
}

fn nrm_v2(n: Vec3) -> V2 {
    V2::new(n.x, n.z)
}

/// A ring on the land with a pointed tip on its rim the way someone faces.
#[allow(clippy::too_many_arguments)]
fn facing_ring(b: &mut Builder, ground: &dyn Fn(V2) -> f32, c: V2, r: f32, width: f32, face: f32, col: Rgb, eye: Vec3, faint: bool) {
    if faint {
        ring_widened(b, ground, c, r, width, 18, col, eye, 0.003);
    } else {
        draped_ring(b, ground, c, r, width, 18, col, eye);
    }
    let f = V2::new(face.cos(), face.sin());
    let side = V2::new(-f.y, f.x);
    let w = if faint { r * 0.2 } else { (r * 0.38).max(width * 2.0) };
    let tip = c.add(f.scale(r + w * 1.25));
    let (l, rr) = (c.add(f.scale(r - width)).add(side.scale(w * 0.75)), c.add(f.scale(r - width)).sub(side.scale(w * 0.75)));
    let lift = 0.55 + r * 0.0008;
    let lc = palette::lin(col);
    let v = [l, tip, tip, rr].map(|p| to3(p, ground(p) + lift));
    b.quad_lin(v, [Vec3::Y; 4], [lc; 4]);
}

/// A ring laid on the land. Far pieces are widened so they stay a pixel or
/// two thick seen edge-on; otherwise the ring breaks into dashes.
#[allow(clippy::too_many_arguments)]
fn draped_ring(b: &mut Builder, ground: &dyn Fn(V2) -> f32, c: V2, r: f32, width: f32, n: usize, col: Rgb, eye: Vec3) {
    ring_widened(b, ground, c, r, width, n, col, eye, 0.011);
}

/// A ring whose pieces are widened by `widen` per metre from the camera.
#[allow(clippy::too_many_arguments)]
fn ring_widened(b: &mut Builder, ground: &dyn Fn(V2) -> f32, c: V2, r: f32, width: f32, n: usize, col: Rgb, eye: Vec3, widen: f32) {
    let n = n.max((r * std::f32::consts::TAU / 18.0) as usize).min(2400);
    let lift = 0.5 + r * 0.0008;
    let lc = palette::lin(col);
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let (u0, u1) = (V2::new(a0.cos(), a0.sin()), V2::new(a1.cos(), a1.sin()));
        let mid = c.add(u0.scale(r));
        let away = to3(mid, ground(mid)).distance(eye);
        let width = width.max(away * widen);
        let q = [c.add(u0.scale(r - width * 0.5)), c.add(u0.scale(r + width * 0.5)), c.add(u1.scale(r + width * 0.5)), c.add(u1.scale(r - width * 0.5))];
        let v = q.map(|p| to3(p, ground(p) + lift));
        b.quad_lin(v, [Vec3::Y; 4], [lc; 4]);
    }
}

/// A square patch of land and sea centred on `centre`, `half` metres each
/// way, in cells `step` wide. With `hole`, cells inside that half-width are
/// skipped (the fine patch covers them).
fn ground_patch(b: &mut Builder, t: &Terrain, centre: V2, half: f32, step: f32, hole: Option<f32>) {
    let cells = (half * 2.0 / step).round() as i64;
    let (x0, y0) = (centre.x - half, centre.y - half);
    // Land where the ground stands above the sea, water where it doesn't:
    // the land itself decides (an authored bay or stack included).
    let vert = |p: V2| -> (Vec3, Vec3, [f32; 4]) {
        let h = t.height(p);
        if h >= 0.0 {
            let h = h.max(0.3);
            let (nx, ny, nz) = t.normal(p, step.max(15.0));
            (to3(p, h), vec3(nx, ny, nz), palette::lin(palette::ground(t, p, h, ny)))
        } else {
            (to3(p, 0.0), Vec3::Y, palette::lin(palette::sea(p)))
        }
    };
    let skip = |xa: f32, xb: f32, ya: f32, yb: f32| -> bool {
        match hole {
            Some(hh) => xa >= centre.x - hh - 0.5 && xb <= centre.x + hh + 0.5 && ya >= centre.y - hh - 0.5 && yb <= centre.y + hh + 0.5,
            None => false,
        }
    };
    for j in 0..cells {
        let (ya, yb) = (y0 + j as f32 * step, y0 + (j + 1) as f32 * step);
        for i in 0..cells {
            let (xa, xb) = (x0 + i as f32 * step, x0 + (i + 1) as f32 * step);
            if skip(xa, xb, ya, yb) {
                continue;
            }
            let ps = [V2::new(xa, ya), V2::new(xb, ya), V2::new(xb, yb), V2::new(xa, yb)];
            let vs = ps.map(vert);
            b.quad_lin(vs.map(|v| v.0), vs.map(|v| v.1), vs.map(|v| v.2));
        }
    }
}


/// The warm window on a Roduro home's hearth side, and its dark door.
fn window_glow(gl: &mut Builder, bd: &Building, ground: f32, r: f32) {
    let (sn, cs) = bd.rot.sin_cos();
    let h = bd.size * 0.5;
    let face = vec3(bd.pos.x, ground, bd.pos.y) + vec3(cs, 0.0, sn) * (r * 0.97);
    gl.patch(face + vec3(0.0, h * 0.35, 0.0), 1.1, 0.8, bd.rot, palette::WINDOW);
}

fn building(b: &mut Builder, gl: &mut Builder, t: &Terrain, bd: &Building, on_ground: &dyn Fn(V2) -> f32) {
    let ground = if bd.kind == BuildingKind::HoraroStilt { 0.0 } else { on_ground(bd.pos) };
    // Sink buildings a little so they sit into slopes rather than float.
    let sink = (t.slope(bd.pos) * bd.size * 0.6).min(4.0);
    let base = to3(bd.pos, ground - sink);
    let r = bd.size * 0.5;
    let s = bd.seed;
    let unit = |k: u64| (rng::key(&[s, k]) >> 40) as f32 / (1u64 << 24) as f32;
    match bd.kind {
        BuildingKind::RoduroHome => {
            // Grown stone: a lumpy banded mound, sometimes with an upper
            // storey budding out of it, and a warm window facing the hearth.
            let h = bd.size * (0.42 + unit(1) * 0.18) + sink;
            b.dome(base, r * (0.95 + unit(2) * 0.2), r * (0.85 + unit(3) * 0.2), h, 0.10, 0.10, s, palette::STONE);
            if bd.size > 9.5 {
                let off = vec3(unit(4) - 0.5, 0.0, unit(5) - 0.5) * r * 0.6;
                b.dome(base + off + vec3(0.0, h * 0.62, 0.0), r * 0.55, r * 0.5, h * 0.55, 0.12, 0.12, s ^ 7, palette::STONE);
            }
            let (sn, cs) = bd.rot.sin_cos();
            let face = vec3(base.x, ground, base.z) + vec3(cs, 0.0, sn) * (r * 0.97);
            gl.patch(face + vec3(0.0, h * 0.35, 0.0), 1.1, 0.8, bd.rot, palette::WINDOW);
            b.patch(face + vec3(0.0, 1.0, 0.0) + vec3(-sn, 0.0, cs) * 1.4, 1.0, 2.0, bd.rot, [0.08, 0.07, 0.06]);
        }
        BuildingKind::HoraroStilt => {
            let pillars = 5;
            for i in 0..pillars {
                let a = i as f32 / pillars as f32 * std::f32::consts::TAU + bd.rot;
                let p = base + vec3(a.cos(), 0.0, a.sin()) * (r * 0.55) + vec3(0.0, -2.0, 0.0);
                b.column(p, 0.45, 0.35, DECK + 2.0, 6, palette::STONE);
            }
            b.column(base + vec3(0.0, DECK, 0.0), r * 0.95, r * 0.95, 0.3, 12, palette::TIMBER);
            let off = vec3(bd.rot.cos(), 0.0, bd.rot.sin()) * (r * 0.12);
            b.dome(base + off + vec3(0.0, DECK + 0.3, 0.0), r * 0.62, r * 0.5, r * 0.62, 0.04, 0.0, s, palette::WEAVE);
            // A lamp in the doorway.
            let (sn, cs) = bd.rot.sin_cos();
            gl.patch(base + vec3(cs, 0.0, sn) * (r * 0.6) + vec3(0.0, DECK + 1.2, 0.0), 0.6, 0.6, bd.rot, palette::WINDOW);
        }
        BuildingKind::QotiroBlock => {
            let tiers = 2 + (unit(1) * 1.5) as usize;
            let mut y = 0.0;
            let mut sz = bd.size;
            for tier in 0..tiers {
                let h = 4.0 + unit(10 + tier as u64) * 1.5 + if tier == 0 { sink } else { 0.0 };
                b.block(base + vec3(0.0, y, 0.0), sz, sz * 0.8, h, bd.rot, palette::QUARRIED);
                // A lit slit window on each tier.
                let (sn, cs) = bd.rot.sin_cos();
                gl.patch(base + vec3(0.0, y + h * 0.55, 0.0) + vec3(cs, 0.0, sn) * (sz * 0.5 + 0.02), 0.5, 1.0, bd.rot, palette::WINDOW);
                y += h;
                sz *= 0.72;
            }
        }
        BuildingKind::QotiroTemple => {
            let mut y = 0.0;
            for (f, h) in [(1.0, 7.0 + sink), (0.68, 8.0), (0.40, 3.0)] {
                b.block(base + vec3(0.0, y, 0.0), bd.size * f, bd.size * f, h, bd.rot, palette::QUARRIED);
                y += h;
            }
            b.block(base + vec3(0.0, y, 0.0), bd.size * 0.16, bd.size * 0.2, 6.0, bd.rot, palette::QUARRIED);
            b.column(base + vec3(0.0, y + 6.0, 0.0), bd.size * 0.06, 0.0, 3.0, 8, palette::METAL_GOLD);
            gl.patch(base + vec3(0.0, y + 9.5, 0.0), 2.6, 2.6, bd.rot, palette::METAL_GOLD);
        }
        BuildingKind::QotiroHall => {
            b.block(base, bd.size, bd.size * 0.8, 4.5 + sink, bd.rot, palette::STONE);
            b.block(base + vec3(0.0, 4.5 + sink, 0.0), bd.size * 0.5, bd.size * 0.45, 3.0, bd.rot, palette::STONE);
            b.column(base + vec3(0.0, 7.5 + sink, 0.0), 0.9, 0.0, 2.2, 8, palette::METAL_GOLD);
        }
        BuildingKind::Hearth => {
            let base = to3(bd.pos, ground);
            for i in 0..9 {
                let a = i as f32 / 9.0 * std::f32::consts::TAU;
                b.block(base + vec3(a.cos(), -0.2, a.sin()) * 2.4, 0.9, 0.7, 0.75, a, palette::STONE);
            }
            gl.column(base + vec3(0.0, -0.2, 0.0), 1.4, 0.3, 1.4, 8, palette::EMBER);
        }
    }
}

/// A plain upright shape for someone far off.
fn simple_person(b: &mut Builder, base: Vec3, race: Race, k: f32) {
    let (h, r) = body_size(race);
    b.column(base, r * k, r * 0.6 * k, h * k, 4, race_color(race));
}

fn body_size(race: Race) -> (f32, f32) {
    match race {
        Race::Roduro => (1.35, 0.34), // short and stocky
        Race::Qotiro => (1.80, 0.34), // tall, broad
        Race::Horaro => (1.65, 0.27),
        Race::Tadoro => (1.95, 0.22), // tall and lean
    }
}

/// Draw one person; returns where their head is, for hover.
#[allow(clippy::too_many_arguments)]
fn person(b: &mut Builder, gl: &mut Builder, fl: &mut Builder, w: &World, pid: PersonId, k: f32, eye: Vec3, floor_at: &dyn Fn(V2) -> Option<f32>, ground: &dyn Fn(V2) -> f32) -> (Vec3, PersonId) {
    let p = &w.people[pid as usize];
    let at = w.person_pos(pid);
    // Indoors, they and their rings stand on the floor (looked up once).
    let inner = floor_at(at);
    let on_ground = &|q: V2| inner.unwrap_or_else(|| ground(q));
    // Horaro at home on a stilt deck stand above the water.
    let floor = if geo::inland(at) < 0.0 { DECK + 0.3 } else { on_ground(at) };
    let (h, r) = body_size(p.race);
    let (h, r) = (h * k, r * k);
    let base = to3(at, floor - 0.1);
    let far = base.distance(eye) > PERSON_SIMPLE * k.max(1.0);
    // Bandits and anyone you're fighting get a red mark at their feet.
    let foe = p.bandit || w.fighter(pid).map(|f| f.side != SQUAD_SIDE).unwrap_or(false);
    if foe && !far {
        // A faint red ring with a tip the way they face.
        facing_ring(fl, on_ground, at, 0.9 * k, 0.04 * k, super::cues::facing(w, pid, at), [0.48, 0.16, 0.13], eye, true);
    }
    let down = w.fighter(pid).map(|f| f.ko || f.dead).unwrap_or(false) || p.dead || body::knocked_out(&p.wounds.hp_at(&p.stats, w.time));
    // Carried: across the carrier's shoulders.
    if w.carried_by(pid).is_some() {
        let rot = (p.seed % 628) as f32 / 100.0;
        let lift = base + vec3(0.0, 1.25 * k, 0.0);
        b.block(lift, h * 0.85, r * 1.5, r * 1.1, rot, palette::scale(race_color(p.race), 0.8));
        b.block(lift + vec3(rot.cos(), 0.0, rot.sin()) * (h * 0.45), r * 1.0, r * 1.0, r * 1.0, rot, palette::skin(p.race));
        return (lift + vec3(0.0, r * 1.5, 0.0), pid);
    }
    // Asleep: lying down under a blanket (knocked out is drawn the same way
    // but without it).
    let asleep = !down && p.cond.as_ref().is_some_and(|c| c.activity == gahturiyu_sim::sim::condition::Activity::Sleeping);
    if asleep {
        let rot = (p.seed % 628) as f32 / 100.0;
        b.block(base, h, r * 2.0, r * 1.2, rot, palette::scale(race_color(p.race), 0.7));
        b.block(base + vec3(rot.cos(), 0.0, rot.sin()) * (h * 0.55), r * 1.0, r * 1.1, r * 1.0, rot, palette::skin(p.race));
        b.block(base + vec3(0.0, r * 1.15, 0.0) - vec3(rot.cos(), 0.0, rot.sin()) * (h * 0.08), h * 0.8, r * 2.3, r * 0.25, rot, [0.45, 0.42, 0.62]);
        return (base + vec3(0.0, r * 1.5, 0.0), pid);
    }
    if down {
        let rot = (p.seed % 628) as f32 / 100.0;
        b.block(base, h, r * 2.0, r * 1.2, rot, palette::scale(race_color(p.race), 0.7));
        b.block(base + vec3(rot.cos(), 0.0, rot.sin()) * (h * 0.55), r * 1.0, r * 1.1, r * 1.0, rot, palette::skin(p.race));
        return (base + vec3(0.0, r * 1.5, 0.0), pid);
    }
    // A meal runner's stack of pots; the dawn boat under its crew; divers
    // in the water.
    let doing = w.doing_now(pid).map(|d| d.0);
    let diving = doing == Some(Doing::Work) && geo::inland(at) < 0.0 && w.workplace_of(pid).map(|p| p.kind.offshore()).unwrap_or(false);
    let base = if diving { to3(at, -0.9) } else { base };
    let base = if doing == Some(Doing::Ferry) {
        let rot = (p.seed % 628) as f32 / 100.0;
        let hull = to3(at, 0.05);
        b.block(hull, 5.2 * k.min(3.0), 1.7 * k.min(3.0), 0.7, rot, palette::TIMBER);
        b.block(hull + vec3(0.0, 0.7, 0.0), 1.2, 1.0, 0.5, rot + 0.3, [0.30, 0.42, 0.50]);
        // A lantern on a pole at the bow (drawn only; it lights nothing).
        let bow = hull + vec3(rot.cos(), 0.0, rot.sin()) * (2.3 * k.min(3.0));
        b.stick(bow, bow + vec3(0.0, 2.0, 0.0), 0.08, palette::TIMBER);
        fl.column(bow + vec3(0.0, 2.0, 0.0), 0.18, 0.12, 0.3, 6, palette::WINDOW);
        hull + vec3(0.0, 0.6, 0.0)
    } else {
        base
    };
    if doing == Some(Doing::Run) {
        // Stacked tiffin pots on the head, and a crate of them on each hip.
        for i in 0..4 {
            b.column(base + vec3(0.0, h + 0.02 + i as f32 * 0.3 * k, 0.0), 0.4 * k, 0.36 * k, 0.27 * k, 8, if i % 2 == 0 { [0.62, 0.40, 0.22] } else { palette::METAL_GOLD });
        }
        for side in [-1.0f32, 1.0] {
            b.block(base + vec3(side * r * 1.6, h * 0.35, 0.0), 0.5 * k, 0.4 * k, 0.4 * k, 0.0, [0.55, 0.36, 0.22]);
        }
    }
    if far {
        b.column(base, r, r * 0.6, h, 4, race_color(p.race));
        return (base + vec3(0.0, h, 0.0), pid);
    }
    // Sneaking: bent over, low, head forward.
    let face = super::cues::facing(w, pid, at);
    let fwd = vec3(face.cos(), 0.0, face.sin());
    let crouch = w.squad.index(pid).is_some() && w.is_sneaking(pid);
    let body = if crouch { h * 0.5 } else { h * 0.8 };
    b.column(base, r, r * 0.8, body, 6, race_color(p.race));
    let head = base + vec3(0.0, body, 0.0) + if crouch { fwd * (r * 0.9) } else { Vec3::ZERO };
    b.block(head, r * 1.1, r * 1.1, h * 0.2, face, palette::skin(p.race));
    if crouch {
        // A hunched back between hips and head.
        b.block(base + vec3(0.0, body * 0.75, 0.0) + fwd * (r * 0.45), r * 1.6, r * 1.5, body * 0.35, face, palette::scale(race_color(p.race), 0.85));
    }
    super::cues::draw(b, gl, w, pid, base, if crouch { h * 0.75 } else { h }, r, face, k.min(2.0));
    // A lit torch, held up beside them.
    if w.torch_lit(pid) {
        let hand = base + vec3(r * 1.3, h * 0.55, 0.0);
        b.stick(hand, hand + vec3(0.05, 0.5 * k, 0.0), 0.06 * k, palette::TIMBER);
    }
    if let Some(f) = w.fighter(pid) {
        if f.paralyzed() {
            b.patch(head + vec3(0.0, h * 0.35, 0.0), r * 2.5, r * 0.6, 0.0, [0.7, 0.4, 1.0]);
        }
        if f.has(Does::Barrier).is_some() {
            ring_widened(fl, on_ground, at, 1.3 * k, 0.06 * k, 16, [0.5, 0.75, 1.0], eye, 0.004);
        }
    }
    (head + vec3(0.0, h * 0.2, 0.0), pid)
}

const RITUAL: [f32; 3] = [0.78, 0.6, 1.0];

/// Draw what magic leaves about: called-up creatures and raised dead in
/// fights, spells on patches of ground, drawn circles, rituals being
/// performed, rituals held ready, and the living sensed through walls.
#[allow(clippy::too_many_arguments)]
fn magic_scene(w: &World, b: &mut Builder, gl: &mut Builder, fl: &mut Builder, on_ground: &dyn Fn(V2) -> f32, centre: V2, radius: f32, k: f32, eye: Vec3, bars: &mut Vec<(Vec3, f32, Option<f32>, bool)>) {
    let kk = k.min(4.0);
    let near = |p: V2| p.dist(centre) < radius;
    let ground_mark = |fl: &mut Builder, does: Does, pos: V2, r: f32| {
        let (col, width) = match does {
            Does::Veil => ([0.75, 0.7, 0.95], 0.25),
            Does::Sanctuary => ([1.0, 0.85, 0.4], 0.35),
            Does::Tripwire => ([0.95, 0.35, 0.3], 0.1),
            Does::Blight => ([0.35, 0.55, 0.2], 0.5),
            _ => return,
        };
        draped_ring(fl, on_ground, pos, r, width, 48, col, eye);
        if does == Does::Blight {
            for q in [0.35, 0.7] {
                draped_ring(fl, on_ground, pos, r * q, width, 24, [0.25, 0.4, 0.12], eye);
            }
        }
    };
    let orb = |gl: &mut Builder, pos: V2, h: f32, r: f32, col: Rgb| {
        gl.dome(to3(pos, on_ground(pos) + h), r, r, r * 1.6, 0.0, 0.0, 3, col);
    };
    // Fighters on fire, frozen or soaked (drawing only: the fire's light
    // comes from the sim's own list of lights).
    for battle in &w.battles {
        for f in battle.fighters.iter().filter(|f| !f.dead && !f.fled && near(f.pos)) {
            let g = on_ground(f.pos);
            if f.has(Does::Burning).is_some() {
                let flick = (battle.time * 9.0 + f.pos.x as f64).sin() as f32 * 0.08;
                for (dx, dy, h, r) in [(0.0, 0.0, 1.1, 0.32), (0.18, 0.1, 1.5, 0.22), (-0.15, -0.12, 0.8, 0.25)] {
                    gl.dome(to3(f.pos.add(V2::new(dx, dy)), g + h + flick), r, r, r * 2.2, 0.0, 0.0, 3, [1.0, 0.5, 0.12]);
                }
            }
            if f.has(Does::Frozen).is_some() {
                b.dome(to3(f.pos, g), 0.55, 0.55, 2.0, 0.0, 0.0, 4, [0.72, 0.86, 0.98]);
            } else if f.has(Does::Wet).is_some() {
                gl.dome(to3(f.pos, g + 2.0), 0.08, 0.08, 0.12, 0.0, 0.0, 2, [0.4, 0.6, 0.95]);
            }
        }
    }
    for battle in &w.battles {
        for (i, f) in battle.fighters.iter().enumerate() {
            let Some(sm) = f.summon else { continue };
            if f.fled || f.dead || f.ko || !near(f.pos) {
                continue;
            }
            let base = to3(f.pos, on_ground(f.pos));
            let top = match sm.kind {
                Summon::SpiritBeast => {
                    gl.dome(base + vec3(0.0, 0.3 * kk, 0.0), 0.95 * kk, 0.45 * kk, 0.75 * kk, 0.2, 0.0, i as u64, [0.4, 0.9, 0.85]);
                    gl.dome(base + vec3(0.7 * kk, 0.7 * kk, 0.0), 0.35 * kk, 0.3 * kk, 0.4 * kk, 0.1, 0.0, i as u64 + 7, [0.55, 1.0, 0.95]);
                    1.3
                }
                Summon::Swarmling => {
                    gl.dome(base + vec3(0.0, 0.25 * kk, 0.0), 0.28 * kk, 0.28 * kk, 0.3 * kk, 0.3, 0.0, i as u64, [0.6, 0.35, 0.8]);
                    0.7
                }
                Summon::Decoy => {
                    gl.column(base, 0.3 * kk, 0.24 * kk, 1.7 * kk, 6, palette::scale(race_color(f.race), 0.6));
                    2.0
                }
                Summon::Guardian => {
                    b.column(base, 0.65 * kk, 0.5 * kk, 2.4 * kk, 6, palette::STONE);
                    b.block(base + vec3(0.0, 2.4 * kk, 0.0), 0.7 * kk, 0.7 * kk, 0.5 * kk, 0.0, palette::scale(palette::STONE, 0.8));
                    gl.patch(base + vec3(0.0, 2.65 * kk, 0.36 * kk), 0.4 * kk, 0.1 * kk, 0.0, [1.0, 0.8, 0.4]);
                    3.1
                }
                Summon::Thrall => {
                    b.column(base, 0.3 * kk, 0.24 * kk, 1.4 * kk, 6, [0.4, 0.45, 0.36]);
                    b.block(base + vec3(0.0, 1.4 * kk, 0.0), 0.32 * kk, 0.32 * kk, 0.3 * kk, 0.0, [0.55, 0.6, 0.5]);
                    1.9
                }
            };
            // Whose it is: a ring at its feet, teal for yours, red for theirs.
            let mark = if f.side == SQUAD_SIDE { [0.45, 0.9, 0.85] } else { [0.9, 0.2, 0.15] };
            ring_widened(fl, on_ground, f.pos, 0.9 * kk, 0.06 * kk, 14, mark, eye, 0.004);
            if base.distance(eye) < 400.0 {
                bars.push((base + vec3(0.0, top * kk, 0.0), f.vitality(), None, false));
            }
        }
        for z in battle.zones.iter().filter(|z| z.until > battle.time && near(z.pos)) {
            ground_mark(fl, z.does, z.pos, z.radius);
            if z.does == Does::Glow {
                orb(gl, z.pos, 2.2, 0.35, [1.0, 0.95, 0.75]);
            }
        }
    }
    // Spells on the ground out in the world.
    for wd in w.wards.iter().filter(|x| x.until > w.time && near(x.pos)) {
        ground_mark(fl, wd.does, wd.pos, wd.radius);
        match wd.does {
            Does::Glow => orb(gl, wd.pos, 2.2, 0.35, [1.0, 0.95, 0.75]),
            Does::Scout => orb(gl, wd.pos, 3.0, 0.3, [0.5, 0.95, 1.0]),
            Does::Summon(_) => {
                let base = to3(wd.pos, on_ground(wd.pos));
                b.column(base, 0.5, 0.4, 1.2, 6, palette::scale(palette::STONE, 0.9));
                draped_ring(fl, on_ground, wd.pos, 1.6, 0.12, 20, [1.0, 0.8, 0.4], eye);
            }
            _ => {}
        }
    }
    // Circles drawn for rituals.
    for &c in w.circles.iter().filter(|c| near(**c)) {
        draped_ring(fl, on_ground, c, 1.8, 0.09, 28, [0.92, 0.9, 0.84], eye);
        draped_ring(fl, on_ground, c, 1.5, 0.05, 28, [0.92, 0.9, 0.84], eye);
    }
    // A ritual being performed: a ring of violet that fills as it goes.
    for j in &w.rituals {
        let at = w.person_pos(j.who);
        let f = ((w.time - j.started) / (j.done_at - j.started)).clamp(0.0, 1.0) as f32;
        draped_ring(fl, on_ground, at, 1.4 * k.min(2.0), 0.12 + 0.2 * f, 24, RITUAL, eye);
    }
    // A ritual held ready: a small violet light over the holder.
    for (&pid, _) in w.held.iter() {
        let at = w.person_pos(pid);
        if near(at) {
            orb(gl, at, 2.4 * kk, 0.14 * kk, RITUAL);
        }
    }
    // A pack spirit: a laden shape of light at its bearer's shoulder.
    for &m in &w.squad.members {
        if w.boon(m, Does::Carry) > 0.0 && w.boon(m, Does::Attr(gahturiyu_sim::sim::stats::Attr::Strength)) <= 0.0 {
            let at = w.person_pos(m).add(V2::new(-1.1, 0.6));
            if near(at) {
                let base = to3(at, on_ground(at) + 0.5 * kk);
                gl.dome(base, 0.4 * kk, 0.35 * kk, 0.5 * kk, 0.1, 0.0, m as u64, [0.45, 0.9, 0.85]);
                b.block(base + vec3(0.0, 0.45 * kk, 0.0), 0.45 * kk, 0.35 * kk, 0.3 * kk, 0.3, palette::CAMP_HIDE);
            }
        }
    }
    // Sense life: the living near whoever senses them, marked through walls.
    for &m in &w.squad.members {
        let sense = w.boon(m, Does::SenseLife);
        if sense <= 0.0 {
            continue;
        }
        let from = w.person_pos(m);
        for (pid, p) in w.people.iter().enumerate() {
            let pid = pid as PersonId;
            if p.dead || pid == m {
                continue;
            }
            let at = w.person_pos(pid);
            if at.dist(from) <= sense {
                orb(gl, at, 2.6, 0.18, [0.55, 1.0, 0.55]);
            }
        }
    }
}

/// Where a lit torch's flame is, for someone holding one.
pub fn torch_flame(w: &World, pid: PersonId, k: f32, on_ground: &dyn Fn(V2) -> f32) -> Vec3 {
    let p = &w.people[pid as usize];
    let at = w.person_pos(pid);
    let (h, r) = body_size(p.race);
    let base = to3(at, on_ground(at) - 0.1);
    base + vec3(r * k * 1.3 + 0.05, h * k * 0.55 + 0.5 * k + 0.1, 0.0)
}

/// A Ṭaḍoro travelling tent: a taut cone of pale cloth around one pole.
fn tent(b: &mut Builder, base: Vec3, k: f32, seed: u64) {
    let r = 2.2 * k.min(4.0);
    b.column(base - vec3(0.0, 0.2, 0.0), r, 0.05, r * 1.5, 10, palette::TENT);
    let tilt = (seed % 7) as f32 * 0.1;
    b.column(base + vec3(0.0, r * 1.5 - 0.2, 0.0), 0.06 * k, 0.04 * k, 0.6 * k + tilt, 4, palette::TIMBER);
}

/// A workplace: fields in rows, stalls with awnings, a post, a dock — simple
/// shapes to say what's done where.
fn workplace(b: &mut Builder, gl: &mut Builder, t: &Terrain, w: &World, wp: &Workplace, on_ground: &dyn Fn(V2) -> f32) {
    let k = wp.kind;
    let size = k.size();
    let wet = geo::inland(wp.pos) < 0.0;
    let ground = if wet { 0.0 } else { on_ground(wp.pos) };
    let sink = if wet { 0.0 } else { (t.slope(wp.pos) * size * 0.4).min(3.0) };
    let base = to3(wp.pos, ground - sink);
    let rot = wp.rot;
    let (sn, cs) = rot.sin_cos();
    let along = V2::new(cs, sn);
    let across = V2::new(-sn, cs);
    let u = |n: u64| (gahturiyu_sim::sim::rng::key(&[wp.seed, n]) >> 40) as f32 / (1u64 << 24) as f32;
    let awning = |b: &mut Builder, at: Vec3, w: f32, col: Rgb, r: f32| {
        b.block(at, w, w * 0.7, 1.0, r, palette::TIMBER);
        for (dx, dz) in [(-0.45, -0.3), (0.45, -0.3), (-0.45, 0.3), (0.45, 0.3)] {
            let (s2, c2) = r.sin_cos();
            let off = vec3(c2 * dx * w - s2 * dz * w, 0.0, s2 * dx * w + c2 * dz * w);
            b.stick(at + off, at + off + vec3(0.0, 2.4, 0.0), 0.12, palette::TIMBER);
        }
        b.block(at + vec3(0.0, 2.4, 0.0), w * 1.1, w * 0.85, 0.15, r, col);
    };
    let banner = |b: &mut Builder, at: Vec3, col: Rgb| {
        b.stick(at, at + vec3(0.0, 4.5, 0.0), 0.15, palette::TIMBER);
        b.patch(at + vec3(0.35, 3.6, 0.0), 0.6, 1.2, rot + 1.57, col);
    };
    match k {
        PlaceKind::Fields => {
            // Rows of crops, laid on the land.
            let ripe = u(1);
            let (a, c2) = (palette::mix([0.36, 0.50, 0.22], [0.66, 0.60, 0.30], ripe), [0.40, 0.33, 0.22]);
            let rows = (size / 2.4) as i32;
            for i in 0..rows {
                let off = across.scale((i as f32 - rows as f32 * 0.5) * 2.4);
                let p0 = wp.pos.add(off).sub(along.scale(size * 0.5));
                let p1 = wp.pos.add(off).add(along.scale(size * 0.5));
                draped_ribbon(b, on_ground, p0, p1, 1.5, 0.12, if i % 2 == 0 { a } else { c2 }, 6.0);
            }
        }
        PlaceKind::Wilds => {
            // A hunters' lean-to and a drying rack.
            b.block(base, 3.0, 2.2, 1.6, rot, palette::WEAVE);
            for i in 0..3 {
                let p = base + vec3(cs, 0.0, sn) * (3.0 + i as f32 * 1.1);
                b.stick(p, p + vec3(0.0, 1.8, 0.0), 0.1, palette::TIMBER);
            }
            b.stick(base + vec3(cs, 0.0, sn) * 3.0 + vec3(0.0, 1.7, 0.0), base + vec3(cs, 0.0, sn) * 5.2 + vec3(0.0, 1.7, 0.0), 0.08, palette::TIMBER);
        }
        PlaceKind::Woodlot => {
            for i in 0..5 {
                let p = base + vec3(-sn, 0.0, cs) * (i as f32 * 0.55);
                b.stick(p - vec3(cs, 0.0, sn) * 2.0 + vec3(0.0, 0.3, 0.0), p + vec3(cs, 0.0, sn) * 2.0 + vec3(0.0, 0.3, 0.0), 0.5, palette::TIMBER);
            }
        }
        PlaceKind::Quarry => {
            // Cut blocks and a heap of spoil, with a timber hoist.
            for i in 0..6 {
                let a = i as f32 * 1.1 + u(2) * 6.0;
                b.block(base + vec3(a.cos(), 0.0, a.sin()) * (4.0 + u(9 + i) * 3.0), 1.6, 1.3, 1.0 + u(3 + i) * 0.8, a, palette::STONE);
            }
            b.dome(base, 4.0, 3.5, 2.2, 0.0, 0.0, 6, [0.42, 0.38, 0.34]);
            b.stick(base + vec3(4.0, 0.0, 0.0), base + vec3(4.0, 4.0, 0.0), 0.15, palette::TIMBER);
            b.stick(base + vec3(4.0, 4.0, 0.0), base + vec3(1.5, 3.6, 0.0), 0.12, palette::TIMBER);
        }
        PlaceKind::CharcoalPit => {
            // A turf-covered stack, smouldering.
            b.dome(base, 2.6, 2.6, 1.8, 0.0, 0.0, 8, [0.25, 0.22, 0.18]);
            gl.column(base + vec3(0.0, 1.7, 0.0), 0.25, 0.1, 0.4, 6, palette::EMBER);
            for i in 0..3 {
                let p = base + vec3(4.0, 0.0, -1.0 + i as f32);
                b.stick(p - vec3(1.2, 0.0, 0.0) + vec3(0.0, 0.25, 0.0), p + vec3(1.2, 0.0, 0.0) + vec3(0.0, 0.25, 0.0), 0.25, palette::TIMBER);
            }
        }
        PlaceKind::Dock => {
            // Planks out over the water on posts.
            let start = wp.pos.add(V2::new(2.0, 0.0));
            let end = wp.pos.add(V2::new(-16.0, 0.0));
            for i in 0..7 {
                let p = start.lerp(end, i as f32 / 6.0);
                b.column(to3(p, -1.5), 0.25, 0.25, 2.8, 5, palette::TIMBER);
            }
            b.block(to3(start.lerp(end, 0.5), 1.25), 18.0, 3.0, 0.2, 0.0, palette::TIMBER);
        }
        PlaceKind::DivePlatform | PlaceKind::Deck if wet => {
            for i in 0..6 {
                let a = i as f32 / 6.0 * std::f32::consts::TAU;
                b.column(base + vec3(a.cos(), 0.0, a.sin()) * (size * 0.4) + vec3(0.0, -2.0, 0.0), 0.4, 0.3, DECK + 2.0, 6, palette::STONE);
            }
            b.column(base + vec3(0.0, DECK, 0.0), size * 0.55, size * 0.55, 0.3, 10, palette::TIMBER);
            if k == PlaceKind::Deck {
                // The village's cooking fire.
                gl.column(base + vec3(0.0, DECK + 0.3, 0.0), 0.6, 0.2, 0.6, 6, palette::EMBER);
                b.column(base + vec3(1.3, DECK + 0.3, 0.0), 0.5, 0.45, 0.6, 8, [0.45, 0.32, 0.22]);
            } else {
                // A ladder down to the water, and a rope frame.
                b.stick(base + vec3(size * 0.55, DECK, 0.0), base + vec3(size * 0.6, -0.5, 0.0), 0.12, palette::TIMBER);
                b.stick(base + vec3(-1.5, DECK, 0.0), base + vec3(-1.5, DECK + 3.0, 0.0), 0.15, palette::TIMBER);
                b.stick(base + vec3(1.5, DECK, 0.0), base + vec3(1.5, DECK + 3.0, 0.0), 0.15, palette::TIMBER);
                b.stick(base + vec3(-1.5, DECK + 3.0, 0.0), base + vec3(1.5, DECK + 3.0, 0.0), 0.12, palette::TIMBER);
            }
        }
        PlaceKind::KelpBeds => {
            for i in 0..14 {
                let a = u(10 + i) * std::f32::consts::TAU;
                let r = u(40 + i) * size * 0.5;
                let p = base + vec3(a.cos(), 0.0, a.sin()) * r;
                b.stick(p + vec3(0.0, -0.3, 0.0), p + vec3(0.0, 1.0, 0.0), 0.08, palette::TIMBER);
                b.patch(p + vec3(0.0, 0.06, 0.0), 2.4, 1.0, a, [0.14, 0.26, 0.14]);
            }
        }
        PlaceKind::Boatyard => {
            // A hull turned over on trestles.
            for dx in [-1.6f32, 1.6] {
                b.block(base + vec3(cs, 0.0, sn) * dx, 0.3, 1.8, 1.0, rot, palette::TIMBER);
            }
            b.dome(base + vec3(0.0, 1.0, 0.0), 3.4, 1.1, 0.9, 0.02, 0.0, wp.seed, palette::TIMBER);
        }
        PlaceKind::Kitchen | PlaceKind::Deck => {
            // An open-sided cookhouse: a roof on posts over a big fire and pots.
            awning(b, base, 6.0, [0.42, 0.34, 0.26], rot);
            gl.column(base + vec3(0.0, 1.0, 0.0), 0.8, 0.25, 0.7, 7, palette::EMBER);
            for i in 0..3 {
                let a = i as f32 * 2.1;
                b.column(base + vec3(a.cos(), 1.0, a.sin()) * 1.8, 0.45, 0.4, 0.7, 8, [0.45, 0.32, 0.22]);
            }
        }
        PlaceKind::MessHall | PlaceKind::Hall => {
            b.block(base, size, size * 0.6, 4.0 + sink, rot, palette::QUARRIED);
            b.block(base + vec3(0.0, 4.0 + sink, 0.0), size * 1.05, size * 0.65, 0.4, rot, palette::TIMBER);
            gl.patch(base + vec3(cs, 0.0, sn) * (size * 0.5 + 0.02) + vec3(0.0, 2.0 + sink, 0.0), 1.6, 1.4, rot, palette::WINDOW);
            if k == PlaceKind::Hall {
                banner(b, base + vec3(cs, 0.0, sn) * (size * 0.5 + 1.0) + vec3(-sn, 0.0, cs) * 2.0, [0.65, 0.20, 0.18]);
            }
        }
        PlaceKind::Market => {
            for i in 0..5 {
                let a = i as f32 / 5.0 * std::f32::consts::TAU + u(1);
                let at = base + vec3(a.cos(), 0.0, a.sin()) * (size * 0.33);
                let col = [[0.70, 0.28, 0.22], [0.25, 0.40, 0.62], [0.75, 0.58, 0.22], [0.40, 0.55, 0.30], [0.55, 0.30, 0.55]][i];
                awning(b, at, 2.6, col, a + 1.57);
            }
        }
        PlaceKind::Shop(shelf) => {
            let col = match shelf {
                Shelf::Food => [0.70, 0.28, 0.22],
                Shelf::Materials => [0.25, 0.40, 0.62],
                Shelf::Goods => [0.75, 0.58, 0.22],
            };
            b.block(base, size * 0.8, size * 0.6, 3.2 + sink, rot, palette::STONE);
            awning(b, base + vec3(cs, 0.0, sn) * (size * 0.5), 2.8, col, rot + 1.57);
        }
        PlaceKind::Inn => {
            b.block(base, size, size * 0.75, 4.0 + sink, rot, palette::STONE);
            b.block(base + vec3(0.0, 4.0 + sink, 0.0), size * 0.85, size * 0.65, 3.2, rot, palette::TIMBER);
            for side in [-1.0f32, 1.0] {
                gl.patch(base + vec3(cs, 0.0, sn) * (size * 0.5 + 0.02) + vec3(-sn, 0.0, cs) * (side * 2.5) + vec3(0.0, 2.2 + sink, 0.0), 1.0, 1.0, rot, palette::WINDOW);
            }
            banner(b, base + vec3(cs, 0.0, sn) * (size * 0.5 + 1.2), palette::METAL_GOLD);
        }
        PlaceKind::Shrine => {
            b.block(base, 4.0, 4.0, 0.8 + sink, rot, palette::QUARRIED);
            b.column(base + vec3(0.0, 0.8 + sink, 0.0), 0.5, 0.3, 3.0, 8, palette::STONE);
            gl.patch(base + vec3(0.0, 4.2 + sink, 0.0), 1.2, 1.2, rot, palette::METAL_GOLD);
        }
        PlaceKind::GuardPost => {
            for (dx, dz) in [(-1.2, -1.2), (1.2, -1.2), (-1.2, 1.2), (1.2, 1.2)] {
                b.stick(base + vec3(dx, 0.0, dz), base + vec3(dx, 5.0, dz), 0.25, palette::TIMBER);
            }
            b.block(base + vec3(0.0, 4.4, 0.0), 3.4, 3.4, 0.3, rot, palette::TIMBER);
            b.column(base + vec3(0.0, 5.6, 0.0), 2.4, 0.0, 1.6, 4, palette::WEAVE);
            banner(b, base + vec3(2.4, 0.0, 0.0), [0.65, 0.20, 0.18]);
        }
        PlaceKind::HealingHouse | PlaceKind::HealersHouse => {
            b.block(base, size, size * 0.7, 3.6 + sink, rot, palette::STONE);
            gl.patch(base + vec3(cs, 0.0, sn) * (size * 0.5 + 0.02) + vec3(0.0, 2.0 + sink, 0.0), 1.0, 1.0, rot, [0.6, 1.0, 0.7]);
            banner(b, base + vec3(cs, 0.0, sn) * (size * 0.5 + 1.0), [0.30, 0.65, 0.40]);
        }
        PlaceKind::TeachingHouse | PlaceKind::ExchangeHouse | PlaceKind::LettersHouse => {
            b.block(base, size, size * 0.7, 3.8 + sink, rot, palette::STONE);
            b.block(base + vec3(0.0, 3.8 + sink, 0.0), size * 0.6, size * 0.5, 2.4, rot, palette::TENT);
            banner(b, base + vec3(cs, 0.0, sn) * (size * 0.5 + 1.0), [0.60, 0.70, 0.95]);
            if k != PlaceKind::TeachingHouse {
                gl.column(base + vec3(cs, 0.0, sn) * (size * 0.5 + 0.2) + vec3(0.0, 3.0 + sink, 0.0), 0.5, 0.5, 0.08, 10, palette::METAL_GOLD);
            }
        }
        PlaceKind::TendersYard => {
            for i in 0..5 {
                let a = i as f32 / 5.0 * std::f32::consts::TAU;
                b.dome(base + vec3(a.cos(), 0.0, a.sin()) * (size * 0.3), 1.2, 1.0, 1.0 + u(5 + i as u64) * 0.8, 0.1, 0.12, wp.seed ^ i as u64, palette::STONE);
            }
        }
        PlaceKind::Workyard | PlaceKind::Forge | PlaceKind::Bench | PlaceKind::Desk | PlaceKind::AlchemyTable | PlaceKind::WeaversShed | PlaceKind::Workshop => {
            if k == PlaceKind::Workyard {
                // A fenced yard.
                let half = size * 0.5;
                let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
                for i in 0..4 {
                    let (ax, az) = corners[i];
                    let (bx, bz) = corners[(i + 1) % 4];
                    let pa = wp.pos.add(along.scale(ax * half)).add(across.scale(az * half));
                    let pb = wp.pos.add(along.scale(bx * half)).add(across.scale(bz * half));
                    b.stick(to3(pa, on_ground(pa) + 0.8), to3(pb, on_ground(pb) + 0.8), 0.12, palette::TIMBER);
                }
                gl.column(base + vec3(-sn, 0.0, cs) * -2.0, 0.6, 0.3, 0.9, 6, palette::EMBER);
            } else {
                let col = if k == PlaceKind::Forge { palette::STONE } else { palette::QUARRIED };
                b.block(base, size * 0.8, size * 0.7, 3.0 + sink, rot, col);
                if k == PlaceKind::Forge {
                    b.block(base + vec3(-sn, 0.0, cs) * 1.5, 0.9, 0.9, 5.0 + sink, rot, palette::STONE);
                    gl.patch(base + vec3(cs, 0.0, sn) * (size * 0.4 + 0.02) + vec3(0.0, 1.2 + sink, 0.0), 1.2, 0.9, rot, palette::EMBER);
                } else if k == PlaceKind::AlchemyTable {
                    gl.patch(base + vec3(cs, 0.0, sn) * (size * 0.4 + 0.02) + vec3(0.0, 1.8 + sink, 0.0), 0.8, 0.8, rot, [0.5, 1.0, 0.6]);
                }
            }
        }
        // (A land deck is drawn as a kitchen above; a wet one as a platform.)
        PlaceKind::DivePlatform => {}
    }
    // A signpost out front, to one side, saying what's done here: the first
    // of a few spots round the place where neither the post nor its board
    // stands in a building.
    if let Some(p) = super::signs::for_place(k) {
        let doors = w.doors_near(wp.pos, size + 8.0);
        let clear = |at: V2, face: f32| {
            let r = V2::new(-face.sin(), face.cos());
            [0.0f32, 1.2, 2.4].iter().all(|&o| {
                let q = at.add(r.scale(o));
                doors.iter().all(|d| !d.contains(q) && !d.contains(q.add(V2::new(0.6, 0.0))) && !d.contains(q.add(V2::new(-0.6, 0.0))) && !d.contains(q.add(V2::new(0.0, 0.6))) && !d.contains(q.add(V2::new(0.0, -0.6))))
            })
        };
        let front = size * 0.5 + 2.0;
        // (out along, out across, which way the board hangs)
        let spots = [(front, -size * 0.35, rot), (front, size * 0.35, rot + std::f32::consts::PI), (-front, -size * 0.35, rot), (-front, size * 0.35, rot + std::f32::consts::PI), (front + 3.0, -size * 0.35, rot), (front + 3.0, size * 0.35, rot + std::f32::consts::PI)];
        let pick = spots.iter().map(|&(a, c, f)| (wp.pos.add(along.scale(a)).add(across.scale(c)), f)).find(|&(at, f)| clear(at, f));
        let (at, face) = pick.unwrap_or((wp.pos.add(along.scale(front)).add(across.scale(-size * 0.35)), rot));
        super::signs::signpost(b, gl, p, to3(at, on_ground(at)), face);
    }
}

/// A rock placed by hand: faceted, lumpy stone of its kind, its shape fixed
/// by its own seed.
fn rock(b: &mut Builder, k: &gahturiyu_sim::sim::mapedit::Rock, ground: f32, slope: f32) {
    let seed = k.seed as u64;
    let n = |i: u64| -> f32 { ((seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15)).wrapping_mul(0xBF58_476D_1CE4_E5B9) >> 40) as f32 / (1u64 << 24) as f32 };
    let tone = 0.8 + n(99) * 0.35;
    let col = [0.46 * tone, 0.45 * tone, 0.42 * tone];
    let s = k.size;
    // (width, depth, height above ground, roughness) by kind:
    // boulder, slab, pillar, scree, outcrop.
    let (rx, rz, h, lump) = match k.kind {
        0 => (s * 0.6, s * 0.5, s * 0.6, 0.35),
        1 => (s * 0.75, s * 0.5, s * 0.25, 0.2),
        2 => (s * 0.35, s * 0.3, s * 1.8, 0.25),
        3 => (s * 0.5, s * 0.45, s * 0.4, 0.4),
        _ => (s * 0.6, s * 0.45, s * 0.75, 0.45),
    };
    let base = vec3(k.pos.x, ground - (slope * s * 0.4).min(s * 0.3), k.pos.y);
    let parts: Vec<(Vec3, f32, f32, f32, u64)> = match k.kind {
        // Scree: a little heap of small stones.
        3 => (0..5u64)
            .map(|j| {
                let a = n(j * 7 + 1) * std::f32::consts::TAU;
                let d = n(j * 7 + 2) * s * 1.3;
                let r = s * (0.3 + n(j * 7 + 3) * 0.4);
                (vec3(a.cos() * d, 0.0, a.sin() * d), r, r * 0.85, r * 0.6, seed ^ j)
            })
            .collect(),
        // An outcrop: a few big stones leaning together.
        4 => (0..3u64)
            .map(|j| {
                let f = 0.6 + n(j * 5 + 3) * 0.5;
                (vec3((n(j * 5 + 1) - 0.5) * rx, 0.0, (n(j * 5 + 2) - 0.5) * rz), rx * f, rz * f, h * (0.7 + n(j * 5 + 4) * 0.6), seed ^ (j + 11))
            })
            .collect(),
        _ => vec![(Vec3::ZERO, rx, rz, h, seed)],
    };
    let (sn, cs) = k.rot.sin_cos();
    for (off, rx, rz, h, sd) in parts {
        stone(b, base + vec3(off.x * cs - off.z * sn, 0.0, off.x * sn + off.z * cs), rx, rz, h, lump, k.rot, sd, col);
    }
}

/// One faceted stone: a lumpy ball, its lower part sunk in the ground,
/// flat-shaded so its faces catch the light.
#[allow(clippy::too_many_arguments)]
fn stone(b: &mut Builder, centre: Vec3, rx: f32, rz: f32, h: f32, lump: f32, rot: f32, seed: u64, col: Rgb) {
    let (rings, sides) = (5usize, 8usize);
    let noise = |i: usize, k: usize| -> f32 {
        let x = (seed ^ ((i as u64) << 20) ^ ((k % sides) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        ((x >> 40) as f32 / (1u64 << 24) as f32) - 0.5
    };
    let (sn, cs) = rot.sin_cos();
    // Latitude from a little below the ground up to the top.
    let point = |i: usize, k: usize| -> Vec3 {
        let lat = -0.35 + i as f32 / rings as f32 * (std::f32::consts::FRAC_PI_2 + 0.35);
        let lon = k as f32 / sides as f32 * std::f32::consts::TAU;
        let bump = if i == rings { 1.0 } else { 1.0 + noise(i, k) * lump };
        let (x, z) = (lat.cos() * lon.cos() * rx * bump, lat.cos() * lon.sin() * rz * bump);
        centre + vec3(x * cs - z * sn, lat.sin() * h * (1.0 + noise(i + 7, k) * lump * 0.5), x * sn + z * cs)
    };
    for i in 0..rings {
        for k in 0..sides {
            let q = [point(i, k), point(i, k + 1), point(i + 1, k + 1), point(i + 1, k)];
            let nrm = (q[2] - q[0]).cross(q[1] - q[3]).normalize_or_zero();
            let nrm = if nrm.y < -0.2 { -nrm } else { nrm };
            let shade = 0.88 + noise(i + 3, k + 5) * 0.25;
            b.quad(q, nrm, [col[0] * shade, col[1] * shade, col[2] * shade]);
        }
    }
}
