//! Saving and loading: a loaded world carries on exactly as the saved one.

use gahturiyu_sim::sim::{geo::V2, save::LoadError, world::HOUR, worldgen, World};

/// Everything that happens, written out so two worlds can be compared.
/// Maps are sorted first (their order in memory means nothing).
fn fingerprint(w: &World) -> String {
    fn sorted<K: Ord + std::fmt::Debug + Clone, V: std::fmt::Debug>(m: &std::collections::HashMap<K, V>) -> String {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        format!("{v:?}")
    }
    let mut s = String::new();
    s += &format!("{:?} {:?}\n", w.time, w.hour_done);
    s += &format!("{:?}\n", w.people);
    s += &format!("{:?}\n", w.groups);
    s += &format!("{:?}\n", w.squad);
    s += &format!("{:?}\n", w.battles);
    s += &format!("{:?}\n", w.npc_fights);
    s += &format!("{:?} {:?}\n", w.encounters, w.camps);
    s += &format!("{:?} {:?}\n", w.corpses, w.ground);
    s += &format!("{:?}\n", w.log);
    s += &format!("{:?} {:?}\n", w.busy_until, w.group_of);
    s += &format!("{:?} {:?}\n", w.standing, w.quests);
    s += &sorted(&w.bounty);
    s += &sorted(&w.news);
    s += &sorted(&w.torches);
    s += &sorted(&w.fighting);
    s += &sorted(&w.carried);
    s += &sorted(&w.suspicion);
    s += &sorted(&w.held);
    s += &sorted(&w.cast_count);
    s += &format!("{:?} {:?} {:?} {:?}\n", w.rituals, w.circles, w.boons, w.wards);
    s += &format!("{:?}\n{:?}\n", w.society, w.stations);
    s
}

fn run(w: &mut World, secs: f64, step: f64) {
    let n = (secs / step).round() as usize;
    for _ in 0..n {
        w.step(step);
    }
}

fn assert_same(a: &World, b: &World) {
    let (fa, fb) = (fingerprint(a), fingerprint(b));
    if fa != fb {
        let line = fa.lines().zip(fb.lines()).position(|(x, y)| x != y).unwrap_or(0);
        panic!("the loaded world went its own way (first difference in part {line})");
    }
}

#[test]
fn a_loaded_world_carries_on_as_the_saved_one_would_have() {
    let mut w = worldgen::generate(3);
    let start = w.squad.pos;
    w.order_squad(start.add(V2::new(600.0, 200.0)));
    run(&mut w, 3.0 * HOUR, 2.0);

    let bytes = w.save_bytes();
    let mut back = World::load_bytes(&bytes).expect("loads");
    assert_same(&w, &back);

    // Both carry on, with the same orders, and stay the same.
    for x in [&mut w, &mut back] {
        x.order_squad(start.add(V2::new(-300.0, 900.0)));
        run(x, 10.0 * HOUR, 2.0);
    }
    assert_same(&w, &back);
    assert!(w.stats.journeys_started > 0);
}

#[test]
fn a_fight_saved_halfway_ends_the_same() {
    let mut w = worldgen::generate(7);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(70.0, 0.0)));
    let mut waited = 0.0;
    while w.battles.is_empty() && waited < 2.0 * HOUR {
        w.step(0.5);
        waited += 0.5;
    }
    assert!(!w.battles.is_empty(), "no fight broke out by the camp");
    run(&mut w, 4.0, 1.0 / 30.0);

    let mut back = World::load_bytes(&w.save_bytes()).expect("loads");
    for x in [&mut w, &mut back] {
        run(x, 90.0, 1.0 / 30.0);
        run(x, 2.0 * HOUR, 2.0);
    }
    assert_same(&w, &back);
}

#[test]
fn the_land_comes_back_from_the_seed() {
    let w = worldgen::generate(11);
    let back = World::load_bytes(&w.save_bytes()).unwrap();
    for p in [w.squad.pos, w.settlements[0].pos, V2::new(5_000.0, 9_000.0)] {
        assert_eq!(w.terrain.height(p), back.terrain.height(p));
        assert_eq!(w.terrain.ground(p), back.terrain.ground(p));
    }
    assert_eq!(w.routes.roads, back.routes.roads);
    // The land is most of the world's size; it isn't in the save.
    assert!(w.save_bytes().len() < 4_000_000, "save is {} bytes", w.save_bytes().len());
}

#[test]
fn saves_round_trip_through_a_file_and_refuse_strangers() {
    let w = worldgen::generate(5);
    let dir = std::env::temp_dir().join(format!("gaht-save-test-{}", std::process::id()));
    let path = dir.join("quick.sav");
    w.save_to(&path).unwrap();
    let back = World::load_from(&path).unwrap();
    assert_same(&w, &back);

    assert!(matches!(World::load_bytes(b"hello"), Err(LoadError::NotASave)));
    let mut old = w.save_bytes();
    old[4] = old[4].wrapping_add(1);
    assert!(matches!(World::load_bytes(&old), Err(LoadError::OldFormat(_))));
    let mut cut = w.save_bytes();
    cut.truncate(cut.len() / 2);
    assert!(matches!(World::load_bytes(&cut), Err(LoadError::Corrupt(_))));
    let _ = std::fs::remove_dir_all(dir);
}
