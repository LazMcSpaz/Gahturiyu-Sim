//! Flames that look like flames (drawing only; Laz, 11 Oct).
//!
//! Anything that burns (a hearth, a campfire, a torch, a forge, a lamp)
//! doesn't draw itself: it notes a `Spot` on whatever builder it's drawn into
//! (`Builder::flame`). Every frame, every spot near the camera (the cached
//! towns' and the moving world's) is drawn afresh into one mesh with the
//! `fire` material, which adds its light to what's behind it: tongues turned
//! to face the camera, white-gold at the root, orange, fading out at the tip,
//! each swaying and flickering on its own beat, and a few sparks rising.
//! The flicker runs on the window's clock, like the lamps' (`light.rs`).

use bevy::math::{vec3, Vec3};

use super::mesh::Builder;

/// One flame: where its root is, how tall and wide it stands, and a number
/// that sets its own beat.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    pub base: Vec3,
    pub height: f32,
    pub width: f32,
    pub seed: u32,
}

/// Beyond this many metres (times the flame's height), a flame is one tongue.
const NEAR: f32 = 120.0;
/// Pieces up a tongue.
const STEPS: usize = 6;

/// Colours up a tongue, linear and already multiplied by how strongly each
/// adds (the fire material adds; it ignores alpha). Above 1 they bloom.
const ROOT: [f32; 3] = [1.9, 1.25, 0.45];
const MID: [f32; 3] = [1.15, 0.36, 0.05];
const TIP: [f32; 3] = [0.0, 0.0, 0.0];

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The colour `v` of the way up a tongue (0 root, 1 tip), times `k`.
fn shade(v: f32, k: f32) -> [f32; 4] {
    let c = if v < 0.45 { mix(ROOT, MID, v / 0.45) } else { mix(MID, TIP, (v - 0.45) / 0.55) };
    [c[0] * k, c[1] * k, c[2] * k, 1.0]
}

/// A noise of the window's clock, -1..1, smooth.
fn wobble(t: f32, rate: f32, seed: u32) -> f32 {
    let s = (seed % 997) as f32 * 0.731;
    ((t * rate + s).sin() * 0.6 + (t * rate * 2.13 + s * 1.7).sin() * 0.4).clamp(-1.0, 1.0)
}

/// One tongue: a ribbon facing the camera, widest a little above its root,
/// closing to a point, its tip swaying.
#[allow(clippy::too_many_arguments)]
fn tongue(b: &mut Builder, root: Vec3, side: Vec3, h: f32, w: f32, sway: f32, k: f32, steps: usize, face: Vec3) {
    let mut prev: Option<(Vec3, Vec3, [f32; 4])> = None;
    for i in 0..=steps {
        let v = i as f32 / steps as f32;
        // Teardrop: rounded at the root, fullest a fifth of the way up,
        // drawn in to a curling point.
        let half = w * 0.5 * (1.0 - v).powf(1.35) * (0.6 + 2.2 * v).min(1.0);
        let lean = side * (sway * w * 0.7 * v * v);
        let c = root + Vec3::Y * (h * v) + lean;
        let (l, r) = (c - side * half, c + side * half);
        let col = shade(v, k);
        if let Some((pl, pr, pc)) = prev {
            b.quad_lin([pl, pr, r, l], [face; 4], [pc, pc, col, col]);
        }
        prev = Some((l, r, col));
    }
}

/// A soft round warmth lying on the ground: bright at the middle, nothing at
/// the rim.
fn glow(b: &mut Builder, c: Vec3, r: f32, k: f32, _face: Vec3) {
    let mid = [1.0 * k, 0.45 * k, 0.12 * k, 1.0];
    let rim = [0.0, 0.0, 0.0, 1.0];
    let n = 12;
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let p0 = c + vec3(a0.cos(), 0.0, a0.sin()) * r;
        let p1 = c + vec3(a1.cos(), 0.0, a1.sin()) * r;
        b.quad_lin([c, p0, p1, c], [Vec3::Y; 4], [mid, rim, rim, mid]);
    }
}

/// Draw every spot into `b` (which goes out with the fire material).
/// `eye`: the camera; `now`: the window's clock, seconds; `reach`: how far
/// out flames are drawn at all.
pub fn draw(b: &mut Builder, spots: &[Spot], eye: Vec3, now: f32, reach: f32) {
    for s in spots {
        let to_eye = eye - s.base;
        let dist = to_eye.length();
        if dist > reach {
            continue;
        }
        let flat = vec3(to_eye.x, 0.0, to_eye.z).normalize_or(Vec3::X);
        let side = vec3(-flat.z, 0.0, flat.x);
        let t = now;
        // Each flame breathes: taller and brighter, then lower.
        let breath = wobble(t, 6.5, s.seed);
        let h = s.height * (1.0 + 0.12 * breath);
        let k = 0.9 + 0.12 * wobble(t, 11.0, s.seed ^ 0x55);
        let near = dist < NEAR * s.height.max(0.3);
        if !near {
            tongue(b, s.base, side, h, s.width, 0.0, k, 2, flat);
            continue;
        }
        // A soft warmth on the ground round its root.
        glow(b, s.base + Vec3::Y * 0.05, s.width * 1.3, k * 0.35, flat);
        // A small flame is one tongue with two beside it; a big fire is a
        // row of tongues across it, each its own height and beat.
        let n = ((s.width / 0.6).round() as u32).clamp(1, 6);
        let tw = if n == 1 { s.width } else { s.width / n as f32 * 2.4 };
        if n > 1 {
            // The body of the fire, low and broad, under the tongues.
            tongue(b, s.base, side, h * 0.45, s.width * 1.05, wobble(t, 2.3, s.seed) * 0.3, k * 0.8, 4, flat);
        }
        for i in 0..n {
            let x = if n == 1 { 0.0 } else { (i as f32 / (n - 1) as f32 - 0.5) * s.width * 0.75 };
            let mid = 1.0 - (x / (s.width * 0.5)).abs() * 0.45;
            let own = s.seed.wrapping_add(i * 7919);
            let tall = if n == 1 { 1.0 } else { 0.55 + 0.35 * ((own % 97) as f32 / 97.0) };
            let hi = h * mid * tall * (1.0 + 0.15 * wobble(t, 7.5, own));
            let depth = flat * (((own >> 8) % 7) as f32 / 7.0 - 0.5) * s.width * 0.25;
            tongue(b, s.base + side * x + depth, side, hi, tw, wobble(t, 3.1, own), k, STEPS, flat);
        }
        if n == 1 {
            for (off, scale, beat) in [(-0.28f32, 0.62f32, 1u32), (0.3, 0.55, 2)] {
                let root = s.base + side * (s.width * off) + Vec3::Y * (s.height * 0.02);
                tongue(b, root, side, h * scale * (1.0 + 0.2 * wobble(t, 8.0, s.seed + beat)), s.width * 0.6, wobble(t, 4.0, s.seed + beat * 7), k * 0.8, STEPS - 2, flat);
            }
        }
        // A bright heart low down.
        tongue(b, s.base + flat * 0.02, side, h * 0.4, s.width * 0.5, wobble(t, 5.0, s.seed + 9) * 0.5, k * 1.25, 3, flat);
        // A few sparks rising and fading.
        if s.height > 0.25 {
            for i in 0..3u32 {
                let cycle = 1.1 + (i as f32) * 0.37;
                let f = ((t / cycle) + (s.seed.wrapping_mul(2654435761).wrapping_add(i * 977) % 1000) as f32 / 1000.0).fract();
                let up = s.base + Vec3::Y * (h * (0.7 + 1.1 * f)) + side * (s.width * 0.4 * wobble(t, 2.0, s.seed + i * 13));
                let z = (s.width * 0.02).clamp(0.015, 0.05);
                let fade = (1.0 - f) * (1.0 - f);
                let c = [2.2 * fade, 0.9 * fade, 0.2 * fade, 1.0];
                b.quad_lin([up - side * z, up + side * z, up + side * z + Vec3::Y * z * 2.0, up - side * z + Vec3::Y * z * 2.0], [flat; 4], [c; 4]);
            }
        }
    }
}

/// Faint wisps of smoke (chimneys, a smouldering stack): three soft grey
/// puffs, each rising from the root, spreading and thinning to nothing a
/// metre or two up, one after another. Drawn into a see-through builder.
pub fn draw_smoke(b: &mut Builder, spots: &[Spot], eye: Vec3, now: f32, reach: f32) {
    const PUFFS: u32 = 3;
    const RISE_SECS: f32 = 3.2;
    for s in spots {
        let to_eye = eye - s.base;
        if to_eye.length() > reach {
            continue;
        }
        let face = to_eye.normalize_or(Vec3::Y);
        let side = vec3(-face.z, 0.0, face.x).normalize_or(Vec3::X);
        let up = face.cross(side).normalize_or(Vec3::Y);
        for i in 0..PUFFS {
            let f = (now / RISE_SECS + i as f32 / PUFFS as f32 + (s.seed % 100) as f32 * 0.01).fract();
            let drift = wobble(now, 0.7, s.seed + i * 31) * 0.3 + 0.25;
            let c = s.base + Vec3::Y * (s.height * f) + side * (s.width * drift * f);
            let r = s.width * (0.35 + 0.9 * f);
            // In quickly, then thinning away as it rises.
            let a = 0.2 * (f * 6.0).min(1.0) * (1.0 - f);
            let grey = 0.62 - 0.12 * f;
            let mid = [grey, grey, grey * 1.02, a];
            let rim = [grey, grey, grey, 0.0];
            let n = 10;
            for k in 0..n {
                let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
                let p0 = c + (side * a0.cos() + up * a0.sin()) * r;
                let p1 = c + (side * a1.cos() + up * a1.sin()) * r;
                b.quad_lin([c, p0, p1, c], [face; 4], [mid, rim, rim, mid]);
            }
        }
    }
}
