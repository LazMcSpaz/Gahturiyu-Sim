//! What a spell cast outside a fight looks like (Laz's P16; drawing only).
//! Fights have their own cues (`cues.rs`); out here the sim notes each cast
//! in `World::cast_marks`, and the window plays it out on its own clock:
//!
//! - the caster: a mote rising from the hand and a ring closing round them;
//! - then something crossing to where it was aimed (if it was aimed away);
//! - then a burst there: a ring spreading on the ground and a flash.
//!
//! A spell that fizzles is a grey puff at the caster and goes no further.
//! The colour is the spell's domain. A sound plays as the casting begins.

use bevy::math::vec3;

use gahturiyu_sim::sim::{casting::CastMark, geo::V2, magic::Domain, World};

use super::app::Game;
use super::cam::to3;
use super::mesh::Builder;
use super::palette::{self, Rgb};

/// Seconds for each part: the caster's, the crossing, the burst.
const GATHER: f32 = 0.35;
const CROSS: f32 = 0.35;
const BURST: f32 = 0.5;

/// Casts being shown, and the last one taken from the world.
#[derive(Default)]
pub struct CastFx {
    shown: Vec<(CastMark, f32)>,
    seen: u64,
    loads: u32,
}

/// The colour of a domain's magic.
pub fn domain_colour(d: Domain) -> Rgb {
    match d {
        Domain::Elemental => [1.0, 0.58, 0.25],
        Domain::Psychic => [0.78, 0.55, 1.0],
        Domain::Illusion => [0.45, 0.95, 0.90],
        Domain::Vital => [0.50, 1.0, 0.55],
        Domain::Warding => [1.0, 0.93, 0.65],
        Domain::Alteration => [0.45, 0.65, 1.0],
        Domain::Summoning => [0.70, 0.90, 1.0],
        Domain::Necromancy => [0.60, 0.80, 0.40],
    }
}

/// Take new casts from the world (with their sound), and forget old ones.
/// `now`: the window's clock, seconds.
pub fn update(game: &mut Game, now: f32) {
    let w = &game.world;
    let fx = &mut game.castfx;
    if fx.loads != game.loads {
        // A save loaded: start afresh (its casts aren't new).
        *fx = CastFx { seen: w.cast_seq, loads: game.loads, ..Default::default() };
    }
    for m in w.cast_marks.iter().filter(|m| m.n > fx.seen) {
        fx.shown.push((*m, now));
        game.sounds.at("cast", m.from);
    }
    fx.seen = fx.seen.max(w.cast_seq);
    fx.shown.retain(|(_, t0)| now - t0 < GATHER + CROSS + BURST);
}

/// Draw the casts being shown: glows in `gl`, marks on the ground in `fl`.
#[allow(clippy::too_many_arguments)]
pub fn draw(fx: &CastFx, w: &World, gl: &mut Builder, fl: &mut Builder, ground: &dyn Fn(V2) -> f32, k: f32, now: f32, ring: &dyn Fn(&mut Builder, V2, f32, f32, Rgb)) {
    let kk = k.min(3.0);
    for (m, t0) in &fx.shown {
        let age = now - t0;
        let col = if m.ok { domain_colour(m.spell.def().domain) } else { [0.62, 0.62, 0.60] };
        let at = w.squad.index(m.by).map(|i| w.member_pos(i)).unwrap_or(m.from);
        let hand = to3(at, ground(at) + 1.3 * kk);
        // The caster: a mote rising, a ring closing.
        if age < GATHER {
            let f = age / GATHER;
            gl.column(hand + vec3(0.0, f * 0.8 * kk, 0.0), 0.16 * kk * (1.0 - 0.4 * f), 0.02, 0.3 * kk, 6, col);
            ring(fl, at, (1.6 - 1.0 * f) * kk, 0.08 * kk, palette::scale(col, 1.0 - 0.5 * f));
        }
        if !m.ok {
            // A fizzle: a grey puff that spreads and fades.
            if age < GATHER + BURST {
                let f = age / (GATHER + BURST);
                gl.column(hand - vec3(0.0, 0.3 * kk, 0.0), (0.2 + 0.5 * f) * kk, (0.3 + 0.6 * f) * kk, (0.3 + 0.4 * f) * kk, 6, palette::scale(col, 1.0 - f));
            }
            continue;
        }
        let far = m.to.dist(at) > 1.5;
        let to = to3(m.to, ground(m.to) + 1.1 * kk);
        // The crossing: a bright point on a low arc.
        if far && age >= GATHER * 0.7 && age < GATHER + CROSS {
            let f = ((age - GATHER * 0.7) / (CROSS + GATHER * 0.3)).clamp(0.0, 1.0);
            let p = hand.lerp(to, f) + vec3(0.0, (f * (1.0 - f)) * 4.0 * 0.6 * kk, 0.0);
            gl.column(p - vec3(0.0, 0.12 * kk, 0.0), 0.14 * kk, 0.14 * kk, 0.24 * kk, 6, col);
            // A short trail behind it.
            let back = hand.lerp(to, (f - 0.12).max(0.0));
            gl.stick(back, p, 0.05 * kk, palette::scale(col, 0.7));
        }
        // The burst where it lands (on the caster themselves if aimed there).
        let start = if far { GATHER + CROSS } else { GATHER };
        if age >= start && age < start + BURST {
            let f = (age - start) / BURST;
            let spot = if far { m.to } else { at };
            ring(fl, spot, (0.4 + 2.0 * f) * kk, (0.18 * (1.0 - f) + 0.03) * kk, palette::scale(col, 1.0 - f));
            if f < 0.5 {
                let base = to3(spot, ground(spot));
                gl.column(base, (0.5 - 0.6 * f) * kk, 0.02, (2.2 - 2.0 * f) * kk, 8, palette::scale(col, 1.0 - 1.6 * f));
            }
        }
    }
}
