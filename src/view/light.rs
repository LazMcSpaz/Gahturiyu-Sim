//! Day and night: sun, moon, sky and the fires and windows that light the
//! dark, all driven by the simulation's clock.
//!
//! The one source of truth for how light it is outdoors is
//! `stealth::daylight(t)`, the same number the simulation uses for who sees
//! whom. The sun's height and colour follow the hour; how strong it is
//! follows `daylight`. At night a moon and a little sky light take over,
//! scaled by `NIGHT_BRIGHTNESS`.
//!
//! The lamps drawn are the same sources the simulation counts in
//! `World::lights`: campfires, towns' lit windows, and torches (held and
//! standing). The nearest `MAX_LAMPS` to the camera are real point lights;
//! they flicker a little (drawing only).

use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use gahturiyu_sim::sim::{
    geo::V2,
    settlement::BuildingKind,
    stealth,
    world::{DAY, HOUR},
};

use super::app::{Game, MainCamera};
use super::cam::to3;
use super::palette::{self, Rgb};
use super::scene::{Mats, Scene3d};

/// How bright night is, 1 = as tuned. Raise it if the squad is hard to make
/// out at night; lower it for darker nights where fires matter more.
pub const NIGHT_BRIGHTNESS: f32 = 1.0;

/// Sunlight at noon, lux.
const SUN_LUX: f32 = 10_000.0;
/// Moonlight at midnight, lux (times NIGHT_BRIGHTNESS).
const MOON_LUX: f32 = 300.0;
/// Sky light: how much light reaches shaded sides by day and by night.
const DAY_AMBIENT: f32 = 700.0;
const NIGHT_AMBIENT: f32 = 90.0;
/// Hours the sun is above the horizon (it is fully up 07:00–19:00, as
/// `daylight` has it; rising from 05:00, set by 21:00).
const SUNRISE: f32 = 5.0;
const SUNSET: f32 = 21.0;
/// How high the sun climbs at noon (radians).
const NOON_HEIGHT: f32 = 1.0;
/// Point lights at most, nearest the camera first.
pub const MAX_LAMPS: usize = 40;
/// Brightness of a fire or torch light, lumens per unit of the simulation's
/// light power.
const FIRE_LUMENS: f32 = 2_400_000.0;
/// A lit window, lumens.
const WINDOW_LUMENS: f32 = 260_000.0;
/// Windows lit as real lights, nearest the camera.
const WINDOW_LAMPS: usize = 18;

const SKY: Rgb = [0.60, 0.70, 0.80];
const DUSK_SKY: Rgb = [0.78, 0.52, 0.40];
const NIGHT_SKY: Rgb = [0.025, 0.035, 0.08];
const SUN_WARM: Rgb = [1.0, 0.62, 0.38];
const MOON: Rgb = [0.62, 0.72, 1.0];
const FIRE: Rgb = [1.0, 0.62, 0.30];
const WINDOW: Rgb = [1.0, 0.72, 0.42];

#[derive(Component)]
pub struct Sun;
#[derive(Component)]
pub struct Moon;
#[derive(Component)]
pub struct Lamp(pub usize);

pub fn setup(mut commands: Commands) {
    let cascades = CascadeShadowConfigBuilder { num_cascades: 3, minimum_distance: 0.5, maximum_distance: 600.0, first_cascade_far_bound: 60.0, overlap_proportion: 0.2 }.build();
    commands.spawn((DirectionalLight { illuminance: SUN_LUX, shadow_maps_enabled: true, ..default() }, cascades, Transform::default(), Sun));
    commands.spawn((DirectionalLight { illuminance: 0.0, shadow_maps_enabled: false, color: palette::bevy(MOON), ..default() }, Transform::default().looking_to(Vec3::new(0.3, -0.85, -0.4), Vec3::Y), Moon));
    commands.insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: DAY_AMBIENT, ..default() });
    for i in 0..MAX_LAMPS {
        commands.spawn((PointLight { intensity: 0.0, range: 20.0, shadow_maps_enabled: false, color: palette::bevy(FIRE), ..default() }, Transform::default(), Visibility::Hidden, Lamp(i)));
    }
}

/// What the 3D camera needs for lights to glow: a wide range of brightness,
/// and a soft bloom round the brightest things.
pub fn camera_bundle() -> impl Bundle {
    (bevy::camera::Hdr, Bloom { intensity: 0.12, ..Bloom::NATURAL })
}

/// Which way the sun is (towards it), how high (0 at the horizon), and the
/// hour as a number 0..24.
pub fn sun_at(t: f64) -> (Vec3, f32) {
    let h = (t.rem_euclid(DAY) / HOUR) as f32;
    // Across the sky from east (+x) through south (+z) to west.
    let phase = ((h - SUNRISE) / (SUNSET - SUNRISE)).clamp(0.0, 1.0) * std::f32::consts::PI;
    let up = phase.sin() * NOON_HEIGHT;
    let dir = Vec3::new(phase.cos(), up.sin().max(0.02), phase.sin() * 0.55).normalize();
    (dir, up.sin().max(0.0))
}

#[allow(clippy::type_complexity)]
pub fn update(
    mut commands: Commands,
    game: Res<Game>,
    time: Res<Time>,
    scene: Res<Scene3d>,
    mats: Res<Mats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform, &mut bevy::light::CascadeShadowConfig), (With<Sun>, Without<Moon>, Without<Lamp>)>,
    mut moon: Query<&mut DirectionalLight, (With<Moon>, Without<Sun>)>,
    mut lamps: Query<(&Lamp, &mut PointLight, &mut Transform, &mut Visibility), (Without<Sun>, Without<Moon>)>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    cam: Query<Entity, With<MainCamera>>,
) {
    let w = &game.world;
    let day = stealth::daylight(w.time);
    let night = ((1.0 - day) / (1.0 - 0.12)).clamp(0.0, 1.0);
    let (to_sun, height) = sun_at(w.time);
    // Low sun is warm; at dawn and dusk the sky warms too.
    let low = (1.0 - height / 0.45).clamp(0.0, 1.0);
    let dusk = (day * (1.0 - day) * 4.0).clamp(0.0, 1.0) * low.sqrt();
    let sky = palette::mix(palette::mix(NIGHT_SKY, SKY, day), palette::scale(DUSK_SKY, day.sqrt()), dusk * 0.85);
    clear.0 = palette::bevy(sky);
    // Shadows reach as far as the camera's view of the near scene.
    let reach = (game.orbit.dist * 3.0).clamp(150.0, 1500.0);
    for (mut s, mut tf, mut cascades) in &mut sun {
        if (cascades.bounds.last().copied().unwrap_or(0.0) - reach).abs() > reach * 0.2 {
            *cascades = CascadeShadowConfigBuilder { num_cascades: 3, minimum_distance: 0.5, maximum_distance: reach, first_cascade_far_bound: reach * 0.12, overlap_proportion: 0.2 }.build();
        }
        s.illuminance = SUN_LUX * (1.0 - night) * (0.25 + 0.75 * (height / 0.3).min(1.0));
        s.color = palette::bevy(palette::mix([1.0, 0.97, 0.92], SUN_WARM, low));
        *tf = Transform::default().looking_to(-to_sun, Vec3::Y);
    }
    for mut m in &mut moon {
        m.illuminance = MOON_LUX * NIGHT_BRIGHTNESS * night;
    }
    ambient.brightness = DAY_AMBIENT * (1.0 - night) + NIGHT_AMBIENT * NIGHT_BRIGHTNESS * night;
    ambient.color = palette::bevy(palette::mix([1.0, 1.0, 1.0], [0.55, 0.65, 1.0], night));
    // Glowing things (windows, fires, flames) shine brighter as it darkens,
    // enough to bloom at night.
    if let Some(mut g) = materials.get_mut(&mats.glow) {
        let k = 1.0 + 5.0 * night;
        g.base_color = Color::linear_rgb(k, k, k);
    }
    let r = game.orbit.draw_radius();
    let far = game.orbit.far_radius();
    if let Ok(e) = cam.single() {
        commands.entity(e).insert(DistanceFog {
            color: palette::bevy(sky),
            directional_light_color: palette::bevy(palette::scale(SUN_WARM, 0.5 * dusk)).with_alpha(0.6),
            directional_light_exponent: 12.0,
            falloff: FogFalloff::Linear { start: r * 0.45, end: far * 0.95 },
        });
    }

    // ---- Lamps --------------------------------------------------------------
    let mut want: Vec<(Vec3, f32, f32, Rgb, f32)> = Vec::new(); // where, lumens, range, colour, flicker seed
    let target = game.orbit.target;
    let ground = |p: V2| scene.grid.height(&w.terrain, p);
    if day < 0.97 {
        for l in w.lights() {
            if l.flat || l.pos.dist(target) > r {
                continue;
            }
            let lift = if w.camps.iter().any(|c| c.pos.dist(l.pos) < 0.1) { 1.2 } else { 2.0 };
            want.push((to3(l.pos, ground(l.pos) + lift), l.power * FIRE_LUMENS * night.max(0.25), l.reach * 1.6, FIRE, l.pos.x * 0.37 + l.pos.y));
        }
        // Lit windows: the town's glow, as the nearest few windows.
        let mut windows: Vec<(f32, Vec3, f32)> = Vec::new();
        for s in &w.settlements {
            if s.pos.dist(target) > r * 0.6 {
                continue;
            }
            for bd in &s.buildings {
                if !matches!(bd.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock | BuildingKind::HoraroStilt) {
                    continue;
                }
                let face = bd.pos.add(V2::new(bd.rot.cos(), bd.rot.sin()).scale(bd.size * 0.5 + 1.5));
                let d = face.dist(target);
                windows.push((d, to3(face, ground(face) + 1.8), bd.seed as f32 * 1e-6));
            }
        }
        windows.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, p, seed) in windows.into_iter().take(WINDOW_LAMPS) {
            want.push((p, WINDOW_LUMENS * night, 9.0, WINDOW, seed));
        }
    }
    want.sort_by(|a, b| a.0.distance(game.orbit.look_at()).total_cmp(&b.0.distance(game.orbit.look_at())));
    let t = time.elapsed_secs();
    for (lamp, mut pl, mut tf, mut vis) in &mut lamps {
        match want.get(lamp.0) {
            Some(&(p, lumens, range, col, seed)) if lumens > 1.0 => {
                // A cheap flicker: two slow waves and a quick one.
                let f = 0.86 + 0.08 * (t * 7.3 + seed).sin() + 0.04 * (t * 13.1 + seed * 2.7).sin() + 0.02 * (t * 23.0 + seed).sin();
                pl.intensity = lumens * f;
                pl.range = range;
                pl.color = palette::bevy(col);
                tf.translation = p + Vec3::new(0.0, 0.05 * (t * 9.0 + seed).sin(), 0.0);
                *vis = Visibility::Inherited;
            }
            _ => {
                pl.intensity = 0.0;
                *vis = Visibility::Hidden;
            }
        }
    }
}
