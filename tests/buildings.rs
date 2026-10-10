//! Building variants and containers (playable-MVP step 6): every variant can
//! be walked into, containers are laid out the same way every time, taking
//! from them is theft, locked ones are picked, and all of it is saved.

use gahturiyu_sim::sim::{
    buildings::{door_of, Door},
    containers::{stock, ContainerId, Owner},
    geo::V2,
    inventory::Entry,
    items,
    jobs::{Job, ALL_JOBS},
    layout::{self, Shape, VARIANTS},
    loot::{LootRef, Source},
    person::PersonId,
    race::Race,
    settlement::{Building, BuildingKind},
    stats::Skill,
    culture::Belonging,
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

/// The job of a building's keeper (what its crates follow).
fn keeper_job(w: &World, door: (u16, u16)) -> Option<Job> {
    w.keeper(door).map(|p| w.life(p).job)
}

/// The first container in the start town (by `town_doors` order) whose
/// predicted stock and lock pass `want`, and its building's door.
fn find_container(w: &World, want: impl Fn(&[Entry], f32) -> bool) -> (Door, ContainerId) {
    for d in town_doors(w) {
        let v = d.variant();
        for (slot, spot) in v.holders.iter().enumerate() {
            let id = (d.id.0, d.id.1, slot as u8);
            let (items, lock) = stock(w.seed, id, spot.what, v.use_, v.key, keeper_job(w, d.id));
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
        let (items, lock) = stock(a.seed, c.id, c.what, v.use_, v.key, keeper_job(&a, d.id));
        assert_eq!(c.items, items);
        assert_eq!(c.lock, lock);
    }
    // `stock` is a pure function of its keys...
    let v = d.variant();
    let id = (d.id.0, d.id.1, 0u8);
    assert_eq!(stock(a.seed, id, v.holders[0].what, v.use_, v.key, None), stock(a.seed, id, v.holders[0].what, v.use_, v.key, None));
    // ...and different slots and buildings don't all hold the same.
    let mut seen: Vec<(Vec<Entry>, f32)> = Vec::new();
    for d in town_doors(&a).into_iter().take(12) {
        let v = d.variant();
        for (slot, spot) in v.holders.iter().enumerate() {
            let got = stock(a.seed, (d.id.0, d.id.1, slot as u8), spot.what, v.use_, v.key, keeper_job(&a, d.id));
            if !seen.contains(&got) {
                seen.push(got);
            }
        }
    }
    assert!(seen.len() > 3, "containers should differ: {} distinct", seen.len());
    // A different world fills the same chest differently (somewhere).
    let other = (0..20u8).any(|s| {
        let id = (d.id.0, d.id.1, s);
        stock(a.seed, id, layout::Holder::Chest, v.use_, v.key, None) != stock(a.seed ^ 0x9E37, id, layout::Holder::Chest, v.use_, v.key, None)
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
                let (items, lock) = stock(7, (3, b, slot as u8), spot.what, v.use_, v.key, ALL_JOBS.get(b as usize % 40).copied());
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
    eprintln!("takes {takes}, bounty {:?}, seen {seen}", w.bounty);
    assert!(seen, "nobody saw a thing in a busy town at noon?");
    assert!(!w.bounty.is_empty() || w.log.iter().any(|l| l.1.contains("seen stealing")), "a seen theft is a crime");
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

// ---- Buildings match their people --------------------------------------------

/// Every town building (with residents' jobs as they are now, head of house
/// first) and its stored variant.
fn town_buildings(w: &World) -> Vec<((u16, u16), Vec<Job>, &'static layout::Variant)> {
    let mut out = Vec::new();
    for s in &w.settlements {
        for i in 0..s.buildings.len() as u16 {
            let Some(v) = layout::variant_in(s, i) else { continue };
            let jobs = w.household_order((s.id, i)).into_iter().map(|p| w.life(p).job).collect();
            out.push(((s.id, i), jobs, v));
        }
    }
    out
}

fn fits(v: &layout::Variant, size: f32) -> bool {
    size >= v.sizes.0 && size < v.sizes.1
}

#[test]
fn buildings_follow_the_trades_of_who_lives_there() {
    let w = worldgen::generate(1);
    let (mut makers, mut tenders, mut traders) = (0, 0, 0);
    for ((t, i), jobs, v) in town_buildings(&w) {
        let s = &w.settlements[t as usize];
        let b = &s.buildings[i as usize];
        assert_eq!(s.styles.len(), s.buildings.len());
        // Stored by key at creation, and that key is what's read back.
        assert_eq!(s.styles[i as usize], Some(v.key), "building {t}/{i}");
        assert_eq!(v.kind, b.kind);
        if !matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock) {
            continue;
        }
        let plain = matches!(v.use_, layout::Use::Home | layout::Use::Quarters);
        if b.kind == BuildingKind::RoduroHome {
            // The head of house's trade, and nobody else's, makes a workshop.
            let want = jobs.first().and_then(|&j| layout::trade_style(b.kind, j)).filter(|k| fits(layout::by_key(k).unwrap(), b.size));
            match want {
                Some(k) => assert_eq!(v.key, k, "{t}/{i}: head works {:?}", jobs[0]),
                None => assert!(plain, "{t}/{i}: head has no trade ({jobs:?}), should be a plain home, is {}", v.key),
            }
        } else {
            // The strongest trade, if it's strong enough and the building can be its kind.
            let need = layout::trade_need(jobs.len());
            let count = |key: &str| jobs.iter().filter(|&&j| layout::trade_style(b.kind, j) == Some(key)).count();
            let keys: Vec<&str> = layout::TRADE_STYLES.iter().filter(|r| r.0 == b.kind && fits(layout::by_key(r.2).unwrap(), b.size)).map(|r| r.2).collect();
            let top = keys.iter().map(|k| count(k)).max().unwrap_or(0);
            let winners: Vec<&str> = keys.iter().copied().filter(|k| count(k) == top).collect();
            if top >= need {
                assert!(winners.contains(&v.key), "{t}/{i}: {jobs:?} should be one of {winners:?}, is {}", v.key);
            } else {
                assert!(plain, "{t}/{i}: no strong trade, should be a plain home, is {}", v.key);
            }
        }
        // Real examples of each.
        if jobs.iter().any(|j| matches!(j, Job::Smith | Job::Armourer)) && (v.key == "roduro_forge" || v.key == "qotiro_workyard") {
            makers += 1;
        }
        if jobs.contains(&Job::StoneTender) && v.key == "roduro_tender" {
            tenders += 1;
        }
        if jobs.iter().any(|j| matches!(j, Job::Merchant | Job::Exchanger)) && (v.key == "roduro_trader" || v.key == "qotiro_market") {
            traders += 1;
        }
        // No trade's building without someone of that trade living there.
        if layout::is_trade_style(v) {
            assert!(jobs.iter().any(|&j| layout::keeps(v, j)), "{t}/{i}: a {} with nobody of its trade ({jobs:?})", v.key);
        }
    }
    assert!(makers > 0, "some smith lives in a forge");
    assert!(tenders > 0, "some stone-tender lives in a tender's workshop");
    assert!(traders > 0, "some merchant lives in a trader's house");
}

#[test]
fn a_buildings_style_never_changes() {
    let mut w = worldgen::generate(1);
    // The keys chosen at creation.
    let chosen: Vec<Vec<Option<&str>>> = w.settlements.iter().map(|s| s.styles.clone()).collect();
    let before: Vec<_> = town_buildings(&w).into_iter().map(|(id, jobs, v)| (id, jobs, v.key)).collect();
    // Several dawns: jobs are re-sorted, households re-formed.
    for _ in 0..(4.0 * DAY / 300.0) as usize {
        w.step(300.0);
    }
    let after: Vec<_> = town_buildings(&w).into_iter().map(|(id, jobs, v)| (id, jobs, v.key)).collect();
    assert_eq!(before.len(), after.len());
    let changed = before.iter().zip(&after).filter(|(a, b)| a.1 != b.1).count();
    eprintln!("{changed} of {} buildings' residents changed jobs", before.len());
    for (a, b) in before.iter().zip(&after) {
        assert_eq!(a.0, b.0);
        assert_eq!(a.2, b.2, "building {:?} changed from {} to {}", a.0, a.2, b.2);
    }
    assert_eq!(chosen, w.settlements.iter().map(|s| s.styles.clone()).collect::<Vec<_>>());
    // Every door is the one the creation-time variant gives: same variant,
    // same outline and rooms.
    let mut doors = 0;
    for (t, s) in w.settlements.iter().enumerate() {
        for i in 0..s.buildings.len() as u16 {
            let Some(d) = door_of(s, i) else { continue };
            let b = &s.buildings[i as usize];
            let v = layout::by_key(chosen[t][i as usize].expect("a town building with a door has a style")).unwrap();
            assert_eq!(d.variant().key, v.key, "{t}/{i}");
            assert_eq!((d.half.x, d.half.y), (v.half.0 * b.size, v.half.1 * b.size), "{t}/{i}: outline");
            assert_eq!(d.round, v.shape == Shape::Round);
            assert_eq!(d.wall_pieces().len(), v.walls.iter().map(|wl| if wl.gap.is_some() { 2 } else { 1 }).sum::<usize>());
            doors += 1;
        }
    }
    assert!(doors > 50);
}

#[test]
fn building_styles_survive_save_and_load() {
    let mut w = worldgen::generate(2);
    for _ in 0..40 {
        w.step(60.0);
    }
    let bytes = w.save_bytes();
    let back = World::load_bytes(&bytes).expect("loads");
    let mut keys: Vec<&str> = Vec::new();
    for (a, b) in w.settlements.iter().zip(&back.settlements) {
        assert!(!a.styles.is_empty() || a.buildings.is_empty());
        assert_eq!(a.styles, b.styles);
        for i in 0..a.buildings.len() as u16 {
            // Every stored style is a real key of the building's kind...
            if let Some(k) = b.styles[i as usize] {
                let v = layout::by_key(k).unwrap_or_else(|| panic!("unknown style {k}"));
                assert_eq!(v.kind, b.buildings[i as usize].kind);
                keys.push(k);
            }
            assert_eq!(layout::variant_in(a, i).map(|v| v.key), layout::variant_in(b, i).map(|v| v.key));
            assert_eq!(door_of(a, i).map(|d| d.variant().key), door_of(b, i).map(|d| d.variant().key));
        }
    }
    // ...and the save holds the keys themselves (not places in `VARIANTS`,
    // which adding a variant would shift and re-skin the town).
    keys.sort_unstable();
    keys.dedup();
    assert!(keys.len() > 4);
    for k in keys {
        assert!(bytes.windows(k.len()).any(|x| x == k.as_bytes()), "{k} written by name");
    }
}

#[test]
fn qotiro_quarters_are_shared_out_room_by_room() {
    let mut found = false;
    for seed in 1..6u64 {
        let w = worldgen::generate(seed);
        for s in &w.settlements {
            for i in 0..s.buildings.len() as u16 {
                let Some(d) = door_of(s, i) else { continue };
                let v = d.variant();
                if v.key != "qotiro_quarters" {
                    continue;
                }
                let hs = w.households_in(d.id);
                if hs.len() < 2 {
                    continue;
                }
                found = true;
                // Chests and cupboards, one to each room in turn.
                let private: Vec<(u8, Owner)> = v
                    .holders
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| matches!(h.what, layout::Holder::Chest | layout::Holder::Cupboard))
                    .map(|(k, h)| (k as u8, w.owner_of_slot(d.id, k as u8, h.what)))
                    .collect();
                assert!(private.len() >= 2);
                assert_ne!(private[0].1, private[1].1, "the first two rooms' chests belong to different households");
                for (n, (_, o)) in private.iter().enumerate() {
                    assert_eq!(*o, Owner::Household(hs[n % hs.len()]));
                }
                // Each owner really lives there (someone of theirs sleeps there).
                for (_, o) in &private {
                    let Owner::Household(h) = o else { panic!("a household's") };
                    assert!(w.society.households[*h as usize].members.iter().any(|&m| w.people[m as usize].dwelling == Some(i)));
                }
                // Shared stores go to the first household.
                for (k, h) in v.holders.iter().enumerate() {
                    if matches!(h.what, layout::Holder::Crate | layout::Holder::Barrel) {
                        assert_eq!(w.owner_of_slot(d.id, k as u8, h.what), Owner::Household(hs[0]));
                    }
                }
                // Nobody living in a building: the town's.
                assert_eq!(w.owner_of_slot((s.id, u16::MAX - 1), 0, layout::Holder::Chest), Owner::Town(s.id));
            }
        }
        if found {
            break;
        }
    }
    assert!(found, "a Qotiro quarters with two households or more");
}

#[test]
fn towns_keep_plain_homes() {
    let mut blocks = (0, 0);
    let mut homes = (0, 0);
    for seed in 1..4u64 {
        let w = worldgen::generate(seed);
        for s in &w.settlements {
            let mut plain = 0;
            let mut built = 0;
            for i in 0..s.buildings.len() as u16 {
                let b = &s.buildings[i as usize];
                if !matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock) {
                    continue;
                }
                built += 1;
                let v = layout::variant_in(s, i).unwrap();
                let is_plain = matches!(v.use_, layout::Use::Home | layout::Use::Quarters);
                plain += is_plain as usize;
                let tally = if b.kind == BuildingKind::QotiroBlock { &mut blocks } else { &mut homes };
                tally.0 += 1;
                tally.1 += is_plain as usize;
            }
            if built >= 3 {
                assert!(plain > 0, "{} has {built} homes and none plain", s.name);
            }
        }
    }
    eprintln!("Roduro homes: {} of {} plain; Qotiro blocks: {} of {} plain", homes.1, homes.0, blocks.1, blocks.0);
    assert!(blocks.0 > 0 && blocks.1 > 0, "some Qotiro blocks are plain quarters");
    assert!(blocks.1 < blocks.0, "and some are workshops, shops or halls");
    assert!(blocks.1 * 3 >= blocks.0, "most Qotiro blocks shouldn't be workshops: {} of {} plain", blocks.1, blocks.0);
    assert!(homes.1 * 3 >= homes.0, "plenty of plain Roduro homes: {} of {}", homes.1, homes.0);
}

// ---- Everything inside the walls -------------------------------------------------

/// A footprint on the floor: centre, turn, half width, half depth (metres,
/// in the building's frame).
type Foot = (V2, f32, f32, f32);

/// Do two turned boxes overlap (by more than a couple of centimetres)?
fn overlap(a: Foot, b: Foot) -> bool {
    let axes = |f: Foot| [V2::new(f.1.cos(), f.1.sin()), V2::new(-f.1.sin(), f.1.cos())];
    let reach = |f: Foot, n: V2| {
        let [u, w] = axes(f);
        f.2 * (u.x * n.x + u.y * n.y).abs() + f.3 * (w.x * n.x + w.y * n.y).abs()
    };
    for n in axes(a).into_iter().chain(axes(b)) {
        let d = b.0.sub(a.0);
        if (d.x * n.x + d.y * n.y).abs() >= reach(a, n) + reach(b, n) - 0.02 {
            return false;
        }
    }
    true
}

/// The sizes a variant's buildings really come in (the ends of the range).
fn real_sizes(v: &layout::Variant) -> [f32; 2] {
    match v.kind {
        BuildingKind::RoduroHome => [v.sizes.0.max(7.0), v.sizes.1.min(12.5)],
        BuildingKind::QotiroBlock => [v.sizes.0.max(14.0), v.sizes.1.min(20.0)],
        BuildingKind::QotiroHall => [16.0, 16.0],
        BuildingKind::QotiroTemple => [40.0, 75.0],
        _ => [9.0, 9.0],
    }
}

/// A door for a variant at a size, standing at the origin facing +x.
fn test_door(v: &layout::Variant, size: f32) -> Door {
    let half = V2::new(v.half.0 * size, v.half.1 * size);
    let round = v.shape == Shape::Round;
    let across = v.door * half.y * 0.8;
    let front = if round { half.x * (1.0 - (across / half.y).powi(2)).max(0.0).sqrt() } else { half.x };
    let face = V2::new(front, across);
    Door { id: (0, 0), outside: face.add(V2::new(1.2, 0.0)), inside: face.sub(V2::new(1.6, 0.0)), centre: V2::new(0.0, 0.0), radius: half.len(), lock: 0.0, rot: 0.0, half, round, variant: layout::index_of(v) as u16 }
}

/// What's wrong with a variant's floor at a size: furniture or a container
/// spot poking into the outer wall or standing in an inner doorway, and
/// anything overlapping anything else (inner walls included; walls meeting
/// walls are joints).
fn floor_faults(v: &layout::Variant, size: f32) -> Vec<String> {
    let d = test_door(v, size);
    let name = |t: layout::Thing| match t {
        layout::Thing::Furniture(k) => format!("{} {k}", v.furniture[k].what.name()),
        layout::Thing::Container(k) => format!("{} (slot {k})", v.holders[k].what.name()),
        layout::Thing::Wall(k) => format!("inner wall {k}"),
        layout::Thing::Extra(k) => format!("extra {k}"),
    };
    let things: Vec<(layout::Thing, Foot)> = layout::footprints(&d).into_iter().map(|(t, r)| (t, as_foot(&d, &r))).collect();
    let doorways: Vec<Foot> = layout::keep_clear(&d).iter().skip(1).map(|r| as_foot(&d, r)).collect();
    let wall = |t: layout::Thing| matches!(t, layout::Thing::Wall(_));
    let mut out = Vec::new();
    for (t, f) in &things {
        if wall(*t) {
            continue;
        }
        if !inside_by(&d, *f, layout::WALL_IN) {
            out.push(format!("{} at {size} m: {} is against or past the wall", v.key, name(*t)));
        }
        if doorways.iter().any(|g| overlap(*f, *g)) {
            out.push(format!("{} at {size} m: {} stands in a doorway", v.key, name(*t)));
        }
    }
    for (i, a) in things.iter().enumerate() {
        for b in &things[i + 1..] {
            if !(wall(a.0) && wall(b.0)) && overlap(a.1, b.1) {
                out.push(format!("{} at {size} m: {} overlaps {}", v.key, name(a.0), name(b.0)));
            }
        }
    }
    out
}

/// Every variant's own furniture and container spots, at both ends of the
/// sizes its buildings come in (layouts scale with the building, furniture
/// doesn't: the smallest is the tightest).
#[test]
fn everything_stands_inside_the_walls() {
    let mut faults = Vec::new();
    for v in VARIANTS.iter().filter(|v| !v.holders.is_empty() || !v.furniture.is_empty()) {
        for size in real_sizes(v) {
            faults.extend(floor_faults(v, size));
        }
    }
    assert!(faults.is_empty(), "{}", faults.join("\n"));
}

// ---- Nothing in a room overlaps, as drawn ---------------------------------------

fn as_foot(d: &Door, r: &layout::Rect) -> Foot {
    // In the building's own frame (metres), like `foot`.
    let q = r.c.sub(d.centre);
    let (c, s) = (d.rot.cos(), d.rot.sin());
    (V2::new(q.x * c + q.y * s, -q.x * s + q.y * c), r.rot - d.rot, r.hw, r.hd)
}

/// Is every corner of a footprint inside the outline, `margin` in?
fn inside_by(d: &Door, f: Foot, margin: f32) -> bool {
    let (hx, hy) = (d.half.x - margin, d.half.y - margin);
    let (u, w) = (V2::new(f.1.cos(), f.1.sin()), V2::new(-f.1.sin(), f.1.cos()));
    [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().all(|&(a, b)| {
        let p = f.0.add(u.scale(a * f.2)).add(w.scale(b * f.3));
        if d.round { (p.x / hx).powi(2) + (p.y / hy).powi(2) <= 1.0 + 1e-4 } else { p.x.abs() <= hx + 1e-4 && p.y.abs() <= hy + 1e-4 }
    })
}

/// Everything drawn on a building's floor, as named footprints: the
/// variant's furniture, containers and inner walls, the extra bedrolls for
/// its residents, and a lodger's corner (always worked out, so every
/// building's floor is checked). Each thing: (name, kind, footprint), kind
/// 'f' furniture, 'c' container, 'w' wall, 'x' extra, 'k' corner prop,
/// 'r' corner rug/drape, 's' parchment wall.
fn floor_plan(w: &World, d: &Door) -> (Vec<(String, char, Foot)>, Option<layout::Corner>) {
    let mut out = Vec::new();
    for (t, r) in layout::footprints(d) {
        let (name, kind) = match t {
            layout::Thing::Furniture(k) => (format!("{} {k}", d.variant().furniture[k].what.name()), 'f'),
            layout::Thing::Container(k) => (format!("{} (slot {k})", d.variant().holders[k].what.name()), 'c'),
            layout::Thing::Wall(k) => (format!("inner wall {k}"), 'w'),
            layout::Thing::Extra(k) => (format!("extra {k}"), 'x'),
        };
        out.push((name, kind, as_foot(d, &r)));
    }
    let rs = w.residents_of(d.id);
    let lodger = rs.iter().any(|&p| w.people[p as usize].race == Race::Tadoro);
    let (extras, corner) = layout::sleeping_plan(d, rs.len(), lodger);
    // No lodger: lay a corner anyway, so every building's floor is checked.
    let corner = corner.or_else(|| layout::lodger_corner(d, &extras));
    for (k, &(at, rot)) in extras.iter().enumerate() {
        out.push((format!("extra bedroll {k}"), 'x', as_foot(d, &layout::bedroll(at, rot))));
    }
    if let Some(c) = &corner {
        if c.on.is_none() {
            out.push(("lodger's fresh bedroll".to_string(), 'x', as_foot(d, &c.bed_rect())));
        }
        for (name, r) in c.pieces() {
            let kind = match name {
                "rug" | "drape" => 'r',
                "parchment wall" => 's',
                _ => 'k',
            };
            out.push((format!("corner {name}"), kind, as_foot(d, &r)));
        }
    }
    (out, corner)
}

/// What's wrong with a building's floor as drawn.
fn plan_faults(w: &World, d: &Door) -> Vec<String> {
    let (things, corner) = floor_plan(w, d);
    let key = d.variant().key;
    let mut out = Vec::new();
    // The corner's own bed: its rug and drape lie under and over it.
    let own = corner.as_ref().and_then(|c| match c.on {
        Some(layout::Thing::Furniture(k)) => Some(format!("{} {k}", d.variant().furniture[k].what.name())),
        Some(layout::Thing::Extra(k)) => Some(format!("extra bedroll {k}")),
        Some(_) => None,
        None => Some("lodger's fresh bedroll".to_string()),
    });
    let corner_part = |n: &str, k: char| matches!(k, 'k' | 'r' | 's') || Some(n) == own.as_deref();
    for (i, a) in things.iter().enumerate() {
        for b in &things[i + 1..] {
            let (ca, cb) = (corner_part(&a.0, a.1), corner_part(&b.0, b.1));
            // Within the corner, only its props must keep apart (from each
            // other and from the bed); the rug, drape and wall patch go under,
            // over and behind them.
            if ca && cb && !(matches!((a.1, b.1), ('k', 'k')) || (a.1 == 'k' && Some(b.0.as_str()) == own.as_deref()) || (b.1 == 'k' && Some(a.0.as_str()) == own.as_deref())) {
                continue;
            }
            // Inner walls meet at joints.
            if a.1 == 'w' && b.1 == 'w' {
                continue;
            }
            if overlap(a.2, b.2) {
                let what = if a.1 == 'c' || b.1 == 'c' { "container spot overlaps" } else { "overlaps" };
                out.push(format!("{key} {:?}: {} {what} {}", d.id, a.0, b.0));
            }
        }
    }
    // Everything laid on top of the variant inside the walls, clear of the
    // way in and the doorways.
    let dir = V2::new(1.0, 0.0);
    let to_frame = |p: V2| {
        let q = p.sub(d.centre);
        let (c, s) = (d.rot.cos(), d.rot.sin());
        V2::new(q.x * c + q.y * s, -q.x * s + q.y * c)
    };
    let inside = to_frame(d.inside);
    let face = to_frame(d.outside).sub(dir.scale(1.2));
    let way_in: Foot = (face.lerp(inside, 0.5), 0.0, face.dist(inside) * 0.5, 0.4);
    let at_inside: Foot = (inside, 0.0, 0.4, 0.4);
    let doorways: Vec<Foot> = d.doorways().into_iter().map(|g| (to_frame(g), 0.0, 0.4, 0.4)).collect();
    for (name, kind, f) in &things {
        if *kind != 'w' && !inside_by(d, *f, layout::WALL_IN) {
            out.push(format!("{key} {:?}: {name} pokes into the outer wall", d.id));
        }
        // (The variant's own pieces keep the doorways clear in
        // `everything_stands_inside_the_walls`, and the way in in
        // `every_variant_can_be_walked_into`.)
        if matches!(kind, 'f' | 'c' | 'w') {
            continue;
        }
        if overlap(*f, way_in) || overlap(*f, at_inside) {
            out.push(format!("{key} {:?}: {name} blocks the way in", d.id));
        }
        if doorways.iter().any(|g| overlap(*f, *g)) {
            out.push(format!("{key} {:?}: {name} blocks a doorway", d.id));
        }
    }
    out
}

/// What's wrong with where a building's loose belongings lie
/// (`layout::loose_spots`, as `World::furnish` lays them; `n` of them).
fn loose_faults(w: &World, d: &Door, seed: u64, n: usize) -> Vec<String> {
    let key = d.variant().key;
    let rs = w.residents_of(d.id);
    let lodger = rs.iter().any(|&p| w.people[p as usize].race == Race::Tadoro);
    let spots = layout::loose_spots(d, rs.len(), lodger, n, seed);
    let mut out = Vec::new();
    if spots.len() != n {
        out.push(format!("{key} {:?}: {} spots for {n} things", d.id, spots.len()));
    }
    let (things, _) = floor_plan(w, d);
    let to_frame = |p: V2| {
        let q = p.sub(d.centre);
        let (c, s) = (d.rot.cos(), d.rot.sin());
        V2::new(q.x * c + q.y * s, -q.x * s + q.y * c)
    };
    let inside = to_frame(d.inside);
    let face = to_frame(d.outside).sub(V2::new(1.2, 0.0));
    let way_in: Foot = (face.lerp(inside, 0.5), 0.0, face.dist(inside) * 0.5, 0.4);
    let at_inside: Foot = (inside, 0.0, 0.4, 0.4);
    let doorways: Vec<Foot> = d.doorways().into_iter().map(|g| (to_frame(g), 0.0, 0.4, 0.4)).collect();
    let h = layout::LOOSE * 0.5;
    let mut laid: Vec<Foot> = Vec::new();
    for (j, sp) in spots.iter().enumerate() {
        if !d.contains(sp.at) || sp.at.dist(d.centre) >= d.radius {
            out.push(format!("{key} {:?}: loose thing {j} lies outside", d.id));
        }
        if let Some(k) = sp.on {
            // On a piece's top: the piece holds things, and it's on it.
            let p = d.variant().furniture[k];
            let on = layout::resting(d, sp.at);
            if !layout::holds_things(p.what) || on.is_none() {
                out.push(format!("{key} {:?}: loose thing {j} isn't on top of {} {k}", d.id, p.what.name()));
            }
            continue;
        }
        let f: Foot = (to_frame(sp.at), 0.0, h, h);
        if !inside_by(d, f, layout::WALL_IN) {
            out.push(format!("{key} {:?}: loose thing {j} pokes into the outer wall", d.id));
        }
        // (A corner laid where no lodger lives is only for checking.)
        for (name, kind, g) in &things {
            let hypothetical = !lodger && (matches!(kind, 'k' | 'r' | 's') || name == "lodger's fresh bedroll");
            if !hypothetical && overlap(f, *g) {
                out.push(format!("{key} {:?}: loose thing {j} lies on {name}", d.id));
            }
        }
        if overlap(f, way_in) || overlap(f, at_inside) {
            out.push(format!("{key} {:?}: loose thing {j} blocks the way in", d.id));
        }
        if doorways.iter().any(|g| overlap(f, *g)) {
            out.push(format!("{key} {:?}: loose thing {j} blocks a doorway", d.id));
        }
        if laid.iter().any(|g| overlap(f, *g)) {
            out.push(format!("{key} {:?}: loose thing {j} lies on another", d.id));
        }
        laid.push(f);
    }
    out
}

#[test]
fn nothing_drawn_in_a_room_overlaps() {
    let mut faults = Vec::new();
    let (mut buildings, mut extras, mut corners, mut lodged, mut fresh, mut loose) = (0, 0, 0, 0, 0, 0);
    for seed in 1..=3u64 {
        let w = worldgen::generate(seed);
        for s in &w.settlements {
            for i in 0..s.buildings.len() as u16 {
                let Some(d) = door_of(s, i) else { continue };
                buildings += 1;
                let rs = w.residents_of(d.id);
                let lodger = rs.iter().any(|&p| w.people[p as usize].race == Race::Tadoro);
                let (laid, corner) = layout::sleeping_plan(&d, rs.len(), lodger);
                extras += laid.len();
                if corner.is_some() {
                    corners += 1;
                }
                if corner.is_some_and(|c| c.on.is_none()) {
                    fresh += 1;
                }
                if lodger {
                    lodged += 1;
                    if corner.is_none() {
                        faults.push(format!("seed {seed} {} {:?}: a Ṭaḍoro lodges here and no corner fits", d.variant().key, d.id));
                    }
                }
                for f in plan_faults(&w, &d) {
                    faults.push(format!("seed {seed} {f}"));
                }
                // Loose belongings, the most `furnish` lays out (3, a temple 6).
                let bd = &s.buildings[i as usize];
                let n = if bd.kind == BuildingKind::QotiroTemple { 6 } else { 3 };
                for f in loose_faults(&w, &d, bd.seed, n) {
                    faults.push(format!("seed {seed} {f}"));
                }
                loose += n;
            }
        }
    }
    eprintln!("{buildings} buildings, {extras} extra bedrolls, {lodged} with a Ṭaḍoro lodger: {corners} corners ({fresh} on a fresh bedroll), {loose} loose things; {} faults", faults.len());
    let shown: Vec<&String> = faults.iter().take(40).collect();
    assert!(faults.is_empty(), "{} faults, first:\n{}", faults.len(), shown.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n"));
}

// ---- Who lives where, and whose things they are ------------------------------

#[test]
fn a_tier_block_household_lives_in_both_its_blocks() {
    let mut found = false;
    'seeds: for seed in 1..6u64 {
        let w = worldgen::generate(seed);
        for (h, hh) in w.society.households.iter().enumerate() {
            let c = &w.society.communities[hh.community as usize];
            if c.customs.belonging != Belonging::TierBlock {
                continue;
            }
            let at_home = |&&m: &&PersonId| {
                let p = &w.people[m as usize];
                !p.dead && !p.in_squad && p.home == Some(c.town)
            };
            let mut blocks: Vec<u16> = hh.members.iter().filter(at_home).filter_map(|&m| w.people[m as usize].dwelling).collect();
            blocks.sort_unstable();
            blocks.dedup();
            if blocks.len() < 2 {
                continue;
            }
            found = true;
            for &b in &blocks {
                let rs = w.residents_of((c.town, b));
                for &m in hh.members.iter().filter(at_home) {
                    if w.people[m as usize].dwelling == Some(b) {
                        assert!(rs.contains(&m), "seed {seed}: {m} sleeps in block {b} but isn't a resident");
                    }
                }
                assert!(!rs.is_empty(), "seed {seed}: block {b} looks empty");
                assert!(w.households_in((c.town, b)).contains(&(h as u32)), "seed {seed}: household {h} lives in block {b}");
                // Its things belong to someone living there, not the town.
                if let Some(d) = door_of(&w.settlements[c.town as usize], b) {
                    for (k, sp) in d.variant().holders.iter().enumerate() {
                        assert!(matches!(w.owner_of_slot(d.id, k as u8, sp.what), Owner::Household(_)), "seed {seed}: block {b}'s things are the town's");
                    }
                }
            }
            break 'seeds;
        }
    }
    assert!(found, "a tier-block household over two blocks");
}

#[test]
fn residents_are_everyone_sleeping_there() {
    let w = worldgen::generate(1);
    for s in &w.settlements {
        for i in 0..s.buildings.len() as u16 {
            let rs = w.residents_of((s.id, i));
            for &p in &s.residents {
                let q = &w.people[p as usize];
                let lives_here = q.dwelling == Some(i) && !q.dead && !q.in_squad;
                assert_eq!(rs.contains(&p), lives_here);
            }
            let hs = w.households_in((s.id, i));
            assert!(hs.windows(2).all(|x| x[0] < x[1]), "households in index order, once each");
            for &p in &rs {
                assert!(w.society.lives[p as usize].household.is_none_or(|h| hs.contains(&h)));
            }
        }
    }
}

#[test]
fn container_owners_follow_renumbered_households() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let m = w.squad.members[0];
    let t = start_town(&w) as u16;
    // A home with people living in it, near the middle.
    let d = town_doors(&w).into_iter().find(|d| !d.variant().holders.is_empty() && !w.households_in(d.id).is_empty()).expect("a lived-in home");
    // Number the households the other way round (an equally good numbering).
    let n = w.society.households.len() as u32;
    let flip = |h: u32| n - 1 - h;
    w.society.households.reverse();
    for l in &mut w.society.lives {
        l.household = l.household.map(flip);
    }
    for hh in &mut w.society.households {
        for f in &mut hh.feelings {
            f.other = flip(f.other);
        }
        for debt in &mut hh.purse.debts {
            if let gahturiyu_sim::sim::lives::Creditor::Household(o) = &mut debt.to {
                *o = flip(*o);
            }
        }
    }
    for r in &mut w.society.rings {
        r.paying.clear();
    }
    assert!(walk_in(&mut w, m, &d), "got in");
    let laid: Vec<ContainerId> = w.containers_in(d.id).map(|c| c.id).collect();
    assert!(!laid.is_empty());
    let before: Vec<Owner> = laid.iter().map(|id| w.container(*id).unwrap().owner).collect();
    assert!(before.iter().any(|o| matches!(o, Owner::Household(_))));
    // Make the town's next dawn re-form every household: its head count looks
    // moved and its belonging looks changed, so it re-blends and regroups.
    let ci = w.society.towns[t as usize].shore;
    let real = w.society.communities[ci as usize].customs.belonging;
    let fake = [Belonging::Lineage, Belonging::TierBlock, Belonging::Village, Belonging::Lodging].into_iter().find(|b| *b != real).unwrap();
    w.society.communities[ci as usize].customs.belonging = fake;
    w.society.communities[ci as usize].counted = [0; 4];
    until_hour(&mut w, 7.0);
    assert_eq!(w.society.communities[ci as usize].customs.belonging, real, "re-blended at dawn");
    let after: Vec<Owner> = laid.iter().map(|id| w.container(*id).unwrap().owner).collect();
    assert_ne!(before, after, "households were renumbered");
    for (id, o) in laid.iter().zip(&after) {
        assert_eq!(Some(*o), w.container_owner(*id), "stored owner is up to date");
        match o {
            Owner::Household(h) => {
                let hh = &w.society.households[*h as usize];
                assert!(hh.members.iter().any(|&p| w.people[p as usize].dwelling == Some(d.id.1)), "owner lives there");
            }
            Owner::Town(x) => assert_eq!(*x, t),
        }
    }
}

// ---- Trades and head counts pick the building ------------------------------------

#[test]
fn trades_get_their_own_building_and_only_the_head_makes_a_roduro_workshop() {
    use Job::*;
    let k = BuildingKind::RoduroHome;
    for (jobs, key) in [(&[Weaver, Tailor][..], "roduro_loomroom"), (&[Tanner, Leatherworker, Woodworker, Carpenter, Mason, Boatwright][..], "roduro_benchroom")] {
        for &job in jobs {
            for seed in 0..20 {
                let v = layout::style_for(k, 10.0, seed, &[job, Farmer, Farmer]).unwrap();
                assert_eq!(v.key, key, "{job:?}");
            }
        }
    }
    for job in [Smith, Armourer, CharcoalBurner] {
        assert_eq!(layout::style_for(k, 10.0, 1, &[job, Farmer]).unwrap().key, "roduro_forge", "{job:?}");
    }
    // Only the head of house counts in a Roduro home.
    for job in [Smith, Weaver, Merchant, StoneTender] {
        let v = layout::style_for(k, 10.0, 1, &[Farmer, job, job]).unwrap();
        assert!(matches!(v.use_, layout::Use::Home), "{job:?} not the head: {}", v.key);
    }
    // Alchemists and scribes work elsewhere; their homes are plain.
    for job in [Alchemist, Scribe] {
        assert_eq!(layout::style_for(k, 10.0, 1, &[job]).unwrap().use_, layout::Use::Home);
    }
    // Qotiro: weavers a weaving court; smiths a yard; benchworkers a bench yard; dealers a market.
    let q = BuildingKind::QotiroBlock;
    assert_eq!(layout::style_for(q, 18.0, 1, &[Weaver, Farmer]).unwrap().key, "qotiro_weavehall");
    assert_eq!(layout::style_for(q, 18.0, 1, &[Tailor, Farmer]).unwrap().key, "qotiro_weavehall");
    assert_eq!(layout::style_for(q, 18.0, 1, &[Smith, Farmer]).unwrap().key, "qotiro_workyard");
    assert_eq!(layout::style_for(q, 18.0, 1, &[Tanner, Farmer]).unwrap().key, "qotiro_benchyard");
    assert_eq!(layout::style_for(q, 18.0, 1, &[Mason, Farmer]).unwrap().key, "qotiro_benchyard");
    assert_eq!(layout::style_for(q, 18.0, 1, &[Merchant, Farmer]).unwrap().key, "qotiro_market");
    assert!(matches!(layout::style_for(q, 18.0, 1, &[Alchemist, Farmer]).unwrap().use_, layout::Use::Home | layout::Use::Quarters));
    // Nobody's home is a mess hall.
    for seed in 0..50 {
        let v = layout::style_for(q, 18.0, seed, &[Cook, Cook, Innkeeper, Runner]).unwrap();
        assert_ne!(v.key, "qotiro_mess");
    }
    // In real towns: no mess hall for a home; some loom rooms and bench rooms.
    let (mut looms, mut benches) = (0, 0);
    for seed in 1..4u64 {
        let w = worldgen::generate(seed);
        for (_, _, v) in town_buildings(&w) {
            assert_ne!(v.key, "qotiro_mess", "a mess hall chosen for a home");
            looms += (v.key == "roduro_loomroom") as usize;
            benches += (v.key == "roduro_benchroom") as usize;
        }
    }
    assert!(looms > 0 && benches > 0, "{looms} loom rooms, {benches} bench rooms");
}

#[test]
fn plain_homes_go_by_head_count() {
    let farmers = |n: usize| vec![Job::Farmer; n];
    let r = BuildingKind::RoduroHome;
    let pick = |kind, size, n| layout::style_for(kind, size, 3, &farmers(n)).unwrap().key;
    // A plot that takes a cottage or a longhouse.
    assert_eq!(pick(r, 9.0, 0), "roduro_cottage");
    assert_eq!(pick(r, 9.0, 3), "roduro_cottage");
    assert_eq!(pick(r, 9.0, 4), "roduro_longhouse");
    assert_eq!(pick(r, 9.0, 5), "roduro_longhouse", "a great house won't fit: the nearest that does");
    assert_eq!(pick(r, 9.0, 8), "roduro_longhouse");
    // A big plot: five or more make a great house.
    assert_eq!(pick(r, 12.0, 2), "roduro_longhouse", "a cottage won't fit");
    assert_eq!(pick(r, 12.0, 4), "roduro_longhouse");
    assert_eq!(pick(r, 12.0, 5), "roduro_great");
    assert_eq!(pick(r, 11.0, 6), "roduro_great");
    // A small plot: a cottage whatever.
    assert_eq!(pick(r, 7.5, 9), "roduro_cottage");
    let q = BuildingKind::QotiroBlock;
    assert_eq!(pick(q, 18.0, 5), "qotiro_court");
    assert_eq!(pick(q, 18.0, 6), "qotiro_quarters");
    assert_eq!(pick(q, 14.0, 3), "qotiro_quarters", "a courtyard house won't fit");
    // The same in real towns, and how they come out.
    let mut tally: std::collections::BTreeMap<&str, usize> = Default::default();
    for seed in 1..4u64 {
        let w = worldgen::generate(seed);
        for ((t, i), jobs, v) in town_buildings(&w) {
            let b = &w.settlements[t as usize].buildings[i as usize];
            if !matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock) {
                continue;
            }
            let trade = layout::is_trade_style(v);
            if !trade {
                assert_eq!(Some(v.key), layout::plain_home(b.kind, b.size, jobs.len()).map(|x| x.key), "{t}/{i}: {} living there", jobs.len());
            }
            *tally.entry(if trade { "trade" } else { "plain" }).or_default() += 1;
            *tally.entry(v.key).or_default() += 1;
        }
    }
    eprintln!("{tally:?}");
    assert!(tally.get("roduro_great").copied().unwrap_or(0) > 0, "some great houses");
}

// ---- One rule for trade, keeper and sign -------------------------------------------

#[test]
fn trade_styles_keepers_and_signs_agree() {
    // Every key in the tables is a real variant of its kind.
    for (kind, jobs, key) in layout::TRADE_STYLES {
        let v = layout::by_key(key).unwrap_or_else(|| panic!("{key} isn't a variant"));
        assert_eq!(v.kind, *kind, "{key}");
        assert!(!jobs.is_empty());
        assert!(layout::sign_of(v).is_some(), "{key}: a trade's building has a sign");
        assert!(!matches!(v.use_, layout::Use::Home | layout::Use::Quarters), "{key}: a trade's building isn't a plain home");
    }
    for (kind, key, _) in layout::PLAIN_HOMES {
        let v = layout::by_key(key).unwrap_or_else(|| panic!("{key} isn't a variant"));
        assert_eq!(v.kind, *kind, "{key}");
        assert_eq!(layout::sign_of(v), None, "{key}: a plain home has no sign");
        assert!(!layout::is_trade_style(v));
    }
    // No job asks for two buildings of one kind.
    for (kind, jobs, key) in layout::TRADE_STYLES {
        for j in *jobs {
            let n = layout::TRADE_STYLES.iter().filter(|r| r.0 == *kind && r.1.contains(j)).count();
            assert_eq!(n, 1, "{j:?} asks for more than one {kind:?} ({key})");
        }
    }
    // `keeps` is exactly the table.
    for v in VARIANTS {
        for &j in ALL_JOBS.iter() {
            let in_table = layout::TRADE_STYLES.iter().any(|r| r.0 == v.kind && r.2 == v.key && r.1.contains(&j));
            assert_eq!(layout::keeps(v, j), in_table, "{} / {j:?}", v.key);
        }
    }
    // Keys are unique, and the stilts, quarters and homes carry no sign.
    for (k, v) in VARIANTS.iter().enumerate() {
        assert!(VARIANTS.iter().skip(k + 1).all(|x| x.key != v.key), "{} twice", v.key);
        if v.kind == BuildingKind::HoraroStilt || matches!(v.use_, layout::Use::Home | layout::Use::Quarters) {
            assert_eq!(layout::sign_of(v), None, "{}", v.key);
        }
    }
    use layout::Sign;
    let sign = |k: &str| layout::sign_of(layout::by_key(k).unwrap());
    assert_eq!(sign("roduro_forge"), Some(Sign::Anvil));
    assert_eq!(sign("qotiro_workyard"), Some(Sign::Anvil));
    assert_eq!(sign("roduro_loomroom"), Some(Sign::Loom));
    assert_eq!(sign("qotiro_weavehall"), Some(Sign::Loom));
    assert_eq!(sign("roduro_benchroom"), Some(Sign::Hammer));
    assert_eq!(sign("qotiro_benchyard"), Some(Sign::Hammer));
    assert_eq!(sign("roduro_trader"), Some(Sign::Coin));
    assert_eq!(sign("qotiro_market"), Some(Sign::Scales));
    assert_eq!(sign("qotiro_mess"), Some(Sign::Bowl));
    assert_eq!(sign("qotiro_temple"), Some(Sign::Sun));
    assert_eq!(sign("qotiro_court"), None);
    // The picture is fixed by the variant: looms only where the sign is a loom.
    let has = |k: &str, f: layout::Furn| layout::by_key(k).unwrap().furniture.iter().any(|p| p.what == f);
    assert!(has("roduro_loomroom", layout::Furn::Loom) && has("qotiro_weavehall", layout::Furn::Loom));
    assert!(!has("roduro_benchroom", layout::Furn::Loom) && has("roduro_benchroom", layout::Furn::Rack));
    for v in VARIANTS.iter().filter(|v| v.furniture.iter().any(|p| p.what == layout::Furn::Loom)) {
        assert_eq!(layout::sign_of(v), Some(Sign::Loom), "{} has a loom", v.key);
    }
}

#[test]
fn every_trade_building_has_a_keeper_at_creation() {
    for seed in 1..=3u64 {
        let w = worldgen::generate(seed);
        let mut tally: std::collections::BTreeMap<&str, usize> = Default::default();
        let (mut signed, mut orphans, mut roduro, mut roduro_shops) = (0, 0, 0, 0);
        for ((t, i), _, v) in town_buildings(&w) {
            *tally.entry(v.key).or_default() += 1;
            if v.kind == BuildingKind::RoduroHome {
                roduro += 1;
                roduro_shops += layout::is_trade_style(v) as usize;
            }
            if layout::sign_of(v).is_some() && layout::is_trade_style(v) {
                signed += 1;
                match w.keeper((t, i)) {
                    Some(p) => {
                        assert!(layout::keeps(v, w.life(p).job));
                        assert!(w.residents_of((t, i)).contains(&p));
                    }
                    None => orphans += 1,
                }
            } else {
                assert_eq!(w.keeper((t, i)), None, "{t}/{i}: {} has no trade to keep", v.key);
            }
        }
        eprintln!("seed {seed}: {tally:?}");
        eprintln!("seed {seed}: {signed} trade buildings, {orphans} without a keeper; Roduro workshops {roduro_shops} of {roduro} ({:.0}%)", 100.0 * roduro_shops as f32 / roduro.max(1) as f32);
        assert_eq!(orphans, 0, "seed {seed}");
        assert!(signed > 0);
        let share = roduro_shops as f32 / roduro.max(1) as f32;
        assert!((0.05..0.3).contains(&share), "seed {seed}: Roduro workshop share {share}");
    }
}

#[test]
fn trade_crates_follow_the_keepers_trade() {
    let smith = ["iron_ingot", "iron_ore", "charcoal", "bronze_ingot"].map(items::id);
    let mason = ["rock", "sand", "clay"].map(items::id);
    for key in ["qotiro_workyard", "roduro_forge"] {
        let v = layout::by_key(key).unwrap();
        for (slot, spot) in v.holders.iter().enumerate().filter(|(_, s)| s.what == layout::Holder::Crate) {
            for b in 0..300u16 {
                let (items, _) = stock(7, (3, b, slot as u8), spot.what, v.use_, v.key, Some(Job::Smith));
                assert!(items.iter().all(|e| smith.contains(&e.0)), "{key}: a smith's crate holds {items:?}");
            }
        }
    }
    for key in ["roduro_benchroom", "qotiro_benchyard"] {
        let v = layout::by_key(key).unwrap();
        for (slot, spot) in v.holders.iter().enumerate().filter(|(_, s)| s.what == layout::Holder::Crate) {
            for b in 0..300u16 {
                let (items, _) = stock(7, (3, b, slot as u8), spot.what, v.use_, v.key, Some(Job::Mason));
                assert!(items.iter().all(|e| mason.contains(&e.0)), "{key}: a mason's crate holds {items:?}");
            }
        }
    }
    // In a real town: a laid-out trade building's crates are what its keeper works.
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let m = w.squad.members[0];
    let d = town_doors(&w).into_iter().find(|d| layout::is_trade_style(d.variant()) && !w.is_locked(d.id)).expect("a trade building");
    let kj = keeper_job(&w, d.id);
    assert!(kj.is_some());
    assert!(walk_in(&mut w, m, &d), "got in");
    for c in w.containers_in(d.id) {
        let v = d.variant();
        assert_eq!(c.items, stock(w.seed, c.id, c.what, v.use_, v.key, kj).0);
    }
}

#[test]
fn container_owners_follow_a_death() {
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 9.0);
    let m = w.squad.members[0];
    let d = town_doors(&w).into_iter().find(|d| !d.variant().holders.is_empty() && !w.households_in(d.id).is_empty()).expect("a lived-in home");
    assert!(walk_in(&mut w, m, &d), "got in");
    let laid: Vec<ContainerId> = w.containers_in(d.id).map(|c| c.id).collect();
    assert!(laid.iter().any(|id| matches!(w.container(*id).unwrap().owner, Owner::Household(_))));
    // Everyone living there dies; their things pass to the town.
    for p in w.residents_of(d.id) {
        w.people[p as usize].dead = true;
    }
    w.refresh_owners_in(d.id);
    for id in &laid {
        assert_eq!(w.container(*id).unwrap().owner, Owner::Town(d.id.0));
        assert_eq!(Some(w.container(*id).unwrap().owner), w.container_owner(*id));
    }
}
