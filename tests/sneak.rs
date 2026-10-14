//! The sneak path through Salt Isle's compound (`tutorial.rs`,
//! `island.rs`): the palisade and its gate are solid; the lessons come up
//! and go as the world says: in by night sneaking, the stores gone through,
//! the gate passed in a smock by day, and sneaking aboard the ship.

use gahturiyu_sim::sim::ferry::Berth;
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::island::Spot;
use gahturiyu_sim::sim::solid::{Id, What};
use gahturiyu_sim::sim::tutorial::Path;
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

fn opening(seed: u64) -> gahturiyu_sim::sim::World {
    worldgen::generate_opening(seed, None, Default::default())
}

/// Run to the next hour of the day `h` (0..24).
fn run_to_hour(w: &mut gahturiyu_sim::sim::World, h: f64) {
    let day = (w.time / HOUR / 24.0).floor();
    let mut t = (day * 24.0 + h) * HOUR;
    if t <= w.time {
        t += 24.0 * HOUR;
    }
    while w.time < t {
        w.step((t - w.time).min(600.0));
    }
}

#[test]
fn the_palisade_is_solid_with_one_gate() {
    let w = opening(14);
    let isle = w.island().unwrap().clone();
    let (c, half) = isle.palisade();
    let near = w.solids_near(c, half.x + half.y);
    assert!(near.iter().filter(|t| matches!(t.id, Id::Palisade(_)) && t.what == What::Wall).count() == 5, "the palisade's five stretches");
    assert!(near.iter().any(|t| t.id == Id::PalisadeGate && t.what == What::Gate), "its gate");
    // Everything of the compound is inside it; the hut, the dock and the briar outside.
    for s in [Spot::Hall, Spot::Bunkhouses, Spot::Workyard, Spot::Stores, Spot::Yard] {
        assert!(isle.in_compound(isle.spot(s)), "{s:?} inside");
    }
    for s in [Spot::Hut, Spot::Dock, Spot::Briar, Spot::Quarry, Spot::Fields, Spot::TestQuarter] {
        assert!(!isle.in_compound(isle.spot(s)), "{s:?} outside");
    }
    // Straight through the east wall is blocked; the way in goes by the gate.
    let outside = c.add(V2::new(half.x + 6.0, 10.0));
    let inside = c.add(V2::new(half.x - 6.0, 10.0));
    assert!(!w.way_clear(outside, inside, 0.4), "the wall stops a walker");
    assert!(w.can_walk(outside, inside), "but there's a way in");
    let (path, _) = w.route(outside, inside);
    let gate = isle.gate();
    assert!(path.iter().any(|p| p.dist(gate.at) < gate.width + 3.0), "through the gate: {path:?}");
}

#[test]
fn the_night_run_sets_the_sneak_path_and_the_lessons_follow() {
    let mut w = opening(14);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    // Through the shared opening by hand.
    for k in ["wake", "salvage", "shelter", "cook", "questions"] {
        w.lesson_done(k);
    }
    assert_eq!(w.lesson_now().map(|x| x.key), Some("night_run"));
    // Inside the palisade by day, walking: taken for a runaway, the caught
    // path (the second's `caught.rs`). (On a copy: this one goes on by night.)
    let mut by_day = w.clone();
    run_to_hour(&mut by_day, 12.0);
    by_day.teleport_squad(isle.spot(Spot::Yard));
    // (The watch sees them, comes over and takes them.)
    for _ in 0..1200 {
        if by_day.tutorial().unwrap().path == Some(Path::Caught) {
            break;
        }
        by_day.step(0.25);
    }
    assert_eq!(by_day.tutorial().unwrap().path, Some(Path::Caught));
    // After dark, sneaking inside: the sneak path.
    run_to_hour(&mut w, 23.0);
    w.teleport_squad(isle.spot(Spot::Yard));
    w.squad.sneaking[0] = true;
    w.step(0.5);
    assert_eq!(w.tutorial().unwrap().path, Some(Path::Sneak), "{:?}", w.tutorial());
    assert_eq!(w.lesson_now().map(|x| x.key), Some("stores"));
    // Something of the compound's gone through (its chests are laid out when someone first goes in).
    let t = isle.settlement;
    let hall = w.settlements[t as usize].buildings[1].pos;
    w.teleport_squad(hall);
    w.step(0.5);
    let id = *w.containers.range((t, 0, 0)..=(t, u16::MAX, u8::MAX)).next().expect("the hall has chests").0;
    w.containers.get_mut(&id).unwrap().taken = 1;
    w.step(0.5);
    assert_eq!(w.lesson_now().map(|x| x.key), Some("gate"));
    // By day at the gate in a labourer's smock. (On before it's light: in
    // the compound plainly by day, they'd be taken: `caught.rs`.)
    let smock = gahturiyu_sim::sim::items::id("labourers_smock");
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.add_piece(smock, gahturiyu_sim::sim::wear::fresh(smock, w.time));
        let _ = d.gear.equip(smock);
    }
    assert!(w.disguise_of(m).is_some(), "passes as a labourer");
    run_to_hour(&mut w, 10.0);
    w.teleport_squad(isle.gate().at);
    w.step(0.5);
    assert_eq!(w.lesson_now().map(|x| x.key), Some("spotted"), "{:?}", w.tutorial());
    // Never spotted: stowing away counts for both. The crew takes a sneaking runaway only.
    let dock = isle.spot(Spot::Dock);
    w.teleport_squad(dock.add(V2::new(3.0, 0.0)));
    w.squad.sneaking[0] = false;
    assert!(w.can_board(m, Berth::Island).is_err(), "not walking up in the open");
    w.squad.sneaking[0] = true;
    assert!(w.order_board(&[m]).is_ok());
    let limit = w.time + 30.0 * HOUR;
    while w.voyage.is_none() && w.time < limit {
        w.step(30.0);
    }
    assert!(w.voyage.is_some(), "sailed");
    let tu = w.tutorial().unwrap();
    assert!(tu.is_done("stow") && tu.is_done("spotted") && tu.is_done("sail"), "{tu:?}");
    // Diʻub's word on the ship, and the Night Market takes them in (`caught.rs`).
    assert!(w.in_guild(gahturiyu_sim::sim::factions::id("night_market")), "counted among the Night Market");
    assert!(w.log.iter().any(|l| l.1.starts_with("Diʻub:") && l.1.contains("Night Market")), "{:?}", w.log);
}

#[test]
fn a_fight_with_the_compounds_people_is_being_spotted() {
    let mut w = opening(14);
    let isle = w.island().unwrap().clone();
    for k in ["wake", "salvage", "shelter", "cook", "questions", "night_run", "stores", "gate"] {
        w.lesson_done(k);
    }
    assert_eq!(w.tutorial().unwrap().path, Some(Path::Sneak));
    assert_eq!(w.lesson_now().map(|x| x.key), Some("spotted"));
    // Seen stealing by one of the compound: the watch comes after them.
    use gahturiyu_sim::sim::{containers::Owner, jobs::Job, law::Wrong};
    let t = isle.settlement;
    run_to_hour(&mut w, 10.0);
    let me = w.squad.members[0];
    let witness = *w.settlements[t as usize].residents.iter().find(|&&p| w.society.lives[p as usize].job != Job::Guard && w.society.lives[p as usize].household.is_some()).unwrap();
    let h = w.society.lives[witness as usize].household.unwrap();
    let at = isle.spot(Spot::Yard);
    w.teleport_squad(at);
    w.wrong_seen(me, t, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(witness), Some(Owner::Household(h)), at);
    assert!(w.pursuits.iter().any(|p| p.town == t), "the compound's watch is told");
    w.step(0.5);
    assert!(w.tutorial().unwrap().is_done("spotted"), "{:?}", w.tutorial());
}
