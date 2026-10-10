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
