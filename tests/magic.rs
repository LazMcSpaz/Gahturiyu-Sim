//! Magic: the three styles, and what each costs.

use gahturiyu_sim::sim::{
    combat::{Act, Battle, Fighter},
    geo::V2,
    magic::{Spell, Style},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
    worldgen, World,
};

fn person(id: u32, race: Race, calling: Calling, skills: &[(Skill, f32)], budget: f32) -> Person {
    let mut p = Person::summary(id, 4000 + id as u64 * 13, race, None);
    p.specialize(calling, skills, budget);
    p.ensure_detail();
    p
}

fn mage() -> Person {
    let mut p = person(1, Race::Tadoro, Calling::Mage, &[(Skill::Structured, 70.0), (Skill::Felt, 70.0), (Skill::Ritual, 70.0)], 60.0);
    p.stats.set_attr(Attr::Willpower, 80.0);
    p.mana = 500.0;
    p
}

fn brute(id: u32) -> Person {
    person(id, Race::Qotiro, Calling::Warrior, &[(Skill::Blade, 50.0)], 400.0)
}

/// Nobody thinks for themselves.
fn duel(a: &Person, b: &Person) -> Battle {
    let mut fa = Fighter::from_person(a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(b, 1, V2::new(6.0, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, 99, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

#[test]
fn felt_spells_go_off_at_once_and_tire_the_caster() {
    let mut b = duel(&mage(), &brute(2));
    assert_eq!(Spell::Heal.def().style, Style::Felt);
    b.fighters[0].hp[1] -= 30.0;
    let hurt = b.fighters[0].hp[1];
    assert!(b.begin_cast(0, Spell::Heal, Some(0), V2::new(0.0, 0.0)));
    b.tick();
    assert!(!matches!(b.fighters[0].act, Act::Cast { .. }), "a felt spell shouldn't take a cast time");
    assert!(b.fighters[0].hp[1] > hurt, "it should have worked in the same moment");
    assert!(b.fighters[0].tire > 0.0, "felt casting tires");
}

#[test]
fn structured_spells_take_a_second_or_two_and_a_hit_spoils_them() {
    for spell in [Spell::Fireball, Spell::LightningBolt, Spell::Paralyze, Spell::Blind, Spell::MageArmor, Spell::Haste] {
        let mut b = duel(&mage(), &brute(2));
        assert_eq!(spell.def().style, Style::Structured);
        assert!(b.begin_cast(0, spell, Some(1), V2::new(6.0, 0.0)));
        let Act::Cast { done, .. } = b.fighters[0].act else { panic!("not casting") };
        assert!((0.9..=2.1).contains(&(done - b.time)), "{spell:?} casts in {} s", done - b.time);
        assert_eq!(b.fighters[0].tire, 0.0);
        // A solid hit mid-cast spoils it; the energy is gone anyway.
        b.tick();
        let mana = b.fighters[0].mana;
        b.wound(0, gahturiyu_sim::sim::body::Part::Torso, 10.0);
        assert!(!matches!(b.fighters[0].act, Act::Cast { .. }), "{spell:?} should be interrupted");
        assert!(b.fighters[0].mana <= mana);
    }
}

#[test]
fn felt_magic_rarely_fails() {
    let mut b = duel(&mage(), &brute(2));
    b.fighters[0].stats.set_skill(Skill::Felt, 15.0);
    let mut worked = 0;
    for _ in 0..100 {
        b.fighters[0].hp[1] = b.fighters[0].max_hp[1] - 40.0;
        b.fighters[0].mana = 100.0;
        assert!(b.begin_cast(0, Spell::Heal, Some(0), V2::new(0.0, 0.0)));
        b.tick();
        if b.fighters[0].hp[1] > b.fighters[0].max_hp[1] - 39.0 {
            worked += 1;
        }
        while !matches!(b.fighters[0].act, Act::Idle) {
            b.tick();
        }
    }
    assert!(worked >= 80, "a novice's felt spell should still mostly work ({worked}/100)");
}

/// The squad's mage in a fight that's about to start.
fn squad_fight(seed: u64) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let at = w.squad.pos.add(V2::new(18.0, 4.0));
    w.spawn_bandits(at, 2, false);
    let mage = *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap();
    while w.battles.is_empty() {
        w.step(0.25);
    }
    (w, mage)
}

fn finish(w: &mut World) {
    while !w.battles.is_empty() {
        w.step(0.5);
    }
}

#[test]
fn felt_casting_adds_to_the_squads_tiredness() {
    let (mut plain, mage) = squad_fight(2);
    let (mut tiring, _) = squad_fight(2);
    let f = tiring.battles[0].fighters.iter_mut().find(|f| f.pid == mage).unwrap();
    f.tire += 6.0;
    finish(&mut plain);
    finish(&mut tiring);
    let t = plain.time.max(tiring.time);
    let (a, b) = (plain.tired_of(mage).unwrap(), tiring.tired_of(mage).unwrap());
    let _ = t;
    assert!((b - a - 6.0).abs() < 0.5, "tiredness {a} vs {b}");
}

#[test]
fn felt_spells_are_learned_by_using_the_style() {
    let (mut w, mage) = squad_fight(2);
    // Just short of mending; a fight's worth of felt casting gets them there.
    let p = &mut w.people[mage as usize];
    p.stats.set_skill(Skill::Felt, Spell::Heal.def().min_skill - 0.5);
    p.detail.as_mut().unwrap().spells.retain(|&s| s != Spell::Heal);
    let f = w.battles[0].fighters.iter_mut().find(|f| f.pid == mage).unwrap();
    f.spells.retain(|&s| s != Spell::Heal);
    f.trained[Skill::Felt as usize] += 10.0;
    finish(&mut w);
    let p = &w.people[mage as usize];
    assert!(p.stats.skill(Skill::Felt) >= Spell::Heal.def().min_skill);
    assert!(p.detail.as_ref().unwrap().spells.contains(&Spell::Heal), "the feel for mending should have come");
}

#[test]
fn every_spell_has_a_domain() {
    use gahturiyu_sim::sim::magic::{Domain, SPELLS};
    for s in SPELLS {
        let _: Domain = s.def().domain;
    }
    assert_eq!(Spell::Fireball.def().domain, Domain::Elemental);
    assert_eq!(Spell::Paralyze.def().domain, Domain::Psychic);
    assert_eq!(Spell::Blind.def().domain, Domain::Illusion);
    assert_eq!(Spell::Heal.def().domain, Domain::Vital);
    assert_eq!(Spell::MageArmor.def().domain, Domain::Warding);
}

/// Damage from one lightning bolt, with the caster's and target's domain
/// numbers set. The same seed every time, so only the hooks differ.
fn bolt(dom: gahturiyu_sim::sim::magic::Domain, power: f32, resist: f32) -> f32 {
    let mut b = duel(&mage(), &brute(2));
    b.fighters[0].domain_power[dom as usize] = power;
    b.fighters[1].domain_resist[dom as usize] = resist;
    let before: f32 = b.fighters[1].hp.iter().sum();
    for _ in 0..40 {
        b.fighters[0].mana = 100.0;
        assert!(b.begin_cast(0, Spell::LightningBolt, Some(1), V2::new(6.0, 0.0)));
        while matches!(b.fighters[0].act, Act::Cast { .. }) {
            b.tick();
        }
        let now: f32 = b.fighters[1].hp.iter().sum();
        if now < before {
            return before - now;
        }
    }
    panic!("never hit");
}

#[test]
fn domains_are_hooks_for_strength_and_wards() {
    use gahturiyu_sim::sim::magic::Domain;
    let plain = bolt(Domain::Elemental, 0.0, 0.0);
    assert!((bolt(Domain::Elemental, 0.5, 0.0) / plain - 1.5).abs() < 0.01, "a +50% elemental boost");
    assert!((bolt(Domain::Elemental, 0.0, 0.5) / plain - 0.5).abs() < 0.01, "a 50% elemental ward");
    // Another domain's numbers don't touch an elemental spell.
    assert!((bolt(Domain::Vital, 3.0, 0.9) - plain).abs() < 1e-3);
}
