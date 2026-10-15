//! The editor's Build tool (F10, then Build): Laz lays a place out in 3D on
//! the PC, with the game's own placeholder buildings, workplaces, pens and
//! props, as he does on the phone site; both write the same place files
//! (`assets/world/places/<key>.json`, `sim/placefile.rs`).
//!
//! Pick a thing from the tray, see it on the ground under the mouse, click to
//! put it down (as many as you like; right-click or Escape to stop). Click a
//! placed thing to pick it: drag it, Q / E to turn it (Shift: finer), [ and ]
//! to size it, Delete to take it away. A place is a mark's area (its dashed
//! box); whatever is put down inside it is that place's. Ctrl+S saves.
//! "Rebuild the world" starts the world again from the saved places, so the
//! town is laid out, peopled and worked as built.
//!
//! Drawing only, until saved: the world's town stands hidden while its place
//! is open here, and what's drawn is the place file as it stands.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use bevy::prelude::*;
use bevy_egui::egui::Color32;
use gahturiyu_sim::sim::{
    atlas,
    catalogue::{self, Entry},
    geo::V2,
    grounds, layout,
    placefile::{PlaceFile, Thing},
    rng,
};

use super::app::Game;
use super::hud::Canvas;
use super::mesh::Builder;
use super::palette::{self, eg, ega, Rgb, DIM, GOLD, TEXT, WARN};
use super::squadui::{Bx, Click};

/// The tray's tabs: (title, catalogue group, people).
const TABS: [(&str, &str, Option<&str>); 6] = [("Roduro", "building", Some("roduro")), ("Qotiro", "building", Some("qotiro")), ("Horaro", "building", Some("horaro")), ("Work", "workplace", None), ("Pens", "pen", None), ("Props", "prop", None)];
/// Rows of the tray shown at once.
const TRAY_ROWS: usize = 9;
/// Open places farther than this from the camera aren't drawn.
const DRAW_REACH: f32 = 2500.0;

/// The catalogue, once.
fn catalogue_list() -> &'static [Entry] {
    static C: OnceLock<Vec<Entry>> = OnceLock::new();
    C.get_or_init(catalogue::catalogue)
}

fn entry(key: &str) -> Option<&'static Entry> {
    catalogue_list().iter().find(|e| e.key == key)
}

/// A place open in the editor: its file as it stands, and where it is.
pub struct Open {
    pub file: PlaceFile,
    pub centre: V2,
    pub label: String,
    /// The world's town for it, if it is one (hidden while it's open here).
    pub sid: Option<u16>,
    pub unsaved: bool,
}

#[derive(Default)]
pub struct Build {
    pub on: bool,
    tab: usize,
    scroll: usize,
    /// The catalogue key being put down, its turn (degrees) and size.
    armed: Option<String>,
    rot: f32,
    scale: f32,
    /// A placed thing picked: its place's key and its index there.
    picked: Option<(String, usize)>,
    /// Dragging the picked thing: where it was grabbed, from its middle.
    drag: Option<V2>,
    pub places: BTreeMap<String, Open>,
    /// The place under the mouse now (its key).
    under: Option<String>,
    pub cursor: Option<V2>,
    pub status: String,
}

impl Build {
    /// Whether the world's town `sid` is being built here (and so hidden).
    pub fn hides(&self, sid: u16) -> bool {
        self.on && self.places.values().any(|o| o.sid == Some(sid))
    }

    pub fn unsaved(&self) -> usize {
        self.places.values().filter(|o| o.unsaved).count()
    }

    /// Escape: let go of what's in hand first. True if it did something.
    pub fn escape(&mut self) -> bool {
        if self.on && (self.armed.is_some() || self.picked.is_some()) {
            self.armed = None;
            self.picked = None;
            self.drag = None;
            return true;
        }
        false
    }
}

/// The place at a spot, opened (read from its file, or a new empty one at
/// its kind's area) if it isn't already. Its key.
fn open_at(b: &mut Build, p: V2) -> Option<String> {
    let (key, label, kind, centre) = grounds::place_at(p)?;
    if !b.places.contains_key(&key) {
        let file = PlaceFile::for_key(&key).unwrap_or_else(|| PlaceFile::blank(&key, &label, grounds::area_of(&kind)));
        let sid = atlas::town_marks().iter().position(|m| m.key() == key).map(|i| i as u16);
        b.places.insert(key.clone(), Open { file, centre, label, sid, unsaved: false });
    }
    Some(key)
}

/// A thing's footprint in the world: its middle, the way it faces (unit),
/// half its depth along that way and half its width across.
fn footprint(o: &Open, t: &Thing) -> Option<(V2, V2, f32, f32)> {
    let e = entry(&t.k)?;
    let a = t.rot.to_radians();
    Some((o.centre.add(V2::new(t.x, t.y)), V2::new(a.cos(), a.sin()), e.depth * t.s / 2.0, e.width * t.s / 2.0))
}

fn inside((c, f, hd, hw): (V2, V2, f32, f32), p: V2) -> bool {
    let d = p.sub(c);
    let along = d.x * f.x + d.y * f.y;
    let across = -d.x * f.y + d.y * f.x;
    along.abs() <= hd && across.abs() <= hw
}

fn corners((c, f, hd, hw): (V2, V2, f32, f32)) -> [V2; 4] {
    let s = V2::new(-f.y, f.x);
    [c.add(f.scale(hd)).add(s.scale(hw)), c.add(f.scale(hd)).sub(s.scale(hw)), c.sub(f.scale(hd)).sub(s.scale(hw)), c.sub(f.scale(hd)).add(s.scale(hw))]
}

/// The placed thing under a spot (the smallest, if several).
fn thing_at(b: &Build, p: V2) -> Option<(String, usize)> {
    let mut best: Option<(f32, String, usize)> = None;
    for (key, o) in &b.places {
        for (i, t) in o.file.things.iter().enumerate() {
            let Some(fp) = footprint(o, t) else { continue };
            if inside(fp, p) {
                let area = fp.2 * fp.3;
                if best.as_ref().is_none_or(|b| area < b.0) {
                    best = Some((area, key.clone(), i));
                }
            }
        }
    }
    best.map(|(_, k, i)| (k, i))
}

fn new_id() -> String {
    let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("t{:x}", n & 0xFFFF_FFFF_FFFF)
}

/// Save every place changed here. Returns what was said.
pub fn save_places(game: &mut Game) -> String {
    let b = &mut game.editor.build;
    let mut done = Vec::new();
    let mut failed = Vec::new();
    for o in b.places.values_mut().filter(|o| o.unsaved) {
        match o.file.save() {
            Ok(_) => {
                o.unsaved = false;
                done.push(o.label.clone());
            }
            Err(e) => failed.push(format!("{}: {e}", o.label)),
        }
    }
    let said = if !failed.is_empty() {
        format!("Couldn't save {}", failed.join("; "))
    } else if done.is_empty() {
        "Nothing new to save.".into()
    } else {
        format!("Saved {}. Rebuild the world to see it lived in.", done.join(", "))
    };
    b.status = said.clone();
    said
}

/// The Build tool's share of the keys and mouse (the camera carries on).
pub fn input(game: &mut Game, keys: &ButtonInput<KeyCode>, buttons: &ButtonInput<MouseButton>, at: Option<V2>, on_panels: bool) {
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        save_places(game);
        return;
    }
    let b = &mut game.editor.build;
    b.cursor = at;
    b.under = at.and_then(grounds::place_at).map(|p| p.0);
    let turn = if keys.just_pressed(KeyCode::KeyQ) {
        -1.0
    } else if keys.just_pressed(KeyCode::KeyE) {
        1.0
    } else {
        0.0
    } * if shift { 5.0 } else { 15.0 };
    let grow = if keys.just_pressed(KeyCode::BracketRight) {
        1.1
    } else if keys.just_pressed(KeyCode::BracketLeft) {
        1.0 / 1.1
    } else {
        1.0
    };
    if buttons.just_pressed(MouseButton::Right) && !on_panels {
        b.armed = None;
        b.picked = None;
        b.drag = None;
    }
    if b.armed.is_some() {
        b.rot = (b.rot + turn).rem_euclid(360.0);
        b.scale = (b.scale * grow).clamp(0.4, 4.0);
    } else if let Some((key, i)) = b.picked.clone() {
        if let Some(o) = b.places.get_mut(&key) {
            if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
                if i < o.file.things.len() {
                    let t = o.file.things.remove(i);
                    o.unsaved = true;
                    b.status = format!("Took away the {}.", entry(&t.k).map(|e| e.name.to_lowercase()).unwrap_or(t.k));
                }
                b.picked = None;
                b.drag = None;
                return;
            }
            if let Some(t) = o.file.things.get_mut(i) {
                let e = entry(&t.k);
                if turn != 0.0 && e.is_none_or(|e| e.rotatable) {
                    t.rot = (t.rot + turn).rem_euclid(360.0);
                    o.unsaved = true;
                }
                if grow != 1.0 && e.is_none_or(|e| e.scalable) {
                    t.s = ((t.s * grow).clamp(0.4, 4.0) * 100.0).round() / 100.0;
                    o.unsaved = true;
                }
            }
        }
    }
    let Some(p) = at else { return };
    if buttons.just_pressed(MouseButton::Left) && !on_panels {
        if let Some(k) = b.armed.clone() {
            let Some(key) = open_at(b, p) else {
                b.status = "That's outside every place: put it inside a mark's dashed box.".into();
                return;
            };
            let (rot, s) = (b.rot, b.scale);
            let o = b.places.get_mut(&key).expect("just opened");
            let (x, y) = ((p.x - o.centre.x) * 10.0, (p.y - o.centre.y) * 10.0);
            o.file.things.push(Thing { id: new_id(), k: k.clone(), x: x.round() / 10.0, y: y.round() / 10.0, rot: rot.round(), s: (s * 100.0).round() / 100.0, label: String::new(), room: None, note: String::new(), use_: String::new() });
            o.unsaved = true;
            let n = o.file.things.len() - 1;
            b.status = format!("Put down a {} in {}.", entry(&k).map(|e| e.name.to_lowercase()).unwrap_or(k), o.label);
            b.picked = Some((key, n));
        } else {
            // Open the place clicked in, so its things can be picked.
            let _ = open_at(b, p);
            b.picked = thing_at(b, p);
            b.drag = b.picked.as_ref().and_then(|(key, i)| {
                let o = b.places.get(key)?;
                let t = o.file.things.get(*i)?;
                Some(p.sub(o.centre.add(V2::new(t.x, t.y))))
            });
        }
    }
    if let (Some(off), Some((key, i)), true) = (b.drag, b.picked.clone(), buttons.pressed(MouseButton::Left)) {
        if let Some(o) = b.places.get_mut(&key) {
            if let Some(t) = o.file.things.get_mut(i) {
                let q = p.sub(off).sub(o.centre);
                let (x, y) = ((q.x * 10.0).round() / 10.0, (q.y * 10.0).round() / 10.0);
                if (x, y) != (t.x, t.y) {
                    (t.x, t.y) = (x, y);
                    o.unsaved = true;
                }
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        b.drag = None;
    }
}

/// A thing as the game draws it, into the frame's meshes: a building as its
/// variant from outside; a workplace's ground and posts; a pen's fence; a
/// prop as a block.
fn draw_thing(b: &mut Builder, gl: &mut Builder, t: &Thing, at: V2, on_ground: &dyn Fn(V2) -> f32) {
    let Some(e) = entry(&t.k) else { return };
    let rot = t.rot.to_radians();
    let ground = on_ground(at);
    let seed = rng::key(&[rng::key(&[t.id.len() as u64]), (at.x * 10.0) as i64 as u64, (at.y * 10.0) as i64 as u64]);
    match e.group {
        "building" => {
            if let Some(v) = layout::by_key(&t.k) {
                let (usual, _) = layout::example_of(v);
                // (As its sign makes it, on its own footprint.)
                let shown = catalogue::variant_for_sign(&t.k, &t.use_).and_then(layout::by_key).unwrap_or(v);
                super::interiors::variant(b, gl, shown, at, rot, usual * t.s, seed, ground, 0.0);
            }
        }
        "pen" => {
            let r = e.radius.unwrap_or(e.depth / 2.0) * t.s;
            let n = ((r * std::f32::consts::TAU / 2.5) as usize).max(8);
            for k in 0..n {
                let a = k as f32 / n as f32 * std::f32::consts::TAU;
                let q = at.add(V2::new(a.cos() * r, a.sin() * r));
                b.column(Vec3::new(q.x, on_ground(q), q.y), 0.08, 0.07, 1.1, 4, palette::TIMBER);
            }
        }
        _ => {
            // A workplace's yard, or a prop: its footprint on the ground, posts at the corners.
            let (f, hd, hw) = (V2::new(rot.cos(), rot.sin()), e.depth * t.s / 2.0, e.width * t.s / 2.0);
            let s = V2::new(-f.y, f.x);
            let cs = [at.add(f.scale(hd)).add(s.scale(hw)), at.add(f.scale(hd)).sub(s.scale(hw)), at.sub(f.scale(hd)).sub(s.scale(hw)), at.sub(f.scale(hd)).add(s.scale(hw))];
            let col: Rgb = if e.group == "workplace" { [0.42, 0.34, 0.24] } else { [0.45, 0.43, 0.40] };
            let up = |q: V2, h: f32| Vec3::new(q.x, on_ground(q) + h, q.y);
            if e.group == "workplace" {
                b.quad([up(cs[0], 0.12), up(cs[1], 0.12), up(cs[2], 0.12), up(cs[3], 0.12)], Vec3::Y, col);
                for q in cs {
                    b.column(up(q, 0.0), 0.12, 0.1, 1.6, 4, palette::TIMBER);
                }
            } else {
                b.block(up(at, 0.0), e.depth * t.s, e.width * t.s, (e.depth.min(e.width) * t.s * 0.6).clamp(0.3, 3.0), rot, col);
            }
        }
    }
}

/// The open places' things, and what's in hand under the mouse, into the
/// frame's meshes (the window calls this while the editor is open).
pub fn draw(b: &mut Builder, gl: &mut Builder, game: &Game, on_ground: &dyn Fn(V2) -> f32) {
    let bs = &game.editor.build;
    if !bs.on {
        return;
    }
    let near = game.orbit.target;
    for o in bs.places.values() {
        if o.centre.dist(near) > DRAW_REACH {
            continue;
        }
        for t in &o.file.things {
            draw_thing(b, gl, t, o.centre.add(V2::new(t.x, t.y)), on_ground);
        }
    }
    if let (Some(k), Some(p)) = (&bs.armed, bs.cursor) {
        let t = Thing { id: "ghost".into(), k: k.clone(), x: 0.0, y: 0.0, rot: bs.rot, s: bs.scale, label: String::new(), room: None, note: String::new(), use_: String::new() };
        draw_thing(b, gl, &t, p, on_ground);
    }
}

/// Lines on the screen: every open place's box (and the one under the
/// mouse), the picked thing's outline, and the ghost's (red where it can't go).
pub fn draw_marks(c: &Canvas, game: &Game, project: &dyn Fn(Vec3) -> Option<Vec2>, drawn: &dyn Fn(V2) -> f32) {
    let bs = &game.editor.build;
    if !bs.on {
        return;
    }
    let line = |a: V2, z: V2, col: Color32, width: f32| {
        let n = ((a.dist(z) / 6.0) as usize).max(1);
        let mut last = None;
        for i in 0..=n {
            let q = a.lerp(z, i as f32 / n as f32);
            let s = project(Vec3::new(q.x, drawn(q) + 0.5, q.y));
            if let (Some(l), Some(s)) = (last, s) {
                c.line(l, s, width, col);
            }
            last = s;
        }
    };
    let outline = |cs: [V2; 4], col: Color32, width: f32| {
        for k in 0..4 {
            line(cs[k], cs[(k + 1) % 4], col, width);
        }
    };
    // The place under the mouse (opened or not) and every open one.
    let mut boxes: Vec<(V2, (usize, usize), bool)> = bs.places.iter().map(|(k, o)| (o.centre, (o.file.size.0 as usize, o.file.size.1 as usize), bs.under.as_deref() == Some(k.as_str()))).collect();
    if let Some(p) = bs.cursor {
        if let Some((key, _, kind, c0)) = grounds::place_at(p) {
            if !bs.places.contains_key(&key) {
                boxes.push((c0, grounds::area_of(&kind), true));
            }
        }
    }
    for (c0, (gw, gh), hot) in boxes {
        if c0.dist(game.orbit.target) > DRAW_REACH {
            continue;
        }
        let (x0, y0) = (c0.x - (gw / 2) as f32 * grounds::CELL, c0.y - (gh / 2) as f32 * grounds::CELL);
        let (x1, y1) = (x0 + gw as f32 * grounds::CELL, y0 + gh as f32 * grounds::CELL);
        outline([V2::new(x0, y0), V2::new(x1, y0), V2::new(x1, y1), V2::new(x0, y1)], if hot { eg(GOLD) } else { ega(GOLD, 0.45) }, if hot { 2.0 } else { 1.0 });
    }
    if let Some((key, i)) = &bs.picked {
        if let Some(fp) = bs.places.get(key).and_then(|o| o.file.things.get(*i).and_then(|t| footprint(o, t))) {
            outline(corners(fp), eg(GOLD), 2.5);
        }
    }
    if let (Some(k), Some(p)) = (&bs.armed, bs.cursor) {
        if let Some(e) = entry(k) {
            let a = bs.rot.to_radians();
            let fp = (p, V2::new(a.cos(), a.sin()), e.depth * bs.scale / 2.0, e.width * bs.scale / 2.0);
            let ok = grounds::place_at(p).is_some() && !game.world.terrain.is_sea(p);
            outline(corners(fp), if ok { eg(GOLD) } else { eg(WARN) }, 2.0);
        }
    }
}

/// The Build tool's part of the editor's panel, from `y` down inside `r`.
#[allow(clippy::too_many_arguments)]
pub fn panel(c: &Canvas, game: &mut Game, click: Option<Click>, r: Bx, mut y: f32, button: &dyn Fn(&Canvas, f32, f32, f32, &str, bool, Option<Click>) -> bool) {
    let x = r.x + 12.0;
    let w = r.w - 24.0;
    let b = &mut game.editor.build;
    if b.scale == 0.0 {
        b.scale = 1.0;
    }
    // The tray: tabs, then the things of that tab.
    let tw = (w - 10.0) / 3.0;
    for (k, (title, _, _)) in TABS.iter().enumerate() {
        let (tx, ty) = (x + (k % 3) as f32 * (tw + 5.0), y + (k / 3) as f32 * 21.0);
        if button(c, tx, ty, tw, title, b.tab == k, click) {
            b.tab = k;
            b.scroll = 0;
        }
    }
    y += 21.0 * 2.0 + 6.0;
    let (_, group, people) = TABS[b.tab];
    let list: Vec<&Entry> = catalogue_list().iter().filter(|e| e.group == group && (people.is_none() || e.people == people)).collect();
    let first = b.scroll.min(list.len().saturating_sub(TRAY_ROWS));
    for (row, e) in list.iter().skip(first).take(TRAY_ROWS).enumerate() {
        let label = if e.missing { format!("{} (no shape yet)", e.name) } else { e.name.clone() };
        if button(c, x, y + row as f32 * 21.0, w, &label, b.armed.as_deref() == Some(e.key.as_str()), click) {
            b.armed = if b.armed.as_deref() == Some(e.key.as_str()) { None } else { Some(e.key.clone()) };
            b.picked = None;
            b.rot = 0.0;
            b.scale = 1.0;
        }
    }
    y += TRAY_ROWS as f32 * 21.0;
    if list.len() > TRAY_ROWS {
        if button(c, x, y, 60.0, "Up", false, click) {
            b.scroll = first.saturating_sub(TRAY_ROWS - 1);
        }
        if button(c, x + 66.0, y, 60.0, "Down", false, click) {
            b.scroll = (first + TRAY_ROWS - 1).min(list.len().saturating_sub(TRAY_ROWS));
        }
        c.text(&format!("{}-{} of {}", first + 1, (first + TRAY_ROWS).min(list.len()), list.len()), x + 136.0, y, 12.0, DIM);
        y += 22.0;
    }
    y += 8.0;
    // What's in hand, or picked (a sign chosen is put on once the panel's drawn).
    let mut signed: Option<(String, usize, String)> = None;
    if let Some(e) = b.armed.as_deref().and_then(entry) {
        c.text(&format!("Putting down: {}", e.name), x, y, 14.0, GOLD);
        y += 18.0;
        c.text(&format!("{:.0}° · size {:.2} · Q/E turn · [ ] size", b.rot, b.scale), x, y, 12.0, DIM);
        y += 18.0;
    } else if let Some((key, i)) = b.picked.clone() {
        let o = b.places.get(&key);
        if let (Some(o), Some(t)) = (o, o.and_then(|o| o.file.things.get(i))) {
            let e = entry(&t.k);
            c.text(&format!("{} in {}", e.map(|e| e.name.clone()).unwrap_or(t.k.clone()), o.label), x, y, 14.0, GOLD);
            y += 18.0;
            let jobs = e.map(|e| e.jobs.join(", ")).unwrap_or_default();
            c.text(&format!("{:.0}° · size {:.2}{}", t.rot, t.s, if jobs.is_empty() { String::new() } else { format!(" · {jobs}") }), x, y, 12.0, DIM);
            y += 18.0;
            c.text("Drag to move · Q/E turn · [ ] size · Delete", x, y, 12.0, DIM);
            y += 18.0;
            // A building's sign: what it's used as.
            if e.is_some_and(|e| e.group == "building") {
                let now = t.use_.clone();
                c.text("Sign:", x, y + 4.0, 13.0, TEXT);
                y += 22.0;
                let cw = (w - 6.0) / 2.0;
                let mut choice: Option<String> = None;
                let all: Vec<(&str, String)> = std::iter::once(("", "Home (no sign)".to_string())).chain(catalogue::SIGNS.iter().map(|s| (s.key, s.name.split(" (").next().unwrap_or(s.name).to_string()))).collect();
                for (k, (key, name)) in all.iter().enumerate() {
                    let (bx, by) = (x + (k % 2) as f32 * (cw + 6.0), y + (k / 2) as f32 * 20.0);
                    let swaps = !key.is_empty() && catalogue::variant_for_sign(&t.k, key).is_some();
                    let label = if swaps { format!("{name} *") } else { name.clone() };
                    if button(c, bx, by, cw, &label, now == *key, click) {
                        choice = Some(key.to_string());
                    }
                }
                y += ((all.len() + 1) / 2) as f32 * 20.0 + 2.0;
                c.text("* this people has its own building for it", x, y, 11.0, DIM);
                y += 16.0;
                if let Some(k) = choice {
                    signed = Some((key.clone(), i, k));
                }
            }
        }
    } else {
        c.text("Pick a thing, then click inside a place's box.", x, y, 12.0, DIM);
        y += 18.0;
        c.text("Click a placed thing to move or turn it.", x, y, 12.0, DIM);
        y += 18.0;
    }
    if let Some((key, i, k)) = signed {
        if let Some(o) = b.places.get_mut(&key) {
            if let Some(t) = o.file.things.get_mut(i) {
                t.use_ = k;
                o.unsaved = true;
            }
        }
    }
    // The place under the mouse.
    if let Some(key) = b.under.clone() {
        let (label, n, room) = match b.places.get(&key) {
            Some(o) => (o.label.clone(), o.file.things.len(), o.file.capacity()),
            None => (grounds::marks().into_iter().find(|m| m.0 == key).map(|m| m.2).unwrap_or(key.clone()), 0, 0),
        };
        c.text(&format!("{label}: {n} things, room for {room}"), x, y, 12.0, TEXT);
        y += 18.0;
    }
    y += 6.0;
    let n = b.unsaved();
    let save = button(c, x, y, w / 2.0 - 3.0, &if n > 0 { format!("Save places ({n})") } else { "Saved".into() }, false, click);
    let rebuild = button(c, x + w / 2.0 + 3.0, y, w / 2.0 - 3.0, "Rebuild the world", false, click);
    y += 24.0;
    let status = b.status.clone();
    if !status.is_empty() {
        c.text(&status, x, y, 12.0, DIM);
    }
    if save {
        save_places(game);
    }
    if rebuild {
        let said = save_places(game);
        if said.starts_with("Couldn't") {
            return;
        }
        super::app::rebuild_world(game);
        game.editor.build.status = "The world was built again from the places.".into();
    }
}

/// For screenshots (`GAHT_EDITBUILD`): the Build tool on, at the mark nearest
/// the squad, with a cottage, a smith's forge and a market put down there and
/// a weaver's house in hand. Returns the place's centre.
pub fn demo(game: &mut Game) -> V2 {
    let at = game.world.squad.pos;
    let c = grounds::marks().into_iter().min_by(|a, b| a.4.dist(at).total_cmp(&b.4.dist(at))).map(|m| m.4).unwrap_or(at);
    let b = &mut game.editor.build;
    b.on = true;
    b.scale = 1.0;
    if let Some(key) = open_at(b, c) {
        let o = b.places.get_mut(&key).expect("opened");
        o.file.things.clear();
        for (k, x, y, rot, sign) in [("roduro_cottage", -16.0, 10.0, 30.0, ""), ("roduro_longhouse", 14.0, -12.0, 200.0, "smith"), ("market", 2.0, 24.0, 0.0, "")] {
            o.file.things.push(Thing { id: format!("demo_{k}"), k: k.into(), x, y, rot, s: 1.0, label: String::new(), room: None, note: String::new(), use_: sign.into() });
        }
        o.unsaved = true;
        b.picked = Some((key, 1));
    }
    // (`GAHT_EDITBUILD=sign`: the forge picked, its sign showing, nothing in hand.)
    if std::env::var("GAHT_EDITBUILD").as_deref() != Ok("sign") {
        b.armed = Some("roduro_loomroom".into());
        b.rot = 120.0;
    }
    c
}
