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

/// Round 1 and 3: a ruin should hold something to search, not a pile on the
/// ground. A ruin's cache is a locked chest and a crate with something to
/// read; a lair's is an old chest. Nobody's: taking is no crime.
#[test]
fn ruin_caches_are_containers() {
    use gahturiyu_sim::sim::{
        containers::{is_wild, Owner},
        items::{self, Kind},
        layout::Holder,
        loot::Source,
    };
    let mut w = worldgen::generate(1);
    for r in &w.ruins {
        let cs: Vec<_> = w.ruin_containers(r.id).cloned().collect();
        assert!(cs.iter().all(|c| is_wild(c.id) && c.owner == Owner::Nobody));
        assert!(w.ground.iter().all(|g| g.pos.dist(r.pos) > 10.0), "nothing lying loose");
        match r.kind {
            RuinKind::Ruin => {
                assert!(cs.iter().any(|c| c.what == Holder::Chest && c.lock > 0.0), "a locked chest");
                let reading = cs.iter().flat_map(|c| c.items.iter()).any(|e| matches!(items::item(e.0).kind, Kind::Text(_) | Kind::Notes(_) | Kind::Scroll(_) | Kind::Manual(_)));
                assert!(reading, "something to read");
            }
            RuinKind::Lair(_) => assert!(cs.iter().any(|c| c.what == Holder::Chest && c.lock == 0.0), "an old chest"),
        }
    }
    // With its wardens gone, a ruin's chest is picked, opened and emptied,
    // and nobody calls it theft.
    let r = w.ruins.iter().find(|r| r.kind == RuinKind::Ruin).unwrap().clone();
    for m in w.group(r.guards.unwrap()).unwrap().members.clone() {
        w.people[m as usize].dead = true;
    }
    let chest = w.ruin_containers(r.id).find(|c| c.what == Holder::Chest).unwrap().id;
    w.teleport_squad(r.pos.add(V2::new(8.0, 0.0)));
    let me = w.squad.members[0];
    w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("lockpick"), 20);
    for k in gahturiyu_sim::sim::stats::SKILLS {
        if k == gahturiyu_sim::sim::stats::Skill::Security {
            w.people[me as usize].stats.set_skill(k, 80.0);
        }
    }
    assert!(w.order_pick_container(me, chest));
    let t0 = w.time;
    while w.container_locked(chest) && w.time < t0 + 3600.0 {
        w.step(1.0);
    }
    assert!(!w.container_locked(chest), "picked: {:?}", w.log);
    assert!(w.order_search(me, chest));
    while w.searching_now(me).is_none() && w.time < t0 + 4000.0 {
        w.step(0.5);
    }
    assert_eq!(w.searching_now(me), Some(chest));
    let coin = w.count_of(me, "coin");
    let n = w.take_all_from(me, Source::Chest(chest));
    assert!(n >= 2, "took {n}");
    assert!(w.count_of(me, "coin") > coin);
    assert!(w.container(chest).unwrap().items.is_empty());
    assert!(!w.log.iter().any(|l| l.1.contains("seen stealing") || l.1.contains("seen picking")), "{:?}", w.log);
}
