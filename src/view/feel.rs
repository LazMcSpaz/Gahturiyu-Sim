//! Small answers to the player's hand (the feel plan's Part 1): a mark
//! where a click landed, a nod from whoever was given an order (and now and
//! then a word), the slow breath of the rings under selected members, and
//! a camera that glides to where it's sent instead of jumping.
//!
//! All of it is drawing only. Nothing here changes what happens in the
//! world; it reads the window's own clock, so it runs the same whether the
//! world is paused or racing. Screenshots step that clock at a fixed pace
//! and the camera snaps, so a given set of flags still gives one picture.

use std::sync::Mutex;

use gahturiyu_sim::sim::{geo::V2, person::PersonId, World};

use super::cam::OrbitCam;

/// How long a click's mark lasts, seconds (under half a second).
pub const MARK_SECS: f32 = 0.45;
/// How long a nod takes, seconds.
pub const NOD_SECS: f32 = 0.35;
/// At most one word in answer this often, seconds; and one order in this
/// many gets one at all.
pub const WORD_EVERY: f32 = 20.0;
pub const WORD_ONE_IN: u32 = 4;
/// How long the word stays over their head, frames.
pub const WORD_FRAMES: u32 = 80;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkKind {
    /// Somewhere to walk to.
    Walk,
    /// Someone to go for.
    Attack,
    /// Not something that can be done.
    Cant,
}

#[derive(Clone, Copy, Debug)]
pub struct Mark {
    pub at: V2,
    pub born: f32,
    pub kind: MarkKind,
}

impl Mark {
    /// How far through its life, 0 to 1.
    pub fn age(&self, now: f32) -> f32 {
        ((now - self.born) / MARK_SECS).clamp(0.0, 1.0)
    }
}

/// Who is mid-nod, and how deep (0 to 1), for the figure drawing to read.
static NODS: Mutex<Vec<(PersonId, f32)>> = Mutex::new(Vec::new());

/// How far this person's figure dips just now, 0 to 1.
pub fn dip(pid: PersonId) -> f32 {
    NODS.lock().ok().and_then(|n| n.iter().find(|x| x.0 == pid).map(|x| x.1)).unwrap_or(0.0)
}

#[derive(Default)]
pub struct Feel {
    /// Seconds since the window opened.
    pub now: f32,
    pub marks: Vec<Mark>,
    /// Who was just given an order, and when.
    acks: Vec<(PersonId, f32)>,
    /// When someone last said a word in answer, and how many orders since.
    spoke: Option<f32>,
    orders: u32,
    /// Screenshots: the marks stay as they are instead of running out.
    held: bool,
}

impl Feel {
    /// Once a frame: the clock on, the spent marks and nods dropped.
    pub fn tick(&mut self, dt: f32) {
        self.now += dt;
        let now = self.now;
        if self.held {
            self.marks.iter_mut().for_each(|m| m.born += dt);
        }
        self.marks.retain(|m| now - m.born < MARK_SECS);
        self.acks.retain(|a| now - a.1 < NOD_SECS);
        if let Ok(mut n) = NODS.lock() {
            n.clear();
            n.extend(self.acks.iter().map(|&(p, _)| (p, self.nod(p))));
        }
    }

    /// How deep this person's nod is just now, 0 to 1: down and up, once.
    pub fn nod(&self, pid: PersonId) -> f32 {
        self.acks.iter().find(|a| a.0 == pid).map_or(0.0, |a| (((self.now - a.1) / NOD_SECS).clamp(0.0, 1.0) * std::f32::consts::PI).sin())
    }

    /// For screenshots (`GAHT_MARK`): one mark of each kind in a row beside
    /// this spot, held part-way closed.
    pub fn show(&mut self, at: V2) {
        self.held = true;
        for (i, kind) in [MarkKind::Walk, MarkKind::Attack, MarkKind::Cant].into_iter().enumerate() {
            self.marks.push(Mark { at: at.add(V2::new(i as f32 * 5.0 - 5.0, 4.0)), born: self.now - MARK_SECS * 0.35, kind });
        }
    }

    /// A click landed here.
    pub fn mark(&mut self, at: V2, kind: MarkKind) {
        self.marks.push(Mark { at, born: self.now, kind });
    }

    /// These were just given an order: they nod, and now and then one of
    /// them has a word to say (who, and a count to vary the line by; the
    /// lines are `about=order` in `data/lines/squad.txt`). Never every time.
    pub fn ordered(&mut self, w: &World, who: &[PersonId]) -> Option<(PersonId, u64)> {
        let now = self.now;
        let up: Vec<PersonId> = who.iter().copied().filter(|&p| w.squad.index(p).is_some() && !w.is_down(p) && !w.is_asleep(p)).collect();
        for &p in &up {
            self.acks.retain(|a| a.0 != p);
            self.acks.push((p, now));
        }
        self.orders += 1;
        let quiet = self.spoke.is_none_or(|t| now - t >= WORD_EVERY);
        let &first = up.first()?;
        if !quiet || self.orders % WORD_ONE_IN != 1 {
            return None;
        }
        self.spoke = Some(now);
        Some((first, self.orders as u64))
    }

    /// The slow breath of a selected member's ring, 0 to 1.
    pub fn breath(&self) -> f32 {
        0.5 + 0.5 * (self.now * 2.1).sin()
    }
}

/// Where the camera is being sent: it eases there over a fraction of a
/// second instead of jumping.
#[derive(Clone, Copy, Debug)]
pub struct Glide {
    pub dist: f32,
    pub pitch: f32,
    /// Where it left the camera last frame. If the camera isn't there any
    /// more, something else set it (a turn by hand, a leap to a scout, a
    /// talk's close-up): that is where it's wanted now.
    left: (f32, f32),
}

/// The tilt that goes with a distance: low toward the ground close in,
/// steep toward a map view far out.
pub fn tilt_for(dist: f32) -> f32 {
    let f = ((dist.max(12.0) / 12.0).ln() / (4500.0f32 / 12.0).ln()).clamp(0.0, 1.0);
    0.30 + f * 0.95
}

impl Glide {
    pub fn of(o: &OrbitCam) -> Glide {
        Glide { dist: o.dist, pitch: o.pitch, left: (o.dist, o.pitch) }
    }

    /// A turn of the wheel: nearer or further, and the tilt goes with it
    /// (by as much as the tilt for the new distance differs from the old,
    /// so a tilt the player set by hand is kept).
    pub fn zoom(&mut self, wheel: f32) {
        let before = self.dist;
        self.dist = super::cam::zoom_step(before, wheel);
        self.pitch = (self.pitch + tilt_for(self.dist) - tilt_for(before)).clamp(0.08, 1.50);
    }

    /// Once a frame: the camera a step nearer to where it's being sent.
    /// `follow` is the spot it should be over, if it's following; `snap`
    /// (screenshots) goes there at once.
    pub fn step(&mut self, o: &mut OrbitCam, follow: Option<V2>, dt: f32, snap: bool) {
        if (o.dist - self.left.0).abs() > 1e-4 {
            self.dist = o.dist;
        }
        if (o.pitch - self.left.1).abs() > 1e-5 {
            self.pitch = o.pitch;
        }
        let k = if snap { 1.0 } else { 1.0 - (-10.0 * dt).exp() };
        o.dist += (self.dist - o.dist) * k;
        o.pitch += (self.pitch - o.pitch) * k;
        self.left = (o.dist, o.pitch);
        if let Some(to) = follow {
            // (Far off, a load or a leap to another member: straight there.)
            let kf = if snap || o.target.dist(to) > 400.0 { 1.0 } else { 1.0 - (-8.0 * dt).exp() };
            o.target = o.target.add(to.sub(o.target).scale(kf));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_is_gone_in_under_half_a_second() {
        let mut f = Feel::default();
        f.mark(V2::new(1.0, 2.0), MarkKind::Walk);
        f.tick(0.2);
        assert_eq!(f.marks.len(), 1);
        assert!(f.marks[0].age(f.now) > 0.3 && f.marks[0].age(f.now) < 0.6);
        f.tick(0.3);
        assert!(f.marks.is_empty());
    }

    #[test]
    fn an_order_gets_a_nod_every_time_and_a_word_only_now_and_then() {
        let w = gahturiyu_sim::sim::worldgen::generate(1);
        let who = w.squad.members.clone();
        let mut f = Feel::default();
        let mut words = 0;
        for _ in 0..12 {
            f.tick(0.5);
            words += f.ordered(&w, &who).is_some() as u32;
            // Everyone sent is mid-nod.
            f.tick(NOD_SECS / 2.0);
            assert!(who.iter().all(|&p| f.nod(p) > 0.9), "no nod");
        }
        assert_eq!(words, 1, "a word every time is a word too many");
        // The nod is over in a moment; a while later there's a word again.
        f.tick(NOD_SECS);
        assert!(who.iter().all(|&p| f.nod(p) == 0.0));
        f.tick(WORD_EVERY);
        let said = (0..WORD_ONE_IN).filter(|_| f.ordered(&w, &who).is_some()).count();
        assert_eq!(said, 1);
        // And there is a line for it, for every member.
        for &m in &who {
            assert!(w.squad_remark(m, "order", &[], 1).is_some_and(|l| !l.is_empty()));
        }
    }

    #[test]
    fn zooming_in_lowers_the_tilt_and_out_raises_it_and_a_hand_set_tilt_is_kept() {
        assert!(tilt_for(12.0) < tilt_for(110.0) && tilt_for(110.0) < tilt_for(4500.0));
        let at = |dist: f32, pitch: f32| Glide { dist, pitch, left: (dist, pitch) };
        let mut g = at(110.0, 0.48);
        g.zoom(1.0);
        assert!(g.dist < 110.0 && g.pitch < 0.48);
        let mut g = at(110.0, 0.48);
        g.zoom(-1.0);
        assert!(g.dist > 110.0 && g.pitch > 0.48);
        // In and back out again: where it was.
        let mut g = at(110.0, 0.9);
        g.zoom(1.0);
        g.zoom(-1.0);
        assert!((g.dist - 110.0).abs() < 0.5 && (g.pitch - 0.9).abs() < 1e-3);
    }

    #[test]
    fn the_camera_glides_and_arrives() {
        let mut o = OrbitCam::new(V2::new(0.0, 0.0));
        let mut g = Glide::of(&o);
        g.dist = 40.0;
        g.pitch = 0.3;
        let was = o.dist;
        g.step(&mut o, Some(V2::new(10.0, 0.0)), 1.0 / 60.0, false);
        // One frame: on its way, not there.
        assert!(o.dist < was && o.dist > 40.0);
        assert!(o.target.x > 0.0 && o.target.x < 10.0);
        for _ in 0..120 {
            g.step(&mut o, Some(V2::new(10.0, 0.0)), 1.0 / 60.0, false);
        }
        assert!((o.dist - 40.0).abs() < 0.1 && (o.pitch - 0.3).abs() < 0.01 && o.target.dist(V2::new(10.0, 0.0)) < 0.05);
        // A screenshot goes straight there.
        let mut o = OrbitCam::new(V2::new(0.0, 0.0));
        let mut g = Glide::of(&o);
        g.dist = 40.0;
        g.pitch = 0.3;
        g.step(&mut o, Some(V2::new(10.0, 0.0)), 1.0 / 30.0, true);
        assert!((o.dist - 40.0).abs() < 1e-3 && (o.pitch - 0.3).abs() < 1e-4 && o.target.dist(V2::new(10.0, 0.0)) < 1e-3);
        // Set by hand since (a turn, a leap): that's where it's wanted.
        o.pitch = 0.7;
        o.dist = 200.0;
        g.step(&mut o, None, 1.0 / 60.0, false);
        assert!((o.dist - 200.0).abs() < 1e-3 && (o.pitch - 0.7).abs() < 1e-4 && (g.dist - 200.0).abs() < 1e-3);
    }
}
