//! The loot panel: shown while a squad member stands over a beaten foe
//! they were sent to go through. Take a thing, or take everything.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{items::item, loot::LootRef, person::PersonId, World};

use super::hud::{Canvas, PANEL};
use super::palette::{eg, ega, DIM, GOLD, TEXT};
use super::squadui::{Bx, Click};

pub enum LootAct {
    Take(PersonId, PersonId, LootRef),
    TakeAll(PersonId, PersonId),
    Close(PersonId),
}

const W: f32 = 380.0;
const ROW: f32 = 19.0;

/// The panel for the first squad member going through a body, if any.
pub fn loot_panel(c: &Canvas, w: &World, mouse: Vec2, click: Option<Click>) -> (Option<LootAct>, Option<Bx>) {
    let Some((who, body)) = w.squad.members.iter().find_map(|&m| w.looting_now(m).map(|b| (m, b))) else { return (None, None) };
    let things = w.loot_of(body);
    let rows = things.len().clamp(1, 22) + 4;
    let r = Bx::new(c.w * 0.5 - W * 0.5, 120.0, W, rows as f32 * ROW + 40.0);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    let clicked = |b: &Bx| click.map(|k| b.contains(k.at) && !k.right).unwrap_or(false);
    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    let mut act = None;
    let name = w.people[body as usize].name().unwrap_or("them");
    let state = if w.people[body as usize].dead { "dead" } else { "out cold" };
    c.text(&format!("{name} ({state})"), x, y, 17.0, GOLD);
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(&close) {
        act = Some(LootAct::Close(who));
    }
    y += ROW;
    c.text(&format!("{} goes through their things.", w.people[who as usize].name().unwrap_or("?")), x, y, 13.0, DIM);
    if things.is_empty() {
        y += ROW;
        c.text("Nothing left on them.", x + 8.0, y, 14.0, DIM);
    }
    for (what, it, n) in things.iter().take(22) {
        y += ROW;
        let row = Bx::new(r.x + 6.0, y - 15.0, r.w - 12.0, ROW);
        if row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.12));
        }
        let worn = matches!(what, LootRef::Worn(_));
        let label = if *n > 1 { format!("{} × {}", n, item(*it).name) } else { item(*it).name.to_string() };
        c.text(&label, x + 8.0, y, 14.0, TEXT);
        c.text(&format!("{}{:.0} coin", if worn { "worn  ·  " } else { "" }, item(*it).value * *n as f32), r.x + r.w - 130.0, y, 12.0, DIM);
        if clicked(&row) {
            act = Some(LootAct::Take(who, body, *what));
        }
    }
    y += ROW + 6.0;
    let all = Bx::new(x, y - 15.0, 110.0, 20.0);
    c.rect(all.x, all.y, all.w, all.h, ega(GOLD, if all.contains(mouse) { 0.3 } else { 0.15 }));
    c.text("Take all", all.x + 22.0, all.y + 15.0, 14.0, GOLD);
    if clicked(&all) && !things.is_empty() {
        act = Some(LootAct::TakeAll(who, body));
    }
    (act, Some(r))
}
