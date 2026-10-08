//! The playtest window. It only reads the simulation and passes on orders;
//! nothing about how the world behaves lives here.
//!
//! Controls are listed along the bottom of the window.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    bands::{BAND1_RADIUS, BAND2_RADIUS},
    geo::{self, V2, WORLD_SIZE},
    group::Kind,
    person::PersonId,
    race::{trait_word, Race, ALL_RACES},
    world::HOUR,
    worldgen, World,
};

const SPEEDS: [(f64, &str); 5] = [(1.0, "1×"), (10.0, "10×"), (60.0, "1 min/s"), (600.0, "10 min/s"), (3600.0, "1 hour/s")];

const LAND: Color = Color::new(0.16, 0.21, 0.17, 1.0);
const SEA: Color = Color::new(0.10, 0.15, 0.19, 1.0);
const SHORE: Color = Color::new(0.30, 0.36, 0.33, 1.0);
const PANEL: Color = Color::new(0.05, 0.07, 0.07, 0.86);
const TEXT: Color = Color::new(0.90, 0.91, 0.88, 1.0);
const DIM: Color = Color::new(0.62, 0.66, 0.63, 1.0);

fn race_color(r: Race) -> Color {
    match r {
        Race::Roduro => Color::new(0.80, 0.72, 0.60, 1.0), // weathered stone
        Race::Qotiro => Color::new(0.95, 0.47, 0.22, 1.0), // ember
        Race::Horaro => Color::new(0.38, 0.68, 0.96, 1.0), // sea
        Race::Tadoro => Color::new(0.78, 0.68, 0.98, 1.0), // pale violet
    }
}

fn window_conf() -> Conf {
    Conf { window_title: "Gahturiyu".to_owned(), window_width: 1600, window_height: 1000, high_dpi: true, ..Default::default() }
}

struct Cam {
    centre: V2,
    /// Pixels per metre.
    zoom: f32,
    follow: bool,
}

impl Cam {
    fn to_screen(&self, p: V2) -> Vec2 {
        vec2((p.x - self.centre.x) * self.zoom + screen_width() / 2.0, (p.y - self.centre.y) * self.zoom + screen_height() / 2.0)
    }
    fn to_world(&self, s: Vec2) -> V2 {
        V2::new((s.x - screen_width() / 2.0) / self.zoom + self.centre.x, (s.y - screen_height() / 2.0) / self.zoom + self.centre.y)
    }
}

enum Hover {
    Person(PersonId),
    Group(u32),
    Town(u16),
}

struct Ui {
    font: Font,
}

impl Ui {
    fn text(&self, s: &str, x: f32, y: f32, size: u16, c: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, color: c, ..Default::default() });
    }
    fn width(&self, s: &str, size: u16) -> f32 {
        measure_text(s, Some(&self.font), size, 1.0).width
    }
    fn panel(&self, lines: &[(String, Color)], x: f32, y: f32, size: u16) {
        let lh = size as f32 * 1.35;
        let w = lines.iter().map(|(l, _)| self.width(l, size)).fold(0.0, f32::max) + 24.0;
        let h = lines.len() as f32 * lh + 16.0;
        // Keep tooltips on screen.
        let x = x.min(screen_width() - w - 8.0).max(8.0);
        let y = y.min(screen_height() - h - 8.0).max(8.0);
        draw_rectangle(x, y, w, h, PANEL);
        for (i, (l, c)) in lines.iter().enumerate() {
            self.text(l, x + 12.0, y + 8.0 + lh * (i as f32 + 0.78), size, *c);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let ui = Ui { font: load_ttf_font_from_bytes(include_bytes!("../assets/DejaVuSans.ttf")).expect("font") };
    let mut world = worldgen::generate(seed);
    let mut cam = Cam { centre: world.squad.pos, zoom: screen_width() / 7000.0, follow: true };
    let mut speed_i = 2usize;
    let mut paused = false;
    let mut rings = true;
    let mut last_mouse: Vec2 = mouse_position().into();
    let mut press_at: Option<Vec2> = None;
    let mut sim_ms = 0.0f64;
    let shot = Shot::from_env();
    if let Some(z) = shot.as_ref().and_then(|s| s.zoom) {
        cam.zoom = z;
    }
    if let Some(s) = shot.as_ref().and_then(|s| s.speed) {
        speed_i = s;
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
            cam.follow = true;
        }
        if is_key_pressed(KeyCode::R) {
            rings = !rings;
        }
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 {
            let before = cam.to_world(mouse);
            cam.zoom = (cam.zoom * if wheel > 0.0 { 1.15 } else { 1.0 / 1.15 }).clamp(screen_width() / WORLD_SIZE / 1.2, 12.0);
            let after = cam.to_world(mouse);
            cam.centre = cam.centre.add(before.sub(after));
        }
        if is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle) {
            let d = mouse - last_mouse;
            if d.length() > 0.0 {
                cam.centre = cam.centre.sub(V2::new(d.x / cam.zoom, d.y / cam.zoom));
                cam.follow = false;
            }
        }
        let pan = 900.0 / cam.zoom * get_frame_time();
        for (k, dx, dy) in [(KeyCode::W, 0.0, -1.0), (KeyCode::S, 0.0, 1.0), (KeyCode::A, -1.0, 0.0), (KeyCode::D, 1.0, 0.0)] {
            if is_key_down(k) {
                cam.centre = cam.centre.add(V2::new(dx * pan, dy * pan));
                cam.follow = false;
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            press_at = Some(mouse);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(p) = press_at.take() {
                if (p - mouse).length() < 6.0 {
                    world.order_squad(cam.to_world(mouse));
                }
            }
        }
        last_mouse = mouse;

        // ------------------------------------------------------------- simulate
        if !paused {
            let t0 = std::time::Instant::now();
            let mut left = get_frame_time().min(0.1) as f64 * SPEEDS[speed_i].0;
            // Small steps keep the squad and nearby people smooth; the world
            // itself would not care if this were one big step.
            let sub = (left / 600.0).max(1.0);
            while left > 1e-9 {
                let dt = left.min(sub);
                world.step(dt);
                left -= dt;
            }
            sim_ms = sim_ms * 0.9 + t0.elapsed().as_secs_f64() * 1000.0 * 0.1;
        }
        if cam.follow {
            cam.centre = world.squad.pos;
        }

        // ----------------------------------------------------------------- draw
        clear_background(LAND);
        draw_sea(&cam);
        if rings {
            draw_bands(&cam, &world);
        }

        let mut hover: Option<(f32, Hover)> = None;
        let mut consider = |d: f32, h: Hover| {
            if d < 10.0 && hover.as_ref().map(|(b, _)| d < *b).unwrap_or(true) {
                hover = Some((d, h));
            }
        };

        // Towns.
        for s in &world.settlements {
            let p = cam.to_screen(s.pos);
            let r = (s.radius() * cam.zoom).max(4.0);
            draw_circle(p.x, p.y, r, with_alpha(race_color(s.founders), 0.22));
            draw_circle_lines(p.x, p.y, r, 1.5, with_alpha(race_color(s.founders), 0.8));
            if let Some(st) = s.stilts {
                let q = cam.to_screen(st);
                let rr = (60.0 * cam.zoom).max(2.5);
                draw_circle(q.x, q.y, rr, with_alpha(race_color(Race::Horaro), 0.25));
            }
            consider((p - mouse).length() - r.min(30.0) + 6.0, Hover::Town(s.id));
            if cam.zoom > 0.06 || world.bands.band_at(s.pos) <= 2 {
                let size = 15;
                ui.text(&s.name, p.x - ui.width(&s.name, size) / 2.0, p.y - r - 6.0, size, TEXT);
            }
        }

        // Residents at home, drawn one by one only where the town is in band 1.
        for s in &world.settlements {
            let near = world.bands.band_at(s.pos) == 1 || s.stilts.map(|p| world.bands.band_at(p) == 1).unwrap_or(false);
            if !near {
                continue;
            }
            for &pid in &s.residents {
                if world.busy_until[pid as usize] > world.time || world.group_of[pid as usize].is_some() {
                    continue;
                }
                let p = cam.to_screen(world.person_pos(pid));
                draw_circle(p.x, p.y, person_dot(cam.zoom), race_color(world.people[pid as usize].race));
                consider((p - mouse).length(), Hover::Person(pid));
            }
        }

        // Groups: dots far away, individuals up close.
        for g in &world.groups {
            let lead = world.people[g.members[0] as usize].race;
            match g.band {
                1 => {
                    for &m in &g.members {
                        let p = cam.to_screen(world.person_pos(m));
                        draw_circle(p.x, p.y, person_dot(cam.zoom), race_color(world.people[m as usize].race));
                        draw_circle_lines(p.x, p.y, person_dot(cam.zoom) + 1.0, 1.0, with_alpha(WHITE, 0.5));
                        consider((p - mouse).length(), Hover::Person(m));
                    }
                }
                b => {
                    let p = cam.to_screen(g.pos);
                    let (r, a) = if b == 2 { (3.6, 0.95) } else { (2.4, 0.55) };
                    draw_circle(p.x, p.y, r + (g.members.len() as f32 - 1.0) * 0.35, with_alpha(race_color(lead), a));
                    consider((p - mouse).length(), Hover::Group(g.id));
                }
            }
        }

        // The squad.
        let sq = cam.to_screen(world.squad.pos);
        let tg = cam.to_screen(world.squad.target);
        if world.squad.pos.dist(world.squad.target) > 1.0 {
            draw_line(sq.x, sq.y, tg.x, tg.y, 1.0, with_alpha(WHITE, 0.35));
            draw_line(tg.x - 5.0, tg.y - 5.0, tg.x + 5.0, tg.y + 5.0, 2.0, WHITE);
            draw_line(tg.x - 5.0, tg.y + 5.0, tg.x + 5.0, tg.y - 5.0, 2.0, WHITE);
        }
        draw_circle_lines(sq.x, sq.y, (14.0 * cam.zoom).max(9.0), 2.0, WHITE);
        for &m in &world.squad.members {
            let p = cam.to_screen(world.person_pos(m));
            draw_circle(p.x, p.y, person_dot(cam.zoom) + 0.5, race_color(world.people[m as usize].race));
            consider((p - mouse).length(), Hover::Person(m));
        }

        // ------------------------------------------------------------------ HUD
        draw_hud(&ui, &world, speed_i, paused, sim_ms);
        if let Some((_, h)) = hover {
            let lines = describe(&world, h);
            ui.panel(&lines, mouse.x + 18.0, mouse.y + 12.0, 16);
        }
        let help = "Left-click: move squad   Right-drag / WASD: pan   Wheel: zoom   C: follow squad   Space: pause   1–5: speed   R: band rings";
        ui.text(help, 14.0, screen_height() - 14.0, 15, DIM);

        if let Some(s) = &shot {
            if frame == s.frames {
                get_screen_data().export_png(&s.path);
                std::process::exit(0);
            }
        }
        next_frame().await;
    }
}

/// Development aid: `GAHT_SHOT=out.png GAHT_FRAMES=120 cargo run` saves a
/// screenshot after that many frames and quits. `GAHT_HOVER=x,y` fakes the
/// mouse, `GAHT_ZOOM` sets pixels per metre, `GAHT_SPEED` picks speed 0–4.
struct Shot {
    path: String,
    frames: u32,
    hover: Option<Vec2>,
    zoom: Option<f32>,
    speed: Option<usize>,
}

impl Shot {
    fn from_env() -> Option<Shot> {
        let path = std::env::var("GAHT_SHOT").ok()?;
        let var = |k: &str| std::env::var(k).ok();
        Some(Shot {
            path,
            frames: var("GAHT_FRAMES").and_then(|v| v.parse().ok()).unwrap_or(120),
            hover: var("GAHT_HOVER").and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some(vec2(x.parse().ok()?, y.parse().ok()?))
            }),
            zoom: var("GAHT_ZOOM").and_then(|v| v.parse().ok()),
            speed: var("GAHT_SPEED").and_then(|v| v.parse().ok()),
        })
    }
}

fn person_dot(zoom: f32) -> f32 {
    (zoom * 1.2).clamp(2.0, 5.0)
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

fn draw_sea(cam: &Cam) {
    let top = cam.to_world(vec2(0.0, 0.0)).y.max(0.0);
    let bottom = cam.to_world(vec2(0.0, screen_height())).y.min(WORLD_SIZE);
    let step = (3.0 / cam.zoom).max(10.0);
    let mut y = top - step;
    while y < bottom + step {
        let a = cam.to_screen(V2::new(geo::coast_x(y), y));
        let b = cam.to_screen(V2::new(geo::coast_x(y + step), y + step));
        draw_rectangle(-10.0, a.y, a.x + 10.0, b.y - a.y + 1.0, SEA);
        draw_line(a.x, a.y, b.x, b.y, 2.0, SHORE);
        y += step;
    }
    // Beyond the map edge.
    let tl = cam.to_screen(V2::new(0.0, 0.0));
    let br = cam.to_screen(V2::new(WORLD_SIZE, WORLD_SIZE));
    let edge = Color::new(0.03, 0.04, 0.04, 1.0);
    draw_rectangle(-10.0, -10.0, screen_width() + 20.0, tl.y + 10.0, edge);
    draw_rectangle(-10.0, br.y, screen_width() + 20.0, screen_height(), edge);
    draw_rectangle(br.x, -10.0, screen_width(), screen_height() + 20.0, edge);
}

fn draw_bands(cam: &Cam, w: &World) {
    let c = cam.to_screen(w.squad.pos);
    draw_circle(c.x, c.y, BAND1_RADIUS * cam.zoom, Color::new(1.0, 1.0, 1.0, 0.045));
    draw_circle_lines(c.x, c.y, BAND1_RADIUS * cam.zoom, 1.5, Color::new(1.0, 1.0, 1.0, 0.45));
    draw_circle_lines(c.x, c.y, BAND2_RADIUS * cam.zoom, 1.0, Color::new(1.0, 1.0, 1.0, 0.25));
}

fn draw_hud(ui: &Ui, w: &World, speed_i: usize, paused: bool, sim_ms: f64) {
    let mut lines: Vec<(String, Color)> = vec![
        (w.clock(), TEXT),
        (if paused { "Paused".into() } else { format!("Speed {}", SPEEDS[speed_i].1) }, DIM),
        (String::new(), TEXT),
        (format!("{} people  ·  {} on the road", w.people.len() - w.squad.members.len(), w.on_road()), TEXT),
        (format!("Named so far: {}", w.stats.detailed), TEXT),
        (format!("Groups in band 1 / 2 / 3:  {} / {} / {}", w.stats.in_band[1], w.stats.in_band[2], w.stats.in_band[3]), TEXT),
        (format!("Simulation: {:.2} ms per frame   ({:.0} fps)", sim_ms, get_fps()), DIM),
        (String::new(), TEXT),
    ];
    for r in ALL_RACES {
        lines.push((format!("●  {}  ({})", r.name(), r.element()), race_color(r)));
    }
    lines.push((String::new(), TEXT));
    for (t, l) in w.log.iter().take(10) {
        let secs = t.rem_euclid(24.0 * HOUR);
        lines.push((format!("{:02}:{:02}  {}", (secs / HOUR) as i64, ((secs % HOUR) / 60.0) as i64, l), DIM));
    }
    ui.panel(&lines, 12.0, 12.0, 16);
}

fn describe(w: &World, h: Hover) -> Vec<(String, Color)> {
    let mut out: Vec<(String, Color)> = Vec::new();
    match h {
        Hover::Person(pid) => {
            let p = &w.people[pid as usize];
            let name = p.name().unwrap_or("(not yet named)");
            out.push((format!("{}  ·  {}", name, p.race.name()), race_color(p.race)));
            let t = p.traits;
            out.push((
                format!(
                    "{}, {}, {}, {}",
                    trait_word(t.wanderlust, "a homebody", "settled", "restless"),
                    trait_word(t.boldness, "cautious", "steady", "bold"),
                    trait_word(t.sociability, "solitary", "easygoing", "gregarious"),
                    trait_word(t.patience, "hot-tempered", "even-tempered", "patient"),
                ),
                TEXT,
            ));
            if let Some(d) = &p.detail {
                out.push((format!("Carries {}, wears {}", d.weapon.describe(p.race), d.armor.describe()), TEXT));
            }
            out.push((format!("Might {:.0}", p.might), DIM));
            if p.in_squad {
                out.push(("Your squad".into(), TEXT));
            } else if let Some(g) = w.group_of[pid as usize].and_then(|g| w.group(g)) {
                out.push((doing(w, g.id), DIM));
            } else if let Some(home) = p.home {
                out.push((format!("At home in {}", w.settlements[home as usize].name), DIM));
            }
        }
        Hover::Group(gid) => {
            let Some(g) = w.group(gid) else { return out };
            let mut by = [0usize; 4];
            for &m in &g.members {
                by[w.people[m as usize].race.index()] += 1;
            }
            let mix: Vec<String> = ALL_RACES.iter().zip(by).filter(|(_, n)| *n > 0).map(|(r, n)| format!("{} {}", r.name(), n)).collect();
            let who = if g.members.len() == 1 { "1 traveller".to_string() } else { format!("{} travellers", g.members.len()) };
            out.push((format!("{who}  ({})", mix.join(", ")), race_color(w.people[g.members[0] as usize].race)));
            out.push((doing(w, gid), TEXT));
            let might: f32 = g.members.iter().map(|&m| w.people[m as usize].might).sum();
            out.push((format!("Combined might {:.0}", might), DIM));
            let named = g.members.iter().filter(|&&m| w.people[m as usize].detail.is_some()).count();
            out.push((
                format!(
                    "Band {}  ·  {}",
                    g.band,
                    if named == 0 {
                        "nobody here has been named yet".to_string()
                    } else {
                        format!("{named} of {} named", g.members.len())
                    }
                ),
                DIM,
            ));
        }
        Hover::Town(sid) => {
            let s = &w.settlements[sid as usize];
            out.push((s.name.clone(), race_color(s.founders)));
            out.push((
                format!("{} town, founded by the {}{}", if s.coastal { "Coastal" } else { "Inland" }, s.founders.name(), if s.coastal { ", Horaro stilts offshore" } else { "" }),
                TEXT,
            ));
            let mut by = [0usize; 4];
            let mut home = 0;
            for &p in &s.residents {
                by[w.people[p as usize].race.index()] += 1;
                if w.busy_until[p as usize] <= w.time {
                    home += 1;
                }
            }
            out.push((format!("{} people, {} home right now", s.residents.len(), home), TEXT));
            for (r, n) in ALL_RACES.iter().zip(by) {
                out.push((format!("   {} {}", r.name(), n), race_color(*r)));
            }
        }
    }
    out
}

fn doing(w: &World, gid: u32) -> String {
    let Some(g) = w.group(gid) else { return String::new() };
    let t = w.time;
    let Some(leg) = g.current_leg(t) else { return String::new() };
    let at = |s: u16| w.settlements[s as usize].name.clone();
    let clock = |secs: f64| {
        let s = secs.rem_euclid(24.0 * HOUR);
        format!("{:02}:{:02}", (s / HOUR) as i64, ((s % HOUR) / 60.0) as i64)
    };
    match g.kind {
        Kind::Wanderer { .. } => {
            if t < leg.arrive && t >= leg.depart {
                "Wandering the wilds".into()
            } else {
                "Resting in the wilds".into()
            }
        }
        Kind::Journey { home } => {
            if t < leg.depart {
                format!("Setting out from {} at {}", at(home), clock(leg.depart))
            } else if t < leg.arrive {
                match leg.dest {
                    Some(d) if d == home => format!("Walking home to {}, due {}", at(d), clock(leg.arrive)),
                    Some(d) => format!("Walking to {}, due {}", at(d), clock(leg.arrive)),
                    None => "On the road".into(),
                }
            } else {
                match leg.dest {
                    Some(d) => format!("Visiting {}", at(d)),
                    None => "Resting".into(),
                }
            }
        }
    }
}
