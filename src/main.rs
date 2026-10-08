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
    map::{self, MapCam},
    scene::{self, OrbitCam},
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

    let mut view = View::Scene;
    let mut map_cam = MapCam { centre: world.squad.pos, zoom: screen_width() / 7000.0 };
    let mut orbit = OrbitCam::new(world.squad.pos);
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
            world.squad.pos = world.squad.pos.add(V2::new(dx, dy));
            world.squad.target = world.squad.pos;
            world.step(0.001);
        }
    }
    let mut frame = 0u32;

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
        if is_mouse_button_pressed(MouseButton::Left) {
            press_at = Some(mouse);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(p) = press_at.take() {
                if (p - mouse).length() < 6.0 {
                    let target = match view {
                        View::Map => Some(map_cam.to_world(mouse)),
                        View::Scene => scene::ground_at(&orbit.camera(), mouse),
                    };
                    if let Some(t) = target {
                        world.order_squad(t);
                    }
                }
            }
        }
        last_mouse = mouse;

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

        // ----------------------------------------------------------------- draw
        let mut pick = Picker::new(mouse);
        let t_draw = std::time::Instant::now();
        let name = match view {
            View::Map => {
                map::draw(&ui, &map_cam, &world, rings, &mut pick);
                "map"
            }
            View::Scene => {
                scene::draw(&ui, &orbit, &world, rings, &mut pick, &mut scene_cache);
                "3D"
            }
        };
        draw_ms = draw_ms * 0.9 + t_draw.elapsed().as_secs_f64() * 1000.0 * 0.1;
        ui::draw_hud(&ui, &world, speed_i, paused, sim_ms, draw_ms, name);
        if let Some((_, h)) = pick.best {
            ui.panel(&describe(&world, h), mouse.x + 18.0, mouse.y + 12.0, 16);
        }
        let help = match view {
            View::Scene => "Left-click: move squad   Right-drag / Q E: turn   Middle-drag / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   R: rings   V: map",
            View::Map => "Left-click: move squad   Right-drag / WASD: pan   Wheel: zoom   C: follow   Space: pause   1–5: speed   R: rings   V: 3D",
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
