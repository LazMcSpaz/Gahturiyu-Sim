//! Fights: they end, they're deterministic, they leave marks, and the
//! numbers behave the way the rules say.

use gahturiyu_sim::sim::{
    body::{Part, HEAL_PER_HOUR},
    combat::{Act, Battle, Fighter},
    geo::V2,
    items,
    magic::{Spell, StatusKind},
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen, World,
};

fn fight_world(seed: u64, bandits: usize, step: f64) -> World {
    let mut w = worldgen::generate(seed);
    let at = w.squad.pos.add(V2::new(18.0, 4.0));
    w.spawn_bandits(at, bandits, true);
    // The first short step is shared, so the fight starts at the same moment
    // in every run; what follows may be stepped however you like. (Exactly
    // when a fight is *noticed* depends on how often the world looks — like
    // the squad's own walking, which is player-driven — but once it starts,
    // how it plays out must not.)
    w.step(0.1);
    let mut n = 0;
    while n < 40_000 {
        w.step(step);
        n += 1;
        if w.battles.is_empty() && w.next_battle > 0 {
            break;
        }
    }
    w
}

#[test]
fn a_fight_starts_and_finishes() {
    let w = fight_world(1, 3, 0.25);
    assert_eq!(w.next_battle, 1, "exactly one fight should have broken out");
    assert!(w.battles.is_empty(), "the fight should be over");
    let hurt = w.people.iter().filter(|p| p.wounds.is_hurt(w.time)).count();
    assert!(hurt >= 2, "a fight should leave people hurt");
}

#[test]
fn fights_come_out_the_same_however_the_world_is_stepped() {
    let a = fight_world(4, 3, 0.1);
    let b = fight_world(4, 3, 2.3);
    // Compare everyone's state at a common later time.
    let t = a.time.max(b.time) + 10.0;
    for (pa, pb) in a.people.iter().zip(&b.people) {
        assert_eq!(pa.dead, pb.dead, "{} died in one run only", pa.id);
        let (ha, hb) = (pa.wounds.hp_at(&pa.stats, t), pb.wounds.hp_at(&pb.stats, t));
        for k in 0..6 {
            assert!((ha[k] - hb[k]).abs() < 0.01, "person {} part {k}: {} vs {}", pa.id, ha[k], hb[k]);
        }
        assert!((pa.mana_at(t) - pb.mana_at(t)).abs() < 0.01);
    }
}

#[test]
fn wounds_heal_with_time() {
    let w = fight_world(1, 3, 0.25);
    // Someone outside the squad: they heal at the plain constant rate (the
    // squad's rate follows rest and food; see tests/condition.rs).
    let p = w.people.iter().find(|p| !p.dead && !p.in_squad && p.wounds.is_hurt(w.time)).expect("someone hurt");
    let lost = p.wounds.lost_at(w.time).iter().cloned().fold(0.0, f32::max);
    let later = p.wounds.lost_at(w.time + 3600.0).iter().cloned().fold(0.0, f32::max);
    assert!((lost - later - HEAL_PER_HOUR).abs() < 0.01 || later == 0.0, "an hour should heal {HEAL_PER_HOUR} per part");
}

#[test]
fn fighting_trains_skills() {
    let before = worldgen::generate(1);
    let after = fight_world(1, 3, 0.25);
    let gained = after.squad.members.iter().any(|&m| {
        let (a, b) = (&before.people[m as usize].stats, &after.people[m as usize].stats);
        [Skill::Blunt, Skill::Spear, Skill::Dodge, Skill::Block, Skill::Destruction, Skill::Illusion, Skill::Alteration].iter().any(|&s| b.skill(s) > a.skill(s) + 0.05)
    });
    assert!(gained, "nobody learned anything from the fight");
}

// ---- One-on-one checks of the rules ---------------------------------------

fn person(id: u32, race: Race, calling: Calling, skills: &[(Skill, f32)], budget: f32) -> Person {
    let mut p = Person::summary(id, 1000 + id as u64 * 17, race, None);
    p.specialize(calling, skills, budget);
    p.ensure_detail();
    p
}

/// A battle where nobody thinks for themselves unless told to.
fn duel(a: Person, b: Person, gap: f32) -> Battle {
    let mut fa = Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(&b, 1, V2::new(gap, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, 77, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

fn total_hp(f: &Fighter) -> f32 {
    f.hp.iter().sum()
}

fn mage() -> Person {
    let mut p = person(1, Race::Tadoro, Calling::Mage, &[(Skill::Destruction, 100.0), (Skill::Alteration, 100.0), (Skill::Illusion, 100.0), (Skill::Restoration, 100.0)], 60.0);
    p.stats.set_attr(gahturiyu_sim::sim::stats::Attr::Willpower, 100.0);
    p.mana = 500.0;
    p
}

fn brute(id: u32) -> Person {
    person(id, Race::Qotiro, Calling::Warrior, &[(Skill::Blade, 50.0)], 400.0)
}

/// Cast until it works (casts can fail even for experts).
fn cast_until(b: &mut Battle, spell: Spell, target: Option<usize>, point: V2, check: impl Fn(&Battle) -> bool) -> f32 {
    let start_mana = b.fighters[0].mana;
    for _ in 0..20 {
        b.fighters[0].mana = b.fighters[0].mana.max(spell.def().cost);
        let before = b.fighters[0].mana;
        assert!(b.begin_cast(0, spell, target, point));
        assert!((before - b.fighters[0].mana - spell.def().cost).abs() < 1e-3, "casting must spend the mana up front");
        while matches!(b.fighters[0].act, Act::Cast { .. }) {
            b.tick();
        }
        if check(b) {
            return start_mana;
        }
    }
    panic!("{:?} never took effect", spell);
}

#[test]
fn paralysis_stops_movement_and_attacks() {
    let mut b = duel(mage(), brute(2), 6.0);
    b.fighters[1].resist_paralysis = 0.0;
    b.fighters[1].stats.set_attr(gahturiyu_sim::sim::stats::Attr::Willpower, 1.0);
    cast_until(&mut b, Spell::Paralyze, Some(1), V2::new(6.0, 0.0), |b| b.fighters[1].paralyzed());
    let pos = b.fighters[1].pos;
    b.fighters[1].target = Some(0);
    for _ in 0..20 {
        b.tick();
    }
    assert_eq!(b.fighters[1].pos, pos, "a paralyzed fighter mustn't move");
    assert!(matches!(b.fighters[1].act, Act::Idle));
    // ...and it wears off.
    for _ in 0..80 {
        b.tick();
    }
    assert!(!b.fighters[1].paralyzed());
}

#[test]
fn fireball_hurts_everyone_in_the_blast_including_friends() {
    let mut b = duel(mage(), brute(2), 12.0);
    let mut friend = Fighter::from_person(&brute(3), 0, V2::new(12.5, 0.0), 0.0);
    friend.think_at = f64::INFINITY;
    b.fighters.push(friend);
    b.names.push("Friend".into());
    let far = Fighter::from_person(&brute(4), 1, V2::new(30.0, 0.0), 0.0);
    b.fighters.push(far);
    b.names.push("Far".into());
    b.fighters[3].think_at = f64::INFINITY;
    let before: Vec<f32> = b.fighters.iter().map(total_hp).collect();
    cast_until(&mut b, Spell::Fireball, Some(1), V2::new(12.0, 0.0), |b| total_hp(&b.fighters[1]) < before[1]);
    assert!(total_hp(&b.fighters[2]) < before[2], "the caster's friend in the blast should burn too");
    assert_eq!(total_hp(&b.fighters[3]), before[3], "someone outside the blast should be untouched");
}

#[test]
fn lightning_ignores_armour() {
    let hit = |armoured: bool| {
        let mut target = brute(2);
        let d = target.detail.as_mut().unwrap();
        if !armoured {
            for s in [items::Slot::Body, items::Slot::Head, items::Slot::Legs] {
                d.gear.unequip(s);
            }
        }
        let mut b = duel(mage(), target, 8.0);
        let before = total_hp(&b.fighters[1]);
        cast_until(&mut b, Spell::LightningBolt, Some(1), V2::new(8.0, 0.0), |b| total_hp(&b.fighters[1]) < before);
        before - total_hp(&b.fighters[1])
    };
    let (with, without) = (hit(true), hit(false));
    assert!((with - without).abs() < 0.01, "armour changed lightning damage: {with} vs {without}");
}

#[test]
fn blindness_makes_attacks_miss() {
    let hits = |blind: bool| {
        let mut b = duel(brute(2), brute(3), 1.0);
        b.fighters[1].hp = [1e6; 6];
        if blind {
            b.fighters[0].statuses.push(gahturiyu_sim::sim::magic::Status { kind: StatusKind::Blinded, until: 1e9, magnitude: 0.65 });
        }
        b.fighters[0].target = Some(1);
        b.fighters[0].fatigue = 1e6;
        b.fighters[0].max_fatigue = 1e6;
        for _ in 0..3000 {
            b.tick();
        }
        b.fighters[1].damage_taken
    };
    let (seeing, blinded) = (hits(false), hits(true));
    assert!(blinded < seeing * 0.6, "blind: {blinded} vs sighted: {seeing}");
}

#[test]
fn mage_armor_takes_the_edge_off() {
    let taken = |warded: bool| {
        let mut b = duel(brute(2), mage(), 1.0);
        b.fighters[1].hp = [1e6; 6];
        if warded {
            cast_inline(&mut b, 1, Spell::MageArmor);
            assert!(b.fighters[1].has(StatusKind::MageArmor).is_some());
        }
        b.fighters[0].target = Some(1);
        b.fighters[0].fatigue = 1e6;
        b.fighters[0].max_fatigue = 1e6;
        for _ in 0..250 {
            b.tick();
        }
        b.fighters[1].damage_taken
    };
    let (bare, warded) = (taken(false), taken(true));
    assert!(warded < bare * 0.8, "warded {warded} vs bare {bare}");
}

/// Have fighter `i` cast a self spell until it works.
fn cast_inline(b: &mut Battle, i: usize, spell: Spell) {
    for _ in 0..20 {
        b.fighters[i].mana = 100.0;
        let mana = b.fighters[i].mana;
        assert!(b.begin_cast(i, spell, None, b.fighters[i].pos));
        assert!(b.fighters[i].mana < mana);
        while matches!(b.fighters[i].act, Act::Cast { .. }) {
            b.tick();
        }
        b.fighters[i].act = Act::Idle;
        if !b.fighters[i].statuses.is_empty() {
            return;
        }
    }
    panic!("{spell:?} never took");
}

#[test]
fn haste_makes_you_faster() {
    let mut b = duel(mage(), brute(2), 40.0);
    let slow = b.fighters[0].speed();
    let swing = b.fighters[0].attack_time();
    cast_inline(&mut b, 0, Spell::Haste);
    assert!(b.fighters[0].speed() > slow * 1.3);
    assert!(b.fighters[0].attack_time() < swing);
}

#[test]
fn no_mana_no_spell() {
    let mut b = duel(mage(), brute(2), 8.0);
    b.fighters[0].mana = 5.0;
    assert!(!b.begin_cast(0, Spell::Fireball, Some(1), V2::new(8.0, 0.0)));
    assert!((b.fighters[0].mana - 5.0).abs() < 1e-6);
}

#[test]
fn ruined_legs_slow_you_down_and_a_ruined_arm_drops_the_weapon() {
    let mut b = duel(brute(2), brute(3), 5.0);
    let fast = b.fighters[0].speed();
    b.fighters[0].hp[Part::LeftLeg as usize] = -1.0;
    assert!(b.fighters[0].speed() < fast * 0.7);
    b.fighters[0].hp[Part::RightLeg as usize] = -1.0;
    assert!(b.fighters[0].speed() < fast * 0.3);
    b.fighters[0].hp[Part::RightArm as usize] = -1.0;
    let (w, penalty) = b.fighters[0].usable_weapon().unwrap();
    assert_eq!(w.skill, Skill::Unarmed);
    assert!(penalty < 1.0);
}

#[test]
fn better_armed_fighters_usually_win() {
    let mut strong_wins = 0;
    for k in 0..40u32 {
        let strong = person(10 + k, Race::Qotiro, Calling::Warrior, &[(Skill::Blade, 55.0), (Skill::Block, 40.0)], 600.0);
        let weak = person(100 + k, Race::Horaro, Calling::Common, &[], 15.0);
        let mut fa = Fighter::from_person(&strong, 0, V2::new(0.0, 0.0), 0.0);
        let mut fb = Fighter::from_person(&weak, 1, V2::new(3.0, 0.0), 0.0);
        fa.boldness = 1.0;
        fb.boldness = 1.0;
        let mut b = Battle::new(k, k as u64, 0.0, vec![fa, fb], vec!["S".into(), "W".into()]);
        while !b.over {
            b.tick();
        }
        if b.winner() == Some(0) {
            strong_wins += 1;
        }
    }
    assert!(strong_wins >= 34, "the well-armed warrior only won {strong_wins} of 40");
}

#[test]
fn healing_mends_wounds_and_gets_the_downed_up() {
    let mut caster = mage();
    caster.stats.set_skill(Skill::Restoration, 100.0);
    caster.recompute_might();
    let mut b = duel(caster, brute(2), 3.0);
    // Make the second fighter a downed friend.
    b.fighters[1].side = 0;
    b.fighters[1].hp[Part::Torso as usize] = -5.0;
    b.fighters[1].ko = true;
    let before = total_hp(&b.fighters[1]);
    cast_until(&mut b, Spell::Heal, Some(1), V2::new(3.0, 0.0), |b| total_hp(&b.fighters[1]) > before);
    assert!(!b.fighters[1].ko, "a healed friend should get back up");
    assert!(b.fighters[1].hp[Part::Torso as usize] > 0.0);
}
