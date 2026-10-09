//! Fire, water and cold: burning, wet, chilled and frozen, and how they meet.

use gahturiyu_sim::sim::{
    combat::{Battle, Fighter},
    effects::Does,
    elements,
    geo::V2,
    items,
    magic::spell,
    person::Person,
    race::Race,
    stats::{Calling, Skill},
    worldgen,
};

fn fighter(id: u32, side: u8, at: f32) -> Fighter {
    let mut p = Person::summary(id, 900 + id as u64, Race::Roduro, None);
    p.specialize(Calling::Warrior, &[(Skill::Blade, 40.0)], 200.0);
    p.ensure_detail();
    let mut f = Fighter::from_person(&p, side, V2::new(at, 0.0), 0.0);
    f.think_at = f64::INFINITY;
    f
}

fn battle() -> Battle {
    Battle::new(0, 3, 0.0, vec![fighter(1, 0, 0.0), fighter(2, 1, 8.0)], vec!["A".into(), "B".into()])
}

/// The caster lets go of a spell (held ready, so it can't fizzle) at fighter 1.
fn hit(b: &mut Battle, key: &str) {
    b.fighters[0].held = Some(spell(key));
    assert!(b.release(0, Some(1), b.fighters[1].pos));
    b.tick();
}

fn hp(b: &Battle) -> f32 {
    b.fighters[1].hp.iter().sum()
}

#[test]
fn fire_sets_people_burning_and_they_light_the_dark() {
    let mut b = battle();
    // A moonless fight.
    b.lights = Some(Vec::new());
    b.start = 2.0 * 3600.0;
    b.time = b.start;
    let dark = b.light_at(b.fighters[1].pos);
    hit(&mut b, "spark");
    assert!(b.fighters[1].has(Does::Burning).is_some(), "on fire");
    assert!(b.light_at(b.fighters[1].pos) > dark + 0.2, "the fire lights them up");
    let after_hit = hp(&b);
    for _ in 0..20 {
        b.tick();
    }
    assert!(hp(&b) < after_hit, "it keeps burning");
    for _ in 0..100 {
        b.tick();
    }
    assert!(b.fighters[1].has(Does::Burning).is_none(), "and burns out");
    assert!(b.fighters[1].burned > 4.0);
}

#[test]
fn water_and_fire_make_steam() {
    let mut b = battle();
    hit(&mut b, "drench");
    assert!(b.fighters[1].has(Does::Wet).is_some());
    hit(&mut b, "spark");
    assert!(b.fighters[1].has(Does::Burning).is_none(), "the wet don't catch");
    assert!(b.fighters[1].has(Does::Wet).is_none(), "and the fire dries them");
    // Water puts a fire out.
    hit(&mut b, "spark");
    assert!(b.fighters[1].has(Does::Burning).is_some());
    hit(&mut b, "drench");
    assert!(b.fighters[1].has(Does::Burning).is_none());
}

#[test]
fn lightning_bites_deeper_on_the_wet() {
    let mut dry = battle();
    let mut wet = battle();
    wet.fighters[1].statuses.push(gahturiyu_sim::sim::magic::Status { does: Does::Wet, power: 1.0, until: 1e9 });
    let (d0, w0) = (hp(&dry), hp(&wet));
    hit(&mut dry, "lightning_bolt");
    hit(&mut wet, "lightning_bolt");
    let (d, w) = (d0 - hp(&dry), w0 - hp(&wet));
    assert!(w > d * (elements::WET_SHOCK - 0.05), "dry {d}, wet {w}");
}

#[test]
fn cold_on_the_wet_freezes_them_stiff() {
    let mut b = battle();
    hit(&mut b, "drench");
    let mut frozen = false;
    for _ in 0..6 {
        hit(&mut b, "chill");
        if b.fighters[1].has(Does::Frozen).is_some() {
            frozen = true;
            break;
        }
    }
    assert!(frozen, "enough cold freezes them");
    assert!(b.fighters[1].helpless());
    // Fire thaws them.
    b.fighters[1].statuses.retain(|s| s.does != Does::Wet);
    hit(&mut b, "spark");
    assert!(b.fighters[1].has(Does::Frozen).is_none());
}

#[test]
fn paper_burns_and_wet_people_cant_light_torches() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let scroll = items::id("scroll_heal");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(scroll, 20);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("note"), 4);
    let mut f = Fighter::from_person(&w.people[m as usize], 0, V2::new(0.0, 0.0), w.time);
    f.burned = 12.0;
    let t = w.time;
    w.paper_spoils(&f, 77, t);
    let left = w.count_of(m, "scroll_heal");
    assert!(left < 20 && left > 0, "some of it burned: {left} of 20 left");
    assert!(elements::is_paper(items::id("note")) && !elements::is_paper(items::id("knife")));
    // Soaked, nothing will light.
    w.boons.push(gahturiyu_sim::sim::casting::Boon { pid: m, does: Does::Wet, power: 1.0, until: t + 600.0 });
    assert!(!w.torch_lit(m));
    assert!(!w.toggle_torch(m), "too wet");
}
