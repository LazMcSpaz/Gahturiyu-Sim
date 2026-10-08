//! Models made in Blender or Meshy, loaded from `assets/models/` as GLB.
//!
//! Each model has standard materials (colour, normal, and one combined
//! occlusion/roughness/metallic image), which the renderer lights like
//! everything else. A model is scaled to the size the simulation gives the
//! building and stood on the ground; nothing about it changes what happens.
//!
//! The first one is the Roduro home, `assets/models/Roduro_Home_5k.glb`. If
//! that file isn't there, homes are drawn as the built-in grown-stone domes.

use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;

/// File name of the Roduro home model (inside `assets/models/`).
pub const RODURO_HOME: &str = "Roduro_Home";
/// Turn models about the vertical so their front faces the way the
/// simulation says the building faces (radians). Models are expected to face
/// +X in Blender's export; change this if they come in sideways.
pub const MODEL_YAW: f32 = 0.0;

/// One model, ready to place: its pieces, and its size in its own units.
#[derive(Clone)]
pub struct Model {
    pub parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
    /// Widest footprint across (x or z), in model units.
    pub width: f32,
    /// Lowest point, so it can be stood on the ground.
    pub floor: f32,
}

#[derive(Resource, Default)]
pub struct Models {
    pub roduro_home: Option<Model>,
    loading: Option<Handle<Gltf>>,
    pub status: String,
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
        m.parts.iter().map(|(mesh, mat)| commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), tf)).id()).collect()
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

/// Start loading whatever models are there.
pub fn start(mut models: ResMut<Models>, server: Res<AssetServer>) {
    let file = format!("models/{RODURO_HOME}_5k.glb");
    if assets_dir().join(&file).is_file() {
        models.loading = Some(server.load(file));
        models.status = "Roduro home: loading".into();
    } else {
        models.status = format!("Roduro home: no assets/{file}, drawing domes");
    }
}

/// When a model has finished loading, gather its pieces and measure it.
pub fn finish(mut models: ResMut<Models>, gltfs: Res<Assets<Gltf>>, gmeshes: Res<Assets<GltfMesh>>, meshes: Res<Assets<Mesh>>, server: Res<AssetServer>) {
    let Some(h) = models.loading.clone() else { return };
    let Some(g) = gltfs.get(&h) else { return };
    let mut parts = Vec::new();
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for gm in &g.meshes {
        let Some(gm) = gmeshes.get(gm) else { return };
        for prim in &gm.primitives {
            let Some(mesh) = meshes.get(&prim.mesh) else { return };
            if let Some(bevy::mesh::VertexAttributeValues::Float32x3(ps)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                for p in ps {
                    lo = lo.min(Vec3::from(*p));
                    hi = hi.max(Vec3::from(*p));
                }
            }
            // The loader makes a standard material for each glTF one,
            // labelled "<material>/std".
            let file = h.path().map(|p| p.without_label().to_string()).unwrap_or_default();
            let label = prim.material.as_ref().and_then(|m| m.path()).and_then(|p| p.label().map(|l| l.to_string())).unwrap_or_else(|| bevy::gltf::GltfAssetLabel::DefaultMaterial.to_string());
            let mat: Handle<StandardMaterial> = server.load(format!("{file}#{label}/std"));
            parts.push((prim.mesh.clone(), mat));
        }
    }
    if parts.is_empty() {
        models.status = "Roduro home: the file has no meshes".into();
        models.loading = None;
        return;
    }
    let width = (hi.x - lo.x).max(hi.z - lo.z);
    models.status = format!("Roduro home: loaded ({} pieces)", parts.len());
    models.roduro_home = Some(Model { parts, width, floor: lo.y });
    models.loading = None;
}
