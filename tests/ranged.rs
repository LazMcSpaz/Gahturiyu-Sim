//! Bows and crossbows: shooting from a distance, running out, closing in.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    geo::V2,
    items,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen,
};

fn archer(id: u32, sidearm: bool) -> Person {
    let mut p = Person::summary(id, 7000 + id as u64, Race::Horaro, None);
    p.specialize(Calling::Hunter, &[(Skill::Marksman, 60.0)], 200.0);
    p.ensure_detail();
    let g = &mut p.detail.as_mut().unwrap().gear;
    g.bag.clear();
    g.add(items::id("short_bow"), 1);
    g.equip(items::id("short_bow")).unwrap();
    g.add(items::id("arrows"), 10);
    if sidearm {
        g.add(items::id("short_sword"), 1);
    }
    p
}

fn target(id: u32) -> Person {
    let mut p = Person::summary(id, 8000 + id as u64, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blunt, 40.0)], 150.0);
    p.ensure_detail();
    p
}

fn battle(a: Fighter, b: Fighter) -> Battle {
    Battle::new(0, 11, 0.0, vec![a, b], vec!["Archer".into(), "Target".into()])
}

#[test]
fn archers_shoot_from_a_distance_and_use_up_arrows() {
    let mut a = Fighter::from_person(&archer(1, false), 0, V2::new(0.0, 0.0), 0.0);
    let mut d = Fighter::from_person(&target(2), 1, V2::new(22.0, 0.0), 0.0);
    d.think_at = f64::INFINITY; // stands there
    d.hp = [1e5; 6];
    a.target = Some(1);
    a.think_at = f64::INFINITY;
    assert!(a.shooting());
    let mut b = battle(a, d);
    while b.fighters[0].shots < 5 && b.ticks < 1000 {
        b.tick();
    }
    assert!(b.fighters[0].pos.dist(V2::default()) < 0.5, "an archer in range doesn't walk up");
    for _ in 0..300 {
        b.tick();
    }
    assert!(b.fighters[1].damage_taken > 0.0, "some arrows should land");
    assert_eq!(b.fighters[0].ammo + b.fighters[0].shots, 10);
    assert!(b.fighters[0].shots >= 5);
}

#[test]
fn out_of_arrows_means_out_of_range() {
    let mut a = Fighter::from_person(&archer(1, false), 0, V2::new(0.0, 0.0), 0.0);
    a.ammo = 0;
    assert!(!a.shooting());
    assert!(a.attack_range() < 3.0);
}

#[test]
fn an_archer_pressed_close_draws_a_hand_weapon_and_goes_back_to_the_bow() {
    let a = Fighter::from_person(&archer(1, true), 0, V2::new(0.0, 0.0), 0.0);
    assert!(a.sidearm.is_some());
    let mut d = Fighter::from_person(&target(2), 1, V2::new(2.0, 0.0), 0.0);
    d.hp = [1e5; 6];
    d.think_at = f64::INFINITY;
    let mut b = battle(a, d);
    for _ in 0..10 {
        b.tick();
    }
    assert_eq!(b.fighters[0].weapon.range, 0.0, "should have switched to the sword");
    // The enemy backs well off: out comes the bow again.
    b.fighters[1].pos = V2::new(20.0, 0.0);
    for _ in 0..20 {
        b.tick();
    }
    assert!(b.fighters[0].weapon.range > 0.0, "back to the bow");
}

#[test]
fn an_archer_without_a_hand_weapon_backs_off() {
    let a = Fighter::from_person(&archer(1, false), 0, V2::new(0.0, 0.0), 0.0);
    let mut d = Fighter::from_person(&target(2), 1, V2::new(2.5, 0.0), 0.0);
    d.think_at = f64::INFINITY;
    let mut b = battle(a, d);
    for _ in 0..10 {
        b.tick();
    }
    assert!(b.fighters[0].pos.x < -0.5, "should have stepped back: {:?}", b.fighters[0].pos);
}

#[test]
fn some_arrows_are_found_after_a_fight() {
    // Bandit archers against the squad, in the world.
    for seed in 1..6 {
        let mut w = worldgen::generate(seed);
        let at = w.squad.pos.add(V2::new(25.0, 0.0));
        let g = w.spawn_bandits(at, 3, false);
        let archer = w.group(g).unwrap().members[1];
        let arrows = |w: &gahturiyu_sim::sim::World| w.people[archer as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::item(e.0).key == "arrows").map(|e| e.1).sum::<u16>();
        let before = arrows(&w);
        assert!(before > 0, "the second bandit is an archer");
        let mut shots = 0;
        for _ in 0..4000 {
            w.step(0.5);
            if let Some(f) = w.fighter(archer) {
                shots = f.shots;
            }
            if w.battles.is_empty() && w.next_battle > 0 {
                break;
            }
        }
        if shots < 4 {
            continue;
        }
        let after = arrows(&w);
        assert!(after < before, "arrows were used");
        assert!(after > before - shots, "some were found again: {before} - {shots} shots -> {after}");
        return;
    }
    panic!("no archer got enough shots off");
}

#[test]
fn camps_have_archers() {
    let w = worldgen::generate(1);
    let archers = w
        .camps
        .iter()
        .filter_map(|c| w.group(c.group))
        .flat_map(|g| g.members.iter())
        .filter(|&&m| w.people[m as usize].kit().weapon().range > 0.0)
        .count();
    assert!(archers >= 3, "{archers} archers in camps");
}

#[test]
fn every_shot_leaves_an_arrow_to_draw() {
    use gahturiyu_sim::sim::combat::FxKind;
    let mut a = Fighter::from_person(&archer(1, false), 0, V2::new(0.0, 0.0), 0.0);
    let mut d = Fighter::from_person(&target(2), 1, V2::new(22.0, 0.0), 0.0);
    d.think_at = f64::INFINITY;
    d.hp = [1e5; 6];
    a.target = Some(1);
    a.think_at = f64::INFINITY;
    let mut b = battle(a, d);
    for _ in 0..1200 {
        b.tick();
    }
    let arrows: Vec<(V2, V2, bool)> = b.fx.iter().filter_map(|f| if let FxKind::Arrow { from, to, hit } = f.kind { Some((from, to, hit)) } else { None }).collect();
    assert_eq!(arrows.len() as u16, b.fighters[0].shots, "one arrow per shot");
    assert!(arrows.iter().any(|a| a.2) && arrows.iter().any(|a| !a.2), "some hit, some miss");
    for (from, to, hit) in arrows {
        assert!(from.dist(V2::default()) < 0.5);
        if hit {
            assert!(to.dist(V2::new(22.0, 0.0)) < 0.5, "hits land on the target");
        } else {
            assert!(to.x > 22.0, "misses come down past the target: {to:?}");
        }
    }
}

#[test]
fn shooting_in_the_dark_misses_more_and_a_torch_helps() {
    // The same shots at midnight in the open, at noon, and at midnight with
    // the target holding a torch.
    let hits = |start: f64, lit: Option<bool>| {
        let mut a = Fighter::from_person(&archer(1, false), 0, V2::new(0.0, 0.0), start);
        let mut d = Fighter::from_person(&target(2), 1, V2::new(22.0, 0.0), start);
        d.think_at = f64::INFINITY;
        d.hp = [1e5; 6];
        a.target = Some(1);
        a.think_at = f64::INFINITY;
        a.ammo = 400;
        d.torch = lit == Some(true);
        let mut b = Battle::new(0, 11, start, vec![a, d], vec!["Archer".into(), "Target".into()]);
        b.lights = lit.map(|_| Vec::new());
        for _ in 0..3000 {
            b.tick();
        }
        let shots = b.fighters[0].shots as f32;
        b.fx.iter().filter(|f| matches!(f.kind, gahturiyu_sim::sim::combat::FxKind::Arrow { hit: true, .. })).count() as f32 / shots
    };
    let midnight = 24.0 * 3600.0;
    let day = hits(midnight + 12.0 * 3600.0, Some(false));
    let dark = hits(midnight, Some(false));
    let torch = hits(midnight, Some(true));
    assert!(dark < day * 0.7, "dark {dark} vs day {day}");
    assert!(torch > dark * 1.3, "a torch on the target: {torch} vs {dark}");
}
