//! The caught path's shifts (`tutorial.rs`, `law.rs`): a squad member bound
//! on Salt Isle is walked to the woodlot or the quarry by day and works,
//! what they cut goes to the lord, the deeds count, and every third day
//! they're put to the workyard.

use gahturiyu_sim::sim::island::Spot;
use gahturiyu_sim::sim::tutorial::{Deed, Path, Shift, SHIFT_HOURS, SHIFT_UNITS};
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

fn opening(seed: u64) -> gahturiyu_sim::sim::World {
    worldgen::generate_opening(seed, None, Default::default())
}

/// Bound as the compound's slave, on the caught path, the shared opening done.
fn enslaved(seed: u64) -> gahturiyu_sim::sim::World {
    let mut w = opening(seed);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    for k in ["wake", "salvage", "shelter", "cook", "night_run"] {
        w.lesson_done(k);
    }
    w.set_path(Path::Caught);
    let t = w.time;
    w.bond(m, isle.settlement, Some(isle.steward), t, f64::INFINITY);
    w.bonds.iter_mut().find(|b| b.who == m).unwrap().slave = true;
    w.teleport_squad(isle.spot(Spot::Yard));
    w
}

fn run_to_hour(w: &mut gahturiyu_sim::sim::World, h: f64) {
    let day = (w.time / HOUR / 24.0).floor();
    let mut t = (day * 24.0 + h) * HOUR;
    if t <= w.time {
        t += 24.0 * HOUR;
    }
    while w.time < t {
        w.step((t - w.time).min(300.0));
    }
}

#[test]
fn the_island_has_faces_to_work() {
    let w = opening(16);
    let t = w.island().unwrap().settlement;
    let faces: Vec<_> = w.deposits.iter().filter(|d| d.town == t).collect();
    assert!(faces.len() >= 2, "a woodlot and a quarry face: {faces:?}");
    assert!(faces.iter().any(|d| gahturiyu_sim::sim::items::item(d.item).key == "timber"));
    assert!(faces.iter().any(|d| gahturiyu_sim::sim::items::item(d.item).key == "rock"));
    // The rota is the clock: off at night, a face by day, the workyard every third day.
    let day0 = (w.time / HOUR / 24.0).floor() * 24.0 * HOUR;
    assert_eq!(w.island_shift(day0 + 3.0 * HOUR), None);
    let mut faces_seen = 0;
    let mut yards = 0;
    for d in 0..6 {
        match w.island_shift(day0 + (d as f64 * 24.0 + SHIFT_HOURS.0 + 1.0) * HOUR) {
            Some(Shift::Face(..)) => faces_seen += 1,
            Some(Shift::Workyard(_)) => yards += 1,
            None => panic!("a shift by day"),
        }
    }
    assert_eq!((faces_seen, yards), (4, 2));
}

#[test]
fn a_bound_member_works_the_shift_for_the_lord() {
    let mut w = enslaved(16);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    assert_eq!(w.lesson_now().map(|x| x.key), Some("labour"));
    // To a face day: run until the shift that's a face, an hour in.
    for _ in 0..3 {
        run_to_hour(&mut w, SHIFT_HOURS.0 + 1.0);
        if matches!(w.island_shift(w.time), Some(Shift::Face(..))) {
            break;
        }
    }
    let Some(Shift::Face(id, at)) = w.island_shift(w.time) else { panic!("a face day") };
    // Walked there and at it.
    run_to_hour(&mut w, SHIFT_HOURS.0 + 4.0);
    assert!(w.person_pos(m).dist(at) < 12.0, "at the face: {:?} vs {at:?}", w.person_pos(m));
    assert!(w.labour.iter().any(|l| l.who == m && l.deposit == id), "working it");
    let deeds = w.island_deeds().unwrap();
    assert!(deeds.laboured >= SHIFT_UNITS, "units cut: {deeds:?}");
    // Nothing of it in their pack: it's the lord's.
    let face_item = w.deposit(id).unwrap().item;
    assert_eq!(w.squad_count(face_item), 0);
    assert!(w.tutorial().unwrap().is_done("labour"));
    assert_eq!(w.lesson_now().map(|x| x.key), Some("workyard"));
    // Dusk: walked back, the work left.
    run_to_hour(&mut w, SHIFT_HOURS.1 + 1.0);
    assert!(!w.labour.iter().any(|l| l.who == m), "off shift");
    assert!(w.person_pos(m).dist(isle.spot(Spot::Hall)) < 30.0, "back by the hall: {:?}", w.person_pos(m));
    assert!(w.is_bonded(m, w.time));
}

#[test]
fn a_workyard_day_and_a_thing_made() {
    let mut w = enslaved(16);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    w.lesson_done("labour");
    for _ in 0..4 {
        run_to_hour(&mut w, SHIFT_HOURS.0 + 1.0);
        if matches!(w.island_shift(w.time), Some(Shift::Workyard(_))) {
            break;
        }
    }
    assert!(matches!(w.island_shift(w.time), Some(Shift::Workyard(_))), "a workyard day");
    run_to_hour(&mut w, SHIFT_HOURS.0 + 3.0);
    assert!(w.person_pos(m).dist(isle.spot(Spot::Workyard)) < 14.0, "at the workyard: {:?}", w.person_pos(m));
    assert!(!w.labour.iter().any(|l| l.who == m));
    // A thing made in the compound counts, and the lesson goes.
    assert_eq!(w.lesson_now().map(|x| x.key), Some("workyard"));
    w.island_deed(Deed::Made);
    w.step(0.5);
    assert!(w.tutorial().unwrap().is_done("workyard"));
    assert_eq!(w.island_deeds().unwrap().made, 1);
    // Off the island nothing counts.
    w.end_tutorial();
    w.island_deed(Deed::FightWon);
    assert_eq!(w.island_deeds().unwrap().fights_won, 0);
}
