//! What's under the mouse, answered before the click: a small name plate
//! over the thing (#159 UI-4; the longer tooltip waits half a second) and a
//! mark beside the pointer saying what a left-click will do (#160 UI-5):
//! walk, talk, attack, go in, take or search, gather or work, and "can't".
//! Red when the act would be a crime.
//!
//! Window code: it reads the same choices `interact.rs` makes (so the mark
//! always tells the truth about the click) and draws.

use bevy::math::{vec2, Vec2};
use bevy_egui::egui::Color32;

use gahturiyu_sim::sim::{geo::V2, person::PersonId, World};

use super::animals::Seen;
use super::app::Hover;
use super::hud::Canvas;
use super::interact::{self, Act};
use super::palette::{ega, BRASS_LIGHT};

/// How long the mouse rests on a thing before its long tooltip shows, seconds.
pub const TIP_AFTER: f32 = 1.5;
/// How far the mouse may wander (px) and still count as resting on a thing.
pub const STILL: f32 = 4.0;

/// What a left-click will do, as the pointer shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Walk,
    Talk,
    Attack,
    Enter,
    Take,
    Work,
    Cant,
}

/// The mark for a click on `hover` (or a wild animal, or bare ground at
/// `ground`) by the selected `who`, and whether it's a crime. None where a
/// click only selects or opens a panel.
pub fn mark_for(w: &World, hover: Option<Hover>, wild: Option<&Seen>, ground: Option<V2>, who: &[PersonId], shift: bool) -> Option<(Mark, bool)> {
    if w.squad.members.is_empty() {
        return None;
    }
    let first = match (hover, wild) {
        // A workplace or a camp is only named; a click walks there.
        (Some(Hover::Place(..)), _) => return Some((Mark::Walk, false)),
        (Some(h), _) => interact::choices(w, h, who, shift).into_iter().find(|c| c.act != Act::Examine),
        (None, Some(s)) => interact::wild_choices(w, s).into_iter().next(),
        (None, None) => {
            let p = ground?;
            // The water: they'd stop at the shore (UI-7).
            return Some((if w.open_water(p) { Mark::Cant } else { Mark::Walk }, false));
        }
    }?;
    let m = match first.act {
        Act::Walk(_) | Act::SneakTo(_) | Act::Sleep(_) | Act::SleepIn(_) => Mark::Walk,
        Act::PackUp(_) | Act::Lay(..) | Act::GatherThrown | Act::DropPack(_) => Mark::Work,
        Act::Talk(_) => Mark::Talk,
        Act::Attack(_) => Mark::Attack,
        Act::Enter(_) | Act::PickLock(_) => Mark::Enter,
        Act::Loot(_) | Act::Carry(_) | Act::Search(_) | Act::PickChest(_) | Act::PickUp(_) | Act::Dose(_) | Act::Bind(_) => Mark::Take,
        Act::Gather(_) | Act::Work(_) | Act::Make(_) | Act::Hunt(_) | Act::Butcher(_) => Mark::Work,
        Act::Select(_) | Act::TownPanel(_) | Act::Examine => return None,
    };
    Some((m, first.crime))
}

/// Draw the mark beside the pointer at `at` (the pointer's tip).
pub fn draw_mark(c: &Canvas, at: Vec2, m: Mark, crime: bool) {
    let red = [0.95, 0.38, 0.30];
    let col = if crime || m == Mark::Cant { red } else { BRASS_LIGHT };
    let ink = ega(col, 0.95);
    // A dark disc behind, so it reads on any ground; below and left of the
    // pointer, clear of the tooltip.
    let o = at + vec2(-16.0, 30.0);
    c.circle(o.x, o.y, 14.0, Color32::from_rgba_unmultiplied(18, 13, 9, 210));
    c.circle_lines(o.x, o.y, 14.0, 1.2, ega(col, 0.6));
    let p = |x: f32, y: f32| o + vec2(x, y);
    match m {
        Mark::Walk => {
            // Two footprints, one a step ahead.
            c.ellipse_lines(o.x - 3.8, o.y + 2.8, 2.8, 4.8, 1.8, ink);
            c.ellipse_lines(o.x + 3.8, o.y - 2.8, 2.8, 4.8, 1.8, ink);
        }
        Mark::Talk => {
            // A speech bubble with its tail.
            c.ellipse_lines(o.x, o.y - 1.5, 7.0, 5.0, 1.6, ink);
            c.line(p(-3.0, 2.5), p(-5.5, 6.5), 1.6, ink);
            c.line(p(-5.5, 6.5), p(0.0, 3.2), 1.6, ink);
        }
        Mark::Attack => {
            // Two crossed blades with their guards.
            c.line(p(-6.0, 6.0), p(6.0, -6.0), 2.0, ink);
            c.line(p(6.0, 6.0), p(-6.0, -6.0), 2.0, ink);
            c.line(p(-6.5, 2.5), p(-2.5, 6.5), 1.6, ink);
            c.line(p(6.5, 2.5), p(2.5, 6.5), 1.6, ink);
        }
        Mark::Enter => {
            // A doorway with a round head.
            c.line(p(-4.5, 7.0), p(-4.5, -1.0), 1.8, ink);
            c.line(p(4.5, 7.0), p(4.5, -1.0), 1.8, ink);
            c.arc(o.x, o.y - 1.0, 4.5, std::f32::consts::PI, std::f32::consts::TAU, 1.8, ink);
            c.line(p(-7.0, 7.0), p(7.0, 7.0), 1.6, ink);
        }
        Mark::Take => {
            // An open bag.
            c.poly_lines(&[p(-6.0, -2.0), p(6.0, -2.0), p(4.5, 7.0), p(-4.5, 7.0)], 1.6, ink, true);
            c.arc(o.x, o.y - 2.0, 3.5, std::f32::consts::PI, std::f32::consts::TAU, 1.6, ink);
        }
        Mark::Work => {
            // A pick: the handle and its curved head.
            c.line(p(-5.5, 7.0), p(3.0, -3.5), 2.0, ink);
            c.arc(o.x + 2.0, o.y + 1.5, 7.0, -2.6, -0.6, 2.0, ink);
        }
        Mark::Cant => {
            // A ring with a slash.
            c.circle_lines(o.x, o.y, 6.5, 1.8, ink);
            c.line(p(-4.6, 4.6), p(4.6, -4.6), 1.8, ink);
        }
    }
}

/// A small plate with just the name, centred over `at` (screen).
pub fn plate(c: &Canvas, at: Vec2, name: &str) {
    let wd = c.width(name, 14.0) + 14.0;
    c.rect(at.x - wd / 2.0, at.y - 21.0, wd, 20.0, Color32::from_rgba_unmultiplied(18, 13, 9, 190));
    c.rule(at.x - wd / 2.0 + 4.0, at.x + wd / 2.0 - 4.0, at.y - 1.0, 1.0, ega(BRASS_LIGHT, 0.5));
    c.centred(name, at.x, at.y - 6.0, 14.0, super::palette::TEXT);
}

/// Where on the ground something hovered stands, and how wide a ring
/// round it should be, metres (the ring underfoot, UI-4).
pub fn spot_of(w: &World, h: Hover) -> Option<(V2, f32)> {
    Some(match h {
        Hover::Person(p) => (w.person_pos(p), 0.95),
        Hover::Group(g) => (w.group(g)?.pos, 1.6),
        Hover::Door(id) => (w.door(id)?.outside, 1.0),
        Hover::Building(id) => {
            let d = w.door(id)?;
            (d.centre, d.radius + 0.6)
        }
        Hover::Container(id) => (w.container(id)?.pos, 0.8),
        Hover::Furniture(id, k) => {
            let d = w.door(id)?;
            let f = d.variant().furniture.get(k as usize)?;
            let (a, b, _) = f.what.size();
            (d.to_world(f.at), (a.max(b) * 0.6).max(0.6))
        }
        Hover::Item(g) => (w.ground.iter().find(|x| x.id == g)?.pos, 0.6),
        Hover::Node(n) => (w.nodes.iter().find(|x| x.id == n)?.pos, 1.2),
        Hover::Deposit(d) => (w.deposit(d)?.pos, 2.5),
        Hover::Station(i) => (w.stations.get(i)?.0, 1.0),
        Hover::Torch(i) => (w.standing.get(i)?.pos, 0.6),
        Hover::Bedding(id) => {
            let b = w.bedding(id)?;
            (b.pos, if b.kind == gahturiyu_sim::sim::bedding::BedKind::Tent { 2.4 } else { 1.0 })
        }
        Hover::Ruin(r) => (w.ruins.get(r as usize)?.pos, 6.0),
        Hover::Place(t, k) => {
            let p = w.society.towns.get(t as usize)?.places.get(k as usize)?;
            (p.pos, p.kind.size() * 0.5)
        }
        Hover::Town(_) => return None,
    })
}
