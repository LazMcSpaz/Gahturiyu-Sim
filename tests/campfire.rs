//! The squad's own campfires (0.6): laid from a timber, lit (quicker with a
//! tinderbox), burning by the clock, fed with more wood; while it burns it's
//! a fire to cook over, a light, and something Douse puts out.

use gahturiyu_sim::sim::{
    campfire::{BURN_HOURS, LIGHT_SECS, TINDER_SECS},
    crafting::{Station, RECIPES},
    geo::V2,
    items,
    world::HOUR,
    worldgen, World,
};

fn wait(w: &mut World, secs: f64, step: f64) {
    let end = w.time + secs;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

/// The squad out in the open (well clear of any town), member 0 with wood.
fn in_the_wild(seed: u64, tinder: bool) -> (World, u32, V2) {
    let mut w = worldgen::generate(seed);
    let mut at = w.squad.pos;
    for k in 0..40 {
        let p = w.squad.pos.add(V2::new(150.0 + 40.0 * k as f32, 90.0));
        if w.town_at(p).is_none() && !w.terrain.is_sea(p) && w.can_stand(p, 0.3) && w.fire_at(p).is_none() {
            at = p;
            break;
        }
    }
    w.teleport_squad(at);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.add(items::id("timber"), 3);
    d.gear.bag.retain(|e| tinder || e.0 != items::id("tinderbox"));
    if tinder {
        d.gear.add(items::id("tinderbox"), 1);
    }
    (w, m, at.add(V2::new(3.0, 0.0)))
}

#[test]
fn a_campfire_is_laid_lit_and_burns_out_by_the_clock() {
    let (mut w, m, spot) = in_the_wild(1, false);
    assert!(w.can_make_fire(m));
    w.order_campfire(m, spot).unwrap();
    wait(&mut w, 20.0, 1.0);
    let f = *w.campfires.first().expect("a fire laid");
    assert!(f.lighting(w.time), "being got going");
    assert!((f.lit_at - f.since - LIGHT_SECS).abs() < 1e-6, "five minutes without a tinderbox");
    assert!(w.fire_at(f.pos.add(V2::new(1.5, 0.0))).is_none(), "nothing to cook over yet");
    wait(&mut w, LIGHT_SECS, 5.0);
    assert!(f.burning(w.time));
    assert!(w.fire_at(f.pos.add(V2::new(1.5, 0.0))).is_some(), "a fire to cook over");
    assert!(w.fixed_lights().iter().any(|l| l.pos.dist(f.pos) < 0.01), "it gives light");
    assert_eq!(f.out_at - f.lit_at, BURN_HOURS * HOUR);
    wait(&mut w, BURN_HOURS * HOUR, 60.0);
    let f = w.campfire(f.id).unwrap();
    assert!(!f.burning(w.time), "burnt out");
    assert!(w.fire_at(f.pos.add(V2::new(1.5, 0.0))).is_none());
    assert_eq!(w.count_of(m, "timber"), 2, "one timber burnt");
}

#[test]
fn a_tinderbox_gets_it_going_quicker() {
    let (mut w, m, spot) = in_the_wild(2, true);
    w.order_campfire(m, spot).unwrap();
    wait(&mut w, 20.0, 1.0);
    let f = w.campfires[0];
    assert!((f.lit_at - f.since - TINDER_SECS).abs() < 1e-6);
}

#[test]
fn when_it_burns_and_goes_out_doesnt_depend_on_the_step() {
    let run = |step: f64| {
        let (mut w, m, spot) = in_the_wild(3, false);
        w.order_campfire(m, spot).unwrap();
        // (Laid the moment they arrive; the walk is the squad's own, so it's
        // timed from there.)
        wait(&mut w, 30.0, step);
        let f = w.campfires[0];
        (f.lit_at - f.since, f.out_at - f.lit_at)
    };
    assert_eq!(run(0.5), run(7.0));
    assert_eq!(run(0.5), (LIGHT_SECS, BURN_HOURS * HOUR));
}

#[test]
fn more_wood_keeps_it_going_and_lights_it_again_once_out() {
    let (mut w, m, spot) = in_the_wild(4, true);
    w.order_campfire(m, spot).unwrap();
    wait(&mut w, 20.0 + TINDER_SECS, 1.0);
    let id = w.campfires[0].id;
    let out = w.campfire(id).unwrap().out_at;
    w.order_feed_fire(m, id).unwrap();
    wait(&mut w, 5.0, 1.0);
    assert_eq!(w.campfire(id).unwrap().out_at, out + BURN_HOURS * HOUR, "another timber's burning");
    // Let it go out, then feed it: alight again.
    wait(&mut w, 2.0 * BURN_HOURS * HOUR + 60.0, 60.0);
    assert!(!w.campfire(id).unwrap().burning(w.time));
    w.order_feed_fire(m, id).unwrap();
    wait(&mut w, 5.0 + TINDER_SECS, 1.0);
    assert!(w.campfire(id).unwrap().burning(w.time), "lit again");
}

#[test]
fn the_squad_cooks_over_its_own_campfire() {
    let (mut w, m, spot) = in_the_wild(5, true);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("raw_meat"), 1);
    w.people[m as usize].stats.set_skill(gahturiyu_sim::sim::stats::Skill::Cooking, 60.0);
    w.order_campfire(m, spot).unwrap();
    wait(&mut w, 20.0 + TINDER_SECS, 1.0);
    let b = w.bench_for(m).expect("the fire is a bench");
    assert_eq!(b.station, Station::Campfire);
    let ri = RECIPES.iter().position(|r| r.output == "roast_meat").unwrap();
    w.make(m, ri, 1).unwrap();
    wait(&mut w, 20.0 * 60.0, 5.0);
    let roast = items::id("roast_meat");
    let n: u16 = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| items::info(e.0).form == roast).map(|e| e.1).sum();
    assert_eq!(n, 1, "roast meat");
}

#[test]
fn no_campfire_indoors_or_without_wood() {
    let (mut w, m, spot) = in_the_wild(6, false);
    let other = w.squad.members[1];
    w.people[other as usize].detail.as_mut().unwrap().gear.bag.retain(|e| e.0 != items::id("timber"));
    assert!(!w.can_make_fire(other));
    assert!(w.order_campfire(other, spot).is_err());
    let d = (0..w.settlements[0].buildings.len() as u16).find_map(|b| gahturiyu_sim::sim::buildings::door_of(&w.settlements[0], b)).unwrap();
    assert!(w.order_campfire(m, d.centre).is_err(), "not indoors");
}
