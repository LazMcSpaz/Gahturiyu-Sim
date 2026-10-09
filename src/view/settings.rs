//! Graphics settings: how much the window draws. None of it changes the
//! world. Press O for the panel; click a row to change it. Saved to
//! `settings.txt` next to `assets/`, and read back when the game starts.

use bevy::prelude::*;

use super::hud::{Canvas, PANEL};
use super::palette::{eg, ega, DIM, GOLD, TEXT};
use super::squadui::{Bx, Click};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Off,
    Low,
    Medium,
    High,
}

impl Level {
    fn name(self) -> &'static str {
        match self {
            Level::Off => "off",
            Level::Low => "low",
            Level::Medium => "medium",
            Level::High => "high",
        }
    }
    fn parse(s: &str) -> Option<Level> {
        [Level::Off, Level::Low, Level::Medium, Level::High].into_iter().find(|l| l.name() == s)
    }
    fn next(self) -> Level {
        match self {
            Level::Off => Level::Low,
            Level::Low => Level::Medium,
            Level::Medium => Level::High,
            Level::High => Level::Off,
        }
    }
}

/// Point lights at most, by setting.
pub const LAMP_STEPS: [usize; 4] = [8, 16, 28, 40];

#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Settings {
    /// Sun shadows: off, low (shorter reach, coarser), medium, high.
    pub shadows: Level,
    /// Grass, bushes and trees: off, low (no grass, thinner woods, shorter
    /// reach), medium (half the grass), high (everything).
    pub foliage: Level,
    /// How many point lights (fires, torches, windows) at once.
    pub lamps: usize,
    /// The soft glow round bright things at night.
    pub bloom: bool,
    /// Rain, snow and low fog (`view/weather`): off, low, medium, high.
    pub weather: Level,
    /// Bumped on every change, so the parts that care can rebuild.
    pub version: u32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { shadows: Level::High, foliage: Level::High, lamps: 40, bloom: true, weather: Level::Medium, version: 0 }
    }
}

fn path() -> std::path::PathBuf {
    let assets = super::models::assets_dir();
    assets.parent().map(|p| p.join("settings.txt")).unwrap_or_else(|| "settings.txt".into())
}

impl Settings {
    pub fn load() -> Settings {
        let mut s = Settings::default();
        let Ok(text) = std::fs::read_to_string(path()) else { return s };
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "shadows" => s.shadows = Level::parse(v).unwrap_or(s.shadows),
                "foliage" => s.foliage = Level::parse(v).unwrap_or(s.foliage),
                "lamps" => s.lamps = v.parse().unwrap_or(s.lamps),
                "bloom" => s.bloom = v == "on",
                "weather" => s.weather = Level::parse(v).unwrap_or(s.weather),
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let text = format!(
            "# Gahturiyu graphics settings (drawing only; delete this file to reset)\nshadows = {}\nfoliage = {}\nlamps = {}\nbloom = {}\nweather = {}\n",
            self.shadows.name(),
            self.foliage.name(),
            self.lamps,
            if self.bloom { "on" } else { "off" },
            self.weather.name()
        );
        let _ = std::fs::write(path(), text);
    }

    /// How much of each plant kind grows, by setting: grass, bushes, trees.
    pub fn foliage_density(&self) -> (f32, f32, f32) {
        match self.foliage {
            Level::Off => (0.0, 0.0, 0.0),
            Level::Low => (0.0, 0.5, 0.6),
            Level::Medium => (0.5, 0.75, 1.0),
            Level::High => (1.0, 1.0, 1.0),
        }
    }

    /// How far out far trees are drawn, by setting.
    pub fn billboard_far(&self) -> f32 {
        match self.foliage {
            Level::Off => 0.0,
            Level::Low => 3500.0,
            Level::Medium => 5000.0,
            Level::High => super::foliage::BILLBOARD_FAR,
        }
    }

    /// How far sun shadows reach, as a share of the usual.
    pub fn shadow_reach(&self) -> f32 {
        match self.shadows {
            Level::Off => 0.0,
            Level::Low => 0.4,
            Level::Medium => 0.7,
            Level::High => 1.0,
        }
    }
}

const W: f32 = 330.0;
const ROW: f32 = 24.0;

/// The settings panel. Returns its box; a click on a row changes it.
pub fn panel(c: &Canvas, s: &mut Settings, mouse: Vec2, click: Option<Click>, frame_ms: f64) -> Bx {
    let r = Bx::new((c.w - W) / 2.0, 120.0, W, 54.0 + 6.0 * ROW + 30.0);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    c.text("Graphics", r.x + 14.0, r.y + 26.0, 17.0, GOLD);
    c.text(&format!("{:.0} fps", 1000.0 / frame_ms.max(0.1)), r.x + r.w - 70.0, r.y + 26.0, 14.0, DIM);
    let rows: [(&str, String); 5] = [
        ("Shadows", s.shadows.name().into()),
        ("Grass and trees", s.foliage.name().into()),
        ("Lights at once", s.lamps.to_string()),
        ("Glow", if s.bloom { "on".into() } else { "off".into() }),
        ("Rain, snow and fog", s.weather.name().into()),
    ];
    let mut changed = false;
    for (i, (label, value)) in rows.iter().enumerate() {
        let y = r.y + 54.0 + i as f32 * ROW;
        let row = Bx::new(r.x + 6.0, y - 16.0, r.w - 12.0, ROW);
        if row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.12));
        }
        c.text(label, r.x + 14.0, y, 15.0, TEXT);
        c.text(value, r.x + r.w - c.width(value, 15.0) - 14.0, y, 15.0, GOLD);
        if click.map(|k| row.contains(k.at) && !k.right).unwrap_or(false) {
            match i {
                0 => s.shadows = s.shadows.next(),
                1 => s.foliage = s.foliage.next(),
                2 => s.lamps = LAMP_STEPS[(LAMP_STEPS.iter().position(|&n| n == s.lamps).unwrap_or(3) + 1) % LAMP_STEPS.len()],
                3 => s.bloom = !s.bloom,
                _ => s.weather = s.weather.next(),
            }
            changed = true;
        }
    }
    c.text("Click a row to change it  ·  O to close  ·  saved for next time", r.x + 14.0, r.y + r.h - 12.0, 12.0, DIM);
    if changed {
        s.version += 1;
        s.save();
    }
    r
}
