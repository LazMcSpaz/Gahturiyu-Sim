//! Spells, Morrowind-style: each belongs to a school, costs mana, takes a
//! moment to cast, and can fail — and a failed cast still spends the mana.
//! How likely a cast is to work depends on the school's skill, willpower and
//! how tired the caster is. Casting trains the school.
//!
//! Six to start with:
//!
//! | Spell | School | Does |
//! |---|---|---|
//! | Paralyze | Illusion | Target can't move or act for a few seconds (willpower may resist) |
//! | Fireball | Destruction | Burst of fire at a point; hurts everyone nearby, friend or foe |
//! | Lightning bolt | Destruction | Heavy damage to one target; armour doesn't help |
//! | Blind | Illusion | Target's attacks mostly miss, and it can't cast at range |
//! | Mage armor | Alteration | Caster takes much less damage for a while |
//! | Enhanced speed | Alteration | Caster moves and attacks faster for a while |
//! | Heal | Restoration | Mends an ally's (or your own) worst wounds; can get the downed back up |

use serde::{Deserialize, Serialize};

use super::stats::{Attr, Skill, Stats};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spell {
    Paralyze,
    Fireball,
    LightningBolt,
    Blind,
    MageArmor,
    Haste,
    Heal,
}

pub const SPELLS: [Spell; 7] = [Spell::Paralyze, Spell::Fireball, Spell::LightningBolt, Spell::Blind, Spell::MageArmor, Spell::Haste, Spell::Heal];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Target {
    /// The caster.
    Caster,
    /// One other person, within range.
    Other,
    /// A friend (or yourself), within range.
    Ally,
    /// A point on the ground, within range.
    Ground,
}

#[derive(Clone, Copy, Debug)]
pub struct SpellDef {
    pub name: &'static str,
    pub school: Skill,
    pub cost: f32,
    /// Seconds spent casting before it goes off.
    pub cast_time: f32,
    /// Metres.
    pub range: f32,
    pub target: Target,
    /// School skill needed to have learned it at all.
    pub min_skill: f32,
    /// Seconds an effect lasts (0 = instant).
    pub duration: f32,
    /// Damage for attacks; strength of the effect otherwise.
    pub magnitude: f32,
    /// Blast radius for area spells.
    pub radius: f32,
}

impl Spell {
    pub fn def(self) -> SpellDef {
        match self {
            Spell::Paralyze => SpellDef { name: "Paralyze", school: Skill::Illusion, cost: 25.0, cast_time: 0.8, range: 15.0, target: Target::Other, min_skill: 35.0, duration: 6.0, magnitude: 1.0, radius: 0.0 },
            Spell::Fireball => SpellDef { name: "Fireball", school: Skill::Destruction, cost: 30.0, cast_time: 1.0, range: 20.0, target: Target::Ground, min_skill: 40.0, duration: 0.0, magnitude: 18.0, radius: 3.0 },
            Spell::LightningBolt => SpellDef { name: "Lightning bolt", school: Skill::Destruction, cost: 22.0, cast_time: 0.7, range: 25.0, target: Target::Other, min_skill: 30.0, duration: 0.0, magnitude: 22.0, radius: 0.0 },
            Spell::Blind => SpellDef { name: "Blind", school: Skill::Illusion, cost: 15.0, cast_time: 0.6, range: 15.0, target: Target::Other, min_skill: 25.0, duration: 10.0, magnitude: 0.65, radius: 0.0 },
            Spell::MageArmor => SpellDef { name: "Mage armor", school: Skill::Alteration, cost: 18.0, cast_time: 0.6, range: 0.0, target: Target::Caster, min_skill: 25.0, duration: 30.0, magnitude: 0.4, radius: 0.0 },
            Spell::Heal => SpellDef { name: "Heal", school: Skill::Restoration, cost: 20.0, cast_time: 0.9, range: 10.0, target: Target::Ally, min_skill: 25.0, duration: 0.0, magnitude: 30.0, radius: 0.0 },
            Spell::Haste => SpellDef { name: "Enhanced speed", school: Skill::Alteration, cost: 15.0, cast_time: 0.5, range: 0.0, target: Target::Caster, min_skill: 28.0, duration: 20.0, magnitude: 0.4, radius: 0.0 },
        }
    }
}

/// Spells someone knows on first meeting: everything their schools are good
/// enough for. (Later, spells can be bought or taught.)
pub fn starting_spells(stats: &Stats) -> Vec<Spell> {
    SPELLS.iter().copied().filter(|s| stats.skill(s.def().school) >= s.def().min_skill).collect()
}

/// Chance a cast goes off, 0..1. `tired` is the fatigue factor (1 = fresh).
pub fn success_chance(stats: &Stats, spell: Spell, tired: f32) -> f32 {
    let d = spell.def();
    let base = 0.25 + stats.skill(d.school) / 100.0 * 0.8 + stats.attr(Attr::Willpower) / 400.0 - d.cost / 200.0;
    (base * tired).clamp(0.05, 0.98)
}

/// Chance the target's mind throws off a paralysis or blindness, before any
/// enchantment helps.
pub fn willpower_resist(target: &Stats) -> f32 {
    (target.attr(Attr::Willpower) / 250.0).clamp(0.0, 0.4)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatusKind {
    Paralyzed,
    Blinded,
    MageArmor,
    Hasted,
}

/// A spell effect in force on someone.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Status {
    pub kind: StatusKind,
    pub until: f64,
    pub magnitude: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::{Race, Traits};

    #[test]
    fn skilled_casters_succeed_more() {
        let mut s = Stats::generate(Race::Tadoro, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 4);
        s.set_skill(Skill::Destruction, 20.0);
        let novice = success_chance(&s, Spell::Fireball, 1.0);
        s.set_skill(Skill::Destruction, 80.0);
        let adept = success_chance(&s, Spell::Fireball, 1.0);
        assert!(adept > novice + 0.3);
        assert!(success_chance(&s, Spell::Fireball, 0.6) < adept, "tired casters fail more");
    }

    #[test]
    fn spells_are_learned_from_skill() {
        let mut s = Stats::generate(Race::Roduro, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 4);
        for k in [Skill::Destruction, Skill::Alteration, Skill::Illusion, Skill::Restoration] {
            s.set_skill(k, 5.0);
        }
        assert!(starting_spells(&s).is_empty());
        for k in [Skill::Destruction, Skill::Alteration, Skill::Illusion, Skill::Restoration] {
            s.set_skill(k, 40.0);
        }
        assert_eq!(starting_spells(&s).len(), 7);
    }
}
