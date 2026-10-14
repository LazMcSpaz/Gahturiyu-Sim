//! The catalogue file (`assets/world/catalogue.json`) is the game's own
//! list of what the place builder can put down. It's written by
//! `headless catalogue`; if it's out of date this test says so, so the page
//! and the game never disagree about a size.

use gahturiyu_sim::sim::catalogue::{catalogue, catalogue_json, find, workplace_of};

#[test]
fn the_catalogue_file_is_current() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/world/catalogue.json");
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
    assert!(on_disk == catalogue_json(), "assets/world/catalogue.json is stale: run `cargo run --release --bin headless -- catalogue`");
}

#[test]
fn every_entry_is_complete_and_keys_are_unique() {
    let all = catalogue();
    let mut keys: Vec<&str> = all.iter().map(|e| e.key.as_str()).collect();
    keys.sort();
    let n = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), n, "a key twice");
    for e in &all {
        assert!(!e.name.is_empty() && !e.glyph.is_empty(), "{}", e.key);
        if e.group != "ground" {
            assert!(e.depth > 0.0 && e.width > 0.0, "{} has a size", e.key);
        }
    }
    for g in ["building", "base", "workplace", "pen", "prop", "foliage", "ground"] {
        assert!(all.iter().any(|e| e.group == g), "a {g}");
    }
}

#[test]
fn what_the_page_reads_matches_the_game() {
    // A courtyard house keeps weavers? No: the weave hall does.
    let hall = find("qotiro_weavehall").expect("the weave hall");
    assert!(hall.jobs.contains(&"Weaver".to_string()), "{:?}", hall.jobs);
    assert!(hall.stations.contains(&"Loom".to_string()));
    let forge = find("roduro_forge").unwrap();
    assert!(forge.jobs.contains(&"Smith".to_string()) && forge.stations.contains(&"Forge".to_string()));
    // A smithy (the workplace) has a smith; fields have farmers.
    assert!(find("smithy").unwrap().jobs.contains(&"Smith".to_string()));
    assert_eq!(find("fields").unwrap().jobs, vec!["Farmer".to_string()]);
    assert_eq!(workplace_of("smithy"), Some(gahturiyu_sim::sim::jobs::PlaceKind::Forge));
    // Quarters sleep several households.
    assert!(find("qotiro_quarters").unwrap().beds >= 4);
    // A pen is round, with its usual number of beasts.
    let pen = find("pen_turiyu").unwrap();
    assert!(pen.radius.unwrap() > 4.0 && pen.limit.is_some());
}
