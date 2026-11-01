//! The quest generator's contract (`src/questgen/data.rs`) reads the design
//! tests' sample worlds and writes them back unchanged in substance.

use gahturiyu_sim::questgen::data::*;
use std::path::Path;

fn load(name: &str) -> QuestSet {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tools/quests/samples")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("sample world");
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name} doesn't fit the contract: {e}"))
}

#[test]
fn the_guild_suite_sample_reads() {
    let w = load("world-A.json");
    assert_eq!(w.cards.len(), 54);
    assert_eq!(w.faction_stories.len(), 5);
    assert!(w.story.is_none() || w.story.as_ref().unwrap().chapters.is_empty());
    assert!(w.cards.iter().all(|c| c.relatedness == Relatedness::Faction));
}

#[test]
fn the_full_world_sample_reads() {
    let w = load("world-B.json");
    assert_eq!(w.cards.len(), 188);
    assert_eq!(w.faction_stories.len(), 5);
    let story = w.story.as_ref().expect("a main story");
    assert_eq!(story.arc, StoryArc::Ripple);
    assert_eq!(story.shape, Shape::Above);
    assert!(!story.chapters.is_empty());
    assert_eq!(w.map.places.len(), 54);
    assert_eq!(w.map.regions.len(), 10);
    // Every rolled way points into its card's pool.
    for c in &w.cards {
        for &i in &c.ways_rolled {
            assert!(i < c.ways_pool.len(), "{}: rolled way {i} outside the pool", c.id);
        }
    }
}

#[test]
fn a_quest_set_survives_a_round_trip() {
    for name in ["world-A.json", "world-B.json"] {
        let w = load(name);
        let text = serde_json::to_string(&w).unwrap();
        let back: QuestSet = serde_json::from_str(&text).unwrap();
        assert_eq!(back.cards.len(), w.cards.len());
        assert_eq!(back.facts.len(), w.facts.len());
        assert_eq!(back.registry.people.len(), w.registry.people.len());
        // Written again, it comes out the same: nothing depends on map order.
        assert_eq!(serde_json::to_string(&back).unwrap(), text, "{name}");
    }
}

#[test]
fn closed_value_sets_are_enforced() {
    let bad = r#"{"cards": [{"id": "c-1", "family": "Heist"}]}"#;
    assert!(serde_json::from_str::<QuestSet>(bad).is_err(), "an unknown family is refused");
    let good = r#"{"cards": [{"id": "c-1", "family": "Contest", "relatedness": "faction"}]}"#;
    let w: QuestSet = serde_json::from_str(good).unwrap();
    assert_eq!(w.cards[0].family, Family::Contest);
}

#[test]
fn a_spec_reads_its_needs() {
    let text = r#"{
        "id": "s-1", "card": "c-1", "role": "captor", "kind": "person", "layer": 4,
        "needs": [
            {"need": "standing", "min": "Middle", "max": "High"},
            {"need": "trait", "name": "lawless"},
            {"need": "link", "type": "creditor/debtor", "as": "creditor", "to_role": "requester"},
            {"need": "where", "rel": "near", "of_place": "pl-saltmarket"},
            {"need": "free"}
        ],
        "prefers": [{"need": "trait", "name": "wealthy"}]
    }"#;
    let s: Spec = serde_json::from_str(text).unwrap();
    assert_eq!(s.needs.len(), 5);
    assert!(matches!(s.needs[0], Need::Standing { min: Standing::Middle, max: Standing::High }));
    assert!(matches!(s.needs[4], Need::Free));
    assert!(s.filled_by.is_none());
}
