//! Smooth noise for landforms, keyed by seed (same seed, same land).

use gahturiyu_sim::sim::rng::mix;

fn hash(seed: u64, i: i64, j: i64) -> f32 {
    let h = mix(seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Value noise in 0..1, features about `scale` metres across.
pub fn noise(seed: u64, x: f32, y: f32, scale: f32) -> f32 {
    let (fx, fy) = (x / scale, y / scale);
    let (i, j) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - i, fy - j);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let (i, j) = (i as i64, j as i64);
    let a = hash(seed, i, j);
    let b = hash(seed, i + 1, j);
    let c = hash(seed, i, j + 1);
    let d = hash(seed, i + 1, j + 1);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Several octaves of `noise`, 0..1.
pub fn fbm(seed: u64, x: f32, y: f32, scale: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut s) = (0.0, 1.0, 0.0, scale);
    for o in 0..octaves {
        sum += noise(seed.wrapping_add(o as u64 * 7919), x, y, s) * amp;
        norm += amp;
        amp *= 0.5;
        s *= 0.5;
    }
    sum / norm
}

/// Sharp crests (1 on the ridge line), 0..1.
pub fn ridged(seed: u64, x: f32, y: f32, scale: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut s) = (0.0, 1.0, 0.0, scale);
    for o in 0..octaves {
        let n = noise(seed.wrapping_add(o as u64 * 104_729), x, y, s);
        let r = 1.0 - (n * 2.0 - 1.0).abs();
        sum += r * r * amp;
        norm += amp;
        amp *= 0.5;
        s *= 0.5;
    }
    sum / norm
}

pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
