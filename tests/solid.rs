//! What is solid (`sim/solid.rs`): every physical thing is listed with its
//! real outline and an id that lasts, nothing overlaps that shouldn't, every
//! door and gate can be walked through, and the questions the walking, the
//! clicking and the spells ask all get the same answers.

use std::collections::{BTreeMap, BTreeSet};

use gahturiyu_sim::sim::{
    base::{self, Built, State, BUILDINGS},
    geo::{self, V2},
    jobs::PlaceKind,
    layout::{self, Furn, Rect, VARIANTS},
    settlement::{Building, BuildingKind},
    society::Workplace,
    solid::{self, Id, Part, Poly, Shape, Solids, Thing, What, BODY},
    torch::StandingTorch,
    world::HOUR,
    worldgen, World,
};

fn unit(a: f32) -> V2 {
    V2::new(a.cos(), a.sin())
}

/// Every list of a world: the wild's, then each town's.
fn sets(w: &World) -> Vec<std::sync::Arc<Solids>> {
    let mut v = vec![w.wild_solids()];
    v.extend(w.settlements.iter().map(|s| w.town_solids(s.id)));
    v
}

/// Is this thing's own wall or body what a straight line from `a` to `b`
/// runs into (for a walker `body` wide)?
fn stops(w: &World, id: Id, a: V2, b: V2, body: f32) -> bool {
    sets(w).iter().any(|set| set.pieces.iter().any(|p| p.part != Part::Hull && set.things[p.thing as usize].id == id && p.poly.crossed(a, b, body)))
}

fn all_things(w: &World) -> Vec<Thing> {
    sets(w).iter().flat_map(|s| s.things.clone()).collect()
}

/// A world with one building of every variant that has a door added to its
/// first town, far out of the way (as the window's showcase does), and
/// which building each is.
fn with_every_variant() -> (World, Vec<(u16, &'static layout::Variant)>) {
    let mut w = worldgen::generate(1);
    let at = w.settlements[0].pos.add(V2::new(0.0, 600.0));
    let mut made = Vec::new();
    for (k, v) in VARIANTS.iter().filter(|v| v.kind != BuildingKind::HoraroStilt).enumerate() {
        let (size, seed) = layout::example_of(v);
        let i = w.settlements[0].buildings.len() as u16;
        w.settlements[0].buildings.push(Building { pos: at.add(V2::new(k as f32 * 70.0, 0.0)), kind: v.kind, size, rot: 0.7 + k as f32, seed });
        made.push((i, v));
    }
    w.settlements[0].reach += 600.0 + made.len() as f32 * 70.0;
    w.solid_changed();
    (w, made)
}

#[test]
fn every_building_variant_and_everything_in_it_is_listed() {
    let (w, made) = with_every_variant();
    let set = w.town_solids(0);
    for (i, v) in made {
        let d = w.door((0, i)).expect("it has a door");
        assert_eq!(d.variant().key, v.key, "the example is that variant");
        let th = w.solid(Id::Building(0, i)).unwrap_or_else(|| panic!("{}: its walls are listed", v.key));
        assert!(th.blocks && th.hollow.is_some() && th.way().is_some(), "{}: a hollow wall with a doorway", v.key);
        // Its outline is the building's own, not a circle round it.
        for k in 0..400 {
            let p = d.centre.add(V2::new((k % 20) as f32 - 9.5, (k / 20) as f32 - 9.5).scale(d.radius / 9.0));
            assert_eq!(th.shape.contains(p), d.contains(p), "{}: the outline at {p:?}", v.key);
        }
        assert_eq!(matches!(th.shape, Shape::Round { .. }), v.shape == layout::Shape::Round, "{}", v.key);
        let door = w.solid(Id::Door(0, i)).unwrap_or_else(|| panic!("{}: its door is listed", v.key));
        assert!(!door.blocks && door.within == Some(th.id) && door.shape.centre().dist(th.way().unwrap().at) < 0.3, "{}", v.key);
        // Every inner wall, piece of furniture and container, where the floor plan has it.
        for (what, r) in layout::footprints(&d) {
            let id = match what {
                layout::Thing::Furniture(k) => Id::Furniture(0, i, k as u8),
                layout::Thing::Container(k) => Id::Container(0, i, k as u8),
                layout::Thing::Wall(k) => Id::InnerWall(0, i, k as u8),
                layout::Thing::Extra(_) => continue,
            };
            let t = w.solid(id).unwrap_or_else(|| panic!("{}: {id:?} is listed", v.key));
            // (An inner wall of a round building stops at the outer wall.)
            match (what, t.shape) {
                (layout::Thing::Wall(_), Shape::Box(got)) if d.round => {
                    assert!(got.rot == r.rot && got.hd == r.hd && got.hw <= r.hw + 1e-3 && got.hw > 0.3, "{}: {id:?} is its wall, cut to the outline", v.key);
                    let [u, _] = got.axes();
                    for end in [got.c.sub(u.scale(got.hw)), got.c.add(u.scale(got.hw))] {
                        assert!(th.shape.grown(0.05).contains(end), "{}: {id:?} ends inside the outer wall", v.key);
                    }
                }
                _ => assert_eq!(t.shape, Shape::Box(r), "{}: {id:?} has its footprint", v.key),
            }
            assert_eq!(t.within, Some(th.id));
            let flat = matches!(t.what, What::Furniture(Furn::Bedroll | Furn::Screen));
            assert_eq!(t.blocks, !flat, "{}: {id:?}", v.key);
        }
        // The outer wall is solid all the way round, except at the door.
        let way = th.way().unwrap();
        let walls: Vec<&solid::Piece> = set.pieces.iter().filter(|p| p.part == Part::Wall && set.things[p.thing as usize].id == th.id).collect();
        assert!(walls.len() >= 4, "{}: {} stretches of wall", v.key, walls.len());
        let mut shut = 0;
        for k in 0..72 {
            let dir = unit(k as f32 / 72.0 * std::f32::consts::TAU);
            let (a, b) = (d.centre.add(dir.scale(d.radius + 1.0)), d.centre.add(dir.scale(0.05)));
            // (Not a line that goes in by the doorway.)
            if layout::on_seg(way.at, a, b).dist(way.at) < way.width * 0.5 + 0.5 {
                continue;
            }
            shut += 1;
            assert!(walls.iter().any(|p| p.poly.crossed(a, b, 0.0)), "{}: the wall stops a line in from {dir:?}", v.key);
        }
        assert!(shut >= 60, "{}: the wall was tried all round ({shut})", v.key);
        // And the doorway is open, for a walker's width.
        assert!(!walls.iter().any(|p| p.poly.crossed(way.outside, way.inside, BODY)), "{}: the doorway is clear", v.key);
        assert_eq!(w.enclosure_at(d.centre).map(|t| t.id), Some(th.id), "{}: standing in it", v.key);
        assert_eq!(w.enclosure_at(way.outside).map(|t| t.id), None, "{}: standing outside it", v.key);
    }
    // The hearth's ring of stones, and the stilt homes (which stop nobody).
    let hearth = w.solid(Id::Building(0, 0)).expect("the hearth");
    assert!(hearth.what == What::Fire && hearth.blocks && hearth.hollow.is_none());
    let (mut stilts, mut seen) = (0, 0);
    for s in &w.settlements {
        for (i, b) in s.buildings.iter().enumerate() {
            if b.kind == BuildingKind::HoraroStilt {
                stilts += 1;
                let t = w.solid(Id::Building(s.id, i as u16)).expect("a stilt home is listed");
                assert!(!t.blocks && t.shape.contains(b.pos));
                seen += 1;
            }
        }
    }
    assert!(stilts > 0 && stilts == seen);
}

#[test]
fn every_building_of_every_town_is_listed() {
    for seed in 1..=3u64 {
        let w = worldgen::generate(seed);
        let mut variants = BTreeSet::new();
        for s in &w.settlements {
            let set = w.town_solids(s.id);
            let ids: BTreeSet<Id> = set.things.iter().map(|t| t.id).collect();
            for i in 0..s.buildings.len() as u16 {
                assert!(ids.contains(&Id::Building(s.id, i)), "seed {seed}: {}'s building {i}", s.name);
                if let Some(d) = w.door((s.id, i)) {
                    variants.insert(d.variant().key);
                    assert!(ids.contains(&Id::Door(s.id, i)));
                    let n = layout::footprints(&d).len();
                    let inside = set.things.iter().filter(|t| t.within == Some(Id::Building(s.id, i)) && !matches!(t.id, Id::Door(..))).count();
                    assert_eq!(inside, n, "seed {seed}: {}'s building {i} has everything in it", s.name);
                    // And none of it reaches out through the walls.
                    let shell = set.things.iter().find(|t| t.id == Id::Building(s.id, i)).unwrap().shape.grown(0.15);
                    for t in set.things.iter().filter(|t| t.within == Some(Id::Building(s.id, i)) && !matches!(t.id, Id::Door(..))) {
                        assert!(t.shape.outline().corners().iter().all(|&c| shell.contains(c)), "seed {seed}: {:?} is inside its walls", t.id);
                    }
                }
            }
        }
        assert!(variants.len() >= 12, "seed {seed}: {} variants stand in its towns", variants.len());
    }
}

/// One of every kind of workplace, on dry land.
fn every_place_kind() -> Vec<PlaceKind> {
    use gahturiyu_sim::sim::jobs::Shelf;
    use PlaceKind as P;
    vec![
        P::Fields, P::Wilds, P::Woodlot, P::Dock, P::DivePlatform, P::KelpBeds, P::Boatyard, P::Kitchen, P::MessHall, P::Deck, P::Market, P::Shop(Shelf::Food), P::Shop(Shelf::Materials),
        P::Shop(Shelf::Goods), P::Inn, P::Shrine, P::GuardPost, P::Hall, P::HealingHouse, P::HealersHouse, P::TeachingHouse, P::ExchangeHouse, P::LettersHouse, P::TendersYard, P::Workyard, P::Forge,
        P::Bench, P::Desk, P::AlchemyTable, P::WeaversShed, P::Workshop, P::Quarry, P::CharcoalPit,
    ]
}

#[test]
fn every_kind_of_workplace_has_its_solid_parts() {
    use PlaceKind as P;
    // (Somewhere well inland on the map's plains.)
    let atlas = gahturiyu_sim::sim::atlas::atlas();
    let at = (0..50_000u64).map(|k| atlas.land_spot(k * 7919 + 3)).find(|&p| atlas.inland(p) > 2000.0 && atlas.class_at(p) == gahturiyu_sim::sim::atlas::Class::Plains).expect("plains inland");
    assert!(geo::inland(at) > 0.0);
    for kind in every_place_kind() {
        let wp = Workplace { kind, pos: at, rot: 0.9, seed: 77 };
        let parts = solid::place_parts(&wp);
        let nothing = matches!(kind, P::Fields | P::KelpBeds | P::Dock | P::DivePlatform);
        assert_eq!(parts.is_empty(), nothing, "{kind:?}");
        for (_, shape, h, _) in &parts {
            assert!(shape.centre().dist(at) <= kind.size() * 0.75 && shape.reach() > 0.2 && *h > 0.0, "{kind:?}: a part stands within its place");
        }
        // The roofed ones are the box the window draws.
        let body = |w: f32, d: f32| assert_eq!(parts[0].1, Shape::Box(Rect::new(at, 0.9, w, d)), "{kind:?}");
        let size = kind.size();
        match kind {
            P::Hall | P::MessHall => body(size, size * 0.6),
            P::Inn => body(size, size * 0.75),
            P::Shop(_) => {
                body(size * 0.8, size * 0.6);
                assert!(parts.len() == 2 && parts[1].0 == What::Counter, "a shop has its counter out front");
            }
            P::HealingHouse | P::HealersHouse | P::TeachingHouse | P::ExchangeHouse | P::LettersHouse => body(size, size * 0.7),
            P::Forge | P::Bench | P::Desk | P::AlchemyTable | P::WeaversShed | P::Workshop => body(size * 0.8, size * 0.7),
            P::Market => assert!(parts.len() == 5 && parts.iter().all(|p| p.0 == What::Counter)),
            P::Workyard => assert!(parts[0].0 == What::Fence && parts[0].1 == Shape::Box(Rect::new(at, 0.9, size, size))),
            _ => {}
        }
        // Out on the water there's nothing to walk into.
        let wet = Workplace { pos: V2::new(geo::coast_x(4000.0) - 80.0, 4000.0), ..wp.clone() };
        assert!(solid::place_parts(&wet).is_empty(), "{kind:?} on the water");
        // Its crafting stations stand inside its footprint.
        for (p, _) in solid::station_spots(&wp) {
            assert!(p.dist(at) <= size * 0.5 - 1.5 || kind.stations().len() == 1, "{kind:?}: a station at {:?}", p.dist(at));
        }
    }
    // In the towns as they are: every workplace on land that has parts is listed, part by part.
    let w = worldgen::generate(2);
    let mut kinds = BTreeSet::new();
    for (t, tl) in w.society.towns.iter().enumerate() {
        let set = w.town_solids(t as u16);
        for (j, wp) in tl.places.iter().enumerate() {
            // (A place inside a building has none: the building is it.)
            if solid::place_in_building(&w.settlements[t], wp) {
                continue;
            }
            let parts = solid::place_parts(wp);
            for (k, (what, shape, _, blocks)) in parts.iter().enumerate() {
                let th = set.things.iter().find(|x| x.id == Id::Place(t as u16, j as u16, k as u8)).unwrap_or_else(|| panic!("{:?} part {k}", wp.kind));
                assert!(th.what == *what && th.shape == *shape && th.blocks == *blocks);
            }
            if !parts.is_empty() {
                kinds.insert(format!("{:?}", wp.kind));
            }
            if wp.kind == P::Workyard {
                let yard = w.solid(Id::Place(t as u16, j as u16, 0)).unwrap();
                assert!(yard.hollow.is_some() && yard.way().is_some(), "a workyard's fence has a gate");
                assert!(w.solid(Id::PlaceGate(t as u16, j as u16)).is_some_and(|g| !g.blocks));
                // Its stations stand in the open, inside the fence.
                for k in 0..wp.kind.stations().len() {
                    let st = w.solid(Id::Station(t as u16, j as u16, k as u8)).expect("a yard's station is listed");
                    assert!(st.blocks && st.within == Some(yard.id) && yard.shape.contains(st.shape.centre()));
                    assert!(w.stations.iter().any(|s| s.0 == st.shape.centre()), "where the crafting has it");
                }
            }
        }
    }
    assert!(kinds.len() >= 18, "{} kinds of workplace stand in world 2: {kinds:?}", kinds.len());
}

#[test]
fn every_base_building_and_wall_piece_is_listed() {
    for (k, def) in BUILDINGS.iter().enumerate() {
        let bl = Built { id: 7, def: k as u16, at: V2::new(100.0, 200.0), rot: 0.4, w: def.w, state: State::Standing { hp: def.hp, since: 0.0, hp_at: 0.0, rate: 0.0 }, placed: 0.0, sealed: false };
        let th = solid::built_thing(3, &bl).unwrap_or_else(|| panic!("{} is listed", def.key));
        assert_eq!(th.id, Id::Base(3, 7));
        assert!(th.shape.contains(bl.at) && th.height == def.h, "{}", def.key);
        match def.kind {
            base::Kind::Marker => assert!(th.what == What::Fire && th.blocks),
            base::Kind::Gate => assert!(th.what == What::Gate && !th.blocks, "a gate lets you through"),
            base::Kind::Wall => assert!(th.what == What::Wall && th.blocks && th.hollow.is_none()),
            base::Kind::Field | base::Kind::Pen => {
                let way = th.way().expect("a fenced plot has a gate");
                assert!(th.blocks && !th.walls().iter().any(|r| Poly::new(&r.corners()).crossed(way.outside, way.inside, BODY)), "{}", def.key);
            }
            _ => assert!(th.blocks && th.hollow.is_none(), "{}", def.key),
        }
        // Its outline is its footprint: `w` across its facing, its depth along it.
        if def.kind != base::Kind::Marker {
            let corners = bl.corners();
            let poly = th.shape.outline();
            for c in corners {
                assert!(poly.depth(c).abs() < 0.01, "{}: a corner of its footprint is a corner of its outline", def.key);
            }
        }
        // A site is in the way as much as the finished thing; a ruin isn't.
        assert!(solid::built_thing(3, &Built { state: State::Ruin, ..bl.clone() }).is_none(), "{}: rubble is walked over", def.key);
    }
    // In a world: a base's marker and a palisade with a gate in it.
    let mut w = worldgen::generate(1);
    let start = w.squad.pos;
    let mut found = None;
    'find: for ring in 0..40 {
        for k in 0..12 {
            let p = start.add(unit(k as f32 / 12.0 * std::f32::consts::TAU).scale(60.0 + ring as f32 * 25.0));
            let mut t = w.clone();
            let Ok(bid) = t.found_base(p) else { continue };
            let wall = World::wall_plans(base::def_index("palisade"), &[p.add(V2::new(-12.0, 18.0)), p.add(V2::new(12.0, 18.0))]);
            if !wall.into_iter().all(|mut q| t.check_place(&mut q).is_ok()) {
                continue;
            }
            found = Some((p, bid));
            break 'find;
        }
    }
    let (p, _) = found.expect("room for a base");
    let before = w.wild_solids().things.len();
    let bid = w.found_base(p).unwrap();
    let ids = w.place_wall(base::def_index("palisade"), &[p.add(V2::new(-12.0, 18.0)), p.add(V2::new(12.0, 18.0))]).expect("a wall");
    assert!(ids.len() >= 6);
    // (The list follows the world: nothing was told it had changed.)
    assert_eq!(w.wild_solids().things.len(), before + 1 + ids.len());
    assert!(w.solid(Id::Base(bid, 0)).is_some_and(|t| t.what == What::Fire));
    let (a, b) = (p.add(V2::new(0.3, 14.0)), p.add(V2::new(0.3, 22.0)));
    assert!(matches!(w.in_the_way(a, b, BODY), Some(Id::Base(..))), "the palisade stops a walk north");
    let mid = ids[ids.len() / 2];
    let piece = w.base(bid).unwrap().buildings.iter().find(|b| b.id == mid).unwrap().clone();
    let gate = w.place_building(base::Plan { def: base::def_index("gate"), at: piece.at, rot: piece.rot, w: piece.w, replaces: Some(mid) }).expect("a gate in the wall");
    let g = w.solid(Id::Base(bid, gate)).expect("the gate is listed");
    assert!(!g.blocks);
    let through = (g.shape.centre().add(V2::new(0.0, -4.0)), g.shape.centre().add(V2::new(0.0, 4.0)));
    assert!(w.way_clear(through.0, through.1, BODY), "and the way through it is open");
}

/// Who a piece belongs to, for telling what may touch what.
fn owner(id: Id) -> String {
    match id {
        Id::Building(t, i) | Id::Door(t, i) | Id::InnerWall(t, i, _) | Id::Furniture(t, i, _) => format!("building {t} {i}"),
        Id::Container(t, i, _) if t != gahturiyu_sim::sim::containers::WILD => format!("building {t} {i}"),
        Id::Place(t, j, _) | Id::PlaceGate(t, j) | Id::Station(t, j, _) => format!("place {t} {j}"),
        Id::Ruin(r, _) | Id::Lair(r) => format!("ruin {r}"),
        Id::Pen(p) | Id::PenGate(p) | Id::PenHut(p) => format!("pen {p}"),
        Id::Base(b, _) => format!("base {b}"),
        other => format!("{other:?}"),
    }
}

#[test]
fn nothing_overlaps_that_shouldnt() {
    for seed in 1..=3u64 {
        let w = worldgen::generate(seed);
        let all = sets(&w);
        let mut faults: Vec<String> = Vec::new();
        let mut ids: BTreeSet<Id> = BTreeSet::new();
        for (n, set) in all.iter().enumerate() {
            for t in &set.things {
                assert!(ids.insert(t.id), "seed {seed}: {:?} is listed once", t.id);
            }
            // Out of doors: two things of different owners never stand in each other.
            let out: Vec<&solid::Piece> = set.pieces.iter().filter(|p| p.part != Part::Wall && p.room.is_none()).collect();
            let near = |a: &Poly, b: &Poly| a.centre().dist(b.centre()) < 80.0;
            for (i, a) in out.iter().enumerate() {
                let ta = &set.things[a.thing as usize];
                for b in out[i + 1..].iter().filter(|b| near(&a.poly, &b.poly)) {
                    let tb = &set.things[b.thing as usize];
                    if owner(ta.id) != owner(tb.id) && a.poly.overlaps(&b.poly, 0.02) {
                        faults.push(format!("{:?} in {:?}", ta.id, tb.id));
                    }
                }
                // And a town's things don't stand in the wild's (pens, camps).
                if n > 0 {
                    let c = a.poly.centre();
                    for b in all[0].pieces_in(V2::new(c.x - 40.0, c.y - 40.0), V2::new(c.x + 40.0, c.y + 40.0), 0.0) {
                        if b.part != Part::Wall && b.room.is_none() && a.poly.overlaps(&b.poly, 0.02) {
                            faults.push(format!("{:?} in {:?}", ta.id, all[0].things[b.thing as usize].id));
                        }
                    }
                }
            }
            // Indoors: everything stands inside the walls it's listed within,
            // and nothing solid stands in anything else there. (Inner walls
            // run into the outer wall and meet each other: that's a wall.)
            let mut rooms: BTreeMap<Id, Vec<&solid::Piece>> = BTreeMap::new();
            for p in set.pieces.iter().filter(|p| p.part == Part::Body) {
                if let Some(r) = p.room {
                    rooms.entry(r).or_default().push(p);
                }
            }
            for (room, inside) in rooms {
                let shell = set.things.iter().find(|t| t.id == room).expect("the room is listed");
                for (i, a) in inside.iter().enumerate() {
                    let ta = &set.things[a.thing as usize];
                    let wall = |t: &Thing| t.what == What::Wall;
                    if !wall(ta) && !a.poly.corners().iter().all(|&c| shell.shape.grown(0.05).contains(c)) {
                        faults.push(format!("{:?} sticks out of {room:?}", ta.id));
                    }
                    for b in &inside[i + 1..] {
                        let tb = &set.things[b.thing as usize];
                        if !(wall(ta) && wall(tb)) && a.poly.overlaps(&b.poly, 0.02) {
                            faults.push(format!("{:?} in {:?}", ta.id, tb.id));
                        }
                    }
                }
            }
        }
        assert!(faults.is_empty(), "seed {seed}: {} things overlap, e.g. {:?}", faults.len(), &faults[..faults.len().min(12)]);
    }
}

#[test]
fn every_door_and_gate_can_be_walked_through_from_outside() {
    for seed in 1..=3u64 {
        let w = worldgen::generate(seed);
        let (mut doors, mut gates) = (0, 0);
        for set in sets(&w) {
            for th in set.things.iter().filter(|t| t.hollow.is_some()) {
                let way = th.way().unwrap_or_else(|| panic!("seed {seed}: {:?} has a way in", th.id));
                match th.id {
                    Id::Building(..) => doors += 1,
                    _ => gates += 1,
                }
                assert!(th.shape.contains(way.inside) && !th.shape.contains(way.outside), "seed {seed}: {:?}: inside is in, outside is out", th.id);
                assert_eq!(w.blocked_at(way.outside, BODY), None, "seed {seed}: {:?}: room to stand outside", th.id);
                assert_eq!(w.blocked_at(way.inside, BODY), None, "seed {seed}: {:?}: room to stand inside", th.id);
                assert_eq!(w.in_the_way(way.outside, way.inside, BODY), None, "seed {seed}: {:?}: the way through is clear", th.id);
                // Not boxed in: from outside there's somewhere to go.
                let away = (0..16).any(|k| w.way_clear(way.outside, way.outside.add(unit(k as f32 / 16.0 * std::f32::consts::TAU).scale(4.0)), BODY));
                assert!(away, "seed {seed}: {:?}: open ground beyond its door", th.id);
                // And the wall is shut beside the opening.
                let through = way.outside.sub(way.inside);
                let side = V2::new(-through.y, through.x).scale((way.width * 0.5 + 0.6) / through.len());
                for s in [side, side.scale(-1.0)] {
                    assert!(stops(&w, th.id, way.outside.add(s), way.inside.add(s), 0.0), "seed {seed}: {:?}: wall beside the opening", th.id);
                }
            }
        }
        assert!(doors > 300 && gates > 20, "seed {seed}: {doors} doors, {gates} gates");
    }
}

#[test]
fn asking_by_place_finds_the_thing() {
    let w = worldgen::generate(1);
    let (mut counters, mut halls, mut pens) = (0, 0, 0);
    for set in sets(&w) {
        for th in &set.things {
            let c = th.shape.centre();
            // By id, as it stands.
            assert_eq!(w.solid(th.id), Some(*th), "{:?} by its id", th.id);
            // Near it and at it.
            assert!(w.solids_near(c, 0.5).iter().any(|t| t.id == th.id), "{:?} near its middle", th.id);
            assert!(!w.solids_near(c.add(V2::new(th.shape.reach() + 3.0, 0.0)), 1.0).iter().any(|t| t.id == th.id), "{:?} isn't near a spot 3 m off it", th.id);
            match (th.what, th.id) {
                // A shop's counter indoors: the thing at it is the counter, not the house.
                (What::Furniture(Furn::Counter), _) => {
                    counters += 1;
                    assert_eq!(w.solid_at(c).map(|t| t.id), Some(th.id), "the counter, not the house round it");
                    assert_eq!(w.blocked_at(c, BODY), Some(th.id));
                    let beside = w.free_spot(c);
                    assert!(beside.dist(c) > 0.3 && beside.dist(c) < 1.6, "beside the counter: {} m from its middle", beside.dist(c));
                    assert_eq!(w.blocked_at(beside, BODY), None, "and free to stand at");
                    assert_eq!(w.enclosure_at(beside).map(|t| t.id), th.within, "still in the shop");
                }
                // The town's hall: a walk can't go through it, and a spot in it is moved beside it.
                (What::Shelter, Id::Place(t, j, 0)) if w.society.towns[t as usize].places[j as usize].kind == PlaceKind::Hall => {
                    halls += 1;
                    let wp = &w.society.towns[t as usize].places[j as usize];
                    let along = unit(wp.rot + 1.2);
                    let (a, b) = (c.sub(along.scale(12.0)), c.add(along.scale(12.0)));
                    assert!(!w.way_clear(a, b, BODY) && stops(&w, th.id, a, b, BODY), "the hall is in the way");
                    let beside = w.free_spot(c);
                    assert!(!th.shape.contains(beside) && w.blocked_at(beside, BODY).is_none() && beside.dist(c) < 5.0, "beside the hall");
                }
                (What::Fence, Id::Pen(_)) => {
                    pens += 1;
                    let way = th.way().unwrap();
                    assert!(w.way_clear(way.outside, way.inside, BODY), "in by the gate");
                    let west = (c.sub(V2::new(th.shape.reach() + 1.5, 0.0)), c.sub(V2::new(th.shape.reach() - 1.5, 0.0)));
                    assert!(!w.way_clear(west.0, west.1, BODY) && stops(&w, th.id, west.0, west.1, BODY), "not over the fence");
                    assert_eq!(w.enclosure_at(way.inside).map(|t| t.id), Some(th.id));
                }
                _ => {}
            }
        }
    }
    assert!(counters > 5 && halls > 5 && pens > 20, "{counters} counters, {halls} halls, {pens} pens");
    // A spot in the open is left where it is, and stops nobody.
    let s = &w.settlements[0];
    let open = (0..400).map(|k| s.pos.add(unit(k as f32 * 0.7).scale(8.0 + k as f32 * 0.3))).find(|&p| w.solids_near(p, 2.0).is_empty()).expect("open ground in town");
    assert_eq!(w.free_spot(open), open);
    assert_eq!(w.solid_at(open), None);
    assert_eq!(w.blocked_at(open, BODY), None);
}

#[test]
fn ids_last_through_a_save_and_what_changes_is_followed() {
    let mut w = worldgen::generate(2);
    for _ in 0..20 {
        w.step(30.0);
    }
    let before = all_things(&w);
    assert_eq!(before, all_things(&w), "asked twice, the same list");
    assert_eq!(before, all_things(&w.clone()), "a copy of the world has the same list");
    let back = World::load_bytes(&w.save_bytes()).expect("loads");
    assert_eq!(before, all_things(&back), "and so does a saved game loaded");
    // A workplace moved (a town laid out again) is followed once told.
    let j = w.society.towns[0].places.iter().position(|p| p.kind == PlaceKind::Hall).expect("a hall");
    let was = w.solid(Id::Place(0, j as u16, 0)).unwrap();
    w.society.towns[0].places[j].pos = was.shape.centre().add(V2::new(3.0, 0.0));
    w.solid_changed();
    let now = w.solid(Id::Place(0, j as u16, 0)).unwrap();
    assert!(now.shape.centre().dist(was.shape.centre().add(V2::new(3.0, 0.0))) < 1e-3, "the hall's outline moved with it");
    // What comes and goes: a standing torch while it burns, by an id that stays.
    let at = w.squad.pos.add(V2::new(4.0, 0.0));
    let t = w.time;
    w.standing.push(StandingTorch { pos: at, lit_at: t, out_at: t + 2.0 * HOUR });
    let torch = w.solids_near(at, 1.0).into_iter().find(|t| t.what == What::Torch).expect("the torch is listed");
    assert!(!torch.blocks);
    w.step(60.0);
    assert_eq!(w.solid(torch.id).map(|t| t.shape), Some(torch.shape), "the same torch a minute on");
    w.standing[0].out_at = w.time - 1.0;
    assert_eq!(w.solid(torch.id), None, "burnt out, it's gone");
}
