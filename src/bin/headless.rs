//! Run the world with no window and print what it is doing.
//!
//!     cargo run --release --bin headless -- [days] [seed]
//!     cargo run --release --bin headless -- society [days] [seed]   (each town's customs, jobs, food and money)
//!
//! With `GAHT_SAVE=path`, the world is saved there at the end (the window
//! can start from it with `GAHT_LOAD=path`).

use std::time::Instant;

use gahturiyu_sim::sim::{race::ALL_RACES, world::HOUR, worldgen};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|a| a == "society").unwrap_or(false) {
        society(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3.0), args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1));
        return;
    }
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
            d.spells.iter().map(|s| s.def().name).collect::<Vec<_>>()
        );
    }
    let at = w.squad.pos.add(V2::new(20.0, 0.0));
    let g = w.spawn_bandits(at, count, true);
    for &m in &w.group(g).unwrap().members.clone() {
        let p = &w.people[m as usize];
        let d = p.detail.as_ref().unwrap();
        println!("BANDIT {:<10} {:<7} {:<8} might {:>5.1}  {:<12} spells {:?}", d.name, p.race.name(), p.stats.calling.name(), p.might, d.gear.weapon_name(), d.spells.iter().map(|s| s.def().name).collect::<Vec<_>>());
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

/// Each town's customs, work, food and money, after some days.
fn society(days: f64, seed: u64) {
    use gahturiyu_sim::sim::jobs::{Job, GOODS};
    use std::collections::BTreeMap;
    let t0 = Instant::now();
    let mut w = worldgen::generate(seed);
    println!("world {seed} built in {:.1} ms", t0.elapsed().as_secs_f64() * 1000.0);
    let t1 = Instant::now();
    w.step(days * 24.0 * HOUR);
    println!("{days} days run in {:.0} ms; {} caravans set out\n", t1.elapsed().as_secs_f64() * 1000.0, w.stats.caravans);
    for (ti, s) in w.settlements.iter().enumerate() {
        let tl = &w.society.towns[ti];
        println!("{} ({} people, {}{})", s.name, s.residents.len(), if s.coastal { "coast" } else { "inland" }, if tl.on_road { format!(", {} roads meet", tl.roads) } else { format!(", {} roads", tl.roads) });
        for ci in std::iter::once(tl.shore).chain(tl.stilts) {
            let c = &w.society.communities[ci as usize];
            let sh = c.blend.share;
            println!(
                "  {:<6} R{:.0}% Q{:.0}% H{:.0}% T{:.0}%  | {:?} · {:?} · {:?} · {:?}{}",
                if c.stilts { "stilts" } else { "land" },
                sh[0] * 100.0,
                sh[1] * 100.0,
                sh[2] * 100.0,
                sh[3] * 100.0,
                c.customs.cooking,
                c.customs.rhythm,
                c.customs.belonging,
                c.customs.layout,
                c.customs.institutions().map(|(_, i)| format!(" + {}", i.name())).collect::<String>()
            );
            let f = &c.food;
            println!(
                "         food {:.0}%  (gardens {:.0}% of {:.0}%, kitchens {:.0}% of {:.0}%, boats {:.0}% of {:.0}%)",
                f.overall * 100.0,
                f.suff[0] * 100.0,
                f.share[0] * 100.0,
                f.suff[1] * 100.0,
                f.share[1] * 100.0,
                f.suff[2] * 100.0,
                f.share[2] * 100.0
            );
            let mut jobs: BTreeMap<Job, usize> = BTreeMap::new();
            for p in w.members_of(ci) {
                *jobs.entry(w.life(p).job).or_default() += 1;
            }
            println!("         {}", jobs.iter().map(|(j, n)| format!("{} {n}", j.name())).collect::<Vec<_>>().join(", "));
        }
        println!("  places: {}", tl.places.iter().map(|p| p.kind.name()).collect::<Vec<_>>().join(", "));
        println!(
            "  stock: {}  | prosperity {:.2}, purse {:.0}/{:.0}, treasury {:.0} (owed {:.0}), landed {:.0}",
            GOODS.iter().filter(|g| w.stock_now(ti as u16, **g).abs() >= 0.5).map(|g| format!("{} {:.0}", g.name(), w.stock_now(ti as u16, *g))).collect::<Vec<_>>().join(", "),
            tl.prosperity,
            w.purse_now(ti as u16),
            tl.purse_cap,
            tl.treasury,
            tl.owed,
            tl.landed
        );
        println!(
            "  land offers: {}",
            GOODS.iter().filter(|g| tl.sources[g.index()] > 0.05 && tl.sources[g.index()] < 1.0).map(|g| format!("{} {:.2}", g.name(), tl.sources[g.index()])).collect::<Vec<_>>().join(", ")
        );
        let mut shelf: BTreeMap<String, usize> = BTreeMap::new();
        for s in &tl.shelf {
            *shelf.entry(gahturiyu_sim::sim::items::item(s.item).name.to_string()).or_default() += 1;
        }
        println!("  shelf ({}): {}", tl.shelf.len(), shelf.iter().map(|(k, n)| format!("{k} ×{n}")).collect::<Vec<_>>().join(", "));
        println!("  dearest: {}", w.dearest(ti as u16, 6).iter().map(|(g, f)| format!("{} ×{:.1}", g.name(), f)).collect::<Vec<_>>().join(", "));
        {
            use gahturiyu_sim::sim::lives::Work;
            let folk: Vec<u32> = s.residents.iter().copied().filter(|&p| !w.people[p as usize].dead).collect();
            let count = |k: Work| folk.iter().filter(|&&p| w.mind(p).work == k).count();
            let hhs: Vec<usize> = (0..w.society.households.len()).filter(|&h| w.society.communities[w.society.households[h].community as usize].town == ti as u16).collect();
            let indebt = hhs.iter().filter(|&&h| w.society.households[h].purse.debt() > 0.0).count();
            let mean = |k: usize| folk.iter().map(|&p| w.mind(p).needs[k]).sum::<f32>() / folk.len().max(1) as f32;
            let coin: f32 = hhs.iter().map(|&h| w.society.households[h].purse.coin).sum::<f32>() / hhs.len().max(1) as f32;
            println!(
                "  lives: working {} injured {} away {} jobless {} bound {} retired {} | households {} ({} in debt, mean purse {:.0}) | needs money {:.2} hunger {:.2} safety {:.2} grievance {:.2} ambition {:.2}",
                count(Work::Working), count(Work::Injured), count(Work::Away), count(Work::Jobless), count(Work::Bonded), count(Work::Retired),
                hhs.len(), indebt, coin, mean(0), mean(1), mean(2), mean(3), mean(4)
            );
        }
        let g = w.government(ti as u16);
        let shore = &w.society.communities[tl.shore as usize].customs;
        println!(
            "  rule: {} | {} | unrest {:.0}, revolts {}, rite {:?}, justice {:?}, bondage {:?}, bonds here {}",
            w.gov_words(ti as u16),
            w.rulers_words(ti as u16),
            g.unrest,
            g.revolts,
            g.last_rite,
            shore.justice,
            shore.slavery,
            w.bonds.iter().filter(|b| b.town == ti as u16).map(|b| if b.slave { "slave" } else { "term" }).collect::<Vec<_>>().join(" ")
        );
    }
}
