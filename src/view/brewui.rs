//! Brewing, at the alchemy table (Part 4; `sim/brewing.rs`). The making
//! screen hands over to this one at an alchemy table.
//!
//! Left: the ingredients in the packs of those at the table, each with what
//! the brewer knows of its four effects ("?" for the rest); a click puts one
//! in the mix. Under them, the mixes the brewer remembers; a click fills the
//! mix with one.
//!
//! Right: the mix (a click takes one back out), what it will carry as far
//! as the brewer knows (and how many shared effects they don't know yet),
//! how strong it comes out at this table, the table's tools, who brews, and
//! a count beside the Brew button.
//!
//! Everything shown is read from the sim (`brew_preview`, `brew_blockers`,
//! `known_effects`, `table_kit`); this file only draws and reports clicks.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    brewing::BREW_SECS,
    crafting::time_words,
    items::{self, herbs, item, ItemId},
    materials::Craft,
    person::PersonId,
    stats::Skill,
    World,
};

use super::hud::{Canvas, Face};
use super::kit;
use super::makeui::{MakeAct, Making, MOST};
use super::palette::{eg, ega, race_color, Rgb, BRASS, BRASS_LIGHT, TEXT};
use super::squadui::{Bx, Click};

const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const GROUP: Rgb = [0.66, 0.57, 0.42];
const FAINT: Rgb = [0.55, 0.50, 0.42];
const SHORT: Rgb = [0.85, 0.47, 0.37];
const GOOD: Rgb = [0.61, 0.76, 0.48];
const BRIGHT: Rgb = [1.0, 0.95, 0.85];
const PANEL_TOP: Rgb = [0.165, 0.125, 0.094];
const PANEL_BOT: Rgb = [0.11, 0.082, 0.063];
const DARK: Rgb = [0.047, 0.035, 0.024];

const W: f32 = 900.0;
const LIST_W: f32 = 380.0;
const ROW_H: f32 = 44.0;

/// Who of the squad could brew at this table: those at it who've taken up
/// alchemy, the most skilled first.
pub fn brewers(w: &World, m: &Making) -> Vec<PersonId> {
    let mut out: Vec<PersonId> = w.bench_hands(&m.bench).into_iter().filter(|&p| w.knows_craft(p, Craft::Alchemy)).collect();
    out.sort_by(|a, b| w.people[*b as usize].stats.skill(Skill::Alchemy).total_cmp(&w.people[*a as usize].stats.skill(Skill::Alchemy)).then(a.cmp(b)));
    out
}

/// The brewer the screen shows: the one picked if they still can, else the
/// most skilled at the table.
pub fn brewer(w: &World, m: &Making) -> Option<PersonId> {
    let all = brewers(w, m);
    m.who.filter(|p| all.contains(p)).or_else(|| all.first().copied())
}

/// The ingredients in the packs of those at the table, and how many.
fn stock(w: &World, m: &Making) -> Vec<(ItemId, u16)> {
    let mut out: Vec<(ItemId, u16)> = Vec::new();
    for p in w.bench_hands(&m.bench) {
        let Some(d) = w.people[p as usize].detail.as_ref() else { continue };
        for e in d.gear.bag.iter().filter(|e| items::ingredient(e.0).is_some()) {
            match out.iter_mut().find(|x| x.0 == e.0) {
                Some(x) => x.1 += e.1,
                None => out.push((e.0, e.1)),
            }
        }
    }
    out.sort_by_key(|x| item(x.0).name);
    out
}

/// A strength in words.
fn strength_words(x: f32) -> &'static str {
    match x {
        x if x >= 1.6 => "very strong",
        x if x >= 1.15 => "strong",
        x if x >= 0.85 => "plain",
        _ => "weak",
    }
}

/// How long harm lingers, in words.
fn harm_words(x: f32) -> &'static str {
    match x {
        x if x <= 0.25 => "a flicker",
        x if x <= 0.6 => "short",
        x if x <= 1.1 => "as long as plain",
        _ => "long",
    }
}

/// Draw the screen; what was clicked, the box it covers, and the scroll.
pub fn brewing(c: &Canvas, w: &World, m: &Making, _mouse: Vec2, click: Option<Click>) -> (Option<MakeAct>, Bx, f32) {
    let b = &m.bench;
    let clicked = |bx: &Bx| click.map(|k| bx.contains(k.at) && !k.right).unwrap_or(false);
    let mut act = None;
    let who = brewer(w, m);

    let h = (c.h - super::frame::BOTTOM_CLEAR - 70.0).clamp(420.0, 900.0);
    let r = Bx::new(40.0, 50.0, W.min(c.w - 80.0), h);
    let dark_h = c.h - super::frame::BOTTOM_CLEAR + 8.0;
    let solid = r.x + r.w + 24.0;
    c.rect(0.0, 0.0, solid, dark_h, eg(DARK));
    c.grad(solid, 0.0, c.w * 0.14, dark_h, eg(DARK), ega(DARK, 0.0), true);

    c.styled("THE ALCHEMY TABLE", r.x + 6.0, r.y + 34.0, 34.0, eg(HEAD), Face::Title, 8.0);
    if !b.place.is_empty() {
        c.styled(&b.place, r.x + 8.0, r.y + 58.0, 17.0, eg(SUB), Face::Italic, 0.0);
    }
    let close = Bx::new(r.x + r.w - 34.0, r.y + 6.0, 30.0, 30.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 8.0, close.y + 24.0 + kit::nudge(cl), 26.0, eg(if cl.hot { BRASS_LIGHT } else { GROUP }), Face::Body, 0.0);
    if clicked(&close) {
        act = Some(MakeAct::Close);
    }
    c.rule_in(r.x, r.x + r.w, r.y + 72.0, 2.0, eg(BRASS), false);

    let top = r.y + 90.0;
    let col_h = r.y + r.h - top;
    let column = |x: f32, wd: f32| {
        c.grad(x, top, wd, col_h, ega(PANEL_TOP, 0.94), ega(PANEL_BOT, 0.94), false);
        c.rect(x, top, wd, 2.0, eg(BRASS));
        c.rect(x, top + col_h - 2.0, wd, 2.0, eg(BRASS));
    };

    // ---- The ingredients, and remembered mixes ---------------------------------
    column(r.x, LIST_W);
    let stock = stock(w, m);
    let used = |id: ItemId| m.mix.iter().filter(|&&x| x == id).count() as u16;
    let mut y = top + 8.0;
    c.styled("INGREDIENTS", r.x + 12.0, y + 18.0, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 28.0;
    if stock.is_empty() {
        c.styled("No ingredients in the packs here.", r.x + 14.0, y + 20.0, 16.0, eg(SUB), Face::Italic, 0.0);
        y += 30.0;
    }
    let mine = |id: ItemId| -> String {
        let known = who.map(|p| w.known_effects(p, id)).unwrap_or([None; 4]);
        known.iter().map(|k| k.map(|a| a.name().to_string()).unwrap_or_else(|| "?".into())).collect::<Vec<_>>().join(" · ")
    };
    let list_bottom = top + col_h * 0.66;
    let most = ((stock.len() as f32 * ROW_H) - (list_bottom - y)).max(0.0);
    let scroll = m.scroll.clamp(0.0, most);
    for (k, &(id, n)) in stock.iter().enumerate() {
        let yy = y + k as f32 * ROW_H - scroll;
        if yy < y - 1.0 || yy + ROW_H > list_bottom {
            continue;
        }
        let bx = Bx::new(r.x, yy, LIST_W, ROW_H);
        let left = n.saturating_sub(used(id));
        let can = left > 0 && m.mix.len() < herbs::MIX_INGREDIENTS;
        let l = kit::row(c, &bx, false);
        let fade = if can { 1.0 } else { 0.5 };
        c.styled(&format!("{} ×{left}", item(id).name), bx.x + 12.0, bx.y + 19.0 + kit::nudge(l), 18.0, ega(TEXT, fade), Face::Body, 0.0);
        let kind = items::ingredient(id).map(|d| format!("{:?}", d.kind).to_lowercase()).unwrap_or_default();
        c.styled_right(&kind, bx.x + bx.w - 12.0, bx.y + 19.0, 13.0, ega(FAINT, fade), Face::Italic, 0.0);
        c.styled(&mine(id), bx.x + 12.0, bx.y + 37.0, 13.0, ega(FAINT, fade), Face::Body, 0.0);
        if can && clicked(&bx) {
            act = Some(MakeAct::Add(id));
        }
    }
    // Remembered mixes.
    let mut y = list_bottom + 8.0;
    c.rect(r.x + 10.0, y - 4.0, LIST_W - 20.0, 1.0, ega(BRASS, 0.22));
    c.styled("REMEMBERED", r.x + 12.0, y + 16.0, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 24.0;
    let mixes = who.and_then(|p| w.lore_of(p)).map(|l| l.mixes.clone()).unwrap_or_default();
    if mixes.is_empty() {
        c.styled("None yet: a brewed mix is remembered.", r.x + 14.0, y + 18.0, 15.0, eg(FAINT), Face::Italic, 0.0);
    }
    // (One who writes can put a mix on a sheet: "write" at the row's end.)
    let writes = who.is_some_and(|p| w.knows_craft(p, Craft::Inscription));
    for (k, rm) in mixes.iter().enumerate() {
        if y + 30.0 > top + col_h - 40.0 {
            break;
        }
        let bx = Bx::new(r.x, y, LIST_W - if writes { 64.0 } else { 0.0 }, 30.0);
        let l = kit::row(c, &bx, false);
        let have = rm.ings.iter().all(|&i| stock.iter().find(|s| s.0 == i).map(|s| s.1).unwrap_or(0) >= rm.ings.iter().filter(|&&x| x == i).count() as u16);
        c.styled(&rm.name, bx.x + 12.0, bx.y + 20.0 + kit::nudge(l), 16.0, ega(if have { TEXT } else { FAINT }, 1.0), Face::Body, 0.0);
        if clicked(&bx) {
            act = Some(MakeAct::Recall(rm.ings.clone()));
        }
        if let (true, Some(p)) = (writes, who) {
            let wb = Bx::new(r.x + LIST_W - 60.0, y + 3.0, 50.0, 24.0);
            let wl = kit::row(c, &wb, false);
            c.styled("write", wb.x + 8.0, wb.y + 17.0 + kit::nudge(wl), 14.0, eg(if wl.hot { BRASS_LIGHT } else { GROUP }), Face::Italic, 0.0);
            if clicked(&wb) {
                act = Some(MakeAct::Write(p, k));
            }
        }
        y += 30.0;
    }
    // Teach the others at the table what the brewer knows.
    if let Some(p) = who.filter(|_| w.bench_hands(&m.bench).len() > 1 && w.lore_of(who.unwrap()).is_some_and(|l| !l.known.is_empty())) {
        let tb = Bx::new(r.x + 12.0, top + col_h - 36.0, LIST_W - 24.0, 28.0);
        let tl = kit::row(c, &tb, false);
        c.styled("Teach the others what you know", tb.x + 8.0, tb.y + 19.0 + kit::nudge(tl), 15.0, eg(if tl.hot { BRASS_LIGHT } else { SUB }), Face::Italic, 0.0);
        if clicked(&tb) {
            act = Some(MakeAct::Teach(p));
        }
    }

    // ---- The mix ------------------------------------------------------------------
    let dx = r.x + LIST_W + 24.0;
    let dw = r.w - LIST_W - 24.0;
    column(dx, dw);
    let x = dx + 24.0;
    let right = dx + dw - 24.0;
    let mut y = top + 30.0;
    c.styled("THE MIX", x, y, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 12.0;
    let card_w = (right - x - 10.0) / 2.0;
    for k in 0..herbs::MIX_INGREDIENTS {
        let cx = x + (k % 2) as f32 * (card_w + 10.0);
        let cy = y + (k / 2) as f32 * 50.0;
        let bx = Bx::new(cx, cy, card_w, 42.0);
        c.rect(cx, cy, card_w, 42.0, ega(DARK, 0.45));
        match m.mix.get(k) {
            Some(&id) => {
                let l = kit::row(c, &bx, false);
                c.rect_lines(cx, cy, card_w, 42.0, 1.0, ega(BRASS, 0.45));
                c.styled(item(id).name, cx + 12.0, cy + 26.0 + kit::nudge(l), 17.0, eg(TEXT), Face::Body, 0.0);
                c.styled_right("×", cx + card_w - 10.0, cy + 26.0, 16.0, eg(FAINT), Face::Body, 0.0);
                if clicked(&bx) {
                    act = Some(MakeAct::Remove(k));
                }
            }
            None => {
                c.rect_lines(cx, cy, card_w, 42.0, 1.0, ega(BRASS, 0.15));
                c.styled("empty", cx + 12.0, cy + 26.0, 15.0, ega(FAINT, 0.6), Face::Italic, 0.0);
            }
        }
    }
    y += 2.0 * 50.0 + 16.0;

    // What it will carry, as far as the brewer knows: an effect shows once
    // they know it on two of the ingredients that share it.
    c.styled("WILL CARRY", x, y, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 22.0;
    let shared = herbs::shared(&m.mix);
    let knows = |a: herbs::Alch| who.is_some_and(|p| m.mix.iter().filter(|&&i| w.known_effects(p, i).contains(&Some(a))).count() >= 2);
    let shown: Vec<herbs::Alch> = shared.iter().copied().filter(|&a| knows(a)).collect();
    let hidden = shared.len() - shown.len();
    if m.mix.len() < 2 {
        c.styled("Put in two to four ingredients, of two kinds or more.", x, y, 16.0, eg(FAINT), Face::Italic, 0.0);
        y += 22.0;
    } else if shared.is_empty() {
        c.styled("Nothing: they share no effect (you'd learn that much).", x, y, 16.0, eg(SHORT), Face::Italic, 0.0);
        y += 22.0;
    } else {
        for a in &shown {
            c.styled(a.name(), x + 6.0, y, 17.0, eg(if a.harmful() { SHORT } else { GOOD }), Face::Body, 0.0);
            y += 22.0;
        }
        if hidden > 0 {
            let words = if hidden == 1 { "and one shared effect you don't know yet".to_string() } else { format!("and {hidden} shared effects you don't know yet") };
            c.styled(&words, x + 6.0, y, 16.0, eg(SUB), Face::Italic, 0.0);
            y += 22.0;
        }
    }
    y += 6.0;
    c.rect(x, y, right - x, 1.0, ega(BRASS, 0.22));
    y += 24.0;
    if let Some(p) = who {
        let mix = w.brew_preview(p, &m.mix);
        c.styled("Comes out", x, y, 16.0, eg(SUB), Face::Body, 0.0);
        let harm = if mix.harms() { format!(", harm lasts {}", harm_words(mix.bad_for)) } else { String::new() };
        c.styled_right(&format!("{}{harm}", strength_words(mix.power)), right, y, 16.0, eg(TEXT), Face::Body, 0.0);
        y += 22.0;
    }
    let kit_ = w.table_kit(b.at);
    let tools: Vec<&str> = [("mortar", kit_.mortar), ("retort", kit_.retort), ("calcinator", kit_.calcinator), ("alembic", kit_.alembic)].iter().filter(|t| t.1.is_some()).map(|t| t.0).collect();
    c.styled("Tools fitted", x, y, 16.0, eg(SUB), Face::Body, 0.0);
    c.styled_right(&if tools.is_empty() { "none".to_string() } else { tools.join(", ") }, right, y, 16.0, eg(TEXT), Face::Body, 0.0);
    y += 22.0;
    c.styled("Time a bottle", x, y, 16.0, eg(SUB), Face::Body, 0.0);
    c.styled_right(&time_words(BREW_SECS), right, y, 16.0, eg(TEXT), Face::Body, 0.0);
    y += 14.0;
    c.rect(x, y, right - x, 1.0, ega(BRASS, 0.22));

    // Who brews.
    y += 26.0;
    c.styled("WHO BREWS", x, y, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 10.0;
    for p in brewers(w, m).into_iter().take(3) {
        let bx = Bx::new(x, y, right - x, 40.0);
        let on = who == Some(p);
        c.rect(bx.x, bx.y, bx.w, bx.h, ega(DARK, 0.38));
        c.rect_lines(bx.x, bx.y, bx.w, bx.h, if on { 1.6 } else { 1.0 }, if on { eg(HEAD) } else { ega(BRASS, 0.28) });
        if !on {
            kit::row(c, &bx, false);
        }
        let pp = &w.people[p as usize];
        c.circle(bx.x + 22.0, bx.y + 20.0, 11.0, ega(race_color(pp.race), 0.85));
        c.styled(pp.name().unwrap_or("?"), bx.x + 44.0, bx.y + 26.0, 17.0, eg(if on { BRASS_LIGHT } else { TEXT }), Face::Caps, 0.0);
        let sk = pp.stats.skill(Skill::Alchemy);
        c.styled_right(&format!("alchemy {sk:.0}"), bx.x + bx.w - 12.0, bx.y + 25.0, 15.0, eg(FAINT), Face::Italic, 0.0);
        if clicked(&bx) {
            act = Some(MakeAct::Who(p));
        }
        y += 46.0;
    }

    // How many, and Brew.
    let by = top + col_h - 74.0;
    let blockers = who.map(|p| w.brew_blockers(p, &m.mix)).unwrap_or_default();
    let why = match (who, blockers.first()) {
        (None, _) => "Nobody at the table has taken up alchemy.".to_string(),
        (Some(_), None) => String::new(),
        (Some(p), Some(b)) => format!("{} {}.", w.people[p as usize].name().unwrap_or("They"), b.say()),
    };
    if !why.is_empty() {
        c.styled(&why, x, by - 14.0, 15.0, eg(SHORT), Face::Italic, 0.0);
    }
    let n = m.n.clamp(1, MOST);
    let cb = Bx::new(x, by, 124.0, 48.0);
    c.rect_lines(cb.x, cb.y, cb.w, cb.h, 1.0, ega(BRASS, 0.45));
    let less = Bx::new(cb.x, cb.y, 40.0, 48.0);
    let more = Bx::new(cb.x + 84.0, cb.y, 40.0, 48.0);
    for (bx, sign) in [(&less, "−"), (&more, "+")] {
        let l = kit::row(c, bx, false);
        c.styled(sign, bx.x + 13.0, bx.y + 33.0 + kit::nudge(l), 24.0, eg(if l.hot { BRASS_LIGHT } else { BRASS }), Face::Body, 0.0);
    }
    let ns = n.to_string();
    c.styled(&ns, cb.x + 62.0 - c.styled_width(&ns, 22.0, Face::Body, 0.0) / 2.0, cb.y + 32.0, 22.0, eg(BRIGHT), Face::Body, 0.0);
    if clicked(&less) {
        act = Some(MakeAct::Count(n.saturating_sub(1).max(1)));
    }
    if clicked(&more) {
        act = Some(MakeAct::Count((n + 1).min(MOST)));
    }
    let ready = who.filter(|_| blockers.is_empty());
    let mb = Bx::new(x + 140.0, by, right - x - 140.0, 48.0);
    kit::button(c, &mb, "BREW", kit::Kind::Primary, ready.is_some());
    if let (Some(p), true) = (ready, clicked(&mb)) {
        act = Some(MakeAct::Brew(p, m.mix.clone(), n));
    }
    let foot = "Click an ingredient to put it in · Enter brews · Esc closes";
    c.styled(foot, r.x + r.w / 2.0 - c.styled_width(foot, 15.0, Face::Body, 0.0) / 2.0, r.y + r.h + 22.0, 15.0, ega(FAINT, 0.85), Face::Body, 0.0);
    (act, r, scroll)
}
