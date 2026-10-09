//! Low fog: the layer that lies on the water and in the hollows, with the
//! hills standing out of it. From the squad camera, looking down, this is
//! what shows a fog; haze in the distance (`look.rs`) does little from above.
//!
//! It is two soft sheets draped over the land round the camera. Each
//! vertex asks the same question the simulation does (`local.rs`): the fog
//! reaches `fog_depth` above the lowest ground round about, so where the
//! ground is well under that the sheet is thick, and where the ground rises
//! through it the sheet thins to nothing. A wispy texture slides across the
//! sheets with the wind. Rebuilt a couple of times a second, and only while
//! there is fog about.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::math::Affine2;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::rng;
use gahturiyu_sim::sim::weather;

use super::WeatherView;
use crate::view::app::{Game, View};
use crate::view::scene::Scene3d;

/// Cells across the draped sheet.
const CELLS: usize = 64;
/// The wisp texture repeats every this many metres.
const TILE: f32 = 1400.0;
/// How opaque the upper and lower sheets get in the thickest fog.
const OPACITY: [f32; 2] = [0.66, 0.5];
/// Round the spot the camera looks at the sheets are thinned to this share,
/// so the squad is never quite lost under them; they are back to full
/// thickness a couple of camera-distances out.
const WINDOW: f32 = 0.34;
/// How far up through the layer the lower sheet lies.
const LOWER_AT: f32 = 0.45;

#[derive(Component)]
pub struct FogSheet(usize);

#[derive(Resource)]
pub struct GroundFog {
    meshes: [Handle<Mesh>; 2],
    mats: [Handle<StandardMaterial>; 2],
    key: Option<(i64, i64, i64, i64, u64, u32)>,
}

fn sheet(pos: Vec<[f32; 3]>, col: Vec<[f32; 4]>, uv: Vec<[f32; 2]>, idx: Vec<u32>) -> Mesh {
    let nrm = vec![[0.0, 1.0, 0.0]; pos.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(idx))
}

fn nothing() -> Mesh {
    sheet(vec![[0.0, -1e4, 0.0]; 3], vec![[0.0; 4]; 3], vec![[0.0; 2]; 3], vec![0, 1, 2])
}

/// A soft, wispy pattern that repeats without a seam: white, with the wisps
/// in its transparency.
fn wisps() -> Image {
    const N: usize = 256;
    let cell = |seed: u64, period: i64, i: i64, j: i64| (rng::key(&[seed, i.rem_euclid(period) as u64, j.rem_euclid(period) as u64]) >> 40) as f32 / (1u64 << 24) as f32;
    let fade = |t: f32| t * t * (3.0 - 2.0 * t);
    let mut px = Vec::with_capacity(N * N * 4);
    for y in 0..N {
        for x in 0..N {
            let (mut sum, mut amp, mut norm) = (0.0, 1.0, 0.0);
            for (o, period) in [3i64, 6, 12, 24, 48].into_iter().enumerate() {
                let (fx, fy) = (x as f32 / N as f32 * period as f32, y as f32 / N as f32 * period as f32);
                let (i, j) = (fx.floor() as i64, fy.floor() as i64);
                let (u, v) = (fade(fx - i as f32), fade(fy - j as f32));
                let s = 0xF06 + o as u64;
                let top = cell(s, period, i, j) * (1.0 - u) + cell(s, period, i + 1, j) * u;
                let bot = cell(s, period, i, j + 1) * (1.0 - u) + cell(s, period, i + 1, j + 1) * u;
                sum += (top * (1.0 - v) + bot * v) * amp;
                norm += amp;
                amp *= 0.55;
            }
            let n = sum / norm;
            let t = ((n - 0.32) / 0.36).clamp(0.0, 1.0);
            let a = 0.5 + 0.5 * t * t * (3.0 - 2.0 * t);
            px.extend_from_slice(&[255, 255, 255, (a * 255.0) as u8]);
        }
    }
    let mut img = Image::new(Extent3d { width: N as u32, height: N as u32, depth_or_array_layers: 1 }, TextureDimension::D2, px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    img
}

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut images: ResMut<Assets<Image>>) {
    let tex = images.add(wisps());
    let make = |k: usize, commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>| {
        let mesh = meshes.add(nothing());
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.91, 0.92),
            base_color_texture: Some(tex.clone()),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            cull_mode: None,
            double_sided: true,
            ..default()
        });
        commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat.clone()), Transform::default(), Visibility::Hidden, NotShadowCaster, NoFrustumCulling, FogSheet(k)));
        (mesh, mat)
    };
    let (m0, a0) = make(0, &mut commands, &mut meshes, &mut materials);
    let (m1, a1) = make(1, &mut commands, &mut meshes, &mut materials);
    commands.insert_resource(GroundFog { meshes: [m0, m1], mats: [a0, a1], key: None });
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[allow(clippy::too_many_arguments)]
pub fn update(
    game: Res<Game>,
    mut view: ResMut<WeatherView>,
    scene: Res<Scene3d>,
    fog: Option<ResMut<GroundFog>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sheets: Query<(&FogSheet, &mut Visibility, &mut Transform)>,
) {
    let Some(mut fog) = fog else { return };
    let started = std::time::Instant::now();
    let v = &mut *view;
    let any = game.view == View::Scene && v.skies.iter().any(|s| s.fog > 0.02);
    if !any {
        if fog.key.is_some() {
            fog.key = None;
            for (_, mut vis, _) in &mut sheets {
                *vis = Visibility::Hidden;
            }
        }
        return;
    }
    let world = &game.world;
    let reach = (game.orbit.dist * 3.5).clamp(450.0, 7000.0);
    let step = 15.0 * 2f32.powf((2.0 * reach / CELLS as f32 / 15.0).log2().ceil().max(0.0));
    let centre = V2::new((game.orbit.target.x / step).round() * step, (game.orbit.target.y / step).round() * step);
    // The sheets hang from two points, one above the other, so the renderer
    // draws the lower sheet first when looking down on them.
    let hang = [Vec3::new(centre.x, 60.0, centre.y), Vec3::new(centre.x, 0.0, centre.y)];
    let key = ((game.orbit.target.x / (game.orbit.dist * 0.08).max(2.0)) as i64, (game.orbit.target.y / (game.orbit.dist * 0.08).max(2.0)) as i64, step as i64 * 1000 + (game.orbit.dist / 4.0) as i64, (v.clock * 2.0) as i64, world.seed, game.loads);
    if fog.key != Some(key) {
        fog.key = Some(key);
        let n = CELLS + 1;
        let half = CELLS as f32 / 2.0;
        let mut top = vec![0.0f32; n * n];
        let mut ground = vec![0.0f32; n * n];
        let mut thick = vec![0.0f32; n * n];
        for j in 0..n {
            for i in 0..n {
                let p = V2::new(centre.x + (i as f32 - half) * step, centre.y + (j as f32 - half) * step);
                let sky = weather::blend(&v.skies, &v.land.mix(p));
                let g = scene.grid.height(&world.terrain, p);
                let k = j * n + i;
                ground[k] = g;
                if sky.fog <= 0.004 {
                    top[k] = g;
                    continue;
                }
                let t = weather::floor(&world.terrain, p) + sky.fog_depth;
                let over = t - g;
                // The sheet fades out towards its own edge, so it never ends in a line.
                let rim = ((i as f32 - half).abs().max((j as f32 - half).abs())) / half;
                let away = p.dist(game.orbit.target) / game.orbit.dist.max(10.0);
                let window = WINDOW + (1.0 - WINDOW) * smooth(0.5, 2.0, away);
                thick[k] = sky.fog * smooth(0.0, 20.0, over) * smooth(1.0, 0.82, rim) * window;
                top[k] = if over > 0.0 { t } else { g };
            }
        }
        let mut any_fog = false;
        for layer in 0..2 {
            let mut pos = Vec::with_capacity(n * n);
            let mut col = Vec::with_capacity(n * n);
            let mut uv = Vec::with_capacity(n * n);
            let mut idx = Vec::new();
            for j in 0..n {
                for i in 0..n {
                    let k = j * n + i;
                    let (x, z) = (centre.x + (i as f32 - half) * step, centre.y + (j as f32 - half) * step);
                    let y = if layer == 0 { top[k] } else { ground[k] + (top[k] - ground[k]) * LOWER_AT };
                    pos.push([x - hang[layer].x, y + 0.3 - hang[layer].y, z - hang[layer].z]);
                    col.push([1.0, 1.0, 1.0, thick[k] * OPACITY[layer]]);
                    uv.push([x / TILE, z / TILE]);
                }
            }
            for j in 0..CELLS {
                for i in 0..CELLS {
                    let (a, b, c, d) = (j * n + i, j * n + i + 1, (j + 1) * n + i + 1, (j + 1) * n + i);
                    if thick[a].max(thick[b]).max(thick[c]).max(thick[d]) > 0.004 {
                        idx.extend_from_slice(&[a as u32, d as u32, c as u32, a as u32, c as u32, b as u32]);
                    }
                }
            }
            any_fog |= !idx.is_empty();
            if let Some(mut m) = meshes.get_mut(&fog.meshes[layer]) {
                *m = if idx.is_empty() { nothing() } else { sheet(pos, col, uv, idx) };
            }
        }
        for (s, mut vis, mut tf) in &mut sheets {
            *vis = if any_fog { Visibility::Inherited } else { Visibility::Hidden };
            tf.translation = hang[s.0];
        }
    }
    // The wisps drift with the wind (and creep a little even in a dead calm).
    let wind = Vec2::new(v.here.wind_to.x, v.here.wind_to.y) * v.here.wind.max(0.3);
    let slide = -(wind * (v.clock as f32)) / TILE;
    for (layer, (scale, turn, speed)) in [(1.0f32, 0.0f32, 1.0f32), (1.9, 0.7, 0.55)].into_iter().enumerate() {
        if let Some(mut m) = materials.get_mut(&fog.mats[layer]) {
            m.uv_transform = Affine2::from_scale_angle_translation(Vec2::splat(scale), turn, slide * speed);
        }
    }
    v.spent += started.elapsed().as_secs_f32() * 1000.0;
}
