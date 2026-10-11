//! The buildings agent's share of the feel items (re-split, 11 Oct): what
//! the window shows rests on these.

use gahturiyu_sim::sim::{
    buildings::door_of,
    geo::V2,
    world::{DAY, HOUR},
    worldgen, World,
};

fn until_hour(w: &mut World, h: f64) {
    while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
        w.step(60.0);
    }
}

/// #125 UI-10: a member inside a building sent to a spot on its floor walks
/// to that spot, not to the room's middle. (The window now sends a click on
/// the floor of a building the squad is in as that spot.)
#[test]
fn a_spot_on_the_floor_indoors_is_walked_to() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let here = w.squad.pos;
    let m = w.squad.members[0];
    let k = w.squad.index(m).unwrap();
    // The nearest open building with a container in it (a spot on the
    // floor known to be reachable: where the container is gone through).
    let mut doors: Vec<_> = w.settlements.iter().flat_map(|s| (0..s.buildings.len() as u16).filter_map(move |i| door_of(s, i))).filter(|d| d.centre.dist(here) < 2000.0).collect();
    doors.sort_by(|a, b| a.centre.dist(here).total_cmp(&b.centre.dist(here)));
    let mut tried = 0;
    for d in doors {
        if w.is_locked(d.id) {
            continue;
        }
        w.teleport_squad(d.outside.add(d.outside.sub(d.centre).scale(6.0 / d.outside.dist(d.centre).max(0.1))));
        w.step(0.001);
        w.order_members(&[m], d.centre);
        for _ in 0..1200 {
            w.step(0.5);
            if w.squad.inside[k] == Some(d.id) && w.squad.route[k].is_empty() {
                break;
            }
        }
        if w.squad.inside[k] != Some(d.id) {
            eprintln!("{}: not in (at {:?}, {:.1} m from the middle)", d.variant().key, w.squad.at[k], w.squad.at[k].dist(d.centre));
            continue;
        }
        let ids: Vec<_> = w.containers_in(d.id).map(|c| c.id).collect();
        for id in ids {
            let Some(spot) = w.container_stand(id) else { continue };
            if spot.dist(d.centre) < 1.0 {
                continue;
            }
            w.order_members(&[m], spot);
            assert!(w.squad.goal[k].dist(spot) < 0.05, "the goal is the spot: {:?} vs {:?}", w.squad.goal[k], spot);
            for _ in 0..600 {
                w.step(0.5);
                if w.squad.route[k].is_empty() && w.squad.at[k].dist(w.squad.goal[k]) < 1e-3 {
                    break;
                }
            }
            let at = w.squad.at[k];
            assert!(at.dist(spot) < 0.1, "{}: stopped {:.2} m from the spot ({:.2} m from the middle)", d.variant().key, at.dist(spot), at.dist(d.centre));
            assert_eq!(w.squad.inside[k], Some(d.id), "still inside");
            tried += 1;
            break;
        }
        if tried >= 3 {
            break;
        }
    }
    assert!(tried >= 1, "no building to try");
}

/// The nearest open water to the squad's start, the last dry footing on
/// the way to it, and a direction back inland.
fn a_shore(w: &World) -> (V2, V2, V2) {
    let here = w.squad.pos;
    for r in (20..6000).step_by(20) {
        for a in 0..64 {
            let ang = a as f32 / 64.0 * std::f32::consts::TAU;
            let p = here.add(V2::new(ang.cos(), ang.sin()).scale(r as f32));
            if w.open_water(p) {
                let back = here.sub(p);
                let back = back.scale(1.0 / back.len().max(0.01));
                let mut shore = p;
                while w.open_water(shore) {
                    shore = shore.add(back.scale(0.5));
                }
                return (p, shore, back);
            }
        }
    }
    panic!("no sea near the start");
}

/// #150 UI-7: a walk out into the sea is planned to the water's edge from
/// the start, so the order line ends where they'll stop; walking it ends
/// there, dry.
#[test]
fn a_walk_into_the_sea_ends_at_the_shore() {
    let mut w = worldgen::generate(1);
    let (_, shore, back) = a_shore(&w);
    w.teleport_squad(shore.add(back.scale(20.0)));
    w.step(0.001);
    let to = shore.sub(back.scale(30.0));
    let m = w.squad.members[0];
    let k = w.squad.index(m).unwrap();
    let from = w.member_pos(k);
    let stop = w.walk_stops_short(from, to).expect("the sea is in the way");
    assert!(!w.open_water(stop));
    w.order_members(&[m], to);
    let goal = w.squad.goal[k];
    eprintln!("from {from:?} to {to:?}: goal {goal:?}, {:.1} m from the shore; way {:?}", goal.dist(shore), w.squad.route[k]);
    assert!(!w.open_water(goal), "the walk is planned to dry footing");
    assert!(goal.dist(stop) < 0.5, "the order and the forecast agree");
    let mut way = vec![from];
    way.extend(w.squad.route[k].iter().copied());
    for s in way.windows(2) {
        for j in 0..=20 {
            assert!(!w.open_water(s[0].lerp(s[1], j as f32 / 20.0)), "the planned way stays dry");
        }
    }
    for _ in 0..400 {
        w.step(0.5);
    }
    let at = w.squad.at[k];
    assert!(at.dist(goal) < 0.05, "they walked to the end of the line: {:.2} m off", at.dist(goal));
    // A walk that stays dry isn't cut.
    assert_eq!(w.walk_stops_short(at, at.add(back.scale(15.0))), None);
}
