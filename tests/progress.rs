//! Progress you can see: a new whole level in a skill is announced once.

use gahturiyu_sim::sim::{stats::Skill, worldgen};

#[test]
fn a_skill_level_up_is_announced_once() {
    let mut w = worldgen::generate(1);
    w.step(0.1);
    let m = w.squad.members[0];
    assert!(w.fresh_level_up(m).is_none());
    let v = w.people[m as usize].stats.skill(Skill::Athletics);
    w.people[m as usize].stats.set_skill(Skill::Athletics, v.floor() + 1.2);
    w.step(0.1);
    let (what, lvl) = w.fresh_level_up(m).expect("shown on the card");
    assert_eq!((what, lvl), ("Athletics", v.floor() as u8 + 1));
    let said = w.log.iter().filter(|l| l.1.contains("improves")).count();
    assert_eq!(said, 1);
    w.step(0.1);
    assert_eq!(w.log.iter().filter(|l| l.1.contains("improves")).count(), 1, "said once");
    for _ in 0..30 {
        w.step(60.0);
    }
    assert!(w.fresh_level_up(m).is_none(), "gone from the card after a while");
    // A load doesn't announce anything again.
    let mut back = gahturiyu_sim::sim::World::load_bytes(&w.save_bytes()).ok().unwrap();
    let count = |w: &gahturiyu_sim::sim::World| w.log.iter().filter(|l| l.1.contains("improves")).count();
    let before = count(&back);
    back.step(0.1);
    assert_eq!(count(&back), before, "nothing announced again after a load");
}
