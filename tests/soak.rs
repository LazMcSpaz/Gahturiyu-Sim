//! Containers, piece B (the items design, 0.4): the wet soaks through a
//! pack on the clock and then spoils papers inside, unless they're guarded
//! (a scroll case, an oilskin) or the pack is sealed; fire likewise; and
//! finery on show changes how people take you.

use gahturiyu_sim::sim::{
    combat::Fighter,
    geo::V2,
    items::{self, Slot},
    soak,
    world::HOUR,
    worldgen, World,
};

fn id(key: &str) -> items::ItemId {
    items::id(key)
}

fn count(w: &World, m: u32, key: &str) -> u16 {
    w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == id(key)).map(|e| e.1).sum()
}

/// A world just before the first spell of steady rain (three hours or more
/// at 0.4 or harder) where the squad stands, out in the open.
fn before_rain(seed: u64) -> Option<World> {
    let w = worldgen::generate(seed);
    let at = w.squad.pos;
    let start = (w.time / HOUR).ceil() as i64;
    for h in start..start + 24 * 20 {
        if (0..3).all(|k| w.weather_at(at, (h + k) as f64 * HOUR).rain >= 0.4) {
            let mut w = w;
            // (Big steps there: nothing of the squad's changes meanwhile.)
            let target = (h - 1) as f64 * HOUR;
            while w.time < target {
                let dt = (target - w.time).min(1800.0);
                w.step(dt);
            }
            return Some(w);
        }
    }
    None
}

/// Member `m` carries `papers` reed paper and the given pack and cases.
fn kit(w: &mut World, m: u32, pack: &str, cases: &[&str], papers: u16) {
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.unequip(Slot::Back);
    d.gear.add(id(pack), 1);
    d.gear.equip(id(pack)).unwrap();
    for c in cases {
        d.gear.add(id(c), 1);
    }
    d.gear.add(id("reed_paper"), papers);
}

#[test]
fn rain_soaks_through_a_plain_pack_and_spoils_what_isnt_guarded() {
    let mut w = (1..6).find_map(before_rain).expect("a spell of rain somewhere");
    let m = w.squad.members[0];
    kit(&mut w, m, "small_pack", &["oilskin_wrap"], 8);
    let end = w.time + 5.0 * HOUR;
    let mut through_at = None;
    while w.time < end {
        w.step(30.0);
        if through_at.is_none() && w.soak_at(m, w.time).0 >= 1.0 {
            through_at = Some(w.time);
        }
    }
    assert!(through_at.is_some(), "soaked through");
    assert!(w.log.iter().any(|l| l.1.contains("ink is running")), "warned first");
    assert_eq!(count(&w, m, "reed_paper"), 3, "only the three in the oilskin survive");
    assert!(w.log.iter().any(|l| l.1.contains("soaked to pulp")));
}

#[test]
fn a_sealed_sea_bag_keeps_it_all_dry() {
    let mut w = (1..6).find_map(before_rain).expect("a spell of rain somewhere");
    let m = w.squad.members[0];
    kit(&mut w, m, "sea_bag", &[], 8);
    w.step(1.0);
    // (Any drizzle before it went on stays where it got.)
    let start = w.soak_at(m, w.time).0;
    let end = w.time + 5.0 * HOUR;
    while w.time < end {
        w.step(60.0);
    }
    assert!(w.soak_at(m, w.time).0 <= start + 1e-4, "nothing more got in");
    assert_eq!(count(&w, m, "reed_paper"), 8);
}

#[test]
fn soaking_comes_out_the_same_however_the_world_is_stepped() {
    let w0 = (1..6).find_map(before_rain).expect("a spell of rain somewhere");
    let m = w0.squad.members[0];
    let run = |step: f64| {
        let mut w = w0.clone();
        kit(&mut w, m, "small_pack", &[], 5);
        let end = w0.time + 4.0 * HOUR;
        while w.time < end - 1e-6 {
            w.step(step.min(end - w.time));
        }
        let pulp = w.log.iter().find(|l| l.1.contains("soaked to pulp")).map(|l| l.0);
        (w.soak_at(m, end), pulp, count(&w, m, "reed_paper"))
    };
    let (a, b) = (run(20.0), run(300.0));
    assert!((a.0 .0 - b.0 .0).abs() < 1e-3 && (a.0 .1 - b.0 .1).abs() < 1e-3, "{:?} vs {:?}", a.0, b.0);
    assert_eq!(a.2, b.2);
    match (a.1, b.1) {
        (Some(x), Some(y)) => assert!((x - y).abs() < 1.0, "pulp at {x} vs {y}"),
        (x, y) => assert_eq!(x.is_some(), y.is_some()),
    }
}

#[test]
fn guards_count_what_they_hold_and_only_against_what_they_seal() {
    let mut g = gahturiyu_sim::sim::inventory::Gear::default();
    g.add(id("reed_paper"), 10);
    g.add(id("scroll_case"), 1);
    let wet = soak::guarded(&g, false);
    assert_eq!(wet.iter().find(|x| x.0 == id("reed_paper")).map(|x| x.1), Some(6), "a scroll case keeps six dry");
    // It only resists fire: in a fire it guards nothing outright.
    assert!(soak::guarded(&g, true).is_empty());
    g.add(id("strongbox"), 1);
    assert!(soak::guarded(&g, true).iter().any(|x| x.0 == id("reed_paper")), "a strongbox is sealed against fire");
}

#[test]
fn papers_in_a_strongbox_survive_a_fire() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.add(id("reed_paper"), 12);
        d.gear.add(id("strongbox"), 1);
    }
    let mut f = Fighter::from_person(&w.people[m as usize], 0, V2::default(), w.time);
    f.burned = 30.0;
    let t = w.time;
    w.paper_spoils(&f, 7, t);
    let left = count(&w, m, "reed_paper");
    assert!(left >= 8, "the eight in the strongbox are safe ({left} left)");
}

#[test]
fn finery_on_show_wins_respect() {
    let mut w = worldgen::generate(1);
    let m = w.squad.members[0];
    let npc = w.settlements[0].residents[0];
    let before = w.disposition(npc, m);
    assert_eq!(w.shown_wealth(m), 0.0);
    {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        for k in ["gold_ring", "pearl_necklace"] {
            d.gear.add(id(k), 1);
            d.gear.equip(id(k)).unwrap();
        }
    }
    assert!(w.shown_wealth(m) >= 230.0);
    let after = w.disposition(npc, m);
    assert!(after > before + 4.0, "better disposed ({before} -> {after})");
}
