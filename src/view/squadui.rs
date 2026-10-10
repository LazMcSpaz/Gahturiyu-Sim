//! Your squad in the window: a card per member along the bottom (click to
//! select, right-click for their pack), the pack, crafting, conversation and
//! journal panels.
//!
//! Works immediate-mode: each frame the panels are drawn from the world as it
//! is, and a click on them comes back as an `Action` for the app to carry out.

use bevy::math::Vec2;

use gahturiyu_sim::sim::{
    body,
    condition::Shelter,
    crafting::{success_chance, Cannot, RECIPES},
    materials::{Craft, Grade, CRAFTS},
    dialogue::Topic,
    inventory,
    items::{self, item, ItemId, Kind, Slot, SLOTS},
    magic::{Aim, Place, Spell, Style},
    person::PersonId,
    quests::Stage,
    stats::Skill,
    World,
};

use super::hud::Canvas;
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
    /// Put on the pack's `k`th entry.
    EquipEntry(PersonId, usize),
    Unequip(PersonId, Slot),
    /// Put down this very entry in the pack.
    DropEntry(PersonId, usize),
    /// Hand this entry to another squad member.
    GiveEntry(PersonId, usize, PersonId),
    /// Seal a worn reed piece with pitch, or mend it yourself.
    Care(PersonId, Slot),
    Use(PersonId, ItemId),
    Craft(PersonId, usize),
    CloseInventory,
    /// Use a spell from the book (cast, perform, or release).
    Spell(PersonId, Spell),
    CloseBook,
}

/// A word in someone's own tongue, in what they say.
pub const NATIVE: Rgb = [0.62, 0.85, 0.80];

/// Violet, for held rituals and other lingering magic.
pub const RITUAL: Rgb = [0.78, 0.6, 1.0];

/// A click this frame: where, which button, and whether Shift was down.
#[derive(Clone, Copy)]
pub struct Click {
    pub at: Vec2,
    pub right: bool,
    pub shift: bool,
}

const INV_W: f32 = 400.0;
const ROW: f32 = 21.0;

fn inv_rect(c: &Canvas, w: &World, pid: PersonId) -> Bx {
    let bag = w.people[pid as usize].detail.as_ref().map(|d| d.gear.bag.len()).unwrap_or(0);
    let h = 70.0 + (SLOTS.len() as f32 + 1.0) * ROW + (bag.max(1) as f32 + 1.0) * ROW + 40.0;
    Bx::new(c.w - INV_W - 12.0, 12.0, INV_W, h.min(c.h - 130.0))
}

pub fn status(w: &World, pid: PersonId, k: usize) -> (&'static str, Rgb) {
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
    if !w.free_to_order(pid) {
        return ("Bound to work", WARN);
    }
    if w.chased_by(pid).is_some() {
        return ("The watch is after them", WARN);
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
            Shelter::Bed => "Asleep (a bed at the inn)",
        };
        return (place, SNEAK);
    }
    if w.crafting.iter().any(|j| j.who == pid) {
        return ("Crafting", GOLD);
    }
    if w.ritual_progress(pid).is_some() {
        return ("Performing a ritual", RITUAL);
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
    if let Some(d) = w.labouring(pid).and_then(|id| w.deposit(id)) {
        return (if items::item(d.item).key == "timber" { "Chopping wood" } else { "Mining" }, GOLD);
    }
    if w.labour.iter().any(|l| l.who == pid) {
        return ("Off to work", TEXT);
    }
    if w.butchering_now(pid) {
        return ("Butchering", GOLD);
    }
    if w.chases.iter().any(|c| c.who.contains(&pid)) {
        return ("Hunting", WARN);
    }
    if w.looting_now(pid).is_some() {
        return ("Looting", GOLD);
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

const BOOK_COL: f32 = 340.0;
const BOOK_ROW: f32 = 19.0;

/// The spell book: everything someone knows, by style, each with its domain
/// and what it costs. Felt spells on the left; structured and ritual on the
/// right. Click one to use it. Returns what was clicked, the spell under the
/// mouse, and the panel's box.
pub fn spell_book(c: &Canvas, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<Spell>, Bx) {
    let p = &w.people[pid as usize];
    let known = w.known_spells(pid);
    let st = p.effective_stats();
    let of = |style: Style| known.iter().copied().filter(|s| s.def().style == style).collect::<Vec<Spell>>();
    let (felt, structured, ritual) = (of(Style::Felt), of(Style::Structured), of(Style::Ritual));
    let left = felt.len().max(1) + 1;
    let right = structured.len().max(1) + ritual.len().max(1) + 3;
    let rows = left.max(right);
    let r = Bx::new(c.w - BOOK_COL * 2.0 - 12.0, 12.0, BOOK_COL * 2.0, (82.0 + rows as f32 * BOOK_ROW).min(c.h - 140.0));
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 4.0, eg(RITUAL));
    let mut act = None;
    let mut hovered = None;
    let clicked = |rect: Bx| click.filter(|k| rect.contains(k.at) && !k.right);
    let x = r.x + 14.0;
    let y0 = r.y + 26.0;
    c.text(&format!("{}  ·  spell book", p.name().unwrap_or("?")), x, y0, 17.0, race_color(p.race));
    let close = Bx::new(r.x + r.w - 26.0, r.y + 8.0, 18.0, 18.0);
    c.text("×", close.x + 3.0, close.y + 15.0, 18.0, if close.contains(mouse) { GOLD } else { DIM });
    if clicked(close).is_some() {
        act = Some(Action::CloseBook);
    }
    let energy = match w.fighter(pid) {
        Some(f) => f.mana,
        None => p.mana_at(w.time),
    };
    c.text(&format!("Energy {:.0} / {:.0}", energy, p.max_mana()), x, y0 + 20.0, 13.0, MANA);
    let held = match w.fighter(pid) {
        Some(f) => f.held,
        None => w.held_ritual(pid),
    };
    let banner = if let Some(s) = held {
        Some(format!("Holding {} — click it to let it go", s.def().name))
    } else {
        w.ritual_progress(pid).map(|(s, f)| format!("Performing {}  {:.0}%", s.def().name, f * 100.0))
    };
    if let Some(t) = banner {
        c.text(&t, r.x + r.w - c.width(&t, 13.0) - 30.0, y0 + 20.0, 13.0, RITUAL);
    }
    let bottom = r.y + r.h - 8.0;
    // One column of styles.
    let mut column = |col_x: f32, styles: &[(Style, &Vec<Spell>)]| {
        let mut y = y0 + 30.0;
        for &(style, mine) in styles {
            y += BOOK_ROW + 4.0;
            if y > bottom {
                break;
            }
            let head = format!("{}  ·  {:.0}", style.name(), st.skill(style.skill()));
            c.text(&head, col_x + 10.0, y, 15.0, GOLD);
            let how = match style {
                Style::Felt => "instant · comes with use",
                Style::Structured => "1–2 s · a hit spoils it",
                Style::Ritual => "performed, then held",
            };
            c.text(how, col_x + BOOK_COL - c.width(how, 11.0) - 14.0, y, 11.0, DIM);
            if mine.is_empty() {
                y += BOOK_ROW;
                c.text("— none yet —", col_x + 20.0, y, 13.0, DIM);
                continue;
            }
            for &s in mine.iter() {
                y += BOOK_ROW;
                if y > bottom {
                    break;
                }
                let d = s.def();
                let row = Bx::new(col_x + 6.0, y - 14.0, BOOK_COL - 12.0, BOOK_ROW);
                let hot = row.contains(mouse);
                if hot {
                    c.rect(row.x, row.y, row.w, row.h, ega(RITUAL, 0.14));
                    hovered = Some(s);
                }
                let is_held = held == Some(s);
                c.text(d.name, col_x + 20.0, y, 13.5, if is_held { RITUAL } else if hot { GOLD } else { TEXT });
                c.text(d.domain.name(), col_x + 140.0, y, 11.5, domain_color(d.domain));
                let cost = match style {
                    Style::Felt => format!("{:.0} energy", d.cost),
                    Style::Structured => format!("{:.0} energy · {:.1} s", d.cost, d.cast_time),
                    Style::Ritual if is_held => "release".to_string(),
                    Style::Ritual => {
                        let place = if d.rite.place == Place::Anywhere { String::new() } else { format!(" · {}", place_word(d.rite.place)) };
                        let blood = if d.rite.health > 0.0 { " · blood" } else { "" };
                        format!("{:.0} min{place}{blood}", d.rite.minutes)
                    }
                };
                c.text(&cost, col_x + BOOK_COL - c.width(&cost, 11.5) - 14.0, y, 11.5, DIM);
                if clicked(row).is_some() {
                    act = Some(Action::Spell(pid, s));
                }
            }
        }
    };
    column(r.x, &[(Style::Felt, &felt)]);
    column(r.x + BOOK_COL, &[(Style::Structured, &structured), (Style::Ritual, &ritual)]);
    (act, hovered, r)
}

fn place_word(p: Place) -> &'static str {
    match p {
        Place::Anywhere => "",
        Place::Hearth => "hearth",
        Place::Shrine => "shrine",
        Place::Circle => "circle",
    }
}

/// A colour per domain, for the tag in the spell book.
pub fn domain_color(d: gahturiyu_sim::sim::magic::Domain) -> Rgb {
    use gahturiyu_sim::sim::magic::Domain::*;
    match d {
        Elemental => [1.0, 0.55, 0.3],
        Psychic => [0.85, 0.55, 0.95],
        Illusion => [0.6, 0.75, 1.0],
        Vital => [0.5, 0.9, 0.55],
        Warding => [0.95, 0.85, 0.45],
        Alteration => [0.75, 0.7, 0.6],
        Summoning => [0.45, 0.9, 0.9],
        Necromancy => [0.6, 0.8, 0.45],
    }
}

/// Lines for a spell's tooltip.
pub fn spell_lines(s: Spell) -> Vec<(String, Rgb)> {
    let d = s.def();
    let mut out = vec![(d.name.to_string(), GOLD), (format!("{} · {}", d.style.name(), d.domain.name()), domain_color(d.domain))];
    let aim = match d.aim {
        Aim::Caster => "on yourself".to_string(),
        Aim::Foe => format!("at an enemy within {:.0} m", d.range),
        Aim::Friend => format!("at a friend within {:.0} m", d.range),
        Aim::Point => format!("at a spot within {:.0} m", d.range),
        Aim::Anyone => format!("at anyone within {:.0} m", d.range),
        Aim::Door => "at a door".to_string(),
        Aim::Corpse => format!("at a body within {:.0} m", d.range),
    };
    out.push((format!("Cast {aim}"), TEXT));
    for e in d.effects {
        out.push((e.describe(), [0.65, 0.78, 1.0]));
    }
    match d.style {
        Style::Felt => out.push((format!("{:.0} energy and a little tiredness · rarely fails", d.cost), DIM)),
        Style::Structured => out.push((format!("{:.0} energy · {:.1} s to cast · can fizzle", d.cost, d.cast_time), DIM)),
        Style::Ritual => {
            let mut parts = vec![format!("{:.0} minutes", d.rite.minutes)];
            if d.rite.place != Place::Anywhere {
                parts.push(d.rite.place.name().to_string());
            }
            if d.rite.health > 0.0 {
                parts.push(format!("{:.0} health", d.rite.health));
            }
            for &(k, n) in d.rite.components {
                parts.push(format!("{n} × {}", item(items::id(k)).name.to_lowercase()));
            }
            out.push((parts.join(" · "), DIM));
            out.push(("Held ready when done; let it go any time, even mid-fight. Lost if you sleep.".to_string(), DIM));
        }
    }
    out
}

/// One person's gear: worn on the left of each row, the pack below.
/// Returns what was clicked, the item under the mouse, and the panel's box.
pub fn inventory(c: &Canvas, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<ItemId>, Option<Bx>) {
    let p = &w.people[pid as usize];
    let Some(d) = p.detail.as_ref() else { return (None, None, None) };
    let r = inv_rect(c, w, pid);
    c.frame_box(r.x, r.y, r.w, r.h);
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
        if let (Some(i), Some(pc)) = (it, gear.piece(s)) {
            name += &wear_word(w, i, pc);
        }
        if it.map(|i| matches!(item(i).kind, Kind::Torch(_))).unwrap_or(false) {
            if let Some(h) = w.torch_hours_left(pid) {
                name = format!("{name} ({}, {:.1} h)", if w.torch_lit(pid) { "lit" } else { "out" }, h);
            }
        }
        c.text(&name, x + 110.0, y, 14.0, if it.is_some() { TEXT } else { DIM });
        if let Some(i) = it {
            let kg = format!("{:.1} kg", item(i).weight);
            c.text(&kg, r.x + r.w - c.width(&kg, 13.0) - 14.0, y, 13.0, DIM);
            if let Some(ck) = clicked(rr).filter(|_| !locked) {
                act = Some(if ck.right { Action::Care(pid, s) } else { Action::Unequip(pid, s) });
            }
        }
    }
    y += ROW + 4.0;
    c.text("Pack", x, y, 15.0, TEXT);
    if gear.bag.is_empty() {
        y += ROW;
        c.text("empty", x + 8.0, y, 14.0, DIM);
    }
    // Shift-click hands a thing to the nearest squadmate.
    let here = w.person_pos(pid);
    let mate = w.squad.members.iter().copied().filter(|&m| m != pid).min_by(|&a, &b| w.person_pos(a).dist(here).total_cmp(&w.person_pos(b).dist(here)));
    for (k, e) in gear.bag.iter().enumerate() {
        let (i, n) = (e.0, e.1);
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
        let worn = e.2.map(|pc| wear_word(w, i, &pc)).unwrap_or_default();
        let label = if n > 1 { format!("{}  ×{n}", item(i).name) } else { format!("{}{worn}", item(i).name) };
        c.text(&label, x + 8.0, y, 14.0, TEXT);
        let kg = format!("{:.1} kg", item(i).weight * n as f32);
        c.text(&kg, r.x + r.w - c.width(&kg, 13.0) - 14.0, y, 13.0, DIM);
        if let Some(ck) = clicked(rr) {
            if ck.shift && !ck.right && !locked && mate.is_some() {
                act = Some(Action::GiveEntry(pid, k, mate.unwrap()));
            } else if ck.right {
                act = Some(Action::DropEntry(pid, k));
            } else if !locked && items::equippable(i) {
                act = Some(Action::EquipEntry(pid, k));
            } else if !locked && matches!(item(i).kind, Kind::Potion | Kind::Scroll(_) | Kind::Food(_) | Kind::StandingTorch(_) | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_)) {
                act = Some(Action::Use(pid, i));
            }
        }
    }
    let give = mate.map(|m| format!("  ·  Shift-click: give to {}", w.people[m as usize].name().unwrap_or("?"))).unwrap_or_default();
    let hint = if locked { "In a fight: gear can't be changed until it's over.".to_string() } else { format!("Click: take off / put on / use  ·  Right-click: drop, or seal/mend what's worn{give}") };
    c.text(&hint, x, r.y + r.h - 12.0, 13.0, if locked { WARN } else { DIM });
    (act, hovered, Some(r))
}

/// "  (63%)": how worn a made piece is, if it is.
pub fn wear_word(w: &World, id: ItemId, pc: &gahturiyu_sim::sim::materials::Piece) -> String {
    let most = items::max_durability(id);
    if most <= 0.0 {
        return String::new();
    }
    let left = pc.left_at(items::info(id).main.def().rots, w.time);
    let mark = if pc.mark.map(|m| m.stamped).unwrap_or(false) { "  ◆" } else { "" };
    let rot = if items::info(id).main.def().rots && !pc.sealed { ", unsealed" } else { "" };
    format!("  ({:.0}%{rot}){mark}", (left / most * 100.0).clamp(0.0, 100.0))
}

/// A few lines describing an item.
pub fn item_lines(id: ItemId) -> Vec<(String, Rgb)> {
    let d = item(id);
    let mut out = vec![(d.name.to_string(), GOLD)];
    let made = items::info(id);
    let most = items::max_durability(id);
    if most > 0.0 {
        let mend = if !items::repairable(id) { "can't be mended" } else { items::craft_of(id).name() };
        out.push((format!("{} ({}){}  ·  {} grade  ·  lasts {:.0} blows  ·  {mend}", made.main.name(), made.main.def().tradition.name(), if made.second != gahturiyu_sim::sim::materials::Material::None { format!(" on {}", made.second.name()) } else { String::new() }, made.grade.name(), most), DIM));
    }
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
        Kind::Manual(s) => out.push((format!("A manual on {}  ·  click to read and take up the craft", s.name().to_lowercase()), TEXT)),
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
        Kind::Scroll(_) | Kind::Errand | Kind::Notes(_) | Kind::Text(_) | Kind::Manual(_) => [0.9, 0.86, 0.7],
        Kind::Material => [0.55, 0.62, 0.45],
        Kind::Tool => [0.5, 0.5, 0.55],
        Kind::Ammo => [0.6, 0.55, 0.45],
        Kind::Food(_) => [0.75, 0.55, 0.35],
        Kind::Torch(_) | Kind::StandingTorch(_) => [0.45, 0.32, 0.2],
    }
}

fn craft_rect(c: &Canvas, rows: usize) -> Bx {
    let rows = rows as f32 + 4.0 + 3.0;
    Bx::new(c.w - 640.0 - 12.0, 12.0, 640.0, (90.0 + rows * ROW).min(c.h - 130.0))
}

/// What a person can make: every recipe of the crafts they've taken up,
/// with what's missing.
pub fn crafting(c: &Canvas, w: &World, pid: PersonId, mouse: Vec2, click: Option<Click>) -> (Option<Action>, Option<ItemId>, Bx) {
    let p = &w.people[pid as usize];
    let shown: Vec<usize> = (0..RECIPES.len()).filter(|&i| w.knows_craft(pid, RECIPES[i].craft())).collect();
    let crafts = shown.iter().map(|&i| RECIPES[i].skill).collect::<std::collections::BTreeSet<_>>().len();
    let r = craft_rect(c, shown.len() + crafts + 1);
    c.frame_box(r.x, r.y, r.w, r.h);
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
    let (known, unknown): (Vec<Craft>, Vec<Craft>) = CRAFTS.iter().copied().partition(|&k| w.knows_craft(pid, k));
    let known_line = known.iter().map(|k| format!("{} {:.0}", k.skill().name(), st.skill(k.skill()))).collect::<Vec<_>>().join("  ·  ");
    c.text(if known.is_empty() { "No crafts taken up yet." } else { &known_line }, x, y, 14.0, TEXT);
    y += 18.0;
    if !unknown.is_empty() {
        c.text("Not yet taken up (learn from a crafter at work, or a manual):", x, y, 13.0, DIM);
        y += 16.0;
        c.text(&unknown.iter().map(|k| k.skill().name()).collect::<Vec<_>>().join(", "), x + 8.0, y, 13.0, DIM);
    }
    if let Some((k, left)) = w.lesson_progress(pid) {
        y += 18.0;
        c.text(&format!("Learning {} — {:.1} h to go", k.name(), left), x, y, 13.0, GOLD);
    }
    let mut last_skill = None;
    for &i in &shown {
        let rc = &RECIPES[i];
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
        let out = rc.item(Grade::Common);
        let ok = w.can_craft(pid, i);
        if row.contains(mouse) {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.12));
            hovered = Some(out);
        }
        let col = if ok.is_ok() { TEXT } else { DIM };
        let name = if rc.makes > 1 { format!("{} ×{}", item(out).name, rc.makes) } else { item(out).name.to_string() };
        c.text(&name, x + 8.0, y, 14.0, col);
        let need: Vec<String> = rc.inputs.iter().map(|(k, n)| format!("{}/{} {}", w.count_of(pid, k).min(*n), n, item(items::id(k)).name.to_lowercase())).collect();
        c.text(&need.join(", "), x + 230.0, y, 13.0, col);
        let why = match ok {
            Ok(()) if rc.skill == Skill::Tending => format!("{:.0}% · {:.0} days", success_chance(st.skill(rc.skill), rc.difficulty) * 100.0, rc.time / gahturiyu_sim::sim::world::DAY),
            Ok(()) => format!("{:.0}%", success_chance(st.skill(rc.skill), rc.difficulty) * 100.0),
            Err(Cannot::NoStation(s)) => format!("at the {}", s.name().to_lowercase()),
            Err(Cannot::Missing(..)) => String::new(),
            Err(Cannot::Busy) => "busy".into(),
            Err(Cannot::Unknown(_)) => "learn first".into(),
        };
        c.text(&why, r.x + r.w - c.width(&why, 13.0) - 14.0, y, 13.0, if ok.is_ok() { GOLD } else { WARN });
        if ok.is_ok() && click.map(|k| row.contains(k.at) && !k.right).unwrap_or(false) {
            act = Some(Action::Craft(pid, i));
        }
    }
    c.text("Click a recipe to make it. Stone grows only if you're by its bed each dawn.", x, r.y + r.h - 12.0, 13.0, DIM);
    (act, hovered, r)
}

fn talk_rect(c: &Canvas, topics: usize, n: usize) -> Bx {
    let w = 760.0f32.min(c.w - 24.0);
    let _ = n;
    let up = super::frame::BOTTOM_CLEAR - 50.0;
    let h = (80.0 + topics as f32 * 22.0).max(380.0).min(c.h - up - 80.0);
    Bx::new((c.w - w) / 2.0, c.h - 30.0 - up - 20.0 - h, w, h)
}

/// The conversation: what's been said on the left, topics to ask on the right.
pub fn talk(c: &Canvas, w: &World, mouse: Vec2, click: Option<Click>) -> (Option<Topic>, Option<Bx>) {
    let Some(cv) = w.talk.as_ref() else { return (None, None) };
    let topics = w.topics();
    let r = talk_rect(c, topics.len(), w.squad.members.len());
    let npc = &w.people[cv.npc as usize];
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 4.0, eg(race_color(npc.race)));
    let x = r.x + 16.0;
    let disp = w.regard_of(cv.npc, cv.with);
    let job = w.life(cv.npc).job;
    let what = if job == gahturiyu_sim::sim::jobs::Job::None { npc.stats.calling.name().to_string() } else if super::lexicon::native() { super::lexicon::thing(w, job.name()) } else { job.title(npc.seed).to_lowercase() };
    c.text(&format!("{}  ·  {} {}", npc.name().unwrap_or("?"), npc.race.name(), what), x, r.y + 28.0, 18.0, race_color(npc.race));
    let d = format!("Disposition {disp:.0}");
    c.text(&d, r.x + r.w - c.width(&d, 14.0) - 16.0, r.y + 26.0, 14.0, if disp < 30.0 { WARN } else { DIM });

    // Trade lines are longer than questions ("Sell all 40 × iron ore — 44
    // coin"), so the column widens while their wares are out.
    let col = if cv.trading { 330.0 } else { 200.0 };
    let tx = r.x + r.w - col;
    let mut ty = r.y + 60.0;
    let mut chosen = None;
    for t in topics {
        let row = Bx::new(tx - 6.0, ty - 15.0, col - 10.0, 21.0);
        let hot = row.contains(mouse);
        if hot {
            c.rect(row.x, row.y, row.w, row.h, ega(GOLD, 0.15));
        }
        let label = w.topic_text(t);
        let fs = (15.0 * (col - 14.0) / c.width(&label, 15.0).max(1.0)).clamp(10.0, 15.0);
        c.text(&label, tx, ty, fs, if hot { GOLD } else { TEXT });
        if click.map(|k| row.contains(k.at) && !k.right).unwrap_or(false) {
            chosen = Some(t);
        }
        ty += 22.0;
    }

    let width = tx - x - 24.0;
    // Words wrapped into rows; a word in the speaker's own tongue keeps its
    // meaning, to be shown when the mouse is over it.
    type Word = (String, Option<String>);
    let mut rows: Vec<(Vec<Word>, Rgb)> = Vec::new();
    let space = c.width(" ", 15.0);
    for (theirs, line) in &cv.lines {
        let col = if *theirs { TEXT } else { GOLD };
        let mut words: Vec<Word> = Vec::new();
        for (text, meaning) in gahturiyu_sim::sim::speech::spans(line) {
            match meaning {
                // A native word may carry punctuation after it in the next run.
                Some(m) => words.push((text, Some(m))),
                None => {
                    let mut first = true;
                    for piece in text.split(' ') {
                        // Punctuation straight after a native word sticks to it.
                        if first && !piece.is_empty() && !text.starts_with(' ') {
                            if let Some(last) = words.last_mut() {
                                last.0.push_str(piece);
                                first = false;
                                continue;
                            }
                        }
                        first = false;
                        if !piece.is_empty() {
                            words.push((piece.to_string(), None));
                        }
                    }
                }
            }
        }
        let mut row: Vec<Word> = Vec::new();
        let mut used = 0.0;
        for wd in words {
            let ww = c.width(&wd.0, 15.0);
            if used + ww > width && !row.is_empty() {
                rows.push((std::mem::take(&mut row), col));
                used = 0.0;
            }
            used += ww + space;
            row.push(wd);
        }
        rows.push((row, col));
        rows.push((Vec::new(), col));
    }
    let max = ((r.h - 70.0) / 19.0) as usize;
    let start = rows.len().saturating_sub(max);
    let mut y = r.y + 60.0;
    let mut gloss: Option<(String, f32, f32)> = None;
    for (row, col) in &rows[start..] {
        let mut wx = x;
        for (word, meaning) in row {
            let ww = c.width(word, 15.0);
            match meaning {
                Some(m) => {
                    c.text(word, wx, y, 15.0, NATIVE);
                    // A faint dotted line under it: there's more to see.
                    let mut ux = wx;
                    while ux < wx + ww {
                        c.rect(ux, y + 3.0, 2.0, 1.0, ega(NATIVE, 0.7));
                        ux += 4.0;
                    }
                    if Bx::new(wx, y - 15.0, ww, 19.0).contains(mouse) {
                        gloss = Some((format!("{}: \u{201c}{m}\u{201d} in {}", word.trim_end_matches(|ch: char| !ch.is_alphanumeric() && ch != '\u{2bb}'), gahturiyu_sim::names::Tongue::from(npc.race).name()), wx, y));
                    }
                }
                None => c.text(word, wx, y, 15.0, *col),
            }
            wx += ww + space;
        }
        y += 19.0;
    }
    // The meaning of a word under the mouse, along the bottom of the panel.
    if let Some((text, _, _)) = gloss {
        c.text(&text, x + 110.0, r.y + r.h - 10.0, 14.0, NATIVE);
    }
    c.text("Esc to leave", x, r.y + r.h - 10.0, 12.0, DIM);
    (chosen, Some(r))
}

fn journal_rect(c: &Canvas, w: &World) -> Bx {
    let n = (w.quests.len() + w.work_lines().len()).max(1) as f32;
    let up = super::frame::BOTTOM_CLEAR - 30.0;
    let wide = if w.work_lines().is_empty() { 620.0 } else { 900.0f32.min(c.w - 24.0) };
    Bx::new(12.0, c.h - 30.0 - up - 30.0 - (60.0 + n * 22.0), wide, 50.0 + n * 22.0)
}

/// Jobs taken on, and what each needs next.
pub fn journal(c: &Canvas, w: &World) -> Bx {
    let r = journal_rect(c, w);
    c.frame_box(r.x, r.y, r.w, r.h);
    c.rect(r.x, r.y, r.w, 4.0, eg(GOLD));
    c.text("Journal", r.x + 14.0, r.y + 26.0, 17.0, GOLD);
    let work = w.work_lines();
    if w.quests.is_empty() && work.is_empty() {
        c.text("No jobs yet. Ask people if they have any work.", r.x + 14.0, r.y + 48.0, 14.0, DIM);
        return r;
    }
    // Town work first (who, where, the wage), then jobs.
    for (i, l) in work.iter().enumerate() {
        c.text(l, r.x + 14.0, r.y + 48.0 + i as f32 * 22.0, 13.0, GOLD);
    }
    for (i, q) in w.quests.iter().enumerate() {
        let col = match q.stage {
            Stage::Done => DIM,
            Stage::Report => GOLD,
            _ => TEXT,
        };
        c.text(&w.quest_line(q), r.x + 14.0, r.y + 48.0 + (i + work.len()) as f32 * 22.0, 14.0, col);
    }
    r
}
