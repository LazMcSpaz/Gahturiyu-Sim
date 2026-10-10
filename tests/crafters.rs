//! Crafters keep their posts for trades that can't be idle in town:
//! carpenters and masons (no town recipes yet — they build for bases) and
//! stone tenders (their rounds are their work); rest days aren't idleness.
//! The idle rule still bites everyone else.

use gahturiyu_sim::sim::{
    jobs::Job,
    lives::Loss,
    world::{DAY, HOUR},
    worldgen, World,
};
use std::collections::BTreeMap;

/// Posts lost to idleness over `days`, by trade.
fn idle_losses(w: &mut World, days: f64) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    let mut jobs: Vec<Job> = w.society.lives.iter().map(|l| l.job).collect();
    for _ in 0..(days * DAY / HOUR).round() as usize {
        w.step(HOUR);
        for (p, l) in w.society.lives.iter().enumerate() {
            let was = jobs[p];
            if l.job != was && w.society.minds[p].lost == Some((was, Loss::Idle)) {
                *out.entry(format!("{was:?}")).or_insert(0) += 1;
            }
            jobs[p] = l.job;
        }
    }
    out
}

#[test]
fn carpenters_masons_and_tenders_are_never_idle() {
    let runs: Vec<_> = [1, 2]
        .map(|seed| {
            let mut w = worldgen::generate(seed);
            let lost = idle_losses(&mut w, 10.0);
            println!("seed {seed}: {} idle losses in 10 days {lost:?}", lost.values().sum::<usize>());
            (seed, lost)
        })
        .into();
    for (seed, lost) in runs {
        let total: usize = lost.values().sum();
        for trade in ["Carpenter", "Mason", "StoneTender"] {
            assert_eq!(lost.get(trade), None, "seed {seed}: {trade}s lose posts to idleness: {lost:?}");
        }
        // Before the fix: 1272 (seed 1) and 1230 (seed 2), half of them
        // carpenters, masons and tenders, plus crafters fired for resting.
        assert!(total < 700, "seed {seed}: idle losses drop sharply: {lost:?}");
    }
}

#[test]
fn a_smith_with_nothing_to_work_still_loses_the_post() {
    let mut w = worldgen::generate(1);
    let p = (0..w.society.lives.len())
        .find(|&p| w.society.lives[p].job == Job::Smith && !w.people[p].dead && !w.people[p].in_squad && w.people[p].home.is_some())
        .expect("a smith") as u32;
    let town = w.people[p as usize].home.unwrap() as usize;
    // An empty store, and nothing in hand: no work to be had.
    for _ in 0..5 * 24 {
        let tl = &mut w.society.towns[town];
        for s in tl.stock.iter_mut() {
            s.base = 0.0;
        }
        tl.making.retain(|m| m.who != p);
        w.step(HOUR);
        if w.society.lives[p as usize].job != Job::Smith {
            break;
        }
    }
    assert_eq!(w.mind(p).lost, Some((Job::Smith, Loss::Idle)), "an idle smith loses the post");
}
