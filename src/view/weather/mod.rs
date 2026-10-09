//! Weather in the window. Nothing here changes the weather: it is looked up
//! from the simulation (`sim::weather`) and shown.
//!
//! So far this is the Weather panel (F7): what the weather is doing where
//! the camera looks, the region, a 48-hour forecast, a way to look at other
//! times (the weather is worked out from the time alone, so looking ahead
//! is just asking about a later hour), and, on the map, the regions
//! coloured by their weather.
//!
//! `GAHT_WEATHER=1` opens the panel for screenshots (`GAHT_WEATHER=folded`:
//! just its headline); `GAHT_WEATHER_HOURS=h` looks h hours ahead (negative:
//! back).

mod panel;

use bevy::prelude::*;
use bevy_egui::egui;

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::weather::{Mix, Weather};

pub use panel::{map_colours, panel};

/// What the Weather panel keeps between frames. Drawing only.
#[derive(Resource, Default)]
pub struct WeatherView {
    /// The panel is open (F7).
    pub open: bool,
    /// Show only the headline (click the top of the panel).
    pub folded: bool,
    /// On the map, colour the land by its weather.
    pub overlay: bool,
    /// Hours added to the world's clock when looking: 0 is now.
    pub ahead: f64,
    /// The forecast last worked out: (world seed, region, first hour) and its hours.
    forecast: Option<((u64, u8, i64), Vec<Weather>)>,
    /// What is fixed about each texel of the map picture: its region shares,
    /// its height and its fog floor; and the lines between regions. Worked
    /// out once per world.
    land: Option<(Stamp, Vec<(Mix, f32, f32)>, Vec<(V2, V2)>)>,
    /// The map picture, and the moment (ten-minute step) it shows.
    picture: Option<(Stamp, i64, egui::TextureHandle)>,
}

/// Which world a cached picture belongs to: (seed, loads, land edits).
type Stamp = (u64, u32, u32);

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        let how = std::env::var("GAHT_WEATHER").ok();
        let open = how.is_some();
        let folded = how.as_deref() == Some("folded");
        let ahead = std::env::var("GAHT_WEATHER_HOURS").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        app.insert_resource(WeatherView { open, folded, overlay: open, ahead, ..Default::default() });
        app.add_systems(Update, keys);
    }
}

fn keys(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<WeatherView>) {
    if keys.just_pressed(KeyCode::F7) {
        view.open = !view.open;
    }
}
