//! Doors, locks, lockpicking, walking round buildings, and what's inside.

use gahturiyu_sim::sim::{
    buildings::{door_of, is_night, Door},
    geo::V2,
    items,
    stats::Skill,
    world::{HOUR, DAY},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// A home in the squad's starting town (with a lock, if asked), and the
/// world. It's 06:00, so doors are open.
fn town_with_lock(locked: bool) -> (World, Door) {
    let w = worldgen::generate(1);
    let s = &w.settlements[start_town(&w)];
    let d = (0..s.buildings.len() as u16).filter_map(|i| door_of(s, i)).find(|d| !locked || (d.lock > 0.0 && d.lock < 60.0)).expect("a door");
    (w, d)
}

fn start_town(w: &World) -> usize {
    w.settlements.iter().enumerate().min_by(|a, b| a.1.pos.dist(w.squad.pos).total_cmp(&b.1.pos.dist(w.squad.pos))).unwrap().0
}

fn until_hour(w: &mut World, h: f64) {
    while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
        w.step(60.0);
    }
}

#[test]
fn walking_in_goes_through_the_door() {
    let (mut w, d) = town_with_lock(false);
    let m = w.squad.members[0];
    w.order_members(&[m], d.centre);
    let k = w.squad.index(m).unwrap();
    let path = w.squad.route[k].clone();
    assert!(path.iter().any(|p| p.dist(d.outside) < 0.01), "the way in should pass the door");
    walk(&mut w, 240.0);
    assert!(w.squad.at[k].dist(d.centre) < 0.6, "should have got inside: {}", w.squad.at[k].dist(d.centre));
    assert_eq!(w.squad.inside[k], Some(d.id));
}

#[test]
fn walking_past_goes_round_buildings() {
    let (w, d) = town_with_lock(false);
    let a = d.centre.add(V2::new(-(d.radius + 6.0), 0.3));
    let b = d.centre.add(V2::new(d.radius + 6.0, -0.3));
    let (path, _) = w.route(a, b);
    let mut prev = a;
    for p in path {
        for k in 0..=20 {
            let q = prev.lerp(p, k as f32 / 20.0);
            assert!(q.dist(d.centre) > d.radius - 0.2, "the path cuts through a wall");
        }
        prev = p;
    }
}

#[test]
fn doors_lock_at_night() {
    let (mut w, d) = town_with_lock(true);
    until_hour(&mut w, 12.0);
    assert!(!is_night(w.time));
    assert!(!w.is_locked(d.id), "open by day");
    until_hour(&mut w, 22.0);
    assert!(w.is_locked(d.id), "locked at night");
    let (path, blocked) = w.route(w.squad.pos, d.centre);
    assert_eq!(blocked, Some(d.id));
    assert!(path.last().unwrap().dist(d.outside) < 0.01, "you get as far as the door");
}

#[test]
fn a_good_thief_picks_a_lock_and_it_stays_open_till_next_night() {
    let (mut w, d) = town_with_lock(true);
    until_hour(&mut w, 2.0);
    let thief = w.squad.members[1];
    w.people[thief as usize].stats.set_skill(Skill::Security, 90.0);
    w.set_sneaking(thief, true);
    assert!(w.order_pick(thief, d.id));
    let before = w.people[thief as usize].stats.skill(Skill::Security);
    walk(&mut w, 400.0);
    let bounty: f32 = w.bounty.values().sum();
    eprintln!("bounty {bounty}, locked {}", w.is_locked(d.id));
    assert!(bounty > 0.0 || !w.is_locked(d.id), "either picked or caught");
    if bounty == 0.0 {
        assert!(!w.is_locked(d.id), "it should be picked");
        assert!(w.people[thief as usize].stats.skill(Skill::Security) > before);
        // Next night it's locked again.
        until_hour(&mut w, 12.0);
        until_hour(&mut w, 23.0);
        assert!(w.is_locked(d.id));
    }
}

#[test]
fn no_lockpicks_no_picking() {
    let (mut w, d) = town_with_lock(true);
    until_hour(&mut w, 2.0);
    let warrior = w.squad.members[0];
    assert!(!w.people[warrior as usize].detail.as_ref().unwrap().gear.bag.iter().any(|e| e.0 == items::id("lockpick")));
    w.order_pick(warrior, d.id);
    walk(&mut w, 300.0);
    assert!(w.is_locked(d.id));
    assert!(w.picking.is_empty(), "they should give up");
}

#[test]
fn a_home_has_things_inside_and_they_belong_to_someone() {
    let (mut w, d) = town_with_lock(false);
    until_hour(&mut w, 11.0);
    let m = w.squad.members[0];
    w.order_members(&[m], d.centre);
    walk(&mut w, 240.0);
    let inside: Vec<_> = w.ground.iter().filter(|g| g.pos.dist(d.centre) < d.radius).cloned().collect();
    assert!(!inside.is_empty(), "nothing inside");
    assert!(inside.iter().all(|g| g.owner == Some(d.id.0)));
    // Coming back doesn't make more.
    let n = w.ground.len();
    w.order_members(&[m], d.outside.add(V2::new(0.0, 0.0)));
    walk(&mut w, 60.0);
    w.order_members(&[m], d.centre);
    walk(&mut w, 60.0);
    assert_eq!(w.ground.len(), n);
    // Taking things in broad daylight in a busy town gets noticed sooner or later.
    for g in inside {
        w.order_pickup(m, g.id);
        walk(&mut w, 20.0);
    }
    let mut more = 0;
    while w.bounty.is_empty() && more < 6 {
        let id = w.put_on_ground(items::id("knife"), 1, d.centre);
        if let Some(g) = w.ground.iter_mut().find(|g| g.id == id) {
            g.owner = Some(d.id.0);
        }
        w.order_pickup(m, id);
        walk(&mut w, 10.0);
        more += 1;
    }
    assert!(!w.bounty.is_empty(), "nobody saw a thing?");
}
