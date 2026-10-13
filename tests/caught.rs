//! The caught path on Salt Isle (docs/tutorial-island.md, Path A): taken for
//! a runaway, judged by the lord, the bond, the yard, bought out.

use gahturiyu_sim::sim::{
    caught::AT_DOCK,
    tutorial::Deed,
    dialogue::Topic,
    factions,
    geo::V2,
    island::Spot,
    tutorial::Path,
    world::{DAY, HOUR},
    worldgen, World,
};

/// The opening's world with the shared lessons done.
fn opened(seed: u64) -> World {
    let mut w = worldgen::generate_opening(seed, None, Default::default());
    for k in ["wake", "salvage", "shelter", "cook", "questions"] {
        w.lesson_done(k);
    }
    w
}

fn run_to_hour(w: &mut World, h: f64) {
    let now = w.time.rem_euclid(DAY) / HOUR;
    let wait = (h - now).rem_euclid(24.0);
    let end = w.time + wait * HOUR;
    while w.time < end {
        w.step(60.0);
    }
}

/// Walk in by day: taken.
fn taken(seed: u64) -> (World, u32) {
    let mut w = opened(seed);
    run_to_hour(&mut w, 11.0);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    w.teleport_squad(isle.spot(Spot::Yard));
    w.step(0.5);
    (w, m)
}

#[test]
fn walking_into_the_compound_by_day_you_are_taken_and_judged() {
    let (w, m) = taken(3);
    assert_eq!(w.tutorial().unwrap().path, Some(Path::Caught));
    let b = w.bond_of(m).expect("bound");
    assert!(b.slave && b.holder == Some(w.island().unwrap().steward));
    assert!(!w.free_to_order(m), "a slave takes no orders off the island");
    // Before the lord, who speaks first.
    let c = w.talk.as_ref().expect("the lord speaks");
    assert!(w.is_lord(c.npc));
    assert!(c.lines[0].1.contains("mine"), "{}", c.lines[0].1);
    assert!(w.person_pos(m).dist(w.person_pos(c.npc)) < 6.0, "stood before him");
}

#[test]
fn by_night_or_in_a_smock_nobody_takes_you() {
    let mut w = opened(3);
    run_to_hour(&mut w, 23.0);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    w.teleport_squad(isle.spot(Spot::Yard));
    w.step(0.5);
    assert!(w.bond_of(m).is_none(), "the dark keeps them");
}

#[test]
fn asking_what_is_owed_is_the_bond_lesson() {
    let (mut w, m) = taken(3);
    assert!(w.topics().contains(&Topic::Owe));
    let said = w.ask(Topic::Owe).unwrap();
    assert!(said.contains("ledger") || said.contains("debt"), "{said}");
    assert!(w.tutorial().unwrap().is_done("bond"));
    let _ = m;
}

#[test]
fn the_yard_bout_is_fought_out_and_nobody_pays() {
    let (mut w, m) = taken(3);
    w.end_talk();
    for k in ["labour", "workyard", "bond"] {
        w.lesson_done(k);
    }
    assert_eq!(w.lesson_now().map(|x| x.key), Some("yard"));
    let isle = w.island().unwrap().clone();
    w.teleport_squad(isle.spot(Spot::Yard));
    let bound = w.bond_of(m).copied();
    let mut n = 0;
    while !w.tutorial().unwrap().is_done("yard") && n < 40_000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.tutorial().unwrap().is_done("yard"), "the bout was fought");
    assert!(w.log.iter().any(|l| l.1.contains("into the yard")), "{:?}", w.log);
    assert_eq!(w.bond_of(m).copied(), bound, "the bout changes nothing of the bond");
    assert!(!w.people[m as usize].dead);
}

#[test]
fn bought_out_by_the_company_for_a_fighter_or_the_writers_for_a_head() {
    let (mut w, m) = taken(3);
    w.end_talk();
    // A fighter.
    w.island_deed(Deed::FightWon);
    w.island_deed(Deed::FightWon);
    assert_eq!(w.buyer(), factions::id("lantern_company"));
    // A head: more made than fought.
    let mut v = w.clone();
    for _ in 0..9 {
        v.island_deed(Deed::Made);
    }
    assert_eq!(v.buyer(), factions::id("writers"));
    // The lessons before it done: the buyer comes.
    for k in ["labour", "workyard", "bond", "yard"] {
        w.lesson_done(k);
    }
    assert_eq!(w.lesson_now().map(|x| x.key), Some("bought"));
    w.step(0.5);
    assert!(w.bond_of(m).is_none(), "free");
    assert!(w.in_guild(factions::id("lantern_company")), "counted among the company");
    assert_eq!(w.caught.owes.map(|o| o.0), Some(factions::id("lantern_company")));
    assert!(!w.tutorial().unwrap().is_done("bought"), "not until the dock");
    let dock = w.island().unwrap().spot(Spot::Dock);
    w.teleport_squad(dock.add(V2::new(AT_DOCK * 0.5, 0.0)));
    w.step(0.5);
    assert!(w.tutorial().unwrap().is_done("bought"));
    assert_eq!(w.lesson_now().map(|x| x.key), Some("sail"));
}
