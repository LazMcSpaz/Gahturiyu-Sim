//! The rules toggles (Laz, 13 Oct) for the second session's systems:
//! potions breaking, thirst, food going off, fire and water on what's
//! carried, and bleeding. Each switched off, its system answers "no"; a
//! switch takes hold at once (`rules_step`).

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    geo::V2,
    items,
    world::{DAY, HOUR},
    worldgen, World,
};

fn off(key: &str) -> World {
    let mut w = worldgen::generate(1);
    assert!(w.rules.set(key, false), "a rule called {key}");
    w
}

#[test]
fn with_potions_breaking_off_no_bottle_is_at_risk_in_a_fight() {
    for on in [true, false] {
        let mut w = worldgen::generate(1);
        w.rules.potions_break = on;
        let m = w.squad.members[0];
        {
            let d = w.people[m as usize].detail.as_mut().unwrap();
            d.gear.bag.retain(|e| items::item(e.0).key != "potion_case");
            d.gear.add(items::id("healing_draught"), 3);
        }
        let band = w.spawn_bandits(w.squad.pos.add(V2::new(10.0, 0.0)), 2, false);
        let foe = w.group(band).unwrap().members[0];
        assert!(w.attack(&w.squad.members.clone(), foe));
        let b = w.squad_battle().expect("a fight");
        let f = b.fighters.iter().find(|f| f.pid == m).unwrap();
        assert_eq!(f.exposed.is_empty(), !on, "bottles at risk only with the rule on");
    }
}

#[test]
fn with_thirst_off_nobody_gets_thirsty() {
    let mut on = worldgen::generate(1);
    let mut no = off("thirst");
    for w in [&mut on, &mut no] {
        let end = w.time + 2.0 * DAY;
        while w.time < end {
            w.step(600.0);
        }
    }
    let m = on.squad.members[0];
    let thirst = |w: &World| w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time);
    assert!(thirst(&on) > 5.0, "thirsty with it on: {}", thirst(&on));
    assert_eq!(thirst(&no), 0.0, "none with it off");
}

#[test]
fn with_food_going_off_off_nothing_spoils() {
    let mut w = off("food_spoils");
    let m = w.squad.members[0];
    let meat = items::id("raw_meat");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(meat, 2);
    w.step(1.0);
    let left = w.days_left(m, meat, w.time);
    let end = w.time + 3.0 * DAY;
    while w.time < end {
        w.step(600.0);
    }
    assert!((w.days_left(m, meat, w.time) - left).abs() < 0.01, "still {left} days left");
}

#[test]
fn with_fire_and_water_on_carried_things_off_paper_comes_through() {
    for on in [true, false] {
        let mut w = worldgen::generate(1);
        w.rules.elements_on_items = on;
        let m = w.squad.members[0];
        let paper = items::id("reed_paper");
        w.people[m as usize].detail.as_mut().unwrap().gear.add(paper, 20);
        let mut f = Fighter::from_person(&w.people[m as usize], 0, w.squad.pos, w.time);
        f.burned = 120.0;
        let t = w.time;
        w.paper_spoils(&f, 7, t);
        let left = w.count_of(m, "reed_paper");
        if on {
            assert!(left < 20, "two minutes alight burns some");
        } else {
            assert_eq!(left, 20, "none lost with it off");
        }
    }
}

#[test]
fn with_bleeding_off_cuts_dont_bleed_and_a_bleed_stops() {
    // In a fight: a cut opens no bleed.
    let w = worldgen::generate(1);
    let m = w.squad.members[0];
    let f = Fighter::from_person(&w.people[m as usize], 0, V2::new(0.0, 0.0), 0.0);
    let mut b = Battle::new(0, 1, 0.0, vec![f.clone(), f], vec!["A".into(), "B".into()]);
    b.bleeding = false;
    b.open_bleed(0, 1.0);
    assert_eq!(b.fighters[0].bleeding(), 0.0);
    b.bleeding = true;
    b.open_bleed(0, 1.0);
    assert!(b.fighters[0].bleeding() > 0.0);
    // A bleed the squad carries stops the moment it's switched off.
    let mut w = w;
    w.people[m as usize].wounds.bleed = 2.0;
    w.rules.bleeding = false;
    w.step(0.5);
    assert_eq!(w.bleeding(m), 0.0);
    // And the world's fights carry the setting.
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(10.0, 0.0)), 2, false);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.attack(&w.squad.members.clone(), foe));
    assert!(!w.squad_battle().unwrap().bleeding);
}

#[test]
fn a_switch_takes_hold_at_once() {
    let mut w = worldgen::generate(1);
    let end = w.time + 12.0 * HOUR;
    while w.time < end {
        w.step(600.0);
    }
    let m = w.squad.members[0];
    assert!(w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time) > 0.0);
    w.rules.thirst = false;
    w.step(0.5);
    assert_eq!(w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time), 0.0, "settled under the new rule");
    assert_eq!(w.rules_applied, w.rules);
}
