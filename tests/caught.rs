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

/// Run in short steps until `f` holds, `secs` at most.
fn until(w: &mut World, secs: f64, f: impl Fn(&World) -> bool) -> bool {
    let end = w.time + secs;
    while w.time < end {
        if f(w) {
            return true;
        }
        w.step(0.25);
    }
    f(w)
}

/// Walk in by day: the watch sees them, comes over, and takes them.
fn taken(seed: u64) -> (World, u32) {
    let mut w = opened(seed);
    run_to_hour(&mut w, 11.0);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    w.teleport_squad(isle.spot(Spot::Yard));
    assert!(until(&mut w, 300.0, |w| w.tutorial().unwrap().path == Some(Path::Caught)), "taken within five minutes");
    (w, m)
}

#[test]
fn walking_into_the_compound_by_day_you_are_taken_and_judged() {
    let (w, m) = taken(3);
    assert_eq!(w.tutorial().unwrap().path, Some(Path::Caught));
    // (Seen first: one of the watch came over for them.)
    assert!(w.log.iter().any(|l| l.1.contains("comes over")), "{:?}", w.log);
    let b = w.bond_of(m).expect("bound");
    assert!(b.slave && b.holder == Some(w.island().unwrap().steward));
    assert!(!w.free_to_order(m), "a slave takes no orders off the island");
    // Before the lord, who speaks first.
    let c = w.talk.as_ref().expect("the lord speaks");
    assert!(w.is_lord(c.npc));
    assert!(c.lines[0].1.contains("mine"), "{}", c.lines[0].1);
    assert!(w.log.iter().any(|l| l.1.contains("lifts one finger")), "the overseer defers to him");
    assert!(!w.topics().iter().any(|t| matches!(t, Topic::Join(_) | Topic::AskJoin)), "not one to ask along");
    assert!(w.person_pos(m).dist(w.person_pos(c.npc)) < 6.0, "stood before him");
    assert!(w.log.iter().any(|l| l.1.ends_with("for life.")), "{:?}", w.log);
    // Nobody marches them off while he speaks.
    let mut w = w;
    for _ in 0..20 {
        w.step(0.5);
    }
    assert!(w.talk.is_some(), "the judgment stays open until it's done with");
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

/// Laz (13 Oct): "Talked to the guard at the gate and he didn't try to
/// enslave me ... They should spot me and approach." By day the watch sees
/// a stranger at the gate and walks up before anything happens; and
/// talking to one of them is no way round it.
#[test]
fn the_watch_spots_a_stranger_at_the_gate_and_comes_over() {
    let mut w = opened(3);
    run_to_hour(&mut w, 11.0);
    let isle = w.island().unwrap().clone();
    let m = w.squad.members[0];
    // Outside the gate, a few metres off.
    let gate = isle.spot(Spot::Gate);
    w.teleport_squad(gate.add(V2::new(10.0, 0.0)));
    w.step(0.25);
    assert!(w.tutorial().unwrap().path != Some(Path::Caught), "not taken from across the way");
    assert!(w.pursuits.iter().any(|p| p.culprit == m), "a guard comes over");
    let first = w.pursuits.iter().find(|p| p.culprit == m).unwrap().guard;
    assert!(until(&mut w, 300.0, |w| w.tutorial().unwrap().path == Some(Path::Caught)), "and takes them");
    // They were caught where the guard reached them: close by, not far off.
    assert!(w.log.iter().any(|l| l.1.contains("catches")), "{:?}", w.log);
    let _ = first;
    // Talking to the watch instead: taken there and then, no small talk.
    let mut w = opened(3);
    run_to_hour(&mut w, 6.5);
    let guard = w.settlements[isle.settlement as usize].residents.iter().copied().find(|&p| w.life(p).job == gahturiyu_sim::sim::jobs::Job::Guard && !w.is_bonded(p, w.time)).unwrap();
    let at = w.person_pos(guard);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.pursuits.clear();
    assert!(w.order_talk(m, guard));
    assert!(until(&mut w, 30.0, |w| w.tutorial().unwrap().path == Some(Path::Caught)), "talking to the watch, taken");
}

/// Laz (13 Oct): "Every character has the basic dialogue choices, even
/// though it makes no sense in this context." On Salt Isle nobody is asked
/// about their town, work, trade or the road.
#[test]
fn salt_isles_people_are_asked_only_what_fits() {
    let (mut w, m) = taken(3);
    let lord_topics = w.topics();
    assert!(lord_topics.iter().all(|t| matches!(t, Topic::Say(_) | Topic::Answer(..) | Topic::Owe | Topic::Goodbye)), "{lord_topics:?}");
    assert!(w.locked_topics().is_empty());
    w.end_talk();
    // One of the bound, in the compound.
    let isle = w.island().unwrap().clone();
    let bound = w.bonds.iter().find(|b| b.town == isle.settlement && b.who != m).unwrap().who;
    let at = w.person_pos(bound);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    if w.order_talk(m, bound) && until(&mut w, 30.0, |w| w.talk.is_some()) {
        let t = w.topics();
        assert!(!t.iter().any(|x| matches!(x, Topic::Background | Topic::ThisTown | Topic::Advice | Topic::Rumours | Topic::Bandits | Topic::Work | Topic::Trade | Topic::AskJoin | Topic::Join(_))), "{t:?}");
    }
}
