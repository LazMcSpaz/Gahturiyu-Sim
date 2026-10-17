use gahturiyu_sim::sim::{prof, worldgen};
use std::time::Instant;
//! What a step of the world costs per 60 Hz frame over a game day at 60×,
//! and which sections spike (`sim::prof`). Run by hand when looking at
//! performance: `cargo test --release --test perf -- --ignored --nocapture`.
//! (17 Oct: mean 0.16 ms a frame; on the hour 15-30 ms, dawn 65 ms, after
//! banding townsfolk by a rough position and keeping today's weather.)
#[test]
#[ignore]
fn a_day_at_60x_by_section() {
    let mut w = worldgen::generate(1);
    let big = w.settlements.iter().max_by_key(|s| s.residents.len()).unwrap().pos;
    w.teleport_squad(big);
    prof::enable(true);
    prof::take();
    let end = w.time + 26.0 * 3600.0;
    let mut spikes: Vec<(f64, f64, Vec<(&'static str, f64, u64)>)> = Vec::new();
    let mut all: std::collections::BTreeMap<&'static str, f64> = Default::default();
    while w.time < end {
        let f = Instant::now();
        let mut left = 1.0;
        let sub = (left / 600.0f64).max(1.0);
        while left > 1e-9 { let s = left.min(sub); w.step(s); left -= s; }
        let ms = f.elapsed().as_secs_f64() * 1000.0;
        let secs = prof::take();
        for (k, v, _) in &secs { *all.entry(k).or_default() += v; }
        if ms > 8.0 { spikes.push((w.time / 3600.0, ms, secs)); }
    }
    let mut tot: Vec<_> = all.into_iter().collect(); tot.sort_by(|a, b| b.1.total_cmp(&a.1));
    eprintln!("PERF whole day by section: {}", tot.iter().take(12).map(|(k, v)| format!("{k} {v:.0}")).collect::<Vec<_>>().join(", "));
    spikes.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (h, ms, secs) in spikes.iter().take(6) { eprintln!("PERF {:.1} ms at hour {:.2}: {}", ms, h, secs.iter().take(14).map(|(k, v, n)| format!("{k} {v:.1}/{n}")).collect::<Vec<_>>().join(", ")); }
}
