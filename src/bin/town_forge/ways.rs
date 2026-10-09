//! Step 4: the ways. The founders' places (doors, the landing, the water,
//! the way in) are joined by the cheapest walks, each worn in so later ones
//! reuse earlier ones; the network that results is cut into stretches:
//! cobbles where it's busiest (and all along the spine from the Eldest Home
//! down to the landing), dirt elsewhere (the road in stays dirt: it's the
//! world's), stairs where it climbs steeper than
//! `STAIR_GRADE`, and a slab bridge where it crosses a stream.

use crate::erode::Land;
use crate::found::{footprint, Router, STAIR_GRADE};
use gahturiyu_sim::sim::forge::{Founding, Home, Way, WayKind, Ways};
use gahturiyu_sim::sim::geo::V2;
use std::collections::{BTreeMap, BTreeSet};

/// A way used by this many of the founders' walks is paved.
const COBBLE_USE: u32 = 5;
/// Rise per stair (metres), for the notes.
pub const RISER: f32 = 0.18;

pub fn width(kind: WayKind) -> f32 {
    match kind {
        WayKind::Cobbles => 3.0,
        WayKind::Dirt => 2.2,
        WayKind::Stairs => 2.6,
        WayKind::Bridge => 2.4,
    }
}

/// Where a home's door opens onto the ground.
pub fn door(h: &Home) -> V2 {
    let fp = footprint(&h.model);
    V2::new(h.at.x + h.rot.cos() * (fp.deep * 0.5 + 2.0), h.at.y + h.rot.sin() * (fp.deep * 0.5 + 2.0))
}

/// Lay the ways between the founding places.
pub fn lay(land: &Land, f: &Founding, approach_from: V2) -> (Ways, Vec<String>) {
    let mut notes = Vec::new();
    let Some(eldest) = f.homes.iter().find(|h| h.eldest).or(f.homes.first()) else {
        notes.push("No homes: nothing to join.".into());
        return (Ways::default(), notes);
    };
    let eldest_door = door(eldest);
    let mut router = Router::new(land, 2);
    router.max_deg = 42.0;
    router.stair_cost = 2.0;
    router.avoid_wet = true;

    // The walks, most important first. The first is the road in (never
    // paved: it's the world's), the second the spine.
    let mut walks: Vec<(V2, V2, &str)> = vec![(approach_from, eldest_door, "the road in"), (eldest_door, f.landing, "the Eldest Home down to the landing")];
    for h in f.homes.iter().filter(|h| !h.eldest) {
        let d = door(h);
        walks.push((d, f.landing, "a home to the landing"));
        walks.push((d, f.spring, "a home to the water"));
        walks.push((d, eldest_door, "a home to the Eldest Home"));
    }
    walks.push((eldest_door, f.spring, "the Eldest Home to the water"));
    walks.push((f.landing, f.spring, "the landing to the water"));

    // Edges of the network, how many walks use each, and the spine's.
    let mut uses: BTreeMap<(usize, usize), u32> = BTreeMap::new();
    let mut spine: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut road: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut failed = 0;
    for (n, (a, b, what)) in walks.iter().enumerate() {
        // The road in takes no stairs if it can help it.
        router.max_deg = if n == 0 { 30.0 } else { 42.0 };
        let route = router.route_nodes(*a, *b).or_else(|| {
            router.max_deg = 42.0;
            router.route_nodes(*a, *b)
        });
        let Some(route) = route else {
            failed += 1;
            notes.push(format!("No way found for {what}."));
            continue;
        };
        for s in route.windows(2) {
            let e = (s[0].min(s[1]), s[0].max(s[1]));
            if n == 0 {
                road.insert(e);
            } else {
                *uses.entry(e).or_default() += 1;
                if n == 1 {
                    spine.insert(e);
                }
            }
        }
        router.wear_nodes(&route);
    }

    // Chains: walk the network from its junctions and ends.
    let mut adj: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let edges: BTreeSet<(usize, usize)> = uses.keys().copied().chain(road.iter().copied()).collect();
    for &(a, b) in &edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut done: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut chains: Vec<Vec<usize>> = Vec::new();
    let starts: Vec<usize> = adj.iter().filter(|(_, n)| n.len() != 2).map(|(k, _)| *k).chain(adj.keys().copied()).collect();
    for start in starts {
        for &next in &adj[&start] {
            let e = (start.min(next), start.max(next));
            if done.contains(&e) {
                continue;
            }
            let mut chain = vec![start, next];
            done.insert(e);
            loop {
                let last = *chain.last().unwrap();
                let prev = chain[chain.len() - 2];
                let ns = &adj[&last];
                if ns.len() != 2 {
                    break;
                }
                let on = if ns[0] == prev { ns[1] } else { ns[0] };
                let e = (last.min(on), last.max(on));
                if done.contains(&e) {
                    break;
                }
                done.insert(e);
                chain.push(on);
            }
            chains.push(chain);
        }
    }

    // Cut each chain into stretches by what it's made of.
    let mut ways: Vec<Way> = Vec::new();
    let kind_of = |a: usize, b: usize| -> WayKind {
        let (pa, pb) = (router.pos(a), router.pos(b));
        let mid = pa.lerp(pb, 0.5);
        let stream = |p: V2| land.inside(p) && land.stream(land.cell_of(p)) > 0.5;
        if stream(mid) && !stream(pa) && !stream(pb) {
            return WayKind::Bridge;
        }
        let grade = (land.height(pa) - land.height(pb)).abs() / pa.dist(pb).max(0.1);
        if grade > STAIR_GRADE {
            return WayKind::Stairs;
        }
        let e = (a.min(b), a.max(b));
        if spine.contains(&e) || (!road.contains(&e) && uses.get(&e).copied().unwrap_or(0) >= COBBLE_USE) {
            WayKind::Cobbles
        } else {
            WayKind::Dirt
        }
    };
    for chain in &chains {
        // Corners off the grid walk (ends and stair nodes stay put).
        let raw: Vec<V2> = chain.iter().map(|&k| router.pos(k)).collect();
        let mut pts = raw.clone();
        for i in 1..raw.len().saturating_sub(1) {
            pts[i] = raw[i - 1].scale(0.25).add(raw[i].scale(0.5)).add(raw[i + 1].scale(0.25));
        }
        let mut run: Vec<V2> = vec![pts[0]];
        let mut kind = kind_of(chain[0], chain[1]);
        for i in 0..chain.len() - 1 {
            let k = kind_of(chain[i], chain[i + 1]);
            if k != kind {
                ways.push(Way { kind, pts: std::mem::replace(&mut run, vec![pts[i]]), width: width(kind) });
                kind = k;
            }
            run.push(pts[i + 1]);
        }
        ways.push(Way { kind, pts: run, width: width(kind) });
    }
    // Stairs: straight treads from the true ground, not the smoothed line.
    ways.retain(|w| w.pts.len() >= 2);

    let total = |k: WayKind| ways.iter().filter(|w| w.kind == k).map(|w| w.length()).sum::<f32>();
    let flights = ways.iter().filter(|w| w.kind == WayKind::Stairs).count();
    let steps: f32 = ways
        .iter()
        .filter(|w| w.kind == WayKind::Stairs)
        .map(|w| w.pts.windows(2).map(|s| (land.height(s[0]) - land.height(s[1])).abs()).sum::<f32>() / RISER)
        .sum();
    let bridges = ways.iter().filter(|w| w.kind == WayKind::Bridge).count();
    notes.push(format!("{} walks laid ({failed} found no way); {} stretches of way.", walks.len(), ways.len()));
    notes.push(format!("Cobbles {:.0} m (the spine and anything {COBBLE_USE}+ walks share), dirt {:.0} m.", total(WayKind::Cobbles), total(WayKind::Dirt)));
    notes.push(format!("{flights} flights of stairs, about {steps:.0} steps in all ({:.0} m of climb at {RISER} m a step).", steps * RISER));
    notes.push(format!("{bridges} slab bridge{} over the stream.", if bridges == 1 { "" } else { "s" }));
    (Ways { ways }, notes)
}
