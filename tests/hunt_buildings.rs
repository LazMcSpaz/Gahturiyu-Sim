//! Bugs found in the bug hunt (the wild), shown by tests. Each is ignored
//! with its ID from `claude/bugs-buildings.md` until it's fixed; run them
//! with `cargo test --release --test hunt_buildings -- --ignored`.

use gahturiyu_sim::sim::{geo::V2, world::HOUR, worldgen};

#[test]
fn bl_1_an_id_that_does_not_exist_is_refused_not_a_crash() {
    // RG-5 = BL-1 = NM-9: every order that names a person.
    use gahturiyu_sim::sim::{items, magic};
    let mut w = worldgen::generate(1);
    let all = w.squad.members.clone();
    for nobody in [w.people.len() as u32, w.people.len() as u32 + 100, 999_999, u32::MAX] {
        assert!(!w.valid_person(nobody));
        assert!(!w.attack(&all, nobody));
        assert!(!w.order_carry(all[0], nobody));
        assert!(!w.can_carry(all[0], nobody));
        assert!(!w.order_talk(all[0], nobody));
        assert!(!w.can_loot(nobody));
        assert!(!w.order_loot(all[0], nobody));
        for m in all.clone() {
            for s in w.known_spells(m) {
                assert!(w.order_cast(m, s, Some(nobody), None).is_err());
                assert!(w.use_spell(m, s, Some(nobody), None).is_err());
            }
        }
        let scroll = items::id("scroll_paralyze");
        assert!(w.order_read(all[0], scroll, Some(nobody), None).is_err());
        assert!(w.use_in_fight(all[0], scroll, Some(nobody)).is_err());
        let _ = magic::SPELLS.len();
    }
    assert!(w.valid_person(all[0]));
}

#[test]
#[ignore = "BL-2"]
fn bl_2_an_order_to_nowhere_leaves_the_squad_where_it_is() {
    let mut w = worldgen::generate(1);
    let before = w.squad.pos;
    w.order_squad(V2::new(f32::NAN, f32::NAN));
    w.step(1.0);
    assert!(w.squad.pos.x.is_finite() && w.squad.pos.y.is_finite(), "the squad's place is lost");
    assert!(w.squad.pos.dist(before) < 5.0);
}

#[test]
#[ignore = "BL-10"]
fn bl_10_attack_needs_the_target_somewhere_near() {
    let mut w = worldgen::generate(1);
    let here = w.squad.pos;
    // A bandit a couple of kilometres off.
    let far = w.camps.iter().map(|c| c.pos).filter(|p| p.dist(here) > 1500.0).min_by(|a, b| a.dist(here).total_cmp(&b.dist(here))).expect("a far camp");
    w.teleport_squad(here);
    let bandit = w.people.iter().find(|p| p.bandit && w.person_pos(p.id).dist(far) < 60.0).map(|p| p.id).expect("a bandit at that camp");
    let all = w.squad.members.clone();
    assert!(!w.attack(&all, bandit), "a fight started with a bandit {:.0} m away", w.person_pos(bandit).dist(here));
    let _ = HOUR;
}
