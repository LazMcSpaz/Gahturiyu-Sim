//! The HUD's frame, after Laz's mockup (2026-10-10): dark umber and brass,
//! serif type.
//!
//! - top left, the squad: a portrait, name, state and two bars per member
//!   (click to select, Shift adds, right-click for the pack);
//! - top right, the job being tracked and the latest news;
//! - top centre, the name of the place you're in;
//! - along the bottom, a band of round buttons (pack, craft, spells,
//!   journal, town, build, map, keys), a raised plate for the lead member
//!   (portrait, health, stamina, load, a body showing each part's wounds,
//!   orders), the day and the hour, the speed, and a little map.
//!
//! Drawing only, immediate-mode like the rest: clicks come back as
//! `FrameAct`s for the app to carry out.

use bevy::math::{vec2, Vec2};
use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    body::{self, Part},
    geo::V2,
    person::PersonId,
    quests::{compass, QuestKind, Stage},
    race::Race,
    world::{DAY, HOUR},
    World,
};

use super::hud::{bar_for, Canvas, Face};
use super::palette::{self, ega, race_color, Rgb, BAR_HEALTH, BAR_LOW, BAR_STAMINA, BRASS, BRASS_DARK, BRASS_LIGHT, DIM, TEXT};
use super::squadui::{Bx, Click, Selection};

/// How far up from the bottom of the window the frame reaches: panels that
/// sit low keep above it.
pub const BOTTOM_CLEAR: f32 = 168.0;
/// The squad list's width.
pub const LIST_W: f32 = 336.0;

/// The round buttons along the bottom.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Button {
    Pack,
    Craft,
    Spells,
    Journal,
    Town,
    Build,
    Map,
    Keys,
}

const BUTTONS: [(Button, &str); 8] = [
    (Button::Pack, "Pack"),
    (Button::Craft, "Craft"),
    (Button::Spells, "Spells"),
    (Button::Journal, "Journal"),
    (Button::Town, "Town"),
    (Button::Build, "Build"),
    (Button::Map, "Map"),
    (Button::Keys, "Keys"),
];

pub enum FrameAct {
    Select(PersonId, bool),
    Pack(PersonId),
    Toggle(Button),
    Sneak,
    Rest,
    Speed(usize),
    Pause,
    Collapse,
}

/// What the frame needs to know about the window's state.
pub struct FrameState<'a> {
    pub sel: &'a Selection,
    pub speed_i: usize,
    pub paused: bool,
    pub collapsed: bool,
    /// Buttons whose panels are open (drawn lit).
    pub open: Vec<Button>,
}

impl FrameState<'_> {
    /// Is a panel open in the top right (where the tracked job goes)?
    pub fn right_busy(&self) -> bool {
        self.open.iter().any(|b| matches!(b, Button::Pack | Button::Craft | Button::Spells | Button::Town | Button::Build))
    }
}

fn a(c: Rgb, alpha: f32) -> Color32 {
    ega(c, alpha)
}

const UMBER: Rgb = [0.07, 0.065, 0.05];
const PLATE: Rgb = [0.27, 0.235, 0.17];
const PLATE_DARK: Rgb = [0.15, 0.13, 0.09];
const ORANGE: Rgb = [0.93, 0.47, 0.26];

/// A bar with a dark track, a thin brass edge and diamond ends.
fn bar(c: &Canvas, x: f32, y: f32, w: f32, h: f32, frac: f32, col: Rgb, big_end: bool) {
    c.rect(x, y - h / 2.0, w, h, a(UMBER, 0.85));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        c.grad(x, y - h / 2.0, w * f, h, a(palette::scale(col, 0.75), 1.0), a(col, 1.0), false);
    }
    c.rect_lines(x, y - h / 2.0 - 0.5, w, h + 1.0, 0.8, a(BRASS_DARK, 0.9));
    c.diamond(x, y, h * 0.75, a(BRASS, 1.0));
    let r = if big_end { h * 1.15 } else { h * 0.75 };
    c.diamond(x + w, y, r, a(BRASS_LIGHT, 1.0));
}

fn health_col(vit: f32, down: bool) -> Rgb {
    if down || vit < 0.3 {
        BAR_LOW
    } else {
        palette::mix(BAR_STAMINA, BAR_HEALTH, (vit - 0.3) / 0.4)
    }
}

/// A member's portrait: a little painted bust in a brass ring.
pub fn portrait(c: &Canvas, x: f32, y: f32, r: f32, race: Race, seed: u64, down: bool, ring: f32) {
    c.circle(x, y, r + ring + 2.0, a([0.05, 0.045, 0.035], 0.9));
    c.circle(x, y, r + ring, a(BRASS, 1.0));
    c.circle(x, y, r, a(palette::scale(race_color(race), 0.22), 1.0));
    // Light from above.
    c.circle(x, y - r * 0.25, r * 0.78, a(palette::scale(race_color(race), 0.32), 0.7));
    // Shoulders: the bottom of an ellipse, kept inside the ring.
    let robe = palette::scale(race_color(race), 0.55);
    let mut pts = Vec::new();
    for i in 0..=24 {
        let t = std::f32::consts::PI * (1.0 + i as f32 / 24.0);
        let p = vec2(x + t.cos() * r * 0.78, y + r * 0.98 + t.sin() * r * 0.5);
        let d = p - vec2(x, y);
        pts.push(if d.length() > r * 0.97 { vec2(x, y) + d.normalize() * r * 0.97 } else { p });
    }
    for i in (0..=12).rev() {
        let t = i as f32 / 12.0 * std::f32::consts::PI;
        pts.push(vec2(x + t.cos() * r * 0.97 * 0.8, y + t.sin().abs() * r * 0.97 * 0.6 + r * 0.36));
    }
    c.poly(&pts, a(robe, 1.0));
    // The head, with a band across the brow and eyes.
    let skin = palette::skin(race);
    let hy = y - r * 0.12;
    let hr = r * 0.38;
    c.circle(x, y + r * 0.3, r * 0.15, a(palette::scale(skin, 0.8), 1.0));
    c.circle(x, hy, hr, a(skin, 1.0));
    let band = if seed % 3 == 0 { palette::scale(race_color(race), 0.7) } else { palette::scale(skin, 0.6) };
    c.rect(x - hr * 0.95, hy - hr * 0.42, hr * 1.9, hr * 0.26, a(band, 0.9));
    let eye = if race == Race::Horaro { r * 0.07 } else { r * 0.045 };
    c.circle(x - hr * 0.38, hy + hr * 0.02, eye, a([0.08, 0.07, 0.06], 1.0));
    c.circle(x + hr * 0.38, hy + hr * 0.02, eye, a([0.08, 0.07, 0.06], 1.0));
    if down {
        c.circle(x, y, r, a([0.45, 0.08, 0.05], 0.45));
    }
    c.circle_lines(x, y, r, 1.0, a(BRASS_DARK, 1.0));
}

// ---- The squad ------------------------------------------------------------------

pub fn squad_list(c: &Canvas, w: &World, st: &FrameState, click: Option<Click>) -> (Option<FrameAct>, Vec<Bx>) {
    let mut act = None;
    let mut boxes = Vec::new();
    let n = w.squad.members.len().max(1);
    let wide = if st.collapsed { 100.0 } else { LIST_W };
    // The heading.
    let hy = 40.0;
    c.diamond(30.0, hy, 4.5, a(BRASS_LIGHT, 1.0));
    if !st.collapsed {
        c.styled("SQUAD", 44.0, hy + 5.0, 13.0, a(TEXT, 0.9), Face::Title, 3.0);
        c.rule_in(120.0, 290.0, hy, 1.2, a(BRASS, 0.9), false);
    }
    let bx = if st.collapsed { 70.0 } else { 312.0 };
    c.circle(bx, hy, 11.0, a(BRASS_DARK, 1.0));
    c.circle_lines(bx, hy, 11.0, 1.5, a(BRASS, 1.0));
    c.styled_centred(if st.collapsed { "›" } else { "‹" }, bx, hy + 5.0, 16.0, a(BRASS_LIGHT, 1.0), Face::Body, 0.0);
    let toggle = Bx::new(bx - 12.0, hy - 12.0, 24.0, 24.0);
    boxes.push(toggle);
    if click.is_some_and(|ck| toggle.contains(ck.at)) {
        act = Some(FrameAct::Collapse);
    }
    let top = 62.0;
    let row_h = 72.0f32.min((c.h - BOTTOM_CLEAR - top - 10.0) / n as f32).max(40.0);
    // A shade behind, so the names read over a bright view.
    let tall = top + row_h * n as f32 + 20.0;
    let clear = a(UMBER, 0.0);
    c.grad4(0.0, 0.0, wide + 60.0, tall, a(UMBER, 0.62), clear, clear, a(UMBER, 0.62));
    c.grad4(0.0, tall, wide + 60.0, 70.0, a(UMBER, 0.62), clear, clear, clear);
    for (k, &pid) in w.squad.members.iter().enumerate() {
        let y0 = top + k as f32 * row_h;
        let cy = y0 + row_h / 2.0;
        let p = &w.people[pid as usize];
        let chosen = st.sel.shows(w, pid);
        if chosen {
            c.grad(0.0, y0 + 3.0, wide, row_h - 6.0, a(BRASS, 0.32), a(BRASS, 0.0), true);
            c.rule_in(0.0, wide, y0 + 3.0, 1.0, a(BRASS_LIGHT, 0.55), false);
            c.rule_in(0.0, wide, y0 + row_h - 3.0, 1.0, a(BRASS_LIGHT, 0.55), false);
        }
        let (vit, mana, down) = bar_for(w, pid).unwrap_or((1.0, None, false));
        let pr = (row_h * 0.4).min(28.0);
        portrait(c, 58.0, cy, pr, p.race, p.seed, down, if chosen { 3.0 } else { 2.0 });
        if !st.collapsed {
            let name = p.name().unwrap_or("?");
            let nx = 100.0;
            let nw = c.styled(name, nx, cy - 8.0, 18.0, a(if chosen { BRASS_LIGHT } else { TEXT }, 1.0), Face::Caps, 0.5);
            let (word, col) = super::squadui::status(w, pid, k);
            let held = match w.fighter(pid) {
                Some(f) => f.held,
                None => w.held_ritual(pid),
            };
            let (word, col) = match (held, w.fresh_level_up(pid)) {
                (Some(s), _) => (format!("Holding {}", s.def().name.to_lowercase()), super::squadui::RITUAL),
                (None, Some((what, v))) if word == "Standing" || word == "Walking" => (format!("{what} {v} ↑"), BRASS_LIGHT),
                // Still sneaking keeps its full colour: it halves their pace,
                // and it's easy to leave on.
                _ if word == "Sneaking" || word == "Crouched" => (word.to_string(), col),
                _ => (word.to_string(), palette::mix(col, DIM, 0.35)),
            };
            // Strayed from the others: say how far, in the warning colour.
            let (word, col) = match w.strayed(pid) {
                Some(d) if !down => (format!("{word} · {d:.0} m off"), palette::WARN),
                _ => (word, col),
            };
            let sw = c.styled(&word, nx + nw + 9.0, cy - 8.0, 13.0, a(col, 1.0), Face::Italic, 0.0);
            // Being noticed: an eye that opens; a lit torch: a flame.
            let mut ix = nx + nw + sw + 20.0;
            let sus = w.suspicion_of(pid);
            if sus > 0.02 && !down && !w.fighting.contains_key(&pid) {
                let col = if sus >= 1.0 { a([0.95, 0.3, 0.25], 1.0) } else { a([0.95, 0.8, 0.35], 1.0) };
                c.ellipse_lines(ix, cy - 13.0, 7.0, 1.0 + 3.5 * sus.min(1.0), 1.4, col);
                c.circle(ix, cy - 13.0, 1.5 + 1.2 * sus.min(1.0), col);
                ix += 18.0;
            }
            if w.torch_lit(pid) {
                c.poly(&[vec2(ix, cy - 20.0), vec2(ix + 4.0, cy - 11.0), vec2(ix - 4.0, cy - 11.0)], a(palette::EMBER, 1.0));
            }
            let bw = LIST_W - 22.0 - (nx - 4.0);
            bar(c, nx - 4.0, cy + 6.0, bw, 7.0, vit, health_col(vit, down), true);
            let second = mana.or_else(|| w.stamina_of(pid)).unwrap_or(1.0);
            bar(c, nx - 4.0, cy + 18.0, bw, 4.0, second, if mana.is_some() { palette::MANA } else { BAR_STAMINA }, false);
        }
        let r = Bx::new(0.0, y0, wide, row_h);
        boxes.push(r);
        if let Some(ck) = click {
            if r.contains(ck.at) && act.is_none() {
                act = Some(if ck.right { FrameAct::Pack(pid) } else { FrameAct::Select(pid, ck.shift) });
            }
        }
    }
    (act, boxes)
}

// ---- What's tracked, and the news ------------------------------------------------

/// The job worth following: a title, what to do, and where.
fn tracked_job(w: &World) -> Option<(String, String, Option<V2>)> {
    let q = w.quests.iter().find(|q| q.stage != Stage::Done)?;
    let giver = w.people[q.giver as usize].name().unwrap_or("someone").to_string();
    let town = w.people[q.giver as usize].home.map(|h| w.settlements[h as usize].name.clone()).unwrap_or_default();
    let at_giver = Some(w.person_pos(q.giver));
    Some(match (&q.kind, q.stage) {
        (_, Stage::Report) => ("Report back".into(), format!("{giver} in {town} is waiting to hear"), at_giver),
        (QuestKind::ClearCamp { at, .. }, _) => ("The bandit camp".into(), format!("Break it, for {giver} of {town}"), Some(*at)),
        (QuestKind::Fetch { item, count }, _) => {
            let name = gahturiyu_sim::sim::items::item(*item).name.to_lowercase();
            (format!("{count} × {name}"), format!("For {giver} in {town}: you have {}", w.squad_count(*item)), at_giver)
        }
        (QuestKind::Deliver { to }, _) => {
            let p = &w.people[*to as usize];
            let place = p.home.map(|h| w.settlements[h as usize].name.clone()).unwrap_or_default();
            ("A sealed letter".into(), format!("Take it to {} in {place}", p.name().unwrap_or("someone")), Some(w.person_pos(*to)))
        }
        (QuestKind::Job { .. }, _) => {
            let line = w.quest_line(q);
            (format!("A job for {giver}"), line.trim_end_matches('.').to_string(), at_giver)
        }
    })
}

/// Recent news for the corner: the log, less the passers-by.
fn news(w: &World) -> Vec<(String, bool)> {
    let quiet = [", bound for", "crosses your path", "comes into view", "wandering", "heading home", "improves:"];
    w.log
        .iter()
        .filter(|(t, l)| w.time - t < 3.0 * HOUR && !quiet.iter().any(|q| l.contains(q)))
        .take(4)
        .map(|(_, l)| {
            let bad = ["attack", "Beaten", "dies", "died", "theft", "Theft", "stole", "on you", "arrest", "bounty", "robbed", "falls", "bandits"].iter().any(|k| l.contains(k));
            (l.trim_end_matches('.').to_string(), bad)
        })
        .collect()
}

fn wrap(s: &str, at: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    for word in s.split(' ') {
        let line = out.last_mut().unwrap();
        if !line.is_empty() && line.chars().count() + word.chars().count() + 1 > at {
            out.push(word.to_string());
        } else {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    out
}

pub fn tracked(c: &Canvas, w: &World) -> Option<Bx> {
    let xr = c.w - 34.0;
    let mut y = 40.0;
    let job = tracked_job(w);
    let news = news(w);
    if job.is_none() && news.is_empty() {
        return None;
    }
    let tall = 60.0 + if job.is_some() { 110.0 } else { 0.0 } + news.len() as f32 * 30.0;
    let clear = a(UMBER, 0.0);
    c.grad4(c.w - 460.0, 0.0, 460.0, tall, clear, a(UMBER, 0.6), a(UMBER, 0.6), clear);
    c.grad4(c.w - 460.0, tall, 460.0, 60.0, clear, a(UMBER, 0.6), clear, clear);
    if let Some((title, what, at)) = &job {
        c.rule_in(c.w - 380.0, c.w - 150.0, y, 1.2, a(BRASS, 0.9), true);
        c.styled_right("TRACKED", xr - 18.0, y + 5.0, 12.0, a(TEXT, 0.85), Face::Title, 3.0);
        c.diamond(xr, y, 4.5, a(BRASS_LIGHT, 1.0));
        y += 34.0;
        c.styled_right(&title.to_uppercase(), xr, y, 20.0, a(TEXT, 1.0), Face::Title, 1.0);
        for l in wrap(what, 44) {
            y += 25.0;
            c.styled_right(&l, xr, y, 17.0, a(TEXT, 0.92), Face::Body, 0.0);
        }
        if let Some(p) = at {
            let d = p.dist(w.squad.pos);
            let dir = compass(p.sub(w.squad.pos));
            let how = if d < 30.0 { "here".to_string() } else if d < 1000.0 { format!("{:.0} m {dir}", d) } else { format!("{:.1} km {dir}", d / 1000.0) };
            y += 23.0;
            c.styled_right(&how, xr, y, 14.0, a(DIM, 1.0), Face::Italic, 0.0);
        }
        y += 14.0;
    }
    for (l, bad) in &news {
        for (i, part) in wrap(l, 46).iter().enumerate() {
            y += 24.0;
            c.styled_right(part, xr - 18.0, y, 15.0, a(TEXT, 0.88), Face::Body, 0.0);
            if i == 0 {
                c.diamond(xr, y - 5.0, 4.0, a(if *bad { ORANGE } else { BRASS_LIGHT }, 1.0));
            }
        }
    }
    Some(Bx::new(c.w - 400.0, 24.0, 380.0, y - 10.0))
}

/// The place the squad is in, across the top.
pub fn banner(c: &Canvas, w: &World) {
    let here = w.squad.pos;
    let name = w
        .settlements
        .iter()
        .filter(|s| s.pos.dist(here) < s.radius() + 150.0)
        .min_by(|x, y| x.pos.dist(here).total_cmp(&y.pos.dist(here)))
        .map(|s| super::lexicon::town(w, s.id))
        .or_else(|| w.ruins.iter().find(|r| r.found && r.pos.dist(here) < 200.0).map(|r| r.name(w)));
    if let Some(n) = name {
        c.styled_centred(&n.to_uppercase(), c.w / 2.0, 118.0, 21.0, a(TEXT, 0.95), Face::Title, 6.0);
    }
}

// ---- The bottom -----------------------------------------------------------------

fn icon(c: &Canvas, b: Button, x: f32, y: f32, col: Color32) {
    let v = |dx: f32, dy: f32| vec2(x + dx, y + dy);
    match b {
        Button::Pack => {
            c.poly(&[v(-6.0, -2.0), v(6.0, -2.0), v(7.0, 8.0), v(-7.0, 8.0)], col);
            c.arc(x, y - 3.0, 4.0, std::f32::consts::PI, std::f32::consts::TAU, 1.6, col);
            c.line(v(-6.0, 2.0), v(6.0, 2.0), 1.2, Color32::from_black_alpha(160));
        }
        Button::Craft => {
            c.line(v(-6.0, 7.0), v(4.0, -3.0), 2.4, col);
            c.poly(&[v(0.0, -8.0), v(8.0, 0.0), v(5.0, 3.0), v(-3.0, -5.0)], col);
        }
        Button::Spells => {
            c.poly(&[v(0.0, -9.0), v(2.2, -2.2), v(9.0, 0.0), v(2.2, 2.2), v(0.0, 9.0), v(-2.2, 2.2), v(-9.0, 0.0), v(-2.2, -2.2)], col);
        }
        Button::Journal => {
            c.rect_lines(x - 6.0, y - 8.0, 12.0, 16.0, 1.6, col);
            c.line(v(-3.0, -8.0), v(-3.0, 8.0), 1.4, col);
            for k in 0..3 {
                c.line(v(-1.0, -4.0 + k as f32 * 4.0), v(4.0, -4.0 + k as f32 * 4.0), 1.0, col);
            }
        }
        Button::Town => {
            c.poly(&[v(-8.0, -1.0), v(0.0, -9.0), v(8.0, -1.0)], col);
            c.rect_lines(x - 6.0, y - 1.0, 12.0, 9.0, 1.6, col);
            c.rect(x - 1.5, y + 3.0, 3.0, 5.0, col);
        }
        Button::Build => {
            for (row, n) in [(-6.0, 2), (-1.0, 3), (4.0, 2)] {
                for k in 0..n {
                    let off = if n == 3 { -9.0 } else { -6.5 };
                    c.rect_lines(x + off + k as f32 * 6.0, y + row, 5.5, 4.5, 1.2, col);
                }
            }
        }
        Button::Map => {
            let pts = [v(-9.0, -6.0), v(-3.0, -8.0), v(3.0, -6.0), v(9.0, -8.0), v(9.0, 6.0), v(3.0, 8.0), v(-3.0, 6.0), v(-9.0, 8.0)];
            c.poly_lines(&pts, 1.5, col, true);
            c.line(v(-3.0, -8.0), v(-3.0, 6.0), 1.0, col);
            c.line(v(3.0, -6.0), v(3.0, 8.0), 1.0, col);
        }
        Button::Keys => {
            c.styled_centred("?", x, y + 7.0, 20.0, col, Face::Title, 0.0);
        }
    }
}

/// A pill-shaped order button.
fn pill(c: &Canvas, x: f32, y: f32, w: f32, h: f32, label: &str, on: bool, eye: bool) {
    let r = h / 2.0;
    let fill = if on { a(BRASS, 0.55) } else { a([0.11, 0.10, 0.075], 0.95) };
    c.rect(x + r, y, w - h, h, fill);
    c.circle(x + r, y + r, r, fill);
    c.circle(x + w - r, y + r, r, fill);
    let edge = a(if on { BRASS_LIGHT } else { BRASS }, 1.0);
    c.arc(x + r, y + r, r, std::f32::consts::FRAC_PI_2, 3.0 * std::f32::consts::FRAC_PI_2, 1.3, edge);
    c.arc(x + w - r, y + r, r, -std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2, 1.3, edge);
    c.line(vec2(x + r, y), vec2(x + w - r, y), 1.3, edge);
    c.line(vec2(x + r, y + h), vec2(x + w - r, y + h), 1.3, edge);
    // The icon in a small ring.
    let (ix, iy) = (x + r, y + r);
    c.circle_lines(ix, iy, r - 4.0, 1.2, edge);
    if eye {
        c.ellipse_lines(ix, iy, r * 0.45, r * 0.25, 1.2, edge);
        c.circle(ix, iy, 2.0, edge);
    } else {
        c.circle(ix, iy, r * 0.32, edge);
        c.circle(ix + 3.0, iy - 2.0, r * 0.28, fill);
    }
    c.styled(label, x + h + 6.0, y + r + 5.0, 13.0, a(TEXT, 0.95), Face::Title, 2.0);
}

/// One part of the little body, coloured by how it's holding up.
fn part_col(hp: f32, max: f32, lost: bool) -> Color32 {
    if lost {
        return a([0.12, 0.1, 0.08], 0.9);
    }
    let f = (hp / max.max(1.0)).clamp(-1.0, 1.0);
    let col = if f <= 0.0 { BAR_LOW } else { palette::mix(BAR_LOW, palette::mix(BAR_STAMINA, BAR_HEALTH, (f - 0.4) / 0.4), f / 0.4) };
    a(col, 1.0)
}

fn figure(c: &Canvas, w: &World, pid: PersonId, x: f32, y: f32) {
    let p = &w.people[pid as usize];
    let (hp, missing) = match w.fighter(pid) {
        Some(f) => (f.hp, f.missing),
        None => (p.wounds.hp_at(&p.stats, w.time), p.wounds.missing),
    };
    let col = |part: Part| {
        let i = part as usize;
        part_col(hp[i], p.stats.max_hp(part), missing[i])
    };
    c.circle(x, y - 22.0, 5.5, col(Part::Head));
    c.rect(x - 6.0, y - 15.0, 12.0, 16.0, col(Part::Torso));
    c.rect(x - 11.0, y - 14.0, 4.0, 15.0, col(Part::LeftArm));
    c.rect(x + 7.0, y - 14.0, 4.0, 15.0, col(Part::RightArm));
    c.rect(x - 6.0, y + 2.0, 5.0, 19.0, col(Part::LeftLeg));
    c.rect(x + 1.0, y + 2.0, 5.0, 19.0, col(Part::RightLeg));
    let _ = body::PARTS;
}

pub fn bottom(c: &Canvas, w: &World, st: &FrameState, click: Option<Click>) -> (Option<FrameAct>, Vec<Bx>) {
    let mut act = None;
    let mut boxes = Vec::new();
    let clicked = |b: &Bx| click.filter(|ck| !ck.right && b.contains(ck.at)).is_some();
    // The band along the bottom.
    let band = 86.0;
    let by = c.h - band;
    c.grad(0.0, by, c.w, band, a([0.30, 0.27, 0.20], 0.96), a([0.17, 0.15, 0.11], 0.98), false);
    c.rule(-40.0, c.w + 40.0, by, 1.6, a(BRASS, 1.0));
    c.rule(0.0, c.w, by + 4.0, 0.8, a(BRASS_DARK, 0.9));
    boxes.push(Bx::new(0.0, by, c.w, band));

    // Round buttons on the left.
    for (i, (b, name)) in BUTTONS.iter().enumerate() {
        let x = 42.0 + i as f32 * 56.0;
        let y = c.h - 52.0;
        let on = st.open.contains(b);
        c.circle(x, y, 21.0, a([0.05, 0.045, 0.035], 0.85));
        c.circle(x, y, 19.0, a(if on { BRASS_DARK } else { [0.17, 0.15, 0.11] }, 1.0));
        c.circle_lines(x, y, 19.0, 2.0, a(if on { BRASS_LIGHT } else { BRASS }, 1.0));
        c.circle_lines(x, y, 15.5, 0.8, a(BRASS_DARK, 1.0));
        icon(c, *b, x, y, a(if on { BRASS_LIGHT } else { BRASS }, 1.0));
        c.styled_centred(&name.to_uppercase(), x, c.h - 14.0, 10.0, a(TEXT, 0.85), Face::Title, 1.2);
        let r = Bx::new(x - 22.0, y - 22.0, 44.0, 58.0);
        if clicked(&r) {
            act = Some(FrameAct::Toggle(*b));
        }
    }

    // The raised plate for the lead member.
    let (px0, px1) = (c.w * 0.28, c.w * 0.71);
    let top = c.h - 152.0;
    let plate = [vec2(px0, c.h), vec2(px0 + 44.0, top), vec2(px1 - 44.0, top), vec2(px1, c.h)];
    c.poly(&plate, a(PLATE, 0.98));
    c.grad(px0 + 44.0, top, px1 - px0 - 88.0, 60.0, a([0.36, 0.32, 0.23], 0.6), a(PLATE, 0.0), false);
    c.poly_lines(&plate, 2.0, a(BRASS, 1.0), false);
    let inset = [vec2(px0 + 9.0, c.h), vec2(px0 + 49.0, top + 7.0), vec2(px1 - 49.0, top + 7.0), vec2(px1 - 9.0, c.h)];
    c.poly_lines(&inset, 1.0, a(BRASS_DARK, 1.0), false);
    boxes.push(Bx::new(px0, top, px1 - px0, c.h - top));
    let lead = st.sel.lead(w).or_else(|| w.squad.members.first().copied());
    if let Some(pid) = lead {
        let p = &w.people[pid as usize];
        let k = w.squad.index(pid).unwrap_or(0);
        let (vit, _, down) = bar_for(w, pid).unwrap_or((1.0, None, false));
        let (cx, cy) = (px0 + 112.0, c.h - 78.0);
        c.circle(cx, cy, 58.0, a(PLATE_DARK, 1.0));
        portrait(c, cx, cy, 50.0, p.race, p.seed, down, 3.5);
        let nx = px0 + 182.0;
        let name = p.name().unwrap_or("?");
        let nw = c.styled(name, nx, c.h - 112.0, 22.0, a(TEXT, 1.0), Face::Caps, 0.6);
        let (word, _) = super::squadui::status(w, pid, k);
        c.styled(&format!("{} · {}", p.race.name(), word.to_lowercase()), nx + nw + 12.0, c.h - 112.0, 14.0, a(DIM, 1.0), Face::Italic, 0.0);
        let load = w.load_of(pid);
        let rows = [("Health", vit, health_col(vit, down)), ("Stamina", w.stamina_of(pid).unwrap_or(1.0), BAR_STAMINA), ("Load", load.min(1.0), if load > 1.0 { BAR_LOW } else { [0.80, 0.78, 0.70] })];
        for (i, (label, f, col)) in rows.iter().enumerate() {
            let y = c.h - 84.0 + i as f32 * 21.0;
            c.styled(&label.to_uppercase(), nx, y + 4.0, 11.0, a(TEXT, 0.75), Face::Title, 1.2);
            bar(c, nx + 70.0, y, 170.0, 6.0, *f, *col, i == 0);
        }
        figure(c, w, pid, px0 + 450.0, c.h - 70.0);
        c.line(vec2(px0 + 484.0, c.h - 122.0), vec2(px0 + 484.0, c.h - 26.0), 1.0, a(BRASS_DARK, 1.0));
        let ox = px0 + 500.0;
        c.styled("ORDERS", ox, c.h - 118.0, 10.0, a(TEXT, 0.75), Face::Title, 2.5);
        let who = st.sel.who(w);
        let sneaking = !who.is_empty() && who.iter().all(|&m| w.is_sneaking(m));
        // Anyone resting: the button gets them up.
        let resting = w.any_resting(&who);
        let s = Bx::new(ox, c.h - 106.0, 112.0, 32.0);
        pill(c, s.x, s.y, s.w, s.h, "SNEAK", sneaking, true);
        let r = Bx::new(ox, c.h - 66.0, 112.0, 32.0);
        pill(c, r.x, r.y, r.w, r.h, if resting { "WAKE" } else { "REST" }, resting, false);
        if clicked(&s) {
            act = Some(FrameAct::Sneak);
        }
        if clicked(&r) {
            act = Some(FrameAct::Rest);
        }
    }

    // The day and the hour: the sun (or moon) on its arc.
    let (ax, ay) = (c.w * 0.75, c.h - 40.0);
    c.arc(ax, ay, 44.0, std::f32::consts::PI, std::f32::consts::TAU, 1.4, a(BRASS, 0.9));
    let tod = w.time.rem_euclid(DAY) / HOUR;
    let day = tod >= 6.0 && tod < 18.0;
    let f = if day { (tod - 6.0) / 12.0 } else { ((tod - 18.0).rem_euclid(24.0)) / 12.0 } as f32;
    let ang = std::f32::consts::PI * (1.0 + f);
    let (sx, sy) = (ax + ang.cos() * 44.0, ay + ang.sin() * 44.0);
    if day {
        c.circle(sx, sy, 10.0, a(BRASS_LIGHT, 0.25));
        c.circle(sx, sy, 7.0, a(BRASS_LIGHT, 1.0));
    } else {
        c.circle(sx, sy, 6.5, a([0.85, 0.87, 0.92], 1.0));
        c.circle(sx + 3.0, sy - 2.0, 5.5, a([0.20, 0.18, 0.13], 1.0));
    }
    let dayn = (w.time / DAY).floor() as i64 + 1;
    c.styled_centred(&format!("DAY {dayn}"), ax, ay - 2.0, 15.0, a(TEXT, 1.0), Face::Title, 1.5);
    c.styled_centred(&super::hud::hhmm(w.time), ax, ay + 16.0, 12.0, a(DIM, 1.0), Face::Body, 0.5);

    // Where, and how fast.
    let lx = c.w * 0.82;
    let place = w
        .settlements
        .iter()
        .min_by(|x, y| x.pos.dist(w.squad.pos).total_cmp(&y.pos.dist(w.squad.pos)))
        .filter(|s| s.pos.dist(w.squad.pos) < s.radius() + 600.0)
        .map(|s| super::lexicon::town(w, s.id).to_uppercase())
        .unwrap_or_else(|| "THE WILDS".into());
    c.styled_centred(&place, lx, c.h - 54.0, 12.0, a(TEXT, 0.85), Face::Title, 2.0);
    let row = c.h - 30.0;
    let px = lx - 58.0;
    c.circle(px, row, 11.0, a(if st.paused { BRASS } else { BRASS_DARK }, 1.0));
    c.circle_lines(px, row, 11.0, 1.4, a(BRASS_LIGHT, 1.0));
    c.rect(px - 4.0, row - 5.0, 2.5, 10.0, a(if st.paused { PLATE_DARK } else { BRASS_LIGHT }, 1.0));
    c.rect(px + 1.5, row - 5.0, 2.5, 10.0, a(if st.paused { PLATE_DARK } else { BRASS_LIGHT }, 1.0));
    let pb = Bx::new(px - 12.0, row - 12.0, 24.0, 24.0);
    if clicked(&pb) {
        act = Some(FrameAct::Pause);
    }
    for i in 0..5 {
        let x = px + 28.0 + i as f32 * 24.0;
        if !st.paused && i <= st.speed_i {
            c.diamond(x, row, 6.0, a(BRASS_LIGHT, 1.0));
        } else {
            c.diamond(x, row, 6.0, a(PLATE_DARK, 1.0));
            c.diamond_lines(x, row, 6.0, 1.2, a(BRASS, 1.0));
        }
        let b = Bx::new(x - 11.0, row - 11.0, 22.0, 22.0);
        if clicked(&b) {
            act = Some(FrameAct::Speed(i));
        }
    }

    // The little map, north up.
    let (mx, my, mr) = (c.w - 112.0, c.h - 112.0, 86.0);
    let mbox = Bx::new(mx - mr - 10.0, my - mr - 22.0, (mr + 10.0) * 2.0, (mr + 10.0) * 2.0 + 12.0);
    boxes.push(mbox);
    minimap(c, w, mx, my, mr, &tracked_job(w).and_then(|j| j.2));
    if clicked(&mbox) {
        act = Some(FrameAct::Toggle(Button::Map));
    }
    (act, boxes)
}

/// Metres per pixel on the little map.
const MINI_SCALE: f32 = 2.2;

fn minimap(c: &Canvas, w: &World, mx: f32, my: f32, mr: f32, target: &Option<V2>) {
    let here = w.squad.pos;
    let to = |p: V2| vec2(mx + (p.x - here.x) / MINI_SCALE, my + (p.y - here.y) / MINI_SCALE);
    let inside = |q: Vec2| (q - vec2(mx, my)).length() < mr - 1.0;
    c.circle(mx, my, mr + 12.0, a([0.05, 0.045, 0.035], 0.9));
    c.circle(mx, my, mr + 9.0, a(BRASS_DARK, 1.0));
    c.circle_lines(mx, my, mr + 6.0, 3.0, a(BRASS, 1.0));
    c.circle(mx, my, mr, a([0.36, 0.34, 0.28], 1.0));
    c.circle(mx, my - mr * 0.2, mr * 0.8, a([0.42, 0.40, 0.33], 0.35));
    // Roads, a few pieces per stretch so only what's inside shows.
    for road in &w.routes.roads {
        for seg in road.windows(2) {
            let (p0, p1) = (to(seg[0]), to(seg[1]));
            if (p0 - vec2(mx, my)).length() > mr * 6.0 && (p1 - vec2(mx, my)).length() > mr * 6.0 && p0.distance(p1) < mr * 4.0 {
                continue;
            }
            let n = ((p0.distance(p1) / 6.0).ceil() as usize).clamp(1, 400);
            for i in 0..n {
                let qa = p0.lerp(p1, i as f32 / n as f32);
                let qb = p0.lerp(p1, (i + 1) as f32 / n as f32);
                if inside(qa) && inside(qb) {
                    c.line(qa, qb, 4.0, a([0.20, 0.18, 0.14], 0.9));
                }
            }
        }
    }
    // Buildings.
    for s in w.settlements.iter().filter(|s| s.pos.dist(here) < s.radius() + mr * MINI_SCALE + 100.0) {
        for b in &s.buildings {
            let q = to(b.pos);
            let half = (b.size * 0.5 / MINI_SCALE).max(2.0);
            if (q - vec2(mx, my)).length() > mr - half * 1.5 {
                continue;
            }
            let (cs, sn) = (b.rot.cos(), b.rot.sin());
            let corner = |dx: f32, dy: f32| q + vec2(dx * cs - dy * sn, dx * sn + dy * cs);
            c.poly(&[corner(-half, -half), corner(half, -half), corner(half, half), corner(-half, half)], a([0.80, 0.78, 0.72], 0.95));
        }
    }
    // The squad.
    for (k, &m) in w.squad.members.iter().enumerate() {
        let q = to(w.squad.at[k]);
        if inside(q) && k > 0 {
            c.circle(q.x, q.y, 3.0, a(race_color(w.people[m as usize].race), 1.0));
        }
    }
    c.circle(mx, my, 6.0, a([0.95, 0.93, 0.88], 1.0));
    c.circle_lines(mx, my, 6.0, 2.0, a([0.12, 0.1, 0.08], 1.0));
    // Where the tracked job is: on the rim if it's further off.
    if let Some(t) = target {
        let q = to(*t);
        let d = q - vec2(mx, my);
        let q = if d.length() > mr - 6.0 { vec2(mx, my) + d.normalize() * (mr + 6.0) } else { q };
        c.diamond(q.x, q.y, 5.5, a(BRASS_LIGHT, 1.0));
        c.diamond_lines(q.x, q.y, 5.5, 1.0, a(PLATE_DARK, 1.0));
    }
    // North.
    let (nx, ny) = (mx, my - mr - 8.0);
    c.circle(nx, ny, 11.0, a(BRASS, 1.0));
    c.circle_lines(nx, ny, 11.0, 1.2, a(BRASS_LIGHT, 1.0));
    c.styled_centred("N", nx, ny + 5.0, 13.0, a(PLATE_DARK, 1.0), Face::Title, 0.0);
}

/// Every key, for the Keys button.
pub fn keys_panel(c: &Canvas, map: bool) -> Bx {
    let help = if map { super::app::HELP_MAP } else { super::app::HELP_3D };
    let mut lines: Vec<(String, Rgb)> = vec![("Keys and clicks".into(), palette::GOLD)];
    lines.extend(help.split("   ").map(|s| (s.trim().to_string(), TEXT)));
    let r = c.panel(&lines, c.w / 2.0 - 220.0, 110.0, 15.0);
    Bx::from(r)
}
