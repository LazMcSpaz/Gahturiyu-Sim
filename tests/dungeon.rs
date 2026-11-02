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
    assert_eq!(back.sites, w.sites);
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
