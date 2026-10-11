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
    w.people[a as usize].detail.as_mut().unwrap().gear.add(items::id("scale_hauberk"), 10);
    let loaded = w.member_speed(a);
    assert!(loaded < free * 0.8, "{loaded} vs {free}");

    // Walk both the same distance, each their own way (sent together, they'd
    // keep to the slower one's pace: BL-47); the loaded one falls behind.
    w.order_members(&[a], w.squad.at[0].add(V2::new(60.0, 0.0)));
    w.order_members(&[b], w.squad.at[1].add(V2::new(-60.0, 0.0)));
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
    let gear = &w.people[victim as usize].detail.as_ref().unwrap().gear;
    assert!(gear.equipped().count() >= 2);
    // What they wear, and what's in their pack (a bandit's purse among it).
    let carried = gear.equipped().count() + gear.bag.len();
    let lying = w.ground.len();
    w.drop_everything(victim, at);
    assert!(w.people[victim as usize].detail.as_ref().unwrap().gear.equipped().next().is_none());
    let near: Vec<u32> = w.ground.iter().filter(|g| g.pos.dist(at) < 3.0).map(|g| g.id).collect();
    assert_eq!(near.len(), carried);
    let me = w.squad.members[0];
    w.order_pickup(me, near[0]);
    for _ in 0..60 {
        w.step(0.5);
    }
    assert_eq!(w.ground.len(), lying + carried - 1);
}

/// BL-67: in a fight, "N m from the others" goes by where the fight has
/// each member, not where they stood when it began.
#[test]
fn bl67_strayed_goes_by_where_the_fight_has_them() {
    let mut w = world();
    let squad = w.squad.members.clone();
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 3, false);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.attack(&squad, foe));
    assert!(w.squad_battle().is_some());
    assert!(squad.iter().all(|&m| w.strayed(m).is_none()), "together as it starts");
    // One of them is drawn 90 m off by the fight.
    let m = squad[1];
    let b = w.battles.iter_mut().find(|b| b.fighters.iter().any(|f| f.pid == m)).unwrap();
    let f = b.fighters.iter_mut().find(|f| f.pid == m).unwrap();
    f.pos = f.pos.add(V2::new(0.0, 90.0));
    let off = w.strayed(m).expect("drawn 90 m off, they've strayed");
    assert!((off - 90.0).abs() < 25.0, "{off}");
    assert!(squad.iter().filter(|&&x| x != m).all(|&x| w.strayed(x).is_none()), "the rest haven't");
}
