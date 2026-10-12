//! Potions break (Part 4 rule 10): a heavy blow, a fall or a burn can crack
//! a bottle the squad carries, unless it's in a potion case.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter, FxKind},
    effects::Does,
    geo::V2,
    inventory::Gear,
    items::{self, herbs::{Alch, Kit, Mix}},
    magic::spell,
    materials::Piece,
    person::Person,
    race::Race,
    soak::bottles_at_risk,
    stats::{Calling, Skill},
    worldgen,
};

fn draught() -> items::ItemId {
    items::id("healing_draught")
}

fn heal_mix() -> Mix {
    Mix::brew(&[Alch::Heal], 50.0, Kit::default())
}

/// A squad member with three draughts and a brewed bottle; `case`: a potion
/// case in the pack.
fn carrier(id: u32, case: bool) -> Person {
    let mut p = Person::summary(id, 900 + id as u64, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blade, 40.0)], 200.0);
    p.ensure_detail();
    p.in_squad = true;
    let d = p.detail.as_mut().unwrap();
    d.gear.add(draught(), 3);
    d.gear.add_piece(items::id("brewed_potion"), Piece::bottle(heal_mix()));
    if case {
        d.gear.add(items::id("potion_case"), 1);
    }
    p
}

fn fighter(p: &Person, side: u8, at: f32) -> Fighter {
    let mut f = Fighter::from_person(p, side, V2::new(at, 0.0), 0.0);
    f.think_at = f64::INFINITY;
    f
}

#[test]
fn a_potion_case_holds_six_and_the_rest_are_at_risk() {
    let mut g = Gear::default();
    g.add(draught(), 4);
    assert_eq!(bottles_at_risk(&g).len(), 4);
    g.add(items::id("potion_case"), 1);
    assert!(bottles_at_risk(&g).is_empty());
    g.add(draught(), 4);
    assert_eq!(bottles_at_risk(&g).len(), 2, "a case holds six");
    // Other things aren't bottles.
    let mut h = Gear::default();
    h.add(items::id("bandage"), 5);
    assert!(bottles_at_risk(&h).is_empty());
}

#[test]
fn only_the_squads_bottles_are_at_risk() {
    let mut p = carrier(1, false);
    assert_eq!(fighter(&p, 0, 0.0).exposed.len(), 4);
    p.in_squad = false;
    assert!(fighter(&p, 0, 0.0).exposed.is_empty(), "strangers restock at home");
    assert!(fighter(&carrier(2, true), 0, 0.0).exposed.is_empty(), "in the case");
}

#[test]
fn a_crack_takes_a_bottle_with_a_cue_and_a_line() {
    let p = carrier(1, false);
    let foe = carrier(2, true);
    let mut b = Battle::new(0, 3, 0.0, vec![fighter(&p, 0, 0.0), fighter(&foe, 1, 8.0)], vec!["Ana".into(), "B".into()]);
    let had = b.fighters[0].potions.len() + b.fighters[0].bottles.len();
    b.crack_bottle(0, 1.0, 1);
    let f = &b.fighters[0];
    assert_eq!(f.cracked.len(), 1);
    assert_eq!(f.exposed.len(), 3);
    assert_eq!(f.potions.len() + f.bottles.len() + 1, had, "it can't be drunk now");
    assert!(b.fx.iter().any(|x| matches!(x.kind, FxKind::Glass { who: 0, .. })), "the glint and splash");
    assert!(b.log.iter().any(|l| l.1.contains("Ana's") && l.1.contains("breaks")), "{:?}", b.log);
    // Nothing at risk: nothing breaks.
    b.crack_bottle(1, 1.0, 2);
    assert!(b.fighters[1].cracked.is_empty());
}

#[test]
fn burning_cracks_bottles_and_the_case_keeps_them() {
    for case in [false, true] {
        let p = carrier(1, case);
        let foe = carrier(2, true);
        let mut b = Battle::new(0, 3, 0.0, vec![fighter(&foe, 0, 0.0), fighter(&p, 1, 8.0)], vec!["A".into(), "B".into()]);
        // Set alight again and again for half a minute.
        for k in 0..40 {
            b.fighters[0].held = Some(spell("spark"));
            b.fighters[1].hp = b.fighters[1].max_hp;
            b.release(0, Some(1), b.fighters[1].pos);
            for _ in 0..25 {
                b.tick();
            }
            let _ = k;
        }
        assert!(b.fighters[1].burned > 30.0, "they burned: {}", b.fighters[1].burned);
        eprintln!("case {case}: burned {:.0} s, cracked {}", b.fighters[1].burned, b.fighters[1].cracked.len());
        let cracked = b.fighters[1].cracked.len();
        if case {
            assert_eq!(cracked, 0, "the case keeps them");
        } else {
            assert!(cracked > 0, "something cracked in the heat");
        }
        let _ = Does::Burning;
    }
}

#[test]
fn after_the_fight_the_cracked_ones_are_gone_from_the_pack() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.bag.retain(|e| items::item(e.0).kind != items::Kind::Potion);
        d.gear.add(draught(), 2);
        d.gear.add_piece(items::id("brewed_potion"), Piece::bottle(heal_mix()));
    }
    let mut f = Fighter::from_person(&w.people[m as usize], 0, w.squad.pos, w.time);
    assert_eq!(f.exposed.len(), 3);
    f.cracked = vec![(items::id("brewed_potion"), Some(heal_mix())), (draught(), None)];
    let t = w.time;
    w.bottles_broke(&f, t);
    let d = w.people[m as usize].detail.as_ref().unwrap();
    assert_eq!(d.gear.bag.iter().filter(|e| e.0 == draught()).map(|e| e.1).sum::<u16>(), 1);
    assert!(!d.gear.bag.iter().any(|e| e.0 == items::id("brewed_potion")));
    assert!(w.log.iter().any(|l| l.1.contains("draught of heal") && l.1.contains("break")), "{:?}", w.log.front());
}
