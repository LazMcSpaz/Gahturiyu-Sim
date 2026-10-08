//! Positions, the map's size, and the coastline.
//!
//! The map is a square, `WORLD_SIZE` metres a side (about half of Kenshi).
//! x runs west to east, y runs north to south. The sea is on the west.

/// One side of the map, in metres.
pub const WORLD_SIZE: f32 = 21_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
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
    p.x > coast_x(p.y) && p.x < WORLD_SIZE && p.y > 0.0 && p.y < WORLD_SIZE
}

/// How far inland a point sits, in metres (negative at sea).
pub fn inland(p: V2) -> f32 {
    p.x - coast_x(p.y)
}

pub fn clamp_to_world(p: V2, margin: f32) -> V2 {
    let x = p.x.clamp(margin, WORLD_SIZE - margin).max(coast_x(p.y) + margin);
    V2::new(x, p.y.clamp(margin, WORLD_SIZE - margin))
}
