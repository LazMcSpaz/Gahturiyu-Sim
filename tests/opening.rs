//! The game's opening on Salt Isle (`tutorial.rs`, `island.rs`): one
//! person wrecked on the shore in a storm, a hut with a fire and raw food,
//! and lessons that come up and go in order as the world says they're done.

use gahturiyu_sim::sim::condition::CHILLED;
use gahturiyu_sim::sim::island::{Spot, HUT_BUILDING};
use gahturiyu_sim::sim::tutorial::{Path, AT_HUT, LESSONS, WALKED};
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

fn opening(seed: u64) -> gahturiyu_sim::sim::World {
    worldgen::generate_opening(seed, None, Default::default())
}

#[test]
fn wrecked_on_the_shore_in_a_storm() {
    let w = opening(5);
    let isle = w.island().unwrap();
    assert_eq!(w.squad.members.len(), 1, "the opening is one person");
    let shore = isle.spot(Spot::Shore);
    assert!(w.squad.pos.dist(shore) < 15.0, "wakes on the shore: {:?} vs {shore:?}", w.squad.pos);
    assert!(isle.contains(w.squad.pos));
    let fx = w.weather_effects(shore, w.time);
    assert!(!fx.outdoor_work_ok || w.time == 6.0 * HOUR, "a storm (or none for ten days): {fx:?}");
    assert!(gahturiyu_sim::sim::stealth::daylight(w.time) > 0.25, "by daylight");
    let m = w.squad.members[0];
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!(c.cold >= CHILLED, "chilled from the sea: {}", c.cold);
    // In the clothes they were wrecked in and a knife; no pack, no coin.
    assert_eq!(w.squad_count(gahturiyu_sim::sim::items::id("knife")), 1);
    assert_eq!(w.squad_count(gahturiyu_sim::sim::items::id("coin")), 0);
    assert_eq!(w.squad_count(gahturiyu_sim::sim::items::id("bedroll")), 0);
    // The tutorial is on, at its first lesson.
    let tu = w.tutorial().expect("the tutorial");
    assert_eq!(tu.path, None);
    assert_eq!(w.lesson_now().map(|x| x.key), Some("wake"));
    // The plain start has none of it.
    let plain = worldgen::generate(5);
    assert!(plain.tutorial().is_none() && plain.lesson_now().is_none() && plain.squad.members.len() == 4);
}

#[test]
fn the_hut_has_a_fire_and_food() {
    let w = opening(9);
    let isle = w.island().unwrap();
    let hut = &w.settlements[isle.settlement as usize].buildings[HUT_BUILDING];
    assert!(hut.pos.dist(isle.spot(Spot::Hut)) < 0.01);
    let fire = w.hut_fire().unwrap();
    assert!(fire.dist(hut.pos) < 12.0 && fire.dist(hut.pos) > hut.size * 0.5, "the fire by the door");
    let f = w.campfires.iter().find(|f| f.pos.dist(fire) < 0.01).expect("a campfire there");
    assert!(f.burning(w.time) && f.burning(w.time + 24.0 * HOUR));
    assert!(w.fire_at(fire.add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0))).is_some(), "something to cook over");
    let food: u16 = w.ground.iter().filter(|g| g.pos.dist(fire) < 4.0).map(|g| g.count).sum();
    assert!(food >= 5, "raw food by the fire: {food}");
    assert!(w.ground.iter().filter(|g| g.pos.dist(fire) < 4.0).all(|g| g.owner.is_none()), "nobody's: taking it is no crime");
}

#[test]
fn lessons_come_up_in_order_and_go_when_done() {
    let mut w = opening(3);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    assert_eq!(w.lesson_now().map(|x| x.key), Some("wake"));
    // Walking off the spot wakes them to the island.
    let from = w.squad.pos;
    w.order_squad(from.add(gahturiyu_sim::sim::geo::V2::new(WALKED + 10.0, 0.0)));
    for _ in 0..200 {
        w.step(0.5);
        if w.lesson_now().map(|x| x.key) != Some("wake") {
            break;
        }
    }
    assert_eq!(w.lesson_now().map(|x| x.key), Some("shelter"), "after walking: {:?}", w.tutorial());
    assert!(w.tutorial().unwrap().is_done("wake"));
    // At the hut's fire they're sheltered.
    let fire = w.hut_fire().unwrap();
    w.teleport_squad(fire.add(gahturiyu_sim::sim::geo::V2::new(AT_HUT * 0.5, 0.0)));
    w.step(0.5);
    assert_eq!(w.lesson_now().map(|x| x.key), Some("cook"));
    // A meal on them and cooking is done; the stranger's lesson is off
    // until it's built, so the ship comes next.
    w.people[m as usize].cond.as_mut().unwrap().meal = Some(gahturiyu_sim::sim::effects::Does::Warmth);
    w.step(0.5);
    assert!(w.tutorial().unwrap().is_done("cook"));
    assert_eq!(w.lesson_now().map(|x| x.key), Some("sail"), "the ship, with the stranger's lesson off");
    // A system can mark a path's lesson done: that sets the path.
    w.lesson_done("labour");
    assert_eq!(w.tutorial().unwrap().path, Some(Path::Caught));
    assert!(w.on_island());
    w.end_tutorial();
    assert!(!w.on_island() && w.lesson_now().is_none());
    // The table is sane: keys unique, the shared ones first.
    let mut keys: Vec<&str> = LESSONS.iter().map(|x| x.key).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), LESSONS.len());
    let first_pathed = LESSONS.iter().position(|x| x.path != Path::Both).unwrap();
    assert!(LESSONS[..first_pathed].iter().all(|x| x.path == Path::Both));
    let _ = isle;
}

#[test]
fn the_opening_saves_and_loads() {
    let mut w = opening(4);
    w.step(1.0);
    let bytes = w.save_bytes();
    let back = gahturiyu_sim::sim::World::load_bytes(&bytes).unwrap();
    assert_eq!(back.tutorial(), w.tutorial());
    assert_eq!(back.island(), w.island());
    assert_eq!(back.lesson_now().map(|x| x.key), w.lesson_now().map(|x| x.key));
}
