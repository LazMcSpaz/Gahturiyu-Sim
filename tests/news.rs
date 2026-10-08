//! News of a bounty travels with people, town to town, at walking pace.

use gahturiyu_sim::sim::{world::HOUR, worldgen, World};

fn with_bounty(seed: u64) -> (World, u16) {
    let mut w = worldgen::generate(seed);
    // A crime seen in the squad's own town.
    let here = w.squad.pos;
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    w.bounty.insert(town, 40.0);
    let t = w.time;
    w.news.insert((town, town), t);
    (w, town)
}

fn run(w: &mut World, hours: f64, step: f64) {
    let n = (hours * HOUR / step).round() as usize;
    for _ in 0..n {
        w.step(step);
    }
}

#[test]
fn news_spreads_over_days_not_at_once() {
    let (mut w, town) = with_bounty(1);
    let start = w.time;
    assert_eq!(w.towns_heard(town), 1, "only the town where it happened knows");
    run(&mut w, 1.0, 60.0);
    assert!(w.towns_heard(town) <= 2, "an hour on, hardly anywhere: {}", w.towns_heard(town));
    run(&mut w, 11.0, 60.0);
    let heard = w.towns_heard(town);
    assert!(heard < w.settlements.len() / 2, "half a day on, most towns haven't heard: {heard}");
    run(&mut w, 36.0, 60.0);
    let heard = w.towns_heard(town);
    assert!(heard >= 3, "two days on it has travelled: {heard} towns");
    // Every other town heard it some time after the crime, at someone's arrival.
    for s in &w.settlements {
        if s.id != town {
            if let Some(t) = w.heard_at(s.id, town) {
                assert!(t > start + 600.0, "{} heard after a walk, not instantly", s.name);
            }
        }
    }
    // Hearing lowers disposition there.
    let far = w.settlements.iter().find(|s| s.id != town && w.has_heard(s.id, town)).unwrap().id;
    assert!(w.bounty_known_in(far) > 0.0);
}

#[test]
fn news_spreads_the_same_however_the_world_is_stepped() {
    let spread = |step: f64| {
        let (mut w, town) = with_bounty(1);
        run(&mut w, 36.0, step);
        let mut v: Vec<(u16, f64)> = w.news.iter().filter(|(k, _)| k.1 == town).map(|(k, t)| (k.0, *t)).collect();
        v.sort_by_key(|x| x.0);
        v
    };
    let fine = spread(30.0);
    let coarse = spread(3600.0);
    assert!(fine.len() > 1);
    assert_eq!(fine, coarse);
}

#[test]
fn a_town_that_has_not_heard_holds_nothing_against_you() {
    let (mut w, town) = with_bounty(1);
    run(&mut w, 2.0, 60.0);
    let unaware = w.settlements.iter().find(|s| !w.has_heard(s.id, town)).unwrap().id;
    assert_eq!(w.bounty_known_in(unaware), 0.0);
    assert!(w.bounty_known_in(town) > 0.0);
}
