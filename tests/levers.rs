//! The player's levers in a fight (play-test 2): a fireball scroll read at a
//! gang opens the fight with it; a squad caster keeps energy back for what
//! you order; a weapon they can't use is pointed out.

use gahturiyu_sim::sim::{geo::V2, items, magic, worldgen};

#[test]
fn a_scroll_read_at_a_gang_opens_the_fight_with_it() {
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    let scroll = items::id("scroll_fireball");
    w.people[me as usize].detail.as_mut().unwrap().gear.add(scroll, 1);
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(15.0, 0.0)), 3, false);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.squad_battle().is_none());
    w.order_read(me, scroll, None, Some(w.person_pos(foe))).unwrap();
    let mut read = false;
    for _ in 0..400 {
        w.step(0.1);
        if let Some(b) = w.squad_battle() {
            read |= b.log.iter().any(|l| l.1.contains("reads a scroll of fireball"));
        }
        if read {
            break;
        }
    }
    assert!(read, "the fight opened with the fireball");
}

#[test]
fn a_harmful_spell_cast_at_a_spot_by_enemies_opens_the_fight() {
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    let fb = magic::spell("fireball");
    w.people[me as usize].detail.as_mut().unwrap().spells.push(fb);
    let t = w.time;
    w.people[me as usize].set_mana(200.0, t);
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(15.0, 0.0)), 3, false);
    let foe = w.group(band).unwrap().members[0];
    if !w.knows(me, fb) {
        return;
    }
    w.order_cast(me, fb, None, Some(w.person_pos(foe))).unwrap();
    for _ in 0..200 {
        w.step(0.1);
    }
    assert!(w.squad_battle().is_some(), "the fight is on");
}

#[test]
fn a_weapon_they_cant_use_is_pointed_out() {
    use gahturiyu_sim::sim::stats::Skill;
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    w.people[me as usize].stats.set_skill(Skill::Blade, 5.0);
    w.people[me as usize].stats.set_skill(Skill::Blunt, 60.0);
    let sword = items::id("short_sword");
    w.people[me as usize].detail.as_mut().unwrap().gear.add(sword, 1);
    assert!(w.equip(me, sword));
    assert!(w.log.front().unwrap().1.contains("fight far better with blunt"), "{:?}", w.log.front());
}

#[test]
fn a_squad_caster_keeps_energy_back_for_orders() {
    use gahturiyu_sim::sim::{ai::RESERVE, stats::Skill};
    let mut w = worldgen::generate(1);
    let me = w.squad.members[0];
    for k in ["spark", "lightning_bolt"] {
        w.people[me as usize].detail.as_mut().unwrap().spells.push(magic::spell(k));
    }
    w.people[me as usize].stats.set_skill(Skill::Felt, 60.0);
    w.people[me as usize].stats.set_skill(Skill::Structured, 60.0);
    w.people[me as usize].recompute_might();
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(12.0, 0.0)), 4, false);
    let foe = w.group(band).unwrap().members[0];
    let squad = w.squad.members.clone();
    assert!(w.attack(&squad, foe));
    let mut low = f32::MAX;
    let mut max = 0.0f32;
    let mut cast = false;
    for _ in 0..6000 {
        w.step(0.1);
        let Some(f) = w.fighter(me) else { break };
        low = low.min(f.mana);
        max = f.max_mana;
        cast |= w.squad_battle().is_some_and(|b| b.log.iter().any(|l| l.1.contains("casts")));
    }
    eprintln!("lowest {low:.0} of {max:.0}; cast anything: {cast}");
    assert!(low >= max * RESERVE - 0.5, "kept back a reserve: lowest {low:.0} of {max:.0}");
}
