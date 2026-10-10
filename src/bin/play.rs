//! Play the game by text: a stand-in for the window, for play-testers who
//! can't click (the loop-testing agents). Each call loads a save, carries
//! out one command the way a click or key in the window would, lets the
//! world run as the command implies, prints what a player would now see,
//! and saves again.
//!
//!     play <save> new [seed]        start a new game
//!     play <save> look              what's on screen: the HUD and what's near
//!     play <save> help              every command
//!
//! Commands mirror the window's orders and panels; ids (p123, d4, h7…) come
//! from `look`. The window's own rules apply: nothing here changes the world
//! except through the same calls the window makes.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use gahturiyu_sim::sim::{
    combat::SQUAD_SIDE,
    containers::ContainerId,
    dialogue::Topic,
    geo::V2,
    items::{self, item, SLOTS},
    loot::{LootRef, Source},
    person::PersonId,
    quests::{compass, Stage},
    world::{DAY, HOUR},
    World,
};

const HELP: &str = "\
Commands (ids come from `look`; NAME is a squad member's first name, or `all`):
  look                      the HUD and everything near the squad
  select NAME [NAME..]      who takes orders (`select all` for everyone)
  go ID | go X,Y | go DIR M move the selected (DIR: n ne e se s sw w nw; M metres)
  attack pID                set the selected on someone (a sneak attack if unseen)
  talk pID                  walk over and talk; then `say N` picks a topic, `bye` ends it
  say N                     pick topic N in the conversation
  bye                       end the conversation
  loot pID                  go through a beaten foe's things (nearest selected member)
  search kID                go through a container in a building (theft if it's not yours)
  take N | takeall          take a line (or everything) from what's being gone through
  put N                     put the looter's pack entry N into the open container
  spells [NAME]             what NAME can cast; `cast NAME N [pID | X,Y]` casts spell N
  pickup gID                pick something up off the ground
  gather nID                gather a plant, rock or log
  work dID                  the selected work a woodlot/mine until their packs are full
  hunt hID                  the selected go after a wild herd
  butcher cID               the nearest selected cuts up a carcass
  enter bID                 walk into a building (picks the lock if locked and you have a lockpick)
  carry pID | putdown       pick up a downed person / put them down
  sneak | rest | torch      toggle for the selected
  pack [NAME]               a member's gear and pack, with entry numbers
  use NAME N [pID] | equip NAME N | drop NAME N   use/eat, put on, or drop pack entry N (a scroll of a harmful spell is read at pID: it starts the fight)
  give NAME N TO_NAME       hand pack entry N to another squad member standing near
  dose GIVER PATIENT        GIVER gives PATIENT (downed, say) a healing draught
  unequip NAME SLOT         take off what's worn in a slot (main, off, head, body, hands, legs, feet, back, ring, neck)
  craft NAME                what NAME could make here; `make NAME N` starts recipe N
  journal | map | town      jobs taken; towns and places; the nearest town's panel
  wait M                    let M minutes pass (stops early if a fight starts, someone goes down, or a talk opens)
  fight                     how the squad's fight is going (during a fight, `wait 1` moves it on)
  shot                      render a screenshot of the game window (slow: about a minute)";

/// What the play-tester's session remembers beside the save: who's selected,
/// which tips have shown, how far the log has been read.
#[derive(Default)]
struct Session {
    selected: Vec<PersonId>,
    tips: Vec<String>,
    log_seen: f64,
}

fn state_path(save: &Path) -> PathBuf {
    save.with_extension("session")
}

fn load_session(save: &Path) -> Session {
    let mut s = Session::default();
    let Ok(t) = std::fs::read_to_string(state_path(save)) else { return s };
    for line in t.lines() {
        let (k, v) = line.split_once('=').unwrap_or((line, ""));
        match k {
            "selected" => s.selected = v.split(',').filter_map(|x| x.parse().ok()).collect(),
            "tips" => s.tips = v.split(',').filter(|x| !x.is_empty()).map(|x| x.to_string()).collect(),
            "log_seen" => s.log_seen = v.parse().unwrap_or(0.0),
            _ => {}
        }
    }
    s
}

fn save_session(save: &Path, s: &Session) {
    let sel: Vec<String> = s.selected.iter().map(|p| p.to_string()).collect();
    let _ = std::fs::write(state_path(save), format!("selected={}\ntips={}\nlog_seen={}\n", sel.join(","), s.tips.join(","), s.log_seen));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("usage: play <save> <command…>\n\n{HELP}");
        return;
    }
    let save = PathBuf::from(&args[1]);
    let cmd = args[2].as_str();
    let rest: Vec<&str> = args[3..].iter().map(|s| s.as_str()).collect();
    if cmd == "help" {
        println!("{HELP}");
        return;
    }
    if cmd == "new" {
        let seed = rest.first().and_then(|s| s.parse().ok()).unwrap_or(1);
        let w = gahturiyu_sim::sim::worldgen::generate(seed);
        let mut s = Session::default();
        s.log_seen = w.time;
        w.save_to(&save).expect("save");
        save_session(&save, &s);
        println!("A new game (world {seed}). Your squad of {} stands in {}.\n", w.squad.members.len(), place_name(&w));
        let mut w = w;
        println!("{}", look(&mut w, &mut s));
        w.save_to(&save).expect("save");
        save_session(&save, &s);
        return;
    }
    let mut w = match World::load_from(&save) {
        Ok(w) => w,
        Err(e) => {
            println!("can't load {}: {e:?} (start with `play {} new`)", save.display(), save.display());
            return;
        }
    };
    let mut s = load_session(&save);
    s.selected.retain(|&p| w.squad.index(p).is_some());
    let out = run(&mut w, &mut s, cmd, &rest, &save);
    print!("{out}");
    w.save_to(&save).expect("save");
    save_session(&save, &s);
}

// ---- Helpers ------------------------------------------------------------------------

fn who(w: &World, s: &Session) -> Vec<PersonId> {
    if s.selected.is_empty() {
        w.squad.members.clone()
    } else {
        s.selected.clone()
    }
}

fn name_of(w: &World, p: PersonId) -> String {
    w.people[p as usize].name().map(|s| s.to_string()).unwrap_or_else(|| format!("someone ({})", w.people[p as usize].race.name()))
}

fn first_name(w: &World, p: PersonId) -> String {
    name_of(w, p).split_whitespace().next().unwrap_or("?").to_string()
}

fn member(w: &World, n: &str) -> Option<PersonId> {
    let n = n.to_lowercase();
    w.squad.members.iter().copied().find(|&m| {
        let f = first_name(w, m).to_lowercase();
        f == n || strip(&f) == strip(&n) || strip(&f).starts_with(&strip(&n))
    })
}

/// Lower case without the accents and dots, so "qidiqule" finds "Qìdìqule".
fn strip(s: &str) -> String {
    s.chars()
        .filter_map(|c| match c {
            'ì' | 'í' | 'î' => Some('i'),
            'ḍ' | 'Ḍ' => Some('d'),
            'ṭ' | 'Ṭ' => Some('t'),
            'ʻ' | '\'' => None,
            c => Some(c.to_ascii_lowercase()),
        })
        .collect()
}

fn dist_dir(from: V2, to: V2) -> String {
    let d = from.dist(to);
    if d < 3.0 {
        "right here".into()
    } else if d < 1000.0 {
        format!("{:.0} m {}", d, compass(to.sub(from)))
    } else {
        format!("{:.1} km {}", d / 1000.0, compass(to.sub(from)))
    }
}

fn place_name(w: &World) -> String {
    let here = w.squad.pos;
    w.settlements
        .iter()
        .filter(|s| s.pos.dist(here) < s.radius() + 300.0)
        .min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here)))
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "the wilds".into())
}

fn hhmm(t: f64) -> String {
    let s = t.rem_euclid(DAY);
    format!("{:02}:{:02}", (s / HOUR) as i64, ((s % HOUR) / 60.0) as i64)
}

fn pct(f: f32) -> String {
    format!("{:.0}%", (f * 100.0).clamp(0.0, 100.0))
}

fn parse_id(s: &str) -> Option<(char, String)> {
    let mut c = s.chars();
    let k = c.next()?;
    Some((k, c.as_str().to_string()))
}

fn container_id(s: &str) -> Option<ContainerId> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?))
}

/// Where a thing named by an id is.
fn pos_of(w: &World, id: &str) -> Option<V2> {
    let (k, rest) = parse_id(id)?;
    match k {
        'p' => rest.parse::<PersonId>().ok().filter(|&p| (p as usize) < w.people.len()).map(|p| w.person_pos(p)),
        'd' => w.deposit(rest.parse().ok()?).map(|d| d.pos),
        'h' => {
            let h: u32 = rest.parse().ok()?;
            w.animals.herds.get(h as usize).map(|_| w.herd_pos(h, w.time))
        }
        'c' => {
            let c: u32 = rest.parse().ok()?;
            w.animals.carcasses.iter().find(|x| x.id == c).map(|x| x.pos)
        }
        'g' => {
            let g: u32 = rest.parse().ok()?;
            w.ground.iter().find(|x| x.id == g).map(|x| x.pos)
        }
        'n' => {
            let n: u32 = rest.parse().ok()?;
            w.nodes.iter().find(|x| x.id == n).map(|x| x.pos)
        }
        't' => w.settlements.get(rest.parse::<usize>().ok()?).map(|s| s.pos),
        'r' => w.ruins.get(rest.parse::<usize>().ok()?).map(|r| r.pos),
        'b' => {
            let (a, b) = rest.split_once('.')?;
            w.door((a.parse().ok()?, b.parse().ok()?)).map(|d| d.outside)
        }
        'k' => w.container(container_id(&rest)?).map(|c| c.pos),
        _ => None,
    }
}

// ---- What a player sees -----------------------------------------------------------

const TIPS: &[(&str, &str)] = &[
    ("welcome", "Welcome. Left-click the ground to walk. Click a squad member to choose who takes orders. Hover over anything to see what it is."),
    ("town", "A town. Click a townsperson to talk: merchants trade, some have work, and a few restless ones will join the squad if asked."),
    ("work", "Short of coin? Every town has a woodlot (and some a mine) nearby, marked by a post. Click it and the selected work until their packs are full."),
    ("fight", "A fight! Click an enemy to set the selected on them. Z sneaks; T lights a torch."),
    ("loot", "A beaten foe: click them and someone goes through their things. Bandits carry coin."),
    ("full", "A full pack slows you down. Talk to a merchant (\"What have you got?\") and use Sell all."),
    ("hungry", "Someone's hungry. They eat from their pack when they need to: buy food from a merchant, or hunt (click a wild animal) and cut up what you kill."),
    ("night", "Night's coming. Press N to rest (a tent in someone's pack makes it a better sleep), or T for torches if you'd rather keep going."),
    ("beaten", "Beaten and robbed. Rest until you can stand (N), get some gear, and go and take it back from their camp."),
];

fn tip_due(w: &World, id: &str) -> bool {
    let near = |p: V2, r: f32| w.squad.pos.dist(p) <= r;
    match id {
        "welcome" => true,
        "town" => w.settlements.iter().any(|s| near(s.pos, s.radius() + 60.0)),
        "work" => w.deposits.iter().any(|d| near(d.pos, 400.0)),
        "fight" => w.squad_battle().is_some(),
        "loot" => w.squad_battle().is_none() && w.groups.iter().filter(|g| g.band <= 1).flat_map(|g| g.members.iter()).any(|&m| w.can_loot(m) && near(w.person_pos(m), 80.0)),
        "full" => w.squad.members.iter().any(|&m| w.load_of(m) > 0.95),
        "hungry" => w.squad.members.iter().any(|&m| w.hunger_of(m).is_some_and(|h| h >= 50.0)),
        "night" => gahturiyu_sim::sim::stealth::daylight(w.time) < 0.35,
        "beaten" => w.log.front().is_some_and(|l| l.1.starts_with("Beaten.") && w.time - l.0 < 3600.0),
        _ => false,
    }
}

fn status(w: &World, pid: PersonId) -> String {
    let k = w.squad.index(pid).unwrap_or(0);
    if w.is_down(pid) {
        return "down".into();
    }
    if let Some(f) = w.fighter(pid) {
        return if f.ko { "down".into() } else { "fighting".into() };
    }
    if w.is_asleep(pid) {
        return "asleep".into();
    }
    if w.carried_by(pid).is_some() {
        return "being carried".into();
    }
    if let Some(c) = w.carrying(pid) {
        return format!("carrying {}", first_name(w, c));
    }
    if w.labouring(pid).is_some() {
        return "working".into();
    }
    if w.butchering_now(pid) {
        return "butchering".into();
    }
    if w.source_now(pid).is_some() {
        return "going through things".into();
    }
    if w.crafting.iter().any(|j| j.who == pid) {
        return "crafting".into();
    }
    let moving = w.squad.at[k].dist(w.squad.goal[k]) > 0.5;
    let sneak = w.is_sneaking(pid);
    match (moving, sneak) {
        (true, true) => "sneaking along".into(),
        (true, false) => "walking".into(),
        (false, true) => "crouched".into(),
        (false, false) => "standing".into(),
    }
}

fn health(w: &World, pid: PersonId) -> f32 {
    let p = &w.people[pid as usize];
    if let Some(f) = w.fighter(pid) {
        return f.vitality();
    }
    let hp = p.wounds.hp_at(&p.stats, w.time);
    use gahturiyu_sim::sim::body::Part;
    (hp[0].max(0.0) / p.stats.max_hp(Part::Head)).min(hp[1].max(0.0) / p.stats.max_hp(Part::Torso))
}

fn hud(w: &World, s: &Session) -> String {
    let mut o = String::new();
    let tod = w.time.rem_euclid(DAY) / HOUR;
    let light = if (6.0..18.0).contains(&tod) { "day" } else { "night" };
    let _ = writeln!(o, "== {} ({light}) · {} ==", w.clock(), place_name(w));
    let coin: u32 = w.squad.members.iter().map(|&m| w.count_of(m, "coin") as u32 + 50 * w.count_of(m, "note") as u32).sum();
    let _ = writeln!(o, "Squad ({} members, {} coin between them){}:", w.squad.members.len(), coin, if s.selected.is_empty() { "" } else { " — selected marked *" });
    for &m in &w.squad.members {
        let p = &w.people[m as usize];
        let mark = if s.selected.contains(&m) { "*" } else { " " };
        let hunger = w.hunger_of(m).map(|h| if h >= 80.0 { " STARVING" } else if h >= 65.0 { " weak with hunger" } else if h >= 50.0 { " hungry" } else { "" }).unwrap_or("");
        let tired = w.tired_of(m).map(|t| if t >= 80.0 { " worn out" } else { "" }).unwrap_or("");
        let sus = w.suspicion_of(m);
        let seen = if sus >= 1.0 { " SPOTTED" } else if sus > 0.3 { " being noticed" } else { "" };
        let lvl = w.fresh_level_up(m).map(|(a, v)| format!(" ({a} {v} ↑)")).unwrap_or_default();
        let _ = writeln!(
            o,
            " {mark}{:<10} {:<8} {:<22} health {:>4} stamina {:>4} load {:.0}/{:.0} kg{hunger}{tired}{seen}{lvl}",
            first_name(w, m),
            p.race.name(),
            status(w, m),
            pct(health(w, m)),
            pct(w.stamina_of(m).unwrap_or(1.0)),
            w.kit_weight_at(m, w.time),
            w.capacity_at(m, w.time)
        );
    }
    if let Some(q) = w.quests.iter().find(|q| q.stage != Stage::Done) {
        let _ = writeln!(o, "Tracked job: {}", w.quest_line(q));
    }
    o
}

fn nearby(w: &World) -> String {
    let mut o = String::new();
    let here = w.squad.pos;
    let near = |p: V2, r: f32| here.dist(p) <= r;
    let mut lines: Vec<(f32, String)> = Vec::new();
    // People: townsfolk about, travellers and bands close by.
    let mut seen_people: Vec<PersonId> = Vec::new();
    for st in w.settlements.iter().filter(|s| near(s.pos, s.radius() + 400.0)) {
        for p in w.residents_in_band1(st.id) {
            let at = w.person_pos(p);
            if !near(at, 70.0) || w.is_indoors_asleep(p) || seen_people.contains(&p) {
                continue;
            }
            seen_people.push(p);
            let pp = &w.people[p as usize];
            let job = w.life(p).job.name();
            let mut tags = Vec::new();
            if w.is_trading(p) {
                tags.push("trading at their stall".to_string());
            }
            match w.join_terms(p) {
                Some(0) => tags.push("restless: might join".into()),
                Some(f) => tags.push(format!("restless: might join for {f} coin")),
                None => {}
            }
            let tag = if tags.is_empty() { String::new() } else { format!(" [{}]", tags.join("; ")) };
            lines.push((here.dist(at), format!("p{p}  {} — {} {}{tag} — {}", name_of(w, p), pp.race.name(), job.to_lowercase(), dist_dir(here, at))));
        }
    }
    for g in w.groups.iter().filter(|g| g.band <= 1) {
        for &m in &g.members {
            let at = w.person_pos(m);
            if !near(at, 150.0) || seen_people.contains(&m) || w.people[m as usize].dead {
                continue;
            }
            seen_people.push(m);
            let pp = &w.people[m as usize];
            let what = if w.can_loot(m) {
                "BEATEN — can be looted"
            } else if w.is_down(m) {
                "down"
            } else if pp.bandit || g.hostile {
                "bandit"
            } else {
                "traveller"
            };
            lines.push((here.dist(at), format!("p{m}  {} — {} {what} — {}", name_of(w, m), pp.race.name(), dist_dir(here, at))));
        }
    }
    // Ruins and camps.
    for r in w.ruins.iter().filter(|r| near(r.pos, 400.0)) {
        let held = if w.ruin_held(r.id) { "guarded" } else { "unguarded" };
        lines.push((here.dist(r.pos), format!("r{}  {} — {held}, {} things lying inside — {}", r.id, r.name(w), w.ruin_cache(r.id), dist_dir(here, r.pos))));
    }
    for c in w.camps.iter().filter(|c| near(c.pos, 300.0)) {
        lines.push((here.dist(c.pos), format!("    a bandit camp — {}", dist_dir(here, c.pos))));
    }
    // Work, game, things.
    for d in w.deposits.iter().filter(|d| near(d.pos, 600.0)) {
        lines.push((
            here.dist(d.pos),
            format!("d{}  {} of {} — {:.0} {} left; a unit is {} kg worth ~{} coin — {}", d.id, d.face().name, w.settlements[d.town as usize].name, d.left_at(w.time).floor(), item(d.item).name.to_lowercase(), item(d.item).weight, item(d.item).value, dist_dir(here, d.pos)),
        ));
    }
    for h in w.animals.herds.iter().filter(|h| h.alive(w.time) > 0) {
        let at = w.herd_pos(h.id, w.time);
        if near(at, 250.0) {
            let n = h.alive(w.time);
            lines.push((here.dist(at), format!("h{}  {} {} ({}) — {}", h.id, n, h.def().name, if h.def().yields.is_empty() { "nothing to take" } else { "huntable" }, dist_dir(here, at))));
        }
    }
    for c in w.animals.carcasses.iter().filter(|c| c.gone_at > w.time && near(c.pos, 200.0)) {
        lines.push((here.dist(c.pos), format!("c{}  a {} carcass — {}", c.id, c.sp.def().name, dist_dir(here, c.pos))));
    }
    for g in w.ground.iter().filter(|g| near(g.pos, 60.0)) {
        let whose = match g.owner {
            Some(town) => {
                let m = w.squad.members.iter().copied().min_by(|&a, &b| w.person_pos(a).dist(g.pos).total_cmp(&w.person_pos(b).dist(g.pos))).unwrap_or(w.squad.members[0]);
                format!(" (someone's: taking it is theft, {:.0}% chance of being seen)", w.catch_chance(m, g.pos, town) * 100.0)
            }
            None => String::new(),
        };
        lines.push((here.dist(g.pos), format!("g{}  {} × {} on the ground{whose} — {}", g.id, g.count, item(g.item).name, dist_dir(here, g.pos))));
    }
    for n in w.nodes.iter().filter(|n| near(n.pos, 120.0) && n.ready(w.time)) {
        lines.push((here.dist(n.pos), format!("n{}  {} ×{} to gather — {}", n.id, item(n.item).name, n.amount, dist_dir(here, n.pos))));
    }
    // Buildings close by, and containers in the one you're in.
    for d in w.doors_near(here, 45.0).into_iter().take(8) {
        let st = &w.settlements[d.id.0 as usize];
        let b = &st.buildings[d.id.1 as usize];
        let what = gahturiyu_sim::sim::layout::variant_of(b).map(|v| v.name.to_string()).unwrap_or_else(|| format!("{:?}", b.kind));
        let lock = if w.is_locked(d.id) { format!(" (locked: {})", w.lock_outlook(d.lock)) } else { String::new() };
        lines.push((here.dist(d.outside), format!("b{}.{}  {what}{lock} — {}", d.id.0, d.id.1, dist_dir(here, d.outside))));
    }
    if let Some(inside) = w.squad.inside.iter().flatten().next() {
        for c in w.containers_in(*inside) {
            let lock = if c.lock > 0.0 && !c.picked { format!(" (locked: {})", w.lock_outlook(c.lock)) } else { String::new() };
            lines.push((here.dist(c.pos), format!("k{}.{}.{}  a {}{lock} — {}", c.id.0, c.id.1, c.id.2, c.what.name(), dist_dir(here, c.pos))));
        }
    }
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    let _ = writeln!(o, "Near you ({} things; closest first):", lines.len());
    for (_, l) in lines.iter().take(40) {
        let _ = writeln!(o, "  {l}");
    }
    if lines.len() > 40 {
        let _ = writeln!(o, "  … and {} more further off", lines.len() - 40);
    }
    o
}

fn news(w: &World, s: &mut Session) -> String {
    let mut o = String::new();
    let fresh: Vec<&(f64, String)> = w.log.iter().filter(|(t, _)| *t > s.log_seen).collect();
    if !fresh.is_empty() {
        let _ = writeln!(o, "News:");
        for (t, l) in fresh.iter().rev() {
            let _ = writeln!(o, "  {}  {l}", hhmm(*t));
        }
    }
    s.log_seen = w.log.front().map(|l| l.0).unwrap_or(s.log_seen).max(s.log_seen);
    o
}

fn tips(w: &World, s: &mut Session) -> String {
    let mut o = String::new();
    for (id, text) in TIPS {
        if !s.tips.iter().any(|t| t == id) && tip_due(w, id) {
            s.tips.push(id.to_string());
            let _ = writeln!(o, "TIP: {text}");
        }
    }
    o
}

fn look(w: &mut World, s: &mut Session) -> String {
    let mut o = hud(w, s);
    o += &news(w, s);
    o += &tips(w, s);
    if w.squad_battle().is_some() {
        o += &fight(w);
    }
    if w.talk.is_some() {
        o += &talk_view(w);
    }
    o += &nearby(w);
    o
}

fn fight(w: &World) -> String {
    let mut o = String::new();
    let Some(b) = w.squad_battle() else { return "No fight going on.\n".into() };
    let _ = writeln!(o, "FIGHT:");
    for (i, f) in b.fighters.iter().enumerate() {
        let side = if f.home == SQUAD_SIDE { "yours" } else { "them " };
        let state = if f.dead {
            "dead"
        } else if f.ko {
            "down"
        } else if f.fled {
            "fled"
        } else if f.fleeing {
            "running"
        } else {
            "fighting"
        };
        let who = if f.is_person() { name_of(w, f.pid) } else { b.names.get(i).cloned().unwrap_or_else(|| "creature".into()) };
        let id = if f.is_person() { format!("p{}", f.pid) } else { "-".into() };
        let _ = writeln!(o, "  [{side}] {id:<7} {who:<22} {state:<8} health {}", pct(f.vitality()));
    }
    o
}

fn talk_view(w: &World) -> String {
    let mut o = String::new();
    let Some(c) = &w.talk else { return o };
    let _ = writeln!(o, "TALKING with {} ({} {}):", name_of(w, c.npc), w.people[c.npc as usize].race.name(), w.life(c.npc).job.name().to_lowercase());
    for (npc, l) in c.lines.iter().rev().take(6).collect::<Vec<_>>().into_iter().rev() {
        let _ = writeln!(o, "  {} {l}", if *npc { "»" } else { "  you:" });
    }
    let _ = writeln!(o, "  Topics (say N):");
    for (i, t) in w.topics().iter().enumerate() {
        let _ = writeln!(o, "    {}. {}", i + 1, w.topic_text(*t));
    }
    o
}

fn pack(w: &World, m: PersonId) -> String {
    let mut o = String::new();
    let p = &w.people[m as usize];
    let Some(d) = p.detail.as_ref() else { return "nothing".into() };
    let _ = writeln!(o, "{} — carrying {:.1} of {:.0} kg", name_of(w, m), w.kit_weight_at(m, w.time), w.capacity_at(m, w.time));
    let _ = writeln!(o, " Worn:");
    for s in SLOTS {
        if let Some(it) = d.gear.in_slot(s) {
            let _ = writeln!(o, "   {:<10} {}", format!("{s:?}"), item(it).name);
        }
    }
    let _ = writeln!(o, " Pack:");
    for (k, e) in d.gear.bag.iter().enumerate() {
        let kind = match item(e.0).kind {
            items::Kind::Food(n) => format!("food, {n:.0}"),
            items::Kind::Coin => "money".into(),
            _ => String::new(),
        };
        let _ = writeln!(o, "   {k:>2}. {} × {} {}  (worth ~{:.0} each)", e.1, item(e.0).name, if kind.is_empty() { String::new() } else { format!("[{kind}]") }, item(e.0).value);
    }
    o
}

// ---- Letting time pass -----------------------------------------------------------------

/// Run the world on for up to `secs`, stopping early at what a player would
/// notice: a fight starting, someone going down, a talk opening, or (with
/// `until_still`) everyone ordered getting where they were going.
fn pass(w: &mut World, secs: f64, until_still: Option<&[PersonId]>) -> String {
    let start = w.time;
    let fighting = w.squad_battle().is_some();
    let down: Vec<bool> = w.squad.members.iter().map(|&m| w.is_down(m)).collect();
    let talking = w.talk.is_some();
    let mut why = String::new();
    while w.time < start + secs {
        let busy = w.squad_battle().is_some() || w.squad.members.iter().enumerate().any(|(k, _)| w.squad.at[k].dist(w.squad.goal[k]) > 0.5);
        let dt: f64 = if busy { 0.25 } else { 2.0 };
        w.step(dt.min(start + secs - w.time).max(0.05));
        if !fighting && w.squad_battle().is_some() {
            why = "A fight has started!".into();
            break;
        }
        if fighting && w.squad_battle().is_none() {
            why = "The fight is over.".into();
            break;
        }
        if w.squad.members.iter().zip(&down).any(|(&m, &d)| !d && w.is_down(m)) {
            why = "Someone in your squad is down!".into();
            break;
        }
        if !talking && w.talk.is_some() {
            why = "A conversation opens.".into();
            break;
        }
        if let Some(who) = until_still {
            let still = who.iter().all(|&m| w.squad.index(m).map(|k| w.squad.at[k].dist(w.squad.goal[k]) < 0.5).unwrap_or(true));
            if still {
                break;
            }
        }
    }
    let mins = (w.time - start) / 60.0;
    let mut o = format!("({:.0} minutes pass.)", mins.max(0.0));
    if !why.is_empty() {
        o += " ";
        o += &why;
    }
    o + "\n"
}

// ---- Commands -----------------------------------------------------------------------------

fn run(w: &mut World, s: &mut Session, cmd: &str, a: &[&str], save: &Path) -> String {
    let sel = who(w, s);
    let lead = sel.first().copied().unwrap_or(w.squad.members[0]);
    let nearest = |w: &World, at: V2| sel.iter().copied().min_by(|&x, &y| w.person_pos(x).dist(at).total_cmp(&w.person_pos(y).dist(at))).unwrap_or(lead);
    let arg = |i: usize| a.get(i).copied().unwrap_or("");
    let mut o = String::new();
    // Most orders walk someone somewhere: let the world run till they're there.
    let walk_then_look = |w: &mut World, s: &mut Session, who: &[PersonId], o: &mut String| {
        *o += &pass(w, 30.0 * 60.0, Some(who));
        *o += &look(w, s);
    };
    match cmd {
        "look" => o += &look(w, s),
        "select" => {
            if a.is_empty() || arg(0) == "all" {
                s.selected.clear();
                o += "Everyone takes orders.\n";
            } else {
                s.selected = a.iter().filter_map(|n| member(w, n)).collect();
                let names: Vec<String> = s.selected.iter().map(|&m| first_name(w, m)).collect();
                o += &format!("Selected: {}\n", if names.is_empty() { "nobody matched; everyone takes orders".into() } else { names.join(", ") });
            }
        }
        "go" => {
            let target = if let Some(p) = pos_of(w, arg(0)) {
                Some(p)
            } else if let Some((x, y)) = arg(0).split_once(',') {
                match (x.parse::<f32>(), y.parse::<f32>()) {
                    (Ok(x), Ok(y)) => Some(V2::new(x, y)),
                    _ => None,
                }
            } else {
                let m: f32 = arg(1).parse().unwrap_or(50.0);
                let (dx, dy) = match arg(0) {
                    "n" | "north" => (0.0, -1.0),
                    "s" | "south" => (0.0, 1.0),
                    "e" | "east" => (1.0, 0.0),
                    "w" | "west" => (-1.0, 0.0),
                    "ne" => (0.707, -0.707),
                    "nw" => (-0.707, -0.707),
                    "se" => (0.707, 0.707),
                    "sw" => (-0.707, 0.707),
                    _ => (f32::NAN, 0.0),
                };
                (!dx.is_nan()).then(|| w.squad.pos.add(V2::new(dx * m, dy * m)))
            };
            match target {
                Some(t) => {
                    if s.selected.is_empty() {
                        w.order_squad(t);
                    } else {
                        w.order_members(&sel, t);
                    }
                    let moving = sel.iter().any(|&m| w.squad.index(m).is_some_and(|k| w.squad.at[k].dist(w.squad.goal[k]) > 1.0));
                    if !moving {
                        let why: Vec<String> = sel.iter().filter_map(|&m| {
                            let name = first_name(w, m);
                            if w.is_down(m) { Some(format!("{name} is down")) } else if !w.free_to_order(m) { Some(format!("{name} is bound to work off a bond")) } else { None }
                        }).collect();
                        o += &if why.is_empty() { "Nobody needs to move: they're already there.\n".to_string() } else { format!("Nobody moves: {}.\n", why.join("; ")) };
                    }
                    walk_then_look(w, s, &sel, &mut o);
                }
                None => o += "Go where? (an id from `look`, X,Y, or a direction and metres)\n",
            }
        }
        "attack" => {
            let p = arg(0).trim_start_matches('p').parse::<PersonId>().ok();
            match p {
                Some(p) if w.attack(&sel, p) => {
                    o += "You go in.\n";
                    o += &pass(w, 60.0, None);
                    o += &look(w, s);
                }
                _ => o += "Can't attack that (too far, or not someone to fight).\n",
            }
        }
        "talk" => {
            let p = arg(0).trim_start_matches('p').parse::<PersonId>().ok();
            match p {
                Some(p) if w.order_talk(lead, p) => {
                    o += &pass(w, 10.0 * 60.0, None);
                    o += &if w.talk.is_some() { talk_view(w) } else { "They're not talking (or you couldn't reach them).\n".into() };
                }
                _ => o += "Can't talk to them.\n",
            }
        }
        "say" => {
            let n: usize = arg(0).parse().unwrap_or(0);
            let t = w.topics();
            match n.checked_sub(1).and_then(|i| t.get(i)).copied() {
                Some(topic) => {
                    w.ask(topic);
                    if topic == Topic::Goodbye {
                        o += "You take your leave.\n";
                        w.end_talk();
                    } else {
                        o += &talk_view(w);
                    }
                }
                None => o += &format!("No topic {n}.\n{}", talk_view(w)),
            }
        }
        "bye" => {
            if w.talk.is_some() {
                w.ask(Topic::Goodbye);
                w.end_talk();
            }
            o += "You take your leave.\n";
        }
        "loot" | "search" => {
            let ok = if cmd == "loot" {
                let p = arg(0).trim_start_matches('p').parse::<PersonId>().ok();
                p.is_some_and(|p| {
                    let m = nearest(w, w.person_pos(p));
                    w.order_loot(m, p)
                })
            } else {
                container_id(arg(0).trim_start_matches('k')).is_some_and(|c| {
                    let at = w.container(c).map(|c| c.pos).unwrap_or(w.squad.pos);
                    let m = nearest(w, at);
                    if w.container_locked(c) {
                        w.order_pick_container(m, c)
                    } else {
                        w.order_search(m, c)
                    }
                })
            };
            if !ok {
                match (cmd, arg(0).trim_start_matches('p').parse::<PersonId>()) {
                    ("loot", Ok(p)) if (p as usize) < w.people.len() => o += &format!("Can't: {}\n", w.why_cant_loot(p)),
                    _ => o += "Can't go through that (is it locked, or not in a building you're in?).\n",
                }
            } else {
                let looter = w.looting.last().map(|l| l.who).unwrap_or(lead);
                let mut n = 0;
                while w.source_now(looter).is_none() && n < 2400 {
                    w.step(0.25);
                    n += 1;
                }
                o += &loot_view(w, looter);
            }
        }
        "put" => {
            let looter = w.looting.iter().map(|l| l.who).find(|&m| w.source_now(m).is_some());
            match (looter, arg(0).parse::<usize>()) {
                (Some(m), Ok(k)) => {
                    o += if w.put_in(m, k) { "Put away.\n" } else { "Couldn't put that there.\n" };
                    o += &loot_view(w, m);
                }
                (None, _) => o += "Nothing is open to put things in (use `search` first).\n",
                _ => o += "Usage: put N (N from the looter's `pack`)\n",
            }
        }
        "spells" => {
            let Some(m) = member(w, arg(0)).or(Some(lead)) else { return "Who?\n".into() };
            let p = &w.people[m as usize];
            let energy = w.fighter(m).map(|f| f.mana).unwrap_or_else(|| p.mana_at(w.time));
            let _ = writeln!(o, "{}'s spells (energy {:.0} / {:.0}):", name_of(w, m), energy, p.max_mana());
            let known = w.known_spells(m);
            if known.is_empty() {
                o += "  none\n";
            }
            for (i, sp) in known.iter().enumerate() {
                let d = sp.def();
                let does: Vec<String> = d.effects.iter().map(|e| e.describe()).collect();
                let _ = writeln!(o, "  {}. {} — {:?}, costs {:.0}, reach {:.0} m, aimed at {:?}: {}", i + 1, d.name, d.style, d.cost, d.range, d.aim, does.join("; "));
            }
        }
        "cast" => {
            let (Some(m), Ok(n)) = (member(w, arg(0)), arg(1).parse::<usize>()) else {
                return "Usage: cast NAME N [pID | X,Y] (N from `spells NAME`)\n".into();
            };
            let Some(&sp) = w.known_spells(m).get(n.saturating_sub(1)) else { return "They don't know that one.\n".into() };
            let t = arg(2);
            let target = t.strip_prefix('p').and_then(|x| x.parse::<PersonId>().ok());
            let point = match target {
                Some(p) => Some(w.person_pos(p)),
                None => t.split_once(',').and_then(|(x, y)| Some(V2::new(x.parse().ok()?, y.parse().ok()?))).or(Some(w.person_pos(m))),
            };
            match w.order_cast(m, sp, target, point) {
                Ok(()) => {
                    o += &format!("{} begins {}.\n", name_of(w, m), sp.def().name);
                    // Let the walk and the casting play out a little.
                    let mut k = 0;
                    while !w.casts.is_empty() && k < 600 {
                        w.step(0.25);
                        k += 1;
                    }
                    w.step(2.0);
                    o += &news(w, s);
                }
                Err(e) => o += &format!("Can't: {}\n", e.0),
            }
        }
        "take" | "takeall" => {
            let looter = w.looting.iter().map(|l| l.who).find(|&m| w.source_now(m).is_some());
            match looter.and_then(|m| w.source_now(m).map(|src| (m, src))) {
                Some((m, src)) => {
                    if cmd == "takeall" {
                        let n = w.take_all_from(m, src);
                        o += &format!("Took {n} lots.\n");
                    } else {
                        let n: usize = arg(0).parse().unwrap_or(0);
                        let what = w.contents(src).get(n.saturating_sub(1)).map(|x| x.0);
                        match what {
                            Some(r) if w.take_from(m, src, r) => o += "Taken.\n",
                            _ => o += "Couldn't take that.\n",
                        }
                    }
                    o += &news(w, s);
                    o += &loot_view(w, m);
                }
                None => o += "Nobody is going through anything right now (use `loot` or `search`).\n",
            }
        }
        "pickup" | "gather" | "work" | "butcher" => {
            let id = arg(0);
            let at = pos_of(w, id).unwrap_or(w.squad.pos);
            let num: u32 = id.get(1..).and_then(|x| x.parse().ok()).unwrap_or(u32::MAX);
            let ok = match cmd {
                "pickup" => w.order_pickup(nearest(w, at), num),
                "gather" => w.order_gather(nearest(w, at), num),
                "butcher" => w.order_butcher(nearest(w, at), num),
                _ => {
                    let mut any = false;
                    for &m in &sel {
                        any |= w.order_labour(m, num);
                    }
                    any
                }
            };
            if ok {
                walk_then_look(w, s, &sel, &mut o);
            } else {
                o += "Can't do that.\n";
            }
        }
        "hunt" => {
            let h: u32 = arg(0).trim_start_matches('h').parse().unwrap_or(u32::MAX);
            if w.order_hunt(&sel, h) {
                o += &pass(w, 15.0 * 60.0, None);
                o += &look(w, s);
            } else {
                o += "Can't hunt that.\n";
            }
        }
        "enter" => {
            let id = arg(0).trim_start_matches('b');
            let door = id.split_once('.').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
            match door.and_then(|d| w.door(d)) {
                Some(d) if w.is_locked(d.id) => {
                    let pick = items::id("lockpick");
                    let picker = sel.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().is_some_and(|x| x.gear.bag.iter().any(|e| e.0 == pick)));
                    match picker {
                        Some(m) => {
                            w.order_pick(m, d.id);
                            o += &pass(w, 5.0 * 60.0, None);
                            o += &look(w, s);
                        }
                        None => o += "It's locked, and nobody selected has a lockpick.\n",
                    }
                }
                Some(d) => {
                    w.order_members(&sel, d.centre);
                    walk_then_look(w, s, &sel, &mut o);
                }
                None => o += "No such building.\n",
            }
        }
        "carry" => {
            let p = arg(0).trim_start_matches('p').parse::<PersonId>().ok();
            match p.and_then(|p| sel.iter().copied().find(|&m| w.can_carry(m, p)).map(|m| (m, p))) {
                Some((m, p)) => {
                    w.order_carry(m, p);
                    walk_then_look(w, s, &[m], &mut o);
                }
                None => o += "Nobody selected can carry them.\n",
            }
        }
        "putdown" => {
            for m in sel.clone() {
                w.put_down(m);
            }
            o += "Put down.\n";
        }
        "sneak" => {
            let on = !sel.iter().all(|&m| w.is_sneaking(m));
            for &m in &sel {
                w.set_sneaking(m, on);
            }
            o += if on { "Sneaking.\n" } else { "Walking normally.\n" };
        }
        "rest" => {
            w.order_rest(&sel);
            o += "Rest order given (again to get them up).\n";
        }
        "torch" => {
            for &m in &sel {
                if w.torch_in_hand(m).is_some() || w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| item(e.0).key == "torch")) {
                    w.toggle_torch(m);
                }
            }
            o += "Torches toggled.\n";
        }
        "pack" => {
            let m = if a.is_empty() { lead } else { member(w, arg(0)).unwrap_or(lead) };
            o += &pack(w, m);
        }
        "use" | "equip" | "drop" => {
            let (Some(m), Ok(k)) = (member(w, arg(0)), arg(1).parse::<usize>()) else {
                return "Usage: use|equip|drop NAME N (N from `pack NAME`)\n".into();
            };
            let it = w.people[m as usize].detail.as_ref().and_then(|d| d.gear.bag.get(k)).map(|e| e.0);
            // A scroll of a harmful spell: read at a target (it starts the fight).
            if let (true, Some(it)) = (cmd == "use", it) {
                if let items::Kind::Scroll(key) = item(it).kind {
                    let sp = gahturiyu_sim::sim::magic::spell(key);
                    if w.fighting.contains_key(&m) || !sp.def().works_outside_fights() {
                        let t = arg(2);
                        let target = t.strip_prefix('p').and_then(|x| x.parse::<PersonId>().ok());
                        let point = match target {
                            Some(p) => Some(w.person_pos(p)),
                            None => t.split_once(',').and_then(|(x, y)| Some(V2::new(x.parse().ok()?, y.parse().ok()?))),
                        };
                        match w.order_read(m, it, target, point) {
                            Ok(()) => {
                                o += &format!("{} reads the scroll of {}.\n", name_of(w, m), sp.def().name.to_lowercase());
                                let mut n = 0;
                                while !w.casts.is_empty() && n < 600 {
                                    w.step(0.25);
                                    n += 1;
                                }
                                w.step(1.0);
                                o += &news(w, s);
                            }
                            Err(e) => o += &format!("Can't: {} (usage: use NAME N pID)\n", e.0),
                        }
                        return o;
                    }
                }
            }
            let ok = match (cmd, it) {
                ("use", Some(it)) => w.use_item(m, it),
                ("equip", Some(_)) => w.equip_entry(m, k),
                ("drop", Some(_)) => w.drop_entry(m, k),
                _ => false,
            };
            if ok {
                o += "Done.\n";
            } else {
                let why = match (cmd, it) {
                    ("use", Some(it)) => w.why_cant_use(m, it),
                    _ => None,
                };
                o += &format!("That didn't work{}.\n", why.map(|y| format!(": {y}")).unwrap_or_default());
            }
            o += &news(w, s);
            o += &pack(w, m);
        }
        "dose" => {
            let (Some(g), Some(p)) = (member(w, arg(0)), member(w, arg(1))) else {
                return "Usage: dose GIVER PATIENT (the giver hands the patient a healing draught)\n".into();
            };
            match w.order_dose(g, p) {
                Ok(line) => {
                    o += &format!("{line}\n");
                    let mut n = 0;
                    while !w.dosing.is_empty() && n < 2400 {
                        w.step(0.25);
                        n += 1;
                    }
                    o += &news(w, s);
                }
                Err(why) => o += &format!("Can't: {why}\n"),
            }
        }
        "give" => {
            let (Some(m), Ok(k), Some(to)) = (member(w, arg(0)), arg(1).parse::<usize>(), member(w, arg(2))) else {
                return "Usage: give NAME N TO_NAME (N from `pack NAME`)\n".into();
            };
            match w.give_entry(m, k, to) {
                Ok(line) => o += &format!("{line}\n"),
                Err(why) => o += &format!("Can't: {why}\n"),
            }
            o += &pack(w, m);
        }
        "unequip" => {
            let Some(m) = member(w, arg(0)) else { return "Who?\n".into() };
            let slot = SLOTS.iter().copied().find(|s| format!("{s:?}").to_lowercase().starts_with(&arg(1).to_lowercase()));
            let ok = slot.is_some_and(|s| w.unequip(m, s));
            o += if ok { "Taken off.\n" } else { "Nothing there.\n" };
            o += &pack(w, m);
        }
        "craft" | "make" => {
            use gahturiyu_sim::sim::crafting::RECIPES;
            let Some(m) = member(w, arg(0)) else { return "Who?\n".into() };
            if cmd == "make" {
                let n: usize = arg(1).parse().unwrap_or(0);
                match w.start_craft(m, n) {
                    Ok(()) => o += "Started.\n",
                    Err(e) => o += &format!("Can't: {e:?}\n"),
                }
            } else {
                let _ = writeln!(o, "What {} could make (recipe number: result — what stops it):", first_name(w, m));
                for (i, r) in RECIPES.iter().enumerate() {
                    let why = match w.can_craft(m, i) {
                        Ok(()) => "ready".to_string(),
                        Err(e) => format!("{e:?}"),
                    };
                    let _ = writeln!(o, "  {i}. {} — {why}", item(r.item(gahturiyu_sim::sim::materials::Grade::Common)).name);
                }
            }
        }
        "journal" => {
            let _ = writeln!(o, "Journal:");
            for q in &w.quests {
                let _ = writeln!(o, "  [{:?}] {}", q.stage, w.quest_line(q));
            }
            if w.quests.is_empty() {
                o += "  No jobs taken.\n";
            }
        }
        "map" => {
            let here = w.squad.pos;
            let mut towns: Vec<_> = w.settlements.iter().collect();
            towns.sort_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here)));
            let _ = writeln!(o, "Towns, nearest first:");
            for t in towns.iter().take(12) {
                let _ = writeln!(o, "  t{}  {} ({} people) — {}", t.id, t.name, t.residents.len(), dist_dir(here, t.pos));
            }
            let _ = writeln!(o, "Ruins and lairs (diamonds on the map):");
            for r in &w.ruins {
                let _ = writeln!(o, "  r{}  {}{} — {}", r.id, r.name(w), if w.ruin_held(r.id) { "" } else { " (unguarded)" }, dist_dir(here, r.pos));
            }
            let _ = writeln!(o, "Bandit camps known (red triangles):");
            let mut camps: Vec<V2> = w.camps.iter().map(|c| c.pos).collect();
            camps.sort_by(|a, b| a.dist(here).total_cmp(&b.dist(here)));
            for c in camps.iter().take(6) {
                let _ = writeln!(o, "  a camp — {}", dist_dir(here, *c));
            }
        }
        "town" => {
            let here = w.squad.pos;
            if let Some(t) = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))) {
                let _ = writeln!(o, "{} — {} people, {}", t.name, t.residents.len(), dist_dir(here, t.pos));
                let mut jobs: std::collections::BTreeMap<&str, usize> = Default::default();
                for &p in &t.residents {
                    *jobs.entry(w.life(p).job.name()).or_default() += 1;
                }
                let list: Vec<String> = jobs.iter().map(|(j, n)| format!("{j} {n}")).collect();
                let _ = writeln!(o, "  Trades: {}", list.join(", "));
                let _ = writeln!(o, "  Merchants:");
                for &p in &t.residents {
                    match w.trades_next(p) {
                        Some(at) if at <= w.time + 1.0 => {
                            let _ = writeln!(o, "    p{p} {} — trading now", name_of(w, p));
                        }
                        Some(at) => {
                            let day = if (at / DAY).floor() > (w.time / DAY).floor() { " tomorrow" } else { "" };
                            let _ = writeln!(o, "    p{p} {} — at their stall from {}{day}", name_of(w, p), hhmm(at));
                        }
                        None => {}
                    }
                }
            }
        }
        "wait" => {
            let m: f64 = arg(0).parse().unwrap_or(10.0);
            o += &pass(w, m * 60.0, None);
            o += &look(w, s);
        }
        "fight" => o += &fight(w),
        "shot" => {
            let png = save.with_extension("png");
            let status = std::process::Command::new("xvfb-run")
                .args(["-a", "-s", "-screen 0 1600x1000x24", "./target/release/gahturiyu"])
                .env("GAHT_LOAD", save)
                .env("GAHT_SHOT", &png)
                .env("GAHT_FRAMES", "30")
                .env("GAHT_SPEED", "0")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            o += &match status {
                Ok(st) if st.success() => format!("Screenshot saved: {}\n", png.display()),
                _ => "Couldn't take a screenshot.\n".into(),
            };
        }
        _ => o += &format!("Unknown command `{cmd}`.\n{HELP}\n"),
    }
    o
}

fn loot_view(w: &World, looter: PersonId) -> String {
    let mut o = String::new();
    let Some(src) = w.source_now(looter) else {
        let why = match w.looting.iter().find(|l| l.who == looter).map(|l| l.from) {
            Some(Source::Body(b)) if !w.can_loot(b) => w.why_cant_loot(b),
            _ => "they couldn't reach it in time (it may be behind a wall or a locked door)".into(),
        };
        return format!("They couldn't get to it: {why}\n");
    };
    let title = match src {
        Source::Body(b) => format!("{} goes through {}'s things", first_name(w, looter), name_of(w, b)),
        Source::Chest(c) => {
            let at = w.container(c).map(|c| c.pos).unwrap_or(w.squad.pos);
            format!("{} opens a {} (taking from it is theft: {:.0}% chance of being seen)", first_name(w, looter), w.container(c).map(|c| c.what.name()).unwrap_or("container"), w.catch_chance(looter, at, c.0) * 100.0)
        }
    };
    let _ = writeln!(o, "{title}:");
    for (i, (r, it, n)) in w.contents(src).iter().enumerate() {
        let worn = matches!(r, LootRef::Worn(_));
        let _ = writeln!(o, "  {}. {} × {}{}  (~{:.0} coin each)", i + 1, n, item(*it).name, if worn { " (worn)" } else { "" }, item(*it).value);
    }
    o += "  (take N, or takeall)\n";
    o
}
