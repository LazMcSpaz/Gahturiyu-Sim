//! The squad's behaviours (Laz, 2 Nov 2026; `docs/third-person.md`): for
//! each member, how ready to fight, how they fight, how close they follow
//! (or a spot to hold), when to heal, and whether to bind wounds and use
//! scrolls and bottles (`sim/behaviour.rs`). B opens it in third person.

use bevy::math::Vec2;

use gahturiyu_sim::sim::behaviour::{FOLLOWS, HEALS, ROLES, STANCES};
use gahturiyu_sim::sim::person::PersonId;
use gahturiyu_sim::sim::world::World;

use super::hud::{Canvas, Face};
use super::kit::{self, Kind};
use super::palette::{eg, DIM, GOLD, TEXT};
use super::squadui::{Bx, Click};

const W: f32 = 620.0;
const ROW: f32 = 34.0;
const LABEL: f32 = 110.0;

/// Draw the panel; a click on a choice changes it. Its box, and whether
/// Close was clicked.
pub fn draw(c: &Canvas, w: &mut World, lead: Option<PersonId>, click: Option<Click>, size: Vec2) -> (Bx, bool) {
    let members = w.squad.members.clone();
    let per = 30.0 + 5.0 * ROW + 14.0;
    let h = 76.0 + members.len() as f32 * per + 52.0;
    let (x, y) = ((size.x - W) / 2.0, ((size.y - h) / 2.0).max(10.0));
    c.frame_box(x, y, W, h);
    let title = "Behaviours";
    let tw = c.styled_width(title, 22.0, Face::Title, 2.0);
    c.styled(title, x + (W - tw) / 2.0, y + 36.0, 22.0, eg(GOLD), Face::Title, 2.0);
    c.text("What each does when you aren't controlling them", x + 30.0, y + 58.0, 13.0, DIM);
    let hit = |b: &Bx| click.is_some_and(|k| b.contains(k.at));
    let mut by = y + 76.0;
    for m in members {
        let name = w.people[m as usize].name().unwrap_or("someone").to_string();
        let you = if Some(m) == lead { "  (you control them now)" } else { "" };
        c.text(&format!("{name}{you}"), x + 30.0, by + 20.0, 16.0, TEXT);
        by += 30.0;
        let mut b = w.behaviour(m);
        let before = b;
        // One row: its label, then a button per choice (the chosen one gold).
        let row = |label: &str, n: usize, chosen: &dyn Fn(usize) -> bool, names: &dyn Fn(usize) -> String, by: f32| -> Option<usize> {
            c.text(label, x + 30.0, by + 22.0, 14.0, DIM);
            let bw = (W - 30.0 - LABEL - 20.0) / n as f32;
            let mut picked = None;
            for k in 0..n {
                let bx = Bx::new(x + 30.0 + LABEL + k as f32 * bw, by + 2.0, bw - 6.0, ROW - 6.0);
                kit::button(c, &bx, &names(k), if chosen(k) { Kind::Primary } else { Kind::Plain }, true);
                if hit(&bx) {
                    picked = Some(k);
                }
            }
            picked
        };
        if let Some(k) = row("Stance", STANCES.len(), &|k| STANCES[k].0 == b.stance, &|k| STANCES[k].1.to_string(), by) {
            b.stance = STANCES[k].0;
        }
        by += ROW;
        if let Some(k) = row("Fights", ROLES.len(), &|k| ROLES[k].0 == b.role, &|k| ROLES[k].1.to_string(), by) {
            b.role = ROLES[k].0;
        }
        by += ROW;
        if let Some(k) = row("Follow", FOLLOWS.len(), &|k| FOLLOWS[k].0 == b.follow, &|k| FOLLOWS[k].1.to_string(), by) {
            b.follow = FOLLOWS[k].0;
        }
        by += ROW;
        if let Some(k) = row("Heal", HEALS.len(), &|k| HEALS[k].0 == b.heal_below, &|k| HEALS[k].1.to_string(), by) {
            b.heal_below = HEALS[k].0;
        }
        by += ROW;
        let also = [("Bind wounds", b.bind), ("Use scrolls & bottles", b.items)];
        if let Some(k) = row("Also", 2, &|k| also[k].1, &|k| also[k].0.to_string(), by) {
            if k == 0 {
                b.bind = !b.bind;
            } else {
                b.items = !b.items;
            }
        }
        by += ROW + 14.0;
        if b != before {
            w.set_behaviour(m, b);
        }
    }
    let close = Bx::new(x + W - 150.0, y + h - 46.0, 120.0, 34.0);
    kit::button(c, &close, "Close", Kind::Plain, true);
    c.text("Hold here: they stay where they stand now", x + 30.0, y + h - 24.0, 12.0, DIM);
    (Bx::new(x, y, W, h), hit(&close))
}
