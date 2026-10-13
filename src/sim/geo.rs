//! Positions, the map's size, and the coastline.
//!
//! The map is a square, `WORLD_SIZE` metres a side (about half of Kenshi).
//! x runs west to east, y runs north to south. The sea is on the west.

use serde::{Deserialize, Serialize};

/// One side of the map, in metres.
pub const WORLD_SIZE: f32 = 21_000.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub const fn new(x: f32, y: f32) -> V2 {
        V2 { x, y }
    }
    pub fn dist(self, o: V2) -> f32 {
        ((self.x - o.x).powi(2) + (self.y - o.y).powi(2)).sqrt()
    }
    pub fn lerp(self, o: V2, t: f32) -> V2 {
        V2::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
    pub fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
    pub fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
    pub fn scale(self, s: f32) -> V2 {
        V2::new(self.x * s, self.y * s)
    }
    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// The x position of the shoreline at a given y. Land lies east of it.
pub fn coast_x(y: f32) -> f32 {
    2300.0 + 550.0 * (y / 2700.0).sin() + 280.0 * (y / 950.0 + 1.3).sin() + 120.0 * (y / 310.0 + 0.4).sin()
}

pub fn is_land(p: V2) -> bool {
    inland(p) > 0.0 && p.x > 0.0 && p.x < WORLD_SIZE && p.y > 0.0 && p.y < WORLD_SIZE
}

/// How far inland a point sits, in metres (negative at sea): from the
/// mainland's shore, or Salt Isle's.
pub fn inland(p: V2) -> f32 {
    let main = p.x - coast_x(p.y);
    if main >= 0.0 {
        return main;
    }
    main.max(isle_inland(p))
}

// ---- Salt Isle (`island.rs`) -------------------------------------------------
//
// The tutorial island is fixed geometry like the coast: the same spot in
// every world, so the land and everything that asks "is this land?" know it
// without a seed.

/// The island's centre is this far down the map...
pub const ISLE_Y: f32 = 10_000.0;
/// ...and this far off the mainland's shore.
pub const ISLE_OFF: f32 = 1150.0;
/// Half the island's width (east-west) and height (north-south), roughly.
pub const ISLE_A: f32 = 420.0;
pub const ISLE_B: f32 = 260.0;

/// The middle of Salt Isle.
pub fn isle_centre() -> V2 {
    V2::new(coast_x(ISLE_Y) - ISLE_OFF, ISLE_Y)
}

/// How far inside Salt Isle's shore a point is, in metres (negative in the
/// sea round it; very negative far away). An ellipse with a fixed wobble, so
/// the shore isn't a perfect oval.
pub fn isle_inland(p: V2) -> f32 {
    let d = p.sub(isle_centre());
    if d.x.abs() > ISLE_A * 2.0 || d.y.abs() > ISLE_B * 2.0 {
        return -(d.len() - ISLE_A);
    }
    let ang = d.y.atan2(d.x);
    let wobble = 1.0 + 0.05 * (3.0 * ang + 1.0).cos() + 0.03 * (5.0 * ang).sin();
    let (a, b) = (ISLE_A * wobble, ISLE_B * wobble);
    let r = ((d.x / a).powi(2) + (d.y / b).powi(2)).sqrt();
    (1.0 - r) * b
}

/// Salt Isle's shoreline as `n` points round it (for the map).
pub fn isle_outline(n: usize) -> Vec<V2> {
    let c = isle_centre();
    (0..n)
        .map(|k| {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU;
            let wobble = 1.0 + 0.05 * (3.0 * ang + 1.0).cos() + 0.03 * (5.0 * ang).sin();
            c.add(V2::new(ang.cos() * ISLE_A * wobble, ang.sin() * ISLE_B * wobble))
        })
        .collect()
}

/// Inside the map's edges only (the sea is left to whoever walks into it).
pub fn clamp_to_bounds(p: V2, margin: f32) -> V2 {
    V2::new(p.x.clamp(margin, WORLD_SIZE - margin), p.y.clamp(margin, WORLD_SIZE - margin))
}

pub fn clamp_to_world(p: V2, margin: f32) -> V2 {
    let x = p.x.clamp(margin, WORLD_SIZE - margin).max(coast_x(p.y) + margin);
    V2::new(x, p.y.clamp(margin, WORLD_SIZE - margin))
}
