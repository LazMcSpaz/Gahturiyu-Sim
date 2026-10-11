//! Panels slide a short way and fade in when they open, and fade out a
//! little faster when they close (the feel plan's #12). Drawing only.
//!
//! A panel is whatever is drawn between `begin` and `end` under one name.
//! Nothing about how a panel draws changes: the shapes it has just put on
//! the canvas are moved and faded afterwards. A panel that is no longer
//! drawn is shown from the last shapes it drew while it fades.
//!
//! Clicks are still tested against where a panel will come to rest, so a
//! click during the fifth of a second it takes to arrive lands where it
//! looks like it should. Screenshots show panels already arrived.

use std::collections::BTreeMap;

use bevy_egui::egui::{self, emath::TSTransform, epaint::ClippedShape, layers::ShapeIdx, Shape};

use super::hud::Canvas;

/// How long a panel takes to arrive and to go, seconds; how far it slides
/// up into place, pixels.
pub const OPEN_SECS: f32 = 0.18;
pub const CLOSE_SECS: f32 = 0.12;
pub const SLIDE: f32 = 12.0;

struct Panel {
    /// When it opened, and the last frame it was drawn in, and when.
    since: f32,
    frame: u64,
    seen: f32,
    /// What it drew then, where it rests.
    last: Vec<ClippedShape>,
}

#[derive(Default)]
pub struct Slides {
    now: f32,
    frame: u64,
    snap: bool,
    /// Screenshots (`GAHT_SLIDE`): every panel held this far arrived.
    held: Option<f32>,
    panels: BTreeMap<&'static str, Panel>,
}

/// How far arrived a panel is this long after opening, 0 to 1 (fast at
/// first, settling).
pub fn arrived(age: f32) -> f32 {
    let t = (age / OPEN_SECS).clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// How much is left of a panel this long after it was last drawn, 1 to 0.
pub fn left(age: f32) -> f32 {
    1.0 - (age / CLOSE_SECS).clamp(0.0, 1.0)
}

fn fade(shape: &mut Shape, by: f32) {
    egui::epaint::shape_transform::adjust_colors(shape, move |c| *c = c.gamma_multiply(by));
}

impl Slides {
    /// Once a frame, before any panel: the clock on. `snap` (screenshots)
    /// shows every panel arrived and none going.
    pub fn tick(&mut self, dt: f32, snap: bool) {
        self.now += dt;
        self.frame += 1;
        self.snap = snap;
    }

    /// For screenshots: hold every panel this far arrived, 0 to 1.
    pub fn hold(&mut self, t: f32) {
        self.held = Some(t.clamp(0.0, 1.0));
    }

    /// Before a panel draws.
    pub fn begin(&self, c: &Canvas) -> ShapeIdx {
        c.p.add(Shape::Noop)
    }

    /// After it has drawn (or not: a panel that drew nothing is closed).
    pub fn end(&mut self, c: &Canvas, name: &'static str, start: ShapeIdx) {
        let end = c.p.add(Shape::Noop);
        if end.0 <= start.0 + 1 {
            return;
        }
        let (now, frame) = (self.now, self.frame);
        let layer = c.p.layer_id();
        let last: Vec<ClippedShape> = c.p.ctx().graphics(|g| g.get(layer).map(|l| l.all_entries().skip(start.0 + 1).take(end.0 - start.0 - 1).cloned().collect()).unwrap_or_default());
        let p = self.panels.entry(name).or_insert(Panel { since: now, frame, seen: now, last: Vec::new() });
        // Not drawn last frame: it has just opened (again).
        if p.frame + 1 < frame {
            p.since = now;
        }
        p.frame = frame;
        p.seen = now;
        p.last = last;
        let t = self.held.unwrap_or(if self.snap { 1.0 } else { arrived(now - p.since) });
        if t < 1.0 {
            c.p.ctx().graphics_mut(|g| {
                let l = g.entry(layer);
                l.transform_range(start, end, TSTransform::from_translation(egui::vec2(0.0, SLIDE * (1.0 - t))));
                for i in start.0..end.0 {
                    l.mutate_shape(ShapeIdx(i), |cs| fade(&mut cs.shape, t));
                }
            });
        }
    }

    /// Once a frame, after every panel: those just closed fade where they
    /// were, sinking a little; those gone are forgotten.
    pub fn finish(&mut self, c: &Canvas) {
        let (now, frame, snap) = (self.now, self.frame, self.snap);
        self.panels.retain(|_, p| p.frame == frame || (!snap && now - p.seen < CLOSE_SECS));
        let layer = c.p.layer_id();
        for p in self.panels.values().filter(|p| p.frame != frame) {
            let k = left(now - p.seen);
            c.p.ctx().graphics_mut(|g| {
                let l = g.entry(layer);
                for cs in &p.last {
                    let mut s = cs.shape.clone();
                    s.translate(egui::vec2(0.0, SLIDE * 0.5 * (1.0 - k)));
                    fade(&mut s, k);
                    l.add(cs.clip_rect, s);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_egui::egui::Color32;

    /// One frame: the panel named "p" draws a white box at (10, 100) if
    /// `open`. Returns every box on the canvas after: (its top, its alpha).
    fn frame(ctx: &egui::Context, s: &mut Slides, dt: f32, snap: bool, open: bool) -> Vec<(f32, u8)> {
        let mut out = Vec::new();
        ctx.begin_pass(egui::RawInput::default());
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("canvas")));
        let c = Canvas { p, w: 800.0, h: 600.0 };
        s.tick(dt, snap);
        let at = s.begin(&c);
        if open {
            c.rect(10.0, 100.0, 50.0, 20.0, Color32::WHITE);
        }
        s.end(&c, "p", at);
        s.finish(&c);
        c.p.for_each_shape(|cs| {
            if let Shape::Rect(r) = &cs.shape {
                out.push((r.rect.top(), r.fill.a()));
            }
        });
        let _ = ctx.end_pass();
        out
    }

    #[test]
    fn a_panel_slides_up_and_fades_in_then_rests() {
        let ctx = egui::Context::default();
        let mut s = Slides::default();
        // The frame it opens: low and unseen.
        let b = frame(&ctx, &mut s, 1.0 / 60.0, false, true);
        assert_eq!(b.len(), 1);
        assert!((b[0].0 - (100.0 + SLIDE)).abs() < 0.01 && b[0].1 == 0, "{b:?}");
        // Part-way: on its way up, part seen.
        let b = frame(&ctx, &mut s, OPEN_SECS / 2.0, false, true);
        assert!(b[0].0 > 100.0 && b[0].0 < 100.0 + SLIDE && b[0].1 > 0 && b[0].1 < 255, "{b:?}");
        // Arrived: exactly where it draws itself, untouched.
        let b = frame(&ctx, &mut s, OPEN_SECS, false, true);
        assert_eq!(b, vec![(100.0, 255)]);
    }

    #[test]
    fn a_closed_panel_fades_out_faster_and_is_gone() {
        let ctx = egui::Context::default();
        let mut s = Slides::default();
        frame(&ctx, &mut s, 1.0 / 60.0, false, true);
        frame(&ctx, &mut s, 1.0, false, true);
        // No longer drawn: still there, fainter and a little lower.
        let b = frame(&ctx, &mut s, CLOSE_SECS / 2.0, false, false);
        assert_eq!(b.len(), 1);
        assert!(b[0].0 > 100.0 && b[0].0 < 100.0 + SLIDE && b[0].1 > 0 && b[0].1 < 255, "{b:?}");
        // And then gone.
        assert!(frame(&ctx, &mut s, CLOSE_SECS, false, false).is_empty());
        assert!(CLOSE_SECS < OPEN_SECS);
        // Opened again, it comes in from the start.
        let b = frame(&ctx, &mut s, 1.0, false, true);
        assert!(b[0].1 == 0, "{b:?}");
    }

    #[test]
    fn screenshots_show_panels_arrived_and_none_going() {
        let ctx = egui::Context::default();
        let mut s = Slides::default();
        assert_eq!(frame(&ctx, &mut s, 1.0 / 30.0, true, true), vec![(100.0, 255)]);
        assert!(frame(&ctx, &mut s, 1.0 / 30.0, true, false).is_empty());
    }
}
