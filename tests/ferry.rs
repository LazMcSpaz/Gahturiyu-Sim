//! The ferry (`ferry.rs`): a timetable read off the clock, a crossing that
//! carries the squad off the island (and back), and the tutorial ending as
//! the ship puts in.

use gahturiyu_sim::sim::ferry::{Berth, ISLE_CALL, PORT_CALL, SHIP_SPEED};
use gahturiyu_sim::sim::geo;
use gahturiyu_sim::sim::island::Spot;
use gahturiyu_sim::sim::world::HOUR;
use gahturiyu_sim::sim::worldgen;

fn opening(seed: u64) -> gahturiyu_sim::sim::World {
    worldgen::generate_opening(seed, None, Default::default())
}

#[test]
fn the_ship_keeps_a_timetable() {
    let w = opening(6);
    let day = (w.time / HOUR / 24.0).floor() * 24.0 * HOUR;
    let at = |h: f64| day + h * HOUR;
    assert!(w.ship_in(Berth::Island, at(ISLE_CALL.0 + 0.5)) && !w.ship_in(Berth::Island, at(ISLE_CALL.1 + 0.5)));
    assert!(w.ship_in(Berth::Port, at(PORT_CALL.0 + 0.5)) && !w.ship_in(Berth::Port, at(ISLE_CALL.0 + 0.5)));
    assert!(!w.ship_in(Berth::Island, at(PORT_CALL.0 + 0.5)));
    // Drawn at a berth while in, nowhere while away.
    let (p, _) = w.ship_pos(at(ISLE_CALL.0 + 1.0)).expect("at the island");
    assert!(w.terrain.is_sea(p) && p.dist(w.island().unwrap().spot(Spot::Dock)) < 60.0, "the ship lies off the dock: {p:?}");
    assert!(w.ship_pos(at(12.5)).is_none());
    // The port is a mainland town's dock.
    let (town, dock) = w.ferry_port().expect("a port");
    assert!(!w.settlements[town as usize].island && geo::inland(dock) >= -5.0);
    let next = w.ship_next(Berth::Island, at(12.0));
    assert!((next - at(24.0 + ISLE_CALL.0)).abs() < 1.0);
}

#[test]
fn boarding_carries_the_squad_to_the_port_and_ends_the_island() {
    let mut w = opening(6);
    let isle = w.island().unwrap().clone();
    let dock = isle.spot(Spot::Dock);
    let m = w.squad.members[0];
    // Wait by the dock for the ship.
    w.teleport_squad(dock.add(geo::V2::new(3.0, 0.0)));
    assert!(w.order_board(&[m]).is_ok());
    let limit = w.time + 30.0 * HOUR;
    while w.voyage.is_none() && w.time < limit {
        w.step(30.0);
    }
    let v = w.voyage.clone().unwrap_or_else(|| panic!("sailed once the ship was in: at {:?}, dock {dock:?}, {:.1} m off, boarding {:?}, in {}", w.person_pos(m), w.person_pos(m).dist(dock), w.boarding, w.ship_in(Berth::Island, w.time)));
    assert!(w.ship_in(Berth::Island, v.depart - 0.01) || (v.depart / HOUR).rem_euclid(24.0) >= ISLE_CALL.0);
    assert_eq!(v.to_berth, Berth::Port);
    assert!((v.arrive - v.depart - (v.length() / SHIP_SPEED) as f64).abs() < 1.0);
    assert!(v.length() > v.from().dist(v.to()), "goes round the island, not through it");
    assert!(w.aboard(m));
    assert!(w.tutorial().unwrap().is_done("sail"));
    // Under way they're at sea, with the ship, the whole way.
    while w.voyage.is_some() && w.time < v.arrive - 30.0 {
        w.step(20.0);
        let at = w.person_pos(m);
        assert!(w.terrain.is_sea(at), "aboard at {at:?} at {:.0} s", w.time - v.depart);
        assert!(at.dist(w.ship_pos(w.time).unwrap().0) < 3.0);
    }
    // Put in: on the port's dock, the island behind them.
    while w.voyage.is_some() {
        w.step(30.0);
    }
    let (_, port) = w.ferry_port().unwrap();
    assert!(w.person_pos(m).dist(port) < 10.0, "landed at {:?}, the port at {port:?}", w.person_pos(m));
    assert!(!w.terrain.is_sea(w.person_pos(m)));
    assert!(w.tutorial().unwrap().over && !w.on_island() && w.lesson_now().is_none());
    // And back again, from the port.
    assert!(w.order_board(&[m]).is_ok());
    let limit = w.time + 30.0 * HOUR;
    while w.voyage.is_none() && w.time < limit {
        w.step(30.0);
    }
    assert_eq!(w.voyage.as_ref().map(|v| v.to_berth), Some(Berth::Island));
    while w.voyage.is_some() {
        w.step(30.0);
    }
    assert!(isle.contains(w.person_pos(m)));
}

#[test]
fn nobody_sails_from_the_mainland_squad_start() {
    let mut w = worldgen::generate(6);
    // The old start: the ship still runs, and the squad may take it from the port.
    let (_, port) = w.ferry_port().unwrap();
    w.teleport_squad(port.add(geo::V2::new(2.0, 0.0)));
    assert!(w.order_board(&w.squad.members.clone()).is_ok());
    assert!(w.tutorial().is_none());
}
