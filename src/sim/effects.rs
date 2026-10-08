//! One list of effects for everything magical: spells, scrolls, potions and
//! worn items all say what they do with the same pieces.
//!
//! An effect is four things:
//!
//! - **what it does** (`Does`): hurt, mend, hold still, quicken, add to an
//!   attribute...
//! - **how strong** (`power`): damage or healing in hit points, a share
//!   (0.4 = 40%), or points of a stat — whatever that kind of effect counts in;
//! - **how long** (`Lasts`): at once, for so many seconds, or for as long as
//!   an item is worn;
//! - **who or what it reaches** (`Reach`): the caster, one target, everyone
//!   in an area, an object, or a spot on the ground.
//!
//! The rules for each kind of effect live in one place per setting: in a
//! fight (`combat::Battle::apply`) and out in the world (`World::apply_effect`).
//! So adding a spell, potion or enchanted item is a line of data.

use serde::{Deserialize, Serialize};

use super::magic::Domain;
use super::stats::{Attr, Skill};

/// What kind of harm a damaging effect does.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Element {
    /// Burns. Armour helps a little (it's mostly heat).
    Fire,
    /// Arcs straight through armour.
    Lightning,
}

impl Element {
    pub fn name(self) -> &'static str {
        match self {
            Element::Fire => "fire",
            Element::Lightning => "lightning",
        }
    }
}

/// What an effect does.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Does {
    // ---- At once -------------------------------------------------------
    /// Hurt, spread over the whole body. Power: hit points.
    Damage(Element),
    /// Mend the worst wounds first; can get the downed back up. Power: hit points.
    Heal,
    /// Restore energy (mana). Power: points.
    Energy,

    // ---- While it lasts (or while worn) --------------------------------
    /// Power: points added.
    Attr(Attr),
    Skill(Skill),
    /// More energy (mana) at most. Power: points.
    MaxEnergy,
    /// Energy regained per game minute. Power: points.
    EnergyRegen,
    /// Extra carrying capacity. Power: kg.
    Carry,
    /// Footspeed. Power: share (0.1 = 10% faster).
    MoveSpeed,
    /// Chance to shrug off paralysis outright. Power: share.
    ResistParalysis,
    /// Chance to shrug off blindness outright. Power: share.
    ResistBlind,
    /// Share of elemental damage ignored.
    ResistElements,
    /// Spells of this domain cast by the bearer are stronger by this share.
    DomainPower(Domain),
    /// Spells of this domain do this share less to the bearer.
    DomainResist(Domain),
    /// Can't move or act. (Willpower may throw it off.)
    Paralyze,
    /// Attacks mostly miss, and no spells beyond arm's reach. Power: how blind, 0..1.
    Blind,
    /// Takes this share less damage.
    Barrier,
    /// Moves and attacks faster by this share.
    Haste,
}

impl Does {
    /// Something done to an enemy (a target can try to throw it off).
    pub fn harmful(self) -> bool {
        matches!(self, Does::Damage(_) | Does::Paralyze | Does::Blind)
    }

    /// A lasting condition that a resist roll can stop, and what resists it.
    pub fn resisted_by(self) -> Option<Does> {
        match self {
            Does::Paralyze => Some(Does::ResistParalysis),
            Does::Blind => Some(Does::ResistBlind),
            _ => None,
        }
    }

    /// Has a meaning outside a fight (the rest only matter in one).
    pub fn works_outside_fights(self) -> bool {
        matches!(self, Does::Heal | Does::Energy)
    }
}

/// How long an effect lasts.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Lasts {
    /// Done at once.
    Now,
    /// For this many game seconds.
    Secs(f32),
    /// For as long as the item is worn.
    Worn,
}

/// Who an area effect touches, from the caster's side of things.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    All,
    Foes,
    Friends,
}

/// Who or what an effect reaches.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Reach {
    /// The caster (or the wearer, or whoever drinks it).
    Caster,
    /// The one person the spell is aimed at.
    Target,
    /// Everyone of `who` within `radius` metres of where it's aimed (nearer
    /// the middle, harder).
    Area { radius: f32, who: Who },
    /// A thing (a torch, a door, an item).
    Object,
    /// A spot on the ground, for as long as it lasts.
    Ground { radius: f32 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Effect {
    pub does: Does,
    pub power: f32,
    pub lasts: Lasts,
    pub reach: Reach,
}

/// Done at once.
pub const fn now(does: Does, power: f32, reach: Reach) -> Effect {
    Effect { does, power, lasts: Lasts::Now, reach }
}

/// Lasting `secs` game seconds.
pub const fn lasting(does: Does, power: f32, secs: f32, reach: Reach) -> Effect {
    Effect { does, power, lasts: Lasts::Secs(secs), reach }
}

/// On an item, for as long as it's worn.
pub const fn worn(does: Does, power: f32) -> Effect {
    Effect { does, power, lasts: Lasts::Worn, reach: Reach::Caster }
}

/// An area, everyone in it.
pub const fn area(radius: f32) -> Reach {
    Reach::Area { radius, who: Who::All }
}

/// Add up the power of every effect that does `does` (for worn effects).
pub fn total(effects: impl Iterator<Item = (Does, f32)>, does: Does) -> f32 {
    effects.filter(|(d, _)| *d == does).map(|(_, p)| p).sum()
}

impl Effect {
    /// One line for the window, e.g. "+5 Agility" or "Heals 25".
    pub fn describe(&self) -> String {
        let p = self.power;
        let pct = p * 100.0;
        let what = match self.does {
            Does::Damage(e) => format!("{p:.0} {} damage", e.name()),
            Does::Heal => format!("Heals {p:.0}"),
            Does::Energy => format!("Restores {p:.0} energy"),
            Does::Attr(a) => format!("{p:+.0} {}", a.name()),
            Does::Skill(k) => format!("{p:+.0} {}", k.name()),
            Does::MaxEnergy => format!("{p:+.0} energy"),
            Does::EnergyRegen => format!("{p:+.1} energy a minute"),
            Does::Carry => format!("{p:+.0} kg carrying"),
            Does::MoveSpeed => format!("{pct:+.0}% speed"),
            Does::ResistParalysis => format!("Resist paralysis {pct:.0}%"),
            Does::ResistBlind => format!("Resist blindness {pct:.0}%"),
            Does::ResistElements => format!("Resist the elements {pct:.0}%"),
            Does::DomainPower(d) => format!("{pct:+.0}% {} magic", d.name()),
            Does::DomainResist(d) => format!("Resist {} magic {pct:.0}%", d.name().to_lowercase()),
            Does::Paralyze => "Paralyzes".to_string(),
            Does::Blind => "Blinds".to_string(),
            Does::Barrier => format!("{pct:.0}% less damage taken"),
            Does::Haste => format!("{pct:.0}% faster"),
        };
        let how_long = match self.lasts {
            Lasts::Secs(s) if s >= 3600.0 => format!(" for {:.0} h", s / 3600.0),
            Lasts::Secs(s) if s >= 60.0 => format!(" for {:.0} min", s / 60.0),
            Lasts::Secs(s) => format!(" for {s:.0} s"),
            _ => String::new(),
        };
        let reach = match self.reach {
            Reach::Area { radius, who } => format!(
                ", {} within {radius:.0} m",
                match who {
                    Who::All => "everyone",
                    Who::Foes => "enemies",
                    Who::Friends => "friends",
                }
            ),
            Reach::Ground { radius } => format!(", over {radius:.0} m"),
            _ => String::new(),
        };
        format!("{what}{how_long}{reach}")
    }
}
