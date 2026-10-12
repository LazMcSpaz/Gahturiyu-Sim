//! The pack (#170 UI-Pack; the feel plan's 5.3): the whole squad side by
//! side, one column each, so giving between members isn't a chore (Kenshi's
//! known pain). Its own screen, the world dimmed behind it.
//!
//! Each column: who it is; what they wear, beside the little body that shows
//! their wounds; their grids (pockets, then the pack's), each thing in its
//! cells with its icon (Laz's items step 3, Kenshi-style), and anything with
//! no room listed loose in red; and a weight bar at the foot that turns
//! amber, then red. Drag a thing to another cell to move it, or onto another
//! column to give it (the column glows, or goes red if they can't take it
//! now); double-click to wear or use it; right-click for the rest (turn it,
//! give, put down). "Tidy every pack" repacks biggest first. Hovering something wearable says what it would
//! replace and what changes, better in green and worse in red.
//!
//! Drawing only: what's done comes back as the same `squadui::Action`s the
//! old pack panel sent.

use bevy::math::{vec2, Vec2};

use gahturiyu_sim::sim::{
    inventory,
    items::{self, item, ItemId, Kind, Slot, SLOTS},
    person::PersonId,
    World,
};

use super::hud::{Canvas, Face};
use super::kit;
use super::palette::{eg, ega, Rgb, BRASS, BRASS_LIGHT, TEXT};
use super::squadui::{Action, Bx, Click};

const HEAD: Rgb = [0.89, 0.76, 0.48];
const SUB: Rgb = [0.66, 0.60, 0.50];
const FAINT: Rgb = [0.55, 0.50, 0.42];
const GOOD: Rgb = [0.61, 0.76, 0.48];
const BAD: Rgb = [0.85, 0.47, 0.37];
const AMBER: Rgb = [0.88, 0.64, 0.26];
const DARK: Rgb = [0.047, 0.035, 0.024];

const COL_W: f32 = 300.0;
const GAP: f32 = 14.0;
const SLOT_H: f32 = 19.0;
const ROW_H: f32 = 23.0;
/// How far the pointer must move with the button held before a press is a drag, px.
const DRAG_AFTER: f32 = 6.0;
/// Two clicks on one row within this long are a double-click, seconds.
const DOUBLE: f32 = 0.35;

/// The pack screen's own memory between frames (the window's, not the world's).
#[derive(Default)]
pub struct PackState {
    /// The first column shown, when the squad is wider than the screen.
    pub first: usize,
    /// A thing pressed on: whose, which pack entry, where the press began.
    pub press: Option<(PersonId, usize, Vec2)>,
    /// It's being dragged.
    pub dragging: bool,
    /// When the press was on a stack in a grid: which (of `arrange().0`),
    /// the cell within it that was grabbed, and whether it lies turned.
    pub grab: Option<(usize, u8, u8, bool)>,
    /// The last row clicked and when (for double-clicks).
    pub last: Option<(PersonId, usize, bool, std::time::Instant)>,
    /// An open right-click menu: whose, a pack entry or a worn slot, where.
    pub menu: Option<(PersonId, Pick, Vec2)>,
    /// A column that was just given something, and when (it flashes).
    pub flash: Option<(PersonId, std::time::Instant)>,
    /// Wheel turned this frame (lines), for the column under the mouse.
    pub wheel: f32,
}

/// A thing in a column: a pack entry, or what's in a slot.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Pick {
    Entry(usize),
    Worn(Slot),
}

/// What wearing `new` instead of what's in its slot would change, as
/// lines: better in green, worse in red. Empty if it isn't worn.
pub fn compare(w: &World, pid: PersonId, new: ItemId) -> Vec<(String, Rgb)> {
    let d = item(new);
    if !items::equippable(new) {
        return Vec::new();
    }
    let old = w.people[pid as usize].detail.as_ref().and_then(|x| x.gear.in_slot(d.slot));
    let mut out = vec![(
        match old {
            Some(o) => format!("Instead of the {}", item(o).name.to_lowercase()),
            None => format!("{}: nothing there now", d.slot.name()),
        },
        SUB,
    )];
    let line = |out: &mut Vec<(String, Rgb)>, what: &str, a: f32, b: f32, pct: bool| {
        if (a - b).abs() < 0.005 {
            return;
        }
        let up = b > a;
        let (a, b) = if pct { (a * 100.0, b * 100.0) } else { (a, b) };
        out.push((format!("{what} {a:.0} to {b:.0}"), if up { GOOD } else { BAD }));
    };
    match (&d.kind, old.map(|o| &item(o).kind)) {
        (Kind::Weapon(n), Some(Kind::Weapon(o))) => {
            line(&mut out, "Cut", o.cut, n.cut, false);
            line(&mut out, "Blunt", o.blunt, n.blunt, false);
            if n.range > 0.0 || o.range > 0.0 {
                line(&mut out, "Range", o.range, n.range, false);
            } else {
                line(&mut out, "Reach", o.reach * 10.0, n.reach * 10.0, false);
            }
        }
        (Kind::Weapon(n), _) => {
            line(&mut out, "Cut", 0.0, n.cut, false);
            line(&mut out, "Blunt", 0.0, n.blunt, false);
        }
        (Kind::Armor(n), Some(Kind::Armor(o))) => {
            line(&mut out, "Covers", o.coverage, n.coverage, true);
            line(&mut out, "Stops cut", o.cut, n.cut, true);
            line(&mut out, "Stops blunt", o.blunt, n.blunt, true);
        }
        (Kind::Armor(n), _) => {
            line(&mut out, "Covers", 0.0, n.coverage, true);
            line(&mut out, "Stops cut", 0.0, n.cut, true);
        }
        (Kind::Shield(n), Some(Kind::Shield(o))) => line(&mut out, "Blocks", *o, *n, true),
        (Kind::Pack(n), Some(Kind::Pack(o))) => line(&mut out, "Holds", *o, *n, false),
        (Kind::Pack(n), _) => line(&mut out, "Holds", 0.0, *n, false),
        _ => {}
    }
    let (ow, nw) = (old.map(|o| item(o).weight).unwrap_or(0.0), d.weight);
    if (ow - nw).abs() > 0.05 {
        // Lighter is better.
        out.push((format!("Weight {ow:.1} to {nw:.1} kg"), if nw < ow { GOOD } else { BAD }));
    }
    out
}

/// Can `to` be given something by `from` just now? Nothing, or why not.
fn why_not_give(w: &World, from: PersonId, to: PersonId) -> Option<&'static str> {
    if from == to {
        return Some("");
    }
    if w.fighter(from).is_some() || w.fighter(to).is_some() {
        return Some("Not while fighting");
    }
    if !w.can_act(from) {
        return Some("They can't hand it over now");
    }
    None
}

/// The screen: `focus` is the member it was opened for (their column comes
/// first in view). `left_held`: the left button is down. Returns what was
/// done and the box it covers.
pub fn pack(c: &Canvas, w: &World, st: &mut PackState, focus: PersonId, mouse: Vec2, left_held: bool, click: Option<Click>) -> (Vec<Action>, Bx) {
    let members: Vec<PersonId> = w.squad.members.clone();
    let mut acts = Vec::new();
    let now = std::time::Instant::now();
    let bottom = c.h - super::frame::BOTTOM_CLEAR - 10.0;
    let top = 54.0;
    let fit = (((c.w - 80.0 + GAP) / (COL_W + GAP)).floor() as usize).max(1);
    // Keep the member it was opened for in view.
    if let Some(i) = members.iter().position(|&m| m == focus) {
        if i < st.first || i >= st.first + fit {
            st.first = i.saturating_sub(fit - 1).min(i);
        }
    }
    st.first = st.first.min(members.len().saturating_sub(fit));
    let shown: Vec<PersonId> = members.iter().copied().skip(st.first).take(fit).collect();
    let total_w = shown.len() as f32 * (COL_W + GAP) - GAP;
    let x0 = ((c.w - total_w) / 2.0).max(40.0);

    // The world dims behind it; the band along the bottom stays as it is.
    let dark_h = c.h - super::frame::BOTTOM_CLEAR + 8.0;
    c.rect(0.0, 0.0, c.w, dark_h, ega(DARK, 0.95));
    let r = Bx::new(x0 - 20.0, top - 44.0, total_w + 40.0, bottom - top + 50.0);
    c.styled("PACKS", x0, top - 14.0, 22.0, eg(HEAD), Face::Title, 6.0);
    // Tidy: everyone's grids repacked, biggest things first.
    {
        let sx = x0 + 150.0;
        let label = "tidy every pack";
        let wd = c.styled_width(label, 13.0, Face::Body, 0.0);
        let b = Bx::new(sx - 4.0, top - 31.0, wd + 8.0, 20.0);
        let l = kit::look(&b);
        c.styled(label, sx, top - 16.0 + kit::nudge(l), 13.0, eg(if l.hot { BRASS_LIGHT } else { SUB }), Face::Body, 0.0);
        if click.is_some_and(|k| !k.right && b.contains(k.at)) {
            for &m in &shown {
                acts.push(Action::Tidy(m));
            }
        }
    }
    // Close.
    let close = Bx::new(r.x + r.w - 34.0, r.y + 6.0, 28.0, 28.0);
    let cl = kit::look(&close);
    c.styled("×", close.x + 8.0, close.y + 22.0 + kit::nudge(cl), 24.0, eg(if cl.hot { BRASS_LIGHT } else { FAINT }), Face::Body, 0.0);
    if click.is_some_and(|k| !k.right && close.contains(k.at)) {
        acts.push(Action::CloseInventory);
    }
    // Paging, when the squad is wider than the screen.
    if members.len() > fit {
        for (dir, label, x) in [(-1i32, "‹", x0 - 34.0), (1, "›", x0 + total_w + 12.0)] {
            let b = Bx::new(x, top + 120.0, 24.0, 40.0);
            let can = if dir < 0 { st.first > 0 } else { st.first + fit < members.len() };
            let l = kit::look(&b);
            c.styled(label, b.x + 6.0, b.y + 28.0, 28.0, eg(if !can { [0.3, 0.27, 0.22] } else if l.hot { BRASS_LIGHT } else { SUB }), Face::Body, 0.0);
            if can && click.is_some_and(|k| !k.right && b.contains(k.at)) {
                st.first = (st.first as i32 + dir).max(0) as usize;
            }
        }
    }

    // The drag: pressed on a pack row, moved far enough with the button down.
    if !left_held && click.is_none() {
        st.press = None;
        st.dragging = false;
        st.grab = None;
    }
    if let Some((_, _, from)) = st.press {
        if left_held && (mouse - from).length() > DRAG_AFTER {
            st.dragging = true;
        }
    }
    let dragged = st.press.filter(|_| st.dragging);

    let mut tip: Option<(ItemId, Option<PersonId>)> = None;
    // A brewed bottle under the mouse: its own mix.
    let mut tip_mix: Option<gahturiyu_sim::sim::items::herbs::Mix> = None;
    let mut tip_sheet: Option<gahturiyu_sim::sim::brewing::Sheet> = None;
    let mut columns = Vec::new();
    // Every grid drawn: whose, which, where, and its cell size.
    let mut grid_boxes: Vec<(PersonId, u8, Bx, f32)> = Vec::new();
    for (ci, &pid) in shown.iter().enumerate() {
        let cx = x0 + ci as f32 * (COL_W + GAP);
        let col = Bx::new(cx, top, COL_W, bottom - top);
        columns.push((pid, col));
        let p = &w.people[pid as usize];
        let Some(d) = p.detail.as_ref() else { continue };
        let gear = &d.gear;
        // The column's ground; it glows as somewhere to drop, red if not now.
        c.frame_box(col.x, col.y, col.w, col.h);
        if let Some((from, _, _)) = dragged {
            if from != pid && col.contains(mouse) {
                let bad = why_not_give(w, from, pid).is_some();
                c.rect(col.x, col.y, col.w, col.h, ega(if bad { BAD } else { BRASS_LIGHT }, 0.10));
                c.rect_lines(col.x - 2.0, col.y - 2.0, col.w + 4.0, col.h + 4.0, 1.6, ega(if bad { BAD } else { BRASS_LIGHT }, 0.85));
            }
        }
        if let Some((who, t)) = st.flash {
            let age = now.duration_since(t).as_secs_f32();
            if who == pid && age < kit::FLASH * 2.0 {
                kit::flash(c, col.x, col.y, col.w, col.h, 1.0 - age / (kit::FLASH * 2.0));
            }
        }
        // Who: their bust, name and what they're doing.
        let ix = col.x + 14.0;
        let down = w.is_down(pid);
        super::frame::portrait(c, col.x + 34.0, col.y + 34.0, 20.0, p.race, p.seed, down, if pid == focus { 2.5 } else { 1.5 });
        c.styled(p.name().unwrap_or("?"), col.x + 64.0, col.y + 32.0, 18.0, eg(if pid == focus { BRASS_LIGHT } else { HEAD }), Face::Caps, 0.3);
        let k = w.squad.index(pid).unwrap_or(0);
        let (doing, _) = super::squadui::status(w, pid, k);
        c.styled(doing, col.x + 64.0, col.y + 50.0, 13.0, eg(SUB), Face::Italic, 0.0);
        let mut y = col.y + 72.0;
        c.rule(ix, col.x + col.w - 14.0, y, 0.8, ega(BRASS, 0.4));
        // Worn: the body (it shows wounds) and the slots beside it.
        super::frame::figure(c, w, pid, ix + 14.0, y + 60.0);
        let sx = ix + 40.0;
        let locked = w.fighter(pid).is_some();
        y += 6.0;
        // Belt, brooch, neck and rings only when something's in them.
        for s in SLOTS.into_iter().filter(|&s| !s.accessory() || gear.in_slot(s).is_some()) {
            let it = gear.in_slot(s);
            let row = Bx::new(sx - 4.0, y, col.x + col.w - sx - 8.0, SLOT_H);
            let l = if it.is_some() { kit::row(c, &row, st.menu.is_some_and(|m| m.0 == pid && m.1 == Pick::Worn(s))) } else { kit::Look::default() };
            let dy = kit::nudge(l);
            c.styled(s.name(), sx, y + 14.0 + dy, 12.0, eg(FAINT), Face::Body, 0.0);
            let mut name = it.map(|i| item(i).name.to_string()).unwrap_or_else(|| "·".into());
            if let (Some(i), Some(pc)) = (it, gear.piece(s)) {
                name += &super::squadui::wear_word(w, i, pc);
            }
            // Its icon, then its name (`looks.rs`).
            let nx = if let Some(i) = it {
                super::looks::icon(c, i, sx + 70.0, y + 1.0 + dy, 17.0, 17.0);
                sx + 92.0
            } else {
                sx + 72.0
            };
            c.styled(&fit_text(c, &name, 13.0, row.w - (nx - sx) - 8.0), nx, y + 14.0 + dy, 13.0, eg(if it.is_some() { TEXT } else { FAINT }), Face::Body, 0.0);
            if let Some(i) = it {
                if l.hot {
                    tip = Some((i, None));
                }
                if let Some(ck) = click.filter(|k| row.contains(k.at)) {
                    if ck.right {
                        st.menu = Some((pid, Pick::Worn(s), ck.at));
                    } else if !locked && double(st, pid, s as usize, true, now) {
                        acts.push(Action::Unequip(pid, s));
                    }
                }
            }
            y += SLOT_H;
        }
        y += 8.0;
        c.rule(ix, col.x + col.w - 14.0, y, 0.8, ega(BRASS, 0.4));
        // The pack: its grids, pockets then the pack's, each thing in its
        // cells with its icon and how many (items step 3). What doesn't fit is
        // carried loose, listed in red underneath.
        let foot = col.y + col.h - 46.0;
        let (placed, loose) = gear.arrange();
        let cell = ((col.w - 28.0) / 8.0).floor().min(32.0);
        let mut gy = y + 2.0;
        let mut boxes: Vec<Bx> = Vec::new();
        for (g, &(gw, gh)) in gear.grids().iter().enumerate() {
            let label = gear.grid_name(g);
            c.styled(&label, ix, gy + 11.0, 12.0, eg(FAINT), Face::Italic, 0.0);
            gy += 15.0;
            let b = Bx::new(ix, gy, gw as f32 * cell, gh as f32 * cell);
            c.rect(b.x, b.y, b.w, b.h, ega([0.0, 0.0, 0.0], 0.35));
            for yy in 0..gh {
                for xx in 0..gw {
                    c.rect_lines(b.x + xx as f32 * cell, b.y + yy as f32 * cell, cell, cell, 1.0, ega(BRASS, 0.16));
                }
            }
            grid_boxes.push((pid, g as u8, b, cell));
            boxes.push(b);
            gy += b.h + 6.0;
        }
        for (k, pl) in placed.iter().enumerate() {
            let e = pl.entry;
            let i = gear.bag[e].0;
            let gb = boxes[pl.grid as usize];
            let r = Bx::new(gb.x + pl.x as f32 * cell + 1.0, gb.y + pl.y as f32 * cell + 1.0, pl.w as f32 * cell - 2.0, pl.h as f32 * cell - 2.0);
            let lifted = dragged.is_some_and(|d| d.0 == pid && st.grab.is_some_and(|g| g.0 == k));
            let l = kit::row(c, &r, st.menu.is_some_and(|m| m.0 == pid && m.1 == Pick::Entry(e)) || lifted);
            c.rect(r.x, r.y, r.w, r.h, ega([0.24, 0.19, 0.12], if lifted { 0.3 } else { 0.85 }));
            if !lifted {
                super::looks::icon(c, i, r.x + 2.0, r.y + 2.0 + kit::nudge(l), r.w - 4.0, r.h - 4.0);
            }
            if pl.n > 1 {
                c.styled_right(&pl.n.to_string(), r.x + r.w - 3.0, r.y + r.h - 3.0, 11.0, eg(TEXT), Face::Body, 0.0);
            }
            let mix = gear.bag[e].2.and_then(|p| p.mix);
            let sheet = gear.bag[e].2.and_then(|p| p.sheet);
            if l.hot && dragged.is_none() {
                tip = Some((i, Some(pid)));
                tip_mix = mix;
                tip_sheet = sheet;
            }
            if let Some(ck) = click.filter(|ck| r.contains(ck.at)) {
                if ck.right {
                    st.menu = Some((pid, Pick::Entry(e), ck.at));
                } else if !st.dragging && !locked && double(st, pid, e, false, now) {
                    if items::equippable(i) {
                        acts.push(Action::EquipEntry(pid, e));
                    } else if mix.is_some() || sheet.is_some() {
                        acts.push(Action::UseEntry(pid, e));
                    } else if usable(i) {
                        acts.push(Action::Use(pid, i));
                    }
                }
            }
            // A press here may become a drag: to another cell, or another column.
            if left_held && st.press.is_none() && r.contains(mouse) && click.is_none() && st.menu.is_none() {
                st.press = Some((pid, e, mouse));
                st.grab = Some((k, ((mouse.x - r.x) / cell).floor().max(0.0) as u8, ((mouse.y - r.y) / cell).floor().max(0.0) as u8, pl.turned));
            }
        }
        if gear.bag.is_empty() {
            c.styled("Nothing in the pack.", ix + 4.0, gy + 14.0, 14.0, eg(FAINT), Face::Italic, 0.0);
        }
        // Loose: no room for these anywhere.
        let mut ly = gy;
        for &(e, n) in &loose {
            if ly > foot - 24.0 {
                break;
            }
            let i = gear.bag[e].0;
            let row = Bx::new(col.x + 6.0, ly, col.w - 12.0, ROW_H);
            let l = kit::row(c, &row, st.menu.is_some_and(|m| m.0 == pid && m.1 == Pick::Entry(e)));
            super::looks::icon(c, i, ix - 2.0, ly + 1.5, 20.0, 20.0);
            let what = if n > 1 { format!("{} × {}", n, item(i).name) } else { item(i).name.to_string() };
            c.styled(&fit_text(c, &format!("{what}: loose, no room"), 13.0, row.w - 40.0), ix + 24.0, ly + 16.0 + kit::nudge(l), 13.0, eg(BAD), Face::Body, 0.0);
            if l.hot && dragged.is_none() {
                tip = Some((i, Some(pid)));
            }
            if let Some(ck) = click.filter(|ck| row.contains(ck.at)) {
                if ck.right {
                    st.menu = Some((pid, Pick::Entry(e), ck.at));
                }
            }
            if left_held && st.press.is_none() && row.contains(mouse) && click.is_none() && st.menu.is_none() {
                st.press = Some((pid, e, mouse));
                st.grab = None;
            }
            ly += ROW_H;
        }
        // The weight bar: amber near full, red over.
        let load = gear.load(&p.stats);
        let bar = Bx::new(ix, foot + 22.0, col.w - 28.0, 6.0);
        c.rect(bar.x, bar.y, bar.w, bar.h, ega([0.0, 0.0, 0.0], 0.5));
        let bc = if load > 1.0 { BAD } else if load > 0.8 { AMBER } else { [0.70, 0.62, 0.45] };
        c.rect(bar.x, bar.y, bar.w * load.clamp(0.0, 1.0), bar.h, eg(bc));
        let words = if load > 1.0 { format!("{:.0} of {:.0} kg: slowed to {:.0}%", gear.weight(), gear.capacity(&p.stats), inventory::encumbrance_factor(load) * 100.0) } else { format!("{:.0} of {:.0} kg", gear.weight(), gear.capacity(&p.stats)) };
        c.styled(&words, ix, foot + 16.0, 12.0, eg(if load > 1.0 { BAD } else { SUB }), Face::Body, 0.0);
    }
    st.wheel = 0.0;

    // Where a dragged stack would land in its own grids: (grid, x, y).
    let grab = st.grab;
    let landing = |at: Vec2, from: PersonId| -> Option<(u8, u8, u8, f32, Bx)> {
        let (_, ox, oy, _) = grab?;
        let &(_, g, b, cell) = grid_boxes.iter().find(|x| x.0 == from && x.2.contains(at))?;
        let x = ((at.x - b.x) / cell).floor() as i32 - ox as i32;
        let y = ((at.y - b.y) / cell).floor() as i32 - oy as i32;
        Some((g, x.max(0) as u8, y.max(0) as u8, cell, b))
    };
    // Let go after a drag: in its own grids, moved there; on another
    // column, a give.
    if let (Some((from, e, _)), Some(k)) = (dragged, click) {
        if !k.right {
            if let (Some((g, x, y, _, _)), Some((which, _, _, turned))) = (landing(k.at, from), grab) {
                acts.push(Action::Place(from, which, g, x, y, turned));
            }
            if let Some(&(to, _)) = columns.iter().find(|(_, b)| b.contains(k.at)) {
                if to != from {
                    match why_not_give(w, from, to) {
                        None => {
                            acts.push(Action::GiveEntry(from, e, to));
                            st.flash = Some((to, now));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        st.press = None;
        st.dragging = false;
        st.grab = None;
    }
    // Over its own grids: the cells it would take, outlined.
    if let Some((from, e, _)) = dragged {
        if let (Some((_, x, y, cell, b)), Some((_, _, _, turned)), Some(i)) = (landing(mouse, from), grab, w.people[from as usize].detail.as_ref().and_then(|d| d.gear.bag.get(e)).map(|x| x.0)) {
            let (fw, fh) = items::footprint(i);
            let (fw, fh) = if turned { (fh, fw) } else { (fw, fh) };
            c.rect_lines(b.x + x as f32 * cell, b.y + y as f32 * cell, fw as f32 * cell, fh as f32 * cell, 2.0, ega(BRASS_LIGHT, 0.9));
        }
    }
    // The thing being dragged follows the pointer, lifted a little.
    if let Some((from, e, _)) = dragged {
        if let Some((i, n)) = w.people[from as usize].detail.as_ref().and_then(|d| d.gear.bag.get(e)).map(|x| (x.0, x.1)) {
            let label = if n > 1 { format!("{} × {}", n, item(i).name) } else { item(i).name.to_string() };
            let wd = c.styled_width(&label, 14.0, Face::Body, 0.0) + 20.0;
            let at = mouse + vec2(12.0, -6.0);
            c.rect(at.x + 3.0, at.y + 4.0, wd, 24.0, ega([0.0, 0.0, 0.0], 0.45));
            c.rect(at.x, at.y, wd, 24.0, ega([0.20, 0.16, 0.11], 0.97));
            c.rect_lines(at.x, at.y, wd, 24.0, 1.0, ega(BRASS_LIGHT, 0.9));
            c.styled(&label, at.x + 10.0, at.y + 17.0, 14.0, eg(TEXT), Face::Body, 0.0);
            // Where it would go, and whether it can.
            if let Some(&(to, _)) = columns.iter().find(|(_, b)| b.contains(mouse)) {
                if to != from {
                    let line = match why_not_give(w, from, to) {
                        None => (format!("Give to {}", w.name_of(to)), BRASS_LIGHT),
                        Some(why) => (why.to_string(), BAD),
                    };
                    c.styled(&line.0, at.x + 10.0, at.y + 42.0, 13.0, eg(line.1), Face::Italic, 0.0);
                }
            }
        }
    }

    // The right-click menu.
    if let Some((pid, pick, at)) = st.menu {
        let mut items_: Vec<(String, Option<Action>)> = Vec::new();
        let gear = w.people[pid as usize].detail.as_ref().map(|d| &d.gear);
        let locked = w.fighter(pid).is_some();
        match pick {
            Pick::Entry(e) => {
                if let Some((i, mix, sheet)) = gear.and_then(|g| g.bag.get(e)).map(|x| (x.0, x.2.and_then(|p| p.mix), x.2.and_then(|p| p.sheet))) {
                    if !locked && items::equippable(i) {
                        items_.push(("Put on".into(), Some(Action::EquipEntry(pid, e))));
                    } else if !locked && mix.is_some() {
                        items_.push(("Drink".into(), Some(Action::UseEntry(pid, e))));
                    } else if !locked && sheet.is_some() {
                        items_.push(("Read".into(), Some(Action::UseEntry(pid, e))));
                    } else if !locked && usable(i) {
                        items_.push((use_word(i).into(), Some(Action::Use(pid, i))));
                    } else if !locked && items::ingredient(i).is_some() {
                        items_.push(("Taste".into(), Some(Action::Taste(pid, i))));
                    }
                    for &m in members.iter().filter(|&&m| m != pid) {
                        if why_not_give(w, pid, m).is_none() {
                            items_.push((format!("Give to {}", w.name_of(m)), Some(Action::GiveEntry(pid, e, m))));
                        }
                    }
                    if gear.is_some_and(|g| g.arrange().0.iter().any(|p| p.entry == e)) {
                        items_.push(("Turn it".into(), Some(Action::Turn(pid, e))));
                    }
                    items_.push(("Put down".into(), Some(Action::DropEntry(pid, e))));
                }
            }
            Pick::Worn(s) => {
                if gear.and_then(|g| g.in_slot(s)).is_some() && !locked {
                    items_.push(("Take off".into(), Some(Action::Unequip(pid, s))));
                    items_.push(("Seal or mend".into(), Some(Action::Care(pid, s))));
                }
            }
        }
        let wd = items_.iter().map(|x| c.styled_width(&x.0, 14.0, Face::Body, 0.0)).fold(120.0, f32::max) + 30.0;
        let h = items_.len() as f32 * 24.0 + 12.0;
        let mx = at.x.min(c.w - wd - 8.0);
        let my = at.y.min(c.h - h - 8.0);
        c.frame_box(mx, my, wd, h);
        let mut picked = false;
        for (i, (label, a)) in items_.into_iter().enumerate() {
            let b = Bx::new(mx + 4.0, my + 6.0 + i as f32 * 24.0, wd - 8.0, 24.0);
            let l = kit::row(c, &b, false);
            c.styled(&label, b.x + 10.0, b.y + 17.0 + kit::nudge(l), 14.0, eg(TEXT), Face::Body, 0.0);
            if click.is_some_and(|k| !k.right && b.contains(k.at)) {
                if let Some(a) = a {
                    if let Action::GiveEntry(_, _, to) = a {
                        st.flash = Some((to, now));
                    }
                    acts.push(a);
                }
                picked = true;
            }
        }
        // Any other click closes it.
        let inside = Bx::new(mx, my, wd, h);
        if picked || click.is_some_and(|k| !inside.contains(k.at) && !(k.right && k.at == at)) {
            st.menu = None;
        }
    }

    // What's under the mouse: what it is, and for something wearable, what
    // it would replace.
    if let (Some((i, who)), None) = (tip, dragged) {
        if st.menu.is_none() {
            let mut lines = match (tip_mix, tip_sheet) {
                (Some(m), _) => super::squadui::mix_lines(&m),
                (_, Some(s)) => super::squadui::sheet_lines(&s),
                _ => super::squadui::item_lines(i),
            };
            if let Some(pid) = who {
                lines.extend(super::squadui::ingredient_lines(w, pid, i));
                lines.extend(compare(w, pid, i));
            }
            let wd = lines.iter().map(|(l, _)| c.width(l, 14.0)).fold(0.0, f32::max) + 24.0;
            let x = if mouse.x + 24.0 + wd < c.w { mouse.x + 24.0 } else { mouse.x - wd - 24.0 };
            c.panel(&lines, x, mouse.y + 8.0, 14.0);
        }
    }
    (acts, r)
}

/// A click on this row: true if it was the second of a quick pair.
fn double(st: &mut PackState, pid: PersonId, k: usize, worn: bool, now: std::time::Instant) -> bool {
    let twice = st.last.is_some_and(|l| l.0 == pid && l.1 == k && l.2 == worn && now.duration_since(l.3).as_secs_f32() < DOUBLE);
    st.last = if twice { None } else { Some((pid, k, worn, now)) };
    twice
}

/// Things used from the pack (eaten, drunk, read, set up).
fn usable(i: ItemId) -> bool {
    matches!(item(i).kind, Kind::Potion | Kind::Scroll(_) | Kind::Food(_) | Kind::StandingTorch(_) | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_))
}

fn use_word(i: ItemId) -> &'static str {
    match item(i).kind {
        Kind::Food(_) => "Eat",
        Kind::Potion => "Drink",
        Kind::Scroll(_) | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_) => "Read",
        Kind::StandingTorch(_) => "Set it up",
        _ => "Use",
    }
}

/// A line cut to fit `room` pixels, with an ellipsis if it had to be.
fn fit_text(c: &Canvas, s: &str, size: f32, room: f32) -> String {
    if c.styled_width(s, size, Face::Body, 0.0) <= room {
        return s.to_string();
    }
    let mut out = s.to_string();
    while !out.is_empty() && c.styled_width(&format!("{out}…"), size, Face::Body, 0.0) > room {
        out.pop();
    }
    format!("{}…", out.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gahturiyu_sim::sim::worldgen;

    #[test]
    fn what_wearing_it_would_change() {
        let w = worldgen::generate(1);
        // Someone holding a hand weapon, and a bow to hold instead.
        let bow = items::id("short_bow");
        let pid = *w.squad.members.iter().find(|&&m| w.people[m as usize].detail.as_ref().and_then(|d| d.gear.in_slot(Slot::MainHand)).is_some_and(|i| matches!(item(i).kind, Kind::Weapon(ref k) if k.range == 0.0))).unwrap();
        let held = w.people[pid as usize].detail.as_ref().unwrap().gear.in_slot(Slot::MainHand).unwrap();
        let lines = compare(&w, pid, bow);
        assert_eq!(lines[0].0, format!("Instead of the {}", item(held).name.to_lowercase()));
        assert!(lines.iter().any(|l| l.0.starts_with("Range 0 to") && l.1 == GOOD), "{lines:?}");
        // Bread isn't worn: nothing to compare.
        assert!(compare(&w, pid, items::id("flatbread")).is_empty());
    }
}
