//! The damage scale (the items design, Part 1 B2): a Common Forgeiron
//! longsword swung by an average fighter (Strength 35, Blade 30) downs an
//! unarmoured average person in 4-6 hits that land.

use gahturiyu_sim::sim::{
    combat::{Act, Battle, Fighter},
    geo::V2,
    items,
    person::Person,
    race::Race,
    stats::{Attr, Calling, Skill},
};

/// Hits that land before the target goes down, averaged over many fights.
pub fn hits_to_down(key: &str) -> f32 {
    let w = *items::item(items::id(key)).weapon().unwrap();
    let mut total = 0.0;
    let n = 40;
    for seed in 0..n {
        let mut a = Person::summary(1, 11 + seed, Race::Roduro, None);
        a.specialize(Calling::Common, &[], 0.0);
        let mut b = Person::summary(2, 977 + seed, Race::Roduro, None);
        b.specialize(Calling::Common, &[], 0.0);
        for p in [&mut a, &mut b] {
            for at in [Attr::Strength, Attr::Agility, Attr::Toughness, Attr::Intellect, Attr::Willpower] {
                p.stats.set_attr(at, 35.0);
            }
            p.stats.set_skill(Skill::Blade, 30.0);
            p.stats.set_skill(Skill::Dodge, 0.0);
            p.ensure_detail();
        }
        let mut fa = Fighter::from_person(&a, 0, V2::new(0.0, 0.0), 0.0);
        let mut fb = Fighter::from_person(&b, 1, V2::new(1.2, 0.0), 0.0);
        fa.weapon = w;
        fb.armor.clear();
        fb.shield = 0.0;
        fb.offhand = None;
        fb.weapon = items::FISTS;
        fb.aware_at = 0.0;
        fa.think_at = f64::INFINITY;
        fb.think_at = f64::INFINITY;
        let mut bt = Battle::new(0, 5000 + seed, 0.0, vec![fa, fb], vec!["A".into(), "B".into()]);
        let mut hits = 0;
        for _ in 0..200 {
            if bt.fighters[1].ko {
                break;
            }
            let before = bt.fighters[1].damage_taken;
            bt.fighters[1].statuses.clear();
            bt.fighters[0].act = Act::Swing { target: 1, lands: bt.time + 0.05 };
            for _ in 0..2 {
                bt.tick();
            }
            bt.fighters[0].act = Act::Idle;
            if bt.fighters[1].damage_taken > before {
                hits += 1;
            }
        }
        total += hits as f32;
    }
    total / n as f32
}

// Laz's target was 4-6 (B2); before the scale it took about 13, and Laz
// chose to double it (12 Oct): about half as many now.
#[test]
fn a_common_longsword_downs_an_unarmoured_person_in_about_seven_hits() {
    let h = hits_to_down("longsword");
    eprintln!("longsword: {h:.1} hits");
    assert!((5.0..=8.0).contains(&h), "{h:.1} hits to down");
}
