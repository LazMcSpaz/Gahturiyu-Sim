//! Crime that means something: walls hide a thief, and a bound squad member
//! is led off to work and can't be ordered until the bond ends.

use gahturiyu_sim::sim::{geo::V2, world::DAY, worldgen};

#[test]
fn walls_hide_a_thief() {
    let w = worldgen::generate(1);
    let me = w.squad.members[0];
    let town = 2;
    // Every building in town where nobody is inside right now: a theft there
    // can't be seen, whoever stands outside.
    let mut empty = 0;
    for b in 0..w.settlements[town as usize].buildings.len() as u16 {
        let Some(d) = w.door((town, b)) else { continue };
        let anyone = w.residents_in_band1(town).into_iter().any(|p| w.building_at(w.person_pos(p)).map(|x| x.id) == Some(d.id));
        if !anyone {
            empty += 1;
            assert_eq!(w.catch_chance(me, d.centre, town), 0.0, "nobody inside building {b} to see");
        }
    }
    assert!(empty > 0);
    // Out in the street with townsfolk about, you can be seen.
    let busy = w.residents_in_band1(town).into_iter().map(|p| w.person_pos(p)).find(|&p| w.building_at(p).is_none()).unwrap();
    assert!(w.catch_chance(me, busy.add(V2::new(1.0, 0.0)), town) > 0.0);
}

#[test]
fn the_bound_are_led_off_and_cant_be_ordered() {
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    let town = 2;
    let t = w.time;
    w.bond(me, town, None, t, t + 2.0 * DAY);
    assert!(!w.free_to_order(me));
    assert!(!w.alerts.is_empty(), "you're told");
    let far = w.squad.pos.add(V2::new(400.0, 0.0));
    w.order_members(&[me], far);
    for _ in 0..2400 {
        w.step(0.5);
    }
    let k = w.squad.index(me).unwrap();
    let spot = w.bound_spot(town);
    assert!(w.squad.at[k].dist(spot) < 12.0, "at their work: {:.0} m off", w.squad.at[k].dist(spot));
    // When the bond is over, they're theirs to order again.
    for _ in 0..60 {
        w.step(3600.0);
    }
    assert!(w.free_to_order(me));
    w.order_members(&[me], far);
    for _ in 0..1200 {
        w.step(0.5);
    }
    let k = w.squad.index(me).unwrap();
    assert!(w.squad.at[k].dist(far) < w.squad.at[k].dist(spot), "free to go");
}
