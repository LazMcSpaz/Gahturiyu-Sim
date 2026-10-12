//! The town panel (P, or click a town's name): what a visitor could learn
//! by looking and asking (#153 UI-Town, plan 5.6): the town's name and
//! people, who rules in words, what's open now and where, the work going,
//! and the talk of the town. No numbers about others.
//!
//! With the testing readout (L) the old panels come back instead: every
//! leaning and share, the store, the money, the government, all the talk
//! (`readout_panel`, `life_panel`).

use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    culture::{BELONGINGS, COOKINGS, RHYTHMS},
    jobs::{GOODS, SERVICES},
    race::ALL_RACES,
    society::PATH_NAMES,
    tide, World,
};

use super::hud::Canvas;
use super::palette::{eg, ega, race_color, Rgb, DIM, GOLD, TEXT, WARN};
use super::squadui::Bx;

const OPEN: Rgb = [0.55, 0.85, 0.5];
const W: f32 = 590.0;

fn pct(x: f32) -> String {
    format!("{:.0}%", x * 100.0)
}

/// A row of weighted options with the chosen one picked out.
fn leanings<T: PartialEq + Copy>(c: &Canvas, x: f32, y: f32, label: &str, opts: &[T], weights: &[f32], chosen: T, name: impl Fn(T) -> &'static str) {
    c.text(label, x, y, 13.0, DIM);
    let mut cx = x + 92.0;
    for (o, wgt) in opts.iter().zip(weights) {
        let s = format!("{} {}", name(*o), pct(*wgt));
        let col = if *o == chosen { GOLD } else { TEXT };
        let wd = c.width(&s, 13.0);
        if *o == chosen {
            c.rect(cx - 3.0, y - 13.0, wd + 6.0, 17.0, ega(GOLD, 0.14));
        }
        c.text(&s, cx, y, 13.0, col);
        cx += wd + 12.0;
    }
}

/// The testing readout of a town (L): everything, numbers and all.
pub fn readout_panel(c: &Canvas, w: &World, town: u16) -> Bx {
    let s = &w.settlements[town as usize];
    let tl = &w.society.towns[town as usize];
    let comms: Vec<u32> = std::iter::once(tl.shore).chain(tl.stilts).collect();
    let stocked = GOODS.iter().filter(|g| w.stock_now(town, **g) >= 0.5).count();
    let gov_rows = w.government(town).chambers.len() as f32 + 6.0;
    let h = 330.0 + 70.0 + gov_rows * 17.0 + 30.0 + comms.len() as f32 * 185.0 + ((stocked as f32 / 4.0).ceil() - 3.0).max(0.0) * 17.0;
    let r = Bx::new(c.w - W - 12.0, 12.0, W, h.min(c.h - 60.0));
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 4.0, eg(race_color(s.founders)));
    let x = r.x + 14.0;
    let mut y = r.y + 28.0;
    c.text(&s.name, x, y, 19.0, race_color(s.founders));
    let note = format!(
        "{}  ·  {} people  ·  {} roads{}",
        if s.coastal { "coast" } else { "inland" },
        s.residents.iter().filter(|&&p| !w.people[p as usize].dead).count(),
        tl.roads,
        if s.coastal { format!("  ·  {}", tide::tide_word(w.time)) } else { String::new() }
    );
    c.text(&note, r.x + r.w - c.width(&note, 13.0) - 14.0, y - 2.0, 13.0, DIM);
    y += 10.0;

    for &ci in &comms {
        let cm = &w.society.communities[ci as usize];
        y += 22.0;
        c.text(if cm.stilts { "The stilt village" } else { "The town" }, x, y, 15.0, TEXT);
        // Who lives here.
        let mut cx = x + 140.0;
        for r_ in ALL_RACES {
            let sh = cm.blend.share[r_.index()];
            let t = format!("{} {}", r_.name(), pct(sh));
            c.text(&t, cx, y, 13.0, if sh > 0.0 { race_color(r_) } else { DIM });
            cx += c.width(&t, 13.0) + 10.0;
        }
        y += 20.0;
        let k = cm.customs;
        leanings(c, x, y, "Who cooks", &COOKINGS, &cm.blend.cooking, k.cooking, |o| match o {
            gahturiyu_sim::sim::culture::Cooking::Household => "home",
            gahturiyu_sim::sim::culture::Cooking::Hearth => "hearth",
            gahturiyu_sim::sim::culture::Cooking::Deck => "deck",
        });
        y += 18.0;
        leanings(c, x, y, "Day's rhythm", &RHYTHMS, &cm.blend.rhythm, k.rhythm, |o| match o {
            gahturiyu_sim::sim::culture::Rhythm::Seasonal => "seasons",
            gahturiyu_sim::sim::culture::Rhythm::Bells => "bells",
            gahturiyu_sim::sim::culture::Rhythm::Tides => "tides",
            gahturiyu_sim::sim::culture::Rhythm::Irregular => "irregular",
        });
        y += 18.0;
        leanings(c, x, y, "Belonging", &BELONGINGS, &cm.blend.belonging, k.belonging, |o| match o {
            gahturiyu_sim::sim::culture::Belonging::Lineage => "lineage",
            gahturiyu_sim::sim::culture::Belonging::TierBlock => "tier block",
            gahturiyu_sim::sim::culture::Belonging::Village => "village",
            gahturiyu_sim::sim::culture::Belonging::Lodging => "lodging",
        });
        y += 18.0;
        let split = format!("Services: {} (leaning to split {})", if k.layout == gahturiyu_sim::sim::culture::Layout::Split { "split" } else { "combined" }, pct(cm.blend.split));
        c.text(&split, x, y, 13.0, TEXT);
        let inst: Vec<String> = k.institutions().map(|(r_, i)| format!("{} ({})", r_.name(), i.name())).collect();
        if !inst.is_empty() {
            y += 18.0;
            c.text(&format!("Strong minorities: {}", inst.join(", ")), x, y, 13.0, TEXT);
        }
        // Food, by path.
        y += 22.0;
        let f = &cm.food;
        c.text(&format!("Fed {}", pct(f.overall)), x, y, 14.0, if f.overall < 0.8 { WARN } else { OPEN });
        let mut cx = x + 92.0;
        for i in 0..3 {
            if f.share[i] <= 0.0 {
                continue;
            }
            let name = if i == 1 { cm.customs.cooking.kitchen_word() } else { PATH_NAMES[i] };
            let t = format!("{name} {} of {}", pct(f.suff[i]), pct(f.share[i]));
            c.text(&t, cx, y, 13.0, if f.suff[i] < 0.75 { WARN } else { TEXT });
            cx += c.width(&t, 13.0) + 14.0;
        }
        if k.rhythm == gahturiyu_sim::sim::culture::Rhythm::Tides {
            let day = World::day_of(w.time);
            let low = tide::work_low(day);
            let (ws, we) = ((low - 3.0).max(4.0), (low + 4.5).min(21.5));
            y += 18.0;
            let hm = |h: f32| format!("{:02}:{:02}", h as i32, ((h.fract()) * 60.0) as i32);
            c.text(&format!("Tide hours today: low water {}, work {}–{} (a little later each day)", hm(low), hm(ws), hm(we)), x, y, 13.0, TEXT);
        }
        if cm.stilts && cm.withdrawn {
            y += 18.0;
            c.text("The village has withdrawn: no boats come ashore.", x, y, 13.0, WARN);
        }
        y += 8.0;
    }

    // The store.
    y += 24.0;
    c.text("In store", x, y, 15.0, TEXT);
    let money = format!("Prosperity {:.2}  ·  merchants' coin {:.0} / {:.0}", tl.prosperity, w.purse_now(town), tl.purse_cap);
    c.text(&money, r.x + r.w - c.width(&money, 13.0) - 14.0, y, 13.0, DIM);
    // What's in store, marked by its price here: dear (↑) or cheap (↓).
    let mut i = 0;
    for g in GOODS.iter().filter(|g| w.stock_now(town, **g) >= 0.5) {
        let col = (i % 4) as f32;
        let row = (i / 4) as f32;
        let f = w.price_factor(town, *g);
        let mark = if f >= 1.3 { " ↑" } else if f <= 0.7 { " ↓" } else { "" };
        c.text(&format!("{} {:.0}{mark}", g.name(), w.stock_now(town, *g)), x + col * 140.0, y + 20.0 + row * 17.0, 12.0, if f >= 1.3 { WARN } else { TEXT });
        i += 1;
    }
    y += 20.0 + ((i as f32 / 4.0).ceil().max(3.0) - 1.0) * 17.0;
    y += 17.0;
    let offers: Vec<String> = GOODS.iter().filter(|g| { let v = w.source(town, **g); v > 0.05 && v < 1.0 }).map(|g| g.name().to_lowercase()).collect();
    c.text(&format!("The land offers: {}", if offers.is_empty() { "nothing special".to_string() } else { offers.join(", ") }), x, y, 13.0, TEXT);
    y += 17.0;
    let crafters = tl.making.iter().filter(|m| m.rate > 0.0).count();
    let marked = tl.shelf.iter().filter(|s| s.piece.and_then(|p| p.mark).map(|m| m.stamped).unwrap_or(false)).count();
    c.text(&format!("On the shelf: {} made things ({} with a maker's mark)  ·  {} crafters at work now", tl.shelf.len(), marked, crafters), x, y, 13.0, TEXT);
    y += 17.0;
    let treasury = format!("Treasury {:.0}{}  ·  taxes in at dawn, guards paid", tl.treasury, if tl.owed > 0.0 { format!(" (owes {:.0} in pay)", tl.owed) } else { String::new() });
    c.text(&treasury, x, y, 13.0, if tl.owed > 0.0 { WARN } else { TEXT });
    if s.coastal {
        y += 17.0;
        c.text(&format!("The dawn boats brought {:.0} ashore", tl.landed), x, y, 13.0, TEXT);
    }

    // Who rules, and the law.
    y += 26.0;
    let g = w.government(town);
    c.text("Government", x, y, 15.0, TEXT);
    let unrest = format!("Unrest {:.0} / 100 (rises at {:.0}){}", g.unrest, gahturiyu_sim::sim::law::REVOLT_AT, if g.revolts > 0 { format!("  ·  {} revolts", g.revolts) } else { String::new() });
    c.text(&unrest, r.x + r.w - c.width(&unrest, 13.0) - 14.0, y, 13.0, if g.unrest >= 50.0 { WARN } else { DIM });
    y += 18.0;
    c.text(&w.gov_words(town), x, y, 13.0, GOLD);
    let name = |p: u32| w.people[p as usize].name().map(|n| n.to_string()).unwrap_or_else(|| gahturiyu_sim::sim::names::person_name(w.people[p as usize].race, w.people[p as usize].seed));
    for ch in &g.chambers {
        y += 17.0;
        let mut line = format!("{}: {}", ch.rule.name(), if ch.holders.is_empty() { "no one".to_string() } else { ch.holders.iter().map(|&p| name(p)).collect::<Vec<_>>().join(", ") });
        if !ch.administrators.is_empty() {
            line += &format!("  ·  administrators {}", ch.administrators.iter().map(|&p| name(p)).collect::<Vec<_>>().join(", "));
        }
        c.text(&line, x + 8.0, y, 13.0, TEXT);
    }
    if g.council {
        y += 17.0;
        let seats: Vec<String> = ALL_RACES.iter().filter(|r| g.seats[r.index()] > 0).map(|r| format!("{} {}", r.name(), g.seats[r.index()])).collect();
        c.text(&format!("Council seats: {}", seats.join(", ")), x + 8.0, y, 13.0, TEXT);
    }
    if let Some(a) = g.arbiter {
        y += 17.0;
        c.text(&format!("Arbiter: {}", name(a)), x + 8.0, y, 13.0, TEXT);
    }
    y += 17.0;
    let owners: Vec<String> = g
        .matters
        .iter()
        .map(|(m, o)| {
            let who = match o {
                gahturiyu_sim::sim::law::Owner::Chamber(r) => r.name().to_lowercase(),
                gahturiyu_sim::sim::law::Owner::Council => "the council".into(),
                gahturiyu_sim::sim::law::Owner::Arbiter => "the arbiter".into(),
            };
            format!("{} → {who}", m.name())
        })
        .collect();
    // (Wrapped to the panel's width.)
    let mut line = String::new();
    for o in owners {
        let next = if line.is_empty() { o.clone() } else { format!("{line}  ·  {o}") };
        if c.width(&next, 12.0) > r.w - 40.0 && !line.is_empty() {
            c.text(&line, x + 8.0, y, 12.0, DIM);
            y += 15.0;
            line = o;
        } else {
            line = next;
        }
    }
    c.text(&line, x + 8.0, y, 12.0, DIM);
    y += 17.0;
    let laws: Vec<String> = std::iter::once(tl.shore).chain(tl.stilts).map(|ci| {
        let cm = &w.society.communities[ci as usize];
        format!("{}: {}, {}", if cm.stilts { "Stilts" } else { "Land" }, cm.customs.justice.name().to_lowercase(), cm.customs.slavery.name().to_lowercase()) + &format!(", {}", cm.customs.garb.name().to_lowercase())
    }).collect();
    c.text(&laws.join("  ·  "), x + 8.0, y, 13.0, TEXT);
    let bonds = w.bonds.iter().filter(|b| b.town == town && b.until > w.time).count();
    let slaves = w.bonds.iter().filter(|b| b.town == town && b.slave && b.until > w.time).count();
    y += 17.0;
    let rite = match g.last_rite {
        Some((d, ok)) => format!("  ·  last rite day {d}: {}", if ok { "well held" } else { "failed" }),
        None => String::new(),
    };
    let withdrawn = tl.stilts.map(|si| w.society.communities[si as usize].withdrawn).unwrap_or(false);
    c.text(
        &format!("{bonds} bound here ({slaves} for life){rite}{}", if withdrawn { "  ·  the stilt village has withdrawn" } else { "" }),
        x + 8.0,
        y,
        13.0,
        if withdrawn { WARN } else { TEXT },
    );

    // Services, open now or not.
    y += 26.0;
    c.text("Services now", x, y, 15.0, TEXT);
    y += 4.0;
    let mut i = 0;
    for svc in SERVICES {
        let n = w.service_workers(town, svc);
        let (word, col) = if n == 0 {
            ("none".to_string(), DIM)
        } else if w.service_open(town, svc) {
            ("open".to_string(), OPEN)
        } else {
            ("closed".to_string(), WARN)
        };
        let col_x = x + (i % 3) as f32 * 160.0;
        let row_y = y + 18.0 + (i / 3) as f32 * 17.0;
        c.text(&format!("{} {}", svc.name(), word), col_x, row_y, 13.0, col);
        i += 1;
    }
    y += 18.0 + (i as f32 / 3.0).ceil() * 17.0;
    let places = tl.places.iter().filter(|p| !p.kind.is_away()).count();
    let foot = format!("{} workplaces  ·  season: {}  ·  P to close", places, tide::season_name(w.time));
    c.text(&foot, x, (y + 8.0).min(r.y + r.h - 10.0), 12.0, DIM);
    let _ = Color32::TRANSPARENT;
    r
}

const LW: f32 = 470.0;

/// Wrap a line to a width, returning the lines.
pub fn wrap(c: &Canvas, s: &str, size: f32, width: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split(' ') {
        let next = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if c.width(&next, size) > width && !line.is_empty() {
            out.push(line);
            line = word.to_string();
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// The town's talk, beside the town panel: what's happened lately, the
/// ring's leader (once the squad knows), and work the squad has heard of.
/// With the debug readout on, everything: every opportunity, event and
/// live storyline.
pub fn life_panel(c: &Canvas, w: &World, town: u16, debug: bool) -> Bx {
    let mut rows: Vec<(String, Rgb, f32)> = Vec::new();
    let head = |rows: &mut Vec<(String, Rgb, f32)>, s: &str| rows.push((s.to_string(), TEXT, 15.0));
    let line = |w: &World, e: &gahturiyu_sim::sim::history::Event| {
        let who = match (e.actor, e.hidden) {
            (Some(a), false) => w.name_of(a),
            _ => "Someone".into(),
        };
        let whom = e.victim.map(|v| w.name_of(v)).unwrap_or_default();
        format!("Day {} {}  {} {} {}", (e.t / 86400.0).floor(), super::hud::hhmm(e.t), who, e.deed.words(), whom)
    };
    head(&mut rows, "Lately");
    let shown: Vec<&gahturiyu_sim::sim::history::Event> = w.events(town).iter().rev().filter(|e| debug || (!e.hidden && e.severity >= 0.3)).take(if debug { 14 } else { 6 }).collect();
    if shown.is_empty() {
        rows.push(("Nothing much.".into(), DIM, 13.0));
    }
    for e in shown {
        rows.push((line(w, e), if e.severity >= 0.5 { WARN } else { DIM }, 13.0));
    }
    if let Some(r) = w.ring(town) {
        if r.found || debug {
            head(&mut rows, "The ring");
            let leader = r.leader.map(|l| w.name_of(l)).unwrap_or_else(|| "no one".into());
            rows.push((format!("Led by {leader}{}", if debug { format!("  ·  {} members, purse {:.0}, heat {:.1}", r.members.len(), r.purse, r.heat) } else { String::new() }), GOLD, 13.0));
            if debug {
                if let Some((mv, d)) = r.last {
                    rows.push((format!("Day {d}: {}", mv.words()), DIM, 13.0));
                }
            }
        }
    }
    let opps: Vec<&gahturiyu_sim::sim::chances::Opportunity> = if debug { w.society.opps.iter().filter(|o| o.town == town).collect() } else { w.known_opps(town) };
    head(&mut rows, if debug { "Opportunities (all)" } else { "Work you've heard of" });
    if opps.is_empty() {
        rows.push(("None.".into(), DIM, 13.0));
    }
    for o in opps {
        rows.push((format!("{}: {}{}", w.name_of(o.asker), w.opp_line(o), if debug { format!("  [{:?}]", o.state) } else { String::new() }), if o.legal { TEXT } else { WARN }, 13.0));
    }
    // Town work a squad member could take, and the wage.
    let posts = w.vacant_posts(town);
    if !posts.is_empty() {
        head(&mut rows, "Town work going (ask an official)");
        for (job, pl) in posts {
            let at = w.society.towns[town as usize].places.get(pl as usize).map(|p| p.kind.name().to_lowercase()).unwrap_or_default();
            rows.push((format!("{} at the {at}: {} coin a day, 8 till 5", job.name(), w.post_wage(job)), TEXT, 13.0));
        }
    }
    let ours: Vec<String> = w.society.contracts.iter().filter(|c| w.squad.index(c.member).is_some()).zip(w.work_lines()).filter(|(c, _)| c.town == town).map(|(_, l)| l).collect();
    if !ours.is_empty() {
        head(&mut rows, "Your people at work here");
        for l in ours {
            rows.push((l, GOLD, 13.0));
        }
    }
    // Where the out-of-town work is, from the town's middle.
    head(&mut rows, "Out of town");
    let mid = w.settlements[town as usize].pos;
    let tl = &w.society.towns[town as usize];
    let fields = tl.places.iter().filter(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields).count();
    let mut field_shown = false;
    for wp in tl.places.iter().filter(|p| p.kind.is_away()) {
        if wp.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields {
            if field_shown {
                continue;
            }
            field_shown = true;
        }
        let v = wp.pos.sub(mid);
        let left: Vec<String> = w.deposits.iter().filter(|d| d.pos.dist(wp.pos) < 15.0).map(|d| format!("{:.0} {} left", d.left_at(w.time).floor(), gahturiyu_sim::sim::items::item(d.item).name.to_lowercase())).collect();
        let name = if wp.kind == gahturiyu_sim::sim::jobs::PlaceKind::Fields { format!("Fields ({fields} plots)") } else { wp.kind.name().to_string() };
        let note = if left.is_empty() { String::new() } else { format!(" — {}", left.join(", ")) };
        let col = if left.is_empty() { DIM } else { TEXT };
        rows.push((format!("{name}: {:.0} m {}{note}", v.len(), gahturiyu_sim::sim::quests::compass(v)), col, 13.0));
    }
    if debug {
        head(&mut rows, &format!("Storylines (cap {}, drama {:.2})", w.story_cap(town), w.drama(town)));
        for s in w.society.stories.iter().filter(|s| s.town == town) {
            rows.push((format!("{} {}{}", w.name_of(s.who), s.plot.words(), if s.waiting.is_some() { " (waiting on an outsider)" } else { "" }), DIM, 13.0));
        }
    }
    // Lay out, wrapping.
    let probe: Vec<(Vec<String>, Rgb, f32)> = rows.iter().map(|(s, col, size)| (wrap(c, s, *size, LW - 28.0), *col, *size)).collect();
    let h: f32 = 24.0 + probe.iter().map(|(ls, _, size)| ls.len() as f32 * (size + 4.0) + if *size > 14.0 { 8.0 } else { 0.0 }).sum::<f32>();
    let r = Bx::new(c.w - W - LW - 24.0, 12.0, LW, h.min(c.h - 60.0));
    c.frame_box(r.x, r.y, r.w, r.h);
    let x = r.x + 14.0;
    let mut y = r.y + 10.0;
    for (ls, col, size) in probe {
        if size > 14.0 {
            y += 8.0;
        }
        for l in ls {
            y += size + 4.0;
            if y > r.y + r.h - 4.0 {
                return r;
            }
            c.text(&l, x, y, size, col);
        }
    }
    r
}

// ---- The visitor's panel (#153) ---------------------------------------------------

/// The visitor's panel's width, and the room it leaves clear below for the
/// little map and the bottom band.
const VW: f32 = 440.0;
const CLEAR_BELOW: f32 = 232.0;

/// What a town's people are, in words: "mostly Roduro", "Roduro and Qotiro".
fn peoples_words(share: &[f32; 4]) -> String {
    let mut by: Vec<(gahturiyu_sim::sim::race::Race, f32)> = ALL_RACES.iter().map(|r| (*r, share[r.index()])).filter(|x| x.1 >= 0.15).collect();
    by.sort_by(|a, b| b.1.total_cmp(&a.1));
    match by.as_slice() {
        [] => "a mix of peoples".into(),
        [(a, s), ..] if *s >= 0.6 => format!("mostly {}", a.name()),
        [(a, _)] => a.name().to_string(),
        [(a, _), (b, _)] => format!("{} and {}", a.name(), b.name()),
        [(a, _), (b, _), rest @ ..] => {
            let more: Vec<&str> = rest.iter().map(|x| x.0.name()).collect();
            format!("{}, {} and {}", a.name(), b.name(), more.join(" and "))
        }
    }
}

/// Where in town a spot is, from its middle: "in the middle", "on the east side".
fn side_words(mid: gahturiyu_sim::sim::geo::V2, at: gahturiyu_sim::sim::geo::V2, radius: f32) -> String {
    let v = at.sub(mid);
    if v.len() < radius * 0.3 {
        "in the middle".into()
    } else if v.len() > radius * 1.4 {
        format!("out to the {}", gahturiyu_sim::sim::quests::compass(v))
    } else {
        format!("on the {} side", gahturiyu_sim::sim::quests::compass(v))
    }
}

fn capital(s: &str) -> String {
    let mut ch = s.chars();
    ch.next().map(|c| c.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default()
}

/// One line of the visitor's panel.
enum Line {
    Head(&'static str),
    Text(String, Rgb),
    /// A row of two: what (left) and its note (right).
    Pair(String, String, Rgb),
}

/// Draw the visitor's panel for a town. Returns where it is, and whether its
/// close cross was clicked.
pub fn town_panel(c: &Canvas, w: &World, town: u16, click: Option<super::squadui::Click>) -> (Bx, bool) {
    use super::hud::Face;
    use super::kit;
    use super::palette::{BRASS, BRASS_LIGHT};
    let s = &w.settlements[town as usize];
    let tl = &w.society.towns[town as usize];
    let shore = &w.society.communities[tl.shore as usize];
    let mut lines: Vec<Line> = Vec::new();

    // Who lives here, and how many, in words.
    let folk = s.residents.iter().filter(|&&p| !w.people[p as usize].dead).count();
    let mut about = format!(
        "{} town{}: {} people, {}.",
        s.founders.name(),
        if s.coastal { " on the coast" } else { "" },
        gahturiyu_sim::sim::talk::folk_words(folk),
        peoples_words(&shore.blend.share)
    );
    about = capital(&format!("a {about}"));
    lines.push(Line::Text(about, TEXT));
    if let Some(si) = tl.stilts {
        let st = &w.society.communities[si as usize];
        let line = if st.withdrawn { "The stilt village offshore has turned its back on the town.".to_string() } else { format!("A stilt village stands offshore. The tide is at {}.", tide::tide_word(w.time)) };
        lines.push(Line::Text(line, if st.withdrawn { WARN } else { DIM }));
    }
    if let Some(owed) = w.bounty.get(&town).filter(|b| **b >= 1.0) {
        lines.push(Line::Text(format!("You're wanted here: {owed:.0} coin owed."), WARN));
    }

    // Who rules.
    lines.push(Line::Head("WHO RULES"));
    lines.push(Line::Text(capital(&w.gov_words(town)), GOLD));
    let rulers = w.rulers_words(town);
    if rulers != "no one" {
        lines.push(Line::Text(capital(&rulers), DIM));
    }

    // What's open now, and where.
    lines.push(Line::Head("OPEN NOW"));
    let mut shut: Vec<String> = Vec::new();
    let mut any = false;
    for svc in SERVICES {
        let workers: Vec<u32> = s.residents.iter().copied().filter(|&p| !w.people[p as usize].dead && w.life(p).job.service() == Some(svc)).collect();
        if workers.is_empty() {
            continue;
        }
        if !w.service_open(town, svc) {
            shut.push(svc.name().to_lowercase());
            continue;
        }
        any = true;
        let place = workers.iter().find(|&&p| w.at_work(p, w.time)).or(workers.first()).and_then(|&p| w.workplace_of(p)).map(|wp| wp.pos);
        let where_ = place.map(|at| side_words(s.pos, at, s.radius())).unwrap_or_default();
        lines.push(Line::Pair(svc.name().to_string(), where_, OPEN));
    }
    if !any {
        lines.push(Line::Text("Nothing just now.".into(), DIM));
    }
    if !shut.is_empty() {
        lines.push(Line::Text(format!("Shut: {}.", shut.join(", ")), DIM));
    }

    // Work going: posts in town, jobs heard of, the squad's own.
    lines.push(Line::Head("WORK"));
    let before = lines.len();
    for (job, pl) in w.vacant_posts(town) {
        let at = tl.places.get(pl as usize).map(|p| p.kind.name().to_lowercase()).unwrap_or_default();
        lines.push(Line::Pair(format!("{} at the {at}", job.name()), format!("{} coin a day", w.post_wage(job)), TEXT));
    }
    for o in w.known_opps(town) {
        lines.push(Line::Text(format!("{}: {}", w.name_of(o.asker), w.opp_line(o)), if o.legal { TEXT } else { WARN }));
    }
    let ours: Vec<String> = w.society.contracts.iter().filter(|c| w.squad.index(c.member).is_some()).zip(w.work_lines()).filter(|(c, _)| c.town == town).map(|(_, l)| l).collect();
    for l in ours {
        lines.push(Line::Text(l, GOLD));
    }
    // Out of town: where the woodlot, the mine and the fields are.
    let mut seen_kinds = Vec::new();
    for wp in tl.places.iter().filter(|p| p.kind.is_away()) {
        if seen_kinds.contains(&wp.kind) {
            continue;
        }
        seen_kinds.push(wp.kind);
        let v = wp.pos.sub(s.pos);
        let worked_out = w.deposits.iter().filter(|d| d.pos.dist(wp.pos) < 15.0).all(|d| d.left_at(w.time) < 1.0) && w.deposits.iter().any(|d| d.pos.dist(wp.pos) < 15.0);
        let note = format!("{} {}{}", gahturiyu_sim::sim::talk::walk_words(v.len()), gahturiyu_sim::sim::quests::compass(v), if worked_out { ", worked out" } else { "" });
        lines.push(Line::Pair(wp.kind.name().to_string(), note, DIM));
    }
    if lines.len() == before {
        lines.push(Line::Text("None that you know of.".into(), DIM));
    }

    // The talk of the town.
    let lately: Vec<&gahturiyu_sim::sim::history::Event> = w.events(town).iter().rev().filter(|e| !e.hidden && e.severity >= 0.3).take(4).collect();
    let ring = w.ring(town).filter(|r| r.found);
    if !lately.is_empty() || ring.is_some() {
        lines.push(Line::Head("TALK OF THE TOWN"));
        for e in lately {
            let who = e.actor.map(|a| w.name_of(a)).unwrap_or_else(|| "Someone".into());
            let whom = e.victim.map(|v| w.name_of(v)).unwrap_or_default();
            let day = World::day_of(e.t) as i64 + 1;
            lines.push(Line::Text(format!("Day {day}: {who} {} {whom}", e.deed.words()).trim_end().to_string(), if e.severity >= 0.5 { WARN } else { DIM }));
        }
        if let Some(r) = ring {
            let leader = r.leader.map(|l| w.name_of(l)).unwrap_or_else(|| "nobody known".into());
            lines.push(Line::Text(format!("A ring works the town, led by {leader}."), GOLD));
        }
    }

    // Lay it out: wrapped, and cut off where the room ends.
    let inner = VW - 32.0;
    let mut laid: Vec<(Line, Vec<String>)> = Vec::new();
    for l in lines {
        let wrapped = match &l {
            Line::Text(t, _) => wrap(c, t, 14.0, inner),
            _ => Vec::new(),
        };
        laid.push((l, wrapped));
    }
    let tall = |l: &Line, wrapped: &[String]| match l {
        Line::Head(_) => 34.0,
        Line::Text(..) => wrapped.len() as f32 * 19.0,
        Line::Pair(..) => 19.0,
    };
    let body: f32 = laid.iter().map(|(l, ws)| tall(l, ws)).sum();
    let h = (96.0 + body + 18.0).min(c.h - CLEAR_BELOW - 12.0).max(200.0);
    let r = Bx::new(c.w - VW - 14.0, 12.0, VW, h);
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 3.0, eg(race_color(s.founders)));

    // The heading: the town's name as the people round it say it, and the other.
    let name = super::lexicon::town(w, town).to_uppercase();
    c.styled(&name, r.x + 16.0, r.y + 40.0, 26.0, eg(BRASS_LIGHT), Face::Title, 4.0);
    if let Some(other) = super::lexicon::town_other(w, town) {
        c.styled(&other, r.x + 18.0, r.y + 62.0, 15.0, eg(DIM), Face::Italic, 0.0);
    }
    let close = Bx::new(r.x + r.w - 38.0, r.y + 10.0, 28.0, 28.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 7.0, close.y + 22.0 + kit::nudge(cl), 24.0, eg(if cl.hot { BRASS_LIGHT } else { BRASS }), Face::Body, 0.0);
    let closed = click.is_some_and(|k| !k.right && close.contains(k.at));
    c.rule_in(r.x + 12.0, r.x + r.w - 12.0, r.y + 76.0, 1.5, eg(BRASS), false);

    let x = r.x + 16.0;
    let right = r.x + r.w - 16.0;
    let mut y = r.y + 84.0;
    let end = r.y + r.h - 10.0;
    for (l, wrapped) in &laid {
        if y + tall(l, wrapped) > end {
            c.styled_right("…", right, end, 14.0, eg(DIM), Face::Body, 0.0);
            break;
        }
        match l {
            Line::Head(t) => {
                y += 34.0;
                c.diamond(x + 3.0, y - 5.0, 3.5, eg(BRASS));
                c.styled(t, x + 14.0, y, 12.0, eg(BRASS), Face::Title, 2.5);
            }
            Line::Text(_, col) => {
                for part in wrapped {
                    y += 19.0;
                    c.text(part, x, y, 14.0, *col);
                }
            }
            Line::Pair(a, b, col) => {
                y += 19.0;
                c.text(a, x, y, 14.0, TEXT);
                c.styled_right(b, right, y, 14.0, eg(*col), Face::Italic, 0.0);
            }
        }
    }
    (r, closed)
}
