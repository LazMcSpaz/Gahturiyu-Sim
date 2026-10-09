//! Lightning in the window: the flash that lights the whole scene, the bolt
//! itself, and the thunder that follows it (handed to the sound slots,
//! delayed by how far off the strike was).
//!
//! What is drawn runs on the window's own clock, not the world's: the game
//! can run at an hour a second, and lightning has to stay lightning. So
//! these flashes come as often as the storm overhead says, but they are not
//! the simulation's own strikes (`sim::weather::strikes`), which exist for
//! rules and omens.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::rng::Rng;

use super::WeatherView;
use crate::view::app::{Game, View};
use crate::view::scene::Scene3d;

/// A flash is over this many seconds after it strikes.
const FLASH: f64 = 0.45;
/// The bolt itself shows for this long.
const BOLT: f64 = 0.2;

/// A strike being shown.
#[derive(Clone, Copy, Debug)]
pub struct Shown {
    pub at: f64,
    pub pos: V2,
    pub power: f32,
    seed: u64,
    heard: bool,
}

#[derive(Component)]
pub struct BoltMesh;

#[derive(Resource)]
pub struct Bolts {
    mesh: Handle<Mesh>,
    live: Vec<Shown>,
    /// The last second of the clock looked at for new strikes.
    looked: i64,
    drawn: Option<u64>,
    /// `GAHT_FLASH=1`: a strike timed for the screenshot.
    staged: bool,
}

fn build(pos: Vec<[f32; 3]>, idx: Vec<u32>) -> Mesh {
    let n = pos.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; n])
        .with_inserted_indices(Indices::U32(idx))
}

fn nothing() -> Mesh {
    build(vec![[0.0, -1e4, 0.0]; 3], vec![0, 1, 2])
}

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(nothing());
    // Far brighter than white, so it blooms.
    let mat = materials.add(StandardMaterial { base_color: Color::linear_rgb(14.0, 16.0, 22.0), unlit: true, cull_mode: None, double_sided: false, fog_enabled: false, ..default() });
    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(mat), Transform::default(), Visibility::Hidden, NotShadowCaster, NoFrustumCulling, BoltMesh));
    commands.insert_resource(Bolts { mesh, live: Vec::new(), looked: i64::MIN, drawn: None, staged: std::env::var("GAHT_FLASH").is_ok() });
}

/// How bright a flash is `dt` seconds after it strikes: a crack, a flicker, gone.
fn glare(dt: f64) -> f32 {
    if !(0.0..FLASH).contains(&dt) {
        return 0.0;
    }
    let dt = dt as f32;
    let first = (-dt / 0.05).exp();
    let second = if dt > 0.13 { 0.7 * (-(dt - 0.13) / 0.06).exp() } else { 0.0 };
    (first + second).min(1.0)
}

/// A jagged ribbon from the cloud down to the ground, with a fork or two.
fn bolt(s: &Shown, ground: f32, top: f32, eye: Vec3) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut r = Rng::from_keys(&[0xB017, s.seed]);
    let foot = Vec3::new(s.pos.x, ground, s.pos.y);
    let far = foot.distance(eye);
    let width = (far * 0.0035).clamp(0.6, 30.0) * (0.6 + 0.4 * s.power);
    let mut pos = Vec::new();
    let mut idx = Vec::new();
    let ribbon = |points: &[Vec3], w: f32, pos: &mut Vec<[f32; 3]>, idx: &mut Vec<u32>| {
        for k in 0..points.len() {
            let along = if k + 1 < points.len() { points[k + 1] - points[k] } else { points[k] - points[k - 1] };
            let side = along.cross(points[k] - eye).normalize_or_zero() * w * (1.0 - 0.5 * k as f32 / points.len() as f32);
            pos.push((points[k] - side).to_array());
            pos.push((points[k] + side).to_array());
            if k > 0 {
                let i = pos.len() as u32 - 4;
                idx.extend_from_slice(&[i, i + 1, i + 3, i, i + 3, i + 2]);
            }
        }
    };
    // From the cloud down, wandering sideways as it comes.
    let lean = Vec3::new(r.range(-0.25, 0.25), 0.0, r.range(-0.25, 0.25)) * (top - ground);
    let steps = 14;
    let mut main = Vec::with_capacity(steps + 1);
    for k in 0..=steps {
        let f = k as f32 / steps as f32;
        let jag = if k == 0 || k == steps { Vec3::ZERO } else { Vec3::new(r.range(-1.0, 1.0), r.range(-0.3, 0.3), r.range(-1.0, 1.0)) * (top - ground) * 0.045 };
        main.push(foot + Vec3::Y * (top - ground) * (1.0 - f) + lean * (1.0 - f) + jag);
    }
    ribbon(&main, width, &mut pos, &mut idx);
    for _ in 0..2 {
        let from = 3 + r.below(7);
        let dir = Vec3::new(r.range(-1.0, 1.0), -0.9, r.range(-1.0, 1.0)).normalize();
        let mut fork = vec![main[from]];
        let reach = (top - ground) * r.range(0.12, 0.28);
        for k in 1..=5 {
            let jag = Vec3::new(r.range(-1.0, 1.0), 0.0, r.range(-1.0, 1.0)) * reach * 0.07;
            fork.push(main[from] + dir * reach * k as f32 / 5.0 + jag);
        }
        ribbon(&fork, width * 0.45, &mut pos, &mut idx);
    }
    (pos, idx)
}

pub fn update(game: Res<Game>, mut view: ResMut<WeatherView>, scene: Res<Scene3d>, bolts: Option<ResMut<Bolts>>, mut meshes: ResMut<Assets<Mesh>>, mut vis: Query<&mut Visibility, With<BoltMesh>>) {
    let Some(mut bolts) = bolts else { return };
    let started = std::time::Instant::now();
    let v = &mut *view;
    let clock = v.clock;
    let per_minute = v.here.lightning;
    let target = game.orbit.target;

    // ---- New strikes: one chance each second of the clock ------------------------
    let second = clock.floor() as i64;
    if bolts.looked == i64::MIN {
        bolts.looked = second - 1;
    }
    let from = bolts.looked.max(second - 3);
    for k in (from + 1)..=second {
        let mut r = Rng::from_keys(&[0x1167_4E, k as u64]);
        let (luck, when, bearing, out, power) = (r.f32(), r.f64(), r.f32() * std::f32::consts::TAU, r.f32(), 0.5 + 0.5 * r.f32());
        if per_minute > 0.0 && luck < per_minute / 60.0 {
            let away = 350.0 + 5500.0 * out * out;
            let pos = V2::new(target.x + bearing.cos() * away, target.y + bearing.sin() * away);
            bolts.live.push(Shown { at: k as f64 + when, pos, power, seed: k as u64, heard: false });
        }
    }
    bolts.looked = second;
    if bolts.staged {
        // For a screenshot: one strike, just before the picture is taken, off
        // to one side of where the camera looks.
        if let Some(shot) = &game.shot {
            bolts.staged = false;
            let eye = game.orbit.eye();
            let ahead = Vec2::new(target.x - eye.x, target.y - eye.z).normalize_or_zero();
            let side = Vec2::new(-ahead.y, ahead.x);
            let at = target.add(V2::new((ahead.x * 0.9 + side.x * 0.35) * 700.0, (ahead.y * 0.9 + side.y * 0.35) * 700.0));
            bolts.live.push(Shown { at: (shot.frames as f64 - 2.0) / 30.0, pos: at, power: 1.0, seed: 7, heard: false });
        }
    }

    // ---- The glare, the thunder to come, and the bolt on show ----------------------
    let mut flash = 0.0f32;
    let mut showing: Option<Shown> = None;
    for s in bolts.live.iter_mut() {
        let dt = clock - s.at;
        let away = s.pos.dist(target);
        if dt >= 0.0 && !s.heard {
            s.heard = true;
            v.sound.thunder(s.at, away, s.power);
        }
        flash = flash.max(glare(dt) * s.power * (1.0 - 0.6 * (away / 6000.0).min(1.0)));
        if (0.0..BOLT).contains(&dt) && showing.map(|o| o.at < s.at).unwrap_or(true) {
            showing = Some(*s);
        }
    }
    bolts.live.retain(|s| clock - s.at < FLASH);
    v.flash = flash;

    let scene_view = game.view == View::Scene;
    match showing.filter(|_| scene_view && v.quality.bolts) {
        Some(s) => {
            if bolts.drawn != Some(s.seed) {
                bolts.drawn = Some(s.seed);
                let ground = scene.grid.height(&game.world.terrain, s.pos);
                let top = ground + (v.here.cloud_base - ground).clamp(400.0, 1400.0);
                let (pos, idx) = bolt(&s, ground, top, game.orbit.eye());
                if let Some(mut m) = meshes.get_mut(&bolts.mesh) {
                    *m = build(pos, idx);
                }
                for mut x in &mut vis {
                    *x = Visibility::Inherited;
                }
            }
        }
        None => {
            if bolts.drawn.is_some() {
                bolts.drawn = None;
                for mut x in &mut vis {
                    *x = Visibility::Hidden;
                }
            }
        }
    }
    v.spent += started.elapsed().as_secs_f32() * 1000.0;
}
