//! The land and the roads across it.

use gahturiyu_sim::sim::{geo, worldgen};

#[test]
fn every_town_can_reach_every_other_over_dry_land() {
    let w = worldgen::generate(1);
    let n = w.settlements.len() as u16;
    for a in 0..n {
        for b in 0..n {
            if a == b {
                continue;
            }
            let p = w.routes.between(a, b);
            assert!(p.len() >= 2, "no route from {a} to {b}");
            let far = w.settlements[a as usize].pos.dist(w.settlements[b as usize].pos) > 600.0;
            assert!(!far || p.len() > 2, "route {a}->{b} fell back to a straight line (unreachable?)");
            for q in &p[1..p.len() - 1] {
                assert!(geo::inland(*q) > -40.0, "route {a}->{b} goes out to sea at {q:?}");
            }
        }
    }
}

#[test]
fn roads_avoid_cliffs_and_go_around_mountains() {
    let w = worldgen::generate(1);
    let mut worst: f32 = 0.0;
    let mut highest: f32 = 0.0;
    for road in &w.routes.roads {
        for s in road.windows(2) {
            let len = s[0].dist(s[1]).max(1.0);
            worst = worst.max((w.terrain.height(s[1]) - w.terrain.height(s[0])).abs() / len);
            highest = highest.max(w.terrain.height(s[0]));
        }
    }
    // Smoothing the route can clip a short steep bit; a scramble, never a wall.
    assert!(worst < 0.75, "a road climbs at a grade of {worst}");
    assert!(highest < 500.0, "a road goes over {highest} m");
}

#[test]
fn towns_stand_on_habitable_ground() {
    let w = worldgen::generate(1);
    for s in &w.settlements {
        assert!(w.terrain.slope(s.pos) < 0.12, "{} sits on a slope", s.name);
        assert!(w.terrain.height(s.pos) < 300.0, "{} is up a mountain", s.name);
    }
}
