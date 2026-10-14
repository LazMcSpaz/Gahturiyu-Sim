//! The town list (`assets/world/towns.ron`) is what `tools/map/marks_to_towns.py`
//! makes of Laz's marks (`assets/world/marks-export.json`): current, every
//! settlement mark in it, each key once.

use gahturiyu_sim::sim::atlas;

const SETTLEMENT_KINDS: &[&str] = &["capital", "city", "town", "village", "hamlet", "fort", "imperial_fort", "outpost", "warden_outpost", "qotiro_outpost", "highfolk_hold", "stilt_village"];

fn export() -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/world/marks-export.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("the export")).expect("it reads")
}

/// A mark's kind on the page as the game's kind (the script's table).
fn kind_of(k: &str) -> &'static str {
    match k {
        "capital" => "Capital",
        "city" => "City",
        "town" | "highfolk_hold" => "Town",
        "village" | "stilt_village" => "Village",
        "hamlet" => "Hamlet",
        "fort" | "imperial_fort" => "Fort",
        _ => "Outpost",
    }
}

#[test]
fn towns_ron_is_what_the_script_makes_of_the_marks() {
    // Every town as its mark says: the spot, the label, the kind, the holder
    // (checked here in Rust, so it holds wherever the tests run).
    let e = export();
    let all = e["marks"].as_array().unwrap();
    for t in atlas::town_marks() {
        let m = all.iter().find(|m| m["id"].as_str() == Some(t.id.as_str())).unwrap_or_else(|| panic!("{}: no mark {}", t.key(), t.id));
        let px = (m["px"][0].as_u64().unwrap() as u32, m["px"][1].as_u64().unwrap() as u32);
        assert_eq!(t.px, px, "{}: the mark moved: run tools/map/marks_to_towns.py", t.key());
        let label = m["label"].as_str().unwrap_or("").replace('\u{fffd}', "\u{2019}");
        assert_eq!(t.label, label.trim(), "{}: relabelled: run tools/map/marks_to_towns.py", t.key());
        assert_eq!(format!("{:?}", t.kind), kind_of(m["kind"].as_str().unwrap()), "{}", t.key());
        assert_eq!(t.holder, m["holder"].as_str().unwrap_or(""), "{}", t.key());
    }
    // And the script itself agrees, keys and all, where Python can be run
    // (Windows' store shortcuts for python can't be started from a test).
    let root = env!("CARGO_MANIFEST_DIR");
    let tries: [(&str, &[&str]); 3] = [("python", &[]), ("python3", &[]), ("py", &["-3"])];
    for (py, pre) in tries {
        let Ok(out) = std::process::Command::new(py).current_dir(root).args(pre).args(["tools/map/marks_to_towns.py", "--check"]).output() else { continue };
        let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(out.status.success(), "{said}");
        return;
    }
    eprintln!("(no Python to run tools/map/marks_to_towns.py --check: the fields were checked, the keys weren't)");
}

#[test]
fn every_settlement_mark_is_a_town_with_its_own_key() {
    let marks = atlas::town_marks();
    let mut keys: Vec<String> = marks.iter().map(|m| m.key()).collect();
    keys.sort();
    let n = keys.len();
    keys.dedup();
    assert_eq!(keys.len(), n, "a key twice");
    assert!(marks.iter().all(|m| !m.id.is_empty()), "every town knows its mark");
    let e = export();
    let settlements: Vec<&serde_json::Value> = e["marks"].as_array().unwrap().iter().filter(|m| SETTLEMENT_KINDS.contains(&m["kind"].as_str().unwrap_or(""))).collect();
    assert_eq!(settlements.len(), marks.len(), "one town per settlement mark");
    for m in settlements {
        let id = m["id"].as_str().unwrap();
        assert!(marks.iter().any(|t| t.id == id), "mark {id} ({}) is a town", m["label"]);
    }
    // The keys the rosters, plans and places name are there.
    for key in ["imperial_capital", "imperial_village", "last_wave_outpost", "last_wave_outpost_north"] {
        assert!(marks.iter().any(|m| m.key() == key), "{key}");
    }
}
