//! Your squad in the window: a card per member along the bottom (click to
//! select, right-click for their pack), and the inventory panel.
//!
//! Works immediate-mode: each frame the panels are drawn from the world as it
//! is, and a click on them comes back as an `Action` for main to carry out.

use macroquad::prelude::*;

use gahturiyu_sim::sim::{
    body,
    inventory,
    items::{self, item, Effect, ItemId, Kind, Slot, SLOTS},
    person::PersonId,
    World,
};

use super::ui::{bar_for, race_color, with_alpha, Ui, DIM, PANEL, TEXT};

pub const GOLD: Color = Color::new(1.0, 0.85, 0.35, 1.0);
const WARN: Color = Color::new(0.95, 0.55, 0.3, 1.0);
pub const SNEAK: Color = Color::new(0.62, 0.70, 0.95, 1.0);

/// Who orders go to. Empty means the whole squad.
#[derive(Default, Clone)]
pub struct Selection(pub Vec<PersonId>);

impl Selection {
    /// Everyone orders apply to right now.
    pub fn who(&self, w: &World) -> Vec<PersonId> {
        let alive: Vec<PersonId> = self.0.iter().copied().filter(|p| w.squad.index(*p).is_some()).collect();
        if alive.is_empty() {
            w.squad.members.clone()
        } else {
            alive
        }
    }
    pub fn is_all(&self, w: &World) -> bool {
        self.who(w).len() == w.squad.members.len()
    }
    pub fn shows(&self, w: &World, pid: PersonId) -> bool {
        !self.is_all(w) && self.0.contains(&pid)
    }
    /// The one who acts when only one can (picking something up).
    pub fn lead(&self, w: &World) -> Option<PersonId> {
        self.who(w).first().copied()
    }
    pub fn pick(&mut self, pid: PersonId, add: bool) {
        if add {
            if let Some(i) = self.0.iter().position(|&p| p == pid) {
                self.0.remove(i);
            } else {
                self.0.push(pid);
            }
        } else {
            self.0 = vec![pid];
        }
    }
}

pub enum Action {
    Select(PersonId, bool),
    OpenInventory(PersonId),
    Unequip(PersonId, Slot),
    Equip(PersonId, ItemId),
    Drop(PersonId, ItemId),
    CloseInventory,
}

/// A click this frame: where, and which button.
#[derive(Clone, Copy)]
pub struct Click {
    pub at: Vec2,
    pub right: bool,
}

const CARD_W: f32 = 210.0;
const CARD_H: f32 = 62.0;
const INV_W: f32 = 400.0;
const ROW: f32 = 21.0;

fn card_rect(k: usize) -> Rect {
    Rect::new(12.0 + k as f32 * (CARD_W + 8.0), screen_height() - 30.0 - CARD_H - 10.0, CARD_W, CARD_H)
}

fn inv_rect(w: &World, pid: PersonId) -> Rect {
    let bag = w.people[pid as usize].detail.as_ref().map(|d| d.gear.bag.len()).unwrap_or(0);
    let h = 70.0 + (SLOTS.len() as f32 + 1.0) * ROW + (bag.max(1) as f32 + 1.0) * ROW + 40.0;
    Rect::new(screen_width() - INV_W - 12.0, 12.0, INV_W, h.min(screen_height() - 130.0))
}

/// Is the mouse over any of the squad panels (so a click there isn't an order)?
pub fn over(w: &World, mouse: Vec2, inventory: Option<PersonId>) -> bool {
    (0..w.squad.members.len()).any(|k| card_rect(k).contains(mouse)) || inventory.map(|p| inv_rect(w, p).contains(mouse)).unwrap_or(false)
}

fn status(w: &World, pid: PersonId, k: usize) -> (&'static str, Color) {
    let p = &w.people[pid as usize];
    if let Some(f) = w.fighter(pid) {
        if f.ko {
            return ("Down", WARN);
        }
        return ("Fighting", Color::new(0.95, 0.4, 0.35, 1.0));
    }
    if body::knocked_out(&p.wounds.hp_at(&p.stats, w.time)) {
        return ("Down", WARN);
    }
    if w.pickups.iter().any(|pk| pk.who == pid) {
        return ("Fetching", TEXT);
    }
    let sneaking = w.squad.sneaking[k];
    if w.squad.at[k].dist(w.squad.goal[k]) > 0.5 {
        return if sneaking { ("Sneaking", SNEAK) } else { ("Walking", TEXT) };
    }
    if sneaking {
        ("Crouched", SNEAK)
    } else {
        ("Standing", DIM)
    }
}

/// The cards along the bottom.
pub fn squad_bar(ui: &Ui, w: &World, sel: &Selection, click: Option<Click>) -> Option<Action> {
    let mut act = None;
    for (k, &pid) in w.squad.members.iter().enumerate() {
        let r = card_rect(k);
        let p = &w.people[pid as usize];
        let chosen = sel.shows(w, pid);
        draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
        draw_rectangle(r.x, r.y, 5.0, r.h, race_color(p.race));
        if chosen {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, GOLD);
        }
        ui.text(p.name().unwrap_or("?"), r.x + 14.0, r.y + 19.0, 16, if chosen { GOLD } else { TEXT });
        let key = format!("F{}", k + 1);
        ui.text(&key, r.x + r.w - ui.width(&key, 13) - 8.0, r.y + 17.0, 13, DIM);

        let (st, sc) = status(w, pid, k);
        let gear = p.kit();
        let load = gear.load(&p.stats);
        ui.text(st, r.x + 14.0, r.y + 37.0, 14, sc);
        // How close to being noticed: an eye that opens.
        let sus = w.suspicion_of(pid);
        if sus > 0.02 {
            let ex = r.x + 14.0 + ui.width(st, 14) + 16.0;
            let ey = r.y + 32.0;
            let c = if sus >= 1.0 { Color::new(0.95, 0.3, 0.25, 1.0) } else { Color::new(0.95, 0.8, 0.35, 1.0) };
            draw_ellipse_lines(ex, ey, 8.0, 1.0 + 4.0 * sus, 0.0, 1.5, c);
            draw_circle(ex, ey, 1.5 + 1.5 * sus, c);
        }
        let l = format!("{:.0}/{:.0} kg", gear.weight(), gear.capacity(&p.stats));
        ui.text(&l, r.x + r.w - ui.width(&l, 13) - 8.0, r.y + 37.0, 13, if load > 1.0 { WARN } else { DIM });

        // Health, and mana for those with spells.
        let (vit, mana, down) = bar_for(w, pid).unwrap_or((1.0, None, false));
        let mana = mana.or_else(|| {
            let d = p.detail.as_ref()?;
            (!d.spells.is_empty()).then(|| p.mana_at(w.time) / p.max_mana().max(1.0))
        });
        let bw = r.w - 22.0;
        draw_rectangle(r.x + 14.0, r.y + 45.0, bw, 5.0, Color::new(0.0, 0.0, 0.0, 0.6));
        let hc = if down { WARN } else { Color::new(0.85 - vit * 0.6, 0.25 + vit * 0.6, 0.25, 1.0) };
        draw_rectangle(r.x + 14.0, r.y + 45.0, bw * vit.clamp(0.0, 1.0), 5.0, hc);
        if let Some(m) = mana {
            draw_rectangle(r.x + 14.0, r.y + 53.0, bw, 3.0, Color::new(0.0, 0.0, 0.0, 0.6));
            draw_rectangle(r.x + 14.0, r.y + 53.0, bw * m.clamp(0.0, 1.0), 3.0, Color::new(0.35, 0.55, 1.0, 1.0));
        }

        if let Some(c) = click {
            if r.contains(c.at) {
                let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                act = Some(if c.right { Action::OpenInventory(pid) } else { Action::Select(pid, shift) });
            }
        }
    }
    act
}

/// One person's gear: worn on the left of each row, the pack below.
/// Returns what was clicked, and the item under the mouse (for a tooltip).
pub fn inventory(ui: &Ui, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<ItemId>) {
    let p = &w.people[pid as usize];
    let Some(d) = p.detail.as_ref() else { return (None, None) };
    let r = inv_rect(w, pid);
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL);
    draw_rectangle(r.x, r.y, r.w, 4.0, race_color(p.race));
    let mut act = None;
    let mut hovered = None;
    let clicked = |rect: Rect| click.filter(|c| rect.contains(c.at));

    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    ui.text(&format!("{}  ·  pack and gear", p.name().unwrap_or("?")), x, y, 17, race_color(p.race));
    let close = Rect::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    ui.text("×", close.x + 3.0, close.y + 15.0, 18, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(close).is_some() {
        act = Some(Action::CloseInventory);
    }
    y += 22.0;
    let gear = &d.gear;
    let load = gear.load(&p.stats);
    let speed = inventory::encumbrance_factor(load);
    let line = format!(
        "Carrying {:.1} of {:.0} kg{}",
        gear.weight(),
        gear.capacity(&p.stats),
        if load > 1.0 { format!("  ·  overloaded, {:.0}% speed", speed * 100.0) } else { String::new() }
    );
    ui.text(&line, x, y, 14, if load > 1.0 { WARN } else { DIM });
    let locked = w.fighter(pid).is_some();
    y += 10.0;

    let row = |y: f32| Rect::new(r.x + 6.0, y - 15.0, r.w - 12.0, ROW);
    y += ROW;
    ui.text("Worn", x, y, 15, TEXT);
    for s in SLOTS {
        y += ROW;
        let rr = row(y);
        let it = gear.in_slot(s);
        if rr.contains(mouse) && it.is_some() {
            draw_rectangle(rr.x, rr.y, rr.w, rr.h, with_alpha(GOLD, 0.12));
            hovered = it;
        }
        ui.text(s.name(), x + 8.0, y, 14, DIM);
        ui.text(it.map(|i| item(i).name).unwrap_or("—"), x + 110.0, y, 14, if it.is_some() { TEXT } else { DIM });
        if let Some(i) = it {
            let kg = format!("{:.1} kg", item(i).weight);
            ui.text(&kg, r.x + r.w - ui.width(&kg, 13) - 14.0, y, 13, DIM);
            if !locked && clicked(rr).is_some() {
                act = Some(Action::Unequip(pid, s));
            }
        }
    }
    y += ROW + 4.0;
    ui.text("Pack", x, y, 15, TEXT);
    if gear.bag.is_empty() {
        y += ROW;
        ui.text("empty", x + 8.0, y, 14, DIM);
    }
    for &(i, n) in &gear.bag {
        y += ROW;
        if y > r.y + r.h - 34.0 {
            ui.text("…", x + 8.0, y, 14, DIM);
            break;
        }
        let rr = row(y);
        if rr.contains(mouse) {
            draw_rectangle(rr.x, rr.y, rr.w, rr.h, with_alpha(GOLD, 0.12));
            hovered = Some(i);
        }
        let label = if n > 1 { format!("{}  ×{n}", item(i).name) } else { item(i).name.to_string() };
        ui.text(&label, x + 8.0, y, 14, TEXT);
        let kg = format!("{:.1} kg", item(i).weight * n as f32);
        ui.text(&kg, r.x + r.w - ui.width(&kg, 13) - 14.0, y, 13, DIM);
        if let Some(c) = clicked(rr) {
            if c.right {
                act = Some(Action::Drop(pid, i));
            } else if !locked && items::equippable(i) {
                act = Some(Action::Equip(pid, i));
            }
        }
    }
    let hint = if locked { "In a fight: gear can't be changed until it's over." } else { "Click: take off / put on  ·  Right-click: drop" };
    ui.text(hint, x, r.y + r.h - 12.0, 13, if locked { WARN } else { DIM });
    (act, hovered)
}

/// A few lines describing an item.
pub fn item_lines(id: ItemId) -> Vec<(String, Color)> {
    let d = item(id);
    let mut out = vec![(d.name.to_string(), GOLD)];
    match &d.kind {
        Kind::Weapon(wd) => {
            out.push((format!("{}  ·  {} weapon{}", d.slot.name(), wd.skill.name().to_lowercase(), if wd.two_handed { ", two-handed" } else { "" }), TEXT));
            out.push((format!("Cut {:.0}  ·  Blunt {:.0}  ·  Reach {:.1} m  ·  Parry {:.0}%", wd.cut, wd.blunt, wd.reach, wd.parry * 100.0), DIM));
        }
        Kind::Armor(a) => {
            out.push((format!("{}  ·  armour", d.slot.name()), TEXT));
            out.push((format!("Covers {:.0}%  ·  stops {:.0}% cut, {:.0}% blunt", a.coverage * 100.0, a.cut * 100.0, a.blunt * 100.0), DIM));
            if a.dodge_penalty > 0.0 {
                out.push((format!("Hampers dodging by {:.0}", a.dodge_penalty * 100.0), DIM));
            }
        }
        Kind::Shield(b) => out.push((format!("Off hand  ·  shield, blocks {:.0}%", b * 100.0), TEXT)),
        Kind::Pack(kg) => out.push((format!("Back  ·  pack, holds {kg:.0} kg more"), TEXT)),
        Kind::Trinket => out.push((format!("{}  ·  trinket", d.slot.name()), TEXT)),
        #[allow(unreachable_patterns)]
        _ => {}
    }
    for e in d.effects {
        out.push((effect_text(e), Color::new(0.65, 0.78, 1.0, 1.0)));
    }
    out.push((format!("{:.1} kg  ·  worth {:.0}", d.weight, d.value), DIM));
    out
}

pub fn effect_text(e: &Effect) -> String {
    match *e {
        Effect::Attr(a, v) => format!("{v:+.0} {}", a.name()),
        Effect::Skill(k, v) => format!("{v:+.0} {}", k.name()),
        Effect::MaxMana(v) => format!("{v:+.0} mana"),
        Effect::ManaRegen(v) => format!("{v:+.1} mana a minute"),
        Effect::Carry(v) => format!("{v:+.0} kg carrying"),
        Effect::MoveSpeed(v) => format!("{:+.0}% speed", v * 100.0),
        Effect::ResistParalysis(v) => format!("Resist paralysis {:.0}%", v * 100.0),
        Effect::ResistBlind(v) => format!("Resist blindness {:.0}%", v * 100.0),
        Effect::ResistElements(v) => format!("Resist fire and lightning {:.0}%", v * 100.0),
    }
}

/// Colour for a thing lying on the ground, by kind.
pub fn ground_color(id: ItemId) -> Color {
    match item(id).kind {
        Kind::Weapon(_) => Color::new(0.72, 0.74, 0.78, 1.0),
        Kind::Armor(_) => Color::new(0.55, 0.38, 0.24, 1.0),
        Kind::Shield(_) => Color::new(0.62, 0.48, 0.30, 1.0),
        Kind::Pack(_) => Color::new(0.70, 0.60, 0.42, 1.0),
        Kind::Trinket => GOLD,
        #[allow(unreachable_patterns)]
        _ => Color::new(0.8, 0.8, 0.7, 1.0),
    }
}
