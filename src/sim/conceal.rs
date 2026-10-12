//! Concealment (the items design, 0.2 rule 9): never a guarantee.
//!
//! Every weapon and worn piece has a concealment value (`WeaponDef::conceal`,
//! `ArmorDef::conceal`); what's worn over it hides it better (a cloak over
//! everything, a coat over what's under it and at the belt), and so does the
//! carrier's Sneak. Whether a look notices something is one keyed roll per
//! looker, carrier, piece, hour and kind of look: concealment against the
//! looker's sharpness (Intellect and Willpower, as watchers in `stealth.rs`)
//! and how hard they're looking. A full search is far harder to beat than a
//! glance. Nothing is ever certain to stay hidden.
//!
//! What it changes: a piece nobody has noticed isn't counted when someone
//! sizes its carrier up (`World::seen_might`: bandits weighing up
//! travellers, a threat made in conversation), and a well-hidden weapon
//! doesn't show on the figure (`shows`). There's no "weapon of war" rule:
//! carrying a big weapon in town is fine (Laz).

use super::combat;
use super::inventory::Gear;
use super::items::{item, Slot, SLOTS};
use super::person::{Person, PersonId};
use super::rng::Rng;
use super::stats::{Attr, Skill};
use super::world::{World, HOUR};

/// How hard someone is looking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    /// A passing look.
    Glance,
    /// Weighing someone up before deciding something about them.
    SizeUp,
    /// Going through their clothes and pack.
    Search,
}

impl Look {
    /// Added to the chance of noticing.
    pub fn effort(self) -> f32 {
        match self {
            Look::Glance => 0.0,
            Look::SizeUp => 0.2,
            Look::Search => 0.65,
        }
    }

    fn tag(self) -> u64 {
        match self {
            Look::Glance => 1,
            Look::SizeUp => 2,
            Look::Search => 3,
        }
    }
}

/// A weapon or piece this well hidden (its own value) isn't drawn on the
/// figure.
pub const HIDES: f32 = 0.6;
/// Worn over the rest: a cloak hides this much more of everything, a coat
/// (anything in `BodyOver`) this much more of what's under it and at the
/// belt (short weapons).
pub const CLOAK_COVER: f32 = 0.25;
pub const COAT_COVER: f32 = 0.15;
/// Sneak skill hides this much at 100.
pub const SNEAK_COVER: f32 = 0.25;
/// The chance a glance notices something not hidden at all, and how much
/// each point of hiding takes off it; the floor and ceiling (never certain).
pub const SEEN_PLAIN: f32 = 0.95;
pub const PER_HIDDEN: f32 = 1.1;
pub const NEVER_SURE: (f32, f32) = (0.02, 0.98);

/// The piece in `slot`'s own concealment (weapons and worn armour; other
/// things, a torch in the hand, a pack on the back, are plain to see).
pub fn conceal_of(gear: &Gear, slot: Slot) -> Option<f32> {
    let id = gear.in_slot(slot)?;
    let d = item(id);
    Some(match (d.weapon(), d.armor()) {
        (Some(w), _) => w.conceal,
        (_, Some(a)) => a.conceal,
        _ => 0.0,
    })
}

/// Does what's in `slot` show on the figure? (Drawing: a well-hidden weapon
/// doesn't.)
pub fn shows(gear: &Gear, slot: Slot) -> bool {
    conceal_of(gear, slot).is_none_or(|c| c < HIDES)
}

/// How well hidden the piece in `slot` is on someone, 0..1: its own value,
/// what's worn over it, and their Sneak.
pub fn hidden(gear: &Gear, sneak: f32, slot: Slot) -> f32 {
    let Some(own) = conceal_of(gear, slot) else { return 0.0 };
    let cloak = gear.in_slot(Slot::Cloak).is_some() && slot != Slot::Cloak;
    let coat = gear.in_slot(Slot::BodyOver).is_some();
    let short = matches!(slot, Slot::MainHand | Slot::OffHand) && gear.in_slot(slot).and_then(|id| item(id).weapon()).is_some_and(|w| w.space == super::items::Space::Short);
    let under = matches!(slot, Slot::BodyUnder | Slot::LegsUnder);
    let cover = if cloak { CLOAK_COVER } else { 0.0 } + if coat && (under || short) { COAT_COVER } else { 0.0 };
    (own + cover + sneak.clamp(0.0, 100.0) / 100.0 * SNEAK_COVER).min(1.0)
}

/// The chance a look notices something this well hidden, by a looker this
/// sharp (0..1).
pub fn notice_chance(hidden: f32, sharp: f32, look: Look) -> f32 {
    (SEEN_PLAIN - hidden * PER_HIDDEN + look.effort() + (sharp - 0.5) * 0.3).clamp(NEVER_SURE.0, NEVER_SURE.1)
}

/// How sharp-eyed someone is, 0..1 (Intellect and Willpower, as watchers).
pub fn sharpness(p: &Person) -> f32 {
    ((p.stats.attr(Attr::Intellect) + p.stats.attr(Attr::Willpower)) / 200.0).clamp(0.0, 1.0)
}

impl World {
    /// Does `looker` notice what `carrier` has in `slot`, looking this hard
    /// at `t`? One roll per looker, carrier, piece, hour and kind of look.
    pub fn notices(&self, looker: PersonId, carrier: PersonId, slot: Slot, look: Look, t: f64) -> bool {
        let p = &self.people[carrier as usize];
        let gear = p.kit();
        let Some(id) = gear.in_slot(slot) else { return false };
        let h = hidden(&gear, p.stats.skill(Skill::Sneak), slot);
        let chance = notice_chance(h, sharpness(&self.people[looker as usize]), look);
        let hour = (t / HOUR).floor() as u64;
        Rng::from_keys(&[self.seed, looker as u64, carrier as u64, slot as u64, id as u64, hour, look.tag(), 0x4849_4445]).f32() < chance
    }

    /// How dangerous `carrier` looks to `looker` sizing them up at `t`: their
    /// might, less what `looker` hasn't noticed. (Their might itself is
    /// untouched: this is only what's seen.)
    pub fn seen_might(&self, looker: PersonId, carrier: PersonId, look: Look, t: f64) -> f32 {
        let p = &self.people[carrier as usize];
        let mut gear = p.kit();
        let mut missed = false;
        for &slot in SLOTS.iter() {
            if conceal_of(&gear, slot).is_some_and(|c| c > 0.0) && !self.notices(looker, carrier, slot, look, t) {
                gear.discard(slot);
                missed = true;
            }
        }
        if !missed {
            return p.might;
        }
        let spells = match &p.detail {
            Some(d) => d.spells.clone(),
            None => super::magic::starting_spells(&p.kit_stats, p.seed),
        };
        combat::rating(p.race, &p.stats, &gear, &spells)
    }
}
