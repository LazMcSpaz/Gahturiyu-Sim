//! The transfer window (Kenshi's two inventories side by side): shown while
//! a squad member stands over a beaten foe, or at an open container, they
//! were sent to go through (`loot::Source`). What's there on the left, their
//! own pack on the right; click a line to move it across. Taking from a
//! container that isn't yours is theft if seen: the odds are shown first.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    items::item,
    loot::{LootRef, Source},
    person::PersonId,
    World,
};

use super::hud::{Canvas, Face};
use super::palette::{eg, ega, BRASS, BRASS_LIGHT, DIM, GOLD, TEXT, WARN};
use super::squadui::{Bx, Click};

pub enum LootAct {
    Take(PersonId, Source, LootRef),
    TakeAll(PersonId, Source),
    /// Put pack entry k into the open container.
    Put(PersonId, usize),
    Close(PersonId),
}

const W: f32 = 780.0;
const ROW: f32 = 21.0;
const LINES: usize = 18;

/// The window for the first squad member going through a body or a
/// container, if any.
pub fn loot_panel(c: &Canvas, w: &World, mouse: Vec2, click: Option<Click>) -> (Option<LootAct>, Option<Bx>) {
    let Some((who, src)) = w.squad.members.iter().find_map(|&m| w.source_now(m).map(|s| (m, s))) else { return (None, None) };
    let things = w.contents(src);
    let chest = matches!(src, Source::Chest(_));
    let mine: Vec<(usize, gahturiyu_sim::sim::items::ItemId, u16)> = w.people[who as usize].detail.as_ref().map(|d| d.gear.bag.iter().enumerate().map(|(k, e)| (k, e.0, e.1)).collect()).unwrap_or_default();
    let rows = things.len().max(mine.len()).clamp(3, LINES);
    let r = Bx::new(c.w * 0.5 - W * 0.5, 110.0, W, rows as f32 * ROW + 150.0);
    c.frame_box(r.x, r.y, r.w, r.h);
    let clicked = |b: &Bx| click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false);
    let mut act = None;
    let x0 = r.x + 18.0;
    let x1 = r.x + r.w / 2.0 + 10.0;
    let col_w = r.w / 2.0 - 28.0;
    // The heading: what it is, whose, and the odds.
    let (title, owner) = match src {
        Source::Body(body) => {
            let name = w.people[body as usize].name().unwrap_or("Them");
            (name.to_string(), if w.people[body as usize].dead { "dead".to_string() } else { "out cold".to_string() })
        }
        Source::Chest(id) => match w.container(id) {
            Some(k) => (format!("A {}", k.what.name()), super::interiors::owner_text(w, w.container_owner(id).unwrap_or(k.owner))),
            None => ("A container".into(), String::new()),
        },
    };
    let mut y = r.y + 30.0;
    let tw = c.styled(&title.to_uppercase(), x0, y, 17.0, eg(TEXT), Face::Title, 2.0);
    c.styled(&owner, x0 + tw + 12.0, y, 14.0, eg(DIM), Face::Italic, 0.0);
    let close = Bx::new(r.x + r.w - 30.0, r.y + 10.0, 20.0, 20.0);
    c.styled("×", close.x + 4.0, close.y + 16.0, 20.0, eg(if close.contains(mouse) { GOLD } else { DIM }), Face::Body, 0.0);
    if clicked(&close) {
        act = Some(LootAct::Close(who));
    }
    y += 22.0;
    if let Source::Chest(id) = src {
        let at = w.container(id).map(|k| k.pos).unwrap_or(w.squad.pos);
        let p = w.catch_chance(who, at, id.0);
        let (line, col) = if gahturiyu_sim::sim::containers::is_wild(id) {
            ("Nobody's now: taking is no crime. Anyone keeping the place may hear the lid.".to_string(), [0.55, 0.80, 0.45])
        } else if p <= 0.0 { ("Nobody can see you here: taking is unseen.".to_string(), [0.55, 0.80, 0.45]) } else { (format!("Taking is theft: {:.0}% chance each time that someone sees.", p * 100.0), WARN) };
        c.styled(&line, x0, y, 14.0, eg(col), Face::Italic, 0.0);
        y += 6.0;
    }
    // Column heads.
    y += 22.0;
    let here = if chest { "In it" } else { "On them" };
    c.styled(&here.to_uppercase(), x0, y, 12.0, eg(BRASS_LIGHT), Face::Title, 2.0);
    let load = format!("{:.0}/{:.0} kg", w.kit_weight_at(who, w.time), w.capacity_at(who, w.time));
    let head = format!("{}'S PACK", w.people[who as usize].name().unwrap_or("?").to_uppercase());
    c.styled(&head, x1, y, 12.0, eg(BRASS_LIGHT), Face::Title, 2.0);
    c.styled_right(&load, r.x + r.w - 18.0, y, 13.0, eg(if w.load_of(who) > 1.0 { WARN } else { DIM }), Face::Body, 0.0);
    c.rule(x0 - 6.0, x0 + col_w, y + 7.0, 1.0, ega(BRASS, 0.8));
    c.rule(x1 - 6.0, x1 + col_w, y + 7.0, 1.0, ega(BRASS, 0.8));
    c.line(Vec2::new(r.x + r.w / 2.0, y - 10.0), Vec2::new(r.x + r.w / 2.0, r.y + r.h - 48.0), 1.0, ega(BRASS, 0.4));
    let top = y + 8.0;
    // What's there: click to take.
    if things.is_empty() {
        c.styled(if chest { "It's empty." } else { "Nothing left on them." }, x0 + 4.0, top + ROW, 14.0, eg(DIM), Face::Italic, 0.0);
    }
    for (i, (what, it, n)) in things.iter().take(LINES).enumerate() {
        let yy = top + (i + 1) as f32 * ROW;
        let row = Bx::new(x0 - 6.0, yy - 15.0, col_w + 6.0, ROW);
        if row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.14));
            c.styled_right("take ›", x0 + col_w, yy, 12.0, eg(GOLD), Face::Italic, 0.0);
        } else {
            let worn = if matches!(what, LootRef::Worn(_)) { "worn · " } else { "" };
            c.styled_right(&format!("{worn}{:.0} c", item(*it).value * *n as f32), x0 + col_w, yy, 12.0, eg(DIM), Face::Body, 0.0);
        }
        let label = if *n > 1 { format!("{} × {}", n, item(*it).name) } else { item(*it).name.to_string() };
        c.text(&label, x0 + 4.0, yy, 15.0, TEXT);
        if clicked(&row) {
            act = Some(LootAct::Take(who, src, *what));
        }
    }
    // Their pack: click to put it in (containers only).
    for (i, (k, it, n)) in mine.iter().take(LINES).enumerate() {
        let yy = top + (i + 1) as f32 * ROW;
        let row = Bx::new(x1 - 6.0, yy - 15.0, col_w + 6.0, ROW);
        if chest && row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.14));
            c.styled_right("‹ put in", x1 + col_w, yy, 12.0, eg(GOLD), Face::Italic, 0.0);
        } else {
            c.styled_right(&format!("{:.1} kg", item(*it).weight * *n as f32), x1 + col_w, yy, 12.0, eg(DIM), Face::Body, 0.0);
        }
        let label = if *n > 1 { format!("{} × {}", n, item(*it).name) } else { item(*it).name.to_string() };
        c.text(&label, x1 + 4.0, yy, 15.0, TEXT);
        if chest && clicked(&row) {
            act = Some(LootAct::Put(who, *k));
        }
    }
    // Buttons.
    let by = r.y + r.h - 30.0;
    let all = Bx::new(x0, by - 16.0, 120.0, 24.0);
    c.rect(all.x, all.y, all.w, all.h, ega(BRASS, if all.contains(mouse) { 0.45 } else { 0.22 }));
    c.rect_lines(all.x, all.y, all.w, all.h, 1.0, eg(BRASS));
    c.styled("TAKE ALL", all.x + 22.0, all.y + 17.0, 12.0, eg(BRASS_LIGHT), Face::Title, 2.0);
    if clicked(&all) && !things.is_empty() {
        act = Some(LootAct::TakeAll(who, src));
    }
    let hint = if chest { "Click a line to move it across" } else { "Click a line to take it" };
    c.styled_right(hint, r.x + r.w - 18.0, by, 13.0, eg(DIM), Face::Italic, 0.0);
    (act, Some(r))
}
