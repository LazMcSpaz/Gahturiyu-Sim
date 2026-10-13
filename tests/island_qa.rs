//! Salt Isle, played through: the ways between its places are on land, the
//! compound works like a town (its people at their work by day, the watch
//! at the gate), and the opening's hut keeps its fire overnight.

use gahturiyu_sim::sim::island::Spot;
use gahturiyu_sim::sim::jobs::PlaceKind;
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

fn opening(seed: u64) -> gahturiyu_sim::sim::World {
    worldgen::generate_opening(seed, None, Default::default())
}

#[test]
fn the_ways_between_the_islands_places_stay_on_land() {
    let w = opening(12);
    let isle = w.island().unwrap().clone();
    let legs = [(Spot::Shore, Spot::Hut), (Spot::Hut, Spot::Gate), (Spot::Gate, Spot::Dock), (Spot::Gate, Spot::Quarry), (Spot::Gate, Spot::Woodlot), (Spot::Gate, Spot::Fields), (Spot::Gate, Spot::Briar), (Spot::Gate, Spot::TestQuarter)];
    for (a, b) in legs {
        // (A spot is a thing's middle: set out from beside it.)
        let (from, to) = (w.free_spot(isle.spot(a)), isle.spot(b));
        let (path, _) = w.travel(from, to);
        // (A straight way is just its end; corners come before it.)
        assert!(!path.is_empty() && w.can_walk(from, to), "{a:?} to {b:?}: a way");
        assert!(!w.crosses_sea(from, &path), "{a:?} to {b:?} goes through the water: {path:?}");
        assert!(path.last().unwrap().dist(to) < 15.0, "{a:?} to {b:?} gets there: ends {:?}, {} m short", path.last(), path.last().unwrap().dist(to));
    }
}

#[test]
fn the_compound_works_by_day_and_keeps_its_fire() {
    let mut w = opening(12);
    let isle = w.island().unwrap().clone();
    let t = isle.settlement;
    let fire = w.hut_fire().unwrap();
    // Run to the next day's mid-morning (a storm may keep the outdoor hands in: so to a clear working hour).
    let start = w.time;
    while w.time < start + 48.0 * HOUR {
        w.step(600.0);
        let h = (w.time / HOUR).rem_euclid(24.0);
        if (9.5..11.0).contains(&h) && w.weather_effects(isle.centre, w.time).outdoor_work_ok && w.time > start + 20.0 * HOUR {
            break;
        }
    }
    // The hut's fire still burns.
    assert!(w.campfires.iter().any(|f| f.pos.dist(fire) < 0.01 && f.burning(w.time)), "the hut's fire burns on");
    // Someone of the compound is at work out at the quarry, woodlot or fields; the watch holds the gate.
    let tl = &w.society.towns[t as usize];
    let at_work = |kind: PlaceKind| {
        let place = tl.places.iter().position(|p| p.kind == kind).expect("the place");
        w.settlements[t as usize].residents.iter().filter(|&&p| w.society.lives[p as usize].place == Some(place as u16)).count()
    };
    let out = at_work(PlaceKind::Quarry) + at_work(PlaceKind::Woodlot) + at_work(PlaceKind::Fields);
    assert!(out >= 3, "hands posted to the quarry, woodlot and fields: {out}");
    assert!(at_work(PlaceKind::GuardPost) >= 2, "a watch at the gate");
    // The debtors are still bound, and the Binders still guard.
    let bound = w.bonds.iter().filter(|b| b.town == t && b.slave).count();
    assert_eq!(bound, gahturiyu_sim::sim::island::DEBTORS.iter().sum::<usize>());
    // And the squad's one member is where they were left (nothing dragged them off), not fighting, not dead.
    let m = w.squad.members[0];
    assert!(!w.people[m as usize].dead && w.squad_battle().is_none());
    assert!(isle.contains(w.person_pos(m)), "still on the island: {:?}", w.person_pos(m));
}

