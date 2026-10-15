//! The land and the roads across it.

use gahturiyu_sim::sim::{geo, worldgen};

#[test]
fn every_town_can_reach_every_other_over_dry_land() {
    let w = worldgen::generate(1);
    // (Salt Isle has no road: it's reached by ship.)
    let n = w.settlements.iter().filter(|s| !s.island).count() as u16;
    for a in 0..n {
        for b in 0..n {
            // (An island's town is reached by sea, not road.)
            if a == b || w.settlements[a as usize].offshore || w.settlements[b as usize].offshore {
                continue;
            }
            let p = w.routes.between(a, b);
            assert!(p.len() >= 2, "no route from {a} to {b}");
            // (A road over flat ground can be one straight stretch; the
            // routes say when no way was found.)
            assert!(!w.routes.fallbacks.contains(&(a.min(b), a.max(b))), "route {a}->{b} fell back to a straight line (unreachable)");
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
    // A road climbs no higher than the highest town it has to reach (Laz puts
    // villages in the high rock), and never over a pass for its own sake.
    let town_top = w.settlements.iter().map(|s| w.terrain.height(s.pos)).fold(0.0f32, f32::max);
    assert!(highest < town_top.max(450.0) + 60.0, "a road goes over {highest} m (the highest town is at {town_top:.0})");
}

#[test]
fn towns_stand_on_habitable_ground() {
    let w = worldgen::generate(1);
    // (Laz's marks are where the towns are, hills and all: a town stands where
    // he put it, on ground a person can stand on; the land under it is his
    // to level in the editor.)
    for s in &w.settlements {
        assert!(!w.terrain.is_sea(s.pos), "{} is in the sea", s.name);
        assert!(w.terrain.slope(s.pos) < 0.45, "{} sits on a cliff", s.name);
        assert!(w.terrain.height(s.pos) < 900.0, "{} is up a mountain", s.name);
    }
}
