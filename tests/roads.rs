//! Roads are quick, the ground underfoot matters, and long trips use the roads.

use gahturiyu_sim::sim::{
    geo::V2,
    group::Leg,
    terrain::{Ground, ROAD_PACE},
    worldgen, World,
};

fn world() -> World {
    worldgen::generate(1)
}

/// A long, fairly flat road stretch and a parallel line 60 m off it.
fn road_and_beside(w: &World) -> (V2, V2, V2, V2) {
    for road in &w.routes.roads {
        for seg in road.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let len = a.dist(b);
            if len < 80.0 {
                continue;
            }
            let d = b.sub(a).scale(1.0 / len);
            let off = V2::new(-d.y, d.x).scale(60.0);
            let (c, e) = (a.add(off), b.add(off));
            let flat = (w.terrain.height(b) - w.terrain.height(a)).abs() < 2.0 && (w.terrain.height(e) - w.terrain.height(c)).abs() < 2.0;
            if flat && w.terrain.on_road(a.lerp(b, 0.5)) && !w.terrain.on_road(c.lerp(e, 0.5)) && w.terrain.ground(c.lerp(e, 0.5)) == Ground::Grass {
                return (a, b, c, e);
            }
        }
    }
    panic!("no suitable stretch");
}

#[test]
fn every_kind_of_ground_turns_up_and_is_stable() {
    let w = world();
    let w2 = world();
    let mut seen = std::collections::HashSet::new();
    for j in 0..140 {
        for i in 0..140 {
            let p = V2::new(i as f32 * 150.0 + 30.0, j as f32 * 150.0 + 30.0);
            if !gahturiyu_sim::sim::geo::is_land(p) {
                continue;
            }
            let g = w.terrain.ground(p);
            assert_eq!(g, w2.terrain.ground(p), "same seed, same ground");
            seen.insert(g.name());
        }
    }
    for kind in ["road", "grass", "scrub", "rock", "sand"] {
        assert!(seen.contains(kind), "no {kind} anywhere");
    }
}

#[test]
fn schedules_charge_for_ground_and_roads() {
    let w = world();
    let (a, b, c, e) = road_and_beside(&w);
    let on = Leg::along(vec![a, b], 0.0, 1.0, None, &w.terrain);
    let off = Leg::along(vec![c, e], 0.0, 1.0, None, &w.terrain);
    let ratio = (off.arrive - off.depart) / (on.arrive - on.depart);
    assert!(ratio > ROAD_PACE as f64 * 0.9, "the road leg should be quicker: {ratio}");
}

#[test]
fn the_squad_walks_faster_on_a_road() {
    let mut w = world();
    let (a, b, c, e) = road_and_beside(&w);
    let m = w.squad.members[0];
    let timed = |w: &mut World, from: V2, to: V2| {
        w.teleport_squad(from);
        let k = w.squad.index(m).unwrap();
        w.squad.at[k] = from;
        w.order_members(&[m], from.lerp(to, 0.8));
        let mut t = 0.0;
        while w.squad.at[k].dist(from.lerp(to, 0.8)) > 0.5 && t < 2000.0 {
            w.step(0.5);
            t += 0.5;
        }
        t
    };
    let road = timed(&mut w, a, b);
    let grass = timed(&mut w, c, e);
    assert!(grass > road * 1.15, "road {road}s vs grass {grass}s");
}

#[test]
fn a_far_destination_goes_by_road_when_thats_quicker() {
    let w = world();
    // From the squad's town to the furthest town a road reaches.
    let here = w.squad.pos;
    let far = w.settlements.iter().map(|s| s.pos).filter(|p| p.dist(here) > 4000.0).min_by(|a, b| a.dist(here).total_cmp(&b.dist(here))).unwrap();
    let (path, _) = w.travel(here, far);
    let on = path.iter().filter(|&&p| w.terrain.on_road(p)).count();
    assert!(on * 2 > path.len(), "most of the way should be road: {on} of {}", path.len());
    // And short trips go straight.
    let (short, _) = w.travel(here, here.add(V2::new(120.0, 40.0)));
    assert!(short.len() <= 3);
}
