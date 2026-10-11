//! Walking that looks like walking (#162 UI-9): a slight bob in step with
//! the pace, easing in and out of a stop, turning at a person's speed, and
//! keeping the way they faced when they stop. Townsfolk face where they
//! walk too.
//!
//! Drawing only. The sim says where everyone is; this keeps, per person
//! drawn, where the figure was drawn last frame and which way it faced,
//! and eases the figure after the sim's position. Nothing here is saved or
//! read back by the world.

use std::collections::HashMap;

use gahturiyu_sim::sim::{geo::V2, person::PersonId};

/// How quickly the figure catches up with where the sim has them, seconds
/// (the time to close about two thirds of the gap): a gentle start and stop.
const EASE: f32 = 0.09;
/// Further than this from the drawn figure (a jump, a load, the clock run
/// fast), the figure is put straight there, metres.
const SNAP: f32 = 1.6;
/// How fast someone turns, radians a second.
const TURN: f32 = 6.0;
/// Metres a step; each step is one rise and fall of the body.
const STRIDE: f32 = 0.75;
/// How far the body rises at the top of a step, as a share of its height.
pub const BOB: f32 = 0.035;
/// Moving less than this between frames counts as standing, metres.
const STILL: f32 = 0.002;
/// An entry not drawn for this many frames is forgotten.
const FORGET: u32 = 240;

#[derive(Clone, Copy)]
struct Gait {
    /// Where the figure was drawn.
    pos: V2,
    /// Where the sim had them last frame.
    was: V2,
    face: f32,
    /// How far into their step (radians, one step per π).
    phase: f32,
    /// 0 standing .. 1 walking, eased.
    going: f32,
    seen: u32,
}

/// How to draw someone this frame.
#[derive(Clone, Copy)]
pub struct Pose {
    pub pos: V2,
    pub face: f32,
    /// The body's lift at this point of the step, as a share of its height.
    pub bob: f32,
}

#[derive(Default)]
pub struct Gaits {
    /// The person under the mouse (drawn a little brighter, UI-4).
    pub hot: Option<PersonId>,
    map: HashMap<PersonId, Gait>,
    frame: u32,
    dt: f32,
}

impl Gaits {
    /// Start a frame `dt` seconds after the last.
    pub fn begin(&mut self, dt: f32) {
        self.frame = self.frame.wrapping_add(1);
        self.dt = dt.clamp(0.0, 0.1);
        let f = self.frame;
        if f % 60 == 0 {
            self.map.retain(|_, g| f.wrapping_sub(g.seen) < FORGET);
        }
    }

    /// The pose of `pid`, whom the sim has at `at`. `aim` is a way they're
    /// made to face (a foe, someone they talk to, a chest, their walk's next
    /// corner); `rest` the way they stand if nothing has turned them yet.
    pub fn pose(&mut self, pid: PersonId, at: V2, aim: Option<f32>, rest: f32) -> Pose {
        let (dt, frame) = (self.dt, self.frame);
        let g = self.map.entry(pid).or_insert(Gait { pos: at, was: at, face: aim.unwrap_or(rest), phase: 0.0, going: 0.0, seen: frame });
        // Not drawn last frame: start afresh where they are.
        if frame.wrapping_sub(g.seen) > 2 {
            *g = Gait { pos: at, was: at, face: aim.unwrap_or(g.face), phase: 0.0, going: 0.0, seen: frame };
        }
        g.seen = frame;
        let moved = at.sub(g.was);
        g.was = at;
        let gap = at.sub(g.pos);
        let snapped = gap.len() > SNAP;
        let before = g.pos;
        if snapped {
            g.pos = at;
        } else if dt > 0.0 {
            g.pos = g.pos.add(gap.scale(1.0 - (-dt / EASE).exp()));
        }
        let step = g.pos.dist(before);
        // Walking: facing along the way, unless they're made to face elsewhere.
        let walking = moved.len() > STILL;
        let want = aim.or(walking.then(|| moved.y.atan2(moved.x)));
        if let Some(want) = want {
            let d = wrap(want - g.face);
            let most = TURN * dt;
            g.face = if snapped || d.abs() <= most { want } else { g.face + most * d.signum() };
        }
        // The step: one rise and fall each stride, fading in and out with
        // the pace.
        let target = if walking && !snapped && dt > 0.0 { 1.0 } else { 0.0 };
        g.going += (target - g.going) * (1.0 - (-dt / 0.15).exp());
        g.phase = (g.phase + step / STRIDE * std::f32::consts::PI) % (std::f32::consts::TAU * 64.0);
        Pose { pos: g.pos, face: wrap(g.face), bob: BOB * g.going * g.phase.sin().abs() }
    }

    /// Where someone drawn this frame was drawn, if they were.
    pub fn drawn(&self, pid: PersonId) -> Option<V2> {
        self.map.get(&pid).filter(|g| g.seen == self.frame).map(|g| g.pos)
    }

    /// The way someone drawn this frame faces, if they were.
    pub fn face(&self, pid: PersonId) -> Option<f32> {
        self.map.get(&pid).filter(|g| g.seen == self.frame).map(|g| wrap(g.face))
    }
}

/// An angle brought into -π..π.
fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let a = a.rem_euclid(t);
    if a > std::f32::consts::PI {
        a - t
    } else {
        a
    }
}
