//! A small mesh builder with the lighting baked into vertex colours.
//!
//! macroquad draws 3D shapes unlit, so a cube is a flat silhouette. Instead,
//! every face is shaded once, as it is built, by how squarely it faces the sun,
//! and faded toward the sky colour with distance from the camera. Cheap, and
//! enough to read form.

use macroquad::models::Vertex;
use macroquad::prelude::*;

/// Indices are 16-bit, so a mesh is cut before it reaches this many vertices.
const MAX_VERTS: usize = 40_000;

pub struct Builder {
    done: Vec<Mesh>,
    cur: Mesh,
    sun: Vec3,
    eye: Vec3,
    fog: (f32, f32, Color),
}

fn empty() -> Mesh {
    Mesh { vertices: Vec::with_capacity(4096), indices: Vec::with_capacity(8192), texture: None }
}

impl Builder {
    pub fn new(eye: Vec3, fog_start: f32, fog_end: f32, sky: Color) -> Builder {
        Builder { done: Vec::new(), cur: empty(), sun: vec3(-0.45, 0.80, -0.35).normalize(), eye, fog: (fog_start, fog_end, sky) }
    }

    pub fn draw(&self) {
        for m in &self.done {
            draw_mesh(m);
        }
        if !self.cur.indices.is_empty() {
            draw_mesh(&self.cur);
        }
    }

    pub fn finish(mut self) -> Builder {
        if !self.cur.indices.is_empty() {
            let m = std::mem::replace(&mut self.cur, empty());
            self.done.push(m);
        }
        self
    }

    fn room(&mut self, n: usize) {
        if self.cur.vertices.len() + n > MAX_VERTS {
            let m = std::mem::replace(&mut self.cur, empty());
            self.done.push(m);
        }
    }

    /// Lit and fogged colour for a surface facing `n` at `p`.
    pub fn shade(&self, c: Color, n: Vec3, p: Vec3) -> Color {
        let lit = 0.40 + 0.60 * n.dot(self.sun).max(0.0) + 0.08 * n.y.max(0.0);
        self.fogged(Color::new(c.r * lit, c.g * lit, c.b * lit, c.a), p)
    }

    /// Fog only — for things that glow.
    pub fn fogged(&self, c: Color, p: Vec3) -> Color {
        let (a, b, sky) = self.fog;
        let t = ((p.distance(self.eye) - a) / (b - a)).clamp(0.0, 1.0);
        Color::new(c.r + (sky.r - c.r) * t, c.g + (sky.g - c.g) * t, c.b + (sky.b - c.b) * t, c.a)
    }

    fn v(&mut self, p: Vec3, c: Color) -> u16 {
        let i = self.cur.vertices.len() as u16;
        self.cur.vertices.push(Vertex::new(p.x, p.y, p.z, 0.0, 0.0, c));
        i
    }

    /// A quad with a colour given per corner, unlit (already shaded by the caller).
    pub fn quad_raw(&mut self, p: [Vec3; 4], c: [Color; 4]) {
        self.room(4);
        let i: Vec<u16> = (0..4).map(|k| self.v(p[k], c[k])).collect();
        self.cur.indices.extend_from_slice(&[i[0], i[1], i[2], i[0], i[2], i[3]]);
    }

    fn face(&mut self, pts: &[Vec3], n: Vec3, col: Color) {
        self.room(pts.len());
        let s = self.shade(col, n, pts[0]);
        let base = self.cur.vertices.len() as u16;
        for &p in pts {
            self.v(p, s);
        }
        for k in 1..pts.len() as u16 - 1 {
            self.cur.indices.extend_from_slice(&[base, base + k, base + k + 1]);
        }
    }

    /// A box sitting on `base`, turned `rot` radians about the vertical.
    pub fn block(&mut self, base: Vec3, w: f32, d: f32, h: f32, rot: f32, col: Color) {
        let (s, c) = rot.sin_cos();
        let ax = vec3(c, 0.0, s) * (w * 0.5);
        let az = vec3(-s, 0.0, c) * (d * 0.5);
        let up = vec3(0.0, h, 0.0);
        let p = [base - ax - az, base + ax - az, base + ax + az, base - ax + az];
        for k in 0..4 {
            let (a, b) = (p[k], p[(k + 1) % 4]);
            let n = (b - a).cross(up).normalize_or_zero();
            let n = if n.dot((a + b) * 0.5 - base) < 0.0 { -n } else { n };
            self.face(&[a, b, b + up, a + up], n, col);
        }
        self.face(&[p[0] + up, p[1] + up, p[2] + up, p[3] + up], Vec3::Y, col);
    }

    /// An upright cylinder or cone frustum standing on `base`.
    pub fn column(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, sides: usize, col: Color) {
        let ring = |r: f32, y: f32, k: usize| {
            let a = k as f32 / sides as f32 * std::f32::consts::TAU;
            base + vec3(a.cos() * r, y, a.sin() * r)
        };
        let slope = (r0 - r1) / h.max(0.001);
        for k in 0..sides {
            let mid = (k as f32 + 0.5) / sides as f32 * std::f32::consts::TAU;
            let n = vec3(mid.cos(), slope, mid.sin()).normalize();
            let (a, b) = (ring(r0, 0.0, k), ring(r0, 0.0, k + 1));
            let (c, d) = (ring(r1, h, k + 1), ring(r1, h, k));
            self.face(&[a, b, c, d], n, col);
        }
        if r1 > 0.01 {
            let top: Vec<Vec3> = (0..sides).map(|k| ring(r1, h, k)).collect();
            self.face(&top, Vec3::Y, col);
        }
    }

    /// A smooth dome (a squashed half-ball) on `base`. `lump` roughens the
    /// surface, seeded, for grown stone. `bands` darkens alternating rings.
    #[allow(clippy::too_many_arguments)]
    pub fn dome(&mut self, base: Vec3, rx: f32, rz: f32, h: f32, lump: f32, bands: f32, seed: u64, col: Color) {
        let (rings, sides) = (6usize, 14usize);
        let noise = |i: usize, k: usize| -> f32 {
            let x = (seed ^ ((i as u64) << 20) ^ ((k % sides) as u64)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            ((x >> 40) as f32 / (1u64 << 24) as f32) - 0.5
        };
        let point = |i: usize, k: usize| -> (Vec3, Vec3) {
            let lat = i as f32 / rings as f32 * std::f32::consts::FRAC_PI_2;
            let lon = k as f32 / sides as f32 * std::f32::consts::TAU;
            let bump = if i == rings { 1.0 } else { 1.0 + noise(i, k) * lump };
            let p = base + vec3(lat.cos() * lon.cos() * rx * bump, lat.sin() * h, lat.cos() * lon.sin() * rz * bump);
            let n = vec3(lat.cos() * lon.cos() / rx, lat.sin() / h, lat.cos() * lon.sin() / rz).normalize();
            (p, n)
        };
        for i in 0..rings {
            let band = if bands > 0.0 && i % 2 == 1 { 1.0 - bands } else { 1.0 };
            let c = Color::new(col.r * band, col.g * band, col.b * band, col.a);
            for k in 0..sides {
                let q = [point(i, k), point(i, k + 1), point(i + 1, k + 1), point(i + 1, k)];
                let cols = [0, 1, 2, 3].map(|j| self.shade(c, q[j].1, q[j].0));
                self.quad_raw([q[0].0, q[1].0, q[2].0, q[3].0], cols);
            }
        }
    }

    /// A small glowing patch (a lit window, embers): fogged but not shaded.
    pub fn glow(&mut self, centre: Vec3, w: f32, h: f32, facing: f32, col: Color) {
        let (s, c) = facing.sin_cos();
        let right = vec3(-s, 0.0, c) * (w * 0.5);
        let up = vec3(0.0, h * 0.5, 0.0);
        let k = self.fogged(col, centre);
        self.quad_raw([centre - right - up, centre + right - up, centre + right + up, centre - right + up], [k; 4]);
    }
}
