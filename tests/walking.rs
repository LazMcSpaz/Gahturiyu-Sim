//! Walking round what's solid (`sim/walk.rs`, over `sim/solid.rs`): Laz's
//! P21 (through a fence, through the hall) and P22 (stuck in a trader's
//! counter), and then the same for every town: no walk passes through
//! anything solid, in and out is by doors and gates, and a walk to a thing
//! ends beside it.

use gahturiyu_sim::sim::{
    geo::V2,
    jobs::PlaceKind,
    layout::Furn,
    person::PersonId,
    solid::{Id, Thing, What, BODY},
    worldgen, World,
};

fn unit(a: f32) -> V2 {
    V2::new(a.cos(), a.sin())
}

/// Put the squad at `from`, send its first member to `to`, and walk them
/// there: every spot they stood on, a quarter second apart.
fn walk(w: &mut World, from: V2, to: V2) -> (PersonId, Vec<V2>) {
    w.teleport_squad(from);
    let m = w.squad.members[0];
    w.order_members(&[m], to);
    let mut trail = vec![w.squad.at[0]];
    let mut still = 0;
    for _ in 0..4000 {
        w.step(0.25);
        let at = w.squad.at[0];
        still = if at.dist(*trail.last().unwrap()) < 1e-4 { still + 1 } else { 0 };
        trail.push(at);
        if still > 8 {
            break;
        }
    }
    (m, trail)
}

/// The first solid thing a trail passes through, if any (a walker's own
/// width isn't asked for here: only walking through the thing itself).
fn walked_through(w: &World, trail: &[V2]) -> Option<(Id, V2)> {
    trail.windows(2).find_map(|s| w.in_the_way(s[0], s[1], 0.0).map(|id| (id, s[1])))
}

fn nearest_place(w: &World, kind: impl Fn(PlaceKind) -> bool) -> (u16, u16) {
    let at = w.squad.pos;
    let mut best: Option<((u16, u16), f32)> = None;
    for (t, tl) in w.society.towns.iter().enumerate() {
        for (j, p) in tl.places.iter().enumerate() {
            if kind(p.kind) && w.solid(Id::Place(t as u16, j as u16, 0)).is_some() && best.is_none_or(|b| p.pos.dist(at) < b.1) {
                best = Some(((t as u16, j as u16), p.pos.dist(at)));
            }
        }
    }
    best.expect("there is one").0
}

fn body_of(w: &World, place: (u16, u16)) -> Thing {
    w.solid(Id::Place(place.0, place.1, 0)).unwrap()
}

#[test]
fn p21_nobody_walks_through_the_hall() {
    let mut w = worldgen::generate(1);
    // The town's hall (a workplace): from one side of it to the other.
    let hall = body_of(&w, nearest_place(&w, |k| k == PlaceKind::Hall));
    let c = hall.shape.centre();
    let wp_rot = match hall.shape {
        gahturiyu_sim::sim::solid::Shape::Box(r) => r.rot,
        _ => 0.0,
    };
    for turn in [0.0f32, 1.5708, 0.6] {
        let dir = unit(wp_rot + turn);
        let (from, to) = (w.free_spot(c.sub(dir.scale(11.0))), w.free_spot(c.add(dir.scale(11.0))));
        let (_, trail) = walk(&mut w, from, to);
        assert_eq!(walked_through(&w, &trail), None, "across the hall at a turn of {turn}");
        assert!(trail.last().unwrap().dist(to) < 0.5, "and gets to the far side ({} m short)", trail.last().unwrap().dist(to));
    }
    // A Qotiro hall (a building with a door): from beside its long wall to
    // beyond the other, where the old way-finder took it for no obstacle.
    let mut found = None;
    'find: for s in &w.settlements {
        for i in 0..s.buildings.len() as u16 {
            if let Some(d) = w.door((s.id, i)) {
                if d.variant().key == "qotiro_island_hall" {
                    found = Some(d);
                    break 'find;
                }
            }
        }
    }
    let d = found.expect("a Qotiro hall stands somewhere");
    let side = unit(d.rot + 1.5708);
    let (from, to) = (w.free_spot(d.centre.add(side.scale(d.half.y + 1.2))), w.free_spot(d.centre.sub(side.scale(d.half.y + 6.0))));
    let (_, trail) = walk(&mut w, from, to);
    assert_eq!(walked_through(&w, &trail), None, "past a Qotiro hall from beside its wall");
    assert!(trail.last().unwrap().dist(to) < 0.5);
}

#[test]
fn p21_nobody_walks_through_a_fence() {
    let mut w = worldgen::generate(1);
    // A workyard's fence: straight across it, and in to its middle.
    let yard = body_of(&w, nearest_place(&w, |k| k == PlaceKind::Workyard));
    let c = yard.shape.centre();
    let gate = yard.way().expect("a gate");
    // (Across the gate's own line, so the straight way is over the fence.)
    let through = gate.outside.sub(gate.inside);
    let side = V2::new(-through.y, through.x).scale(1.0 / through.len());
    let (from, to) = (w.free_spot(c.add(side.scale(13.0))), w.free_spot(c.sub(side.scale(13.0))));
    let (_, trail) = walk(&mut w, from, to);
    assert_eq!(walked_through(&w, &trail), None, "past a workyard");
    assert!(trail.last().unwrap().dist(to) < 0.5);
    let inside = w.free_spot(c.add(side.scale(3.0)));
    let (_, trail) = walk(&mut w, from, inside);
    assert_eq!(walked_through(&w, &trail), None, "into a workyard");
    assert!(trail.last().unwrap().dist(inside) < 0.5, "and gets in");
    assert!(trail.iter().any(|p| p.dist(gate.at) < 1.0), "by its gate");
    // An animal pen's fence.
    let pen = w.wild_solids().things.iter().filter(|t| matches!(t.id, Id::Pen(_))).min_by(|a, b| a.shape.centre().dist(c).total_cmp(&b.shape.centre().dist(c))).copied().expect("a pen");
    let pc = pen.shape.centre();
    let r = pen.shape.reach();
    let (from, to) = (w.free_spot(pc.sub(V2::new(r + 3.0, 0.3))), w.free_spot(pc.add(V2::new(0.0, r + 3.0))));
    let (_, trail) = walk(&mut w, from, to);
    assert_eq!(walked_through(&w, &trail), None, "round a pen");
    assert!(trail.last().unwrap().dist(to) < 0.5);
    let middle = w.free_spot(pc.sub(V2::new(0.0, 1.0)));
    let (_, trail) = walk(&mut w, from, middle);
    assert_eq!(walked_through(&w, &trail), None, "into a pen");
    assert!(trail.last().unwrap().dist(middle) < 0.5 && trail.iter().any(|p| p.dist(pen.way().unwrap().at) < 1.5), "by its gate");
}

#[test]
fn p22_a_walk_to_a_counter_ends_beside_it() {
    let mut w = worldgen::generate(1);
    // A shop's counter out front, under its awning.
    let shop = nearest_place(&w, |k| matches!(k, PlaceKind::Shop(_)));
    let counter = w.solid(Id::Place(shop.0, shop.1, 1)).expect("a shop has a counter");
    assert_eq!(counter.what, What::Counter);
    let c = counter.shape.centre();
    let from = w.free_spot(c.add(V2::new(14.0, 3.0)));
    let (m, trail) = walk(&mut w, from, c);
    let end = *trail.last().unwrap();
    assert_eq!(w.blocked_at(end, 0.0), None, "they don't stand in the counter");
    assert!(end.dist(c) < 2.2, "but beside it: {} m from its middle", end.dist(c));
    assert_eq!(walked_through(&w, &trail), None);
    // And they can walk away again.
    w.order_members(&[m], from);
    for _ in 0..400 {
        w.step(0.25);
    }
    assert!(w.squad.at[0].dist(from) < 0.5, "not stuck: {} m from where they were sent back to", w.squad.at[0].dist(from));
    // A trader's counter indoors: the same.
    let mut found: Option<Thing> = None;
    for s in &w.settlements {
        for t in w.town_solids(s.id).things.iter().filter(|t| t.what == What::Furniture(Furn::Counter)) {
            if found.is_none_or(|f| t.shape.centre().dist(w.squad.pos) < f.shape.centre().dist(w.squad.pos)) {
                found = Some(*t);
            }
        }
    }
    let counter = found.expect("a counter indoors");
    let door = w.door(counter.within.and_then(|i| i.door()).unwrap()).unwrap();
    let c = counter.shape.centre();
    let (m, trail) = walk(&mut w, door.outside.add(unit(door.rot).scale(3.0)), c);
    let end = *trail.last().unwrap();
    assert_eq!(walked_through(&w, &trail), None, "in by the door, round the furniture");
    assert_eq!(w.blocked_at(end, 0.0), None, "they don't stand in the counter");
    assert!(end.dist(c) < 1.9 && w.building_at(end).map(|d| d.id) == Some(door.id), "but beside it, in the shop: {} m", end.dist(c));
    let out = door.outside.add(unit(door.rot).scale(3.0));
    w.order_members(&[m], out);
    for _ in 0..400 {
        w.step(0.25);
    }
    assert!(w.squad.at[0].dist(out) < 0.5, "and out again");
    let _ = BODY;
}

#[test]
fn a_spot_boxed_in_indoors_is_swapped_for_one_the_way_in_gets_to() {
    use gahturiyu_sim::sim::{solid::enclosure, walk::Floor};
    let w = worldgen::generate(1);
    // Spots on a building's floor that a walker can stand on but no way in
    // by the door reaches: between two beds whose feet are against a wall
    // and whose heads are against a chest, say.
    let mut boxed = 0;
    'towns: for s in &w.settlements {
        for i in 0..s.buildings.len() as u16 {
            let Some(d) = w.door((s.id, i)) else { continue };
            let sets = w.solid_sets(d.centre, d.radius + 10.0);
            let Some(th) = enclosure(&sets, d.centre) else { continue };
            let Some(floor) = Floor::of(&sets, &th, BODY) else { continue };
            for gx in -8..=8 {
                for gy in -8..=8 {
                    let p = d.to_world((gx as f32 / 9.0, gy as f32 / 9.0));
                    if !w.can_stand(p, BODY) || floor.gets_to(p) || enclosure(&sets, p).map(|t| t.id) != Some(th.id) {
                        continue;
                    }
                    boxed += 1;
                    let q = w.free_spot(p);
                    // (Out of a room with no doorway, the nearest spot may be beyond its wall.)
                    assert!(q.dist(p) > 1e-3 && q.dist(p) < 6.5, "{:?}: a boxed-in spot is swapped for one nearby ({} m)", th.id, q.dist(p));
                    assert!(w.can_stand(q, BODY) && floor.gets_to(q) && w.building_at(q).map(|b| b.id) == Some(d.id), "{:?}: one the way in gets to, in the same building", th.id);
                    let f = w.find_way(d.outside.add(unit(d.rot).scale(3.0)), p);
                    assert!(f.reached && f.way.last().is_some_and(|e| e.dist(q) < 1e-3), "{:?}: a walk there gets to it", th.id);
                    if boxed >= 40 {
                        break 'towns;
                    }
                }
            }
        }
    }
    // (World 1's longhouses and Qotiro halls have such spots today. If the
    // layouts change so that none do, this has nothing left to check: drop
    // this line.)
    assert!(boxed > 0, "no boxed-in spot found to check");
}

#[test]
fn the_finder_gets_everywhere_on_a_floor_that_the_floor_joins_to_the_door() {
    use gahturiyu_sim::sim::{solid::enclosure, walk::Floor};
    let w = worldgen::generate(1);
    // A few buildings of every kind: flood the floor from the door, 10 cm
    // at a time (coarser in the biggest halls), over spots a walker can
    // stand on; the finder must get to (nearly) every spot the flood does.
    // (A flood can squeeze through a gap a hair too narrow for the finder's
    // corners now and then.)
    let mut per_kind: std::collections::BTreeMap<&str, u32> = Default::default();
    let (mut worst, mut all, mut all_missed) = ((0.0f32, ""), 0u32, 0u32);
    for s in &w.settlements {
        for i in 0..s.buildings.len() as u16 {
            let Some(d) = w.door((s.id, i)) else { continue };
            let seen_kind = per_kind.entry(d.variant().key).or_default();
            if *seen_kind >= 3 {
                continue;
            }
            *seen_kind += 1;
            let sets = w.solid_sets(d.centre, d.radius + 10.0);
            let Some(th) = enclosure(&sets, d.centre) else { continue };
            let Some(floor) = Floor::of(&sets, &th, BODY) else { continue };
            let (dir, side) = (unit(d.rot), unit(d.rot + 1.5708));
            let step = (d.half.x.max(d.half.y) / 120.0).max(0.1);
            let (nx, ny) = ((d.half.x / step) as i32 + 1, (d.half.y / step) as i32 + 1);
            let wd = 2 * nx + 1;
            let at = |k: i32, j: i32| d.centre.add(dir.scale(k as f32 * step)).add(side.scale(j as f32 * step));
            let ix = |k: i32, j: i32| ((j + ny) * wd + k + nx) as usize;
            let mut ok = vec![false; (wd * (2 * ny + 1)) as usize];
            for j in -ny..=ny {
                for k in -nx..=nx {
                    let p = at(k, j);
                    ok[ix(k, j)] = enclosure(&sets, p).map(|t| t.id) == Some(th.id) && w.can_stand(p, BODY);
                }
            }
            let door = th.way().unwrap().inside;
            let start = (-ny..=ny).flat_map(|j| (-nx..=nx).map(move |k| (k, j))).filter(|&(k, j)| ok[ix(k, j)]).min_by(|a, b| at(a.0, a.1).dist(door).total_cmp(&at(b.0, b.1).dist(door)));
            let Some(start) = start else { continue };
            let mut seen = vec![false; ok.len()];
            seen[ix(start.0, start.1)] = true;
            let mut todo = vec![start];
            let (mut flooded, mut missed) = (0u32, 0u32);
            while let Some((k, j)) = todo.pop() {
                flooded += 1;
                if !floor.gets_to(at(k, j)) {
                    missed += 1;
                }
                for (a, b) in [(k + 1, j), (k - 1, j), (k, j + 1), (k, j - 1)] {
                    if a.abs() <= nx && b.abs() <= ny && ok[ix(a, b)] && !seen[ix(a, b)] {
                        seen[ix(a, b)] = true;
                        todo.push((a, b));
                    }
                }
            }
            let share = missed as f32 / flooded.max(1) as f32;
            (all, all_missed) = (all + flooded, all_missed + missed);
            if share > worst.0 {
                worst = (share, d.variant().key);
            }
        }
    }
    println!("{} buildings: {all} spots joined to their doors, the finder misses {all_missed}", per_kind.values().sum::<u32>());
    assert!(worst.0 < 0.02, "the finder misses {:.1}% of a {}'s floor that is joined to its door", worst.0 * 100.0, worst.1);
}

/// Spots to walk between in a town: anywhere in it, inside its buildings,
/// at its workplaces (their very middles: often inside something).
fn spots(w: &World, town: usize, r: &mut gahturiyu_sim::sim::rng::Rng, n: usize) -> Vec<V2> {
    let s = &w.settlements[town];
    let mut out = Vec::new();
    let doors: Vec<_> = (0..s.buildings.len() as u16).filter_map(|i| w.door((s.id, i))).collect();
    let places = &w.society.towns[town].places;
    while out.len() < n {
        let p = match r.below(4) {
            0 if !doors.is_empty() => {
                let d = &doors[r.below(doors.len())];
                let a = r.f32() * std::f32::consts::TAU;
                d.to_world((a.cos() * 0.75 * r.f32(), a.sin() * 0.75 * r.f32()))
            }
            1 if !places.is_empty() => {
                let p = &places[r.below(places.len())];
                p.pos.add(unit(r.f32() * 6.28).scale(r.f32() * p.kind.size() * 0.5))
            }
            _ => s.pos.add(unit(r.f32() * 6.28).scale(r.f32() * s.reach.min(260.0))),
        };
        if !w.terrain.is_sea(p) && !w.open_water(p) {
            out.push(p);
        }
    }
    out
}

#[test]
fn no_walk_in_any_town_passes_through_anything() {
    use gahturiyu_sim::sim::{rng::Rng, walk};
    for seed in 1..=2u64 {
        let w = worldgen::generate(seed);
        let mut r = Rng::from_keys(&[seed, 0x5741_4C4B]);
        let (mut walks, mut short, mut turns, mut secs) = (0, Vec::new(), 0usize, 0.0f64);
        for town in 0..w.settlements.len() {
            let pts = spots(&w, town, &mut r, 60);
            for pair in pts.chunks(2) {
                let (a, b) = (pair[0], pair[1]);
                let t0 = std::time::Instant::now();
                let f = w.find_way(a, b);
                secs += t0.elapsed().as_secs_f64();
                walks += 1;
                turns += f.way.len();
                let sets = w.solid_sets(a.lerp(b, 0.5), a.dist(b) * 0.5 + 60.0);
                let (from, to) = (w.free_spot(a), w.free_spot(b));
                assert!(w.can_stand(from, BODY) && w.can_stand(to, BODY), "seed {seed}, town {town}: the ends {a:?} and {b:?} have somewhere to stand");
                assert!(from.dist(a) < 6.5 && to.dist(b) < 6.5, "seed {seed}, town {town}: and it's beside them ({} and {} m)", from.dist(a), to.dist(b));
                // (The first step may be out of whatever `a` is inside.)
                let way: &[V2] = if from.dist(a) > 1e-3 { &f.way[1..] } else { &f.way };
                assert_eq!(walk::runs_into(&sets, from, way, 0.1), None, "seed {seed}, town {town}: from {a:?} to {b:?} by {:?}", f.way);
                for p in way {
                    assert!(w.can_stand(*p, BODY - 0.01), "seed {seed}, town {town}: a walker can stand at every turn ({p:?})");
                }
                if f.reached {
                    assert!(way.last().map(|e| e.dist(to) < 1e-3).unwrap_or(from.dist(to) < 1e-3), "seed {seed}, town {town}: and it ends where it was going");
                } else {
                    short.push((town, a, b));
                }
                // Asked again, the same way.
                assert_eq!(*w.find_way(a, b), *f);
            }
        }
        println!("seed {seed}: {walks} walks, {:.1} turns each, {:.0} µs each; {} found no way", turns as f32 / walks as f32, secs * 1e6 / walks as f64, short.len());
        for s in short.iter().take(10) {
            println!("   no way in town {}: {:?} -> {:?}", s.0, s.1, s.2);
        }
        assert!(short.is_empty(), "seed {seed}: {} of {walks} walks found no way", short.len());
    }
}

#[test]
fn townsfolk_stand_where_a_walker_can_stand() {
    use gahturiyu_sim::sim::{
        routine::Spot,
        world::{DAY, HOUR},
    };
    let mut w = worldgen::generate(1);
    // At night, at work, about the town, in the evening: whoever of three
    // towns is at a spot of their day (not on a round that moves along, not
    // out on the water) stands clear of everything, and drifts only as far
    // as there's room.
    let (mut looked, mut bad) = (0, Vec::new());
    for h in [3.0, 10.0, 13.5, 19.0] {
        while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
            w.step(60.0);
        }
        let (day, hour) = (World::day_of(w.time), (w.time.rem_euclid(DAY) / HOUR) as f32);
        for town in 0..3 {
            for &pid in &w.settlements[town].residents {
                if w.people[pid as usize].dead || w.people[pid as usize].in_squad {
                    continue;
                }
                let plan = w.day_plan(pid, day);
                let seg = plan.segs[plan.index_at(hour)];
                if matches!(seg.spot, Spot::Rounds | Spot::Watch | Spot::Boat | Spot::RunRound) {
                    continue;
                }
                let p = w.person_pos(pid);
                if w.terrain.is_sea(p) || w.open_water(p) {
                    continue;
                }
                looked += 1;
                if !w.can_stand(p, BODY - 0.02) {
                    bad.push((pid, h, seg.spot, w.blocked_at(p, BODY - 0.02)));
                }
            }
        }
    }
    println!("{looked} townsfolk looked at, {} where nobody could stand", bad.len());
    for b in bad.iter().take(10) {
        println!("   {b:?}");
    }
    assert!(looked > 300 && bad.is_empty(), "{} of {looked} townsfolk stand where nobody could", bad.len());
}
