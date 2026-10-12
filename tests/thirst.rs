//! Thirst (the items design, 0.6: light): it rises on the clock, faster on
//! the march; they drink from their skins and fill them at water; rain
//! slakes it; it never kills.

use gahturiyu_sim::sim::{
    condition::{self, COLLAPSE, DRINK_AT, SKIN_DRINKS},
    geo::V2,
    items,
    world::HOUR,
    worldgen, World,
};

/// The squad somewhere dry and far from any town, standing about.
fn out_in_the_wilds(seed: u64) -> World {
    let mut w = worldgen::generate(seed);
    let start = w.squad.pos;
    'find: for r in 1..40 {
        for k in 0..16 {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let p = start.add(V2::new(a.cos(), a.sin()).scale(r as f32 * 150.0));
            if !w.water_at(p) && gahturiyu_sim::sim::geo::is_land(p) && !w.terrain.is_sea(p) {
                w.teleport_squad(p);
                break 'find;
            }
        }
    }
    w
}

fn thirst(w: &World, m: u32) -> f32 {
    w.people[m as usize].cond.as_ref().unwrap().thirst_at(w.time)
}

fn water(w: &World, m: u32) -> f32 {
    w.people[m as usize].cond.as_ref().unwrap().water
}

/// Step the world `hours` on, in steps of `step` seconds.
fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

#[test]
fn they_drink_from_their_skins_and_then_go_dry() {
    let mut w = out_in_the_wilds(1);
    let m = w.squad.members[0];
    assert_eq!(water(&w, m), SKIN_DRINKS, "they set out with a full skin");
    wait(&mut w, 24.0, 30.0);
    assert!(water(&w, m) < SKIN_DRINKS, "drank from the skin");
    assert!(thirst(&w, m) < DRINK_AT + 5.0 || water(&w, m) < 1.0, "kept slaked while there was water");
    // Without water, through a dry spell: thirsty, parched, collapsed, but
    // alive. (Rain would slake them: out in it, it brings thirst down.)
    let at = w.squad.pos;
    let from = (w.time / HOUR).ceil() as i64;
    let dry = (from..from + 24 * 30).find(|&h| (0..30).all(|k| w.weather_at(at, (h + k) as f64 * HOUR).rain < 0.02)).expect("a dry spell");
    let hrs = dry as f64 - w.time / HOUR;
    wait(&mut w, hrs, 600.0);
    for &x in &w.squad.members.clone() {
        let c = w.people[x as usize].cond.as_mut().unwrap();
        c.water = 0.0;
        c.thirst = c.thirst.max(80.0);
    }
    wait(&mut w, 24.0, 120.0);
    let p = &w.people[m as usize];
    assert!(water(&w, m) < 1.0);
    assert!(thirst(&w, m) >= COLLAPSE - 0.5, "parched to collapse ({})", thirst(&w, m));
    assert!(w.is_down(m), "collapsed");
    assert!(!p.dead, "thirst never kills");
    wait(&mut w, 6.0, 600.0);
    assert!(!w.people[m as usize].dead, "it never kills");
}

#[test]
fn at_a_town_they_drink_their_fill_and_fill_their_skins() {
    let mut w = out_in_the_wilds(1);
    let m = w.squad.members[0];
    wait(&mut w, 30.0, 60.0);
    let t = &w.settlements[0];
    let at = t.pos;
    w.teleport_squad(at);
    wait(&mut w, 0.5, 5.0);
    assert!(w.water_at(at));
    assert!(thirst(&w, m) < DRINK_AT, "slaked ({})", thirst(&w, m));
    assert_eq!(water(&w, m), SKIN_DRINKS, "skins full");
}

#[test]
fn thirst_comes_out_the_same_however_the_world_is_stepped() {
    let w0 = out_in_the_wilds(2);
    let m = w0.squad.members[0];
    let run = |step: f64| {
        let mut w = w0.clone();
        wait(&mut w, 30.0, step);
        (thirst(&w, m), water(&w, m))
    };
    let (a, b) = (run(10.0), run(250.0));
    assert!((a.0 - b.0).abs() < 0.05 && (a.1 - b.1).abs() < 1e-3, "{a:?} vs {b:?}");
}

#[test]
fn a_long_march_with_no_water_is_warned_of() {
    let mut w = out_in_the_wilds(1);
    for &m in &w.squad.members.clone() {
        let c = w.people[m as usize].cond.as_mut().unwrap();
        c.water = 0.0;
        c.thirst = 40.0;
    }
    let far = w.squad.pos.add(V2::new(25_000.0, 0.0));
    w.order_squad(far);
    assert!(w.log.iter().any(|l| l.1.contains("little water for a march")), "warned");
}

#[test]
fn waterskins_hold_their_drinks() {
    let skin = items::id("waterskin");
    let w = worldgen::generate(1);
    let m = w.squad.members[0];
    assert!(w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().any(|e| e.0 == skin), "each member carries a skin");
    assert_eq!(w.skins_of(m) * SKIN_DRINKS, 4.0);
    assert!(condition::THIRST_WALKING > 1.0 && condition::THIRST_SLEEPING < 1.0);
}
