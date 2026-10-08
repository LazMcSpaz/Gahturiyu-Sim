//! Run the world with no window and print what it is doing.
//!
//!     cargo run --release --bin headless -- [days] [seed]

use std::time::Instant;

use gahturiyu_sim::sim::{race::ALL_RACES, world::HOUR, worldgen};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let t0 = Instant::now();
    let mut w = worldgen::generate(seed);
    println!("world {seed} built in {:.1} ms", t0.elapsed().as_secs_f64() * 1000.0);
    println!("{} settlements, {} people", w.settlements.len(), w.people.len());
    for s in &w.settlements {
        let mut by = [0usize; 4];
        for &p in &s.residents {
            by[w.people[p as usize].race.index()] += 1;
        }
        println!(
            "  {:<14} {:<7} {:>4} people  {}",
            s.name,
            if s.coastal { "coast" } else { "inland" },
            s.residents.len(),
            ALL_RACES.iter().zip(by).map(|(r, n)| format!("{} {n}", r.name())).collect::<Vec<_>>().join(" · ")
        );
    }

    let step = 1.0; // one game second, like the window at full detail
    let mut worst = 0.0f64;
    let t1 = Instant::now();
    let steps = (days * 24.0 * HOUR / step) as usize;
    for i in 0..steps {
        let s = Instant::now();
        w.step(step);
        worst = worst.max(s.elapsed().as_secs_f64());
        if i % (6 * 3600) == 0 {
            println!(
                "{}  on the road {:>4}  groups b1/b2/b3 {:>2}/{:>3}/{:>4}  detailed {:>4}",
                w.clock(),
                w.on_road(),
                w.stats.in_band[1],
                w.stats.in_band[2],
                w.stats.in_band[3],
                w.stats.detailed
            );
        }
    }
    let el = t1.elapsed().as_secs_f64();
    println!(
        "{days} game days in {:.2} s real ({:.1} µs per step, worst {:.2} ms); {} journeys",
        el,
        el / steps as f64 * 1e6,
        worst * 1000.0,
        w.stats.journeys_started
    );
}
