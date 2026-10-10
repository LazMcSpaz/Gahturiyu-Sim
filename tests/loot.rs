//! Looting the beaten: a squad member sent to a downed bandit goes over and
//! can take what they wear and carry.

use gahturiyu_sim::sim::{geo::V2, items, stats::{Skill, SKILLS}, worldgen};

#[test]
fn a_beaten_bandit_can_be_stripped() {
    let mut w = worldgen::generate(1);
    // A squad of veterans, and a small band of bandits beside them.
    for m in w.squad.members.clone() {
        for k in SKILLS {
            if matches!(k, Skill::Blunt | Skill::Spear | Skill::Blade | Skill::Dodge | Skill::Block) {
                w.people[m as usize].stats.set_skill(k, 80.0);
            }
        }
        w.people[m as usize].recompute_might();
    }
    let at = w.squad.pos.add(V2::new(12.0, 0.0));
    let band = w.spawn_bandits(at, 2, false);
    let foes: Vec<_> = w.group(band).unwrap().members.clone();
    // Too proud to run.
    for &f in &foes {
        w.people[f as usize].traits.boldness = 1.0;
    }
    let all = w.squad.members.clone();
    assert!(w.attack(&all, foes[0]), "the squad goes in");
    let mut n = 0;
    while w.squad_battle().is_some() && n < 20_000 {
        w.step(0.1);
        n += 1;
    }
    assert!(w.squad_battle().is_none(), "the fight ended");
    // (One lying on land: the sea is off-limits to the squad.)
    let body = *foes.iter().find(|&&f| w.can_loot(f) && !w.terrain.is_sea(w.person_pos(f))).expect("a beaten bandit to loot");
    let looter = w.squad_fit()[0];
    let coin_before = w.count_of(looter, "coin");
    let had = w.loot_of(body).len();
    assert!(had > 0, "they carry something");
    assert!(w.order_loot(looter, body));
    let mut n = 0;
    while w.looting_now(looter).is_none() && n < 2000 {
        w.step(0.1);
        n += 1;
    }
    assert_eq!(w.looting_now(looter), Some(body), "the looter got there");
    let took = w.take_all_loot(looter, body);
    assert_eq!(took, had);
    assert!(w.loot_of(body).is_empty(), "stripped bare");
    assert!(w.count_of(looter, "coin") > coin_before, "and the bandit's purse went too");
    // Townsfolk aren't fair game.
    let local = w.settlements[0].residents[0];
    assert!(!w.can_loot(local));
    let _ = items::id("coin");
}
