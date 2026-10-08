//! Magic: the three styles, and what each costs.

use gahturiyu_sim::sim::{
    combat::{Act, Battle, Fighter},
    effects::Does,
    geo::V2,
    magic::{spell, Style},
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
    assert_eq!(spell("heal").def().style, Style::Felt);
    b.fighters[0].hp[1] -= 30.0;
    let hurt = b.fighters[0].hp[1];
    assert!(b.begin_cast(0, spell("heal"), Some(0), V2::new(0.0, 0.0)));
    b.tick();
    assert!(!matches!(b.fighters[0].act, Act::Cast { .. }), "a felt spell shouldn't take a cast time");
    assert!(b.fighters[0].hp[1] > hurt, "it should have worked in the same moment");
    assert!(b.fighters[0].tire > 0.0, "felt casting tires");
}

#[test]
fn structured_spells_take_a_second_or_two_and_a_hit_spoils_them() {
    for spell in [spell("fireball"), spell("lightning_bolt"), spell("paralyze"), spell("blind"), spell("mage_armor"), spell("haste")] {
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
        assert!(b.begin_cast(0, spell("heal"), Some(0), V2::new(0.0, 0.0)));
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
    p.stats.set_skill(Skill::Felt, spell("heal").def().min_skill - 0.5);
    p.detail.as_mut().unwrap().spells.retain(|&s| s != spell("heal"));
    let f = w.battles[0].fighters.iter_mut().find(|f| f.pid == mage).unwrap();
    f.spells.retain(|&s| s != spell("heal"));
    f.trained[Skill::Felt as usize] += 10.0;
    finish(&mut w);
    let p = &w.people[mage as usize];
    assert!(p.stats.skill(Skill::Felt) >= spell("heal").def().min_skill);
    assert!(p.detail.as_ref().unwrap().spells.contains(&spell("heal")), "the feel for mending should have come");
}

#[test]
fn every_spell_has_a_domain() {
    use gahturiyu_sim::sim::magic::{all_spells, Domain};
    for s in all_spells() {
        let _: Domain = s.def().domain;
    }
    assert_eq!(spell("fireball").def().domain, Domain::Elemental);
    assert_eq!(spell("paralyze").def().domain, Domain::Psychic);
    assert_eq!(spell("blind").def().domain, Domain::Illusion);
    assert_eq!(spell("heal").def().domain, Domain::Vital);
    assert_eq!(spell("mage_armor").def().domain, Domain::Warding);
}

/// Damage from one lightning bolt, with the caster's and target's domain
/// numbers set. The same seed every time, so only the hooks differ.
fn bolt(dom: gahturiyu_sim::sim::magic::Domain, power: f32, resist: f32) -> f32 {
    let mut b = duel(&mage(), &brute(2));
    b.fighters[0].worn.push((Does::DomainPower(dom), power));
    b.fighters[1].worn.push((Does::DomainResist(dom), resist));
    let before: f32 = b.fighters[1].hp.iter().sum();
    for _ in 0..40 {
        b.fighters[0].mana = 100.0;
        assert!(b.begin_cast(0, spell("lightning_bolt"), Some(1), V2::new(6.0, 0.0)));
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

// ---- One shared effect list ------------------------------------------------

#[test]
fn the_enchanted_items_keep_their_bonuses() {
    use gahturiyu_sim::sim::{effects::Lasts, items};
    let expect: &[(&str, Does, f32)] = &[
        ("ring_swiftness", Does::MoveSpeed, 0.15),
        ("ring_swiftness", Does::Attr(Attr::Agility), 5.0),
        ("ring_might", Does::Attr(Attr::Strength), 12.0),
        ("amulet_wellspring", Does::MaxEnergy, 30.0),
        ("amulet_wellspring", Does::EnergyRegen, 0.6),
        ("amulet_clear_mind", Does::ResistParalysis, 0.6),
        ("amulet_clear_mind", Does::Attr(Attr::Willpower), 6.0),
        ("ring_hearth", Does::ResistElements, 0.4),
        ("seers_hood", Does::ResistBlind, 0.7),
        ("seers_hood", Does::Skill(Skill::Structured), 8.0),
        ("striders_boots", Does::MoveSpeed, 0.20),
        ("striders_boots", Does::Skill(Skill::Athletics), 10.0),
        ("porters_belt_pack", Does::Carry, 20.0),
        ("duelists_gloves", Does::Skill(Skill::Blade), 10.0),
        ("duelists_gloves", Does::Skill(Skill::Block), 6.0),
    ];
    let mut enchanted = std::collections::HashSet::new();
    for (key, does, power) in expect {
        let e = items::item(items::id(key)).effects.iter().find(|e| e.does == *does).unwrap_or_else(|| panic!("{key} lost {does:?}"));
        assert_eq!(e.power, *power, "{key}");
        assert_eq!(e.lasts, Lasts::Worn);
        enchanted.insert(*key);
    }
    assert_eq!(enchanted.len(), 9);
    // Worn effects reach the wearer's numbers.
    let mut p = brute(3);
    let plain = p.effective_stats().attr(Attr::Strength);
    let d = p.detail.as_mut().unwrap();
    d.gear.add(items::id("ring_might"), 1);
    d.gear.equip(items::id("ring_might")).unwrap();
    assert!((p.effective_stats().attr(Attr::Strength) - plain - 12.0).abs() < 0.01);
}

#[test]
fn potions_and_scrolls_say_what_they_do_with_the_same_effects() {
    use gahturiyu_sim::sim::items::{self, Kind};
    let heal = items::item(items::id("healing_draught"));
    assert_eq!(heal.kind, Kind::Potion);
    assert_eq!(heal.effects[0].does, Does::Heal);
    assert_eq!(items::item(items::id("mana_tonic")).effects[0].does, Does::Energy);
    // A scroll carries its spell, and the spell carries the effects.
    let Kind::Scroll(key) = items::item(items::id("scroll_fireball")).kind else { panic!() };
    assert_eq!(spell(key), spell("fireball"));
    assert!(matches!(spell(key).def().effects[0].does, Does::Damage(_)));
    // Every effect of every spell and item can describe itself.
    for s in gahturiyu_sim::sim::magic::all_spells() {
        for e in s.def().effects {
            assert!(!e.describe().is_empty());
        }
    }
}

#[test]
fn a_healing_scroll_works_on_the_road_but_an_attack_scroll_waits_for_a_fight() {
    use gahturiyu_sim::sim::items;
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    let p = &mut w.people[m as usize];
    let base = p.stats.clone();
    let mut hp = p.wounds.hp_at(&base, t);
    hp[1] -= 30.0;
    p.wounds.set(&base, &hp, t);
    let d = p.detail.as_mut().unwrap();
    d.gear.add(items::id("scroll_heal"), 1);
    d.gear.add(items::id("scroll_fireball"), 1);
    let before = w.people[m as usize].wounds.hp_at(&base, t)[1];
    assert!(w.use_item(m, items::id("scroll_heal")));
    assert!(w.people[m as usize].wounds.hp_at(&base, t)[1] > before + 10.0);
    assert!(!w.use_item(m, items::id("scroll_fireball")), "an attack scroll is for a fight");
}
