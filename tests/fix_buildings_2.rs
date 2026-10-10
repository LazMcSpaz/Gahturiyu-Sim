//! Fixes from the known-issues list, batch 2 (the buildings agent).

use gahturiyu_sim::sim::{geo::V2, world::{DAY, HOUR}, worldgen};

#[test]
fn bl_3_a_walk_between_towns_goes_round_the_water() {
    let w = worldgen::generate(1);
    let mut tried = 0;
    for a in &w.settlements {
        for b in &w.settlements {
            if a.id >= b.id || a.pos.dist(b.pos) > 8000.0 || !w.crosses_sea(a.pos, &[b.pos]) {
                continue;
            }
            tried += 1;
            let (path, _) = w.travel(a.pos, b.pos);
            assert!(!w.crosses_sea(a.pos, &path), "{} to {} goes through the water", a.name, b.name);
        }
    }
    assert!(tried > 0, "some towns face each other across water");
}

#[test]
fn nm_48_a_member_far_from_their_work_doesnt_walk_off_to_it() {
    let mut w = worldgen::generate(1);
    let (job, place) = w.vacant_posts(0)[0];
    let who = w.squad.members[0];
    let at = w.society.towns[0].places[place as usize].pos;
    w.teleport_squad(at.add(V2::new(4000.0, 0.0)));
    assert!(w.take_post_work(who, 0, job, place));
    let c = w.contract_of(who).unwrap().clone();
    while w.time < c.first_day as f64 * DAY + 9.0 * HOUR {
        w.step(30.0);
    }
    let k = w.squad.index(who).unwrap();
    assert!(w.squad.at[k].dist(w.squad.goal[k]) < 1.0, "they stay put");
    assert!(w.log.iter().any(|(_, l)| l.contains("too far")));
}

#[test]
fn rg_17_too_much_to_carry_is_too_much_to_walk() {
    use gahturiyu_sim::sim::{inventory::encumbrance_factor, items};
    // A smooth fall to nothing at three times the limit.
    let mut last = 1.0;
    for i in 0..=40 {
        let f = encumbrance_factor(1.0 + i as f32 * 0.05);
        assert!(f <= last + 1e-6 && last - f < 0.05, "a jump at {}", 1.0 + i as f32 * 0.05);
        last = f;
    }
    assert_eq!(encumbrance_factor(3.0), 0.0);
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("scale_hauberk"), 40);
    let k = w.squad.index(m).unwrap();
    let from = w.squad.at[k];
    w.order_members(&[m], from.add(V2::new(100.0, 0.0)));
    for _ in 0..60 {
        w.step(1.0);
    }
    assert!(w.squad.at[k].dist(from) < 0.5, "they can't move");
    assert!(w.log.iter().any(|(_, l)| l.contains("can't move under that load")));
}

#[test]
fn rg_15_nobody_asks_a_big_favour_for_nothing() {
    use gahturiyu_sim::sim::{chances::{Chance, FAVOUR_MAX}, items};
    for seed in [1, 3] {
        let mut w = worldgen::generate(seed);
        for tl in &mut w.society.towns {
            tl.drama = 0.8;
        }
        for _ in 0..6 * 24 {
            w.step(HOUR);
        }
        for o in w.society.opps.iter().filter(|o| o.favour) {
            if let Chance::Fetch { item, count } = o.kind {
                let worth = items::item(item).value * count as f32 * 2.5 + 10.0;
                assert!(worth <= FAVOUR_MAX, "a {worth:.0}-coin fetch asked as a favour");
            }
        }
    }
}

#[test]
fn bl_7_ordered_about_at_night_they_stay_up() {
    let mut w = worldgen::generate(1);
    while (w.time.rem_euclid(DAY) / HOUR - 23.0).abs() > 0.05 {
        w.step(60.0);
    }
    let all = w.squad.members.clone();
    let to = w.squad.pos.add(V2::new(40.0, 0.0));
    w.order_members(&all, to);
    for _ in 0..(HOUR / 5.0) as usize {
        w.step(5.0);
    }
    for &m in &all {
        assert!(!w.is_asleep(m), "bedded down after a night order");
    }
}

#[test]
fn bl_8_the_down_and_the_sleeping_cant_light_torches_or_change_gear() {
    use gahturiyu_sim::sim::{body, items::Slot};
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    let p = &mut w.people[m as usize];
    let max = p.stats.max_hp(body::Part::Torso);
    p.wounds.lost = p.wounds.lost_at(t);
    p.wounds.lost[1] = max + 20.0;
    p.wounds.at = t;
    w.step(1.0);
    assert!(w.is_down(m));
    assert!(!w.toggle_torch(m), "lit a torch out cold");
    assert!(!w.unequip(m, Slot::MainHand), "took off gear out cold");
    assert!(!w.drop_entry(m, 0), "dropped something out cold");
}
