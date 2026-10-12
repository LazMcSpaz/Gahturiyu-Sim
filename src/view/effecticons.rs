//! Small badges over the heads of anyone under a lasting magic effect
//! (Laz's P20; drawing only). Out of a fight they're the person's boons; in a
//! fight, their fighter's statuses. One badge per kind of effect, its rim red
//! for harm and brass for help, with a glyph for the family it belongs to.
//! Hovering a badge names the effect and how long it has left.

use bevy::math::{vec2, Vec2};

use gahturiyu_sim::sim::{
    effects::{Does, Effect, Lasts, Reach},
    person::PersonId,
    World,
};

use super::hud::Canvas;
use super::palette::{ega, eg, Rgb, BRASS, DIM, GOLD, TEXT};

/// Badge radius, pixels; and the most shown over one head.
const R: f32 = 7.0;
const MOST: usize = 6;

#[derive(Clone, Copy, PartialEq)]
enum Glyph {
    Flame,
    Drop,
    Frost,
    Shield,
    Up,
    Down,
    Mind,
    Hidden,
    Light,
    Dot,
}

fn glyph(d: Does) -> (Glyph, Rgb) {
    use Does::*;
    match d {
        Burning => (Glyph::Flame, [1.0, 0.55, 0.22]),
        Wet => (Glyph::Drop, [0.45, 0.70, 1.0]),
        Bleed => (Glyph::Drop, [0.80, 0.10, 0.08]),
        Chilled | Frozen => (Glyph::Frost, [0.80, 0.92, 1.0]),
        Barrier | Toughen | ResistElements | ResistParalysis | ResistBlind | DomainResist(_) | Veil => (Glyph::Shield, [0.95, 0.85, 0.55]),
        Paralyze | Calm | Fear | Dominate | Sway | Daze | Blind | KnockDown => (Glyph::Mind, [0.80, 0.60, 1.0]),
        Hide | Silent | Disguise | Gloom => (Glyph::Hidden, [0.62, 0.66, 0.80]),
        Glow | Nightsight | SenseLife => (Glyph::Light, [1.0, 0.95, 0.70]),
        d if d.harmful() => (Glyph::Down, [0.95, 0.42, 0.34]),
        Attr(_) | Skill(_) | Haste | MoveSpeed | MaxEnergy | EnergyRegen | Carry | Lighten | Sustain | DomainPower(_) => (Glyph::Up, [0.55, 0.92, 0.55]),
        _ => (Glyph::Dot, [0.80, 0.80, 0.80]),
    }
}

/// The lasting effects on someone now: what, how strong, until when.
pub fn effects_on(w: &World, pid: PersonId) -> Vec<(Does, f32, f64)> {
    let mut out: Vec<(Does, f32, f64)> = match w.fighter(pid) {
        Some(f) => f.statuses.iter().map(|s| (s.does, s.power, s.until)).collect(),
        None => w.boons_on(pid).iter().map(|b| (b.does, b.power, b.until)).collect(),
    };
    out.dedup_by(|a, b| a.0 == b.0);
    out.truncate(MOST);
    out
}

fn draw_glyph(c: &Canvas, g: Glyph, x: f32, y: f32, col: bevy_egui::egui::Color32) {
    let v = |dx: f32, dy: f32| vec2(x + dx, y + dy);
    match g {
        Glyph::Flame => c.poly(&[v(0.0, -4.5), v(3.0, 1.0), v(1.5, 3.8), v(-1.5, 3.8), v(-3.0, 1.0)], col),
        Glyph::Drop => {
            c.triangle(v(0.0, -4.5), v(2.6, 0.5), v(-2.6, 0.5), col);
            c.circle(x, y + 1.3, 2.7, col);
        }
        Glyph::Frost => {
            for k in 0..3 {
                let a = k as f32 * std::f32::consts::PI / 3.0;
                let d = vec2(a.cos(), a.sin()) * 4.0;
                c.line(vec2(x, y) - d, vec2(x, y) + d, 1.3, col);
            }
        }
        Glyph::Shield => c.poly(&[v(-3.5, -3.5), v(3.5, -3.5), v(3.0, 1.0), v(0.0, 4.2), v(-3.0, 1.0)], col),
        Glyph::Up => c.poly(&[v(0.0, -4.0), v(4.0, 1.5), v(-4.0, 1.5)], col),
        Glyph::Down => c.poly(&[v(0.0, 4.0), v(4.0, -1.5), v(-4.0, -1.5)], col),
        Glyph::Mind => {
            c.circle_lines(x, y, 3.4, 1.3, col);
            c.circle(x, y, 1.3, col);
        }
        Glyph::Hidden => c.circle_lines(x, y, 3.4, 1.2, col),
        Glyph::Light => {
            for k in 0..4 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                let d = vec2(a.cos(), a.sin()) * 4.2;
                c.line(vec2(x, y) - d, vec2(x, y) + d, 1.1, col);
            }
        }
        Glyph::Dot => c.circle(x, y, 2.0, col),
    }
}

/// Draw the badges over someone whose head is at `head` on screen (centred,
/// above their health bar). Returns the tooltip lines if the mouse is over one.
pub fn draw(c: &Canvas, w: &World, pid: PersonId, head: Vec2, mouse: Vec2) -> Option<Vec<(String, Rgb)>> {
    let list = effects_on(w, pid);
    if list.is_empty() {
        return None;
    }
    let gap = R * 2.0 + 3.0;
    let x0 = head.x - (list.len() as f32 - 1.0) * gap / 2.0;
    let y = head.y - 30.0;
    let mut tip = None;
    for (i, &(does, power, until)) in list.iter().enumerate() {
        let x = x0 + i as f32 * gap;
        let (g, col) = glyph(does);
        c.circle(x, y, R + 1.0, ega([0.0, 0.0, 0.0], 0.55));
        c.circle(x, y, R, ega([0.10, 0.08, 0.06], 0.9));
        c.circle_lines(x, y, R, 1.2, ega(if does.harmful() { [0.90, 0.32, 0.26] } else { BRASS }, 0.95));
        draw_glyph(c, g, x, y, eg(col));
        if mouse.distance(vec2(x, y)) <= R + 1.5 {
            let what = Effect { does, power, lasts: Lasts::Secs(0.0), reach: Reach::Caster }.describe();
            let mins = ((until - w.time) / 60.0).max(0.0);
            let left = if mins >= 90.0 { format!("{:.0} hours left", mins / 60.0) } else if mins >= 1.0 { format!("{mins:.0} minutes left") } else { "a moment left".to_string() };
            tip = Some(vec![(what, GOLD), (left, DIM), (if does.harmful() { "Harmful".into() } else { "Helpful".into() }, TEXT)]);
        }
    }
    tip
}
