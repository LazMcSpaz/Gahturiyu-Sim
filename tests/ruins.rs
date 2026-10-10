//! Ruins and lairs: placed from the seed, guarded, each with a cache.

use gahturiyu_sim::sim::{geo::V2, ruins::{self, RuinKind}, worldgen};

#[test]
fn ruins_and_lairs_are_placed_guarded_and_stocked() {
    ruins::check_tables().unwrap();
    let w = worldgen::generate(1);
    let warded = w.ruins.iter().filter(|r| r.kind == RuinKind::Ruin).count();
    let lairs = w.ruins.iter().filter(|r| matches!(r.kind, RuinKind::Lair(_))).count();
    assert!(warded >= 3, "{warded} ruins");
    assert!(lairs >= 1, "{lairs} lairs");
    for r in &w.ruins {
        assert!(w.ruin_held(r.id), "{:?} has its guard", r.kind);
        assert!(w.ruin_cache(r.id) >= 2, "{:?} has a cache", r.kind);
        assert!(w.settlements.iter().all(|s| s.pos.dist(r.pos) > s.radius()), "not in a town");
    }
    // Wardens are harder than roadside bandits.
    let might = |gid| {
        let g = w.group(gid).unwrap();
        g.members.iter().map(|&m| w.people[m as usize].might).sum::<f32>() / g.members.len() as f32
    };
    let wardens: Vec<_> = w.ruins.iter().filter_map(|r| r.guards).collect();
    let roadside: Vec<_> = w.camps.iter().map(|c| c.group).filter(|g| !wardens.contains(g)).collect();
    let avg = |v: &[u32]| v.iter().map(|&g| might(g)).sum::<f32>() / v.len() as f32;
    assert!(avg(&wardens) > avg(&roadside), "wardens {} vs roadside {}", avg(&wardens), avg(&roadside));
    // The same world gets the same.
    assert_eq!(worldgen::generate(1).ruins, w.ruins);
}

#[test]
fn coming_upon_a_ruin_is_noted() {
    let mut w = worldgen::generate(1);
    let r = w.ruins[0].clone();
    w.teleport_squad(r.pos.add(V2::new(120.0, 0.0)));
    w.step(0.1);
    assert!(w.ruins[0].found);
    assert!(w.log.iter().any(|l| l.1.contains("come upon")), "{:?}", w.log);
}
