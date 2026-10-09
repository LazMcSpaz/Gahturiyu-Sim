//! The land is made by wearing a block of rock down: the sea undercuts and
//! collapses the coast (fast through soft rock and cracks, slow through
//! hard), rain and streams cut valleys, and anything steeper than its rock
//! can hold slumps downhill. Nothing here is drawn; the shapes come out of
//! the rock recipe (`geology.rs`) and the time the processes run.

use crate::geology::{column, Rock, Strata};
use crate::noise::{noise, smooth};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::rng::Rng;

/// What the surface is, for colouring.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Turf,
    Rock,
    Scree,
    Shingle,
    /// Wave-washed rock at the tideline.
    Wet,
}

/// Where a stream comes into the site from the mountains behind.
#[derive(Clone, Copy, Debug)]
pub struct Inflow {
    pub at: V2,
    /// Droplets per epoch; the stream's size.
    pub flow: u32,
}

pub struct Params {
    /// Epochs of weathering to run.
    pub epochs: u32,
    /// How fast streams cut down into rock.
    pub stream_cut: f32,
    /// Wave strength: how fast an exposed soft coast is notched.
    pub waves: f32,
    /// Rain droplets per epoch.
    pub rain: u32,
    pub inflows: Vec<Inflow>,
    /// Which way the swell comes from (0 = from the east, 90 = from the south).
    pub swell_from_deg: f32,
}

pub struct Land {
    pub x0: f32,
    pub y0: f32,
    pub cell: f32,
    pub w: usize,
    pub h: usize,
    pub z: Vec<f32>,
    pub sea: Vec<bool>,
    /// How undercut a coastal cell is; it collapses at 1.
    notch: Vec<f32>,
    /// Loose material lying on the rock: beach and river sediment.
    pub sediment: Vec<f32>,
    /// Fallen rock lying on a slope.
    pub scree: Vec<f32>,
    /// How much rain water passed over the cell lately (for streams).
    pub wetness: Vec<f32>,
    /// How much land drains through each cell (cells), for streams.
    pub drain: Vec<f32>,
    /// 1 inside the site, fading to 0 where the world's own land must stay.
    pub site: Vec<f32>,
    shift: Vec<f32>,
    factor: Vec<f32>,
    body: Vec<f32>,
    cracks: Vec<f32>,
    strata: Strata,
    rock_seed: u64,
}

const SEA_LEVEL: f32 = 0.0;
/// The wave-cut platform the sea leaves behind, under low water.
const PLATFORM: f32 = -1.4;
/// Waves reach this far up a cliff foot.
const WAVE_BAND: f32 = 3.0;

impl Land {
    /// Start from a surface: `surface(p)` gives the uneroded height, `is_sea(p)` the open water.
    pub fn new(x0: f32, y0: f32, cell: f32, w: usize, h: usize, rock: &Rock, surface: impl Fn(V2) -> (f32, f32) + Sync) -> Land {
        let n = w * h;
        let mut z = vec![0.0f32; n];
        let mut site = vec![0.0f32; n];
        let mut shift = vec![0.0f32; n];
        let mut factor = vec![0.0f32; n];
        let mut body = vec![0.0f32; n];
        let mut cracks = vec![0.0f32; n];
        let threads = std::thread::available_parallelism().map(|t| t.get()).unwrap_or(2);
        let rows = h.div_ceil(threads);
        std::thread::scope(|s| {
            for (k, (((((zc, sc), fc), mc), bc), cc)) in z.chunks_mut(rows * w).zip(shift.chunks_mut(rows * w)).zip(factor.chunks_mut(rows * w)).zip(site.chunks_mut(rows * w)).zip(body.chunks_mut(rows * w)).zip(cracks.chunks_mut(rows * w)).enumerate() {
                let surface = &surface;
                s.spawn(move || {
                    for i in 0..zc.len() {
                        let (ix, iy) = (i % w, k * rows + i / w);
                        let p = V2::new(x0 + ix as f32 * cell, y0 + iy as f32 * cell);
                        let (zz, m) = surface(p);
                        zc[i] = zz;
                        mc[i] = m;
                        let c = column(rock, p);
                        sc[i] = c.shift;
                        fc[i] = c.factor;
                        bc[i] = c.body;
                        cc[i] = c.cracks;
                    }
                });
            }
        });
        let sea: Vec<bool> = z.iter().map(|&v| v < SEA_LEVEL).collect();
        Land {
            x0,
            y0,
            cell,
            w,
            h,
            z,
            sea,
            notch: vec![0.0; n],
            sediment: vec![0.0; n],
            scree: vec![0.0; n],
            wetness: vec![0.0; n],
            drain: vec![0.0; n],
            site,
            shift,
            factor,
            body,
            cracks,
            strata: Strata::new(rock),
            rock_seed: rock.seed,
        }
    }

    #[inline]
    fn idx(&self, i: usize, j: usize) -> usize {
        j * self.w + i
    }

    pub fn pos(&self, k: usize) -> V2 {
        V2::new(self.x0 + (k % self.w) as f32 * self.cell, self.y0 + (k / self.w) as f32 * self.cell)
    }

    /// Hardness of the rock at cell `k`, height `zz` (0 soft .. 1 hard).
    /// Loose sediment on top is soft.
    #[inline]
    pub fn hardness(&self, k: usize, zz: f32) -> f32 {
        // Loose stuff lies on top of the rock, only as deep as there is of it.
        let depth = self.z[k] - zz;
        if self.sediment[k] > 0.3 && depth < self.sediment[k] {
            return 0.06;
        }
        if self.scree[k] > 0.5 && depth < self.scree[k] {
            return 0.12;
        }
        let layered = (self.strata.at(zz - self.shift[k]) * self.factor[k]).clamp(0.03, 1.0);
        let solid = (0.96 * self.cracks[k]).max(0.03);
        layered + (solid - layered) * self.body[k]
    }

    /// Hardness of the rock at a point and height.
    pub fn hardness_at(&self, p: V2, z: f32) -> f32 {
        if !self.inside(p) {
            return 0.5;
        }
        self.hardness(self.cell_of(p), z)
    }

    pub fn inside(&self, p: V2) -> bool {
        p.x >= self.x0 && p.y >= self.y0 && p.x < self.x0 + (self.w - 1) as f32 * self.cell && p.y < self.y0 + (self.h - 1) as f32 * self.cell
    }

    pub fn height(&self, p: V2) -> f32 {
        let fx = ((p.x - self.x0) / self.cell).clamp(0.0, (self.w - 2) as f32);
        let fy = ((p.y - self.y0) / self.cell).clamp(0.0, (self.h - 2) as f32);
        let (i, j) = (fx.floor() as usize, fy.floor() as usize);
        let (tx, ty) = (fx - i as f32, fy - j as f32);
        let k = self.idx(i, j);
        let a = self.z[k] + (self.z[k + 1] - self.z[k]) * tx;
        let b = self.z[k + self.w] + (self.z[k + self.w + 1] - self.z[k + self.w]) * tx;
        a + (b - a) * ty
    }

    pub fn cell_of(&self, p: V2) -> usize {
        let i = (((p.x - self.x0) / self.cell).round().max(0.0) as usize).min(self.w - 1);
        let j = (((p.y - self.y0) / self.cell).round().max(0.0) as usize).min(self.h - 1);
        self.idx(i, j)
    }

    /// Run the whole history.
    pub fn weather(&mut self, p: &Params) {
        let mut rng = Rng::from_keys(&[self.rock_seed, 0x4552_4F44]);
        for e in 0..p.epochs {
            self.waves(p.waves, p.swell_from_deg);
            self.rain(&mut rng, p.rain, &p.inflows, e);
            if e % 2 == 0 {
                self.streams(&p.inflows, p.stream_cut);
            }
            self.slump();
            self.creep();
            if e % 25 == 0 {
                eprintln!("  epoch {e}/{}", p.epochs);
            }
        }
        // A last settling so nothing is left mid-collapse.
        for _ in 0..6 {
            self.slump();
        }
        self.silt_hollows(3.0);
    }

    /// Hollows that would hold water have long since silted up to their
    /// spill point (the shallow ones, up to `max` metres deep); deeper ones
    /// stay as lochans.
    pub fn silt_hollows(&mut self, max: f32) {
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;
        #[derive(PartialEq)]
        struct Item(f32, u32);
        impl Eq for Item {}
        impl PartialOrd for Item {
            fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
                Some(self.cmp(o))
            }
        }
        impl Ord for Item {
            fn cmp(&self, o: &Self) -> Ordering {
                o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
            }
        }
        let (w, h) = (self.w, self.h);
        let n = w * h;
        let mut filled = vec![f32::NAN; n];
        let mut heap = BinaryHeap::new();
        for k in 0..n {
            let (i, j) = (k % w, k / w);
            if self.sea[k] || i == 0 || j == 0 || i == w - 1 || j == h - 1 {
                filled[k] = self.z[k];
                heap.push(Item(self.z[k], k as u32));
            }
        }
        while let Some(Item(lvl, k)) = heap.pop() {
            let k = k as usize;
            if filled[k] < lvl {
                continue;
            }
            let (i, j) = (k % w, k / w);
            if i == 0 || j == 0 || i >= w - 1 || j >= h - 1 {
                continue;
            }
            for nb in [k - 1, k + 1, k - w, k + w] {
                if filled[nb].is_nan() {
                    filled[nb] = self.z[nb].max(lvl);
                    heap.push(Item(filled[nb], nb as u32));
                }
            }
        }
        for k in 0..n {
            if !self.sea[k] && filled[k].is_finite() && filled[k] > self.z[k] && filled[k] - self.z[k] <= max {
                self.sediment[k] += filled[k] - self.z[k];
                self.z[k] = filled[k] + 0.01;
            }
        }
    }

    // ---- The sea ------------------------------------------------------------

    /// How open to the swell a coastal cell is: 0 sheltered .. 1 open sea.
    /// Waves funnel down narrow inlets, so one long open line to the sea
    /// counts for a lot, not just the average openness.
    pub fn exposure(&self, k: usize, swell_from_deg: f32) -> f32 {
        let (ci, cj) = ((k % self.w) as i32, (k / self.w) as i32);
        let reach = 60i32;
        let mut sum = 0.0;
        let mut wsum = 0.0;
        let mut best = 0.0f32;
        for d in 0..24 {
            let a = d as f32 * std::f32::consts::TAU / 24.0;
            let (dx, dy) = (a.cos(), a.sin());
            // Rays toward where the swell comes from count most.
            let toward = ((a - swell_from_deg.to_radians()).cos() + 1.0) * 0.5;
            let wgt = 0.2 + toward * toward;
            let mut open = 0;
            for t in 1..=reach {
                let (x, y) = (ci + (dx * t as f32).round() as i32, cj + (dy * t as f32).round() as i32);
                if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
                    open = reach;
                    break;
                }
                if !self.sea[self.idx(x as usize, y as usize)] {
                    break;
                }
                open = t;
            }
            let frac = open as f32 / reach as f32;
            sum += wgt * frac;
            wsum += wgt;
            best = best.max(frac * (0.4 + 0.6 * toward));
        }
        0.4 * sum / wsum + 0.6 * best
    }

    fn waves(&mut self, strength: f32, swell_from_deg: f32) {
        let (w, h) = (self.w, self.h);
        let mut falls = Vec::new();
        for j in 1..h - 1 {
            for i in 1..w - 1 {
                let k = self.idx(i, j);
                if self.sea[k] || self.z[k] > 90.0 {
                    continue;
                }
                let coastal = [k - 1, k + 1, k - w, k + w].iter().any(|&n| self.sea[n]);
                if !coastal {
                    continue;
                }
                let ex = self.exposure(k, swell_from_deg);
                if ex < 0.02 {
                    continue;
                }
                // The waves work on the rock at the foot of the cliff, but
                // the whole face has to come down: the toughest layer in
                // the wave band and the face's average both count.
                let mut foot = 0.0f32;
                for s in 0..4 {
                    foot = foot.max(self.hardness(k, SEA_LEVEL + s as f32 * WAVE_BAND * 0.5));
                }
                let top = self.z[k].max(2.0);
                let mut face = 0.0;
                let steps = 6;
                for s in 0..steps {
                    face += self.hardness(k, top * (s as f32 + 0.5) / steps as f32);
                }
                let face = face / steps as f32;
                // Even the hardest rock gives way in the end.
                let soft = (1.0 - (0.5 * foot + 0.5 * face)).max(0.14);
                // A high cliff needs a deeper notch before it falls.
                let need = 1.0 + (self.z[k].max(0.0) / 150.0).min(0.4);
                self.notch[k] += strength * ex * soft * soft * self.site[k];
                if self.notch[k] >= need {
                    falls.push(k);
                }
            }
        }
        // The sea slowly grinds its shallow floor down, so old platform
        // slopes away from the cliff.
        for k in 0..self.z.len() {
            if self.sea[k] && self.z[k] > -9.0 {
                self.z[k] -= 0.012;
            }
        }
        for k in falls {
            let fallen = self.z[k] - PLATFORM;
            self.z[k] = PLATFORM + (noise(self.rock_seed ^ 0x77, self.pos(k).x, self.pos(k).y, 6.0) - 0.5) * 0.8;
            self.sea[k] = true;
            self.notch[k] = 0.0;
            self.sediment[k] = 0.0;
            // Rubble spreads onto the sea floor around; most is carried off.
            let share = fallen * 0.08;
            for n in [k - 1, k + 1, k - w, k + w] {
                if self.sea[n] {
                    self.z[n] = (self.z[n] + share).min(SEA_LEVEL - 0.4);
                }
            }
        }
    }

    // ---- Rain and streams ---------------------------------------------------

    fn rain(&mut self, rng: &mut Rng, drops: u32, inflows: &[Inflow], epoch: u32) {
        // Rain falls more on the high ground behind, which is where it drains from.
        let (w, h) = (self.w as f32, self.h as f32);
        for _ in 0..drops {
            let (x, y) = (rng.range(1.0, w - 2.0), rng.range(1.0, h - 2.0));
            let k = self.idx(x as usize, y as usize);
            if self.sea[k] {
                continue;
            }
            let hz = self.z[k];
            if rng.f32() > 0.25 + 0.75 * smooth(0.0, 200.0, hz) {
                continue;
            }
            self.droplet(x, y, 1.0, rng, 0.25);
        }
        for f in inflows {
            if !self.inside(f.at) {
                continue;
            }
            let k = self.cell_of(f.at);
            let (ci, cj) = ((k % self.w) as f32, (k / self.w) as f32);
            for n in 0..f.flow {
                let a = (epoch * 131 + n) as f32 * 0.37;
                self.droplet(ci + a.cos() * 2.0, cj + a.sin() * 2.0, 4.0, rng, 0.4);
            }
        }
        for v in self.wetness.iter_mut() {
            *v *= 0.97;
        }
    }

    /// One drop of water running downhill, picking up and dropping ground.
    fn droplet(&mut self, mut x: f32, mut y: f32, mut water: f32, rng: &mut Rng, erode: f32) {
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        let mut speed = 1.0f32;
        let mut sediment = 0.0f32;
        let (inertia, capacity, min_slope, deposit, evap, gravity) = (0.1, 6.0, 0.01, 0.3, 0.012, 4.0);
        for _ in 0..700 {
            let (i, j) = (x as usize, y as usize);
            if i < 1 || j < 1 || i >= self.w - 2 || j >= self.h - 2 {
                return;
            }
            let (fx, fy) = (x - i as f32, y - j as f32);
            let k = self.idx(i, j);
            let (z00, z10, z01, z11) = (self.z[k], self.z[k + 1], self.z[k + self.w], self.z[k + self.w + 1]);
            let gx = (z10 - z00) * (1.0 - fy) + (z11 - z01) * fy;
            let gy = (z01 - z00) * (1.0 - fx) + (z11 - z10) * fx;
            dx = dx * inertia - gx * (1.0 - inertia);
            dy = dy * inertia - gy * (1.0 - inertia);
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-5 {
                let a = rng.range(0.0, std::f32::consts::TAU);
                dx = a.cos();
                dy = a.sin();
            } else {
                dx /= len;
                dy /= len;
            }
            let old = z00 * (1.0 - fx) * (1.0 - fy) + z10 * fx * (1.0 - fy) + z01 * (1.0 - fx) * fy + z11 * fx * fy;
            self.wetness[k] += water;
            let (nx, ny) = (x + dx, y + dy);
            let (ni, nj) = (nx as usize, ny as usize);
            if ni < 1 || nj < 1 || ni >= self.w - 2 || nj >= self.h - 2 {
                return;
            }
            let nk = self.idx(ni, nj);
            if self.sea[nk] {
                // Into the sea: the load drops as a beach, which waves will sort out.
                let under = [nk, nk + 1, nk + self.w, nk + self.w + 1];
                for u in under {
                    self.z[u] += sediment * 0.25;
                    self.sediment[u] += sediment * 0.25;
                    if self.z[u] > SEA_LEVEL + 0.3 {
                        self.sea[u] = false;
                    }
                }
                return;
            }
            let (nfx, nfy) = (nx - ni as f32, ny - nj as f32);
            let (n00, n10, n01, n11) = (self.z[nk], self.z[nk + 1], self.z[nk + self.w], self.z[nk + self.w + 1]);
            let new = n00 * (1.0 - nfx) * (1.0 - nfy) + n10 * nfx * (1.0 - nfy) + n01 * (1.0 - nfx) * nfy + n11 * nfx * nfy;
            let dh = new - old;
            let cap = (-dh).max(min_slope) * speed * water * capacity;
            if sediment > cap || dh > 0.0 {
                let drop = if dh > 0.0 { sediment.min(dh) } else { (sediment - cap) * deposit };
                sediment -= drop;
                let wts = [(k, (1.0 - fx) * (1.0 - fy)), (k + 1, fx * (1.0 - fy)), (k + self.w, (1.0 - fx) * fy), (k + self.w + 1, fx * fy)];
                for (c, wt) in wts {
                    self.z[c] += drop * wt;
                    self.sediment[c] += drop * wt;
                }
            } else {
                let hard = self.hardness(k, old);
                let amount = ((cap - sediment) * erode * (1.0 - 0.92 * hard) * self.site[k]).min(-dh);
                // Wear a small patch, not a point.
                let wts = [(k, 0.4), (k + 1, 0.15), (k + self.w, 0.15), (k + self.w + 1, 0.1), (k - 1, 0.1), (k - self.w, 0.1)];
                for (c, wt) in wts {
                    let take = amount * wt;
                    self.z[c] -= take;
                    self.sediment[c] = (self.sediment[c] - take).max(0.0);
                }
                sediment += amount;
            }
            speed = (speed * speed + dh.abs() * gravity).sqrt().max(0.2);
            water *= 1.0 - evap;
            x = nx;
            y = ny;
            if water < 0.05 {
                return;
            }
        }
    }

    // ---- Streams ------------------------------------------------------------

    /// Work out where water collects and cut the channels: each cell passes
    /// its water to its lowest neighbour, and a cell wears down by how much
    /// water crosses it times how steep it is, slower in hard rock. Pits
    /// silt up a little each time, so lakes drain in the end.
    fn streams(&mut self, inflows: &[Inflow], cut: f32) {
        let (w, h) = (self.w, self.h);
        let n = w * h;
        // Fill every hollow to its spill level first, so water always has
        // a way to the sea; the order the flood reaches cells is downhill
        // on that filled surface.
        use std::cmp::Ordering;
        use std::collections::BinaryHeap;
        #[derive(PartialEq)]
        struct Item(f32, u32, u32);
        impl Eq for Item {}
        impl PartialOrd for Item {
            fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
                Some(self.cmp(o))
            }
        }
        impl Ord for Item {
            fn cmp(&self, o: &Self) -> Ordering {
                o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
            }
        }
        let mut filled = vec![f32::NAN; n];
        let mut parent = vec![u32::MAX; n];
        let mut heap = BinaryHeap::new();
        for k in 0..n {
            let (i, j) = (k % w, k / w);
            if self.sea[k] || i == 0 || j == 0 || i == w - 1 || j == h - 1 {
                filled[k] = self.z[k];
                heap.push(Item(self.z[k], k as u32, u32::MAX));
            }
        }
        let mut order: Vec<u32> = Vec::with_capacity(n);
        while let Some(Item(lvl, k, from)) = heap.pop() {
            let k = k as usize;
            if parent[k] != u32::MAX || (from == u32::MAX && !self.sea[k] && order.contains(&(k as u32))) {
                continue;
            }
            if from != u32::MAX {
                if filled[k] < lvl {
                    continue;
                }
                parent[k] = from;
                order.push(k as u32);
            }
            let (i, j) = (k % w, k / w);
            if i == 0 || j == 0 || i >= w - 1 || j >= h - 1 {
                continue;
            }
            for nb in [k - 1, k + 1, k - w, k + w, k - w - 1, k - w + 1, k + w - 1, k + w + 1] {
                if filled[nb].is_nan() {
                    filled[nb] = self.z[nb].max(lvl);
                    heap.push(Item(filled[nb], nb as u32, k as u32));
                }
            }
        }
        // Water flows from the last-reached cells back toward the sea.
        let mut acc = vec![1.0f32; n];
        for f in inflows {
            if self.inside(f.at) {
                let k = self.cell_of(f.at);
                acc[k] += f.flow as f32 * 400.0;
            }
        }
        let mut down = vec![u32::MAX; n];
        let mut slope = vec![0.0f32; n];
        for &k in order.iter().rev() {
            let k = k as usize;
            let zk = self.z[k];
            let mut best = (0.0f32, usize::MAX);
            if filled[k] <= zk + 1e-4 {
                for (nb, dist) in [(k - 1, 1.0), (k + 1, 1.0), (k - w, 1.0), (k + w, 1.0), (k - w - 1, 1.414), (k - w + 1, 1.414), (k + w - 1, 1.414), (k + w + 1, 1.414)] {
                    let s = (zk - self.z[nb]) / (dist * self.cell);
                    if s > best.0 && filled[nb] <= self.z[nb] + 1e-4 {
                        best = (s, nb);
                    }
                }
            }
            let d = if best.1 != usize::MAX { best.1 } else { parent[k] as usize };
            down[k] = d as u32;
            slope[k] = best.0;
            acc[d] += acc[k];
        }
        let lake: Vec<bool> = (0..n).map(|k| filled[k] > self.z[k] + 1e-4).collect();
        let order: Vec<u32> = order;
        for &k in &order {
            let k = k as usize;
            self.drain[k] = acc[k];
            if lake[k] {
                // Under a lake: silt settles, and the water cuts the rim at
                // the outlet (that's the parent chain), so lakes drain in time.
                self.z[k] = (self.z[k] + 0.03 * self.site[k]).min(filled[k]);
                self.sediment[k] += 0.03;
                continue;
            }
            if acc[k] < 30.0 {
                continue;
            }
            let hard = self.hardness(k, self.z[k] - 0.3);
            let power = cut * acc[k].sqrt() * slope[k] * (1.0 - 0.9 * hard) * self.site[k];
            let take = power.min(1.0).min((self.z[k] - self.z[down[k] as usize]).max(0.0) * 0.9);
            self.z[k] -= take;
            self.sediment[k] = (self.sediment[k] - take).max(0.0);
            // Streams carry their load to the sea: it lands on the first sea cells.
            let d = down[k] as usize;
            if self.sea[d] {
                self.z[d] = (self.z[d] + take * 0.5).min(SEA_LEVEL - 0.2);
                self.sediment[d] += take * 0.5;
            }
        }
    }

    // ---- Gravity ------------------------------------------------------------

    /// Ground steeper than its rock can hold slides downhill. Hard rock
    /// stands near vertical; soft rock and loose material rest at about 35°.
    fn slump(&mut self) {
        let (w, h, cell) = (self.w, self.h, self.cell);
        let mut delta = vec![0.0f32; w * h];
        let mut fell = vec![0.0f32; w * h];
        for j in 1..h - 1 {
            for i in 1..w - 1 {
                let k = self.idx(i, j);
                let zk = self.z[k];
                let hard = self.hardness(k, zk - 0.5);
                let loose = self.sediment[k] > 0.3 || self.scree[k] > 0.5;
                let angle = if loose { 33.0 } else { 34.0 + 46.0 * hard * hard };
                let limit = angle.to_radians().tan();
                for (n, dist) in [(k - 1, cell), (k + 1, cell), (k - w, cell), (k + w, cell), (k - w - 1, cell * 1.414), (k - w + 1, cell * 1.414), (k + w - 1, cell * 1.414), (k + w + 1, cell * 1.414)] {
                    let drop = zk - self.z[n];
                    let over = drop - limit * dist;
                    if over > 0.0 {
                        let moved = over * 0.12;
                        delta[k] -= moved;
                        delta[n] += moved;
                        fell[n] += moved;
                    }
                }
            }
        }
        for k in 0..w * h {
            self.z[k] += delta[k];
            if delta[k] < 0.0 {
                self.scree[k] = (self.scree[k] + delta[k]).max(0.0);
            }
            if fell[k] > 0.0 {
                // Rock that falls into the sea is washed around; on land it lies as scree.
                if self.sea[k] {
                    self.z[k] = self.z[k].min(SEA_LEVEL - 0.3);
                } else {
                    self.scree[k] += fell[k];
                }
            }
            self.scree[k] *= 0.995;
        }
    }

    /// Soil creep: on gentle ground the soil smooths everything over, so
    /// hills are rounded rather than stepped. Cliffs and crags are left alone.
    fn creep(&mut self) {
        let (w, h) = (self.w, self.h);
        let mut delta = vec![0.0f32; w * h];
        for j in 1..h - 1 {
            for i in 1..w - 1 {
                let k = self.idx(i, j);
                if self.sea[k] {
                    continue;
                }
                let zk = self.z[k];
                let mean = (self.z[k - 1] + self.z[k + 1] + self.z[k - w] + self.z[k + w]) * 0.25;
                let d = mean - zk;
                // Steep ground is bare rock: no soil to creep.
                let steep = ((self.z[k + 1] - self.z[k - 1]).abs().max((self.z[k + w] - self.z[k - w]).abs())) / (2.0 * self.cell);
                let soil = 1.0 - smooth(0.45, 0.8, steep);
                delta[k] = d * 0.12 * soil * self.site[k];
            }
        }
        for k in 0..w * h {
            self.z[k] += delta[k];
        }
    }

    // ---- Reading the result -------------------------------------------------

    pub fn slope_deg(&self, k: usize) -> f32 {
        let (i, j) = (k % self.w, k / self.w);
        if i == 0 || j == 0 || i >= self.w - 1 || j >= self.h - 1 {
            return 0.0;
        }
        let dx = (self.z[k + 1] - self.z[k - 1]) / (2.0 * self.cell);
        let dy = (self.z[k + self.w] - self.z[k - self.w]) / (2.0 * self.cell);
        (dx * dx + dy * dy).sqrt().atan().to_degrees()
    }

    pub fn kind(&self, k: usize) -> Kind {
        let z = self.z[k];
        let slope = self.slope_deg(k);
        if z < SEA_LEVEL {
            return if self.sediment[k] > 0.4 { Kind::Shingle } else { Kind::Wet };
        }
        if z < WAVE_BAND * 0.6 {
            return if self.sediment[k] > 0.4 { Kind::Shingle } else { Kind::Wet };
        }
        if self.sediment[k] > 0.6 && slope < 20.0 && z < 5.0 {
            return Kind::Shingle;
        }
        if self.scree[k] > 0.6 && slope < 40.0 {
            return Kind::Scree;
        }
        let hard = self.hardness(k, z - 0.3);
        if slope > 28.0 + 20.0 * hard || (slope > 22.0 && hard > 0.8) {
            return Kind::Rock;
        }
        // Hard rock on a convex crown sheds its thin soil: outcrops in patches.
        if hard > 0.72 && self.sediment[k] < 0.25 {
            let (i, j) = (k % self.w, k / self.w);
            if i >= 4 && j >= 4 && i + 4 < self.w && j + 4 < self.h {
                let around = (self.z[k - 4] + self.z[k + 4] + self.z[k - 4 * self.w] + self.z[k + 4 * self.w]) * 0.25;
                let convex = z - around;
                if convex > 0.35 + (1.0 - hard) * 2.0 && slope > 6.0 {
                    return Kind::Rock;
                }
            }
        }
        Kind::Turf
    }

    /// Is there a stream here (0..1 by its size)?
    pub fn stream(&self, k: usize) -> f32 {
        smooth(8000.0, 20000.0, self.drain[k])
    }
}
