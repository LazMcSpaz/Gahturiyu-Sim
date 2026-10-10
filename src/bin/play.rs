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
    containers::{is_stash, ContainerId, WILD},
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
  look [places|people]      the HUD and everything near the squad (or just places, or just people)
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
  sneak | torch             toggle for the selected
  rest | wake               the selected lie down where they are, or get up
  pack [NAME]               a member's gear and pack, with entry numbers
  use NAME N [pID] | equip NAME N | drop NAME N   use/eat, put on, or drop pack entry N (a scroll of a harmful spell is read at pID: it starts the fight)
  give NAME N TO_NAME       hand pack entry N to another squad member standing near
  dose GIVER PATIENT        GIVER gives PATIENT (downed, say) a healing draught
  unequip NAME SLOT         take off what's worn in a slot (main, off, head, body, hands, legs, feet, back, ring, neck)
  craft NAME                what NAME could make here; `make NAME N` starts recipe N
  journal | map | town      jobs taken; towns and places; the nearest town's panel
  build                     what can be built; `build camp` founds a base where the first selected member stands
  build kit                 TESTERS ONLY, not part of the game: teaches building and hands over materials and coin
  build KEY [DIR M] [DEG]   lay a building (a site) DIR M metres from that member, turned DEG degrees
  build wall KEY DIR M [DIR M ..]   lay a wall from that member's spot along the legs given
  base                      the bases: buildings, sites, store, who lives there, what's happened
  base store | leave NAME | fetch NAME | job NAME JOB | recipe NAME [N] | seal ID | down ID
                            put materials in the store; leave a member / fetch them (or let a hired hand go);
                            set a resident's job or recipe; seal a roof with pitch; take a building down
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
    /// The news lines already shown (`line_key`), so each shows once
    /// however it was stamped (NM-76).
    seen: Vec<u64>,
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
            "seen" => s.seen = v.split(',').filter_map(|x| x.parse().ok()).collect(),
            _ => {}
        }
    }
    s
}

fn save_session(save: &Path, s: &Session) {
    let sel: Vec<String> = s.selected.iter().map(|p| p.to_string()).collect();
    let seen: Vec<String> = s.seen.iter().map(|k| k.to_string()).collect();
    let _ = std::fs::write(state_path(save), format!("selected={}\ntips={}\nlog_seen={}\nseen={}\n", sel.join(","), s.tips.join(","), s.log_seen, seen.join(",")));
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
        s.seen = w.log.iter().map(|(t, l)| line_key(*t, l)).collect();
        w.save_to(&save).expect("save");
        save_session(&save, &s);
        println!("A new game (world {seed}). Your squad of {} stands in {}.\n", w.squad.members.len(), place_name(&w, w.squad.pos));
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
    w.alerts.clear();
    let out = run(&mut w, &mut s, cmd, &rest, &save);
    // What matters most (a fight, an arrest, a robbery) first, set apart.
    for a in w.alerts.drain(..) {
        println!("!! {a}");
    }
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
    // No name is nobody, not the first member (NM-69).
    if strip(n).is_empty() {
        return None;
    }
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

/// Where the tool looks from: the first selected member, or the squad's
/// middle when everyone takes orders (NM-74).
fn focus(w: &World, s: &Session) -> V2 {
    s.selected.first().filter(|&&m| w.squad.index(m).is_some()).map(|&m| w.person_pos(m)).unwrap_or(w.squad.pos)
}

/// Why this member can't take an order just now, if they can't (NM-72).
fn unable(w: &World, m: PersonId) -> Option<String> {
    let name = first_name(w, m);
    if w.is_down(m) {
        Some(format!("{name} is down"))
    } else {
        w.bond_of(m).map(|b| format!("{name} is bound in {}", w.settlements[b.town as usize].name))
    }
}

/// One news line, by when it is stamped and what it says.
fn line_key(t: f64, l: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in t.to_bits().to_le_bytes().iter().chain(l.as_bytes()) {
        h = (h ^ *b as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn place_name(w: &World, here: V2) -> String {
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

/// The number after an id's letter: `p12` asked for as a 'p' gives 12; an
/// id of another kind (`x12`, `g12`) or no letter at all is refused.
fn id_num<T: std::str::FromStr>(s: &str, letter: char) -> Option<T> {
    s.strip_prefix(letter)?.parse().ok()
}

/// A person named as `pN` who is in the world.
fn person_arg(w: &World, s: &str) -> Option<PersonId> {
    id_num::<PersonId>(s, 'p').filter(|&p| w.valid_person(p))
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
        'w' => {
            let (a, b) = rest.split_once('.')?;
            w.society.towns.get(a.parse::<usize>().ok()?)?.places.get(b.parse::<usize>().ok()?).map(|p| p.pos)
        }
        _ => None,
    }
}

// ---- What a player sees -----------------------------------------------------------

const TIPS: &[(&str, &str)] = &[
    ("welcome", "Welcome. Left-click the ground to walk. Click a squad member to choose who takes orders. Hover over anything to see what it is."),
    ("town", "A town. Click a townsperson to talk: merchants trade, some have work, and a few restless ones will join the squad if asked (`town` lists who, and their price)."),
    ("work", "Short of coin? Every town has a woodlot (and some a mine) nearby, marked by a post. Click it and the selected work until their packs are full."),
    ("fight", "A fight! Click an enemy to set the selected on them."),
    ("loot", "A beaten foe: click them and someone goes through their things. Bandits carry coin."),
    ("full", "A full pack slows you down. Talk to a merchant (\"What have you got?\") and use Sell all."),
    ("hungry", "Someone's hungry. They eat from their pack when they need to: buy food from a merchant, or hunt (click a wild animal) and cut up what you kill."),
    ("night", "Night's coming. Press N to rest (a tent in someone's pack makes it a better sleep), or T for torches if you'd rather keep going."),
    ("beaten", "Beaten and robbed. Rest until you can stand (N), get some gear, and go and take it back from their camp."),
    ("feel", "Felt spells come with use. As someone casts and fights, their feel for that kind of magic grows, and the spells within reach come to them on their own. Their spell book (M) shows what they know."),
];

fn tip_due(w: &World, id: &str) -> bool {
    let near = |p: V2, r: f32| w.squad.pos.dist(p) <= r;
    match id {
        "welcome" => true,
        "town" => w.settlements.iter().any(|s| near(s.pos, s.radius() + 60.0)),
        "work" => w.deposits.iter().any(|d| near(d.pos, 400.0)),
        "fight" => w.squad_battle().is_some(),
        "loot" => w.squad_battle().is_none() && w.groups.iter().filter(|g| g.band <= 1).flat_map(|g| g.members.iter()).any(|&m| w.can_loot(m) && near(w.person_pos(m), 80.0)),
        "full" => w.squad.members.iter().any(|&m| w.pack_load_of(m) > 0.95),
        "hungry" => w.squad.members.iter().any(|&m| w.hunger_of(m).is_some_and(|h| h >= 50.0)),
        "night" => gahturiyu_sim::sim::stealth::daylight(w.time) < 0.35,
        "beaten" => w.log.front().is_some_and(|l| l.1.starts_with("Beaten.") && w.time - l.0 < 3600.0),
        "feel" => w.log.iter().take(12).any(|l| w.time - l.0 < 3600.0 && (l.1.contains(" has a feel for ") || l.1.contains(" the feel of "))),
        _ => false,
    }
}

fn status(w: &World, pid: PersonId) -> String {
    let doing = doing(w, pid);
    // Bound to work in a town: where, and how long is left (NM-36).
    let Some(b) = w.bond_of(pid) else { return doing };
    let place = &w.settlements[b.town as usize].name;
    let held = match w.bond_days_left(pid) {
        Some(1) => format!("bound in {place}, a day left"),
        Some(n) => format!("bound in {place}, {n} days left"),
        None => format!("held in {place}"),
    };
    if matches!(doing.as_str(), "standing" | "walking" | "walking to work" | "at work") {
        held
    } else {
        format!("{held}, {doing}")
    }
}

fn doing(w: &World, pid: PersonId) -> String {
    let k = w.squad.index(pid).unwrap_or(0);
    if let Some(c) = w.carried_by(pid) {
        return format!("carried by {}", first_name(w, c));
    }
    if w.is_down(pid) {
        return "down".into();
    }
    if let Some(f) = w.fighter(pid) {
        return if f.ko { "down".into() } else { "fighting".into() };
    }
    if w.is_asleep(pid) {
        return "asleep".into();
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
    match w.on_shift(pid) {
        Some(true) => return "at work".into(),
        Some(false) if moving => return "walking to work".into(),
        _ => {}
    }
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
    // The word follows the light, not the clock.
    let tod = w.time.rem_euclid(DAY) / HOUR;
    let sun = gahturiyu_sim::sim::stealth::daylight(w.time);
    let light = if sun >= 0.6 {
        "day"
    } else if sun < 0.35 {
        "night"
    } else if tod < 12.0 {
        "dawn"
    } else {
        "dusk"
    };
    let _ = writeln!(o, "== {} ({light}) · {} ==", w.clock(), place_name(w, focus(w, s)));
    let coin: u32 = w.squad.members.iter().map(|&m| w.count_of(m, "coin") as u32 + 50 * w.count_of(m, "note") as u32).sum();
    let _ = writeln!(o, "Squad ({} members, {} coin between them){}:", w.squad.members.len(), coin, if s.selected.is_empty() { "" } else { " — selected marked *" });
    for &m in &w.squad.members {
        let p = &w.people[m as usize];
        let mark = if s.selected.contains(&m) { "*" } else { " " };
        let hunger = w.hunger_of(m).map(|h| if h >= 80.0 { " STARVING" } else if h >= 65.0 { " weak with hunger" } else if h >= 50.0 { " hungry" } else { "" }).unwrap_or("");
        let tired = w.tired_of(m).map(|t| if t >= 80.0 { " worn out" } else { "" }).unwrap_or("");
        let sus = if w.fighting.contains_key(&m) || w.is_down(m) { 0.0 } else { w.suspicion_of(m) };
        let seen = if sus >= 1.0 { " SPOTTED" } else if sus > 0.3 { " being noticed" } else { "" };
        let lvl = w.fresh_level_up(m).map(|(a, v)| format!(" ({a} {v} ↑)")).unwrap_or_default();
        // Strayed from the others, or left sneaking (half pace): both are
        // easy to miss, so both are flagged.
        let off = match w.strayed(m) {
            Some(d) if !w.is_down(m) => format!(" [{d:.0} m from the others]"),
            _ => String::new(),
        };
        let slow = if w.is_sneaking(m) && !w.is_down(m) { " [sneaking: half pace]" } else { "" };
        let _ = writeln!(
            o,
            " {mark}{:<10} {:<8} {:<22} health {:>4} stamina {:>4} load {:.0}/{:.0} kg{}{hunger}{tired}{seen}{lvl}{off}{slow}",
            first_name(w, m),
            p.race.name(),
            status(w, m),
            // Anyone on their feet has something left.
            pct(if w.is_down(m) { health(w, m) } else { health(w, m).max(0.01) }),
            pct(w.stamina_of(m).unwrap_or(1.0)),
            w.kit_weight_at(m, w.time).max(0.0),
            w.capacity_at(m, w.time),
            // Someone carried weighs on them too.
            w.carrying(m).map(|c| format!(" + {}", first_name(w, c))).unwrap_or_default()
        );
    }
    if let Some(q) = w.quests.iter().find(|q| q.stage != Stage::Done) {
        let _ = writeln!(o, "Tracked job: {}", w.quest_line(q));
    }
    for l in w.work_lines() {
        let _ = writeln!(o, "Town work: {l}");
    }
    o
}

/// What's near the squad: places and things first (all of them, up to a
/// point), then people. `only`: "places" or "people" to list just those.
fn nearby(w: &World, here: V2, only: &str) -> String {
    let mut o = String::new();
    let near = |p: V2, r: f32| here.dist(p) <= r;
    let mut lines: Vec<(f32, String)> = Vec::new();
    let mut folk: Vec<(f32, String)> = Vec::new();
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
            let job = w.life(p).job.title(w.people[p as usize].seed);
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
            // The willing first, with what they'd bring.
            match w.join_terms(p) {
                Some(_) => folk.push((here.dist(at) - 1e6, format!("p{p}  {} — {} {}{tag} — {}\n        {}", name_of(w, p), pp.race.name(), job.to_lowercase(), dist_dir(here, at), w.recruit_card(p)))),
                None => folk.push((here.dist(at), format!("p{p}  {} — {} {}{tag} — {}", name_of(w, p), pp.race.name(), job.to_lowercase(), dist_dir(here, at)))),
            }
        }
    }
    for g in w.groups.iter().filter(|g| g.band <= 1) {
        // A band of bandits in sight but not yet close: one line for where
        // they are (who and how many is for when they're near).
        if g.hostile {
            let nearest = g.members.iter().filter(|&&m| !w.people[m as usize].dead && !w.is_down(m)).map(|&m| w.person_pos(m)).min_by(|a, b| here.dist(*a).total_cmp(&here.dist(*b)));
            if let Some(at) = nearest.filter(|&at| !near(at, 150.0)) {
                lines.push((here.dist(at), format!("    bandits in sight — {}", dist_dir(here, at))));
            }
        }
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
            // Foes and the beaten among the places: they matter more than chat.
            let row = (here.dist(at), format!("p{m}  {} — {} {what} — {}", name_of(w, m), pp.race.name(), dist_dir(here, at)));
            if what == "traveller" {
                folk.push(row);
            } else {
                lines.push(row);
            }
        }
    }
    // Stilt villages, out over the water.
    for st in w.settlements.iter() {
        if let Some(sp) = st.stilts.filter(|&sp| near(sp, 1500.0)) {
            lines.push((here.dist(sp), format!("    the stilt village of {} — out over the water, {}", st.name, dist_dir(here, sp))));
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
            format!("d{}  {} of {} — {:.0} {} left; a unit is {} kg and fetches about {} coin in town (less each for a big lot) — {}", d.id, d.face().name, w.settlements[d.town as usize].name, d.left_at(w.time).floor(), item(d.item).name.to_lowercase(), item(d.item).weight, w.fetches_in(d.town, d.item), dist_dir(here, d.pos)),
        ));
    }
    // Out-of-town workplaces (fields, hunting grounds, docks...); a woodlot or
    // mine shows as its deposit above.
    // (A town's many fields as one line: the nearest, and how many.)
    for (ti, tl) in w.society.towns.iter().enumerate() {
        let fields = tl.places.iter().filter(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields).count();
        let nearest_field = tl.places.iter().enumerate().filter(|(_, p)| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields).min_by(|a, b| a.1.pos.dist(here).total_cmp(&b.1.pos.dist(here))).map(|(i, _)| i);
        for (i, wp) in tl.places.iter().enumerate() {
            if !wp.kind.is_away() || !near(wp.pos, 800.0) || w.deposits.iter().any(|d| d.pos.dist(wp.pos) < 15.0) {
                continue;
            }
            let what = if wp.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields {
                if Some(i) != nearest_field {
                    continue;
                }
                format!("Fields of {} (the nearest of {fields})", w.settlements[ti].name)
            } else {
                format!("{} of {}", wp.kind.name(), w.settlements[ti].name)
            };
            lines.push((here.dist(wp.pos), format!("w{ti}.{i}  {what} — {}", dist_dir(here, wp.pos))));
        }
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
        // The building as it stands (its stored style), not its first roll:
        // the two differ since buildings were matched to their people, and
        // the first roll named a great house "Maker's forge".
        let _ = b;
        let what = d.variant().name.to_string();
        let lock = if w.is_locked(d.id) { format!(" (locked: {})", w.lock_outlook(d.lock)) } else { String::new() };
        lines.push((here.dist(d.outside), format!("b{}.{}  {what}{lock} — {}", d.id.0, d.id.1, dist_dir(here, d.outside))));
    }
    // Containers out in the wild close by: ruins' caches, camps' stashes.
    for c in w.containers.range((WILD, 0, 0)..=(WILD, u16::MAX, u8::MAX)).map(|(_, c)| c).filter(|c| near(c.pos, 60.0)) {
        let lock = if c.lock > 0.0 && !c.picked { format!(" (locked: {})", w.lock_outlook(c.lock)) } else { String::new() };
        let place = if is_stash(c.id) { "the bandits' stash".to_string() } else { format!("a {} in the {}", c.what.name(), w.ruins.get(c.id.1 as usize).map(|r| r.name(w).to_lowercase()).unwrap_or_else(|| "ruin".into())) };
        lines.push((here.dist(c.pos), format!("k{}.{}.{}  {place}{lock}, {} things in it, nobody's — {}", c.id.0, c.id.1, c.id.2, c.items.len(), dist_dir(here, c.pos))));
    }
    if let Some(inside) = w.squad.inside.iter().flatten().next() {
        for c in w.containers_in(*inside) {
            let lock = if c.lock > 0.0 && !c.picked { format!(" (locked: {})", w.lock_outlook(c.lock)) } else { String::new() };
            lines.push((here.dist(c.pos), format!("k{}.{}.{}  a {}{lock} — {}", c.id.0, c.id.1, c.id.2, c.what.name(), dist_dir(here, c.pos))));
        }
    }
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    folk.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (most_places, most_folk) = match only {
        "places" => (200, 0),
        "people" => (0, 200),
        _ => (30, 20),
    };
    if most_places > 0 {
        let _ = writeln!(o, "Near you ({} places and things; closest first):", lines.len());
        for (_, l) in lines.iter().take(most_places) {
            let _ = writeln!(o, "  {l}");
        }
        if lines.len() > most_places {
            let _ = writeln!(o, "  … and {} more further off (`look places` for all)", lines.len() - most_places);
        }
    }
    if most_folk > 0 {
        let _ = writeln!(o, "People about ({}; closest first):", folk.len());
        for (_, l) in folk.iter().take(most_folk) {
            let _ = writeln!(o, "  {l}");
        }
        if folk.len() > most_folk {
            let _ = writeln!(o, "  … and {} more further off (`look people` for all)", folk.len() - most_folk);
        }
    }
    o
}

fn news(w: &World, s: &mut Session) -> String {
    let mut o = String::new();
    // Each line once (NM-76): lines are stamped with when the thing happened,
    // which isn't the order they were written in, so they're told apart by
    // stamp and words, not by being later than the last one read. What was
    // just shown as an alert ("!! …") isn't said again here.
    let by_time = s.seen.is_empty();
    let mut fresh: Vec<&(f64, String)> = w.log.iter().rev().filter(|(t, l)| !w.alerts.iter().any(|a| a == l) && if by_time { *t > s.log_seen } else { !s.seen.contains(&line_key(*t, l)) }).collect();
    fresh.sort_by(|a, b| a.0.total_cmp(&b.0));
    if !fresh.is_empty() {
        let _ = writeln!(o, "News:");
        for (t, l) in fresh {
            let _ = writeln!(o, "  {}  {l}", hhmm(*t));
        }
    }
    s.seen = w.log.iter().map(|(t, l)| line_key(*t, l)).collect();
    s.log_seen = w.log.iter().map(|l| l.0).fold(s.log_seen, f64::max);
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
    look_only(w, s, "")
}

fn look_only(w: &mut World, s: &mut Session, only: &str) -> String {
    let mut o = hud(w, s);
    o += &news(w, s);
    o += &tips(w, s);
    if w.squad_battle().is_some() {
        o += &fight(w);
    }
    if w.talk.is_some() {
        o += &talk_view(w);
    }
    o += &nearby(w, focus(w, s), only);
    o
}

fn fight(w: &World) -> String {
    let mut o = String::new();
    let Some(b) = w.squad_battle() else { return "No fight going on.\n".into() };
    let _ = writeln!(o, "FIGHT:");
    for (i, f) in b.fighters.iter().enumerate() {
        // A spent decoy or a fallen summoned beast is gone, not "down".
        if !f.is_person() && (f.ko || f.dead) {
            continue;
        }
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
    let _ = writeln!(o, "TALKING with {} ({} {}):", name_of(w, c.npc), w.people[c.npc as usize].race.name(), w.life(c.npc).job.title(w.people[c.npc as usize].seed).to_lowercase());
    for (npc, l) in c.lines.iter().rev().take(6).collect::<Vec<_>>().into_iter().rev() {
        let _ = writeln!(o, "  {} {}", if *npc { "»" } else { "  you:" }, gahturiyu_sim::sim::speech::plain(l));
    }
    if let Some(fee) = w.join_terms(c.npc) {
        let _ = writeln!(o, "  (Would join{}: {}.)", if fee > 0 { format!(" for {fee} coin") } else { " for nothing".into() }, w.recruit_card(c.npc));
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
    // What a thing does, where it does something (a ring, a potion).
    let does = |it: items::ItemId| -> String {
        let fx: Vec<String> = item(it).effects.iter().map(|e| e.describe()).collect();
        if fx.is_empty() { String::new() } else { format!(" — {}", fx.join("; ")) }
    };
    // What it would fetch in the town the squad is standing in: the number
    // a merchant there gives, not the round "worth".
    let fetch = |it: items::ItemId, pc: Option<&gahturiyu_sim::sim::materials::Piece>| -> String {
        match w.sells_for_held(m, it, pc) {
            Some((p, town)) if p > 0 => format!("; sells for {p} in {}", w.settlements[town as usize].name),
            Some((_, town)) => format!("; nobody in {} pays for it", w.settlements[town as usize].name),
            None => String::new(),
        }
    };
    for s in SLOTS {
        if let Some(it) = d.gear.in_slot(s) {
            let _ = writeln!(o, "   {:<10} {}{}  (worth ~{:.0}{})", format!("{s:?}"), item(it).name, does(it), item(it).value, fetch(it, d.gear.piece(s)));
        }
    }
    let _ = writeln!(o, " Pack:");
    for (k, e) in d.gear.bag.iter().enumerate() {
        let kind = match item(e.0).kind {
            items::Kind::Food(n) => format!("food, {n:.0}"),
            items::Kind::Coin => "money".into(),
            _ => String::new(),
        };
        let _ = writeln!(o, "   {k:>2}. {} × {} {}{}  (worth ~{:.0} each{})", e.1, item(e.0).name, if kind.is_empty() { String::new() } else { format!("[{kind}]") }, does(e.0), item(e.0).value, fetch(e.0, e.2.as_ref()));
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
        // (The window shows a band of bandits coming into sight in the news,
        // in red: the tool stops on it, as a player would look up.)
        if w.log.iter().take_while(|l| l.0 > start).any(|l| l.1.ends_with(": bandits.")) {
            why = "Bandits in sight.".into();
            break;
        }
        if let Some(who) = until_still {
            // (Nobody waits for someone who can't move: NM-72.)
            let still = who.iter().all(|&m| unable(w, m).is_some() || w.squad.index(m).map(|k| w.squad.at[k].dist(w.squad.goal[k]) < 0.5).unwrap_or(true));
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

/// Further than this from everyone ordered, someone isn't "here" to be
/// talked to (the lists in `look` reach 70 m).
const TALK_FAR: f32 = 150.0;

fn run(w: &mut World, s: &mut Session, cmd: &str, a: &[&str], save: &Path) -> String {
    let mut o = String::new();
    let mut sel = who(w, s);
    // An order goes to those who can take it, and is refused at once, with
    // the reason, when none of them can (NM-72): not the downed, not the bound.
    const ORDERS: &[&str] = &["go", "attack", "talk", "loot", "search", "pickup", "gather", "work", "butcher", "hunt", "enter", "carry", "sneak", "rest", "torch"];
    if ORDERS.contains(&cmd) {
        let why: Vec<String> = sel.iter().filter_map(|&m| unable(w, m)).collect();
        sel.retain(|&m| unable(w, m).is_none());
        if sel.is_empty() {
            return format!("Nobody can: {}.\n", why.join("; "));
        }
        // (Said only when they were picked by name.)
        if !why.is_empty() && !s.selected.is_empty() {
            o += &format!("({}.)\n", why.join("; "));
        }
    }
    let sel = sel;
    let lead = sel.first().copied().unwrap_or(w.squad.members[0]);
    let nearest = |w: &World, at: V2| sel.iter().copied().min_by(|&x, &y| w.person_pos(x).dist(at).total_cmp(&w.person_pos(y).dist(at))).unwrap_or(lead);
    let arg = |i: usize| a.get(i).copied().unwrap_or("");
    // Most orders walk someone somewhere: let the world run till they're there.
    let walk_then_look = |w: &mut World, s: &mut Session, who: &[PersonId], o: &mut String| {
        *o += &pass(w, 30.0 * 60.0, Some(who));
        *o += &look(w, s);
    };
    match cmd {
        "look" => o += &look_only(w, s, arg(0)),
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
                // Measured from whoever is ordered (NM-16).
                (!dx.is_nan()).then(|| focus(w, s).add(V2::new(dx * m, dy * m)))
            };
            // Only real numbers make a place.
            let target = target.filter(|t| t.x.is_finite() && t.y.is_finite());
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
                        o += &if !why.is_empty() {
                            format!("Nobody moves: {}.\n", why.join("; "))
                        } else if w.terrain.is_sea(t) {
                            "That's the sea: the squad can't cross it.\n".to_string()
                        } else {
                            "Nobody needs to move: they're already there.\n".to_string()
                        };
                    }
                    if moving && w.terrain.is_sea(t) && w.building_at(t).is_none() {
                        o += "That's the sea: they'll stop at the water's edge.\n";
                    }
                    walk_then_look(w, s, &sel, &mut o);
                }
                None => o += "Go where? (an id from `look`, X,Y, or a direction and metres)\n",
            }
        }
        "attack" => {
            let p = person_arg(w, arg(0));
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
            let Some(p) = person_arg(w, arg(0)) else { return "Can't talk to them.\n".into() };
            // Already talking with them: as quick as the first time (NM-73).
            if w.talk.as_ref().is_some_and(|c| c.npc == p) {
                return talk_view(w);
            }
            let at = w.person_pos(p);
            // Whoever of those ordered is nearest goes, not always the first.
            let m = nearest(w, at);
            let them = &w.people[p as usize];
            if them.dead || w.is_down(p) {
                return "They're in no state to talk.\n".into();
            }
            if w.is_indoors_asleep(p) {
                return "They're abed.\n".into();
            }
            if w.person_pos(m).dist(at) > TALK_FAR {
                return "They aren't here.\n".into();
            }
            // One talk at a time: leave the other first, and say so.
            if let Some(other) = w.talk.as_ref().map(|c| c.npc) {
                w.ask(Topic::Goodbye);
                w.end_talk();
                o += &format!("You take your leave of {}.\n", first_name(w, other));
            }
            if !w.order_talk(m, p) {
                return o + "Can't talk to them.\n";
            }
            let start = w.time;
            while w.talk.as_ref().map(|c| c.npc) != Some(p) && w.time < start + 10.0 * 60.0 && w.squad_battle().is_none() {
                w.step(0.25);
            }
            let mins = ((w.time - start) / 60.0).round();
            if mins >= 1.0 {
                o += &format!("({mins:.0} minutes pass.)\n");
            }
            o += &if w.talk.is_some() { talk_view(w) } else { format!("{} couldn't get to them.\n", first_name(w, m)) };
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
                let p = person_arg(w, arg(0));
                p.is_some_and(|p| {
                    let m = nearest(w, w.person_pos(p));
                    w.order_loot(m, p)
                })
            } else {
                arg(0).strip_prefix('k').and_then(container_id).is_some_and(|c| {
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
                match (cmd, person_arg(w, arg(0))) {
                    ("loot", Some(p)) => o += &format!("Can't: {}\n", w.why_cant_loot(p)),
                    _ => o += "Can't go through that (is it locked, or not in a building you're in?).\n",
                }
            } else {
                let looter = w.looting.last().map(|l| l.who).or_else(|| w.picking.last().map(|p| p.who)).unwrap_or(lead);
                let mut n = 0;
                while w.source_now(looter).is_none() && n < 2400 && w.chased_by(looter).is_none() && w.free_to_order(looter) {
                    // Still at a lock: keep going while they have picks.
                    if w.looting.iter().all(|l| l.who != looter) && w.picking.iter().all(|p| p.who != looter) {
                        break;
                    }
                    w.step(0.25);
                    n += 1;
                }
                o += &news(w, s);
                if w.source_now(looter).is_none() {
                    let name = first_name(w, looter);
                    if let Some(g) = w.chased_by(looter) {
                        o += &format!("{name} was seen: {} of the watch is after them.\n", first_name(w, g));
                    } else if !w.free_to_order(looter) {
                        o += &format!("{name} was caught and is bound to work it off (see News).\n");
                    } else if w.count_of(looter, "lockpick") == 0 && arg(0).starts_with('k') {
                        o += &format!("{name} has no lockpicks left; the lock holds.\n");
                    } else {
                        o += &loot_view(w, looter);
                    }
                } else {
                    o += &loot_view(w, looter);
                }
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
            let m = match (a.is_empty(), member(w, arg(0))) {
                (true, _) => lead,
                (false, Some(m)) => m,
                (false, None) => return format!("Nobody in the squad is called {}.\n", arg(0)),
            };
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
            let (Some(m), Some(n)) = (member(w, arg(0)), arg(1).parse::<usize>().ok().filter(|&n| n >= 1)) else {
                return "Usage: cast NAME N [pID | X,Y] (N from `spells NAME`)\n".into();
            };
            let Some(&sp) = w.known_spells(m).get(n - 1) else { return "They don't know that one.\n".into() };
            let t = arg(2);
            if t.starts_with('p') && person_arg(w, t).is_none() {
                return "Can't: nobody there.\n".into();
            }
            let target = person_arg(w, t);
            let point = match target {
                Some(p) => Some(w.person_pos(p)),
                None => t.split_once(',').and_then(|(x, y)| Some(V2::new(x.parse().ok()?, y.parse().ok()?))).filter(|p: &V2| p.x.is_finite() && p.y.is_finite()).or(Some(w.person_pos(m))),
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
                        // A line from the list, by its number (NM-69).
                        let Some(n) = arg(0).parse::<usize>().ok().filter(|&n| n >= 1) else {
                            return format!("Usage: take N (a line of the list below), or takeall\n{}", loot_view(w, m));
                        };
                        let what = w.contents(src).get(n - 1).map(|x| x.0);
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
            // Each takes its own kind of id (g: on the ground, n: a plant or
            // rock, d: a woodlot or mine, c: a carcass).
            let letter = match cmd {
                "pickup" => 'g',
                "gather" => 'n',
                "work" => 'd',
                _ => 'c',
            };
            let Some(num) = id_num::<u32>(id, letter) else {
                return format!("`{cmd}` takes a {letter} id from `look` (like {letter}3).\n");
            };
            let at = pos_of(w, id).unwrap_or(w.squad.pos);
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
            let h: u32 = id_num(arg(0), 'h').unwrap_or(u32::MAX);
            if w.order_hunt(&sel, h) {
                o += &pass(w, 15.0 * 60.0, None);
                o += &look(w, s);
            } else {
                o += "Can't hunt that.\n";
            }
        }
        "enter" => {
            let id = arg(0).strip_prefix('b').unwrap_or("");
            let door = id.split_once('.').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
            let is_in = |w: &World, m: PersonId, id| w.squad.index(m).is_some_and(|k| w.squad.inside[k] == Some(id));
            let who_inside = |w: &World, id, name: &str| {
                let inside: Vec<String> = sel.iter().copied().filter(|&m| is_in(w, m, id)).map(|m| first_name(w, m)).collect();
                if inside.is_empty() { format!("Nobody got inside the {name}.\n") } else { format!("{} {} inside the {name}.\n", inside.join(", "), if inside.len() == 1 { "is" } else { "are" }) }
            };
            match door.and_then(|d| w.door(d)) {
                // Already in: nothing to pick or walk.
                Some(d) if !sel.is_empty() && sel.iter().all(|&m| is_in(w, m, d.id)) => o += &format!("Already inside the {}.\n", d.variant().name),
                Some(d) if w.is_locked(d.id) => {
                    let pick = items::id("lockpick");
                    let picker = sel.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().is_some_and(|x| x.gear.bag.iter().any(|e| e.0 == pick)));
                    match picker {
                        Some(m) => {
                            w.order_pick(m, d.id);
                            o += &pass(w, 5.0 * 60.0, None);
                            // Picked: in they go.
                            if !w.is_locked(d.id) && w.picking.iter().all(|p| p.who != m) {
                                w.order_members(&sel, d.centre);
                                o += &pass(w, 30.0 * 60.0, Some(&sel));
                                o += &who_inside(w, d.id, d.variant().name);
                            }
                            o += &look(w, s);
                        }
                        None => o += "It's locked, and nobody selected has a lockpick.\n",
                    }
                }
                Some(d) => {
                    w.order_members(&sel, d.centre);
                    o += &pass(w, 30.0 * 60.0, Some(&sel));
                    o += &who_inside(w, d.id, d.variant().name);
                    o += &look(w, s);
                }
                None => o += "No such building.\n",
            }
        }
        "carry" => {
            let p = person_arg(w, arg(0));
            match p.and_then(|p| sel.iter().copied().find(|&m| w.can_carry(m, p)).map(|m| (m, p))) {
                Some((m, p)) => {
                    w.order_carry(m, p);
                    walk_then_look(w, s, &[m], &mut o);
                }
                None if p.is_some_and(|p| w.fighting.contains_key(&p)) || sel.iter().any(|m| w.fighting.contains_key(m)) => o += "Not while the fight is on.\n",
                None => o += "Nobody selected can carry them.\n",
            }
        }
        "putdown" => {
            let mut any = false;
            for m in sel.clone() {
                any |= w.put_down(m);
            }
            o += if any { "Put down.\n" } else { "Nobody selected is carrying anyone.\n" };
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
            o += "Resting.\n";
        }
        "wake" => {
            // Only those resting or asleep get up; the downed stay down, and
            // it says so (NM-72).
            let down: Vec<String> = sel.iter().filter(|&&m| w.is_down(m)).map(|&m| first_name(w, m)).collect();
            let up: Vec<PersonId> = sel.iter().copied().filter(|&m| !w.is_down(m) && (w.is_asleep(m) || w.squad.index(m).is_some_and(|k| w.squad.resting[k]))).collect();
            w.order_wake(&up);
            let is = |n: usize| if n == 1 { "is" } else { "are" };
            if !up.is_empty() {
                let names: Vec<String> = up.iter().map(|&m| first_name(w, m)).collect();
                o += &format!("{} {} up.\n", names.join(", "), is(names.len()));
            }
            if !down.is_empty() {
                o += &format!("{} {} down, and can't be woken.\n", down.join(", "), is(down.len()));
            }
            if up.is_empty() && down.is_empty() {
                o += "Nobody selected is resting.\n";
            }
        }
        "torch" => {
            let (mut lit, mut out) = (0, 0);
            for &m in &sel {
                if w.torch_in_hand(m).is_some() || w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| item(e.0).key == "torch")) {
                    let had = w.torch_in_hand(m).is_some();
                    if w.toggle_torch(m) {
                        if had {
                            out += 1;
                        } else {
                            lit += 1;
                        }
                    }
                }
            }
            o += &match (lit, out) {
                (0, 0) if w.squad_battle().is_some() => "Not while fighting.\n".to_string(),
                (0, 0) => "Nobody selected has a torch to light.\n".to_string(),
                (l, 0) => format!("{l} torch{} lit.\n", if l == 1 { "" } else { "es" }),
                (0, x) => format!("{x} torch{} put out.\n", if x == 1 { "" } else { "es" }),
                (l, x) => format!("{l} lit, {x} put out.\n"),
            };
        }
        "pack" => {
            let m = match (a.is_empty(), member(w, arg(0))) {
                (true, _) => lead,
                (false, Some(m)) => m,
                (false, None) => return format!("Nobody in the squad is called {}.\n", arg(0)),
            };
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
                        if t.starts_with('p') && person_arg(w, t).is_none() {
                            return "Can't: nobody there.\n".into();
                        }
                        let target = person_arg(w, t);
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
                o += &format!("That didn't work{}.\n", why.map(|y| format!(": {}", y.trim_end_matches('.'))).unwrap_or_default());
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
            match w.order_give(m, k, to) {
                Ok(line) => {
                    o += &format!("{line}\n");
                    // If they had to walk over, see it through.
                    let mut n = 0;
                    while !w.giving.is_empty() && n < 2400 {
                        w.step(0.25);
                        n += 1;
                    }
                    if n > 0 {
                        o += &news(w, s);
                    }
                }
                Err(why) => o += &format!("Can't: {why}\n"),
            }
            o += &pack(w, m);
        }
        "unequip" => {
            let (Some(m), false) = (member(w, arg(0)), arg(1).is_empty()) else {
                return "Usage: unequip NAME SLOT (a slot as `pack NAME` names it, like head or mainhand)\n".into();
            };
            let slot = SLOTS.iter().copied().find(|s| format!("{s:?}").to_lowercase().starts_with(&arg(1).to_lowercase()));
            let ok = slot.is_some_and(|s| w.unequip(m, s));
            o += if ok { "Taken off.\n" } else { "Nothing there.\n" };
            o += &pack(w, m);
        }
        "craft" | "make" => {
            use gahturiyu_sim::sim::crafting::RECIPES;
            let Some(m) = member(w, arg(0)) else { return "Who?\n".into() };
            let grade = gahturiyu_sim::sim::materials::Grade::Common;
            let made = |i: usize| item(RECIPES[i].item(grade)).name.to_string();
            if cmd == "make" {
                let n: usize = arg(1).parse().unwrap_or(usize::MAX);
                if n >= RECIPES.len() {
                    return "Usage: make NAME N (N from `craft NAME`)\n".into();
                }
                // Everything in the way at once, in plain words.
                let stops = w.craft_blockers(m, n);
                match w.start_craft(m, n) {
                    Ok(()) => {
                        let r = &RECIPES[n];
                        let used: Vec<String> = r.inputs.iter().map(|(k, c)| format!("{c} × {}", item(items::id(k)).name.to_lowercase())).collect();
                        let hours = r.time / HOUR;
                        let long = if hours >= 24.0 { format!("{:.0} days", hours / 24.0) } else if hours >= 1.0 { format!("{hours:.1} hours") } else { format!("{:.0} minutes", hours * 60.0) };
                        o += &format!("{} starts on {}: {long}. Used {}.\n", first_name(w, m), made(n).to_lowercase(), if used.is_empty() { "nothing".to_string() } else { used.join(", ") });
                    }
                    Err(_) => o += &format!("Can't make {}: {}.\n", made(n).to_lowercase(), stops.iter().map(|c| c.say()).collect::<Vec<_>>().join("; ")),
                }
            } else {
                // As the window's craft panel has it: the crafts they've
                // taken up, each thing with all it takes and what's in the
                // way; the crafts they haven't, in one line. `craft NAME all`
                // lists every recipe.
                use gahturiyu_sim::sim::materials::CRAFTS;
                let all = arg(1) == "all";
                let st = w.people[m as usize].effective_stats();
                let (known, unknown): (Vec<_>, Vec<_>) = CRAFTS.iter().copied().partition(|&k| w.knows_craft(m, k));
                let _ = writeln!(o, "What {} can make (`make {} N`):", first_name(w, m), first_name(w, m));
                let mut last = None;
                for (i, r) in RECIPES.iter().enumerate() {
                    if !all && !w.knows_craft(m, r.craft()) {
                        continue;
                    }
                    if last != Some(r.skill) {
                        let _ = writeln!(o, " {} ({:.0}):", r.skill.name(), st.skill(r.skill));
                        last = Some(r.skill);
                    }
                    let takes: Vec<String> = r.inputs.iter().map(|(k, n)| format!("{}/{} {}", w.count_of(m, k).min(*n), n, item(items::id(k)).name.to_lowercase())).collect();
                    let stops = w.craft_blockers(m, i);
                    // The counts already say what's short; the rest is where
                    // it's made and anything else in the way.
                    let other: Vec<String> = stops.iter().filter(|c| !matches!(c, gahturiyu_sim::sim::crafting::Cannot::Missing(..))).map(|c| c.say()).collect();
                    let state = if stops.is_empty() {
                        format!("ready ({:.0}% to come out right)", gahturiyu_sim::sim::crafting::success_chance(st.skill(r.skill), r.difficulty) * 100.0)
                    } else if other.is_empty() {
                        "short of materials".to_string()
                    } else {
                        other.join("; ")
                    };
                    let _ = writeln!(o, "  {i:>2}. {}{} — takes {} — {state}", made(i), if r.makes > 1 { format!(" ×{}", r.makes) } else { String::new() }, if takes.is_empty() { "nothing".to_string() } else { takes.join(", ") });
                }
                if !all && !unknown.is_empty() {
                    let _ = writeln!(o, " Not taken up yet (a crafter at work or a manual teaches): {}. (`craft {} all` lists everything.)", unknown.iter().map(|k| k.skill().name()).collect::<Vec<_>>().join(", "), first_name(w, m));
                }
                if known.is_empty() && !all {
                    o += " No crafts taken up yet.\n";
                }
            }
        }
        "journal" => {
            let _ = writeln!(o, "Journal:");
            for q in &w.quests {
                let _ = writeln!(o, "  [{:?}] {}", q.stage, w.quest_line(q));
            }
            for l in w.work_lines() {
                let _ = writeln!(o, "  [Town work] {l}");
            }
            if w.quests.is_empty() && w.society.contracts.is_empty() {
                o += "  No jobs taken.\n";
            }
        }
        "map" => {
            let here = focus(w, s);
            let mut towns: Vec<_> = w.settlements.iter().collect();
            towns.sort_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here)));
            // `map NAME` finds a town by name, however far (a town heard of
            // in talk can be looked up); plain `map` lists the nearest.
            let want = gahturiyu_sim::names::plain(&arg(0).to_lowercase());
            if !want.is_empty() {
                let found: Vec<_> = towns.iter().filter(|t| gahturiyu_sim::names::plain(&t.name.to_lowercase()).contains(&want)).collect();
                if found.is_empty() {
                    let _ = writeln!(o, "No town called that.");
                }
                for t in found {
                    let _ = writeln!(o, "  t{}  {} ({} people) — {}", t.id, t.name, t.residents.len(), dist_dir(here, t.pos));
                }
                return o;
            }
            let _ = writeln!(o, "Towns, nearest first (`map NAME` finds one by name):");
            for t in towns.iter().take(12) {
                let _ = writeln!(o, "  t{}  {} ({} people) — {}", t.id, t.name, t.residents.len(), dist_dir(here, t.pos));
            }
            let _ = writeln!(o, "Ruins and lairs (diamonds on the map):");
            for r in &w.ruins {
                let _ = writeln!(o, "  r{}  {}{} — {}", r.id, r.name(w), if w.ruin_held(r.id) { "" } else { " (unguarded)" }, dist_dir(here, r.pos));
            }
            let _ = writeln!(o, "Woodlots and mines, nearest first:");
            let mut faces: Vec<_> = w.deposits.iter().collect();
            faces.sort_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here)));
            for d in faces.iter().take(8) {
                let _ = writeln!(o, "  d{}  {} of {} — {:.0} {} left — {}", d.id, d.face().name, w.settlements[d.town as usize].name, d.left_at(w.time).floor(), item(d.item).name.to_lowercase(), dist_dir(here, d.pos));
            }
            let _ = writeln!(o, "Bandit camps known (red triangles):");
            let mut camps: Vec<V2> = w.camps.iter().map(|c| c.pos).collect();
            camps.sort_by(|a, b| a.dist(here).total_cmp(&b.dist(here)));
            for c in camps.iter().take(6) {
                let _ = writeln!(o, "  a camp — {}", dist_dir(here, *c));
            }
        }
        "town" => {
            let here = focus(w, s);
            if let Some(t) = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))) {
                let _ = writeln!(o, "{} — {} people, {}", t.name, t.residents.len(), dist_dir(here, t.pos));
                let mut jobs: std::collections::BTreeMap<&str, usize> = Default::default();
                for &p in &t.residents {
                    *jobs.entry(w.life(p).job.name()).or_default() += 1;
                }
                let list: Vec<String> = jobs.iter().map(|(j, n)| format!("{j} {n}")).collect();
                let _ = writeln!(o, "  Trades: {}", list.join(", "));
                let tl = &w.society.towns[t.id as usize];
                let _ = writeln!(o, "  Out of town:");
                let fields = tl.places.iter().filter(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields).count();
                let mut field_shown = false;
                for (i, wp) in tl.places.iter().enumerate().filter(|(_, p)| p.kind.is_away()) {
                    if wp.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields {
                        if !field_shown {
                            field_shown = true;
                            let _ = writeln!(o, "    w{}.{i} Fields ({fields} plots round the town) — {}", t.id, dist_dir(here, wp.pos));
                        }
                        continue;
                    }
                    let face: Vec<String> = w.deposits.iter().filter(|d| d.pos.dist(wp.pos) < 15.0).map(|d| format!("d{} {} {:.0} {} left", d.id, d.face().name.to_lowercase(), d.left_at(w.time).floor(), item(d.item).name.to_lowercase())).collect();
                    let face = if face.is_empty() { String::new() } else { format!(" ({})", face.join("; ")) };
                    let _ = writeln!(o, "    w{}.{i} {}{face} — {}", t.id, wp.kind.name(), dist_dir(here, wp.pos));
                }
                let posts = w.vacant_posts(t.id);
                if !posts.is_empty() {
                    let _ = writeln!(o, "  Work going (ask an official or the hall):");
                    for (job, pl) in posts {
                        let at = tl.places.get(pl as usize).map(|p| p.kind.name().to_lowercase()).unwrap_or_default();
                        let _ = writeln!(o, "    {} at the {at} — {} coin a day, 8 till 5", job.name(), w.post_wage(job));
                    }
                }
                let willing = w.willing_in(t.id);
                let _ = writeln!(o, "  Willing to join ({}):", willing.len());
                for (p, fee) in willing {
                    let price = if fee > 0 { format!("{fee} coin") } else { "for nothing".into() };
                    let at = if w.is_indoors_asleep(p) { "asleep indoors".to_string() } else { dist_dir(here, w.person_pos(p)) };
                    let _ = writeln!(o, "    p{p} {} ({}), {price} — {at}\n        {}", name_of(w, p), w.life(p).job.name().to_lowercase(), w.recruit_card(p));
                }
                let _ = writeln!(o, "  Merchants:");
                for &p in &t.residents {
                    match w.trades_next(p) {
                        Some(at) if at <= w.time + 1.0 => {
                            let _ = writeln!(o, "    p{p} {} — trading now", name_of(w, p));
                        }
                        Some(at) => {
                            let _ = writeln!(o, "    p{p} {} — at their stall from {}{}", name_of(w, p), hhmm(at), w.day_word(at));
                        }
                        None => {}
                    }
                }
            }
        }
        "build" => o += &build(w, lead, a),
        "base" => o += &base_cmd(w, lead, a),
        "wait" => {
            // A whole number of minutes, a week at most.
            const MOST: f64 = 7.0 * 24.0 * 60.0;
            let Some(m) = arg(0).parse::<f64>().ok().filter(|m| m.is_finite() && *m >= 1.0 && m.fract() == 0.0) else {
                return "Usage: wait M (whole minutes, 1 to 10080: a week at most)\n".into();
            };
            if m > MOST {
                return "That's too long: a week (10080 minutes) at most.\n".into();
            }
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

// ---- Base building (the window's Build panel and Base tab) -------------------------

fn dir_of(d: &str) -> Option<V2> {
    Some(match d {
        "n" | "north" => V2::new(0.0, -1.0),
        "s" | "south" => V2::new(0.0, 1.0),
        "e" | "east" => V2::new(1.0, 0.0),
        "w" | "west" => V2::new(-1.0, 0.0),
        "ne" => V2::new(0.707, -0.707),
        "nw" => V2::new(-0.707, -0.707),
        "se" => V2::new(0.707, 0.707),
        "sw" => V2::new(-0.707, 0.707),
        _ => return None,
    })
}

fn metres(s: &str) -> Option<f32> {
    s.parse::<f32>().ok().filter(|m| m.is_finite() && m.abs() <= 5000.0)
}

fn build(w: &mut World, lead: PersonId, a: &[&str]) -> String {
    use gahturiyu_sim::sim::base::{def_index, Kind, Plan, BUILDINGS};
    let arg = |i: usize| a.get(i).copied().unwrap_or("");
    let here = w.person_pos(lead);
    let mut o = String::new();
    if arg(0).is_empty() {
        o += "What can be built (`build KEY`; the builder has to stand at the base):\n";
        for (i, d) in BUILDINGS.iter().enumerate() {
            let needs: Vec<String> = d.needs.iter().map(|&(k, n)| format!("{n} {}", item(items::id(k)).name)).collect();
            let who: Vec<String> = w.squad_builders(i).iter().map(|&m| first_name(w, m)).collect();
            let _ = writeln!(o, "  {:<16} {} [{}] — {} — needs {}; {:.0} builder-hours; can build: {}", d.key, d.name, d.group, d.does, if needs.is_empty() { "nothing".into() } else { needs.join(", ") }, d.labour, if who.is_empty() { "nobody in the squad".into() } else { who.join(", ") });
        }
        return o;
    }
    if arg(0) == "kit" {
        // For testers only (the window's GAHT_BUILD demo does the same): the
        // squad is taught carpentry and masonry and the first selected
        // member is handed materials and coin, so a base can be tried
        // without days of gathering first.
        use gahturiyu_sim::sim::stats::Skill;
        for m in w.squad.members.clone() {
            let p = &mut w.people[m as usize];
            for sk in [Skill::Carpentry, Skill::Masonry] {
                if let Some(d) = p.detail.as_mut() {
                    if !d.crafts.contains(&sk) {
                        d.crafts.push(sk);
                    }
                }
                if p.stats.skill(sk) < 50.0 {
                    p.stats.set_skill(sk, 50.0);
                }
            }
        }
        if let Some(d) = w.people[lead as usize].detail.as_mut() {
            for (k, n) in [("timber", 80), ("rock", 50), ("seareed", 30), ("clay", 12), ("iron_ingot", 3), ("coin", 300), ("flatbread", 20), ("grain", 10)] {
                d.gear.add(items::id(k), n);
            }
        }
        w.people[lead as usize].recompute_might();
        return format!("TESTERS' KIT (not part of the game): everyone knows carpentry and masonry; {} carries 80 timber, 50 rock, 30 seareed, 12 clay, 3 iron ingots, 300 coin, 20 flatbread and 10 grain.\n", first_name(w, lead));
    }
    if arg(0) == "camp" {
        return match w.found_base(here) {
            Ok(id) => format!("A camp marker is laid where {} stands: {}.\n", first_name(w, lead), w.base(id).map(|b| b.name.clone()).unwrap_or_default()),
            Err(e) => format!("Can't found a base here: {}.\n", e.why()),
        };
    }
    if arg(0) == "wall" {
        let Some(def) = BUILDINGS.iter().position(|d| d.key == arg(1) && d.kind == Kind::Wall) else { return "Which wall? (`build wall palisade e 12 n 12`, or rubble_wall)\n".into() };
        let mut pts = vec![here];
        let mut k = 2;
        while k + 1 < a.len() + 1 && !arg(k).is_empty() {
            let (Some(d), Some(m)) = (dir_of(arg(k)), metres(arg(k + 1))) else { return "A wall's legs are a direction and metres each (`build wall palisade e 12 n 12`).\n".into() };
            let last = *pts.last().unwrap();
            pts.push(last.add(d.scale(m)));
            k += 2;
        }
        let planned = World::wall_plans(def, &pts).len();
        return match w.place_wall(def, &pts) {
            Ok(ids) => format!("Laid {} of {} wall pieces (ids {}).\n", ids.len(), planned, ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ")),
            Err(e) => format!("Can't lay that wall: {}.\n", e.why()),
        };
    }
    let Some(def) = BUILDINGS.iter().position(|d| d.key == arg(0)) else { return format!("No building called `{}` (`build` lists them).\n", arg(0)) };
    let (at, next) = match (dir_of(arg(1)), metres(arg(2))) {
        (Some(d), Some(m)) => (here.add(d.scale(m)), 3),
        _ => (here, 1),
    };
    let rot = arg(next).parse::<f32>().ok().filter(|r| r.is_finite()).unwrap_or(0.0).to_radians();
    if def == def_index("camp_marker") {
        return "Use `build camp`.\n".into();
    }
    match w.place_building(Plan { def, at, rot, w: BUILDINGS[def].w, replaces: None }) {
        Ok(id) => format!("{} site laid (id {id}). `base` shows how it's coming on.\n", BUILDINGS[def].name),
        Err(e) => format!("Can't put a {} there: {}.\n", BUILDINGS[def].name.to_lowercase(), e.why()),
    }
}

fn base_cmd(w: &mut World, lead: PersonId, a: &[&str]) -> String {
    use gahturiyu_sim::sim::baselife::{round_place, JOBS};
    let arg = |i: usize| a.get(i).copied().unwrap_or("");
    let here = w.person_pos(lead);
    let mut o = String::new();
    // A resident by first name (they aren't in the travelling squad).
    let resident = |w: &World, n: &str| -> Option<PersonId> {
        let n = strip(&n.to_lowercase());
        w.bases.iter().flat_map(|b| b.residents.iter().map(|r| r.who)).find(|&p| !n.is_empty() && strip(&first_name(w, p).to_lowercase()).starts_with(&n))
    };
    let at_base = w.base_at(here);
    match arg(0) {
        "" => {
            if w.bases.is_empty() {
                return "No base yet. `build camp` founds one where the first selected member stands.\n".into();
            }
            for b in &w.bases {
                let _ = writeln!(o, "{} — {} — camp marker {} — builds within {:.0} m", b.name, match b.land { gahturiyu_sim::sim::base::Land::Wilds => "in the wilds".to_string(), gahturiyu_sim::sim::base::Land::Unclaimed(s) => format!("on {}'s land", w.settlements[s as usize].name) }, dist_dir(here, b.at), b.reach());
                o += " Buildings:\n";
                for bl in &b.buildings {
                    let what = if let Some(site) = bl.site() {
                        let missing: Vec<String> = bl.missing().iter().map(|&(it, n)| format!("{n} {}", item(it).name)).collect();
                        let when = match site.done_at {
                            Some(t) => format!("stands in {:.1} h", (t - w.time) / HOUR),
                            None if !missing.is_empty() => "waiting for materials".to_string(),
                            None => "nobody is building it".to_string(),
                        };
                        format!("site, {:.0}% built, {}{}", bl.progress(w.time) * 100.0, when, if missing.is_empty() { String::new() } else { format!(" (still needs {})", missing.join(", ")) })
                    } else if bl.standing() {
                        format!("standing, {:.0}% sound{}", bl.hp_at(w.time) / bl.def().hp * 100.0, if bl.sealed { ", sealed" } else if bl.rots() { ", thatch rotting" } else { "" })
                    } else {
                        "a ruin".to_string()
                    };
                    let _ = writeln!(o, "  {:>3}  {} — {} — {}", bl.id, bl.def().name, what, dist_dir(here, bl.at));
                }
                let store: Vec<String> = b.store.iter().map(|e| format!("{} × {}", e.1, item(e.0).name)).collect();
                let _ = writeln!(o, " Store ({:.0} of {:.0} kg): {}", b.load().max(0.0), b.capacity(), if store.is_empty() { "empty".into() } else { store.join(", ") });
                let wages: u32 = b.residents.iter().filter_map(|r| r.hire.as_ref()).map(|h| h.wage as u32).sum();
                let _ = writeln!(o, " Living here ({} beds){}:", b.beds(), if wages > 0 { format!(", wages {wages} coin at each dawn") } else { String::new() });
                for r in &b.residents {
                    let doing = match (&r.cycle, r.job) {
                        (Some(cy), _) => format!("at the {}, done in {:.1} h", round_place(b, cy), (cy.done_at - w.time) / HOUR),
                        (None, j) => format!("{} (nothing under way)", j.name().to_lowercase()),
                    };
                    let hire = match &r.hire {
                        Some(h) if h.arrives > w.time => format!(" — hired, {} a day, on the way (here in {:.1} h)", h.wage, (h.arrives - w.time) / HOUR),
                        Some(h) => format!(" — hired, {} a day, loyalty {:.0}%{}", h.wage, h.loyalty * 100.0, if h.owed > 0 { format!(", owed {}", h.owed) } else { String::new() }),
                        None => " — one of yours".to_string(),
                    };
                    let _ = writeln!(o, "  {} — {} — {}{} — hunger {:.0}{}", first_name(w, r.who), r.job.name(), doing, hire, w.hunger_of(r.who).unwrap_or(0.0), if w.is_down(r.who) { " — DOWN" } else { "" });
                }
                if b.residents.is_empty() {
                    o += "  nobody\n";
                }
                o += " Lately:\n";
                for (t, line) in b.log.iter().rev().take(8) {
                    let _ = writeln!(o, "  Day {}, {}  {line}", World::day_of(*t) + 1, hhmm(*t));
                }
            }
        }
        "store" => match at_base {
            Some(bid) => {
                let n = w.store_materials(bid);
                let _ = writeln!(o, "{n} things put in the store.");
            }
            None => o += "Nobody selected is standing at a base.\n",
        },
        "leave" => match (member(w, arg(1)), at_base) {
            (Some(m), Some(bid)) => match w.leave_at_base(m, bid) {
                Ok(()) => { let _ = writeln!(o, "{} stays at the base.", first_name(w, m)); }
                Err(e) => { let _ = writeln!(o, "Can't: {e}."); }
            },
            (None, _) => o += "Leave whom? (a squad member's first name)\n",
            (_, None) => o += "Nobody selected is standing at a base.\n",
        },
        "fetch" => match resident(w, arg(1)) {
            Some(p) => match w.pick_up(p) {
                Ok(()) => { let _ = writeln!(o, "{} is with the squad again (a hired hand is let go).", first_name(w, p)); }
                Err(e) => { let _ = writeln!(o, "Can't: {e}."); }
            },
            None => o += "Fetch whom? (the first name of someone living at a base)\n",
        },
        "job" => match (resident(w, arg(1)), JOBS.iter().copied().find(|j| j.name().eq_ignore_ascii_case(arg(2)))) {
            (Some(p), Some(j)) => {
                w.set_base_job(p, j);
                let _ = writeln!(o, "{} is now: {}.", first_name(w, p), j.name());
            }
            _ => { let _ = writeln!(o, "`base job NAME JOB`: JOB is one of {}.", JOBS.iter().map(|j| j.name().to_lowercase()).collect::<Vec<_>>().join(", ")); }
        },
        "recipe" => match resident(w, arg(1)) {
            Some(p) => {
                let list = w.base_recipes(p);
                match arg(2).parse::<usize>() {
                    Ok(n) if n >= 1 && n <= list.len() => {
                        w.set_base_recipe(p, Some(list[n - 1]));
                        let _ = writeln!(o, "{} will make: {}.", first_name(w, p), gahturiyu_sim::sim::crafting::RECIPES[list[n - 1] as usize].output);
                    }
                    _ => {
                        let _ = writeln!(o, "What {} could make here (`base recipe NAME N`):", first_name(w, p));
                        for (k, &ri) in list.iter().enumerate() {
                            let _ = writeln!(o, "  {}. {}", k + 1, gahturiyu_sim::sim::crafting::RECIPES[ri as usize].output);
                        }
                        if list.is_empty() {
                            o += "  nothing\n";
                        }
                    }
                }
            }
            None => o += "Whose recipe? (the first name of someone living at a base)\n",
        },
        "seal" | "down" => match (at_base, arg(1).parse::<u32>()) {
            (Some(bid), Ok(id)) => {
                if arg(0) == "seal" {
                    match w.seal_building(bid, id) {
                        Ok(()) => o += "Sealed.\n",
                        Err(e) => { let _ = writeln!(o, "Can't: {e}."); }
                    }
                } else if w.deconstruct(bid, id) {
                    o += "Taken down.\n";
                } else {
                    o += "Can't take that down.\n";
                }
            }
            _ => o += "Stand at the base and give the building's number from `base`.\n",
        },
        _ => o += "`base`, or `base store | leave NAME | fetch NAME | job NAME JOB | recipe NAME [N] | seal ID | down ID`.\n",
    }
    o
}
