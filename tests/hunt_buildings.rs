//! Bugs found in the bug hunt (the wild), shown by tests. Each is ignored
//! with its ID from `claude/bugs-buildings.md` until it's fixed; run them
//! with `cargo test --release --test hunt_buildings -- --ignored`.

use gahturiyu_sim::sim::{geo::V2, world::HOUR, worldgen};

#[test]
#[ignore = "BL-1"]
fn bl_1_an_id_that_does_not_exist_is_refused_not_a_crash() {
    let mut w = worldgen::generate(1);
    let all = w.squad.members.clone();
    let nobody = w.people.len() as u32 + 100;
    assert!(!w.attack(&all, nobody));
    assert!(!w.order_carry(all[0], nobody));
}

#[test]
#[ignore = "BL-2"]
fn bl_2_an_order_to_nowhere_leaves_the_squad_where_it_is() {
    let mut w = worldgen::generate(1);
    let before = w.squad.pos;
    w.order_squad(V2::new(f32::NAN, f32::NAN));
    w.step(1.0);
    assert!(w.squad.pos.x.is_finite() && w.squad.pos.y.is_finite(), "the squad's place is lost");
    assert!(w.squad.pos.dist(before) < 5.0);
}

#[test]
#[ignore = "BL-10"]
fn bl_10_attack_needs_the_target_somewhere_near() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    // A bandit a couple of kilometres off.
    let far = w.camps.iter().map(|c| c.pos).filter(|p| p.dist(here) > 1500.0).min_by(|a, b| a.dist(here).total_cmp(&b.dist(here))).expect("a far camp");
    w.teleport_squad(here);
    let bandit = w.people.iter().find(|p| p.bandit && w.person_pos(p.id).dist(far) < 60.0).map(|p| p.id).expect("a bandit at that camp");
    let all = w.squad.members.clone();
    assert!(!w.attack(&all, bandit), "a fight started with a bandit {:.0} m away", w.person_pos(bandit).dist(here));
    let _ = HOUR;
}
