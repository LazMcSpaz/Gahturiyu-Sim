//! The two sketch pictures: a top-down map (hillshade, 5 m contours, marks,
//! labels, scale bar) and a low view from the sea.

use crate::canvas::{rgb, Canvas, Rgb};
use crate::field::Field;
use crate::noise::{fbm, noise, smooth};
use gahturiyu_sim::sim::geo::V2;

/// Something placed on the land in a sketch (used from Step 3 on).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Thing {
    /// A Roduro grown-stone home (`r` = footprint radius in metres).
    Home { at: V2, r: f32, eldest: bool },
    /// A Horaro woven dome on stone pillars in the water.
    Stilt { at: V2, r: f32 },
    /// A Ṭaḍoro tent.
    Tent { at: V2 },
    /// The Qotiro stepped hall.
    Hall { at: V2, size: f32, yaw: f32 },
    /// A bridge deck between two points at given heights; `sag` for rope bridges.
    Bridge { a: V2, za: f32, b: V2, zb: f32, sag: f32 },
    /// A beacon tower.
    Beacon { at: V2 },
    /// A stone quay or landing, from `a` to `b`, `w` wide.
    Quay { a: V2, b: V2, w: f32 },
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WayKind {
    /// A cobbled lane.
    Lane,
    /// Carved stone stairs.
    Stairs,
    /// A dirt track (the road in).
    Road,
    /// A stream.
    Stream,
}

#[derive(Clone, Debug)]
pub struct Way {
    pub kind: WayKind,
    pub pts: Vec<V2>,
}

impl WayKind {
    pub fn colour(self) -> Rgb {
        match self {
            WayKind::Lane => rgb(150, 144, 128),
            WayKind::Stairs => rgb(186, 180, 162),
            WayKind::Road => rgb(128, 108, 80),
            WayKind::Stream => rgb(60, 92, 104),
        }
    }
    pub fn width(self) -> f32 {
        match self {
            WayKind::Lane => 3.0,
            WayKind::Stairs => 2.5,
            WayKind::Road => 4.0,
            WayKind::Stream => 3.5,
        }
    }
}

pub const STONE: [f32; 3] = [0.64, 0.57, 0.44];
const ELDEST: [f32; 3] = [0.52, 0.44, 0.32];
const WOVEN: [f32; 3] = [0.58, 0.44, 0.28];
const PILLAR: [f32; 3] = [0.40, 0.44, 0.48];
const HIDE: [f32; 3] = [0.66, 0.58, 0.66];
const OCHRE: [f32; 3] = [0.72, 0.50, 0.30];
const WOOD: [f32; 3] = [0.42, 0.31, 0.21];

// ---- Ways on the ground ---------------------------------------------------

/// Paint the ways into the field's ground colour, so the views show them.
pub fn paint_ways(f: &mut Field, ways: &[gahturiyu_sim::sim::forge::Way]) {
    use gahturiyu_sim::sim::forge::WayKind as K;
    for w in ways {
        let col = match w.kind {
            K::Cobbles => [0.58, 0.56, 0.52],
            K::Dirt => [0.50, 0.38, 0.24],
            K::Stairs => [0.76, 0.73, 0.66],
            K::Bridge => continue,
        };
        let half = w.width * 0.5 + f.cell * 0.4;
        for s in w.pts.windows(2) {
            let (a, b) = (s[0], s[1]);
            let (lo_x, hi_x) = (a.x.min(b.x) - half, a.x.max(b.x) + half);
            let (lo_y, hi_y) = (a.y.min(b.y) - half, a.y.max(b.y) + half);
            let (i0, i1) = (((lo_x - f.x0) / f.cell).floor().max(0.0) as usize, (((hi_x - f.x0) / f.cell).ceil().max(0.0) as usize).min(f.w - 1));
            let (j0, j1) = (((lo_y - f.y0) / f.cell).floor().max(0.0) as usize, (((hi_y - f.y0) / f.cell).ceil().max(0.0) as usize).min(f.h - 1));
            let d = b.sub(a);
            let len2 = d.x * d.x + d.y * d.y;
            for j in j0..=j1 {
                for i in i0..=i1 {
                    let p = V2::new(f.x0 + i as f32 * f.cell, f.y0 + j as f32 * f.cell);
                    let t = if len2 > 0.0 { ((p.x - a.x) * d.x + (p.y - a.y) * d.y) / len2 } else { 0.0 }.clamp(0.0, 1.0);
                    let q = a.add(d.scale(t));
                    if p.dist(q) <= half {
                        let k = j * f.w + i;
                        // Stairs: a tread line every other metre along.
                        let shade = if w.kind == K::Stairs && ((t * a.dist(b)) / 0.8).floor() as i32 % 2 == 0 { 0.8 } else { 1.0 };
                        f.albedo[k] = [col[0] * shade, col[1] * shade, col[2] * shade];
                    }
                }
            }
        }
    }
}

// ---- Ground colour --------------------------------------------------------

/// Colour every cell of the field from its height, slope and rockiness.
pub fn colour_ground(f: &mut Field, seed: u64) {
    for k in 0..f.z.len() {
        let (x, y) = f.cell_pos(k);
        let h = f.z[k];
        let slope = f.slope(x, y);
        f.albedo[k] = ground_colour(seed, x, y, h, slope, f.rock[k], f.kind[k]);
    }
}

fn mixc(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn ground_colour(seed: u64, x: f32, y: f32, h: f32, slope: f32, rock: f32, kind: u8) -> Rgb {
    let n1 = noise(seed ^ 0x6A55, x, y, 40.0);
    let n2 = noise(seed ^ 0x6A56, x, y, 9.0);
    if h < 0.0 {
        // Sea floor: only seen through shallow water; the renderers tint it.
        let floor = if kind == 3 { rgb(160, 150, 125) } else { rgb(120, 118, 105) };
        return mixc(floor, rgb(70, 80, 78), smooth(0.0, 8.0, -h));
    }
    match kind {
        2 => return mixc(mixc(rgb(124, 116, 102), rgb(92, 88, 84), n2 * 0.8), rgb(104, 110, 80), smooth(0.6, 0.9, n1) * 0.3),
        3 => return mixc(rgb(160, 152, 132), rgb(122, 118, 108), n2),
        4 => return mixc(rgb(74, 74, 70), rgb(48, 54, 50), n2 * 0.6 + 0.3),
        5 => return rgb(60, 92, 104),
        _ => {}
    }
    let moss = rgb(92, 112, 60);
    let grass = rgb(104, 118, 70);
    let heather = rgb(104, 82, 70);
    let mut c = mixc(grass, moss, n1);
    c = mixc(c, heather, smooth(0.55, 0.85, fbm(seed ^ 0x6A57, x, y, 60.0, 3)) * 0.45);
    c = mixc(c, mixc(c, rgb(130, 128, 90), 0.5), n2 * 0.4);
    // Shingle and wet rock along the water's edge.
    let shore = 1.0 - smooth(1.2, 3.0, h);
    c = mixc(c, mixc(rgb(150, 144, 128), rgb(110, 108, 100), n2), shore * (1.0 - smooth(20.0, 35.0, slope)));
    // Bare rock: crags and anything steep.
    let bare = rock.max(smooth(34.0, 46.0, slope));
    let rk = mixc(rgb(118, 116, 110), rgb(84, 82, 80), n2 * 0.7 + smooth(50.0, 75.0, slope) * 0.4);
    let rk = mixc(rk, rgb(128, 132, 96), smooth(0.6, 0.9, n1) * 0.35 * (1.0 - smooth(50.0, 70.0, slope))); // lichen
    c = mixc(c, rk, bare);
    // High ground: scree, then snow on the gentler tops.
    c = mixc(c, mixc(rgb(120, 112, 100), rgb(96, 92, 88), n2), smooth(330.0, 430.0, h) * 0.8);
    c = mixc(c, rgb(232, 234, 238), smooth(520.0, 580.0, h + n1 * 40.0) * (1.0 - smooth(38.0, 52.0, slope)));
    c
}

// ---- Top-down map ---------------------------------------------------------

pub struct MapView {
    pub centre: V2,
    /// Metres shown across (and down).
    pub span: f32,
    pub px: usize,
}

impl MapView {
    pub fn to_px(&self, p: V2) -> (f32, f32) {
        let s = self.px as f32 / self.span;
        ((p.x - self.centre.x + self.span * 0.5) * s, (p.y - self.centre.y + self.span * 0.5) * s)
    }
    fn to_world(&self, i: f32, j: f32) -> V2 {
        let s = self.span / self.px as f32;
        V2::new(self.centre.x - self.span * 0.5 + i * s, self.centre.y - self.span * 0.5 + j * s)
    }
}

pub fn top_down(f: &Field, v: &MapView, things: &[Thing], ways: &[Way], labels: &[(V2, String)]) -> Canvas {
    let n = v.px;
    let mut c = Canvas::new(n, n, [0.0; 3]);
    let light = norm3([-1.0, -1.0, 1.4]); // from the north-west, map-style
    let mut hs = vec![0.0f32; n * n];
    for j in 0..n {
        for i in 0..n {
            let p = v.to_world(i as f32 + 0.5, j as f32 + 0.5);
            let h = f.height(p);
            hs[j * n + i] = h;
            let nn = f.normal(p.x, p.y);
            let lit = dot3(nn, light).max(0.0);
            let col = if h < 0.0 {
                let d = smooth(0.0, 25.0, -h);
                let sea = mixc(rgb(126, 160, 168), rgb(58, 88, 108), d);
                mixc(sea, f.colour(p.x, p.y), (1.0 - smooth(0.0, 2.5, -h)) * 0.35)
            } else {
                let base = f.colour(p.x, p.y);
                let sh = 0.42 + 0.78 * lit;
                [base[0] * sh, base[1] * sh, base[2] * sh]
            };
            c.px[j * n + i] = col;
        }
    }
    // Contours every 5 m, darker every 25 m; the coastline bold.
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let h = hs[j * n + i];
            let (hr, hd) = (hs[j * n + i + 1], hs[(j + 1) * n + i]);
            let band = |x: f32| (x / 5.0).floor() as i32;
            let crosses = |a: f32, b: f32, step: f32| (a / step).floor() != (b / step).floor();
            if (h > 0.0) != (hr > 0.0) || (h > 0.0) != (hd > 0.0) {
                c.blend(i as i32, j as i32, rgb(30, 40, 46), 0.9);
            } else if h > 0.0 && (band(h) != band(hr) || band(h) != band(hd)) {
                let major = crosses(h, hr, 25.0) || crosses(h, hd, 25.0);
                c.blend(i as i32, j as i32, rgb(52, 44, 34), if major { 0.55 } else { 0.22 });
            }
        }
    }
    let s = n as f32 / v.span;
    for w in ways {
        let pts: Vec<(f32, f32)> = w.pts.iter().map(|&p| v.to_px(p)).collect();
        let width = (w.kind.width() * s).max(1.5);
        match w.kind {
            WayKind::Stairs => {
                c.polyline(&pts, width + 2.0, rgb(40, 36, 30), 0.6);
                c.dashed(&pts, width, 2.5, 1.5, w.kind.colour(), 1.0);
            }
            WayKind::Stream => c.polyline(&pts, width, w.kind.colour(), 0.9),
            _ => {
                c.polyline(&pts, width + 1.5, rgb(40, 36, 30), 0.5);
                c.polyline(&pts, width, w.kind.colour(), 1.0);
            }
        }
    }
    for t in things {
        match *t {
            Thing::Home { at, r, eldest } => {
                let (x, y) = v.to_px(at);
                let r = (r * s).max(2.0);
                c.disc(x, y, r + 1.2, rgb(48, 36, 26), 1.0);
                c.disc(x, y, r, if eldest { ELDEST } else { STONE }, 1.0);
                if eldest {
                    for k in 1..4 {
                        let rr = r * (1.0 - k as f32 * 0.22);
                        c.disc(x, y, rr, if k % 2 == 1 { STONE } else { ELDEST }, 1.0);
                    }
                    c.disc(x, y, r + 4.0, rgb(230, 190, 80), 0.0);
                }
            }
            Thing::Stilt { at, r } => {
                let (x, y) = v.to_px(at);
                let r = (r * s).max(2.0);
                c.disc(x, y, r + 1.2, rgb(30, 30, 30), 1.0);
                c.disc(x, y, r, WOVEN, 1.0);
            }
            Thing::Tent { at } => {
                let (x, y) = v.to_px(at);
                let r = (3.0 * s).max(3.0);
                for k in 0..3 {
                    let a0 = -std::f32::consts::FRAC_PI_2 + k as f32 * 2.094;
                    let a1 = a0 + 2.094;
                    c.line(x + r * a0.cos(), y + r * a0.sin(), x + r * a1.cos(), y + r * a1.sin(), 1.5, rgb(40, 30, 40), 1.0);
                }
                c.disc(x, y, r * 0.55, HIDE, 1.0);
            }
            Thing::Hall { at, size, .. } => {
                let (x, y) = v.to_px(at);
                let r = size * s * 0.5;
                c.fill_rect(x - r - 1.0, y - r - 1.0, 2.0 * r + 2.0, 2.0 * r + 2.0, rgb(40, 28, 20), 1.0);
                for k in 0..3 {
                    let rr = r * (1.0 - k as f32 * 0.28);
                    let shade = 1.0 - k as f32 * 0.1;
                    c.fill_rect(x - rr, y - rr, 2.0 * rr, 2.0 * rr, [OCHRE[0] * shade, OCHRE[1] * shade, OCHRE[2] * shade], 1.0);
                }
            }
            Thing::Bridge { a, b, .. } => {
                let (ax, ay) = v.to_px(a);
                let (bx, by) = v.to_px(b);
                c.line(ax, ay, bx, by, (3.0 * s).max(2.0) + 2.0, rgb(30, 22, 16), 1.0);
                c.line(ax, ay, bx, by, (3.0 * s).max(2.0), WOOD, 1.0);
            }
            Thing::Beacon { at } => {
                let (x, y) = v.to_px(at);
                c.disc(x, y, 6.0, rgb(40, 20, 10), 1.0);
                c.disc(x, y, 4.0, rgb(240, 150, 50), 1.0);
            }
            Thing::Quay { a, b, w } => {
                let (ax, ay) = v.to_px(a);
                let (bx, by) = v.to_px(b);
                c.line(ax, ay, bx, by, w * s + 2.0, rgb(30, 30, 30), 1.0);
                c.line(ax, ay, bx, by, w * s, rgb(150, 150, 146), 1.0);
            }
        }
    }
    for (at, text) in labels {
        let (x, y) = v.to_px(*at);
        let w = c.text_width(text, 15.0);
        c.label(x - w * 0.5, y, text, 15.0, rgb(250, 248, 240), rgb(20, 22, 24));
    }
    scale_bar(&mut c, s);
    north_arrow(&mut c);
    c
}

fn scale_bar(c: &mut Canvas, s: f32) {
    let (x0, y0) = (24.0, c.h as f32 - 30.0);
    c.fill_rect(x0 - 10.0, y0 - 26.0, 100.0 * s + 52.0, 40.0, rgb(250, 248, 240), 0.8);
    for k in 0..4 {
        let col = if k % 2 == 0 { rgb(20, 20, 20) } else { rgb(250, 250, 250) };
        c.fill_rect(x0 + k as f32 * 25.0 * s, y0, 25.0 * s, 6.0, col, 1.0);
    }
    c.line(x0, y0, x0 + 100.0 * s, y0, 1.0, rgb(20, 20, 20), 1.0);
    c.line(x0, y0 + 6.0, x0 + 100.0 * s, y0 + 6.0, 1.0, rgb(20, 20, 20), 1.0);
    c.text(x0 - 3.0, y0 - 6.0, "0", 12.0, rgb(20, 20, 20));
    c.text(x0 + 50.0 * s - 10.0, y0 - 6.0, "50 m", 12.0, rgb(20, 20, 20));
    c.text(x0 + 100.0 * s - 14.0, y0 - 6.0, "100", 12.0, rgb(20, 20, 20));
}

fn north_arrow(c: &mut Canvas) {
    let (x, y) = (c.w as f32 - 34.0, 52.0);
    c.disc(x, y - 6.0, 22.0, rgb(250, 248, 240), 0.75);
    c.line(x, y + 8.0, x, y - 22.0, 2.0, rgb(20, 20, 20), 1.0);
    c.line(x, y - 22.0, x - 6.0, y - 12.0, 2.0, rgb(20, 20, 20), 1.0);
    c.line(x, y - 22.0, x + 6.0, y - 12.0, 2.0, rgb(20, 20, 20), 1.0);
    c.text(x - 5.0, y + 9.0, "N", 12.0, rgb(20, 20, 20));
}

// ---- The view from the sea -----------------------------------------------

pub struct Cam {
    pub pos: V2,
    pub z: f32,
    pub look: V2,
    /// Degrees above level (positive lowers the horizon in the picture).
    pub pitch_deg: f32,
    pub fov_deg: f32,
    pub w: usize,
    pub h: usize,
}

/// Late-afternoon sun from the west-south-west: it lights the sea-facing cliffs.
const SUN: [f32; 3] = [-0.80, 0.30, 0.52];
const SUN_COL: [f32; 3] = [1.08, 0.98, 0.84];
const HAZE: [f32; 3] = [0.70, 0.76, 0.80];

/// Colour of bare rock at a point and height (strata), if the field knows it.
pub type FaceFn<'a> = &'a (dyn Fn(V2, f32) -> Option<Rgb> + Sync);

/// A picture from a camera: every pixel's ray is followed through the air
/// until it meets the ground or the sea, and shaded there by the sun.
pub fn view(f: &Field, cam: &Cam, things: &[Thing], face: FaceFn) -> Canvas {
    let (w, h) = (cam.w, cam.h);
    let mut c = Canvas::new(w, h, [0.0; 3]);
    let mut depth = vec![f32::INFINITY; w * h];
    let fwd = norm2(cam.look.sub(cam.pos));
    let right = V2::new(-fwd.y, fwd.x);
    let focal = (w as f32 * 0.5) / (cam.fov_deg.to_radians() * 0.5).tan();
    let pitch = cam.pitch_deg.to_radians();
    let horizon = h as f32 * 0.5 + pitch.tan() * focal;
    let sun = norm3(SUN);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    let rows = h.div_ceil(threads);
    std::thread::scope(|s| {
        for (k, (pc, dc)) in c.px.chunks_mut(rows * w).zip(depth.chunks_mut(rows * w)).enumerate() {
            s.spawn(move || {
                for (n, (px, dp)) in pc.iter_mut().zip(dc.iter_mut()).enumerate() {
                    let (i, j) = (n % w, k * rows + n / w);
                    let a = (i as f32 + 0.5 - w as f32 * 0.5) / focal;
                    let b = (horizon - j as f32 - 0.5) / focal;
                    // Ray direction: forward, sideways, up.
                    let dir2 = fwd.add(right.scale(a));
                    let (col, d) = march(f, cam, dir2, b, sun, face, (j as f32 - horizon) / h as f32);
                    *px = col;
                    *dp = d;
                }
            });
        }
    });
    // Buildings and bridges as clouds of small shaded points.
    let mut pts = Vec::new();
    for t in things {
        shape(f, t, &mut pts);
    }
    for (p, n, col) in pts {
        let rel = V2::new(p[0], p[1]).sub(cam.pos);
        let z = rel.x * fwd.x + rel.y * fwd.y;
        if z < 2.0 {
            continue;
        }
        let x = rel.x * right.x + rel.y * right.y;
        let sx = w as f32 * 0.5 + x * focal / z;
        let sy = horizon - (p[2] - cam.z) * focal / z;
        let size = (0.45 * focal / z).max(1.0);
        let lit = 0.42 + 0.75 * dot3(n, sun).max(0.0) * shadow(f, V2::new(p[0], p[1]), p[2] + 0.5, sun);
        let shaded = [col[0] * lit * SUN_COL[0], col[1] * lit * SUN_COL[1], col[2] * lit * SUN_COL[2]];
        let shaded = fog(shaded, z);
        let (x0, y0) = ((sx - size * 0.5).floor() as i32, (sy - size * 0.5).floor() as i32);
        let n = size.ceil() as i32;
        for jj in y0..y0 + n {
            for ii in x0..x0 + n {
                if ii < 0 || jj < 0 || ii as usize >= w || jj as usize >= h {
                    continue;
                }
                let k = jj as usize * w + ii as usize;
                if z < depth[k] + 0.3 {
                    depth[k] = z;
                    c.px[k] = shaded;
                }
            }
        }
    }
    c
}

/// Follow one ray; returns the colour and the forward distance of what it hit.
fn march(f: &Field, cam: &Cam, dir2: V2, up: f32, sun: [f32; 3], face: FaceFn, sky_t: f32) -> (Rgb, f32) {
    let sky = {
        let t = (-sky_t).clamp(0.0, 1.0);
        mixc(rgb(214, 216, 212), rgb(120, 148, 176), t.powf(0.7))
    };
    let mut t = 1.0f32;
    let mut prev_t = t;
    let mut prev_gap = 1.0f32;
    let horiz = dir2.len();
    while t < 9000.0 {
        let p = cam.pos.add(dir2.scale(t));
        let z = cam.z + up * t;
        if !f.inside(p.x, p.y) {
            return (sky, f32::INFINITY);
        }
        let g = f.height(p);
        let ground = g.max(0.0); // the sea surface counts as ground
        let gap = z - ground;
        if gap < 0.0 {
            // Crossed: pin the crossing down between the last two samples.
            let (mut lo, mut hi) = (prev_t, t);
            for _ in 0..6 {
                let mid = (lo + hi) * 0.5;
                let pm = cam.pos.add(dir2.scale(mid));
                let zm = cam.z + up * mid;
                if zm - f.height(pm).max(0.0) < 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            let th = (lo + hi) * 0.5;
            let ph = cam.pos.add(dir2.scale(th));
            let zh = cam.z + up * th;
            let dist = th * horiz;
            let gh = f.height(ph);
            let col = if gh < 0.05 && zh < 0.3 {
                water(f, ph, gh, dist, cam.z)
            } else {
                let n = f.normal(ph.x, ph.y);
                let base = surface_colour(f, ph, gh, n, face);
                let lit = 0.36 + 0.84 * dot3(n, sun).max(0.0) * shadow(f, ph, gh + 0.4, sun);
                // Faces turned from the sun still catch the sky.
                let lit = lit + 0.12 * (n[2] * 0.5 + 0.5);
                [base[0] * lit * SUN_COL[0], base[1] * lit * SUN_COL[1], base[2] * lit * SUN_COL[2]]
            };
            return (fog(col, dist), th);
        }
        prev_t = t;
        prev_gap = gap;
        // Step by how far above the ground we are (never more than a cell or so near by).
        let step = (gap * 0.5).clamp(0.25, 0.8 + t * 0.004);
        t += step;
    }
    let _ = prev_gap;
    (sky, f32::INFINITY)
}

/// What the ground looks like here: the painted surface, or bare rock in
/// layers where the face is steep.
fn surface_colour(f: &Field, p: V2, g: f32, n: [f32; 3], face: FaceFn) -> Rgb {
    let base = f.colour(p.x, p.y);
    let steep = smooth(0.75, 0.5, n[2]); // 1 on faces steeper than ~60°
    if steep <= 0.0 {
        return base;
    }
    let rock = face(p, g).unwrap_or_else(|| {
        let band = noise(0x5747, g * 0.45, (p.x + p.y) * 0.02, 1.0);
        mixc(rgb(112, 108, 100), rgb(70, 68, 66), band * 0.8)
    });
    let rock = mixc(rock, rgb(52, 56, 50), (1.0 - smooth(0.0, 4.0, g)) * 0.6); // wet and weedy low down
    mixc(base, rock, steep)
}

/// 0 in shadow, 1 in sun: march toward the sun over the field.
fn shadow(f: &Field, p: V2, z: f32, sun: [f32; 3]) -> f32 {
    let hz = (sun[0] * sun[0] + sun[1] * sun[1]).sqrt();
    let (dx, dy, rise) = (sun[0] / hz, sun[1] / hz, sun[2] / hz);
    let mut t = 1.5;
    while t < 1500.0 {
        let q = V2::new(p.x + dx * t, p.y + dy * t);
        if !f.inside(q.x, q.y) {
            return 1.0;
        }
        if f.height(q) > z + rise * t {
            return 0.0;
        }
        t += 0.8 + t * 0.03;
    }
    1.0
}

fn water(f: &Field, p: V2, g: f32, dist: f32, cam_z: f32) -> Rgb {
    let deep = rgb(34, 58, 70);
    let shallow = rgb(62, 104, 104);
    let mut c = mixc(shallow, deep, smooth(0.0, 10.0, -g));
    // The sea mirrors the sky more the flatter you look across it.
    let graze = (cam_z.max(0.5) / dist.max(1.0)).atan();
    let fres = 0.04 + 0.96 * (1.0 - graze.sin()).powi(5);
    c = mixc(c, rgb(190, 200, 204), fres * 0.65);
    // Surf where the water meets rock.
    let foam = (1.0 - smooth(0.0, 0.7, -g)) * (0.3 + 0.7 * noise(0xF0A3, p.x, p.y, 3.0));
    let near_cliff = (1.0 - smooth(0.4, 2.5, -g)) * smooth(30.0, 50.0, f.slope(p.x + 2.0, p.y).max(f.slope(p.x - 2.0, p.y)));
    mixc(c, rgb(236, 240, 238), (foam.max(near_cliff * 0.8) * 0.85).min(1.0))
}

fn fog(c: Rgb, dist: f32) -> Rgb {
    mixc(c, HAZE, (1.0 - (-dist / 4200.0).exp()) * 0.9)
}

type Pt = ([f32; 3], [f32; 3], Rgb);

/// Points on a dome of radius `r`, `hgt` tall, standing at `base`.
fn dome(base: [f32; 3], r: f32, hgt: f32, col: impl Fn(f32) -> Rgb, out: &mut Vec<Pt>) {
    let sp = 0.35;
    let rings = ((hgt.max(r) * 1.6) / sp) as usize + 2;
    for k in 0..=rings {
        let th = k as f32 / rings as f32 * std::f32::consts::FRAC_PI_2; // 0 = rim, pi/2 = top
        let (rr, zz) = (r * th.cos(), hgt * th.sin());
        let around = ((rr * std::f32::consts::TAU) / sp) as usize + 1;
        for m in 0..around {
            let ph = m as f32 / around as f32 * std::f32::consts::TAU;
            let n = norm3([th.cos() * ph.cos() / r, th.cos() * ph.sin() / r, th.sin() / hgt]);
            out.push(([base[0] + rr * ph.cos(), base[1] + rr * ph.sin(), base[2] + zz], n, col(zz / hgt)));
        }
    }
}

fn cylinder(base: [f32; 3], r: f32, hgt: f32, col: Rgb, out: &mut Vec<Pt>) {
    let sp = 0.3;
    let around = ((r * std::f32::consts::TAU) / sp) as usize + 3;
    let rows = (hgt / sp) as usize + 1;
    for k in 0..=rows {
        for m in 0..around {
            let ph = m as f32 / around as f32 * std::f32::consts::TAU;
            out.push(([base[0] + r * ph.cos(), base[1] + r * ph.sin(), base[2] + hgt * k as f32 / rows as f32], [ph.cos(), ph.sin(), 0.0], col));
        }
    }
}

/// An upright box, `w` × `d` × `hgt`, turned by `yaw`; faces only.
fn boxy(c: [f32; 3], w: f32, d: f32, hgt: f32, yaw: f32, col: Rgb, out: &mut Vec<Pt>) {
    let sp = 0.35;
    let (cy, sy) = (yaw.cos(), yaw.sin());
    let put = |u: f32, v: f32, z: f32, n: [f32; 3], out: &mut Vec<Pt>| {
        let nn = [n[0] * cy - n[1] * sy, n[0] * sy + n[1] * cy, n[2]];
        out.push(([c[0] + u * cy - v * sy, c[1] + u * sy + v * cy, c[2] + z], nn, col));
    };
    let nu = (w / sp) as usize + 1;
    let nv = (d / sp) as usize + 1;
    let nz = (hgt / sp) as usize + 1;
    for a in 0..=nu {
        for b in 0..=nv {
            let (u, v) = (-w / 2.0 + w * a as f32 / nu as f32, -d / 2.0 + d * b as f32 / nv as f32);
            put(u, v, hgt, [0.0, 0.0, 1.0], out);
        }
    }
    for a in 0..=nu {
        for k in 0..=nz {
            let (u, z) = (-w / 2.0 + w * a as f32 / nu as f32, hgt * k as f32 / nz as f32);
            put(u, -d / 2.0, z, [0.0, -1.0, 0.0], out);
            put(u, d / 2.0, z, [0.0, 1.0, 0.0], out);
        }
    }
    for b in 0..=nv {
        for k in 0..=nz {
            let (v, z) = (-d / 2.0 + d * b as f32 / nv as f32, hgt * k as f32 / nz as f32);
            put(-w / 2.0, v, z, [-1.0, 0.0, 0.0], out);
            put(w / 2.0, v, z, [1.0, 0.0, 0.0], out);
        }
    }
}

fn shape(f: &Field, t: &Thing, out: &mut Vec<Pt>) {
    match *t {
        Thing::Home { at, r, eldest } => {
            // Sit on the lowest ground under the footprint, so it never floats.
            let mut g = f.height(at);
            for k in 0..8 {
                let a = k as f32 * 0.785;
                g = g.min(f.height(V2::new(at.x + r * 0.8 * a.cos(), at.y + r * 0.8 * a.sin())));
            }
            let hgt = if eldest { r * 1.05 } else { r * 0.8 };
            let bands = if eldest { 7.0 } else { 3.0 };
            dome([at.x, at.y, g - 0.5], r, hgt + 0.5, |t| {
                let ring = ((t * bands).fract() < 0.18) as i32 as f32;
                let base = if eldest { ELDEST } else { STONE };
                mixc(base, [base[0] * 0.7, base[1] * 0.7, base[2] * 0.7], ring)
            }, out);
        }
        Thing::Stilt { at, r } => {
            let deck = 3.6;
            for k in 0..4 {
                let a = k as f32 * 1.571 + 0.4;
                cylinder([at.x + r * 0.7 * a.cos(), at.y + r * 0.7 * a.sin(), -4.0], 0.7, deck + 4.0, PILLAR, out);
            }
            boxy([at.x, at.y, deck - 0.4], r * 2.3, r * 2.3, 0.4, 0.3, WOOD, out);
            dome([at.x, at.y, deck], r, r * 0.9, |t| mixc(WOVEN, [0.46, 0.34, 0.22], ((t * 9.0).fract() < 0.3) as i32 as f32), out);
        }
        Thing::Tent { at } => {
            let g = f.height(at);
            dome([at.x, at.y, g], 2.6, 2.8, |t| mixc(HIDE, [0.5, 0.42, 0.5], t * 0.4), out);
        }
        Thing::Hall { at, size, yaw } => {
            let g = f.height(at) - 1.0;
            for k in 0..3 {
                let s = size * (1.0 - k as f32 * 0.25);
                let shade = 1.0 - k as f32 * 0.06;
                boxy([at.x, at.y, g + k as f32 * 4.0], s, s * 0.8, 4.2, yaw, [OCHRE[0] * shade, OCHRE[1] * shade, OCHRE[2] * shade], out);
            }
        }
        Thing::Bridge { a, za, b, zb, sag } => {
            let len = a.dist(b);
            let steps = (len / 0.3) as usize + 1;
            let dir = norm2(b.sub(a));
            let side = V2::new(-dir.y, dir.x);
            for k in 0..=steps {
                let t = k as f32 / steps as f32;
                let p = a.lerp(b, t);
                let z = za + (zb - za) * t - sag * (t * std::f32::consts::PI).sin();
                for m in -4..=4 {
                    let q = p.add(side.scale(m as f32 * 0.3));
                    out.push(([q.x, q.y, z], [0.0, 0.0, 1.0], WOOD));
                    out.push(([q.x, q.y, z - 0.5], [0.0, 0.0, -1.0], [0.3, 0.22, 0.15]));
                }
                for m in [-1.0f32, 1.0] {
                    let q = p.add(side.scale(m * 1.3));
                    out.push(([q.x, q.y, z + 1.0], [side.x * m, side.y * m, 0.0], [0.30, 0.24, 0.18]));
                    if k % 8 == 0 {
                        for hh in 0..4 {
                            out.push(([q.x, q.y, z + hh as f32 * 0.3], [side.x * m, side.y * m, 0.0], [0.30, 0.24, 0.18]));
                        }
                    }
                }
            }
        }
        Thing::Beacon { at } => {
            let g = f.height(at) - 0.5;
            cylinder([at.x, at.y, g], 2.2, 9.0, rgb(110, 104, 96), out);
            dome([at.x, at.y, g + 9.0], 2.4, 2.6, |_| [1.6, 0.9, 0.35], out);
        }
        Thing::Quay { a, b, w } => {
            let mid = a.lerp(b, 0.5);
            let dir = b.sub(a);
            boxy([mid.x, mid.y, -3.0], a.dist(b), w, 4.6, dir.y.atan2(dir.x), rgb(140, 138, 132), out);
        }
    }
}

fn norm2(v: V2) -> V2 {
    let l = v.len().max(1e-6);
    V2::new(v.x / l, v.y / l)
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
