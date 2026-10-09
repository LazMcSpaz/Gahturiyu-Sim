//! Which climate regions each part of the map belongs to, worked out once
//! per world on a coarse grid, so the drawing can ask about thousands of
//! spots a frame. (Regions fade into each other over about 200 m, so a grid
//! this coarse loses nothing.)

use gahturiyu_sim::sim::geo::{V2, WORLD_SIZE};
use gahturiyu_sim::sim::terrain::Terrain;
use gahturiyu_sim::sim::weather::{self, Mix, REGIONS};

/// Grid points a side.
const N: usize = 151;

/// Which world a cache belongs to: (seed, loads, land edits).
pub type Stamp = (u64, u32, u32);

#[derive(Default)]
pub struct LandGrid {
    built: Option<Stamp>,
    shares: Vec<Mix>,
}

impl LandGrid {
    pub fn is_for(&self, stamp: Stamp) -> bool {
        self.built == Some(stamp)
    }

    pub fn build(&mut self, terrain: &Terrain, stamp: Stamp) {
        let step = WORLD_SIZE / (N - 1) as f32;
        self.shares.clear();
        self.shares.reserve(N * N);
        for j in 0..N {
            for i in 0..N {
                self.shares.push(weather::mix(terrain, V2::new(i as f32 * step, j as f32 * step)));
            }
        }
        self.built = Some(stamp);
    }

    /// The region shares at a spot (read between the grid's points).
    pub fn mix(&self, p: V2) -> Mix {
        if self.shares.is_empty() {
            let mut m = [0.0; REGIONS];
            m[weather::Region::Lowland as usize] = 1.0;
            return m;
        }
        let step = WORLD_SIZE / (N - 1) as f32;
        let fx = (p.x / step).clamp(0.0, (N - 1) as f32 - 1e-3);
        let fy = (p.y / step).clamp(0.0, (N - 1) as f32 - 1e-3);
        let (i, j) = (fx as usize, fy as usize);
        let (u, v) = (fx - i as f32, fy - j as f32);
        let at = |i: usize, j: usize| &self.shares[j * N + i];
        let (a, b, c, d) = (at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1));
        let mut out = [0.0; REGIONS];
        for k in 0..REGIONS {
            out[k] = (a[k] * (1.0 - u) + b[k] * u) * (1.0 - v) + (c[k] * (1.0 - u) + d[k] * u) * v;
        }
        out
    }
}
