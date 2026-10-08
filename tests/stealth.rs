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
    // (The fight may already be over.)
    assert!(w.squad_battle().is_some() || w.alerts.iter().any(|a| a.contains("attacked")), "bandits 28 m away at noon should spot you");
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

// ---- Torches ----------------------------------------------------------------

/// Light the first squad member's torch (each starts with two in the pack).
fn light_torch(w: &mut World) -> u32 {
    let m = w.squad.members[0];
    assert!(w.toggle_torch(m), "should light");
    assert!(w.torch_lit(m));
    m
}

#[test]
fn a_torch_at_night_is_seen_from_far_away() {
    // Without one: bandits 120 m off in the dark don't notice anyone.
    let mut w = out_in_the_wild(18.0);
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(120.0, 0.0)), 3, false);
    watch(&mut w, 30.0);
    assert!(!w.has_noticed(g), "nobody should be seen at 120 m in the dark");
    // With one lit, they do.
    let mut w = out_in_the_wild(18.0);
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(120.0, 0.0)), 3, false);
    let m = light_torch(&mut w);
    watch(&mut w, 30.0);
    assert!(w.has_noticed(g) || w.squad_battle().is_some(), "a torch at night should give them away");
    assert!(w.suspicion_of(m) >= 1.0 || w.fighter(m).is_some());
}

#[test]
fn a_torch_makes_no_difference_by_day() {
    let mut w = out_in_the_wild(6.0); // noon
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(120.0, 0.0)), 3, false);
    light_torch(&mut w);
    watch(&mut w, 30.0);
    assert!(!w.has_noticed(g), "at noon a torch is just a stick");
}

#[test]
fn sneaking_with_a_lit_torch_is_close_to_useless() {
    let mut w = out_in_the_wild(18.0);
    for m in w.squad.members.clone() {
        w.set_sneaking(m, true);
    }
    let m = light_torch(&mut w);
    let at = w.squad.pos.add(V2::new(28.0, 0.0));
    w.spawn_bandits(at, 3, false);
    watch(&mut w, 5.0);
    // The same spot and time without a torch goes unnoticed for 30 s (see
    // `sneaking_at_night_gets_you_past`).
    assert!(w.squad_battle().is_some() || w.suspicion_of(m) >= 1.0, "a sneaking torch-bearer is still seen");
    assert!(w.visibility_of(m) > 0.5, "hardly hidden: {}", w.visibility_of(m));
}

#[test]
fn a_standing_torch_lights_up_whoever_is_beside_it() {
    let run = |torch: bool| {
        let mut w = out_in_the_wild(18.0);
        let has = |w: &World, m: u32| w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().any(|e| e.0 == items::id("standing_torch"));
        let m = w.squad.members.iter().copied().find(|&m| has(&w, m)).expect("the squad's hunter carries standing torches");
        for x in w.squad.members.clone() {
            w.set_sneaking(x, true);
        }
        let dark = w.light_at(w.squad.pos);
        if torch {
            assert!(w.place_torch(m));
            assert!(w.light_at(w.squad.pos) > dark + 0.5, "it lights the ground");
        }
        let g = w.spawn_bandits(w.squad.pos.add(V2::new(14.0, 0.0)), 3, false);
        // (Checked soon: noticed, they attack, and the fight may be over in half a minute.)
        watch(&mut w, 6.0);
        w.has_noticed(g) || w.squad_battle().is_some()
    };
    assert!(!run(false), "sneaking in the dark at 14 m: unseen");
    assert!(run(true), "sneaking beside a standing torch: seen");
}

#[test]
fn torches_burn_down_by_the_clock() {
    use gahturiyu_sim::sim::torch::TORCH_HOURS;
    let burn = |step: f64| {
        let mut w = out_in_the_wild(18.0);
        let m = light_torch(&mut w);
        let start = w.time;
        let torches = |w: &World| w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == items::id("torch")).map(|e| e.1).sum::<u16>() + w.torch_in_hand(m).map(|_| 1).unwrap_or(0);
        assert_eq!(torches(&w), 2);
        let mut seen = Vec::new();
        while w.time < start + (2.0 * TORCH_HOURS as f64 + 1.0) * HOUR {
            w.step(step);
            seen.push((w.time, torches(&w), w.torch_lit(m)));
        }
        // Lit at every moment before 2 × burn time, out after; one torch per burn.
        for &(t, n, lit) in &seen {
            let burnt = ((t - start) / (TORCH_HOURS as f64 * HOUR)).floor() as u16;
            assert_eq!(n, 2u16.saturating_sub(burnt), "at {:.2} h", (t - start) / HOUR);
            assert_eq!(lit, burnt < 2, "at {:.2} h", (t - start) / HOUR);
        }
        // (Only the burn-outs: the log is short, and how many travellers get
        // announced depends on how often the squad looks round.)
        w.log.iter().filter(|l| l.1.contains("burns")).map(|l| l.0).collect::<Vec<_>>()
    };
    let fine = burn(7.0);
    let coarse = burn(1800.0);
    assert_eq!(fine, coarse, "burn-outs happen at the same moments whatever the step");
}

#[test]
fn a_torch_put_out_keeps_what_is_left() {
    let mut w = out_in_the_wild(18.0);
    let m = light_torch(&mut w);
    watch(&mut w, 0.0);
    let mut t = 0.0;
    while t < HOUR {
        w.step(60.0);
        t += 60.0;
    }
    w.toggle_torch(m);
    assert!(!w.torch_lit(m));
    let left = w.torch_hours_left(m).unwrap();
    assert!((left - 3.0).abs() < 0.05, "{left}");
    for _ in 0..120 {
        w.step(60.0);
    }
    assert!((w.torch_hours_left(m).unwrap() - left).abs() < 1e-3, "an unlit torch doesn't burn");
}



// ---- Travellers' torches ------------------------------------------------------

#[test]
fn travellers_on_the_road_at_night_carry_lights() {
    use gahturiyu_sim::sim::torch::TRAVEL_TORCH_DARK;
    let mut w = worldgen::generate(1);
    // Midday: nobody's torch is lit.
    let mut t = 0.0;
    while t < 6.0 * HOUR {
        w.step(60.0);
        t += 60.0;
    }
    assert!(w.groups.iter().all(|g| !w.group_torch_lit(g, w.time)));
    // Late evening: everyone on the move has one.
    while stealth::daylight(w.time) >= TRAVEL_TORCH_DARK || w.groups.iter().filter(|g| g.is_moving(w.time) && !g.hostile).count() == 0 {
        w.step(60.0);
    }
    let moving: Vec<_> = w.groups.iter().filter(|g| g.is_moving(w.time) && !g.hostile && !w.fighting_groups.contains(&g.id)).collect();
    assert!(!moving.is_empty());
    assert!(moving.iter().all(|g| w.group_torch_lit(g, w.time)));
    let lights = w.lights();
    for g in moving {
        let at = g.position_at(w.time);
        assert!(lights.iter().any(|l| l.pos.dist(at) < 0.01), "a light where they walk");
        assert!(w.light_at(at) > stealth::daylight(w.time) + 0.5, "and it lights the road");
    }
}

#[test]
fn camps_spot_torch_bearers_from_further_at_night() {
    use gahturiyu_sim::sim::{encounters::{camp_sees, CAMP_SIGHT}, group::Leg, torch::TORCH_SEEN};
    let w = worldgen::generate(1);
    let camp = w.camps[0].pos;
    // A straight walk passing the camp at 160 m: beyond plain sight, within a torch's.
    let pass = (CAMP_SIGHT + TORCH_SEEN) / 2.0;
    let (a, b) = (camp.add(V2::new(-600.0, pass)), camp.add(V2::new(600.0, pass)));
    let at = |start: f64| Leg::along(vec![a, b], start, 1.3, None, &w.terrain);
    let noon = Leg { ..at(12.0 * HOUR) };
    let midnight = at(24.0 * HOUR);
    assert!(camp_sees(&noon, camp).is_none(), "by day they'd pass unseen at {pass} m");
    assert!(camp_sees(&midnight, camp).is_some(), "at night their torch gives them away");
    // Close by, they're seen either way.
    let near = Leg::along(vec![camp.add(V2::new(-600.0, 50.0)), camp.add(V2::new(600.0, 50.0))], 12.0 * HOUR, 1.3, None, &w.terrain);
    assert!(camp_sees(&near, camp).is_some());
}

