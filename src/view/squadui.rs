//! Your squad in the window: a card per member along the bottom (click to
//! select, right-click for their pack), the pack, crafting, conversation and
//! journal panels.
//!
//! Works immediate-mode: each frame the panels are drawn from the world as it
//! is, and a click on them comes back as an `Action` for the app to carry out.

use bevy::math::Vec2;
use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    body,
    condition::{self, HungerStage, Shelter},
    crafting::{success_chance, Cannot, Station, RECIPES},
    dialogue::Topic,
    inventory,
    items::{self, item, ItemId, Kind, Slot, SLOTS},
    person::PersonId,
    quests::Stage,
    stats::Skill,
    World,
};

use super::hud::{bar_for, health_color, Canvas, PANEL};
use super::palette::{eg, ega, race_color, Rgb, DIM, GOLD, MANA, SNEAK, TEXT, WARN};

/// A box on screen, pixels from the top left.
#[derive(Clone, Copy, Debug)]
pub struct Bx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Bx {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Bx {
        Bx { x, y, w, h }
    }
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

/// Who orders go to. Empty means the whole squad.
#[derive(Default, Clone)]
pub struct Selection(pub Vec<PersonId>);

impl Selection {
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
    Use(PersonId, ItemId),
    Craft(PersonId, usize),
    CloseInventory,
}

/// A click this frame: where, which button, and whether Shift was down.
#[derive(Clone, Copy)]
pub struct Click {
    pub at: Vec2,
    pub right: bool,
    pub shift: bool,
}

const CARD_W: f32 = 210.0;
pub const CARD_H: f32 = 100.0;
const INV_W: f32 = 400.0;
const ROW: f32 = 21.0;

fn card_rect(c: &Canvas, k: usize) -> Bx {
    Bx::new(12.0 + k as f32 * (CARD_W + 8.0), c.h - 30.0 - CARD_H - 10.0, CARD_W, CARD_H)
}

fn inv_rect(c: &Canvas, w: &World, pid: PersonId) -> Bx {
    let bag = w.people[pid as usize].detail.as_ref().map(|d| d.gear.bag.len()).unwrap_or(0);
    let h = 70.0 + (SLOTS.len() as f32 + 1.0) * ROW + (bag.max(1) as f32 + 1.0) * ROW + 40.0;
    Bx::new(c.w - INV_W - 12.0, 12.0, INV_W, h.min(c.h - 130.0))
}

fn status(w: &World, pid: PersonId, k: usize) -> (&'static str, Rgb) {
    let p = &w.people[pid as usize];
    if let Some(f) = w.fighter(pid) {
        if f.ko {
            return ("Down", WARN);
        }
        return ("Fighting", [0.95, 0.4, 0.35]);
    }
    if body::knocked_out(&p.wounds.hp_at(&p.stats, w.time)) {
        return ("Down", WARN);
    }
    if w.carried_by(pid).is_some() {
        return ("Being carried", WARN);
    }
    if w.carrying(pid).is_some() {
        return ("Carrying", GOLD);
    }
    if w.is_asleep(pid) {
        let c = w.people[pid as usize].cond.as_ref().unwrap();
        let place = match c.shelter {
            Shelter::Open => "Asleep (open)",
            Shelter::Tent => "Asleep (tent)",
            Shelter::Indoors => "Asleep (indoors)",
        };
        return (place, SNEAK);
    }
    if w.crafting.iter().any(|j| j.who == pid) {
        return ("Crafting", GOLD);
    }
    if w.picking.iter().any(|pk| pk.who == pid) {
        return ("Picking a lock", SNEAK);
    }
    if w.gathering.iter().any(|g| g.0 == pid) {
        return ("Gathering", TEXT);
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

/// The cards along the bottom. Returns a click's action and the cards' boxes.
pub fn squad_bar(c: &Canvas, w: &World, sel: &Selection, click: Option<Click>) -> (Option<Action>, Vec<Bx>) {
    let mut act = None;
    let mut boxes = Vec::new();
    for (k, &pid) in w.squad.members.iter().enumerate() {
        let r = card_rect(c, k);
        boxes.push(r);
        let p = &w.people[pid as usize];
        let chosen = sel.shows(w, pid);
        c.rect(r.x, r.y, r.w, r.h, PANEL);
        c.rect(r.x, r.y, 5.0, r.h, eg(race_color(p.race)));
        if chosen {
            c.rect_lines(r.x, r.y, r.w, r.h, 2.0, eg(GOLD));
        }
        let name = p.name().unwrap_or("?");
        c.text(name, r.x + 14.0, r.y + 19.0, 16.0, if chosen { GOLD } else { TEXT });
        // Lost limbs, in red after the name; a lit torch after that.
        let mut after = r.x + 18.0 + c.width(name, 16.0);
        let gone = p.wounds.lost_limbs();
        if !gone.is_empty() {
            let short: Vec<String> = gone.iter().map(|g| g.split(' ').map(|w| w[..1].to_uppercase()).collect::<String>()).collect();
            let tag = format!("−{}", short.join(" −"));
            c.text(&tag, after, r.y + 19.0, 13.0, [0.95, 0.35, 0.3]);
            after += c.width(&tag, 13.0) + 6.0;
        }
        if w.torch_lit(pid) {
            c.text("torch", after, r.y + 19.0, 12.0, [1.0, 0.7, 0.35]);
        }
        let key = format!("F{}", k + 1);
        c.text(&key, r.x + r.w - c.width(&key, 13.0) - 8.0, r.y + 17.0, 13.0, DIM);

        let (st, sc) = status(w, pid, k);
        let gear = p.kit();
        let load = gear.load(&p.stats);
        c.text(st, r.x + 14.0, r.y + 37.0, 14.0, sc);
        // How close to being noticed: an eye that opens.
        let sus = w.suspicion_of(pid);
        if sus > 0.02 {
            let ex = r.x + 14.0 + c.width(st, 14.0) + 16.0;
            let ey = r.y + 32.0;
            let col = if sus >= 1.0 { eg([0.95, 0.3, 0.25]) } else { eg([0.95, 0.8, 0.35]) };
            c.ellipse_lines(ex, ey, 8.0, 1.0 + 4.0 * sus, 1.5, col);
            c.circle(ex, ey, 1.5 + 1.5 * sus, col);
        }
        let burden = w.burden_weight(pid);
        let load = if burden > 0.0 { w.load_of(pid) } else { load };
        let l = format!("{:.0}/{:.0} kg", gear.weight() + burden, gear.capacity(&p.stats));
        c.text(&l, r.x + r.w - c.width(&l, 13.0) - 8.0, r.y + 37.0, 13.0, if load > 1.0 { WARN } else { DIM });

        // Health, and mana for those with spells.
        let (vit, mana, down) = bar_for(w, pid).unwrap_or((1.0, None, false));
        let mana = mana.or_else(|| {
            let d = p.detail.as_ref()?;
            (!d.spells.is_empty()).then(|| p.mana_at(w.time) / p.max_mana().max(1.0))
        });
        let bw = r.w - 22.0;
        c.rect(r.x + 14.0, r.y + 45.0, bw, 5.0, Color32::from_black_alpha(153));
        c.rect(r.x + 14.0, r.y + 45.0, bw * vit.clamp(0.0, 1.0), 5.0, if down { eg(WARN) } else { health_color(vit, false) });
        if let Some(f) = w.craft_progress(pid) {
            c.rect(r.x + 14.0, r.y + 40.0, bw * f, 2.0, eg(GOLD));
        }
        if let Some(m) = mana {
            c.rect(r.x + 14.0, r.y + 53.0, bw, 3.0, Color32::from_black_alpha(153));
            c.rect(r.x + 14.0, r.y + 53.0, bw * m.clamp(0.0, 1.0), 3.0, eg(MANA));
        }

        // Food, stamina and rest: three small bars (full = good).
        let third = (bw - 16.0) / 3.0;
        let bars = [
            ("food", w.hunger_of(pid).map(|h| 1.0 - h / 100.0), [0.85, 0.6, 0.25]),
            ("stam", w.stamina_of(pid), [0.45, 0.8, 0.55]),
            ("rest", w.tired_of(pid).map(|t| 1.0 - t / 100.0), [0.65, 0.55, 0.95]),
        ];
        let cond = p.cond.as_ref();
        let hunger_word = cond.map(|cd| match condition::stage_of(cd.hunger_at(w.time)) {
            HungerStage::Fed => "food",
            HungerStage::Hungry => "hungry",
            HungerStage::Weak => "weak",
            HungerStage::Starving => "starving",
        });
        let tired_word = cond.map(|cd| if cd.tired_at(w.time) >= condition::EXHAUSTED { "worn out" } else { "rest" });
        for (n, (label, v, col)) in bars.iter().enumerate() {
            let Some(v) = v else { continue };
            let x = r.x + 14.0 + n as f32 * (third + 8.0);
            let word = match n {
                0 => hunger_word.unwrap_or(label),
                2 => tired_word.unwrap_or(label),
                _ => label,
            };
            let warn = word != *label;
            c.text(word, x, r.y + 79.0, 11.0, if warn { WARN } else { DIM });
            c.rect(x, r.y + 62.0, third, 5.0, Color32::from_black_alpha(153));
            let bc = if *v < 0.25 { [0.95, 0.35, 0.3] } else { *col };
            c.rect(x, r.y + 62.0, third * v.clamp(0.0, 1.0), 5.0, eg(bc));
        }
        let line = if let Some(cp) = w.carrying(pid) {
            Some(format!("Carrying {}", w.people[cp as usize].name().unwrap_or("someone")))
        } else {
            w.carried_by(pid).map(|cp| format!("Carried by {}", w.people[cp as usize].name().unwrap_or("someone")))
        };
        if let Some(l) = line {
            c.text(&l, r.x + 14.0, r.y + 92.0, 12.0, GOLD);
        }

        if let Some(ck) = click {
            if r.contains(ck.at) {
                act = Some(if ck.right { Action::OpenInventory(pid) } else { Action::Select(pid, ck.shift) });
            }
        }
    }
    (act, boxes)
}

/// One person's gear: worn on the left of each row, the pack below.
/// Returns what was clicked, the item under the mouse, and the panel's box.
pub fn inventory(c: &Canvas, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<ItemId>, Option<Bx>) {
    let p = &w.people[pid as usize];
    let Some(d) = p.detail.as_ref() else { return (None, None, None) };
    let r = inv_rect(c, w, pid);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(race_color(p.race)));
    let mut act = None;
    let mut hovered = None;
    let clicked = |rect: Bx| click.filter(|k| rect.contains(k.at));

    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    c.text(&format!("{}  ·  pack and gear", p.name().unwrap_or("?")), x, y, 17.0, race_color(p.race));
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(close).is_some() {
        act = Some(Action::CloseInventory);
    }
    y += 22.0;
    let gear = &d.gear;
    let load = gear.load(&p.stats);
    let speed = inventory::encumbrance_factor(load);
    let line = format!("Carrying {:.1} of {:.0} kg{}", gear.weight(), gear.capacity(&p.stats), if load > 1.0 { format!("  ·  overloaded, {:.0}% speed", speed * 100.0) } else { String::new() });
    c.text(&line, x, y, 14.0, if load > 1.0 { WARN } else { DIM });
    let locked = w.fighter(pid).is_some();
    y += 10.0;

    let row = |y: f32| Bx::new(r.x + 6.0, y - 15.0, r.w - 12.0, ROW);
    y += ROW;
    c.text("Worn", x, y, 15.0, TEXT);
    for s in SLOTS {
        y += ROW;
        let rr = row(y);
        let it = gear.in_slot(s);
        if rr.contains(mouse) && it.is_some() {
            c.rect(rr.x, rr.y, rr.w, rr.h, ega(GOLD, 0.12));
            hovered = it;
        }
        c.text(s.name(), x + 8.0, y, 14.0, DIM);
        let mut name = it.map(|i| item(i).name.to_string()).unwrap_or("—".into());
        if it.map(|i| matches!(item(i).kind, Kind::Torch(_))).unwrap_or(false) {
            if let Some(h) = w.torch_hours_left(pid) {
                name = format!("{name} ({}, {:.1} h)", if w.torch_lit(pid) { "lit" } else { "out" }, h);
            }
        }
        c.text(&name, x + 110.0, y, 14.0, if it.is_some() { TEXT } else { DIM });
        if let Some(i) = it {
            let kg = format!("{:.1} kg", item(i).weight);
            c.text(&kg, r.x + r.w - c.width(&kg, 13.0) - 14.0, y, 13.0, DIM);
            if !locked && clicked(rr).is_some() {
                act = Some(Action::Unequip(pid, s));
            }
        }
    }
    y += ROW + 4.0;
    c.text("Pack", x, y, 15.0, TEXT);
    if gear.bag.is_empty() {
        y += ROW;
        c.text("empty", x + 8.0, y, 14.0, DIM);
    }
    for &(i, n) in &gear.bag {
        y += ROW;
        if y > r.y + r.h - 34.0 {
            c.text("…", x + 8.0, y, 14.0, DIM);
            break;
        }
        let rr = row(y);
        if rr.contains(mouse) {
            c.rect(rr.x, rr.y, rr.w, rr.h, ega(GOLD, 0.12));
            hovered = Some(i);
        }
        let label = if n > 1 { format!("{}  ×{n}", item(i).name) } else { item(i).name.to_string() };
        c.text(&label, x + 8.0, y, 14.0, TEXT);
        let kg = format!("{:.1} kg", item(i).weight * n as f32);
        c.text(&kg, r.x + r.w - c.width(&kg, 13.0) - 14.0, y, 13.0, DIM);
        if let Some(ck) = clicked(rr) {
            if ck.right {
                act = Some(Action::Drop(pid, i));
            } else if !locked && items::equippable(i) {
                act = Some(Action::Equip(pid, i));
            } else if !locked && matches!(item(i).kind, Kind::Potion | Kind::Scroll(_) | Kind::Food(_) | Kind::StandingTorch(_) | Kind::Notes(_) | Kind::Text(_)) {
                act = Some(Action::Use(pid, i));
            }
        }
    }
    let hint = if locked { "In a fight: gear can't be changed until it's over." } else { "Click: take off / put on / use  ·  Right-click: drop  ·  T: torch" };
    c.text(hint, x, r.y + r.h - 12.0, 13.0, if locked { WARN } else { DIM });
    (act, hovered, Some(r))
}

/// A few lines describing an item.
pub fn item_lines(id: ItemId) -> Vec<(String, Rgb)> {
    let d = item(id);
    let mut out = vec![(d.name.to_string(), GOLD)];
    match &d.kind {
        Kind::Weapon(wd) => {
            out.push((format!("{}  ·  {} weapon{}", d.slot.name(), wd.skill.name().to_lowercase(), if wd.two_handed { ", two-handed" } else { "" }), TEXT));
            if wd.range > 0.0 {
                out.push((format!("Cut {:.0}  ·  Blunt {:.0}  ·  Range {:.0} m  ·  shoots {}", wd.cut, wd.blunt, wd.range, wd.ammo.map(|k| item(items::id(k)).name.to_lowercase()).unwrap_or_default()), DIM));
            } else {
                out.push((format!("Cut {:.0}  ·  Blunt {:.0}  ·  Reach {:.1} m  ·  Parry {:.0}%", wd.cut, wd.blunt, wd.reach, wd.parry * 100.0), DIM));
            }
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
        Kind::Tool => out.push(("Tool".into(), TEXT)),
        Kind::Coin => out.push(("Money".into(), TEXT)),
        Kind::Ammo => out.push(("Ammunition: used up a shot at a time; about half is found again after a fight".into(), TEXT)),
        Kind::Food(n) => out.push((format!("Food: takes {n:.0} off hunger  ·  eaten when hungry, or click to eat"), TEXT)),
        Kind::Errand => out.push(("Someone else's: deliver it".into(), TEXT)),
        Kind::Torch(h) => {
            out.push((format!("Off hand  ·  burns {h:.0} hours  ·  T to light or put out"), TEXT));
            out.push(("Lights the ground round you at night; also makes you easy to see from far off".into(), DIM));
        }
        Kind::StandingTorch(h) => out.push((format!("Click in the pack to set it in the ground  ·  burns {h:.0} hours"), TEXT)),
        Kind::Notes(key) | Kind::Text(key) => {
            let sp = gahturiyu_sim::sim::magic::spell(key).def();
            let need = sp.min_skill * gahturiyu_sim::sim::casting::READ_SKILL;
            out.push((format!("Teaches {} ({} · {})  ·  click to read", sp.name, sp.style.name(), sp.domain.name()), TEXT));
            out.push((format!("Needs {need:.0} {} to follow; kept after reading", sp.style.skill().name()), DIM));
        }
        Kind::Material => {
            let uses: Vec<&str> = RECIPES.iter().filter(|r| r.inputs.iter().any(|(k, _)| *k == d.key)).map(|r| item(items::id(r.output)).name).collect();
            out.push(("Material".into(), TEXT));
            if !uses.is_empty() {
                out.push((format!("Used for: {}", uses.join(", ")), DIM));
            }
        }
        Kind::Potion => out.push(("Potion  ·  click in the pack to drink".into(), TEXT)),
        Kind::Scroll(key) => {
            let sp = gahturiyu_sim::sim::magic::spell(key).def();
            out.push((format!("Scroll: casts {} once, no energy, can't fail", sp.name.to_lowercase()), TEXT));
            for e in sp.effects {
                out.push((e.describe(), [0.65, 0.78, 1.0]));
            }
        }
    }
    for e in d.effects {
        out.push((e.describe(), [0.65, 0.78, 1.0]));
    }
    out.push((format!("{:.1} kg  ·  worth {:.0}", d.weight, d.value), DIM));
    out
}

/// Colour for a thing lying on the ground, by kind.
pub fn ground_color(id: ItemId) -> Rgb {
    match item(id).kind {
        Kind::Weapon(_) => [0.72, 0.74, 0.78],
        Kind::Armor(_) => [0.55, 0.38, 0.24],
        Kind::Shield(_) => [0.62, 0.48, 0.30],
        Kind::Pack(_) => [0.70, 0.60, 0.42],
        Kind::Trinket | Kind::Coin => GOLD,
        Kind::Potion => [0.85, 0.25, 0.3],
        Kind::Scroll(_) | Kind::Errand | Kind::Notes(_) | Kind::Text(_) => [0.9, 0.86, 0.7],
        Kind::Material => [0.55, 0.62, 0.45],
        Kind::Tool => [0.5, 0.5, 0.55],
        Kind::Ammo => [0.6, 0.55, 0.45],
        Kind::Food(_) => [0.75, 0.55, 0.35],
        Kind::Torch(_) | Kind::StandingTorch(_) => [0.45, 0.32, 0.2],
    }
}

fn craft_rect(c: &Canvas) -> Bx {
    let rows = RECIPES.len() as f32 + 4.0 + 3.0;
    Bx::new(c.w - 600.0 - 12.0, 12.0, 600.0, (70.0 + rows * ROW).min(c.h - 130.0))
}

/// What a person can make: every recipe, with what's missing.
pub fn crafting(c: &Canvas, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<ItemId>, Bx) {
    let p = &w.people[pid as usize];
    let r = craft_rect(c);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    let x = r.x + 14.0;
    let mut y = r.y + 26.0;
    let mut act = None;
    let mut hovered = None;
    c.text(&format!("{}  ·  crafting", p.name().unwrap_or("?")), x, y, 17.0, GOLD);
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if click.map(|k| close.contains(k.at)).unwrap_or(false) {
        act = Some(Action::CloseInventory);
    }
    y += 20.0;
    let st = p.effective_stats();
    c.text(
        &format!(
            "Alchemy {:.0}  ·  Inscription {:.0}  ·  Smithing {:.0}  ·  Armoring {:.0}",
            st.skill(Skill::Alchemy),
            st.skill(Skill::Inscription),
            st.skill(Skill::Smithing),
            st.skill(Skill::Armoring)
        ),
        x,
        y,
        14.0,
        DIM,
    );
    let mut last_skill = None;
    for (i, rc) in RECIPES.iter().enumerate() {
        if last_skill != Some(rc.skill) {
            y += ROW + 2.0;
            c.text(rc.skill.name(), x, y, 15.0, TEXT);
            last_skill = Some(rc.skill);
        }
        y += ROW;
        if y > r.y + r.h - 30.0 {
            break;
        }
        let row = Bx::new(r.x + 6.0, y - 15.0, r.w - 12.0, ROW);
        let out = items::id(rc.output);
        let ok = w.can_craft(pid, i);
        if row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.12));
            hovered = Some(out);
        }
        let col = if ok.is_ok() { TEXT } else { DIM };
        c.text(item(out).name, x + 8.0, y, 14.0, col);
        let need: Vec<String> = rc.inputs.iter().map(|(k, n)| format!("{}/{} {}", w.count_of(pid, k).min(*n), n, item(items::id(k)).name.to_lowercase())).collect();
        c.text(&need.join(", "), x + 200.0, y, 13.0, col);
        let why = match ok {
            Ok(()) => format!("{:.0}%", success_chance(st.skill(rc.skill), rc.difficulty) * 100.0),
            Err(Cannot::NoStation(s)) => format!(
                "at {}",
                match s {
                    Station::Forge => "forge",
                    Station::Bench => "bench",
                    Station::Desk => "desk",
                    Station::AlchemyTable => "table",
                }
            ),
            Err(Cannot::Missing(..)) => String::new(),
            Err(Cannot::Busy) => "busy".into(),
        };
        c.text(&why, r.x + r.w - c.width(&why, 13.0) - 14.0, y, 13.0, if ok.is_ok() { GOLD } else { WARN });
        if ok.is_ok() && click.map(|k| row.contains(k.at) && !k.right).unwrap_or(false) {
            act = Some(Action::Craft(pid, i));
        }
    }
    c.text("Click a recipe to make it. Stations stand round every town's hearth.", x, r.y + r.h - 12.0, 13.0, DIM);
    (act, hovered, r)
}

fn talk_rect(c: &Canvas) -> Bx {
    let w = 760.0f32.min(c.w - 24.0);
    Bx::new((c.w - w) / 2.0, c.h - 30.0 - CARD_H - 20.0 - 380.0, w, 380.0)
}

/// The conversation: what's been said on the left, topics to ask on the right.
pub fn talk(c: &Canvas, w: &World, mouse: Vec2, click: Option<Click>) -> (Option<Topic>, Option<Bx>) {
    let Some(cv) = w.talk.as_ref() else { return (None, None) };
    let r = talk_rect(c);
    let npc = &w.people[cv.npc as usize];
    c.rect(r.x, r.y, r.w, r.h, Color32::from_rgba_unmultiplied(13, 15, 18, 240));
    c.rect(r.x, r.y, r.w, 4.0, eg(race_color(npc.race)));
    let x = r.x + 16.0;
    let disp = w.disposition(cv.npc, cv.with);
    c.text(&format!("{}  ·  {} {}", npc.name().unwrap_or("?"), npc.race.name(), npc.stats.calling.name()), x, r.y + 28.0, 18.0, race_color(npc.race));
    let d = format!("Disposition {disp:.0}");
    c.text(&d, r.x + r.w - c.width(&d, 14.0) - 16.0, r.y + 26.0, 14.0, if disp < 30.0 { WARN } else { DIM });

    let tx = r.x + r.w - 200.0;
    let mut ty = r.y + 60.0;
    let mut chosen = None;
    for t in w.topics() {
        let row = Bx::new(tx - 6.0, ty - 15.0, 190.0, 21.0);
        let hot = row.contains(mouse);
        if hot {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.15));
        }
        let label = t.text();
        let fs = (15.0 * 186.0 / c.width(&label, 15.0).max(1.0)).clamp(10.0, 15.0);
        c.text(&label, tx, ty, fs, if hot { GOLD } else { TEXT });
        if click.map(|k| row.contains(k.at) && !k.right).unwrap_or(false) {
            chosen = Some(t);
        }
        ty += 22.0;
    }

    let width = tx - x - 24.0;
    let mut rows: Vec<(String, Rgb)> = Vec::new();
    for (theirs, line) in &cv.lines {
        let col = if *theirs { TEXT } else { GOLD };
        let mut cur = String::new();
        for word in line.split(' ') {
            let next = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
            if c.width(&next, 15.0) > width && !cur.is_empty() {
                rows.push((cur, col));
                cur = word.to_string();
            } else {
                cur = next;
            }
        }
        rows.push((cur, col));
        rows.push((String::new(), col));
    }
    let max = ((r.h - 70.0) / 19.0) as usize;
    let start = rows.len().saturating_sub(max);
    let mut y = r.y + 60.0;
    for (line, col) in &rows[start..] {
        c.text(line, x, y, 15.0, *col);
        y += 19.0;
    }
    c.text("Esc to leave", x, r.y + r.h - 10.0, 12.0, DIM);
    (chosen, Some(r))
}

fn journal_rect(c: &Canvas, w: &World) -> Bx {
    let n = w.quests.len().max(1) as f32;
    Bx::new(12.0, c.h - 30.0 - CARD_H - 30.0 - (60.0 + n * 22.0), 620.0, 50.0 + n * 22.0)
}

/// Jobs taken on, and what each needs next.
pub fn journal(c: &Canvas, w: &World) -> Bx {
    let r = journal_rect(c, w);
    c.rect(r.x, r.y, r.w, r.h, PANEL);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    c.text("Journal", r.x + 14.0, r.y + 26.0, 17.0, GOLD);
    if w.quests.is_empty() {
        c.text("No jobs yet. Ask people if they have any work.", r.x + 14.0, r.y + 48.0, 14.0, DIM);
        return r;
    }
    for (i, q) in w.quests.iter().enumerate() {
        let col = match q.stage {
            Stage::Done => DIM,
            Stage::Report => GOLD,
            _ => TEXT,
        };
        c.text(&w.quest_line(q), r.x + 14.0, r.y + 48.0 + i as f32 * 22.0, 14.0, col);
    }
    r
}
