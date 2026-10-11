//! Fixes from the known-issues list, batch 5 (the buildings agent): Laz's
//! answers B5 (stamina), B6 (towns spend, grain spoils) and B2 (starving).

use gahturiyu_sim::sim::{
    geo::V2,
    stats::Attr,
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, hours: f64, step: f64) {
    for _ in 0..(hours * HOUR / step).round() as usize {
        w.step(step);
    }
}

/// Empty a member's breath now (on their condition timeline).
fn spend_breath(w: &mut World, m: u32) {
    let t = w.time;
    let c = w.people[m as usize].cond.as_mut().unwrap();
    c.settle(t);
    c.stamina = 0.0;
}

/// B5 / BL-48: spent, they walk at half pace and are weaker; an hour's rest
/// brings it back.
#[test]
fn b5_out_of_breath_is_slow_and_weak_and_takes_an_hour_back() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let m = w.squad.members[0];
    let fresh = w.people[m as usize].effective_stats().attr(Attr::Strength);
    let t = w.time;
    let full = w.people[m as usize].cond.as_ref().unwrap().pace_factor(t);
    spend_breath(&mut w, m);
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert!((c.pace_factor(t) - full * 0.5).abs() < 0.05, "pace {} of {}", c.pace_factor(t), full);
    assert!(w.people[m as usize].effective_stats().attr(Attr::Strength) < fresh * 0.9, "winded and as strong");
    // Standing about: back to full in about an hour, and strong again.
    run(&mut w, 0.5, 30.0);
    assert!(w.stamina_of(m).unwrap() < 0.8, "back too soon: {}", w.stamina_of(m).unwrap());
    run(&mut w, 1.0, 30.0);
    assert!(w.stamina_of(m).unwrap() > 0.95, "still out of breath: {}", w.stamina_of(m).unwrap());
    assert!((w.people[m as usize].effective_stats().attr(Attr::Strength) - fresh).abs() < 0.01);
}

/// B5: a fight starts with the breath they had, and it comes back slowly.
#[test]
fn b5_a_fight_starts_with_the_breath_they_had() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let here = w.squad.pos;
    let foe = (0..w.people.len() as u32).find(|&p| w.people[p as usize].bandit && !w.people[p as usize].dead && w.person_pos(p).dist(here) < 20000.0).unwrap();
    let at = w.person_pos(foe);
    w.teleport_squad(at.add(V2::new(3.0, 0.0)));
    spend_breath(&mut w, m);
    // (No mending spells about: a mend gives breath back too.)
    for &s in &w.squad.members.clone() {
        w.people[s as usize].detail.as_mut().unwrap().spells.clear();
    }
    assert!(w.attack(&[m], foe));
    let f = w.fighter(m).unwrap();
    assert!(f.fatigue < 1.0, "fresh at the start: {}", f.fatigue);
    for _ in 0..20 {
        w.step(0.5);
    }
    if let Some(f) = w.fighter(m) {
        assert!(f.fatigue < 0.25 * f.max_fatigue, "breath back in ten seconds: {} of {}", f.fatigue, f.max_fatigue);
    }
}

#[allow(dead_code)]
fn days(w: &mut World, n: f64) {
    for _ in 0..(n * DAY / 600.0) as usize {
        w.step(600.0);
    }
}

/// B6 / NM-78: town money and grain level off instead of piling up.
#[test]
fn b6_treasuries_and_grain_level_off() {
    use gahturiyu_sim::sim::jobs::Good;
    let mut w = worldgen::generate(1);
    let snap = |w: &World| -> Vec<(f32, f32)> { w.society.towns.iter().map(|t| (t.treasury, t.stock[Good::Grain.index()].base)).collect() };
    days(&mut w, 40.0);
    let mid = snap(&w);
    days(&mut w, 40.0);
    let end = snap(&w);
    for (i, (a, b)) in mid.iter().zip(&end).enumerate() {
        let heads = w.settlements[i].residents.len() as f32;
        eprintln!("town {i} ({heads} folk): treasury {:.0} -> {:.0}, grain {:.0} -> {:.0}", a.0, b.0, a.1, b.1);
        // Toward a few weeks' keep, and no further.
        use gahturiyu_sim::sim::economy::{GUARD_WAGE, TAX_PER_HEAD, TREASURY_KEEP_DAYS};
        let guards = w.settlements[i].residents.iter().filter(|&&p| w.life(p).job == gahturiyu_sim::sim::jobs::Job::Guard).count() as f32;
        let keep = (guards * GUARD_WAGE + heads * TAX_PER_HEAD) * TREASURY_KEEP_DAYS;
        assert!(b.0 < a.0 * 1.25 + 50.0 || b.0 < keep * 1.3, "town {i}'s treasury still climbs: {:.0} -> {:.0} (keep {keep:.0})", a.0, b.0);
        assert!(b.1 < a.1 * 1.25 + 50.0, "town {i}'s grain still climbs: {:.0} -> {:.0}", a.1, b.1);
    }
}

/// B2 / NM-10: a member with no food starves, is said to, collapses, and in
/// the end dies; with nobody left, the squad is gone.
#[test]
fn b2_starving_kills_in_the_end() {
    use gahturiyu_sim::sim::items::{item, Kind};
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let m = w.squad.members[0];
    let others: Vec<u32> = w.squad.members.iter().copied().filter(|&x| x != m).collect();
    // The others leave; m has no food and nobody to share.
    for &o in &others {
        w.squad.retain(|x| x != o);
        w.people[o as usize].in_squad = false;
    }
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| !matches!(item(e.0).kind, Kind::Food(_)));
    let mut said = false;
    let mut down_at = None;
    for _ in 0..(10.0 * DAY / 600.0) as usize {
        w.step(600.0);
        said |= w.log.iter().any(|(_, l)| l.contains("is starving"));
        if down_at.is_none() && w.is_down(m) {
            down_at = Some(w.time);
        }
        if w.people[m as usize].dead {
            break;
        }
    }
    assert!(said, "nobody said they were starving");
    let down = down_at.expect("collapsed first");
    assert!(w.people[m as usize].dead, "still alive after ten days without food");
    assert!(w.time - down > DAY, "dead too soon after collapsing: {:.1} h", (w.time - down) / HOUR);
    assert!(w.squad.members.is_empty());
    assert!(w.log.iter().any(|(_, l)| l.contains("starved to death")));
    // The world goes on without them.
    w.step(600.0);
}
