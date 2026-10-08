//! The spell list, domain by domain: each spell does what it says.

use gahturiyu_sim::sim::{
    body::Part,
    combat::{Act, Battle, Fighter},
    effects::Does,
    geo::V2,
    items,
    magic::{spell, Spell},
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
    worldgen, World,
};

fn person(id: u32, race: Race, calling: Calling, skills: &[(Skill, f32)], budget: f32) -> Person {
    let mut p = Person::summary(id, 7000 + id as u64 * 11, race, None);
    p.specialize(calling, skills, budget);
    p.ensure_detail();
    p
}

/// A mage who knows every spell and rarely fails.
fn mage() -> Person {
    let mut p = person(1, Race::Tadoro, Calling::Mage, &[(Skill::Structured, 90.0), (Skill::Felt, 90.0), (Skill::Ritual, 90.0)], 60.0);
    p.stats.set_attr(Attr::Willpower, 90.0);
    p.detail.as_mut().unwrap().spells = gahturiyu_sim::sim::magic::all_spells().collect();
    p
}

fn brute(id: u32) -> Person {
    let mut p = person(id, Race::Qotiro, Calling::Warrior, &[(Skill::Blade, 50.0)], 400.0);
    p.stats.set_attr(Attr::Willpower, 1.0);
    p
}

/// Mage (0) and others; nobody thinks for themselves.
fn fight(others: &[(Person, u8, V2)]) -> Battle {
    let mut fs = vec![Fighter::from_person(&mage(), 0, V2::new(0.0, 0.0), 0.0)];
    let mut names = vec!["Mage".to_string()];
    for (k, (p, side, at)) in others.iter().enumerate() {
        fs.push(Fighter::from_person(p, *side, *at, 0.0));
        names.push(format!("F{}", k + 1));
    }
    for f in fs.iter_mut() {
        f.think_at = f64::INFINITY;
        f.mana = 1000.0;
        f.max_mana = 1000.0;
    }
    Battle::new(0, 31, 0.0, fs, names)
}

/// Cast (or release, for a ritual) until `check` holds. Panics if it never does.
fn cast_until(b: &mut Battle, s: Spell, target: Option<usize>, point: V2, check: impl Fn(&Battle) -> bool) {
    for _ in 0..30 {
        b.fighters[0].mana = 1000.0;
        if s.def().style == gahturiyu_sim::sim::magic::Style::Ritual {
            b.fighters[0].held = Some(s);
            assert!(b.release(0, target, point));
        } else {
            assert!(b.begin_cast(0, s, target, point), "{} wouldn't cast", s.def().name);
        }
        b.tick();
        while matches!(b.fighters[0].act, Act::Cast { .. }) {
            b.tick();
        }
        if check(b) {
            return;
        }
        while !matches!(b.fighters[0].act, Act::Idle) {
            b.tick();
        }
    }
    panic!("{} never took effect", s.def().name);
}

fn hp(b: &Battle, i: usize) -> f32 {
    b.fighters[i].hp.iter().sum()
}

// ---- Elemental ----------------------------------------------------------------

#[test]
fn spark_and_chill_sting_and_chill_slows() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    let full = hp(&b, 1);
    cast_until(&mut b, spell("spark"), Some(1), V2::new(8.0, 0.0), |b| hp(b, 1) < full);
    assert!(full - hp(&b, 1) < 15.0, "a spark is small");
    let speed = b.fighters[1].speed();
    cast_until(&mut b, spell("chill"), Some(1), V2::new(8.0, 0.0), |b| b.fighters[1].has(Does::Slow).is_some());
    assert!(b.fighters[1].speed() < speed * 0.8);
}

#[test]
fn stone_spikes_hit_only_the_legs() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    let before = b.fighters[1].hp;
    cast_until(&mut b, spell("stone_spikes"), None, V2::new(8.0, 0.0), |b| hp(b, 1) < before.iter().sum::<f32>());
    let f = &b.fighters[1];
    for p in [Part::Head, Part::Torso, Part::LeftArm, Part::RightArm] {
        assert_eq!(f.hp[p as usize], before[p as usize], "{p:?} shouldn't be touched");
    }
    assert!(f.hp[Part::LeftLeg as usize] < before[Part::LeftLeg as usize]);
}

#[test]
fn a_released_firestorm_burns_everyone_in_it() {
    let mut b = fight(&[(brute(2), 1, V2::new(10.0, 0.0)), (brute(3), 1, V2::new(12.0, 2.0)), (brute(4), 0, V2::new(13.0, -1.0)), (brute(5), 1, V2::new(30.0, 0.0))]);
    let before: Vec<f32> = (0..5).map(|i| hp(&b, i)).collect();
    cast_until(&mut b, spell("firestorm"), None, V2::new(11.0, 0.0), |b| hp(b, 1) < before[1]);
    assert!(hp(&b, 2) < before[2] && hp(&b, 3) < before[3], "friend and foe alike");
    assert_eq!(hp(&b, 4), before[4], "but not anyone outside it");
    assert!(before[1] - hp(&b, 1) > 15.0, "a firestorm is big");
}

#[test]
fn tremor_throws_enemies_down_and_spares_friends() {
    let mut b = fight(&[(brute(2), 1, V2::new(4.0, 0.0)), (brute(3), 0, V2::new(-3.0, 0.0))]);
    cast_until(&mut b, spell("tremor"), None, V2::new(0.0, 0.0), |b| b.fighters[1].helpless());
    assert!(!b.fighters[2].helpless(), "friends keep their feet");
    assert_eq!(b.fighters[1].speed(), 0.0);
}

#[test]
fn rituals_cant_be_cast_only_released() {
    let mut b = fight(&[(brute(2), 1, V2::new(8.0, 0.0))]);
    assert!(!b.begin_cast(0, spell("firestorm"), None, V2::new(8.0, 0.0)));
    assert!(!b.release(0, None, V2::new(8.0, 0.0)), "nothing held");
}

/// A world with a bandit camp, the squad 20 m from it.
fn by_a_camp() -> World {
    let mut w = worldgen::generate(7);
    let camp = w.camps[0].pos;
    w.teleport_squad(camp.add(V2::new(9.0, 0.0)));
    w
}

fn squad_mage(w: &World) -> u32 {
    *w.squad.members.iter().find(|&&m| w.people[m as usize].stats.calling == Calling::Mage).unwrap()
}

fn knows_all(w: &mut World, m: u32) {
    let p = &mut w.people[m as usize];
    p.detail.as_mut().unwrap().spells = gahturiyu_sim::sim::magic::all_spells().collect();
    for k in [Skill::Felt, Skill::Structured, Skill::Ritual] {
        p.stats.set_skill(k, 90.0);
    }
    p.stats.set_attr(Attr::Willpower, 90.0);
    p.mana = 1000.0;
}

/// Cast out of a fight until it works (a felt spell almost always does).
fn cast_world(w: &mut World, who: u32, s: Spell, target: Option<u32>, point: Option<V2>, check: impl Fn(&World) -> bool) {
    for _ in 0..20 {
        w.people[who as usize].mana = 1000.0;
        w.people[who as usize].mana_at = w.time;
        w.cast(who, s, target, point).unwrap_or_else(|e| panic!("{}: {}", s.def().name, e.0));
        if check(w) {
            return;
        }
    }
    panic!("{} never took effect", s.def().name);
}

#[test]
fn douse_puts_out_a_campfire_and_kindle_lights_it_again() {
    let mut w = by_a_camp();
    let m = squad_mage(&w);
    knows_all(&mut w, m);
    let camp = w.camps[0].pos;
    let lit = |w: &World| w.fixed_lights().iter().any(|l| l.pos.dist(camp) < 0.1);
    assert!(lit(&w));
    cast_world(&mut w, m, spell("douse"), None, Some(camp), |w| !lit(w));
    // It stays out for hours by the clock, not by anyone watching.
    w.teleport_squad(camp.add(V2::new(2000.0, 0.0)));
    w.step(3600.0);
    assert!(!lit(&w));
    w.teleport_squad(camp.add(V2::new(9.0, 0.0)));
    cast_world(&mut w, m, spell("kindle"), None, Some(camp), lit);
    // And a squad member's torch.
    let other = w.squad.members[0];
    let at = w.person_pos(other);
    cast_world(&mut w, m, spell("kindle"), None, Some(at), |w| w.torch_lit(other));
    cast_world(&mut w, m, spell("douse"), None, Some(at), |w| !w.torch_lit(other));
    let _ = items::id("torch");
}
