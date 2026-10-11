//! Run the world with no window and print what it is doing.
//!
//!     cargo run --release --bin headless -- [days] [seed]
//!     cargo run --release --bin headless -- society [days] [seed]   (each town's customs, jobs, food and money)
//!     cargo run --release --bin headless -- trade [days] [seed]     (which goods bought in one town sell for more in another)
//!     cargo run --release --bin headless -- weather [years] [seed]  (each region's weather added up over the years)
//!     cargo run --release --bin headless -- omens [years] [seed]    (how often each omen comes, and the first of each)
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
    if args.get(1).map(|a| a == "weather").unwrap_or(false) {
        let seed = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
        print!("{}", gahturiyu_sim::sim::weather::report::regions(&gahturiyu_sim::sim::terrain::Terrain::generate(seed)));
        let years = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
        print!("{}", gahturiyu_sim::sim::weather::report::yearly(seed, years));
        let w = world(seed);
        print!("{}", gahturiyu_sim::sim::weather::report::omen_table(&w.terrain, &w.landmarks(), seed, years.max(40)));
        return;
    }
    if args.get(1).map(|a| a == "omens").unwrap_or(false) {
        let seed = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
        let w = world(seed);
        print!("{}", gahturiyu_sim::sim::weather::report::omen_table(&w.terrain, &w.landmarks(), seed, args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100)));
        return;
    }
    if args.get(1).map(|a| a == "trade").unwrap_or(false) {
        trade(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5.0), args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1));
        return;
    }
    if args.get(1).map(|a| a == "fight").unwrap_or(false) {
        fight(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3), args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1));
        return;
    }
    let days: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let t0 = Instant::now();
    let mut w = world(seed);
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
    let mut w = world(seed);
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
    let mut w = world(seed);
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
            let mean = |k: usize| folk.iter().map(|&p| w.mind(p).needs()[k]).sum::<f32>() / folk.len().max(1) as f32;
            let coin: f32 = hhs.iter().map(|&h| w.society.households[h].purse.coin).sum::<f32>() / hhs.len().max(1) as f32;
            println!(
                "  lives: working {} injured {} away {} jobless {} bound {} retired {} | households {} ({} in debt, mean purse {:.0}) | needs money {:.2} hunger {:.2} safety {:.2} grievance {:.2} ambition {:.2}",
                count(Work::Working), count(Work::Injured), count(Work::Away), count(Work::Jobless), count(Work::Bonded), count(Work::Retired),
                hhs.len(), indebt, coin, mean(0), mean(1), mean(2), mean(3), mean(4)
            );
            let mems: usize = folk.iter().map(|&p| w.mind(p).memories.len()).sum();
            let mut stages = [0usize; 6];
            for &h in &hhs {
                for f in &w.society.households[h].feelings {
                    stages[f.stage as usize] += 1;
                }
            }
            let mut deeds: std::collections::BTreeMap<String, usize> = Default::default();
            for e in w.events(ti as u16) {
                *deeds.entry(format!("{:?}", e.deed)).or_default() += 1;
            }
            println!(
                "  ties: memories {:.2}/person | feelings {} (avoid {} dispute {} harm {} blows {} feud {}) | events {}",
                mems as f32 / folk.len().max(1) as f32,
                stages.iter().sum::<usize>(), stages[1], stages[2], stages[3], stages[4], stages[5],
                deeds.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", ")
            );
            let live: Vec<String> = w.stories_in(ti as u16).iter().map(|s| format!("{:?}", s.plot).split(' ').next().unwrap_or("").to_string()).collect();
            let opps: Vec<String> = w.society.opps.iter().filter(|o| o.town == ti as u16).map(|o| format!("{}:{:?}", o.kind.name(), o.state)).collect();
            let ring = w.ring(ti as u16).map(|r| format!("ring of {} (purse {:.0}, heat {:.1}, last {:?})", r.members.len(), r.purse, r.heat, r.last.map(|l| l.0.words()))).unwrap_or_default();
            println!("  stories: cap {} live [{}] | opps [{}] | {}", w.story_cap(ti as u16), live.join(" "), opps.join(", "), ring);
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

/// A world from a seed, on its authored map if there is one (`maps/`), so
/// what's printed matches what the window would show.
fn world(seed: u64) -> gahturiyu_sim::sim::World {
    let path = gahturiyu_sim::sim::mapedit::MapEdits::path_for(seed);
    let forge = gahturiyu_sim::sim::forge::load(std::path::Path::new("assets/towns/demo"), seed).unwrap_or(None);
    let edits = match gahturiyu_sim::sim::mapedit::MapEdits::load_from(&path) {
        Ok((_, edits)) => {
            eprintln!("(on the map in {})", path.display());
            edits
        }
        Err(_) => Default::default(),
    };
    worldgen::generate_authored(seed, forge, edits)
}

/// `headless trade [days] [seed]`: is there a trade route that pays? After
/// `days`, at eleven in the morning: every good a town has to spare, what a
/// lot of ten costs there and what it fetches in each other town within a
/// morning's walk (the price falls as the buyer's stock grows, as it does
/// in play), under the margins as they are and under a few others.
fn trade(days: f64, seed: u64) {
    use gahturiyu_sim::sim::economy::{BUY_MARKUP, SELL_SHARE};
    use gahturiyu_sim::sim::items;
    use gahturiyu_sim::sim::jobs::{good_of, GOODS};
    use gahturiyu_sim::sim::society::Flow;
    use gahturiyu_sim::sim::World;
    const NEAR: f32 = 6000.0;
    const LOT: usize = 10;
    let mut w = world(seed);
    let until = days.floor() * 24.0 + 11.0;
    while w.time / HOUR < until {
        w.step(HOUR / 2.0);
    }
    let t = w.time;
    println!("Trade in world {seed}, day {}, 11:00. A route: buy a lot of {LOT} in one town, carry it to another within {} km, sell it there.", days.floor() as i64 + 1, NEAR / 1000.0);
    // (item, from, to, what one is worth where it's bought, what each of the lot is worth where it's sold, metres)
    let mut routes: Vec<(items::ItemId, usize, usize, f32, Vec<f32>, f32)> = Vec::new();
    for a in 0..w.settlements.len() {
        for g in GOODS {
            let keep = if g.is_food() { w.society.communities[w.society.towns[a].shore as usize].food.need } else { 0.0 };
            let spare = (w.stock_now(a as u16, g) - keep).max(0.0);
            for key in g.items() {
                let it = items::id(key);
                if ((spare / g.items().len() as f32) / World::units_of(g, it)).floor() < LOT as f32 {
                    continue;
                }
                for b in 0..w.settlements.len() {
                    let far = w.settlements[a].pos.dist(w.settlements[b].pos);
                    if a == b || far > NEAR {
                        continue;
                    }
                    // Each one sold adds to the buyer's stock and lowers the next one's price.
                    let stock = w.society.towns[b].stock[g.index()];
                    let each: Vec<f32> = (0..LOT).map(|k| w.worth_holding(b as u16, it, None, t, Some(Flow { base: stock.base + k as f32 * World::units_of(g, it), rate: stock.rate }))).collect();
                    routes.push((it, a, b, w.worth_in(a as u16, it, None), each, far));
                }
            }
        }
    }
    let towns = w.settlements.len();
    println!("{} lots on offer to carry somewhere near, from {} towns.", routes.len(), towns);
    for (buy, sell) in [(BUY_MARKUP, SELL_SHARE), (1.25, 0.6), (1.2, 0.7), (1.1, 0.8)] {
        let cost = |r: &(items::ItemId, usize, usize, f32, Vec<f32>, f32)| (r.3 * buy).ceil().max(1.0) * LOT as f32;
        let take = |r: &(items::ItemId, usize, usize, f32, Vec<f32>, f32)| r.4.iter().map(|x| (x * sell).floor()).sum::<f32>();
        let mut paying: Vec<&(items::ItemId, usize, usize, f32, Vec<f32>, f32)> = routes.iter().filter(|r| take(r) > cost(r)).collect();
        paying.sort_by(|x, y| ((take(y) - cost(y)) / cost(y)).total_cmp(&((take(x) - cost(x)) / cost(x))));
        let from: std::collections::BTreeSet<usize> = paying.iter().map(|r| r.1).collect();
        let kinds: std::collections::BTreeSet<&str> = paying.iter().filter_map(|r| good_of(items::item(r.0).key)).map(|g| g.name()).collect();
        println!(
            "\nBuy at worth x {buy:.2}, sell at worth x {sell:.2}{}: {} lots pay; {} of {towns} towns have something that pays to carry; goods: {}.",
            if buy == BUY_MARKUP && sell == SELL_SHARE { " (as it is now)" } else { "" },
            paying.len(),
            from.len(),
            if kinds.is_empty() { "none".to_string() } else { kinds.into_iter().collect::<Vec<_>>().join(", ") }
        );
        // The best lot of each kind of thing.
        let mut shown: Vec<items::ItemId> = Vec::new();
        for r in &paying {
            if shown.contains(&r.0) || shown.len() >= 6 {
                continue;
            }
            shown.push(r.0);
            println!(
                "  {:<16} {:<12} -> {:<12} {:>4.1} km: {LOT} cost {:>4.0}, fetch {:>4.0}, gain {:>4.0} ({:.0}% on the coin, {:.0} kg)",
                items::item(r.0).name,
                w.settlements[r.1].name,
                w.settlements[r.2].name,
                r.5 / 1000.0,
                cost(r),
                take(r),
                take(r) - cost(r),
                (take(r) - cost(r)) / cost(r) * 100.0,
                items::item(r.0).weight * LOT as f32
            );
        }
    }
}
