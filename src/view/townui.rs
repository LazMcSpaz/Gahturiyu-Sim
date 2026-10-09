//! The town panel (P, or click a town's name): what its people lean toward,
//! what they've settled on, what's in store, how well they ate, the money,
//! and which services are open right now.

use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    culture::{BELONGINGS, COOKINGS, RHYTHMS},
    jobs::{GOODS, SERVICES},
    race::ALL_RACES,
    society::PATH_NAMES,
    tide, World,
};

use super::hud::{Canvas, PANEL};
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

/// Draw the panel for a town; returns where it is.
pub fn town_panel(c: &Canvas, w: &World, town: u16) -> Bx {
    let s = &w.settlements[town as usize];
    let tl = &w.society.towns[town as usize];
    let comms: Vec<u32> = std::iter::once(tl.shore).chain(tl.stilts).collect();
    let stocked = GOODS.iter().filter(|g| w.stock_now(town, **g) >= 0.5).count();
    let gov_rows = w.government(town).chambers.len() as f32 + 6.0;
    let h = 330.0 + 70.0 + gov_rows * 17.0 + 30.0 + comms.len() as f32 * 185.0 + ((stocked as f32 / 4.0).ceil() - 3.0).max(0.0) * 17.0;
    let r = Bx::new(c.w - W - 12.0, 12.0, W, h.min(c.h - 60.0));
    c.rect(r.x, r.y, r.w, r.h, PANEL);
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
        format!("{}: {}, {}", if cm.stilts { "Stilts" } else { "Land" }, cm.customs.justice.name().to_lowercase(), cm.customs.slavery.name().to_lowercase())
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
    c.rect(r.x, r.y, r.w, r.h, PANEL);
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
