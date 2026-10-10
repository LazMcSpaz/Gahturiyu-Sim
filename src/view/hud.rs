//! Text and panels drawn over the view: the side panel, hover descriptions,
//! health bars and the help line.
//!
//! Everything flat is drawn with egui's painter through `Canvas`, a thin
//! layer that keeps the old window's coordinates (pixels from the top left,
//! text placed by its baseline) so the layouts carried over unchanged.

use bevy::math::Vec2;
use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind};

use gahturiyu_sim::sim::{
    body::{self, Part, PARTS},
    group::Kind,
    items,
    person::PersonId,
    race::{trait_word, ALL_RACES},
    stats::{Attr, Skill, SKILLS},
    world::HOUR,
    World,
};

use super::app::Hover;
use super::palette::{self, eg, ega, race_color, Rgb, DIM, GOLD, SNEAK, TEXT, WARN};

pub const SPEEDS: [(f64, &str); 5] = [(1.0, "1×"), (10.0, "10×"), (60.0, "1 min/s"), (600.0, "10 min/s"), (3600.0, "1 hour/s")];
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(16, 15, 11, 228);

pub struct Canvas {
    pub p: egui::Painter,
    pub w: f32,
    pub h: f32,
}

pub fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
}

fn font(size: f32) -> FontId {
    FontId::proportional(size * 1.02)
}

/// The HUD's typefaces (`app.rs` loads them): `Body` is Alegreya (the
/// default for all panel text), `Title` Cinzel for headings and place names,
/// `Caps` Alegreya small caps for names, `Italic` for asides and states.
#[derive(Clone, Copy, PartialEq)]
pub enum Face {
    Body,
    Title,
    Caps,
    Italic,
}

pub fn face(f: Face, size: f32) -> FontId {
    match f {
        Face::Body => font(size),
        Face::Title => FontId::new(size, egui::FontFamily::Name("title".into())),
        Face::Caps => FontId::new(size * 1.04, egui::FontFamily::Name("caps".into())),
        Face::Italic => FontId::new(size * 1.02, egui::FontFamily::Name("italic".into())),
    }
}

impl Canvas {
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color32) {
        self.p.rect_filled(r(x, y, w, h), 0.0, c);
    }
    pub fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, t: f32, c: Color32) {
        self.p.rect_stroke(r(x, y, w, h), 0.0, Stroke::new(t, c), StrokeKind::Inside);
    }
    /// Text with its baseline at `y`.
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Rgb) {
        self.text_c(s, x, y, size, eg(c));
    }
    pub fn text_c(&self, s: &str, x: f32, y: f32, size: f32, c: Color32) {
        self.p.text(Pos2::new(x, y - size * 0.86), Align2::LEFT_TOP, s, font(size), c);
    }
    pub fn width(&self, s: &str, size: f32) -> f32 {
        self.p.layout_no_wrap(s.to_string(), font(size), Color32::WHITE).size().x
    }
    pub fn centred(&self, s: &str, x: f32, y: f32, size: f32, c: Rgb) {
        let w = self.width(s, size);
        self.text_c(s, x - w / 2.0 + 1.0, y + 1.0, size, Color32::from_black_alpha(150));
        self.text(s, x - w / 2.0, y, size, c);
    }
    /// A dark box of lines; returns where it went.
    pub fn panel(&self, lines: &[(String, Rgb)], x: f32, y: f32, size: f32) -> Rect {
        // A gold first line is a title: set in capitals, spaced out.
        let titled = lines.first().is_some_and(|l| l.1 == GOLD);
        let lh = size * 1.35;
        let tw = |l: &str| if titled { self.styled_width(&l.to_uppercase(), size, Face::Title, 1.5) } else { self.width(l, size) };
        let w = lines.iter().enumerate().map(|(i, (l, _))| if i == 0 { tw(l) } else { self.width(l, size) }).fold(0.0, f32::max) + 32.0;
        let h = lines.len() as f32 * lh + 20.0;
        let x = x.min(self.w - w - 8.0).max(8.0);
        let y = y.min(self.h - h - 8.0).max(8.0);
        // Nearly opaque: tooltips often sit over other panels.
        self.grad(x, y, w, h, Color32::from_rgba_premultiplied(24, 22, 16, 246), Color32::from_rgba_premultiplied(12, 11, 8, 246), false);
        let gold = eg(palette::BRASS);
        self.rule(x - 6.0, x + w + 6.0, y, 1.5, gold);
        self.rule(x - 6.0, x + w + 6.0, y + h, 1.5, gold);
        for (i, (l, c)) in lines.iter().enumerate() {
            let by = y + 10.0 + lh * (i as f32 + 0.78);
            if i == 0 && titled {
                self.styled(&l.to_uppercase(), x + 16.0, by, size, eg(*c), Face::Title, 1.5);
            } else {
                self.text(l, x + 16.0, by, size, *c);
            }
        }
        r(x, y, w, h)
    }
    pub fn circle(&self, x: f32, y: f32, rad: f32, c: Color32) {
        self.p.circle_filled(Pos2::new(x, y), rad, c);
    }
    pub fn circle_lines(&self, x: f32, y: f32, rad: f32, t: f32, c: Color32) {
        self.p.circle_stroke(Pos2::new(x, y), rad, Stroke::new(t, c));
    }
    pub fn line(&self, a: Vec2, b: Vec2, t: f32, c: Color32) {
        self.p.line_segment([Pos2::new(a.x, a.y), Pos2::new(b.x, b.y)], Stroke::new(t, c));
    }
    pub fn triangle(&self, a: Vec2, b: Vec2, c: Vec2, col: Color32) {
        self.p.add(Shape::convex_polygon(vec![Pos2::new(a.x, a.y), Pos2::new(b.x, b.y), Pos2::new(c.x, c.y)], col, Stroke::NONE));
    }
    /// Text in one of the HUD's faces, with extra space between letters;
    /// baseline at `y`. Returns its width.
    pub fn styled(&self, s: &str, x: f32, y: f32, size: f32, c: Color32, f: Face, spacing: f32) -> f32 {
        let g = self.galley(s, size, c, f, spacing);
        let w = g.size().x;
        self.p.galley(Pos2::new(x, y - size * 0.86), g, c);
        w
    }
    /// As `styled`, right-aligned to `x`.
    pub fn styled_right(&self, s: &str, x: f32, y: f32, size: f32, c: Color32, f: Face, spacing: f32) -> f32 {
        let w = self.styled_width(s, size, f, spacing);
        self.styled(s, x - w, y, size, c, f, spacing)
    }
    /// As `styled`, centred on `x`, with a soft shadow.
    pub fn styled_centred(&self, s: &str, x: f32, y: f32, size: f32, c: Color32, f: Face, spacing: f32) {
        let w = self.styled_width(s, size, f, spacing);
        self.styled(s, x - w / 2.0 + 1.0, y + 1.5, size, Color32::from_black_alpha(170), f, spacing);
        self.styled(s, x - w / 2.0, y, size, c, f, spacing);
    }
    pub fn styled_width(&self, s: &str, size: f32, f: Face, spacing: f32) -> f32 {
        self.galley(s, size, Color32::WHITE, f, spacing).size().x
    }
    fn galley(&self, s: &str, size: f32, c: Color32, f: Face, spacing: f32) -> std::sync::Arc<egui::Galley> {
        let mut job = egui::text::LayoutJob::default();
        job.append(s, 0.0, egui::TextFormat { font_id: face(f, size), color: c, extra_letter_spacing: spacing, ..Default::default() });
        self.p.layout_job(job)
    }
    /// A filled shape (convex or not: it's fanned from its first point).
    pub fn poly(&self, pts: &[Vec2], c: Color32) {
        if pts.len() < 3 {
            return;
        }
        let mut m = egui::Mesh::default();
        for p in pts {
            m.colored_vertex(Pos2::new(p.x, p.y), c);
        }
        for i in 1..pts.len() as u32 - 1 {
            m.add_triangle(0, i, i + 1);
        }
        self.p.add(Shape::mesh(m));
    }
    pub fn poly_lines(&self, pts: &[Vec2], t: f32, c: Color32, closed: bool) {
        let v: Vec<Pos2> = pts.iter().map(|p| Pos2::new(p.x, p.y)).collect();
        if closed {
            self.p.add(Shape::closed_line(v, Stroke::new(t, c)));
        } else {
            self.p.add(Shape::line(v, Stroke::new(t, c)));
        }
    }
    /// A box shaded from one colour to another, left to right or top to bottom.
    pub fn grad(&self, x: f32, y: f32, w: f32, h: f32, a: Color32, b: Color32, across: bool) {
        let mut m = egui::Mesh::default();
        let (tl, tr, br, bl) = if across { (a, b, b, a) } else { (a, a, b, b) };
        m.colored_vertex(Pos2::new(x, y), tl);
        m.colored_vertex(Pos2::new(x + w, y), tr);
        m.colored_vertex(Pos2::new(x + w, y + h), br);
        m.colored_vertex(Pos2::new(x, y + h), bl);
        m.add_triangle(0, 1, 2);
        m.add_triangle(0, 2, 3);
        self.p.add(Shape::mesh(m));
    }
    /// A panel's ground: dark umber, shaded, ruled in brass top and bottom.
    pub fn frame_box(&self, x: f32, y: f32, w: f32, h: f32) {
        self.grad(x, y, w, h, Color32::from_rgba_unmultiplied(30, 27, 20, 238), Color32::from_rgba_unmultiplied(14, 13, 10, 242), false);
        let brass = eg(palette::BRASS);
        self.rule(x - 8.0, x + w + 8.0, y, 1.6, brass);
        self.rule(x - 8.0, x + w + 8.0, y + h, 1.6, brass);
        let side = Color32::from_rgba_unmultiplied(110, 85, 45, 90);
        self.line(Vec2::new(x, y + 6.0), Vec2::new(x, y + h - 6.0), 1.0, side);
        self.line(Vec2::new(x + w, y + 6.0), Vec2::new(x + w, y + h - 6.0), 1.0, side);
    }
    /// A box shaded with a colour at each corner (top left, top right,
    /// bottom right, bottom left).
    pub fn grad4(&self, x: f32, y: f32, w: f32, h: f32, tl: Color32, tr: Color32, br: Color32, bl: Color32) {
        let mut m = egui::Mesh::default();
        m.colored_vertex(Pos2::new(x, y), tl);
        m.colored_vertex(Pos2::new(x + w, y), tr);
        m.colored_vertex(Pos2::new(x + w, y + h), br);
        m.colored_vertex(Pos2::new(x, y + h), bl);
        m.add_triangle(0, 1, 2);
        m.add_triangle(0, 2, 3);
        self.p.add(Shape::mesh(m));
    }
    /// A thin rule that fades in from nothing at both ends.
    pub fn rule(&self, x0: f32, x1: f32, y: f32, t: f32, c: Color32) {
        let mid = (x0 + x1) / 2.0;
        let clear = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 0);
        self.grad(x0, y - t / 2.0, mid - x0, t, clear, c, true);
        self.grad(mid, y - t / 2.0, x1 - mid, t, c, clear, true);
    }
    /// A rule fading in from the left only.
    pub fn rule_in(&self, x0: f32, x1: f32, y: f32, t: f32, c: Color32, from_left: bool) {
        let clear = Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 0);
        if from_left {
            self.grad(x0, y - t / 2.0, x1 - x0, t, clear, c, true);
        } else {
            self.grad(x0, y - t / 2.0, x1 - x0, t, c, clear, true);
        }
    }
    /// A small diamond (the HUD's bullet and bar end).
    pub fn diamond(&self, x: f32, y: f32, r: f32, c: Color32) {
        self.poly(&[Vec2::new(x, y - r), Vec2::new(x + r, y), Vec2::new(x, y + r), Vec2::new(x - r, y)], c);
    }
    pub fn diamond_lines(&self, x: f32, y: f32, r: f32, t: f32, c: Color32) {
        self.poly_lines(&[Vec2::new(x, y - r), Vec2::new(x + r, y), Vec2::new(x, y + r), Vec2::new(x - r, y)], t, c, true);
    }
    /// An arc of a circle, angles in radians (0 = right, clockwise on screen).
    pub fn arc(&self, x: f32, y: f32, rad: f32, a0: f32, a1: f32, t: f32, c: Color32) {
        let n = ((a1 - a0).abs() * rad / 4.0).ceil().max(4.0) as usize;
        let pts: Vec<Pos2> = (0..=n).map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            Pos2::new(x + a.cos() * rad, y + a.sin() * rad)
        }).collect();
        self.p.add(Shape::line(pts, Stroke::new(t, c)));
    }
    pub fn ellipse_lines(&self, x: f32, y: f32, rx: f32, ry: f32, t: f32, c: Color32) {
        let pts: Vec<Pos2> = (0..24).map(|i| {
            let a = i as f32 / 24.0 * std::f32::consts::TAU;
            Pos2::new(x + a.cos() * rx, y + a.sin() * ry)
        }).collect();
        self.p.add(Shape::closed_line(pts, Stroke::new(t, c)));
    }
}

pub fn draw_hud(c: &Canvas, w: &World, speed_i: usize, paused: bool, sim_ms: f64, frame_ms: f64, view: &str) -> Rect {
    let mut lines: Vec<(String, Rgb)> = vec![
        (w.clock(), TEXT),
        (if paused { "Paused".into() } else { format!("Speed {}", SPEEDS[speed_i].1) }, DIM),
        (String::new(), TEXT),
        (format!("{} people  ·  {} on the road", w.people.len() - w.squad.members.len(), w.on_road()), TEXT),
        (format!("Named so far: {}", w.stats.detailed), TEXT),
        (format!("Light here: {}  ·  ambushes so far: {}", gahturiyu_sim::sim::stealth::light_word(w.light_at(w.squad.pos)), w.stats.ambushes), TEXT),
        (format!("Groups in band 1 / 2 / 3:  {} / {} / {}", w.stats.in_band[1], w.stats.in_band[2], w.stats.in_band[3]), TEXT),
        (format!("Simulation {:.2} ms  ·  frame {:.1} ms  ·  {:.0} fps", sim_ms, frame_ms, 1000.0 / frame_ms.max(0.1)), DIM),
        (format!("View: {view}  (V to switch)"), DIM),
        (String::new(), TEXT),
    ];
    for r in ALL_RACES {
        lines.push((format!("●  {}  ({})", r.name(), r.element()), race_color(r)));
    }
    for (town, b) in &w.bounty {
        let heard = w.towns_heard(*town);
        let spread = if heard > 1 { format!("  ·  word has reached {} towns", heard) } else { String::new() };
        lines.push((format!("Bounty in {}: {:.0}{spread}", w.settlements[*town as usize].name, b), [0.95, 0.45, 0.35]));
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
    c.panel(&lines, 12.0, 12.0, 16.0)
}

pub fn hhmm(secs: f64) -> String {
    let s = secs.rem_euclid(24.0 * HOUR);
    format!("{:02}:{:02}", (s / HOUR) as i64, ((s % HOUR) / 60.0) as i64)
}

/// Whether hovers show everything about strangers (the L readout, for
/// testing). Otherwise a stranger shows only what you could see or would
/// know of them (Laz: a game, not a simulator): who they're with, their
/// people and trade, what they wear and carry, roughly how they are.
static SEE_ALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_see_all(on: bool) {
    SEE_ALL.store(on, std::sync::atomic::Ordering::Relaxed);
}

fn see_all() -> bool {
    SEE_ALL.load(std::sync::atomic::Ordering::Relaxed)
}

/// How someone looks, in a word, from their wounds.
fn condition_word(hp: &[f32; 6], max: &[f32; 6], dead: bool) -> Option<&'static str> {
    if dead {
        return Some("Dead");
    }
    if body::knocked_out(hp) {
        return Some("Down");
    }
    let (h, m) = hp.iter().zip(max).fold((0.0, 0.0), |a, (h, m)| (a.0 + h.max(0.0), a.1 + m));
    let share = if m > 0.0 { h / m } else { 1.0 };
    if share < 0.5 {
        Some("Badly hurt")
    } else if share < 0.9 {
        Some("Hurt")
    } else {
        None
    }
}

pub fn describe(w: &World, h: Hover) -> Vec<(String, Rgb)> {
    let mut out: Vec<(String, Rgb)> = Vec::new();
    match h {
        Hover::Person(pid) => {
            let p = &w.people[pid as usize];
            let name = p.name().unwrap_or("(not yet named)");
            out.push((format!("{}  ·  {}", name, p.race.name()), race_color(p.race)));
            if w.can_loot(pid) {
                out.push(("Beaten: click to go through their things (Shift-click to carry)".to_string(), GOLD));
            }
            // A merchant: open now, or when.
            if let Some(at) = w.trades_next(pid) {
                if at <= w.time + 1.0 {
                    out.push(("Trading now".to_string(), GOLD));
                } else {
                    out.push((format!("Merchant: at their stall from {}{}", hhmm(at), w.day_word(at)), DIM));
                }
            }
            match w.join_terms(pid) {
                Some(0) => out.push(("Restless: might join the squad if asked".to_string(), GOLD)),
                Some(fee) => out.push((format!("Restless: might join the squad, for {fee} coin"), GOLD)),
                None => {}
            }
            // A stranger: only what shows.
            if !p.in_squad && !see_all() {
                stranger_lines(w, pid, &mut out);
                return out;
            }
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
                    out.push((format!("Spells: {}", d.spells.iter().map(|s| s.def().name).collect::<Vec<_>>().join(", ")), [0.65, 0.75, 1.0]));
                }
            }
            let (hp, mana, statuses) = match w.fighter(pid) {
                Some(f) => (f.hp, f.mana, f.statuses.iter().map(|s| format!("{:?}", s.does)).collect::<Vec<_>>()),
                None => (p.wounds.hp_at(&p.stats, w.time), p.mana_at(w.time), vec![]),
            };
            let missing = w.fighter(pid).map(|f| f.missing).unwrap_or(p.wounds.missing);
            let parts = PARTS
                .iter()
                .enumerate()
                .map(|(i, part)| if missing[i] { format!("{} lost", short_part(*part)) } else { format!("{} {:.0}/{:.0}", short_part(*part), hp[i], p.stats.max_hp(*part)) })
                .collect::<Vec<_>>()
                .join("  ");
            let ko = body::knocked_out(&hp);
            out.push((parts, if p.dead { [0.9, 0.3, 0.3] } else if ko { [0.95, 0.6, 0.3] } else { DIM }));
            let gone: Vec<&str> = PARTS.iter().enumerate().filter(|(i, _)| missing[*i]).map(|(_, p)| p.name()).collect();
            if !gone.is_empty() {
                out.push((format!("Lost for good: {}", gone.join(", ")), [0.9, 0.35, 0.3]));
            }
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
                let state = match w.group_of[pid as usize] {
                    _ if w.fighter(pid).is_some() => "fighting",
                    Some(g) if w.has_noticed(g) => "has seen you",
                    Some(g) if w.suspicion.iter().any(|((x, _), v)| *x == g && *v > 0.3) => "suspicious",
                    _ => "unaware of you",
                };
                out.push((format!("Bandit  ·  {state}  ·  click to attack"), [0.95, 0.35, 0.3]));
            }
            if let Some(c) = w.carrying(pid) {
                out.push((format!("Carrying {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
            }
            if let Some(c) = w.carried_by(pid) {
                out.push((format!("Carried by {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
            } else if body::knocked_out(&hp) && !p.dead {
                out.push(("Down. Select someone and click to carry them.".into(), DIM));
            }
            if let Some(hrs) = w.torch_hours_left(pid) {
                let lit = if w.torch_lit(pid) { "lit" } else { "out" };
                out.push((format!("Torch {lit}  ·  {:.1} h left  ·  T to light or put out", hrs), GOLD));
            }
            if p.in_squad && w.is_sneaking(pid) {
                out.push((format!("Sneaking  ·  noise {:.1}  ·  visibility {:.0}%", w.noise_of(pid), w.visibility_of(pid) * 100.0), SNEAK));
            }
            // Bound to work for someone, and for how long.
            if let Some(b) = w.bond_of(pid) {
                let place = &w.settlements[b.town as usize].name;
                let line = if b.slave { format!("Enslaved in {place}") } else { format!("Bound to work in {place}  ·  {:.1} days left{}", (b.until - w.time) / 86400.0, if b.recorded { "  ·  recorded" } else { "" }) };
                out.push((line, [0.95, 0.6, 0.3]));
            }
            // Your squad member's name in the town they're in.
            if p.in_squad {
                if let Some(town) = w.town_at(w.person_pos(pid)) {
                    let s = w.standing(pid, town);
                    let known = w.bounty_known_in(town);
                    out.push((
                        format!(
                            "Standing in {} {:.0}: {}{}",
                            w.settlements[town as usize].name,
                            s,
                            gahturiyu_sim::sim::World::standing_word(s),
                            if known > 0.0 { format!("  ·  bounties known here {known:.0}") } else { String::new() }
                        ),
                        if known > 0.0 { [0.95, 0.6, 0.3] } else { DIM },
                    ));
                }
            }
            if p.in_squad {
                out.push(("Your squad".into(), TEXT));
                if let Some(c) = w.contract_of(pid) {
                    let what = c.post.map(|j| format!("Working as {}", j.name().to_lowercase())).unwrap_or_else(|| "On guard".into());
                    out.push((format!("{what} at the {} in {}, {:02.0}:00–{:02.0}:00  ·  {:.0} coin a day  ·  {} days paid", w.society.towns[c.town as usize].places[c.place as usize].kind.name().to_lowercase(), w.settlements[c.town as usize].name, c.hours.0, c.hours.1, c.pay, c.days_paid), GOLD));
                }
            } else {
                out.extend(work_lines(w, pid));
                if let Some(fee) = w.join_terms(pid) {
                    let price = if fee > 0 { format!("for {fee} coin") } else { "for nothing".into() };
                    out.push((format!("Would join the squad {price}: {}", w.recruit_card(pid)), GOLD));
                }
                out.extend(life_lines(w, pid));
                if let Some(g) = w.group_of[pid as usize].and_then(|g| w.group(g)) {
                    out.push((doing(w, g.id), DIM));
                } else if let Some(home) = p.home {
                    match w.doing_now(pid) {
                        Some((d, spot)) => out.push((format!("{} — {}", capital(d.word()), where_word(w, pid, spot, home)), GOLD)),
                        None => out.push((format!("At home in {}", w.settlements[home as usize].name), DIM)),
                    }
                }
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
            if see_all() {
                let might: f32 = g.members.iter().map(|&m| w.people[m as usize].might).sum();
                out.push((format!("Combined might {:.0}", might), DIM));
                let named = g.members.iter().filter(|&&m| w.people[m as usize].detail.is_some()).count();
                out.push((
                    format!("Band {}  ·  {}", g.band, if named == 0 { "nobody here has been named yet".to_string() } else { format!("{named} of {} named", g.members.len()) }),
                    DIM,
                ));
            }
        }
        Hover::Item(gid) => {
            let Some(g) = w.ground.iter().find(|g| g.id == gid) else { return out };
            out = super::squadui::item_lines(g.item);
            let english = items::item(g.item).name;
            out[0].0 = super::lexicon::thing(w, english);
            if let Some(other) = super::lexicon::thing_other(w, english) {
                out.insert(1, (other, DIM));
            }
            if g.count > 1 {
                out[0].0 = format!("{}  ×{}", out[0].0, g.count);
            }
            // Someone's: show the odds of being seen, as for a container.
            if let Some(town) = g.owner {
                let near = w.squad.members.iter().copied().min_by(|&a, &b| w.person_pos(a).dist(g.pos).total_cmp(&w.person_pos(b).dist(g.pos)));
                if let Some(m) = near {
                    let c = w.catch_chance(m, g.pos, town);
                    out.push((if c <= 0.0 { "Someone's; nobody would see it taken from here".to_string() } else { format!("Someone's: taking it is theft, {:.0}% chance of being seen", c * 100.0) }, WARN));
                }
            }
            out.push(("Click to pick up".into(), TEXT));
        }
        Hover::Door(id) => {
            let s = &w.settlements[id.0 as usize];
            let Some(d) = gahturiyu_sim::sim::buildings::door_of(s, id.1) else { return out };
            // What it is, by trade and who keeps it ("Maker's forge · kept by
            // Ana, smith"), or just its variant ("Grown cottage").
            match super::signs::keeper(w, id) {
                Some(_) => {
                    out.push((super::signs::building_title(w, id).unwrap_or_else(|| d.variant().name.to_string()), TEXT));
                    out.push((s.name.clone(), DIM));
                }
                None => out.push((format!("{}  ·  {}", d.variant().name, s.name), TEXT)),
            }
            if d.lock <= 0.0 {
                out.push(("No lock.  Click to go in.".into(), DIM));
            } else if w.is_locked(id) {
                out.push((format!("Locked for the night: {}", w.lock_outlook(d.lock)), WARN));
                out.push(("Click to pick the lock (needs a lockpick).".into(), DIM));
            } else {
                out.push((format!("Open  ·  lock {:.0}, locked 20:00–06:00", d.lock), DIM));
                out.push(("Click to go in.".into(), DIM));
            }
        }
        Hover::Building(id) => {
            let st = &w.settlements[id.0 as usize];
            let Some(d) = w.door(id) else { return out };
            out.push((d.variant().name.to_string(), GOLD));
            let owner = super::interiors::owner_text(w, w.belongs_to(id));
            out.push((format!("{}  ·  belongs to {owner}", st.name), TEXT));
            let lock = if d.lock <= 0.0 {
                "No lock".to_string()
            } else if w.is_locked(id) {
                format!("Locked for the night (lock {:.0})", d.lock)
            } else {
                format!("Open by day; locked 20:00–06:00 (lock {:.0})", d.lock)
            };
            out.push((lock, if w.is_locked(id) { WARN } else { DIM }));
            let inside = w.residents_in_band1(id.0).into_iter().filter(|&p| w.building_at(w.person_pos(p)).map(|x| x.id) == Some(id)).count();
            if inside > 0 {
                out.push((format!("{inside} inside"), DIM));
            }
        }
        Hover::Furniture(id, k) => {
            let Some(d) = w.door(id) else { return out };
            let Some(p) = d.variant().furniture.get(k as usize) else { return out };
            out.push((capital(p.what.name()), GOLD));
            out.push((format!("In the {}", d.variant().name.to_lowercase()), DIM));
        }
        Hover::Camp(k) => {
            out.push(("Bandit camp".to_string(), GOLD));
            if let Some(c) = w.camps.get(k) {
                let n = w.group(c.group).map(|g| g.members.iter().filter(|&&m| !w.people[m as usize].dead && !w.is_down(m)).count()).unwrap_or(0);
                out.push((format!("{n} standing  ·  they watch the roads, and anyone who comes close"), WARN));
                if w.is_warden(c.group) {
                    out.push(("Dug in at a ruin".to_string(), WARN));
                }
            }
        }
        Hover::Place(t, k) => {
            let Some(p) = w.society.towns.get(t as usize).and_then(|tl| tl.places.get(k as usize)) else { return out };
            out.push((p.kind.name().to_string(), GOLD));
            let here = w.settlements[t as usize].residents.iter().filter(|&&q| w.workplace_of(q).is_some_and(|wp| wp.pos.dist(p.pos) < 1.0) && w.at_work(q, w.time)).count();
            out.push((format!("{}  ·  {here} at work here now", w.settlements[t as usize].name), DIM));
        }
        Hover::Container(id) => {
            let Some(k) = w.container(id) else { return out };
            if gahturiyu_sim::sim::containers::is_stash(id) {
                out.push(("The bandits' stash".to_string(), TEXT));
            } else {
                out.push((format!("A {}  ·  belongs to {}", k.what.name(), super::interiors::owner_text(w, w.container_owner(id).unwrap_or(k.owner))), TEXT));
            }
            if w.container_locked(id) {
                out.push((format!("Locked: {}", w.lock_outlook(k.lock)), WARN));
                out.push(("Click to pick the lock (needs a lockpick).".into(), DIM));
            } else if gahturiyu_sim::sim::containers::is_wild(id) {
                out.push(("Click to open (nobody's: taking is no crime, but keepers nearby may hear)".into(), DIM));
            } else {
                out.push(("Click to open (taking is theft)".into(), DIM));
            }
        }
        Hover::Node(id) => {
            let Some(n) = w.nodes.iter().find(|n| n.id == id) else { return out };
            out.push((format!("{}  ×{}", items::item(n.item).name, n.amount), GOLD));
            if n.ready(w.time) {
                out.push(("Click to gather".into(), TEXT));
            } else {
                out.push((format!("Picked; grows back at {}", hhmm(n.picked_at.unwrap_or(0.0) + 24.0 * HOUR)), DIM));
            }
        }
        Hover::Ruin(id) => {
            let Some(ru) = w.ruins.get(id as usize) else { return out };
            out.push((ru.name(w), GOLD));
            let held = w.ruin_held(id);
            let line = match ru.kind {
                gahturiyu_sim::sim::ruins::RuinKind::Ruin if held => "Held by wardens: a hard band, dug in",
                gahturiyu_sim::sim::ruins::RuinKind::Lair(_) if held => "Its owner is about",
                _ => "Nobody guards it now",
            };
            out.push((line.to_string(), if held { WARN } else { TEXT }));
            let n = w.ruin_cache(id);
            out.push((if n > 0 { format!("{n} things lying inside") } else { "Picked clean".to_string() }, DIM));
        }
        Hover::Deposit(id) => {
            let Some(d) = w.deposit(id) else { return out };
            let town = &w.settlements[d.town as usize].name;
            out.push((format!("{} of {town}", d.face().name), GOLD));
            let left = d.left_at(w.time).floor();
            out.push((format!("{} × {} left (of {}); a unit {:.0} kg, fetches about {} coin in {town} (less each for a big lot)", left, items::item(d.item).name.to_lowercase(), d.face().cap, items::item(d.item).weight, w.fetches_in(d.town, d.item)), TEXT));
            if let Some(h) = w.deposit_full_in(id) {
                out.push((format!("Grows back: full again in {h:.0} h"), DIM));
            }
            out.push(("Click: the selected work it until their packs are full".into(), DIM));
        }
        Hover::Station(i) => {
            let (_, st) = w.stations[i];
            use gahturiyu_sim::sim::crafting::Station as S;
            out.push((st.name().to_string(), GOLD));
            let what = match st {
                S::Forge => "Smithing: weapons, smelting ore; Armoring: iron pieces",
                S::Bench => "Armoring: tanning, leather and hide pieces, bucklers",
                S::Desk => "Inscription: scrolls",
                S::AlchemyTable => "Alchemy: potions (a mortar and pestle works anywhere)",
                S::Loom => "Weaving: reed, shell, fishskin and tentsilk; sealing with pitch",
                S::Workbench => "Handcraft: leather, cloth and wood — the shared basics",
                S::GrowerBed => "Stone-tending: grown stone, slow to come",
            };
            out.push((what.into(), TEXT));
            out.push(("Stand by it and press K".into(), DIM));
        }
        Hover::Town(sid) => {
            let s = &w.settlements[sid as usize];
            out.push((super::lexicon::town(w, sid), race_color(s.founders)));
            if let Some(other) = super::lexicon::town_other(w, sid) {
                out.push((other, DIM));
            }
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
            if let Some(tl) = w.society.towns.get(sid as usize) {
                let k = w.society.communities[tl.shore as usize].customs;
                out.push((format!("{}  ·  {}", k.cooking.name(), k.rhythm.name()), TEXT));
                out.push((format!("Fed {:.0}% yesterday  ·  click for the town panel", w.town_food(sid) * 100.0), DIM));
            }
        }
        Hover::Torch(i) => {
            let Some(st) = w.standing.get(i) else { return out };
            out.push(("Standing torch".into(), GOLD));
            out.push((format!("Burns until {}", hhmm(st.out_at)), DIM));
        }
    }
    out
}

pub fn doing(w: &World, gid: u32) -> String {
    let Some(g) = w.group(gid) else { return String::new() };
    let carrying = match &g.cargo {
        Some(c) if c.robbed => "  ·  a caravan, robbed of its goods".to_string(),
        Some(c) if c.delivered => format!("  ·  a caravan bringing {:.0} coin home", c.coin),
        Some(c) => format!("  ·  a caravan carrying {:.0} {} to {}", c.amount, c.good.name().to_lowercase(), w.settlements[c.to as usize].name),
        None => String::new(),
    };
    doing_on_road(w, g) + &carrying
}

fn doing_on_road(w: &World, g: &gahturiyu_sim::sim::group::Group) -> String {
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
    let p = w.people.get(pid as usize)?;
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

pub fn health_color(vit: f32, down: bool) -> Color32 {
    if down {
        eg([0.9, 0.55, 0.2])
    } else {
        eg([0.85 - vit * 0.6, 0.25 + vit * 0.6, 0.25])
    }
}

/// A small health bar (and mana bar under it) centred at x, sitting at y.
pub fn draw_bar(c: &Canvas, x: f32, y: f32, vit: f32, mana: Option<f32>, down: bool) {
    let w = 30.0;
    c.rect(x - w / 2.0 - 1.0, y - 1.0, w + 2.0, 6.0, Color32::from_black_alpha(178));
    c.rect(x - w / 2.0, y, w * vit.clamp(0.0, 1.0), 4.0, health_color(vit, down));
    if let Some(m) = mana {
        c.rect(x - w / 2.0 - 1.0, y + 5.0, w + 2.0, 4.0, Color32::from_black_alpha(178));
        c.rect(x - w / 2.0, y + 6.0, w * m.clamp(0.0, 1.0), 2.0, eg(palette::MANA));
    }
}

/// The squad's fight, as a few lines for the side panel.
pub fn battle_lines(w: &World) -> Vec<(String, Rgb)> {
    let Some(b) = w.squad_battle() else { return vec![] };
    let mut out = vec![(format!("FIGHT  ·  {:.0}s", b.time - b.start), [0.95, 0.45, 0.35])];
    for (_, l) in b.log.iter().rev().take(7).collect::<Vec<_>>().into_iter().rev() {
        out.push((l.clone(), DIM));
    }
    out
}

pub fn shadow(a: f32) -> Color32 {
    ega([0.0, 0.0, 0.0], a)
}


fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Someone's work: what they do, where, and the hours they keep.
/// Their work status, what's on their mind, their honour, a grudge (if the
/// squad has talked with them), and their household's money and feelings.
/// What a stranger shows: who they're with, what they do, what they wear
/// and carry, and how they look. (Their nature, skills, purse and needs are
/// theirs to keep; you learn some by talking.)
fn stranger_lines(w: &World, pid: PersonId, out: &mut Vec<(String, Rgb)>) {
    let p = &w.people[pid as usize];
    if p.bandit {
        out.push(("Bandit  ·  click to attack".to_string(), [0.95, 0.35, 0.3]));
    }
    out.extend(work_lines(w, pid));
    if let Some(d) = &p.detail {
        let worn: Vec<String> = d.gear.equipped().filter(|&id| items::item(id).slot != items::Slot::MainHand).map(|id| items::item(id).name.to_lowercase()).collect();
        let weapon = d.gear.weapon_name();
        let mut line = if worn.is_empty() { String::new() } else { format!("Wearing {}", worn.join(", ")) };
        if weapon != "bare hands" {
            line = if line.is_empty() { format!("Carries a {}", weapon.to_lowercase()) } else { format!("{line}; carries a {}", weapon.to_lowercase()) };
        }
        if !line.is_empty() {
            out.push((line, TEXT));
        }
    }
    let (hp, missing) = match w.fighter(pid) {
        Some(f) => (f.hp, f.missing),
        None => (p.wounds.hp_at(&p.stats, w.time), p.wounds.missing),
    };
    let max: [f32; 6] = std::array::from_fn(|i| p.stats.max_hp(PARTS[i]));
    if let Some(c) = condition_word(&hp, &max, p.dead) {
        out.push((c.to_string(), [0.95, 0.6, 0.3]));
    }
    let gone: Vec<&str> = PARTS.iter().enumerate().filter(|(i, _)| missing[*i]).map(|(_, p)| p.name()).collect();
    if !gone.is_empty() {
        out.push((format!("Missing the {}", gone.join(", ")), [0.95, 0.6, 0.3]));
    }
    if let Some(c) = w.carrying(pid) {
        out.push((format!("Carrying {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
    }
    if let Some(c) = w.carried_by(pid) {
        out.push((format!("Carried by {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
    } else if body::knocked_out(&hp) && !p.dead {
        out.push(("Select someone and click to carry them.".into(), DIM));
    }
    if let Some(b) = w.bond_of(pid) {
        let place = &w.settlements[b.town as usize].name;
        out.push((if b.slave { format!("Enslaved in {place}") } else { format!("Bound to work in {place}") }, [0.95, 0.6, 0.3]));
    }
    // What talking to them has turned up.
    if pid as usize >= w.society.minds.len() || p.bandit {
        // (Bandits and the like aren't in the town's lists.)
    } else {
        if let Some(why) = w.why_not_working(pid) {
            out.push((capital(&why), DIM));
        }
        if w.talk_said.iter().any(|s| s.0 == pid) {
            let day = gahturiyu_sim::sim::World::day_of(w.time) as i32;
            if let Some(g) = w.top_grudge(pid, day) {
                let who = match g.about {
                    gahturiyu_sim::sim::memory::Who::Person(q) => w.name_of(q),
                    gahturiyu_sim::sim::memory::Who::Ring(_) => "the ring".into(),
                    _ => "whoever it was".into(),
                };
                out.push((format!("Holds a grudge against {who}"), [0.95, 0.6, 0.3]));
            }
        }
    }
    if let Some(g) = w.group_of[pid as usize].and_then(|g| w.group(g)) {
        out.push((doing(w, g.id), DIM));
    } else if let Some(home) = p.home {
        match w.doing_now(pid) {
            Some((d, spot)) => out.push((format!("{} — {}", capital(d.word()), where_word(w, pid, spot, home)), GOLD)),
            None => out.push((format!("At home in {}", w.settlements[home as usize].name), DIM)),
        }
    }
}

fn life_lines(w: &World, pid: PersonId) -> Vec<(String, Rgb)> {
    let mut out = Vec::new();
    if pid as usize >= w.society.minds.len() || w.people[pid as usize].bandit {
        return out;
    }
    let l = w.life(pid);
    if let Some(why) = w.why_not_working(pid) {
        out.push((capital(&why), [0.95, 0.6, 0.3]));
    }
    let (need, v) = w.top_need(pid);
    let honour = match l.habits.honour {
        h if h < 0.35 => "little honour",
        h if h > 0.7 => "a strong sense of honour",
        _ => "an ordinary sense of honour",
    };
    let mind = if v > 0.25 { format!("Troubled by {} ({:.0}%)", need.name(), v * 100.0) } else { "Nothing much troubling them".to_string() };
    out.push((format!("{mind}  ·  {honour}"), DIM));
    // What the squad has learned by talking to them.
    let talked = w.talk_said.iter().any(|s| s.0 == pid);
    if talked {
        let day = gahturiyu_sim::sim::World::day_of(w.time) as i32;
        if let Some(g) = w.top_grudge(pid, day) {
            let who = match g.about {
                gahturiyu_sim::sim::memory::Who::Person(q) => w.name_of(q),
                gahturiyu_sim::sim::memory::Who::Ring(_) => "the ring".into(),
                _ => "whoever it was".into(),
            };
            out.push((format!("Holds a grudge against {who} ({})", g.deed.words().split(' ').next().unwrap_or("")), [0.95, 0.6, 0.3]));
        }
    }
    // The household.
    if let Some(h) = l.household {
        let hh = &w.society.households[h as usize];
        let pu = &hh.purse;
        let debt = pu.debt();
        let mut line = format!("Household of {}  ·  purse {:.0}", hh.members.iter().filter(|&&m| !w.people[m as usize].dead).count(), pu.coin);
        if debt > 0.0 {
            let to: Vec<String> = pu
                .debts
                .iter()
                .map(|d| match d.to {
                    gahturiyu_sim::sim::lives::Creditor::Household(o) => format!("{}'s household{}", w.society.households.get(o as usize).and_then(|x| x.members.first()).map(|&m| w.name_of(m)).unwrap_or_default(), if d.dodged { " (not paying)" } else { "" }),
                    gahturiyu_sim::sim::lives::Creditor::Merchants(_) => "the merchants".into(),
                    gahturiyu_sim::sim::lives::Creditor::Hall(_) => "the hall".into(),
                })
                .collect();
            line += &format!("  ·  owes {:.0} to {}", debt, to.join(", "));
        }
        let (n, nv) = w.household_need(h);
        if nv > 0.25 {
            line += &format!("  ·  needs: {}", n.name());
        }
        out.push((line, DIM));
        let feel: Vec<String> = hh
            .feelings
            .iter()
            .filter(|f| f.warmth.abs() > 0.15)
            .map(|f| {
                let head = w.society.households.get(f.other as usize).and_then(|x| x.members.first()).map(|&m| w.name_of(m)).unwrap_or_default();
                let stage = gahturiyu_sim::sim::memory::STAGES[f.stage as usize];
                format!("{} toward {head}'s{}", if f.warmth > 0.0 { "warm" } else { "cold" }, if stage.is_empty() { String::new() } else { format!(" ({stage})") })
            })
            .collect();
        if !feel.is_empty() {
            out.push((capital(&feel.join(", ")), DIM));
        }
    }
    out
}

fn work_lines(w: &World, pid: PersonId) -> Vec<(String, Rgb)> {
    use gahturiyu_sim::sim::jobs::Job;
    let l = w.life(pid);
    if l.job == Job::None {
        return vec![];
    }
    let mut line = if super::lexicon::native() { super::lexicon::thing(w, l.job.name()) } else { l.job.title(w.people[pid as usize].seed).to_string() };
    if l.job == Job::Guard && l.shift == 1 {
        line += " (night watch)";
    }
    if let Some(wp) = w.workplace_of(pid) {
        line += &format!("  ·  {}", wp.kind.name());
    }
    if l.job == Job::StoneTender {
        line += "  ·  rounds of the homes";
    }
    let mut extra = Vec::new();
    if let Some(r) = l.habits.own_rhythm.filter(|&r| w.community_of(pid).map(|c| c.customs.rhythm != r).unwrap_or(true)) {
        extra.push(format!("keeps {} whatever the town does", r.name().to_lowercase()));
    }
    if l.habits.lodger {
        extra.push("lodges with a host household".to_string());
    }
    if w.gardener_of(pid).is_some() {
        extra.push("tends the garden".to_string());
    }
    let mut out = vec![(line, TEXT)];
    if !extra.is_empty() {
        out.push((capital(&extra.join(", ")), DIM));
    }
    out
}

fn where_word(w: &World, pid: PersonId, spot: gahturiyu_sim::sim::routine::Spot, home: u16) -> String {
    use gahturiyu_sim::sim::routine::Spot;
    let town = &w.settlements[home as usize].name;
    match spot {
        Spot::Home => format!("home in {town}"),
        Spot::Work => w.workplace_of(pid).map(|p| format!("the {}", p.kind.name().to_lowercase())).unwrap_or_else(|| town.clone()),
        Spot::Place(i) => format!("the {}", w.society.towns[home as usize].places[i as usize].kind.name().to_lowercase()),
        Spot::Hearth => "by the hearth".into(),
        Spot::Visit(_) => "at a neighbour's".into(),
        Spot::Garden => "the garden".into(),
        Spot::Rounds => "going from home to home".into(),
        Spot::Boat => "on the dawn boat".into(),
        Spot::RunRound => "out to the fields and yards".into(),
    }
}
