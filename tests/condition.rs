//! Hunger (and the rest of how the squad holds up), worked out from the clock.

use gahturiyu_sim::sim::{
    body,
    condition::{Activity, Condition, HungerStage, EAT_AT, HUNGER_PER_HOUR},
    items::{self, item, Kind},
    person::PersonId,
    stats::Attr,
    world::HOUR,
    worldgen, World,
};

fn run(w: &mut World, hours: f64, step: f64) {
    let n = (hours * HOUR / step).round() as usize;
    for _ in 0..n {
        w.step(step);
    }
}

fn food_count(w: &World, pid: PersonId) -> u16 {
    w.people[pid as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Food(_))).map(|e| e.1).sum()
}

fn take_all_food(w: &mut World, pid: PersonId) {
    let d = w.people[pid as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| !matches!(item(e.0).kind, Kind::Food(_)));
}

/// The mage: lightly loaded, so no load surcharge on hunger.
fn mage(w: &World) -> PersonId {
    w.squad.members[3]
}

#[test]
fn hunger_rises_with_the_clock() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    take_all_food(&mut w, m);
    let start = w.hunger_of(m).unwrap();
    run(&mut w, 10.0, 60.0);
    let now = w.hunger_of(m).unwrap();
    assert!((now - start - HUNGER_PER_HOUR * 10.0).abs() < 0.01, "{start} -> {now}");
}

#[test]
fn walking_load_and_wounds_make_you_hungrier() {
    let base = Condition::new(0.0);
    let mut walking = base.clone();
    walking.activity = Activity::Walking;
    let mut loaded = walking.clone();
    loaded.load = 1.4;
    let mut hurt = loaded.clone();
    hurt.wounded = true;
    let mut asleep = base.clone();
    asleep.activity = Activity::Sleeping;
    assert!(walking.hunger_rate() > base.hunger_rate());
    assert!(loaded.hunger_rate() > walking.hunger_rate());
    assert!(hurt.hunger_rate() > loaded.hunger_rate());
    assert!(asleep.hunger_rate() < base.hunger_rate());
}

#[test]
fn members_eat_from_their_pack_when_hungry() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    let food = food_count(&w, m);
    assert!(food > 0);
    run(&mut w, 20.0, 60.0);
    assert_eq!(food_count(&w, m), food - 1, "one meal by now");
    assert!(w.hunger_of(m).unwrap() < EAT_AT);
}

#[test]
fn hunger_and_meals_dont_depend_on_step_size() {
    let make = || {
        let mut w = worldgen::generate(2);
        // Someone hurt, someone with no food, so every stage gets exercised.
        let a = w.squad.members[0];
        let t = w.time;
        let p = &mut w.people[a as usize];
        p.wounds.lost = [5.0, 30.0, 10.0, 0.0, 0.0, 0.0];
        p.wounds.at = t;
        let b = w.squad.members[1];
        take_all_food(&mut w, b);
        w
    };
    let (mut fine, mut coarse) = (make(), make());
    run(&mut fine, 70.0, 7.0);
    run(&mut coarse, 70.0, HOUR);
    for &m in &fine.squad.members.clone() {
        let (pf, pc) = (&fine.people[m as usize], &coarse.people[m as usize]);
        assert!((fine.hunger_of(m).unwrap() - coarse.hunger_of(m).unwrap()).abs() < 1e-3, "member {m} hunger");
        assert_eq!(food_count(&fine, m), food_count(&coarse, m), "member {m} ate differently");
        let (lf, lc) = (pf.wounds.lost_at(fine.time), pc.wounds.lost_at(coarse.time));
        for k in 0..6 {
            assert!((lf[k] - lc[k]).abs() < 1e-3, "member {m} part {k}: {} vs {}", lf[k], lc[k]);
        }
    }
}

#[test]
fn going_hungry_weakens_then_wastes_but_never_kills() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    take_all_food(&mut w, m);
    let strength = w.people[m as usize].effective_stats().attr(Attr::Strength);
    // Weak from hunger after a day and a half or so.
    run(&mut w, 40.0, 120.0);
    assert_eq!(w.people[m as usize].cond.as_ref().unwrap().stage(), HungerStage::Weak);
    assert!(w.people[m as usize].effective_stats().attr(Attr::Strength) < strength * 0.9);
    // Starving: the torso wastes until they collapse.
    run(&mut w, 60.0, 120.0);
    let p = &w.people[m as usize];
    assert_eq!(p.cond.as_ref().unwrap().stage(), HungerStage::Starving);
    let hp = p.wounds.hp_at(&p.stats, w.time);
    assert!(body::knocked_out(&hp), "should have collapsed: torso {}", hp[1]);
    assert!(!body::dead(&hp, &p.stats) && !p.dead, "starving never kills");
    // Fed again, they come round.
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("salted_meat"), 3);
    run(&mut w, 30.0, 120.0);
    let p = &w.people[m as usize];
    assert!(!body::knocked_out(&p.wounds.hp_at(&p.stats, w.time)), "should be back on their feet");
}

#[test]
fn food_is_found_in_homes_and_on_the_land() {
    let w = worldgen::generate(1);
    let foods = w.nodes.iter().filter(|n| matches!(item(n.item).kind, Kind::Food(_))).count();
    assert!(foods > 20, "berries and mussels to gather: {foods}");
}
