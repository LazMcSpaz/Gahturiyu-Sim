//! The Weather panel (F7) and the map's weather colours.

use bevy::math::{vec2, Vec2};
use bevy_egui::egui::{self, Color32, ColorImage, Pos2, Rect, TextureOptions};

use gahturiyu_sim::sim::geo::{V2, WORLD_SIZE};
use gahturiyu_sim::sim::tide;
use gahturiyu_sim::sim::weather::{self, Kind, Region, Weather, REGIONS};
use gahturiyu_sim::sim::world::{DAY, HOUR};

use super::land::Stamp;
use super::presets::Preset;
use super::WeatherView;
use crate::view::app::{Game, View};
use crate::view::hud::{Canvas, PANEL};
use crate::view::palette::{eg, ega, Rgb, DIM, GOLD, TEXT, WARN};
use crate::view::squadui::{Bx, Click};

const W: f32 = 470.0;
const ROW: f32 = 20.0;
/// Hours in the forecast strip.
const AHEAD: usize = 48;
/// The map picture is this many texels a side.
const MAP: usize = 150;

/// The colour that stands for a kind of weather, on the strip and the map.
pub fn colour(k: Kind) -> Rgb {
    match k {
        Kind::Clear => [0.52, 0.74, 0.96],
        Kind::BrokenCloud => [0.68, 0.79, 0.88],
        Kind::Overcast => [0.52, 0.55, 0.58],
        Kind::Drizzle => [0.42, 0.53, 0.66],
        Kind::Rain => [0.28, 0.44, 0.70],
        Kind::HeavyRain => [0.18, 0.31, 0.64],
        Kind::Downpour => [0.10, 0.17, 0.50],
        Kind::Mist => [0.80, 0.83, 0.80],
        Kind::SeaMist => [0.76, 0.87, 0.86],
        Kind::Fog => [0.89, 0.89, 0.86],
        Kind::ThickFog => [0.98, 0.98, 0.95],
        Kind::HillFog => [0.72, 0.71, 0.78],
        Kind::Gale => [0.92, 0.64, 0.24],
        Kind::Storm => [0.86, 0.36, 0.20],
        Kind::Thunderstorm => [0.64, 0.34, 0.84],
        Kind::SnowFlurries => [0.80, 0.93, 0.98],
        Kind::Snow => [0.66, 0.90, 1.0],
        Kind::HeavySnow => [0.50, 0.84, 1.0],
        Kind::Blizzard => [0.36, 0.72, 0.96],
    }
}

fn clock(t: f64) -> String {
    let day = (t / DAY).floor() as i64 + 1;
    let secs = t.rem_euclid(DAY);
    format!("Day {}, {:02}:{:02}", day, (secs / HOUR) as i64, ((secs % HOUR) / 60.0) as i64)
}

fn sea_word(s: f32) -> &'static str {
    match s {
        s if s < 0.12 => "calm",
        s if s < 0.4 => "slight",
        s if s < 0.7 => "rough",
        _ => "heavy surf",
    }
}

fn rain_word(r: f32) -> &'static str {
    match r {
        r if r <= 0.03 => "none",
        r if r <= 0.2 => "drizzle",
        r if r <= 0.55 => "rain",
        r if r <= 0.8 => "heavy",
        _ => "downpour",
    }
}

/// Draw the panel (and the map's weather colours). Returns the boxes a
/// click shouldn't pass through.
pub fn panel(c: &Canvas, game: &Game, v: &mut WeatherView, click: Option<Click>) -> Vec<Bx> {
    if !v.open {
        return Vec::new();
    }
    let world = &game.world;
    let size = vec2(c.w, c.h);
    let t = world.time + v.ahead * HOUR;
    let r = Bx::new(c.w - W - 12.0, 64.0, W, 0.0);

    // Where we are looking: the camera's spot, or on the map the spot under
    // the mouse (so the regions can be read off by pointing).
    let at = match game.view {
        View::Scene => game.orbit.target,
        View::Map => {
            let over_panel = game.mouse.x > r.x - 6.0 && game.mouse.y > r.y - 6.0;
            if over_panel {
                game.map_cam.centre
            } else {
                game.map_cam.to_world(size, game.mouse)
            }
        }
    };
    let at = V2::new(at.x.clamp(0.0, WORLD_SIZE), at.y.clamp(0.0, WORLD_SIZE));
    // What is being shown there: the real weather, or the forced kind.
    let w = v.at(game, at);
    let shares = weather::mix(&world.terrain, at);

    // Folded up: just the headline. A click on the top of the panel folds and unfolds it.
    let head = Bx::new(r.x, r.y, r.w, 56.0);
    if click.map(|k| head.contains(k.at) && !k.right).unwrap_or(false) {
        v.folded = !v.folded;
    }
    if v.folded {
        let r = Bx::new(r.x, r.y, r.w, 84.0);
        c.rect(r.x, r.y, r.w, r.h, PANEL);
        c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
        c.text("Weather", r.x + 14.0, r.y + 26.0, 17.0, GOLD);
        c.rect(r.x + 14.0, r.y + 36.0, 14.0, 14.0, eg(colour(w.kind)));
        c.text(w.label(), r.x + 36.0, r.y + 49.0, 18.0, TEXT);
        let place = format!("{}  ·  {}", w.region.name(), clock(t));
        c.text(&place, r.x + 14.0, r.y + 72.0, 14.0, DIM);
        let nums = format!("{:.0}°C  ·  {:.0} m/s  ·  sight {}", w.temperature, w.wind, weather::sight_words(w.visibility));
        c.text(&nums, r.x + r.w - c.width(&nums, 14.0) - 14.0, r.y + 49.0, 14.0, DIM);
        c.text("click to unfold", r.x + r.w - 104.0, r.y + 26.0, 12.0, DIM);
        return vec![r];
    }

    // ---- Lay the panel out as lines first, so its box fits them ---------------
    let mut lines: Vec<(String, Rgb)> = Vec::new();
    let mut found: Vec<(Region, f32)> = Region::ALL.iter().map(|&r| (r, shares[r as usize])).filter(|x| x.1 > 0.05).collect();
    found.sort_by(|a, b| b.1.total_cmp(&a.1));
    let regions: Vec<String> = found.iter().map(|&(r, s)| if s > 0.95 { r.name().to_string() } else { format!("{} {:.0}%", r.name(), s * 100.0) }).collect();
    lines.push((format!("{}  ·  {}", clock(t), tide::season_name(t)), TEXT));
    lines.push((regions.join("  +  "), DIM));
    lines.push((String::new(), TEXT));
    for part in wrap(c, &w.describe(), W - 28.0, 15.0) {
        lines.push((part, TEXT));
    }
    lines.push((String::new(), TEXT));
    let numbers: [(String, String); 8] = [
        (format!("Cloud {:.0}%", w.cloud * 100.0), format!("Cloud base {:.0} m", w.cloud_base)),
        (format!("Rain {:.2} ({})", w.rain, rain_word(w.rain)), format!("Snow {:.2}", w.snow)),
        (format!("Wind {:.1} m/s, {}", w.wind, weather::quarter(w.wind_to)), format!("Gusts {:.0}%", w.gust * 100.0)),
        (format!("Fog {:.2} ({:.0} m deep here)", w.fog, w.fog_height), format!("Sight {}", weather::sight_words(w.visibility))),
        (format!("Temperature {:.1}°C", w.temperature), format!("Snowline {:.0} m", w.snowline)),
        (format!("Wet ground {:.0}%", w.wetness * 100.0), format!("Snow lies above {:.0} m", w.snow_lies_above.max(0.0))),
        (format!("Storm {:.0}%", w.storm * 100.0), format!("Lightning {:.1} a minute", w.lightning)),
        (format!("Sea {:.2} ({})", w.sea, sea_word(w.sea)), format!("Ground here {:.0} m", world.terrain.surface(at))),
    ];
    let numbers_at = lines.len();
    for _ in &numbers {
        lines.push((String::new(), TEXT));
    }
    let top = 62.0;
    let strip_y = r.y + top + lines.len() as f32 * ROW + 36.0;
    let regions_y = strip_y + 88.0;
    let buttons_y = regions_y + REGIONS as f32 * ROW + 22.0;
    let r = Bx::new(r.x, r.y, r.w, buttons_y + 4.0 * 28.0 + 62.0 - r.y);

    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    c.text("Weather", r.x + 14.0, r.y + 26.0, 17.0, GOLD);
    let label = w.label();
    c.rect(r.x + 14.0, r.y + 36.0, 14.0, 14.0, eg(colour(w.kind)));
    c.text(label, r.x + 36.0, r.y + 49.0, 18.0, TEXT);
    let note = match (v.force, v.ahead != 0.0) {
        (Some(p), _) => Some(format!("showing: {} (forced)", p.name())),
        (None, true) => Some(looking(v.ahead)),
        _ => None,
    };
    if let Some(s) = note {
        c.text(&s, r.x + r.w - c.width(&s, 14.0) - 14.0, r.y + 26.0, 14.0, WARN);
    }
    for (i, (l, col)) in lines.iter().enumerate() {
        c.text(l, r.x + 14.0, r.y + top + (i as f32 + 0.8) * ROW, 15.0, *col);
    }
    for (i, (a, b)) in numbers.iter().enumerate() {
        let y = r.y + top + ((numbers_at + i) as f32 + 0.8) * ROW;
        c.text(a, r.x + 14.0, y, 15.0, TEXT);
        c.text(b, r.x + 14.0 + W * 0.56, y, 15.0, TEXT);
    }

    // ---- The next 48 hours ---------------------------------------------------------
    let first = (t / HOUR).floor() as i64;
    let key = (world.seed, ((at.x / 200.0) as i64, (at.y / 200.0) as i64), first);
    if v.forecast.as_ref().map(|f| f.0 != key).unwrap_or(true) {
        v.forecast = Some((key, world.forecast_at(at, first as f64 * HOUR, AHEAD)));
    }
    let hours = &v.forecast.as_ref().unwrap().1;
    c.text(&format!("Next {AHEAD} hours here{}", if v.force.is_some() { " (the real weather)" } else { "" }), r.x + 14.0, strip_y - 8.0, 14.0, DIM);
    let cell = (W - 28.0) / AHEAD as f32;
    let mut tip = None;
    for (k, h) in hours.iter().enumerate() {
        let x = r.x + 14.0 + k as f32 * cell;
        c.rect(x, strip_y, cell + 0.5, 22.0, eg(colour(h.kind)));
        // Wind as a bar under the strip (full height = a gale).
        let bar = (h.wind / weather::GALE).min(1.4) * 10.0;
        c.rect(x, strip_y + 24.0, cell - 1.0, bar, ega(if h.wind >= weather::GALE { WARN } else { DIM }, 0.9));
        let hour = (first + k as i64).rem_euclid(24);
        if hour % 6 == 0 {
            c.rect(x, strip_y - 3.0, 1.0, 28.0, ega(TEXT, 0.5));
            c.text(&format!("{hour:02}"), x + 2.0, strip_y + 52.0, 12.0, DIM);
        }
        if Bx::new(x, strip_y, cell, 40.0).contains(game.mouse) {
            tip = Some((k, *h));
        }
    }

    // ---- Every region now ------------------------------------------------------------
    c.text("Every region at this hour", r.x + 14.0, regions_y - 6.0, 14.0, DIM);
    for (i, &reg) in Region::ALL.iter().enumerate() {
        let y = regions_y + (i as f32 + 0.8) * ROW;
        let typical = weather::climate().of(reg).ref_height;
        let rw = weather::localise(&v.skies[reg as usize], reg, typical, typical);
        c.rect(r.x + 14.0, y - 12.0, 12.0, 12.0, eg(colour(rw.kind)));
        c.text(reg.name(), r.x + 34.0, y, 14.0, if reg == w.region { GOLD } else { TEXT });
        c.text(rw.label(), r.x + 250.0, y, 14.0, TEXT);
        let nums = format!("{:.0}°C  {:.0} m/s", rw.temperature, rw.wind);
        c.text(&nums, r.x + r.w - c.width(&nums, 14.0) - 14.0, y, 14.0, DIM);
    }

    // ---- Looking at other times -----------------------------------------------------------
    let steps: [(&str, Option<f64>); 8] = [("-1 day", Some(-24.0)), ("-6 h", Some(-6.0)), ("-1 h", Some(-1.0)), ("now", None), ("+1 h", Some(1.0)), ("+6 h", Some(6.0)), ("+1 day", Some(24.0)), ("+season", Some(12.0 * 24.0))];
    let bw = (W - 28.0) / steps.len() as f32;
    for (i, (name, step)) in steps.iter().enumerate() {
        let b = Bx::new(r.x + 14.0 + i as f32 * bw, buttons_y, bw - 4.0, 24.0);
        let hot = b.contains(game.mouse);
        c.rect(b.x, b.y, b.w, b.h, ega(GOLD, if hot { 0.28 } else { 0.12 }));
        c.centred(name, b.x + b.w / 2.0, b.y + 17.0, 13.0, if step.is_none() && v.ahead == 0.0 { DIM } else { TEXT });
        if click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false) {
            v.ahead = step.map(|s| v.ahead + s).unwrap_or(0.0);
        }
    }
    let toggle = Bx::new(r.x + 14.0, buttons_y + 30.0, W - 28.0, 24.0);
    c.rect(toggle.x, toggle.y, toggle.w, toggle.h, ega(GOLD, if toggle.contains(game.mouse) { 0.2 } else { 0.08 }));
    c.text(&format!("Colour the map by weather: {}  (V shows the map)", if v.overlay { "on" } else { "off" }), toggle.x + 8.0, toggle.y + 17.0, 14.0, TEXT);
    if click.map(|k| toggle.contains(k.at) && !k.right).unwrap_or(false) {
        v.overlay = !v.overlay;
    }
    // ---- Showing a kind of weather on demand ------------------------------------------------
    let force = Bx::new(r.x + 14.0, buttons_y + 60.0, W - 28.0, 24.0);
    c.rect(force.x, force.y, force.w, force.h, ega(GOLD, if force.contains(game.mouse) { 0.2 } else { 0.08 }));
    let shown = v.force.map(|p| p.name()).unwrap_or("the real weather");
    c.text(&format!("Show: {shown}  (click or U for the next; right-click back)"), force.x + 8.0, force.y + 17.0, 14.0, if v.force.is_some() { WARN } else { TEXT });
    if let Some(k) = click.filter(|k| force.contains(k.at)) {
        v.force = Preset::step(v.force, k.right);
    }
    let much = Bx::new(r.x + 14.0, buttons_y + 90.0, W - 28.0, 24.0);
    c.rect(much.x, much.y, much.w, much.h, ega(GOLD, if much.contains(game.mouse) { 0.2 } else { 0.08 }));
    c.text(&format!("How much of it: {:.0}%  (click to change)", v.strength * 100.0), much.x + 8.0, much.y + 17.0, 14.0, if v.force.is_some() { TEXT } else { DIM });
    if click.map(|k| much.contains(k.at) && !k.right).unwrap_or(false) {
        v.strength = if v.strength > 0.99 { 0.25 } else { (v.strength + 0.25).min(1.0) };
    }
    // What would be heard (no sound files yet: these are the slots' volumes).
    let mut heard: Vec<String> = v.sound.loudest().iter().map(|(s, vol)| format!("{} {:.0}%", s.name(), vol * 100.0)).collect();
    if let Some((slot, when)) = v.sound.last.filter(|l| v.clock - l.1 < 4.0) {
        let _ = when;
        heard.push(format!("{}!", slot.name()));
    }
    let heard = if heard.is_empty() { "quiet".to_string() } else { heard.join("  ·  ") };
    c.text(&format!("Sound slots{}: {heard}", if v.sound.muffled { " (indoors)" } else { "" }), r.x + 14.0, r.y + r.h - 46.0, 12.0, DIM);
    c.text(&format!("Drawing the weather takes {:.2} ms a frame", v.cost_ms), r.x + 14.0, r.y + r.h - 28.0, 12.0, DIM);
    c.text("F7 closes  ·  click the top to fold  ·  on the map, point at a place to read it", r.x + 14.0, r.y + r.h - 10.0, 12.0, DIM);

    if let Some((k, h)) = tip {
        let when = clock((first + k as i64) as f64 * HOUR);
        let lines = vec![(format!("{when}: {}", h.label()), GOLD), (format!("{:.0}°C  ·  wind {:.0} m/s  ·  sight {}", h.temperature, h.wind, weather::sight_words(h.visibility)), TEXT)];
        c.panel(&lines, game.mouse.x - 260.0, game.mouse.y + 18.0, 14.0);
    }
    vec![r]
}

fn looking(hours: f64) -> String {
    let (n, unit) = if hours.abs() >= 48.0 { (hours.abs() / 24.0, "days") } else { (hours.abs(), "h") };
    format!("looking {:.0} {unit} {}", n, if hours > 0.0 { "ahead" } else { "back" })
}

/// Break a line into pieces no wider than `width`.
fn wrap(c: &Canvas, text: &str, width: f32, size: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let tried = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if c.width(&tried, size) > width && !line.is_empty() {
            out.push(std::mem::take(&mut line));
            line = word.to_string();
        } else {
            line = tried;
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// On the map: the land coloured by what the weather is doing everywhere,
/// with the regions' borders drawn in. Drawn straight after the map, so it
/// lies under the panels.
pub fn map_colours(c: &Canvas, ctx: &egui::Context, game: &Game, v: &mut WeatherView) {
    if !(v.open && v.overlay) {
        return;
    }
    let world = &game.world;
    let t = world.time + v.ahead * HOUR;
    let stamp: Stamp = (world.seed, game.loads, world.terrain.edits.version);
    let step = WORLD_SIZE / MAP as f32;
    if v.map.as_ref().map(|l| l.0 != stamp).unwrap_or(true) {
        let mut land = Vec::with_capacity(MAP * MAP);
        for j in 0..MAP {
            for i in 0..MAP {
                let p = V2::new((i as f32 + 0.5) * step, (j as f32 + 0.5) * step);
                land.push((weather::mix(&world.terrain, p), world.terrain.surface(p), weather::floor(&world.terrain, p)));
            }
        }
        // Where one region gives way to another: the texel edges between them.
        let region = |i: usize, j: usize| weather::strongest(&land[j * MAP + i].0);
        let mut borders = Vec::new();
        for j in 0..MAP {
            for i in 0..MAP {
                let (x, y) = (i as f32 * step, j as f32 * step);
                if i + 1 < MAP && region(i + 1, j) != region(i, j) {
                    borders.push((V2::new(x + step, y), V2::new(x + step, y + step)));
                }
                if j + 1 < MAP && region(i, j + 1) != region(i, j) {
                    borders.push((V2::new(x, y + step), V2::new(x + step, y + step)));
                }
            }
        }
        v.map = Some((stamp, land, borders));
        v.picture = None;
    }
    // Redrawn every ten game minutes, or when the forced weather changes.
    let forced = v.force.is_some();
    let tick = match v.force {
        Some(p) => p as i64 * 1000 + (v.strength * 100.0) as i64,
        None => (t / 600.0).floor() as i64,
    };
    if v.picture.as_ref().map(|p| p.0 != stamp || p.1 != tick || p.2 != forced).unwrap_or(true) {
        let land = &v.map.as_ref().unwrap().1;
        // The weather reaches each column of the map a little later than the
        // one to its west, so each column has its own skies.
        // The weather reaches each part of the map a little later than the
        // parts west of it: work the skies out for a run of delays, and give
        // each texel the nearest.
        const DELAYS: usize = 25;
        let whole = (weather::climate().crossing_hours as f64 * HOUR).max(1.0);
        let delays: Vec<_> = (0..DELAYS).map(|k| if forced { v.skies } else { weather::skies(world.seed, tick as f64 * 600.0, whole * k as f64 / (DELAYS - 1) as f64) }).collect();
        let delay_of = |k: usize| {
            let p = V2::new(((k % MAP) as f32 + 0.5) * step, ((k / MAP) as f32 + 0.5) * step);
            ((weather::lag(p) / whole * (DELAYS - 1) as f64).round() as usize).min(DELAYS - 1)
        };
        let px: Vec<Color32> = land
            .iter()
            .enumerate()
            .map(|(k, (shares, height, low))| {
                let w: Weather = weather::weather_with(&delays[delay_of(k)], shares, *height, *low);
                ega(colour(w.kind), 0.6)
            })
            .collect();
        let image = ColorImage::new([MAP, MAP], px);
        v.picture = Some((stamp, tick, forced, ctx.load_texture("weather-map", image, TextureOptions::NEAREST)));
    }
    let size = vec2(c.w, c.h);
    let a: Vec2 = game.map_cam.to_screen(size, V2::new(0.0, 0.0));
    let b: Vec2 = game.map_cam.to_screen(size, V2::new(WORLD_SIZE, WORLD_SIZE));
    let tex = &v.picture.as_ref().unwrap().3;
    c.p.image(tex.id(), Rect::from_min_max(Pos2::new(a.x, a.y), Pos2::new(b.x, b.y)), Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)), Color32::WHITE);
    let ink = Color32::from_rgba_unmultiplied(14, 16, 20, 210);
    for &(p, q) in &v.map.as_ref().unwrap().2 {
        let (p, q) = (game.map_cam.to_screen(size, p), game.map_cam.to_screen(size, q));
        if (p.x.max(q.x) >= 0.0 && p.x.min(q.x) <= c.w) && (p.y.max(q.y) >= 0.0 && p.y.min(q.y) <= c.h) {
            c.line(p, q, 1.5, ink);
        }
    }
}
