//! The two cameras: an orbiting 3D camera over the squad, and the top-down
//! map's pan and zoom. Plus turning world points into screen points and
//! back, for hovering and clicking.

use bevy::math::{vec2, vec3, Mat4, Vec2, Vec3, Vec4};

use gahturiyu_sim::sim::{
    geo::{V2, WORLD_SIZE},
    terrain::Terrain,
};

/// Vertical field of view of the 3D camera.
pub const FOV_DEG: f32 = 50.0;

pub fn to3(p: V2, h: f32) -> Vec3 {
    vec3(p.x, h, p.y)
}

pub struct OrbitCam {
    /// The point the camera circles, on the ground (sim x, sim y).
    pub target: V2,
    /// Ground height under the target; kept up to date by the caller.
    pub ground: f32,
    pub yaw: f32,
    pub pitch: f32,
    /// Metres from target.
    pub dist: f32,
}

impl OrbitCam {
    pub fn new(target: V2) -> OrbitCam {
        OrbitCam { target, ground: 0.0, yaw: 0.35, pitch: 0.48, dist: 110.0 }
    }

    pub fn look_at(&self) -> Vec3 {
        vec3(self.target.x, self.ground, self.target.y)
    }

    pub fn eye(&self) -> Vec3 {
        self.look_at() + vec3(self.pitch.cos() * self.yaw.cos(), self.pitch.sin(), self.pitch.cos() * self.yaw.sin()) * self.dist
    }

    pub fn near(&self) -> f32 {
        (self.dist * 0.01).clamp(0.2, 5.0)
    }

    pub fn far(&self) -> f32 {
        self.far_radius() * 1.3 + self.dist
    }

    /// How far out the detailed world (buildings, people) is drawn.
    pub fn draw_radius(&self) -> f32 {
        (self.dist * 5.0).clamp(900.0, 5500.0)
    }

    /// How far out land of any kind is drawn.
    pub fn far_radius(&self) -> f32 {
        (self.draw_radius() * 3.0).max(9000.0)
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.006;
        self.pitch = (self.pitch + dy * 0.004).clamp(0.08, 1.50);
    }

    pub fn zoom(&mut self, wheel: f32) {
        self.dist = (self.dist * if wheel > 0.0 { 0.87 } else { 1.0 / 0.87 }).clamp(12.0, 4500.0);
    }

    /// Slide the target across the ground, relative to where the camera faces.
    pub fn pan(&mut self, right: f32, forward: f32) {
        let fwd = V2::new(-self.yaw.cos(), -self.yaw.sin());
        let side = V2::new(-fwd.y, fwd.x);
        let s = self.dist * 0.9;
        let p = self.target.add(fwd.scale(forward * s)).add(side.scale(-right * s));
        self.target = V2::new(p.x.clamp(0.0, WORLD_SIZE), p.y.clamp(0.0, WORLD_SIZE));
    }

    /// World to clip space for a screen of the given size.
    pub fn view_proj(&self, size: Vec2) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye(), self.look_at(), Vec3::Y);
        let proj = Mat4::perspective_rh(FOV_DEG.to_radians(), size.x / size.y.max(1.0), self.near(), self.far());
        proj * view
    }

    /// Screen position of a world point, or None if it is behind the camera.
    pub fn project(&self, vp: &Mat4, size: Vec2, p: Vec3) -> Option<Vec2> {
        let c = *vp * p.extend(1.0);
        if c.w <= 0.0 {
            return None;
        }
        let n = c.truncate() / c.w;
        if n.z > 1.0 {
            return None;
        }
        Some(vec2((n.x + 1.0) * 0.5 * size.x, (1.0 - n.y) * 0.5 * size.y))
    }

    /// The view ray through a screen point: origin and direction.
    pub fn ray(&self, size: Vec2, s: Vec2) -> (Vec3, Vec3) {
        let inv = self.view_proj(size).inverse();
        let nx = s.x / size.x * 2.0 - 1.0;
        let ny = 1.0 - s.y / size.y * 2.0;
        let a: Vec4 = inv * Vec4::new(nx, ny, -1.0, 1.0);
        let b: Vec4 = inv * Vec4::new(nx, ny, 1.0, 1.0);
        let (a, b) = (a.truncate() / a.w, b.truncate() / b.w);
        (a, (b - a).normalize())
    }

    /// The point on the land (or sea) under a screen position: march along
    /// the view ray until it dips below the surface, then home in.
    pub fn ground_at(&self, size: Vec2, s: Vec2, t: &Terrain) -> Option<V2> {
        let (a, dir) = self.ray(size, s);
        let above = |p: Vec3| p.y - t.surface(V2::new(p.x, p.z));
        let (mut lo, mut step) = (0.0f32, 2.0f32);
        for _ in 0..600 {
            let hi = lo + step;
            if above(a + dir * hi) < 0.0 {
                let (mut l, mut h) = (lo, hi);
                for _ in 0..20 {
                    let m = (l + h) * 0.5;
                    if above(a + dir * m) < 0.0 {
                        h = m;
                    } else {
                        l = m;
                    }
                }
                let q = a + dir * h;
                return Some(V2::new(q.x, q.z));
            }
            lo = hi;
            step *= 1.04;
            if lo > 30_000.0 {
                break;
            }
        }
        None
    }
}

pub struct MapCam {
    pub centre: V2,
    /// Pixels per metre.
    pub zoom: f32,
}

impl MapCam {
    pub fn to_screen(&self, size: Vec2, p: V2) -> Vec2 {
        vec2((p.x - self.centre.x) * self.zoom + size.x / 2.0, (p.y - self.centre.y) * self.zoom + size.y / 2.0)
    }
    pub fn to_world(&self, size: Vec2, s: Vec2) -> V2 {
        V2::new((s.x - size.x / 2.0) / self.zoom + self.centre.x, (s.y - size.y / 2.0) / self.zoom + self.centre.y)
    }
    pub fn zoom_at(&mut self, size: Vec2, mouse: Vec2, wheel: f32) {
        let before = self.to_world(size, mouse);
        self.zoom = (self.zoom * if wheel > 0.0 { 1.15 } else { 1.0 / 1.15 }).clamp(size.x / WORLD_SIZE / 1.2, 12.0);
        let after = self.to_world(size, mouse);
        self.centre = self.centre.add(before.sub(after));
    }
}
