//! Bug hunt (ruins-gangs session): sweeps that look for trouble across many
//! people and topics. Ignored: they print what they find rather than fail.
//! `cargo test --release --test hunt_talk -- --ignored --nocapture`

use std::collections::BTreeMap;

use gahturiyu_sim::sim::{dialogue::Topic, jobs::Job, speech, worldgen, World};

/// Things in a line of text that look wrong.
fn faults(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let plain = speech::plain(text);
    if plain.trim().is_empty() {
        out.push("empty".into());
    }
    for bad in ["{", "}", "⟦", "⟧", "..", " ,", " .", "  ", "None", "Some(", "NaN", "inf ", "_"] {
        if plain.contains(bad) {
            out.push(format!("contains {bad:?}"));
        }
    }
    let words: Vec<&str> = plain.split_whitespace().collect();
    for w in words.windows(2) {
        let (a, b) = (w[0].to_lowercase(), w[1].to_lowercase());
        if a == b && a.chars().all(|c| c.is_alphabetic()) {
            out.push(format!("doubled word {a:?}"));
        }
        if a == "a" && b.starts_with(['a', 'e', 'i', 'o', 'u']) && !b.starts_with("one") && !b.starts_with("use") {
            out.push(format!("a/an: \"a {b}\""));
        }
        if a == "an" && !b.starts_with(['a', 'e', 'i', 'o', 'u', 'h']) {
            out.push(format!("a/an: \"an {b}\""));
        }
    }
    out
}

/// Open a conversation between squad member 0 and `npc` (teleporting the
/// squad beside them).
fn open(w: &mut World, npc: u32) -> bool {
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0)));
    let who = w.squad.members[0];
    if !w.order_talk(who, npc) {
        return false;
    }
    for _ in 0..40 {
        if w.talk.is_some() {
            return true;
        }
        w.step(0.25);
    }
    w.talk.is_some()
}

#[test]
#[ignore]
fn talk_sweep() {
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut opened = 0;
    let mut said = 0;
    let mut greetings: BTreeMap<String, usize> = BTreeMap::new();
    let mut backgrounds: BTreeMap<String, usize> = BTreeMap::new();
    for seed in [1u64, 21, 34] {
        let mut w = worldgen::generate(seed);
        // Into the day, so merchants and workers are about.
        while w.time < 10.0 * 3600.0 {
            w.step(60.0);
        }
        // One person of each job and people in each town, at most.
        let mut picked: Vec<u32> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for s in 0..w.settlements.len() {
            for &p in &w.settlements[s].residents {
                let job = w.life(p).job;
                let key = (s, format!("{job:?}"), w.people[p as usize].race.name());
                if w.people[p as usize].dead || !seen.insert(key) {
                    continue;
                }
                picked.push(p);
            }
        }
        picked.truncate(400);
        for &npc in &picked {
            let job = w.life(npc).job;
            let mut ww = w.clone();
            if !open(&mut ww, npc) {
                continue;
            }
            opened += 1;
            let tag = |t: &Topic| format!("{t:?}").split('(').next().unwrap_or("?").to_string();
            let check = |ww: &World, what: &str, found: &mut BTreeMap<String, Vec<String>>| {
                if let Some(c) = &ww.talk {
                    if let Some((_, text)) = c.lines.last() {
                        for f in faults(text) {
                            found.entry(format!("{what}: {f}")).or_default().push(format!("seed {seed} p{npc} {job:?}: {}", speech::plain(text)));
                        }
                    }
                }
            };
            check(&ww, "Greeting", &mut found);
            if let Some(c) = &ww.talk {
                if let Some((_, g)) = c.lines.first() {
                    *greetings.entry(speech::bare(g)).or_default() += 1;
                }
            }
            // Every topic once, in turn (not the ones that change the world much).
            for round in 0..3 {
                let topics = ww.topics();
                for t in topics {
                    if matches!(t, Topic::Goodbye | Topic::Join(_) | Topic::Hire(_) | Topic::TakePost(_) | Topic::BuyOut(..) | Topic::Accept | Topic::TakeJob(_) | Topic::PostWork(..) | Topic::Learn(_) | Topic::CraftLesson(_) | Topic::Order(..) | Topic::PayBounty) {
                        continue;
                    }
                    if round > 0 && !matches!(t, Topic::Say(_)) {
                        continue;
                    }
                    if ww.talk.is_none() {
                        break;
                    }
                    ww.ask(t);
                    said += 1;
                    check(&ww, &tag(&t), &mut found);
                    if t == Topic::Background {
                        if let Some(c) = &ww.talk {
                            if let Some((_, g)) = c.lines.last() {
                                *backgrounds.entry(speech::bare(g)).or_default() += 1;
                            }
                        }
                    }
                }
            }
            let _ = Job::None;
        }
    }
    eprintln!("opened {opened} conversations, asked {said} topics");
    let mut g: Vec<_> = greetings.iter().collect();
    g.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("{} different greetings in {opened} conversations; most used:", g.len());
    for (t, n) in g.iter().take(5) {
        eprintln!("   {n}×  {t}");
    }
    let mut b: Vec<_> = backgrounds.iter().collect();
    b.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("{} different background answers; most used:", b.len());
    for (t, n) in b.iter().take(5) {
        eprintln!("   {n}×  {}", t.chars().take(160).collect::<String>());
    }
    for (k, v) in &found {
        eprintln!("\n== {k} ({} times)", v.len());
        for e in v.iter().take(3) {
            eprintln!("   {e}");
        }
    }
}

/// Buy something and sell it straight back, in the same town or the next:
/// anywhere the sale fetches at least what the purchase cost is coin from
/// nothing (or a trade route, if it's another town).
#[test]
#[ignore]
fn trade_sweep() {
    use gahturiyu_sim::sim::items;
    for seed in [1u64, 21, 34] {
        let mut w = worldgen::generate(seed);
        while w.time < 10.0 * 3600.0 {
            w.step(60.0);
        }
        // Give the talker coin so every buy line shows.
        let me = w.squad.members[0];
        w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 5000);
        let merchants: Vec<u32> = (0..w.people.len() as u32).filter(|&p| w.trades_next(p).is_some_and(|t| t <= w.time + 1.0)).collect();
        eprintln!("seed {seed}: {} merchants at work", merchants.len());
        let mut buys: Vec<(u32, u16, items::ItemId, u16)> = Vec::new(); // (npc, town, item, price)
        for &npc in &merchants {
            let mut ww = w.clone();
            if !open(&mut ww, npc) {
                eprintln!("  couldn't open talk with merchant p{npc}");
                continue;
            }
            ww.ask(Topic::Trade);
            let town = ww.people[npc as usize].home.unwrap_or(0);
            for t in ww.topics() {
                if let Topic::Buy(it, p) = t {
                    buys.push((npc, town, it, p));
                }
            }
        }
        // Who in each town would pay what for each thing bought.
        let mut pumps = 0;
        for &(npc, town, it, price) in &buys {
            for &m in &merchants {
                if let Some(o) = w.offer_for(m, it) {
                    let mt = w.people[m as usize].home.unwrap_or(0);
                    if o >= price {
                        let same = if mt == town { "SAME TOWN" } else { "other town" };
                        if mt == town || o as f32 >= price as f32 * 1.6 {
                            eprintln!("  {same}: buy {} from p{npc} ({}) at {price}, p{m} ({}) pays {o}", items::item(it).name, w.settlements[town as usize].name, w.settlements[mt as usize].name);
                            pumps += 1;
                        }
                    }
                }
            }
        }
        eprintln!("  {} buy lines, {pumps} flagged", buys.len());
        // The best trade routes: bought in one town, sold in another.
        let mut best: Vec<(f32, String)> = Vec::new();
        for &(_, town, it, price) in &buys {
            for &m in &merchants {
                let mt = w.people[m as usize].home.unwrap_or(0);
                if mt == town { continue; }
                if let Some(o) = w.offer_for(m, it) {
                    best.push((o as f32 / price.max(1) as f32, format!("{} {}→{} {}→{} ({:.1} km)", items::item(it).name, w.settlements[town as usize].name, w.settlements[mt as usize].name, price, o, w.settlements[town as usize].pos.dist(w.settlements[mt as usize].pos) / 1000.0)));
                }
            }
        }
        best.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (r, l) in best.iter().take(5) {
            eprintln!("    best route x{r:.2}: {l}");
        }
    }
}

/// Where can each kind of thing be made? Stations in the world, by kind, and
/// recipes whose station or inputs can be had nowhere.
#[test]
#[ignore]
fn making_sweep() {
    use gahturiyu_sim::sim::crafting::{Station, RECIPES};
    use gahturiyu_sim::sim::items;
    for seed in [1u64, 21, 34] {
        let w = worldgen::generate(seed);
        let mut by: BTreeMap<String, usize> = BTreeMap::new();
        for (_, st) in &w.stations {
            *by.entry(st.name().to_string()).or_default() += 1;
        }
        eprintln!("seed {seed}: {} towns; stations {:?}", w.settlements.len(), by);
        let all = [Station::Forge, Station::Bench, Station::Desk, Station::AlchemyTable, Station::Loom, Station::Workbench, Station::GrowerBed];
        for st in all {
            let n = RECIPES.iter().filter(|r| r.station == st).count();
            if !w.stations.iter().any(|(_, s)| *s == st) {
                eprintln!("  NO {} anywhere: {n} recipes need it", st.name());
            }
        }
        let _ = items::ITEMS.len();
    }
}

/// Every spell that can be cast outside a fight, at yourself, at a
/// townsperson and at the ground among them: what happens, and does the
/// town notice anything that should be a crime?
#[test]
#[ignore]
fn magic_sweep() {
    use gahturiyu_sim::sim::{magic::{all_spells, Style}, stats::SKILLS};
    let mut w = worldgen::generate(21);
    while w.time < 11.0 * 3600.0 {
        w.step(60.0);
    }
    let me = w.squad.members[0];
    for k in SKILLS {
        w.people[me as usize].stats.set_skill(k, 90.0);
    }
    w.people[me as usize].recompute_might();
    // A townsperson out in the street, and the squad beside them.
    let town = 0usize;
    let npc = *w.settlements[town].residents.iter().find(|&&p| !w.people[p as usize].dead && w.doing_now(p).is_some()).unwrap();
    for s in all_spells() {
        let d = s.def();
        if d.style == Style::Ritual || !d.works_outside_fights() {
            continue;
        }
        for (what, target, point) in [("self", Some(me), None), ("townsperson", Some(npc), None), ("ground", None, Some(true))] {
            let mut ww = w.clone();
            let at = ww.person_pos(npc);
            ww.teleport_squad(at.add(gahturiyu_sim::sim::geo::V2::new(3.0, 0.0)));
            if let Some(dd) = ww.people[me as usize].detail.as_mut() {
                if !dd.spells.contains(&s) {
                    dd.spells.push(s);
                }
            }
            let t = ww.time;
            let m = ww.people[me as usize].max_mana();
            ww.people[me as usize].set_mana(m, t);
            let hp_before: f32 = ww.people[npc as usize].wounds.hp_at(&ww.people[npc as usize].stats, t).iter().sum();
            let rec_before = ww.records.get(&me).copied().unwrap_or(0.0);
            let log_before = ww.log.len();
            let r = ww.order_cast(me, s, target, point.map(|_| at));
            for _ in 0..20 {
                ww.step(0.5);
            }
            let t2 = ww.time;
            let hp_after: f32 = ww.people[npc as usize].wounds.hp_at(&ww.people[npc as usize].stats, t2).iter().sum();
            let rec_after = ww.records.get(&me).copied().unwrap_or(0.0);
            let fight = ww.squad_battle().is_some();
            let _ = log_before;
            let news: Vec<String> = ww.log.iter().filter(|l| l.0 >= t && !l.1.contains("improves") && !l.1.contains("comes into view") && !l.1.contains("bound for") && !l.1.contains("heading home")).map(|l| l.1.clone()).collect();
            let hurt = hp_after < hp_before - 0.5;
            if true {
                eprintln!("{} at {what}: {:?}; townsperson hurt {hurt} ({hp_before:.0}→{hp_after:.0}); fight {fight}; record {rec_before}→{rec_after}; {:?}", d.name, r.as_ref().map(|_| "ok").map_err(|e| format!("{e:?}")), news);
            }
        }
    }
}

/// Everything that happens, written out so two worlds can be compared.
/// Maps are sorted first (their order in memory means nothing).
fn fingerprint(w: &World) -> String {
    fn sorted<K: Ord + std::fmt::Debug + Clone, V: std::fmt::Debug>(m: &std::collections::HashMap<K, V>) -> String {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| a.0.cmp(b.0));
        format!("{v:?}")
    }
    let mut s = String::new();
    s += &format!("{:?} {:?}\n", w.time, w.hour_done);
    s += &format!("{:?}\n", w.people);
    s += &format!("{:?}\n", w.groups);
    s += &format!("{:?}\n", w.squad);
    s += &format!("{:?}\n", w.battles);
    s += &format!("{:?}\n", w.npc_fights);
    s += &format!("{:?} {:?}\n", w.encounters, w.camps);
    s += &format!("{:?} {:?}\n", w.corpses, w.ground);
    s += &format!("{:?}\n", w.log);
    s += &format!("{:?} {:?}\n", w.busy_until, w.group_of);
    s += &format!("{:?} {:?}\n", w.standing, w.quests);
    s += &sorted(&w.bounty);
    s += &sorted(&w.news);
    s += &sorted(&w.torches);
    s += &sorted(&w.fighting);
    s += &sorted(&w.carried);
    s += &sorted(&w.suspicion);
    s += &sorted(&w.held);
    s += &sorted(&w.cast_count);
    s += &format!("{:?} {:?} {:?} {:?}\n", w.rituals, w.circles, w.boons, w.wards);
    s += &format!("{:?}\n{:?}\n", w.society, w.stations);
    s += &format!("{:?} {:?} {:?}\n", w.crafting, w.orders, w.lessons);
    s += &format!("{:?} {:?} {:?} {:?} {:?}\n", w.standing_in, w.records, w.bonds, w.duels, w.shunned);
    // The land's hand edits, the authored land under them, and the forged town.
    s += &format!("{:?}\n", bincode::serialize(&w.terrain.edits).unwrap());
    s += &format!("{:?}\n{:?}\n", bincode::serialize(&w.terrain.authored).unwrap(), w.forge);
    // Wildlife and livestock.
    s += &format!("{:?}\n", w.animals);
    // The squad's outposts.
    s += &format!("{:?} {}\n{:?}\n", w.bases, w.next_base, w.looting);
    s += &format!("{:?}\n{:?}\n{:?}\n{:?}\n", w.deposits, w.labour, w.butchering, w.chases);
    s += &format!("{:?}\n{:?}\n{:?}\n", w.levels, w.ruins, w.casts);
    s += &format!("{:?}\n{:?}\n{:?}\n", w.pursuits, w.dosing, w.giving);
    // Containers in buildings, and locks being worked.
    s += &format!("{:?}\n{:?}\n", w.containers, w.picking);
    // Each building's chosen style.
    s += &format!("{:?}\n", w.settlements.iter().map(|s| &s.styles).collect::<Vec<_>>());
    // Who slept in each building when the world was made (loose belongings follow it).
    s += &format!("{:?}\n", w.settlements.iter().map(|s| &s.sleepers).collect::<Vec<_>>());
    s
}


/// Save and load at awkward moments in trade, making, magic and talk: the
/// loaded world must carry on exactly as the saved one would have.
#[test]
#[ignore]
fn save_sweep() {
    use gahturiyu_sim::sim::{crafting::{Station, RECIPES}, items, magic::{all_spells, Style}, stats::SKILLS};
    let mut base = worldgen::generate(21);
    while base.time < 10.0 * 3600.0 {
        base.step(60.0);
    }
    let me = base.squad.members[0];
    for k in SKILLS {
        base.people[me as usize].stats.set_skill(k, 80.0);
    }
    base.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 500);
    let mut scenes: Vec<(&str, World)> = Vec::new();
    // Mid-conversation, trading.
    {
        let mut w = base.clone();
        let npc = (0..w.people.len() as u32).find(|&p| w.trades_next(p).is_some_and(|t| t <= w.time + 1.0)).unwrap();
        if open(&mut w, npc) {
            w.ask(Topic::Trade);
            scenes.push(("trading", w));
        }
    }
    // Mid-craft at a station.
    {
        let mut made = false;
        let mut why = BTreeMap::new();
        for ri in 0..RECIPES.len() {
            let mut w = base.clone();
            let st = RECIPES[ri].station;
            let Some(&(at, _)) = w.stations.iter().find(|(_, s)| *s == st) else { continue };
            w.teleport_squad(at);
            for &(k, n) in RECIPES[ri].inputs {
                w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id(k), n * 2);
            }
            match w.start_craft(me, ri) {
                Ok(()) => {
                    scenes.push(("crafting", w));
                    made = true;
                    break;
                }
                Err(e) => *why.entry(format!("{e:?}").split('(').next().unwrap_or("?").to_string()).or_insert(0) += 1,
            }
        }
        if !made {
            eprintln!("couldn't start any craft: {why:?}");
        }
        let _ = Station::Forge;
    }
    // A spell ordered from too far away (walking over to cast).
    {
        let mut w = base.clone();
        let s = all_spells().find(|s| s.def().style != Style::Ritual && s.def().works_outside_fights() && s.def().range < 20.0).unwrap();
        w.people[me as usize].detail.as_mut().unwrap().spells.push(s);
        let far = w.squad.pos.add(gahturiyu_sim::sim::geo::V2::new(80.0, 0.0));
        match w.order_cast(me, s, None, Some(far)) {
            Ok(()) => scenes.push(("walking to cast", w)),
            Err(e) => eprintln!("couldn't order the cast: {e:?}"),
        }
    }
    // A give walking over.
    {
        let mut w = base.clone();
        let other = w.squad.members[3];
        let k2 = w.squad.index(other).unwrap();
        w.squad.at[k2] = w.squad.at[k2].add(gahturiyu_sim::sim::geo::V2::new(40.0, 0.0));
        match w.order_give(me, 0, other) {
            Ok(_) => scenes.push(("giving", w)),
            Err(e) => eprintln!("couldn't order a give: {e}"),
        }
    }
    for (what, w) in scenes {
        let mut a = w.clone();
        let mut b = World::load_bytes(&w.save_bytes()).expect("loads");
        let mut first = None;
        for i in 0..1800 {
            a.step(1.0);
            b.step(1.0);
            if i % 60 == 59 {
                let (fa, fb) = (fingerprint(&a), fingerprint(&b));
                if fa != fb {
                    let line = fa.lines().zip(fb.lines()).position(|(x, y)| x != y).unwrap_or(0);
                    let la: Vec<&str> = fa.lines().collect();
                    let lb: Vec<&str> = fb.lines().collect();
                    let (x, y) = (la[line], lb[line]);
                    let at = x.chars().zip(y.chars()).position(|(p, q)| p != q).unwrap_or(0);
                    let lo = at.saturating_sub(120);
                    first = Some(format!("after {} s, part {line}: {:?} vs {:?}", i + 1, x.chars().skip(lo).take(240).collect::<String>(), y.chars().skip(lo).take(240).collect::<String>()));
                    break;
                }
            }
        }
        eprintln!("{what}: {}", first.unwrap_or_else(|| "same after 30 minutes".into()));
    }
}

/// Use every kind of thing outside a fight: what's said, and is anything
/// used up for nothing?
#[test]
#[ignore]
fn use_sweep() {
    use gahturiyu_sim::sim::items::{self, Kind, ITEMS};
    let mut base = worldgen::generate(21);
    while base.time < 10.0 * 3600.0 {
        base.step(60.0);
    }
    let me = base.squad.members[0];
    for def in ITEMS.iter() {
        if !matches!(def.kind, Kind::Potion | Kind::Scroll(_) | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_) | Kind::Food(_) | Kind::StandingTorch(_)) {
            continue;
        }
        let mut w = base.clone();
        let it = items::id(def.key);
        w.people[me as usize].detail.as_mut().unwrap().gear.add(it, 1);
        let before = w.count_of(me, def.key);
        let why = w.why_cant_use(me, it);
        let ok = w.use_item(me, it);
        let after = w.count_of(me, def.key);
        for _ in 0..10 {
            w.step(1.0);
        }
        let news: Vec<String> = w.log.iter().filter(|l| l.0 >= base.time && !l.1.contains("comes into view") && !l.1.contains("bound for") && !l.1.contains("heading home") && !l.1.contains("crosses your path")).map(|l| l.1.clone()).collect();
        let used = after < before;
        let flag = if ok && !used && !matches!(def.kind, Kind::Manual(_) | Kind::Notes(_) | Kind::Text(_)) { "  <-- ok but not used up" } else if !ok && used { "  <-- refused but used up" } else if ok && news.is_empty() { "  <-- nothing said" } else { "" };
        eprintln!("{:<28} {:?} ok={ok} used={used} why={why:?} {:?}{flag}", def.name, std::mem::discriminant(&def.kind), news);
    }
}

/// Every job on offer in a few worlds, over a few days: can each be done?
/// (A camp that exists, a person who's alive and somewhere you can reach,
/// a thing that can be had, a reward that makes sense.)
#[test]
#[ignore]
fn quest_sweep() {
    use gahturiyu_sim::sim::quests::QuestKind;
    use gahturiyu_sim::sim::items;
    for seed in [1u64, 21, 34] {
        let mut w = worldgen::generate(seed);
        let mut seen = std::collections::BTreeSet::new();
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        for day in 0..4 {
            while w.time < (day as f64 * 24.0 + 11.0) * 3600.0 {
                w.step(300.0);
            }
            for p in 0..w.people.len() as u32 {
                let Some((kind, coin, bonus)) = w.quest_offer(p) else { continue };
                if !seen.insert((p, format!("{kind:?}"))) {
                    continue;
                }
                let tag = format!("{kind:?}").split(' ').next().unwrap_or("?").to_string();
                *kinds.entry(tag).or_default() += 1;
                let giver = &w.people[p as usize];
                let problem = match kind {
                    QuestKind::ClearCamp { camp, at } => {
                        let c = w.camps.iter().find(|c| c.group == camp);
                        match c {
                            None => Some("the camp doesn't exist".to_string()),
                            Some(c) if c.pos.dist(at) > 50.0 => Some(format!("the camp is {:.0} m from where they say", c.pos.dist(at))),
                            _ => None,
                        }
                    }
                    QuestKind::Deliver { to } => {
                        let q = &w.people[to as usize];
                        if q.dead {
                            Some("the one to deliver to is dead".into())
                        } else if to == p {
                            Some("deliver to themselves".into())
                        } else if q.in_squad {
                            Some("deliver to a squad member".into())
                        } else if q.bandit {
                            Some("deliver to a bandit".into())
                        } else if w.terrain.is_sea(w.person_pos(to)) {
                            Some("the one to deliver to is out at sea".into())
                        } else {
                            None
                        }
                    }
                    QuestKind::Fetch { item, count } => {
                        if count == 0 { Some("fetch nothing".into()) } else if items::item(item).value * count as f32 > coin as f32 * 3.0 + 50.0 { Some(format!("{} × {} worth far more than the {coin} coin paid", count, items::item(item).name)) } else { None }
                    }
                    QuestKind::Job { .. } => None,
                };
                if coin == 0 && bonus.is_none() {
                    eprintln!("seed {seed} day {day} p{p} ({:?}): pays nothing", giver.race);
                }
                if let Some(pr) = problem {
                    eprintln!("seed {seed} day {day} p{p}: {kind:?}: {pr}");
                }
            }
        }
        eprintln!("seed {seed}: jobs offered {kinds:?}");
    }
}

/// The pack at its edges: two hands and a shield, far too much to carry,
/// dropping and picking up, a squadmate's gear.
#[test]
#[ignore]
fn pack_sweep() {
    use gahturiyu_sim::sim::{geo::V2, items::{self, Slot}};
    let mut base = worldgen::generate(21);
    while base.time < 10.0 * 3600.0 {
        base.step(60.0);
    }
    let me = base.squad.members[0];
    // Two-handed weapons with a shield already held, and the other way round.
    for (a, b) in [("glaive", "kite_shield"), ("kite_shield", "glaive"), ("crossbow", "buckler"), ("short_bow", "buckler"), ("buckler", "short_bow")] {
        let mut w = base.clone();
        let g = &mut w.people[me as usize].detail.as_mut().unwrap().gear;
        g.add(items::id(a), 1);
        g.add(items::id(b), 1);
        let ea = w.equip(me, items::id(a));
        let eb = w.equip(me, items::id(b));
        let g = &w.people[me as usize].detail.as_ref().unwrap().gear;
        eprintln!("hold {a} then {b}: {ea} {eb}; main {:?}, off {:?}", g.in_slot(Slot::MainHand).map(|i| items::item(i).name), g.in_slot(Slot::OffHand).map(|i| items::item(i).name));
    }
    // Far too much to carry: how does it walk?
    for kg in [0u16, 100, 400, 4000] {
        let mut w = base.clone();
        let n = kg / 5; // rock feedstock is about 5 kg? use timber at 2.5 kg
        w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("timber"), n * 2);
        let cap = w.capacity_at(me, w.time);
        let load = w.kit_weight_at(me, w.time);
        let start = w.person_pos(me);
        w.order_members(&[me], start.add(V2::new(150.0, 0.0)));
        let t0 = w.time;
        let mut arrived = None;
        while w.time < t0 + 3600.0 {
            w.step(1.0);
            if w.person_pos(me).dist(start.add(V2::new(150.0, 0.0))) < 3.0 {
                arrived = Some(w.time - t0);
                break;
            }
        }
        eprintln!("load {load:.0} of {cap:.0} kg: 150 m in {:?} s; news {:?}", arrived.map(|s| s as i64), w.log.iter().filter(|l| l.0 >= t0).map(|l| l.1.clone()).filter(|l| !l.contains("view") && !l.contains("bound") && !l.contains("heading") && !l.contains("crosses")).collect::<Vec<_>>());
    }
    // Drop it, and someone else picks it up.
    {
        let mut w = base.clone();
        let other = w.squad.members[1];
        let before = w.ground.len();
        let ok = w.drop_entry(me, 0);
        let g = w.ground.last().cloned();
        eprintln!("drop: {ok}; ground {} → {}", before, w.ground.len());
        if let Some(g) = g {
            let ok2 = w.order_pickup(other, g.id);
            for _ in 0..120 {
                w.step(0.5);
            }
            eprintln!("pickup by another: {ok2}; on the ground still: {}", w.ground.iter().any(|x| x.id == g.id));
        }
    }
}

/// Every ritual: can it be started where the squad stands, and if not, does
/// the refusal say where to go? Then one performed to the end, saved and
/// loaded halfway.
#[test]
#[ignore]
fn ritual_sweep() {
    use gahturiyu_sim::sim::{items, magic::{all_spells, Style}, stats::SKILLS};
    let mut base = worldgen::generate(21);
    while base.time < 10.0 * 3600.0 {
        base.step(60.0);
    }
    let me = base.squad.members[0];
    for k in SKILLS {
        base.people[me as usize].stats.set_skill(k, 90.0);
    }
    let mut picked = None;
    for s in all_spells().filter(|s| s.def().style == Style::Ritual) {
        let mut w = base.clone();
        w.people[me as usize].detail.as_mut().unwrap().spells.push(s);
        for &(key, n) in s.def().rite.components {
            w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
        }
        let r = w.can_perform(me, s);
        eprintln!("{:<26} place {:?}: {:?}", s.def().name, s.def().rite.place, r);
        if r.is_ok() && picked.is_none() {
            picked = Some(s);
        }
    }
    let Some(s) = picked else { return };
    let mut w = base.clone();
    w.people[me as usize].detail.as_mut().unwrap().spells.push(s);
    for &(key, n) in s.def().rite.components {
        w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id(key), n);
    }
    let r = w.use_spell(me, s, None, None);
    eprintln!("perform {}: {r:?}", s.def().name);
    for _ in 0..300 {
        w.step(1.0);
    }
    let mut b = World::load_bytes(&w.save_bytes()).unwrap();
    for i in 0..7200 {
        w.step(1.0);
        b.step(1.0);
        if i % 600 == 599 && fingerprint(&w) != fingerprint(&b) {
            eprintln!("mid-ritual save: the loaded world went its own way after {} s", i + 1);
            break;
        }
    }
    eprintln!("held after: {:?}; news {:?}", w.held.get(&me), w.log.iter().take(4).map(|l| l.1.clone()).collect::<Vec<_>>());
}

/// The pack says what each thing "sells for in <town>": does any merchant
/// of that town actually buy it?
#[test]
#[ignore]
fn sells_for_sweep() {
    use gahturiyu_sim::sim::items;
    let mut w = worldgen::generate(21);
    while w.time < 34.0 * 3600.0 {
        w.step(300.0);
    }
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    let merchants: Vec<u32> = w.settlements[town as usize].residents.iter().copied().filter(|&p| w.trades_next(p).is_some_and(|t| t <= w.time + 1.0)).collect();
    eprintln!("{} merchants at work at {:.1} h", merchants.len(), w.time / 3600.0);
    let mut seen = std::collections::BTreeSet::new();
    for &m in &w.squad.members {
        let d = w.people[m as usize].detail.as_ref().unwrap();
        for it in d.gear.bag.iter().map(|e| e.0).chain(items::SLOTS.iter().filter_map(|&s| d.gear.in_slot(s))) {
            if !seen.insert(it) {
                continue;
            }
            let said = w.sells_for(it, None).map(|x| x.0).unwrap_or(0);
            let best = merchants.iter().filter_map(|&n| w.offer_for(n, it)).max();
            if said > 0 && best.is_none() {
                eprintln!("{}: pack says sells for {said} in {}, but none of its {} merchants buys it", items::item(it).name, w.settlements[town as usize].name, merchants.len());
            }
        }
    }
}

/// Money at its edges: selling a mountain of one thing to one merchant,
/// buying out a stall, coin in the tens of thousands.
#[test]
#[ignore]
fn money_sweep() {
    use gahturiyu_sim::sim::items;
    let mut w = worldgen::generate(21);
    while w.time < 34.0 * 3600.0 {
        w.step(300.0);
    }
    let me = w.squad.members[0];
    let town = w.settlements.iter().min_by(|a, b| a.pos.dist(w.squad.pos).total_cmp(&b.pos.dist(w.squad.pos))).unwrap().id;
    let merchants: Vec<u32> = w.settlements[town as usize].residents.iter().copied().filter(|&p| w.trades_next(p).is_some_and(|t| t <= w.time + 1.0)).collect();
    eprintln!("purse at start {:.0}", w.purse_now(town));
    // A mountain of iron ingots, sold all at once.
    w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("iron_ingot"), 2000);
    for &m in &merchants {
        let mut ww = w.clone();
        if !open(&mut ww, m) {
            continue;
        }
        ww.ask(Topic::Trade);
        let coin0 = ww.squad_count(items::id("coin"));
        let sell = ww.topics().into_iter().find(|t| format!("{t:?}").starts_with("SellAll"));
        if let Some(t) = sell {
            let label = t.label();
            ww.ask(t);
            let got = ww.squad_count(items::id("coin")) - coin0;
            eprintln!("p{m}: {label:?} → got {got} coin; ingots left {}; purse now {:.0}; said {:?}", ww.count_of(me, "iron_ingot"), ww.purse_now(town), ww.talk.as_ref().and_then(|c| c.lines.last()).map(|l| l.1.clone()));
            // And again, straight away.
            if let Some(t2) = ww.topics().into_iter().find(|t| format!("{t:?}").starts_with("SellAll")) {
                let c1 = ww.squad_count(items::id("coin"));
                let l2 = t2.label();
                ww.ask(t2);
                eprintln!("   again: {l2:?} → {} more", ww.squad_count(items::id("coin")) - c1);
            }
        }
    }
    // Buy everything a merchant has, with a fortune.
    w.people[me as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 60000);
    for &m in &merchants {
        let mut ww = w.clone();
        if !open(&mut ww, m) {
            continue;
        }
        ww.ask(Topic::Trade);
        let mut bought = 0;
        let mut last = String::new();
        for _ in 0..400 {
            let Some(t) = ww.topics().into_iter().find(|t| matches!(t, Topic::Buy(..))) else { break };
            ww.ask(t);
            last = ww.talk.as_ref().and_then(|c| c.lines.last()).map(|l| l.1.clone()).unwrap_or_default();
            bought += 1;
        }
        eprintln!("p{m}: bought {bought} times; last {last:?}; buy lines left {}", ww.topics().iter().filter(|t| matches!(t, Topic::Buy(..))).count());
    }
    // Notes: change a fortune both ways.
    let mut ww = w.clone();
    let ex = (0..ww.people.len() as u32).find(|&p| ww.life(p).job == gahturiyu_sim::sim::jobs::Job::Exchanger && ww.people[p as usize].home == Some(town));
    if let Some(ex) = ex {
        if open(&mut ww, ex) {
            let c0 = ww.squad_count(items::id("coin"));
            ww.ask(Topic::Trade);
            eprintln!("exchanger topics: {:?}; at work {}", ww.topics(), ww.trades_next(ex).is_some_and(|t| t <= ww.time + 1.0));
            for _ in 0..50 {
                if ww.topics().contains(&Topic::ToNote) {
                    ww.ask(Topic::ToNote);
                }
            }
            for _ in 0..50 {
                if ww.topics().contains(&Topic::ToCoin) {
                    ww.ask(Topic::ToCoin);
                }
            }
            eprintln!("exchanger: 50 notes there and back: coin {c0} → {}; notes {}", ww.squad_count(items::id("coin")), ww.squad_count(items::id("note")));
        } else {
            eprintln!("exchanger p{ex} wouldn't talk");
        }
    }
}

/// When is each town's exchanger at work, over three days?
#[test]
#[ignore]
fn exchanger_hours() {
    let mut w = worldgen::generate(21);
    let ex: Vec<u32> = (0..w.people.len() as u32).filter(|&p| w.life(p).job == gahturiyu_sim::sim::jobs::Job::Exchanger).collect();
    let mut hours: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for h in 0..72u32 {
        while w.time < h as f64 * 3600.0 + 1800.0 {
            w.step(300.0);
        }
        for &e in &ex {
            if w.is_exchanging(e) {
                hours.entry(e).or_default().push(h);
            }
        }
    }
    eprintln!("{} exchangers; at work in some hour of 3 days: {}", ex.len(), hours.len());
    for e in ex.iter().filter(|e| !hours.contains_key(e)) {
        eprintln!("  p{e} never trades; next {:?}", w.trades_next(*e).map(|t| t / 3600.0));
    }
}

/// Cast the same lasting spell over and over: do boons and wards pile up?
/// And energy: does it come back, and never past the top?
#[test]
#[ignore]
fn stacking_sweep() {
    use gahturiyu_sim::sim::{magic::{all_spells, Style}, stats::SKILLS};
    let mut w = worldgen::generate(21);
    while w.time < 10.0 * 3600.0 {
        w.step(60.0);
    }
    let me = w.squad.members[0];
    for k in SKILLS {
        w.people[me as usize].stats.set_skill(k, 95.0);
    }
    for name in ["Haste", "Might", "Toughen", "Glow", "Tripwire", "Barrier", "Silent step"] {
        let Some(s) = all_spells().find(|s| s.def().name == name && s.def().style != Style::Ritual) else { continue };
        let mut ww = w.clone();
        ww.people[me as usize].detail.as_mut().unwrap().spells.push(s);
        let mut ok = 0;
        for _ in 0..12 {
            let t = ww.time;
            let m = ww.people[me as usize].max_mana();
            ww.people[me as usize].set_mana(m, t);
            if ww.cast(me, s, Some(me), None).is_ok() {
                ok += 1;
            }
            ww.step(1.0);
        }
        let boons = ww.boons.iter().filter(|b| b.pid == me).count();
        let wards = ww.wards.len();
        eprintln!("{name}: cast {ok} times → boons on caster {boons}, wards in the world {wards}");
    }
    // Energy: spent, then left a day.
    let t = w.time;
    let m = w.people[me as usize].max_mana();
    w.people[me as usize].set_mana(0.0, t);
    for h in [0.1, 0.25, 0.5, 1.0] {
        let mut ww = w.clone();
        while ww.time < t + h * 3600.0 {
            ww.step(60.0);
        }
        eprintln!("energy after {h} h: {:.1} of {m:.1}", ww.people[me as usize].mana_at(ww.time));
    }
}
