//! A band that notices the squad comes for those on their feet: it doesn't
//! start a fight with people already down (a fight over within a step, seen
//! by the second session while fixing `robbery`), and the fallen aren't
//! drawn into the fight it starts with the rest.

use gahturiyu_sim::sim::{body::Part, geo::V2, worldgen};

fn knock_out(w: &mut gahturiyu_sim::sim::World, m: u32) {
    let t = w.time;
    let p = &mut w.people[m as usize];
    p.wounds.lost[1] = p.stats.max_hp(Part::Torso) + 10.0;
    p.wounds.at = t;
}

#[test]
fn a_band_doesnt_start_a_fight_with_the_fallen() {
    let mut w = worldgen::generate(21);
    for m in w.squad.members.clone() {
        knock_out(&mut w, m);
    }
    assert!(w.squad.members.iter().all(|&m| w.is_down(m)));
    let at = w.squad.pos.add(V2::new(6.0, 0.0));
    w.spawn_bandits(at, 3, false);
    for _ in 0..240 {
        w.step(0.5);
    }
    assert!(w.squad_battle().is_none(), "no fight with people already down: {:?}", w.log);
    assert!(!w.log.iter().any(|(_, l)| l.contains("attacked")), "{:?}", w.log);
}

#[test]
fn only_the_fit_are_drawn_into_a_bands_attack() {
    let mut w = worldgen::generate(21);
    let down = w.squad.members[2];
    knock_out(&mut w, down);
    let at = w.squad.pos.add(V2::new(6.0, 0.0));
    w.spawn_bandits(at, 3, false);
    for _ in 0..240 {
        w.step(0.5);
        if w.squad_battle().is_some() {
            break;
        }
    }
    let b = w.squad_battle().expect("the band comes for the three on their feet");
    assert!(!b.fighters.iter().any(|f| f.pid == down), "the fallen one is left out of it");
    assert!(b.fighters.iter().filter(|f| f.side == gahturiyu_sim::sim::combat::SQUAD_SIDE).count() == 3);
}
