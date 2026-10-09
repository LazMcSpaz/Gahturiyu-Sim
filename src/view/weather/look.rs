//! How the weather changes the light: the sun dims and cools under cloud,
//! the sky greys, storms go dark, haze and fog close in, and wet ground
//! darkens and shines.
//!
//! Runs straight after `light::update`, which sets the clear-day light for
//! the hour; this scales what it set.

use bevy::light::GlobalAmbientLight;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

use gahturiyu_sim::sim::stealth;
use gahturiyu_sim::sim::weather::{Region, Weather};

use super::WeatherView;
use crate::view::app::{Game, MainCamera};
use crate::view::light::{self, Moon, Sun};
use crate::view::palette::{self, Rgb};
use crate::view::scene::Mats;

/// Sky colours by day; by night everything sinks towards `NIGHT`.
const GREY_SKY: Rgb = [0.60, 0.63, 0.66];
const STORM_SKY: Rgb = [0.21, 0.24, 0.30];
const FOG_SKY: Rgb = [0.80, 0.82, 0.83];
const NIGHT: Rgb = [0.02, 0.025, 0.035];
/// The light that comes through cloud: cooler than the sun.
const CLOUD_LIGHT: Rgb = [0.86, 0.90, 0.98];
const SUN_WARM: Rgb = [1.0, 0.62, 0.38];
const FLASH_SKY: Rgb = [0.78, 0.82, 0.95];
/// Light a full lightning flash adds to everything, lux.
const FLASH_LIGHT: f32 = 9000.0;

/// How the light is scaled by a sky.
#[derive(Clone, Copy, Debug)]
pub struct Light {
    /// 0 clear .. 1 fully grey.
    pub grey: f32,
    /// Share of direct sunlight (and moonlight) that gets through.
    pub sun: f32,
    /// How much brighter the shaded sides are than on a clear day: cloud
    /// scatters the light it stops, so a grey day is flat, not dark.
    pub sky_day: f32,
    pub sky_night: f32,
}

pub fn light_for(w: &Weather) -> Light {
    let grey = smooth(0.3, 0.9, w.cloud);
    let dark = 1.0 - 0.6 * w.storm;
    Light { grey, sun: (1.0 - 0.82 * grey) * dark, sky_day: (1.0 + 3.6 * grey) * dark, sky_night: (1.0 - 0.4 * grey) * dark }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn rgb(c: Color) -> Rgb {
    let s = c.to_srgba();
    [s.red, s.green, s.blue]
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn apply(
    mut commands: Commands,
    game: Res<Game>,
    mut view: ResMut<WeatherView>,
    mats: Res<Mats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun: Query<&mut DirectionalLight, (With<Sun>, Without<Moon>)>,
    mut moon: Query<&mut DirectionalLight, (With<Moon>, Without<Sun>)>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
    cam: Query<Entity, With<MainCamera>>,
    mut wet_drawn: Local<f32>,
) {
    let started = std::time::Instant::now();
    let v = &mut *view;
    let (here, eye) = (v.here, v.eye);
    let day = stealth::daylight(game.world.time);
    let night = ((1.0 - day) / (1.0 - 0.12)).clamp(0.0, 1.0);
    let l = light_for(&here);

    // ---- Sun, moon and sky light ----------------------------------------------
    for mut s in &mut sun {
        s.illuminance *= l.sun;
        s.color = palette::bevy(palette::mix(rgb(s.color), CLOUD_LIGHT, l.grey * 0.8));
    }
    for mut m in &mut moon {
        m.illuminance *= l.sun;
    }
    ambient.brightness *= l.sky_day * (1.0 - night) + l.sky_night * night;
    ambient.color = palette::bevy(palette::mix(rgb(ambient.color), CLOUD_LIGHT, l.grey * 0.6));

    // ---- The sky ---------------------------------------------------------------
    let lit = |c: Rgb| palette::mix(NIGHT, c, day);
    let mut sky = palette::mix(rgb(clear.0), lit(GREY_SKY), l.grey);
    sky = palette::mix(sky, lit(STORM_SKY), here.storm * 0.85);
    sky = palette::mix(sky, lit(FOG_SKY), eye.fog * 0.9);
    // Lightning: for an instant the whole sky is lit from inside the cloud,
    // and everything under it stands out hard.
    if v.flash > 0.0 {
        sky = palette::mix(sky, FLASH_SKY, v.flash * 0.75);
        ambient.brightness += FLASH_LIGHT * v.flash;
        ambient.color = palette::bevy(palette::mix(rgb(ambient.color), [0.80, 0.86, 1.0], v.flash));
    }
    clear.0 = palette::bevy(sky);

    // ---- Haze ----------------------------------------------------------------------
    // How far the camera sees, from where it is: above a low fog it looks down
    // on it (the fog itself is drawn by `groundfog`); in it, or in rain, the
    // distance closes in. The spot being looked at is never quite lost, however
    // far out the camera is, or the game couldn't be played in a fog.
    let sight = eye.visibility.max(game.orbit.dist * 2.2);
    let r = game.orbit.draw_radius();
    let far = game.orbit.far_radius();
    let (_, height) = light::sun_at(game.world.time);
    let low = (1.0 - height / 0.45).clamp(0.0, 1.0);
    let dusk = (day * (1.0 - day) * 4.0).clamp(0.0, 1.0) * low.sqrt() * (1.0 - l.grey);
    if let Ok(e) = cam.single() {
        commands.entity(e).insert(DistanceFog {
            color: palette::bevy(sky),
            directional_light_color: palette::bevy(palette::scale(SUN_WARM, 0.5 * dusk)).with_alpha(0.6),
            directional_light_exponent: 12.0,
            falloff: FogFalloff::Linear { start: (r * 0.45).min(sight * 0.1), end: (far * 0.95).min(sight * 1.05) },
        });
    }

    // ---- Wet surfaces ----------------------------------------------------------------
    // Rain wets what it falls on at once; the ground stays wet for hours after.
    v.wet = here.wetness.max(here.rain * 1.4).min(1.0);
    if (v.wet - *wet_drawn).abs() > 0.01 || (v.wet == 0.0 && *wet_drawn != 0.0) {
        *wet_drawn = v.wet;
        if let Some(mut m) = materials.get_mut(&mats.lit) {
            let k = 1.0 - 0.3 * v.wet;
            m.base_color = Color::linear_rgb(k, k, k);
            m.perceptual_roughness = 0.92 - 0.5 * v.wet;
            m.reflectance = 0.2 + 0.3 * v.wet;
        }
    }

    // ---- Snow on the ground ------------------------------------------------------------
    // Where it lies: where the last days have been cold enough here, or in
    // the mountains if they are colder (it is their tops that show).
    let lying = here.snow_lies_above.min(v.skies[Region::Mountain as usize].snow_lying);
    palette::set_snow_line(lying);
    v.spent += started.elapsed().as_secs_f32() * 1000.0;
}
