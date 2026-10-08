//! Squad members move on their own, at their own pace, and can drop and pick
//! things up.

use gahturiyu_sim::sim::{geo::V2, items, worldgen, World};

fn walk(w: &mut World, secs: f64) {
    let mut t = 0.0;
    while t < secs {
        w.step(0.5);
        t += 0.5;
    }
}

/// A world with the squad on flat-ish open land and nobody hostile about.
fn world() -> World {
    worldgen::generate(21)
}

#[test]
fn only_the_members_told_to_move_do() {
    let mut w = world();
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let (a0, b0) = (w.person_pos(a), w.person_pos(b));
    let target = a0.add(V2::new(25.0, 10.0));
    w.order_members(&[a], target);
    walk(&mut w, 60.0);
    assert!(w.person_pos(a).dist(target) < 1.0, "the ordered member should have arrived");
    assert!(w.person_pos(b).dist(b0) < 1e-3, "the other should have stayed put");
}

#[test]
fn a_heavy_pack_slows_one_member_down() {
    let mut w = world();
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    let free = w.member_speed(a);
    w.people[a as usize].detail.as_mut().unwrap().gear.add(items::id("scale_hauberk"), 6);
    let loaded = w.member_speed(a);
    assert!(loaded < free * 0.8, "{loaded} vs {free}");

    // Walk both the same distance; the loaded one falls behind.
    let target = w.squad.pos.add(V2::new(60.0, 0.0));
    w.order_members(&[a, b], target);
    walk(&mut w, 20.0);
    let ka = w.squad.index(a).unwrap();
    let kb = w.squad.index(b).unwrap();
    let (da, db) = (w.squad.at[ka].dist(w.squad.goal[ka]), w.squad.at[kb].dist(w.squad.goal[kb]));
    assert!(da > db + 3.0, "loaded member should lag: {da} left vs {db}");
}

#[test]
fn enchanted_boots_speed_walking() {
    let mut w = world();
    let a = w.squad.members[2];
    let before = w.member_speed(a);
    w.people[a as usize].detail.as_mut().unwrap().gear.wear(items::id("striders_boots"));
    assert!(w.member_speed(a) > before * 1.15);
}

#[test]
fn equipping_changes_might() {
    let mut w = world();
    let a = w.squad.members[0];
    let before = w.people[a as usize].might;
    w.people[a as usize].detail.as_mut().unwrap().gear.add(items::id("longsword"), 1);
    assert!(w.equip(a, items::id("longsword")));
    assert_ne!(w.people[a as usize].might, before);
    let slot = items::item(items::id("longsword")).slot;
    assert!(w.unequip(a, slot));
}

#[test]
fn things_can_be_dropped_and_picked_up_by_someone_else() {
    let mut w = world();
    let (a, b) = (w.squad.members[0], w.squad.members[3]);
    let ring = items::id("ring_might");
    w.people[a as usize].detail.as_mut().unwrap().gear.add(ring, 1);
    assert!(w.drop_item(a, ring));
    assert!(!w.people[a as usize].detail.as_ref().unwrap().gear.bag.iter().any(|e| e.0 == ring));
    let thing = w.ground.iter().find(|g| g.item == ring).expect("on the ground").id;

    // Somebody further away walks over for it.
    assert!(w.order_pickup(b, thing));
    assert!(w.ground.iter().any(|g| g.id == thing), "can't pick it up from across the way");
    walk(&mut w, 30.0);
    assert!(!w.ground.iter().any(|g| g.id == thing), "should have been picked up");
    assert!(w.people[b as usize].detail.as_ref().unwrap().gear.bag.iter().any(|e| e.0 == ring));
}

#[test]
fn a_body_can_be_stripped_and_its_things_picked_up() {
    // Fights call this for everyone who dies in them.
    let mut w = world();
    let at = w.squad.pos.add(V2::new(6.0, 0.0));
    // Spawned well away, so they don't start a fight.
    let g = w.spawn_bandits(w.squad.pos.add(V2::new(400.0, 0.0)), 1, false);
    let victim = w.group(g).unwrap().members[0];
    let carried = w.people[victim as usize].detail.as_ref().unwrap().gear.equipped().count();
    assert!(carried >= 2);
    w.drop_everything(victim, at);
    assert!(w.people[victim as usize].detail.as_ref().unwrap().gear.equipped().next().is_none());
    let near: Vec<u32> = w.ground.iter().filter(|g| g.pos.dist(at) < 3.0).map(|g| g.id).collect();
    assert_eq!(near.len(), carried);
    let me = w.squad.members[0];
    w.order_pickup(me, near[0]);
    for _ in 0..60 {
        w.step(0.5);
    }
    assert_eq!(w.ground.len(), carried - 1);
}
