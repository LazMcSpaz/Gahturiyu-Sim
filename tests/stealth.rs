//! Seeing, hearing, sneaking and sneak attacks.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    geo::{self, V2},
    items,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    stealth,
    world::HOUR,
    worldgen, World,
};

/// A world with the squad out in the open, `hours` after dawn on day 1.
fn out_in_the_wild(hours: f64) -> World {
    let mut w = worldgen::generate(3);
    // Somewhere on open land, well away from towns and camps.
    let mut spot = w.squad.pos;
    'find: for r in 1..60 {
        for k in 0..16 {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let p = w.squad.pos.add(V2::new(a.cos(), a.sin()).scale(r as f32 * 100.0));
            if geo::inland(p) > 300.0
                && w.terrain.slope(p) < 0.15
                && w.settlements.iter().all(|s| s.pos.dist(p) > s.reach + 400.0)
                && w.camps.iter().all(|c| c.pos.dist(p) > 600.0)
            {
                spot = p;
                break 'find;
            }
        }
    }
    w.teleport_squad(spot);
    let mut t = 0.0;
    while t < hours * HOUR {
        w.step(60.0);
        t += 60.0;
    }
    w
}

fn watch(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.25);
        t += 0.25;
    }
}

#[test]
fn walking_up_by_day_gets_you_noticed() {
    let mut w = out_in_the_wild(6.0); // noon
    let at = w.squad.pos.add(V2::new(28.0, 0.0));
    w.spawn_bandits(at, 3, false);
    watch(&mut w, 15.0);
    assert!(w.squad_battle().is_some(), "bandits 28 m away at noon should spot you");
}

#[test]
fn sneaking_at_night_gets_you_past() {
    let mut w = out_in_the_wild(18.0); // midnight
    for m in w.squad.members.clone() {
        w.set_sneaking(m, true);
    }
    let at = w.squad.pos.add(V2::new(28.0, 0.0));
    w.spawn_bandits(at, 3, false);
    watch(&mut w, 30.0);
    assert!(w.squad_battle().is_none(), "sneaking still in the dark at 28 m shouldn't be noticed");
    // ...but walk right up and they will.
    let close = at.sub(V2::new(2.0, 0.0));
    w.order_squad(close);
    watch(&mut w, 60.0);
    assert!(w.next_battle > 0, "bumping into them should be noticed");
}

#[test]
fn darkness_firelight_and_town_change_the_light() {
    let mut w = out_in_the_wild(18.0);
    let here = w.squad.pos;
    let dark = w.light_at(here);
    assert!(dark < 0.2, "midnight in the wild: {dark}");
    let g = w.spawn_bandits(here.add(V2::new(300.0, 0.0)), 2, false);
    let camp = w.camps.iter().find(|c| c.group == g).unwrap().pos;
    assert!(w.light_at(camp.add(V2::new(4.0, 0.0))) > dark + 0.4, "a campfire lights its surroundings");
    let town = &w.settlements[0];
    assert!(w.light_at(town.pos) > dark + 0.2, "lit windows");
    assert!(stealth::daylight(12.0 * HOUR) > stealth::daylight(20.0 * HOUR));
}

#[test]
fn armour_is_loud_and_sneaking_is_quiet() {
    let mut w = out_in_the_wild(0.0);
    let m = w.squad.members[3]; // the mage, in cloth
    w.order_members(&[m], w.squad.pos.add(V2::new(50.0, 0.0)));
    let walking = w.noise_of(m);
    w.people[m as usize].detail.as_mut().unwrap().gear.wear(items::id("scale_hauberk"));
    w.people[m as usize].detail.as_mut().unwrap().gear.wear(items::id("iron_helm"));
    let clanking = w.noise_of(m);
    assert!(clanking > walking + 0.3, "{clanking} vs {walking}");
    w.set_sneaking(m, true);
    assert!(w.noise_of(m) < clanking * 0.5);
    assert!(w.visibility_of(m) < 0.6);
    let fast = w.member_speed(m);
    w.set_sneaking(m, false);
    assert!(fast < w.member_speed(m) * 0.5, "sneaking is slow");
}

#[test]
fn creeping_near_watchers_trains_sneak() {
    let mut w = out_in_the_wild(18.0);
    let m = w.squad.members[3];
    for x in w.squad.members.clone() {
        w.set_sneaking(x, true);
    }
    let before = w.people[m as usize].stats.skill(Skill::Sneak);
    w.spawn_bandits(w.squad.pos.add(V2::new(40.0, 0.0)), 2, false);
    watch(&mut w, 120.0);
    assert!(w.squad_battle().is_none());
    assert!(w.people[m as usize].stats.skill(Skill::Sneak) > before + 0.05);
}

fn person(id: u32) -> Person {
    let mut p = Person::summary(id, 3000 + id as u64 * 7, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blade, 50.0), (Skill::Sneak, 50.0)], 300.0);
    p.ensure_detail();
    p
}

#[test]
fn sneak_attacks_hit_harder() {
    let hit = |unaware: bool| {
        let mut a = Fighter::from_person(&person(1), 0, V2::new(0.0, 0.0), 0.0);
        let mut d = Fighter::from_person(&person(2), 1, V2::new(1.0, 0.0), 0.0);
        d.hp = [1e5; 6];
        d.think_at = f64::INFINITY;
        if unaware {
            d.aware_at = 1e9;
        }
        a.target = Some(1);
        a.think_at = f64::INFINITY;
        let mut b = Battle::new(0, 5, 0.0, vec![a, d], vec!["A".into(), "B".into()]);
        for _ in 0..600 {
            b.tick();
            if b.fighters[1].damage_taken > 0.0 {
                break;
            }
        }
        (b.fighters[1].damage_taken, b.fighters[1].aware_at)
    };
    let (normal, _) = hit(false);
    let (sneak, woke) = hit(true);
    assert!(normal > 0.0 && sneak > 0.0);
    assert!(sneak > normal * 2.5, "{sneak} vs {normal}");
    assert!(woke < 1e9, "being hit wakes you up");
}

#[test]
fn attacking_unnoticed_bandits_catches_them_unawares() {
    let mut w = out_in_the_wild(18.0);
    for m in w.squad.members.clone() {
        w.set_sneaking(m, true);
    }
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(30.0, 0.0)), 3, false);
    watch(&mut w, 5.0);
    assert!(w.squad_battle().is_none());
    let target = w.group(g).unwrap().members[0];
    let who = w.squad.members.clone();
    assert!(w.attack(&who, target));
    let b = w.squad_battle().expect("a fight");
    assert!(b.fighters.iter().filter(|f| f.side != 0).all(|f| f.unaware(w.time)));
}
