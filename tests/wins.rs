//! Winning pays (play-test 2): a beaten gang leaves bodies to go through,
//! rather than all running off the moment one of them breaks.

use gahturiyu_sim::sim::{geo::V2, stats::SKILLS, worldgen};

/// The squad, made handy, against four ordinary bandits: (left behind, ran).
fn fight(seed: u64) -> (usize, usize) {
    let mut w = worldgen::generate(seed);
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            let v = w.people[m as usize].stats.skill(k).max(45.0);
            w.people[m as usize].stats.set_skill(k, v);
        }
        w.people[m as usize].recompute_might();
    }
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(12.0, 0.0)), 4, false);
    let foes = w.group(band).unwrap().members.clone();
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    let mut last = None;
    while w.squad_battle().is_some() && n < 40_000 {
        last = w.squad_battle().cloned();
        w.step(0.1);
        n += 1;
    }
    let b = last.unwrap();
    let left = b.fighters.iter().filter(|f| foes.contains(&f.pid) && (f.ko || f.dead)).count();
    let ran = b.fighters.iter().filter(|f| foes.contains(&f.pid) && (f.fled || f.fleeing)).count();
    eprintln!("seed {seed}: {left} left behind, {ran} ran; {}", w.log.iter().find(|l| l.1.starts_with("The fight is over")).map(|l| l.1.as_str()).unwrap_or("?"));
    (left, ran)
}

#[test]
fn a_won_fight_leaves_bodies() {
    let mut left = 0;
    let mut ran = 0;
    for seed in 1..=6 {
        let (l, r) = fight(seed);
        left += l;
        ran += r;
    }
    eprintln!("in all: {left} left behind, {ran} ran");
    assert!(left * 3 >= left + ran, "a good share of a beaten gang stays down: {left} left, {ran} ran");
}
