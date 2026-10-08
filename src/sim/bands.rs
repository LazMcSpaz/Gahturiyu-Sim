//! Simulation bands: how much attention each part of the map gets.
//!
//! The map is cut into square chunks. Each chunk is assigned a band by how far
//! its centre is from the squad, and everything inside a chunk shares that
//! band. That way the question "how closely do I simulate this?" is one lookup,
//! not a distance check against every person in the world.
//!
//! - Band 1, near: everyone is an individual, with a name and gear.
//! - Band 2, middle: groups are simulated, but as groups, refreshed every few seconds.
//! - Band 3, far: groups are numbers on a schedule, refreshed once a game-minute.

use super::geo::{V2, WORLD_SIZE};

/// Side of one chunk, in metres.
pub const CHUNK: f32 = 250.0;
/// Chunks along one side of the map.
pub const CHUNKS: usize = (WORLD_SIZE / CHUNK) as usize + 1;

/// Outer edge of band 1, metres from the squad.
pub const BAND1_RADIUS: f32 = 500.0;
/// Outer edge of band 2. Everything beyond is band 3.
pub const BAND2_RADIUS: f32 = 2500.0;

/// How often a group in each band has its position refreshed, in game seconds.
/// Band 1 is every step. These are the knobs that buy speed.
pub const REFRESH: [f64; 4] = [0.0, 0.0, 5.0, 60.0];

#[derive(Clone, Debug)]
pub struct BandMap {
    bands: Vec<u8>,
    /// The chunk the squad was in when the map was last rebuilt.
    centre: (usize, usize),
}

pub fn chunk_of(p: V2) -> (usize, usize) {
    let cx = (p.x / CHUNK).floor().clamp(0.0, (CHUNKS - 1) as f32) as usize;
    let cy = (p.y / CHUNK).floor().clamp(0.0, (CHUNKS - 1) as f32) as usize;
    (cx, cy)
}

impl BandMap {
    pub fn new(squad: V2) -> BandMap {
        let mut m = BandMap { bands: vec![3; CHUNKS * CHUNKS], centre: (usize::MAX, usize::MAX) };
        m.update(squad);
        m
    }

    /// Rebuild only when the squad crosses into a new chunk.
    pub fn update(&mut self, squad: V2) -> bool {
        let c = chunk_of(squad);
        if c == self.centre {
            return false;
        }
        self.centre = c;
        let centre = V2::new((c.0 as f32 + 0.5) * CHUNK, (c.1 as f32 + 0.5) * CHUNK);
        for cy in 0..CHUNKS {
            for cx in 0..CHUNKS {
                let mid = V2::new((cx as f32 + 0.5) * CHUNK, (cy as f32 + 0.5) * CHUNK);
                let d = mid.dist(centre);
                self.bands[cy * CHUNKS + cx] = if d <= BAND1_RADIUS {
                    1
                } else if d <= BAND2_RADIUS {
                    2
                } else {
                    3
                };
            }
        }
        true
    }

    pub fn band_at(&self, p: V2) -> u8 {
        let (cx, cy) = chunk_of(p);
        self.bands[cy * CHUNKS + cx]
    }

    pub fn band_of_chunk(&self, cx: usize, cy: usize) -> u8 {
        self.bands[cy * CHUNKS + cx]
    }
}
