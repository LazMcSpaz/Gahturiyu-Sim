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
        assert!((fine.tired_of(m).unwrap() - coarse.tired_of(m).unwrap()).abs() < 1e-3, "member {m} tiredness");
        assert!((fine.stamina_of(m).unwrap() - coarse.stamina_of(m).unwrap()).abs() < 1e-3, "member {m} stamina");
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

// ---- Stamina and tiredness ---------------------------------------------

use gahturiyu_sim::sim::{
    combat::Fighter,
    condition::{Shelter, EXHAUSTED},
    geo::V2,
};

#[test]
fn walking_uses_stamina_and_standing_gets_it_back() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let full = w.stamina_of(m).unwrap();
    // A long walk.
    let far = w.squad.pos.add(V2::new(0.0, 5000.0));
    w.order_members(&[m], far);
    run(&mut w, 0.5, 1.0);
    let walked = w.stamina_of(m).unwrap();
    assert!(walked < full - 0.02, "{full} -> {walked}");
    // A fight now starts with what's left.
    let f = Fighter::from_person(&w.people[m as usize], 0, V2::default(), w.time);
    assert!((f.fatigue / f.max_fatigue - walked).abs() < 0.02);
    // Standing still brings it back.
    let here = w.squad.at[w.squad.index(m).unwrap()];
    w.order_members(&[m], here);
    run(&mut w, 0.5, 1.0);
    assert!(w.stamina_of(m).unwrap() > 0.99);
}

#[test]
fn climbs_and_loads_cost_more_stamina() {
    let mut c = Condition::new(0.0);
    let flat = c.climb_cost(100.0);
    c.load = 1.5;
    assert!(c.climb_cost(100.0) > flat * 1.5);
    c.activity = Activity::Walking;
    let loaded = c.stamina_rate();
    c.load = 0.2;
    assert!(loaded < c.stamina_rate(), "a heavy pack drains faster");
}

#[test]
fn tiredness_builds_awake_and_only_sleep_clears_it() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    let start = w.tired_of(m).unwrap();
    // (Standing about tires slower than a march: BL-49.)
    run(&mut w, 10.0, 60.0);
    let awake = w.tired_of(m).unwrap();
    assert!(awake > start + 30.0, "{start} -> {awake}");
    w.order_rest(&[m]);
    run(&mut w, 1.0, 60.0);
    assert!(w.is_asleep(m));
    assert!(w.tired_of(m).unwrap() < awake);
    // Moving wakes them.
    w.order_members(&[m], w.squad.pos.add(V2::new(20.0, 0.0)));
    run(&mut w, 0.01, 1.0);
    assert!(!w.is_asleep(m));
}

#[test]
fn indoors_beats_a_tent_beats_the_open() {
    let mut c = Condition::new(0.0);
    c.tired = 80.0;
    c.activity = Activity::Sleeping;
    let rate = |c: &Condition| -c.tired_rate();
    c.shelter = Shelter::Open;
    let open = rate(&c);
    c.shelter = Shelter::Tent;
    let tent = rate(&c);
    c.shelter = Shelter::Indoors;
    assert!(rate(&c) > tent && tent > open);
}

#[test]
fn the_tent_counts_when_someone_nearby_carries_it() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let m = mage(&w);
    assert_eq!(w.shelter_of(m), Shelter::Tent, "the hunter carries one");
    for x in w.squad.members.clone() {
        w.people[x as usize].detail.as_mut().unwrap().gear.bag.retain(|e| item(e.0).key != "tent");
    }
    assert_eq!(w.shelter_of(m), Shelter::Open);
}

#[test]
fn the_exhausted_are_slower_and_weaker() {
    let mut w = worldgen::generate(1);
    let m = mage(&w);
    let pace = w.member_speed(m);
    let strength = w.people[m as usize].effective_stats().attr(Attr::Strength);
    let t = w.time;
    let c = w.people[m as usize].cond.as_mut().unwrap();
    c.tired = EXHAUSTED + 5.0;
    c.at = t;
    assert!(w.member_speed(m) < pace * 0.85);
    assert!(w.people[m as usize].effective_stats().attr(Attr::Strength) < strength * 0.9);
}

#[test]
fn sleepers_are_caught_unawares() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let who = w.squad.members.clone();
    w.order_rest(&who);
    run(&mut w, 0.05, 1.0);
    assert!(who.iter().all(|&m| w.is_asleep(m)));
    w.spawn_bandits(w.squad.pos.add(V2::new(3.0, 0.0)), 2, false);
    w.step(0.25);
    w.step(0.25);
    let b = w.squad_battle().expect("attacked in their sleep");
    // They start the fight unaware (a blow — or a spark — wakes them early).
    assert!(b.fighters.iter().filter(|f| f.side == 0).all(|f| f.aware_at > b.start));
}

// ---- Healing follows condition -----------------------------------------

use gahturiyu_sim::sim::condition::HungerStage as Stage;

#[test]
fn rested_and_fed_heals_fastest_marching_and_starving_hardly_at_all() {
    let mut c = Condition::new(0.0);
    c.activity = Activity::Sleeping;
    c.shelter = Shelter::Indoors;
    let best = c.heal_rate();
    c.activity = Activity::Resting;
    let resting = c.heal_rate();
    c.activity = Activity::Walking;
    let marching = c.heal_rate();
    c.hunger = 90.0;
    assert_eq!(c.stage(), Stage::Starving);
    let starving_march = c.heal_rate();
    assert!(best > resting && resting > marching);
    assert!(starving_march < 0.01);
}

#[test]
fn healing_comes_out_the_same_however_finely_stepped() {
    // Wounded members; at fixed moments some lie down to sleep, some get up;
    // one has no food and crosses the hunger stages along the way.
    let make = || {
        let mut w = worldgen::generate(4);
        w.teleport_squad(w.squad.pos.add(V2::new(250.0, 0.0)));
        let t = w.time;
        for (i, m) in w.squad.members.clone().into_iter().enumerate() {
            let p = &mut w.people[m as usize];
            p.wounds.lost = [8.0 + i as f32, 40.0, 25.0, 5.0, 12.0, 0.0];
            p.wounds.at = t;
        }
        let hungry = w.squad.members[2];
        take_all_food(&mut w, hungry);
        w
    };
    let script = |w: &mut World, step: f64| {
        let all = w.squad.members.clone();
        run(w, 3.0, step);
        w.order_rest(&all[..2]);
        run(w, 5.0, step);
        w.order_wake(&all[..1]); // the first gets up again
        w.order_rest(&all[2..]);
        run(w, 40.0, step);
    };
    let (mut fine, mut coarse) = (make(), make());
    script(&mut fine, 10.0);
    script(&mut coarse, 1800.0);
    for &m in &fine.squad.members.clone() {
        let (lf, lc) = (fine.people[m as usize].wounds.lost_at(fine.time), coarse.people[m as usize].wounds.lost_at(coarse.time));
        for k in 0..6 {
            assert!((lf[k] - lc[k]).abs() < 1e-3, "member {m} part {k}: {} vs {}", lf[k], lc[k]);
        }
        assert!((fine.hunger_of(m).unwrap() - coarse.hunger_of(m).unwrap()).abs() < 1e-3);
        assert!((fine.tired_of(m).unwrap() - coarse.tired_of(m).unwrap()).abs() < 1e-3);
    }
}

// ---- Bedding down at night ----------------------------------------------------

/// A world with the squad stood still out of town (day 1 starts at 06:00).
fn idle_world() -> World {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(gahturiyu_sim::sim::geo::V2::new(300.0, 0.0)));
    w
}

#[test]
fn idle_members_bed_down_at_night_on_their_own() {
    let mut w = idle_world();
    run(&mut w, 15.0, 60.0); // 21:00
    assert!(w.squad.members.iter().all(|&m| !w.is_asleep(m)), "nobody's in bed before 22:00");
    run(&mut w, 2.0, 60.0); // 23:00
    assert!(w.squad.members.iter().all(|&m| w.is_asleep(m)), "everyone idle is asleep by 23:00");
    // ...and up again rested in the morning.
    run(&mut w, 10.0, 60.0); // 09:00
    assert!(w.squad.members.iter().all(|&m| !w.is_asleep(m)), "up by morning");
    // (In a tent they're rested by about 04:00 and get up then.)
    for &m in &w.squad.members {
        assert!(w.tired_of(m).unwrap() < 35.0, "rested: {:?}", w.tired_of(m));
    }
}

#[test]
fn bedtime_doesnt_depend_on_step_size() {
    let times = |step: f64| {
        let mut w = idle_world();
        run(&mut w, 17.0, step);
        let mut t: Vec<f64> = w.log.iter().filter(|l| l.1.contains("beds down")).map(|l| l.0).collect();
        t.sort_by(|a, b| a.total_cmp(b));
        t
    };
    let fine = times(5.0);
    let coarse = times(900.0);
    assert_eq!(fine.len(), 4);
    assert_eq!(fine, coarse);
}

#[test]
fn got_up_at_night_they_stay_up_until_morning() {
    let mut w = idle_world();
    run(&mut w, 17.0, 60.0); // 23:00, all asleep
    let all = w.squad.members.clone();
    w.order_wake(&all); // get up
    run(&mut w, 1.0, 60.0);
    assert!(all.iter().all(|&m| !w.is_asleep(m)), "kept up");
    run(&mut w, 8.0, 60.0); // past 06:00 again: day, so no bedding down either
    assert!(all.iter().all(|&m| !w.is_asleep(m)));
}

#[test]
fn busy_members_dont_bed_down() {
    let mut w = idle_world();
    run(&mut w, 15.5, 60.0); // 21:30
    let m = w.squad.members[0];
    let far = w.person_pos(m).add(gahturiyu_sim::sim::geo::V2::new(6000.0, 0.0));
    w.order_members(&[m], far);
    run(&mut w, 1.0, 1.0); // past 22:00, still on the way
    assert!(w.squad.at[0].dist(w.squad.goal[0]) > 1.0, "still walking");
    assert!(!w.is_asleep(m), "walking, not sleeping");
}
