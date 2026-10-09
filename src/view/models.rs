//! Models made in Blender or Meshy, loaded from `assets/models/` as GLB.
//!
//! Each model has standard materials (colour, normal, and one combined
//! occlusion/roughness/metallic image), which the renderer lights like
//! everything else. A model is scaled to the size the simulation gives the
//! building and stood on the ground; nothing about it changes what happens.
//!
//! **Detail levels.** A model ships as a set of files found by name:
//! `Roduro_Home_5k.glb` (the most detailed, about 5,000 triangles),
//! `Roduro_Home_2k.glb` and `Roduro_Home_500.glb`. Any lower version that's
//! missing is made when the game loads, by simplifying the next one up (the
//! meshoptimizer library). Which one is drawn depends on distance from the
//! camera (`MODEL_LOD1`, `MODEL_LOD2`); near each switch the two overlap for
//! `MODEL_FADE` metres and are dithered into each other, so nothing pops.
//!
//! **The Roduro buildings** are four whole models (`RODURO_HOMES`): a
//! building's kind is rolled from its seed, the great house for the biggest.
//! If no file for a kind is there, those homes are drawn as the built-in
//! grown-stone domes.
//!
//! **The Horaro kit** (`assets/models/horaro/`) is pieces (hull, woven
//! drums, rock pillars, root base, deck, ramp, dock), each `<id>_lod0.glb`
//! close and `<id>.glb` further off, put together by the assemblies listed
//! in `horaro_kit.json` (the file is in Blender's frame, z up, and is turned
//! to the renderer's y up here). A stilt home is one of those assemblies,
//! stood on the water line.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::VisibilityRange;
use bevy::gltf::{Gltf, GltfMesh};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use std::collections::HashMap;

/// The Roduro buildings (inside `assets/models/`), before the detail suffix.
pub const RODURO_HOMES: [&str; 4] = ["Roduro_Home", "Roduro_Drum", "Roduro_Vault", "Roduro_Great"];
/// Which of them is the great house (for the biggest homes).
pub const RODURO_GREAT: usize = 3;
/// Detail levels: file suffix, and the triangle count a made one aims for.
pub const MODEL_LEVELS: [(&str, usize); 3] = [("5k", 5000), ("2k", 2000), ("500", 500)];
/// Beyond this many metres from the camera, the 2k version is drawn...
pub const MODEL_LOD1: f32 = 70.0;
/// ...and beyond this, the 500.
pub const MODEL_LOD2: f32 = 220.0;
/// Metres either side of each switch where both are drawn, dithered.
pub const MODEL_FADE: f32 = 12.0;
/// Turn models about the vertical so their front faces the way the
/// simulation says the building faces (radians). Models are expected to face
/// +X; change this if they come in sideways.
pub const MODEL_YAW: f32 = 0.0;
/// The Horaro kit's folder (inside `assets/models/`) and its list.
pub const HORARO_DIR: &str = "horaro";
pub const HORARO_KIT: &str = "horaro_kit.json";
/// The assemblies a stilt home can be, and their weights.
pub const STILT_HOMES: [(&str, u32); 3] = [("horaro_rock_home", 6), ("horaro_hull_on_roots", 2), ("horaro_twin_on_pillars", 2)];

type Parts = Vec<(Handle<Mesh>, Handle<StandardMaterial>)>;

/// One model, ready to place: its pieces at each detail level, and its size
/// in its own units.
#[derive(Clone)]
pub struct Model {
    pub levels: [Parts; 3],
    /// Triangles at each level.
    pub triangles: [usize; 3],
    /// Widest footprint across (x or z), in model units.
    pub width: f32,
    /// Lowest point, so it can be stood on the ground.
    pub floor: f32,
}

/// One piece of an assembly: which model, where it goes (renderer frame,
/// metres), its turn about the vertical (radians) and its scale.
#[derive(Clone, Debug)]
pub struct Placed {
    pub model: String,
    pub at: Vec3,
    pub yaw: f32,
    pub scale: Vec3,
}

#[derive(Resource, Default)]
pub struct Models {
    /// Loaded models by name (`Roduro_Home`, `horaro/horaro_hull_dome`, ...).
    pub models: HashMap<String, Model>,
    /// The Horaro kit's assemblies by name.
    pub assemblies: HashMap<String, Vec<Placed>>,
    /// Files still loading: model name, level, handle.
    loading: Vec<(String, usize, Handle<Gltf>)>,
    /// Names that failed to load (drawn the built-in way).
    failed: Vec<String>,
    pub status: String,
    /// Goes up whenever a model finishes, so towns know to redraw.
    pub generation: u32,
}

/// The distances a detail level is drawn at.
pub fn level_range(i: usize) -> VisibilityRange {
    let (a, b) = (MODEL_LOD1, MODEL_LOD2);
    let f = MODEL_FADE;
    match i {
        0 => VisibilityRange { start_margin: 0.0..0.0, end_margin: (a - f)..(a + f), use_aabb: false },
        1 => VisibilityRange { start_margin: (a - f)..(a + f), end_margin: (b - f)..(b + f), use_aabb: false },
        _ => VisibilityRange { start_margin: (b - f)..(b + f), end_margin: 1.0e9..1.0e9, use_aabb: false },
    }
}

impl Models {
    pub fn has(&self, name: &str) -> bool {
        self.models.contains_key(name)
    }

    /// The Roduro building for a home: rolled from its seed, the great house
    /// for the biggest. None if that model isn't loaded.
    pub fn roduro_kind(&self, seed: u64, size: f32) -> Option<&str> {
        let pick = if size > 11.0 { RODURO_GREAT } else { ((seed >> 20) % 3) as usize };
        let name = RODURO_HOMES[pick];
        if self.has(name) {
            Some(name)
        } else if self.has(RODURO_HOMES[0]) {
            Some(RODURO_HOMES[0])
        } else {
            None
        }
    }

    /// The stilt-home assembly for a seed, if the kit is loaded.
    pub fn stilt_kind(&self, seed: u64) -> Option<&str> {
        let total: u32 = STILT_HOMES.iter().map(|(_, w)| w).sum();
        let mut roll = ((seed >> 24) % total as u64) as u32;
        for (name, w) in STILT_HOMES {
            if roll < w {
                return self.assemblies.contains_key(name).then_some(name);
            }
            roll -= w;
        }
        None
    }

    /// Place a model: standing at `base`, facing `rot` (the simulation's
    /// facing), `size` metres across (0 = the model's own size).
    pub fn spawn(&self, commands: &mut Commands, name: &str, base: Vec3, rot: f32, size: f32) -> Vec<Entity> {
        let Some(m) = self.models.get(name) else { return Vec::new() };
        let s = if size > 0.0 { size / m.width.max(1e-3) } else { 1.0 };
        let tf = Transform::from_translation(base - Vec3::Y * (m.floor * s)).with_rotation(Quat::from_rotation_y(-rot + MODEL_YAW)).with_scale(Vec3::splat(s));
        self.spawn_with(commands, m, tf)
    }

    fn spawn_with(&self, commands: &mut Commands, m: &Model, tf: Transform) -> Vec<Entity> {
        let mut out = Vec::new();
        for (i, parts) in m.levels.iter().enumerate() {
            for (mesh, mat) in parts {
                out.push(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), tf, level_range(i))).id());
            }
        }
        out
    }

    /// Place an assembly with its origin at `base` (for stilt homes, on the
    /// water line), facing `rot`, scaled by `scale`.
    pub fn spawn_assembly(&self, commands: &mut Commands, name: &str, base: Vec3, rot: f32, scale: f32) -> Vec<Entity> {
        let Some(pieces) = self.assemblies.get(name) else { return Vec::new() };
        let root = Transform::from_translation(base).with_rotation(Quat::from_rotation_y(-rot + MODEL_YAW)).with_scale(Vec3::splat(scale));
        let mut out = Vec::new();
        for p in pieces {
            let Some(m) = self.models.get(&p.model) else { continue };
            let local = Transform::from_translation(p.at).with_rotation(Quat::from_rotation_y(p.yaw)).with_scale(p.scale);
            out.extend(self.spawn_with(commands, m, root.mul_transform(local)));
        }
        out
    }
}

/// Where `assets/` is: next to Cargo.toml when run with cargo, else the
/// working directory, else next to the program.
pub fn assets_dir() -> std::path::PathBuf {
    let candidates = [
        std::env::var("BEVY_ASSET_ROOT").ok().map(|p| std::path::PathBuf::from(p).join("assets")),
        std::env::var("CARGO_MANIFEST_DIR").ok().map(|p| std::path::PathBuf::from(p).join("assets")),
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")),
        std::env::current_dir().ok().map(|p| p.join("assets")),
        std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("assets"))),
    ];
    candidates.into_iter().flatten().find(|p| p.is_dir()).unwrap_or_else(|| std::path::PathBuf::from("assets"))
}

/// Start loading whichever models and detail levels are there.
pub fn start(mut models: ResMut<Models>, server: Res<AssetServer>) {
    let dir = assets_dir();
    for name in RODURO_HOMES {
        for (i, (suffix, _)) in MODEL_LEVELS.iter().enumerate() {
            let file = format!("models/{name}_{suffix}.glb");
            if dir.join(&file).is_file() {
                models.loading.push((name.to_string(), i, server.load(file)));
            }
        }
    }
    // The Horaro kit: its pieces, and how they go together.
    let kit_path = dir.join("models").join(HORARO_DIR).join(HORARO_KIT);
    if let Ok(text) = std::fs::read_to_string(&kit_path) {
        match read_kit(&text) {
            Ok((pieces, assemblies)) => {
                for id in pieces {
                    let name = format!("{HORARO_DIR}/{id}");
                    let close = format!("models/{HORARO_DIR}/{id}_lod0.glb");
                    let medium = format!("models/{HORARO_DIR}/{id}.glb");
                    match (dir.join(&close).is_file(), dir.join(&medium).is_file()) {
                        (true, true) => {
                            models.loading.push((name.clone(), 0, server.load(close)));
                            models.loading.push((name, 1, server.load(medium)));
                        }
                        (false, true) => models.loading.push((name, 0, server.load(medium))),
                        (true, false) => models.loading.push((name, 0, server.load(close))),
                        _ => {}
                    }
                }
                models.assemblies = assemblies;
            }
            Err(e) => models.status = format!("Horaro kit: {e}"),
        }
    }
    models.status = if models.loading.is_empty() {
        "no models in assets/models, drawing the built-in shapes".into()
    } else {
        format!("loading {} model files", models.loading.len())
    };
}

/// Read the kit list: the piece ids, and the assemblies turned into the
/// renderer's frame (the file is z up; here y is up and z runs the other way).
fn read_kit(text: &str) -> Result<(Vec<String>, HashMap<String, Vec<Placed>>), String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let pieces: Vec<String> = v["pieces"].as_object().ok_or("no pieces")?.keys().cloned().collect();
    let mut assemblies = HashMap::new();
    for (name, list) in v["assemblies"].as_object().ok_or("no assemblies")? {
        let mut placed = Vec::new();
        for p in list.as_array().ok_or("assembly isn't a list")? {
            let num = |x: &serde_json::Value, i: usize| x.get(i).and_then(|n| n.as_f64()).unwrap_or(0.0) as f32;
            let at = &p["at"];
            let sc = &p["scale"];
            placed.push(Placed {
                model: format!("{HORARO_DIR}/{}", p["piece"].as_str().unwrap_or("")),
                at: Vec3::new(num(at, 0), num(at, 2), -num(at, 1)),
                yaw: (p["yaw_deg"].as_f64().unwrap_or(0.0) as f32).to_radians(),
                scale: if sc.is_array() { Vec3::new(num(sc, 0), num(sc, 2), num(sc, 1)) } else { Vec3::ONE },
            });
        }
        assemblies.insert(name.clone(), placed);
    }
    Ok((pieces, assemblies))
}

/// The pieces of a loaded glTF file, if everything in it has loaded.
fn parts_of(h: &Handle<Gltf>, gltfs: &Assets<Gltf>, gmeshes: &Assets<GltfMesh>, meshes: &Assets<Mesh>, server: &AssetServer) -> Option<Parts> {
    let g = gltfs.get(h)?;
    let mut parts = Vec::new();
    let file = h.path().map(|p| p.without_label().to_string()).unwrap_or_default();
    for gm in &g.meshes {
        let gm = gmeshes.get(gm)?;
        for prim in &gm.primitives {
            meshes.get(&prim.mesh)?;
            // The loader makes a standard material for each glTF one,
            // labelled "<material>/std".
            let label = prim.material.as_ref().and_then(|m| m.path()).and_then(|p| p.label().map(|l| l.to_string())).unwrap_or_else(|| bevy::gltf::GltfAssetLabel::DefaultMaterial.to_string());
            let mat: Handle<StandardMaterial> = server.load(format!("{file}#{label}/std"));
            parts.push((prim.mesh.clone(), mat));
        }
    }
    Some(parts)
}

fn triangles(meshes: &Assets<Mesh>, parts: &Parts) -> usize {
    parts.iter().filter_map(|(m, _)| meshes.get(m)).map(|m| m.indices().map(|i| i.len()).unwrap_or(m.count_vertices()) / 3).sum()
}

/// A simplified copy of a mesh with about `ratio` of its triangles: the same
/// vertices, fewer of them used.
pub fn simplify(mesh: &Mesh, ratio: f32) -> Option<Mesh> {
    let Some(VertexAttributeValues::Float32x3(ps)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None };
    let idx: Vec<u32> = match mesh.indices() {
        Some(Indices::U32(v)) => v.clone(),
        Some(Indices::U16(v)) => v.iter().map(|&i| i as u32).collect(),
        None => (0..ps.len() as u32).collect(),
    };
    let bytes: Vec<u8> = ps.iter().flat_map(|p| p.iter().flat_map(|c| c.to_le_bytes())).collect();
    let adapter = meshopt::VertexDataAdapter::new(&bytes, 12, 0).ok()?;
    let target = ((idx.len() / 3) as f32 * ratio).max(1.0) as usize * 3;
    let out = meshopt::simplify(&idx, &adapter, target, 0.05, meshopt::SimplifyOptions::LockBorder, None);
    let out = if out.len() > target * 3 / 2 {
        // Couldn't get there keeping the borders; let them go.
        meshopt::simplify(&idx, &adapter, target, 0.2, meshopt::SimplifyOptions::None, None)
    } else {
        out
    };
    let mut m = mesh.clone();
    m.asset_usage = RenderAssetUsages::default();
    m.insert_indices(Indices::U32(out));
    Some(m)
}

/// When a model's files have finished loading, gather their pieces, make
/// any missing detail levels, and measure the model.
pub fn finish(mut models: ResMut<Models>, gltfs: Res<Assets<Gltf>>, gmeshes: Res<Assets<GltfMesh>>, mut meshes: ResMut<Assets<Mesh>>, server: Res<AssetServer>) {
    if models.loading.is_empty() {
        return;
    }
    // Work one model at a time: the first whose files are all in (or one has failed).
    let names: Vec<String> = models.loading.iter().map(|(n, _, _)| n.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    for name in names {
        let mut found: [Option<Parts>; 3] = [None, None, None];
        let mut all_in = true;
        let mut failed = false;
        for (n, i, h) in &models.loading {
            if *n != name {
                continue;
            }
            match parts_of(h, &gltfs, &gmeshes, &meshes, &server) {
                Some(p) => found[*i] = Some(p),
                None => {
                    all_in = false;
                    if server.load_state(h).is_failed() {
                        failed = true;
                    }
                }
            }
        }
        if failed {
            models.loading.retain(|(n, _, _)| *n != name);
            models.failed.push(name.clone());
            models.status = format!("couldn't read {name}");
            models.generation += 1;
            return;
        }
        if !all_in {
            continue;
        }
        let Some(top) = found.iter().position(|f| f.is_some()) else { continue };
        let mut levels: [Parts; 3] = [Vec::new(), Vec::new(), Vec::new()];
        levels[top] = found[top].take().unwrap();
        for i in 0..3 {
            if i == top {
                continue;
            }
            if let Some(p) = found[i].take() {
                levels[i] = p;
                continue;
            }
            // Missing: from the nearest level above (or, above the top, the top itself).
            let src = if i < top { top } else { i - 1 };
            let have = triangles(&meshes, &levels[src]).max(1);
            let ratio = (MODEL_LEVELS[i].1 as f32 / have as f32).min(1.0);
            let mut parts = Vec::new();
            for (mh, mat) in levels[src].clone() {
                let simple = meshes.get(&mh).and_then(|m| simplify(m, ratio));
                match simple {
                    Some(s) => parts.push((meshes.add(s), mat)),
                    None => parts.push((mh, mat)),
                }
            }
            levels[i] = parts;
        }
        let tris = [triangles(&meshes, &levels[0]), triangles(&meshes, &levels[1]), triangles(&meshes, &levels[2])];
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for (mh, _) in &levels[0] {
            if let Some(VertexAttributeValues::Float32x3(ps)) = meshes.get(mh).and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION).cloned()) {
                for p in ps {
                    lo = lo.min(Vec3::from(p));
                    hi = hi.max(Vec3::from(p));
                }
            }
        }
        // Kit pieces keep their own metres and anchors; whole buildings stand on their floor.
        let floor = if name.starts_with(HORARO_DIR) { 0.0 } else { lo.y };
        models.models.insert(name.clone(), Model { levels, triangles: tris, width: (hi.x - lo.x).max(hi.z - lo.z), floor });
        models.loading.retain(|(n, _, _)| *n != name);
        models.generation += 1;
        let left = models.loading.iter().map(|(n, _, _)| n).collect::<std::collections::BTreeSet<_>>().len();
        models.status = if left == 0 {
            format!("{} models loaded{}", models.models.len(), if models.failed.is_empty() { String::new() } else { format!(", {} failed", models.failed.len()) })
        } else {
            format!("{} models loaded, {left} loading", models.models.len())
        };
        return;
    }
}
