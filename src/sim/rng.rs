//! Deterministic randomness.
//!
//! Everything random in the world comes from here, and every draw is keyed by
//! *what* is being decided (a person's seed, a settlement and an hour, a group
//! and a leg number), never by *when* the code happened to run. That is what
//! lets a far-away group be simulated coarsely and still end up exactly where a
//! finely simulated one would.
//!
//! Hand-rolled on purpose: library RNGs are allowed to change their output
//! between versions, and a world seed has to mean the same world forever.

/// Mix a 64-bit value into a well-scrambled one (SplitMix64 finaliser).
pub fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Combine several keys into one seed. Order matters.
pub fn key(parts: &[u64]) -> u64 {
    let mut h = 0x243F_6A88_85A3_08D3u64;
    for &p in parts {
        h = mix(h ^ p);
    }
    h
}

#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: mix(seed) }
    }

    pub fn from_keys(parts: &[u64]) -> Self {
        Rng::new(key(parts))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    /// Uniform integer in [0, n).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    /// Standard normal (mean 0, spread 1), Box–Muller.
    pub fn normal(&mut self) -> f32 {
        let u1 = self.f32().max(1e-7);
        let u2 = self.f32();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    /// Index chosen in proportion to `weights`. Returns None if all are zero.
    pub fn weighted(&mut self, weights: &[f32]) -> Option<usize> {
        let total: f32 = weights.iter().sum();
        if total <= 0.0 {
            return None;
        }
        let mut t = self.f32() * total;
        for (i, w) in weights.iter().enumerate() {
            if t < *w {
                return Some(i);
            }
            t -= w;
        }
        Some(weights.len() - 1)
    }
}
