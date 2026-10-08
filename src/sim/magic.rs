//! Spells. Every spell belongs to one of three **styles**, and the styles are
//! the magic skills:
//!
//! | | Felt | Structured | Ritual |
//! |---|---|---|---|
//! | Power | small | medium | large |
//! | Cast | instant, mid-fight | 1–2 s, interrupted by a solid hit | minutes to hours, never mid-fight |
//! | Cost | a little energy and a little tiredness | energy | components and/or health |
//! | Fails | rarely | sometimes (energy still spent) | backlash: components lost, caster hurt |
//! | Learned | by using the style | teacher or notes | teacher or rare texts |
//!
//! Energy is the mana pool. Casting trains the style used.

use serde::{Deserialize, Serialize};

use super::stats::{Attr, Skill, Stats};

/// The three styles of magic.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Felt,
    Structured,
    Ritual,
}

impl Style {
    /// The skill that is this style.
    pub fn skill(self) -> Skill {
        match self {
            Style::Felt => Skill::Felt,
            Style::Structured => Skill::Structured,
            Style::Ritual => Skill::Ritual,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Style::Felt => "Felt",
            Style::Structured => "Structured",
            Style::Ritual => "Ritual",
        }
    }
}

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
    pub style: Style,
    /// Energy (mana).
    pub cost: f32,
    /// Tiredness added (felt spells), 0..100 scale like the squad's own.
    pub tire: f32,
    /// Seconds spent casting before it goes off.
    pub cast_time: f32,
    /// Metres.
    pub range: f32,
    pub target: Target,
    /// Style skill needed to have learned it at all.
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
            Spell::Paralyze => SpellDef { name: "Paralyze", style: Style::Structured, cost: 25.0, tire: 0.0, cast_time: 1.4, range: 15.0, target: Target::Other, min_skill: 35.0, duration: 6.0, magnitude: 1.0, radius: 0.0 },
            Spell::Fireball => SpellDef { name: "Fireball", style: Style::Structured, cost: 30.0, tire: 0.0, cast_time: 1.6, range: 20.0, target: Target::Ground, min_skill: 40.0, duration: 0.0, magnitude: 18.0, radius: 3.0 },
            Spell::LightningBolt => SpellDef { name: "Lightning bolt", style: Style::Structured, cost: 22.0, tire: 0.0, cast_time: 1.2, range: 25.0, target: Target::Other, min_skill: 30.0, duration: 0.0, magnitude: 22.0, radius: 0.0 },
            Spell::Blind => SpellDef { name: "Blind", style: Style::Structured, cost: 15.0, tire: 0.0, cast_time: 1.0, range: 15.0, target: Target::Other, min_skill: 25.0, duration: 10.0, magnitude: 0.65, radius: 0.0 },
            Spell::MageArmor => SpellDef { name: "Mage armor", style: Style::Structured, cost: 18.0, tire: 0.0, cast_time: 1.0, range: 0.0, target: Target::Caster, min_skill: 25.0, duration: 30.0, magnitude: 0.4, radius: 0.0 },
            Spell::Heal => SpellDef { name: "Heal", style: Style::Felt, cost: 8.0, tire: 0.6, cast_time: 0.0, range: 10.0, target: Target::Ally, min_skill: 15.0, duration: 0.0, magnitude: 16.0, radius: 0.0 },
            Spell::Haste => SpellDef { name: "Enhanced speed", style: Style::Structured, cost: 15.0, tire: 0.0, cast_time: 1.0, range: 0.0, target: Target::Caster, min_skill: 28.0, duration: 20.0, magnitude: 0.4, radius: 0.0 },
        }
    }
}

impl SpellDef {
    pub fn skill(&self) -> Skill {
        self.style.skill()
    }
}

/// Spells someone knows on first meeting: every felt spell their feel for it
/// has reached, and the structured and ritual spells their skill would have
/// let them learn along the way.
pub fn starting_spells(stats: &Stats) -> Vec<Spell> {
    SPELLS.iter().copied().filter(|s| stats.skill(s.def().skill()) >= s.def().min_skill).collect()
}

/// Felt spells come with use: the ones this skill has reached that aren't
/// known yet.
pub fn felt_reached(stats: &Stats, known: &[Spell]) -> Vec<Spell> {
    SPELLS.iter().copied().filter(|s| s.def().style == Style::Felt && !known.contains(s) && stats.skill(Skill::Felt) >= s.def().min_skill).collect()
}

/// Chance a cast goes off, 0..1. `tired` is the fatigue factor (1 = fresh).
/// Felt magic rarely fails; structured magic fails more the harder the
/// spell; rituals are a gamble for the unskilled.
pub fn success_chance(stats: &Stats, spell: Spell, tired: f32) -> f32 {
    let d = spell.def();
    let skill = stats.skill(d.skill());
    let will = stats.attr(Attr::Willpower);
    match d.style {
        Style::Felt => ((0.82 + skill / 400.0 + will / 800.0) * (0.9 + 0.1 * tired)).clamp(0.6, 0.99),
        Style::Structured => ((0.25 + skill / 100.0 * 0.8 + will / 400.0 - d.cost / 200.0) * tired).clamp(0.05, 0.98),
        Style::Ritual => (0.3 + skill * 0.007 + will / 400.0 - d.min_skill / 300.0).clamp(0.05, 0.97),
    }
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
        s.set_skill(Skill::Structured, 20.0);
        let novice = success_chance(&s, Spell::Fireball, 1.0);
        s.set_skill(Skill::Structured, 80.0);
        let adept = success_chance(&s, Spell::Fireball, 1.0);
        assert!(adept > novice + 0.3);
        assert!(success_chance(&s, Spell::Fireball, 0.6) < adept, "tired casters fail more");
    }

    #[test]
    fn spells_are_learned_from_skill() {
        let mut s = Stats::generate(Race::Roduro, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 4);
        for k in crate::sim::stats::MAGIC_SKILLS {
            s.set_skill(k, 5.0);
        }
        assert!(starting_spells(&s).is_empty());
        for k in crate::sim::stats::MAGIC_SKILLS {
            s.set_skill(k, 40.0);
        }
        assert_eq!(starting_spells(&s).len(), 7);
    }

    #[test]
    fn felt_magic_rarely_fails_and_comes_with_use() {
        let mut s = Stats::generate(Race::Roduro, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 4);
        s.set_skill(Skill::Felt, 20.0);
        s.set_skill(Skill::Structured, 20.0);
        assert!(success_chance(&s, Spell::Heal, 1.0) > 0.85);
        assert!(success_chance(&s, Spell::Heal, 1.0) > success_chance(&s, Spell::Fireball, 1.0) + 0.3);
        s.set_skill(Skill::Felt, 5.0);
        assert!(felt_reached(&s, &[]).is_empty());
        s.set_skill(Skill::Felt, 30.0);
        assert_eq!(felt_reached(&s, &[]), vec![Spell::Heal]);
        assert!(felt_reached(&s, &[Spell::Heal]).is_empty());
    }
}
