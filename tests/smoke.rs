//! Smoking (the items design, Part 5 rule 7 and "Smokes"): haze on the
//! condition timeline, a habit's cravings by the clock.

use gahturiyu_sim::sim::{
    haze::{self, Haze, CRAVE_AFTER, CRAVE_HARD, HABIT_FADES, HAZE_DRAIN},
    items,
    stats::Attr,
    world::{DAY, HOUR},
    worldgen, World,
};

fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

fn with(seed: u64, key: &str, n: u16, pipe: bool) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| e.0 != items::id("pipe"));
    d.gear.add(items::id(key), n);
    if pipe {
        d.gear.add(items::id("pipe"), 1);
    }
    w.step(1.0);
    (w, m)
}

#[test]
fn a_pipe_blend_wants_a_pipe_and_incense_doesnt() {
    let (mut w, m) = with(1, "drift_smoke", 1, false);
    assert!(w.why_cant_smoke(m, items::id("drift_smoke")).is_some());
    assert!(!w.smoke(m, items::id("drift_smoke")));
    let (mut w, m) = with(1, "hall_incense", 1, false);
    assert!(w.smoke(m, items::id("hall_incense")));
    assert!(w.haze_of(m) > 10.0);
}

#[test]
fn haze_rises_with_each_smoke_and_drains_over_hours() {
    let (mut w, m) = with(2, "elders_blend", 2, true);
    assert!(w.smoke(m, items::id("elders_blend")));
    assert_eq!(Haze::of(w.haze_of(m)), Haze::Mellow);
    assert!(w.smoke(m, items::id("elders_blend")));
    assert_eq!(Haze::of(w.haze_of(m)), Haze::Hazy);
    let peak = w.haze_of(m);
    let base = w.people[m as usize].effective_stats().attr(Attr::Agility);
    wait(&mut w, 1.0, 60.0);
    assert!((peak - w.haze_of(m) - HAZE_DRAIN).abs() < 0.5);
    // Clear in a few hours, and said so.
    wait(&mut w, (peak / HAZE_DRAIN) as f64, 300.0);
    assert_eq!(w.haze_of(m), 0.0);
    assert!(w.log.iter().any(|l| l.1.contains("head has cleared")));
    assert!(w.people[m as usize].effective_stats().attr(Attr::Agility) > base, "hazy was clumsier");
}

#[test]
fn a_stupor_dozes_them_off() {
    let (mut w, m) = with(3, "ghost_incense", 3, false);
    for _ in 0..3 {
        w.smoke(m, items::id("ghost_incense"));
    }
    assert_eq!(Haze::of(w.haze_of(m)), Haze::Stupor);
    let k = w.squad.index(m).unwrap();
    assert!(w.squad.resting[k], "dozed off");
}

#[test]
fn a_habit_craves_by_the_clock_and_fades() {
    let (mut w, m) = with(4, "drift_smoke", 1, true);
    assert!(w.smoke(m, items::id("drift_smoke")));
    // Taken up (by the dice or not, for the test it is).
    w.people[m as usize].cond.as_mut().unwrap().habit = true;
    let base = w.people[m as usize].effective_stats().attr(Attr::Intellect);
    assert_eq!(w.craving_of(m), 0);
    wait(&mut w, CRAVE_AFTER + 0.5, 600.0);
    assert_eq!(w.craving_of(m), 1, "craving after a day");
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert_eq!(c.craving, 1, "settled on the timeline");
    assert!(w.people[m as usize].effective_stats().attr(Attr::Intellect) < base, "focus worse");
    wait(&mut w, CRAVE_HARD - CRAVE_AFTER, 600.0);
    assert_eq!(w.craving_of(m), 2, "worse after two");
    wait(&mut w, HABIT_FADES * DAY / HOUR - CRAVE_HARD, 1800.0);
    assert_eq!(w.craving_of(m), 0);
    assert!(!w.people[m as usize].cond.as_ref().unwrap().habit, "the habit has faded");
}

#[test]
fn some_take_to_it_more_than_others_and_it_is_keyed() {
    let w = worldgen::generate(5);
    let s: Vec<f32> = (0..200).map(|p| haze::susceptibility(w.seed, p)).collect();
    assert!(s.iter().all(|&x| (0.2..=1.8).contains(&x)));
    assert!(s.iter().cloned().fold(0.0, f32::max) > 1.4 && s.iter().cloned().fold(9.0, f32::min) < 0.6);
    assert_eq!(haze::susceptibility(w.seed, 7), haze::susceptibility(w.seed, 7));
    // Smoking long enough, a habit comes for most.
    let mut took = 0;
    for seed in 0..6 {
        let (mut w, m) = with(10 + seed, "hall_incense", 120, false);
        for _ in 0..120 {
            w.smoke(m, items::id("hall_incense"));
            wait(&mut w, 2.0, 1800.0);
            if w.people[m as usize].cond.as_ref().unwrap().habit {
                took += 1;
                break;
            }
        }
    }
    assert!(took >= 3, "{took} of 6 took up the habit");
}

#[test]
fn smokes_are_sold_but_not_the_rite_incense() {
    assert!(gahturiyu_sim::sim::jobs::good_of("drift_smoke").is_some());
    assert!(gahturiyu_sim::sim::jobs::good_of("sun_incense").is_none());
}
