//! Bodies, Kenshi-style: damage lands on a part, not on one health bar.
//!
//! - Head or torso at zero: knocked out. Below minus their maximum: dead.
//! - An arm at zero can't hold a weapon; the other arm fights badly.
//! - Both legs at zero: down and crawling; one leg: limping.
//! - An arm or leg battered past `LIMB_LOSS` times its maximum below zero is
//!   lost for good: it never heals. A lost arm can't hold a weapon or
//!   shield (and two-handed weapons need both); one lost leg is a permanent
//!   limp, both mean crawling.
//!
//! Wounds outlast the fight and heal over time. Healing is worked out from the
//! clock rather than ticked, like journeys, so it costs nothing while nobody is
//! looking and comes out the same however the world is stepped.

use serde::{Deserialize, Serialize};

use super::stats::Stats;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

pub const PARTS: [Part; 6] = [Part::Head, Part::Torso, Part::LeftArm, Part::RightArm, Part::LeftLeg, Part::RightLeg];

impl Part {
    pub fn name(self) -> &'static str {
        match self {
            Part::Head => "head",
            Part::Torso => "torso",
            Part::LeftArm => "left arm",
            Part::RightArm => "right arm",
            Part::LeftLeg => "left leg",
            Part::RightLeg => "right leg",
        }
    }

    /// How often a blow lands here, out of 1.
    pub fn hit_weight(self) -> f32 {
        match self {
            Part::Head => 0.10,
            Part::Torso => 0.36,
            Part::LeftArm | Part::RightArm => 0.14,
            Part::LeftLeg | Part::RightLeg => 0.13,
        }
    }

    pub fn vital(self) -> bool {
        matches!(self, Part::Head | Part::Torso)
    }
}

/// Health lost per body part per game hour while resting outside a fight.
pub const HEAL_PER_HOUR: f32 = 6.0;
/// An arm or leg is lost for good once its health falls to this many times
/// its maximum below zero (1.0 = at minus its maximum).
pub const LIMB_LOSS: f32 = 1.0;

impl Part {
    pub fn is_limb(self) -> bool {
        !self.vital()
    }
}

/// Damage carried on each part, as of a moment in time, and how fast it's
/// mending from then on.
///
/// Most people in the world heal at the constant `HEAL_PER_HOUR` (their
/// lives aren't followed closely enough for anything finer — a deliberate
/// simplification for now). Your squad's rate follows how they're doing:
/// `condition.rs` rewrites `rate` (and `drain`, for starvation) each time
/// their circumstances change, and settles `lost` at that moment, so healing
/// is worked out piece by piece from the clock.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Wounds {
    pub lost: [f32; 6],
    /// When `lost` was last written.
    pub at: f64,
    /// Health mended per part per hour from `at` on.
    pub rate: f32,
    /// Torso health wasting away per hour from `at` on (starvation).
    pub drain: f32,
    /// Wasting stops once torso damage reaches this (knocked out, not dead).
    pub drain_cap: f32,
    /// Limbs lost for good. They never heal.
    pub missing: [bool; 6],
}

impl Default for Wounds {
    fn default() -> Wounds {
        Wounds { lost: [0.0; 6], at: 0.0, rate: HEAL_PER_HOUR, drain: 0.0, drain_cap: 0.0, missing: [false; 6] }
    }
}

impl Wounds {
    /// Damage remaining on each part at time `t`, after healing (or wasting).
    pub fn lost_at(&self, t: f64) -> [f32; 6] {
        let h = ((t - self.at).max(0.0) / 3600.0) as f32;
        let healed = self.rate * h;
        let mut out = self.lost.map(|l| (l - healed).max(0.0));
        for i in 0..6 {
            if self.missing[i] {
                out[i] = self.lost[i];
            }
        }
        if self.drain > 0.0 {
            let torso = Part::Torso as usize;
            out[torso] = (self.lost[torso] + self.drain * h).min(self.drain_cap.max(self.lost[torso]));
        }
        out
    }

    /// Health on each part at time `t`.
    pub fn hp_at(&self, stats: &Stats, t: f64) -> [f32; 6] {
        let lost = self.lost_at(t);
        let mut out = [0.0; 6];
        for (i, p) in PARTS.iter().enumerate() {
            out[i] = stats.max_hp(*p) - lost[i];
        }
        out
    }

    /// Record health as it stands at time `t` (after a fight, say).
    pub fn set(&mut self, stats: &Stats, hp: &[f32; 6], t: f64) {
        for (i, p) in PARTS.iter().enumerate() {
            if self.missing[i] {
                continue; // a lost limb stays lost, whatever mends the rest
            }
            self.lost[i] = (stats.max_hp(*p) - hp[i]).max(0.0);
        }
        self.at = t;
    }

    pub fn is_hurt(&self, t: f64) -> bool {
        self.lost_at(t).iter().enumerate().any(|(i, &l)| l > 0.5 && !self.missing[i])
    }

    /// Names of limbs lost for good.
    pub fn lost_limbs(&self) -> Vec<&'static str> {
        PARTS.iter().enumerate().filter(|(i, _)| self.missing[*i]).map(|(_, p)| p.name()).collect()
    }

    /// Overall health, 0..1, for summaries and the far bands.
    pub fn fraction(&self, stats: &Stats, t: f64) -> f32 {
        let hp = self.hp_at(stats, t);
        let total: f32 = PARTS.iter().map(|p| stats.max_hp(*p)).sum();
        (hp.iter().map(|h| h.max(0.0)).sum::<f32>() / total).clamp(0.0, 1.0)
    }
}

/// What a set of part healths means for someone's ability to act.
pub fn knocked_out(hp: &[f32; 6]) -> bool {
    hp[Part::Head as usize] <= 0.0 || hp[Part::Torso as usize] <= 0.0
}

pub fn dead(hp: &[f32; 6], stats: &Stats) -> bool {
    hp[Part::Head as usize] <= -stats.max_hp(Part::Head) || hp[Part::Torso as usize] <= -stats.max_hp(Part::Torso)
}

/// Footspeed multiplier from leg health: fine, limping, or crawling.
pub fn leg_factor(hp: &[f32; 6]) -> f32 {
    let (l, r) = (hp[Part::LeftLeg as usize], hp[Part::RightLeg as usize]);
    match (l > 0.0, r > 0.0) {
        (true, true) => 1.0,
        (true, false) | (false, true) => 0.55,
        (false, false) => 0.2,
    }
}
