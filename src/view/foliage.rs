//! Grass, bushes and trees: decoration only. Nothing here changes the
//! simulation, and the plants that can be gathered stay as they are.
//!
//! **Where plants grow** is worked out from the world seed and the terrain,
//! the same every time: each plant has a cell on a fixed grid (grass every
//! `GRASS_GRID` metres, bushes `BUSH_GRID`, trees `TREE_GRID`) and grows
//! there or not by the ground under it — its height, slope, kind (the sim's
//! `Ground`), how near the coast it is, and how far into the dry Qotiro
//! plateau. Woods are where a slow noise is high. Nothing grows on roads, in
//! towns, round bandit camps or in water.
//!
//! **Drawing many plants cheaply.** Every plant of a kind shares one mesh and
//! one material, so the graphics card draws thousands of copies in one go
//! (instancing). Plants are loaded in square chunks round the camera and
//! dropped when it moves away.
//!
//! **Detail levels** (all distances named below):
//! - grass blades fade out by `GRASS_FADE`; the ground under them is green
//!   anyway, so fields still read as grass from afar;
//! - bushes are full up close, a simple shape further out, gone by
//!   `BUSH_SIMPLE_END`;
//! - trees are full up close, then a low-poly blob, then a flat picture that
//!   always faces the camera (a billboard), so woods stay visible to the
//!   horizon. Far billboards are gathered into one mesh per big chunk.
//! Changes between levels are dithered (a fine speckle that crossfades one
//! into the other) so nothing pops.
//!
//! **Wind.** Plants sway gently, worked out on the graphics card (a vertex
//! shader): each vertex carries how much it may move (its colour's alpha:
//! 0 at the root, more towards the tips).

use std::collections::HashMap;

use bevy::asset::{uuid_handle, RenderAssetUsages};
use bevy::camera::visibility::{NoFrustumCulling, VisibilityRange};
use bevy::image::{Image, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::mesh::Mesh;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::{Shader, ShaderRef};
use bevy_egui::egui;

use gahturiyu_sim::sim::{
    geo::{self, V2},
    rng::Rng,
    terrain::{self, Ground},
    World,
};

use super::app::{Game, View};
use super::cam::{to3, OrbitCam};
use super::hud::Canvas;
use super::mesh::Builder;
use super::models::{Models, MODEL_LOD1, MODEL_LOD2};
use super::palette::{self, Rgb};
use super::scene::Scene3d;
use super::settings::Settings;

// ---- Placement ----------------------------------------------------------------

/// Grid spacing for grass tufts, bushes and trees, metres.
pub const GRASS_GRID: f32 = 1.7;
pub const BUSH_GRID: f32 = 9.0;
pub const TREE_GRID: f32 = 13.0;
/// Size of the slow noise that decides where woods are, metres.
pub const WOOD_SCALE: f32 = 750.0;
/// Trees thin out above this height and stop by `TREE_LINE + 120`.
pub const TREE_LINE: f32 = 420.0;

// ---- Detail distances (from the camera, metres) --------------------------------

/// Grass fades out between these.
pub const GRASS_FADE: (f32, f32) = (60.0, 80.0);
/// Bushes: full detail until this, crossfading to the simple shape...
pub const BUSH_FULL_END: (f32, f32) = (110.0, 140.0);
/// ...which is gone by this.
pub const BUSH_SIMPLE_END: (f32, f32) = (260.0, 300.0);
/// Trees: full detail until this, crossfading to the blob...
pub const TREE_FULL_END: (f32, f32) = (150.0, 200.0);
/// ...which gives way to the billboard between these.
pub const TREE_BLOB_END: (f32, f32) = (650.0, 750.0);
/// Billboards are kept out to this far.
pub const BILLBOARD_FAR: f32 = 7000.0;

// ---- Chunks ------------------------------------------------------------------

const GRASS_CHUNK: f32 = 32.0;
const BUSH_CHUNK: f32 = 64.0;
const TREE_CHUNK: f32 = 128.0;
const FAR_CHUNK: f32 = 512.0;
/// Milliseconds a frame may spend building chunks, so moving the camera
/// doesn't stall (the rest wait for the next frame).
const CHUNK_MS: f32 = 4.0;

// ---- Wind --------------------------------------------------------------------

/// How far the tip of a plant sways, metres at full weight.
pub const WIND_SWAY: f32 = 0.35;

const SHADER: Handle<Shader> = uuid_handle!("6b1d6c0e-0d0f-4d5e-9a0b-6f2a3c1e7d51");

/// The wind (and billboard) shader, added to the standard material.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct Sway {}

impl MaterialExtension for Sway {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }
}

pub type PlantMat = ExtendedMaterial<StandardMaterial, Sway>;

fn shader_source() -> String {
    format!(
        r#"
#import bevy_pbr::{{
    mesh_functions,
    forward_io::{{Vertex, VertexOutput}},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::{{view, globals}},
}}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {{
    var out: VertexOutput;
    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    var wp = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    var weight = 0.0;
#ifdef VERTEX_COLORS
    weight = vertex.color.a;
    out.color = vec4<f32>(vertex.color.rgb, 1.0);
#endif
#ifdef VERTEX_UVS_B
    // A billboard: the quad's corners are offsets, turned to face the
    // camera, and it grows in as the blob it replaces fades out.
    let corner = vertex.uv_b;
    if (abs(corner.x) + abs(corner.y) > 0.0) {{
        let to_cam = view.world_position - wp.xyz;
        let side = normalize(vec3<f32>(-to_cam.z, 0.0, to_cam.x));
        let d = length(to_cam);
        let grow = smoothstep({bb_start:.1}, {bb_end:.1}, d);
        wp = vec4<f32>(wp.xyz + (side * corner.x + vec3<f32>(0.0, corner.y, 0.0)) * grow, 1.0);
    }}
#endif
    // Wind: two slow waves rolling across the land, and a quicker flutter.
    let t = globals.time;
    let phase = wp.x * 0.11 + wp.z * 0.07;
    let gust = sin(t * 0.9 + phase) * 0.6 + sin(t * 2.3 + phase * 1.7) * 0.3 + sin(t * 6.1 + phase * 3.1) * 0.1;
    wp = vec4<f32>(wp.xyz + vec3<f32>(0.8, 0.0, 0.5) * gust * weight * {sway:.3}, 1.0);

    out.world_position = wp;
    out.position = position_world_to_clip(wp.xyz);
#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
#endif
#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(vertex.instance_index, world_from_local[3]);
#endif
    return out;
}}
"#,
        bb_start = TREE_BLOB_END.0,
        bb_end = TREE_BLOB_END.1,
        sway = WIND_SWAY,
    )
}

// ---- What grows where -----------------------------------------------------------

/// Things plants keep clear of, near one chunk: the built-up middle of each
/// town, every building, bandit camps, and things to gather.
struct Clear {
    towns: Vec<(V2, f32)>,
    buildings: Vec<(V2, f32)>,
    camps: Vec<V2>,
    nodes: Vec<V2>,
}

/// Metres kept clear round the built-up middle of a town, and round each
/// building.
const TOWN_CLEAR: f32 = 15.0;
const BUILDING_CLEAR: f32 = 4.0;

impl Clear {
    fn near(w: &World, centre: V2, half: f32) -> Clear {
        let r = half * 1.5;
        let near = |p: V2, extra: f32| p.dist(centre) < r + extra;
        Clear {
            towns: w.settlements.iter().filter(|s| near(s.pos, s.radius() + TOWN_CLEAR)).map(|s| (s.pos, s.radius() + TOWN_CLEAR)).collect(),
            buildings: w.settlements.iter().filter(|s| near(s.pos, s.reach + 20.0)).flat_map(|s| s.buildings.iter()).filter(|b| near(b.pos, b.size + BUILDING_CLEAR)).map(|b| (b.pos, b.size * 0.6 + BUILDING_CLEAR)).collect(),
            camps: w.camps.iter().map(|c| c.pos).filter(|p| near(*p, 30.0)).collect(),
            nodes: w.nodes.iter().map(|n| n.pos).filter(|p| near(*p, 5.0)).collect(),
        }
    }
    fn ok(&self, p: V2) -> bool {
        self.towns.iter().all(|(c, r)| c.dist(p) > *r) && self.buildings.iter().all(|(c, r)| c.dist(p) > *r) && self.camps.iter().all(|c| c.dist(p) > 26.0) && self.nodes.iter().all(|c| c.dist(p) > 2.5)
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How wooded a place is, 0..1 (before ground and height).
fn wood(w: &World, p: V2) -> f32 {
    smooth(0.52, 0.66, terrain::noise(w.seed ^ 0xF0E5_7D11, p.x, p.y, WOOD_SCALE))
}

/// The ground at a plant's spot, if anything can grow there at all.
struct Spot {
    h: f32,
    slope: f32,
    arid: f32,
    shore: f32,
    ground: Ground,
}

fn spot(w: &World, p: V2) -> Option<Spot> {
    let t = &w.terrain;
    let shore = geo::inland(p);
    if !geo::is_land(p) || shore < 6.0 {
        return None;
    }
    let ground = t.ground(p);
    if ground == Ground::Road || t.on_road(p) {
        return None;
    }
    Some(Spot { h: t.height(p), slope: t.slope(p), arid: t.plateau(p), shore, ground })
}

/// A plant's place in its grid cell, and a roll for it, from the seed.
fn cell(w: &World, kind: u64, i: i64, j: i64, grid: f32) -> (V2, Rng) {
    let mut r = Rng::from_keys(&[w.seed, 0xF011_A6E0 ^ kind, i as u64, j as u64]);
    let p = V2::new((i as f32 + r.f32()) * grid, (j as f32 + r.f32()) * grid);
    (p, r)
}

fn grass_at(w: &World, p: V2, r: &mut Rng, density: f32) -> bool {
    let Some(s) = spot(w, p) else { return false };
    if s.ground != Ground::Grass {
        return false;
    }
    let chance = 0.8 * (1.0 - s.arid) * smooth(0.75, 0.45, s.slope) * smooth(620.0, 460.0, s.h) * smooth(6.0, 22.0, s.shore) * (1.0 - 0.5 * wood(w, p));
    let (m, add) = w.terrain.edits.plants_at(p)[0];
    r.f32() < (chance * m + add * smooth(0.9, 0.6, s.slope)) * density
}

fn bush_at(w: &World, p: V2, r: &mut Rng, density: f32) -> bool {
    let Some(s) = spot(w, p) else { return false };
    if !matches!(s.ground, Ground::Grass | Ground::Scrub | Ground::Dirt) {
        return false;
    }
    let wd = wood(w, p);
    let chance = (0.10 + 0.35 * wd * (1.0 - wd) * 4.0 * 0.5 + 0.25 * s.arid) * smooth(0.8, 0.5, s.slope) * smooth(720.0, 560.0, s.h) * smooth(6.0, 14.0, s.shore);
    let (m, add) = w.terrain.edits.plants_at(p)[1];
    r.f32() < (chance * m + add * smooth(0.9, 0.6, s.slope)) * density
}

/// Whether a tree grows in this cell, and if so which kind (0 broadleaf,
/// 1 conifer).
fn tree_at(w: &World, p: V2, r: &mut Rng, density: f32) -> Option<u8> {
    let s = spot(w, p)?;
    if !matches!(s.ground, Ground::Grass | Ground::Scrub | Ground::Dirt) {
        return None;
    }
    let wd = wood(w, p);
    let chance = (0.85 * wd + 0.025) * (1.0 - 0.95 * s.arid) * smooth(TREE_LINE + 120.0, TREE_LINE, s.h) * smooth(0.7, 0.4, s.slope) * smooth(15.0, 80.0, s.shore);
    let (m, add) = w.terrain.edits.plants_at(p)[2];
    if r.f32() >= (chance * m + add * smooth(0.8, 0.5, s.slope)) * density {
        return None;
    }
    // Conifers up the slopes, broadleaf below, mixed between.
    let conifer = r.f32() < smooth(160.0, 340.0, s.h) * 0.9 + 0.1 * wd;
    Some(conifer as u8)
}

// ---- Meshes ----------------------------------------------------------------------

/// Set each vertex's sway weight from its height (0 at the root).
fn sway_by_height(mut m: Mesh, top: f32, amount: f32) -> Mesh {
    let ys: Vec<f32> = match m.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(bevy::mesh::VertexAttributeValues::Float32x3(ps)) => ps.iter().map(|p| p[1]).collect(),
        _ => return m,
    };
    if let Some(bevy::mesh::VertexAttributeValues::Float32x4(cs)) = m.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for (c, y) in cs.iter_mut().zip(ys) {
            c[3] = (y / top).clamp(0.0, 1.0).powf(1.5) * amount;
        }
    }
    m
}

fn grass_mesh(r: &mut Rng, tint: f32) -> Mesh {
    let mut b = Builder::new();
    let base: Rgb = palette::scale([0.22, 0.34, 0.15], tint);
    let tip: Rgb = palette::scale([0.45, 0.58, 0.26], tint);
    for _ in 0..7 {
        let a = r.f32() * std::f32::consts::TAU;
        let (x, z) = (r.f32() * 0.5 - 0.25, r.f32() * 0.5 - 0.25);
        let h = 0.35 + r.f32() * 0.4;
        let lean = Vec3::new(a.cos(), 0.0, a.sin()) * (0.1 + r.f32() * 0.15);
        let side = Vec3::new(-a.sin(), 0.0, a.cos()) * 0.07;
        let root = Vec3::new(x, 0.0, z);
        let top = root + Vec3::Y * h + lean;
        let n = side.cross(top - root).normalize_or_zero();
        let (cb, ct) = (palette::lin(base), palette::lin(tip));
        b.quad_lin([root - side, root + side, top + side * 0.15, top - side * 0.15], [n; 4], [cb, cb, ct, ct]);
    }
    sway_by_height(b.mesh(), 0.75, 0.45)
}

fn bush_mesh(full: bool, tint: f32) -> Mesh {
    let mut b = Builder::new();
    let leaf = palette::scale([0.24, 0.36, 0.18], tint);
    if full {
        b.dome(Vec3::new(0.0, -0.1, 0.0), 1.0, 0.9, 1.1, 0.25, 0.0, 17, leaf);
        b.dome(Vec3::new(0.5, -0.1, 0.3), 0.7, 0.6, 0.8, 0.25, 0.0, 23, palette::scale(leaf, 1.12));
    } else {
        b.column(Vec3::new(0.0, -0.1, 0.0), 1.1, 0.4, 1.0, 6, leaf);
    }
    sway_by_height(b.mesh(), 1.2, 0.35)
}

/// A tree, about 9 m tall at scale 1. `kind` 0 broadleaf, 1 conifer;
/// `full` or the low-poly blob.
fn tree_mesh(kind: u8, full: bool, tint: f32) -> Mesh {
    let mut b = Builder::new();
    let bark: Rgb = [0.30, 0.22, 0.15];
    let sides = if full { 6 } else { 4 };
    if kind == 0 {
        let leaf = palette::scale([0.22, 0.34, 0.16], tint);
        b.column(Vec3::new(0.0, -0.2, 0.0), 0.28, 0.18, 4.2, sides, bark);
        if full {
            b.dome(Vec3::new(0.0, 3.6, 0.0), 2.6, 2.4, 3.4, 0.3, 0.0, 5, leaf);
            b.dome(Vec3::new(0.9, 4.6, 0.6), 1.8, 1.6, 2.8, 0.3, 0.0, 9, palette::scale(leaf, 1.1));
            b.dome(Vec3::new(-0.8, 5.0, -0.5), 1.6, 1.7, 2.6, 0.3, 0.0, 13, palette::scale(leaf, 0.92));
        } else {
            b.column(Vec3::new(0.0, 3.4, 0.0), 2.6, 1.4, 2.4, 6, leaf);
            b.column(Vec3::new(0.0, 5.8, 0.0), 1.4, 0.2, 1.8, 6, leaf);
        }
    } else {
        let leaf = palette::scale([0.15, 0.27, 0.17], tint);
        b.column(Vec3::new(0.0, -0.2, 0.0), 0.24, 0.12, 3.0, sides, bark);
        let tiers: &[(f32, f32, f32)] = if full { &[(1.6, 2.4, 3.2), (3.6, 1.9, 2.8), (5.6, 1.4, 2.6), (7.4, 0.9, 2.0)] } else { &[(1.6, 2.3, 4.0), (5.0, 1.4, 4.0)] };
        for &(y, r, h) in tiers {
            b.column(Vec3::new(0.0, y, 0.0), r, 0.05, h, if full { 10 } else { 6 }, leaf);
        }
    }
    sway_by_height(b.mesh(), 9.0, if full { 0.6 } else { 0.4 })
}

/// A picture of a tree for far billboards: broadleaf on the left half,
/// conifer on the right, transparent round them.
fn billboard_image() -> Image {
    let (w, h) = (128u32, 128u32);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (u, v) = ((x % 64) as f32 / 64.0, y as f32 / h as f32); // v = 0 at the top
            let conifer = x >= 64;
            let n = (((x * 7919 + y * 104_729) % 97) as f32 / 97.0) * 0.18;
            let trunk = (u - 0.5).abs() < 0.04 && v > 0.78;
            let canopy = if conifer {
                // A cone from v=0.05 to v=0.85.
                let t = ((v - 0.05) / 0.8).clamp(0.0, 1.0);
                v > 0.05 && v < 0.85 && (u - 0.5).abs() < 0.42 * t + 0.02
            } else {
                let (dx, dy) = ((u - 0.5) / 0.46, (v - 0.42) / 0.38);
                dx * dx + dy * dy < 1.0 - n
            };
            let i = ((y * w + x) * 4) as usize;
            let (c, a): ([f32; 3], u8) = if canopy {
                let shade = 0.75 + 0.35 * (1.0 - v) + n;
                (if conifer { [0.15 * shade, 0.27 * shade, 0.17 * shade] } else { [0.22 * shade, 0.34 * shade, 0.16 * shade] }, 255)
            } else if trunk {
                ([0.30, 0.22, 0.15], 255)
            } else {
                ([0.2, 0.3, 0.15], 0)
            };
            px[i] = (c[0].clamp(0.0, 1.0) * 255.0) as u8;
            px[i + 1] = (c[1].clamp(0.0, 1.0) * 255.0) as u8;
            px[i + 2] = (c[2].clamp(0.0, 1.0) * 255.0) as u8;
            px[i + 3] = a;
        }
    }
    let mut img = Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    img.sampler = ImageSampler::linear();
    img
}

/// Triangles in each mesh (for the readout).
fn tris(m: &Mesh) -> usize {
    m.indices().map(|i| i.len() / 3).unwrap_or(0)
}

// ---- State ---------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Layer {
    Grass,
    Bush,
    Tree,
    Far,
}

impl Layer {
    fn chunk(self) -> f32 {
        match self {
            Layer::Grass => GRASS_CHUNK,
            Layer::Bush => BUSH_CHUNK,
            Layer::Tree => TREE_CHUNK,
            Layer::Far => FAR_CHUNK,
        }
    }
    /// How far from the camera (along the ground) this layer is kept.
    fn reach(self, s: &Settings) -> f32 {
        let (g, b, t) = s.foliage_density();
        match self {
            Layer::Grass if g > 0.0 => GRASS_FADE.1,
            Layer::Bush if b > 0.0 => BUSH_SIMPLE_END.1,
            Layer::Tree if t > 0.0 => TREE_BLOB_END.1,
            Layer::Far if t > 0.0 => s.billboard_far(),
            _ => 0.0,
        }
    }
}

struct Chunk {
    entities: Vec<Entity>,
    /// Plants in it: (where, kind index).
    plants: Vec<(Vec3, u8)>,
}

#[derive(Resource, Default)]
pub struct Foliage {
    chunks: HashMap<(Layer, i64, i64), Chunk>,
    grass: Vec<Handle<Mesh>>,
    bush_full: Vec<Handle<Mesh>>,
    bush_simple: Vec<Handle<Mesh>>,
    tree_full: Vec<Handle<Mesh>>,
    tree_blob: Vec<Handle<Mesh>>,
    tris_grass: usize,
    tris_bush: (usize, usize),
    tris_tree: (usize, usize),
    plant_mat: Handle<PlantMat>,
    billboard_mat: Handle<PlantMat>,
    /// Plants counted near the camera last frame, by what's drawn of them.
    pub counts: Counts,
    /// How much of each kind grows (from the graphics settings), and which
    /// version of the settings the loaded chunks were built for.
    density: (f32, f32, f32),
    settings_version: Option<(u32, u32)>,
    /// The last land edit (by the editor's count) redrawn.
    edits_seen: u32,
}

#[derive(Default, Clone, Copy)]
pub struct Counts {
    pub grass: usize,
    pub bush_full: usize,
    pub bush_simple: usize,
    pub tree_full: usize,
    pub tree_blob: usize,
    pub billboards: usize,
    pub triangles: usize,
    pub chunks: usize,
}

pub struct FoliagePlugin;

impl Plugin for FoliagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PlantMat>::default());
    }
}

pub fn setup(mut f: ResMut<Foliage>, mut meshes: ResMut<Assets<Mesh>>, mut images: ResMut<Assets<Image>>, mut mats: ResMut<Assets<PlantMat>>, mut shaders: ResMut<Assets<Shader>>) {
    let _ = shaders.insert(&SHADER, Shader::from_wgsl(shader_source(), "foliage_sway.wgsl"));
    let mut r = Rng::from_keys(&[0xF01A_6E5E]);
    for k in 0..3 {
        let m = grass_mesh(&mut r, 0.9 + k as f32 * 0.1);
        f.tris_grass = tris(&m);
        f.grass.push(meshes.add(m));
    }
    for k in 0..2 {
        let tint = 0.92 + k as f32 * 0.16;
        let (a, b) = (bush_mesh(true, tint), bush_mesh(false, tint));
        f.tris_bush = (tris(&a), tris(&b));
        f.bush_full.push(meshes.add(a));
        f.bush_simple.push(meshes.add(b));
    }
    for kind in 0..2u8 {
        let (a, b) = (tree_mesh(kind, true, 1.0), tree_mesh(kind, false, 1.0));
        f.tris_tree = (f.tris_tree.0.max(tris(&a)), f.tris_tree.1.max(tris(&b)));
        f.tree_full.push(meshes.add(a));
        f.tree_blob.push(meshes.add(b));
    }
    f.plant_mat = mats.add(PlantMat { base: StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.95, reflectance: 0.15, cull_mode: None, ..default() }, extension: Sway {} });
    let img = images.add(billboard_image());
    f.billboard_mat = mats.add(PlantMat {
        base: StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(img), alpha_mode: AlphaMode::Mask(0.5), perceptual_roughness: 1.0, reflectance: 0.1, cull_mode: None, ..default() },
        extension: Sway {},
    });
}

fn range(start: Option<(f32, f32)>, end: (f32, f32)) -> VisibilityRange {
    VisibilityRange { start_margin: start.map(|(a, b)| a..b).unwrap_or(0.0..0.0), end_margin: end.0..end.1, use_aabb: false }
}

/// Fill one chunk: the plants it holds, and the entities that draw them.
fn build_chunk(commands: &mut Commands, meshes: &mut Assets<Mesh>, f: &Foliage, w: &World, ground: &dyn Fn(V2) -> f32, layer: Layer, ci: i64, cj: i64) -> Chunk {
    let size = layer.chunk();
    let (x0, y0) = (ci as f32 * size, cj as f32 * size);
    let centre = V2::new(x0 + size * 0.5, y0 + size * 0.5);
    let clear = Clear::near(w, centre, size * 0.5);
    let grid = match layer {
        Layer::Grass => GRASS_GRID,
        Layer::Bush => BUSH_GRID,
        Layer::Tree | Layer::Far => TREE_GRID,
    };
    let (i0, i1) = ((x0 / grid).floor() as i64, ((x0 + size) / grid).floor() as i64);
    let (j0, j1) = ((y0 / grid).floor() as i64, ((y0 + size) / grid).floor() as i64);
    let mut plants = Vec::new();
    let mut entities = Vec::new();
    let mut far = Builder::new();
    let mut far_uv: Vec<[f32; 2]> = Vec::new();
    let mut far_corner: Vec<[f32; 2]> = Vec::new();
    for j in j0..=j1 {
        for i in i0..=i1 {
            let kind = match layer {
                Layer::Grass => 1,
                Layer::Bush => 2,
                Layer::Tree | Layer::Far => 3,
            };
            let (p, mut r) = cell(w, kind, i, j, grid);
            if p.x < x0 || p.x >= x0 + size || p.y < y0 || p.y >= y0 + size || !clear.ok(p) {
                continue;
            }
            let yaw = r.f32() * std::f32::consts::TAU;
            let scale = 0.8 + r.f32() * 0.45;
            let variant = r.below(3);
            let grows = match layer {
                Layer::Grass => grass_at(w, p, &mut r, f.density.0).then_some(0),
                Layer::Bush => bush_at(w, p, &mut r, f.density.1).then_some(0),
                Layer::Tree | Layer::Far => tree_at(w, p, &mut r, f.density.2),
            };
            let Some(sub) = grows else { continue };
            let at = to3(p, ground(p) - 0.05);
            let tf = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw)).with_scale(Vec3::splat(scale));
            plants.push((at, sub));
            match layer {
                Layer::Grass => {
                    entities.push(commands.spawn((Mesh3d(f.grass[variant % f.grass.len()].clone()), MeshMaterial3d(f.plant_mat.clone()), tf, range(None, GRASS_FADE), NotShadowCaster)).id());
                }
                Layer::Bush => {
                    let v = variant % f.bush_full.len();
                    entities.push(commands.spawn((Mesh3d(f.bush_full[v].clone()), MeshMaterial3d(f.plant_mat.clone()), tf, range(None, BUSH_FULL_END))).id());
                    entities.push(commands.spawn((Mesh3d(f.bush_simple[v].clone()), MeshMaterial3d(f.plant_mat.clone()), tf, range(Some(BUSH_FULL_END), BUSH_SIMPLE_END), NotShadowCaster)).id());
                }
                Layer::Tree => {
                    let k = sub as usize;
                    entities.push(commands.spawn((Mesh3d(f.tree_full[k].clone()), MeshMaterial3d(f.plant_mat.clone()), tf, range(None, TREE_FULL_END))).id());
                    entities.push(commands.spawn((Mesh3d(f.tree_blob[k].clone()), MeshMaterial3d(f.plant_mat.clone()), tf, range(Some(TREE_FULL_END), TREE_BLOB_END))).id());
                }
                Layer::Far => {
                    // One camera-facing quad, its corners as offsets from the
                    // trunk's foot (the shader turns them to face the camera).
                    let (hw, ht) = (3.2 * scale, 9.5 * scale);
                    let u0 = if sub == 0 { 0.0 } else { 0.5 };
                    let corners = [(-hw, 0.0), (hw, 0.0), (hw, ht), (-hw, ht)];
                    let uvs = [[u0, 1.0], [u0 + 0.5, 1.0], [u0 + 0.5, 0.0], [u0, 0.0]];
                    let c = [1.0, 1.0, 1.0, 0.0];
                    far.quad_lin([at; 4], [Vec3::Y; 4], [c; 4]);
                    for k in 0..4 {
                        far_uv.push(uvs[k]);
                        far_corner.push([corners[k].0, corners[k].1]);
                    }
                }
            }
        }
    }
    if layer == Layer::Far && !far.is_empty() {
        let m = far.mesh().with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, far_uv).with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, far_corner);
        let h = meshes.add(m);
        entities.push(commands.spawn((Mesh3d(h), MeshMaterial3d(f.billboard_mat.clone()), Transform::default(), NotShadowCaster, NoFrustumCulling)).id());
    }
    Chunk { entities, plants }
}

/// Load the chunks round the camera (nearest first, a few a frame) and drop
/// the ones it has left behind.
pub fn update(mut commands: Commands, mut f: ResMut<Foliage>, game: Res<Game>, scene: Res<Scene3d>, mut meshes: ResMut<Assets<Mesh>>, settings: Res<Settings>) {
    // Settings changed, or a save was loaded: start again.
    if f.settings_version != Some((settings.version, game.loads)) {
        for (_, c) in f.chunks.drain() {
            for e in c.entities {
                commands.entity(e).despawn();
            }
        }
        f.density = settings.foliage_density();
        f.settings_version = Some((settings.version, game.loads));
    }
    // Land edited by hand: the chunks it touched grow again.
    let fresh: Vec<(V2, V2)> = game.editor.dirty.iter().filter(|d| d.0 > f.edits_seen).map(|d| d.1).collect();
    if let Some(n) = game.editor.dirty.last().map(|d| d.0) {
        f.edits_seen = f.edits_seen.max(n);
    }
    if !fresh.is_empty() {
        let hit: Vec<(Layer, i64, i64)> = f
            .chunks
            .keys()
            .filter(|(layer, i, j)| {
                let s = layer.chunk();
                let (x0, y0, x1, y1) = (*i as f32 * s, *j as f32 * s, (*i + 1) as f32 * s, (*j + 1) as f32 * s);
                fresh.iter().any(|(a, b)| a.x <= x1 && b.x >= x0 && a.y <= y1 && b.y >= y0)
            })
            .copied()
            .collect();
        for k in hit {
            if let Some(c) = f.chunks.remove(&k) {
                for e in c.entities {
                    commands.entity(e).despawn();
                }
            }
        }
    }
    let w = &game.world;
    let show = game.view == View::Scene;
    let eye = game.orbit.eye();
    let eye2 = V2::new(eye.x, eye.z);
    let above = eye.y - w.terrain.surface(eye2).max(0.0);
    let ground = |p: V2| scene.grid.height(&w.terrain, p);
    let t0 = std::time::Instant::now();
    let over = |t0: &std::time::Instant| game.shot.is_none() && t0.elapsed().as_secs_f32() * 1000.0 > CHUNK_MS;
    let mut keep: HashMap<(Layer, i64, i64), ()> = HashMap::new();
    for layer in [Layer::Grass, Layer::Bush, Layer::Tree, Layer::Far] {
        let reach = layer.reach(&settings);
        // Off, or too high above the ground for this layer to show at all?
        if !show || reach <= 0.0 || above > reach {
            continue;
        }
        let flat = (reach * reach - above * above).max(0.0).sqrt();
        let size = layer.chunk();
        let (ci, cj) = ((eye2.x / size).floor() as i64, (eye2.y / size).floor() as i64);
        let n = (flat / size).ceil() as i64 + 1;
        let mut want = Vec::new();
        for dj in -n..=n {
            for di in -n..=n {
                let (i, j) = (ci + di, cj + dj);
                let c = V2::new((i as f32 + 0.5) * size, (j as f32 + 0.5) * size);
                let d = c.dist(eye2) - size * 0.71;
                // Far billboards are only needed beyond the blobs.
                let inner = if layer == Layer::Far { (TREE_BLOB_END.0 - size * 1.5).max(0.0) } else { 0.0 };
                if d < flat && c.dist(eye2) + size * 0.71 > inner {
                    want.push((d, i, j));
                }
            }
        }
        want.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, i, j) in want {
            let key = (layer, i, j);
            keep.insert(key, ());
            if !f.chunks.contains_key(&key) && !over(&t0) {
                let c = build_chunk(&mut commands, &mut meshes, &f, w, &ground, layer, i, j);
                f.chunks.insert(key, c);
            }
        }
    }
    let gone: Vec<(Layer, i64, i64)> = f.chunks.keys().filter(|k| !keep.contains_key(k)).copied().collect();
    for k in gone {
        if let Some(c) = f.chunks.remove(&k) {
            for e in c.entities {
                commands.entity(e).despawn();
            }
        }
    }
    // Count what's drawn, by distance, for the readout.
    let mut n = Counts { chunks: f.chunks.len(), ..default() };
    let mid = |r: (f32, f32)| (r.0 + r.1) * 0.5;
    for ((layer, _, _), c) in &f.chunks {
        for (p, _) in &c.plants {
            let d = p.distance(eye);
            match layer {
                Layer::Grass if d < mid(GRASS_FADE) => n.grass += 1,
                Layer::Bush if d < mid(BUSH_FULL_END) => n.bush_full += 1,
                Layer::Bush if d < mid(BUSH_SIMPLE_END) => n.bush_simple += 1,
                Layer::Tree if d < mid(TREE_FULL_END) => n.tree_full += 1,
                Layer::Tree if d < mid(TREE_BLOB_END) => n.tree_blob += 1,
                Layer::Far if d >= mid(TREE_BLOB_END) => n.billboards += 1,
                _ => {}
            }
        }
    }
    n.triangles = n.grass * f.tris_grass + n.bush_full * f.tris_bush.0 + n.bush_simple * f.tris_bush.1 + n.tree_full * f.tris_tree.0 + n.tree_blob * f.tris_tree.1 + n.billboards * 2;
    f.counts = n;
}

/// The middle of the biggest wood within a few kilometres of `p` (for the
/// screenshot over a forest).
pub fn biggest_wood_near(w: &World, p: V2) -> Option<V2> {
    let mut best: Option<(f32, V2)> = None;
    let step = 150.0;
    for j in -40..=40 {
        for i in -40..=40 {
            let q = p.add(V2::new(i as f32 * step, j as f32 * step));
            let Some(s) = spot(w, q) else { continue };
            if s.h > TREE_LINE || s.arid > 0.3 || w.settlements.iter().any(|t| t.pos.dist(q) < t.reach + 100.0) {
                continue;
            }
            // How wooded the neighbourhood is, near the squad preferred.
            let around: f32 = (0..8).map(|k| {
                let a = k as f32 * 0.785;
                wood(w, q.add(V2::new(a.cos(), a.sin()).scale(250.0)))
            }).sum::<f32>() + wood(w, q) * 4.0;
            let score = around - q.dist(p) / 4000.0;
            if best.map(|b| score > b.0).unwrap_or(true) {
                best = Some((score, q));
            }
        }
    }
    best.map(|b| b.1)
}

/// The detail readout (L): camera distance, what each kind of thing is drawn
/// as, and how many triangles that comes to.
pub fn draw_readout(c: &Canvas, oc: &OrbitCam, scene: &Scene3d, f: &Foliage, models: &Models) -> egui::Rect {
    let n = f.counts;
    let model_tris = models.models.get(super::models::RODURO_HOMES[0]).map(|m| format!("{} / {} / {}", m.triangles[0], m.triangles[1], m.triangles[2])).unwrap_or_else(|| "—".into());
    let lines = vec![
        ("Detail".to_string(), palette::GOLD),
        (format!("Camera {:.0} m from the squad", oc.dist), palette::TEXT),
        (format!("Grass tufts {}  (fade {:.0}–{:.0} m)", n.grass, GRASS_FADE.0, GRASS_FADE.1), palette::TEXT),
        (format!("Bushes {} full, {} simple  (full to {:.0} m, gone by {:.0} m)", n.bush_full, n.bush_simple, BUSH_FULL_END.1, BUSH_SIMPLE_END.1), palette::TEXT),
        (format!("Trees {} full, {} blob, {} billboard", n.tree_full, n.tree_blob, n.billboards), palette::TEXT),
        (format!("   full to {:.0} m, blob to {:.0} m, billboards to {:.0} m", TREE_FULL_END.1, TREE_BLOB_END.1, BILLBOARD_FAR), palette::DIM),
        (format!("Foliage chunks loaded {}", n.chunks), palette::DIM),
        (format!("Models: {}", models.status), palette::DIM),
        (format!("   levels {model_tris} triangles, switch at {:.0} / {:.0} m", MODEL_LOD1, MODEL_LOD2), palette::DIM),
        (format!("People: plain shape beyond {:.0} m; markers beyond band 2", super::scene::PERSON_SIMPLE), palette::DIM),
        (format!("Triangles: foliage {}, scene {}, ground {}", n.triangles, scene.triangles, scene.ground_triangles), palette::TEXT),
        (format!("Total about {}", n.triangles + scene.triangles + scene.ground_triangles + scene.town_triangles), palette::GOLD),
    ];
    c.panel(&lines, c.w - 470.0, c.h - 30.0 - 100.0 - 20.0 - 300.0, 14.0)
}
