//! The land editor (F10): brushes to shape the land, paint its ground, grow
//! or clear plants and place rocks, with undo and saving to a map file.
//!
//! The brushes themselves are in the sim (`sim/mapedit.rs`); this is the
//! panel, the brush under the mouse, and the keys. The world stands still
//! while you edit. Leaving the editor finds the roads again if the land's
//! shape or ground changed.

use bevy::prelude::*;
use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    geo::V2,
    mapedit::{Brush, Dab, MapEdits, PLANTS, ROCKS, TEXTURES},
    World,
};

use super::app::{Game, View};
use super::hud::Canvas;
use super::palette::{self, eg, ega, Rgb, DIM, GOLD, TEXT, WARN};
use super::squadui::{Bx, Click};

/// The tools on the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Raise,
    Lower,
    Smooth,
    Roughen,
    Flatten,
    SetHeight,
    Terrace,
    Ramp,
    Restore,
    Paint(u8),
    Unpaint,
    More(u8),
    Fewer(u8),
    Natural,
    Rocks(u8),
    ClearRocks,
}

const SHAPE: [(Tool, &str); 9] = [
    (Tool::Raise, "Raise"),
    (Tool::Lower, "Lower"),
    (Tool::Smooth, "Smooth"),
    (Tool::Roughen, "Roughen"),
    (Tool::Flatten, "Flatten"),
    (Tool::SetHeight, "Set height"),
    (Tool::Terrace, "Terrace"),
    (Tool::Ramp, "Ramp / path"),
    (Tool::Restore, "Restore"),
];

/// What the brush settings are, and what's going on.
pub struct Editor {
    pub on: bool,
    pub tool: Tool,
    /// Metres.
    pub radius: f32,
    pub strength: f32,
    pub softness: f32,
    pub height: f32,
    pub step: f32,
    /// A stroke in progress (left button held over the land).
    pub stroking: bool,
    /// Flatten's height, taken where the stroke began.
    target: f32,
    /// A ramp's start.
    ramp_from: Option<V2>,
    /// The point under the mouse, if over the land.
    pub cursor: Option<V2>,
    /// Changes the window must redraw: (number, area), newest last.
    pub dirty: Vec<(u32, (V2, V2))>,
    dirty_n: u32,
    pending: Option<(V2, V2)>,
    last_flush: f32,
    /// The land's shape or ground changed since the roads were found.
    pub reshaped: bool,
    /// Edits not yet saved to the map file.
    pub unsaved: bool,
    /// "Clear all" or "Back to saved" asked once; ask again to do it.
    confirm_clear: bool,
    confirm_reload: bool,
    /// Whether the world was paused before the editor opened.
    was_paused: bool,
    pub status: String,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            on: false,
            tool: Tool::Raise,
            radius: 30.0,
            strength: 0.5,
            softness: 0.6,
            height: 40.0,
            step: 4.0,
            stroking: false,
            target: 0.0,
            ramp_from: None,
            cursor: None,
            dirty: Vec::new(),
            dirty_n: 0,
            pending: None,
            last_flush: 0.0,
            reshaped: false,
            unsaved: false,
            confirm_clear: false,
            confirm_reload: false,
            was_paused: false,
            status: String::new(),
        }
    }
}

impl Editor {
    /// The brush for this tool (Shift gives its opposite).
    fn brush(&self, shift: bool) -> Brush {
        match (self.tool, shift) {
            (Tool::Raise, false) | (Tool::Lower, true) => Brush::Raise,
            (Tool::Lower, false) | (Tool::Raise, true) => Brush::Lower,
            (Tool::Smooth, false) | (Tool::Roughen, true) => Brush::Smooth,
            (Tool::Roughen, false) | (Tool::Smooth, true) => Brush::Roughen,
            (Tool::Flatten, _) | (Tool::Ramp, _) => Brush::Flatten,
            (Tool::SetHeight, _) => Brush::SetHeight(self.height),
            (Tool::Terrace, _) => Brush::Terrace(self.step),
            (Tool::Restore, _) => Brush::Restore,
            (Tool::Paint(k), false) => Brush::Paint(k),
            (Tool::Paint(_), true) | (Tool::Unpaint, _) => Brush::Unpaint,
            (Tool::More(k), s) => Brush::Plants { kind: k, less: s },
            (Tool::Fewer(k), s) => Brush::Plants { kind: k, less: !s },
            (Tool::Natural, _) => Brush::NaturalPlants,
            (Tool::Rocks(k), false) => Brush::Rocks(k),
            (Tool::Rocks(_), true) | (Tool::ClearRocks, _) => Brush::ClearRocks,
        }
    }

    fn mark(&mut self, area: (V2, V2)) {
        self.pending = Some(match self.pending {
            Some((a, b)) => (V2::new(a.x.min(area.0.x), a.y.min(area.0.y)), V2::new(b.x.max(area.1.x), b.y.max(area.1.y))),
            None => area,
        });
    }

    /// Hand what's changed to the window to redraw (now and then mid-stroke).
    fn flush(&mut self) {
        if let Some(a) = self.pending.take() {
            self.dirty_n += 1;
            self.dirty.push((self.dirty_n, a));
            if self.dirty.len() > 64 {
                self.dirty.remove(0);
            }
        }
    }
}

/// F10: into the editor, or out (finding the roads again if need be).
pub fn toggle(game: &mut Game) {
    let e = &mut game.editor;
    if e.stroking {
        game.world.terrain.edits.end_stroke();
        e.stroking = false;
    }
    e.ramp_from = None;
    e.flush();
    e.on = !e.on;
    if e.on {
        e.was_paused = game.paused;
        game.paused = true;
        e.status = "Editing the land. The world stands still.".into();
    } else {
        if e.reshaped {
            game.world.refit_land();
            everything_changed(game);
            game.editor.reshaped = false;
            game.notice = Some(("The roads are found again over the edited land.".into(), std::time::Instant::now()));
        }
        game.paused = game.editor.was_paused;
    }
}

/// Save the edits to the map file for this world's seed.
pub fn save_map(game: &mut Game) {
    let path = super::app::map_path(game.world.seed);
    game.editor.status = match game.world.terrain.edits.save_to(game.world.seed, &path) {
        Ok(()) => {
            game.editor.unsaved = false;
            format!("Saved to {}. New games on seed {} start from it.", path.display(), game.world.seed)
        }
        Err(e) => format!("Couldn't save the map: {e}"),
    };
}

/// Put back the map as last saved.
fn reload_map(game: &mut Game) {
    let path = super::app::map_path(game.world.seed);
    match MapEdits::load_from(&path) {
        Ok((_, edits)) => {
            let old = std::mem::replace(&mut game.world.terrain.edits, edits);
            let e = &mut game.world.terrain.edits;
            e.version = old.version;
            e.ground_v = old.ground_v;
            e.rocks_v = old.rocks_v;
            e.changed(true, true);
            everything_changed(game);
            game.editor.unsaved = false;
            game.editor.status = "Back to the map as saved.".into();
        }
        Err(e) => game.editor.status = format!("No saved map to go back to ({e})."),
    }
}

fn everything_changed(game: &mut Game) {
    let e = &mut game.editor;
    e.mark((V2::new(-1e5, -1e5), V2::new(1e5, 1e5)));
    e.flush();
    e.reshaped = true;
}

/// The land under the mouse.
fn under(game: &Game, mouse: Vec2) -> Option<V2> {
    match game.view {
        View::Scene => game.orbit.ground_at(game.screen, mouse, &game.world.terrain),
        View::Map => Some(game.map_cam.to_world(game.screen, mouse)),
    }
}

/// The editor's share of the keys and mouse. Camera controls carry on as
/// usual; left-dragging over the land uses the brush.
pub fn input(game: &mut Game, keys: &ButtonInput<KeyCode>, buttons: &ButtonInput<MouseButton>, mouse: Vec2, on_panels: bool, dt: f32, time: f32) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    // (Not mid-stroke: the stroke would be kept against the wrong land.)
    let busy = game.editor.stroking;
    if ctrl && keys.just_pressed(KeyCode::KeyZ) && !busy {
        if shift { redo(game) } else { undo(game) }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) && !busy {
        redo(game);
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        save_map(game);
    }
    let e = &mut game.editor;
    if keys.just_pressed(KeyCode::BracketLeft) {
        e.radius = (e.radius / 1.25).max(3.0);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        e.radius = (e.radius * 1.25).min(MAX_RADIUS);
    }
    if keys.just_pressed(KeyCode::Minus) {
        e.strength = (e.strength - 0.1).max(0.05);
    }
    if keys.just_pressed(KeyCode::Equal) {
        e.strength = (e.strength + 0.1).min(1.0);
    }
    let at = under(game, mouse);
    game.editor.cursor = at;
    // A stroke ends when the button comes up, whatever else is held.
    if game.editor.stroking && !buttons.pressed(MouseButton::Left) {
        game.world.terrain.edits.end_stroke();
        game.editor.stroking = false;
        game.editor.flush();
    }
    // Ctrl-click picks up what's under the brush: its height, or its paint.
    if ctrl && buttons.just_pressed(MouseButton::Left) && !on_panels {
        if let Some(p) = at {
            let w = &game.world;
            match game.editor.tool {
                Tool::Paint(_) => {
                    if let Some((k, _)) = w.terrain.edits.paint_at(p) {
                        game.editor.tool = Tool::Paint(k);
                    }
                }
                _ => {
                    game.editor.height = w.terrain.height(p).round();
                    game.editor.tool = Tool::SetHeight;
                }
            }
        }
        return;
    }
    if ctrl {
        return;
    }
    // A ramp: from where the button went down to where it comes up.
    if game.editor.tool == Tool::Ramp {
        if buttons.just_pressed(MouseButton::Left) && !on_panels {
            game.editor.ramp_from = at;
        }
        if buttons.just_released(MouseButton::Left) {
            let from = game.editor.ramp_from.take();
            if let (Some(a), Some(b), false) = (from, at, on_panels) {
                if a.dist(b) > 2.0 {
                    let (r, soft) = (game.editor.radius, game.editor.softness);
                    let t = &mut game.world.terrain;
                    t.edits.begin_stroke();
                    if let Some(area) = t.ramp(a, b, r * 2.0, soft) {
                        game.editor.mark(area);
                    }
                    t.edits.end_stroke();
                    game.editor.flush();
                    game.editor.reshaped = true;
                    game.editor.unsaved = true;
                }
            }
        }
        return;
    }
    if buttons.just_pressed(MouseButton::Left) && !on_panels && at.is_some() {
        game.world.terrain.edits.begin_stroke();
        game.editor.stroking = true;
        game.editor.target = game.world.terrain.height(at.unwrap());
    }
    if game.editor.stroking {
        if let Some(p) = at {
            let e = &game.editor;
            let brush = e.brush(shift);
            let d = Dab { brush, at: p, radius: e.radius, strength: e.strength, falloff: e.softness, dt: dt.min(0.1), target: e.target };
            if let Some(area) = game.world.terrain.dab(d) {
                game.editor.mark(area);
                game.editor.unsaved = true;
                if brush.shapes() || matches!(brush, Brush::Paint(_) | Brush::Unpaint) {
                    game.editor.reshaped = true;
                }
            }
            // Plants and rocks are redrawn a few times a second while painting.
            if time - game.editor.last_flush > 0.25 {
                game.editor.flush();
                game.editor.last_flush = time;
            }
        }
        if !buttons.pressed(MouseButton::Left) {
            game.world.terrain.edits.end_stroke();
            game.editor.stroking = false;
            game.editor.flush();
        }
    }
}

fn undo(game: &mut Game) {
    if game.world.terrain.edits.undo() {
        everything_changed(game);
        game.editor.unsaved = true;
        game.editor.status = "Undone.".into();
    }
}

fn redo(game: &mut Game) {
    if game.world.terrain.edits.redo() {
        everything_changed(game);
        game.editor.unsaved = true;
        game.editor.status = "Redone.".into();
    }
}

// ---- Drawing ---------------------------------------------------------------------

const W: f32 = 330.0;
/// The biggest brush, metres across from the middle.
const MAX_RADIUS: f32 = 500.0;
const ROW: f32 = 21.0;

/// A clickable button; true if clicked this frame.
fn button(c: &Canvas, x: f32, y: f32, w: f32, label: &str, on: bool, click: Option<Click>, swatch: Option<Rgb>) -> bool {
    let b = Bx::new(x, y - 15.0, w, ROW - 2.0);
    c.rect(b.x, b.y, b.w, b.h, if on { ega(GOLD, 0.28) } else { Color32::from_rgba_unmultiplied(40, 44, 44, 200) });
    if on {
        c.rect_lines(b.x, b.y, b.w, b.h, 1.0, eg(GOLD));
    }
    let mut tx = x + 6.0;
    if let Some(col) = swatch {
        c.rect(x + 4.0, y - 12.0, 13.0, 13.0, eg(col));
        tx += 17.0;
    }
    c.text(label, tx, y, 13.0, if on { GOLD } else { TEXT });
    click.is_some_and(|k| !k.right && b.contains(k.at))
}

/// A value with − and + either side; returns −1, 0 or 1.
fn stepper(c: &Canvas, x: f32, y: f32, label: &str, value: &str, click: Option<Click>) -> i32 {
    c.text(label, x, y, 13.0, DIM);
    let bx = x + 92.0;
    let minus = button(c, bx, y, 24.0, "−", false, click, None);
    c.text(value, bx + 32.0, y, 13.0, TEXT);
    let plus = button(c, bx + 110.0, y, 24.0, "+", false, click, None);
    minus as i32 * -1 + plus as i32
}

/// The editor's panel, on the right. Returns where it is.
pub fn panel(c: &Canvas, game: &mut Game, click: Option<Click>) -> Bx {
    let h = 700.0f32.min(c.h - 24.0);
    let r = Bx::new(c.w - W - 12.0, 12.0, W, h);
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    let x = r.x + 12.0;
    let mut y = r.y + 28.0;
    c.text("Land editor", x, y, 18.0, GOLD);
    let leave = button(c, r.x + r.w - 92.0, y, 80.0, "Done (F10)", false, click, None);
    let e = &mut game.editor;
    let col = (W - 30.0) / 2.0;
    let pick = |e: &mut Editor, t: Tool, hit: bool| {
        if hit {
            e.tool = t;
        }
    };
    // Shape.
    y += 24.0;
    c.text("Shape the land", x, y, 14.0, TEXT);
    for (k, (t, name)) in SHAPE.iter().enumerate() {
        let (cx, cy) = (x + (k % 2) as f32 * (col + 6.0), y + ROW * (1 + k / 2) as f32);
        let hit = button(c, cx, cy, col, name, e.tool == *t, click, None);
        pick(e, *t, hit);
    }
    y += ROW * ((SHAPE.len() + 1) / 2) as f32 + 24.0;
    // Ground.
    c.text("Paint the ground", x, y, 14.0, TEXT);
    let cols = 3usize;
    let cw = (W - 24.0 - (cols - 1) as f32 * 6.0) / cols as f32;
    let n = TEXTURES.len() + 1;
    for k in 0..n {
        let (cx, cy) = (x + (k % cols) as f32 * (cw + 6.0), y + ROW * (1 + k / cols) as f32);
        if k < TEXTURES.len() {
            let t = Tool::Paint(k as u8);
            let hit = button(c, cx, cy, cw, TEXTURES[k].name, e.tool == t, click, Some(palette::texture(k as u8)));
            pick(e, t, hit);
        } else {
            let hit = button(c, cx, cy, cw, "Unpaint", e.tool == Tool::Unpaint, click, None);
            pick(e, Tool::Unpaint, hit);
        }
    }
    y += ROW * ((n + cols - 1) / cols) as f32 + 24.0;
    // Plants.
    c.text("Plants", x, y, 14.0, TEXT);
    for (k, name) in PLANTS.iter().enumerate() {
        let cy = y + ROW * (1 + k) as f32;
        c.text(name, x, cy, 13.0, DIM);
        let more = button(c, x + 70.0, cy, 70.0, "More", e.tool == Tool::More(k as u8), click, None);
        pick(e, Tool::More(k as u8), more);
        let fewer = button(c, x + 146.0, cy, 70.0, "Fewer", e.tool == Tool::Fewer(k as u8), click, None);
        pick(e, Tool::Fewer(k as u8), fewer);
    }
    let nat = button(c, x + 222.0, y + ROW, 84.0, "Natural", e.tool == Tool::Natural, click, None);
    pick(e, Tool::Natural, nat);
    y += ROW * 3.0 + 24.0;
    // Rocks.
    c.text("Rocks", x, y, 14.0, TEXT);
    for k in 0..ROCKS.len() + 1 {
        let (cx, cy) = (x + (k % cols) as f32 * (cw + 6.0), y + ROW * (1 + k / cols) as f32);
        if k < ROCKS.len() {
            let t = Tool::Rocks(k as u8);
            let hit = button(c, cx, cy, cw, ROCKS[k], e.tool == t, click, None);
            pick(e, t, hit);
        } else {
            let hit = button(c, cx, cy, cw, "Clear rocks", e.tool == Tool::ClearRocks, click, None);
            pick(e, Tool::ClearRocks, hit);
        }
    }
    y += ROW * 2.0 + 24.0;
    // The brush.
    c.text("Brush", x, y, 14.0, TEXT);
    y += ROW;
    let s = stepper(c, x, y, "Size", &format!("{:.0} m across", e.radius * 2.0), click);
    if s != 0 {
        e.radius = if s > 0 { (e.radius * 1.25).min(MAX_RADIUS) } else { (e.radius / 1.25).max(3.0) };
    }
    y += ROW;
    let s = stepper(c, x, y, "Strength", &format!("{:.0}%", e.strength * 100.0), click);
    e.strength = (e.strength + s as f32 * 0.1).clamp(0.05, 1.0);
    y += ROW;
    let s = stepper(c, x, y, "Soft edge", &format!("{:.0}%", e.softness * 100.0), click);
    e.softness = (e.softness + s as f32 * 0.1).clamp(0.0, 1.0);
    if e.tool == Tool::SetHeight {
        y += ROW;
        let s = stepper(c, x, y, "Height", &format!("{:.0} m", e.height), click);
        e.height = (e.height + s as f32 * 5.0).max(0.0);
    }
    if e.tool == Tool::Terrace {
        y += ROW;
        let s = stepper(c, x, y, "Step", &format!("{:.0} m", e.step), click);
        e.step = (e.step + s as f32).clamp(1.0, 40.0);
    }
    // Actions.
    y += ROW + 10.0;
    let bw = (W - 24.0 - 12.0) / 3.0;
    let can_undo = game.world.terrain.edits.can_undo();
    let can_redo = game.world.terrain.edits.can_redo();
    let undo_hit = button(c, x, y, bw, if can_undo { "Undo" } else { "(undo)" }, false, click, None);
    let redo_hit = button(c, x + bw + 6.0, y, bw, if can_redo { "Redo" } else { "(redo)" }, false, click, None);
    let save_hit = button(c, x + 2.0 * (bw + 6.0), y, bw, "Save map", game.editor.unsaved, click, None);
    y += ROW;
    let reload_label = if game.editor.confirm_reload { "Sure? Click again" } else { "Back to saved" };
    let reload_hit = button(c, x, y, bw * 1.5 + 3.0, reload_label, game.editor.confirm_reload, click, None);
    let clear_label = if game.editor.confirm_clear { "Sure? Click again" } else { "Clear all edits" };
    let clear_hit = button(c, x + bw * 1.5 + 9.0, y, bw * 1.5 + 3.0, clear_label, game.editor.confirm_clear, click, None);
    // What's under the brush, and how things stand.
    y += ROW + 6.0;
    if let Some(p) = game.editor.cursor {
        let t = &game.world.terrain;
        let paint = t.edits.paint_at(p).map(|(k, w)| format!(" (painted {} {:.0}%)", TEXTURES[k as usize].name.to_lowercase(), w * 100.0)).unwrap_or_default();
        let raised = t.height(p) - t.base_height(p);
        let line = format!("{:.0} m high{}, {:.0}° slope, {}{}", t.height(p), if raised.abs() > 0.05 { format!(" ({:+.1})", raised) } else { String::new() }, t.slope(p).atan().to_degrees(), t.ground(p).name(), paint);
        c.text(&line, x, y, 12.0, TEXT);
    }
    y += 16.0;
    let state = if game.editor.unsaved { ("Unsaved changes", WARN) } else { ("All saved", DIM) };
    c.text(state.0, x, y, 12.0, state.1);
    y += 16.0;
    for l in super::townui::wrap(c, &game.editor.status, 12.0, W - 24.0) {
        c.text(&l, x, y, 12.0, DIM);
        y += 15.0;
    }
    let hint = "Left-drag: use the brush · Shift: the opposite · Ctrl-click: pick up height or paint · [ ] size · − = strength · Ctrl+Z undo · Ctrl+S save · right-drag turns, wheel zooms";
    for l in super::townui::wrap(c, hint, 11.0, W - 24.0) {
        if y > r.y + r.h - 6.0 {
            break;
        }
        c.text(&l, x, y, 11.0, DIM);
        y += 14.0;
    }
    // Do what was clicked.
    if leave {
        toggle(game);
    }
    if undo_hit {
        undo(game);
    }
    if redo_hit {
        redo(game);
    }
    if save_hit {
        save_map(game);
    }
    if reload_hit {
        if game.editor.confirm_reload {
            reload_map(game);
            game.editor.confirm_reload = false;
        } else {
            game.editor.confirm_reload = true;
        }
    } else if click.is_some() {
        game.editor.confirm_reload = false;
    }
    if clear_hit {
        if game.editor.confirm_clear {
            game.world.terrain.edits.clear_all();
            everything_changed(game);
            game.editor.unsaved = true;
            game.editor.confirm_clear = false;
            game.editor.status = "Every edit cleared (Undo brings them back).".into();
        } else {
            game.editor.confirm_clear = true;
        }
    } else if click.is_some() {
        game.editor.confirm_clear = false;
    }
    r
}

/// The brush on the land: a ring where it reaches (and its soft edge), or a
/// ramp's line.
pub fn draw_cursor(c: &Canvas, game: &Game, project: &dyn Fn(Vec3) -> Option<Vec2>, drawn: &dyn Fn(V2) -> f32) {
    let e = &game.editor;
    let Some(p) = e.cursor else { return };
    let ring = |r: f32, col: Color32, width: f32| {
        let pts: Vec<Option<Vec2>> = (0..=48)
            .map(|i| {
                let a = i as f32 / 48.0 * std::f32::consts::TAU;
                let q = p.add(V2::new(a.cos() * r, a.sin() * r));
                project(vec3(q.x, drawn(q) + 0.6, q.y))
            })
            .collect();
        for w in pts.windows(2) {
            if let (Some(a), Some(b)) = (w[0], w[1]) {
                c.line(a, b, width, col);
            }
        }
    };
    ring(e.radius, eg(GOLD), 2.0);
    let hard = e.radius * (1.0 - e.softness);
    if hard > 1.0 && e.softness > 0.05 {
        ring(hard, ega(GOLD, 0.45), 1.0);
    }
    if let (Tool::Ramp, Some(a)) = (e.tool, e.ramp_from) {
        let n = 24;
        let mut last = None;
        for i in 0..=n {
            let q = a.lerp(p, i as f32 / n as f32);
            let s = project(vec3(q.x, drawn(q) + 0.6, q.y));
            if let (Some(l), Some(s)) = (last, s) {
                c.line(l, s, 3.0, eg(GOLD));
            }
            last = s;
        }
    }
}

/// A spot near `p` clear of towns and the sea (for the screenshot).
pub fn open_ground(w: &World, p: V2) -> V2 {
    for k in 0..200 {
        let a = k as f32 * 0.7;
        let q = p.add(V2::new(a.cos(), a.sin()).scale(150.0 + k as f32 * 15.0));
        let clear = w.settlements.iter().all(|s| s.pos.dist(q) > s.reach + 120.0);
        if clear && gahturiyu_sim::sim::geo::inland(q) > 200.0 && !w.terrain.on_road(q) && w.terrain.slope(q) < 0.15 {
            return q;
        }
    }
    p
}

/// For screenshots: a few strokes of each kind round a spot, so the picture
/// shows what the brushes do.
pub fn demo(w: &mut World, at: V2) {
    let t = &mut w.terrain;
    let dab = |t: &mut gahturiyu_sim::sim::terrain::Terrain, brush: Brush, p: V2, r: f32, n: usize| {
        t.edits.begin_stroke();
        for _ in 0..n {
            t.dab(Dab { brush, at: p, radius: r, strength: 1.0, falloff: 0.6, dt: 0.1, target: 0.0 });
        }
        t.edits.end_stroke();
    };
    dab(t, Brush::Raise, at.add(V2::new(-40.0, -30.0)), 55.0, 14);
    dab(t, Brush::Terrace(4.0), at.add(V2::new(-40.0, -30.0)), 60.0, 20);
    let sand = TEXTURES.iter().position(|x| x.name == "Sand").unwrap() as u8;
    let snow = TEXTURES.iter().position(|x| x.name == "Snow").unwrap() as u8;
    let mud = TEXTURES.iter().position(|x| x.name == "Mud").unwrap() as u8;
    dab(t, Brush::Paint(snow), at.add(V2::new(-40.0, -30.0)), 18.0, 30);
    dab(t, Brush::Paint(sand), at.add(V2::new(40.0, 20.0)), 30.0, 30);
    dab(t, Brush::Paint(mud), at.add(V2::new(0.0, 45.0)), 18.0, 30);
    dab(t, Brush::Plants { kind: 2, less: false }, at.add(V2::new(60.0, -50.0)), 35.0, 30);
    dab(t, Brush::Rocks(0), at.add(V2::new(30.0, 30.0)), 25.0, 12);
    dab(t, Brush::Rocks(4), at.add(V2::new(-10.0, 60.0)), 20.0, 8);
    dab(t, Brush::Rocks(2), at.add(V2::new(70.0, 10.0)), 12.0, 10);
    dab(t, Brush::Rocks(3), at.add(V2::new(40.0, -10.0)), 15.0, 10);
}
