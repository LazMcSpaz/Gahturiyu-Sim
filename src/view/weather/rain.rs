//! Rain and snow: streaks and flakes falling through a box of air that goes
//! wherever the camera does. Each drop has a fixed place in the box and falls (and blows) at a
//! steady rate, wrapping round when it leaves, so there is nothing to keep
//! track of: where every drop is comes straight from the clock.
//!
//! How many drops follows how hard it is raining; how they slant follows
//! the wind. Snow is the same drops made slow, small and wandering. One
//! mesh, rebuilt each frame (a few thousand thin quads).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

use gahturiyu_sim::sim::rng::Rng;
use gahturiyu_sim::sim::stealth;

use super::{Quality, WeatherView};
use crate::view::app::{Game, View};

/// The box of air, metres a side, centred on the camera.
const BOX: f32 = 22.0;
/// How fast rain falls, metres a second.
const FALL: f32 = 9.0;
/// A streak is as long as a drop travels in this many seconds.
const BLUR: f32 = 0.05;
/// How fast snow falls, metres a second.
const SNOW_FALL: f32 = 1.5;

#[derive(Component)]
pub struct RainMesh;

#[derive(Resource)]
pub struct Rain {
    mesh: Handle<Mesh>,
    /// Each drop's place in the box (0..1 each way), how fast it falls
    /// compared with the rest, and where it is in its own gusts.
    drops: Vec<[f32; 5]>,
    showing: bool,
}

fn build(pos: Vec<[f32; 3]>, col: Vec<[f32; 4]>, idx: Vec<u32>) -> Mesh {
    let nrm = vec![[0.0, 1.0, 0.0]; pos.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(Indices::U32(idx))
}

fn nothing() -> Mesh {
    build(vec![[0.0, -1e4, 0.0]; 3], vec![[0.0; 4]; 3], vec![0, 1, 2])
}

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(nothing());
    let mat = materials.add(StandardMaterial { base_color: Color::WHITE, unlit: true, alpha_mode: AlphaMode::Blend, cull_mode: None, double_sided: false, fog_enabled: false, ..default() });
    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat), Transform::default(), Visibility::Hidden, NotShadowCaster, NoFrustumCulling, RainMesh));
    let mut r = Rng::from_keys(&[0x5241_494E]);
    let drops = (0..Quality::MOST_DROPS).map(|_| [r.f32(), r.f32(), r.f32(), r.f32(), r.f32()]).collect();
    commands.insert_resource(Rain { mesh, drops, showing: false });
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn update(game: Res<Game>, mut view: ResMut<WeatherView>, rain: Option<ResMut<Rain>>, mut meshes: ResMut<Assets<Mesh>>, mut vis: Query<&mut Visibility, With<RainMesh>>) {
    let Some(mut rain) = rain else { return };
    let started = std::time::Instant::now();
    let w = view.eye;
    // How many drops the heaviest fall has, by the graphics setting.
    let most = view.quality.drops.min(rain.drops.len());
    if game.view != View::Scene || most == 0 || (w.rain < 0.02 && w.snow < 0.02) {
        if rain.showing {
            rain.showing = false;
            for mut v in &mut vis {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let count = if w.rain < 0.02 { 0 } else { ((most as f32 * w.rain.powf(0.8)) as usize).min(most) };
    let flakes = if w.snow < 0.02 { 0 } else { ((most as f32 * w.snow.powf(0.7)) as usize).min(most) };
    let eye = game.orbit.eye();
    let forward = (game.orbit.look_at() - eye).normalize_or_zero();
    let blow = Vec3::new(w.wind_to.x, 0.0, w.wind_to.y) * w.wind * 0.9;
    let up = Vec3::Y;
    let right = forward.cross(up).normalize_or_zero();
    let over = right.cross(forward);
    let clock = view.clock;
    // Unlit, so it has to be dimmed by hand at night and under a storm.
    let day = stealth::daylight(game.world.time);
    let shade = (0.12 + 0.88 * day) * (1.0 - 0.35 * w.storm);
    let tint = [0.80 * shade, 0.85 * shade, 0.92 * shade];
    let mut pos = Vec::with_capacity((count + flakes) * 4);
    let mut col = Vec::with_capacity((count + flakes) * 4);
    let mut idx = Vec::with_capacity((count + flakes) * 6);
    let half = BOX / 2.0;
    for d in &rain.drops[..count] {
        let gust = 1.0 + w.gust * 0.35 * ((clock as f32) * 1.3 + d[4] * std::f32::consts::TAU).sin();
        let vel = Vec3::new(blow.x * gust, -FALL * (0.85 + 0.3 * d[3]), blow.z * gust);
        // Where the drop is in the box round the camera, straight from the clock.
        let wrap = |base: f32, v: f32, e: f32| (((base * BOX) as f64 + v as f64 * clock - e as f64).rem_euclid(BOX as f64)) as f32 - half;
        let rel = Vec3::new(wrap(d[0], vel.x, eye.x), wrap(d[1], vel.y, eye.y), wrap(d[2], vel.z, eye.z));
        let dist = rel.length();
        if dist < 0.7 || rel.dot(forward) < 0.3 {
            continue;
        }
        let speed = vel.length();
        let dir = vel / speed;
        let side = dir.cross(rel).normalize_or_zero() * (0.004 + dist * 0.0011);
        let edge = rel.abs().max_element();
        let a = 0.42 * smooth(half, half * 0.75, edge) * smooth(0.7, 2.2, dist);
        let head = eye + rel;
        let tail = head - dir * speed * BLUR;
        let k = pos.len() as u32;
        for (p, alpha) in [(head - side, a), (head + side, a), (tail + side, a * 0.25), (tail - side, a * 0.25)] {
            pos.push(p.to_array());
            col.push([tint[0], tint[1], tint[2], alpha]);
        }
        idx.extend_from_slice(&[k, k + 1, k + 2, k, k + 2, k + 3]);
    }
    // Snow: the drops from the other end of the list, slow and wandering,
    // each a small soft square facing the camera.
    let white = [0.95 * shade, 0.96 * shade, 1.0 * shade];
    for d in rain.drops[..most].iter().rev().take(flakes) {
        let t = clock as f32;
        let drift = Vec3::new((t * 0.9 + d[4] * 31.0).sin(), 0.0, (t * 0.7 + d[3] * 17.0).cos()) * 0.6;
        let vel = Vec3::new(blow.x, -SNOW_FALL * (0.7 + 0.6 * d[3]), blow.z);
        let wrap = |base: f32, v: f32, e: f32| (((base * BOX) as f64 + v as f64 * clock - e as f64).rem_euclid(BOX as f64)) as f32 - half;
        let rel = Vec3::new(wrap(d[0], vel.x, eye.x), wrap(d[1], vel.y, eye.y), wrap(d[2], vel.z, eye.z)) + drift * (0.3 + 0.7 * d[4]);
        let dist = rel.length();
        if dist < 0.7 || rel.dot(forward) < 0.3 {
            continue;
        }
        let size = 0.012 + dist * 0.0022 * (0.7 + 0.6 * d[3]);
        let edge = rel.abs().max_element();
        let a = 0.85 * smooth(half, half * 0.75, edge) * smooth(0.7, 2.2, dist);
        let at = eye + rel;
        // A soft dot: solid in the middle, fading to nothing at its corners.
        let k = pos.len() as u32;
        pos.push(at.to_array());
        col.push([white[0], white[1], white[2], a]);
        for (sx, sy) in [(-1.0, 0.0), (0.0, -1.0), (1.0, 0.0), (0.0, 1.0)] {
            pos.push((at + right * sx * size * 1.5 + over * sy * size * 1.5).to_array());
            col.push([white[0], white[1], white[2], 0.0]);
        }
        idx.extend_from_slice(&[k, k + 1, k + 2, k, k + 2, k + 3, k, k + 3, k + 4, k, k + 4, k + 1]);
    }
    if let Some(mut m) = meshes.get_mut(&rain.mesh) {
        *m = if idx.is_empty() { nothing() } else { build(pos, col, idx) };
    }
    if !rain.showing {
        rain.showing = true;
        for mut v in &mut vis {
            *v = Visibility::Inherited;
        }
    }
    view.spent += started.elapsed().as_secs_f32() * 1000.0;
}
