//! Fixes from the known-issues list, batch 7 (the buildings agent): the
//! docks and rope bridges out to the stilt villages (RG-14).

use gahturiyu_sim::sim::{geo::V2, settlement::BuildingKind, worldgen};

/// RG-14: a squad member walks the dock and a bridge out to a stilt home,
/// and back; elsewhere the sea still stops them.
#[test]
fn rg_14_the_stilt_homes_can_be_walked_to() {
    let mut w = worldgen::generate(1);
    let town = w.settlements.iter().find(|s| s.stilts.is_some()).expect("a stilt village").id;
    let s = w.settlements[town as usize].clone();
    let home = s.buildings.iter().find(|b| b.kind == BuildingKind::HoraroStilt).unwrap().pos;
    // The walkways are dry footing.
    for (a, b) in w.walkways(town) {
        for k in 0..=10 {
            let p = a.lerp(b, k as f32 / 10.0);
            assert!(!w.open_water(p), "the walkway is open water at {p:?}");
        }
    }
    w.teleport_squad(s.pos);
    let m = w.squad.members[0];
    w.order_members(&[m], home);
    for _ in 0..(40.0 * 60.0 / 0.5) as usize {
        w.step(0.5);
    }
    let k = w.squad.index(m).unwrap();
    assert!(w.squad.at[k].dist(home) < 2.0, "{:.0} m short of the stilt home", w.squad.at[k].dist(home));
    // And back ashore.
    w.order_members(&[m], s.pos);
    for _ in 0..(40.0 * 60.0 / 0.5) as usize {
        w.step(0.5);
    }
    let k = w.squad.index(m).unwrap();
    assert!(w.squad.at[k].dist(s.pos) < 5.0, "stuck out on the water");
    // Off the walkways, the sea still stops them.
    let out = home.add(V2::new(-400.0, 0.0));
    assert!(w.open_water(out));
}
