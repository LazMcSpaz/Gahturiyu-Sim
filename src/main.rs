//! The playtest window. It only reads the simulation and passes on orders;
//! nothing about how the world behaves lives here.
//!
//! Two views of the same world: a 3D view around the squad (the default) and
//! a top-down map. V switches between them. Controls are listed along the
//! bottom of the window.

mod view;

use macroquad::prelude::*;

use gahturiyu_sim::sim::{geo::V2, worldgen};
use view::{
    map::{self, MapCam, Relief},
    scene::{self, OrbitCam},
    squadui::{self, Action, Click, Selection},
    ui::{self, describe, Picker, Shot, Ui, DIM, SPEEDS},
};

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title: "Gahturiyu".to_owned(),
            window_width: 1600,
            window_height: 1000,
            high_dpi: true,
            sample_count: 4,
            ..Default::default()
        },
        // The 3D view hands over big pre-built meshes; give them room.
        draw_call_vertex_capacity: 65_536,
        draw_call_index_capacity: 196_608,
        ..Default::default()
    }
}

#[derive(PartialEq, Clone, Copy)]
enum View {
    Scene,
    Map,
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let ui = Ui { font: load_ttf_font_from_bytes(include_bytes!("../assets/DejaVuSans.ttf")).expect("font") };
    let mut world = worldgen::generate(seed);
    let relief = Relief::new(&world);

    let mut view = View::Scene;
    let mut map_cam = MapCam { centre: world.squad.pos, zoom: screen_width() / 7000.0 };
    let mut orbit = OrbitCam::new(world.squad.pos);
    orbit.ground = world.terrain.surface(world.squad.pos);
    let mut follow = true;
    let mut speed_i = 2usize;
    let mut paused = false;
    let mut rings = true;
    let mut last_mouse: Vec2 = mouse_position().into();
    let mut press_at: Option<Vec2> = None;
    let mut sim_ms = 0.0f64;
    let mut draw_ms = 0.0f64;
    let mut scene_cache = scene::SceneCache::default();

    let shot = Shot::from_env();
    if let Some(s) = &shot {
        if s.view.as_deref() == Some("map") {
            view = View::Map;
        }
        if let Some(z) = s.zoom {
            if view == View::Map {
                map_cam.zoom = z;
            } else {
                orbit.dist = z;
            }
        }
        if let Some(p) = s.pitch {
            orbit.pitch = p;
        }
        if let Some(y) = s.yaw {
            orbit.yaw = y;
        }
        if let Some(sp) = s.speed {
            speed_i = sp;
        }
        if let Some((dx, dy)) = s.nudge {
            world.teleport_squad(world.squad.pos.add(V2::new(dx, dy)));
            world.step(0.001);
        }
        if let Some(h) = s.hours {
            let end = world.time + h * 3600.0;
            while world.time < end {
                world.step(60.0);
            }
        }
        if s.sneak {
            for m in world.squad.members.clone() {
                world.set_sneaking(m, true);
            }
        }
        if let Some(k) = s.camp {
            if let Some(c) = world.camps.get(k) {
                let at = c.pos.add(V2::new(70.0, 0.0));
                world.teleport_squad(at);
            }
        }
        if let Some(h) = s.wait {
            // Run until a fight is on nearby, or the time is up.
            let end = world.time + h * 3600.0;
            let near = |w: &gahturiyu_sim::sim::World| w.battles.iter().any(|b| b.fighters.iter().any(|f| f.pos.dist(w.squad.pos) < 300.0));
            while world.time < end && !near(&world) {
                world.step(1.0);
            }
        }
        if s.enter {
            let m = world.squad.members[0];
            let here = world.squad.pos;
            let d = world.doors_near(here, 200.0).into_iter().filter(|d| !world.is_locked(d.id)).min_by(|a, b| a.centre.dist(here).total_cmp(&b.centre.dist(here)));
            if let Some(d) = d {
                world.order_members(&[m], d.centre);
                for _ in 0..600 {
                    world.step(0.5);
                }
            }
        }
        if let Some(n) = s.bandits {
            let at = world.squad.pos.add(V2::new(14.0, 6.0));
            world.spawn_bandits(at, n, true);
        }
        if let Some(k) = s.drop {
            if let Some(&m) = world.squad.members.get(k) {
                use gahturiyu_sim::sim::items::Slot;
                for slot in [Slot::MainHand, Slot::Body, Slot::Back] {
                    let it = world.people[m as usize].detail.as_ref().and_then(|d| d.gear.in_slot(slot));
                    if let Some(it) = it {
                        world.unequip(m, slot);
                        world.drop_item(m, it);
                    }
                }
                // ...and walks off a little, so they're visible.
                let to = world.person_pos(m).add(V2::new(-12.0, 8.0));
                world.order_members(&[m], to);
            }
        }
    }
    let mut frame = 0u32;
    let mut last_hover: Option<ui::Hover> = None;
    let mut sel = Selection::default();
    let mut inv: Option<u32> = None;
    if let Some(s) = &shot {
        if let Some(k) = s.select {
            if let Some(&m) = world.squad.members.get(k) {
                sel.pick(m, false);
            }
        }
        if let Some(k) = s.inventory {
            inv = world.squad.members.get(k).copied();
        }
    }

    loop {
        frame += 1;
        // ---------------------------------------------------------------- input
        let mut mouse: Vec2 = mouse_position().into();
        if let Some(h) = shot.as_ref().and_then(|s| s.hover) {
            mouse = h;
        }
        for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5].iter().enumerate() {
            if is_key_pressed(*key) {
                speed_i = i;
                paused = false;
            }
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::F) {
            follow = true;
        }
        if is_key_pressed(KeyCode::R) {
            rings = !rings;
        }
        // Squad selection: F1–F4 pick one (Shift adds), ` or Esc picks everyone.
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        for (k, key) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4, KeyCode::F5, KeyCode::F6].iter().enumerate() {
            if is_key_pressed(*key) {
                if let Some(&m) = world.squad.members.get(k) {
                    sel.pick(m, shift);
                    if inv.is_some() {
                        inv = Some(m);
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::GraveAccent) || is_key_pressed(KeyCode::Escape) {
            sel = Selection::default();
            inv = None;
        }
        // Z: the selected sneak (or stop sneaking).
        if is_key_pressed(KeyCode::Z) {
            let who = sel.who(&world);
            let on = !who.iter().all(|&m| world.is_sneaking(m));
            for m in who {
                world.set_sneaking(m, on);
            }
        }
        if is_key_pressed(KeyCode::I) {
            inv = match inv {
                Some(_) => None,
                None => sel.lead(&world),
            };
        }
        // Testing aid: B drops a band of bandits (with a mage) near the squad.
        if is_key_pressed(KeyCode::B) {
            let n = 2 + (world.time as usize % 3);
            let at = world.squad.pos.add(V2::new(26.0, 12.0));
            world.spawn_bandits(at, n, true);
        }
        if is_key_pressed(KeyCode::V) || is_key_pressed(KeyCode::Tab) {
            view = if view == View::Scene { View::Map } else { View::Scene };
            // Keep looking at the same place.
            if view == View::Map {
                map_cam.centre = orbit.target;
            } else {
                orbit.target = map_cam.centre;
            }
        }

        let d = mouse - last_mouse;
        let (_, wheel) = mouse_wheel();
        let dt = get_frame_time();
        let mut pan = (0.0f32, 0.0f32);
        for (k, dx, dy) in [(KeyCode::W, 0.0, 1.0), (KeyCode::S, 0.0, -1.0), (KeyCode::A, -1.0, 0.0), (KeyCode::D, 1.0, 0.0)] {
            if is_key_down(k) {
                pan.0 += dx;
                pan.1 += dy;
            }
        }
        match view {
            View::Map => {
                if wheel != 0.0 {
                    map_cam.zoom_at(mouse, wheel);
                }
                if (is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle)) && d.length() > 0.0 {
                    map_cam.centre = map_cam.centre.sub(V2::new(d.x / map_cam.zoom, d.y / map_cam.zoom));
                    follow = false;
                }
                if pan != (0.0, 0.0) {
                    let s = 900.0 / map_cam.zoom * dt;
                    map_cam.centre = map_cam.centre.add(V2::new(pan.0 * s, -pan.1 * s));
                    follow = false;
                }
            }
            View::Scene => {
                if wheel != 0.0 {
                    orbit.zoom(wheel);
                }
                if is_mouse_button_down(MouseButton::Right) && d.length() > 0.0 {
                    orbit.orbit(d.x, d.y);
                }
                if is_mouse_button_down(MouseButton::Middle) && d.length() > 0.0 {
                    orbit.pan(-d.x / screen_height() * 1.6, d.y / screen_height() * 1.6);
                    follow = false;
                }
                if pan != (0.0, 0.0) {
                    orbit.pan(pan.0 * dt, pan.1 * dt);
                    follow = false;
                }
                if is_key_down(KeyCode::Q) {
                    orbit.orbit(-260.0 * dt, 0.0);
                }
                if is_key_down(KeyCode::E) {
                    orbit.orbit(260.0 * dt, 0.0);
                }
            }
        }
        // Clicks on the squad panels are handled when they're drawn; a short
        // click anywhere else is an order.
        let mut ui_click: Option<Click> = None;
        let on_panels = squadui::over(&world, mouse, inv);
        if is_mouse_button_pressed(MouseButton::Left) {
            press_at = Some(mouse);
        }
        if is_mouse_button_pressed(MouseButton::Right) && on_panels {
            ui_click = Some(Click { at: mouse, right: true });
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(p) = press_at.take() {
                if (p - mouse).length() < 6.0 {
                    if on_panels {
                        ui_click = Some(Click { at: mouse, right: false });
                    } else {
                        click_world(&mut world, &mut sel, last_hover, view, &map_cam, &orbit, mouse, shift);
                    }
                }
            }
        }        last_mouse = mouse;

        // A fight breaking out near the squad drops the game to real time.
        if !world.alerts.is_empty() {
            world.alerts.clear();
            speed_i = 0;
            paused = false;
        }

        // ------------------------------------------------------------- simulate
        if !paused {
            let t0 = std::time::Instant::now();
            let mut left = dt.min(0.1) as f64 * SPEEDS[speed_i].0;
            // Small steps keep the squad and nearby people smooth; the world
            // itself would not care if this were one big step.
            let sub = (left / 600.0).max(1.0);
            while left > 1e-9 {
                let step = left.min(sub);
                world.step(step);
                left -= step;
            }
            sim_ms = sim_ms * 0.9 + t0.elapsed().as_secs_f64() * 1000.0 * 0.1;
        }
        if follow {
            map_cam.centre = world.squad.pos;
            orbit.target = world.squad.pos;
        }
        // Keep the camera's pivot on the ground, easing so it doesn't jolt.
        let g = world.terrain.surface(orbit.target);
        orbit.ground += (g - orbit.ground) * (1.0 - (-8.0 * dt).exp());
        if (g - orbit.ground).abs() > 60.0 {
            orbit.ground = g;
        }

        // ----------------------------------------------------------------- draw
        let mut pick = Picker::new(mouse);
        let t_draw = std::time::Instant::now();
        let name = match view {
            View::Map => {
                map::draw(&ui, &map_cam, &world, rings, &mut pick, &relief, &sel);
                "map"
            }
            View::Scene => {
                scene::draw(&ui, &orbit, &world, rings, &mut pick, &mut scene_cache, &sel);
                "3D"
            }
        };
        draw_ms = draw_ms * 0.9 + t_draw.elapsed().as_secs_f64() * 1000.0 * 0.1;
        ui::draw_hud(&ui, &world, speed_i, paused, sim_ms, draw_ms, name);
        // Squad cards and the inventory panel.
        if inv.map(|p| world.squad.index(p).is_none()).unwrap_or(false) {
            inv = None;
        }
        let mut actions = Vec::new();
        actions.extend(squadui::squad_bar(&ui, &world, &sel, ui_click));
        let mut item_tip = None;
        if let Some(pid) = inv {
            let (a, h) = squadui::inventory(&ui, &world, pid, mouse, ui_click);
            actions.extend(a);
            item_tip = h;
        }
        for a in actions {
            match a {
                Action::Select(pid, add) => sel.pick(pid, add),
                Action::OpenInventory(pid) => inv = if inv == Some(pid) { None } else { Some(pid) },
                Action::CloseInventory => inv = None,
                Action::Equip(pid, it) => {
                    world.equip(pid, it);
                }
                Action::Unequip(pid, slot) => {
                    world.unequip(pid, slot);
                }
                Action::Drop(pid, it) => {
                    world.drop_item(pid, it);
                }
            }
        }

        last_hover = if on_panels { None } else { pick.best.map(|(_, h)| h) };
        if let Some(it) = item_tip {
            let lines = squadui::item_lines(it);
            let w = lines.iter().map(|(l, _)| ui.width(l, 15)).fold(0.0, f32::max) + 24.0;
            ui.panel(&lines, mouse.x - w - 18.0, mouse.y, 15);
        } else if let Some(h) = last_hover {
            ui.panel(&describe(&world, h), mouse.x + 18.0, mouse.y + 12.0, 16);
        }
        let help = match view {
            View::Scene => "Click: move / attack / pick up / select   F1–F4: select (Shift adds)   `: all   Z: sneak   I: pack   Right-drag / Q E: turn   Middle / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   V: map   B: bandits",
            View::Map => "Click: move / attack / pick up / select   F1–F4: select (Shift adds)   `: all   Z: sneak   I: pack   Right-drag / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   V: 3D   B: bandits",
        };
        draw_rectangle(0.0, screen_height() - 30.0, screen_width(), 30.0, Color::new(0.0, 0.0, 0.0, 0.45));
        ui.text(help, 14.0, screen_height() - 10.0, 15, DIM);

        if let Some(s) = &shot {
            if frame == s.frames {
                get_screen_data().export_png(&s.path);
                std::process::exit(0);
            }
        }
        next_frame().await;
    }
}

/// A short left-click in the world: select a squad member, attack an enemy,
/// pick something up, or walk there.
#[allow(clippy::too_many_arguments)]
fn click_world(world: &mut gahturiyu_sim::sim::World, sel: &mut Selection, hover: Option<ui::Hover>, view: View, map_cam: &MapCam, orbit: &OrbitCam, mouse: Vec2, shift: bool) {
    let who = sel.who(world);
    match hover {
        Some(ui::Hover::Person(pid)) if world.squad.index(pid).is_some() => {
            sel.pick(pid, shift);
            return;
        }
        Some(ui::Hover::Person(pid)) => {
            // An enemy: go for them (starting a fight if there isn't one).
            if world.attack(&who, pid) {
                return;
            }
        }
        Some(ui::Hover::Door(id)) => {
            if world.is_locked(id) {
                // The best lock-picker among the selected who has picks.
                let pick = gahturiyu_sim::sim::items::id("lockpick");
                let picker = who
                    .iter()
                    .copied()
                    .filter(|&m| world.people[m as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == pick)).unwrap_or(false))
                    .max_by(|&a, &b| world.pick_chance(a, 50.0).total_cmp(&world.pick_chance(b, 50.0)));
                match picker {
                    Some(p) => {
                        world.order_pick(p, id);
                    }
                    None => world.log.push_front((world.time, "Nobody selected has a lockpick.".into())),
                }
                return;
            }
            if let Some(d) = world.door(id) {
                world.order_members(&who, d.centre);
                return;
            }
        }
        Some(ui::Hover::Item(thing)) => {
            // Whoever's selected and closest goes for it.
            let pos = world.ground.iter().find(|g| g.id == thing).map(|g| g.pos);
            if let Some(pos) = pos {
                let fetcher = who.iter().copied().min_by(|&a, &b| world.person_pos(a).dist(pos).total_cmp(&world.person_pos(b).dist(pos)));
                if let Some(f) = fetcher {
                    world.order_pickup(f, thing);
                    return;
                }
            }
        }
        _ => {}
    }
    let target = match view {
        View::Map => Some(map_cam.to_world(mouse)),
        View::Scene => scene::ground_at(&orbit.camera(), mouse, &world.terrain),
    };
    if let Some(t) = target {
        if sel.is_all(world) {
            world.order_squad(t);
        } else {
            world.order_members(&who, t);
        }
    }
}
