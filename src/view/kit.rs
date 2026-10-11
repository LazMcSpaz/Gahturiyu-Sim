//! The shared feel of everything clickable (drawing only): a thing under the
//! mouse brightens, held down it sinks and darkens, let go it flashes once,
//! and something that can't be used looks it.
//!
//! Panels don't need to carry any of this around: `app.rs` tells the kit
//! where the mouse is and what the left button is doing once a frame
//! (`kit::input`), and any panel asks how one of its boxes should look
//! (`kit::look`) or has the kit draw a standard button or row.
//!
//! - `look(&bx)` → `Look { hot, held, flash }` for any box, to style your own.
//! - `button(c, bx, label, Kind::Primary | Kind::Plain, enabled)`: the gold
//!   DEAL / MAKE / TAKE ALL button, or a plain framed one.
//! - `row(c, bx, selected)`: the band behind a list row (hover, held, flash).
//! - `nudge(look)`: how far to push a pressed thing's contents down (px).

use std::cell::RefCell;

use bevy::math::{vec2, Vec2};
use bevy_egui::egui::Color32;

use super::hud::{Canvas, Face};
use super::palette::{ega, eg, BRASS, BRASS_DARK, BRASS_LIGHT};
use super::squadui::Bx;

/// How long a let-go flash lasts, in seconds.
pub const FLASH: f32 = 0.28;
/// How far a held button sinks, in pixels.
pub const SINK: f32 = 1.5;

#[derive(Clone, Copy, Default)]
struct Input {
    mouse: Vec2,
    /// Where the left button went down, while it's held.
    held_from: Option<Vec2>,
    /// Where the last click began, and when it was let go.
    released: Option<(Vec2, f32)>,
    /// Where it was let go.
    release_end: Vec2,
    now: f32,
}

thread_local! {
    static INPUT: RefCell<Input> = RefCell::new(Input::default());
}

/// Once a frame, from `app.rs`: the mouse, whether the left button went
/// down or was let go this frame, and the time. A let-go counts on a box
/// only if the press began on it too.
pub fn input(mouse: Vec2, pressed_now: bool, released_now: bool, now: f32) {
    INPUT.with(|i| {
        let mut i = i.borrow_mut();
        i.mouse = mouse;
        i.now = now;
        if pressed_now {
            i.held_from = Some(mouse);
        }
        if released_now {
            if let Some(p) = i.held_from.take() {
                // Where it began and ended; `look` wants both inside.
                i.released = Some((p, now));
                i.release_end = mouse;
            }
        }
    });
}

/// How a clickable box should look this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    /// The mouse is over it.
    pub hot: bool,
    /// The button went down on it and is still held, over it.
    pub held: bool,
    /// Let go on it a moment ago: 1 at the click, fading to 0.
    pub flash: f32,
}

pub fn look(b: &Bx) -> Look {
    INPUT.with(|i| {
        let i = i.borrow();
        let hot = b.contains(i.mouse);
        let held = hot && i.held_from.is_some_and(|p| b.contains(p));
        let flash = match i.released {
            Some((p, t)) if b.contains(p) && b.contains(i.release_end) && i.now - t < FLASH => 1.0 - (i.now - t) / FLASH,
            _ => 0.0,
        };
        Look { hot, held, flash }
    })
}

/// How far to push a held thing's contents down.
pub fn nudge(l: Look) -> f32 {
    if l.held {
        SINK
    } else {
        0.0
    }
}

/// The kinds of standard button.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // `Plain` waits for the screens that use it.
pub enum Kind {
    /// Gold, for the one thing a screen is for: Deal, Make, Take all.
    Primary,
    /// Dark with a brass edge: Clear, Close, and the rest.
    Plain,
}

/// Draw a standard button with its label (Cinzel capitals). Returns how it
/// looks this frame; the click itself is still the panel's to test.
pub fn button(c: &Canvas, b: &Bx, label: &str, kind: Kind, enabled: bool) -> Look {
    let l = if enabled { look(b) } else { Look::default() };
    let dy = nudge(l);
    let (x, y, w, h) = (b.x, b.y + dy, b.w, b.h);
    let size = (h * 0.42).clamp(12.0, 20.0);
    let spacing = size * 0.22;
    match (kind, enabled) {
        (Kind::Primary, true) => {
            let top = if l.held { eg(BRASS) } else if l.hot { eg([1.0, 0.90, 0.64]) } else { eg(BRASS_LIGHT) };
            let bottom = if l.held { eg(BRASS_DARK) } else { eg(BRASS) };
            if l.hot && !l.held {
                c.rect(x - 3.0, y - 3.0, w + 6.0, h + 6.0, ega([1.0, 0.85, 0.5], 0.18));
            }
            c.grad(x, y, w, h, top, bottom, false);
            c.rect_lines(x, y, w, h, 1.0, eg(BRASS_LIGHT));
            let tw = c.styled_width(label, size, Face::Title, spacing);
            c.styled(label, x + (w - tw) / 2.0, y + h / 2.0 + size * 0.36, size, Color32::from_rgb(27, 19, 12), Face::Title, spacing);
        }
        (Kind::Plain, true) => {
            let fill = if l.held { ega([0.05, 0.04, 0.03], 0.95) } else if l.hot { ega([0.20, 0.16, 0.11], 0.95) } else { ega([0.10, 0.08, 0.06], 0.85) };
            c.rect(x, y, w, h, fill);
            c.rect_lines(x, y, w, h, 1.0, ega(if l.hot { BRASS_LIGHT } else { BRASS }, if l.hot { 0.9 } else { 0.55 }));
            let tw = c.styled_width(label, size, Face::Title, spacing);
            c.styled(label, x + (w - tw) / 2.0, y + h / 2.0 + size * 0.36, size, ega(if l.hot { BRASS_LIGHT } else { [0.85, 0.80, 0.68] }, 1.0), Face::Title, spacing);
        }
        (_, false) => {
            c.rect(x, y, w, h, ega([0.08, 0.07, 0.05], 0.6));
            dashed(c, x, y, w, h, ega(BRASS_DARK, 0.8));
            let tw = c.styled_width(label, size, Face::Title, spacing);
            c.styled(label, x + (w - tw) / 2.0, y + h / 2.0 + size * 0.36, size, ega(BRASS, 0.4), Face::Title, spacing);
        }
    }
    flash(c, x, y, w, h, l.flash);
    l
}

/// The band behind a list row: brass tint under the mouse, darker while
/// held, a short glow when let go, and a steady band when `selected`.
pub fn row(c: &Canvas, b: &Bx, selected: bool) -> Look {
    let l = look(b);
    let k = if l.held {
        0.08
    } else if l.hot {
        0.16
    } else if selected {
        0.12
    } else {
        0.0
    };
    if k > 0.0 {
        c.grad(b.x, b.y, b.w, b.h, ega(BRASS, k * 1.6), ega(BRASS, k * 0.2), true);
        c.rule(b.x, b.x + b.w * 0.6, b.y, 0.8, ega(BRASS_LIGHT, k * 2.0));
    }
    if l.flash > 0.0 {
        c.rect(b.x, b.y, b.w, b.h, ega([1.0, 0.9, 0.6], 0.22 * l.flash));
    }
    l
}

/// A short bright wash over a box that was just let go on.
pub fn flash(c: &Canvas, x: f32, y: f32, w: f32, h: f32, k: f32) {
    if k > 0.0 {
        c.rect(x, y, w, h, ega([1.0, 0.95, 0.75], 0.35 * k));
        let grow = (1.0 - k) * 6.0;
        c.rect_lines(x - grow, y - grow, w + 2.0 * grow, h + 2.0 * grow, 1.5, ega(BRASS_LIGHT, 0.8 * k));
    }
}

/// A round button's ring and glow for its state (the bottom band's buttons):
/// draws the halo and pressed shading round a circle at (x, y).
pub fn round(c: &Canvas, x: f32, y: f32, r: f32, l: Look) {
    if l.hot && !l.held {
        c.circle(x, y, r + 6.0, ega([1.0, 0.85, 0.5], 0.14));
        c.circle_lines(x, y, r + 1.5, 1.6, ega(BRASS_LIGHT, 0.9));
    }
    if l.held {
        c.circle(x, y, r, ega([0.0, 0.0, 0.0], 0.35));
    }
    if l.flash > 0.0 {
        c.circle(x, y, r, ega([1.0, 0.95, 0.75], 0.35 * l.flash));
        c.circle_lines(x, y, r + (1.0 - l.flash) * 8.0, 1.6, ega(BRASS_LIGHT, 0.8 * l.flash));
    }
}

fn dashed(c: &Canvas, x: f32, y: f32, w: f32, h: f32, col: Color32) {
    let dash = 5.0;
    let mut t = 0.0;
    while t < w {
        let e = (t + dash).min(w);
        c.line(vec2(x + t, y), vec2(x + e, y), 1.0, col);
        c.line(vec2(x + t, y + h), vec2(x + e, y + h), 1.0, col);
        t += dash * 2.0;
    }
    let mut t = 0.0;
    while t < h {
        let e = (t + dash).min(h);
        c.line(vec2(x, y + t), vec2(x, y + e), 1.0, col);
        c.line(vec2(x + w, y + t), vec2(x + w, y + e), 1.0, col);
        t += dash * 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_held_and_a_flash_only_where_the_click_began_and_ended() {
        let b = Bx::new(10.0, 10.0, 40.0, 20.0);
        let inside = vec2(20.0, 15.0);
        input(inside, false, false, 0.0);
        assert_eq!(look(&b), Look { hot: true, held: false, flash: 0.0 });
        input(inside, true, false, 0.1);
        assert!(look(&b).held);
        input(inside, false, true, 0.2);
        let l = look(&b);
        assert!(!l.held && l.flash > 0.99);
        input(inside, false, false, 0.2 + FLASH * 0.5);
        assert!((look(&b).flash - 0.5).abs() < 0.01, "fades");
        input(inside, false, false, 0.2 + FLASH + 0.01);
        assert_eq!(look(&b).flash, 0.0, "gone");
        // A press dragged off and let go elsewhere: no flash, not held.
        input(inside, true, false, 1.0);
        input(vec2(200.0, 200.0), false, false, 1.1);
        assert!(!look(&b).held && !look(&b).hot);
        input(vec2(200.0, 200.0), false, true, 1.2);
        assert_eq!(look(&b).flash, 0.0);
    }
}
