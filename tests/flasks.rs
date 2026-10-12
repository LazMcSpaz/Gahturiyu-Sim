//! Thrown flasks and blade poisons (the items design, Part 4 rules 8 and 9).

use gahturiyu_sim::sim::{
    combat::{Act, Battle, Fighter, FxKind, FLASK_RADIUS},
    effects::Does,
    geo::V2,
    items::{
        self,
        herbs::{Alch, Kit, Mix},
    },
    materials::Piece,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen,
};

fn person(id: u32) -> Person {
    let mut p = Person::summary(id, 100 + id as u64, Race::Roduro, None);
    p.specialize(Calling::Common, &[], 0.0);
    p.stats.set_skill(Skill::Dodge, 0.0);
    p.ensure_detail();
    p
}

fn brew(a: &[Alch]) -> Mix {
    Mix::brew(a, 60.0, Kit::default())
}

/// A thrower at the origin and three foes bunched round (10, 0); none of
/// them thinks unless told to.
fn knot(seed: u64) -> Battle {
    let mut fs = vec![Fighter::from_person(&person(1), 0, V2::new(0.0, 0.0), 0.0)];
    for (k, at) in [V2::new(10.0, 0.0), V2::new(10.8, 0.6), V2::new(10.5, -0.8)].into_iter().enumerate() {
        fs.push(Fighter::from_person(&person(2 + k as u32), 1, at, 0.0));
    }
    let names = (0..fs.len()).map(|i| format!("F{i}")).collect();
    let mut b = Battle::new(0, seed, 0.0, fs, names);
    for f in &mut b.fighters {
        f.think_at = f64::INFINITY;
        f.aware_at = 0.0;
    }
    b
}

fn tick(b: &mut Battle, n: usize) {
    for _ in 0..n {
        b.tick();
    }
}

#[test]
fn a_flask_bursts_over_everyone_in_the_splash() {
    let mut caught = 0;
    for seed in 0..6 {
        let mut b = knot(seed);
        let mix = brew(&[Alch::Slumber]);
        assert!(mix.harms());
        b.fighters[0].bottles = vec![mix];
        b.fighters[0].flasks = 1;
        assert!(b.begin_flask(0, mix, V2::new(10.4, 0.0)));
        assert!(matches!(b.fighters[0].act, Act::Flask { .. }));
        assert!(b.fighters[0].bottles.is_empty() && b.fighters[0].flasks == 0, "the bottle and a flask used up");
        tick(&mut b, 15);
        assert!(b.fx.iter().any(|f| matches!(f.kind, FxKind::Splash { .. })), "it burst");
        caught += (1..4).filter(|&j| b.fighters[j].has(Does::Slumber).is_some()).count();
        // (Written back like a bottle drunk, and the flask with it.)
        assert_eq!(b.fighters[0].drunk, vec![mix]);
        assert!(b.fighters[0].used.contains(&items::id("sandglass_flask")));
    }
    assert!(caught >= 10, "most of the knot put to sleep ({caught} of 18)");
}

#[test]
fn no_flask_without_one_to_throw_it_in_or_out_of_range() {
    let mut b = knot(1);
    let mix = brew(&[Alch::Poison]);
    b.fighters[0].bottles = vec![mix];
    b.fighters[0].flasks = 0;
    assert!(!b.begin_flask(0, mix, V2::new(10.0, 0.0)), "no flask");
    b.fighters[0].flasks = 1;
    assert!(!b.begin_flask(0, mix, V2::new(30.0, 0.0)), "too far");
    let healing = brew(&[Alch::Heal]);
    b.fighters[0].bottles.push(healing);
    assert!(!b.begin_flask(0, healing, V2::new(10.0, 0.0)), "nothing harmful in it");
}

#[test]
fn a_smoke_flask_leaves_a_cloud() {
    let mut b = knot(2);
    let mix = brew(&[Alch::Smoke]);
    b.fighters[0].bottles = vec![mix];
    b.fighters[0].flasks = 1;
    assert!(b.begin_flask(0, mix, V2::new(10.4, 0.0)));
    tick(&mut b, 15);
    assert!(b.in_smoke(V2::new(10.4, 0.0)), "a cloud where it burst");
}

#[test]
fn the_ai_throws_at_a_knot_of_foes_but_not_over_a_friend() {
    let throws = |friend: bool| {
        let mut b = knot(3);
        b.fighters[0].bottles = vec![brew(&[Alch::Poison])];
        b.fighters[0].flasks = 1;
        b.fighters[0].think_at = 0.0;
        b.fighters[0].weapon = items::FISTS;
        if friend {
            let mut f = Fighter::from_person(&person(9), 0, V2::new(10.2, 1.0), 0.0);
            f.think_at = f64::INFINITY;
            b.fighters.push(f);
            b.names.push("Friend".into());
        }
        tick(&mut b, 30);
        b.fx.iter().any(|f| matches!(f.kind, FxKind::Splash { .. }))
    };
    assert!(throws(false), "three bunched foes: the flask goes");
    assert!(!throws(true), "a friend among them: it doesn't");
    let _ = FLASK_RADIUS;
}

#[test]
fn poison_on_a_blade_goes_in_with_blows_that_wound() {
    let mut poisoned = 0;
    for seed in 0..8 {
        let fs = vec![Fighter::from_person(&person(1), 0, V2::new(0.0, 0.0), 0.0), Fighter::from_person(&person(2), 1, V2::new(1.0, 0.0), 0.0)];
        let mut b = Battle::new(0, 40 + seed, 0.0, fs, vec!["A".into(), "B".into()]);
        b.fighters[1].think_at = f64::INFINITY;
        b.fighters[1].armor.clear();
        b.fighters[1].aware_at = 0.0;
        b.fighters[0].think_at = f64::INFINITY;
        b.fighters[0].weapon = *items::item(items::id("knife")).weapon().unwrap();
        b.fighters[0].coating = Some((brew(&[Alch::Poison]), 2));
        for _ in 0..12 {
            b.fighters[0].act = Act::Swing { target: 1, lands: b.time + 0.05 };
            tick(&mut b, 2);
            b.fighters[0].act = Act::Idle;
            if b.fighters[1].dead || b.fighters[1].ko {
                break;
            }
        }
        let left = b.fighters[0].coating.map(|c| c.1).unwrap_or(0);
        assert!(left <= 1, "used on the blows that wounded");
        if b.log.iter().any(|l| l.1.contains("poisoned blade bites")) {
            poisoned += 1;
        }
    }
    assert!(poisoned >= 6, "{poisoned} of 8");
}

#[test]
fn a_harmful_brew_coats_the_blade_and_lasts_some_blows() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.add_piece(items::id("brewed_potion"), Piece::bottle(brew(&[Alch::Poison])));
    d.gear.add_piece(items::id("brewed_potion"), Piece::bottle(brew(&[Alch::Heal])));
    let entry = |w: &gahturiyu_sim::sim::World, harmful: bool| w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().position(|e| e.2.and_then(|p| p.mix).is_some_and(|x| x.harms() == harmful)).unwrap();
    let heal = entry(&w, false);
    assert!(w.why_cant_coat(m, heal).is_some(), "a healing draught won't do");
    let k = entry(&w, true);
    if w.why_cant_coat(m, k) == Some("they need a hand weapon to coat") {
        w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("knife"), 1);
        w.people[m as usize].detail.as_mut().unwrap().gear.equip(items::id("knife")).unwrap();
    }
    let k = entry(&w, true);
    assert!(w.coat_weapon(m, k));
    let c = *w.coating_of(m).expect("coated");
    assert!(c.hits >= 3);
    assert!(w.people[m as usize].detail.as_ref().unwrap().gear.bottles().iter().all(|x| !x.harms()), "the bottle used up");
}
