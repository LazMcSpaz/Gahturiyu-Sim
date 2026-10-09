//! The promises the band system makes, checked.
//!
//! 1. How finely the world is stepped does not change what happens.
//! 2. Where the squad stands (and so which band everything is in) does not
//!    change what happens either.
//! 3. Details, once built, are never rebuilt or changed.
//! 4. Details are only built for what actually came close.
//!
//! "What happens" includes the world's own fights: bandits ambushing
//! travellers far from the squad must come out the same either way.

use gahturiyu_sim::sim::{geo::V2, world::HOUR, worldgen, World};

fn run(mut w: World, hours: f64, step: f64) -> World {
    let steps = (hours * HOUR / step).round() as usize;
    for _ in 0..steps {
        w.step(step);
    }
    w
}

/// Everything that counts as "what happened", ignoring cached views.
fn assert_same_history(a: &World, b: &World) {
    assert!((a.time - b.time).abs() < 1e-6, "clocks differ: {} vs {}", a.time, b.time);
    assert_eq!(a.hour_done, b.hour_done);
    assert_eq!(a.groups.len(), b.groups.len(), "different number of groups on the road");
    for (ga, gb) in a.groups.iter().zip(&b.groups) {
        assert_eq!(ga.id, gb.id);
        assert_eq!(ga.members, gb.members);
        assert_eq!(ga.kind, gb.kind);
        assert_eq!(ga.legs, gb.legs, "group {} took a different route", ga.id);
        let (pa, pb) = (ga.position_at(a.time), gb.position_at(b.time));
        assert!(pa.dist(pb) < 0.01, "group {} is in a different place", ga.id);
    }
    assert_eq!(a.busy_until, b.busy_until);
    assert_eq!(a.group_of, b.group_of);
    assert_eq!(a.stats.ambushes, b.stats.ambushes, "a different number of ambushes");
    for (pa, pb) in a.people.iter().zip(&b.people) {
        assert_eq!(pa.dead, pb.dead, "person {} died in one run only", pa.id);
        let (la, lb) = (pa.wounds.lost_at(a.time), pb.wounds.lost_at(b.time));
        for k in 0..6 {
            assert!((la[k] - lb[k]).abs() < 1e-3, "person {} has different wounds", pa.id);
        }
        assert!((pa.might - pb.might).abs() < 1e-3, "person {} has a different might", pa.id);
    }
    // Towns: the same customs, the same work, the same goods and money.
    let (sa, sb) = (&a.society, &b.society);
    assert_eq!(sa.lives, sb.lives, "someone's job or household differs");
    for (ca, cb) in sa.communities.iter().zip(&sb.communities) {
        assert_eq!(ca.customs, cb.customs, "a community chose different customs");
        assert_eq!(ca.food, cb.food, "a community ate differently");
    }
    for (ta, tb) in sa.towns.iter().zip(&sb.towns) {
        for (x, y) in ta.stock.iter().zip(&tb.stock) {
            assert!((x.base - y.base).abs() < 1e-3 && (x.rate - y.rate).abs() < 1e-4, "a town's stockpile differs");
        }
        assert!((ta.treasury - tb.treasury).abs() < 1e-3 && (ta.purse - tb.purse).abs() < 1e-3 && ta.purse_at == tb.purse_at, "a town's money differs");
        assert!(ta.owed == tb.owed && ta.prosperity == tb.prosperity && ta.purse_cap == tb.purse_cap && ta.purse_rate == tb.purse_rate && ta.landed == tb.landed, "a town's books differ");
        assert_eq!(ta.places, tb.places, "a town laid out its workplaces differently");
        assert_eq!(ta.gardens, tb.gardens, "a town's gardens are tended by someone else");
        // The same crafters made the same things, at the same grades, with the same marks.
        assert_eq!(ta.making, tb.making, "a town's crafters are at different work");
        assert_eq!(ta.shelf, tb.shelf, "a town's shelf holds different things");
        // The same rulers, rites, unrest and verdicts.
        assert_eq!(ta.gov, tb.gov, "a town is governed differently");
    }
    assert_eq!(sa.renown, sb.renown, "makers are known differently");
    assert_eq!(a.bonds, b.bonds, "different bonds");
    assert_eq!(a.records, b.records, "a different public record");
    assert_eq!(sa.households, sb.households, "households formed (or spent) differently");
    assert_eq!(sa.minds, sb.minds, "someone's work, needs, memories or knowledge differ");
    assert_eq!(sa.history, sb.history, "the towns' histories differ");
    assert_eq!(a.stats.caravans, b.stats.caravans, "a different number of caravans");
}

#[test]
fn step_size_does_not_change_history() {
    let fine = run(worldgen::generate(7), 48.0, 1.0);
    let coarse = run(worldgen::generate(7), 48.0, HOUR);
    let lumpy = run(worldgen::generate(7), 48.0, 337.5);
    assert_same_history(&fine, &coarse);
    assert_same_history(&fine, &lumpy);
    assert!(fine.stats.journeys_started > 50, "the world should actually be doing something");
    assert!(fine.stats.ambushes > 5, "bandits should be busy: {}", fine.stats.ambushes);
}

#[test]
fn step_size_does_not_change_history_in_other_worlds() {
    // The default world and one more, so a bug that hides in one seed's
    // particular timing still gets caught.
    for seed in [1, 23] {
        let fine = run(worldgen::generate(seed), 30.0, 2.0);
        let coarse = run(worldgen::generate(seed), 30.0, HOUR);
        assert_same_history(&fine, &coarse);
    }
}

#[test]
fn lives_come_out_the_same_over_days_in_big_steps() {
    // Purses, memories, grudges and gossip are settled at dawn: a world
    // stepped six hours at a time must have the same lives as one stepped
    // by the hour.
    let fine = run(worldgen::generate(3), 6.0 * 24.0, HOUR);
    let coarse = run(worldgen::generate(3), 6.0 * 24.0, 6.0 * HOUR);
    assert_same_history(&fine, &coarse);
    assert!(fine.society.minds.iter().any(|m| !m.memories.is_empty() && !m.knows.is_empty()));
}

#[test]
fn the_squads_position_does_not_change_history() {
    let here = run(worldgen::generate(11), 36.0, 2.0);
    let mut elsewhere = worldgen::generate(11);
    // Teleport the squad to the far corner, so different groups are in each band.
    elsewhere.teleport_squad(V2::new(18_000.0, 18_000.0));
    let elsewhere = run(elsewhere, 36.0, 2.0);
    assert_same_history(&here, &elsewhere);
}

#[test]
fn details_once_built_are_kept() {
    let mut w = worldgen::generate(3);
    let start = w.squad.pos;
    w = run(w, 2.0, 1.0);
    // (Not the squad: they eat, pick things up and so on.)
    let seen: Vec<_> = w.people.iter().filter(|p| !p.in_squad).filter_map(|p| p.detail.clone().map(|d| (p.id, d))).collect();
    assert!(!seen.is_empty());

    // Walk far away, wait, and come back.
    w.order_squad(V2::new(15_000.0, 10_000.0));
    w = run(w, 30.0, 5.0);
    w.order_squad(start);
    w = run(w, 30.0, 5.0);

    // Never rebuilt or rerolled. (What strangers use up in a fight they
    // restock at home, so their packs don't change either.)
    for (id, d) in seen {
        let now = w.people[id as usize].detail.as_ref().expect("details kept");
        assert_eq!(now.name, d.name, "person {id} was renamed");
        assert_eq!(now.spells, d.spells, "person {id}'s spells changed");
        for s in gahturiyu_sim::sim::items::SLOTS {
            assert_eq!(now.gear.in_slot(s), d.gear.in_slot(s), "person {id}'s gear changed while out of sight");
        }
        for e in &now.gear.bag {
            let before = d.gear.bag.iter().find(|x| x.0 == e.0).map(|x| x.1).unwrap_or(0);
            assert!(e.1 <= before, "person {id} gained things while out of sight");
        }
    }
}

#[test]
fn details_are_only_built_for_what_came_close() {
    let w = run(worldgen::generate(5), 24.0, 2.0);
    let detailed = w.people.iter().filter(|p| p.detail.is_some()).count();
    assert!(
        detailed < w.people.len() / 4,
        "{detailed} of {} people got details without the squad going anywhere",
        w.people.len()
    );
    assert_eq!(detailed, w.stats.detailed, "the running count drifted from the truth");
}

#[test]
fn a_roadside_fight_you_watch_is_the_one_you_would_have_missed() {
    // Stand near a bandit camp (close enough to watch, too far to be jumped)
    // in one run, and on the far side of the world in the other.
    let mut near = worldgen::generate(7);
    let camp = near.camps[0].pos;
    near.teleport_squad(camp.add(V2::new(160.0, 0.0)));
    let mut far = worldgen::generate(7);
    far.teleport_squad(V2::new(1_000.0, 20_000.0));
    let (near, far) = (run(near, 30.0, 2.0), run(far, 30.0, 2.0));
    assert_eq!(near.next_battle, near.stats.ambushes as u32, "the squad shouldn't have been in a fight itself");
    assert_same_history(&near, &far);
}

#[test]
fn a_far_fight_with_a_mage_in_it_is_the_one_you_would_have_watched() {
    // A band with a mage (who calls up help and casts) camped by a road; the
    // squad near it in one run, far away in the other.
    let camp_at = |w: &World| w.camps[1].pos.add(V2::new(30.0, 30.0));
    let mut near = worldgen::generate(7);
    let at = camp_at(&near);
    near.spawn_bandits(at, 3, true);
    near.teleport_squad(at.add(V2::new(170.0, 0.0)));
    let mut far = worldgen::generate(7);
    far.spawn_bandits(at, 3, true);
    far.teleport_squad(V2::new(1_000.0, 20_000.0));
    let (near, far) = (run(near, 30.0, 2.0), run(far, 30.0, 2.0));
    assert_same_history(&near, &far);
}
