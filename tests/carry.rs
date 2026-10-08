//! Carrying the downed: nobody gets left behind.

use gahturiyu_sim::sim::{body, geo::V2, person::PersonId, worldgen, World};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// Knock someone out (torso to zero), as of now.
fn knock_out(w: &mut World, pid: PersonId) {
    let t = w.time;
    let p = &mut w.people[pid as usize];
    let max = p.stats.max_hp(body::Part::Torso);
    p.wounds.lost = p.wounds.lost_at(t);
    p.wounds.lost[1] = max + 20.0;
    p.wounds.at = t;
}

fn out_in_the_open() -> World {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    w
}

#[test]
fn a_downed_member_is_carried_not_left_behind() {
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    let speed = w.member_speed(a);
    assert!(w.order_carry(a, b));
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    assert!(w.member_speed(a) < speed * 0.8, "a body is heavy: {} vs {speed}", w.member_speed(a));
    // Slow going: a body is a heavy load.
    let dest = w.person_pos(a).add(V2::new(15.0, 0.0));
    w.order_members(&[a], dest);
    walk(&mut w, 300.0);
    assert!(w.person_pos(b).dist(w.person_pos(a)) < 0.01, "the carried go where the carrier goes");
    assert!(w.person_pos(b).dist(dest) < 1.0);
    assert!(w.put_down(a));
    assert_eq!(w.carried_by(b), None);
    assert!(w.person_pos(b).dist(dest) < 2.0, "set down where they stood");
}

#[test]
fn a_carrier_who_falls_drops_their_burden() {
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    w.order_carry(a, b);
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    knock_out(&mut w, a);
    walk(&mut w, 1.0);
    assert_eq!(w.carried_by(b), None);
    assert!(w.person_pos(b).dist(w.person_pos(a)) < 2.0, "dropped beside them");
}

#[test]
fn carriers_dont_fight() {
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    w.order_carry(a, b);
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    let at = w.person_pos(a).add(V2::new(3.0, 0.0));
    w.spawn_bandits(at, 2, false);
    let mut seen = false;
    for _ in 0..400 {
        w.step(0.25);
        let Some(battle) = w.squad_battle() else { continue };
        seen = true;
        let f = battle.fighters.iter().find(|f| f.pid == a).expect("in the fight");
        assert!(f.burdened);
        assert!(!matches!(f.act, gahturiyu_sim::sim::combat::Act::Swing { .. } | gahturiyu_sim::sim::combat::Act::Cast { .. }), "a carrier shouldn't attack");
    }
    assert!(seen, "there should have been a fight");
}

#[test]
fn a_downed_stranger_can_be_carried_and_left_somewhere() {
    let mut w = out_in_the_open();
    let a = w.squad.members[0];
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(600.0, 0.0)), 1, false);
    let s = w.group(g).unwrap().members[0];
    knock_out(&mut w, s);
    // Bring them over to the squad first (the quick way): walk out to them.
    assert!(w.order_carry(a, s));
    walk(&mut w, 600.0);
    assert_eq!(w.carried_by(s), Some(a));
    let spot = w.person_pos(a);
    w.put_down(a);
    walk(&mut w, 30.0);
    assert!(w.person_pos(s).dist(spot) < 2.0, "they stay where they were left while out cold");
}

#[test]
fn your_own_dead_can_be_carried_home() {
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    // b dies (as a fight would leave it).
    let at = w.person_pos(b);
    let race = w.people[b as usize].race;
    w.people[b as usize].dead = true;
    let t = w.time;
    w.corpses.push((at, race, t, b));
    let people = w.people.clone();
    w.squad.retain(|m| !people[m as usize].dead);
    assert!(w.order_carry(a, b));
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    let dest = w.person_pos(a).add(V2::new(15.0, 0.0));
    w.order_members(&[a], dest);
    walk(&mut w, 300.0);
    w.put_down(a);
    let c = w.corpses.iter().find(|c| c.3 == b).unwrap();
    assert!(c.0.dist(dest) < 2.0);
}

#[test]
fn carrying_is_slow_but_nobody_crawls() {
    use gahturiyu_sim::sim::carry::CARRY_PACE_RANGE;
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    let free = w.member_speed(a);
    w.order_carry(a, b);
    walk(&mut w, 20.0);
    assert_eq!(w.carried_by(b), Some(a));
    let pace = w.member_speed(a) / free;
    assert!(pace >= CARRY_PACE_RANGE.0 - 1e-3 && pace <= CARRY_PACE_RANGE.1 + 1e-3, "carrying pace {pace}");
    assert!(pace < 0.85, "a body still slows you: {pace}");
}

#[test]
fn stronger_carriers_go_faster() {
    let mut w = out_in_the_open();
    let (a, b) = (w.squad.members[0], w.squad.members[2]);
    knock_out(&mut w, b);
    w.order_carry(a, b);
    walk(&mut w, 20.0);
    let weak = w.carry_pace(a);
    let p = &mut w.people[a as usize];
    p.stats.attrs[gahturiyu_sim::sim::stats::Attr::Strength as usize] += 30.0;
    assert!(w.carry_pace(a) > weak, "{} vs {weak}", w.carry_pace(a));
}
