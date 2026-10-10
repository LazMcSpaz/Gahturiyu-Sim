//! Fixes from the bug hunt, batch 1 (ruins-gangs): the end of a fight,
//! Unlock with nothing to open, wards that pile up, torches in bed.

use gahturiyu_sim::sim::{
    geo::V2,
    magic,
    stats::{Attr, Calling, Skill, SKILLS},
    worldgen, World,
};

fn squad_mage(w: &World) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap()
}

fn knows_all(w: &mut World, m: u32) {
    let p = &mut w.people[m as usize];
    p.detail.as_mut().unwrap().spells = magic::all_spells().collect();
    for k in [Skill::Felt, Skill::Structured, Skill::Ritual] {
        p.stats.set_skill(k, 90.0);
    }
    p.stats.set_attr(Attr::Willpower, 90.0);
    p.mana = 1000.0;
}

/// A feeble squad sent at a hard band that won't run, after being sent off
/// on a long march: the fight is lost.
fn lose_a_fight(w: &mut World) -> (u32, V2) {
    let squad = w.squad.members.clone();
    for &m in &squad {
        for k in SKILLS {
            w.people[m as usize].stats.set_skill(k, 1.0);
        }
        w.people[m as usize].recompute_might();
    }
    let far = w.squad.pos.add(V2::new(-600.0, 0.0));
    w.order_squad(far);
    w.step(1.0);
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
    assert!(w.squad_battle().is_none(), "the fight ended");
    (band, at)
}

#[test]
fn bl5_bl12_bl16_after_a_fight() {
    let mut w = worldgen::generate(1);
    let (band, at) = lose_a_fight(&mut w);
    // BL-5: the march from before the fight is forgotten.
    for k in 0..w.squad.members.len() {
        assert!(w.squad.route[k].is_empty(), "member {k} still has the old march");
        assert!(w.squad.goal[k].dist(w.squad.at[k]) < 0.01, "member {k} still has the old goal");
    }
    // BL-12: the band that just fought them knows they're there.
    if w.group(band).is_some_and(|g| g.ends > w.time) {
        assert!(w.has_noticed(band), "no catching them unawares straight after");
        // BL-16: and they're near the fight, not hundreds of metres off.
        let g = w.group(band).unwrap();
        assert!(g.position_at(w.time).dist(at) < 80.0, "the band is {} m from the fight", g.position_at(w.time).dist(at));
    }
}

#[test]
fn rg13_unlock_with_no_lock_spends_nothing() {
    let mut w = worldgen::generate(2);
    w.teleport_squad(w.squad.pos.add(V2::new(400.0, 0.0)));
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let t = w.time;
    w.people[m as usize].set_mana(50.0, t);
    let here = w.person_pos(m).add(V2::new(2.0, 0.0));
    let r = w.cast(m, magic::spell("unlock"), None, Some(here));
    assert!(r.is_err(), "refused: no lock there");
    assert!((w.people[m as usize].mana_at(w.time) - 50.0).abs() < 0.5, "and no energy spent");
}

#[test]
fn rg22_a_tripwire_cast_twice_is_one_ward() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let tw = magic::spell("tripwire");
    let mut cast = 0;
    for _ in 0..40 {
        let t = w.time;
        w.people[m as usize].set_mana(1000.0, t);
        if w.cast(m, tw, None, None).is_ok() {
            cast += 1;
        }
        w.step(0.5);
    }
    assert!(cast > 5);
    let wires = w.wards.iter().filter(|x| x.does == gahturiyu_sim::sim::effects::Does::Tripwire).count();
    assert!(wires <= 1, "{wires} tripwires on one spot");
}

#[test]
fn bl45_nobody_sleeps_with_a_lit_torch() {
    let mut w = worldgen::generate(1);
    w.teleport_squad(w.squad.pos.add(V2::new(300.0, 0.0)));
    let who = w.squad.members.clone();
    let m = who[0];
    assert!(w.toggle_torch(m), "lit");
    assert!(w.torch_lit(m));
    w.order_rest(&who);
    for _ in 0..200 {
        w.step(1.0);
    }
    assert!(w.is_asleep(m));
    assert!(!w.torch_lit(m), "put out at bed-down");
    // And it isn't lit again in their sleep.
    for _ in 0..600 {
        w.step(5.0);
    }
    assert!(!w.is_asleep(m) || !w.torch_lit(m));
}
