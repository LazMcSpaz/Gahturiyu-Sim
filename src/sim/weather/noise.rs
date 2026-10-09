//! Smooth randomness over time.
//!
//! Each weather variable in each region follows its own slowly wandering
//! curve. A curve is fixed by its key (world seed, region, which variable):
//! its value at any moment comes straight from the moment, so yesterday and
//! next week cost the same as now and asking in any order gives the same
//! answer.

use std::sync::OnceLock;

use crate::sim::rng;

/// A fixed random value in 0..1 for a whole number on a curve.
fn lattice(key: u64, i: i64) -> f32 {
    (rng::key(&[key, i as u64]) >> 40) as f32 / (1u64 << 24) as f32
}

/// One smooth curve, 0..1, with a new random value at each whole `x`.
fn value(key: u64, x: f64) -> f32 {
    let i = x.floor();
    let f = (x - i) as f32;
    let i = i as i64;
    let (a, b) = (lattice(key, i), lattice(key, i + 1));
    let s = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    a + (b - a) * s
}

/// A slow curve with a faster one riding on it, 0..1 (bunched round the middle).
fn raw(key: u64, x: f64) -> f32 {
    (value(key, x) + 0.5 * value(key ^ 0x5EED_0F_0C7A_FE, x * 2.0 + 0.37)) / 1.5
}

const BINS: usize = 512;

/// How `raw` values are spread, measured once: `table[k]` is the share of
/// the time `raw` is below `k / BINS`.
fn spread() -> &'static [f32; BINS + 1] {
    static T: OnceLock<[f32; BINS + 1]> = OnceLock::new();
    T.get_or_init(|| {
        let mut count = [0u32; BINS];
        let n = 400_000;
        for k in 0..n {
            let v = raw(0xC0FF_EE00_1234, k as f64 * 0.0731);
            count[((v * BINS as f32) as usize).min(BINS - 1)] += 1;
        }
        let mut t = [0.0f32; BINS + 1];
        let mut run = 0u32;
        for k in 0..BINS {
            run += count[k];
            t[k + 1] = run as f32 / n as f32;
        }
        t
    })
}

/// A smooth curve over time, 0..1, that spends an equal share of the time at
/// every level: it is below 0.3 for three-tenths of a year, and so on. That
/// is what lets the climate tables be written as shares of the time.
/// `period` is roughly how long (seconds) it takes to wander somewhere new.
pub fn even(key: u64, t: f64, period: f64) -> f32 {
    let v = raw(key, t / period).clamp(0.0, 0.999_999);
    let x = v * BINS as f32;
    let k = x as usize;
    let table = spread();
    table[k] + (table[k + 1] - table[k]) * (x - k as f32)
}

/// Smooth step from 0 at `a` to 1 at `b` (either way round).
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_even_curve_spends_equal_time_at_every_level() {
        let key = 0xABCD;
        let n = 200_000;
        let mut below = [0u32; 4];
        for k in 0..n {
            let v = even(key, k as f64 * 977.0, 3600.0 * 9.0);
            for (j, lim) in [0.1, 0.3, 0.6, 0.9].iter().enumerate() {
                if v < *lim {
                    below[j] += 1;
                }
            }
        }
        for (j, lim) in [0.1f32, 0.3, 0.6, 0.9].iter().enumerate() {
            let share = below[j] as f32 / n as f32;
            assert!((share - lim).abs() < 0.03, "below {lim} for {share} of the time");
        }
    }
}
