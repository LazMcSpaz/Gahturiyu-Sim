//! Going through a body or a chest (#166 UI-Loot; the feel plan's 5.5,
//! after Baldur's Gate 3): a small box beside the thing on screen, not a
//! big panel in the middle. Its name, whose it is (in red when taking is
//! theft, with the odds of being seen, which Laz keeps), a short list to
//! click from, and Take all (Enter). A chest the squad can also put things
//! in has a Put in tab, which lists the pack of whoever is at it.
//!
//! Drawing only: what's clicked goes back to `app.rs` as a `LootAct`.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    items::item,
    loot::{LootRef, Source},
    person::PersonId,
    World,
};

use super::hud::{Canvas, Face};
use super::kit;
use super::palette::{eg, ega, Rgb, BRASS, BRASS_LIGHT, TEXT, WARN};
use super::squadui::{Bx, Click};

pub enum LootAct {
    Take(PersonId, Source, LootRef),
    TakeAll(PersonId, Source),
    /// Put pack entry k into the open container.
    Put(PersonId, usize),
    Close(PersonId),
    /// Show what's in it, or the pack to put from.
    Tab(bool),
}

const W: f32 = 320.0;
const ROW: f32 = 26.0;
/// Rows listed before the rest are counted.
const LINES: usize = 9;
const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const FAINT: Rgb = [0.55, 0.50, 0.42];
const SAFE: Rgb = [0.61, 0.76, 0.48];

/// Who of the squad is going through something, and what.
pub fn open_loot(w: &World) -> Option<(PersonId, Source)> {
    w.squad.members.iter().find_map(|&m| w.source_now(m).map(|s| (m, s)))
}

/// Where the thing being gone through stands in the world.
pub fn loot_spot(w: &World, src: Source) -> gahturiyu_sim::sim::geo::V2 {
    match src {
        Source::Body(p) => w.body_pos(p),
        Source::Chest(id) => w.container(id).map(|k| k.pos).unwrap_or(w.squad.pos),
        Source::Store(b) => w.base(b).map(|b| b.at).unwrap_or(w.squad.pos),
    }
}

/// The box, beside `at` (the thing on screen, if it's in view; else the
/// middle). `putting`: the Put in tab is showing. `take_key`: Enter was
/// pressed (Take all).
pub fn loot_panel(c: &Canvas, w: &World, at: Option<Vec2>, putting: bool, take_key: bool, click: Option<Click>) -> (Option<LootAct>, Option<Bx>) {
    let Some((who, src)) = open_loot(w) else { return (None, None) };
    let things = w.contents(src);
    // (A base's store and a chest take things in as well as giving them up.)
    let chest = matches!(src, Source::Chest(_) | Source::Store(_));
    let putting = chest && putting;
    let mine: Vec<(usize, gahturiyu_sim::sim::items::ItemId, u16)> = w.people[who as usize].detail.as_ref().map(|d| d.gear.bag.iter().enumerate().map(|(k, e)| (k, e.0, e.1)).collect()).unwrap_or_default();
    let listed = if putting { mine.len() } else { things.len() };
    let rows = listed.clamp(1, LINES) + usize::from(listed > LINES);

    // The heading: what it is, and whose.
    let (title, whose, whose_col) = match src {
        Source::Body(body) => {
            let p = &w.people[body as usize];
            (p.name().unwrap_or("Them").to_string(), if p.dead { "Dead".to_string() } else { "Out cold".to_string() }, SUB)
        }
        Source::Chest(id) => match w.container(id) {
            Some(k) => {
                let o = w.container_owner(id).unwrap_or(k.owner);
                let owner = super::interiors::owner_text(w, o);
                let wild = gahturiyu_sim::sim::containers::is_wild(id);
                let theirs = !wild && !matches!(o, gahturiyu_sim::sim::containers::Owner::Nobody);
                (k.what.name().to_string(), if wild { "Nobody's".to_string() } else { owner }, if theirs { WARN } else { SUB })
            }
            None => ("Container".into(), String::new(), SUB),
        },
        Source::Store(b) => match w.base(b) {
            Some(b) => ("Store".to_string(), b.name.clone(), SUB),
            None => ("Store".into(), String::new(), SUB),
        },
    };
    // The theft odds (Laz keeps these), under the owner.
    let odds = match src {
        Source::Chest(id) if !gahturiyu_sim::sim::containers::is_wild(id) => {
            let at = w.container(id).map(|k| k.pos).unwrap_or(w.squad.pos);
            let p = w.catch_chance(who, at, id.0);
            Some(if p <= 0.0 { ("Nobody can see you.".to_string(), SAFE) } else { (format!("{:.0}% chance someone sees", p * 100.0), WARN) })
        }
        _ => None,
    };
    let h = 64.0 + if odds.is_some() { 18.0 } else { 0.0 } + if chest { 30.0 } else { 0.0 } + rows as f32 * ROW + 54.0;
    // Beside the thing, on whichever side has room.
    let (x, y) = match at {
        Some(p) => {
            let right = p.x + 46.0;
            let x = if right + W < c.w - 12.0 { right } else { p.x - 46.0 - W };
            (x.clamp(12.0, c.w - W - 12.0), (p.y - h * 0.5).clamp(60.0, c.h - super::frame::BOTTOM_CLEAR - h - 8.0))
        }
        None => (c.w * 0.5 - W * 0.5, 110.0),
    };
    let r = Bx::new(x, y, W, h);
    c.frame_box(r.x, r.y, r.w, r.h);
    let clicked = |b: &Bx| click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false);
    let mut act = None;
    let x0 = r.x + 16.0;
    let right = r.x + r.w - 16.0;
    let mut yy = r.y + 30.0;
    // A person's name in small capitals (Cinzel has no marks for the
    // people's letters); a thing's in the title face.
    if matches!(src, Source::Body(_)) {
        c.styled(&title, x0, yy, 19.0, eg(HEAD), Face::Caps, 0.5);
    } else {
        c.styled(&title.to_uppercase(), x0, yy, 17.0, eg(HEAD), Face::Title, 3.0);
    }
    let close = Bx::new(r.x + r.w - 30.0, r.y + 8.0, 22.0, 22.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 5.0, close.y + 17.0 + kit::nudge(cl), 20.0, eg(if cl.hot { BRASS_LIGHT } else { FAINT }), Face::Body, 0.0);
    if clicked(&close) {
        act = Some(LootAct::Close(who));
    }
    yy += 20.0;
    c.styled(&whose, x0, yy, 14.0, eg(whose_col), Face::Italic, 0.0);
    if let Some((line, col)) = &odds {
        yy += 18.0;
        c.styled(line, x0, yy, 13.0, eg(*col), Face::Italic, 0.0);
    }
    yy += 10.0;
    // The tabs: what's in it, or the pack (a chest only).
    if chest {
        let half = (r.w - 32.0) / 2.0;
        for (k, (label, on)) in [("IN IT", !putting), ("PUT IN", putting)].into_iter().enumerate() {
            let b = Bx::new(x0 + k as f32 * half, yy + 2.0, half, 24.0);
            let l = kit::look(&b);
            let col = if on { BRASS_LIGHT } else if l.hot { [0.85, 0.80, 0.68] } else { FAINT };
            let tw = c.styled_width(label, 12.0, Face::Title, 2.0);
            c.styled(label, b.x + (b.w - tw) / 2.0, b.y + 17.0 + kit::nudge(l), 12.0, eg(col), Face::Title, 2.0);
            c.rule(b.x + 6.0, b.x + b.w - 6.0, b.y + b.h, if on { 1.6 } else { 0.8 }, ega(if on { BRASS_LIGHT } else { BRASS }, if on { 0.9 } else { 0.35 }));
            kit::flash(c, b.x, b.y, b.w, b.h, l.flash);
            if clicked(&b) && !on {
                act = Some(LootAct::Tab(k == 1));
            }
        }
        yy += 30.0;
    } else {
        c.rule(x0, right, yy + 2.0, 0.8, ega(BRASS, 0.5));
    }
    let top = yy + 6.0;
    let label = |n: u16, it| if n > 1 { format!("{} × {}", n, item(it).name) } else { item(it).name.to_string() };
    if putting {
        if mine.is_empty() {
            c.styled("Nothing in the pack.", x0, top + 18.0, 14.0, eg(FAINT), Face::Italic, 0.0);
        }
        for (i, (k, it, n)) in mine.iter().take(LINES).enumerate() {
            let row = Bx::new(r.x + 6.0, top + i as f32 * ROW, r.w - 12.0, ROW);
            let l = kit::row(c, &row, false);
            let dy = kit::nudge(l) + row.y + 18.0;
            c.styled(&label(*n, *it), x0, dy, 15.0, eg(TEXT), Face::Body, 0.0);
            c.styled_right(if l.hot { "put in" } else { "" }, right, dy, 12.0, eg(BRASS_LIGHT), Face::Italic, 0.0);
            if clicked(&row) {
                act = Some(LootAct::Put(who, *k));
            }
        }
    } else {
        if things.is_empty() {
            c.styled(if chest { "Empty." } else { "Nothing left on them." }, x0, top + 18.0, 14.0, eg(FAINT), Face::Italic, 0.0);
        }
        for (i, (what, it, n)) in things.iter().take(LINES).enumerate() {
            let row = Bx::new(r.x + 6.0, top + i as f32 * ROW, r.w - 12.0, ROW);
            let l = kit::row(c, &row, false);
            let dy = kit::nudge(l) + row.y + 18.0;
            let worn = matches!(what, LootRef::Worn(_));
            c.styled(&label(*n, *it), x0, dy, 15.0, eg(TEXT), Face::Body, 0.0);
            let side = if l.hot { "take".to_string() } else if worn { "worn".to_string() } else { String::new() };
            c.styled_right(&side, right, dy, 12.0, eg(if l.hot { BRASS_LIGHT } else { FAINT }), Face::Italic, 0.0);
            if clicked(&row) {
                act = Some(LootAct::Take(who, src, *what));
            }
        }
    }
    if listed > LINES {
        let yy = top + LINES as f32 * ROW + 18.0;
        c.styled(&format!("And {} more.", listed - LINES), x0, yy, 13.0, eg(FAINT), Face::Italic, 0.0);
    }
    // Take all, and how full the pack is.
    let by = r.y + r.h - 40.0;
    let all = Bx::new(x0, by, 124.0, 28.0);
    let can = !things.is_empty();
    kit::button(c, &all, "TAKE ALL", kit::Kind::Primary, can);
    if can && (clicked(&all) || take_key) {
        act = Some(LootAct::TakeAll(who, src));
    }
    let load = w.load_of(who);
    let name = w.people[who as usize].name().unwrap_or("?").to_string();
    c.styled_right(&name, right, by + 12.0, 13.0, eg(SUB), Face::Caps, 0.0);
    // The pack's weight as a bar: amber near full, red over.
    let (bx, bw) = (right - 110.0, 110.0);
    c.rect(bx, by + 20.0, bw, 4.0, ega([0.0, 0.0, 0.0], 0.45));
    let col = if load > 1.0 { [0.85, 0.36, 0.26] } else if load > 0.8 { [0.88, 0.64, 0.26] } else { [0.70, 0.62, 0.45] };
    c.rect(bx, by + 20.0, bw * load.clamp(0.0, 1.0), 4.0, eg(col));
    (act, Some(r))
}
