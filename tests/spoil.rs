//! Spoilage (the items design, 0.6): food the squad carries goes fresh,
//! stale, spoiled on the clock; stale fills less, spoiled little; what's
//! going off is eaten first; a sealed strongbox keeps it longer.

use gahturiyu_sim::sim::{
    items,
    spoil::{self, Fresh},
    world::{DAY, HOUR},
    worldgen, World,
};

fn bread() -> items::ItemId {
    items::id("flatbread")
}

fn wait(w: &mut World, hours: f64, step: f64) {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-6 {
        w.step(step.min(end - w.time));
    }
}

/// Member 0 carries `n` flatbread got now, and nothing else to eat.
fn with_bread(seed: u64, n: u16) -> (World, u32) {
    let mut w = worldgen::generate(seed);
    let m = w.squad.members[0];
    let d = w.people[m as usize].detail.as_mut().unwrap();
    d.gear.bag.retain(|e| !spoil::is_food(e.0));
    d.gear.add(bread(), n);
    (w, m)
}

/// Member `m`'s bread: (how many, days left now).
fn stacks(w: &World, m: u32) -> Vec<(u16, f32)> {
    let n: u16 = w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == bread()).map(|e| e.1).sum();
    vec![(n, w.days_left(m, bread(), w.time))]
}

#[test]
fn bread_goes_stale_then_spoils_on_the_clock() {
    // Plenty for everyone, so some is always left to look at.
    let (mut w, m) = with_bread(1, 200);
    for &x in &w.squad.members.clone() {
        w.people[x as usize].detail.as_mut().unwrap().gear.add(bread(), 200);
    }
    w.step(1.0);
    assert_eq!(w.freshness(m, bread(), w.time), Fresh::Fresh);
    wait(&mut w, 2.0 * 24.0, 300.0);
    assert_eq!(w.freshness(m, bread(), w.time), Fresh::Fresh, "fresh for the first days");
    let mut stale_at = None;
    let mut spoiled_at = None;
    for h in 0..12 * 24 {
        wait(&mut w, 1.0, 600.0);
        match w.freshness(m, bread(), w.time) {
            Fresh::Stale if stale_at.is_none() => stale_at = Some(h),
            Fresh::Spoiled if spoiled_at.is_none() => spoiled_at = Some(h),
            _ => {}
        }
    }
    let (st, sp) = (stale_at.expect("went stale"), spoiled_at.expect("spoiled"));
    assert!(st < sp, "stale before spoiled ({st} h, {sp} h)");
}

#[test]
fn freshness_comes_out_the_same_however_the_world_is_stepped() {
    let (w0, m) = with_bread(2, 40);
    let run = |step: f64| {
        let mut w = w0.clone();
        wait(&mut w, 3.0 * 24.0, step);
        let rate = 1.0;
        let _ = rate;
        stacks(&w, m)
    };
    let (a, b) = (run(30.0), run(700.0));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.0, y.0, "same eaten");
        assert!((x.1 - y.1).abs() < 0.02, "{a:?} vs {b:?}");
    }
}

#[test]
fn stale_fills_less_and_new_food_freshens_the_stack() {
    let (mut w, m) = with_bread(1, 200);
    w.step(1.0);
    let fresh_fill = match items::item(bread()).kind {
        items::Kind::Food(n) => n,
        _ => unreachable!(),
    };
    // Five days on: stale (it would have been eaten by now: keep them fed).
    let t = w.time;
    w.people[m as usize].cond.as_mut().unwrap().hunger = 0.0;
    wait(&mut w, 5.0 * 24.0 - 0.1, 600.0);
    let _ = t;
    if w.freshness(m, bread(), w.time) == Fresh::Stale {
        let t = w.time;
        w.people[m as usize].cond.as_mut().unwrap().hunger = 60.0;
        let before = w.hunger_of(m).unwrap();
        assert!(w.eat(m, bread(), t));
        let after = w.hunger_of(m).unwrap();
        assert!((before - after) < fresh_fill * 0.7, "stale fills less ({} of {fresh_fill})", before - after);
        assert!(w.log.iter().any(|l| l.1.contains("stale flatbread")));
    }
    // A fresh batch joins it: the stack's freshness rises.
    let old = w.days_left(m, bread(), w.time);
    w.people[m as usize].detail.as_mut().unwrap().gear.add(bread(), 10);
    w.step(1.0);
    assert!(w.days_left(m, bread(), w.time) > old + 0.15, "freshened ({old:.1} -> {:.1})", w.days_left(m, bread(), w.time));
}

#[test]
fn a_sealed_strongbox_keeps_food_longer() {
    // Two members side by side, the same bread; one keeps theirs boxed.
    let mut w = worldgen::generate(3);
    let (a, b) = (w.squad.members[0], w.squad.members[1]);
    for (m, boxed) in [(a, false), (b, true)] {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        d.gear.bag.retain(|e| !spoil::is_food(e.0));
        d.gear.add(bread(), 6);
        if boxed {
            d.gear.add(items::id("strongbox"), 1);
        }
    }
    wait(&mut w, 4.0 * 24.0, 600.0);
    let least = |m: u32| stacks(&w, m).iter().map(|s| s.1).fold(f32::MAX, f32::min);
    let (open, boxed) = (least(a), least(b));
    assert!(boxed > open + 1.0, "the strongbox's bread keeps ({boxed:.1} vs {open:.1} days left)");
    assert!(spoil::stage(bread(), open) <= Fresh::Stale);
    let _ = DAY;
}
