//! Fixes from the bug hunt, batch 2 (ruins-gangs): who stays down after a
//! fight, attacking out of reach, beaten camps, learning to sneak.

use gahturiyu_sim::sim::{
    geo::V2,
    stats::{Skill, SKILLS},
    worldgen, World,
};

/// A strong squad against a weak band that won't run: the squad wins, and
/// returns (band, who of the band lay out cold when it ended, where).
fn win_a_fight(w: &mut World) -> (u32, Vec<(u32, V2)>) {
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 70.0);
        }
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 4, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 5.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut last = Vec::new();
    let mut n = 0;
    while n < 40_000 {
        match w.squad_battle() {
            Some(b) => last = b.fighters.iter().filter(|f| f.is_person() && f.ko && !f.dead).map(|f| (f.pid, f.pos)).collect(),
            None => break,
        }
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ended");
    (band, last)
}

#[test]
fn bl11_bl13_the_out_cold_stay_down_where_they_fell() {
    let mut w = worldgen::generate(1);
    let (_, out) = win_a_fight(&mut w);
    assert!(!out.is_empty(), "somebody was knocked out");
    for &(p, at) in &out {
        if w.people[p as usize].dead {
            continue;
        }
        // BL-11: still down the moment it ends.
        assert!(w.is_down(p), "{p} stood up as the fight ended");
        // BL-13: and lying where they fell, not walking home with the band.
        assert!(w.person_pos(p).dist(at) < 1.0, "{p} moved {:.0} m", w.person_pos(p).dist(at));
    }
}

#[test]
fn bl10_no_attack_on_a_bandit_kilometres_away() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    let far = w.camps.iter().map(|c| c.pos).filter(|p| p.dist(here) > 1500.0).min_by(|a, b| a.dist(here).total_cmp(&b.dist(here))).expect("a far camp");
    let bandit = w.people.iter().find(|p| p.bandit && w.person_pos(p.id).dist(far) < 60.0).map(|p| p.id).expect("a bandit at that camp");
    let all = w.squad.members.clone();
    assert!(!w.attack(&all, bandit), "a fight started with a bandit {:.0} m away", w.person_pos(bandit).dist(here));
}

#[test]
fn bl23_a_beaten_camp_is_on_its_feet_again_after_its_rest() {
    let mut w = worldgen::generate(1);
    let c = w.camps[0].clone();
    w.beaten_camps.insert(c.group);
    w.camps[0].ready_at = w.time + 3600.0;
    w.step(1.0);
    assert!(w.beaten_camps.contains(&c.group), "still beaten while they rest");
    while w.time < c.ready_at.max(w.camps[0].ready_at) + 60.0 {
        w.step(60.0);
    }
    assert!(!w.beaten_camps.contains(&c.group), "back on their feet after it");
}

#[test]
fn bl6_crouching_still_by_a_camp_teaches_nothing() {
    let mut w = worldgen::generate(1);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(41.0, 0.0)));
    let who = w.squad.members.clone();
    for &m in &who {
        w.set_sneaking(m, true);
    }
    let before: Vec<f32> = who.iter().map(|&m| w.people[m as usize].stats.skill(Skill::Sneak)).collect();
    for _ in 0..60 {
        w.step(10.0);
        if w.squad_battle().is_some() {
            return; // spotted: nothing to check
        }
    }
    for (k, &m) in who.iter().enumerate() {
        let now = w.people[m as usize].stats.skill(Skill::Sneak);
        assert!(now - before[k] < 0.5, "{m}'s Sneak rose {:.1} standing still", now - before[k]);
    }
}
