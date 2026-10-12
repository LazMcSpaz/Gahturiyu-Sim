//! Cold and wet on the body (0.3, `docs/weather-hooks.md`): the weather's
//! exposure against what's worn and a Warmth draught; roofs, tents and fires
//! keep it out; it builds on the condition timeline, the same however the
//! world is stepped, and it never kills.

use gahturiyu_sim::sim::{
    condition::{CHILLED, FREEZING},
    effects::{lasting, Does, Reach},
    geo::{is_land, V2, WORLD_SIZE},
    items::{self, Slot},
    person::PersonId,
    world::HOUR,
    worldgen, World,
};

/// A spot and a time out in bitter weather (exposure well up for hours),
/// and the world run on to just before it, the squad standing there.
fn out_in_the_cold(seed: u64) -> (World, PersonId) {
    let mut w = worldgen::generate(seed);
    let start = w.time;
    let mut best: Option<(V2, f64, f32)> = None;
    for i in 1..14 {
        for j in 1..14 {
            let p = V2::new(i as f32 / 14.0 * WORLD_SIZE, j as f32 / 14.0 * WORLD_SIZE);
            if !is_land(p) || w.terrain.is_sea(p) {
                continue;
            }
            for h in 0..(24 * 2) / 3 {
                let t = ((start / HOUR).ceil() + (h * 3) as f64) * HOUR;
                // Bitter for three hours on end.
                let e = (0..3).map(|k| w.weather_effects(p, t + k as f64 * HOUR).exposure).fold(1.0f32, f32::min);
                if best.is_none_or(|b| e > b.2) {
                    best = Some((p, t, e));
                }
            }
        }
    }
    let (p, t, e) = best.unwrap();
    assert!(e >= 0.5, "somewhere bitter enough ({e})");
    while w.time < t - 60.0 {
        w.step(600.0f64.min(t - 60.0 - w.time));
    }
    w.teleport_squad(p);
    w.step(30.0);
    let m = w.squad.members[0];
    (w, m)
}

fn cold(w: &World, m: PersonId) -> f32 {
    w.cold_of(m).unwrap()
}

fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

fn strip(w: &mut World, m: PersonId) {
    let d = w.people[m as usize].detail.as_mut().unwrap();
    for s in [Slot::Cloak, Slot::BodyOver, Slot::BodyUnder, Slot::Head, Slot::Hands, Slot::LegsUnder, Slot::LegsOver, Slot::Feet] {
        d.gear.discard(s);
    }
}

fn wear(w: &mut World, m: PersonId, keys: &[&str]) {
    for k in keys {
        let id = items::id(k);
        w.people[m as usize].detail.as_mut().unwrap().gear.add(id, 1);
        assert!(w.equip(m, id), "put on the {k}");
    }
}

#[test]
fn the_bare_get_cold_and_the_well_wrapped_less_so() {
    let (mut w0, m) = out_in_the_cold(1);
    let other = w0.squad.members[1];
    strip(&mut w0, m);
    strip(&mut w0, other);
    wear(&mut w0, other, &["fur_mantle", "wool_cap", "banded_tunic", "trousers"]);
    let mut w = w0.clone();
    wait(&mut w, 2.0, 30.0);
    let (bare, wrapped) = (cold(&w, m), cold(&w, other));
    assert!(bare > CHILLED, "out in it with nothing on ({bare})");
    assert!(wrapped < bare * 0.75, "wrapped up: {wrapped} vs {bare}");
    // A Warmth draught helps too.
    let mut w = w0.clone();
    assert!(w.apply_effect(m, &lasting(Does::Warmth, 1.0, 3.0 * 3600.0, Reach::Caster), 1.0));
    wait(&mut w, 2.0, 30.0);
    assert!(cold(&w, m) < bare, "warmed by a draught ({} vs {bare})", cold(&w, m));
}

#[test]
fn a_fire_keeps_it_off_and_it_never_kills() {
    let (mut w, m) = out_in_the_cold(1);
    strip(&mut w, m);
    wait(&mut w, 3.0, 60.0);
    let c = cold(&w, m);
    assert!(c > CHILLED);
    // Left out long enough: they collapse, but live.
    w.people[m as usize].cond.as_mut().unwrap().cold = FREEZING + 2.0;
    wait(&mut w, 2.0, 60.0);
    assert!(w.is_down(m) || cold(&w, m) < FREEZING, "collapsed with the cold");
    assert!(!w.people[m as usize].dead, "the cold never kills");
    // By a fire it eases off.
    let (mut w, m) = out_in_the_cold(1);
    strip(&mut w, m);
    wait(&mut w, 2.0, 60.0);
    let before = cold(&w, m);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("timber"), 3);
    let k = w.squad.index(m).unwrap();
    let at = w.squad.at[k].add(V2::new(2.0, 0.0));
    w.order_campfire(m, at).expect("a fire laid");
    wait(&mut w, 1.5, 30.0);
    assert!(w.campfires.iter().any(|f| f.burning(w.time)), "it's burning");
    assert!(cold(&w, m) < before, "warmer by the fire ({before} then {})", cold(&w, m));
}

#[test]
fn cold_comes_out_the_same_however_the_world_is_stepped() {
    let (mut w0, m) = out_in_the_cold(2);
    strip(&mut w0, m);
    let run = |step: f64| {
        let mut w = w0.clone();
        wait(&mut w, 5.0, step);
        cold(&w, m)
    };
    let (a, b) = (run(10.0), run(300.0));
    assert!((a - b).abs() < 0.1, "{a} vs {b}");
}
