//! People: a cheap summary that always exists, and details that only exist
//! once something needs them.
//!
//! Every person carries a seed, a race, a temperament and a cached `might`
//! score. That is all the far-away world ever reads. Name and gear are built
//! from the seed and the score the first time the person comes close enough to
//! matter — and from then on they are kept for good, so someone you have met
//! never comes back as a stranger.

use super::names;
use super::race::{Race, Traits, TRAIT_SPREAD};
use super::rng::Rng;
use super::settlement::SettlementId;

pub type PersonId = u32;

#[derive(Clone, Debug)]
pub struct Person {
    pub id: PersonId,
    /// Everything about this person that is not stored is derived from this.
    pub seed: u64,
    pub race: Race,
    /// None for wanderers, who belong nowhere.
    pub home: Option<SettlementId>,
    pub traits: Traits,
    /// Cached fighting strength. The far bands only ever look at this number.
    /// Recomputed only when gear or traits change.
    pub might: f32,
    /// Built on first approach, then kept forever.
    pub detail: Option<Detail>,
    pub in_squad: bool,
    /// Which building in their home town they live in.
    pub dwelling: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Detail {
    pub name: String,
    pub weapon: Weapon,
    pub armor: Armor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weapon {
    pub tier: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Armor {
    pub tier: u8,
}

const WEAPON_VALUE: [f32; 5] = [0.0, 3.0, 6.0, 9.0, 12.0];
const ARMOR_VALUE: [f32; 4] = [0.0, 2.0, 5.0, 8.0];

impl Weapon {
    pub fn value(self) -> f32 {
        WEAPON_VALUE[self.tier as usize]
    }

    pub fn describe(self, race: Race) -> String {
        let (common, good) = match race {
            Race::Roduro => ("stone maul", "hooked staff"),
            Race::Qotiro => ("spear", "war-pick"),
            Race::Horaro => ("harpoon", "trident"),
            Race::Tadoro => ("sling", "long knife"),
        };
        match self.tier {
            0 => "unarmed".into(),
            1 => "a knife".into(),
            2 => format!("a {common}"),
            3 => format!("a {good}"),
            _ => format!("a fine {good}"),
        }
    }
}

impl Armor {
    pub fn value(self) -> f32 {
        ARMOR_VALUE[self.tier as usize]
    }

    pub fn describe(self) -> &'static str {
        match self.tier {
            0 => "no armor",
            1 => "padded cloth",
            2 => "hide and leather",
            _ => "scale",
        }
    }
}

/// Fighting strength from body and nerve alone, before gear.
pub fn base_might(race: Race, traits: &Traits) -> f32 {
    10.0 * race.build() * (0.55 + 0.45 * traits.boldness)
}

impl Person {
    /// Create the cheap summary. No name, no gear: just the numbers.
    pub fn summary(id: PersonId, seed: u64, race: Race, home: Option<SettlementId>) -> Person {
        let mut rng = Rng::from_keys(&[seed, 0x5045_5253]);
        let m = race.trait_means();
        let mut draw = |mean: f32| (mean + rng.normal() * TRAIT_SPREAD).clamp(0.0, 1.0);
        let traits = Traits {
            wanderlust: draw(m.wanderlust),
            boldness: draw(m.boldness),
            sociability: draw(m.sociability),
            patience: draw(m.patience),
        };
        // What this person could afford to carry, as one number. Bolder people
        // tend to be better armed; most people carry little.
        let gear_budget = (rng.f32().powf(1.6) * 14.0 + traits.boldness * 6.0).min(20.0);
        let might = base_might(race, &traits) + gear_budget;
        Person { id, seed, race, home, traits, might, detail: None, in_squad: false, dwelling: None }
    }

    /// Build name and gear, if they do not exist yet. Gear is chosen to add up
    /// to the stored score, so the person who appears matches the number the
    /// far-away simulation has been using. Returns true if details were made.
    pub fn ensure_detail(&mut self) -> bool {
        if self.detail.is_some() {
            return false;
        }
        let target = self.might - base_might(self.race, &self.traits);
        let mut rng = Rng::from_keys(&[self.seed, 0x4745_4152]);
        // Every weapon/armour pairing, scored by how close it lands to the
        // target. A little seeded jitter breaks ties so equals are not identical.
        let mut best = (f32::MAX, 0u8, 0u8);
        for w in 0..WEAPON_VALUE.len() as u8 {
            for a in 0..ARMOR_VALUE.len() as u8 {
                let err = (WEAPON_VALUE[w as usize] + ARMOR_VALUE[a as usize] - target).abs() + rng.f32() * 0.4;
                if err < best.0 {
                    best = (err, w, a);
                }
            }
        }
        self.detail = Some(Detail {
            name: names::person_name(self.race, self.seed),
            weapon: Weapon { tier: best.1 },
            armor: Armor { tier: best.2 },
        });
        self.recompute_might();
        true
    }

    /// The only place `might` changes once details exist. Call it whenever
    /// gear or traits change; nothing else should touch the score.
    pub fn recompute_might(&mut self) {
        if let Some(d) = &self.detail {
            self.might = base_might(self.race, &self.traits) + d.weapon.value() + d.armor.value();
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.detail.as_ref().map(|d| d.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_match_the_score_they_were_built_from() {
        let mut worst = 0.0f32;
        for i in 0..2000u32 {
            let mut p = Person::summary(i, i as u64 * 7919, super::super::race::ALL_RACES[i as usize % 4], None);
            let before = p.might;
            p.ensure_detail();
            worst = worst.max((p.might - before).abs());
        }
        // Gear comes in steps, so the score moves a little. Never by much.
        assert!(worst <= 2.0, "score drifted by {worst} when details were built");
    }

    #[test]
    fn details_are_a_function_of_the_seed() {
        let mut a = Person::summary(1, 99, Race::Horaro, None);
        let mut b = Person::summary(1, 99, Race::Horaro, None);
        a.ensure_detail();
        b.ensure_detail();
        assert_eq!(a.detail, b.detail);
    }
}
