//! The solid lists, and the spots and ways found among them, are kept until
//! something they're built from changes (`solid::Cache`), and the world says
//! when (`World::solid_changed`) instead of every ask looking it all over.
//! These run with the check on (`solid::check_marks`): every ask also works
//! the marks out in full and stops if anything changed untold. A copy of the
//! world keeps nothing, so it's the answer worked out afresh.

use gahturiyu_sim::sim::{
    base::{def_index, Land, Plan, BUILDINGS},
    bedding::BedKind,
    buildings::door_of,
    geo::V2,
    jobs::PlaceKind,
    solid::{self, Id},
    worldgen, World,
};

/// The spot to stand at `p`, from the kept lists and worked out afresh.
fn same_spot(w: &World, p: V2, what: &str) -> V2 {
    let kept = w.free_spot(p);
    assert_eq!(kept, w.clone().free_spot(p), "{what}: the kept spot is the one found afresh");
    kept
}

/// Somewhere in the wilds near the squad where a base can go.
fn base_ground(w: &World) -> V2 {
    let start = w.squad.pos;
    for ring in 0..60 {
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let p = start.add(V2::new(a.cos(), a.sin()).scale(80.0 + ring as f32 * 30.0));
            let mut c = w.clone();
            let Ok(bid) = c.found_base(p) else { continue };
            if c.base(bid).unwrap().land == Land::Wilds && c.check_place(&mut hut(p)).is_ok() {
                return p;
            }
        }
    }
    panic!("nowhere for a base");
}

fn hut(p: V2) -> Plan {
    let def = def_index("hut");
    Plan { def, at: p.add(V2::new(-9.0, 4.0)), rot: 0.0, w: BUILDINGS[def].w, replaces: None }
}

#[test]
fn a_base_built_and_pulled_down_is_seen_at_the_next_ask() {
    solid::check_marks(true);
    let mut w = worldgen::generate(1);
    let p = base_ground(&w);
    let at = hut(p).at;
    w.teleport_squad(p.add(V2::new(0.0, -2.0)));
    assert_eq!(same_spot(&w, at, "open ground"), at, "nothing there yet");
    // The camp marker, then a hut's site: the spot moves out of it at once.
    let bid = w.found_base(p).expect("a base");
    let id = w.place_building(hut(p)).expect("a hut's site");
    let beside = same_spot(&w, at, "the hut's site");
    assert_ne!(beside, at, "the site is in the way");
    assert!(w.base(bid).unwrap().building(id).is_some_and(|b| !b.standing()));
    // Built (everyone lends a hand), it still stands there.
    let hunter = w.squad.members[0];
    for &m in &w.squad.members.clone() {
        if let Some(d) = w.people[m as usize].detail.as_mut() {
            if !d.crafts.contains(&gahturiyu_sim::sim::stats::Skill::Carpentry) {
                d.crafts.push(gahturiyu_sim::sim::stats::Skill::Carpentry);
            }
        }
    }
    for (k, n) in [("timber", 60), ("seareed", 16)] {
        w.people[hunter as usize].detail.as_mut().unwrap().gear.add(gahturiyu_sim::sim::items::id(k), n);
    }
    w.store_materials(bid);
    let end = w.time + 30.0 * 3600.0;
    while w.time < end && !w.base(bid).unwrap().building(id).is_some_and(|b| b.standing()) {
        w.step(60.0);
        same_spot(&w, at, "while it goes up");
    }
    assert!(w.base(bid).unwrap().building(id).is_some_and(|b| b.standing()), "the hut went up");
    assert_ne!(same_spot(&w, at, "the hut standing"), at);
    // Pulled down: the ground is clear again.
    assert!(w.deconstruct(bid, id));
    assert_eq!(same_spot(&w, at, "the hut pulled down"), at, "clear again");
}

#[test]
fn tents_doors_and_a_town_laid_out_again() {
    solid::check_marks(true);
    let mut w = worldgen::generate(2);
    for _ in 0..20 {
        w.step(30.0);
    }
    // A tent pitched where someone was going to stand: tents let you by, so
    // the spot is the same, and the same as found afresh.
    let m = w.squad.members[0];
    let at = w.squad.pos.add(V2::new(6.0, 0.0));
    let before = same_spot(&w, at, "before the tent");
    if w.can_lay(m, BedKind::Tent) {
        w.order_lay(m, BedKind::Tent, at, false).expect("pitched");
        for _ in 0..80 {
            w.step(15.0);
        }
        assert!(w.tents().iter().any(|t| t.1.dist(at) < 3.0), "the tent stands");
    }
    assert_eq!(same_spot(&w, at, "beside the tent"), before);
    // Doors are open by day and locked at night; the spots inside are the same.
    let t = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    let d = (0..w.settlements[t as usize].buildings.len() as u16).find_map(|i| door_of(&w.settlements[t as usize], i)).expect("a door");
    let day = same_spot(&w, d.centre, "inside by day");
    let night = w.time + 14.0 * 3600.0;
    while w.time < night {
        w.step(120.0);
    }
    assert_eq!(same_spot(&w, d.centre, "inside at night"), day);
    // A workplace moved (as when a town is laid out again), told: followed.
    let j = w.society.towns[0].places.iter().position(|p| p.kind == PlaceKind::Hall).expect("a hall");
    let was = w.solid(Id::Place(0, j as u16, 0)).unwrap().shape.centre();
    let in_hall = same_spot(&w, was, "the hall where it was");
    w.society.towns[0].places[j].pos = was.add(V2::new(40.0, 0.0));
    w.solid_changed();
    assert!(w.solid(Id::Place(0, j as u16, 0)).unwrap().shape.centre().dist(was.add(V2::new(40.0, 0.0))) < 1e-3);
    let after = same_spot(&w, was, "where the hall was");
    assert!(after != in_hall || in_hall == was, "the spot follows the hall away");
}

#[test]
fn a_busy_world_never_works_from_stale_lists() {
    // Two days near a town and out in the wilds: townsfolk standing about,
    // bandit camps fought and emptied, a dawn's tally (towns laid out
    // again). The check stops the run if anything changed untold.
    solid::check_marks(true);
    let mut w = worldgen::generate(1);
    // (Two-second steps, near the window's: townsfolk looked up often.)
    let end = w.time + 2.0 * 24.0 * 3600.0;
    while w.time < end {
        w.step(2.0);
    }
}

#[test]
#[should_panic(expected = "without World::solid_changed")]
fn a_change_left_untold_is_caught() {
    solid::check_marks(true);
    let mut w = worldgen::generate(2);
    let j = w.society.towns[0].places.iter().position(|p| p.kind == PlaceKind::Hall).expect("a hall");
    let was = w.solid(Id::Place(0, j as u16, 0)).unwrap().shape.centre();
    w.free_spot(was);
    w.society.towns[0].places[j].pos = was.add(V2::new(40.0, 0.0));
    w.free_spot(was);
}
