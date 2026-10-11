//! Glowing outlines round what's selected or under the mouse (Laz's P3:
//! "selected objects get a glowing outline, not a circle underneath").
//! Drawing only.
//!
//! How: take the corners of the shape as drawn (any set of 3D points), find
//! their outer edge as the camera sees it (the convex hull of the points on
//! screen; for our block figures and buildings that is their silhouette), and
//! lay a thin band along that edge, just outside it, in the glow material. The
//! band sits at the thing's own depth, so walls and the ground in front still
//! hide it. Its width is in pixels, so it reads the same near and far.

use bevy::math::{vec2, Mat4, Vec2, Vec3};

use super::cam::{OrbitCam, FOV_DEG};
use super::mesh::Builder;
use super::palette::Rgb;

/// What the outline needs to know about the camera this frame.
pub struct Lens {
    vp: Mat4,
    size: Vec2,
    eye: Vec3,
    fwd: Vec3,
    right: Vec3,
    up: Vec3,
    /// World metres per screen pixel, one metre in front of the camera.
    per_px: f32,
}

impl Lens {
    pub fn of(orbit: &OrbitCam, size: Vec2) -> Lens {
        let eye = orbit.eye();
        let fwd = (orbit.look_at() - eye).normalize_or_zero();
        let right = fwd.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(fwd);
        let per_px = 2.0 * (FOV_DEG.to_radians() * 0.5).tan() / size.y.max(1.0);
        Lens { vp: orbit.view_proj(size), size, eye, fwd, right, up, per_px }
    }

    fn project(&self, p: Vec3) -> Option<Vec2> {
        let c = self.vp * p.extend(1.0);
        if c.w <= 1e-4 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some(vec2((n.x + 1.0) * 0.5 * self.size.x, (1.0 - n.y) * 0.5 * self.size.y))
    }

    /// A step of `px` pixels across the screen in direction `d` (y down), as a
    /// world vector at point `p`'s depth.
    fn across(&self, p: Vec3, d: Vec2, px: f32) -> Vec3 {
        let depth = (p - self.eye).dot(self.fwd).max(0.1);
        (self.right * d.x - self.up * d.y) * (px * self.per_px * depth)
    }
}

/// The convex hull of `pts` (indices, anticlockwise on screen), Andrew's
/// monotone chain.
fn hull(pts: &[Vec2]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..pts.len()).collect();
    idx.sort_by(|&a, &b| pts[a].x.total_cmp(&pts[b].x).then(pts[a].y.total_cmp(&pts[b].y)));
    idx.dedup_by(|a, b| pts[*a].distance(pts[*b]) < 0.01);
    if idx.len() < 3 {
        return idx;
    }
    let cross = |o: Vec2, a: Vec2, b: Vec2| (a - o).perp_dot(b - o);
    let mut out: Vec<usize> = Vec::with_capacity(idx.len() * 2);
    for pass in 0..2 {
        let start = out.len();
        let it: Box<dyn Iterator<Item = &usize>> = if pass == 0 { Box::new(idx.iter()) } else { Box::new(idx.iter().rev()) };
        for &i in it {
            while out.len() >= start + 2 && cross(pts[out[out.len() - 2]], pts[out[out.len() - 1]], pts[i]) <= 0.0 {
                out.pop();
            }
            out.push(i);
        }
        out.pop();
    }
    out
}

/// A glowing band `px` pixels wide round the outer edge of `pts` as seen
/// from the camera. It's drawn `pull` metres nearer the camera along the line
/// of sight (which leaves it where it is on screen), so the ground at a lying
/// figure's feet doesn't hide it; anything further in front still does.
pub fn outline(gl: &mut Builder, lens: &Lens, pts: &[Vec3], col: Rgb, px: f32, pull: f32) {
    let mut world = Vec::with_capacity(pts.len());
    let mut screen = Vec::with_capacity(pts.len());
    for &p in pts {
        if let Some(s) = lens.project(p) {
            world.push(p);
            screen.push(s);
        }
    }
    let h = hull(&screen);
    let n = h.len();
    if n < 3 {
        return;
    }
    let centre = h.iter().map(|&i| screen[i]).sum::<Vec2>() / n as f32;
    // Each edge's outward direction on screen.
    let normal = |a: Vec2, b: Vec2| {
        let e = (b - a).normalize_or_zero();
        let m = vec2(-e.y, e.x);
        if m.dot((a + b) * 0.5 - centre) < 0.0 { -m } else { m }
    };
    let edges: Vec<Vec2> = (0..n).map(|k| normal(screen[h[k]], screen[h[(k + 1) % n]])).collect();
    // Out from each corner (mitred, so the band keeps its width round corners).
    let outward: Vec<Vec3> = (0..n)
        .map(|k| {
            let (a, b) = (edges[(k + n - 1) % n], edges[k]);
            let m = (a + b).normalize_or_zero();
            let miter = (1.0 / m.dot(b).max(0.5)).min(2.0);
            lens.across(world[h[k]], m, px * miter)
        })
        .collect();
    let toward = |p: Vec3| (lens.eye - p).normalize_or_zero();
    for k in 0..n {
        let j = (k + 1) % n;
        let (a, b) = (world[h[k]], world[h[j]]);
        // (Starting a little inside, so no gap shows at the edge.)
        let quad = [a - outward[k] * 0.2, b - outward[j] * 0.2, b + outward[j], a + outward[k]].map(|p| p + toward(p) * pull);
        gl.quad(quad, toward(a), col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hull_of_a_square_with_a_point_inside_is_its_corners() {
        let pts = [vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0), vec2(0.5, 0.5)];
        let mut h = hull(&pts);
        h.sort();
        assert_eq!(h, vec![0, 1, 2, 3]);
    }
}
