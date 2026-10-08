//! Light: sun, sky and fog, from the simulation's clock.

use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use gahturiyu_sim::sim::stealth;

use super::app::{Game, MainCamera};

const SKY: [f32; 3] = [0.63, 0.69, 0.72];
const NIGHT_SKY: [f32; 3] = [0.04, 0.05, 0.10];

#[derive(Component)]
pub struct Sun;

pub fn setup(mut commands: Commands) {
    commands.spawn((DirectionalLight { illuminance: 8000.0, shadow_maps_enabled: false, ..default() }, Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::new(0.45, -0.80, 0.35), Vec3::Y), Sun));
    commands.insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 600.0, ..default() });
}

pub fn update(mut commands: Commands, game: Res<Game>, mut sun: Query<&mut DirectionalLight, With<Sun>>, mut ambient: ResMut<GlobalAmbientLight>, mut clear: ResMut<ClearColor>, cam: Query<Entity, With<MainCamera>>) {
    let day = stealth::daylight(game.world.time);
    let sky = super::palette::mix(NIGHT_SKY, SKY, day);
    clear.0 = super::palette::bevy(sky);
    for mut s in &mut sun {
        s.illuminance = 8000.0 * day;
    }
    ambient.brightness = 150.0 + 450.0 * day;
    let r = game.orbit.draw_radius();
    let far = game.orbit.far_radius();
    if let Ok(e) = cam.single() {
        commands.entity(e).insert(DistanceFog { color: super::palette::bevy(sky), falloff: FogFalloff::Linear { start: r * 0.45, end: far * 0.95 }, ..default() });
    }
}
