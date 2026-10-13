//! The Binders' slave raids (docs/factions.md, F6).

use gahturiyu_sim::sim::{
    binders::{Captive, BINDER_CAMPS, CAPTIVE_DAYS},
    geo::V2,
    items,
    race::Race,
    stats::SKILLS,
    world::{DAY, HOUR},
    worldgen, World,
};

fn coin_of(w: &World, who: &[u32]) -> u32 {
    who.iter().map(|&m| w.count_of(m, "coin") as u32).sum()
}

/// A feeble squad with coin, beaten by a hard band of slavers.
fn beaten(seed: u64) -> (World, Vec<u32>, u32) {
    let mut w = worldgen::generate(seed);
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 50);
        w.people[m as usize].recompute_might();
    }
    let band = w.spawn_binders(w.squad.pos.add(V2::new(10.0, 0.0)), 6);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none() && w.squad_fit().is_empty(), "the squad lost");
    (w, squad, band)
}

#[test]
fn slavers_camp_by_the_roads_and_are_mostly_qotiro() {
    let w = worldgen::generate(1);
    let camps: Vec<_> = w.camps.iter().filter(|c| c.slavers).collect();
    assert_eq!(camps.len(), BINDER_CAMPS);
    let people: Vec<u32> = camps.iter().flat_map(|c| w.group(c.group).unwrap().members.clone()).collect();
    let qotiro = people.iter().filter(|&&p| w.people[p as usize].race == Race::Qotiro).count();
    assert!(qotiro * 2 > people.len(), "mostly Qotiro ({qotiro} of {})", people.len());
    // The bandit camps are as they were.
    assert!(w.camps.iter().filter(|c| !c.slavers).count() >= 8);
}

#[test]
fn beaten_by_slavers_the_downed_are_carried_off_not_robbed() {
    let (w, squad, band) = beaten(1);
    let alive: Vec<u32> = squad.iter().copied().filter(|&m| !w.people[m as usize].dead).collect();
    let held: Vec<&Captive> = w.captives.iter().collect();
    assert!(!held.is_empty(), "someone taken");
    assert!(held.iter().all(|c| c.camp == band));
    assert_eq!(coin_of(&w, &alive), alive.len() as u32 * 50, "they take people, not purses");
    for c in &held {
        assert!(!w.free_to_order(c.who), "a captive takes no orders");
        let camp = w.camps.iter().find(|x| x.group == band).unwrap().pos;
        assert!(w.person_pos(c.who).dist(camp) < 20.0, "held at their camp");
    }
    assert!(w.log.iter().any(|l| l.1.contains("carry") && l.1.contains("off to their camp")));
}

#[test]
fn held_long_enough_a_captive_is_sold_or_slips_away() {
    let (mut w, _, _) = beaten(2);
    let first: Vec<(u32, f64)> = w.captives.iter().map(|c| (c.who, c.since)).collect();
    let held: Vec<u32> = first.iter().map(|c| c.0).collect();
    assert!(!held.is_empty());
    let end = w.time + (CAPTIVE_DAYS + 1.5) * DAY;
    while w.time < end {
        w.step(600.0);
    }
    for &m in &held {
        // (This captivity: once sold on, a member who wanders back by the
        // camp may be taken again, and that's another.)
        let since = first.iter().find(|c| c.0 == m).unwrap().1;
        let gone = !w.captives.iter().any(|c| c.who == m && c.since == since);
        let sold = w.bond_of(m).is_some_and(|b| b.slave || b.until > w.time + 5.0 * DAY);
        let slipped = w.log.iter().any(|l| l.1.contains("slips away")) || w.alerts.iter().any(|a| a.contains("slips away"));
        assert!(gone, "not held for ever");
        assert!(sold || slipped || w.slave_market(w.person_pos(m)).is_none(), "sold into bondage, or got away");
    }
}

#[test]
fn beating_the_band_frees_everyone_they_hold() {
    let mut w = worldgen::generate(3);
    let squad = w.squad.members.clone();
    let band = w.spawn_binders(w.squad.pos.add(V2::new(12.0, 0.0)), 1);
    let foe = w.group(band).unwrap().members[0];
    for k in SKILLS {
        w.people[foe as usize].stats.set_skill(k, 1.0);
    }
    w.people[foe as usize].traits.boldness = 1.0;
    w.people[foe as usize].recompute_might();
    // One of the squad already held there (taken earlier).
    let held = squad[3];
    w.captives.push(Captive { who: held, camp: band, since: w.time });
    assert!(!w.free_to_order(held));
    let fighters: Vec<u32> = squad[..3].to_vec();
    assert!(w.attack(&fighters, foe));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.captive_of(held).is_none(), "free");
    assert!(w.free_to_order(held));
    let _ = HOUR;
}

/// A captive coming round by their captors' fire isn't set on by them
/// again (found in a visual check: they were attacked, beaten and carried
/// off over and over).
#[test]
fn captors_leave_their_captive_alone() {
    let mut w = worldgen::generate(4);
    let at = w.squad.pos.add(V2::new(200.0, 60.0));
    let band = w.spawn_binders(at, 4);
    let held = w.squad.members[3];
    w.captives.push(Captive { who: held, camp: band, since: w.time });
    let fire = w.free_spot(at.add(V2::new(4.0, 2.0)));
    w.squad.at[3] = fire;
    w.squad.goal[3] = fire;
    w.squad.route[3].clear();
    for _ in 0..600 {
        w.step(0.5);
    }
    assert!(w.squad_battle().is_none(), "no fight with their own captive");
    assert!(!w.is_down(held), "left standing by the fire");
    assert_eq!(w.captive_of(held).map(|c| c.camp), Some(band));
}
