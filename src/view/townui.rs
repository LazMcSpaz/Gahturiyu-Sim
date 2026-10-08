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
    let h = 330.0 + comms.len() as f32 * 185.0;
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
    for (i, g) in GOODS.iter().enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        c.text(&format!("{} {:.0}", g.name(), w.stock_now(town, *g)), x + col * 160.0, y + 20.0 + row * 17.0, 13.0, TEXT);
    }
    y += 20.0 + 3.0 * 17.0;
    let treasury = format!("Treasury {:.0}{}  ·  taxes in at dawn, guards paid", tl.treasury, if tl.owed > 0.0 { format!(" (owes {:.0} in pay)", tl.owed) } else { String::new() });
    c.text(&treasury, x, y, 13.0, if tl.owed > 0.0 { WARN } else { TEXT });
    if s.coastal {
        y += 17.0;
        c.text(&format!("The dawn boats brought {:.0} ashore", tl.landed), x, y, 13.0, TEXT);
    }

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
