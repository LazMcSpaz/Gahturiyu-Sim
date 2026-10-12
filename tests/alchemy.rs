//! The alchemist's effects (Part 4 of `docs/items-design.md`): every
//! ingredient effect does something when drunk, each new kind has its rule
//! in a fight and out of one, and the toxicity meter rises with each draught,
//! drains on the clock, collapses someone who drinks too much, and never
//! kills.

use gahturiyu_sim::sim::{
    body::Part,
    combat::{self, Act, Battle, Fighter, Side, Zone, NOBODY},
    condition::{QUEASY, SICK, TOX_DRAIN, TOX_WAKE},
    effects::{lasting, now, Does, Effect, Lasts, Reach},
    geo::V2,
    items::{self, herbs::{Alch, INGREDIENTS}, WeaponDef},
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    toxic::{self, DRAUGHT_TOX},
    world::HOUR,
    worldgen, World,
};

const C: Reach = Reach::Caster;

fn person(id: u32) -> Person {
    let mut p = Person::summary(id, 1000 + id as u64 * 17, Race::Qotiro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blunt, 60.0), (Skill::Blade, 60.0)], 400.0);
    p.ensure_detail();
    p
}

/// Two fighters a step apart; nobody thinks for themselves.
fn pair(seed: u64) -> Battle {
    let mut fa = Fighter::from_person(&person(1), 0, V2::new(0.0, 0.0), 0.0);
    let mut fb = Fighter::from_person(&person(2), 1, V2::new(1.2, 0.0), 0.0);
    fa.think_at = f64::INFINITY;
    fb.think_at = f64::INFINITY;
    Battle::new(0, seed, 0.0, vec![fa, fb], vec!["A".into(), "B".into()])
}

fn weapon(key: &str) -> WeaponDef {
    *items::item(items::id(key)).weapon().unwrap()
}

/// A swings once at B with `w`; B's health is put back first.
fn one_blow(b: &mut Battle, w: WeaponDef) {
    b.fighters[0].weapon = w;
    b.fighters[1].hp = b.fighters[1].max_hp;
    b.fighters[1].ko = false;
    b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
    for _ in 0..3 {
        b.tick();
        if !matches!(b.fighters[0].act, Act::Swing { .. }) {
            break;
        }
    }
    b.fighters[0].act = Act::Idle;
}

fn ticks(b: &mut Battle, secs: f64) {
    for _ in 0..(secs / combat::DT).round() as usize {
        b.tick();
    }
}

fn total_hp(b: &Battle, j: usize) -> f32 {
    b.fighters[j].hp.iter().sum()
}

// ---- The data ------------------------------------------------------------------

#[test]
fn every_ingredient_effect_does_something_when_drunk() {
    let mut seen = Vec::new();
    for ing in INGREDIENTS {
        for &a in &ing.effects {
            if !seen.contains(&a) {
                seen.push(a);
            }
        }
    }
    assert_eq!(seen.len(), 27, "all 27 effects are carried by some ingredient");
    for a in seen {
        let dose = a.dose();
        if matches!(a, Alch::Burn | Alch::Smoke) {
            assert!(dose.is_empty(), "{} is for flasks", a.name());
            continue;
        }
        assert!(!dose.is_empty(), "{} does something", a.name());
        for e in dose {
            assert!(!e.describe().is_empty());
            assert_eq!(e.reach, Reach::Caster, "{} reaches whoever drinks it", a.name());
            // A harmful ingredient effect is harmful (or a lowered attribute).
            if a.harmful() {
                assert!(e.does.harmful() || matches!(e.does, Does::Daze) || e.power < 0.0, "{} hurts", a.name());
            }
        }
    }
}

// ---- Toxicity out of a fight ---------------------------------------------------

fn tox(w: &World, m: u32) -> f32 {
    w.toxicity_of(m).unwrap()
}

fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

fn give(w: &mut World, m: u32, key: &str, n: u16) -> items::ItemId {
    let id = items::id(key);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(id, n);
    id
}

#[test]
fn each_draught_adds_toxicity_and_it_drains_on_the_clock() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let d = give(&mut w, m, "healing_draught", 3);
    let start = tox(&w, m);
    for _ in 0..3 {
        assert!(w.use_item(m, d));
    }
    let after = tox(&w, m);
    assert!((after - (start + 3.0 * DRAUGHT_TOX)).abs() < 0.05, "three draughts: {start} to {after}");
    assert!(after >= QUEASY, "queasy after three");
    wait(&mut w, 2.0, 20.0);
    let later = tox(&w, m);
    assert!((later - (after - 2.0 * TOX_DRAIN)).abs() < 0.1, "drains {TOX_DRAIN} an hour: {after} to {later}");
}

#[test]
fn toxicity_comes_out_the_same_however_the_world_is_stepped() {
    let mut w0 = worldgen::generate(2);
    let m = w0.squad.members[0];
    let d = give(&mut w0, m, "healing_draught", 4);
    for _ in 0..4 {
        w0.use_item(m, d);
    }
    let run = |step: f64| {
        let mut w = w0.clone();
        wait(&mut w, 5.0, step);
        tox(&w, m)
    };
    let (a, b) = (run(10.0), run(300.0));
    assert!((a - b).abs() < 0.05, "{a} vs {b}");
}

#[test]
fn too_much_and_they_collapse_then_come_round_alive() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let d = give(&mut w, m, "healing_draught", 9);
    for _ in 0..8 {
        w.use_item(m, d);
    }
    assert!(tox(&w, m) >= 90.0, "past the limit ({})", tox(&w, m));
    wait(&mut w, 1.0, 20.0);
    assert!(w.is_down(m), "collapsed");
    // Out until it's drained to TOX_WAKE, then they come round.
    let hours = ((tox(&w, m) - TOX_WAKE) / TOX_DRAIN) as f64 + 2.0;
    wait(&mut w, hours, 60.0);
    assert!(!w.people[m as usize].dead, "it never kills");
    assert!(!w.is_down(m), "came round (toxicity {})", tox(&w, m));
}

#[test]
fn sick_a_draught_may_come_straight_back_up() {
    let up = |tox: f32| (0..400u64).filter(|&k| toxic::throws_up(7, 3, 11, k, tox)).count();
    assert_eq!(up(SICK - 1.0), 0, "never before they're sick");
    let n = up(SICK + 5.0);
    assert!((100..180).contains(&n), "about a third of the time ({n} of 400)");
    assert_eq!(up(SICK + 5.0), n, "the same roll every time");
}

#[test]
fn cleanse_takes_toxicity_away() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let d = give(&mut w, m, "healing_draught", 3);
    for _ in 0..3 {
        w.use_item(m, d);
    }
    let before = tox(&w, m);
    assert!(w.apply_effect(m, &now(Does::Cleanse, 25.0, C), 1.0));
    assert!((tox(&w, m) - (before - 25.0)).abs() < 0.05);
}

#[test]
fn a_side_effect_reaches_whoever_drinks_it_out_of_a_fight() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let hp = |w: &World| w.people[m as usize].wounds.hp_at(&w.people[m as usize].stats, w.time).iter().sum::<f32>();
    let before = hp(&w);
    w.take_dose(m, &lasting(Does::Poison, 0.5, 30.0, C));
    assert!(hp(&w) < before - 10.0, "poison ran its course at once");
    w.take_dose(m, &lasting(Does::ThinBlood, 0.5, 300.0, C));
    assert!(w.boon(m, Does::ThinBlood) > 0.0, "the rest stay on them");
    let t0 = tox(&w, m);
    w.take_dose(m, &lasting(Does::Nausea, 0.5, 60.0, C));
    assert!(tox(&w, m) > t0, "nausea adds toxicity");
}

#[test]
fn rouse_brings_the_knocked_out_round_out_of_a_fight() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let t = w.time;
    let p = &mut w.people[m as usize];
    let base = p.stats.clone();
    let mut hp = p.wounds.hp_at(&base, t);
    hp[Part::Head as usize] = -5.0;
    p.wounds.set(&base, &hp, t);
    assert!(w.is_down(m));
    assert!(w.apply_effect(m, &now(Does::Rouse, 1.0, C), 1.0));
    assert!(!w.is_down(m), "brought round");
}

// ---- In a fight -----------------------------------------------------------------

#[test]
fn poison_eats_at_the_body_but_never_the_downed() {
    let mut b = pair(1);
    let before = total_hp(&b, 1);
    b.dose(1, &lasting(Does::Poison, 1.0, 30.0, C));
    ticks(&mut b, 5.0);
    assert!(total_hp(&b, 1) < before - 3.0, "poisoned");
    b.fighters[1].ko = true;
    let down = total_hp(&b, 1);
    ticks(&mut b, 5.0);
    assert_eq!(total_hp(&b, 1), down, "not once they're down");
}

#[test]
fn slumber_puts_them_out_of_it_until_a_blow_wakes_them() {
    let mut b = pair(1);
    b.dose(1, &lasting(Does::Slumber, 1.0, 20.0, C));
    assert!(b.fighters[1].helpless(), "asleep");
    one_blow(&mut b, weapon("knife"));
    assert!(!b.fighters[1].helpless(), "a blow wakes them");
}

#[test]
fn rouse_gets_a_downed_fighter_back_up() {
    let mut b = pair(1);
    let head = b.fighters[1].hp[Part::Head as usize];
    b.wound(1, Part::Head, head + 3.0);
    assert!(b.fighters[1].ko);
    b.dose(1, &now(Does::Rouse, 1.0, C));
    assert!(!b.fighters[1].ko && b.fighters[1].active(), "back on their feet");
}

#[test]
fn thin_blood_bleeds_worse() {
    let mut b = pair(1);
    b.open_bleed(1, 0.2);
    let plain = b.fighters[1].bleeding();
    let mut b = pair(1);
    b.dose(1, &lasting(Does::ThinBlood, 0.5, 300.0, C));
    b.open_bleed(1, 0.2);
    assert!((b.fighters[1].bleeding() - plain * 1.5).abs() < 1e-4);
}

#[test]
fn steady_keeps_them_on_their_feet() {
    let floored = |steady: bool| {
        let mut b = pair(9);
        let mut n = 0;
        for _ in 0..80 {
            b.fighters[1].statuses.retain(|s| s.does != Does::KnockDown);
            if steady {
                b.dose(1, &lasting(Does::Steady, 0.8, 300.0, C));
            }
            one_blow(&mut b, weapon("stone_maul"));
            n += b.fighters[1].has(Does::KnockDown).is_some() as u32;
        }
        n
    };
    let (plain, steady) = (floored(false), floored(true));
    assert!(plain >= 5 && steady * 2 < plain, "floored {plain} plain, {steady} steady");
}

#[test]
fn warmth_keeps_the_cold_out_and_resist_fire_the_heat() {
    let chill = |warm: bool| {
        let mut b = pair(1);
        if warm {
            b.dose(1, &lasting(Does::Warmth, 0.5, 300.0, C));
        }
        b.dose(1, &Effect { does: Does::Damage(gahturiyu_sim::sim::effects::Element::Frost), power: 4.0, lasts: Lasts::Now, reach: C });
        b.fighters[1].power(Does::Chilled)
    };
    assert!(chill(true) < chill(false) * 0.6, "warmth: {} vs {}", chill(true), chill(false));
    let burnt = |proof: bool| {
        let mut b = pair(1);
        if proof {
            b.dose(1, &lasting(Does::ResistFire, 0.5, 300.0, C));
        }
        let before = total_hp(&b, 1);
        b.dose(1, &Effect { does: Does::Damage(gahturiyu_sim::sim::effects::Element::Fire), power: 10.0, lasts: Lasts::Now, reach: C });
        ticks(&mut b, 3.0);
        before - total_hp(&b, 1)
    };
    assert!(burnt(true) < burnt(false) * 0.7, "resist fire: {} vs {}", burnt(true), burnt(false));
}

#[test]
fn smoke_hides_beyond_arms_reach_and_keen_senses_see_further() {
    let mut b = pair(1);
    b.fighters[1].pos = V2::new(6.0, 0.0);
    assert!(b.can_make_out(0, 1));
    let side: Side = 1;
    b.zones.push(Zone { does: Does::Smoke, pos: V2::new(6.0, 0.0), radius: 3.0, power: 1.0, until: 60.0, side, owner: NOBODY, fresh: true });
    assert!(!b.can_make_out(0, 1), "lost in the smoke");
    b.fighters[1].pos = V2::new(3.5, 0.0);
    assert!(b.can_make_out(0, 1), "close enough to make out");
    b.fighters[1].pos = V2::new(6.0, 0.0);
    b.dose(0, &lasting(Does::KeenSenses, 1.0, 300.0, C));
    assert!(b.can_make_out(0, 1), "keen senses see further in");
}

#[test]
fn drinking_in_a_fight_adds_toxicity_and_too_much_drops_them() {
    let mut b = pair(1);
    let d = items::id("healing_draught");
    b.fighters[1].potions = vec![d];
    b.fighters[1].toxic = 85.0;
    assert!(b.begin_drink(1, d));
    ticks(&mut b, combat::DRINK_SECS + 0.3);
    assert!(b.fighters[1].toxic >= 90.0);
    assert!(b.fighters[1].ko && !b.fighters[1].dead, "collapsed, not killed");
}
