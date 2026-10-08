//! Run the world with no window and print what it is doing.
//!
//!     cargo run --release --bin headless -- [days] [seed]
//!
//! With `GAHT_SAVE=path`, the world is saved there at the end (the window
//! can start from it with `GAHT_LOAD=path`).

use std::time::Instant;

use gahturiyu_sim::sim::{race::ALL_RACES, world::HOUR, worldgen};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|a| a == "fight").unwrap_or(false) {
        fight(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3), args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1));
        return;
    }
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
    if let Ok(p) = std::env::var("GAHT_SAVE") {
        match w.save_to(std::path::Path::new(&p)) {
            Ok(()) => println!("saved to {p}"),
            Err(e) => println!("couldn't save to {p}: {e}"),
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

/// Spawn bandits next to the squad and print the fight blow by blow.
fn fight(count: usize, seed: u64) {
    use gahturiyu_sim::sim::geo::V2;
    let mut w = worldgen::generate(seed);
    for &m in &w.squad.members.clone() {
        let p = &w.people[m as usize];
        let d = p.detail.as_ref().unwrap();
        println!(
            "SQUAD {:<10} {:<7} {:<8} might {:>5.1}  {:<12} spells {:?}",
            d.name,
            p.race.name(),
            p.stats.calling.name(),
            p.might,
            d.gear.weapon_name(),
            d.spells
        );
    }
    let at = w.squad.pos.add(V2::new(20.0, 0.0));
    let g = w.spawn_bandits(at, count, true);
    for &m in &w.group(g).unwrap().members.clone() {
        let p = &w.people[m as usize];
        let d = p.detail.as_ref().unwrap();
        println!("BANDIT {:<10} {:<7} {:<8} might {:>5.1}  {:<12} spells {:?}", d.name, p.race.name(), p.stats.calling.name(), p.might, d.gear.weapon_name(), d.spells);
    }
    let mut printed = 0;
    for _ in 0..20000 {
        w.step(0.1);
        if let Some(b) = w.battles.first() {
            for (t, l) in &b.log[printed.min(b.log.len())..] {
                println!("  {:>6.1}s  {}", t - b.start, l);
            }
            printed = b.log.len();
        } else if printed > 0 {
            break;
        }
    }
    for (t, l) in w.log.iter().take(4) {
        println!("{} {}", t, l);
    }
    for &m in &w.squad.members.clone() {
        let p = &w.people[m as usize];
        let hp = p.wounds.hp_at(&p.stats, w.time);
        println!("{:<10} hp {:?} mana {:.0}", p.name().unwrap(), hp.map(|h| h.round()), p.mana_at(w.time));
    }
}
