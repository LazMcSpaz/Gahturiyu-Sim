//! Fixes from the known-issues list (the buildings agent's queue), a test
//! each. Named after the list's IDs.

use gahturiyu_sim::sim::{
    body,
    buildings::door_of,
    geo::V2,
    items,
    person::PersonId,
    stats::Skill,
    world::{DAY, HOUR},
    worldgen, World,
};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

fn until_hour(w: &mut World, h: f64) {
    while (w.time.rem_euclid(DAY) / HOUR - h).abs() > 0.05 {
        w.step(60.0);
    }
}

fn knock_out(w: &mut World, pid: PersonId) {
    let t = w.time;
    let p = &mut w.people[pid as usize];
    let max = p.stats.max_hp(body::Part::Torso);
    p.wounds.lost = p.wounds.lost_at(t);
    p.wounds.lost[1] = max + 20.0;
    p.wounds.at = t;
}

#[test]
fn bl_2_an_order_to_nowhere_leaves_the_squad_where_it_is() {
    let mut w = worldgen::generate(1);
    let before = w.squad.pos;
    w.order_squad(V2::new(f32::NAN, f32::NAN));
    let all = w.squad.members.clone();
    w.order_members(&all, V2::new(f32::INFINITY, 3.0));
    w.step(1.0);
    assert!(w.squad.pos.x.is_finite() && w.squad.pos.y.is_finite(), "the squad's place is lost");
    assert!(w.squad.pos.dist(before) < 5.0);
}

#[test]
fn rg_4_only_gear_is_held_and_a_draught_isnt_wasted_on_the_unhurt() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let bread = items::id("flatbread");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(bread, 2);
    assert!(!w.equip(m, bread), "bread isn't held");
    assert_ne!(w.people[m as usize].detail.as_ref().unwrap().gear.in_slot(items::Slot::MainHand), Some(bread));
    // A healing draught for someone who isn't hurt: refused, and kept.
    let other = w.squad.members[1];
    let draught = items::ITEMS.iter().find(|d| d.kind == items::Kind::Potion && d.effects.iter().any(|e| e.does == gahturiyu_sim::sim::effects::Does::Heal)).unwrap().key;
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id(draught), 1);
    let had = w.count_of(m, draught);
    assert!(w.order_dose(m, other).is_err());
    assert_eq!(w.count_of(m, draught), had);
}

#[test]
fn nm_44_trying_a_lock_again_is_a_fresh_try() {
    let mut w = worldgen::generate(1);
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    let s = &w.settlements[town as usize];
    let d = (0..s.buildings.len() as u16).filter_map(|i| door_of(s, i)).find(|d| d.lock > 40.0).expect("a stiff lock");
    until_hour(&mut w, 2.0);
    let thief = w.squad.members[1];
    w.people[thief as usize].stats.set_skill(Skill::Security, 1.0);
    w.people[thief as usize].detail.as_mut().unwrap().gear.add(items::id("lockpick"), 60);
    w.set_sneaking(thief, true);
    // Five goes of a few tries each, giving up in between: if each go
    // replayed the same rolls, every go would snap the same picks.
    let mut goes = Vec::new();
    for _ in 0..5 {
        let before = w.count_of(thief, "lockpick");
        assert!(w.order_pick(thief, d.id));
        walk(&mut w, 60.0);
        let k = w.squad.index(thief).unwrap();
        let here = w.squad.at[k];
        w.order_members(&[thief], here.add(V2::new(2.0, 0.0)));
        walk(&mut w, 5.0);
        if !w.bounty.is_empty() || !w.is_locked(d.id) {
            eprintln!("ended early after {goes:?}");
            assert!(goes.len() < 2 || goes.windows(2).any(|p| p[0] != p[1]), "every go snapped the same picks: {goes:?}");
            return; // caught or through: the goes ended differently anyway
        }
        goes.push(before - w.count_of(thief, "lockpick"));
    }
    eprintln!("goes {goes:?}");
    assert!(goes.windows(2).any(|p| p[0] != p[1]), "every go snapped the same picks: {goes:?}");
}

#[test]
fn bl_29_a_carried_body_isnt_a_full_pack() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    assert!(w.order_carry(a, b));
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    assert!(w.load_of(a) > w.pack_load_of(a) + 0.3, "the body weighs on the carrier");
    assert!(w.pack_load_of(a) < 0.95, "but the pack isn't full");
}

#[test]
fn bl_30_a_fresh_member_starts_with_full_stamina() {
    let mut w = worldgen::generate(1);
    for _ in 0..120 {
        w.step(1.0);
    }
    for &m in &w.squad.members.clone() {
        assert!(w.stamina_of(m).unwrap() > 0.999, "stamina {}", w.stamina_of(m).unwrap());
    }
}

#[test]
fn nm_61_the_next_trading_day_is_named_right() {
    let w = worldgen::generate(1);
    let t = w.time;
    assert_eq!(w.day_word(t + HOUR), "");
    assert_eq!(w.day_word(t + DAY), " tomorrow");
    assert_eq!(w.day_word(t + 2.0 * DAY), " the day after tomorrow");
    assert_eq!(w.day_word(t + 4.0 * DAY), " in 4 days");
}

#[test]
fn bl_51_a_rest_with_nothing_to_sleep_off_ends_by_itself() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    // Fully rested at midday.
    until_hour(&mut w, 11.0);
    let now = w.time;
    if let Some(c) = w.people[m as usize].cond.as_mut() {
        c.settle(now);
        c.tired = 0.0;
    }
    w.order_rest(&[m]);
    walk(&mut w, 30.0);
    // Sleep off what little there is, then a nap at most.
    let mut slept = false;
    for _ in 0..(3.0 * HOUR / 60.0) as usize {
        w.step(60.0);
        slept |= w.is_asleep(m);
        if slept && !w.is_asleep(m) {
            eprintln!("woke at {:.2}h", (w.time % DAY) / HOUR);
            return;
        }
    }
    assert!(!w.is_asleep(m), "still asleep at {:.1}h (tired {:?})", (w.time % DAY) / HOUR, w.tired_of(m));
}
