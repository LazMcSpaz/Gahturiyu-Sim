//! Building variants and containers (playable-MVP step 6): every variant can
//! be walked into, containers are laid out the same way every time, taking
//! from them is theft, locked ones are picked, and all of it is saved.

use gahturiyu_sim::sim::{
    buildings::{door_of, Door},
    containers::{stock, ContainerId},
    geo::V2,
    inventory::Entry,
    items,
    layout::{self, Shape, VARIANTS},
    loot::{LootRef, Source},
    person::PersonId,
    settlement::{Building, BuildingKind},
    stats::Skill,
    world::{DAY, HOUR},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

fn start_town(w: &World) -> usize {
    w.settlements.iter().enumerate().min_by(|a, b| a.1.pos.dist(w.squad.pos).total_cmp(&b.1.pos.dist(w.squad.pos))).unwrap().0
}

fn until_hour(w: &mut World, h: f64) {
    while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
        w.step(60.0);
    }
}

/// Walk squad member `m` into a building; true once they're in.
fn walk_in(w: &mut World, m: PersonId, d: &Door) -> bool {
    let k = w.squad.index(m).unwrap();
    w.order_members(&[m], d.centre);
    for _ in 0..1200 {
        w.step(0.5);
        if w.squad.inside[k] == Some(d.id) && w.squad.at[k].dist(d.centre) < 0.6 {
            return true;
        }
    }
    w.squad.inside[k] == Some(d.id)
}

/// Step until `m` stands at container `id` and is going through it.
fn reach(w: &mut World, m: PersonId, id: ContainerId) -> bool {
    for _ in 0..1200 {
        if w.searching_now(m) == Some(id) {
            return true;
        }
        w.step(0.5);
    }
    w.searching_now(m) == Some(id)
}

/// The doors of the start town, nearest its middle first (the busiest).
fn town_doors(w: &World) -> Vec<Door> {
    let t = start_town(w);
    let s = &w.settlements[t];
    let mut v: Vec<Door> = (0..s.buildings.len() as u16).filter_map(|i| door_of(s, i)).collect();
    v.sort_by(|a, b| a.centre.dist(s.pos).total_cmp(&b.centre.dist(s.pos)));
    v
}

/// The first container in the start town (by `town_doors` order) whose
/// predicted stock and lock pass `want`, and its building's door.
fn find_container(w: &World, want: impl Fn(&[Entry], f32) -> bool) -> (Door, ContainerId) {
    for d in town_doors(w) {
        let v = d.variant();
        for (slot, spot) in v.holders.iter().enumerate() {
            let id = (d.id.0, d.id.1, slot as u8);
            let (items, lock) = stock(w.seed, id, spot.what, v.use_, v.key);
            if want(&items, lock) {
                return (d, id);
            }
        }
    }
    panic!("no such container in the start town");
}

#[test]
fn containers_are_stocked_the_same_way_every_time() {
    let (mut a, mut b) = (worldgen::generate(1), worldgen::generate(1));
    let d = town_doors(&a).into_iter().find(|d| !d.variant().holders.is_empty()).expect("a door");
    let m = a.squad.members[0];
    assert!(walk_in(&mut a, m, &d), "got in (a)");
    assert!(walk_in(&mut b, m, &d), "got in (b)");
    let ca: Vec<_> = a.containers_in(d.id).cloned().collect();
    let cb: Vec<_> = b.containers_in(d.id).cloned().collect();
    assert_eq!(ca.len(), d.variant().holders.len(), "one container per spot");
    assert_eq!(ca, cb, "same world, same containers");
    // What's in them is exactly what `stock` says.
    for c in &ca {
        let v = d.variant();
        let (items, lock) = stock(a.seed, c.id, c.what, v.use_, v.key);
        assert_eq!(c.items, items);
        assert_eq!(c.lock, lock);
    }
    // `stock` is a pure function of its keys...
    let v = d.variant();
    let id = (d.id.0, d.id.1, 0u8);
    assert_eq!(stock(a.seed, id, v.holders[0].what, v.use_, v.key), stock(a.seed, id, v.holders[0].what, v.use_, v.key));
    // ...and different slots and buildings don't all hold the same.
    let mut seen: Vec<(Vec<Entry>, f32)> = Vec::new();
    for d in town_doors(&a).into_iter().take(12) {
        let v = d.variant();
        for (slot, spot) in v.holders.iter().enumerate() {
            let got = stock(a.seed, (d.id.0, d.id.1, slot as u8), spot.what, v.use_, v.key);
            if !seen.contains(&got) {
                seen.push(got);
            }
        }
    }
    assert!(seen.len() > 3, "containers should differ: {} distinct", seen.len());
    // A different world fills the same chest differently (somewhere).
    let other = (0..20u8).any(|s| {
        let id = (d.id.0, d.id.1, s);
        stock(a.seed, id, layout::Holder::Chest, v.use_, v.key) != stock(a.seed ^ 0x9E37, id, layout::Holder::Chest, v.use_, v.key)
    });
    assert!(other, "the world's seed keys the roll");
}

#[test]
fn every_variant_stocks_without_unknown_items() {
    // Many rolls of every kind of container in every variant: each names only
    // things that exist (`items::id` panics on a bad key).
    for v in VARIANTS {
        for (slot, spot) in v.holders.iter().enumerate() {
            for b in 0..300u16 {
                let (items, lock) = stock(7, (3, b, slot as u8), spot.what, v.use_, v.key);
                assert!(items.iter().all(|e| e.1 > 0));
                assert!((0.0..=100.0).contains(&lock));
            }
        }
    }
}

#[test]
fn taking_from_a_container_is_theft_when_seen() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 12.0);
    let (d, id) = find_container(&w, |items, lock| lock == 0.0 && !items.is_empty());
    let m = w.squad.members[0];
    assert!(walk_in(&mut w, m, &d), "got in");
    assert!(w.container(id).is_some(), "stocked on entering");
    assert!(!w.container_locked(id));
    assert!(w.order_search(m, id));
    assert!(reach(&mut w, m, id), "got to the container");
    // Opening and looking is free.
    assert!(w.bounty.is_empty(), "looking isn't a crime");
    let had = w.contents(Source::Chest(id)).len();
    assert!(had > 0);
    let mut takes = 0;
    while w.bounty.is_empty() && takes < 80 {
        if w.container(id).unwrap().items.is_empty() {
            w.containers.get_mut(&id).unwrap().items.push(Entry(items::id("knife"), 1, None));
        }
        if w.searching_now(m) != Some(id) {
            // Arrested or moved off: go back to it.
            if !w.order_search(m, id) || !reach(&mut w, m, id) {
                break;
            }
        }
        let before = w.container(id).unwrap().items.len();
        assert!(w.take_from(m, Source::Chest(id), LootRef::Pack(0)), "took something");
        assert_eq!(w.container(id).unwrap().items.len(), before - 1);
        takes += 1;
        walk(&mut w, 2.0);
    }
    let seen = w.alerts.iter().any(|a| a.contains("seen stealing"));
    let chance = w.catch_chance(m, w.container(id).unwrap().pos, id.0);
    eprintln!("takes {takes}, bounty {:?}, seen {seen}, chance {chance}", w.bounty);
    if chance > 0.0 {
        // Someone shares the room: sooner or later they see it.
        assert!(seen, "nobody saw a thing with someone watching?");
        assert!(!w.bounty.is_empty() || w.log.iter().any(|l| l.1.contains("seen stealing")), "a seen theft is a crime");
    } else {
        // Nobody inside: walls hide you, however busy the street.
        assert!(!seen, "seen through walls");
    }
    assert_eq!(w.container(id).unwrap().taken as usize, takes, "every take is counted");
}

#[test]
fn locked_containers_must_be_picked() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 10.0);
    let (d, id) = find_container(&w, |_, lock| lock > 0.0 && lock < 60.0);
    let thief = w.squad.members[1];
    assert!(walk_in(&mut w, thief, &d), "got in");
    assert!(w.container_locked(id), "locked");
    assert!(!w.order_search(thief, id), "can't just open it");
    w.people[thief as usize].stats.set_skill(Skill::Security, 90.0);
    // Plenty of picks, so running out isn't a third outcome.
    w.people[thief as usize].detail.as_mut().unwrap().gear.add(items::id("lockpick"), 10);
    w.set_sneaking(thief, true);
    assert!(w.order_pick_container(thief, id));
    walk(&mut w, 400.0);
    let picked = w.container(id).unwrap().picked;
    let bounty: f32 = w.bounty.values().sum();
    let caught = bounty > 0.0 || w.alerts.iter().any(|a| a.contains("picking a lock"));
    eprintln!("picked {picked}, bounty {bounty}, caught {caught}");
    assert!(picked || caught, "either picked or caught");
    if picked {
        assert!(!w.container_locked(id), "a picked container stays open");
        assert!(w.order_search(thief, id), "and can now be opened");
        // For good: the next night too.
        until_hour(&mut w, 23.0);
        assert!(!w.container_locked(id));
    }
    // Something not locked can't be picked.
    let open = w.containers_in(d.id).find(|c| c.lock == 0.0).map(|c| c.id);
    if let Some(o) = open {
        assert!(!w.order_pick_container(thief, o));
    }
}

#[test]
fn containers_survive_save_and_load() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let (d, id) = find_container(&w, |items, lock| lock == 0.0 && items.len() >= 2);
    let m = w.squad.members[0];
    assert!(walk_in(&mut w, m, &d));
    assert!(w.order_search(m, id));
    assert!(reach(&mut w, m, id));
    assert!(w.take_from(m, Source::Chest(id), LootRef::Pack(0)));
    assert!(!w.looting.is_empty());
    let mut back = World::load_bytes(&w.save_bytes()).expect("loads");
    assert_eq!(back.containers, w.containers);
    assert_eq!(back.looting, w.looting);
    assert_eq!(back.searching_now(m), Some(id));
    // Both carry on the same: another take, then a while longer.
    assert!(w.take_from(m, Source::Chest(id), LootRef::Pack(0)));
    assert!(back.take_from(m, Source::Chest(id), LootRef::Pack(0)));
    for _ in 0..240 {
        w.step(0.5);
        back.step(0.5);
    }
    assert_eq!(back.containers, w.containers);
    assert_eq!(back.looting, w.looting);
    assert_eq!(format!("{:?}", back.squad), format!("{:?}", w.squad));
    let sorted = |w: &World| {
        let mut v: Vec<_> = w.bounty.iter().map(|(k, v)| (*k, *v)).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    };
    assert_eq!(sorted(&back), sorted(&w));
}

#[test]
fn each_people_has_several_distinct_variants() {
    for (kind, at_least) in [(BuildingKind::RoduroHome, 5), (BuildingKind::QotiroBlock, 5), (BuildingKind::HoraroStilt, 3)] {
        let vs: Vec<_> = layout::variants_of(kind).collect();
        assert!(vs.len() >= at_least, "{kind:?} has {} variants", vs.len());
        for (i, a) in vs.iter().enumerate() {
            for b in &vs[i + 1..] {
                assert_ne!(a.key, b.key);
                let walls = |v: &layout::Variant| format!("{:?}", v.walls);
                let same = a.half == b.half && a.shape == b.shape && a.door == b.door && walls(a) == walls(b) && a.holders.len() == b.holders.len() && a.storeys == b.storeys;
                assert!(!same, "{} and {} look the same", a.key, b.key);
            }
        }
    }
    // Every variant is chosen by some building (its example finds it).
    for v in VARIANTS {
        let (size, seed) = layout::example_of(v);
        assert_eq!(layout::variant_for(v.kind, size, seed).map(|x| x.key), Some(v.key), "example of {}", v.key);
    }
}

/// Do segments `a`–`b` and `p`–`q` cross?
fn crosses(a: V2, b: V2, p: V2, q: V2) -> bool {
    let cross = |o: V2, x: V2, y: V2| (x.x - o.x) * (y.y - o.y) - (x.y - o.y) * (y.x - o.x);
    let (d1, d2) = (cross(p, q, a), cross(p, q, b));
    let (d3, d4) = (cross(a, b, p), cross(a, b, q));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

/// A way that never walks through an inner wall and ends at `to`.
fn check_path(d: &Door, from: V2, path: &[V2], to: V2, what: &str) {
    assert!(path.last().is_some_and(|p| p.dist(to) < 0.01), "{what}: the way should end there");
    let pieces = d.wall_pieces();
    let mut prev = from;
    for &p in path {
        for &(a, b) in &pieces {
            assert!(!crosses(prev, p, a, b), "{what}: the way goes through an inner wall ({prev:?} -> {p:?})");
        }
        prev = p;
    }
}

/// Open, fairly flat land near the start town with no buildings close by.
fn open_ground(w: &World, room: f32) -> V2 {
    let t = start_town(w);
    let s = &w.settlements[t];
    for ring in 0..8 {
        for k in 0..16 {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let c = s.pos.add(V2::new(a.cos(), a.sin()).scale(s.reach + 60.0 + ring as f32 * 50.0));
            let h0 = w.terrain.height(c);
            let ok = (0..24).all(|j| {
                let b = j as f32 * std::f32::consts::TAU / 24.0;
                (1..=3).all(|r| {
                    let p = c.add(V2::new(b.cos(), b.sin()).scale(room * r as f32 / 3.0));
                    !w.terrain.is_sea(p) && (w.terrain.height(p) - h0).abs() < 4.0
                })
            });
            let clear = w.settlements.iter().all(|o| o.buildings.iter().all(|b| b.pos.dist(c) > room + b.size));
            if ok && clear {
                return c;
            }
        }
    }
    panic!("no open ground");
}

#[test]
fn every_variant_can_be_walked_into() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let t = start_town(&w);
    let spot = open_ground(&w, 32.0);
    let m = w.squad.members[0];
    let k = w.squad.index(m).unwrap();
    for v in VARIANTS.iter().filter(|v| v.kind != BuildingKind::HoraroStilt) {
        let (size, seed) = layout::example_of(v);
        let rot = 0.7;
        let s = &mut w.settlements[t];
        s.buildings.push(Building { pos: spot, kind: v.kind, size, rot, seed });
        s.reach = s.reach.max(s.pos.dist(spot) + size);
        let i = (s.buildings.len() - 1) as u16;
        let d = door_of(&w.settlements[t], i).expect("a door");
        assert_eq!(d.variant().key, v.key, "door_of gives the variant");
        assert_eq!(d.round, v.shape == Shape::Round);
        assert!(d.contains(d.inside), "{}: inside the door is inside", v.key);
        assert!(!d.contains(d.outside), "{}: outside the door is outside", v.key);
        assert!(d.contains(d.centre));
        assert_eq!(w.building_at(d.centre).map(|x| x.id), Some(d.id), "{}: found by place", v.key);
        // Every container stands indoors.
        let spots: Vec<V2> = v.holders.iter().map(|h| d.to_world(h.at)).collect();
        for (n, p) in spots.iter().enumerate() {
            assert!(d.contains(*p), "{}: container {n} is outside the walls", v.key);
        }
        // The ways in.
        let (path, locked) = w.route(d.outside, d.centre);
        assert_eq!(locked, None, "{}: open by day", v.key);
        check_path(&d, d.outside, &path, d.centre, &format!("{} to the middle", v.key));
        // Walk in for real: through the door, and the containers are laid out.
        let dir = V2::new(rot.cos(), rot.sin());
        w.teleport_squad(d.outside.add(dir.scale(10.0)));
        assert!(walk_in(&mut w, m, &d), "{}: walked in (at {:?}, {} m from the middle)", v.key, w.squad.at[k], w.squad.at[k].dist(d.centre));
        assert_eq!(w.squad.inside[k], Some(d.id));
        assert_eq!(w.containers_in(d.id).count(), v.holders.len(), "{}: one container per spot", v.key);
        for c in w.containers_in(d.id).cloned().collect::<Vec<_>>() {
            let stand = w.container_stand(c.id).unwrap();
            assert!(d.contains(stand), "{}: container {} is reached from inside", v.key, c.id.2);
            let (path, _) = w.route(d.outside, stand);
            check_path(&d, d.outside, &path, stand, &format!("{} to container {}", v.key, c.id.2));
            let (path, _) = w.route(d.centre, stand);
            check_path(&d, d.centre, &path, stand, &format!("{} middle to container {}", v.key, c.id.2));
            // And actually get there, if it's open.
            if !w.container_locked(c.id) {
                assert!(w.order_search(m, c.id));
                assert!(reach(&mut w, m, c.id), "{}: got to container {} (at {:?}, stand {:?})", v.key, c.id.2, w.squad.at[k], stand);
                w.stop_looting(m);
            }
        }
        // Out of the way for the next one.
        w.teleport_squad(spot.add(V2::new(40.0, 0.0)));
        w.settlements[t].buildings[i as usize].pos = V2::new(-1.0e6, -1.0e6);
    }
}
