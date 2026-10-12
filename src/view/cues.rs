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
//! - sneaking crouches the figure (drawn in `scene::person`);
//! - a caster stands in a glowing ring that closes as the spell comes;
//! - two fighters trading blows are joined by a thin red line on the ground;
//! - someone bleeding drips red, and leaves red spots where they stand;
//! - a heavy blow that rocks someone (a stagger) spins a ring of pale
//!   sparks round their head; knocked off their feet or out cold by it, a
//!   ring of dust on the ground too (and they lie flat, `scene::person`);
//! - a blow from someone never seen coming bursts red-gold where it lands
//!   (`sneak_burst`, drawn with the fight's other effects).
//!
//! Drawing only: everything here reads the sim's own states and times.

use bevy::math::{vec3, Vec3};

use gahturiyu_sim::sim::{
    combat::{Act, FxKind},
    geo::V2,
    person::PersonId,
    World,
};

use super::mesh::Builder;
use super::palette::Rgb;

const STICK: Rgb = [0.30, 0.24, 0.18];
const BLADE: Rgb = [0.72, 0.74, 0.78];
const ARC: Rgb = [1.0, 0.97, 0.85];
const MOTE: Rgb = [0.65, 0.75, 1.0];
const FLASK: Rgb = [0.85, 0.35, 0.30];
const LOCKED: Rgb = [0.85, 0.18, 0.14];
const BLOOD: Rgb = [0.55, 0.04, 0.03];
const SPARK: Rgb = [1.0, 0.95, 0.7];
const DUST: Rgb = [0.62, 0.56, 0.46];
const SNEAK: Rgb = [1.0, 0.55, 0.25];

/// How long a stagger's sparks show, seconds.
const JOLT_SHOWS: f64 = 0.7;

/// How long after a blow lands its arc still shows, seconds.
const ARC_AFTER: f64 = 0.2;
/// The share of a swing's wind-up the arc sweeps through.
pub const ARC_SHARE: f64 = 0.5;

/// Which way someone faces, radians (sim convention): at their target in a
/// fight, along their way if walking, else as they happen to stand.
pub fn facing(w: &World, pid: PersonId, at: V2) -> f32 {
    aim(w, pid, at).unwrap_or_else(|| rest(w, pid))
}

/// The way someone stands when nothing has turned them.
pub fn rest(w: &World, pid: PersonId) -> f32 {
    (w.people[pid as usize].seed % 628) as f32 / 100.0
}

/// The way someone is made to face, if anything makes them: who they talk
/// to or fight, what they go through, the squad's next corner.
pub fn aim(w: &World, pid: PersonId, at: V2) -> Option<f32> {
    // Talking: the two face each other.
    if let Some(c) = w.talk.as_ref().filter(|c| c.with == pid || c.npc == pid) {
        let d = w.person_pos(if c.with == pid { c.npc } else { c.with }).sub(at);
        if d.len() > 0.05 {
            return Some(d.y.atan2(d.x));
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
                return Some(d.y.atan2(d.x));
            }
        }
        if let Act::Cast { point, .. } = f.act {
            let d = point.sub(at);
            if d.len() > 0.05 {
                return Some(d.y.atan2(d.x));
            }
        }
    }
    // Going through a container, or picking its lock: facing it.
    let chest = w.searching_now(pid).or_else(|| w.picking.iter().find(|p| p.who == pid).and_then(|p| p.holder.map(|s| (p.door.0, p.door.1, s))));
    if let Some(c) = chest.and_then(|c| w.container(c)) {
        let d = c.pos.sub(at);
        if d.len() > 0.05 && w.squad.index(pid).is_none_or(|k| w.squad.route[k].is_empty()) {
            return Some(d.y.atan2(d.x));
        }
    }
    if let Some(k) = w.squad.index(pid) {
        let to = w.squad.route[k].first().copied().unwrap_or(w.squad.goal[k]);
        let d = to.sub(w.squad.at[k]);
        if d.len() > 0.3 {
            return Some(d.y.atan2(d.x));
        }
    }
    None
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
        // Out of a fight: still bleeding (not yet bound).
        let bleed = w.bleeding(pid) / 3600.0;
        if bleed > 0.0 {
            drips(b, base, h, r, bleed, w.time, pid, k);
        }
        // Out of a fight: a tool going up and down while they work.
        if working(w, pid) {
            let bob = ((w.time * 4.0 + pid as f64 * 1.7).sin() * 0.5 + 0.5) as f32;
            let tip = hand + fwd * (0.45 * k) + vec3(0.0, (0.35 - 0.6 * bob) * k, 0.0);
            b.stick(hand, tip, 0.06 * k, STICK);
            b.block(tip - vec3(0.0, 0.06 * k, 0.0), 0.16 * k, 0.1 * k, 0.12 * k, face, BLADE);
        }
        return;
    };
    let t = battle.time;
    if !f.dead && !f.fled && f.bleeding() > 0.0 {
        drips(b, base, h, r, f.bleeding(), t, pid, k);
    }
    // Rocked by a blow: sparks round the head; floored, dust round them.
    let me = battle.fighters.iter().position(|x| std::ptr::eq(x, f)).unwrap_or(usize::MAX);
    if let Some((age, down)) = battle.fx.iter().rev().find_map(|x| match x.kind {
        FxKind::Jolt { who, down } if who == me && t - x.at < JOLT_SHOWS && t >= x.at => Some(((t - x.at) / JOLT_SHOWS, down)),
        _ => None,
    }) {
        let age = age as f32;
        let head = base + vec3(0.0, if down || f.ko { r * 1.6 } else { h * 1.02 }, 0.0);
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU + age * 9.0;
            let at = head + vec3(a.cos(), 0.0, a.sin()) * ((0.3 + 0.25 * age) * k);
            gl.block(at, 0.09 * k, 0.09 * k, 0.09 * k, a, SPARK);
        }
        if down {
            ground_ring(gl, base + vec3(0.0, 0.05, 0.0), (0.6 + 1.2 * age) * k, 0.1 * k * (1.0 - age), DUST);
        }
    }
    if f.ko || f.dead || f.fled || !f.is_person() || f.has(gahturiyu_sim::sim::effects::Does::KnockDown).is_some() {
        return;
    }
    let ranged = f.weapon.range > 0.0;
    // Casting: a mote rising above the hand; drinking: a flask at the mouth.
    match f.act {
        Act::Cast { done, .. } => {
            let lift = (1.0 - ((done - t).max(0.0) / 1.5).min(1.0)) as f32;
            gl.dome(hand + vec3(0.0, (0.4 + 0.5 * lift) * k, 0.0), 0.16 * k, 0.16 * k, 0.24 * k, 0.0, 0.0, 2, MOTE);
            // A glowing circle on the ground round the caster, closing in
            // as the spell comes (UI-13).
            ground_ring(gl, base + vec3(0.0, 0.06, 0.0), (1.4 - 0.5 * lift) * k, 0.07 * k, MOTE);
            return;
        }
        Act::Drink { .. } => {
            let mouth = base + vec3(0.0, h * 0.88, 0.0) + fwd * (r * 0.9);
            b.column(mouth, 0.07 * k, 0.05 * k, 0.18 * k, 5, FLASK);
            return;
        }
        _ => {}
    }
    // Locked together: a thin red line on the ground to the one they're
    // trading blows with (drawn once, from the lower index; UI-13).
    if let Some(j) = f.target.filter(|&j| j < battle.fighters.len()) {
        let o = &battle.fighters[j];
        let me = battle.fighters.iter().position(|x| std::ptr::eq(x, f)).unwrap_or(usize::MAX);
        let close = o.pos.dist(f.pos) <= f.weapon.reach.max(o.weapon.reach) + 1.5;
        if close && o.active() && o.target == Some(me) && me < j && !ranged {
            let d = o.pos.sub(f.pos);
            let to = base + vec3(d.x, 0.0, d.y) + vec3(0.0, 0.05, 0.0);
            let from = base + vec3(0.0, 0.05, 0.0);
            let across = vec3(-d.y, 0.0, d.x).normalize_or_zero() * (0.03 * k);
            gl.quad([from - across, from + across, to + across, to - across], Vec3::Y, LOCKED);
        }
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
        // The bow or crossbow itself (`looks.rs`), else a stick with a crossbar.
        if super::looks::wield(b, w, pid, shoulder - dir * (0.25 * k), dir, k) {
            return;
        }
        b.stick(shoulder - dir * (0.25 * k), shoulder + dir * (len * k), 0.07 * k, STICK);
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
            if !super::looks::wield(b, w, pid, hand, tip - hand, k) {
                b.stick(hand, tip, 0.07 * k, BLADE);
            }
            return;
        }
        // Drawing back before the blow.
        let a = face + 1.3;
        let tip = hand + vec3(a.cos(), 0.35, a.sin()) * (reach * 0.8 * k);
        if !super::looks::wield(b, w, pid, hand, tip - hand, k) {
            b.stick(hand, tip, 0.07 * k, BLADE);
        }
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
    if !super::looks::wield(b, w, pid, hand, tip - hand, k) {
        b.stick(hand, tip, 0.07 * k, BLADE);
    }
}

/// Blood: drops falling from the body, faster the worse the bleed, and a
/// few red spots on the ground round their feet.
#[allow(clippy::too_many_arguments)]
fn drips(b: &mut Builder, base: Vec3, h: f32, r: f32, rate: f32, t: f64, pid: PersonId, k: f32) {
    let n = if rate >= 0.4 { 3 } else if rate >= 0.12 { 2 } else { 1 };
    let fall = h * 0.55;
    for i in 0..n {
        let phase = ((t * (0.8 + rate as f64 * 1.5) + i as f64 / n as f64 + pid as f64 * 0.37).fract()) as f32;
        let a = pid as f32 * 2.3 + i as f32 * 2.1;
        let off = vec3(a.cos(), 0.0, a.sin()) * (r * 0.9);
        let at = base + off + vec3(0.0, fall * (1.0 - phase * phase), 0.0);
        b.block(at, 0.05 * k, 0.05 * k, 0.09 * k, a, BLOOD);
    }
    for i in 0..(2 + n) {
        let a = pid as f32 * 1.3 + i as f32 * 2.4;
        let d = r * (0.6 + 0.5 * ((i * 7 + pid as usize) % 5) as f32 / 5.0);
        let at = base + vec3(a.cos() * d, 0.04, a.sin() * d);
        let (u, v) = (vec3(a.cos(), 0.0, a.sin()) * (0.07 * k), vec3(-a.sin(), 0.0, a.cos()) * (0.05 * k));
        b.quad([at - u - v, at + u - v, at + u + v, at - u + v], Vec3::Y, BLOOD);
    }
}

/// A blow on someone who never saw it coming: a red-gold burst where it
/// landed, swelling and gone in half a second (`age` in seconds).
pub fn sneak_burst(gl: &mut Builder, at: Vec3, age: f32, k: f32) {
    let p = (age / 0.5).clamp(0.0, 1.0);
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let d = vec3(a.cos(), 0.0, a.sin());
        let r0 = (0.2 + 0.9 * p) * k;
        gl.stick(at + d * r0, at + d * (r0 + 0.35 * k * (1.0 - p)), 0.06 * k, SNEAK);
    }
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

/// A flat ring on the ground (glowing), centred at `c`.
fn ground_ring(gl: &mut Builder, c: Vec3, r: f32, w: f32, col: Rgb) {
    let n = 28;
    for i in 0..n {
        let (a0, a1) = (i as f32 / n as f32 * std::f32::consts::TAU, (i + 1) as f32 / n as f32 * std::f32::consts::TAU);
        let (d0, d1) = (vec3(a0.cos(), 0.0, a0.sin()), vec3(a1.cos(), 0.0, a1.sin()));
        gl.quad([c + d0 * (r - w), c + d0 * (r + w), c + d1 * (r + w), c + d1 * (r - w)], Vec3::Y, col);
    }
}
