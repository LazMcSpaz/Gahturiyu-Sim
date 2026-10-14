//! Authored people, stage 1 (`roster.rs`, `docs/authored-people.md`): a
//! town with a roster file gets exactly its people, with their stored
//! fields; a town without one is as before; a roster reads back as written.
//! The roster here is a throwaway for the tests, not anyone's town.

use gahturiyu_sim::sim::{
    atlas,
    jobs::Job,
    race::Race,
    roster::{Authored, Roster, Rosters, Wealth},
    worldgen, World,
};

/// A small made-up roster for the village of the fifth mark.
const TEST_ROSTER: &str = r#"(
    town: "imperial_village",
    ruler: Some("test_reeve"),
    characters: [
        (
            key: "test_reeve",
            name: "Test Reeve",
            people: Qotiro,
            sex: Woman,
            profession: Official,
            faction: Some("throne"),
            wealth: Rich,
            traits: (boldness: Some(0.9), honour: Some(0.15), asks: Some(0.7)),
            home: Some(1),
            relations: [ (to: "test_trader", kind: Rival) ],
            background: "Came from the capital with a grudge.",
            importance: "Only a test.",
            schedule: [ (from: 19.0, to: 23.0, at: Hearth) ],
            appearance: (style: "court", colours: ["grey"], marks: ["a scar"]),
            lines: Some("test_reeve"),
        ),
        (
            key: "test_trader",
            name: "Test Trader",
            people: Horaro,
            sex: Man,
            profession: Merchant,
            wealth: Destitute,
        ),
    ],
    crowd_rows: [ (role: Farmer, count: 3, mix: None) ],
    crowd: [
        (key: "imperial_village.0.0", name: "Crowd One", people: Roduro, sex: Man, profession: Farmer, household: 0, detail: "keeps bees"),
        (key: "imperial_village.0.1", name: "Crowd Two", people: Roduro, sex: Woman, profession: Farmer, household: 0, traits: (patience: Some(0.05))),
        (key: "imperial_village.0.2", name: "Crowd Three", people: Tadoro, sex: Woman, profession: Farmer, household: 1, wealth: Poor),
    ],
)"#;

fn town_index(key: &str) -> usize {
    atlas::town_marks().iter().position(|m| m.key() == key).expect("the mark")
}

fn with_roster() -> (World, Roster) {
    let r = Roster::from_ron(TEST_ROSTER).expect("the test roster reads");
    let mut rs = Rosters::default();
    rs.towns.insert(r.town.clone(), r.clone());
    (worldgen::generate_with_rosters(1, &rs), r)
}

#[test]
fn every_mark_has_a_stable_key_and_no_two_alike() {
    let keys: Vec<String> = atlas::town_marks().iter().map(|m| m.key()).collect();
    assert!(keys.iter().all(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')), "{keys:?}");
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), keys.len(), "keys are unique");
    assert_eq!(gahturiyu_sim::sim::roster::key_of_label("Imperial capital"), "imperial_capital");
}

#[test]
fn without_rosters_the_world_is_as_before() {
    let a = worldgen::generate(1);
    let b = worldgen::generate_with_rosters(1, &Rosters::default());
    assert_eq!(format!("{:?}", a.people), format!("{:?}", b.people));
    assert_eq!(a.settlements.iter().map(|s| s.residents.clone()).collect::<Vec<_>>(), b.settlements.iter().map(|s| s.residents.clone()).collect::<Vec<_>>());
    assert!(a.people.iter().all(|p| p.authored.is_none()), "nobody authored");
}

#[test]
fn a_town_with_a_roster_has_exactly_its_people_with_their_fields() {
    let (w, r) = with_roster();
    let t = town_index("imperial_village");
    let residents = &w.settlements[t].residents;
    assert_eq!(residents.len(), r.len(), "exactly the roster's people");
    assert!(w.people.iter().filter(|p| p.home == Some(t as u16)).all(|p| p.authored.is_some()), "nobody seeded lives there");
    let by_key = |k: &str| *residents.iter().find(|&&p| w.people[p as usize].authored.as_ref().is_some_and(|a| a.key() == k)).unwrap_or_else(|| panic!("{k} lives there"));
    // The reeve: every fact as written.
    let reeve = by_key("test_reeve");
    let p = &w.people[reeve as usize];
    assert_eq!(p.given_name(), "Test Reeve");
    assert_eq!(w.name_of(reeve), "Test Reeve");
    assert_eq!(p.race, Race::Qotiro);
    assert_eq!(gahturiyu_sim::sim::names::gender(p.seed), gahturiyu_sim::names::Gender::Female, "her seed reads as a woman (names, offices)");
    assert!((p.traits.boldness - 0.9).abs() < 1e-6, "a trait given overrides the roll");
    assert_eq!(w.life(reeve).job, Job::Official);
    assert!((w.life(reeve).habits.honour - 0.15).abs() < 1e-6);
    assert!((w.life(reeve).habits.asks - 0.7).abs() < 1e-6);
    assert_eq!(p.dwelling, Some(1), "the home given");
    match p.authored.as_deref() {
        Some(Authored::Character(c)) => assert_eq!(c, &r.characters[0], "every stored fact kept"),
        other => panic!("{other:?}"),
    }
    // Rich: the household starts with a month's keep and more.
    let h = w.life(reeve).household.unwrap();
    assert!(w.society.households[h as usize].purse.coin >= w.daily_cost(h) * Wealth::Rich.days() - 1e-3);
    // The trader: a man, a merchant.
    let trader = by_key("test_trader");
    assert_eq!(gahturiyu_sim::sim::names::gender(w.people[trader as usize].seed), gahturiyu_sim::names::Gender::Male);
    assert_eq!(w.life(trader).job, Job::Merchant);
    // The crowd, as stored.
    for c in &r.crowd {
        let id = by_key(&c.key);
        let p = &w.people[id as usize];
        assert_eq!(p.given_name(), c.name);
        assert_eq!(p.race, c.people);
        assert_eq!(w.life(id).job, Job::Farmer);
        assert!(p.authored.as_ref().unwrap().is_crowd());
    }
    let two = by_key("imperial_village.0.1");
    assert!((w.people[two as usize].traits.patience - 0.05).abs() < 1e-6);
}

#[test]
fn the_rest_of_the_world_keeps_its_seeded_people() {
    let (w, r) = with_roster();
    let plain = worldgen::generate(1);
    // As many seeded townsfolk and wanderers as before (those bound for the
    // village live elsewhere), plus the roster's.
    let seeded = |w: &World| w.people.iter().filter(|p| p.authored.is_none() && !p.in_squad && !p.bandit).count();
    assert_eq!(seeded(&w), seeded(&plain));
    assert_eq!(w.people.iter().filter(|p| p.authored.is_some()).count(), r.len());
}

#[test]
fn wanderers_from_their_roster_belong_nowhere() {
    let r = Roster::from_ron(r#"(characters: [ (key: "test_walker", name: "Test Walker", people: Tadoro, sex: Man, profession: Drifter) ])"#).unwrap();
    let rs = Rosters { wanderers: Some(r), ..Default::default() };
    let w = worldgen::generate_with_rosters(1, &rs);
    let id = w.people.iter().position(|p| p.authored.as_ref().is_some_and(|a| a.key() == "test_walker")).expect("in the world");
    assert_eq!(w.people[id].home, None);
    assert!(w.group_of[id].is_some(), "on the road");
}

#[test]
fn a_roster_round_trips_through_its_file_unchanged() {
    let r = Roster::from_ron(TEST_ROSTER).unwrap();
    let text = r.to_ron();
    assert_eq!(Roster::from_ron(&text).unwrap(), r, "written and read back, the same");
    assert_eq!(text, Roster::from_ron(&text).unwrap().to_ron(), "the same text again");
    let dir = std::env::temp_dir().join(format!("gaht-roster-{}", std::process::id()));
    let path = dir.join("people").join("imperial_village.ron");
    r.save(&path).unwrap();
    assert_eq!(Roster::load(&path).unwrap(), r);
    // And a folder of them reads as the rosters, by file name.
    let rs = Rosters::load_from(&dir.join("people"));
    assert_eq!(rs.for_town("imperial_village"), Some(&r));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn authored_people_save_and_load() {
    let (w, _) = with_roster();
    let back = World::load_bytes(&w.save_bytes()).expect("loads");
    assert_eq!(format!("{:?}", w.people), format!("{:?}", back.people));
}
