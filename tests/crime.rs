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

#[test]
fn things_can_be_put_in_a_container() {
    use gahturiyu_sim::sim::loot::Source;
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    let d = w.door((2, 3)).unwrap();
    w.order_members(&[me], d.centre);
    for _ in 0..600 {
        w.step(0.25);
    }
    let id = w.containers_in(d.id).find(|c| c.lock == 0.0).map(|c| c.id).expect("an open container");
    assert!(w.order_search(me, id));
    for _ in 0..600 {
        if w.source_now(me).is_some() {
            break;
        }
        w.step(0.25);
    }
    assert_eq!(w.source_now(me), Some(Source::Chest(id)));
    let before = w.contents(Source::Chest(id)).iter().map(|x| x.2 as u32).sum::<u32>();
    let mine = w.people[me as usize].detail.as_ref().unwrap().gear.bag.len();
    assert!(mine > 0);
    assert!(w.put_in(me, 0));
    let after = w.contents(Source::Chest(id)).iter().map(|x| x.2 as u32).sum::<u32>();
    assert!(after > before, "it's in the container");
    assert_eq!(w.people[me as usize].detail.as_ref().unwrap().gear.bag.len(), mine - 1);
    assert!(w.bounty.is_empty(), "putting things in isn't a crime");
}

#[test]
fn one_left_behind_doesnt_drag_the_squads_centre_away() {
    let mut w = worldgen::generate(12);
    let ms = w.squad.members.clone();
    let town = 1u16;
    let far = w.settlements[town as usize].pos.add(V2::new(1500.0, 0.0));
    w.teleport_squad(far);
    let t = w.time;
    // Bound in town for a while; the rest are out in the wilds.
    w.bond(ms[1], town, None, t, t + 2.0 * DAY);
    let k1 = w.squad.index(ms[1]).unwrap();
    w.squad.at[k1] = w.bound_spot(town);
    w.squad.goal[k1] = w.squad.at[k1];
    w.step(0.5);
    let k0 = w.squad.index(ms[0]).unwrap();
    assert!(w.squad.pos.dist(w.squad.at[k0]) < 30.0, "the centre stays with the bunch: {:.0} m off", w.squad.pos.dist(w.squad.at[k0]));
}
