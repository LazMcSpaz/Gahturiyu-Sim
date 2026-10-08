//! Limbs battered past the limit are lost for good.

use gahturiyu_sim::sim::{
    body::{self, Part, LIMB_LOSS},
    combat::{Battle, Fighter},
    geo::V2,
    items,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen,
};

fn warrior(id: u32) -> Person {
    let mut p = Person::summary(id, 9100 + id as u64, Race::Qotiro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blunt, 50.0), (Skill::Block, 40.0)], 400.0);
    p.ensure_detail();
    p
}

fn duel(a: &Person, b: &Person) -> Battle {
    let mut fa = Fighter::from_person(a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(b, 1, V2::new(1.5, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, 3, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

#[test]
fn a_limb_battered_past_the_limit_is_lost_and_never_heals() {
    let p = warrior(1);
    let mut b = duel(&p, &warrior(2));
    let max = b.fighters[0].max_hp[Part::LeftArm as usize];
    b.wound(0, Part::LeftArm, max * 1.5);
    assert!(!b.fighters[0].missing[Part::LeftArm as usize], "ruined, not lost yet");
    b.wound(0, Part::LeftArm, max * LIMB_LOSS);
    assert!(b.fighters[0].missing[Part::LeftArm as usize], "lost");
    // Write it back the way a fight does, then let years pass.
    let mut person = p.clone();
    let stats = person.stats.clone();
    person.wounds.set(&stats, &b.fighters[0].hp, 0.0);
    person.wounds.missing = b.fighters[0].missing;
    let much_later = person.wounds.hp_at(&stats, 1e7);
    assert!(much_later[Part::LeftArm as usize] <= -max * LIMB_LOSS + 0.01, "a lost limb never heals");
    assert!(much_later[Part::RightArm as usize] > 0.0);
    assert_eq!(person.wounds.lost_limbs(), vec!["left arm"]);
    // Healing (a potion, a spell) can't bring it back either.
    let mut hp = much_later;
    hp[Part::LeftArm as usize] = max;
    person.wounds.set(&stats, &hp, 1e7);
    assert!(person.wounds.hp_at(&stats, 1e7)[Part::LeftArm as usize] < 0.0);
}

#[test]
fn a_lost_arm_holds_no_shield_and_no_two_handed_weapon() {
    let mut p = warrior(3);
    let max = p.stats.max_hp(Part::LeftArm);
    p.wounds.lost[Part::LeftArm as usize] = max * (1.0 + LIMB_LOSS);
    p.wounds.missing[Part::LeftArm as usize] = true;
    // A two-handed maul: fists only.
    let g = &mut p.detail.as_mut().unwrap().gear;
    g.add(items::id("stone_maul"), 1);
    g.equip(items::id("stone_maul")).unwrap();
    let f = Fighter::from_person(&p, 0, V2::default(), 0.0);
    assert_eq!(f.usable_weapon().unwrap().0.skill, Skill::Unarmed);
    // A shield gives nothing.
    let g = &mut p.detail.as_mut().unwrap().gear;
    g.add(items::id("short_sword"), 1);
    g.equip(items::id("short_sword")).unwrap();
    g.add(items::id("kite_shield"), 1);
    g.equip(items::id("kite_shield")).unwrap();
    let f = Fighter::from_person(&p, 0, V2::default(), 0.0);
    assert_eq!(f.shield_up(), 0.0);
    assert_eq!(f.usable_weapon().unwrap().0.skill, Skill::Blade, "a one-handed sword is fine");
}

#[test]
fn the_squad_cant_strap_a_shield_to_a_lost_arm() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    w.people[m as usize].wounds.missing[Part::LeftArm as usize] = true;
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("kite_shield"), 1);
    assert!(!w.equip(m, items::id("kite_shield")));
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("glaive"), 1);
    assert!(!w.equip(m, items::id("glaive")));
}

#[test]
fn one_lost_leg_is_a_limp_for_good_and_two_mean_crawling() {
    let mut p = warrior(4);
    let stats = p.stats.clone();
    let leg = Part::LeftLeg as usize;
    p.wounds.lost[leg] = stats.max_hp(Part::LeftLeg) * (1.0 + LIMB_LOSS);
    p.wounds.missing[leg] = true;
    let hp = p.wounds.hp_at(&stats, 1e7);
    assert!((body::leg_factor(&hp) - 0.55).abs() < 1e-6, "a permanent limp");
    let other = Part::RightLeg as usize;
    p.wounds.lost[other] = stats.max_hp(Part::RightLeg) * (1.0 + LIMB_LOSS);
    p.wounds.missing[other] = true;
    let hp = p.wounds.hp_at(&stats, 1e7);
    assert!((body::leg_factor(&hp) - 0.2).abs() < 1e-6, "crawling");
}

#[test]
fn fights_write_lost_limbs_back_to_everyone() {
    // Same rule for anyone: a stranger's lost limb is kept on them too.
    let mut w = worldgen::generate(1);
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(20.0, 0.0)), 2, false);
    let bandit = w.group(g).unwrap().members[0];
    let mut lost = false;
    for _ in 0..4000 {
        w.step(0.25);
        if let Some(bid) = w.fighting.get(&bandit).copied() {
            if !lost {
                let b = w.battles.iter_mut().find(|b| b.id == bid).unwrap();
                let i = b.index_of(bandit).unwrap();
                let max = b.fighters[i].max_hp[Part::RightLeg as usize];
                b.wound(i, Part::RightLeg, max * (1.0 + LIMB_LOSS) + 1.0);
                lost = true;
            }
        }
        if lost && w.battles.is_empty() {
            break;
        }
    }
    assert!(lost);
    assert!(w.people[bandit as usize].wounds.missing[Part::RightLeg as usize]);
}
