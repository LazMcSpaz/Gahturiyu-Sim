//! The hidden test quarter in the window (drawing only; the rules are
//! `sim/testquarter.rs`), and the dev panel. Home, with the detail switch
//! (L) on, opens the quarter and brings the squad in; Insert, anywhere,
//! opens and closes the dev panel (Laz, 13 Oct: "a little window where I
//! can turn weather on or off, ... spawn fire, ... spawn, turn things on or
//! off"). Nothing here is part of the story.
//!
//! - **Drawing:** the wall and its gate (the same boxes `solid.rs` lists),
//!   the arena's ring of posts and sand, pegged-out build plots, and a
//!   signpost at each culture lot in its people's colour.
//! - **The dev panel** (`panel`): a small window under the tracked corner
//!   that stays open while the game goes on and other screens come and go
//!   (it isn't one of `screens.rs`'s). Tabs: items (into the lead's pack;
//!   Shift for ten), animals (a herd of three), people (a band, someone of
//!   each people or of a faction), time (hours, a day, a season on), weather
//!   (forced in the sim: cold, wet, pace, sight and fires all follow;
//!   `World::set_forced_sky`), fire (a campfire, a standing torch), the squad
//!   (heal, bind, knock out, soak, chill, starve, coin), the rules toggles,
//!   go (the nearest town, Salt Isle, the test quarter), and the yard
//!   (dummies stood up again, the gate, a weapon's row of the duel grid).

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

/// The dev panel's tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Items,
    Creatures,
    People,
    World,
    Weather,
    Fire,
    Squad,
    Rules,
    Go,
    Yard,
}

const TABS: [(Tab, &str); 10] = [
    (Tab::Items, "Items"),
    (Tab::Creatures, "Animals"),
    (Tab::People, "People"),
    (Tab::World, "Time"),
    (Tab::Weather, "Weather"),
    (Tab::Fire, "Fire"),
    (Tab::Squad, "Squad"),
    (Tab::Rules, "Rules"),
    (Tab::Go, "Go"),
    (Tab::Yard, "Yard"),
];

/// The dev panel while it's open.
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
    /// A band: Binders or bandits, how many, and a mage among them.
    Band(bool, usize, bool),
    Person(Race),
    Faction(FactionId),
    Skip(f64),
    Weather(Option<Preset>),
    Campfire,
    Torch,
    Heal,
    KnockOut,
    Soak,
    Chill,
    Starve,
    Coin,
    Bind,
    Rule(&'static str),
    GoTown,
    GoIsland(gahturiyu_sim::sim::island::Spot),
    GoQuarter,
    ToArena,
    ResetDummies,
    Gate(bool),
    Weapon(i32),
    RunGrid,
}

/// A rule switched off, dimmed.
const OFF: Rgb = [0.55, 0.50, 0.42];

const W: f32 = 440.0;
const ROWS: usize = 11;
const ROW: f32 = 26.0;

/// The dev panel: a small window under the tracked corner that stays open
/// while the game goes on. What was clicked, and the box it covers.
pub fn panel(c: &Canvas, w: &World, p: &DevPanel, click: Option<Click>, shift: bool) -> (Option<DevAct>, Bx) {
    let x = c.w - W - 24.0;
    let y = 250.0;
    let tab_h = 26.0;
    let h = 46.0 + 2.0 * tab_h + 12.0 + ROWS as f32 * ROW + 40.0;
    c.frame_box(x, y, W, h);
    let mut act = None;
    let hit = |b: &Bx| click.is_some_and(|k| !k.right && b.contains(k.at));
    c.styled("DEV PANEL", x + 16.0, y + 28.0, 15.0, eg(GOLD), Face::Title, 3.0);
    c.styled("Insert to close", x + 150.0, y + 28.0, 12.0, ega(TEXT, 0.55), Face::Italic, 0.0);
    let close = Bx::new(x + W - 38.0, y + 10.0, 24.0, 24.0);
    kit::button(c, &close, "×", Kind::Plain, true);
    if hit(&close) {
        act = Some(DevAct::Close);
    }
    // Tabs, two rows of five.
    let tw = (W - 32.0) / 5.0;
    for (k, (t, name)) in TABS.iter().enumerate() {
        let b = Bx::new(x + 16.0 + (k % 5) as f32 * tw, y + 42.0 + (k / 5) as f32 * (tab_h + 4.0), tw - 4.0, tab_h);
        kit::button(c, &b, name, if *t == p.tab { Kind::Primary } else { Kind::Plain }, true);
        if hit(&b) {
            act = Some(DevAct::Tab(*t));
        }
    }
    let top = y + 42.0 + 2.0 * (tab_h + 4.0) + 8.0;
    // A list of labelled rows in two columns, paged.
    let list = |labels: Vec<(String, Rgb)>, acts: Vec<DevAct>, act: &mut Option<DevAct>| {
        let per = ROWS * 2;
        let pages = labels.len().div_ceil(per).max(1);
        let page = p.page.min(pages - 1);
        let mut acts: Vec<Option<DevAct>> = acts.into_iter().map(Some).collect();
        for (k, (label, col)) in labels.iter().enumerate().skip(page * per).take(per) {
            let i = k - page * per;
            let b = Bx::new(x + 16.0 + (i / ROWS) as f32 * (W - 32.0) / 2.0, top + (i % ROWS) as f32 * ROW, (W - 32.0) / 2.0 - 4.0, ROW - 2.0);
            let l = kit::row(c, &b, false);
            c.styled(label, b.x + 6.0, b.y + 18.0 + kit::nudge(l), 14.0, eg(*col), Face::Body, 0.0);
            if hit(&b) {
                *act = acts[k].take();
            }
        }
        if pages > 1 {
            let by = top + ROWS as f32 * ROW + 6.0;
            let (prev, next) = (Bx::new(x + 16.0, by, 32.0, 24.0), Bx::new(x + W - 48.0, by, 32.0, 24.0));
            kit::button(c, &prev, "<", Kind::Plain, page > 0);
            kit::button(c, &next, ">", Kind::Plain, page + 1 < pages);
            c.styled(&format!("page {} of {pages}", page + 1), x + W / 2.0 - 36.0, by + 17.0, 13.0, ega(TEXT, 0.7), Face::Body, 0.0);
            if hit(&prev) && page > 0 {
                *act = Some(DevAct::Page(-1));
            }
            if hit(&next) && page + 1 < pages {
                *act = Some(DevAct::Page(1));
            }
        }
    };
    let plain = |s: &str| (s.to_string(), TEXT);
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
            let mut labels: Vec<(String, Rgb)> = vec![plain("A lone bandit"), plain("Bandits (three)"), plain("Bandits (three, a mage)"), plain("Bandits (six)"), plain("Binders (three)")];
            let mut acts = vec![DevAct::Band(false, 1, false), DevAct::Band(false, 3, false), DevAct::Band(false, 3, true), DevAct::Band(false, 6, false), DevAct::Band(true, 3, false)];
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
            let labels = vec![plain("An hour on"), plain("Six hours on"), plain("A day on"), plain("A season on (12 days)")];
            let acts = vec![DevAct::Skip(1.0), DevAct::Skip(6.0), DevAct::Skip(24.0), DevAct::Skip(12.0 * 24.0)];
            list(labels, acts, &mut act);
        }
        Tab::Weather => {
            let forced = w.forced_sky.is_some();
            let mut labels: Vec<(String, Rgb)> = vec![(if forced { "The real weather (now forced)".into() } else { "The real weather".into() }, GOLD)];
            let mut acts = vec![DevAct::Weather(None)];
            for pr in PRESETS {
                labels.push(plain(pr.name()));
                acts.push(DevAct::Weather(Some(pr)));
            }
            list(labels, acts, &mut act);
        }
        Tab::Fire => {
            let labels = vec![plain("A campfire at the lead's feet"), plain("A standing torch by the lead")];
            list(labels, vec![DevAct::Campfire, DevAct::Torch], &mut act);
        }
        Tab::Squad => {
            let labels = vec![plain("Heal everyone"), plain("Bind every bleed"), plain("Knock the lead out"), plain("Soak the lead's pack"), plain("Chill the lead"), plain("Starve the lead"), plain("50 coin to the lead")];
            let acts = vec![DevAct::Heal, DevAct::Bind, DevAct::KnockOut, DevAct::Soak, DevAct::Chill, DevAct::Starve, DevAct::Coin];
            list(labels, acts, &mut act);
        }
        Tab::Rules => {
            let labels = gahturiyu_sim::sim::rules::RULES.iter().map(|r| {
                let on = w.rules.get(r.key).unwrap_or(true);
                (format!("{}: {}", r.name, if on { "on" } else { "off" }), if on { TEXT } else { OFF })
            });
            let labels: Vec<(String, Rgb)> = labels.collect();
            let acts = gahturiyu_sim::sim::rules::RULES.iter().map(|r| DevAct::Rule(r.key)).collect();
            list(labels, acts, &mut act);
        }
        Tab::Go => {
            use gahturiyu_sim::sim::island::Spot;
            let labels = vec![plain("The nearest town"), plain("Salt Isle: the shore"), plain("Salt Isle: the compound gate"), plain("Salt Isle: the dock"), plain("The test quarter")];
            let acts = vec![DevAct::GoTown, DevAct::GoIsland(Spot::Shore), DevAct::GoIsland(Spot::Gate), DevAct::GoIsland(Spot::Dock), DevAct::GoQuarter];
            list(labels, acts, &mut act);
        }
        Tab::Yard => {
            let open = w.test_quarter().is_some_and(|q| q.open);
            let labels = vec![plain("The squad into the arena"), plain("Stand the dummies up"), plain(if open { "Lock the quarter's gate" } else { "Open the quarter's gate" })];
            list(labels, vec![DevAct::ToArena, DevAct::ResetDummies, DevAct::Gate(!open)], &mut act);
            // The duel grid: a weapon's row against every armour set.
            let weapons = duelgrid::weapons();
            let wk = p.weapon % weapons.len().max(1);
            let name = weapons.get(wk).map(|k| gahturiyu_sim::sim::items::item(gahturiyu_sim::sim::items::id(k)).name).unwrap_or("?");
            let gy = top + 3.0 * ROW + 6.0;
            c.styled("DUEL GRID", x + 16.0, gy + 18.0, 13.0, eg(GOLD), Face::Title, 3.0);
            let (prev, next, run) = (Bx::new(x + 120.0, gy, 26.0, 24.0), Bx::new(x + 330.0, gy, 26.0, 24.0), Bx::new(x + 362.0, gy, 62.0, 24.0));
            kit::button(c, &prev, "<", Kind::Plain, true);
            kit::button(c, &next, ">", Kind::Plain, true);
            kit::button(c, &run, "Run", Kind::Primary, true);
            c.styled(name, x + 154.0, gy + 18.0, 14.0, eg(TEXT), Face::Body, 0.0);
            if hit(&prev) {
                act = Some(DevAct::Weapon(-1));
            }
            if hit(&next) {
                act = Some(DevAct::Weapon(1));
            }
            if hit(&run) {
                act = Some(DevAct::RunGrid);
            }
            let ty = gy + 44.0;
            c.styled("against", x + 16.0, ty, 12.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            c.styled("secs to down", x + 200.0, ty, 12.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            c.styled("wins of 8", x + 330.0, ty, 12.0, ega(TEXT, 0.6), Face::Italic, 0.0);
            for (k, (set, _)) in SETS.iter().enumerate() {
                let ry = ty + 20.0 + k as f32 * 19.0;
                c.styled(set, x + 16.0, ry, 13.0, eg(TEXT), Face::Body, 0.0);
                if let Some(r) = p.row.as_ref().filter(|r| r.weapon == weapons.get(wk).copied().unwrap_or("")) {
                    let s = r.secs[k].map(|t| format!("{t:.0}")).unwrap_or_else(|| "-".into());
                    c.styled(&s, x + 220.0, ry, 13.0, eg(TEXT), Face::Body, 0.0);
                    c.styled(&r.wins[k].to_string(), x + 345.0, ry, 13.0, eg(TEXT), Face::Body, 0.0);
                }
            }
        }
    }
    (act, Bx::new(x, y, W, h))
}

/// Carry out what was clicked. `lead`: whose pack and person the squad
/// buttons act on; `force`: the weather picture (as U), kept with the sim's.
pub fn apply(w: &mut World, p: &mut DevPanel, act: DevAct, lead: Option<gahturiyu_sim::sim::person::PersonId>, force: &mut Option<Preset>) -> Option<String> {
    // Things are put a little way from the lead (or inside the quarter's
    // gate, in the quarter), each at its own spot; in the quarter, foes and
    // beasts come in across the arena from the squad.
    let quarter = w.test_quarter().filter(|q| q.contains(w.squad.pos)).map(|q| (q.pad(), q.foe_side()));
    let base = match (quarter, lead) {
        (Some((pad, _)), _) => pad,
        (None, Some(m)) => w.person_pos(m).add(V2::new(18.0, 0.0)),
        (None, None) => w.squad.pos,
    };
    let near = |k: usize| base.add(V2::new(-((k % 4) as f32) * 5.0, -10.0 + (k / 4 % 5) as f32 * 5.0));
    let foe = |k: usize| match quarter {
        Some((_, side)) => side.add(V2::new(0.0, ((k % 3) as f32 - 1.0) * 3.0)),
        None => near(k).add(V2::new(-30.0, 0.0)),
    };
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
            let at = if quarter.is_some() { foe(k) } else { near(k) };
            w.dev_herd(sp, at, 3);
            Some(format!("A herd of {}", sp.def().name.to_lowercase()))
        }
        DevAct::Band(slavers, n, mage) => {
            if mage {
                w.spawn_bandits(foe(k), n, true);
            } else {
                w.dev_band(foe(k), n, slavers);
            }
            Some(match (slavers, n, mage) {
                (true, _, _) => format!("{n} Binders"),
                (_, 1, _) => "A lone bandit".into(),
                (_, _, true) => format!("{n} bandits, a mage among them"),
                _ => format!("{n} bandits"),
            })
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
            // In the sim (cold, wet, pace, sight, fires all follow) and the picture.
            w.set_forced_sky(pr.map(|x| x.sky(1.0)));
            *force = pr;
            Some(match pr {
                Some(x) => format!("Weather forced: {}", x.name()),
                None => "The real weather again".into(),
            })
        }
        DevAct::Campfire => {
            let at = w.person_pos(lead?).add(V2::new(1.8, 0.6));
            w.dev_fire(at);
            Some("A campfire, lit".into())
        }
        DevAct::Torch => {
            let m = lead?;
            Some(if w.dev_torch(m) { "A standing torch, lit".into() } else { "No torch set (in a fight?)".into() })
        }
        DevAct::Heal => {
            w.dev_heal_all();
            Some("Everyone whole again".into())
        }
        DevAct::Bind => {
            w.dev_bind_all();
            Some("Every bleed stopped".into())
        }
        DevAct::KnockOut => {
            let m = lead?;
            w.dev_knock_out(m);
            Some(format!("{} is knocked out", w.name_of(m)))
        }
        DevAct::Soak => {
            let m = lead?;
            w.dev_soak(m);
            Some(format!("{}'s pack is soaked through", w.name_of(m)))
        }
        DevAct::Chill => {
            let m = lead?;
            w.dev_chill(m);
            Some(format!("{} is chilled to the bone", w.name_of(m)))
        }
        DevAct::Starve => {
            let m = lead?;
            w.dev_starve(m);
            Some(format!("{} is starving", w.name_of(m)))
        }
        DevAct::Coin => {
            let m = lead?;
            w.dev_give(m, gahturiyu_sim::sim::items::id("coin"), 50);
            Some(format!("50 coin to {}", w.name_of(m)))
        }
        DevAct::Rule(key) => {
            w.rules.toggle(key);
            let on = w.rules.get(key).unwrap_or(true);
            let name = gahturiyu_sim::sim::rules::RULES.iter().find(|r| r.key == key).map(|r| r.name).unwrap_or(key);
            Some(format!("{name}: {}", if on { "on" } else { "off" }))
        }
        DevAct::GoTown => {
            w.dev_go_town();
            Some("To the nearest town".into())
        }
        DevAct::GoIsland(spot) => {
            w.dev_go_island(spot);
            Some(format!("To Salt Isle: {}", spot.name()))
        }
        DevAct::GoQuarter => Some(if w.enter_test_quarter() { "Into the test quarter".into() } else { "No test quarter here".into() }),
        DevAct::ToArena => Some(if w.dev_to_arena() { "Into the arena".into() } else { "No test quarter here".into() }),
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
