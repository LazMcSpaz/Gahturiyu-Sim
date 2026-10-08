//! Shared window pieces: colours, text, the side panel and hover tooltips.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    body::{self, Part, PARTS},
    items,
    stats::{Attr, Skill, SKILLS},
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
    let fight = battle_lines(w);
    if !fight.is_empty() {
        lines.push((String::new(), TEXT));
        lines.extend(fight);
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
            let st = p.effective_stats();
            out.push((
                format!(
                    "{}  ·  STR {:.0}  AGI {:.0}  TOU {:.0}  INT {:.0}  WIL {:.0}",
                    st.calling.name(),
                    st.attr(Attr::Strength),
                    st.attr(Attr::Agility),
                    st.attr(Attr::Toughness),
                    st.attr(Attr::Intellect),
                    st.attr(Attr::Willpower)
                ),
                TEXT,
            ));
            let mut best: Vec<(Skill, f32)> = SKILLS.iter().map(|&k| (k, st.skill(k))).collect();
            best.sort_by(|a, b| b.1.total_cmp(&a.1));
            out.push((best.iter().take(4).map(|(k, v)| format!("{} {:.0}", k.name(), v)).collect::<Vec<_>>().join("  ·  "), DIM));
            if let Some(d) = &p.detail {
                let worn: Vec<&str> = d.gear.equipped().filter(|&id| items::item(id).slot != items::Slot::MainHand).map(|id| items::item(id).name).collect();
                out.push((format!("{}  ·  {}", d.gear.weapon_name(), worn.join(", ")), TEXT));
                if !d.spells.is_empty() {
                    out.push((format!("Spells: {}", d.spells.iter().map(|s| s.def().name).collect::<Vec<_>>().join(", ")), Color::new(0.65, 0.75, 1.0, 1.0)));
                }
            }
            // Body, now: from the fight if they're in one, else their wounds.
            let (hp, mana, statuses) = match w.fighter(pid) {
                Some(f) => (f.hp, f.mana, f.statuses.iter().map(|s| format!("{:?}", s.kind)).collect::<Vec<_>>()),
                None => (p.wounds.hp_at(&p.stats, w.time), p.mana_at(w.time), vec![]),
            };
            let parts = PARTS.iter().enumerate().map(|(i, part)| format!("{} {:.0}/{:.0}", short_part(*part), hp[i], p.stats.max_hp(*part))).collect::<Vec<_>>().join("  ");
            let ko = body::knocked_out(&hp);
            out.push((parts, if p.dead { Color::new(0.9, 0.3, 0.3, 1.0) } else if ko { Color::new(0.95, 0.6, 0.3, 1.0) } else { DIM }));
            out.push((
                format!(
                    "Mana {:.0}/{:.0}  ·  Might {:.0}{}{}",
                    mana,
                    p.max_mana(),
                    p.might,
                    if p.dead { "  ·  DEAD" } else if ko { "  ·  down" } else { "" },
                    if statuses.is_empty() { String::new() } else { format!("  ·  {}", statuses.join(", ")) }
                ),
                DIM,
            ));
            if p.bandit {
                out.push(("Bandit".into(), Color::new(0.95, 0.35, 0.3, 1.0)));
            }
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
    /// `GAHT_BANDITS=n`: start with n bandits right next to the squad.
    pub bandits: Option<usize>,
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
            bandits: var("GAHT_BANDITS").and_then(|v| v.parse().ok()),
            nudge: var("GAHT_NUDGE").and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.parse().ok()?, y.parse().ok()?))
            }),
        })
    }
}

fn short_part(p: Part) -> &'static str {
    match p {
        Part::Head => "Hd",
        Part::Torso => "To",
        Part::LeftArm => "LA",
        Part::RightArm => "RA",
        Part::LeftLeg => "LL",
        Part::RightLeg => "RL",
    }
}

/// Health (vital share), mana share and whether down, for anyone worth a bar:
/// in a fight, or carrying wounds.
pub fn bar_for(w: &World, pid: PersonId) -> Option<(f32, Option<f32>, bool)> {
    let p = &w.people[pid as usize];
    if p.dead {
        return None;
    }
    if let Some(f) = w.fighter(pid) {
        let mana = if f.spells.is_empty() { None } else { Some(f.mana / f.max_mana.max(1.0)) };
        return Some((f.vitality(), mana, f.ko));
    }
    if !p.wounds.is_hurt(w.time) {
        return None;
    }
    let hp = p.wounds.hp_at(&p.stats, w.time);
    let vit = (hp[0].max(0.0) / p.stats.max_hp(Part::Head)).min(hp[1].max(0.0) / p.stats.max_hp(Part::Torso));
    Some((vit, None, body::knocked_out(&hp)))
}

/// A small health bar (and mana bar under it) centred at x, sitting at y.
pub fn draw_bar(x: f32, y: f32, vit: f32, mana: Option<f32>, down: bool) {
    let w = 30.0;
    draw_rectangle(x - w / 2.0 - 1.0, y - 1.0, w + 2.0, 6.0, Color::new(0.0, 0.0, 0.0, 0.7));
    let c = if down { Color::new(0.9, 0.55, 0.2, 1.0) } else { Color::new(0.85 - vit * 0.6, 0.25 + vit * 0.6, 0.25, 1.0) };
    draw_rectangle(x - w / 2.0, y, w * vit.clamp(0.0, 1.0), 4.0, c);
    if let Some(m) = mana {
        draw_rectangle(x - w / 2.0 - 1.0, y + 5.0, w + 2.0, 4.0, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle(x - w / 2.0, y + 6.0, w * m.clamp(0.0, 1.0), 2.0, Color::new(0.35, 0.55, 1.0, 1.0));
    }
}

/// The squad's fight, as a few lines for the side panel.
pub fn battle_lines(w: &World) -> Vec<(String, Color)> {
    let Some(b) = w.squad_battle() else { return vec![] };
    let mut out = vec![(format!("FIGHT  ·  {:.0}s", b.time - b.start), Color::new(0.95, 0.45, 0.35, 1.0))];
    for (_, l) in b.log.iter().rev().take(7).collect::<Vec<_>>().into_iter().rev() {
        out.push((l.clone(), DIM));
    }
    out
}
