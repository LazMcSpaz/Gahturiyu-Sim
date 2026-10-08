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
//!
//! Every spell also carries one of eight **domains** — what it works on.
//! Domains have no rules of their own yet; they're tags for other things to
//! hook into. Two hooks exist: `Does::DomainPower` (an item or blessing
//! that strengthens a domain's spells) and `Does::DomainResist` (one that
//! wards against them). A shrine boosting one domain, or a birth god's
//! bonus, would add one of those.
//!
//! **The spell list is data** (`SPELLS`): each spell is its style, domain,
//! costs, aim and a list of effects from `effects`. What the effects do is
//! worked out in one place for fights (`combat`) and one for the world.

use serde::{Deserialize, Serialize};

use super::effects::{area, lasting, now, Does, Effect, Element, Reach};
use super::stats::{Attr, Skill, Stats};

/// What a spell works on. Tags for now (see the module notes).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Domain {
    Elemental,
    Psychic,
    Illusion,
    Vital,
    Warding,
    Alteration,
    Summoning,
    Necromancy,
}

pub const DOMAINS: [Domain; 8] = [Domain::Elemental, Domain::Psychic, Domain::Illusion, Domain::Vital, Domain::Warding, Domain::Alteration, Domain::Summoning, Domain::Necromancy];

impl Domain {
    pub fn name(self) -> &'static str {
        match self {
            Domain::Elemental => "Elemental",
            Domain::Psychic => "Psychic",
            Domain::Illusion => "Illusion",
            Domain::Vital => "Vital",
            Domain::Warding => "Warding",
            Domain::Alteration => "Alteration",
            Domain::Summoning => "Summoning",
            Domain::Necromancy => "Necromancy",
        }
    }
}

/// The three styles of magic.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Felt,
    Structured,
    Ritual,
}

pub const STYLES: [Style; 3] = [Style::Felt, Style::Structured, Style::Ritual];

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

/// What a spell is pointed at when it's cast.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Nothing: it works on the caster (or round them).
    Caster,
    /// An enemy within range.
    Foe,
    /// A friend (or yourself) within range.
    Friend,
    /// A spot on the ground within range.
    Point,
}

/// A spell: an index into `SPELLS`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Spell(pub u16);

#[derive(Clone, Copy, Debug)]
pub struct SpellDef {
    /// For looking it up (`spell("fireball")`) and for scrolls and notes.
    pub key: &'static str,
    pub name: &'static str,
    pub style: Style,
    pub domain: Domain,
    /// Energy (mana).
    pub cost: f32,
    /// Tiredness added (felt spells), 0..100 scale like the squad's own.
    pub tire: f32,
    /// Seconds spent casting before it goes off (structured spells).
    pub cast_time: f32,
    /// Metres.
    pub range: f32,
    pub aim: Aim,
    /// Style skill needed to have learned it at all.
    pub min_skill: f32,
    pub effects: &'static [Effect],
}

#[allow(clippy::too_many_arguments)]
const fn felt(key: &'static str, name: &'static str, domain: Domain, cost: f32, tire: f32, range: f32, aim: Aim, min_skill: f32, effects: &'static [Effect]) -> SpellDef {
    SpellDef { key, name, style: Style::Felt, domain, cost, tire, cast_time: 0.0, range, aim, min_skill, effects }
}

#[allow(clippy::too_many_arguments)]
const fn structured(key: &'static str, name: &'static str, domain: Domain, cost: f32, cast_time: f32, range: f32, aim: Aim, min_skill: f32, effects: &'static [Effect]) -> SpellDef {
    SpellDef { key, name, style: Style::Structured, domain, cost, tire: 0.0, cast_time, range, aim, min_skill, effects }
}

use Aim::*;
use Domain::*;

/// Every spell there is.
pub static SPELLS: &[SpellDef] = &[
    //          key               name              domain     cost cast  range aim     min
    structured("paralyze", "Paralyze", Psychic, 25.0, 1.4, 15.0, Foe, 35.0, &[lasting(Does::Paralyze, 1.0, 6.0, Reach::Target)]),
    structured("fireball", "Fireball", Elemental, 30.0, 1.6, 20.0, Point, 40.0, &[now(Does::Damage(Element::Fire), 18.0, area(3.0))]),
    structured("lightning_bolt", "Lightning bolt", Elemental, 22.0, 1.2, 25.0, Foe, 30.0, &[now(Does::Damage(Element::Lightning), 22.0, Reach::Target)]),
    structured("blind", "Blind", Illusion, 15.0, 1.0, 15.0, Foe, 25.0, &[lasting(Does::Blind, 0.65, 10.0, Reach::Target)]),
    structured("mage_armor", "Mage armor", Warding, 18.0, 1.0, 0.0, Caster, 25.0, &[lasting(Does::Barrier, 0.4, 30.0, Reach::Caster)]),
    structured("haste", "Enhanced speed", Vital, 15.0, 1.0, 0.0, Caster, 28.0, &[lasting(Does::Haste, 0.4, 20.0, Reach::Caster)]),
    //    key     name    domain cost tire  range aim     min
    felt("heal", "Heal", Vital, 8.0, 0.6, 10.0, Friend, 15.0, &[now(Does::Heal, 16.0, Reach::Target)]),
];

/// Look a spell up by its key. Panics on a typo, which is what tests want.
pub fn spell(key: &str) -> Spell {
    Spell(SPELLS.iter().position(|d| d.key == key).unwrap_or_else(|| panic!("no spell called {key}")) as u16)
}

/// Every spell, in list order.
pub fn all_spells() -> impl Iterator<Item = Spell> {
    (0..SPELLS.len() as u16).map(Spell)
}

impl Spell {
    pub fn def(self) -> &'static SpellDef {
        &SPELLS[self.0 as usize]
    }
}

impl SpellDef {
    pub fn skill(&self) -> Skill {
        self.style.skill()
    }

    /// Does anything that hurts or hinders whoever it reaches.
    pub fn harmful(&self) -> bool {
        self.effects.iter().any(|e| e.does.harmful())
    }

    /// Area of the first area effect, if any.
    pub fn radius(&self) -> f32 {
        self.effects.iter().find_map(|e| if let Reach::Area { radius, .. } = e.reach { Some(radius) } else { None }).unwrap_or(0.0)
    }

    /// Can be cast outside a fight (it does something there).
    pub fn works_outside_fights(&self) -> bool {
        self.effects.iter().all(|e| e.does.works_outside_fights())
    }
}

/// Spells someone knows on first meeting: every felt spell their feel for it
/// has reached, and the structured and ritual spells their skill would have
/// let them learn along the way.
pub fn starting_spells(stats: &Stats) -> Vec<Spell> {
    all_spells().filter(|s| stats.skill(s.def().skill()) >= s.def().min_skill).collect()
}

/// Felt spells come with use: the ones this skill has reached that aren't
/// known yet.
pub fn felt_reached(stats: &Stats, known: &[Spell]) -> Vec<Spell> {
    all_spells().filter(|s| s.def().style == Style::Felt && !known.contains(s) && stats.skill(Skill::Felt) >= s.def().min_skill).collect()
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

/// How strongly a caster's skill drives a spell's numbers: 0.8 at nothing,
/// 1.2 at mastery.
pub fn skill_power(stats: &Stats, spell: Spell) -> f32 {
    0.8 + stats.skill(spell.def().skill()) / 250.0
}

/// Chance the target's mind throws off a paralysis or blindness, before any
/// enchantment helps.
pub fn willpower_resist(target: &Stats) -> f32 {
    (target.attr(Attr::Willpower) / 250.0).clamp(0.0, 0.4)
}

/// A lasting effect in force on someone in a fight.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Status {
    pub does: Does,
    pub power: f32,
    pub until: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::{Race, Traits};

    fn stats(race: Race) -> Stats {
        Stats::generate(race, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 4)
    }

    #[test]
    fn skilled_casters_succeed_more() {
        let mut s = stats(Race::Tadoro);
        s.set_skill(Skill::Structured, 20.0);
        let novice = success_chance(&s, spell("fireball"), 1.0);
        s.set_skill(Skill::Structured, 80.0);
        let adept = success_chance(&s, spell("fireball"), 1.0);
        assert!(adept > novice + 0.3);
        assert!(success_chance(&s, spell("fireball"), 0.6) < adept, "tired casters fail more");
    }

    #[test]
    fn spells_are_learned_from_skill() {
        let mut s = stats(Race::Roduro);
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
        let mut s = stats(Race::Roduro);
        s.set_skill(Skill::Felt, 20.0);
        s.set_skill(Skill::Structured, 20.0);
        let heal = spell("heal");
        assert!(success_chance(&s, heal, 1.0) > 0.85);
        assert!(success_chance(&s, heal, 1.0) > success_chance(&s, spell("fireball"), 1.0) + 0.3);
        s.set_skill(Skill::Felt, 5.0);
        assert!(felt_reached(&s, &[]).is_empty());
        s.set_skill(Skill::Felt, 30.0);
        assert_eq!(felt_reached(&s, &[]), vec![heal]);
        assert!(felt_reached(&s, &[heal]).is_empty());
    }

    #[test]
    fn spell_keys_are_unique() {
        for (i, a) in SPELLS.iter().enumerate() {
            for b in &SPELLS[i + 1..] {
                assert_ne!(a.key, b.key);
            }
        }
    }
}
