//! Making things, at the bench (the feel plan's 5.4; mockup 4, "Making, at
//! the forge"). The screen belongs to a station: it's opened by going to a
//! forge, bench or bed (or from the mortar in a pack), is titled with the
//! station and where it stands, and lists what that station makes for
//! whoever of the squad is there.
//!
//! Left: the things it makes, by craft. What can be made now is bright;
//! what's short of something is dim, with the gap in red; what nobody here
//! is good enough for is dimmer still. Crafts nobody near has taken up
//! aren't listed at all.
//!
//! Right: the chosen thing. Its parts as cards (have / need, from the packs
//! of everyone at the bench), how long it takes, who of the squad would make
//! it (in words, not chances), and a count beside the Make button.
//!
//! Everything shown is read from `crafting::making_list` and `makers`; this
//! file only draws and reports what was clicked.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    crafting::{time_words, Bench, MakeRow, Maker, Standing, RECIPES},
    items::{item, Kind},
    materials::Grade,
    person::PersonId,
    stats::Skill,
    World,
};

use super::hud::{Canvas, Face};
use super::palette::{eg, ega, race_color, Rgb, BRASS, BRASS_LIGHT, TEXT};
use super::squadui::{Bx, Click};

/// The making screen while it's open.
#[derive(Clone, Debug)]
pub struct Making {
    pub bench: Bench,
    /// The recipe chosen (none yet: the first that can be made).
    pub pick: Option<usize>,
    /// Who is to make it (none chosen: the best hand at the bench).
    pub who: Option<PersonId>,
    /// How many.
    pub n: u16,
    /// How far the list is scrolled, pixels.
    pub scroll: f32,
}

impl Making {
    pub fn at(bench: Bench) -> Making {
        Making { bench, pick: None, who: None, n: 1, scroll: 0.0 }
    }
}

pub enum MakeAct {
    Close,
    Pick(usize),
    Who(PersonId),
    Count(u16),
    Make(PersonId, usize, u16),
}

/// The most that can be asked for in one go.
pub const MOST: u16 = 20;

const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const GROUP: Rgb = [0.66, 0.57, 0.42];
const FAINT: Rgb = [0.55, 0.50, 0.42];
const SHORT: Rgb = [0.85, 0.47, 0.37];
const ENOUGH: Rgb = [0.61, 0.76, 0.48];
const BRIGHT: Rgb = [1.0, 0.95, 0.85];
const PANEL_TOP: Rgb = [0.165, 0.125, 0.094];
const PANEL_BOT: Rgb = [0.11, 0.082, 0.063];
const DARK: Rgb = [0.047, 0.035, 0.024];

const W: f32 = 860.0;
const LIST_W: f32 = 330.0;
const ROW_H: f32 = 46.0;
const GROUP_H: f32 = 34.0;

/// What a thing is, in a word or two.
fn kind_words(k: &Kind) -> &'static str {
    match k {
        Kind::Weapon(_) => "a weapon",
        Kind::Armor(_) => "armour",
        Kind::Potion => "a draught",
        Kind::Scroll(_) => "a scroll",
        Kind::Food(_) => "food",
        Kind::Material => "a material",
        Kind::Tool => "a tool",
        Kind::Ammo => "shot",
        Kind::Torch(_) | Kind::StandingTorch(_) => "a light",
        _ => "",
    }
}

/// The recipe the screen shows: the one picked, else the first that can be
/// made, else the first listed.
pub fn shown(rows: &[MakeRow], pick: Option<usize>) -> Option<usize> {
    pick.filter(|p| rows.iter().any(|r| r.recipe == *p)).or_else(|| rows.iter().find(|r| r.standing == Standing::Ready).map(|r| r.recipe)).or_else(|| rows.first().map(|r| r.recipe))
}

/// Who the screen has making it: the one picked if they still can, else the
/// first of the makers who is at the bench and free.
pub fn maker(makers: &[Maker], who: Option<PersonId>) -> Option<PersonId> {
    let able = |m: &&Maker| m.at_bench && !m.busy;
    who.filter(|p| makers.iter().filter(able).any(|m| m.who == *p)).or_else(|| makers.iter().find(able).map(|m| m.who))
}

/// Draw the screen; what was clicked, the box it covers, and how far the
/// list is really scrolled (it can't go past its end).
pub fn making(c: &Canvas, w: &World, m: &Making, mouse: Vec2, click: Option<Click>) -> (Option<MakeAct>, Bx, f32) {
    let b = &m.bench;
    let rows = w.making_list(b);
    let clicked = |bx: &Bx| click.map(|k| bx.contains(k.at) && !k.right).unwrap_or(false);
    let mut act = None;

    let h = (c.h - super::frame::BOTTOM_CLEAR - 70.0).clamp(420.0, 900.0);
    let r = Bx::new(40.0, 50.0, W.min(c.w - 80.0), h);
    // The world goes dark behind it, from the left: all but black under the
    // screen itself (the squad's list is under there), thinning out beyond
    // it. The band along the bottom stays as it is.
    let dark_h = c.h - super::frame::BOTTOM_CLEAR + 8.0;
    let solid = r.x + r.w + 24.0;
    c.rect(0.0, 0.0, solid, dark_h, ega(DARK, 0.985));
    c.grad(solid, 0.0, c.w * 0.14, dark_h, ega(DARK, 0.985), ega(DARK, 0.0), true);

    // The heading: the station, and where it stands.
    let title = if b.carried_by.is_some() { b.place.to_uppercase() } else { format!("THE {}", b.station.name().to_uppercase()) };
    c.styled(&title, r.x + 6.0, r.y + 34.0, 34.0, eg(HEAD), Face::Title, 8.0);
    if b.carried_by.is_none() && !b.place.is_empty() {
        c.styled(&b.place, r.x + 8.0, r.y + 58.0, 17.0, eg(SUB), Face::Italic, 0.0);
    }
    let close = Bx::new(r.x + r.w - 34.0, r.y + 6.0, 30.0, 30.0);
    c.styled("×", close.x + 8.0, close.y + 24.0, 26.0, eg(if close.contains(mouse) { BRASS_LIGHT } else { GROUP }), Face::Body, 0.0);
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

    // ---- The list ---------------------------------------------------------------
    column(r.x, LIST_W);
    let pick = shown(&rows, m.pick);
    if rows.is_empty() {
        c.styled("Nobody here knows this work.", r.x + 14.0, top + 34.0, 17.0, eg(SUB), Face::Italic, 0.0);
    }
    // Lay the rows out first, so the list can be scrolled within its length.
    let mut laid: Vec<(Option<Skill>, Option<&MakeRow>, f32)> = Vec::new();
    let mut y = 6.0;
    let mut last = None;
    for row in &rows {
        let sk = RECIPES[row.recipe].skill;
        if last != Some(sk) {
            laid.push((Some(sk), None, y));
            y += GROUP_H;
            last = Some(sk);
        }
        laid.push((None, Some(row), y));
        y += ROW_H;
    }
    let most = (y + 6.0 - col_h).max(0.0);
    let scroll = m.scroll.clamp(0.0, most);
    for (group, row, at) in laid {
        let yy = top + at - scroll;
        if let Some(sk) = group {
            if yy >= top && yy + GROUP_H <= top + col_h {
                c.styled(&sk.name().to_uppercase(), r.x + 12.0, yy + 24.0, 12.0, eg(GROUP), Face::Title, 4.0);
            }
            continue;
        }
        let Some(row) = row else { continue };
        if yy < top + 2.0 || yy + ROW_H > top + col_h - 2.0 {
            continue;
        }
        let rc = &RECIPES[row.recipe];
        let out = rc.item(Grade::Common);
        let bx = Bx::new(r.x, yy, LIST_W, ROW_H);
        let chosen = pick == Some(row.recipe);
        if chosen {
            c.grad(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.30), ega(BRASS, 0.05), true);
            c.rect(bx.x, bx.y, bx.w, 1.0, ega(BRASS_LIGHT, 0.35));
            c.rect(bx.x, bx.y + bx.h - 1.0, bx.w, 1.0, ega(BRASS_LIGHT, 0.2));
        } else if bx.contains(mouse) {
            c.rect(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.12));
        }
        c.rect(bx.x + 8.0, bx.y + bx.h - 1.0, bx.w - 16.0, 1.0, ega(BRASS, 0.07));
        let fade = match row.standing {
            Standing::Ready => 1.0,
            Standing::Short => 0.6,
            Standing::Beyond => 0.42,
        };
        let name = if rc.makes > 1 { format!("{} ×{}", item(out).name, rc.makes) } else { item(out).name.to_string() };
        c.styled(&name, bx.x + 12.0, bx.y + 21.0, 19.0, ega(if chosen { BRIGHT } else { TEXT }, fade), Face::Body, 0.0);
        let mut x = bx.x + 12.0;
        let sy = bx.y + 39.0;
        match row.standing {
            Standing::Beyond => {
                c.styled("beyond anyone's hand yet", x, sy, 14.0, ega(FAINT, 0.75), Face::Italic, 0.0);
            }
            Standing::Ready => {
                let parts: Vec<String> = row.parts.iter().map(|&(it, _, need)| if need > 1 { format!("{} ×{need}", item(it).name.to_lowercase()) } else { item(it).name.to_lowercase() }).collect();
                c.styled(&parts.join(", "), x, sy, 14.0, eg(if chosen { SUB } else { FAINT }), Face::Body, 0.0);
            }
            Standing::Short => {
                // What's wanting first, in red; then the rest.
                let mut parts: Vec<(gahturiyu_sim::sim::items::ItemId, u16, u16)> = row.parts.clone();
                parts.sort_by_key(|p| p.1 >= p.2);
                let gap_is_empty = parts.iter().all(|p| p.1 >= p.2);
                for (k, &(it, have, need)) in parts.iter().enumerate() {
                    if k > 0 {
                        x += c.styled(" · ", x, sy, 14.0, ega(FAINT, 0.7), Face::Body, 0.0);
                    }
                    let col = if have < need { eg(SHORT) } else { ega(FAINT, 0.8) };
                    x += c.styled(&format!("{} {have}/{need}", item(it).name.to_lowercase()), x, sy, 14.0, col, Face::Body, 0.0);
                    if x > bx.x + bx.w - 40.0 {
                        break;
                    }
                }
                if gap_is_empty {
                    c.styled("nobody free at the bench", x, sy, 14.0, eg(SHORT), Face::Italic, 0.0);
                }
            }
        }
        if clicked(&bx) {
            act = Some(MakeAct::Pick(row.recipe));
        }
    }

    // More above or below than fits: a mark at that end of the list.
    let mx = r.x + LIST_W - 22.0;
    if scroll > 0.5 {
        c.triangle(Vec2::new(mx - 6.0, top + 16.0), Vec2::new(mx + 6.0, top + 16.0), Vec2::new(mx, top + 8.0), ega(BRASS_LIGHT, 0.8));
    }
    if scroll < most - 0.5 {
        let by = top + col_h - 8.0;
        c.triangle(Vec2::new(mx - 6.0, by - 8.0), Vec2::new(mx + 6.0, by - 8.0), Vec2::new(mx, by), ega(BRASS_LIGHT, 0.8));
    }

    // ---- The chosen thing -------------------------------------------------------
    let dx = r.x + LIST_W + 24.0;
    let dw = r.w - LIST_W - 24.0;
    column(dx, dw);
    let Some(ri) = pick else { return (act, r, scroll) };
    let Some(row) = rows.iter().find(|x| x.recipe == ri) else { return (act, r, scroll) };
    let rc = &RECIPES[ri];
    let out = rc.item(Grade::Common);
    let x = dx + 24.0;
    let right = dx + dw - 24.0;
    let mut y = top + 52.0;
    c.styled(item(out).name, x, y, 30.0, eg(BRASS_LIGHT), Face::Title, 0.0);
    y += 24.0;
    let what = kind_words(&item(out).kind);
    let sub = match (rc.makes > 1, what.is_empty()) {
        (true, false) => format!("{what} · makes {}", rc.makes),
        (true, true) => format!("makes {}", rc.makes),
        (false, _) => what.to_string(),
    };
    c.styled(&sub, x, y, 17.0, eg(SUB), Face::Italic, 0.0);

    y += 36.0;
    c.styled("TAKES", x, y, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 12.0;
    let card_w = (right - x - 10.0) / 2.0;
    for (k, &(it, have, need)) in row.parts.iter().enumerate() {
        let cx = x + (k % 2) as f32 * (card_w + 10.0);
        let cy = y + (k / 2) as f32 * 62.0;
        c.rect(cx, cy, card_w, 52.0, ega(DARK, 0.45));
        c.rect_lines(cx, cy, card_w, 52.0, 1.0, ega(BRASS, 0.3));
        c.styled(item(it).name, cx + 14.0, cy + 22.0, 18.0, eg(TEXT), Face::Body, 0.0);
        c.styled(&format!("{have} / {need}"), cx + 14.0, cy + 42.0, 15.0, eg(if have >= need { ENOUGH } else { SHORT }), Face::Body, 0.0);
    }
    y += row.parts.len().div_ceil(2) as f32 * 62.0 + 8.0;
    // Whose packs it comes out of.
    let hands = w.bench_hands(b);
    let givers: Vec<&str> = hands.iter().filter(|&&p| rc.inputs.iter().any(|(k, _)| w.count_of(p, k) > 0)).filter_map(|&p| w.people[p as usize].name()).collect();
    if !givers.is_empty() {
        let from = match givers.len() {
            1 => format!("From {}'s pack", givers[0]),
            _ => format!("From {}'s and {}'s packs", givers[..givers.len() - 1].join("'s, "), givers[givers.len() - 1]),
        };
        c.styled(&from, x, y, 15.0, eg(FAINT), Face::Italic, 0.0);
        y += 14.0;
    }

    y += 8.0;
    c.rect(x, y, right - x, 1.0, ega(BRASS, 0.22));
    y += 26.0;
    let at = if b.carried_by.is_some() { "Time".to_string() } else { format!("Time at the {}", b.station.name().to_lowercase()) };
    c.styled(&at, x, y, 18.0, eg(SUB), Face::Body, 0.0);
    c.styled_right(&time_words(rc.time), right, y, 18.0, eg(TEXT), Face::Body, 0.0);
    y += 12.0;
    c.rect(x, y, right - x, 1.0, ega(BRASS, 0.22));

    // Who makes it.
    y += 30.0;
    c.styled("WHO MAKES IT", x, y, 12.0, eg(GROUP), Face::Title, 4.0);
    y += 10.0;
    let makers = w.makers(b, ri);
    let who = maker(&makers, m.who);
    for mk in makers.iter().take(5) {
        let bx = Bx::new(x, y, right - x, 44.0);
        let able = mk.at_bench && !mk.busy;
        let on = who == Some(mk.who);
        let fade = if able { 1.0 } else { 0.45 };
        c.rect(bx.x, bx.y, bx.w, bx.h, ega(DARK, 0.38));
        c.rect_lines(bx.x, bx.y, bx.w, bx.h, if on { 1.6 } else { 1.0 }, if on { eg(HEAD) } else { ega(BRASS, 0.28 * fade) });
        if able && !on && bx.contains(mouse) {
            c.rect(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.10));
        }
        let p = &w.people[mk.who as usize];
        c.circle(bx.x + 26.0, bx.y + 22.0, 13.0, ega(race_color(p.race), 0.85 * fade));
        c.circle_lines(bx.x + 26.0, bx.y + 22.0, 15.0, if on { 2.0 } else { 1.0 }, if on { eg(HEAD) } else { ega(BRASS, 0.5 * fade) });
        c.styled(p.name().unwrap_or("?"), bx.x + 52.0, bx.y + 28.0, 18.0, ega(if on { BRASS_LIGHT } else { TEXT }, fade), Face::Caps, 0.0);
        let words = if !mk.at_bench {
            "too far".to_string()
        } else if mk.busy {
            match w.making_more.iter().find(|q| q.who == mk.who) {
                Some(q) => format!("at work, {} more to follow", q.left),
                None => "at other work".to_string(),
            }
        } else {
            mk.hand.words().to_string()
        };
        c.styled_right(&words, bx.x + bx.w - 14.0, bx.y + 27.0, 16.0, ega(if on { SUB } else { FAINT }, fade.max(0.7)), Face::Italic, 0.0);
        if able && clicked(&bx) {
            act = Some(MakeAct::Who(mk.who));
        }
        y += 52.0;
    }

    // How many, and Make.
    let by = top + col_h - 74.0;
    let ready = who.and_then(|p| w.can_craft(p, ri).ok().map(|_| p));
    let why = match (who, ready) {
        (_, Some(_)) => String::new(),
        (None, _) if makers.iter().any(|k| k.at_bench) => "Whoever could make it is at other work.".to_string(),
        (None, _) => "Nobody at the bench knows this work.".to_string(),
        (Some(p), None) => {
            let gap: Vec<String> = row.parts.iter().filter(|q| q.1 < q.2).map(|&(it, have, need)| format!("{} more {}", need - have, item(it).name.to_lowercase())).collect();
            if gap.is_empty() { format!("{} can't just now.", w.people[p as usize].name().unwrap_or("They")) } else { format!("Wants {}.", gap.join(" and ")) }
        }
    };
    if !why.is_empty() {
        c.styled(&why, x, by - 14.0, 16.0, eg(SHORT), Face::Italic, 0.0);
    }
    let n = m.n.clamp(1, MOST);
    let cb = Bx::new(x, by, 124.0, 48.0);
    c.rect_lines(cb.x, cb.y, cb.w, cb.h, 1.0, ega(BRASS, 0.45));
    let less = Bx::new(cb.x, cb.y, 40.0, 48.0);
    let more = Bx::new(cb.x + 84.0, cb.y, 40.0, 48.0);
    for (bx, sign) in [(&less, "−"), (&more, "+")] {
        if bx.contains(mouse) {
            c.rect(bx.x, bx.y, bx.w, bx.h, ega(BRASS, 0.18));
        }
        c.styled(sign, bx.x + 13.0, bx.y + 33.0, 24.0, eg(BRASS), Face::Body, 0.0);
    }
    let ns = n.to_string();
    c.styled(&ns, cb.x + 62.0 - c.styled_width(&ns, 22.0, Face::Body, 0.0) / 2.0, cb.y + 32.0, 22.0, eg(BRIGHT), Face::Body, 0.0);
    if clicked(&less) {
        act = Some(MakeAct::Count(n.saturating_sub(1).max(1)));
    }
    if clicked(&more) {
        act = Some(MakeAct::Count((n + 1).min(MOST)));
    }
    let mb = Bx::new(x + 140.0, by, right - x - 140.0, 48.0);
    if ready.is_some() {
        let hot = mb.contains(mouse);
        c.grad(mb.x, mb.y, mb.w, mb.h, eg(if hot { BRIGHT } else { BRASS_LIGHT }), eg(BRASS), false);
        c.rect_lines(mb.x, mb.y, mb.w, mb.h, 1.0, eg(BRASS_LIGHT));
        c.styled("MAKE", mb.x + mb.w / 2.0 - c.styled_width("MAKE", 21.0, Face::Title, 6.0) / 2.0, mb.y + 32.0, 21.0, eg(DARK), Face::Title, 6.0);
    } else {
        c.rect(mb.x, mb.y, mb.w, mb.h, ega(BRASS, 0.10));
        c.rect_lines(mb.x, mb.y, mb.w, mb.h, 1.0, ega(BRASS, 0.3));
        c.styled("MAKE", mb.x + mb.w / 2.0 - c.styled_width("MAKE", 21.0, Face::Title, 6.0) / 2.0, mb.y + 32.0, 21.0, ega(BRASS, 0.45), Face::Title, 6.0);
    }
    if let (Some(p), true) = (ready, clicked(&mb)) {
        act = Some(MakeAct::Make(p, ri, n));
    }
    c.styled("Enter makes · Esc closes", r.x + r.w / 2.0 - c.styled_width("Enter makes · Esc closes", 15.0, Face::Body, 0.0) / 2.0, r.y + r.h + 22.0, 15.0, ega(FAINT, 0.85), Face::Body, 0.0);
    (act, r, scroll)
}
