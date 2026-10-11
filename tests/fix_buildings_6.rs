//! Fixes from the known-issues list, batch 6 (the buildings agent).

use gahturiyu_sim::sim::{
    geo::V2,
    items::{self, item, Kind},
    world::DAY,
    worldgen,
};

/// RG-21 = NM-53 (food): one with nothing to eat is fed by a squadmate close
/// by; one far off isn't, and goes hungry.
#[test]
fn rg_21_a_squadmate_close_by_shares_their_food() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let (hungry, fed) = (w.squad.members[0], w.squad.members[1]);
    for &m in &w.squad.members.clone() {
        w.people[m as usize].detail.as_mut().unwrap().gear.bag.retain(|e| !matches!(item(e.0).kind, Kind::Food(_)));
    }
    w.people[fed as usize].detail.as_mut().unwrap().gear.add(items::id("salted_meat"), 20);
    let mut shared = false;
    for _ in 0..(3.0 * DAY / 600.0) as usize {
        w.step(600.0);
        shared |= w.log.iter().any(|(_, l)| l.contains("shares some"));
    }
    assert!(shared, "nobody shared");
    assert!(w.hunger_of(hungry).unwrap() < 60.0, "went hungry beside food: {}", w.hunger_of(hungry).unwrap());
}

/// NM-11: by day some of those at home are indoors (and so see who comes in).
#[test]
fn nm_11_by_day_some_are_at_home_indoors() {
    use gahturiyu_sim::sim::{routine::Doing, world::HOUR};
    let mut w = worldgen::generate(1);
    while (w.time.rem_euclid(DAY) / HOUR - 11.0).abs() > 0.1 {
        w.step(300.0);
    }
    let (mut home, mut inside) = (0, 0);
    for s in &w.settlements {
        for &p in &s.residents {
            if matches!(w.doing_now(p), Some((Doing::Home, _))) {
                home += 1;
                inside += w.building_at(w.person_pos(p)).is_some() as usize;
            }
        }
    }
    eprintln!("{inside} of {home} at home are indoors");
    assert!(home > 0 && inside > 0 && inside < home, "{inside} of {home}");
}

/// BL-47: sent off together, the squad keeps together on the march.
#[test]
fn bl_47_the_squad_keeps_together_on_a_long_march() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    // One of them carries a lot, and is slow.
    let slow = w.squad.members[0];
    w.people[slow as usize].detail.as_mut().unwrap().gear.add(items::id("iron_ore"), 25);
    let to = w.squad.pos.add(V2::new(0.0, 1500.0));
    w.order_squad(to);
    let mut widest = 0.0f32;
    for _ in 0..(15.0 * 60.0 / 0.5) as usize {
        w.step(0.5);
        let at = &w.squad.at;
        for a in at {
            for b in at {
                widest = widest.max(a.dist(*b));
            }
        }
    }
    let moved = w.squad.at[1].dist(to);
    eprintln!("widest {widest:.0} m; still {moved:.0} m to go");
    assert!(widest < 40.0, "strung out over {widest:.0} m");
}

/// NM-53 (rest): `give` can hand over some of a stack.
#[test]
fn nm_53_some_of_a_stack_can_be_given() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let arrow = items::id("arrows");
    w.people[a as usize].detail.as_mut().unwrap().gear.add(arrow, 20);
    let had_b = w.count_of(b, "arrows");
    let k = w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == arrow).unwrap();
    let had_a = w.count_of(a, "arrows");
    w.give_some(a, k, b, 5).unwrap();
    assert_eq!(w.count_of(a, "arrows"), had_a - 5);
    assert_eq!(w.count_of(b, "arrows"), had_b + 5);
}

/// NM-26: what the squad puts in a chest is still theirs to take back.
#[test]
fn nm_26_taking_your_own_things_back_isnt_theft() {
    use gahturiyu_sim::sim::{
        buildings::door_of,
        loot::{LootRef, Source},
        world::HOUR,
    };
    let mut w = worldgen::generate(1);
    while (w.time.rem_euclid(DAY) / HOUR - 10.0).abs() > 0.1 {
        w.step(300.0);
    }
    let m = w.squad.members[0];
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    let s = w.settlements[town as usize].clone();
    let mut found = None;
    for d in (0..s.buildings.len() as u16).filter_map(|i| door_of(&s, i)).filter(|d| d.lock > 0.0).take(12) {
        let out = d.outside.sub(d.centre);
        w.teleport_squad(d.outside.add(out.scale(6.0 / out.len().max(0.1))));
        w.order_members(&[m], d.centre);
        for _ in 0..120 {
            w.step(0.5);
        }
        if let Some(c) = w.containers_in(d.id).find(|c| c.lock <= 0.0).map(|c| c.id) {
            found = Some(c);
            break;
        }
    }
    let c = found.expect("a house with an open chest");
    assert!(w.order_search(m, c));
    for _ in 0..120 {
        w.step(0.5);
    }
    assert!(w.source_now(m).is_some());
    // Something of theirs the chest doesn't already hold.
    let key = ["arrows", "lockpick", "flatbread", "dried_fish", "salted_meat"].into_iter().find(|k| w.container(c).unwrap().items.iter().all(|e| e.0 != items::id(k))).unwrap();
    let it = items::id(key);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(it, 3);
    let had = w.count_of(m, key);
    let k = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.0 == it).unwrap();
    assert!(w.put_in(m, k));
    let taken = w.container(c).unwrap().taken;
    let at = w.container(c).unwrap().items.iter().position(|e| e.0 == it).unwrap();
    assert!(w.take_from(m, Source::Chest(c), LootRef::Pack(at)));
    assert_eq!(w.count_of(m, key), had);
    assert_eq!(w.container(c).unwrap().taken, taken, "taking back their own {key} counted as taking from the chest");
}

/// NM-51: posts given up for want of work go back to plain work, so "no
/// trade" doesn't pile up.
#[test]
fn nm_51_no_trade_doesnt_pile_up() {
    use gahturiyu_sim::sim::jobs::Job;
    let mut w = worldgen::generate(1);
    let none = |w: &gahturiyu_sim::sim::World| w.settlements.iter().flat_map(|s| s.residents.iter()).filter(|&&p| !w.people[p as usize].dead && w.life(p).job == Job::None).count();
    let start = none(&w);
    for _ in 0..(30.0 * DAY / 600.0) as usize {
        w.step(600.0);
    }
    let end = none(&w);
    eprintln!("no trade: {start} -> {end}");
    assert!(end <= start + 10, "no trade: {start} -> {end}");
}
