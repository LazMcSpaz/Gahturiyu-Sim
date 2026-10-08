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
//! The first model is the Roduro home. If no file for it is there, homes are
//! drawn as the built-in grown-stone domes.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::VisibilityRange;
use bevy::gltf::{Gltf, GltfMesh};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

/// File name of the Roduro home model (inside `assets/models/`), before the
/// detail suffix.
pub const RODURO_HOME: &str = "Roduro_Home";
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

#[derive(Resource, Default)]
pub struct Models {
    pub roduro_home: Option<Model>,
    loading: Vec<(usize, Handle<Gltf>)>,
    pub status: String,
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
    pub fn ready(&self) -> bool {
        self.roduro_home.is_some()
    }

    /// Place a Roduro home: standing at `base`, facing `rot` (the
    /// simulation's facing), `size` metres across.
    pub fn spawn_roduro_home(&self, commands: &mut Commands, base: Vec3, rot: f32, size: f32) -> Vec<Entity> {
        let Some(m) = &self.roduro_home else { return Vec::new() };
        let s = size / m.width.max(1e-3);
        let tf = Transform::from_translation(base - Vec3::Y * (m.floor * s)).with_rotation(Quat::from_rotation_y(-rot + MODEL_YAW)).with_scale(Vec3::splat(s));
        let mut out = Vec::new();
        for (i, parts) in m.levels.iter().enumerate() {
            for (mesh, mat) in parts {
                out.push(commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), tf, level_range(i))).id());
            }
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

/// Start loading whichever detail levels of the models are there.
pub fn start(mut models: ResMut<Models>, server: Res<AssetServer>) {
    let dir = assets_dir();
    for (i, (suffix, _)) in MODEL_LEVELS.iter().enumerate() {
        let file = format!("models/{RODURO_HOME}_{suffix}.glb");
        if dir.join(&file).is_file() {
            models.loading.push((i, server.load(file)));
        }
    }
    models.status = if models.loading.is_empty() {
        format!("Roduro home: no assets/models/{RODURO_HOME}_5k.glb, drawing domes")
    } else {
        "Roduro home: loading".into()
    };
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

/// When the files have finished loading, gather their pieces, make any
/// missing detail levels, and measure the model.
pub fn finish(mut models: ResMut<Models>, gltfs: Res<Assets<Gltf>>, gmeshes: Res<Assets<GltfMesh>>, mut meshes: ResMut<Assets<Mesh>>, server: Res<AssetServer>) {
    if models.loading.is_empty() || models.roduro_home.is_some() {
        return;
    }
    let mut found: [Option<Parts>; 3] = [None, None, None];
    for (i, h) in &models.loading {
        match parts_of(h, &gltfs, &gmeshes, &meshes, &server) {
            Some(p) => found[*i] = Some(p),
            None => {
                if server.load_state(h).is_failed() {
                    models.status = format!("Roduro home: couldn't read level {}", MODEL_LEVELS[*i].0);
                    models.loading.clear();
                }
                return;
            }
        }
    }
    // The most detailed one there is the starting point.
    let Some(top) = found.iter().position(|f| f.is_some()) else { return };
    let mut made = [false; 3];
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
        made[i] = true;
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
    let label = |i: usize| format!("{}{}", tris[i], if made[i] { " (made)" } else { "" });
    models.status = format!("Roduro home: {} / {} / {} triangles", label(0), label(1), label(2));
    models.roduro_home = Some(Model { levels, triangles: tris, width: (hi.x - lo.x).max(hi.z - lo.z), floor: lo.y });
    models.loading.clear();
}
