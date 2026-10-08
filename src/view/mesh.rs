//! A small mesh builder: boxes, columns, domes and flat strips, with a colour
//! per vertex. Lighting is done by the renderer (sun, moon, fires), so the
//! builder only records shape, facing and colour.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::{vec3, Vec3};

use super::palette::{lin, Rgb};

#[derive(Default)]
pub struct Builder {
    pos: Vec<[f32; 3]>,
    nrm: Vec<[f32; 3]>,
    col: Vec<[f32; 4]>,
    idx: Vec<u32>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn triangles(&self) -> usize {
        self.idx.len() / 3
    }

    pub fn is_empty(&self) -> bool {
        self.idx.is_empty()
    }

    /// The finished mesh. An empty builder gives one invisible sliver, so the
    /// renderer always has something to hold.
    pub fn mesh(mut self) -> Mesh {
        if self.idx.is_empty() {
            for _ in 0..3 {
                self.pos.push([0.0, -1e4, 0.0]);
                self.nrm.push([0.0, 1.0, 0.0]);
                self.col.push([0.0; 4]);
            }
            self.idx.extend_from_slice(&[0, 1, 2]);
        }
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nrm)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.col)
            .with_inserted_indices(Indices::U32(self.idx))
    }

    fn v(&mut self, p: Vec3, n: Vec3, c: [f32; 4]) -> u32 {
        let i = self.pos.len() as u32;
        self.pos.push(p.to_array());
        self.nrm.push(n.to_array());
        self.col.push(c);
        i
    }

    /// A quad with a facing and a colour per corner (colours already linear).
    pub fn quad_lin(&mut self, p: [Vec3; 4], n: [Vec3; 4], c: [[f32; 4]; 4]) {
        let i: Vec<u32> = (0..4).map(|k| self.v(p[k], n[k], c[k])).collect();
        self.idx.extend_from_slice(&[i[0], i[1], i[2], i[0], i[2], i[3]]);
    }

    /// A flat quad of one colour facing `n`.
    pub fn quad(&mut self, p: [Vec3; 4], n: Vec3, c: Rgb) {
        let c = lin(c);
        self.quad_lin(p, [n; 4], [c; 4]);
    }

    /// A convex polygon, a colour and facing per corner.
    pub fn poly_lin(&mut self, p: &[Vec3], n: &[Vec3], c: &[[f32; 4]]) {
        if p.len() < 3 {
            return;
        }
        let base = self.pos.len() as u32;
        for k in 0..p.len() {
            self.v(p[k], n[k], c[k]);
        }
        for k in 1..p.len() as u32 - 1 {
            self.idx.extend_from_slice(&[base, base + k, base + k + 1]);
        }
    }

    fn face(&mut self, pts: &[Vec3], n: Vec3, col: [f32; 4]) {
        let base = self.pos.len() as u32;
        for &p in pts {
            self.v(p, n, col);
        }
        for k in 1..pts.len() as u32 - 1 {
            self.idx.extend_from_slice(&[base, base + k, base + k + 1]);
        }
    }

    /// A box sitting on `base`, turned `rot` radians about the vertical.
    pub fn block(&mut self, base: Vec3, w: f32, d: f32, h: f32, rot: f32, col: Rgb) {
        let col = lin(col);
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

    /// A box between two points (a stick: a shaft, a pole), `t` thick.
    pub fn stick(&mut self, a: Vec3, b: Vec3, t: f32, col: Rgb) {
        let col = lin(col);
        let d = (b - a).normalize_or_zero();
        let side = if d.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let u = d.cross(side).normalize_or_zero() * (t * 0.5);
        let v = d.cross(u).normalize_or_zero() * (t * 0.5);
        let ring = [u + v, u - v, -u - v, -u + v];
        for k in 0..4 {
            let (r0, r1) = (ring[k], ring[(k + 1) % 4]);
            let n = (r0 + r1).normalize_or_zero();
            self.face(&[a + r0, a + r1, b + r1, b + r0], n, col);
        }
    }

    /// An upright cylinder or cone frustum standing on `base`.
    pub fn column(&mut self, base: Vec3, r0: f32, r1: f32, h: f32, sides: usize, col: Rgb) {
        let col = lin(col);
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
    pub fn dome(&mut self, base: Vec3, rx: f32, rz: f32, h: f32, lump: f32, bands: f32, seed: u64, col: Rgb) {
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
            let c = lin([col[0] * band, col[1] * band, col[2] * band]);
            for k in 0..sides {
                let q = [point(i, k), point(i, k + 1), point(i + 1, k + 1), point(i + 1, k)];
                self.quad_lin([q[0].0, q[1].0, q[2].0, q[3].0], [q[0].1, q[1].1, q[2].1, q[3].1], [c; 4]);
            }
        }
    }

    /// A small upright patch (a lit window, embers), turned to face `facing`.
    pub fn patch(&mut self, centre: Vec3, w: f32, h: f32, facing: f32, col: Rgb) {
        let (s, c) = facing.sin_cos();
        let right = vec3(-s, 0.0, c) * (w * 0.5);
        let up = vec3(0.0, h * 0.5, 0.0);
        let n = vec3(c, 0.0, s);
        self.quad([centre - right - up, centre + right - up, centre + right + up, centre - right + up], n, col);
    }

    /// Append another builder's geometry.
    pub fn append(&mut self, o: Builder) {
        let base = self.pos.len() as u32;
        self.pos.extend(o.pos);
        self.nrm.extend(o.nrm);
        self.col.extend(o.col);
        self.idx.extend(o.idx.into_iter().map(|i| i + base));
    }
}
