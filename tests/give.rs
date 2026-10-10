//! Handing things between squad members (playtest 1: "no way to give an
//! item to a squadmate"), and recruits who bring their own bread.

use gahturiyu_sim::sim::{geo::V2, items, worldgen};

#[test]
fn a_squadmate_can_hand_things_over_when_near() {
    let mut w = worldgen::generate(1);
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let bread = items::id("flatbread");
    w.people[a as usize].detail.as_mut().unwrap().gear.add(bread, 4);
    let k = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == bread).unwrap();
    let had = w.count_of(b, "flatbread");
    // Too far apart: told why.
    let kb = w.squad.index(b).unwrap();
    let far = w.squad.at[kb].add(V2::new(40.0, 0.0));
    w.squad.at[kb] = far;
    w.squad.goal[kb] = far;
    let why = w.give_entry(a, k, b).unwrap_err();
    assert!(why.contains("m away"), "{why}");
    // Side by side: it's theirs.
    let ka = w.squad.index(a).unwrap();
    let near = w.squad.at[ka].add(V2::new(1.0, 0.0));
    w.squad.at[kb] = near;
    w.squad.goal[kb] = near;
    let stack = w.count_of(a, "flatbread");
    assert!(stack >= 4);
    w.give_entry(a, k, b).unwrap();
    assert_eq!(w.count_of(b, "flatbread"), had + stack, "the whole stack");
    assert_eq!(w.count_of(a, "flatbread"), 0);
}

#[test]
fn recruits_bring_their_own_bread() {
    let mut w = worldgen::generate(1);
    // Run to the first dawn so who's out of work is sorted out.
    while w.time < 30.0 * 3600.0 {
        w.step(600.0);
    }
    let me = w.squad.members[0];
    let town = (0..w.settlements.len()).min_by(|&a, &b| w.settlements[a].pos.dist(w.squad.pos).total_cmp(&w.settlements[b].pos.dist(w.squad.pos))).unwrap();
    let willing = w.settlements[town].residents.iter().copied().find(|&p| w.join_terms(p) == Some(0));
    let p = willing.expect("someone free to join near the start");
    w.recruit(p, me).unwrap();
    assert!(w.count_of(p, "flatbread") >= 3, "they came with bread");
}
