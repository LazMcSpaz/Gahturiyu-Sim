//! A plain RGB picture with the few drawing tools the review images need:
//! lines, discs, boxes and text.

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};

pub type Rgb = [f32; 3];

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Rgb>,
}

static FONT: &[u8] = include_bytes!("../../../assets/DejaVuSans.ttf");

pub fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

impl Canvas {
    pub fn new(w: usize, h: usize, fill: Rgb) -> Canvas {
        Canvas { w, h, px: vec![fill; w * h] }
    }

    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, a: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h || a <= 0.0 {
            return;
        }
        let p = &mut self.px[y as usize * self.w + x as usize];
        let a = a.min(1.0);
        for k in 0..3 {
            p[k] += (c[k] - p[k]) * a;
        }
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgb, a: f32) {
        for j in y.floor() as i32..(y + h).ceil() as i32 {
            for i in x.floor() as i32..(x + w).ceil() as i32 {
                self.blend(i, j, c, a);
            }
        }
    }

    /// A filled disc with a soft edge.
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, c: Rgb, a: f32) {
        let (x0, x1) = ((cx - r - 1.0).floor() as i32, (cx + r + 1.0).ceil() as i32);
        let (y0, y1) = ((cy - r - 1.0).floor() as i32, (cy + r + 1.0).ceil() as i32);
        for j in y0..=y1 {
            for i in x0..=x1 {
                let d = ((i as f32 + 0.5 - cx).powi(2) + (j as f32 + 0.5 - cy).powi(2)).sqrt();
                self.blend(i, j, c, a * (r + 0.5 - d).clamp(0.0, 1.0));
            }
        }
    }

    /// A line `w` pixels thick with soft edges.
    pub fn line(&mut self, ax: f32, ay: f32, bx: f32, by: f32, w: f32, c: Rgb, a: f32) {
        let r = w * 0.5;
        let (x0, x1) = ((ax.min(bx) - r - 1.0).floor() as i32, (ax.max(bx) + r + 1.0).ceil() as i32);
        let (y0, y1) = ((ay.min(by) - r - 1.0).floor() as i32, (ay.max(by) + r + 1.0).ceil() as i32);
        let (dx, dy) = (bx - ax, by - ay);
        let l2 = (dx * dx + dy * dy).max(1e-6);
        for j in y0..=y1 {
            for i in x0..=x1 {
                let (px, py) = (i as f32 + 0.5, j as f32 + 0.5);
                let t = (((px - ax) * dx + (py - ay) * dy) / l2).clamp(0.0, 1.0);
                let d = ((px - ax - dx * t).powi(2) + (py - ay - dy * t).powi(2)).sqrt();
                self.blend(i, j, c, a * (r + 0.5 - d).clamp(0.0, 1.0));
            }
        }
    }

    pub fn polyline(&mut self, pts: &[(f32, f32)], w: f32, c: Rgb, a: f32) {
        for s in pts.windows(2) {
            self.line(s[0].0, s[0].1, s[1].0, s[1].1, w, c, a);
        }
    }

    /// A dashed line: `on` pixels drawn, `off` skipped, carried across corners.
    pub fn dashed(&mut self, pts: &[(f32, f32)], w: f32, on: f32, off: f32, c: Rgb, a: f32) {
        let mut phase = 0.0f32;
        for s in pts.windows(2) {
            let (ax, ay, bx, by) = (s[0].0, s[0].1, s[1].0, s[1].1);
            let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
            let mut t = 0.0;
            while t < len {
                let period = on + off;
                let pos = phase % period;
                let step = if pos < on { (on - pos).min(len - t) } else { (period - pos).min(len - t) };
                if pos < on {
                    let (u0, u1) = (t / len, (t + step) / len);
                    self.line(ax + (bx - ax) * u0, ay + (by - ay) * u0, ax + (bx - ax) * u1, ay + (by - ay) * u1, w, c, a);
                }
                t += step.max(0.01);
                phase += step.max(0.01);
            }
        }
    }

    pub fn text_width(&self, s: &str, size: f32) -> f32 {
        let font = FontRef::try_from_slice(FONT).expect("font");
        let f = font.as_scaled(PxScale::from(size));
        s.chars().map(|ch| f.h_advance(f.glyph_id(ch))).sum()
    }

    /// Text with its baseline at `y`.
    pub fn text(&mut self, x: f32, y: f32, s: &str, size: f32, c: Rgb) {
        let font = FontRef::try_from_slice(FONT).expect("font");
        let f = font.as_scaled(PxScale::from(size));
        let mut pen = x;
        for ch in s.chars() {
            let id = f.glyph_id(ch);
            let g = id.with_scale_and_position(PxScale::from(size), ab_glyph::point(pen, y));
            if let Some(o) = font.outline_glyph(g) {
                let b = o.px_bounds();
                o.draw(|gx, gy, cov| self.blend(b.min.x as i32 + gx as i32, b.min.y as i32 + gy as i32, c, cov));
            }
            pen += f.h_advance(id);
        }
    }

    /// Text with a halo so it reads over a busy map.
    pub fn label(&mut self, x: f32, y: f32, s: &str, size: f32, c: Rgb, halo: Rgb) {
        for (dx, dy) in [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (-1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0)] {
            self.text(x + dx, y + dy, s, size, halo);
        }
        self.text(x, y, s, size, c);
    }

    /// Wrap `s` to `width` pixels; returns the lines.
    pub fn wrap(&self, s: &str, size: f32, width: f32) -> Vec<String> {
        let mut out = Vec::new();
        let mut line = String::new();
        for word in s.split_whitespace() {
            let trial = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if self.text_width(&trial, size) > width && !line.is_empty() {
                out.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = trial;
            }
        }
        if !line.is_empty() {
            out.push(line);
        }
        out
    }

    /// Copy another picture in at (x, y), shrunk by whole-pixel averaging
    /// to `w` wide (keeps its proportions).
    pub fn paste_scaled(&mut self, src: &Canvas, x: usize, y: usize, w: usize) {
        let s = src.w as f32 / w as f32;
        let h = (src.h as f32 / s) as usize;
        for j in 0..h {
            for i in 0..w {
                let (sx0, sy0) = ((i as f32 * s) as usize, (j as f32 * s) as usize);
                let (sx1, sy1) = ((((i + 1) as f32 * s) as usize).max(sx0 + 1).min(src.w), (((j + 1) as f32 * s) as usize).max(sy0 + 1).min(src.h));
                let mut acc = [0.0; 3];
                let mut n = 0.0;
                for sy in sy0..sy1 {
                    for sx in sx0..sx1 {
                        let p = src.px[sy * src.w + sx];
                        for k in 0..3 {
                            acc[k] += p[k];
                        }
                        n += 1.0;
                    }
                }
                if x + i < self.w && y + j < self.h {
                    self.px[(y + j) * self.w + x + i] = [acc[0] / n, acc[1] / n, acc[2] / n];
                }
            }
        }
    }

    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut img = image::RgbImage::new(self.w as u32, self.h as u32);
        for (k, p) in self.px.iter().enumerate() {
            let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            img.put_pixel((k % self.w) as u32, (k / self.w) as u32, image::Rgb([c(p[0]), c(p[1]), c(p[2])]));
        }
        img.save(path).map_err(std::io::Error::other)
    }
}
