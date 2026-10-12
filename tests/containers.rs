//! Containers (the items design, 0.4), piece A: what each is like, bulk that
//! grows with fullness, packs that slow, quick slots on a belt, a duffel
//! in both hands, and putting a pack down and picking it up again.

use gahturiyu_sim::sim::{
    combat::{self, Battle, Fighter},
    geo::V2,
    inventory::Gear,
    items::{self, Holds, Proof, Slot},
    worldgen, World,
};

fn id(key: &str) -> items::ItemId {
    items::id(key)
}

fn wearing(keys: &[&str]) -> Gear {
    let mut g = Gear::default();
    for k in keys {
        g.add(id(k), 1);
        g.equip(id(k)).unwrap();
    }
    g
}

#[test]
fn every_container_says_what_it_is_like() {
    let sea = items::container(id("sea_bag")).unwrap();
    assert_eq!(sea.water, Proof::Sealed, "a sea bag keeps the wet out");
    let large = items::container(id("large_pack")).unwrap();
    let satchel = items::container(id("satchel")).unwrap();
    assert!(large.bulk > satchel.bulk * 3.0, "a satchel is the sneak's bag");
    assert!(items::container(id("duffel")).unwrap().hands);
    assert_eq!(items::container(id("quiver")).unwrap().holds, Holds::Ammo);
    assert_eq!(items::container(id("bandolier")).unwrap().quick, 4);
    assert!(items::container(id("longsword")).is_none());
}

#[test]
fn bulk_and_slowing_grow_as_a_pack_fills() {
    let mut g = wearing(&["large_pack"]);
    let (empty_bulk, empty_pace) = (g.bulk(), g.pack_pace());
    assert!(empty_bulk > 0.0 && empty_pace < 1.0, "even empty it costs a little");
    // (Pockets fill first: 30 cells, then the pack.)
    g.add(id("timber"), 900);
    assert!(g.bulk() > empty_bulk * 1.5, "full, much more ({} vs {empty_bulk})", g.bulk());
    assert!(g.pack_pace() < empty_pace);
    let none = Gear::default();
    assert_eq!((none.bulk(), none.pack_pace()), (0.0, 1.0), "nothing carried, nothing lost");
    // A satchel hardly shows.
    let mut s = wearing(&["satchel"]);
    s.add(id("timber"), 900);
    assert!(s.bulk() < g.bulk() / 3.0);
}

#[test]
fn a_bulky_pack_costs_dodging_and_pace_in_a_fight() {
    let w = worldgen::generate(1);
    let m = w.squad.members[0];
    let mut p = w.people[m as usize].clone();
    let base = Fighter::from_person(&p, 0, V2::default(), 0.0);
    let d = p.detail.as_mut().unwrap();
    d.gear.add(id("large_pack"), 1);
    d.gear.equip(id("large_pack")).unwrap();
    d.gear.add(id("timber"), 900);
    let laden = Fighter::from_person(&p, 0, V2::default(), 0.0);
    assert!(laden.dodge_penalty > base.dodge_penalty, "dodges worse");
    assert!(laden.base_speed < base.base_speed, "and is slower");
}

#[test]
fn quick_slots_take_only_what_is_quick_to_use_and_it_is_quicker() {
    let mut g = wearing(&["bandolier"]);
    let draught = id("healing_draught");
    g.add(draught, 3);
    g.add(id("timber"), 2);
    let quick = g.quick();
    assert_eq!(quick.iter().filter(|&&x| x == draught).count(), 3, "the draughts go in the quick slots first");
    assert!(!quick.contains(&id("timber")), "timber doesn't");
    let names: Vec<String> = (0..g.grids().len()).map(|k| g.grid_name(k)).collect();
    assert!(names.iter().any(|n| n.contains("quick")), "{names:?}");
    // A quiver takes only ammunition.
    let mut q = wearing(&["quiver"]);
    q.add(id("arrows"), 20);
    q.add(id("bandage"), 1);
    let holders = q.holders();
    let (placed, _) = q.arrange();
    for p in &placed {
        let it = q.bag[p.entry].0;
        if holders[p.grid as usize].2 == Holds::Ammo {
            assert_eq!(it, id("arrows"), "only arrows in the quiver");
        }
    }
    // In a fight, a draught from the belt is drunk in half the time.
    let w = worldgen::generate(1);
    let m = w.squad.members[0];
    let drink_time = |belt: bool| {
        let mut p = w.people[m as usize].clone();
        let d = p.detail.as_mut().unwrap();
        if belt {
            d.gear.add(id("bandolier"), 1);
            d.gear.equip(id("bandolier")).unwrap();
        }
        d.gear.add(draught, 1);
        let f = Fighter::from_person(&p, 0, V2::default(), 0.0);
        let mut b = Battle::new(0, 1, 0.0, vec![f], vec!["A".into()]);
        assert!(b.begin_drink(0, draught));
        match b.fighters[0].act {
            combat::Act::Drink { done, .. } => done,
            _ => panic!("drinking"),
        }
    };
    assert!(drink_time(true) < drink_time(false) * 0.6, "quicker from the belt");
}

#[test]
fn a_duffel_takes_both_hands_and_is_put_down_to_fight() {
    let mut g = wearing(&["longsword"]);
    g.add(id("round_shield"), 1);
    g.equip(id("round_shield")).unwrap();
    g.add(id("duffel"), 1);
    g.equip(id("duffel")).unwrap();
    assert!(g.hands_full());
    assert_eq!(g.in_slot(Slot::OffHand), None, "nothing else held");
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.add(id("duffel"), 1);
        d.gear.equip(id("duffel")).unwrap();
        // (Past what the pockets take, so some is in the duffel.)
        d.gear.add(id("timber"), 600);
    }
    let at = w.squad.pos.add(V2::new(10.0, 0.0));
    let band = w.spawn_bandits(at, 2, false);
    let foe = w.group(band).unwrap().members[0];
    assert!(w.attack(&[m], foe));
    assert!(!w.people[m as usize].detail.as_ref().unwrap().gear.hands_full(), "put down to fight");
    assert!(w.ground.iter().any(|g| g.item == id("duffel") && !g.inside.is_empty()), "on the ground with what's in it");
}

#[test]
fn a_pack_put_down_keeps_what_is_in_it_and_goes_back_on() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let held = |w: &World, key: &str| w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == id(key)).map(|e| e.1).sum::<u16>();
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.add(id("small_pack"), 1);
        d.gear.equip(id("small_pack")).unwrap();
        d.gear.add(id("timber"), 400);
    }
    let timber = held(&w, "timber");
    let weight = w.people[m as usize].kit().weight();
    let line = w.drop_pack(m).expect("put down");
    assert!(line.contains("puts down"));
    assert!(held(&w, "timber") < timber, "what was in the pack went with it");
    assert!(w.people[m as usize].kit().weight() < weight);
    let g = w.ground.iter().find(|g| g.item == id("small_pack")).expect("on the ground").clone();
    assert!(!g.inside.is_empty());
    // Picked up: back on, with everything in it.
    assert!(w.order_pickup(m, g.id));
    for _ in 0..200 {
        w.step(0.25);
        if w.ground.iter().all(|x| x.id != g.id) {
            break;
        }
    }
    assert!(w.ground.iter().all(|x| x.id != g.id), "picked up");
    assert_eq!(w.people[m as usize].detail.as_ref().unwrap().gear.in_slot(Slot::Back), Some(id("small_pack")), "worn again");
    assert_eq!(held(&w, "timber"), timber, "everything back");
}
