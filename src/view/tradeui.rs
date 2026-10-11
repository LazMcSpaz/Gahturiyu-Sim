//! Trade, across a table (the feel plan's 5.2; mockup 2). The squad's side
//! on the left, the merchant's on the right, and between them the table:
//! what's laid on it to give and to take, what it comes to, and Deal.
//!
//! Left: who of the squad is standing with the talker (whose pack is
//! showing; the rest of the squad named as too far), that member's pack
//! with what the merchant pays for each thing, their load and the purse.
//! Right: the merchant, how full their purse looks (a word, not a number),
//! and the stall with what they ask. A price is marked against what the
//! thing usually fetches: good for the squad in green, poor in red.
//!
//! Click a line to lay one on the table (again for another); click a line
//! on the table to take one back; click twice quickly to trade that one
//! thing at once. Deal carries the whole table out, or says why it can't.
//!
//! Everything shown comes from the world: prices from `offer` and
//! `for_sale`, the sums from `deal_quote`, which works the table through
//! thing by thing as `deal_table` will, so what the table says is what changes
//! hands. This file draws, and reports what was clicked.

use bevy::math::Vec2;

use gahturiyu_sim::names::Gender;
use gahturiyu_sim::sim::{
    economy::{Deal, Table},
    items::{self, item, ItemId, Kind, SLOTS},
    names::gender,
    person::PersonId,
    World,
};

use super::hud::{Canvas, Face};
use super::palette::{eg, ega, race_color, Rgb, BRASS, BRASS_LIGHT, TEXT};
use super::squadui::{Bx, Click};

/// The trade screen while it's open.
#[derive(Clone, Debug, Default)]
pub struct Trading {
    pub table: Table,
    /// How the table would go (worked out when it changes).
    pub quote: Option<Deal>,
    /// Whose pack is showing (none chosen: the talker's).
    pub member: Option<PersonId>,
    /// How far each list is scrolled (the pack, the stall), pixels.
    pub scroll: [f32; 2],
}

pub enum TradeAct {
    Close,
    Member(PersonId),
    /// One of this member's things onto the table.
    Give(PersonId, ItemId),
    /// One of the merchant's things onto the table.
    Get(ItemId),
    /// One back off the table: (from the giving side?, which line).
    Back(bool, usize),
    Clear,
    Deal,
}

const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const GROUP: Rgb = [0.66, 0.57, 0.42];
const FAINT: Rgb = [0.55, 0.50, 0.42];
const POOR: Rgb = [0.85, 0.47, 0.37];
const GOOD: Rgb = [0.61, 0.76, 0.48];
const EVEN: Rgb = [0.85, 0.80, 0.69];
const BRIGHT: Rgb = [1.0, 0.95, 0.85];
const PANEL_TOP: Rgb = [0.165, 0.125, 0.094];
const PANEL_BOT: Rgb = [0.11, 0.082, 0.063];
const WOOD_TOP: Rgb = [0.29, 0.20, 0.13];
const WOOD_BOT: Rgb = [0.20, 0.14, 0.10];
const DARK: Rgb = [0.047, 0.035, 0.024];

const SIDE_W: f32 = 500.0;
const TABLE_W: f32 = 400.0;
const GAP: f32 = 40.0;
const ROW: f32 = 32.0;

/// A price is marked when the going rate is this far from the usual.
const DEAR: f32 = 1.3;
const CHEAP: f32 = 0.85;

/// A small arrowhead left of a price: up for dear here, down for cheap.
fn arrow(c: &Canvas, x: f32, y: f32, up: bool, col: bevy_egui::egui::Color32) {
    let (a, b) = if up { (y - 4.0, y - 13.0) } else { (y - 13.0, y - 4.0) };
    c.triangle(Vec2::new(x - 5.5, a), Vec2::new(x + 5.5, a), Vec2::new(x, b), col);
}

/// "She", "Her" and so on for someone.
fn they(seed: u64) -> (&'static str, &'static str, &'static str) {
    match gender(seed) {
        Gender::Female => ("She", "Her", "her"),
        Gender::Male => ("He", "His", "him"),
        Gender::Either => ("They", "Their", "them"),
    }
}

/// The lists' real scroll positions, after drawing.
pub type Scrolls = [f32; 2];

/// Draw the screen for the talk that is open; what was clicked, the box it
/// covers (all of the window: nothing behind it takes a click), and where
/// its two lists are really scrolled to.
pub fn trade(c: &Canvas, w: &World, t: &Trading, mouse: Vec2, click: Option<Click>) -> (Option<TradeAct>, Bx, Scrolls) {
    let all = Bx::new(0.0, 0.0, c.w, c.h);
    let Some(cv) = w.talk.as_ref() else { return (None, all, t.scroll) };
    let npc = cv.npc;
    let them = &w.people[npc as usize];
    let (she, her_cap, her) = they(them.seed);
    let clicked = |bx: &Bx| click.map(|k| bx.contains(k.at) && !k.right).unwrap_or(false);
    let mut act = None;
    let mut scrolls = t.scroll;

    // The world steps back.
    c.rect(0.0, 0.0, c.w, c.h, ega(DARK, 0.94));
    c.styled_centred("TRADE", c.w / 2.0, 56.0, 26.0, eg(HEAD), Face::Title, 10.0);
    c.rule(c.w / 2.0 - 100.0, c.w / 2.0 + 100.0, 72.0, 1.0, eg(BRASS));

    let total = SIDE_W * 2.0 + TABLE_W + GAP * 2.0;
    let x0 = ((c.w - total) / 2.0).max(10.0);
    let top = 110.0;
    let side_h = (c.h - top - 100.0).clamp(420.0, 790.0);
    let side = |x: f32| {
        c.grad(x, top, SIDE_W, side_h, eg(PANEL_TOP), eg(PANEL_BOT), false);
        c.rect(x, top, SIDE_W, 2.0, eg(BRASS));
        c.rect(x, top + side_h - 2.0, SIDE_W, 2.0, eg(BRASS));
    };

    // ---- The squad's side --------------------------------------------------------
    let lx = x0;
    side(lx);
    let hand = w.at_hand();
    let member = t.member.filter(|m| hand.contains(m)).unwrap_or(cv.with);
    let mut px = lx + 18.0;
    for &m in w.squad.members.iter().take(8) {
        let p = &w.people[m as usize];
        let near = hand.contains(&m);
        let on = m == member;
        let rad = if on { 27.0 } else { 22.0 };
        let cx = px + 27.0;
        let cy = top + 46.0;
        let fade = if near { 1.0 } else { 0.38 };
        c.circle(cx, cy, rad - 2.0, ega(race_color(p.race), 0.85 * fade));
        c.circle_lines(cx, cy, rad, if on { 2.0 } else { 1.0 }, if on { eg(HEAD) } else { ega(BRASS, 0.5 * fade) });
        let name = p.name().unwrap_or("?");
        let short = name.split(' ').next().unwrap_or(name);
        let nw = c.styled_width(short, 14.0, Face::Caps, 0.0);
        c.styled(short, cx - nw / 2.0, top + 92.0, 14.0, ega(if on { BRASS_LIGHT } else { EVEN }, fade.max(0.55)), Face::Caps, 0.0);
        if !near {
            let fw = c.styled_width("too far", 12.0, Face::Italic, 0.0);
            c.styled("too far", cx - fw / 2.0, top + 106.0, 12.0, ega(EVEN, 0.5), Face::Italic, 0.0);
        }
        let bx = Bx::new(cx - 30.0, top + 14.0, 60.0, 84.0);
        if near && !on && clicked(&bx) {
            act = Some(TradeAct::Member(m));
        }
        px += 76.0;
    }
    let me = &w.people[member as usize];
    let my_name = me.name().unwrap_or("?").split(' ').next().unwrap_or("?").to_uppercase();
    let hy = top + 136.0;
    c.styled(&format!("{my_name}'S PACK"), lx + 18.0, hy, 13.0, eg(GROUP), Face::Title, 4.0);
    c.styled_right(&format!("{she} pays"), lx + SIDE_W - 18.0, hy, 15.0, eg(FAINT), Face::Body, 0.0);
    c.rect(lx + 18.0, hy + 8.0, SIDE_W - 36.0, 1.0, ega(BRASS, 0.35));

    // The pack: what can be sold first, then what they won't take, then what's worn.
    let list_top = hy + 14.0;
    let list_h = side_h - (list_top - top) - 96.0;
    let mut rows: Vec<(ItemId, u16, Option<u16>, bool)> = Vec::new();
    if let Some(d) = me.detail.as_ref() {
        for e in &d.gear.bag {
            if matches!(item(e.0).kind, Kind::Coin) {
                continue;
            }
            let price = w.offer(npc, e.0, e.2.as_ref());
            match rows.iter_mut().find(|r| r.0 == e.0 && r.2 == price && !r.3) {
                Some(r) => r.1 += e.1,
                None => rows.push((e.0, e.1, price, false)),
            }
        }
        rows.sort_by_key(|r| r.2.is_none());
        for s in SLOTS {
            if let Some(it) = d.gear.in_slot(s) {
                rows.push((it, 1, w.offer(npc, it, d.gear.piece(s)), true));
            }
        }
    }
    let most = (rows.len() as f32 * ROW - list_h).max(0.0);
    scrolls[0] = t.scroll[0].clamp(0.0, most);
    for (k, &(it, n, price, worn)) in rows.iter().enumerate() {
        let yy = list_top + k as f32 * ROW - scrolls[0];
        if yy < list_top - 1.0 || yy + ROW > list_top + list_h + 1.0 {
            continue;
        }
        let bx = Bx::new(lx + 6.0, yy, SIDE_W - 12.0, ROW);
        let laid: u16 = t.table.give.iter().filter(|g| g.0 == member && g.1 == it).map(|g| g.2).sum();
        let sells = price.is_some() && !worn;
        let hot = sells && bx.contains(mouse);
        if hot {
            c.grad(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.22), ega(BRASS, 0.04), true);
        }
        c.rect(bx.x + 6.0, bx.y + bx.h - 1.0, bx.w - 12.0, 1.0, ega(BRASS, 0.08));
        let fade = if sells { 1.0 } else { 0.5 };
        let label = if n > 1 { format!("{} ×{n}", item(it).name) } else { item(it).name.to_string() };
        let lw = c.styled(&label, bx.x + 12.0, yy + 22.0, 19.0, ega(if hot { BRIGHT } else { TEXT }, fade), Face::Body, 0.0);
        if worn {
            c.styled("worn", bx.x + 20.0 + lw, yy + 22.0, 15.0, ega(FAINT, 0.9), Face::Italic, 0.0);
        } else if laid > 0 {
            c.styled(&format!("{laid} on the table"), bx.x + 20.0 + lw, yy + 22.0, 15.0, eg(HEAD), Face::Italic, 0.0);
        }
        c.styled_right(&format!("{:.1} kg", item(it).weight * n as f32), bx.x + bx.w - 100.0, yy + 22.0, 15.0, ega(FAINT, fade.max(0.7)), Face::Body, 0.0);
        let rate = w.going_rate(npc, it);
        let each = if n > 1 { " each" } else { "" };
        let (text, col, mark) = match price {
            None => ("—".to_string(), ega(FAINT, 0.7), None),
            Some(p) if worn => (format!("~{p}"), ega(FAINT, 0.9), None),
            // They pay well for what's dear here: good for the squad.
            Some(p) if rate >= DEAR => (format!("{p}{each}"), eg(GOOD), Some(true)),
            Some(p) if rate <= CHEAP => (format!("{p}{each}"), eg(POOR), Some(false)),
            Some(p) => (format!("{p}{each}"), eg(EVEN), None),
        };
        let pw = c.styled_right(&text, bx.x + bx.w - 12.0, yy + 22.0, 18.0, col, Face::Body, 0.0);
        if let Some(up) = mark {
            arrow(c, bx.x + bx.w - 24.0 - pw, yy + 22.0, up, col);
        }
        if sells && laid < n && clicked(&bx) {
            act = Some(TradeAct::Give(member, it));
        }
    }
    if rows.is_empty() {
        c.styled("Nothing in this pack.", lx + 22.0, list_top + 30.0, 16.0, eg(FAINT), Face::Italic, 0.0);
    }
    // Load and purse.
    let fy = top + side_h - 84.0;
    c.rect(lx + 18.0, fy, SIDE_W - 36.0, 1.0, ega(BRASS, 0.25));
    let (kit, cap) = (w.kit_weight_at(member, w.time), w.capacity_at(member, w.time));
    c.styled("Load", lx + 18.0, fy + 24.0, 15.0, eg(SUB), Face::Body, 0.0);
    c.styled_right(&format!("{kit:.1} / {cap:.0} kg"), lx + SIDE_W - 18.0, fy + 24.0, 15.0, eg(SUB), Face::Body, 0.0);
    c.rect(lx + 18.0, fy + 32.0, SIDE_W - 36.0, 6.0, ega(DARK, 0.9));
    let share = (kit / cap.max(1.0)).clamp(0.0, 1.0);
    c.rect(lx + 18.0, fy + 32.0, (SIDE_W - 36.0) * share, 6.0, eg(if share > 0.95 { POOR } else if share > 0.75 { HEAD } else { [0.66, 0.60, 0.42] }));
    c.styled("Purse", lx + 18.0, fy + 64.0, 17.0, eg(SUB), Face::Body, 0.0);
    c.styled_right(&format!("{} coin", w.squad_count(items::id("coin"))), lx + SIDE_W - 18.0, fy + 64.0, 17.0, eg(BRASS_LIGHT), Face::Body, 0.0);

    // ---- The merchant's side -------------------------------------------------------
    let rx = x0 + SIDE_W + GAP + TABLE_W + GAP;
    side(rx);
    c.circle(rx + 53.0, top + 53.0, 33.0, ega(race_color(them.race), 0.85));
    c.circle_lines(rx + 53.0, top + 53.0, 35.0, 2.0, eg(BRASS));
    let their_name = them.name().unwrap_or("?");
    c.styled(their_name.split(' ').next().unwrap_or(their_name), rx + 104.0, top + 50.0, 26.0, eg(BRASS_LIGHT), Face::Caps, 0.0);
    let job = w.life(npc).job.title(them.seed).to_lowercase();
    c.styled(&format!("{} · {job}", them.race.name()), rx + 104.0, top + 74.0, 16.0, eg(SUB), Face::Italic, 0.0);
    let purse = w.purse_words(npc);
    let pw = c.styled_width(purse, 13.0, Face::Body, 0.0);
    // (A pouch: fuller the heavier the purse.)
    let full = match purse {
        "heavy purse" => 1.0,
        "fair purse" => 0.7,
        "light purse" => 0.45,
        _ => 0.25,
    };
    let (bxc, byc) = (rx + SIDE_W - 30.0 - pw / 2.0, top + 44.0);
    c.circle(bxc, byc + 4.0, 9.0 + 7.0 * full, ega([0.35, 0.25, 0.15], 0.95));
    c.circle_lines(bxc, byc + 4.0, 9.0 + 7.0 * full, 1.3, eg(BRASS));
    c.rect(bxc - 5.0, byc - 10.0 - 6.0 * full, 10.0, 5.0, eg(BRASS));
    c.styled(purse, rx + SIDE_W - 30.0 - pw, top + 86.0, 13.0, eg(SUB), Face::Body, 0.0);
    c.styled(&format!("{} STALL", her_cap.to_uppercase()), rx + 18.0, hy, 13.0, eg(GROUP), Face::Title, 4.0);
    c.styled_right(&format!("{she} asks"), rx + SIDE_W - 18.0, hy, 15.0, eg(FAINT), Face::Body, 0.0);
    c.rect(rx + 18.0, hy + 8.0, SIDE_W - 36.0, 1.0, ega(BRASS, 0.35));
    let wares = w.for_sale(npc);
    let stall_h = side_h - (list_top - top) - 16.0;
    let most = (wares.len() as f32 * ROW - stall_h).max(0.0);
    scrolls[1] = t.scroll[1].clamp(0.0, most);
    for (k, &(it, n, price)) in wares.iter().enumerate() {
        let yy = list_top + k as f32 * ROW - scrolls[1];
        if yy < list_top - 1.0 || yy + ROW > list_top + stall_h + 1.0 {
            continue;
        }
        let bx = Bx::new(rx + 6.0, yy, SIDE_W - 12.0, ROW);
        let laid: u16 = t.table.get.iter().filter(|g| g.0 == it).map(|g| g.1).sum();
        let hot = bx.contains(mouse);
        if hot {
            c.grad(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.22), ega(BRASS, 0.04), true);
        }
        c.rect(bx.x + 6.0, bx.y + bx.h - 1.0, bx.w - 12.0, 1.0, ega(BRASS, 0.08));
        let lw = c.styled(item(it).name, bx.x + 12.0, yy + 22.0, 19.0, eg(if hot { BRIGHT } else { TEXT }), Face::Body, 0.0);
        let cw = c.styled(&format!("×{n}"), bx.x + 20.0 + lw, yy + 22.0, 15.0, eg(FAINT), Face::Body, 0.0);
        if laid > 0 {
            c.styled(&format!("{laid} on the table"), bx.x + 30.0 + lw + cw, yy + 22.0, 15.0, eg(HEAD), Face::Italic, 0.0);
        }
        c.styled_right(&format!("{:.1} kg", item(it).weight), bx.x + bx.w - 100.0, yy + 22.0, 15.0, eg(FAINT), Face::Body, 0.0);
        let rate = w.going_rate(npc, it);
        // Cheap here is good for a buyer; dear is poor.
        let (col, mark) = if rate <= CHEAP { (eg(GOOD), Some(false)) } else if rate >= DEAR { (eg(POOR), Some(true)) } else { (eg(EVEN), None) };
        let pw = c.styled_right(&price.to_string(), bx.x + bx.w - 12.0, yy + 22.0, 18.0, col, Face::Body, 0.0);
        if let Some(up) = mark {
            arrow(c, bx.x + bx.w - 24.0 - pw, yy + 22.0, up, col);
        }
        if laid < n && clicked(&bx) {
            act = Some(TradeAct::Get(it));
        }
    }
    if wares.is_empty() {
        c.styled("The shelves are bare.", rx + 22.0, list_top + 30.0, 16.0, eg(FAINT), Face::Italic, 0.0);
    }

    // ---- The table ----------------------------------------------------------------
    let tx = x0 + SIDE_W + GAP;
    let ty = top + 40.0;
    let th = side_h - 90.0;
    c.grad(tx, ty, TABLE_W, th, eg(WOOD_TOP), eg(WOOD_BOT), false);
    c.rect_lines(tx, ty, TABLE_W, th, 1.0, ega(DARK, 0.8));
    let mut gy = ty + 47.0;
    while gy < ty + th - 4.0 {
        c.rect(tx + 1.0, gy, TABLE_W - 2.0, 1.5, ega(DARK, 0.20));
        gy += 48.0;
    }
    c.styled_centred("THE TABLE", tx + TABLE_W / 2.0, ty + 32.0, 13.0, eg([0.85, 0.75, 0.52]), Face::Title, 4.0);
    let quote = t.quote.as_ref();
    let coin_of_give = |k: usize| quote.and_then(|q| q.gives.get(k)).map(|g| g.3);
    let coin_of_get = |k: usize| quote.and_then(|q| q.gets.get(k)).map(|g| g.2);
    let mut y = ty + 52.0;
    let bxw = TABLE_W - 40.0;
    let pile = |title: &str, sum: Option<u32>, lines: Vec<(String, Option<u32>)>, giving: bool, y: &mut f32, act: &mut Option<TradeAct>| {
        let h = 44.0 + lines.len().max(1) as f32 * 28.0;
        c.rect(tx + 20.0, *y, bxw, h, ega(DARK, 0.45));
        c.rect_lines(tx + 20.0, *y, bxw, h, 1.0, ega(BRASS, 0.3));
        c.styled(title, tx + 34.0, *y + 26.0, 17.0, eg(BRASS), Face::Caps, 0.0);
        if let Some(s) = sum {
            c.styled_right(&format!("{s} coin"), tx + 20.0 + bxw - 14.0, *y + 26.0, 17.0, eg(BRASS_LIGHT), Face::Body, 0.0);
        }
        if lines.is_empty() {
            c.styled("nothing yet", tx + 34.0, *y + 54.0, 16.0, ega(FAINT, 0.8), Face::Italic, 0.0);
        }
        for (k, (label, coin)) in lines.iter().enumerate() {
            let ly = *y + 54.0 + k as f32 * 28.0;
            let bx = Bx::new(tx + 24.0, ly - 20.0, bxw - 8.0, 27.0);
            if bx.contains(mouse) {
                c.rect(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.14));
            }
            c.styled(label, tx + 34.0, ly, 18.0, eg(TEXT), Face::Body, 0.0);
            if let Some(v) = coin {
                c.styled_right(&v.to_string(), tx + 20.0 + bxw - 14.0, ly, 18.0, eg(EVEN), Face::Body, 0.0);
            }
            if clicked(&bx) {
                *act = Some(TradeAct::Back(giving, k));
            }
        }
        *y += h + 12.0;
    };
    let name_n = |it: ItemId, n: u16| if n > 1 { format!("{} ×{n}", item(it).name) } else { item(it).name.to_string() };
    let gives: Vec<(String, Option<u32>)> = t.table.give.iter().enumerate().map(|(k, g)| (name_n(g.1, g.2), coin_of_give(k))).collect();
    let gets: Vec<(String, Option<u32>)> = t.table.get.iter().enumerate().map(|(k, g)| (name_n(g.0, g.1), coin_of_get(k))).collect();
    pile("You give", quote.map(|q| q.given()), gives, true, &mut y, &mut act);
    pile("You get", quote.map(|q| q.got()), gets, false, &mut y, &mut act);

    // What it comes to.
    if let Some(q) = quote.filter(|_| !t.table.is_empty()) {
        let b = q.balance();
        let line = match b {
            0 => "Even".to_string(),
            b if b > 0 => format!("{she} owes you {b}"),
            b => format!("You owe {her} {}", -b),
        };
        c.styled_centred(&line, tx + TABLE_W / 2.0, (y + 34.0).min(ty + th - 108.0), 26.0, eg(BRASS_LIGHT), Face::Body, 0.0);
    }
    // Clear, and Deal.
    let by = ty + th - 64.0;
    let stuck = quote.and_then(|q| q.stuck);
    let ready = quote.is_some() && stuck.is_none() && !t.table.is_empty();
    if let Some(s) = stuck.filter(|_| !t.table.is_empty()) {
        let why = s.say(she);
        c.styled_centred(&why, tx + TABLE_W / 2.0, by - 14.0, 16.0, eg(POOR), Face::Italic, 0.0);
    }
    let clear = Bx::new(tx + 46.0, by + 2.0, 110.0, 40.0);
    c.rect(clear.x, clear.y, clear.w, clear.h, ega(DARK, if clear.contains(mouse) { 0.8 } else { 0.6 }));
    c.rect_lines(clear.x, clear.y, clear.w, clear.h, 1.0, ega(EVEN, 0.4));
    c.styled("CLEAR", clear.x + clear.w / 2.0 - c.styled_width("CLEAR", 15.0, Face::Title, 3.0) / 2.0, clear.y + 26.0, 15.0, eg(EVEN), Face::Title, 3.0);
    if clicked(&clear) {
        act = Some(TradeAct::Clear);
    }
    let deal = Bx::new(tx + 172.0, by, 182.0, 44.0);
    if ready {
        c.grad(deal.x, deal.y, deal.w, deal.h, eg(if deal.contains(mouse) { BRIGHT } else { BRASS_LIGHT }), eg(BRASS), false);
        c.rect_lines(deal.x, deal.y, deal.w, deal.h, 1.0, eg(BRASS_LIGHT));
        c.styled("DEAL", deal.x + deal.w / 2.0 - c.styled_width("DEAL", 19.0, Face::Title, 5.0) / 2.0, deal.y + 30.0, 19.0, eg(DARK), Face::Title, 5.0);
        if clicked(&deal) {
            act = Some(TradeAct::Deal);
        }
    } else {
        c.rect(deal.x, deal.y, deal.w, deal.h, ega(BRASS, 0.10));
        c.rect_lines(deal.x, deal.y, deal.w, deal.h, 1.0, ega(BRASS, 0.3));
        c.styled("DEAL", deal.x + deal.w / 2.0 - c.styled_width("DEAL", 19.0, Face::Title, 5.0) / 2.0, deal.y + 30.0, 19.0, ega(BRASS, 0.45), Face::Title, 5.0);
    }

    // Leaving, and how it works.
    let close = Bx::new(c.w - 60.0, 26.0, 34.0, 34.0);
    c.styled("×", close.x + 9.0, close.y + 26.0, 28.0, eg(if close.contains(mouse) { BRASS_LIGHT } else { GROUP }), Face::Body, 0.0);
    if clicked(&close) {
        act = Some(TradeAct::Close);
    }
    let fy = top + side_h + 30.0;
    let mut fx = c.w / 2.0 - 440.0;
    arrow(c, fx, fy, true, eg(GOOD));
    arrow(c, fx + 16.0, fy, false, eg(GOOD));
    fx += 30.0;
    fx += c.styled("good for you", fx, fy, 15.0, eg(FAINT), Face::Body, 0.0) + 34.0;
    arrow(c, fx, fy, true, eg(POOR));
    arrow(c, fx + 16.0, fy, false, eg(POOR));
    fx += 30.0;
    fx += c.styled("poor for you", fx, fy, 15.0, eg(FAINT), Face::Body, 0.0) + 34.0;
    c.styled("Click to put on the table · twice to trade at once · Space deal · Esc leave", fx, fy, 15.0, eg(FAINT), Face::Body, 0.0);
    (act, all, scrolls)
}
