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

use super::effects::{area, lasting, now, Does, Effect, Element, Reach, Summon, Who};
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
    /// Anyone within range, friend or foe.
    Anyone,
    /// A door within range.
    Door,
    /// A body within range.
    Corpse,
}

/// Where a ritual has to be performed.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Anywhere,
    /// Beside a town's hearth.
    Hearth,
    /// At a shrine. (For now the Qotiro temples and halls are the only holy
    /// buildings in the world.)
    Shrine,
    /// Inside a drawn circle: drawing one adds `CIRCLE_MINUTES`, and it stays
    /// on the ground for next time.
    Circle,
}

impl Place {
    pub fn name(self) -> &'static str {
        match self {
            Place::Anywhere => "anywhere",
            Place::Hearth => "at a hearth",
            Place::Shrine => "at a shrine",
            Place::Circle => "in a drawn circle",
        }
    }
}

/// What a ritual takes beyond skill: time, blood, components, a place.
#[derive(Clone, Copy, Debug)]
pub struct Rite {
    /// Game minutes to perform.
    pub minutes: f32,
    /// Health given (a wound spread over the body, healing like any other).
    pub health: f32,
    /// Used up whether it works or not: (item key, how many).
    pub components: &'static [(&'static str, u16)],
    pub place: Place,
}

/// Not a ritual.
pub const NO_RITE: Rite = Rite { minutes: 0.0, health: 0.0, components: &[], place: Place::Anywhere };

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
    /// For rituals: what performing it takes.
    pub rite: Rite,
}

#[allow(clippy::too_many_arguments)]
const fn felt(key: &'static str, name: &'static str, domain: Domain, cost: f32, tire: f32, range: f32, aim: Aim, min_skill: f32, effects: &'static [Effect]) -> SpellDef {
    SpellDef { key, name, style: Style::Felt, domain, cost, tire, cast_time: 0.0, range, aim, min_skill, effects, rite: NO_RITE }
}

#[allow(clippy::too_many_arguments)]
const fn structured(key: &'static str, name: &'static str, domain: Domain, cost: f32, cast_time: f32, range: f32, aim: Aim, min_skill: f32, effects: &'static [Effect]) -> SpellDef {
    SpellDef { key, name, style: Style::Structured, domain, cost, tire: 0.0, cast_time, range, aim, min_skill, effects, rite: NO_RITE }
}

/// A ritual: performed over minutes or hours (never mid-fight), then held
/// ready and released when wanted. `range` and `aim` are for the release.
#[allow(clippy::too_many_arguments)]
const fn ritual(key: &'static str, name: &'static str, domain: Domain, rite: Rite, range: f32, aim: Aim, min_skill: f32, effects: &'static [Effect]) -> SpellDef {
    SpellDef { key, name, style: Style::Ritual, domain, cost: 0.0, tire: 0.0, cast_time: 0.0, range, aim, min_skill, effects, rite }
}

const fn rite(minutes: f32, health: f32, components: &'static [(&'static str, u16)], place: Place) -> Rite {
    Rite { minutes, health, components, place }
}

/// The whole squad, near enough.
const SQUAD: Reach = Reach::Area { radius: 40.0, who: Who::Friends };

use Aim::*;
use Domain::*;

/// Every spell there is.
pub static SPELLS: &[SpellDef] = &[
    // ---- Elemental --------------------------------------------------------
    //    key       name      domain     cost tire range aim    min
    felt("spark", "Spark", Elemental, 5.0, 0.4, 12.0, Foe, 5.0, &[now(Does::Damage(Element::Fire), 7.0, Reach::Target)]),
    felt("chill", "Chill", Elemental, 6.0, 0.5, 12.0, Foe, 12.0, &[now(Does::Damage(Element::Frost), 4.0, Reach::Target), lasting(Does::Slow, 0.35, 8.0, Reach::Target)]),
    felt("kindle", "Kindle", Elemental, 3.0, 0.2, 15.0, Point, 3.0, &[now(Does::Kindle, 1.0, Reach::Object)]),
    felt("douse", "Douse", Elemental, 4.0, 0.3, 15.0, Point, 6.0, &[now(Does::Douse, 1.0, Reach::Object)]),
    felt("drench", "Drench", Elemental, 5.0, 0.4, 12.0, Foe, 14.0, &[lasting(Does::Wet, 1.0, 600.0, Reach::Target)]),
    //          key               name              domain     cost cast  range aim     min
    structured("fireball", "Fireball", Elemental, 30.0, 1.6, 20.0, Point, 40.0, &[now(Does::Damage(Element::Fire), 18.0, area(3.0))]),
    structured("lightning_bolt", "Lightning bolt", Elemental, 22.0, 1.2, 25.0, Foe, 30.0, &[now(Does::Damage(Element::Lightning), 22.0, Reach::Target)]),
    structured("stone_spikes", "Stone spikes", Elemental, 24.0, 1.4, 15.0, Point, 32.0, &[now(Does::Damage(Element::Stone), 20.0, area(2.5))]),
    //      key          name          domain     rite: minutes health components                       place           range aim    min
    ritual("firestorm", "Firestorm", Elemental, rite(60.0, 10.0, &[("emberroot", 3)], Place::Circle), 25.0, Point, 45.0, &[now(Does::Damage(Element::Fire), 32.0, area(6.0))]),
    ritual("tremor", "Tremor", Elemental, rite(45.0, 0.0, &[("salt_crystal", 2), ("iron_ore", 1)], Place::Anywhere), 0.0, Caster, 40.0, &[
        now(Does::Damage(Element::Stone), 8.0, Reach::Area { radius: 9.0, who: Who::Foes }),
        lasting(Does::KnockDown, 1.0, 4.0, Reach::Area { radius: 9.0, who: Who::Foes }),
    ]),
    // ---- Psychic ----------------------------------------------------------
    felt("daze", "Daze", Psychic, 6.0, 0.5, 10.0, Foe, 10.0, &[now(Does::Daze, 1.2, Reach::Target)]),
    felt("sense_life", "Sense life", Psychic, 4.0, 0.3, 0.0, Caster, 12.0, &[lasting(Does::SenseLife, 60.0, 180.0, Reach::Caster)]),
    felt("nightsight", "Nightsight", Psychic, 4.0, 0.3, 0.0, Caster, 10.0, &[lasting(Does::Nightsight, 0.75, 900.0, Reach::Caster)]),
    structured("calm", "Calm", Psychic, 20.0, 1.2, 15.0, Foe, 28.0, &[lasting(Does::Calm, 1.0, 15.0, Reach::Target)]),
    structured("fear", "Fear", Psychic, 20.0, 1.2, 15.0, Foe, 30.0, &[lasting(Does::Fear, 1.0, 10.0, Reach::Target)]),
    structured("paralyze", "Paralyze", Psychic, 25.0, 1.4, 15.0, Foe, 35.0, &[lasting(Does::Paralyze, 1.0, 6.0, Reach::Target)]),
    structured("sway", "Sway", Psychic, 15.0, 1.5, 0.0, Caster, 20.0, &[lasting(Does::Sway, 20.0, 600.0, Reach::Caster)]),
    ritual("dominate", "Dominate", Psychic, rite(60.0, 15.0, &[("storm_glass", 1)], Place::Circle), 12.0, Foe, 50.0, &[lasting(Does::Dominate, 1.0, 20.0, Reach::Target)]),
    // ---- Illusion ---------------------------------------------------------
    felt("silent_step", "Silent step", Illusion, 4.0, 0.3, 0.0, Caster, 8.0, &[lasting(Does::Silent, 0.6, 300.0, Reach::Caster)]),
    felt("glow", "Glow", Illusion, 3.0, 0.2, 0.0, Caster, 4.0, &[lasting(Does::Glow, 0.6, 600.0, Reach::Caster)]),
    felt("gloom", "Gloom", Illusion, 4.0, 0.3, 0.0, Caster, 10.0, &[lasting(Does::Gloom, 0.5, 300.0, Reach::Caster)]),
    structured("blind", "Blind", Illusion, 15.0, 1.0, 15.0, Foe, 25.0, &[lasting(Does::Blind, 0.65, 10.0, Reach::Target)]),
    structured("hide", "Hide", Illusion, 18.0, 1.5, 0.0, Caster, 28.0, &[lasting(Does::Hide, 0.7, 60.0, Reach::Caster)]),
    structured("decoy", "Decoy", Illusion, 20.0, 1.2, 12.0, Point, 25.0, &[lasting(Does::Summon(Summon::Decoy), 1.0, 20.0, Reach::Object)]),
    structured("disguise", "Disguise", Illusion, 20.0, 2.0, 0.0, Caster, 30.0, &[lasting(Does::Disguise, 1.0, 1800.0, Reach::Caster)]),
    ritual("veil", "Veil", Illusion, rite(30.0, 0.0, &[("ash_moss", 3)], Place::Anywhere), 0.0, Caster, 35.0, &[lasting(Does::Veil, 1.0, 6.0 * 3600.0, Reach::Ground { radius: 30.0 })]),
    // ---- Vital ------------------------------------------------------------
    felt("mend", "Mend", Vital, 8.0, 0.6, 10.0, Friend, 15.0, &[now(Does::Heal, 16.0, Reach::Target)]),
    felt("second_wind", "Second wind", Vital, 6.0, 0.6, 8.0, Friend, 10.0, &[now(Does::Stamina, 50.0, Reach::Target)]),
    structured("haste", "Haste", Vital, 15.0, 1.0, 0.0, Caster, 28.0, &[lasting(Does::Haste, 0.4, 20.0, Reach::Caster)]),
    structured("might", "Might", Vital, 20.0, 1.4, 8.0, Friend, 25.0, &[lasting(Does::Attr(Attr::Strength), 15.0, 600.0, Reach::Target), lasting(Does::Carry, 25.0, 600.0, Reach::Target)]),
    structured("toughen", "Toughen", Vital, 18.0, 1.2, 0.0, Caster, 22.0, &[lasting(Does::Toughen, 0.25, 60.0, Reach::Caster)]),
    ritual("restore", "Restore", Vital, rite(40.0, 0.0, &[("ghostcap", 2), ("kelp_frond", 2)], Place::Hearth), 0.0, Caster, 30.0, &[now(Does::Heal, 400.0, SQUAD), now(Does::Rest, 100.0, SQUAD)]),
    ritual("sustain", "Sustain", Vital, rite(30.0, 0.0, &[("salted_meat", 1), ("ghostcap", 1)], Place::Anywhere), 0.0, Caster, 35.0, &[lasting(Does::Sustain, 1.0, 24.0 * 3600.0, SQUAD)]),
    ritual("regrow", "Regrow", Vital, rite(180.0, 20.0, &[("storm_glass", 2), ("ghostcap", 4), ("emberroot", 2)], Place::Shrine), 3.0, Friend, 55.0, &[now(Does::Regrow, 1.0, Reach::Target)]),
    // ---- Warding ----------------------------------------------------------
    felt("brace", "Brace", Warding, 5.0, 0.4, 0.0, Caster, 8.0, &[lasting(Does::Brace, 1.0, 30.0, Reach::Caster)]),
    felt("tripwire", "Tripwire", Warding, 6.0, 0.4, 0.0, Caster, 12.0, &[lasting(Does::Tripwire, 1.0, 8.0 * 3600.0, Reach::Ground { radius: 25.0 })]),
    structured("resist", "Resist", Warding, 15.0, 1.2, 8.0, Friend, 20.0, &[lasting(Does::ResistElements, 0.5, 120.0, Reach::Target)]),
    structured("barrier", "Barrier", Warding, 18.0, 1.0, 0.0, Caster, 25.0, &[lasting(Does::Barrier, 0.4, 30.0, Reach::Caster)]),
    structured("dispel", "Dispel", Warding, 20.0, 1.4, 15.0, Anyone, 30.0, &[now(Does::Dispel, 1.0, Reach::Target)]),
    ritual("sanctuary", "Sanctuary", Warding, rite(60.0, 0.0, &[("salt_crystal", 4)], Place::Circle), 0.0, Caster, 40.0, &[lasting(Does::Sanctuary, 1.0, 8.0 * 3600.0, Reach::Ground { radius: 12.0 })]),
    // ---- Alteration -------------------------------------------------------
    felt("lighten", "Lighten", Alteration, 5.0, 0.4, 6.0, Friend, 6.0, &[lasting(Does::Lighten, 0.35, 3600.0, Reach::Target)]),
    felt("burden", "Burden", Alteration, 6.0, 0.4, 10.0, Foe, 10.0, &[lasting(Does::Burden, 0.5, 20.0, Reach::Target)]),
    structured("shatter", "Shatter item", Alteration, 25.0, 1.6, 10.0, Foe, 35.0, &[now(Does::Shatter, 1.0, Reach::Target)]),
    structured("unlock", "Unlock", Alteration, 12.0, 1.5, 6.0, Door, 20.0, &[now(Does::Unlock, 1.0, Reach::Object)]),
    structured("shrink", "Shrink", Alteration, 18.0, 1.3, 12.0, Foe, 25.0, &[lasting(Does::Shrink, 0.35, 20.0, Reach::Target)]),
    structured("enlarge", "Enlarge", Alteration, 18.0, 1.3, 8.0, Friend, 25.0, &[lasting(Does::Enlarge, 0.3, 30.0, Reach::Target)]),
    structured("rust", "Rust", Alteration, 18.0, 1.2, 12.0, Foe, 22.0, &[lasting(Does::Rust, 0.5, 60.0, Reach::Target)]),
    ritual("transmute", "Transmute", Alteration, rite(30.0, 6.0, &[("ash_moss", 1)], Place::Circle), 0.0, Caster, 40.0, &[now(Does::Transmute, 5.0, Reach::Object)]),
    // ---- Summoning --------------------------------------------------------
    felt("wisp", "Wisp", Summoning, 4.0, 0.3, 20.0, Point, 5.0, &[lasting(Does::Glow, 0.6, 600.0, Reach::Ground { radius: 14.0 })]),
    felt("scout", "Scout", Summoning, 6.0, 0.5, 150.0, Point, 12.0, &[lasting(Does::Scout, 1.0, 180.0, Reach::Ground { radius: 0.0 })]),
    structured("spirit_beast", "Spirit beast", Summoning, 30.0, 2.0, 6.0, Point, 35.0, &[lasting(Does::Summon(Summon::SpiritBeast), 1.0, 60.0, Reach::Object)]),
    structured("pack_spirit", "Pack spirit", Summoning, 15.0, 2.0, 0.0, Caster, 20.0, &[lasting(Does::Carry, 40.0, 4.0 * 3600.0, Reach::Caster)]),
    ritual("guardian", "Guardian", Summoning, rite(45.0, 8.0, &[("iron_ingot", 1), ("salt_crystal", 2)], Place::Circle), 0.0, Caster, 45.0, &[lasting(Does::Summon(Summon::Guardian), 1.0, 8.0 * 3600.0, Reach::Ground { radius: 30.0 })]),
    ritual("swarm", "Swarm", Summoning, rite(40.0, 12.0, &[("ash_moss", 2), ("kelp_frond", 2)], Place::Anywhere), 15.0, Point, 40.0, &[lasting(Does::Summon(Summon::Swarmling), 1.0, 30.0, Reach::Object)]),
    // ---- Necromancy -------------------------------------------------------
    felt("drain", "Drain", Necromancy, 6.0, 0.6, 6.0, Foe, 10.0, &[now(Does::Drain, 6.0, Reach::Target)]),
    felt("preserve", "Preserve", Necromancy, 4.0, 0.3, 8.0, Point, 5.0, &[now(Does::Preserve, 1.0, Reach::Object)]),
    structured("wither", "Wither", Necromancy, 22.0, 1.4, 10.0, Foe, 30.0, &[now(Does::Wither, 22.0, Reach::Target)]),
    structured("raise_thrall", "Raise thrall", Necromancy, 28.0, 2.0, 10.0, Corpse, 35.0, &[lasting(Does::Raise, 1.0, 45.0, Reach::Object)]),
    ritual("grave_call", "Grave call", Necromancy, rite(60.0, 15.0, &[("ash_moss", 2), ("salt_crystal", 2)], Place::Circle), 0.0, Caster, 50.0, &[lasting(Does::Raise, 1.0, 60.0, Reach::Area { radius: 20.0, who: Who::All })]),
    ritual("blight", "Blight", Necromancy, rite(50.0, 10.0, &[("ghostcap", 3)], Place::Anywhere), 20.0, Point, 45.0, &[lasting(Does::Blight, 3.0, 30.0, Reach::Ground { radius: 6.0 })]),
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
        self.effects
            .iter()
            .find_map(|e| match e.reach {
                Reach::Area { radius, .. } | Reach::Ground { radius } => Some(radius),
                _ => None,
            })
            .unwrap_or(0.0)
    }

    /// Can be cast outside a fight (it does something there).
    pub fn works_outside_fights(&self) -> bool {
        self.effects.iter().all(|e| e.does.works_outside_fights())
    }
}

/// Share of the structured spells and rituals within someone's skill that
/// they picked up before you met them (from teachers and notes along the way).
pub const KNOWN_SHARE: f32 = 0.45;

/// Spells someone knows on first meeting: every felt spell their feel for it
/// has reached, and some of the structured spells and rituals their skill
/// would let them follow — which ones is down to their seed.
pub fn starting_spells(stats: &Stats, seed: u64) -> Vec<Spell> {
    all_spells()
        .filter(|s| stats.skill(s.def().skill()) >= s.def().min_skill)
        .filter(|s| s.def().style == Style::Felt || super::rng::Rng::from_keys(&[seed, s.0 as u64, 0x4C45_524E]).f32() < KNOWN_SHARE)
        .collect()
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
            s.set_skill(k, 2.0);
        }
        assert!(starting_spells(&s, 1).is_empty());
        for k in crate::sim::stats::MAGIC_SKILLS {
            s.set_skill(k, 40.0);
        }
        // Every felt spell within reach; only some of the rest.
        let within = |st: Style| all_spells().filter(|x| x.def().min_skill <= 40.0 && x.def().style == st).count();
        let mut counts = Vec::new();
        for seed in 0..40 {
            let known = starting_spells(&s, seed);
            assert_eq!(known.iter().filter(|x| x.def().style == Style::Felt).count(), within(Style::Felt));
            counts.push(known.iter().filter(|x| x.def().style == Style::Structured).count());
        }
        let avg = counts.iter().sum::<usize>() as f32 / counts.len() as f32;
        assert!(avg > within(Style::Structured) as f32 * 0.3 && avg < within(Style::Structured) as f32 * 0.6, "{avg}");
    }

    #[test]
    fn felt_magic_rarely_fails_and_comes_with_use() {
        let mut s = stats(Race::Roduro);
        s.set_skill(Skill::Felt, 20.0);
        s.set_skill(Skill::Structured, 20.0);
        let heal = spell("mend");
        assert!(success_chance(&s, heal, 1.0) > 0.85);
        assert!(success_chance(&s, heal, 1.0) > success_chance(&s, spell("fireball"), 1.0) + 0.3);
        s.set_skill(Skill::Felt, 2.0);
        assert!(felt_reached(&s, &[]).is_empty());
        s.set_skill(Skill::Felt, 30.0);
        let reached = felt_reached(&s, &[]);
        assert!(reached.contains(&heal));
        assert!(felt_reached(&s, &reached).is_empty());
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
