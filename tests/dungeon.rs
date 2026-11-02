//! Sites from the story-led dungeon generator (`sim/dungeon.rs`): its three
//! exports read as given, and a single character (Laz: "parties of a single
//! character" for testing) walks, climbs and swims them. The import's own
//! failures, the way-finder's and the generator's verdict are kept apart.

use gahturiyu_sim::sim::dungeon::*;
use gahturiyu_sim::sim::geo::V2;

const F43: &str = "assets/dungeons/story-43-0.json";
const F276: &str = "assets/dungeons/story-276-0.json";
const F5: &str = "assets/dungeons/story-5-0.json";

fn load(f: &str) -> Site {
    Site::load(f).unwrap_or_else(|e| panic!("{f}: {e}"))
}

fn node(s: &Site, x: i32, y: i32, h: f32) -> Node {
    s.node_near(V2::new(x as f32 + 0.5, y as f32 + 0.5), h).unwrap()
}

/// A small site written out as the generator writes one: one row of cells,
/// `(terrain, floor, water surface, deck)` each, and its objects.
fn strip(cells: &[(&str, f32, Option<f32>, Option<f32>)], objects: &str, spawn: f32) -> Site {
    let tiles: Vec<String> = cells
        .iter()
        .map(|(t, f, w, d)| {
            let o = |v: &Option<f32>| v.map(|v| v.to_string()).unwrap_or("null".into());
            format!(r#"{{"terrain":"{t}","floor":{f},"water_surface":{},"deck":{},"low_arch":false,"steps":false,"room":null,"cause":0}}"#, o(w), o(d))
        })
        .collect();
    let json = format!(r#"{{"schema_version":1,"status":"complete","dungeon":{{"width":{},"height":1,"tiles":[{}],"objects":[{objects}],"spawn":[{spawn},0.5]}}}}"#, cells.len(), tiles.join(","));
    Site::from_json(&json, "strip").unwrap()
}

#[test]
fn the_three_exports_are_read_as_given() {
    let s = load(F43);
    assert_eq!((s.schema, s.status.as_str(), s.unmet, s.w, s.h), (1, "complete", 0, 112, 112));
    assert_eq!(s.objects.len(), 20);
    assert_eq!(s.objects.iter().filter(|o| o.kind == ObjKind::Ladder).count(), 2);
    assert_eq!(s.cells.iter().filter(|c| c.ground == Ground::Pit && c.deck.is_some()).count(), 10, "the repaired crossing: pit cells with a deck");
    assert_eq!(s.spawn_h, 0.0);
    // The objective: the chest holding the relic, object 9, up on 2.4 m.
    let o = s.objects.iter().find(|o| o.kind == ObjKind::Chest && o.reward.as_deref() == Some(s.relic.as_str())).unwrap();
    assert_eq!((o.index, o.z), (9, 2.4));
    assert!(s.notes.is_empty(), "{:?}", s.notes);
    let s = load(F276);
    assert_eq!((s.status.as_str(), s.spawn_h), ("complete", 2.4), "276 arrives on its raised floor");
    assert!(s.cells.iter().any(|c| c.water.is_some()));
    let s = load(F5);
    assert_eq!(s.status, "complete");
    let c = s.clearance.as_ref().unwrap();
    assert_eq!((c.marker, c.from, c.to, c.restored), (3, [79, 98], [84, 98], false));
}

#[test]
fn versions_and_broken_files_are_refused_not_guessed() {
    let text = std::fs::read_to_string(F43).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    // No version: the legacy files, read the same.
    v.as_object_mut().unwrap().remove("schema_version");
    let s = Site::from_json(&v.to_string(), "legacy").unwrap();
    assert_eq!(s.schema, 0);
    assert_eq!(s.cells, load(F43).cells);
    // A version this doesn't know.
    v["schema_version"] = 2.into();
    assert!(matches!(Site::from_json(&v.to_string(), "v2"), Err(ImportError::Schema(2))));
    // A ladder whose cell has no deck: an error, not a top made up.
    v["schema_version"] = 1.into();
    let w = v["dungeon"]["width"].as_u64().unwrap() as usize;
    v["dungeon"]["tiles"][40 * w + 56]["deck"] = serde_json::Value::Null;
    assert!(matches!(Site::from_json(&v.to_string(), "no deck"), Err(ImportError::Shape(_))));
}

#[test]
fn in_fixture_43_one_character_reaches_every_thing_and_room_and_gets_back() {
    let s = load(F43);
    let spawn = s.spawn_node().unwrap();
    let reach = s.reach(spawn);
    for o in &s.objects {
        let (at, _) = s.approach(o.index, &reach).unwrap_or_else(|| panic!("object {} {:?} {:?} out of reach", o.index, o.kind, o.name));
        assert!(s.route(at, spawn).is_some(), "no way back from object {}", o.index);
    }
    for r in &s.rooms {
        let n = node(&s, r.centre[0], r.centre[1], 0.0);
        assert!(reach.contains_key(&n), "room {:?} out of reach", r.name);
    }
    // The objective is up on the dry store: the way there climbs a ladder
    // (two nodes of one cell in turn) and comes back down.
    let (at, _) = s.approach(9, &reach).unwrap();
    assert_eq!(s.level(at).unwrap().0, 2.4);
    let there = s.route(spawn, at).unwrap();
    let climbs = |w: &Way| w.nodes.windows(2).filter(|p| p[0].x == p[1].x && p[0].y == p[1].y).count();
    assert_eq!(climbs(&there), 1);
    assert_eq!(climbs(&s.route(at, spawn).unwrap()), 1);
}

#[test]
fn a_ladder_joins_its_foot_and_its_top_and_only_there() {
    let s = load(F43);
    // Ladder 6: floor 0 below, deck 2.4 above, in cell (56, 40).
    let foot = Node { x: 56, y: 40, lv: Lv::Low };
    let top = Node { x: 56, y: 40, lv: Lv::Deck };
    assert_eq!((s.level(foot).unwrap().0, s.level(top).unwrap().0), (0.0, 2.4));
    assert!(s.neighbours(foot).iter().any(|&(n, c)| n == top && (c - 2.4 * CLIMB_SECS_PER_M).abs() < 1e-4));
    assert!(s.neighbours(top).iter().any(|&(n, _)| n == foot));
    // The raised floor beside it is reached from the top, not from the foot.
    let beside = Node { x: 55, y: 40, lv: Lv::Low };
    assert_eq!(s.level(beside).unwrap().0, 2.4);
    assert!(!s.neighbours(foot).iter().any(|&(n, _)| n == beside));
    assert!(s.neighbours(top).iter().any(|&(n, _)| n == beside));
    // Without its ladders the raised store, and the objective on it, are out of reach.
    let mut bare = s.clone();
    bare.objects.retain(|o| o.kind != ObjKind::Ladder);
    let spawn = bare.spawn_node().unwrap();
    let reach = bare.reach(spawn);
    assert!(!reach.contains_key(&top));
    assert!(bare.approach(9, &reach).is_none(), "the chest on 2.4 m can be reached without a ladder");
    assert!(s.approach(9, &s.reach(spawn)).is_some());
}

#[test]
fn walls_stand_and_steps_are_kept_to() {
    let s = load(F43);
    // Every floor–wall–floor run across the site at one height: never a step between.
    let mut checked = 0;
    for y in 0..s.h as i32 {
        for x in 1..s.w as i32 - 1 {
            let (a, w, b) = (s.cell(x - 1, y).unwrap(), s.cell(x, y).unwrap(), s.cell(x + 1, y).unwrap());
            if w.ground == Ground::Wall && a.ground == Ground::Floor && b.ground == Ground::Floor && a.floor == b.floor && a.deck.is_none() && b.deck.is_none() {
                let (na, nb) = (Node { x: (x - 1) as u16, y: y as u16, lv: Lv::Low }, Node { x: (x + 1) as u16, y: y as u16, lv: Lv::Low });
                assert!(s.nodes_in(x, y).is_empty());
                assert!(!s.neighbours(na).iter().any(|&(n, _)| n.x as i32 == x && n.y as i32 == y));
                if let Some(w) = s.route(na, nb) {
                    assert!(w.cost > 2.9, "through the wall at ({x}, {y}): {}", w.cost);
                }
                checked += 1;
            }
        }
    }
    assert!(checked > 20, "only {checked} walls checked");
    // A step: 0.23 m is walked, more is not; down into water is fine, out
    // onto a bank only at most 0.23 m above the surface.
    let s = strip(&[("Floor", 0.0, None, None), ("Floor", 0.23, None, None), ("Floor", 0.5, None, None)], "", 0.5);
    let n = |x| Node { x, y: 0, lv: Lv::Low };
    assert!(s.neighbours(n(0)).iter().any(|&(m, _)| m == n(1)));
    assert!(!s.neighbours(n(1)).iter().any(|&(m, _)| m == n(2)));
    let s = strip(&[("Floor", 0.0, None, None), ("Water", -2.0, Some(-0.8), None), ("Floor", -0.6, None, None), ("Water", -2.0, Some(-0.8), None), ("Floor", 0.0, None, None)], "", 0.5);
    assert!(s.neighbours(n(0)).iter().any(|&(m, _)| m == n(1)), "in from the bank");
    assert!(s.neighbours(n(1)).iter().any(|&(m, _)| m == n(2)), "out onto a ledge 0.2 m up");
    assert!(!s.neighbours(n(3)).iter().any(|&(m, _)| m == n(4)), "out onto a bank 0.8 m up");
    // A pit has no floor; its deck, if any, is a bridge.
    let s = strip(&[("Floor", 0.0, None, None), ("Pit", -3.0, None, Some(0.0)), ("Pit", -3.0, None, None), ("Floor", 0.0, None, None)], "", 0.5);
    assert_eq!(s.nodes_in(1, 0), vec![Node { x: 1, y: 0, lv: Lv::Deck }]);
    assert!(s.nodes_in(2, 0).is_empty());
}

#[test]
fn a_closed_door_is_opened_on_the_way_and_a_walled_doorway_stops_it() {
    let s = load(F43);
    // Door 2, timber, in cell (30, 51): its doorway, crossed north to south.
    let d = s.objects.iter().find(|o| o.index == 2).unwrap();
    assert_eq!((d.kind, d.open), (ObjKind::Door, false));
    assert!(s.closed_door_in(30, 51).is_some());
    let (a, b) = (node(&s, 30, 50, 0.0), node(&s, 30, 52, 0.0));
    let w = s.route(a, b).unwrap();
    assert!(w.nodes.contains(&Node { x: 30, y: 51, lv: Lv::Low }));
    assert!((w.cost - (2.0 + DOOR_SECS)).abs() < 1e-4, "{}", w.cost);
    // The same crossing with the doorway walled up: not through there.
    let mut shut = s.clone();
    shut.cells[51 * shut.w as usize + 30].ground = Ground::Wall;
    if let Some(w) = shut.route(a, b) {
        assert!(!w.nodes.contains(&Node { x: 30, y: 51, lv: Lv::Low }));
        assert!(w.cost > 2.0 + DOOR_SECS + 1.0);
    }
    // No slanting through a doorway.
    for (n, _) in s.neighbours(Node { x: 30, y: 51, lv: Lv::Low }) {
        assert!(n.x == 30 || n.y == 51, "{n:?}");
    }
}

#[test]
fn in_fixture_276_the_water_is_swum_into_and_out_of() {
    let s = load(F276);
    let spawn = s.spawn_node().unwrap();
    let reach = s.reach(spawn);
    // Every water cell is reached, swimming at its surface.
    let water: Vec<Node> = (0..s.h as i32).flat_map(|y| (0..s.w as i32).map(move |x| (x, y))).filter(|&(x, y)| s.cell(x, y).unwrap().water.is_some()).map(|(x, y)| Node { x: x as u16, y: y as u16, lv: Lv::Low }).collect();
    assert!(!water.is_empty());
    for n in &water {
        let (h, how) = s.level(*n).unwrap();
        assert_eq!((how, h), (How::Swim, -0.8), "{n:?}");
        assert!(reach.contains_key(n), "{n:?}");
    }
    // And back out: from the middle of the basin to the spawn.
    let mid = water[water.len() / 2];
    let back = s.route(mid, spawn).unwrap();
    assert!(back.nodes.iter().any(|&n| s.level(n).unwrap().1 == How::Walk));
    // Swimming is slower: a way across water costs more than its length.
    let wet: Vec<&Node> = back.nodes.iter().filter(|&&n| s.level(n).unwrap().1 == How::Swim).collect();
    assert!(!wet.is_empty() || water.len() == 1);
    // The tools are reached dry (the mission's own route), with the ladder.
    let (at, _) = s.approach(4, &reach).unwrap();
    let w = s.route(spawn, at).unwrap();
    assert!(w.nodes.iter().all(|&n| s.level(n).unwrap().1 == How::Walk), "the dry route to the tools swims");
    // Three nodes can't be reached from the spawn: kept, and listed here so a change shows.
    let all: usize = (0..s.h as i32).flat_map(|y| (0..s.w as i32).map(move |x| (x, y))).map(|(x, y)| s.nodes_in(x, y).len()).sum();
    assert_eq!(all - reach.len(), 3);
}

#[test]
fn in_fixture_5_the_cleared_debris_opens_the_route_and_nothing_else_changes() {
    let s = load(F5);
    let c = s.clearance.clone().unwrap();
    let (a, b) = (node(&s, c.from[0], c.from[1], 0.0), node(&s, c.to[0], c.to[1], 0.0));
    assert!(s.route(a, b).is_none(), "one character gets past the debris before it's cleared (even swimming)");
    let mut done = s.clone();
    done.clear_route();
    assert!(done.route(a, b).is_some());
    assert!(done.route(b, a).is_some());
    let changed: Vec<usize> = (0..s.cells.len()).filter(|&i| s.cells[i] != done.cells[i]).collect();
    assert!(changed.len() <= c.blocked.len());
    for i in changed {
        let (x, y) = ((i % s.w as usize) as i32, (i / s.w as usize) as i32);
        assert!(c.blocked.contains(&[x, y]));
        assert_eq!((done.cells[i].ground, done.cells[i].floor), (Ground::Floor, 0.0));
    }
    // The spawn reaches the work marker before, and the far side only after.
    let spawn = s.spawn_node().unwrap();
    assert!(s.approach(c.marker, &s.reach(spawn)).is_some());
    assert!(!s.reach(spawn).contains_key(&b));
    assert!(done.reach(spawn).contains_key(&b));
}

#[test]
fn a_squad_of_one_goes_in_climbs_to_the_chest_and_comes_out() {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let lead = w.squad.members[0];
    w.squad.retain(|m| m == lead);
    let portal = w.squad.at[0];
    let id = w.import_site(F43, portal).unwrap();
    assert!(w.enter_site(lead, id));
    let s = w.sites.get(id).unwrap().clone();
    assert!(s.contains_world(w.squad.at[0]));
    assert_eq!(w.site_height(lead), Some(s.base_h + 0.0));
    // Sent to the chest's spot, up on the dry store (by world height).
    let reach = s.reach(s.spawn_node().unwrap());
    let (at, _) = s.approach(9, &reach).unwrap();
    let spot = s.to_world(Site::centre(at));
    assert!(w.order_in_site(lead, spot, Some(s.base_h + 2.4)));
    let mut secs = 0.0;
    let mut climbed = false;
    while w.site_of(lead).map(|i| !i.way.is_empty() || i.climbing.is_some()).unwrap_or(false) && secs < 600.0 {
        w.step(0.1);
        secs += 0.1;
        climbed |= w.site_of(lead).unwrap().climbing.is_some();
    }
    assert!(climbed, "never climbed");
    assert!(w.squad.at[0].dist(spot) < 0.05, "stopped at {:?}, not {:?}", s.to_local(w.squad.at[0]), Site::centre(at));
    assert!((w.site_height(lead).unwrap() - (s.base_h + 2.4)).abs() < 1e-3);
    // The way there costs about 112 s at 1 m/s; walking pace is slower.
    assert!(secs > 60.0 && secs < 400.0, "{secs}");
    // The doors they went through are open now (and saved so).
    let opened = w.sites.get(id).unwrap().objects.iter().filter(|o| o.kind == ObjKind::Door && o.open).count();
    assert!(opened > 0);
    let back = gahturiyu_sim::sim::world::World::load_bytes(&w.save_bytes()).unwrap();
    assert_eq!(back.sites.sites, w.sites.sites);
    assert_eq!(back.sites.inside, w.sites.inside);
    // Back to the exit, and out by it.
    let exit = s.to_world(s.spawn);
    assert!(w.order_in_site(lead, exit, None));
    let mut secs = 0.0;
    while w.site_of(lead).map(|i| !i.way.is_empty() || i.climbing.is_some()).unwrap_or(false) && secs < 600.0 {
        w.step(0.1);
        secs += 0.1;
    }
    assert!(w.squad.at[0].dist(exit) < 0.05);
    assert_eq!(w.site_height(lead), Some(s.base_h));
    w.leave_site(lead);
    assert!(w.site_of(lead).is_none());
    assert!(w.squad.at[0].dist(portal) < 6.0);
}

/// The generator's own program makes a new site and it imports. Needs the
/// generator's workspace on this PC (`dungeon::understory_root`), so it's
/// run by hand: `cargo test --release --test dungeon -- --ignored`.
#[test]
#[ignore]
fn the_generator_makes_a_site_that_imports() {
    let stories = understory_stories().unwrap();
    assert_eq!(stories.len(), 2000);
    let accepted: Vec<u64> = stories.iter().filter(|s| s.1 == "complete").map(|s| s.0).collect();
    assert_eq!(accepted.len(), 303);
    let path = make_with_understory(accepted[1], 1).unwrap();
    let s = Site::load(path.to_str().unwrap()).unwrap();
    let reach = s.reach(s.spawn_node().unwrap());
    println!("story {} layout 1: {} ({}), status {}, {} nodes reachable, notes {:?}", accepted[1], s.kind, s.culture, s.status, reach.len(), s.notes);
    assert!(reach.len() > 100);
}

#[test]
fn clicking_the_chest_and_the_exit_goes_there_and_uses_them() {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let lead = w.squad.members[0];
    w.squad.retain(|m| m == lead);
    let portal = w.squad.at[0];
    let id = w.import_site(F43, portal).unwrap();
    assert!(w.enter_site(lead, id));
    let run = |w: &mut gahturiyu_sim::sim::world::World| {
        let mut secs = 0.0;
        while w.site_of(lead).is_some_and(|i| i.using.is_some()) && secs < 600.0 {
            w.step(0.1);
            secs += 0.1;
        }
        secs
    };
    // The chest (the objective, up the ladder): gone to, opened, its find named.
    assert!(w.order_use_in_site(lead, 9));
    assert!(run(&mut w) < 600.0);
    let chest = w.sites.get(id).unwrap().objects.iter().find(|o| o.index == 9).unwrap().open;
    assert!(chest);
    assert!(w.log.iter().any(|l| l.1.starts_with("Found: a reserve of repair fittings")), "{:?}", w.log.front());
    // A plaque is read where it stands.
    assert!(w.order_use_in_site(lead, 17));
    run(&mut w);
    assert!(w.log.front().unwrap().1.contains("A packing note"));
    // The exit: back down, across, and out by the portal.
    assert!(w.order_use_in_site(lead, 19));
    let mut secs = 0.0;
    while w.site_of(lead).is_some() && secs < 600.0 {
        w.step(0.1);
        secs += 0.1;
    }
    assert!(w.site_of(lead).is_none(), "still inside");
    assert!(w.squad.at[0].dist(portal) < 6.0);
}

#[test]
fn in_fixture_5_using_the_work_marker_clears_the_way() {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let lead = w.squad.members[0];
    w.squad.retain(|m| m == lead);
    let portal = w.squad.at[0];
    let id = w.import_site(F5, portal).unwrap();
    assert!(w.enter_site(lead, id));
    let s = w.sites.get(id).unwrap().clone();
    let c = s.clearance.clone().unwrap();
    let far = s.to_world(V2::new(c.to[0] as f32 + 0.5, c.to[1] as f32 + 0.5));
    // Sent to the far side first: no way (they stay put).
    assert!(!w.order_in_site(lead, far, Some(s.base_h)));
    assert!(w.order_use_in_site(lead, c.marker));
    let mut secs = 0.0;
    while w.site_of(lead).is_some_and(|i| i.using.is_some()) && secs < 600.0 {
        w.step(0.1);
        secs += 0.1;
    }
    assert!(w.sites.get(id).unwrap().clearance.as_ref().unwrap().restored);
    assert!(w.order_in_site(lead, far, Some(s.base_h)));
}

#[test]
fn sight_reaches_40_m_and_stops_at_walls_and_closed_doors() {
    let s = load(F43);
    let eye = s.spawn;
    let seen = s.sight(eye, SIGHT);
    let at = |x: i32, y: i32| seen[y as usize * s.w as usize + x as usize];
    let mut n = 0;
    for y in 0..s.h as i32 {
        for x in 0..s.w as i32 {
            if !at(x, y) {
                continue;
            }
            n += 1;
            let c = V2::new(x as f32 + 0.5, y as f32 + 0.5);
            assert!(c.dist(eye) <= SIGHT + 1.0, "({x}, {y}) seen from {} m", c.dist(eye));
        }
    }
    assert!(n > 50);
    // Every floor cell seen has a clear line from the eye: no wall between.
    for y in 0..s.h as i32 {
        for x in 0..s.w as i32 {
            if !at(x, y) || s.cell(x, y).unwrap().ground == Ground::Wall {
                continue;
            }
            // A cell is seen when any part of it is: some point of it (its
            // middle or near a corner) has a line from the eye crossing no wall.
            let clear = [(0.5, 0.5), (0.1, 0.1), (0.9, 0.1), (0.1, 0.9), (0.9, 0.9)].iter().any(|&(ox, oy)| {
                let c = V2::new(x as f32 + ox, y as f32 + oy);
                let len = c.dist(eye);
                (1..(len * 20.0) as i32).all(|k| {
                    let p = eye.add(c.sub(eye).scale(k as f32 / (len * 20.0)));
                    s.cell(p.x.floor() as i32, p.y.floor() as i32).unwrap().ground != Ground::Wall
                })
            });
            assert!(clear, "({x}, {y}) seen through a wall");
        }
    }
    // A closed door hides the room behind it; open, it doesn't.
    let d = s.objects.iter().find(|o| o.index == 2).unwrap().clone();
    let north = V2::new(30.5, 49.5);
    let behind = (30, 54);
    assert!(s.cell(behind.0, behind.1).unwrap().ground != Ground::Wall);
    let shut = s.sight(north, SIGHT);
    assert!(!shut[behind.1 as usize * s.w as usize + behind.0 as usize]);
    let mut open = s.clone();
    open.objects.iter_mut().find(|o| o.index == d.index).unwrap().open = true;
    let wide = open.sight(north, SIGHT);
    assert!(wide[behind.1 as usize * s.w as usize + behind.0 as usize]);
}

#[test]
fn what_the_squad_sees_is_kept_as_seen() {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let lead = w.squad.members[0];
    w.squad.retain(|m| m == lead);
    let id = w.import_site(F43, w.squad.pos).unwrap();
    assert!(w.enter_site(lead, id));
    let first = w.sites.get(id).unwrap().seen.iter().filter(|&&b| b).count();
    assert!(first > 0 && w.sites.sees(id).is_some());
    let s = w.sites.get(id).unwrap().clone();
    let reach = s.reach(s.spawn_node().unwrap());
    let (at, _) = s.approach(9, &reach).unwrap();
    assert!(w.order_in_site(lead, s.to_world(Site::centre(at)), Some(s.base_h + 2.4)));
    for _ in 0..3000 {
        w.step(0.1);
    }
    let after = w.sites.get(id).unwrap().seen.iter().filter(|&&b| b).count();
    assert!(after > first * 2, "{first} -> {after}");
    // Now: only what's in sight from the chest, all of it seen before.
    let now = w.sites.sees(id).unwrap();
    assert!(now.iter().zip(&w.sites.get(id).unwrap().seen).all(|(n, s)| !n || *s));
    assert!(now.iter().filter(|&&b| b).count() < after);
}

#[test]
fn steered_by_the_keys_they_walk_slide_on_walls_and_keep_off_ledges() {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let lead = w.squad.members[0];
    w.squad.retain(|m| m == lead);
    let id = w.import_site(F43, w.squad.pos).unwrap();
    assert!(w.enter_site(lead, id));
    let s = w.sites.get(id).unwrap().clone();
    let start = s.to_local(w.squad.at[0]);
    // East for ten seconds: they go east, and never into a wall cell.
    w.steer_in_site(lead, Some(V2::new(1.0, 0.0)));
    for _ in 0..100 {
        w.step(0.1);
        let l = s.to_local(w.squad.at[0]);
        assert_ne!(s.cell_at(l).unwrap().ground, Ground::Wall, "in a wall at {l:?}");
    }
    let l = s.to_local(w.squad.at[0]);
    assert!(l.x > start.x + 1.0, "{start:?} -> {l:?}");
    // Pressed into a wall for long: they stop at it, still standing clear.
    for _ in 0..600 {
        w.step(0.1);
    }
    let stop = s.to_local(w.squad.at[0]);
    let c = s.cell_at(stop).unwrap();
    assert_ne!(c.ground, Ground::Wall);
    assert!(s.cell((stop.x + 0.3).floor() as i32, stop.y.floor() as i32).is_some_and(|c| c.ground != Ground::Wall) || s.cell((stop.x + 0.31).floor() as i32, stop.y.floor() as i32).unwrap().ground == Ground::Wall);
    // Let go: they stop.
    w.steer_in_site(lead, None);
    w.step(0.5);
    let still = w.squad.at[0];
    w.step(0.5);
    assert!(w.squad.at[0].dist(still) < 1e-4);
    // Off a ledge: at the foot of ladder 6, steered west into the raised
    // floor (2.4 m up), they don't go up it.
    let foot = Node { x: 57, y: 41, lv: Lv::Low };
    let mut s2 = s.clone();
    assert!(s2.stand_at(foot, V2::new(56.4, 41.5)).is_none(), "walked up 2.4 m");
    s2.objects.clear();
    assert!(s2.stand_at(foot, V2::new(57.5, 42.5)).is_some());
}

#[test]
fn caverns_and_built_halls_are_told_apart() {
    assert!(load(F43).cells.iter().all(|c| !c.natural), "43 is a built fortress");
    assert!(load(F276).cells.iter().all(|c| c.natural), "276 is a cave");
}

/// A world with a squad of one (and `mates` more) standing at a site's spawn.
fn in_site(site: Site, mates: usize) -> (gahturiyu_sim::sim::world::World, u32, Vec<u32>) {
    let mut w = gahturiyu_sim::sim::worldgen::generate(1);
    let keep: Vec<u32> = w.squad.members.iter().copied().take(1 + mates).collect();
    w.squad.retain(|m| keep.contains(&m));
    let id = w.add_site(site, w.squad.pos);
    for &m in &keep {
        assert!(w.enter_site(m, id));
    }
    w.sites.lead = Some(keep[0]);
    (w, id, keep)
}

fn local_x(w: &gahturiyu_sim::sim::world::World, id: u32) -> f32 {
    w.sites.get(id).unwrap().to_local(w.squad.at[0]).x
}

#[test]
fn a_jump_crosses_a_pit_and_gets_up_a_low_ledge_that_walking_cannot() {
    // Floor, floor, two cells of open pit, floor, a ledge 0.8 m up.
    let cells = [("Floor", 0.0, None, None), ("Floor", 0.0, None, None), ("Pit", -3.0, None, None), ("Pit", -3.0, None, None), ("Floor", 0.0, None, None), ("Floor", 0.0, None, None), ("Floor", 0.8, None, None), ("Floor", 0.8, None, None)];
    // Walking east: they stop at the pit's edge.
    let (mut w, id, team) = in_site(strip(&cells, "", 1.5), 0);
    let lead = team[0];
    w.steer_in_site(lead, Some(V2::new(1.0, 0.0)));
    for _ in 0..40 {
        w.step(0.1);
    }
    assert!(local_x(&w, id) < 2.0, "walked over the pit: {}", local_x(&w, id));
    // A running jump from there: over, and on across.
    w.jump_in_site(lead);
    for _ in 0..30 {
        w.step(0.05);
    }
    assert!(local_x(&w, id) > 4.0, "the jump fell short: {}", local_x(&w, id));
    assert!(w.site_of(lead).unwrap().air.is_none());
    // Walking on, the 0.8 m ledge stops them; a jump gets them up.
    for _ in 0..40 {
        w.step(0.1);
    }
    assert!(local_x(&w, id) < 6.0, "walked up 0.8 m: {}", local_x(&w, id));
    w.jump_in_site(lead);
    for _ in 0..30 {
        w.step(0.05);
    }
    assert!(local_x(&w, id) > 6.0, "{}", local_x(&w, id));
    assert!((w.site_of(lead).unwrap().h - 0.8).abs() < 1e-3);
}

#[test]
fn a_fall_into_a_pit_puts_them_back_where_they_last_stood() {
    let cells = [("Floor", 0.0, None, None), ("Floor", 0.0, None, None), ("Pit", -3.0, None, None), ("Pit", -3.0, None, None), ("Pit", -3.0, None, None), ("Pit", -3.0, None, None), ("Floor", 0.0, None, None)];
    let (mut w, id, team) = in_site(strip(&cells, "", 1.5), 0);
    let lead = team[0];
    w.steer_in_site(lead, Some(V2::new(1.0, 0.0)));
    for _ in 0..20 {
        w.step(0.1);
    }
    // Too far to jump: they come down in the pit and are put back on the floor.
    w.jump_in_site(lead);
    let mut lowest = f32::MAX;
    for _ in 0..80 {
        w.step(0.05);
        lowest = lowest.min(w.site_of(lead).unwrap().h);
    }
    assert!(lowest < -1.0, "never fell: {lowest}");
    let i = w.site_of(lead).unwrap();
    assert_eq!((i.h, i.air), (0.0, None));
    assert!(local_x(&w, id) < 2.0, "{}", local_x(&w, id));
}

#[test]
fn a_dodge_steps_back_for_stamina() {
    let cells: Vec<_> = (0..12).map(|_| ("Floor", 0.0, None, None)).collect();
    let (mut w, id, team) = in_site(strip(&cells, "", 6.5), 0);
    let lead = team[0];
    let before = local_x(&w, id);
    let s0 = w.stamina_of(lead).unwrap();
    // Facing east, the dodge goes west.
    w.dodge_in_site(lead, V2::new(1.0, 0.0));
    assert!((s0 - w.stamina_of(lead).unwrap() - DODGE_STAMINA).abs() < 0.005, "stamina {s0} -> {}", w.stamina_of(lead).unwrap());
    for _ in 0..10 {
        w.step(0.05);
    }
    let moved = before - local_x(&w, id);
    assert!((moved - DODGE_M).abs() < 0.15, "dodged {moved} m");
}

#[test]
fn the_mate_follows_the_one_you_control() {
    let (mut w, id, team) = in_site(load(F43), 1);
    let (lead, mate) = (team[0], team[1]);
    let s = w.sites.get(id).unwrap().clone();
    // The lead goes to the chest (up the ladder, far off); the mate keeps up.
    let reach = s.reach(s.spawn_node().unwrap());
    let (at, _) = s.approach(9, &reach).unwrap();
    assert!(w.order_in_site(lead, s.to_world(Site::centre(at)), Some(s.base_h + 2.4)));
    let mut worst = 0.0f32;
    for _ in 0..3000 {
        w.step(0.1);
        worst = worst.max(w.squad.at[0].dist(w.squad.at[1]));
    }
    let gap = w.squad.at[0].dist(w.squad.at[1]);
    let allowed = gahturiyu_sim::sim::behaviour::Follow::Medium.gap();
    assert!(gap < allowed * 2.5, "the mate is {gap} m behind (worst {worst})");
    assert!((w.site_of(mate).unwrap().h - 2.4).abs() < 1e-3, "the mate didn't come up the ladder");
}

#[test]
fn e_uses_only_what_is_within_reach() {
    let (mut w, id, team) = in_site(load(F43), 0);
    let lead = team[0];
    let s = w.sites.get(id).unwrap().clone();
    // Plaque 16 is a few metres from the arrival: out of reach from there.
    assert!(!w.site_in_reach(lead, 16));
    assert!(!w.use_in_reach(lead, 16));
    assert!(w.site_use_label(id, 16).unwrap().starts_with("Read: "));
    let o = s.objects.iter().find(|o| o.index == 16).unwrap().at;
    assert!(w.order_in_site(lead, s.to_world(o.add(V2::new(0.0, -1.0))), None));
    for _ in 0..200 {
        w.step(0.1);
    }
    assert!(w.site_in_reach(lead, 16));
    assert!(w.use_in_reach(lead, 16));
    assert!(w.log.front().unwrap().1.contains("A surviving account"));
    assert_eq!(w.site_use_label(id, 19).unwrap(), "Leave: Return to the Company");
}

/// Walk the lead `secs` seconds east along a long open strip; the gap to the mate.
fn gap_after_walk(follow: gahturiyu_sim::sim::behaviour::Follow) -> f32 {
    let cells: Vec<_> = (0..80).map(|_| ("Floor", 0.0, None, None)).collect();
    let (mut w, _id, team) = in_site(strip(&cells, "", 2.5), 1);
    let (lead, mate) = (team[0], team[1]);
    let mut b = w.behaviour(mate);
    b.follow = follow;
    w.set_behaviour(mate, b);
    w.steer_in_site(lead, Some(V2::new(1.0, 0.0)));
    for _ in 0..200 {
        w.step(0.1);
    }
    w.steer_in_site(lead, None);
    for _ in 0..100 {
        w.step(0.1);
    }
    w.squad.at[0].dist(w.squad.at[1])
}

#[test]
fn the_follow_setting_decides_how_close_the_mate_keeps() {
    use gahturiyu_sim::sim::behaviour::Follow;
    let (close, far) = (gap_after_walk(Follow::Close), gap_after_walk(Follow::Far));
    assert!(close <= Follow::Close.gap() + 0.5, "close: {close}");
    assert!(far > Follow::Close.gap() + 1.0 && far <= Follow::Far.gap() + 0.5, "far: {far}");
    // Holding, they stay where they were told and the lead walks off.
    let held = gap_after_walk(Follow::Hold);
    assert!(held > 20.0, "held: {held}");
}

#[test]
fn holding_brings_them_back_to_their_spot() {
    use gahturiyu_sim::sim::behaviour::Follow;
    let cells: Vec<_> = (0..40).map(|_| ("Floor", 0.0, None, None)).collect();
    let (mut w, id, team) = in_site(strip(&cells, "", 2.5), 1);
    let mate = team[1];
    let mut b = w.behaviour(mate);
    b.follow = Follow::Hold;
    w.set_behaviour(mate, b);
    let spot = w.behaviour(mate).hold.unwrap();
    assert_eq!(spot.0, id);
    // Sent off by hand, they come back to it.
    assert!(w.order_in_site(mate, w.sites.get(id).unwrap().to_world(V2::new(15.5, 0.5)), None));
    for _ in 0..400 {
        w.step(0.1);
    }
    assert!(w.squad.at[1].dist(spot.1) < 1.0, "{:?} vs {:?}", w.squad.at[1], spot.1);
}

#[test]
fn sprinting_is_faster_and_costs_stamina_by_the_metre() {
    let cells: Vec<_> = (0..80).map(|_| ("Floor", 0.0, None, None)).collect();
    let run = |sprint: bool| {
        let (mut w, id, team) = in_site(strip(&cells, "", 2.5), 0);
        let lead = team[0];
        let s0 = w.stamina_of(lead).unwrap();
        w.steer_in_site(lead, Some(V2::new(1.0, 0.0)));
        w.sprint_in_site(lead, sprint);
        for _ in 0..50 {
            w.step(0.1);
        }
        (local_x(&w, id) - 2.5, s0 - w.stamina_of(lead).unwrap())
    };
    let (walked, walk_cost) = run(false);
    let (ran, run_cost) = run(true);
    assert!((ran / walked - SPRINT).abs() < 0.1, "walked {walked} m, sprinted {ran} m");
    assert!(run_cost - walk_cost > SPRINT_PER_M * ran * 0.8, "sprint cost {run_cost} vs walk {walk_cost}");
}

/// An open room `w` × `h` cells (floor at 0), as the generator writes one,
/// with wall cells where `wall(x, y)`; the arrival at (`sx`, `sy`).
fn room(w: usize, h: usize, wall: impl Fn(usize, usize) -> bool, sx: f32, sy: f32) -> Site {
    let mut tiles = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let t = if wall(x, y) || x == 0 || y == 0 || x == w - 1 || y == h - 1 { "Wall" } else { "Floor" };
            tiles.push(format!(r#"{{"terrain":"{t}","floor":0,"water_surface":null,"deck":null,"low_arch":false,"natural":false}}"#));
        }
    }
    let json = format!(r#"{{"schema_version":1,"status":"complete","dungeon":{{"width":{w},"height":{h},"tiles":[{}],"objects":[],"spawn":[{sx},{sy}]}}}}"#, tiles.join(","));
    Site::from_json(&json, "room").unwrap()
}

fn battle_of(w: &gahturiyu_sim::sim::world::World, pid: u32) -> usize {
    let bid = w.fighting[&pid];
    w.battles.iter().position(|b| b.id == bid).unwrap()
}

fn fighter(w: &gahturiyu_sim::sim::world::World, b: usize, pid: u32) -> usize {
    w.battles[b].fighters.iter().position(|f| f.pid == pid && f.is_person()).unwrap()
}

fn hp(w: &gahturiyu_sim::sim::world::World, b: usize, i: usize) -> f32 {
    w.battles[b].fighters[i].hp.iter().sum()
}

#[test]
fn foes_in_a_site_come_when_they_see_you_and_fights_keep_to_the_rooms() {
    let (mut w, id, team) = in_site(load(F43), 1);
    let foes = w.spawn_site_foes(id, 3);
    assert_eq!(foes.len(), 3, "no spot for foes in 43");
    let s = w.sites.get(id).unwrap().clone();
    // Out of sight of the arrival: nobody fights yet.
    w.step(1.0);
    assert!(team.iter().all(|p| !w.fighting.contains_key(p)));
    // Walk the lead toward them: once in sight, the fight starts.
    let at = w.sites.foes[&foes[0]];
    assert!(w.order_in_site(team[0], at, None));
    let mut started = false;
    let mut in_wall = 0;
    for _ in 0..1200 {
        w.step(0.1);
        started |= w.fighting.contains_key(&team[0]);
        for b in &w.battles {
            for f in &b.fighters {
                if s.contains_world(f.pos) && s.cell_at(s.to_local(f.pos)).unwrap().ground == Ground::Wall {
                    in_wall += 1;
                }
            }
        }
    }
    assert!(started, "the foes never came");
    assert_eq!(in_wall, 0, "fighters stood in walls {in_wall} times");
}

#[test]
fn a_strike_lands_where_you_aim_and_nowhere_else() {
    let run = |toward: bool| -> (f32, f32) {
        let (mut w, id, team) = in_site(room(20, 10, |_, _| false, 5.5, 5.5), 0);
        let lead = team[0];
        let s = w.sites.get(id).unwrap().clone();
        let foes = w.put_site_foes(s.to_world(V2::new(6.7, 5.5)), 1);
        let dir = if toward { V2::new(1.0, 0.0) } else { V2::new(-1.0, 0.0) };
        w.aim_in_site(lead, dir, false);
        assert!(w.strike_in_site(lead, dir) || w.fighting.contains_key(&lead));
        let b = battle_of(&w, lead);
        let (me, foe) = (fighter(&w, b, lead), fighter(&w, b, foes[0]));
        // The foe stands and takes it (no fight back), so only the lead's blows count.
        w.battles[b].fighters[foe].think_at = f64::INFINITY;
        w.battles[b].fighters[foe].target = None;
        let full = hp(&w, b, foe);
        for _ in 0..200 {
            w.strike_in_site(lead, dir);
            w.step(0.05);
            if !w.fighting.contains_key(&lead) {
                break;
            }
        }
        let b = w.battles.iter().position(|x| x.fighters.iter().any(|f| f.pid == foes[0])).map(|b| b);
        let after = b.map(|b| hp(&w, b, fighter(&w, b, foes[0]))).unwrap_or(0.0);
        let _ = me;
        (full, after)
    };
    let (full, after) = run(true);
    assert!(after < full, "struck toward the foe 200 times, never hurt them");
    let (full, after) = run(false);
    assert_eq!(after, full, "struck away from the foe, yet hurt them");
}

#[test]
fn an_arrow_hits_the_first_body_on_its_line_friend_or_foe_and_stops_at_walls() {
    use gahturiyu_sim::sim::combat::FxKind;
    use gahturiyu_sim::sim::items;
    // The lead with a longbow and arrows put in their hands; one mate; one
    // foe. `wall`: a wall across the room at x = 15. `foe_at`: where the foe
    // stands for the fight to start (in sight), before being put at x = 20.5.
    let run = |wall: bool, mate_on_line: bool| -> (f32, f32, f32, f32, Vec<f32>) {
        let mut w = gahturiyu_sim::sim::worldgen::generate(1);
        let (archer, mate) = (w.squad.members[0], w.squad.members[1]);
        w.squad.retain(|m| m == archer || m == mate);
        let id = w.add_site(room(30, 9, move |x, y| wall && x == 15 && y > 0, 3.5, 4.5), w.squad.pos);
        assert!(w.enter_site(archer, id) && w.enter_site(mate, id));
        w.sites.lead = Some(archer);
        let s = w.sites.get(id).unwrap().clone();
        let foe = w.put_site_foes(s.to_world(V2::new(10.5, 4.5)), 1)[0];
        let east = V2::new(1.0, 0.0);
        w.aim_in_site(archer, east, false);
        w.step(0.1);
        assert!(w.fighting.contains_key(&archer), "no fight began");
        let b = battle_of(&w, archer);
        let (ia, im, ifoe) = (fighter(&w, b, archer), fighter(&w, b, mate), fighter(&w, b, foe));
        let bow = *items::item(items::id("longbow")).weapon().unwrap();
        w.battles[b].fighters[ia].weapon = bow;
        w.battles[b].fighters[ia].ammo = 40;
        for &k in &[im, ifoe] {
            w.battles[b].fighters[k].think_at = f64::INFINITY;
            w.battles[b].fighters[k].target = None;
        }
        w.battles[b].fighters[ia].pos = s.to_world(V2::new(3.5, 4.5));
        w.battles[b].fighters[im].pos = s.to_world(if mate_on_line { V2::new(9.5, 4.5) } else { V2::new(5.5, 1.5) });
        w.battles[b].fighters[ifoe].pos = s.to_world(V2::new(20.5, 4.5));
        let (mate_full, foe_full) = (hp(&w, b, im), hp(&w, b, ifoe));
        for _ in 0..300 {
            w.strike_in_site(archer, east);
            w.step(0.05);
            if !w.fighting.contains_key(&archer) {
                break;
            }
        }
        let b = battle_of(&w, archer);
        let ends: Vec<f32> = w.battles[b].fx.iter().filter_map(|x| match x.kind { FxKind::Arrow { to, .. } => Some(s.to_local(to).x), _ => None }).collect();
        (mate_full, hp(&w, b, fighter(&w, b, mate)), foe_full, hp(&w, b, fighter(&w, b, foe)), ends)
    };
    // A mate on the line: the arrows go into them, never past to the foe.
    let (mate_full, mate_now, foe_full, foe_now, ends) = run(false, true);
    assert!(!ends.is_empty(), "no arrows flew");
    assert_eq!(foe_now, foe_full, "an arrow passed the mate to the foe");
    assert!(mate_now < mate_full, "no arrow hit the mate in the way ({} shots)", ends.len());
    // The line clear: the foe is hit.
    let (_, _, foe_full, foe_now, _) = run(false, false);
    assert!(foe_now < foe_full, "a clear line, and the foe never hit");
    // A wall between: nothing reaches the foe, and every arrow comes down this side of it.
    let (_, _, foe_full, foe_now, ends) = run(true, false);
    assert!(!ends.is_empty());
    assert_eq!(foe_now, foe_full, "an arrow went through the wall");
    assert!(ends.iter().all(|&x| x <= 15.0), "arrows came down past the wall: {ends:?}");
}

#[test]
fn blocking_toward_a_foe_turns_more_of_their_blows() {
    // A foe beside the lead, left to fight; the lead holds a block facing
    // them, or not. Damage taken over the same stretch.
    let taken = |guard: bool| -> f32 {
        let (mut w, id, team) = in_site(room(20, 10, |_, _| false, 5.5, 5.5), 0);
        let lead = team[0];
        let s = w.sites.get(id).unwrap().clone();
        let foes = w.put_site_foes(s.to_world(V2::new(6.8, 5.5)), 1);
        let east = V2::new(1.0, 0.0);
        w.aim_in_site(lead, east, guard);
        w.step(0.1);
        assert!(w.fighting.contains_key(&lead));
        let b = battle_of(&w, lead);
        let me = fighter(&w, b, lead);
        let full = hp(&w, b, me);
        let _ = foes;
        // Ten seconds, both still standing (checked), summed over a few foes' seeds.
        for _ in 0..100 {
            w.aim_in_site(lead, east, guard);
            w.step(0.1);
        }
        assert!(w.fighting.contains_key(&lead), "the fight ended inside ten seconds");
        let b = battle_of(&w, lead);
        full - hp(&w, b, fighter(&w, b, lead))
    };
    let (open, guarded) = (taken(false), taken(true));
    assert!(open > 0.0, "the foe never hurt them");
    assert!(guarded < open * 0.7, "blocking took {guarded} against {open} open");
}
