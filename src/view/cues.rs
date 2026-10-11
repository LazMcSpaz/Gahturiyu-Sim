//! Visual cues for what people are doing (Laz's rule: every action the
//! player orders shows in the 3D view, not only on the HUD). Placeholder
//! shapes, no rigs:
//! - a hand weapon is a stick held forward and down; a swing draws a fast
//!   bright arc in front of the fighter as the blow comes in;
//! - every ranged weapon is the same stick: pointed at the target while
//!   aiming and loosing, pointed at the ground while reloading;
//! - casting raises a glowing mote above the hand; drinking lifts a flask;
//! - building, crafting, gathering, picking a lock and going through a body
//!   or a container bob a tool (a hand) up and down, facing what's worked;
//! - sneaking crouches the figure (drawn in `scene::person`).
//!
//! Drawing only: everything here reads the sim's own states and times.

use bevy::math::{vec3, Vec3};

use gahturiyu_sim::sim::{combat::Act, geo::V2, person::PersonId, World};

use super::mesh::Builder;
use super::palette::Rgb;

const STICK: Rgb = [0.30, 0.24, 0.18];
const BLADE: Rgb = [0.72, 0.74, 0.78];
const ARC: Rgb = [1.0, 0.97, 0.85];
const MOTE: Rgb = [0.65, 0.75, 1.0];
const FLASK: Rgb = [0.85, 0.35, 0.30];

/// How long after a blow lands its arc still shows, seconds.
const ARC_AFTER: f64 = 0.2;
/// The share of a swing's wind-up the arc sweeps through.
pub const ARC_SHARE: f64 = 0.5;

/// Which way someone faces, radians (sim convention): at their target in a
/// fight, along their way if walking, else as they happen to stand.
pub fn facing(w: &World, pid: PersonId, at: V2) -> f32 {
    // Talking: the two face each other.
    if let Some(c) = w.talk.as_ref().filter(|c| c.with == pid || c.npc == pid) {
        let d = w.person_pos(if c.with == pid { c.npc } else { c.with }).sub(at);
        if d.len() > 0.05 {
            return d.y.atan2(d.x);
        }
    }
    if let Some((b, f)) = fight(w, pid) {
        let t = match f.act {
            Act::Swing { target, .. } => Some(target),
            Act::Cast { target, .. } => target,
            _ => f.target,
        };
        if let Some(j) = t.filter(|&j| j < b.fighters.len()) {
            let d = b.fighters[j].pos.sub(at);
            if d.len() > 0.05 {
                return d.y.atan2(d.x);
            }
        }
        if let Act::Cast { point, .. } = f.act {
            let d = point.sub(at);
            if d.len() > 0.05 {
                return d.y.atan2(d.x);
            }
        }
    }
    // Going through a container, or picking its lock: facing it.
    let chest = w.searching_now(pid).or_else(|| w.picking.iter().find(|p| p.who == pid).and_then(|p| p.holder.map(|s| (p.door.0, p.door.1, s))));
    if let Some(c) = chest.and_then(|c| w.container(c)) {
        let d = c.pos.sub(at);
        if d.len() > 0.05 && w.squad.index(pid).is_none_or(|k| w.squad.route[k].is_empty()) {
            return d.y.atan2(d.x);
        }
    }
    if let Some(k) = w.squad.index(pid) {
        let to = w.squad.route[k].first().copied().unwrap_or(w.squad.goal[k]);
        let d = to.sub(w.squad.at[k]);
        if d.len() > 0.3 {
            return d.y.atan2(d.x);
        }
    }
    (w.people[pid as usize].seed % 628) as f32 / 100.0
}

fn fight(w: &World, pid: PersonId) -> Option<(&gahturiyu_sim::sim::combat::Battle, &gahturiyu_sim::sim::combat::Fighter)> {
    let id = *w.fighting.get(&pid)?;
    let b = w.battle(id)?;
    let f = b.fighters.iter().find(|f| f.pid == pid)?;
    Some((b, f))
}

/// Is someone at work with their hands (building, crafting, gathering)?
fn working(w: &World, pid: PersonId) -> bool {
    w.crafting.iter().any(|j| j.who == pid && j.bed.is_none())
        || w.gathering.iter().any(|g| g.0 == pid)
        || w.picking.iter().any(|p| p.who == pid)
        || w.pickups.iter().any(|p| p.who == pid)
        || w.source_now(pid).is_some()
        || w.labouring(pid).is_some()
        || w.butchering_now(pid)
        || w.bases.iter().any(|b| b.builders.iter().any(|h| h.0 == pid))
        || w.bases.iter().any(|b| b.residents.iter().any(|r| r.who == pid && r.cycle.is_some()))
}

/// The cues for one person drawn as a full figure: `base` at their feet,
/// `h` and `r` their height and girth, `face` the way they face.
#[allow(clippy::too_many_arguments)]
pub fn draw(b: &mut Builder, gl: &mut Builder, w: &World, pid: PersonId, base: Vec3, h: f32, r: f32, face: f32, k: f32) {
    let fwd = vec3(face.cos(), 0.0, face.sin());
    let side = vec3(-face.sin(), 0.0, face.cos());
    let hand = base + side * (r * 1.25) + vec3(0.0, h * 0.5, 0.0) + fwd * (r * 0.4);
    // In conversation: a small speech mark over the head.
    if w.talk.as_ref().is_some_and(|c| c.npc == pid || c.with == pid) {
        let top = base + vec3(0.0, h * 1.12, 0.0);
        gl.block(top, 0.22 * k, 0.08 * k, 0.16 * k, face, [0.95, 0.95, 0.9]);
        gl.block(top - vec3(0.0, 0.08 * k, 0.0) + side * (0.05 * k), 0.06 * k, 0.06 * k, 0.08 * k, face, [0.95, 0.95, 0.9]);
    }
    let Some((battle, f)) = fight(w, pid) else {
        // Out of a fight: a tool going up and down while they work.
        if working(w, pid) {
            let bob = ((w.time * 4.0 + pid as f64 * 1.7).sin() * 0.5 + 0.5) as f32;
            let tip = hand + fwd * (0.45 * k) + vec3(0.0, (0.35 - 0.6 * bob) * k, 0.0);
            b.stick(hand, tip, 0.06 * k, STICK);
            b.block(tip - vec3(0.0, 0.06 * k, 0.0), 0.16 * k, 0.1 * k, 0.12 * k, face, BLADE);
        }
        return;
    };
    if f.ko || f.dead || f.fled || !f.is_person() {
        return;
    }
    let t = battle.time;
    let ranged = f.weapon.range > 0.0;
    // Casting: a mote rising above the hand; drinking: a flask at the mouth.
    match f.act {
        Act::Cast { done, .. } => {
            let lift = (1.0 - ((done - t).max(0.0) / 1.5).min(1.0)) as f32;
            gl.dome(hand + vec3(0.0, (0.4 + 0.5 * lift) * k, 0.0), 0.16 * k, 0.16 * k, 0.24 * k, 0.0, 0.0, 2, MOTE);
            return;
        }
        Act::Drink { .. } => {
            let mouth = base + vec3(0.0, h * 0.88, 0.0) + fwd * (r * 0.9);
            b.column(mouth, 0.07 * k, 0.05 * k, 0.18 * k, 5, FLASK);
            return;
        }
        _ => {}
    }
    if ranged {
        // Aiming: pointed at the target at shoulder height. Reloading:
        // pointed at the ground.
        let shoulder = base + side * (r * 1.1) + vec3(0.0, h * 0.72, 0.0);
        let (dir, len) = match f.act {
            Act::Swing { .. } => (fwd, 0.9),
            Act::Recover { .. } => ((fwd * 0.35 + vec3(0.0, -1.0, 0.0)).normalize(), 0.8),
            _ => ((fwd + vec3(0.0, -0.6, 0.0)).normalize(), 0.8),
        };
        b.stick(shoulder - dir * (0.25 * k), shoulder + dir * (len * k), 0.07 * k, STICK);
        // A crossbar, so it reads as a bow of some kind.
        let mid = shoulder + dir * (0.45 * k);
        let bar = side * (0.28 * k);
        b.stick(mid - bar, mid + bar, 0.05 * k, STICK);
        return;
    }
    // A hand weapon: held forward and down; swung, it flashes an arc.
    let reach = (f.weapon.reach * 0.75).clamp(0.6, 2.4);
    if let Act::Swing { lands, .. } = f.act {
        let windup = f.weapon.windup.max(0.2) as f64;
        let start = lands - windup * ARC_SHARE;
        let p = ((t - start) / (windup * ARC_SHARE + ARC_AFTER)).clamp(0.0, 1.0) as f32;
        if t >= start {
            swing_arc(gl, base + vec3(0.0, h * 0.6, 0.0), face, reach + 0.3, p, k);
            let a = face + (1.2 - 2.4 * p);
            let tip = hand + vec3(a.cos(), 0.0, a.sin()) * (reach * k);
            b.stick(hand, tip, 0.07 * k, BLADE);
            return;
        }
        // Drawing back before the blow.
        let a = face + 1.3;
        let tip = hand + vec3(a.cos(), 0.35, a.sin()) * (reach * 0.8 * k);
        b.stick(hand, tip, 0.07 * k, BLADE);
        return;
    }
    if let Act::Recover { until } = f.act {
        // Just after a blow: the arc lingers a moment.
        let since = f.weapon.recover as f64 - (until - t);
        if since < ARC_AFTER {
            swing_arc(gl, base + vec3(0.0, h * 0.6, 0.0), face, reach + 0.3, 1.0, k);
        }
    }
    let tip = hand + (fwd + vec3(0.0, -0.45, 0.0)).normalize() * (reach * k);
    b.stick(hand, tip, 0.07 * k, BLADE);
}

/// A swept arc in front of a fighter, from their right to their left,
/// drawn up to `p` (0..1) of the way, brightest at its leading edge.
fn swing_arc(gl: &mut Builder, centre: Vec3, face: f32, radius: f32, p: f32, k: f32) {
    let steps = 12;
    let (a0, a1) = (face + 1.2, face - 1.2);
    let end = (steps as f32 * p).ceil() as usize;
    let tail = 9;
    for i in end.saturating_sub(tail)..end {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let (u0, u1) = (a0 + (a1 - a0) * t0, a0 + (a1 - a0) * t1);
        // Thicker toward the leading edge.
        let w = 0.08 + 0.22 * (1.0 - (end - i) as f32 / tail as f32);
        let d0 = vec3(u0.cos(), 0.0, u0.sin());
        let d1 = vec3(u1.cos(), 0.0, u1.sin());
        let (r_in, r_out) = ((radius - w * 2.0) * k, radius * k);
        let drop = |t: f32| vec3(0.0, (0.25 - 0.5 * t) * k, 0.0);
        let q = [centre + d0 * r_in + drop(t0), centre + d0 * r_out + drop(t0), centre + d1 * r_out + drop(t1), centre + d1 * r_in + drop(t1)];
        gl.quad(q, Vec3::Y, ARC);
    }
}
