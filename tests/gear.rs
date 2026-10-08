//! Every kind of enchantment does what it says, both on paper (the numbers
//! the rest of the game reads) and in a fight.

use gahturiyu_sim::sim::{
    combat::{Act, Battle, Fighter},
    geo::V2,
    items::{self, Slot},
    effects::Does,
    magic::{spell, Spell},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
};

fn person(id: u32, race: Race, calling: Calling, skills: &[(Skill, f32)], budget: f32) -> Person {
    let mut p = Person::summary(id, 2000 + id as u64 * 31, race, None);
    p.specialize(calling, skills, budget);
    p.ensure_detail();
    p
}

fn brute(id: u32) -> Person {
    person(id, Race::Roduro, Calling::Warrior, &[(Skill::Blade, 45.0)], 300.0)
}

fn mage() -> Person {
    let mut p = person(1, Race::Tadoro, Calling::Mage, &[(Skill::Structured, 100.0)], 60.0);
    p.stats.set_attr(Attr::Willpower, 100.0);
    p.mana = 5000.0;
    p
}

/// Put an item on someone (from nowhere).
fn wear(p: &mut Person, key: &str) {
    p.detail.as_mut().unwrap().gear.wear(items::id(key));
    p.recompute_might();
}

fn duel(a: &Person, b: &Person) -> Battle {
    let mut fa = Fighter::from_person(a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(b, 1, V2::new(8.0, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, 99, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

/// Cast at fighter 1 `n` times; count how often the status took hold.
fn stick_rate(target: &Person, spell: Spell, kind: Does, n: usize) -> usize {
    let mut b = duel(&mage(), target);
    b.fighters[1].stats.set_attr(Attr::Willpower, 1.0);
    let mut took = 0;
    let mut cast = 0;
    while cast < n {
        b.fighters[1].statuses.clear();
        b.fighters[0].mana = 1000.0;
        b.begin_cast(0, spell, Some(1), b.fighters[1].pos);
        while matches!(b.fighters[0].act, Act::Cast { .. }) {
            b.tick();
        }
        // Only count casts that went off (a fizzle says nothing about resistance).
        let fizzled = b.log.last().map(|l| l.1.contains("fizzle")).unwrap_or(false);
        if fizzled {
            continue;
        }
        cast += 1;
        if b.fighters[1].has(kind).is_some() {
            took += 1;
        }
    }
    took
}

#[test]
fn attribute_enchantments_raise_attributes_and_what_follows_from_them() {
    let plain = brute(2);
    let mut strong = brute(2);
    wear(&mut strong, "ring_might");
    let (a, b) = (plain.effective_stats(), strong.effective_stats());
    assert!((b.attr(Attr::Strength) - a.attr(Attr::Strength) - 12.0).abs() < 0.01);
    assert!(b.carry_capacity() > a.carry_capacity() + 9.0, "stronger should carry more");
    assert!(strong.might > plain.might, "a stronger fighter should rate higher");

    let mut quick = brute(2);
    wear(&mut quick, "ring_swiftness");
    let fq = Fighter::from_person(&quick, 0, V2::default(), 0.0);
    let fp = Fighter::from_person(&plain, 0, V2::default(), 0.0);
    assert!(fq.attack_time() < fp.attack_time(), "more agility should mean quicker swings");
}

#[test]
fn skill_enchantments_raise_the_skill_used_in_fights() {
    let plain = brute(3);
    let mut gloved = brute(3);
    wear(&mut gloved, "duelists_gloves");
    let (fp, fg) = (Fighter::from_person(&plain, 0, V2::default(), 0.0), Fighter::from_person(&gloved, 0, V2::default(), 0.0));
    assert!((fg.stats.skill(Skill::Blade) - fp.stats.skill(Skill::Blade) - 10.0).abs() < 0.01);
    assert!((fg.stats.skill(Skill::Block) - fp.stats.skill(Skill::Block) - 6.0).abs() < 0.01);
    // ...but the person's own, trainable skill is untouched.
    assert_eq!(gloved.stats.skill(Skill::Blade), plain.stats.skill(Skill::Blade));
}

#[test]
fn mana_enchantments_add_mana_and_regeneration() {
    let plain = mage();
    let mut m = mage();
    wear(&mut m, "amulet_wellspring");
    assert!((m.max_mana() - plain.max_mana() - 30.0).abs() < 0.01);
    assert!((m.mana_regen() - plain.mana_regen() - 0.6).abs() < 0.001);
    // Regeneration really happens faster: an hour after emptying.
    let (mut a, mut b) = (plain.clone(), m.clone());
    a.set_mana(0.0, 0.0);
    b.set_mana(0.0, 0.0);
    assert!(b.mana_at(600.0) > a.mana_at(600.0) + 5.0);
    let f = Fighter::from_person(&m, 0, V2::default(), 0.0);
    assert!((f.max_mana - m.max_mana()).abs() < 0.01, "fights must see the bigger pool");
}

#[test]
fn carrying_enchantments_and_packs_raise_capacity() {
    let mut p = brute(4);
    let d = p.detail.as_mut().unwrap();
    d.gear.unequip(Slot::Back);
    let base = d.gear.capacity(&p.stats);
    d.gear.wear(items::id("porters_belt_pack"));
    let with = d.gear.capacity(&p.stats);
    assert!((with - base - 80.0).abs() < 0.01, "frame pack holds 60 kg and its enchantment adds 20: got {}", with - base);
}

#[test]
fn speed_enchantments_make_fighters_faster() {
    let plain = brute(5);
    let mut shod = brute(5);
    wear(&mut shod, "striders_boots");
    let (fp, fs) = (Fighter::from_person(&plain, 0, V2::default(), 0.0), Fighter::from_person(&shod, 0, V2::default(), 0.0));
    assert!(fs.speed() > fp.speed() * 1.15, "{} vs {}", fs.speed(), fp.speed());
}

#[test]
fn paralysis_resistance_works() {
    let plain = brute(6);
    let mut warded = brute(6);
    wear(&mut warded, "amulet_clear_mind");
    let (a, b) = (stick_rate(&plain, spell("paralyze"), Does::Paralyze, 60), stick_rate(&warded, spell("paralyze"), Does::Paralyze, 60));
    assert!(a >= 55, "unwarded should nearly always be held: {a}/60");
    assert!(b <= 35 && b >= 10, "a 60% ward should stop most: {b}/60");
}

#[test]
fn blindness_resistance_works() {
    let plain = brute(7);
    let mut hooded = brute(7);
    wear(&mut hooded, "seers_hood");
    let (a, b) = (stick_rate(&plain, spell("blind"), Does::Blind, 60), stick_rate(&hooded, spell("blind"), Does::Blind, 60));
    assert!(a >= 55, "{a}/60");
    assert!(b <= 30, "a 70% ward should stop most: {b}/60");
}

#[test]
fn elemental_resistance_cuts_fire_and_lightning() {
    let burn = |ring: bool, spell: Spell| {
        let mut t = brute(8);
        if ring {
            wear(&mut t, "ring_hearth");
        }
        let mut b = duel(&mage(), &t);
        let before: f32 = b.fighters[1].hp.iter().sum();
        for _ in 0..20 {
            b.begin_cast(0, spell, Some(1), b.fighters[1].pos);
            while matches!(b.fighters[0].act, Act::Cast { .. }) {
                b.tick();
            }
            let now: f32 = b.fighters[1].hp.iter().sum();
            if now < before {
                return before - now;
            }
        }
        panic!("never landed");
    };
    for spell in [spell("fireball"), spell("lightning_bolt")] {
        let (bare, ringed) = (burn(false, spell), burn(true, spell));
        assert!((ringed / bare - 0.6).abs() < 0.02, "{spell:?}: {ringed} vs {bare}");
    }
}
