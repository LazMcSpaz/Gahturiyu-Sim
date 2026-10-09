//! The squad's outposts in the window (Part 7): the Build panel (B), placing
//! a building with a ghost footprint that says whether it can go there, walls
//! drawn as a chain of clicks, and placeholder boxes for what's built.
//!
//! Everything here reads the sim and sends it orders (`found_base`,
//! `place_building`, `place_wall`, `deconstruct`, `store_materials`); the
//! rules for where a building may go live in `sim/base.rs`.

use bevy::math::{Vec2, Vec3};

use gahturiyu_sim::sim::{
    base::{self, Bad, BaseId, Kind, Plan, State, BUILDINGS},
    baselife::{round_place, Job},
    crafting::RECIPES,
    geo::V2,
    items::item,
    materials::Grade,
    person::PersonId,
    World,
};

use super::cam::to3;
use super::hud::{Canvas, PANEL};
use super::mesh::Builder;
use super::palette::{self, eg, ega, Rgb, DIM, GOLD, TEXT, WARN};
use super::squadui::{Bx, Click};

/// A building waiting to be put down: the cursor carries its ghost.
#[derive(Clone, Debug, Default)]
pub struct Placing {
    pub def: usize,
    /// Facing, radians (Shift + wheel turns it).
    pub rot: f32,
    /// A wall's points so far.
    pub chain: Vec<V2>,
    /// Where the cursor's ghost is this frame, and whether it can go there.
    pub ghost: Vec<(Plan, Result<(), Bad>)>,
    /// The ground under the cursor.
    pub cursor: Option<V2>,
}

/// What the Build panel asks for.
pub enum BuildAction {
    Pick(usize),
    Store(BaseId),
    Deconstruct(BaseId, u32),
    Close,
    /// Switch between the Build and Base tabs.
    Tab(bool),
    Leave(PersonId, BaseId),
    PickUp(PersonId),
    /// The next job for a resident.
    NextJob(PersonId),
    /// The next recipe for a crafter.
    NextRecipe(PersonId),
    Seal(BaseId, u32),
}

/// The base nearest a point, if it's within reach of it.
pub fn base_near(w: &World, p: V2) -> Option<BaseId> {
    w.base_at(p)
}

const W: f32 = 700.0;
const ROW: f32 = 19.0;

/// The Build panel: the base here (if any), its sites, and what can be built.
pub fn build_panel(c: &Canvas, w: &World, here: V2, placing: Option<usize>, base_tab: bool, mouse: Vec2, click: Option<Click>) -> (Option<BuildAction>, Bx) {
    let bid = base_near(w, here);
    if base_tab {
        if let Some(b) = bid {
            return base_panel(c, w, b, mouse, click);
        }
    }
    let rows = BUILDINGS.len() + 6 + bid.and_then(|b| w.base(b)).map(|b| b.buildings.iter().filter(|x| !x.standing()).count().min(8) + 4).unwrap_or(1);
    let h = (rows as f32 * ROW + 70.0).min(c.h - 140.0);
    let r = Bx::new(c.w - W - 14.0, 60.0, W, h);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    let mut act = None;
    let clicked = |b: &Bx| click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false);
    c.text("Build", x, y, 17.0, GOLD);
    if bid.is_some() {
        let tab = Bx::new(x + 60.0, y - 16.0, 54.0, 20.0);
        c.text("Base", tab.x + 8.0, y, 16.0, if tab.contains(mouse) { GOLD } else { DIM });
        if clicked(&tab) {
            act = Some(BuildAction::Tab(true));
        }
    }
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(&close) {
        act = Some(BuildAction::Close);
    }
    y += 22.0;
    // The base here.
    match bid.and_then(|b| w.base(b)) {
        Some(b) => {
            let land = match b.land {
                base::Land::Wilds => "the open wilds".to_string(),
                base::Land::Unclaimed(s) => format!("{}'s land, no claim", w.settlements[s as usize].name),
            };
            c.text(&format!("{}  ·  {}  ·  reach {:.0} m  ·  {} beds", b.name, land, b.reach(), b.beds()), x, y, 14.0, TEXT);
            y += ROW;
            let store: Vec<String> = b.store.iter().map(|e| format!("{} {}", e.1, item(e.0).name.to_lowercase())).collect();
            c.text(&format!("Store: {}", if store.is_empty() { "empty".to_string() } else { store.join(", ") }), x, y, 13.0, DIM);
            let bt = Bx::new(r.x + r.w - 170.0, y - 14.0, 156.0, 18.0);
            c.rect(bt.x, bt.y, bt.w, bt.h, ega(GOLD, if bt.contains(mouse) { 0.3 } else { 0.15 }));
            c.text("Store materials", bt.x + 22.0, bt.y + 14.0, 13.0, GOLD);
            if clicked(&bt) {
                act = Some(BuildAction::Store(b.id));
            }
            y += ROW;
            let at_work: Vec<String> = b.builders.iter().map(|(m, id, _)| format!("{} on the {}", w.people[*m as usize].name().unwrap_or("?"), b.building(*id).map(|x| x.def().name.to_lowercase()).unwrap_or_default())).collect();
            c.text(&if at_work.is_empty() { "Nobody building.".to_string() } else { at_work.join("; ") }, x, y, 13.0, DIM);
            // Sites under way.
            for bl in b.buildings.iter().filter(|x| !x.standing()).take(8) {
                y += ROW;
                let d = bl.def();
                let status = match &bl.state {
                    State::Site(s) => {
                        let miss = bl.missing();
                        if !miss.is_empty() {
                            format!("needs {}", miss.iter().map(|(i, n)| format!("{n} {}", item(*i).name.to_lowercase())).collect::<Vec<_>>().join(", "))
                        } else if let Some(t) = s.done_at {
                            format!("{:.0}%  ·  done in {:.1} h", bl.progress(w.time) * 100.0, (t - w.time) / 3600.0)
                        } else if b.present.iter().any(|&m| w.can_build(m, bl.def as usize)) {
                            format!("{:.0}%  ·  queued", bl.progress(w.time) * 100.0)
                        } else {
                            format!("{:.0}%  ·  nobody here can build it", bl.progress(w.time) * 100.0)
                        }
                    }
                    State::Ruin => "ruin".to_string(),
                    State::Standing { .. } => String::new(),
                };
                c.text(&format!("{}: {status}", d.name), x + 8.0, y, 13.0, TEXT);
                if d.kind != Kind::Marker {
                    let del = Bx::new(r.x + r.w - 30.0, y - 14.0, 16.0, 16.0);
                    c.text("×", del.x + 3.0, del.y + 13.0, 15.0, if del.contains(mouse) { WARN } else { DIM });
                    if clicked(&del) {
                        act = Some(BuildAction::Deconstruct(b.id, bl.id));
                    }
                }
            }
        }
        None => {
            c.text("No base here. Lay a camp marker to found one.", x, y, 14.0, DIM);
        }
    }
    // What can be built.
    let mut group = "";
    for (i, d) in BUILDINGS.iter().enumerate() {
        if d.group != group {
            group = d.group;
            y += ROW + 4.0;
            c.text(group, x, y, 15.0, TEXT);
        }
        y += ROW;
        if y > r.y + r.h - 8.0 {
            break;
        }
        let row = Bx::new(r.x + 6.0, y - 15.0, r.w - 12.0, ROW);
        let who = w.squad_builders(i);
        let why: Option<String> = if who.is_empty() {
            Some(match d.skill {
                Some(s) => {
                    let best = w.squad.members.iter().filter(|&&m| w.people[m as usize].detail.as_ref().is_some_and(|x| x.crafts.contains(&s))).map(|&m| w.people[m as usize].stats.skill(s)).fold(None, |a: Option<f32>, v| Some(a.map_or(v, |a| a.max(v))));
                    match best {
                        Some(v) => format!("needs {} {:.0} (best {v:.0})", s.name().to_lowercase(), d.min_skill),
                        None => format!("no one knows {}", s.name().to_lowercase()),
                    }
                }
                None => "no one".to_string(),
            })
        } else if d.kind == Kind::Marker && bid.is_some() {
            Some("a base is already here".to_string())
        } else if d.kind != Kind::Marker && bid.is_none() {
            Some("lay a camp marker first".to_string())
        } else {
            None
        };
        let ok = why.is_none();
        if placing == Some(i) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.25));
        } else if row.contains(mouse) && ok {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.12));
        }
        let col = if ok { TEXT } else { DIM };
        c.text(d.name, x + 8.0, y, 14.0, col);
        let cost = if d.needs.is_empty() { "free".to_string() } else { d.needs.iter().map(|(k, n)| format!("{n} {}", item(gahturiyu_sim::sim::items::id(k)).name.to_lowercase())).collect::<Vec<_>>().join(", ") };
        c.text(&format!("{cost}  ·  {:.0} h", d.labour), x + 218.0, y, 13.0, col);
        let tail = match &why {
            Some(s) => s.clone(),
            None if who.len() == w.squad.members.len() => "anyone".to_string(),
            None if who.len() > 2 => format!("{} and {} more", w.people[who[0] as usize].name().unwrap_or("?"), who.len() - 1),
            None => who.iter().map(|&m| w.people[m as usize].name().unwrap_or("?").to_string()).collect::<Vec<_>>().join(", "),
        };
        let tw = c.width(&tail, 12.0);
        c.text(&tail, r.x + r.w - 14.0 - tw, y, 12.0, if ok { DIM } else { WARN });
        if ok && clicked(&row) {
            act = Some(BuildAction::Pick(i));
        }
    }
    (act, r)
}

/// A line under the cursor while placing.
pub fn placing_hint(c: &Canvas, p: &Placing, mouse: Vec2) {
    let d = &BUILDINGS[p.def];
    let bad = p.ghost.iter().find_map(|g| g.1.err());
    let (line, col) = match bad {
        Some(b) => (format!("{}: {}", d.name, b.why()), WARN),
        None if d.kind == Kind::Wall && p.chain.is_empty() => (format!("{}: click the first point", d.name), GOLD),
        None if d.kind == Kind::Wall => (format!("{}: click the next point  ·  Esc to finish the wall", d.name), GOLD),
        None => (format!("{}: click to place  ·  Shift + wheel turns it  ·  Esc to stop", d.name), GOLD),
    };
    let wd = c.width(&line, 14.0) + 14.0;
    c.rect(mouse.x + 16.0, mouse.y + 14.0, wd, 22.0, super::hud::shadow(0.7));
    c.text(&line, mouse.x + 23.0, mouse.y + 30.0, 14.0, col);
}

/// Work out the ghost for the cursor at `at`.
pub fn update_ghost(w: &World, p: &mut Placing, at: Option<V2>) {
    p.ghost.clear();
    p.cursor = at;
    let Some(at) = at else { return };
    let d = &BUILDINGS[p.def];
    if d.kind == Kind::Wall {
        match p.chain.last() {
            Some(&last) => {
                for mut plan in World::wall_plans(p.def, &[last, at]) {
                    let ok = w.check_place(&mut plan).map(|_| ());
                    p.ghost.push((plan, ok));
                }
            }
            None => {
                let mut plan = Plan { def: p.def, at, rot: p.rot, w: d.d, replaces: None };
                let ok = w.check_place(&mut plan).map(|_| ());
                p.ghost.push((plan, ok));
            }
        }
    } else {
        let mut plan = Plan { def: p.def, at, rot: p.rot, w: d.w, replaces: None };
        let ok = w.check_place(&mut plan).map(|_| ());
        p.ghost.push((plan, ok));
    }
}

const SITE: Rgb = [0.72, 0.62, 0.42];
const TIMBER: Rgb = [0.42, 0.31, 0.20];
const THATCH: Rgb = [0.55, 0.47, 0.28];
const RUBBLE: Rgb = [0.48, 0.47, 0.44];
const SOOT: Rgb = [0.12, 0.11, 0.10];
const GOOD: Rgb = [0.35, 0.95, 0.45];

/// Placeholder boxes for every base near the camera, and the ghost.
pub fn draw(w: &World, b: &mut Builder, gl: &mut Builder, fl: &mut Builder, ground: &dyn Fn(V2) -> f32, target: V2, radius: f32, placing: Option<&Placing>) {
    for base in &w.bases {
        if base.at.dist(target) > radius + base::REACH_MAX {
            continue;
        }
        for bl in &base.buildings {
            let d = bl.def();
            let g = ground(bl.at);
            let lift = match &bl.state {
                State::Site(_) => 0.15 + 0.85 * bl.progress(w.time),
                State::Standing { .. } => 1.0,
                State::Ruin => 0.2,
            };
            let main = match (&bl.state, d.armour) {
                (State::Ruin, _) => SOOT,
                (State::Site(_), _) => SITE,
                (_, base::Armour::Stone) => RUBBLE,
                _ => TIMBER,
            };
            match d.kind {
                Kind::Marker => {
                    // A ring of stones and, once laid, embers.
                    for k in 0..10 {
                        let a = k as f32 / 10.0 * std::f32::consts::TAU;
                        let p = bl.at.add(V2::new(a.cos(), a.sin()).scale(1.2));
                        b.block(to3(p, ground(p) - 0.05), 0.4, 0.35, 0.3 * lift + 0.05, a, RUBBLE);
                    }
                    if bl.standing() {
                        gl.column(to3(bl.at, g), 0.6, 0.1, 0.5, 6, palette::EMBER);
                    }
                }
                Kind::Field | Kind::Pen => {
                    // A low fence round the plot.
                    let cs = bl.corners();
                    for k in 0..4 {
                        let (p, q) = (cs[k], cs[(k + 1) % 4]);
                        let n = (p.dist(q) / 2.0).ceil() as usize;
                        for j in 0..=n {
                            let s = p.lerp(q, j as f32 / n as f32);
                            b.stick(to3(s, ground(s) - 0.2), to3(s, ground(s) + d.h * lift), 0.15, main);
                        }
                        b.stick(to3(p, ground(p) + d.h * lift * 0.8), to3(q, ground(q) + d.h * lift * 0.8), 0.1, main);
                    }
                    if d.kind == Kind::Field && bl.standing() {
                        b.block(to3(bl.at, g - 0.1), d.d - 0.8, bl.w - 0.8, 0.15, bl.rot, [0.30, 0.24, 0.15]);
                    }
                }
                Kind::Wall | Kind::Gate | Kind::Tower => {
                    b.block(to3(bl.at, g - 0.4), d.d, bl.w, d.h * lift + 0.4, bl.rot, main);
                    if d.kind == Kind::Gate && bl.standing() {
                        b.block(to3(bl.at, g + d.h * 0.85), d.d + 0.2, bl.w + 0.4, 0.4, bl.rot, SOOT);
                    }
                }
                _ => {
                    // Walls, and a roof that sits on them once the work's done.
                    let wall_h = d.h * 0.62;
                    b.block(to3(bl.at, g - 0.4), d.d, bl.w, (wall_h * lift + 0.4).max(0.5), bl.rot, main);
                    if bl.standing() {
                        let roof = if d.armour == base::Armour::Stone { RUBBLE } else { THATCH };
                        b.block(to3(bl.at, g + wall_h), d.d + 0.6, bl.w + 0.6, d.h - wall_h, bl.rot, roof);
                    }
                }
            }
        }
    }
    // The ghost.
    if let Some(p) = placing {
        for (plan, ok) in &p.ghost {
            let d = &BUILDINGS[plan.def];
            let col = if ok.is_ok() { GOOD } else { palette::WARN };
            let cs = base::corners(plan.at, plan.rot, plan.w, d.d);
            for k in 0..4 {
                let (a, c) = (cs[k], cs[(k + 1) % 4]);
                gl.stick(to3(a, ground(a) + 0.15), to3(c, ground(c) + 0.15), 0.18, col);
                gl.stick(to3(a, ground(a)), to3(a, ground(a) + d.h.min(4.0)), 0.12, col);
            }
            let mid: Vec3 = to3(plan.at, ground(plan.at) + 0.08);
            fl.block(mid, d.d, plan.w, 0.02, plan.rot, col);
        }
        for s in p.chain.windows(2) {
            gl.stick(to3(s[0], ground(s[0]) + 0.3), to3(s[1], ground(s[1]) + 0.3), 0.15, GOLD);
        }
    }
}

/// The Base tab: who lives here and what they do, who of the squad could
/// stay, the store, buildings that need care, and the log.
fn base_panel(c: &Canvas, w: &World, bid: BaseId, mouse: Vec2, click: Option<Click>) -> (Option<BuildAction>, Bx) {
    let b = w.base(bid).unwrap();
    let care: Vec<&base::Built> = b.buildings.iter().filter(|x| x.standing() && (x.rots() || x.hp_at(w.time) < x.def().hp * 0.999)).collect();
    let rows = 9 + b.residents.len() + b.present.len() + care.len().min(6) + 7;
    let h = (rows as f32 * ROW + 60.0).min(c.h - 140.0);
    let r = Bx::new(c.w - W - 14.0, 60.0, W, h);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    let mut act = None;
    let clicked = |b: &Bx| click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false);
    let button = |label: &str, bx: &Bx| {
        c.rect(bx.x, bx.y, bx.w, bx.h, ega(GOLD, if bx.contains(mouse) { 0.3 } else { 0.15 }));
        c.text(label, bx.x + 8.0, bx.y + 14.0, 13.0, GOLD);
    };
    let tab = Bx::new(x - 4.0, y - 16.0, 54.0, 20.0);
    c.text("Build", x, y, 16.0, if tab.contains(mouse) { GOLD } else { DIM });
    if clicked(&tab) {
        act = Some(BuildAction::Tab(false));
    }
    c.text("Base", x + 64.0, y, 17.0, GOLD);
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(&close) {
        act = Some(BuildAction::Close);
    }
    y += 24.0;
    let land = match b.land {
        base::Land::Wilds => "the open wilds".to_string(),
        base::Land::Unclaimed(s) => format!("{}'s land, no claim", w.settlements[s as usize].name),
    };
    c.text(&format!("{}  ·  {}  ·  {} beds  ·  store {:.0}/{:.0} kg  ·  worth {:.0}  ·  defence {:.0}", b.name, land, b.beds(), b.load(), b.capacity(), b.wealth, b.defence), x, y, 14.0, TEXT);

    // Residents.
    y += ROW + 6.0;
    c.text("Living here", x, y, 15.0, TEXT);
    if b.residents.is_empty() {
        y += ROW;
        c.text("Nobody. Leave a squad member here to work the base.", x + 8.0, y, 13.0, DIM);
    }
    for res in &b.residents {
        y += ROW;
        let p = &w.people[res.who as usize];
        c.text(p.name().unwrap_or("?"), x + 8.0, y, 14.0, TEXT);
        let jb = Bx::new(x + 140.0, y - 14.0, 96.0, 18.0);
        button(res.job.name(), &jb);
        if clicked(&jb) {
            act = Some(BuildAction::NextJob(res.who));
        }
        let mut doing = match (&res.cycle, res.job) {
            (Some(cy), _) => format!("at the {}, done in {:.1} h", round_place(b, cy), (cy.done_at - w.time) / 3600.0),
            (None, Job::Builder) => match b.builders.iter().find(|h| h.0 == res.who).and_then(|h| b.building(h.1)) {
                Some(bl) if bl.standing() => format!("mending the {}", bl.def().name.to_lowercase()),
                Some(bl) => format!("building the {}", bl.def().name.to_lowercase()),
                None => "nothing to build".to_string(),
            },
            (None, Job::Guard) => "on watch".to_string(),
            (None, Job::Idle) => String::new(),
            (None, Job::Farmer) => "needs a free field plot".to_string(),
            (None, Job::Cook) => "needs the kitchen, grain and timber".to_string(),
            (None, Job::Hauler) => "the store is full".to_string(),
            (None, Job::Crafter) => "needs a recipe, its shed and materials".to_string(),
        };
        if res.job == Job::Crafter {
            let rb = Bx::new(x + 244.0, y - 14.0, 150.0, 18.0);
            let name = res.recipe.map(|ri| item(RECIPES[ri as usize].item(Grade::Common)).name.to_string()).unwrap_or_else(|| "pick a recipe".to_string());
            button(&name, &rb);
            if clicked(&rb) {
                act = Some(BuildAction::NextRecipe(res.who));
            }
            doing = doing.replace("needs a recipe, its shed and materials", "needs its shed and materials");
        }
        let dx = if res.job == Job::Crafter { 402.0 } else { 244.0 };
        let hunger = w.hunger_of(res.who).unwrap_or(0.0);
        let tail = format!("{doing}{}hunger {hunger:.0}", if doing.is_empty() { "" } else { "  ·  " });
        c.text(&tail, x + dx, y, 12.0, DIM);
        let pb = Bx::new(r.x + r.w - 82.0, y - 14.0, 68.0, 18.0);
        button("Pick up", &pb);
        if clicked(&pb) {
            act = Some(BuildAction::PickUp(res.who));
        }
    }
    // The squad here.
    if !b.present.is_empty() {
        y += ROW + 6.0;
        c.text("The squad here", x, y, 15.0, TEXT);
        for &m in &b.present {
            y += ROW;
            c.text(w.people[m as usize].name().unwrap_or("?"), x + 8.0, y, 14.0, TEXT);
            let doing = b.builders.iter().find(|h| h.0 == m).and_then(|h| b.building(h.1)).map(|bl| format!("{} the {}", if bl.standing() { "mending" } else { "building" }, bl.def().name.to_lowercase())).unwrap_or_default();
            c.text(&doing, x + 140.0, y, 12.0, DIM);
            let lb = Bx::new(r.x + r.w - 102.0, y - 14.0, 88.0, 18.0);
            button("Leave here", &lb);
            if clicked(&lb) {
                act = Some(BuildAction::Leave(m, bid));
            }
        }
    }
    // Buildings needing care.
    if !care.is_empty() {
        y += ROW + 6.0;
        c.text(&format!("Upkeep  ·  {} pitch in store", b.count_in_store(gahturiyu_sim::sim::items::id("pitch"))), x, y, 15.0, TEXT);
        for bl in care.iter().take(6) {
            y += ROW;
            let pct = bl.hp_at(w.time) / bl.def().hp * 100.0;
            let note = if bl.rots() { "thatch rotting (1% a day)" } else { "sealed" };
            c.text(&format!("{}: {pct:.0}%  ·  {note}", bl.def().name), x + 8.0, y, 13.0, TEXT);
            if bl.rots() {
                let sb = Bx::new(r.x + r.w - 102.0, y - 14.0, 88.0, 18.0);
                button("Seal (pitch)", &sb);
                if clicked(&sb) {
                    act = Some(BuildAction::Seal(bid, bl.id));
                }
            }
        }
    }
    // The store.
    y += ROW + 6.0;
    c.text("Store", x, y, 15.0, TEXT);
    y += ROW;
    let store: Vec<String> = b.store.iter().map(|e| format!("{} {}", e.1, item(e.0).name.to_lowercase())).collect();
    c.text(&if store.is_empty() { "empty".to_string() } else { store.join(", ") }, x + 8.0, y, 13.0, DIM);
    // The log.
    y += ROW + 6.0;
    c.text("Lately", x, y, 15.0, TEXT);
    for (t, line) in b.log.iter().rev().take(5) {
        y += ROW;
        if y > r.y + r.h - 6.0 {
            break;
        }
        c.text(&format!("{}  {line}", super::hud::hhmm(*t)), x + 8.0, y, 13.0, DIM);
    }
    (act, r)
}
