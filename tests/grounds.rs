//! The land under each mark for the place builder (`sim/grounds.rs`): what
//! the file says is the game's land, and a town laid out by hand stands on
//! its mark's spot in every world, so the layout sits on the land it was
//! drawn on.

use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::{atlas, grounds, plan::Plans, plants, roster::Rosters, worldgen};

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
    assert_eq!(land.place.size, grounds::SIZE);
    let (gw, gh) = land.place.size;
    let ground: Vec<char> = land.place.ground.chars().collect();
    let heights: Vec<i16> = decode(&land.place.height).chunks(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
    assert_eq!((ground.len(), heights.len()), (gw * gh, gw * gh));
    let h0 = w.terrain.height(c);
    for (i, j) in [(0, 0), (80, 60), (159, 119), (17, 101), (140, 9)] {
        let k = j * gw + i;
        let p = c.add(V2::new((i as f32 - 80.0 + 0.5) * grounds::CELL, (j as f32 - 60.0 + 0.5) * grounds::CELL));
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
