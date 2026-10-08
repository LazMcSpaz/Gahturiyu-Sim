//! What a person is capable of: attributes, skills, and what follows from them.
//!
//! Morrowind's shape, Kenshi's habits. Five attributes describe the body and
//! mind someone was born with; a dozen skills describe what they have learned.
//! Skills grow by *use* — swing a sword and your blade skill creeps up, get hit
//! and your toughness does — and each point is harder to earn than the last.
//! Attributes grow too, slowly, from the skills that lean on them.
//!
//! Every person has stats from the moment they exist (they're cheap: seventeen
//! numbers), drawn from their seed, race and temperament. The far-away world
//! reads them only through the single `might` rating.

use super::race::{Race, Traits};
use super::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attr {
    /// Hitting power and how much can be carried.
    Strength,
    /// Accuracy, dodging, attack speed, footspeed.
    Agility,
    /// Health on every body part, and resisting knockdown.
    Toughness,
    /// The size of the mana pool.
    Intellect,
    /// Casting reliably, resisting spells, and fatigue.
    Willpower,
}

pub const ATTRS: [Attr; 5] = [Attr::Strength, Attr::Agility, Attr::Toughness, Attr::Intellect, Attr::Willpower];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Skill {
    Blade,
    Blunt,
    Spear,
    Unarmed,
    Block,
    Dodge,
    Athletics,
    /// Fire and lightning.
    Destruction,
    /// Shaping the body and its surroundings: armour of force, speed.
    Alteration,
    /// The mind: blinding, holding still.
    Illusion,
    /// Mending flesh.
    Restoration,
    /// Moving unseen and unheard.
    Sneak,
    /// Locks: picking them, and knowing how hard one is.
    Security,
    /// Potions from plants, minerals and parts.
    Alchemy,
    /// Binding spells into scrolls.
    Inscription,
    /// Forging and shaping weapons.
    Smithing,
    /// Making armour.
    Armoring,
}

/// How many skills there are.
pub const N_SKILLS: usize = 17;

pub const SKILLS: [Skill; N_SKILLS] = [
    Skill::Blade,
    Skill::Blunt,
    Skill::Spear,
    Skill::Unarmed,
    Skill::Block,
    Skill::Dodge,
    Skill::Athletics,
    Skill::Destruction,
    Skill::Alteration,
    Skill::Illusion,
    Skill::Restoration,
    Skill::Sneak,
    Skill::Security,
    Skill::Alchemy,
    Skill::Inscription,
    Skill::Smithing,
    Skill::Armoring,
];

impl Skill {
    pub fn name(self) -> &'static str {
        match self {
            Skill::Blade => "Blade",
            Skill::Blunt => "Blunt",
            Skill::Spear => "Spear",
            Skill::Unarmed => "Unarmed",
            Skill::Block => "Block",
            Skill::Dodge => "Dodge",
            Skill::Athletics => "Athletics",
            Skill::Destruction => "Destruction",
            Skill::Alteration => "Alteration",
            Skill::Illusion => "Illusion",
            Skill::Restoration => "Restoration",
            Skill::Sneak => "Sneak",
            Skill::Security => "Security",
            Skill::Alchemy => "Alchemy",
            Skill::Inscription => "Inscription",
            Skill::Smithing => "Smithing",
            Skill::Armoring => "Armoring",
        }
    }

    /// The attribute that slowly rises as this skill is used.
    pub fn governed_by(self) -> Attr {
        match self {
            Skill::Blade | Skill::Dodge => Attr::Agility,
            Skill::Blunt | Skill::Spear | Skill::Unarmed => Attr::Strength,
            Skill::Block | Skill::Athletics => Attr::Toughness,
            Skill::Destruction => Attr::Intellect,
            Skill::Alteration | Skill::Illusion | Skill::Restoration => Attr::Willpower,
            Skill::Sneak | Skill::Security => Attr::Agility,
            Skill::Alchemy | Skill::Inscription => Attr::Intellect,
            Skill::Smithing | Skill::Armoring => Attr::Strength,
        }
    }

    pub fn is_magic(self) -> bool {
        matches!(self, Skill::Destruction | Skill::Alteration | Skill::Illusion | Skill::Restoration)
    }
}

impl Attr {
    pub fn name(self) -> &'static str {
        match self {
            Attr::Strength => "Strength",
            Attr::Agility => "Agility",
            Attr::Toughness => "Toughness",
            Attr::Intellect => "Intellect",
            Attr::Willpower => "Willpower",
        }
    }
    pub fn short(self) -> &'static str {
        match self {
            Attr::Strength => "STR",
            Attr::Agility => "AGI",
            Attr::Toughness => "TOU",
            Attr::Intellect => "INT",
            Attr::Willpower => "WIL",
        }
    }
}

/// What someone does with their life, which sets where their skills start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Calling {
    /// Most people: a little of everything, mastery of nothing.
    Common,
    Warrior,
    Hunter,
    Mage,
}

impl Calling {
    pub fn name(self) -> &'static str {
        match self {
            Calling::Common => "commoner",
            Calling::Warrior => "warrior",
            Calling::Hunter => "hunter",
            Calling::Mage => "mage",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub attrs: [f32; 5],
    pub skills: [f32; N_SKILLS],
    pub calling: Calling,
}

/// Attributes and skills top out here.
pub const STAT_MAX: f32 = 100.0;

impl Stats {
    pub fn attr(&self, a: Attr) -> f32 {
        self.attrs[a as usize]
    }
    pub fn skill(&self, s: Skill) -> f32 {
        self.skills[s as usize]
    }
    pub fn set_attr(&mut self, a: Attr, v: f32) {
        self.attrs[a as usize] = v.clamp(1.0, STAT_MAX);
    }
    pub fn set_skill(&mut self, s: Skill, v: f32) {
        self.skills[s as usize] = v.clamp(0.0, STAT_MAX);
    }

    /// Born stats: race sets the averages, temperament nudges them, the seed
    /// does the rest. Callings follow temperament — the bold take up arms, the
    /// patient and clever take up magic — but anyone can be anything.
    pub fn generate(race: Race, traits: &Traits, seed: u64) -> Stats {
        let mut rng = Rng::from_keys(&[seed, 0x5354_4154]);
        let bias: [f32; 5] = match race {
            //                 STR   AGI   TOU   INT   WIL
            Race::Roduro => [8.0, -4.0, 12.0, 0.0, 4.0],
            Race::Qotiro => [12.0, 6.0, 6.0, -4.0, 0.0],
            Race::Horaro => [-4.0, 4.0, 0.0, 6.0, 10.0],
            Race::Tadoro => [-8.0, 10.0, -6.0, 14.0, 4.0],
        };
        let mut attrs = [0.0f32; 5];
        for (i, a) in attrs.iter_mut().enumerate() {
            *a = (32.0 + bias[i] + rng.normal() * 8.0).clamp(10.0, 70.0);
        }
        attrs[Attr::Strength as usize] += (traits.boldness - 0.5) * 8.0;
        attrs[Attr::Willpower as usize] += (traits.patience - 0.5) * 8.0;

        // Calling. Ṭaḍoro magic is structural and teachable, so more of them
        // study it; Qotiro lean martial.
        let magic_lean = match race {
            Race::Tadoro => 2.2,
            Race::Horaro => 1.3,
            Race::Roduro => 1.0,
            Race::Qotiro => 0.7,
        };
        let w = [
            3.0,
            0.6 + traits.boldness * 1.6,
            0.4 + traits.wanderlust * 1.2,
            (0.25 + attrs[Attr::Intellect as usize] / 100.0 + traits.patience * 0.3) * magic_lean,
        ];
        let calling = match rng.weighted(&w) {
            Some(1) => Calling::Warrior,
            Some(2) => Calling::Hunter,
            Some(3) => Calling::Mage,
            _ => Calling::Common,
        };

        let mut skills = [0.0f32; N_SKILLS];
        for s in skills.iter_mut() {
            *s = 4.0 + rng.f32() * 10.0;
        }
        let mut lift = |s: Skill, by: f32, rng: &mut Rng| {
            skills[s as usize] += by * (0.6 + rng.f32() * 0.8);
        };
        match calling {
            Calling::Common => {}
            Calling::Warrior => {
                let main = *rng.pick(&[Skill::Blade, Skill::Blunt, Skill::Spear]);
                lift(main, 32.0, &mut rng);
                lift(Skill::Block, 20.0, &mut rng);
                lift(Skill::Athletics, 12.0, &mut rng);
            }
            Calling::Hunter => {
                lift(Skill::Sneak, 18.0, &mut rng);
                lift(Skill::Alchemy, 8.0, &mut rng);
                lift(Skill::Spear, 24.0, &mut rng);
                lift(Skill::Dodge, 22.0, &mut rng);
                lift(Skill::Athletics, 20.0, &mut rng);
                lift(Skill::Blade, 10.0, &mut rng);
            }
            Calling::Mage => {
                lift(Skill::Destruction, 26.0, &mut rng);
                lift(Skill::Alteration, 22.0, &mut rng);
                lift(Skill::Illusion, 22.0, &mut rng);
                lift(Skill::Restoration, 20.0, &mut rng);
                lift(Skill::Inscription, 14.0, &mut rng);
                lift(Skill::Alchemy, 10.0, &mut rng);
                lift(Skill::Dodge, 8.0, &mut rng);
                attrs[Attr::Intellect as usize] += 6.0;
            }
        }
        // Race habits.
        match race {
            Race::Roduro => lift(Skill::Blunt, 6.0, &mut rng),
            Race::Qotiro => lift(Skill::Spear, 8.0, &mut rng),
            Race::Horaro => lift(Skill::Spear, 5.0, &mut rng),
            Race::Tadoro => lift(Skill::Dodge, 6.0, &mut rng),
        }
        let mut st = Stats { attrs, skills, calling };
        for a in ATTRS {
            let v = st.attr(a);
            st.set_attr(a, v);
        }
        for s in SKILLS {
            let v = st.skill(s);
            st.set_skill(s, v);
        }
        st
    }

    // ---- What follows from the numbers ---------------------------------

    /// Health of each body part at full strength.
    pub fn max_hp(&self, part: super::body::Part) -> f32 {
        use super::body::Part;
        let t = self.attr(Attr::Toughness);
        match part {
            Part::Head => 35.0 + t * 0.5,
            Part::Torso => 50.0 + t * 0.8,
            _ => 40.0 + t * 0.6,
        }
    }

    /// Fatigue (stamina) pool. Swinging, running and blocking drain it; a
    /// tired fighter hits less often and less hard.
    pub fn max_fatigue(&self) -> f32 {
        50.0 + (self.attr(Attr::Strength) + self.attr(Attr::Agility) + self.attr(Attr::Toughness) + self.attr(Attr::Willpower)) * 0.5
    }

    /// Mana pool. Intellect times a racial gift.
    pub fn max_mana(&self, race: Race) -> f32 {
        let gift = match race {
            Race::Roduro => 1.0,
            Race::Horaro => 1.25,
            Race::Qotiro => 0.75,
            Race::Tadoro => 1.5,
        };
        self.attr(Attr::Intellect) * gift
    }

    /// How much can be carried without slowing down, in kilograms.
    pub fn carry_capacity(&self) -> f32 {
        30.0 + self.attr(Attr::Strength) * 0.8
    }

    /// Footspeed multiplier from agility and athletics.
    pub fn move_factor(&self) -> f32 {
        0.85 + self.attr(Attr::Agility) * 0.002 + self.skill(Skill::Athletics) * 0.002
    }

    // ---- Learning by doing ---------------------------------------------

    /// Use a skill once. `amount` is how much it was exercised (1 = one
    /// ordinary use). Gains shrink as the skill rises; the governing
    /// attribute creeps up at a tenth of the rate.
    pub fn exercise(&mut self, s: Skill, amount: f32) {
        let v = self.skill(s);
        let gain = 0.35 * amount * (1.0 - v / STAT_MAX).max(0.0).powf(1.5);
        self.set_skill(s, v + gain);
        let a = s.governed_by();
        let av = self.attr(a);
        self.set_attr(a, av + gain * 0.1);
    }

    /// Taking hits toughens you, the Kenshi way.
    pub fn harden(&mut self, damage: f32) {
        let v = self.attr(Attr::Toughness);
        let gain = 0.004 * damage * (1.0 - v / STAT_MAX).max(0.0).powf(1.5);
        self.set_attr(Attr::Toughness, v + gain);
    }

    /// The best of the three weapon skills plus unarmed, for summaries.
    pub fn best_melee(&self) -> f32 {
        [Skill::Blade, Skill::Blunt, Skill::Spear, Skill::Unarmed].iter().map(|&s| self.skill(s)).fold(0.0, f32::max)
    }

    pub fn best_magic(&self) -> f32 {
        [Skill::Destruction, Skill::Alteration, Skill::Illusion, Skill::Restoration].iter().map(|&s| self.skill(s)).fold(0.0, f32::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::ALL_RACES;

    fn traits() -> Traits {
        Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }
    }

    #[test]
    fn races_lean_the_way_the_lore_says() {
        let avg = |r: Race, a: Attr| (0..400u64).map(|s| Stats::generate(r, &traits(), s).attr(a)).sum::<f32>() / 400.0;
        assert!(avg(Race::Roduro, Attr::Toughness) > avg(Race::Tadoro, Attr::Toughness) + 10.0);
        assert!(avg(Race::Tadoro, Attr::Intellect) > avg(Race::Qotiro, Attr::Intellect) + 10.0);
        assert!(avg(Race::Qotiro, Attr::Strength) > avg(Race::Tadoro, Attr::Strength) + 10.0);
        // ...but they overlap: some Ṭaḍoro are stronger than some Qotiro.
        let weakest_qotiro = (0..400u64).map(|s| Stats::generate(Race::Qotiro, &traits(), s).attr(Attr::Strength)).fold(f32::MAX, f32::min);
        let strongest_tadoro = (0..400u64).map(|s| Stats::generate(Race::Tadoro, &traits(), s).attr(Attr::Strength)).fold(0.0, f32::max);
        assert!(strongest_tadoro > weakest_qotiro);
    }

    #[test]
    fn skills_grow_with_use_and_slow_down() {
        let mut s = Stats::generate(Race::Roduro, &traits(), 3);
        s.set_skill(Skill::Blade, 10.0);
        let before = s.skill(Skill::Blade);
        for _ in 0..50 {
            s.exercise(Skill::Blade, 1.0);
        }
        let early = s.skill(Skill::Blade) - before;
        s.set_skill(Skill::Blade, 80.0);
        for _ in 0..50 {
            s.exercise(Skill::Blade, 1.0);
        }
        let late = s.skill(Skill::Blade) - 80.0;
        assert!(early > 5.0, "fifty swings taught almost nothing: {early}");
        assert!(late < early / 4.0, "mastery should come slowly ({late} vs {early})");
    }

    #[test]
    fn every_race_produces_every_calling() {
        for r in ALL_RACES {
            let mut seen = std::collections::HashSet::new();
            for s in 0..600u64 {
                seen.insert(format!("{:?}", Stats::generate(r, &traits(), s).calling));
            }
            assert_eq!(seen.len(), 4, "{:?} only produced {:?}", r, seen);
        }
    }
}
