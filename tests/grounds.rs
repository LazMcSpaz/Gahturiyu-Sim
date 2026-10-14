//! The land under each mark for the place builder (`sim/grounds.rs`): what
//! the file says is the game's land, and a town laid out by hand stands on
//! its mark's spot in every world, so the layout sits on the land it was
//! drawn on.

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::{atlas, grounds, placefile::PlaceFile, plan::Plans, plants, roster::Rosters, worldgen};

fn decode(b64: &str) -> Vec<u8> {
    let val = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        _ => 63,
    };
    let (mut bits, mut n, mut out) = (0u32, 0, Vec::new());
    for c in b64.bytes().take_while(|&c| c != b'=') {
        bits = bits << 6 | val(c) as u32;
        n += 6;
        if n >= 8 {
            n -= 8;
            out.push((bits >> n) as u8);
        }
    }
    out
}

#[test]
fn base64_is_the_standard_one() {
    assert_eq!(grounds::base64(b"Man"), "TWFu");
    assert_eq!(grounds::base64(b"Ma"), "TWE=");
    assert_eq!(grounds::base64(b"M"), "TQ==");
    assert_eq!(decode(&grounds::base64(&[1, 2, 3, 250, 251])), vec![1, 2, 3, 250, 251]);
}

#[test]
fn a_laid_out_town_stands_on_its_mark_in_every_world() {
    let plans = Plans::load();
    let marks = atlas::town_marks();
    let (k, mark) = marks.iter().enumerate().find(|(_, m)| plans.for_town(&m.key()).is_some()).expect("a town laid out by hand (the Last Wave outpost's place)");
    let spot = worldgen::town_spot(mark);
    for seed in [3, 4] {
        let w = worldgen::generate_with_plans(seed, &Rosters::load(), &plans);
        assert_eq!(w.settlements[k].pos, spot, "{} in world {seed}", mark.key());
    }
}

#[test]
fn the_file_is_the_game_land() {
    let w = grounds::world_for(5);
    let all = grounds::marks();
    let mut keys: Vec<&String> = all.iter().map(|m| &m.0).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), all.len(), "a key twice among the marks");
    assert_eq!(all.len(), atlas::town_marks().len() + grounds::landmarks().len());
    let (key, id, label, kind, c) = all.iter().find(|m| m.0 == "last_wave_outpost").expect("the outpost").clone();
    let land = grounds::land_at(&w, &key, &id, &label, &kind, c);
    assert_eq!(land.place.size, grounds::area_of(&kind), "an outpost's area");
    assert_eq!(land.around.size, land.place.size);
    let (gw, gh) = land.place.size;
    let (oi, oj) = land.place.origin;
    let ground: Vec<char> = land.place.ground.chars().collect();
    let heights: Vec<i16> = decode(&land.place.height).chunks(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
    assert_eq!((ground.len(), heights.len()), (gw * gh, gw * gh));
    let h0 = w.terrain.height(c);
    for (i, j) in [(0, 0), (oi, oj), (gw - 1, gh - 1), (17, gh - 9), (gw - 10, 9)] {
        let k = j * gw + i;
        let p = c.add(V2::new((i as f32 - oi as f32 + 0.5) * grounds::CELL, (j as f32 - oj as f32 + 0.5) * grounds::CELL));
        assert_eq!(ground[k], grounds::letter(&w, p), "ground at cell {i},{j}");
        assert!((heights[k] as f32 / 10.0 - (w.terrain.height(p) - h0)).abs() <= 0.051, "height at cell {i},{j}");
    }
    // The trees are the land's own, where it grows them.
    let half = (gw as f32 * grounds::CELL / 2.0, gh as f32 * grounds::CELL / 2.0);
    let own = plants::land_trees_in(&w, c.x - half.0, c.y - half.1, c.x + half.0, c.y + half.1);
    assert_eq!(land.trees.len(), own.len());
    assert!(!own.is_empty(), "the outpost's woods");
    for (x, y, _, _) in &land.trees {
        assert!(x.abs() <= half.0 && y.abs() <= half.1);
        assert!(plants::spot(&w, c.add(V2::new(*x, *y))).is_some(), "a tree at {x},{y} where nothing grows");
    }
}

#[test]
fn a_place_is_as_big_as_its_kind() {
    assert_eq!(grounds::area_of("hamlet"), (60, 45));
    assert_eq!(grounds::area_of("town"), (160, 120));
    assert_eq!(grounds::area_of("capital"), (300, 225));
    assert_eq!(grounds::area_of("ruin"), grounds::area_of("outpost"));
    // Every kind a town mark can be has its own line.
    for m in atlas::town_marks() {
        let k = format!("{:?}", m.kind).to_lowercase();
        assert!(grounds::AREAS.iter().any(|a| a.0 == k), "no area for a {k}");
    }
}

#[test]
fn the_sites_land_is_every_marks() {
    // `tools/map/grounds` (written by `headless site`) holds a file for every
    // mark, and its index gives each mark's id its key.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/map/grounds");
    let index: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("index.json")).expect("tools/map/grounds/index.json: run headless site")).expect("it reads");
    let index = index.as_object().expect("an object");
    let all = grounds::marks();
    assert_eq!(index.len(), all.len(), "the index has every mark and no other: run headless site");
    for (key, id, _, kind, _) in &all {
        assert_eq!(index.get(id).and_then(|e| e["key"].as_str()), Some(key.as_str()), "mark {id}: run headless site");
        let file: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join(format!("{key}.json"))).unwrap_or_else(|_| panic!("grounds/{key}.json"))).expect("it reads");
        let (w, h) = grounds::area_of(kind);
        assert_eq!((file["place"]["size"][0].as_u64(), file["place"]["size"][1].as_u64()), (Some(w as u64), Some(h as u64)), "{key}'s area: run headless site");
    }
}

#[test]
fn a_place_file_of_any_size_is_read() {
    // A place drawn on a hamlet's grid (60 x 45 at 2 m), not the town's the
    // builder once had: its land covers exactly that, and its paint is where
    // it was put.
    let mut ground = String::from(".").repeat(60 * 45);
    ground.replace_range(0..1, "n");
    let text = serde_json::json!({
        "format": 1, "key": "test_hamlet", "title": "A hamlet", "notes": "",
        "cell": 2.0, "size": [60, 45], "origin": [30, 22],
        "ground": ground, "height": ".".repeat(60 * 45),
        "foliage": [{ "k": "tree_broad", "x": -55.0, "y": -40.0, "s": 1.0 }],
        "roads": [], "things": []
    })
    .to_string();
    let place = PlaceFile::from_json(&text).expect("it reads");
    let plan = place.to_plan();
    let land = plan.land.as_ref().expect("the land comes with it");
    assert!(land.covers(V2::new(-59.0, -43.0)) && land.covers(V2::new(59.0, 45.0)));
    assert!(!land.covers(V2::new(-61.0, 0.0)) && !land.covers(V2::new(0.0, 47.0)) && !land.covers(V2::new(100.0, 0.0)));
    assert_eq!(land.plants_at(V2::new(0.0, 0.0)).count(), 1);
}
