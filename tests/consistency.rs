//! The promises the band system makes, checked.
//!
//! 1. How finely the world is stepped does not change what happens.
//! 2. Where the squad stands (and so which band everything is in) does not
//!    change what happens either.
//! 3. Details, once built, are never rebuilt or changed.
//! 4. Details are only built for what actually came close.

use gahturiyu_sim::sim::{geo::V2, world::HOUR, worldgen, World};

fn run(mut w: World, hours: f64, step: f64) -> World {
    let steps = (hours * HOUR / step).round() as usize;
    for _ in 0..steps {
        w.step(step);
    }
    w
}

/// Everything that counts as "what happened", ignoring cached views.
fn assert_same_history(a: &World, b: &World) {
    assert!((a.time - b.time).abs() < 1e-6, "clocks differ: {} vs {}", a.time, b.time);
    assert_eq!(a.hour_done, b.hour_done);
    assert_eq!(a.groups.len(), b.groups.len(), "different number of groups on the road");
    for (ga, gb) in a.groups.iter().zip(&b.groups) {
        assert_eq!(ga.id, gb.id);
        assert_eq!(ga.members, gb.members);
        assert_eq!(ga.kind, gb.kind);
        assert_eq!(ga.legs, gb.legs, "group {} took a different route", ga.id);
        let (pa, pb) = (ga.position_at(a.time), gb.position_at(b.time));
        assert!(pa.dist(pb) < 0.01, "group {} is in a different place", ga.id);
    }
    assert_eq!(a.busy_until, b.busy_until);
    assert_eq!(a.group_of, b.group_of);
}

#[test]
fn step_size_does_not_change_history() {
    let fine = run(worldgen::generate(7), 48.0, 1.0);
    let coarse = run(worldgen::generate(7), 48.0, HOUR);
    let lumpy = run(worldgen::generate(7), 48.0, 337.5);
    assert_same_history(&fine, &coarse);
    assert_same_history(&fine, &lumpy);
    assert!(fine.stats.journeys_started > 50, "the world should actually be doing something");
}

#[test]
fn step_size_does_not_change_history_in_other_worlds() {
    // The default world and one more, so a bug that hides in one seed's
    // particular timing still gets caught.
    for seed in [1, 23] {
        let fine = run(worldgen::generate(seed), 30.0, 2.0);
        let coarse = run(worldgen::generate(seed), 30.0, HOUR);
        assert_same_history(&fine, &coarse);
    }
}

#[test]
fn the_squads_position_does_not_change_history() {
    let here = run(worldgen::generate(11), 36.0, 2.0);
    let mut elsewhere = worldgen::generate(11);
    // Teleport the squad to the far corner, so different groups are in each band.
    elsewhere.teleport_squad(V2::new(18_000.0, 18_000.0));
    let elsewhere = run(elsewhere, 36.0, 2.0);
    assert_same_history(&here, &elsewhere);
}

#[test]
fn details_once_built_are_kept() {
    let mut w = worldgen::generate(3);
    let start = w.squad.pos;
    w = run(w, 2.0, 1.0);
    let seen: Vec<_> = w.people.iter().filter_map(|p| p.detail.clone().map(|d| (p.id, d))).collect();
    assert!(!seen.is_empty());

    // Walk far away, wait, and come back.
    w.order_squad(V2::new(15_000.0, 10_000.0));
    w = run(w, 30.0, 5.0);
    w.order_squad(start);
    w = run(w, 30.0, 5.0);

    for (id, d) in seen {
        assert_eq!(w.people[id as usize].detail.as_ref(), Some(&d), "person {id} changed while out of sight");
    }
}

#[test]
fn details_are_only_built_for_what_came_close() {
    let w = run(worldgen::generate(5), 24.0, 2.0);
    let detailed = w.people.iter().filter(|p| p.detail.is_some()).count();
    assert!(
        detailed < w.people.len() / 4,
        "{detailed} of {} people got details without the squad going anywhere",
        w.people.len()
    );
    assert_eq!(detailed, w.stats.detailed, "the running count drifted from the truth");
}
