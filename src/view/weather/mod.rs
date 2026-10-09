//! Weather in the window. Nothing here changes the weather: it is looked up
//! from the simulation (`sim::weather`) and shown.
//!
//! - `observe` works out, once a frame, the weather being shown: every
//!   region's sky, and the weather where the camera looks and at the
//!   camera's own height. Everything else here reads that (`WeatherView`).
//! - `look.rs`: light, sky colour, haze, wet surfaces and the snow line.
//! - `groundfog.rs`: the low fog that pools on water and in hollows.
//! - `rain.rs`: rain and snow round the camera.
//! - `lightning.rs`: flashes, bolts, and the thunder that follows.
//! - `sound.rs`: the sound slots and how loud each is (no files yet).
//! - `panel.rs`: the Weather panel (F7) and the map's weather colours.
//! - `presets.rs`: forced weather (U cycles through the kinds; Shift+U back).
//!
//! For screenshots: `GAHT_WEATHER=1` opens the panel (`folded`: just its
//! headline; `off`: nothing, for use with the flags below),
//! `GAHT_WEATHER_HOURS=h` looks h hours ahead (negative: back),
//! `GAHT_PRESET=clear|overcast|drizzle|seafog|downpour|gale|thunderstorm|snow`
//! forces a kind of weather and `GAHT_PRESET_STRENGTH=0..1` how much of it;
//! `GAHT_FLASH=1` times a lightning strike for the picture;
//! `GAHT_WEATHER_TAB=effects` opens the panel on what the weather does;
//! `GAHT_WEATHER_QUALITY=off|low|medium|high` draws at that setting whatever
//! `settings.txt` says; `GAHT_WEATHER_COST=1` prints what the drawing costs.
//!
//! How much is drawn follows the graphics setting "Rain, snow and fog"
//! (`Quality`): the light, haze, wet ground and snow line cost nothing and
//! are always on; the setting scales the drops, the fog sheets and how often
//! they are rebuilt.

mod groundfog;
mod land;
mod lightning;
mod look;
mod panel;
mod presets;
mod rain;
pub mod sound;

use bevy::prelude::*;
use bevy_egui::egui;

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::weather::{self, Mix, Omen, Region, Sky, Weather, REGIONS};
use gahturiyu_sim::sim::world::HOUR;

use super::app::Game;
use super::settings::{Level, Settings};
use land::{LandGrid, Stamp};
pub use panel::{map_colours, panel};
pub use presets::Preset;

/// How much weather is drawn, by the graphics setting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quality {
    /// Drops (or flakes) in the heaviest fall.
    pub drops: usize,
    /// Sheets of low fog, one over the other: 0, 1 or 2.
    pub fog_sheets: usize,
    /// Cells across a fog sheet.
    pub fog_cells: usize,
    /// How many times a second the fog sheets are redraped.
    pub fog_hz: f64,
    /// How much of a redraping is done in one frame: this many of the
    /// sheet's points (so no one frame pays for the whole sheet).
    pub fog_slice: usize,
    /// Whether lightning bolts are drawn (the flash always is).
    pub bolts: bool,
}

impl Quality {
    /// The most drops any setting asks for.
    pub const MOST_DROPS: usize = 6500;

    pub fn of(level: Level) -> Quality {
        match level {
            Level::Off => Quality { drops: 0, fog_sheets: 0, fog_cells: 32, fog_hz: 1.0, fog_slice: 600, bolts: false },
            Level::Low => Quality { drops: 1500, fog_sheets: 1, fog_cells: 40, fog_hz: 1.0, fog_slice: 600, bolts: true },
            Level::Medium => Quality { drops: 4200, fog_sheets: 2, fog_cells: 64, fog_hz: 2.0, fog_slice: 900, bolts: true },
            Level::High => Quality { drops: Quality::MOST_DROPS, fog_sheets: 2, fog_cells: 96, fog_hz: 4.0, fog_slice: 1800, bolts: true },
        }
    }
}

/// The weather being shown, and what the Weather panel keeps between
/// frames. Drawing only.
#[derive(Resource)]
pub struct WeatherView {
    /// The panel is open (F7).
    pub open: bool,
    /// Show only the headline (click the top of the panel).
    pub folded: bool,
    /// On the map, colour the land by its weather.
    pub overlay: bool,
    /// Hours added to the world's clock when looking: 0 is now.
    pub ahead: f64,
    /// A kind of weather shown instead of the real one, and how much of it.
    pub force: Option<Preset>,
    pub strength: f32,
    /// The panel shows what the weather does (sight, travel, boats ...)
    /// in place of the weather's own numbers.
    pub effects_tab: bool,
    /// How much is drawn (from the graphics settings).
    pub quality: Quality,
    /// A setting to draw at whatever the graphics settings say (screenshots).
    pub quality_forced: Option<Level>,

    // ---- Worked out each frame by `observe` -------------------------------------
    /// The moment being shown, game seconds.
    pub time: f64,
    /// Every region's sky, in `Region::ALL` order.
    pub skies: [Sky; REGIONS],
    /// The weather where the camera looks, on the ground.
    pub here: Weather,
    /// The weather at the camera itself (above low fog, it sees over it).
    pub eye: Weather,
    /// How wet surfaces are drawn, 0..1. Materials may read this.
    pub wet: f32,
    /// The wind this instant, gusts and all: the way it blows (x east, y
    /// south) times its speed in metres a second. For anything that sways.
    pub wind: Vec2,
    /// A lightning flash lighting the scene, 0..1.
    pub flash: f32,
    /// The sound slots: how loud each loop is, and one-shots to start.
    pub sound: sound::Mixer,
    /// Seconds for things that move (drawing only).
    pub clock: f64,
    /// Milliseconds a frame spent working out the weather's drawing,
    /// smoothed, and the worst single frame of the last few seconds.
    pub cost_ms: f32,
    pub worst_ms: f32,
    spent: f32,
    worst_so_far: (f32, u32),

    land: LandGrid,
    /// The forecast last worked out: (world seed, about where, first hour) and its hours.
    forecast: Option<((u64, (i64, i64), i64), Vec<Weather>)>,
    /// The omens last worked out: (world seed, loads, about where, day) and what they were.
    omens: Option<((u64, u32, (i64, i64), i64), Vec<Omen>)>,
    /// What is fixed about each texel of the map picture: its region shares,
    /// its height and its fog floor; and the lines between regions. Worked
    /// out once per world.
    map: Option<(Stamp, Vec<(Mix, f32, f32)>, Vec<(V2, V2)>)>,
    /// The map picture, and the moment (ten-minute step) it shows.
    picture: Option<(Stamp, i64, bool, egui::TextureHandle)>,
}

impl Default for WeatherView {
    fn default() -> Self {
        let calm = weather::localise(&Sky::default(), Region::Lowland, 0.0, 0.0);
        WeatherView {
            open: false,
            folded: false,
            overlay: false,
            ahead: 0.0,
            force: None,
            strength: 1.0,
            effects_tab: false,
            quality: Quality::of(Level::Medium),
            quality_forced: None,
            time: 0.0,
            skies: [Sky::default(); REGIONS],
            here: calm,
            eye: calm,
            wet: 0.0,
            wind: Vec2::ZERO,
            flash: 0.0,
            sound: sound::Mixer::default(),
            clock: 0.0,
            cost_ms: 0.0,
            worst_ms: 0.0,
            spent: 0.0,
            worst_so_far: (0.0, 0),
            land: LandGrid::default(),
            forecast: None,
            omens: None,
            map: None,
            picture: None,
        }
    }
}

impl WeatherView {
    /// The weather shown at a spot on the ground.
    pub fn at(&self, game: &Game, p: V2) -> Weather {
        let t = &game.world.terrain;
        weather::weather_with(&self.skies, &self.land.mix(p), t.surface(p), weather::floor(t, p))
    }
}

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        let var = |k: &str| std::env::var(k).ok();
        let how = var("GAHT_WEATHER");
        let open = how.as_deref().map(|h| h != "off").unwrap_or(false);
        let view = WeatherView {
            open,
            folded: how.as_deref() == Some("folded"),
            overlay: open,
            ahead: var("GAHT_WEATHER_HOURS").and_then(|v| v.parse().ok()).unwrap_or(0.0),
            force: var("GAHT_PRESET").and_then(|v| Preset::parse(&v)),
            strength: var("GAHT_PRESET_STRENGTH").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            effects_tab: var("GAHT_WEATHER_TAB").as_deref() == Some("effects"),
            quality_forced: var("GAHT_WEATHER_QUALITY").and_then(|v| match v.as_str() {
                "off" => Some(Level::Off),
                "low" => Some(Level::Low),
                "medium" => Some(Level::Medium),
                "high" => Some(Level::High),
                _ => None,
            }),
            ..Default::default()
        };
        app.insert_resource(view);
        app.add_systems(Startup, (groundfog::setup, rain::setup, lightning::setup));
        app.add_systems(Update, (keys, observe, lightning::update, look::apply, groundfog::update, rain::update).chain().after(super::light::update));
    }
}

fn keys(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<WeatherView>, mut game: ResMut<Game>) {
    if keys.just_pressed(KeyCode::F7) {
        view.open = !view.open;
    }
    if keys.just_pressed(KeyCode::KeyU) && !game.editor.on {
        let back = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        view.force = Preset::step(view.force, back);
        let msg = match view.force {
            Some(p) => format!("Weather shown: {} (forced; U for the next, Shift+U back)", p.name()),
            None => "Weather shown: the real weather".to_string(),
        };
        game.notice = Some((msg, std::time::Instant::now()));
    }
}

/// Work out the weather being shown this frame.
fn observe(game: Res<Game>, mut view: ResMut<WeatherView>, time: Res<Time>, settings: Res<Settings>) {
    let v = &mut *view;
    v.cost_ms += (v.spent - v.cost_ms) * 0.1;
    // The worst frame of each few seconds (the fog sheets are redraped a
    // few times a second, which costs more than the frames between).
    // (The very first frames set things up once; they are not counted.)
    v.worst_so_far = (if game.frame < 20 { 0.0 } else { v.worst_so_far.0.max(v.spent) }, v.worst_so_far.1 + 1);
    if v.worst_so_far.1 >= 150 {
        v.worst_ms = v.worst_so_far.0;
        if std::env::var("GAHT_WEATHER_COST").is_ok() {
            eprintln!("weather cost: {:.2} ms a frame, worst {:.2} ms", v.cost_ms, v.worst_ms);
        }
        v.worst_so_far = (0.0, 0);
    } else {
        v.worst_ms = v.worst_ms.max(v.worst_so_far.0);
    }
    v.spent = 0.0;
    v.quality = Quality::of(v.quality_forced.unwrap_or(settings.weather));
    let world = &game.world;
    let stamp: Stamp = (world.seed, game.loads, world.terrain.edits.version);
    if !v.land.is_for(stamp) && !game.editor.stroking {
        // Once per world (a fraction of a second); not counted as a frame's cost.
        v.land.build(&world.terrain, stamp);
    }
    let started = std::time::Instant::now();
    // Screenshots run on a fixed clock, so a set of flags gives one picture.
    v.clock = if game.shot.is_some() { game.frame as f64 / 30.0 } else { time.elapsed_secs_f64() };
    v.time = world.time + v.ahead * HOUR;
    v.skies = match v.force {
        Some(p) => [p.sky(v.strength); REGIONS],
        // The weather as it has reached the spot the camera looks at. (Across the
        // view it differs by minutes at most.)
        None => weather::skies(world.seed, v.time, weather::lag(game.orbit.target)),
    };
    let at = game.orbit.target;
    v.here = v.at(&game, at);
    let eye = game.orbit.eye();
    let under = V2::new(eye.x, eye.z);
    let t = &world.terrain;
    v.eye = weather::weather_with(&v.skies, &v.land.mix(under), eye.y.max(t.surface(under)), weather::floor(t, under));
    // The wind comes in gusts: a slow swell and a quicker flutter on top.
    let c = v.clock as f32;
    let gust = 1.0 + v.here.gust * (0.55 * (c * 0.83).sin() + 0.3 * (c * 2.1 + 1.0).sin() + 0.15 * (c * 5.3).sin());
    v.wind = Vec2::new(v.here.wind_to.x, v.here.wind_to.y) * v.here.wind * gust.max(0.2);
    // What the listener hears: under a roof if the camera is with a squad that
    // has all gone indoors; the sea within half a mile; less from far overhead.
    let squad = &world.squad;
    let ears = sound::Ears {
        indoors: game.follow && !squad.inside.is_empty() && squad.inside.iter().all(|d| d.is_some()),
        by_sea: 1.0 - (gahturiyu_sim::sim::geo::inland(at).abs() / 800.0).clamp(0.0, 1.0),
        closeness: 1.0 - 0.7 * ((game.orbit.dist - 150.0) / 1200.0).clamp(0.0, 1.0),
    };
    let heard = v.here;
    v.sound.update(&heard, ears, v.clock);
    v.spent += started.elapsed().as_secs_f32() * 1000.0;
}
