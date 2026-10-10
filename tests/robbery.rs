//! Losing is a story: bandits who beat the squad rob the downed, and what
//! they take can be won back from them.

use gahturiyu_sim::sim::{geo::V2, items, stats::SKILLS, worldgen, World};

fn coin_of(w: &World, who: &[u32]) -> u32 {
    who.iter().map(|&m| w.count_of(m, "coin") as u32).sum()
}

#[test]
fn bandits_rob_a_beaten_squad() {
    let mut w = worldgen::generate(1);
    // A feeble squad with money on them, against a hard band that won't run.
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 50);
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    let had = coin_of(&w, &squad);
    let foes_had = coin_of(&w, &foes);
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ended");
    assert!(w.squad_fit().is_empty(), "the squad lost");
    let alive: Vec<u32> = squad.iter().copied().filter(|&m| !w.people[m as usize].dead).collect();
    assert!(!alive.is_empty());
    assert_eq!(coin_of(&w, &alive), 0, "every coin taken from the living");
    assert!(coin_of(&w, &foes) >= foes_had + alive.len() as u32 * 50, "and it's in the bandits' purses");
    assert!(had > 0);
    assert!(w.log.iter().any(|l| l.1.starts_with("Beaten.")), "{:?}", w.log);
}

#[test]
fn a_beaten_squad_is_left_alone_until_dawn() {
    use gahturiyu_sim::sim::encounters::{next_dawn, CAMP_REST};
    let mut w = worldgen::generate(1);
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 6, false);
    let foes: Vec<u32> = w.group(band).unwrap().members.clone();
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
        for k in SKILLS {
            w.people[f as usize].stats.set_skill(k, 70.0);
        }
        w.people[f as usize].recompute_might();
    }
    assert!(w.attack(&squad, foes[0]));
    let mut n = 0;
    while w.squad_battle().is_some() && n < 40_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_fit().is_empty(), "the squad lost");
    let ended = w.time;
    let camp = w.camps.iter().find(|c| c.group == band).unwrap();
    assert!(camp.ready_at >= ended + CAMP_REST, "they rest first");
    assert_eq!(camp.ready_at, next_dawn(camp.ready_at), "and stay home till a dawn");
    // Right beside their camp, the squad comes round and rests: no second
    // beating before that dawn.
    let until = camp.ready_at;
    while w.time < until - 60.0 {
        w.step(30.0);
        assert!(w.squad_battle().is_none(), "attacked again at {:.1} h (safe till {:.1} h)", w.time / 3600.0, until / 3600.0);
    }
}

#[test]
fn the_downed_come_round_within_a_couple_of_hours() {
    use gahturiyu_sim::sim::body::{knocked_out, Part};
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    let max = w.people[m as usize].stats.max_hp(Part::Torso);
    // Torso at minus half its health: out cold.
    w.people[m as usize].wounds.lost[Part::Torso as usize] = max * 1.5;
    w.people[m as usize].wounds.at = t;
    assert!(knocked_out(&w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)));
    let mut n = 0;
    while knocked_out(&w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)) && n < 2000 {
        w.step(60.0);
        n += 1;
    }
    let hours = (w.time - t) / 3600.0;
    eprintln!("came round after {hours:.2} h");
    assert!(hours < 2.0, "out for {hours:.1} h");
    // And the rest heals at the ordinary rate: no faster than that.
    let hp = w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time)[Part::Torso as usize];
    assert!(hp < max * 0.5, "not healed outright: {hp:.0} of {max:.0}");
}
