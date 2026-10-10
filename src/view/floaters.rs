//! Floating words over people, as in Baldur's Gate 3: what a squad member
//! picked up, the hurt anyone in the squad's fight took (or healed), a "?" and
//! then a "!" as someone grows suspicious of a squad member, a skill going up.
//!
//! Drawing only: worked out each frame by comparing what the window saw the
//! frame before, so nothing here reaches the world.

use std::collections::HashMap;

use bevy::math::Vec2;
use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{
    combat::SQUAD_SIDE,
    geo::V2,
    items::{item, ItemId},
    person::PersonId,
    World,
};

use super::hud::{Canvas, Face};
use super::palette::{eg, GOLD};

/// How long a word floats, in seconds.
const LIFE: f32 = 1.9;
/// How far it rises over that time, in pixels.
const RISE: f32 = 64.0;
/// A second word over the same head waits this long behind the first.
const STAGGER: f32 = 0.6;
/// Healing smaller than this in one frame is just the day's mending.
const HEAL_SHOWN: f32 = 3.0;
/// Hurt within this many seconds of the last number joins it.
const MERGE: f32 = 0.5;

const HURT: [f32; 3] = [0.95, 0.32, 0.25];
const HEALED: [f32; 3] = [0.45, 0.9, 0.5];
const WARY: [f32; 3] = [0.95, 0.8, 0.3];
const SPOTTED: [f32; 3] = [1.0, 0.25, 0.2];

/// Who or where a word hangs over.
#[derive(Clone, Copy, PartialEq)]
pub enum Over {
    Person(PersonId),
    /// A fighter that isn't a person (a summoned beast): where it stood.
    Spot(V2),
}

pub struct Float {
    pub over: Over,
    pub text: String,
    pub colour: [f32; 3],
    pub big: bool,
    /// Seconds since it appeared; below zero while it waits its turn.
    pub age: f32,
    /// Hurt or healing: blows landing close together add up into one number.
    pub amount: Option<f32>,
    /// Something picked up: more of the same soon after adds to the count.
    pub item: Option<ItemId>,
}

#[derive(Default, Clone)]
struct Seen {
    items: Vec<(ItemId, u32)>,
    hp: f32,
    /// Whether `hp` was read from a fight (the two are compared only like for like).
    fighting: bool,
    suspicion: f32,
    level: Option<(String, u8)>,
}

#[derive(Default)]
pub struct Floaters {
    seen: HashMap<PersonId, Seen>,
    /// Fighters in the squad's fight that aren't squad members: (fight, place in it) → health.
    foes: HashMap<(u32, usize), f32>,
    pub list: Vec<Float>,
    loads: u32,
}

fn pack(w: &World, pid: PersonId) -> Vec<(ItemId, u32)> {
    let mut v: Vec<(ItemId, u32)> = Vec::new();
    if let Some(d) = w.people[pid as usize].detail.as_ref() {
        for e in &d.gear.bag {
            match v.iter_mut().find(|x| x.0 == e.0) {
                Some(x) => x.1 += e.1 as u32,
                None => v.push((e.0, e.1 as u32)),
            }
        }
    }
    v
}

fn health(w: &World, pid: PersonId) -> (f32, bool) {
    if let Some(f) = w.fighter(pid) {
        return (f.hp.iter().sum(), true);
    }
    let p = &w.people[pid as usize];
    (p.wounds.hp_at(&p.stats, w.time).iter().sum(), false)
}

fn whole(x: f32) -> String {
    format!("{:.0}", x.abs().max(1.0))
}

impl Floaters {
    fn add(&mut self, over: Over, text: String, colour: [f32; 3], big: bool) {
        // Queue behind anything still fresh over the same head.
        let waiting = self.list.iter().filter(|f| f.over == over && f.age < STAGGER).map(|f| f.age).fold(f32::INFINITY, f32::min);
        let age = if waiting.is_finite() { waiting - STAGGER } else { 0.0 };
        self.list.push(Float { over, text, colour, big, age, amount: None, item: None });
    }

    fn add_item(&mut self, over: Over, id: ItemId, n: u32) {
        let text = |n: u32| if n > 1 { format!("+{n} {}", item(id).name) } else { format!("+ {}", item(id).name) };
        if let Some(f) = self.list.iter_mut().find(|f| f.over == over && f.item == Some(id) && f.age < LIFE * 0.5) {
            let x = f.amount.unwrap_or(1.0) as u32 + n;
            f.amount = Some(x as f32);
            f.text = text(x);
            f.age = f.age.min(0.0);
            return;
        }
        self.add(over, text(n), GOLD, false);
        if let Some(f) = self.list.last_mut() {
            f.amount = Some(n as f32);
            f.item = Some(id);
        }
    }

    /// Hurt (below zero) or healing: added to a fresh number over the same
    /// head if there is one, so a flurry reads as one figure.
    fn add_amount(&mut self, over: Over, d: f32) {
        let colour = if d < 0.0 { HURT } else { HEALED };
        let text = |x: f32| if x < 0.0 { format!("\u{2212}{}", whole(x)) } else { format!("+{}", whole(x)) };
        if let Some(f) = self.list.iter_mut().find(|f| f.over == over && f.colour == colour && f.item.is_none() && f.amount.is_some() && f.age < MERGE) {
            let x = f.amount.unwrap() + d;
            f.amount = Some(x);
            f.text = text(x);
            f.age = f.age.min(0.0);
            return;
        }
        self.add(over, text(d), colour, true);
        if let Some(f) = self.list.last_mut() {
            f.amount = Some(d);
        }
    }

    /// Compare the world with last frame and float whatever changed.
    pub fn update(&mut self, w: &World, dt: f32, loads: u32) {
        if loads != self.loads {
            // A different world (loaded or swapped): nothing carries over.
            *self = Floaters { loads, ..Default::default() };
        }
        for f in &mut self.list {
            f.age += dt;
        }
        self.list.retain(|f| f.age < LIFE);

        for &m in &w.squad.members {
            if w.people[m as usize].dead {
                continue;
            }
            let (hp, fighting) = health(w, m);
            let now = Seen { items: pack(w, m), hp, fighting, suspicion: w.suspicion_of(m), level: w.fresh_level_up(m).map(|(s, v)| (s.to_string(), v)) };
            let Some(was) = self.seen.insert(m, now.clone()) else { continue };
            let over = Over::Person(m);
            // Picked up: the two biggest gains by name, the rest as a count.
            let mut gains: Vec<(ItemId, u32)> = now
                .items
                .iter()
                .filter_map(|&(id, n)| {
                    let before = was.items.iter().find(|x| x.0 == id).map_or(0, |x| x.1);
                    (n > before).then_some((id, n - before))
                })
                .collect();
            gains.sort_by(|a, b| b.1.cmp(&a.1));
            for &(id, n) in gains.iter().take(2) {
                self.add_item(over, id, n);
            }
            if gains.len() > 2 {
                self.add(over, format!("+{} more things", gains.len() - 2), GOLD, false);
            }
            if now.fighting == was.fighting {
                let d = now.hp - was.hp;
                if d <= -0.5 || d >= HEAL_SHOWN {
                    self.add_amount(over, d);
                }
            }
            if was.suspicion < 1.0 && now.suspicion >= 1.0 {
                self.add(over, "!".into(), SPOTTED, true);
            } else if was.suspicion < 0.5 && now.suspicion >= 0.5 {
                self.add(over, "?".into(), WARY, true);
            }
            if now.level != was.level {
                if let Some((what, v)) = &now.level {
                    self.add(over, format!("{what} rises to {v}"), GOLD, false);
                }
            }
        }
        self.seen.retain(|m, _| w.squad.members.contains(m));

        // Everyone else in the squad's fight: their hurt and healing.
        let Some(b) = w.squad_battle() else {
            self.foes.clear();
            return;
        };
        self.foes.retain(|k, _| k.0 == b.id);
        for (i, f) in b.fighters.iter().enumerate() {
            if f.side == SQUAD_SIDE && w.squad.members.contains(&f.pid) {
                continue;
            }
            let hp: f32 = f.hp.iter().sum();
            let Some(was) = self.foes.insert((b.id, i), hp) else { continue };
            let over = if f.is_person() { Over::Person(f.pid) } else { Over::Spot(f.pos) };
            let d = hp - was;
            if d <= -0.5 || d >= HEAL_SHOWN {
                self.add_amount(over, d);
            }
        }
    }

    /// Draw them; `place` finds the screen point above a head (or spot).
    /// Words over the same head stack: a newer one pushes older ones up.
    pub fn draw(&self, c: &Canvas, w: &World, place: &dyn Fn(V2) -> Option<Vec2>) {
        let mut shown: Vec<&Float> = self.list.iter().filter(|f| f.age >= 0.0).collect();
        shown.sort_by(|a, b| a.age.total_cmp(&b.age));
        // The top of the last word drawn over each head (youngest first).
        let mut tops: Vec<(Over, f32)> = Vec::new();
        for f in shown {
            let at = match f.over {
                Over::Person(p) => w.fighter(p).map(|x| x.pos).unwrap_or_else(|| w.person_pos(p)),
                Over::Spot(p) => p,
            };
            let Some(s) = place(at) else { continue };
            let k = f.age / LIFE;
            let size = if f.big { 26.0 } else { 19.0 };
            // Up quickly, then drifting; fading over the last part.
            let mut y = s.y - RISE * (1.0 - (1.0 - k) * (1.0 - k));
            match tops.iter_mut().find(|t| t.0 == f.over) {
                Some(t) => {
                    y = y.min(t.1 - 4.0);
                    t.1 = y - size * 0.8;
                }
                None => tops.push((f.over, y - size * 0.8)),
            }
            let fade = ((1.0 - k) / 0.4).clamp(0.0, 1.0);
            let face = if f.big { Face::Title } else { Face::Caps };
            let width = c.styled_width(&f.text, size, face, 0.5);
            let x = s.x - width / 2.0;
            c.styled(&f.text, x + 1.5, y + 2.0, size, Color32::from_black_alpha((190.0 * fade) as u8), face, 0.5);
            c.styled(&f.text, x, y, size, eg(f.colour).gamma_multiply(fade), face, 0.5);
        }
    }
}
