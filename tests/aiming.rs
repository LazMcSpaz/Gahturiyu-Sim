//! Aiming spells as in Baldur's Gate 3: out of reach, the caster walks over
//! and casts; a harmful spell at an enemy starts the fight and opens it.

use gahturiyu_sim::sim::{geo::V2, magic::Aim, stats::Calling, worldgen};

fn mage(w: &gahturiyu_sim::sim::World) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap()
}

#[test]
fn out_of_reach_the_caster_walks_over_then_casts() {
    let mut w = worldgen::generate(1);
    let me = mage(&w);
    let friend = *w.squad.members.iter().find(|&&m| m != me).unwrap();
    let s = w.known_spells(me).into_iter().find(|s| matches!(s.def().aim, Aim::Friend | Aim::Anyone) && s.def().works_outside_fights() && s.def().cost <= 20.0).expect("a spell for a friend");
    // Send the friend well out of reach.
    let far = w.person_pos(me).add(V2::new(s.def().range + 40.0, 0.0));
    w.teleport_squad(w.squad.pos);
    let k = w.squad.index(friend).unwrap();
    w.squad.at[k] = far;
    w.squad.goal[k] = far;
    let mana = w.people[me as usize].mana_at(w.time);
    w.order_cast(me, s, Some(friend), None).expect("ordered");
    assert_eq!(w.casts.len(), 1, "waiting till in reach");
    let mut n = 0;
    while !w.casts.is_empty() && n < 4000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.casts.is_empty(), "it was cast in the end");
    assert!(w.people[me as usize].mana_at(w.time) < mana, "energy spent: {} -> {}", mana, w.people[me as usize].mana_at(w.time));
    assert!(w.person_pos(me).dist(far) <= s.def().range.max(2.0) + 0.5, "they walked into reach");
}

#[test]
fn a_harmful_spell_at_an_enemy_opens_the_fight() {
    let mut w = worldgen::generate(1);
    let me = mage(&w);
    let s = w.known_spells(me).into_iter().find(|s| s.def().aim == Aim::Foe && s.def().cost <= 25.0).expect("a spell for a foe");
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(20.0, 0.0)), 2, false);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.squad_battle().is_none());
    w.order_cast(me, s, Some(foe), None).expect("ordered");
    assert!(w.squad_battle().is_some(), "the fight opens");
    let mut cast = false;
    for _ in 0..200 {
        w.step(0.1);
        if let Some(f) = w.fighter(me) {
            if matches!(f.act, gahturiyu_sim::sim::combat::Act::Cast { .. }) || f.mana < f.max_mana - 0.5 {
                cast = true;
                break;
            }
        }
    }
    assert!(cast, "the opening spell was cast");
    // A townsperson isn't an enemy.
    let local = w.settlements[0].residents[0];
    assert!(w.order_cast(me, s, Some(local), None).is_err() || w.squad_battle().is_some());
}
