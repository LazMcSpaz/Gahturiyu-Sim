//! The tools' jobs (items design, Part 6): an axe for timber, a pick for a
//! seam, a skinning knife for a carcass, a repair kit away from a station.

use gahturiyu_sim::sim::{geo::V2, items, items::Slot, worldgen, World};

/// Member 0 standing on the nearest deposit of this kind.
fn at_face(item: &str) -> (World, u32, u32) {
    let mut w = worldgen::generate(1);
    let d = *w.deposits.iter().find(|d| d.item == items::id(item)).unwrap();
    w.teleport_squad(d.pos.add(V2::new(30.0, 0.0)));
    w.squad.at[0] = d.pos;
    w.squad.goal[0] = d.pos;
    w.squad.route[0].clear();
    let who = w.squad.members[0];
    (w, who, d.id)
}

fn give(w: &mut World, who: u32, key: &str) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(items::id(key), 1);
}

#[test]
fn an_axe_cuts_timber_faster_and_a_hatchet_a_little() {
    let (mut w, who, d) = at_face("timber");
    w.people[who as usize].detail.as_mut().unwrap().gear.bag.retain(|e| !matches!(items::item(items::info(e.0).form).key, "hatchet" | "woodcutters_axe"));
    let bare = w.unit_minutes(who, d);
    give(&mut w, who, "hatchet");
    let hatchet = w.unit_minutes(who, d);
    give(&mut w, who, "woodcutters_axe");
    let axe = w.unit_minutes(who, d);
    assert!(axe < hatchet && hatchet < bare, "axe {axe}, hatchet {hatchet}, bare hands {bare}");
}

#[test]
fn a_seam_needs_a_pick() {
    let (mut w, who, d) = at_face("iron_ore");
    w.people[who as usize].detail.as_mut().unwrap().gear.bag.retain(|e| !matches!(items::item(items::info(e.0).form).key, "mining_pick" | "war_pick"));
    if w.people[who as usize].detail.as_ref().unwrap().gear.in_slot(Slot::MainHand).is_some_and(|i| items::item(items::info(i).form).key == "war_pick") {
        w.people[who as usize].detail.as_mut().unwrap().gear.discard(Slot::MainHand);
    }
    assert!(!w.order_labour(who, d), "no pick, no ore");
    assert!(w.log.iter().any(|l| l.1.contains("needs a mining pick")), "{:?}", w.log);
    give(&mut w, who, "mining_pick");
    assert!(w.order_labour(who, d));
}

#[test]
fn a_repair_kit_mends_part_of_the_way() {
    let mut w = worldgen::generate(1);
    // Out in the wilds, away from any station.
    w.teleport_squad(w.squad.pos.add(V2::new(400.0, 0.0)));
    let who = w.squad.members[0];
    let slot = Slot::BodyOver;
    let (_, most) = w.wear_of(who, slot).expect("a coat on");
    w.people[who as usize].detail.as_mut().unwrap().gear.piece_mut(slot).unwrap().left = most * 0.2;
    w.care_for(who, slot);
    assert!(w.wear_of(who, slot).unwrap().0 < most * 0.25, "no kit, no mending out here");
    give(&mut w, who, "repair_kit");
    w.care_for(who, slot);
    let (left, _) = w.wear_of(who, slot).unwrap();
    assert!((left - most * gahturiyu_sim::sim::wear::FIELD_REPAIR).abs() < 0.5, "{left} of {most}");
    assert_eq!(w.count_of(who, "repair_kit"), 0, "the kit is used up");
    // It can do no more than that.
    give(&mut w, who, "repair_kit");
    w.care_for(who, slot);
    assert_eq!(w.count_of(who, "repair_kit"), 1);
}
