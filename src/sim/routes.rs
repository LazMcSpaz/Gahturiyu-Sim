//! Roads: the way people actually get between towns.
//!
//! Built once at world creation by searching the land for the cheapest walk
//! between towns, where "cheap" means short, not too steep, and dry. Each
//! town is first linked to its nearest neighbours; ground that already
//! carries a road is cheaper to use, so later routes bend onto existing ones
//! and the links merge into a network of trunk roads instead of a web of
//! straight lines. Travellers then follow these routes instead of walking
//! through mountains.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::geo::{self, V2, WORLD_SIZE};
use super::settlement::{Settlement, SettlementId};
use super::terrain::{walk_factor, Terrain};

/// Spacing of the search grid, metres.
const GRID: f32 = 120.0;
const G: usize = (WORLD_SIZE / GRID) as usize + 1;
/// Each town is linked by road to this many of its nearest neighbours.
const NEIGHBOURS: usize = 3;
/// Cost multiplier on ground that already carries a road.
const ROAD_DISCOUNT: f32 = 0.5;
/// Grades steeper than this can't be walked at all.
const MAX_GRADE: f32 = 0.42;

#[derive(Clone, Debug, Default)]
pub struct Routes {
    /// Smoothed route between each pair of towns, stored once with the lower
    /// id first.
    paths: HashMap<(SettlementId, SettlementId), Vec<V2>>,
    /// The roads worth drawing: the neighbour links the network grew from.
    pub roads: Vec<Vec<V2>>,
}

impl Routes {
    /// The route from town `a` to town `b`, centre to centre.
    pub fn between(&self, a: SettlementId, b: SettlementId) -> Vec<V2> {
        if a < b {
            self.paths.get(&(a, b)).cloned().unwrap_or_default()
        } else {
            let mut p = self.paths.get(&(b, a)).cloned().unwrap_or_default();
            p.reverse();
            p
        }
    }

    pub fn build(terrain: &Terrain, towns: &[Settlement]) -> Routes {
        let idx = |i: usize, j: usize| j * G + i;
        let centre = |i: usize, j: usize| V2::new(i as f32 * GRID, j as f32 * GRID);
        let height: Vec<f32> = (0..G * G).map(|k| terrain.height(centre(k % G, k / G))).collect();
        // The steepest bit of ground along each step out of each cell, read
        // off the fine terrain: a coarse grid alone would step straight over
        // a cliff that sits between two of its points.
        let sub = |a: V2, b: V2| -> f32 {
            let mut worst = 0.0f32;
            let n = 4;
            for k in 0..n {
                let p = a.lerp(b, k as f32 / n as f32);
                let q = a.lerp(b, (k + 1) as f32 / n as f32);
                worst = worst.max((terrain.height(q) - terrain.height(p)).abs() / p.dist(q));
            }
            worst
        };
        let steep: Vec<[f32; 8]> = (0..G * G)
            .map(|k| {
                let (i, j) = ((k % G) as i32, (k / G) as i32);
                let mut out = [f32::INFINITY; 8];
                for (n, (di, dj, _)) in STEPS.iter().enumerate() {
                    let (vi, vj) = (i + di, j + dj);
                    if vi >= 0 && vj >= 0 && vi < G as i32 && vj < G as i32 {
                        out[n] = sub(centre(i as usize, j as usize), centre(vi as usize, vj as usize));
                    }
                }
                out
            })
            .collect();
        let dry: Vec<bool> = (0..G * G).map(|k| geo::inland(centre(k % G, k / G)) > 15.0).collect();
        let mut road = vec![false; G * G];
        let cell_of = |p: V2| -> usize {
            let i = (p.x / GRID).round().clamp(0.0, (G - 1) as f32) as usize;
            let j = (p.y / GRID).round().clamp(0.0, (G - 1) as f32) as usize;
            // Nudge onto dry ground if the town centre's cell is wet.
            let mut best = idx(i, j);
            if !dry[best] {
                'f: for r in 1..6i64 {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let (ii, jj) = (i as i64 + di, j as i64 + dj);
                            if ii >= 0 && jj >= 0 && (ii as usize) < G && (jj as usize) < G && dry[idx(ii as usize, jj as usize)] {
                                best = idx(ii as usize, jj as usize);
                                break 'f;
                            }
                        }
                    }
                }
            }
            best
        };
        let nodes: Vec<usize> = towns.iter().map(|t| cell_of(t.pos)).collect();

        let mut out = Routes::default();
        let mut raw: HashMap<(SettlementId, SettlementId), Vec<usize>> = HashMap::new();

        // Which pairs get a road of their own: each town's nearest few.
        let mut links: Vec<(usize, usize)> = Vec::new();
        for a in 0..towns.len() {
            let mut near: Vec<usize> = (0..towns.len()).filter(|&b| b != a).collect();
            near.sort_by(|&x, &y| towns[a].pos.dist(towns[x].pos).total_cmp(&towns[a].pos.dist(towns[y].pos)));
            for &b in near.iter().take(NEIGHBOURS) {
                let pair = (a.min(b), a.max(b));
                if !links.contains(&pair) {
                    links.push(pair);
                }
            }
        }

        // Phase 1: grow the road network from the neighbour links.
        for a in 0..towns.len() {
            let wanted: Vec<usize> = links.iter().filter(|(x, _)| *x == a).map(|(_, y)| *y).collect();
            if wanted.is_empty() {
                continue;
            }
            let parent = search(nodes[a], &height, &steep, &dry, &road);
            for b in wanted {
                let cells = trace(&parent, nodes[a], nodes[b]);
                for &c in &cells {
                    road[c] = true;
                }
                raw.insert((a as SettlementId, b as SettlementId), cells);
            }
        }
        // Phase 2: every other pair, now leaning on the roads that exist.
        for a in 0..towns.len() {
            let parent = search(nodes[a], &height, &steep, &dry, &road);
            for b in a + 1..towns.len() {
                raw.entry((a as SettlementId, b as SettlementId)).or_insert_with(|| trace(&parent, nodes[a], nodes[b]));
            }
        }

        for (&(a, b), cells) in &raw {
            let mut pts: Vec<V2> = cells.iter().map(|&c| centre(c % G, c / G)).collect();
            if pts.is_empty() {
                // Unreachable: fall back to a straight line rather than nothing.
                pts = vec![towns[a as usize].pos, towns[b as usize].pos];
            }
            pts[0] = towns[a as usize].pos;
            let last = pts.len() - 1;
            pts[last] = towns[b as usize].pos;
            let pts = chaikin(chaikin(straighten(&pts, terrain)));
            if links.contains(&(a as usize, b as usize)) {
                out.roads.push(pts.clone());
            }
            out.paths.insert((a, b), pts);
        }
        out.roads.sort_by(|x, y| x[0].x.total_cmp(&y[0].x).then(x[0].y.total_cmp(&y[0].y)));
        out
    }
}

/// Cheapest-walk search over the grid from `start`. Returns each cell's
/// predecessor on its cheapest route back.
const STEPS: [(i32, i32, f32); 8] = [
    (1, 0, 1.0),
    (-1, 0, 1.0),
    (0, 1, 1.0),
    (0, -1, 1.0),
    (1, 1, std::f32::consts::SQRT_2),
    (1, -1, std::f32::consts::SQRT_2),
    (-1, 1, std::f32::consts::SQRT_2),
    (-1, -1, std::f32::consts::SQRT_2),
];

fn search(start: usize, height: &[f32], steep: &[[f32; 8]], dry: &[bool], road: &[bool]) -> Vec<u32> {
    let mut dist = vec![f32::INFINITY; G * G];
    let mut parent = vec![u32::MAX; G * G];
    let mut heap = BinaryHeap::new();
    dist[start] = 0.0;
    heap.push(Reverse((0u32, start as u32)));
    while let Some(Reverse((dbits, u))) = heap.pop() {
        let u = u as usize;
        let du = f32::from_bits(dbits);
        if du > dist[u] {
            continue;
        }
        let (ui, uj) = ((u % G) as i32, (u / G) as i32);
        for (n, &(di, dj, len)) in STEPS.iter().enumerate() {
            let (vi, vj) = (ui + di, uj + dj);
            if vi < 0 || vj < 0 || vi >= G as i32 || vj >= G as i32 {
                continue;
            }
            let v = vj as usize * G + vi as usize;
            if !dry[v] {
                continue;
            }
            let run = len * GRID;
            let grade = (height[v] - height[u]) / run;
            let worst = steep[u][n];
            if worst > MAX_GRADE {
                continue;
            }
            // Walking time, plus a premium on steep ground either way (people
            // avoid it even when they could manage it) and on high passes.
            let mut cost = run / walk_factor(grade) * (1.0 + 5.0 * (worst - 0.12).max(0.0)) * (1.0 + (height[v] - 350.0).max(0.0) / 400.0);
            if road[v] {
                cost *= ROAD_DISCOUNT;
            }
            let nd = du + cost;
            if nd < dist[v] {
                dist[v] = nd;
                parent[v] = u as u32;
                heap.push(Reverse((nd.to_bits(), v as u32)));
            }
        }
    }
    parent
}

fn trace(parent: &[u32], start: usize, goal: usize) -> Vec<usize> {
    if start == goal {
        return vec![start];
    }
    if parent[goal] == u32::MAX {
        return Vec::new();
    }
    let mut out = vec![goal];
    let mut c = goal;
    while c != start {
        c = parent[c] as usize;
        out.push(c);
        if out.len() > G * G {
            return Vec::new();
        }
    }
    out.reverse();
    out
}

/// The grid only allows eight directions, which leaves routes as staircases
/// and long north-south or east-west runs. Where the ground allows, replace a
/// run of grid points with one straight line at whatever angle it needs: the
/// line must stay dry, never get steeper than a road can be, and not cost much
/// more walking than the run it replaces.
fn straighten(p: &[V2], t: &Terrain) -> Vec<V2> {
    const WINDOW: usize = 14;
    let effort = |a: V2, b: V2| -> Option<f32> {
        let n = ((a.dist(b) / 30.0).ceil() as usize).max(1);
        let mut total = 0.0;
        for k in 0..n {
            let (x, y) = (a.lerp(b, k as f32 / n as f32), a.lerp(b, (k + 1) as f32 / n as f32));
            if geo::inland(y) < 15.0 && k + 1 < n {
                return None;
            }
            let len = x.dist(y).max(1e-3);
            let grade = (t.height(y) - t.height(x)) / len;
            if grade.abs() > MAX_GRADE * 0.9 {
                return None;
            }
            total += len / walk_factor(grade);
        }
        Some(total)
    };
    let mut out = vec![p[0]];
    let mut i = 0;
    while i < p.len() - 1 {
        let mut best = i + 1;
        let mut along = 0.0;
        let mut run = vec![0.0f32];
        for k in i + 1..p.len().min(i + WINDOW + 1) {
            along += t.effort(p[k - 1], p[k]);
            run.push(along);
        }
        for j in (i + 2..p.len().min(i + WINDOW + 1)).rev() {
            if let Some(e) = effort(p[i], p[j]) {
                if e <= run[j - i] * 1.08 {
                    best = j;
                    break;
                }
            }
        }
        out.push(p[best]);
        i = best;
    }
    out
}

/// Corner-cutting smoothing: turns a grid staircase into a gentle curve.
fn chaikin(p: Vec<V2>) -> Vec<V2> {
    if p.len() < 3 {
        return p;
    }
    let mut out = vec![p[0]];
    for w in p.windows(2) {
        out.push(w[0].lerp(w[1], 0.25));
        out.push(w[0].lerp(w[1], 0.75));
    }
    out.push(*p.last().unwrap());
    out
}
