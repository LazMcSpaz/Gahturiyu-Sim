//! A height grid over a patch of the world, with a second value per cell
//! (how rocky the ground is) and a colour per cell.

use gahturiyu_sim::sim::geo::V2;

pub struct Field {
    pub x0: f32,
    pub y0: f32,
    pub cell: f32,
    pub w: usize,
    pub h: usize,
    pub z: Vec<f32>,
    /// 0..1: bare rock showing through (crags, cliff faces).
    pub rock: Vec<f32>,
    /// Ground colour, filled in after the heights.
    pub albedo: Vec<[f32; 3]>,
}

impl Field {
    /// Fill the grid from `f(x, y) -> (height, rock)`, rows split across threads.
    pub fn build(x0: f32, y0: f32, cell: f32, w: usize, h: usize, f: impl Fn(f32, f32) -> (f32, f32) + Sync) -> Field {
        let mut z = vec![0.0f32; w * h];
        let mut rock = vec![0.0f32; w * h];
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).max(1);
        let rows = h.div_ceil(threads);
        std::thread::scope(|s| {
            for (k, (zc, rc)) in z.chunks_mut(rows * w).zip(rock.chunks_mut(rows * w)).enumerate() {
                let f = &f;
                s.spawn(move || {
                    for (n, (zv, rv)) in zc.iter_mut().zip(rc.iter_mut()).enumerate() {
                        let (i, j) = (n % w, k * rows + n / w);
                        let (a, b) = f(x0 + i as f32 * cell, y0 + j as f32 * cell);
                        *zv = a;
                        *rv = b;
                    }
                });
            }
        });
        Field { x0, y0, cell, w, h, z, rock, albedo: vec![[0.0; 3]; w * h] }
    }

    fn coords(&self, x: f32, y: f32) -> (usize, usize, f32, f32) {
        let fx = ((x - self.x0) / self.cell).clamp(0.0, (self.w - 2) as f32);
        let fy = ((y - self.y0) / self.cell).clamp(0.0, (self.h - 2) as f32);
        let (i, j) = (fx.floor() as usize, fy.floor() as usize);
        (i, j, fx - i as f32, fy - j as f32)
    }

    pub fn inside(&self, x: f32, y: f32) -> bool {
        x >= self.x0 && y >= self.y0 && x < self.x0 + (self.w - 1) as f32 * self.cell && y < self.y0 + (self.h - 1) as f32 * self.cell
    }

    pub fn at(&self, x: f32, y: f32) -> f32 {
        let (i, j, tx, ty) = self.coords(x, y);
        let k = j * self.w + i;
        let a = self.z[k] + (self.z[k + 1] - self.z[k]) * tx;
        let b = self.z[k + self.w] + (self.z[k + self.w + 1] - self.z[k + self.w]) * tx;
        a + (b - a) * ty
    }

    pub fn height(&self, p: V2) -> f32 {
        self.at(p.x, p.y)
    }

    pub fn colour(&self, x: f32, y: f32) -> [f32; 3] {
        let (i, j, tx, ty) = self.coords(x, y);
        let k = j * self.w + i;
        let mut out = [0.0; 3];
        for (c, o) in out.iter_mut().enumerate() {
            let a = self.albedo[k][c] + (self.albedo[k + 1][c] - self.albedo[k][c]) * tx;
            let b = self.albedo[k + self.w][c] + (self.albedo[k + self.w + 1][c] - self.albedo[k + self.w][c]) * tx;
            *o = a + (b - a) * ty;
        }
        out
    }

    /// Unit surface normal (x east, y south, z up).
    pub fn normal(&self, x: f32, y: f32) -> [f32; 3] {
        let e = self.cell;
        let dx = (self.at(x + e, y) - self.at(x - e, y)) / (2.0 * e);
        let dy = (self.at(x, y + e) - self.at(x, y - e)) / (2.0 * e);
        let n = [-dx, -dy, 1.0];
        let l = (n[0] * n[0] + n[1] * n[1] + 1.0).sqrt();
        [n[0] / l, n[1] / l, n[2] / l]
    }

    /// Slope in degrees.
    pub fn slope(&self, x: f32, y: f32) -> f32 {
        self.normal(x, y)[2].clamp(-1.0, 1.0).acos().to_degrees()
    }

    pub fn cell_pos(&self, k: usize) -> (f32, f32) {
        (self.x0 + (k % self.w) as f32 * self.cell, self.y0 + (k / self.w) as f32 * self.cell)
    }

    /// Paint a ribbon of `colour` along a line, `width` metres wide, `mix` strong.
    pub fn paint_line(&mut self, pts: &[V2], width: f32, colour: [f32; 3], mix: f32) {
        for s in pts.windows(2) {
            let (a, b) = (s[0], s[1]);
            let (lo_x, hi_x) = (a.x.min(b.x) - width, a.x.max(b.x) + width);
            let (lo_y, hi_y) = (a.y.min(b.y) - width, a.y.max(b.y) + width);
            let i0 = (((lo_x - self.x0) / self.cell).floor().max(0.0)) as usize;
            let i1 = (((hi_x - self.x0) / self.cell).ceil() as usize).min(self.w - 1);
            let j0 = (((lo_y - self.y0) / self.cell).floor().max(0.0)) as usize;
            let j1 = (((hi_y - self.y0) / self.cell).ceil() as usize).min(self.h - 1);
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let p = V2::new(self.x0 + i as f32 * self.cell, self.y0 + j as f32 * self.cell);
                    let d = seg_dist(p, a, b);
                    if d < width * 0.5 {
                        let k = j * self.w + i;
                        let t = mix * (1.0 - (d / (width * 0.5)).powi(4));
                        for c in 0..3 {
                            self.albedo[k][c] += (colour[c] - self.albedo[k][c]) * t;
                        }
                    }
                }
            }
        }
    }
}

pub fn seg_dist(p: V2, a: V2, b: V2) -> f32 {
    let ab = b.sub(a);
    let l2 = ab.x * ab.x + ab.y * ab.y;
    let t = if l2 > 0.0 { ((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2 } else { 0.0 };
    p.dist(a.add(ab.scale(t.clamp(0.0, 1.0))))
}
