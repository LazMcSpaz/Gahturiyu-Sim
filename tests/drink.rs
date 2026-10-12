//! Drink (the items design, Part 5 rule 5 and "Alcohol"): drunkenness on the
//! condition timeline, its stages acting through the condition, a hangover
//! after, one small effect a drink.

use gahturiyu_sim::sim::{
    drunk::{self, Drunk, DRUNK_DRAIN, HANGOVER_HOURS},
    effects::Does,
    items,
    stats::{Attr, Skill},
    world::HOUR,
    worldgen, World,
};

fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

/// Member 0 with `n` of a drink, nothing else to drink.
fn with(seed: u64, key: &str, n: u16) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
    w.step(1.0);
    (w, m)
}

fn stage(w: &World, m: u32) -> Drunk {
    Drunk::of(w.drunk_of(m))
}

#[test]
fn each_drink_a_jump_and_it_drains_over_hours() {
    let (mut w, m) = with(1, "stone_spirit", 3);
    assert_eq!(stage(&w, m), Drunk::Sober);
    assert!(w.drink(m, items::id("stone_spirit")));
    assert_eq!(stage(&w, m), Drunk::Tipsy);
    assert!(w.drink(m, items::id("stone_spirit")));
    assert_eq!(stage(&w, m), Drunk::VeryDrunk, "two spirits: very drunk");
    let peak = w.drunk_of(m);
    wait(&mut w, 2.0, 60.0);
    let later = w.drunk_of(m);
    assert!((peak - later - 2.0 * DRUNK_DRAIN).abs() < 0.5, "drains at its rate ({peak:.1} -> {later:.1})");
    assert_eq!(w.count_of(m, "stone_spirit"), 1);
}

#[test]
fn drunk_is_clumsy_and_poor_company_and_tipsy_is_merry() {
    let (mut w, m) = with(2, "moorberry_wine", 3);
    let base = w.people[m as usize].effective_stats();
    w.drink(m, items::id("moorberry_wine"));
    assert_eq!(stage(&w, m), Drunk::Tipsy);
    assert!(w.drunk_talk(m) > 0.0, "merry");
    w.drink(m, items::id("moorberry_wine"));
    assert_eq!(stage(&w, m), Drunk::Drunk);
    assert!(w.drunk_talk(m) < 0.0, "poor company");
    let s = w.people[m as usize].effective_stats();
    assert!(s.attr(Attr::Agility) < base.attr(Attr::Agility) - 5.0, "clumsier");
    assert!(s.skill(Skill::Sneak) < base.skill(Skill::Sneak) - 5.0 || base.skill(Skill::Sneak) < 6.0, "sneaks worse");
}

#[test]
fn sobering_up_from_drunk_leaves_a_hangover_on_the_clock() {
    let (mut w, m) = with(3, "stone_spirit", 2);
    w.drink(m, items::id("stone_spirit"));
    w.drink(m, items::id("stone_spirit"));
    let d = w.drunk_of(m);
    let sober_in = d as f64 / DRUNK_DRAIN as f64;
    wait(&mut w, sober_in + 0.2, 300.0);
    assert_eq!(stage(&w, m), Drunk::Sober);
    assert!(w.hungover(m), "a sore head");
    assert!(w.log.iter().any(|l| l.1.contains("sore head")));
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!(c.hungover, "the condition knows (breath back slower)");
    wait(&mut w, HANGOVER_HOURS, 300.0);
    assert!(!w.hungover(m), "over");
    assert!(!w.people[m as usize].cond.as_ref().unwrap().hungover);
}

#[test]
fn when_it_wears_off_doesnt_depend_on_the_step() {
    let run = |step: f64| {
        let (mut w, m) = with(4, "stone_spirit", 2);
        w.drink(m, items::id("stone_spirit"));
        w.drink(m, items::id("stone_spirit"));
        wait(&mut w, 15.0, step);
        let c = w.people[m as usize].cond.as_ref().unwrap();
        (c.hangover_until, (w.drunk_of(m) * 100.0).round())
    };
    let (a, b) = (run(30.0), run(900.0));
    assert!((a.0 - b.0).abs() < 1e-3, "{a:?} vs {b:?}");
    assert_eq!(a.1, b.1);
}

#[test]
fn one_drink_effect_at_a_time_and_spirits_add_toxicity() {
    let (mut w, m) = with(5, "ashberry_wine", 1);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("stone_spirit"), 1);
    w.drink(m, items::id("ashberry_wine"));
    assert!(w.boons_on(m).iter().any(|b| b.does == Does::Warmth));
    let tox = w.toxicity_of(m).unwrap_or(0.0);
    w.drink(m, items::id("stone_spirit"));
    assert!(!w.boons_on(m).iter().any(|b| b.does == Does::Warmth), "the wine's warmth replaced");
    assert!(w.boons_on(m).iter().any(|b| b.does == Does::Steady), "by the spirit's steadiness");
    assert!(w.toxicity_of(m).unwrap_or(0.0) > tox, "a little toxicity");
}

#[test]
fn ale_quenches_and_spirits_parch() {
    let (mut w, m) = with(6, "hardgrain_ale", 1);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("stone_spirit"), 1);
    w.people[m as usize].cond.as_mut().unwrap().thirst = 40.0;
    w.drink(m, items::id("hardgrain_ale"));
    let after_ale = w.people[m as usize].cond.as_ref().unwrap().thirst;
    assert!(after_ale < 35.0);
    w.drink(m, items::id("stone_spirit"));
    assert!(w.people[m as usize].cond.as_ref().unwrap().thirst > after_ale + 5.0);
}

#[test]
fn very_drunk_stumbles_and_merchants_sell_drink() {
    let (mut w, m) = with(7, "stone_spirit", 3);
    let pace = w.member_speed(m);
    for _ in 0..3 {
        w.drink(m, items::id("stone_spirit"));
    }
    assert_eq!(stage(&w, m), Drunk::VeryDrunk);
    assert!(w.member_speed(m) < pace * 0.9, "stumbling");
    assert!(gahturiyu_sim::sim::jobs::good_of("hardgrain_ale").is_some());
    assert!(gahturiyu_sim::sim::jobs::good_of("sacrament_spirit").is_none(), "never sold");
    let _ = drunk::PASS_OUT;
}
