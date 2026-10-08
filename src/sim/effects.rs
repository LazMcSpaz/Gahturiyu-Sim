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
    /// Bites with cold. Armour helps a little.
    Frost,
    /// Arcs straight through armour.
    Lightning,
    /// Stone thrust up from the ground: hits the legs, and armour there helps.
    Stone,
    /// Rot: nothing wards it but a ward against its domain.
    Rot,
}

impl Element {
    /// Fire, frost and lightning are the elements that elemental wards stop.
    pub fn elemental(self) -> bool {
        matches!(self, Element::Fire | Element::Frost | Element::Lightning)
    }

    pub fn name(self) -> &'static str {
        match self {
            Element::Fire => "fire",
            Element::Frost => "frost",
            Element::Lightning => "lightning",
            Element::Stone => "stone",
            Element::Rot => "rot",
        }
    }
}

/// Creatures a spell can call up (or raise) to fight on the caster's side.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Summon {
    /// A spirit in a beast's shape: claws and speed.
    SpiritBeast,
    /// One of a swarm: small, quick, weak, many.
    Swarmling,
    /// An illusion of the caster: draws attacks, does nothing.
    Decoy,
    /// A heavy warden that holds a spot.
    Guardian,
    /// A corpse raised to fight, mindless.
    Thrall,
}

impl Summon {
    pub fn name(self) -> &'static str {
        match self {
            Summon::SpiritBeast => "Spirit beast",
            Summon::Swarmling => "Swarm",
            Summon::Decoy => "Decoy",
            Summon::Guardian => "Guardian",
            Summon::Thrall => "Thrall",
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
    /// Take tiredness away (out of a fight; points on the 0..100 scale) or
    /// restore all stamina (in one).
    Rest,
    /// Give back stamina. Power: points.
    Stamina,
    /// Grow back a limb lost for good (the only cure).
    Regrow,
    /// End every spell on them (a called-up creature is sent back).
    Dispel,
    /// Break the weapon in their hand (or their shield): it's gone for good.
    Shatter,
    /// Open a locked door (it stays open until the next night).
    Unlock,
    /// Turn materials in the caster's pack into others (`casting::TRANSMUTE`).
    /// Power: how many lots.
    Transmute,
    /// Light a torch (one carried, or a doused campfire).
    Kindle,
    /// Put out a torch, a standing torch or a campfire (for `DOUSE_HOURS`).
    Douse,
    /// Lose the next action (whatever they were doing is spoilt).
    Daze,
    /// Call up creatures to fight for the caster for as long as it lasts.
    /// Power: how strong they are (1 = as made).
    Summon(Summon),

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
    /// Moves slower by this share, and attacks a little slower.
    Slow,
    /// Knocked off their feet: can't move or act, can't dodge or block.
    KnockDown,
    /// Won't attack anyone (a blow breaks it).
    Calm,
    /// Runs from the fight while it lasts.
    Fear,
    /// Fights for the caster's side while it lasts.
    Dominate,
    /// Sees in the dark (light counts as at least this).
    Nightsight,
    /// Senses living things nearby, through walls (shown by the window).
    SenseLife,
    /// Words land better: this much on everyone's disposition.
    Sway,
    /// Steps make this share less noise.
    Silent,
    /// Gives off light round the bearer (or at a spot on the ground).
    Glow,
    /// Darkens the ground round the bearer.
    Gloom,
    /// This share harder to see; enemies lose track of them beyond arm's reach.
    Hide,
    /// Not recognised: no one counts your bounty against you, and crimes
    /// seen aren't laid at your door.
    Disguise,
    /// Ground: those inside go unnoticed by passers-by and lookouts.
    Veil,
    /// Skin as tough as light armour: stops this share of cut damage
    /// (a little less of blunt), everywhere.
    Toughen,
    /// No hunger, and no tiredness building up.
    Sustain,
    /// What they carry weighs this share less.
    Lighten,
    /// What they carry weighs this share more (and in a fight they're slower).
    Burden,
    /// The next blow that would land is turned aside.
    Brace,
    /// Ground: an alarm that wakes sleepers inside when they're attacked.
    Tripwire,
    /// Ground: enemies can't step inside, and lookouts won't come for
    /// anyone in it.
    Sanctuary,
    /// Smaller: hits this share softer, reaches less, harder to hit.
    Shrink,
    /// Bigger: hits this share harder, reaches further, easier to hit, slower.
    Enlarge,
    /// Their armour stops this share less.
    Rust,
    /// Ground: a spirit at a spot the caster can see through (the window
    /// can look from there).
    Scout,
}

impl Does {
    /// Something done to an enemy (a target can try to throw it off).
    pub fn harmful(self) -> bool {
        matches!(self, Does::Damage(_) | Does::Paralyze | Does::Blind | Does::Slow | Does::KnockDown | Does::Douse | Does::Daze | Does::Calm | Does::Fear | Does::Dominate | Does::Burden | Does::Shatter | Does::Shrink | Does::Rust)
    }

    /// Guards against harm (worth putting up when a fight reaches you).
    pub fn guards(self) -> bool {
        matches!(self, Does::Barrier | Does::ResistElements | Does::ResistParalysis | Does::ResistBlind | Does::DomainResist(_) | Does::Hide | Does::Toughen | Does::Brace)
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
    /// Anything done at once that has a meaning out in the world, and any
    /// lasting effect that isn't an attack (it's kept as a blessing on them).
    pub fn works_outside_fights(self) -> bool {
        match self {
            Does::Heal | Does::Energy | Does::Rest | Does::Stamina | Does::Regrow | Does::Kindle | Does::Douse | Does::Dispel | Does::Unlock | Does::Transmute => true,
            // A guardian can wait on the ground for a fight; the rest only
            // come to one.
            Does::Summon(k) => k == Summon::Guardian,
            d if d.harmful() => false,
            _ => true,
        }
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
            Does::Heal if p >= 300.0 => "Heals every wound".to_string(),
            Does::Heal => format!("Heals {p:.0}"),
            Does::Energy => format!("Restores {p:.0} energy"),
            Does::Rest => "Takes away tiredness".to_string(),
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
            Does::Slow => format!("{pct:.0}% slower"),
            Does::KnockDown => "Knocks down".to_string(),
            Does::Kindle => "Lights a torch or fire".to_string(),
            Does::Douse => "Puts out a torch or fire".to_string(),
            Does::Daze => "Loses their next action".to_string(),
            Does::Calm => "Won't fight (a blow breaks it)".to_string(),
            Does::Fear => "Flees".to_string(),
            Does::Dominate => "Fights for you".to_string(),
            Does::Nightsight => "Sees in the dark".to_string(),
            Does::SenseLife => "Senses the living through walls".to_string(),
            Does::Sway => format!("{p:+.0} disposition"),
            Does::Summon(k) => format!("Calls up: {}", k.name().to_lowercase()),
            Does::Silent => format!("{pct:.0}% quieter"),
            Does::Glow => "Gives off light".to_string(),
            Does::Gloom => "Darkens the ground round them".to_string(),
            Does::Hide => format!("{pct:.0}% harder to see"),
            Does::Disguise => "Unrecognised".to_string(),
            Does::Veil => "Hidden from passers-by".to_string(),
            Does::Stamina => format!("Restores {p:.0} stamina"),
            Does::Regrow => "Regrows a lost limb".to_string(),
            Does::Toughen => format!("Skin stops {pct:.0}% of cuts"),
            Does::Sustain => "No hunger or tiredness".to_string(),
            Does::Lighten => format!("Load {pct:.0}% lighter"),
            Does::Dispel => "Ends every spell on them".to_string(),
            Does::Shatter => "Shatters their weapon or shield".to_string(),
            Does::Unlock => "Opens a locked door".to_string(),
            Does::Transmute => "Turns one material into another".to_string(),
            Does::Shrink => format!("{pct:.0}% smaller"),
            Does::Enlarge => format!("{pct:.0}% bigger"),
            Does::Rust => format!("Armour {pct:.0}% weaker"),
            Does::Scout => "A spirit to see through".to_string(),
            Does::Brace => "Turns the next blow".to_string(),
            Does::Tripwire => "Wakes sleepers when danger comes".to_string(),
            Does::Sanctuary => "Enemies can't enter".to_string(),
            Does::Burden => format!("Load {pct:.0}% heavier"),
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
