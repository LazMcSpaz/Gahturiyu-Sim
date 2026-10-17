//! Roads, and the way people get between towns.
//!
//! The roads are Laz's, as he drew them (`drawn_roads.rs`), and only those
//! (Laz, 17 Oct): nothing else is drawn, painted or counted as a road. The way
//! between each pair of towns is found once at world creation by searching
//! the land for the cheapest walk, where "cheap" means short, not too steep,
//! and dry; ground his roads cross is much cheaper, so a traveller takes the
//! road where there is one and walks the land where there isn't.

use serde::{Deserialize, Serialize};

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::geo::{self, V2, WORLD_SIZE};
use super::settlement::{Settlement, SettlementId};
use super::terrain::{walk_factor, Terrain};

/// Spacing of the search grid, metres.
/// The pathfinding grid for the roads between towns, metres: coarse, for a
/// continent (the roads will come from Laz's map's own lines).
// (200 m: on a 68 km map that's 342 cells a side, fewer than the old 600 m on 386 km.)
const GRID: f32 = 200.0;
const G: usize = (WORLD_SIZE / GRID) as usize + 1;
/// Cost multiplier on ground one of Laz's roads crosses.
const ROAD_DISCOUNT: f32 = 0.4;
/// Grades steeper than this can't be walked at all.
const MAX_GRADE: f32 = 0.42;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Routes {
    /// Smoothed route between each pair of towns, stored once with the lower
    /// id first.
    paths: HashMap<(SettlementId, SettlementId), Vec<V2>>,
    /// The roads: Laz's drawn lines, in metres (`drawn_roads.rs`).
    pub roads: Vec<Vec<V2>>,
    /// Pairs no way could be found between (a straight line stands in).
    pub fallbacks: Vec<(SettlementId, SettlementId)>,
    /// The roads as a network to find a way along: points, and for each the
    /// points it joins with the walking effort between them.
    pub nodes: Vec<V2>,
    pub links: Vec<Vec<(u32, f32)>>,
}

impl Routes {
    /// Turn the roads into a network (after they're laid on the land, so the
    /// effort between points includes the road's own speed).
    pub fn build_network(&mut self, terrain: &Terrain) {
        let mut ids: HashMap<(i64, i64), u32> = HashMap::new();
        let mut node = |p: V2, nodes: &mut Vec<V2>, links: &mut Vec<Vec<(u32, f32)>>| -> u32 {
            // Points within a couple of metres are the same junction.
            let key = ((p.x / 2.0).round() as i64, (p.y / 2.0).round() as i64);
            *ids.entry(key).or_insert_with(|| {
                nodes.push(p);
                links.push(Vec::new());
                (nodes.len() - 1) as u32
            })
        };
        let (mut nodes, mut links) = (Vec::new(), Vec::new());
        for road in &self.roads {
            let mut prev: Option<(u32, V2)> = None;
            for &p in road {
                let id = node(p, &mut nodes, &mut links);
                if let Some((q, qp)) = prev {
                    if q != id {
                        let (e1, e2) = (terrain.effort(qp, p), terrain.effort(p, qp));
                        links[q as usize].push((id, e1));
                        links[id as usize].push((q, e2));
                    }
                }
                prev = Some((id, p));
            }
        }
        self.nodes = nodes;
        self.links = links;
    }

    /// The network point nearest `p`.
    /// Is `p` on a road (within `half` metres of its line)? A road laid
    /// along a low shore is walkable where the land dips under the water.
    pub fn on_road(&self, p: V2, half: f32) -> bool {
        self.roads.iter().any(|r| {
            r.windows(2).any(|s| {
                let (a, b) = (s[0], s[1]);
                let ab = b.sub(a);
                let l2 = (ab.x * ab.x + ab.y * ab.y).max(1e-6);
                let t = (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0);
                a.add(ab.scale(t)).dist(p) <= half
            })
        })
    }

    pub fn nearest_node(&self, p: V2) -> Option<u32> {
        (0..self.nodes.len()).min_by(|&a, &b| self.nodes[a].dist(p).total_cmp(&self.nodes[b].dist(p))).map(|i| i as u32)
    }

    /// The least-effort way along the roads between two network points:
    /// the points, and the total effort.
    pub fn along_roads(&self, from: u32, to: u32) -> Option<(Vec<V2>, f32)> {
        let n = self.nodes.len();
        let mut best = vec![f32::INFINITY; n];
        let mut prev = vec![u32::MAX; n];
        let mut heap = BinaryHeap::new();
        best[from as usize] = 0.0;
        heap.push(Reverse((0u64, from)));
        while let Some(Reverse((c, u))) = heap.pop() {
            let cost = f32::from_bits(c as u32);
            if cost > best[u as usize] {
                continue;
            }
            if u == to {
                break;
            }
            for &(v, e) in &self.links[u as usize] {
                let nc = cost + e;
                if nc < best[v as usize] {
                    best[v as usize] = nc;
                    prev[v as usize] = u;
                    heap.push(Reverse((nc.to_bits() as u64, v)));
                }
            }
        }
        if !best[to as usize].is_finite() {
            return None;
        }
        let mut path = vec![self.nodes[to as usize]];
        let mut at = to;
        while at != from {
            at = prev[at as usize];
            path.push(self.nodes[at as usize]);
        }
        path.reverse();
        Some((path, best[to as usize]))
    }

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
        // No road reaches Salt Isle (`island.rs`); it's always the last
        // settlement, so the rest keep their ids as indices here.
        let mainland = towns.iter().position(|s| s.island).unwrap_or(towns.len());
        debug_assert!(towns[mainland..].iter().all(|s| s.island), "island settlements come last");
        let towns = &towns[..mainland];
        let idx = |i: usize, j: usize| j * G + i;
        let grid_centre = |i: usize, j: usize| V2::new(i as f32 * GRID, j as f32 * GRID);
        // At the continent's scale the roads only need the map's own
        // heights and shores (`atlas.rs`), not the seed's small relief: a
        // cheap lookup for the half-million cells.
        let atlas = super::atlas::atlas();
        let coarse = |p: V2| atlas.base_height(p);
        // A cell is dry if any of nine spots in it is dry land, and the
        // route goes through the dry spot nearest its middle: a coastal
        // strip or a valley narrower than the grid still carries a road.
        let mut dry = vec![false; G * G];
        let mut rep: Vec<V2> = Vec::with_capacity(G * G);
        for k in 0..G * G {
            let c = grid_centre(k % G, k / G);
            let mut best: Option<(f32, V2)> = None;
            for dj in -1..=1 {
                for di in -1..=1 {
                    let p = c.add(V2::new(di as f32, dj as f32).scale(GRID / 3.0));
                    if p.x < 0.0 || p.y < 0.0 || p.x > WORLD_SIZE || p.y > WORLD_SIZE {
                        continue;
                    }
                    if atlas.inland(p) > 15.0 {
                        let d = p.dist(c);
                        if best.is_none_or(|(bd, _)| d < bd) {
                            best = Some((d, p));
                        }
                    }
                }
            }
            dry[k] = best.is_some();
            rep.push(best.map(|b| b.1).unwrap_or(c));
        }
        let centre = |i: usize, j: usize| rep[idx(i, j)];
        let height: Vec<f32> = (0..G * G).map(|k| coarse(rep[k])).collect();
        // The steepest bit of ground along each step out of each cell, read
        // off the map's heights: a coarse grid alone would step straight
        // over a cliff that sits between two of its points. A step that
        // crosses water can't be taken at all.
        let sub = |a: V2, b: V2| -> f32 {
            let mut worst = 0.0f32;
            let n = 6;
            for k in 0..n {
                let p = a.lerp(b, k as f32 / n as f32);
                let q = a.lerp(b, (k + 1) as f32 / n as f32);
                if k + 1 < n && atlas.inland(q) < 15.0 {
                    return f32::INFINITY;
                }
                worst = worst.max((coarse(q) - coarse(p)).abs() / p.dist(q).max(1.0));
            }
            worst
        };
        let steep: Vec<[f32; 8]> = (0..G * G)
            .map(|k| {
                let (i, j) = ((k % G) as i32, (k / G) as i32);
                let mut out = [f32::INFINITY; 8];
                if !dry[k] {
                    return out;
                }
                for (n, (di, dj, _)) in STEPS.iter().enumerate() {
                    let (vi, vj) = (i + di, j + dj);
                    if vi >= 0 && vj >= 0 && vi < G as i32 && vj < G as i32 && dry[idx(vi as usize, vj as usize)] {
                        out[n] = sub(centre(i as usize, j as usize), centre(vi as usize, vj as usize));
                    }
                }
                out
            })
            .collect();
        let snow: Vec<bool> = (0..G * G).map(|k| atlas.class_at(rep[k]) == super::atlas::Class::Snow).collect();
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
        // (An offshore town has no road and no path: it's reached by sea.)
        let ashore = |i: usize| !towns[i].offshore;

        let mut out = Routes::default();
        let mut raw: HashMap<(SettlementId, SettlementId), Vec<usize>> = HashMap::new();

        // Laz's roads, and the grid cells they cross (cheap to walk).
        let drawn = super::drawn_roads::drawn_roads();
        for line in drawn {
            for s in line.windows(2) {
                let n = ((s[0].dist(s[1]) / (GRID / 3.0)).ceil() as usize).max(1);
                for k in 0..=n {
                    let p = s[0].lerp(s[1], k as f32 / n as f32);
                    let (i, j) = ((p.x / GRID).round(), (p.y / GRID).round());
                    if i >= 0.0 && j >= 0.0 && (i as usize) < G && (j as usize) < G {
                        road[idx(i as usize, j as usize)] = true;
                    }
                }
            }
        }
        out.roads = drawn.to_vec();

        // Every pair's way, leaning on the roads. (The roads are fixed, so
        // each town's search stands alone: run side by side, gathered in
        // town order, the same as one by one.)
        let starts: Vec<usize> = (0..towns.len()).filter(|&a| ashore(a)).collect();
        let found = par_map(&starts, |&a| {
            let wants: Vec<usize> = (a + 1..towns.len()).filter(|&b| ashore(b) && !raw.contains_key(&(a as SettlementId, b as SettlementId))).collect();
            if wants.is_empty() {
                return Vec::new();
            }
            let parent = search(nodes[a], &height, &steep, &dry, &snow, &road);
            wants.into_iter().map(|b| (b, trace(&parent, nodes[a], nodes[b]))).collect::<Vec<_>>()
        });
        for (&a, traces) in starts.iter().zip(found) {
            for (b, cells) in traces {
                raw.insert((a as SettlementId, b as SettlementId), cells);
            }
        }

        // Each pair's way, straightened and smoothed (side by side too, in a
        // fixed order).
        let mut pairs: Vec<(SettlementId, SettlementId)> = raw.keys().copied().collect();
        pairs.sort();
        let ways = par_map(&pairs, |&(a, b)| {
            let cells = &raw[&(a, b)];
            let mut pts: Vec<V2> = cells.iter().map(|&c| centre(c % G, c / G)).collect();
            let fell_back = pts.is_empty();
            if fell_back {
                // Unreachable: fall back to a straight line rather than nothing.
                pts = vec![towns[a as usize].pos, towns[b as usize].pos];
            }
            pts[0] = towns[a as usize].pos;
            let last = pts.len() - 1;
            pts[last] = towns[b as usize].pos;
            // (Smoothing cuts corners: a point left in the water comes ashore.)
            let pts: Vec<V2> = chaikin(chaikin(straighten(&pts, terrain))).into_iter().map(|p| if atlas.inland(p) < 20.0 { atlas.landward(p, 25.0) } else { p }).collect();
            (pts, fell_back)
        });
        for (&(a, b), (pts, fell_back)) in pairs.iter().zip(ways) {
            if fell_back {
                out.fallbacks.push((a, b));
            }
            out.paths.insert((a, b), pts);
        }
        out
    }
}

/// `f` over every item, on as many threads as the machine has, the results
/// in the items' order (so the same as a plain loop).
fn par_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 16);
    if items.len() < 2 || threads < 2 {
        return items.iter().map(&f).collect();
    }
    let chunk = items.len().div_ceil(threads);
    let f = &f;
    std::thread::scope(|s| {
        let handles: Vec<_> = items.chunks(chunk).map(|c| s.spawn(move || c.iter().map(f).collect::<Vec<R>>())).collect();
        handles.into_iter().flat_map(|h| h.join().expect("a road worker")).collect()
    })
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

fn search(start: usize, height: &[f32], steep: &[[f32; 8]], dry: &[bool], snow: &[bool], road: &[bool]) -> Vec<u32> {
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
            // (A pass at 500 m costs like ground six times as far, so a road goes
            // round the mountains when there's any way round; the grid is 200 m
            // now, so the old factors let a short pass win against a long detour.)
            let mut cost = run / walk_factor(grade) * (1.0 + 5.0 * (worst - 0.12).max(0.0)) * (1.0 + (height[v] - 250.0).max(0.0) / 50.0);
            // (Nobody builds a road over the snow if there's any way round.)
            if snow[v] {
                cost *= 20.0;
            }
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
