//! Step 1's checks and Step 3's founding: where the first settlers would
//! land, drink and build, and the paths they'd wear between those places.

use crate::erode::{Kind, Land};
use gahturiyu_sim::sim::geo::V2;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A Roduro building model and its footprint (metres: front-to-back along
/// its facing, and across).
#[derive(Clone, Copy, Debug)]
pub struct Footprint {
    pub model: &'static str,
    #[allow(dead_code)]
    pub deep: f32,
    pub wide: f32,
}

pub const COTTAGE: Footprint = Footprint { model: "Roduro_Home", deep: 8.1, wide: 11.4 };
pub const DRUM: Footprint = Footprint { model: "Roduro_Drum", deep: 10.2, wide: 11.4 };
pub const VAULT: Footprint = Footprint { model: "Roduro_Vault", deep: 9.6, wide: 9.0 };
pub const GREAT: Footprint = Footprint { model: "Roduro_Great", deep: 7.2, wide: 11.4 };

pub fn footprint(model: &str) -> Footprint {
    match model {
        "Roduro_Drum" => DRUM,
        "Roduro_Vault" => VAULT,
        "Roduro_Great" => GREAT,
        _ => COTTAGE,
    }
}

/// A home as the step file records it.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Home {
    pub at: V2,
    /// Facing (radians, the simulation's convention): the door looks this way.
    pub rot: f32,
    pub model: String,
    pub eldest: bool,
    /// Why it's here (for the notes).
    pub why: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Founding {
    pub landing: V2,
    pub spring: V2,
    pub eldest_site: V2,
    pub homes: Vec<Home>,
    /// Footpaths, each a line of points.
    pub paths: Vec<Vec<V2>>,
    /// The way in from the world's road, as far as the first home.
    pub approach: Vec<V2>,
}

// ---- Walking over the land ----------------------------------------------------

/// Steeper than this can't be walked (degrees).
pub const WALK_MAX_DEG: f32 = 26.0;
/// Steeper than this (rise per metre, tan 18°) gets stairs cut.
pub const STAIR_GRADE: f32 = 0.3249;

/// A route-finder over the land at `step` metres.
pub struct Router<'a> {
    land: &'a Land,
    step: usize,
    w: usize,
    h: usize,
    /// How worn each cell is (0 fresh ground, 1 a path): cheaper to reuse.
    worn: Vec<f32>,
    /// The steepest grade allowed (degrees).
    pub max_deg: f32,
    /// Extra cost per metre on stretches steep enough to need stairs
    /// (cutting them is work; 1 = none).
    pub stair_cost: f32,
    /// Keep out of streams (wet feet): extra cost on stream cells.
    pub avoid_wet: bool,
}

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

impl<'a> Router<'a> {
    pub fn new(land: &'a Land, step: usize) -> Router<'a> {
        let (w, h) = (land.w / step, land.h / step);
        Router { land, step, w, h, worn: vec![0.0; w * h], max_deg: WALK_MAX_DEG, stair_cost: 1.0, avoid_wet: false }
    }

    fn node(&self, p: V2) -> usize {
        let i = (((p.x - self.land.x0) / (self.land.cell * self.step as f32)).round().max(0.0) as usize).min(self.w - 1);
        let j = (((p.y - self.land.y0) / (self.land.cell * self.step as f32)).round().max(0.0) as usize).min(self.h - 1);
        j * self.w + i
    }

    pub fn pos(&self, n: usize) -> V2 {
        V2::new(self.land.x0 + (n % self.w) as f32 * self.land.cell * self.step as f32, self.land.y0 + (n / self.w) as f32 * self.land.cell * self.step as f32)
    }

    fn height(&self, n: usize) -> f32 {
        self.land.height(self.pos(n))
    }

    /// Walking cost from `a` to every cell (infinite where it can't be reached).
    pub fn distances(&self, a: V2) -> Vec<f32> {
        self.search(a, None).0
    }

    /// Is `p` reachable on foot from where `distances` started?
    pub fn reachable(&self, dist: &[f32], p: V2) -> bool {
        dist[self.node(p)].is_finite()
    }

    /// The cheapest walk from `a` to `b` as grid nodes, if there is one.
    pub fn route_nodes(&self, a: V2, b: V2) -> Option<Vec<usize>> {
        let (sa, sb) = (self.node(a), self.node(b));
        let (dist, prev) = self.search(a, Some(sb));
        if !dist[sb].is_finite() {
            return None;
        }
        let mut path = vec![sb];
        while let Some(&last) = path.last() {
            if last == sa || prev[last] == u32::MAX {
                break;
            }
            path.push(prev[last] as usize);
        }
        path.reverse();
        Some(path)
    }

    /// Wear a route of nodes in.
    pub fn wear_nodes(&mut self, path: &[usize]) {
        for &k in path {
            self.worn[k] = 1.0;
        }
    }

    /// The cheapest walk from `a` to `b`, if there is one, and the steepest
    /// grade along it (degrees).
    pub fn route(&self, a: V2, b: V2) -> Option<(Vec<V2>, f32)> {
        let path = self.route_nodes(a, b)?;
        let mut steepest: f32 = 0.0;
        for s in path.windows(2) {
            let len = self.pos(s[0]).dist(self.pos(s[1])).max(0.1);
            steepest = steepest.max(((self.height(s[1]) - self.height(s[0])).abs() / len).atan().to_degrees());
        }
        let pts: Vec<V2> = path.iter().map(|&k| self.pos(k)).collect();
        // Take the corners off the grid walk.
        let mut smooth = pts.clone();
        for i in 1..pts.len().saturating_sub(1) {
            smooth[i] = pts[i - 1].scale(0.25).add(pts[i].scale(0.5)).add(pts[i + 1].scale(0.25));
        }
        Some((smooth, steepest))
    }

    fn search(&self, a: V2, stop: Option<usize>) -> (Vec<f32>, Vec<u32>) {
        let sa = self.node(a);
        let n = self.w * self.h;
        let mut dist = vec![f32::INFINITY; n];
        let mut prev = vec![u32::MAX; n];
        let mut heap = BinaryHeap::new();
        dist[sa] = 0.0;
        heap.push(Item(0.0, sa as u32));
        let cell = self.land.cell * self.step as f32;
        let limit = self.max_deg.to_radians().tan();
        while let Some(Item(d, k)) = heap.pop() {
            let k = k as usize;
            if d > dist[k] {
                continue;
            }
            if Some(k) == stop {
                break;
            }
            let (i, j) = (k % self.w, k / self.w);
            let zk = self.height(k);
            for (di, dj) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (ni, nj) = (i as i32 + di, j as i32 + dj);
                if ni < 0 || nj < 0 || ni >= self.w as i32 || nj >= self.h as i32 {
                    continue;
                }
                let nk = nj as usize * self.w + ni as usize;
                let zn = self.height(nk);
                if zn < 0.0 {
                    continue; // the sea
                }
                let len = cell * if di != 0 && dj != 0 { 1.414 } else { 1.0 };
                let grade = (zn - zk).abs() / len;
                if grade > limit {
                    continue;
                }
                // Climbing costs; a worn path is easier; rock underfoot is slower.
                let cell = self.land.cell_of(self.pos(nk));
                let mut rough = if self.land.kind(cell) == Kind::Rock { 1.3 } else { 1.0 };
                if grade > STAIR_GRADE {
                    rough *= self.stair_cost;
                }
                if self.avoid_wet {
                    rough *= 1.0 + 3.0 * self.land.stream(cell);
                }
                let cost = len * (1.0 + 9.0 * grade * grade) * rough * (1.0 - 0.55 * self.worn[nk]);
                let nd = d + cost;
                if nd < dist[nk] {
                    dist[nk] = nd;
                    prev[nk] = k as u32;
                    heap.push(Item(nd, nk as u32));
                }
            }
        }
        (dist, prev)
    }

    /// Wear a path in, so later routes prefer it.
    pub fn wear(&mut self, path: &[V2]) {
        for p in path {
            let k = self.node(*p);
            self.worn[k] = 1.0;
        }
    }
}

// ---- Looking across the land --------------------------------------------------

/// Is any sea in view from here (a fan of rays toward the west)?
pub fn sees_sea(land: &Land, from: V2, eye: f32) -> bool {
    let z0 = land.height(from) + eye;
    for k in 0..9 {
        let a = std::f32::consts::PI + (k as f32 - 4.0) * 0.17;
        let (dx, dy) = (a.cos(), a.sin());
        let mut top = -1.0f32; // steepest line of sight so far (rise per metre)
        let mut t = 4.0;
        while t < 1200.0 {
            let p = V2::new(from.x + dx * t, from.y + dy * t);
            if !land.inside(p) {
                break;
            }
            let h = land.height(p);
            let rise = (h.max(0.0) - z0) / t;
            if h < 0.0 && rise > top {
                return true;
            }
            top = top.max(rise);
            t += 2.0;
        }
    }
    false
}

// ---- Founding ---------------------------------------------------------------

/// Score the land and place the founders.
pub fn found(land: &Land, seed: u64, swell_from_deg: f32, approach_from: V2, centre: V2, core_r: f32) -> (Founding, Vec<String>) {
    let mut notes = Vec::new();
    let n = land.w * land.h;
    let cellpos = |k: usize| land.pos(k);
    // Only the site proper: not the fade to the world's own shore.
    let in_core = |p: V2| (p.y - centre.y).abs() < core_r && crate::bake::rim_fade(land, p) >= 1.0;

    // The landing: a sheltered bit of shore a boat can be pulled up on,
    // that can be walked up from.
    // A scramble is allowed to reach it; Step 4 cuts the stairs.
    let mut router0 = Router::new(land, 2);
    router0.max_deg = 42.0;
    let from_inland = router0.distances(approach_from);
    let mut best_landing: Option<(f32, usize)> = None;
    for k in 0..n {
        let (i, j) = (k % land.w, k / land.w);
        if i < 3 || j < 3 || i >= land.w - 3 || j >= land.h - 3 || land.sea[k] {
            continue;
        }
        let z = land.z[k];
        if z > 4.0 || land.slope_deg(k) > 20.0 || !in_core(cellpos(k)) {
            continue;
        }
        let shore = [k - 3, k + 3, k - 3 * land.w, k + 3 * land.w, k - 1, k + 1, k - land.w, k + land.w].iter().any(|&q| land.sea[q]);
        if !shore || !router0.reachable(&from_inland, cellpos(k)) {
            continue;
        }
        let shelter = 1.0 - land.exposure(k, swell_from_deg);
        // How much gentle shore there is round it.
        let mut beach = 0;
        for dj in -8i32..=8 {
            for di in -8i32..=8 {
                let q = ((j as i32 + dj) * land.w as i32 + i as i32 + di) as usize;
                if q < n && !land.sea[q] && land.z[q] < 4.0 && land.slope_deg(q) < 20.0 {
                    beach += 1;
                }
            }
        }
        let beach = (beach as f32 / 120.0).min(1.0);
        let score = shelter * 2.0 + beach + if land.kind(k) == Kind::Shingle { 0.5 } else { 0.0 };
        if best_landing.is_none_or(|(s, _)| score > s) {
            best_landing = Some((score, k));
        }
    }
    let Some((lscore, lk)) = best_landing else {
        notes.push("No sheltered landing found.".into());
        return (Founding::default(), notes);
    };
    let landing = cellpos(lk);
    notes.push(format!("Landing: a sheltered shore {:.0}% out of the swell (score {lscore:.1}).", (1.0 - land.exposure(lk, swell_from_deg)) * 100.0));

    // Fresh water: the nearest stream to the landing.
    let mut spring: Option<(f32, usize)> = None;
    for k in 0..n {
        if land.sea[k] || land.z[k] < 1.0 || land.stream(k) < 0.5 {
            continue;
        }
        let d = cellpos(k).dist(landing);
        if d < 350.0 && spring.is_none_or(|(sd, _)| d < sd) {
            spring = Some((d, k));
        }
    }
    let spring_at = spring.map(|(_, k)| cellpos(k)).unwrap_or(landing);
    match spring {
        Some((d, _)) => notes.push(format!("Fresh water: a stream {d:.0} m from the landing.")),
        None => notes.push("No stream within 350 m of the landing: water would be a well.".into()),
    }

    // Home sites: flat pockets backed by rock, near the landing and the water.
    struct Cand {
        at: V2,
        score: f32,
        backed: bool,
        view: bool,
        rise: f32,
    }
    let mut cands: Vec<Cand> = Vec::new();
    let sw = (swell_from_deg.to_radians().cos(), swell_from_deg.to_radians().sin());
    for k in 0..n {
        let (i, j) = (k % land.w, k / land.w);
        if i < 10 || j < 10 || i >= land.w - 10 || j >= land.h - 10 || land.sea[k] {
            continue;
        }
        if (i + j) % 2 != 0 {
            continue; // every other cell is plenty
        }
        let p = cellpos(k);
        let z = land.z[k];
        let dl = p.dist(landing);
        if z < 4.0 || z > 75.0 || dl > 260.0 || land.slope_deg(k) > 9.0 || !in_core(p) {
            continue;
        }
        if !flat_enough(land, p, 10.2, 11.4, 1.6) {
            continue;
        }
        // Backed by rock: ground 4 m+ higher within 14 m on some side.
        let mut backed = false;
        let mut wind_shelter = 0.0f32;
        for a in 0..12 {
            let ang = a as f32 * 0.5236;
            let q = V2::new(p.x + 12.0 * ang.cos(), p.y + 12.0 * ang.sin());
            let rise = land.height(q) - z;
            if rise > 4.0 {
                backed = true;
                // Rock between the home and the weather.
                let toward = ang.cos() * sw.0 + ang.sin() * sw.1;
                if toward > 0.5 {
                    wind_shelter = wind_shelter.max((rise / 10.0).min(1.0));
                }
            }
        }
        let view = sees_sea(land, p, 2.0);
        let near_water = 1.0 - (p.dist(spring_at) / 250.0).min(1.0);
        let near_landing = 1.0 - (dl / 260.0).min(1.0);
        let score = (backed as i32 as f32) * 1.2 + wind_shelter * 1.0 + (view as i32 as f32) * 0.8 + near_water * 0.8 + near_landing * 1.0 + (z / 75.0) * 0.3;
        cands.push(Cand { at: p, score, backed, view, rise: z });
    }
    cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    notes.push(format!("{} flat pockets within 260 m of the landing fit a home.", cands.len()));

    // The Eldest Home's site: prominent, seen from the sea, near the landing.
    let mut eldest: Option<(f32, V2)> = None;
    for c in &cands {
        if !c.view || c.rise < 10.0 || c.at.dist(landing) > 180.0 {
            continue;
        }
        // Higher than its surroundings.
        let mut around = 0.0;
        for a in 0..8 {
            let ang = a as f32 * 0.785;
            around += land.height(V2::new(c.at.x + 35.0 * ang.cos(), c.at.y + 35.0 * ang.sin()));
        }
        let prom = c.rise - around / 8.0;
        let s = c.score + prom * 0.15;
        if eldest.is_none_or(|(es, _)| s > es) {
            eldest = Some((s, c.at));
        }
    }
    let eldest_site = eldest.map(|(_, p)| p).unwrap_or(cands.first().map(|c| c.at).unwrap_or(landing));
    notes.push(format!("Eldest Home's site: {:.0} m up, {:.0} m from the landing, open to the sea.", land.height(eldest_site), eldest_site.dist(landing)));

    // The founders: the Eldest's site first, then the best pockets that keep
    // their distance from each other and from the shore.
    let mut rng = gahturiyu_sim::sim::rng::Rng::from_keys(&[seed, 0x464F_554E]);
    let mut homes: Vec<Home> = Vec::new();
    let models = [COTTAGE, VAULT, COTTAGE, DRUM, VAULT];
    let mut picks: Vec<(V2, bool, String)> = vec![(eldest_site, true, "the first home, on the lookout above the landing".into())];
    for c in &cands {
        if picks.len() >= 5 {
            break;
        }
        if picks.iter().any(|(p, _, _)| p.dist(c.at) < 16.0) || c.at.dist(landing) < 12.0 {
            continue;
        }
        let why = match (c.backed, c.view) {
            (true, true) => "backed by rock, with the sea in view",
            (true, false) => "tucked against rock, out of the weather",
            (false, true) => "on open ground above the shore",
            _ => "on the flattest ground near the water",
        };
        picks.push((c.at, false, why.into()));
    }
    // Footpaths: from each home to the landing and the water, worn in turn
    // so later ones reuse earlier ones.
    let mut router = Router::new(land, 2);
    router.max_deg = 42.0;
    let mut paths = Vec::new();
    for (n, (at, eldest, why)) in picks.iter().enumerate() {
        let fp = if *eldest { COTTAGE } else { models[n % models.len()] };
        // Face the landing, or the path's first step.
        let mut rot = (landing.y - at.y).atan2(landing.x - at.x);
        if let Some((path, _)) = router.route(*at, landing) {
            if path.len() > 2 {
                let q = path[2.min(path.len() - 1)];
                rot = (q.y - at.y).atan2(q.x - at.x);
            }
            router.wear(&path);
            paths.push(path);
        }
        if let Some((path, _)) = router.route(*at, spring_at) {
            router.wear(&path);
            paths.push(path);
        }
        homes.push(Home { at: *at, rot: rot + rng.range(-0.15, 0.15), model: fp.model.to_string(), eldest: *eldest, why: why.clone() });
    }
    // The way in from the world: to the first home.
    let approach = router.route(approach_from, homes.first().map(|h| h.at).unwrap_or(landing)).map(|(p, g)| {
        notes.push(format!("The approach from inland can be walked: steepest {g:.0}° (limit {WALK_MAX_DEG:.0}°)."));
        p
    });
    if approach.is_none() {
        notes.push("The approach from inland can't be walked under the slope limit: it needs stairs or a cut.".into());
    }
    notes.push(format!("{} founding homes placed; {} footpaths.", homes.len(), paths.len()));
    (Founding { landing, spring: spring_at, eldest_site, homes, paths, approach: approach.unwrap_or_default() }, notes)
}

/// Does a footprint `deep` × `wide` centred here sit on ground that varies
/// by no more than `tol` metres?
pub fn flat_enough(land: &Land, p: V2, deep: f32, wide: f32, tol: f32) -> bool {
    let z = land.height(p);
    let r = deep.max(wide) * 0.5;
    for k in 0..8 {
        let a = k as f32 * 0.785;
        let q = V2::new(p.x + r * 0.8 * a.cos(), p.y + r * 0.8 * a.sin());
        if !land.inside(q) || (land.height(q) - z).abs() > tol {
            return false;
        }
    }
    true
}
