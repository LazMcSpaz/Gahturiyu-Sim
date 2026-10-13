//! The hidden test quarter in the window (drawing only; the rules are
//! `sim/testquarter.rs`). Home, with the detail switch (L) on, opens the
//! quarter and brings the squad in; pressed again inside it, it shows the
//! dev spawner. Nothing here is part of the story.
//!
//! - **Drawing:** the wall and its gate (the same boxes `solid.rs` lists),
//!   the arena's ring of posts and sand, pegged-out build plots, and a
//!   signpost at each culture lot in its people's colour.
//! - **The spawner** (`panel`): tabs for items (into the lead's pack; Shift
//!   for ten), creatures (a herd of three), people (a band, someone of each
//!   people or of a faction), the world (skip hours, days or a season; show
//!   a kind of weather, as U does) and the yard (dummies stood up again, the
//!   gate, and a weapon's row of the duel grid).

use bevy::math::vec3;

use gahturiyu_sim::sim::{
    animals::{Sp, ALL_SPECIES},
    duelgrid::{self, Row, SETS},
    factions::{FactionId, FACTIONS},
    geo::V2,
    items::{ItemId, ITEMS},
    race::{Race, ALL_RACES},
    World,
};

use super::cam::to3;
use super::hud::{Canvas, Face};
use super::kit::{self, Kind};
use super::mesh::Builder;
use super::palette::{eg, ega, race_color, Rgb, BRASS, GOLD, TEXT};
use super::squadui::{Bx, Click};
use super::weather::{Preset, PRESETS};

const WALL: Rgb = [0.46, 0.43, 0.40];
const GATE: Rgb = [0.42, 0.30, 0.18];
const SAND: Rgb = [0.78, 0.70, 0.52];
const POST: Rgb = [0.40, 0.29, 0.18];
const PEG: Rgb = [0.85, 0.80, 0.68];

/// Draw the quarter (the wall follows the drawn ground in short stretches).
pub fn draw(b: &mut Builder, w: &World, on_ground: &dyn Fn(V2) -> f32, near: V2, reach: f32) {
    let Some(q) = w.test_quarter() else { return };
    if q.centre().dist(near) > reach + 200.0 {
        return;
    }
    let h = gahturiyu_sim::sim::testquarter::WALL_H;
    for r in q.walls() {
        let [u, _] = r.axes();
        let n = ((r.hw * 2.0) / 8.0).ceil().max(1.0) as usize;
        for i in 0..n {
            let s = -r.hw + r.hw * 2.0 * (i as f32 + 0.5) / n as f32;
            let at = r.c.add(u.scale(s));
            b.block(to3(at, on_ground(at) - 0.3), r.hw * 2.0 / n as f32 + 0.05, r.hd * 2.0, h + 0.3, r.rot, WALL);
        }
    }
    // The gate: shut across the opening, or swung back inside.
    let g = q.gate_leaf();
    if q.open {
        let hinge = g.c.add(V2::new(0.0, -g.hw));
        let at = hinge.add(V2::new(-g.hw, 0.0));
        b.block(to3(at, on_ground(at)), g.hw * 2.0, g.hd * 2.0, h * 0.8, 0.0, GATE);
    } else {
        b.block(to3(g.c, on_ground(g.c)), g.hw * 2.0, g.hd * 2.0, h * 0.8, g.rot, GATE);
    }
    // The arena: a sand floor and a ring of posts.
    let (c, rad) = q.arena();
    b.block(to3(c, on_ground(c) - 0.12), rad * 2.0, rad * 2.0, 0.15, 0.0, SAND);
    for i in 0..16 {
        let a = i as f32 / 16.0 * std::f32::consts::TAU;
        let p = c.add(V2::new(a.cos(), a.sin()).scale(rad));
        b.column(to3(p, on_ground(p)), 0.12, 0.1, 1.2, 6, POST);
    }
    // Build plots: a peg at each corner and a line between.
    for r in q.plots() {
        let [u, v] = r.axes();
        let corners = [r.c.add(u.scale(-r.hw)).add(v.scale(-r.hd)), r.c.add(u.scale(r.hw)).add(v.scale(-r.hd)), r.c.add(u.scale(r.hw)).add(v.scale(r.hd)), r.c.add(u.scale(-r.hw)).add(v.scale(r.hd))];
        for k in 0..4 {
            let (a, z) = (corners[k], corners[(k + 1) % 4]);
            b.column(to3(a, on_ground(a)), 0.06, 0.05, 0.7, 4, PEG);
            b.stick(to3(a, on_ground(a) + 0.5), to3(z, on_ground(z) + 0.5), 0.03, PEG);
        }
    }
    // The culture row: a signpost at each lot, in its people's colour.
    for (race, r) in q.culture_lots() {
        let p = r.c.add(V2::new(-r.hw, r.hd + 1.0));
        b.column(to3(p, on_ground(p)), 0.08, 0.07, 2.0, 5, POST);
        b.block(to3(p, on_ground(p) + 1.6) + vec3(0.4, 0.0, 0.0), 0.9, 0.08, 0.4, 0.0, race_color(race));
    }
}

/// The spawner's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Items,
    Creatures,
    People,
    World,
    Yard,
}

const TABS: [(Tab, &str); 5] = [(Tab::Items, "Items"), (Tab::Creatures, "Creatures"), (Tab::People, "People"), (Tab::World, "World"), (Tab::Yard, "Yard")];

/// The spawner while it's open.
#[derive(Clone, Debug, Default)]
pub struct DevPanel {
    pub tab: Tab,
    pub page: usize,
    /// The duel grid's weapon (index into `duelgrid::weapons()`), and its
    /// last row.
    pub weapon: usize,
    pub row: Option<Row>,
}

pub enum DevAct {
    Close,
    Tab(Tab),
    Page(i32),
    Give(ItemId, u16),
    Herd(Sp),
    Band(bool),
    Person(Race),
    Faction(FactionId),
    Skip(f64),
    Weather(Option<Preset>),
    ResetDummies,
    Gate(bool),
    Weapon(i32),
    RunGrid,
}

const W: f32 = 520.0;
const ROWS: usize = 16;
const ROW: f32 = 28.0;

/// The spawner panel, on the right. What was clicked, and the box it covers.
pub fn panel(c: &Canvas, w: &World, p: &DevPanel, click: Option<Click>, shift: bool) -> (Option<DevAct>, Bx) {
    let x = c.w - W - 24.0;
    let y = 90.0;
    let h = 140.0 + ROWS as f32 * ROW;
    c.frame_box(x, y, W, h);
    let mut act = None;
    let hit = |b: &Bx| click.is_some_and(|k| !k.right && b.contains(k.at));
    c.styled("TEST QUARTER", x + 20.0, y + 34.0, 18.0, eg(GOLD), Face::Title, 4.0);
    c.styled("dev only", x + 210.0, y + 34.0, 14.0, ega(TEXT, 0.6), Face::Italic, 0.0);
    let close = Bx::new(x + W - 44.0, y + 14.0, 28.0, 28.0);
    kit::button(c, &close, "×", Kind::Plain, true);
    if hit(&close) {
        act = Some(DevAct::Close);
    }
    // Tabs.
    let tw = (W - 40.0) / TABS.len() as f32;
    for (k, (t, name)) in TABS.iter().enumerate() {
        let b = Bx::new(x + 20.0 + k as f32 * tw, y + 50.0, tw - 6.0, 30.0);
        kit::button(c, &b, name, if *t == p.tab { Kind::Primary } else { Kind::Plain }, true);
        if hit(&b) {
            act = Some(DevAct::Tab(*t));
        }
    }
    let top = y + 96.0;
    // A list of labelled buttons in two columns, paged.
    let list = |labels: Vec<(String, Rgb)>, acts: Vec<DevAct>, act: &mut Option<DevAct>| {
        let per = ROWS * 2;
        let pages = labels.len().div_ceil(per).max(1);
        let page = p.page.min(pages - 1);
        let mut acts: Vec<Option<DevAct>> = acts.into_iter().map(Some).collect();
        for (k, (label, col)) in labels.iter().enumerate().skip(page * per).take(per) {
            let i = k - page * per;
            let b = Bx::new(x + 20.0 + (i / ROWS) as f32 * (W - 40.0) / 2.0, top + (i % ROWS) as f32 * ROW, (W - 40.0) / 2.0 - 6.0, ROW - 3.0);
            let l = kit::row(c, &b, false);
            c.styled(label, b.x + 8.0, b.y + 19.0 + kit::nudge(l), 15.0, eg(*col), Face::Body, 0.0);
            if hit(&b) {
                *act = acts[k].take();
            }
        }
        if pages > 1 {
            let by = top + ROWS as f32 * ROW + 6.0;
            let (prev, next) = (Bx::new(x + 20.0, by, 40.0, 28.0), Bx::new(x + W - 60.0, by, 40.0, 28.0));
            kit::button(c, &prev, "<", Kind::Plain, page > 0);
            kit::button(c, &next, ">", Kind::Plain, page + 1 < pages);
            c.styled(&format!("page {} of {pages}", page + 1), x + W / 2.0 - 40.0, by + 19.0, 14.0, ega(TEXT, 0.7), Face::Body, 0.0);
            if hit(&prev) && page > 0 {
                *act = Some(DevAct::Page(-1));
            }
            if hit(&next) && page + 1 < pages {
                *act = Some(DevAct::Page(1));
            }
        }
    };
    match p.tab {
        Tab::Items => {
            let n = if shift { 10 } else { 1 };
            let labels = ITEMS.iter().map(|d| (d.name.to_string(), TEXT)).collect();
            let acts = (0..ITEMS.len()).map(|i| DevAct::Give(i as ItemId, n)).collect();
            list(labels, acts, &mut act);
        }
        Tab::Creatures => {
            let labels = ALL_SPECIES.iter().map(|s| (s.def().name.to_string(), TEXT)).collect();
            let acts = ALL_SPECIES.iter().map(|&s| DevAct::Herd(s)).collect();
            list(labels, acts, &mut act);
        }
        Tab::People => {
            let mut labels: Vec<(String, Rgb)> = vec![("Bandits (three)".into(), TEXT), ("Binders (three)".into(), TEXT)];
            let mut acts = vec![DevAct::Band(false), DevAct::Band(true)];
            for r in ALL_RACES {
                labels.push((format!("A {}", r.name()), race_color(r)));
                acts.push(DevAct::Person(r));
            }
            for (i, f) in FACTIONS.iter().enumerate() {
                labels.push((f.name.to_string(), BRASS));
                acts.push(DevAct::Faction(i as FactionId));
            }
            list(labels, acts, &mut act);
        }
        Tab::World => {
            let mut labels: Vec<(String, Rgb)> = vec![("An hour on".into(), TEXT), ("Six hours on".into(), TEXT), ("A day on".into(), TEXT), ("A season on (12 days)".into(), TEXT), ("Real weather".into(), GOLD)];
            let mut acts = vec![DevAct::Skip(1.0), DevAct::Skip(6.0), DevAct::Skip(24.0), DevAct::Skip(12.0 * 24.0), DevAct::Weather(None)];
            for pr in PRESETS {
                labels.push((format!("Show {}", pr.name().to_lowercase()), TEXT));
                acts.push(DevAct::Weather(Some(pr)));
            }
            list(labels, acts, &mut act);
        }
        Tab::Yard => {
            let open = w.test_quarter().is_some_and(|q| q.open);
            let b1 = Bx::new(x + 20.0, top, 220.0, 32.0);
            let b2 = Bx::new(x + 250.0, top, 250.0, 32.0);
            kit::button(c, &b1, "Stand the dummies up", Kind::Plain, true);
            kit::button(c, &b2, if open { "Lock the gate" } else { "Open the gate" }, Kind::Plain, true);
            if hit(&b1) {
                act = Some(DevAct::ResetDummies);
            }
            if hit(&b2) {
                act = Some(DevAct::Gate(!open));
            }
            // The duel grid: a weapon's row against every armour set.
            let weapons = duelgrid::weapons();
            let wk = p.weapon % weapons.len().max(1);
            let name = weapons.get(wk).map(|k| gahturiyu_sim::sim::items::item(gahturiyu_sim::sim::items::id(k)).name).unwrap_or("?");
            let gy = top + 50.0;
            c.styled("DUEL GRID", x + 20.0, gy + 20.0, 14.0, eg(GOLD), Face::Title, 3.0);
            let (prev, next, run) = (Bx::new(x + 140.0, gy, 32.0, 28.0), Bx::new(x + 380.0, gy, 32.0, 28.0), Bx::new(x + 420.0, gy, 80.0, 28.0));
            kit::button(c, &prev, "<", Kind::Plain, true);
            kit::button(c, &next, ">", Kind::Plain, true);
            kit::button(c, &run, "Run", Kind::Primary, true);
            c.styled(name, x + 182.0, gy + 20.0, 16.0, eg(TEXT), Face::Body, 0.0);
            if hit(&prev) {
                act = Some(DevAct::Weapon(-1));
            }
            if hit(&next) {
                act = Some(DevAct::Weapon(1));
            }
            if hit(&run) {
                act = Some(DevAct::RunGrid);
            }
            let ty = gy + 50.0;
            c.styled("against", x + 20.0, ty, 13.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            c.styled("secs to down", x + 230.0, ty, 13.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            c.styled("wins of 8 v. longsword", x + 340.0, ty, 13.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            for (k, (set, _)) in SETS.iter().enumerate() {
                let ry = ty + 26.0 + k as f32 * 24.0;
                c.styled(set, x + 20.0, ry, 15.0, eg(TEXT), Face::Body, 0.0);
                if let Some(r) = p.row.as_ref().filter(|r| r.weapon == weapons.get(wk).copied().unwrap_or("")) {
                    let s = r.secs[k].map(|t| format!("{t:.0}")).unwrap_or_else(|| "-".into());
                    c.styled(&s, x + 250.0, ry, 15.0, eg(TEXT), Face::Body, 0.0);
                    c.styled(&r.wins[k].to_string(), x + 380.0, ry, 15.0, eg(TEXT), Face::Body, 0.0);
                }
            }
            c.styled("Run fights the full rules: a moment's pause.", x + 20.0, ty + 26.0 + SETS.len() as f32 * 24.0 + 10.0, 13.0, ega(TEXT, 0.6), Face::Italic, 0.0);
        }
    }
    (act, Bx::new(x, y, W, h))
}

/// Carry out what was clicked. `lead`: whose pack things go into.
pub fn apply(w: &mut World, p: &mut DevPanel, act: DevAct, lead: Option<gahturiyu_sim::sim::person::PersonId>, force: &mut Option<Preset>) -> Option<String> {
    let pad = w.test_quarter().map(|q| q.pad()).unwrap_or(w.squad.pos);
    // Things are put a little way off the pad, each at its own spot.
    let near = |k: usize| pad.add(V2::new(-14.0 - (k % 4) as f32 * 5.0, -10.0 + (k / 4 % 5) as f32 * 5.0));
    let k = w.people.len() + w.animals.herds.len();
    match act {
        DevAct::Close => None,
        DevAct::Tab(t) => {
            p.tab = t;
            p.page = 0;
            None
        }
        DevAct::Page(d) => {
            p.page = (p.page as i32 + d).max(0) as usize;
            None
        }
        DevAct::Give(it, n) => {
            let who = lead?;
            w.dev_give(who, it, n);
            Some(format!("{n} × {} into {}'s pack", gahturiyu_sim::sim::items::item(it).name, w.name_of(who)))
        }
        DevAct::Herd(sp) => {
            w.dev_herd(sp, near(k), 3);
            Some(format!("A herd of {}", sp.def().name.to_lowercase()))
        }
        DevAct::Band(slavers) => {
            w.dev_band(near(k).add(V2::new(-30.0, 0.0)), 3, slavers);
            Some(if slavers { "Three Binders".into() } else { "Three bandits".into() })
        }
        DevAct::Person(r) => {
            w.dev_person(r, near(k));
            Some(format!("A {}", r.name()))
        }
        DevAct::Faction(f) => {
            let who = w.dev_faction_member(f, near(k));
            Some(format!("{}, of {}", w.name_of(who), FACTIONS[f as usize].name))
        }
        DevAct::Skip(h) => {
            w.dev_skip(h);
            Some(format!("{h} hours on"))
        }
        DevAct::Weather(pr) => {
            *force = pr;
            Some(match pr {
                Some(x) => format!("Weather shown: {} (drawing only, as U)", x.name()),
                None => "Weather shown: the real weather".into(),
            })
        }
        DevAct::ResetDummies => {
            w.dev_reset_dummies();
            Some("The dummies stand again".into())
        }
        DevAct::Gate(open) => {
            if open {
                w.enter_test_quarter();
            } else {
                w.lock_test_quarter();
            }
            Some(if open { "The gate is open".into() } else { "The gate is locked".into() })
        }
        DevAct::Weapon(d) => {
            let n = duelgrid::weapons().len().max(1) as i32;
            p.weapon = ((p.weapon as i32 + d).rem_euclid(n)) as usize;
            None
        }
        DevAct::RunGrid => {
            let weapons = duelgrid::weapons();
            let wk = weapons.get(p.weapon % weapons.len().max(1)).copied()?;
            p.row = Some(duelgrid::row(wk));
            Some(format!("Duel grid: {}", gahturiyu_sim::sim::items::item(gahturiyu_sim::sim::items::id(wk)).name))
        }
    }
}
