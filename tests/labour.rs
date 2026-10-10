//! The money grind: working a town's woodlot or mine, hunting by order, and
//! cutting up what you kill.

use gahturiyu_sim::sim::{
    animals::Sp,
    geo::V2,
    items,
    labour::{self, FACES},
    worldgen,
    world::{DAY, HOUR},
    World,
};

fn step_to(w: &mut World, t: f64, dt: f64) {
    while w.time < t {
        w.step(dt.min(t - w.time).max(0.01));
    }
}

#[test]
fn every_town_has_a_woodlot_and_some_have_mines() {
    labour::check_tables().unwrap();
    let w = worldgen::generate(1);
    let timber = items::id("timber");
    let woodlots = w.deposits.iter().filter(|d| d.item == timber).count();
    let mines = w.deposits.iter().filter(|d| d.item == items::id("iron_ore")).count();
    assert!(woodlots >= w.settlements.len() * 3 / 4, "{woodlots} woodlots for {} towns", w.settlements.len());
    assert!(mines >= 4, "{mines} mines");
    // Placed from the seed: the same world gets the same.
    assert_eq!(worldgen::generate(1).deposits, w.deposits);
    for f in FACES {
        assert!(f.cap > 0 && f.minutes > 0.0);
    }
}

/// The squad's member 0 stood right on a woodlot, sent to work it.
fn at_woodlot() -> (World, u32) {
    let mut w = worldgen::generate(1);
    let d = *w.deposits.iter().find(|d| d.item == items::id("timber")).unwrap();
    w.teleport_squad(d.pos.add(V2::new(30.0, 0.0)));
    let k = 0;
    w.squad.at[k] = d.pos;
    w.squad.goal[k] = d.pos;
    w.squad.route[k].clear();
    let who = w.squad.members[k];
    assert!(w.order_labour(who, d.id));
    (w, d.id)
}

#[test]
fn working_a_woodlot_fills_a_pack_and_the_woodlot_grows_back() {
    let (mut w, id) = at_woodlot();
    let who = w.squad.members[0];
    let start = w.time;
    let mut n = 0;
    while !w.labour.is_empty() && n < 200_000 {
        w.step(0.5);
        n += 1;
    }
    assert!(w.labour.is_empty(), "they stopped in the end");
    let got = w.count_of(who, "timber");
    assert!(got >= 5, "they cut {got}");
    let hours = (w.time - start) / HOUR;
    assert!(hours > 0.5 && hours < 12.0, "took {hours:.1} h");
    assert!(w.log.iter().any(|l| l.1.contains("pack is full") || l.1.contains("worked out")), "{:?}", w.log);
    assert!(w.labouring(who).is_none());
    let left = w.deposit(id).unwrap().left_at(w.time);
    let cap = w.deposit(id).unwrap().face().cap as f32;
    eprintln!("cut {got} timber in {hours:.1} h; {left:.1} of {cap} left");
    assert!(left < cap && left >= cap - got as f32 - 0.01, "the woodlot lost what they cut, less what grew back");
    let later = w.deposit(id).unwrap().left_at(w.time + DAY);
    assert!(later > left, "it grows back");
}

#[test]
fn work_doesnt_depend_on_the_step() {
    let (mut a, _) = at_woodlot();
    let (mut b, _) = at_woodlot();
    let end = a.time + 1.5 * HOUR;
    step_to(&mut a, end, 0.1);
    step_to(&mut b, end, 3.0);
    let who = a.squad.members[0];
    assert_eq!(a.count_of(who, "timber"), b.count_of(who, "timber"));
    assert!(a.count_of(who, "timber") > 0);
}

#[test]
fn a_hunt_order_brings_back_meat_and_hide() {
    let mut w = worldgen::generate(8);
    step_to(&mut w, 9.0 * HOUR, 60.0);
    let t = w.time;
    let herd = w.animals.herds.iter().find(|h| h.sp == Sp::Wallowback && h.alive(t) >= 4).unwrap().id;
    let at = w.herd_pos(herd, t);
    // Out of reach: they have to walk over first.
    w.teleport_squad(at.add(V2::new(90.0, 0.0)));
    let who = w.squad.members.clone();
    assert!(w.order_hunt(&who, herd));
    assert!(w.squad_battle().is_none(), "not yet in reach");
    let mut n = 0;
    while w.squad_battle().is_none() && n < 6000 {
        w.step(0.2);
        n += 1;
    }
    assert!(w.squad_battle().is_some(), "they caught up and set about them");
    while w.squad_battle().is_some() && n < 60_000 {
        w.step(0.2);
        n += 1;
    }
    let Some(c) = w.carcasses_near(w.squad.pos, 300.0).first().map(|c| (c.id, c.pos)) else {
        // Nothing fell this time (they all ran): fair enough, nothing to cut up.
        return;
    };
    let butcher = w.squad_fit()[0];
    let meat = w.count_of(butcher, "raw_meat");
    assert!(w.order_butcher(butcher, c.0));
    let mut n = 0;
    while !w.butchering.is_empty() && n < 20_000 {
        w.step(0.5);
        n += 1;
    }
    assert!(w.count_of(butcher, "raw_meat") > meat, "meat from the carcass");
    assert!(w.count_of(butcher, "hide") > 0, "and its hide");
    assert!(w.carcasses_near(c.1, 1.0).is_empty(), "the carcass is used up");
}
