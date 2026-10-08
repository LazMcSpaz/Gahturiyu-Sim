//! Shared window pieces: colours, text, the side panel and hover tooltips.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    group::Kind,
    person::PersonId,
    race::{trait_word, Race, ALL_RACES},
    world::HOUR,
    World,
};

pub const SPEEDS: [(f64, &str); 5] = [(1.0, "1×"), (10.0, "10×"), (60.0, "1 min/s"), (600.0, "10 min/s"), (3600.0, "1 hour/s")];

pub const PANEL: Color = Color::new(0.05, 0.07, 0.07, 0.86);
pub const TEXT: Color = Color::new(0.90, 0.91, 0.88, 1.0);
pub const DIM: Color = Color::new(0.62, 0.66, 0.63, 1.0);

pub fn race_color(r: Race) -> Color {
    match r {
        Race::Roduro => Color::new(0.80, 0.72, 0.60, 1.0), // weathered stone
        Race::Qotiro => Color::new(0.95, 0.47, 0.22, 1.0), // ember
        Race::Horaro => Color::new(0.38, 0.68, 0.96, 1.0), // sea
        Race::Tadoro => Color::new(0.78, 0.68, 0.98, 1.0), // pale violet
    }
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

#[derive(Clone, Copy)]
pub enum Hover {
    Person(PersonId),
    Group(u32),
    Town(u16),
}

/// Collects things under the mouse and keeps the closest.
pub struct Picker {
    pub mouse: Vec2,
    pub best: Option<(f32, Hover)>,
}

impl Picker {
    pub fn new(mouse: Vec2) -> Picker {
        Picker { mouse, best: None }
    }
    pub fn offer(&mut self, at: Vec2, slack: f32, h: Hover) {
        let d = (at - self.mouse).length() - slack;
        if d < 10.0 && self.best.as_ref().map(|(b, _)| d < *b).unwrap_or(true) {
            self.best = Some((d, h));
        }
    }
}

pub struct Ui {
    pub font: Font,
}

impl Ui {
    pub fn text(&self, s: &str, x: f32, y: f32, size: u16, c: Color) {
        draw_text_ex(s, x, y, TextParams { font: Some(&self.font), font_size: size, color: c, ..Default::default() });
    }
    pub fn width(&self, s: &str, size: u16) -> f32 {
        measure_text(s, Some(&self.font), size, 1.0).width
    }
    pub fn centred(&self, s: &str, x: f32, y: f32, size: u16, c: Color) {
        let w = self.width(s, size);
        self.text(s, x - w / 2.0 + 1.0, y + 1.0, size, Color::new(0.0, 0.0, 0.0, 0.6));
        self.text(s, x - w / 2.0, y, size, c);
    }
    pub fn panel(&self, lines: &[(String, Color)], x: f32, y: f32, size: u16) {
        let lh = size as f32 * 1.35;
        let w = lines.iter().map(|(l, _)| self.width(l, size)).fold(0.0, f32::max) + 24.0;
        let h = lines.len() as f32 * lh + 16.0;
        let x = x.min(screen_width() - w - 8.0).max(8.0);
        let y = y.min(screen_height() - h - 8.0).max(8.0);
        draw_rectangle(x, y, w, h, PANEL);
        for (i, (l, c)) in lines.iter().enumerate() {
            self.text(l, x + 12.0, y + 8.0 + lh * (i as f32 + 0.78), size, *c);
        }
    }
}

pub fn draw_hud(ui: &Ui, w: &World, speed_i: usize, paused: bool, sim_ms: f64, draw_ms: f64, view: &str) {
    let mut lines: Vec<(String, Color)> = vec![
        (w.clock(), TEXT),
        (if paused { "Paused".into() } else { format!("Speed {}", SPEEDS[speed_i].1) }, DIM),
        (String::new(), TEXT),
        (format!("{} people  ·  {} on the road", w.people.len() - w.squad.members.len(), w.on_road()), TEXT),
        (format!("Named so far: {}", w.stats.detailed), TEXT),
        (format!("Groups in band 1 / 2 / 3:  {} / {} / {}", w.stats.in_band[1], w.stats.in_band[2], w.stats.in_band[3]), TEXT),
        (format!("Simulation {:.2} ms  ·  drawing {:.1} ms  ·  {:.0} fps", sim_ms, draw_ms, get_fps()), DIM),
        (format!("View: {view}  (V to switch)"), DIM),
        (String::new(), TEXT),
    ];
    for r in ALL_RACES {
        lines.push((format!("●  {}  ({})", r.name(), r.element()), race_color(r)));
    }
    lines.push((String::new(), TEXT));
    for (t, l) in w.log.iter().take(8) {
        lines.push((format!("{}  {}", hhmm(*t), l), DIM));
    }
    ui.panel(&lines, 12.0, 12.0, 16);
}

pub fn hhmm(secs: f64) -> String {
    let s = secs.rem_euclid(24.0 * HOUR);
    format!("{:02}:{:02}", (s / HOUR) as i64, ((s % HOUR) / 60.0) as i64)
}

pub fn describe(w: &World, h: Hover) -> Vec<(String, Color)> {
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
                    if named == 0 { "nobody here has been named yet".to_string() } else { format!("{named} of {} named", g.members.len()) }
                ),
                DIM,
            ));
        }
        Hover::Town(sid) => {
            let s = &w.settlements[sid as usize];
            out.push((s.name.clone(), race_color(s.founders)));
            out.push((
                format!(
                    "{} town, founded by the {}{}",
                    if s.coastal { "Coastal" } else { "Inland" },
                    s.founders.name(),
                    if s.coastal { ", Horaro stilts offshore" } else { "" }
                ),
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

pub fn doing(w: &World, gid: u32) -> String {
    let Some(g) = w.group(gid) else { return String::new() };
    let t = w.time;
    let Some(leg) = g.current_leg(t) else { return String::new() };
    let at = |s: u16| w.settlements[s as usize].name.clone();
    match g.kind {
        Kind::Wanderer { .. } => {
            if t < leg.arrive && t >= leg.depart {
                "Wandering the wilds".into()
            } else {
                "Camped in the wilds".into()
            }
        }
        Kind::Journey { home } => {
            if t < leg.depart {
                format!("Setting out from {} at {}", at(home), hhmm(leg.depart))
            } else if t < leg.arrive {
                match leg.dest {
                    Some(d) if d == home => format!("Walking home to {}, due {}", at(d), hhmm(leg.arrive)),
                    Some(d) => format!("Walking to {}, due {}", at(d), hhmm(leg.arrive)),
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

/// Development aid: `GAHT_SHOT=out.png GAHT_FRAMES=120 cargo run` saves a
/// screenshot after that many frames and quits. `GAHT_HOVER=x,y` fakes the
/// mouse, `GAHT_ZOOM` sets map pixels per metre (or 3D camera distance in
/// metres), `GAHT_SPEED` picks speed 0–4, `GAHT_VIEW=map|3d`, `GAHT_PITCH`
/// and `GAHT_YAW` aim the 3D camera (radians), `GAHT_NUDGE=dx,dy` moves the
/// squad's starting spot by that many metres.
pub struct Shot {
    pub path: String,
    pub frames: u32,
    pub hover: Option<Vec2>,
    pub zoom: Option<f32>,
    pub speed: Option<usize>,
    pub view: Option<String>,
    pub pitch: Option<f32>,
    pub yaw: Option<f32>,
    pub nudge: Option<(f32, f32)>,
}

impl Shot {
    pub fn from_env() -> Option<Shot> {
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
            view: var("GAHT_VIEW"),
            pitch: var("GAHT_PITCH").and_then(|v| v.parse().ok()),
            yaw: var("GAHT_YAW").and_then(|v| v.parse().ok()),
            nudge: var("GAHT_NUDGE").and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            }),
        })
    }
}
