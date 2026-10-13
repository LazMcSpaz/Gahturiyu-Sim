//! The dev panel's squad and place calls (Laz, 13 Oct: "a little window ...
//! to spawn, turn things on or off, to test these features").

use gahturiyu_sim::sim::{geo::V2, island::Spot, items, worldgen};

#[test]
fn heal_knock_out_and_bind() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    w.dev_knock_out(m);
    assert!(w.is_down(m), "out cold");
    w.people[m as usize].wounds.bleed = 2.0;
    w.dev_bind_all();
    assert_eq!(w.bleeding(m), 0.0);
    w.dev_heal_all();
    assert!(!w.is_down(m), "whole again");
    assert!(!w.people[m as usize].wounds.is_hurt(w.time));
}

#[test]
fn starve_chill_and_soak_the_lead() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    w.dev_starve(m);
    assert!(w.people[m as usize].cond.as_ref().unwrap().hunger_at(w.time) >= 89.0);
    w.dev_chill(m);
    assert!(w.people[m as usize].cond.as_ref().unwrap().cold >= 69.0);
    w.dev_soak(m);
    assert!(w.soak_at(m, w.time).0 >= 0.99, "soaked through");
}

#[test]
fn a_fire_and_a_torch_by_the_lead() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let at = w.person_pos(m);
    let fires = w.campfires.len();
    w.dev_fire(at);
    assert_eq!(w.campfires.len(), fires + 1);
    let torches = w.standing.len();
    assert!(w.dev_torch(m));
    assert_eq!(w.standing.len(), torches + 1);
}

#[test]
fn go_to_a_town_and_to_the_island() {
    let mut w = worldgen::generate(1);
    w.dev_go_island(Spot::Shore);
    let isle = w.island().unwrap().clone();
    assert!(isle.contains(w.squad.pos), "on Salt Isle");
    w.dev_go_town();
    assert!(!isle.contains(w.squad.pos));
    let near = w.settlements.iter().map(|s| s.pos.dist(w.squad.pos)).fold(f32::MAX, f32::min);
    assert!(near < 60.0, "by a town's hearth: {near}");
}

#[test]
fn into_the_arena_and_foes_across_it() {
    let mut w = worldgen::generate(1);
    assert!(w.dev_to_arena());
    let q = w.test_quarter().unwrap().clone();
    assert!(w.squad.pos.dist(q.squad_side()) < 1.0, "on the arena's west side");
    // A band called up there stands across the arena, inside the walls.
    let band = w.dev_band(q.foe_side(), 3, false);
    let at = w.group(band).unwrap().position_at(w.time);
    assert!(q.contains(at) && at.dist(q.foe_side()) < 1.0);
    assert!(at.x > w.squad.pos.x, "across from the squad");
}

#[test]
fn the_building_arenas_store_is_stocked() {
    let mut w = worldgen::generate(1);
    let start = w.squad.pos;
    let bid = (0..200)
        .find_map(|k| {
            let a = (k % 12) as f32 / 12.0 * std::f32::consts::TAU;
            w.found_base(start.add(V2::new(a.cos(), a.sin()).scale(80.0 + (k / 12) as f32 * 30.0))).ok()
        })
        .expect("somewhere for a base");
    let put = w.dev_stock_base(bid);
    assert!(put > 100, "{put} things in");
    let b = w.base(bid).unwrap();
    assert!(b.buildings.iter().any(|bl| bl.def().key == "storehouse" && bl.standing()), "a storehouse up");
    assert_eq!(b.count_in_store(items::id("timber")), 80);
    assert!(b.count_in_store(items::id("flatbread")) >= 40);
    assert!(b.load() <= b.capacity(), "{} of {} kg", b.load(), b.capacity());
}
