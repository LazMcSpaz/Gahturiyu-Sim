//! People: a cheap summary that always exists, and details that only exist
//! once something needs them.
//!
//! Every person always carries a seed, a race, a temperament, their stats, the
//! state of their body and mana, and a cached `might` rating. That is all the
//! far-away world reads. Name, kit and spells are built from the seed the first
//! time the person comes close enough to matter — and from then on they are
//! kept for good, so someone you have met never comes back as a stranger.

use serde::{Deserialize, Serialize};

use super::body::Wounds;
use super::combat;
use super::inventory::{self, Gear};
use super::magic::{self, Spell};
use super::names;
use super::race::{Race, Traits, TRAIT_SPREAD};
use super::rng::Rng;
use super::settlement::SettlementId;
use super::stats::{Calling, Stats};

pub type PersonId = u32;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Person {
    pub id: PersonId,
    /// Everything about this person that is not stored is derived from this.
    pub seed: u64,
    pub race: Race,
    /// None for wanderers and bandits, who belong nowhere.
    pub home: Option<SettlementId>,
    pub traits: Traits,
    pub stats: Stats,
    /// What their kit is worth, in coin. The kit itself is built from this
    /// (and the seed) when details are made.
    pub budget: f32,
    /// Their stats as they were when their kit was chosen. The kit is picked
    /// from these, not from their stats now, so training a skill never
    /// changes the kit a stranger is assumed to carry — whether or not
    /// they've been met.
    pub kit_stats: Stats,
    /// Cached fighting strength. The far bands only ever look at this number.
    /// Recomputed only when gear or stats change.
    pub might: f32,
    /// Built on first approach, then kept forever.
    pub detail: Option<Detail>,
    pub in_squad: bool,
    /// Which building in their home town they live in.
    pub dwelling: Option<u16>,
    pub wounds: Wounds,
    /// Mana as of `mana_at`; it refills over time on its own.
    pub mana: f32,
    pub mana_at: f64,
    pub dead: bool,
    pub bandit: bool,
    /// Hunger and the like — your squad only.
    pub cond: Option<super::condition::Condition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Detail {
    pub name: String,
    pub gear: Gear,
    pub spells: Vec<Spell>,
}

/// Mana regained per game minute with no help from enchantments.
pub const MANA_REGEN: f32 = 0.5;

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
        let stats = Stats::generate(race, &traits, seed);
        let kit_stats = stats.clone();
        // What their kit is worth. Fighters spend on it; most people carry little.
        let luck = rng.f32().powf(1.6);
        let budget = match stats.calling {
            Calling::Common => 15.0 + luck * 70.0,
            Calling::Warrior => 120.0 + luck * 450.0 * (0.5 + traits.boldness),
            Calling::Hunter => 70.0 + luck * 220.0,
            Calling::Mage => 50.0 + luck * 160.0,
        };
        let mut p = Person {
            id,
            seed,
            race,
            home,
            traits,
            stats,
            budget,
            might: 0.0,
            kit_stats,
            detail: None,
            in_squad: false,
            dwelling: None,
            wounds: Wounds::default(),
            mana: 0.0,
            mana_at: 0.0,
            dead: false,
            bandit: false,
            cond: None,
        };
        p.mana = p.max_mana();
        p.recompute_might();
        p
    }

    /// The kit this person has (or would have, if met).
    pub fn kit(&self) -> Gear {
        match &self.detail {
            Some(d) => d.gear.clone(),
            None => inventory::starting_kit(self.race, &self.kit_stats, self.budget, self.seed),
        }
    }

    /// Build name, kit and spells, if they do not exist yet. The kit is the
    /// same one the summary was rated with, so the person who appears matches
    /// the number the far-away world has been using. Returns true if details
    /// were made.
    pub fn ensure_detail(&mut self) -> bool {
        if self.detail.is_some() {
            return false;
        }
        self.detail = Some(Detail {
            name: names::person_name(self.race, self.seed),
            gear: inventory::starting_kit(self.race, &self.kit_stats, self.budget, self.seed),
            spells: magic::starting_spells(&self.kit_stats),
        });
        self.recompute_might();
        true
    }

    /// The only place `might` changes. Call it whenever gear or stats change.
    pub fn recompute_might(&mut self) {
        let gear = self.kit();
        let spells = match &self.detail {
            Some(d) => d.spells.clone(),
            None => magic::starting_spells(&self.kit_stats),
        };
        self.might = combat::rating(self.race, &self.stats, &gear, &spells);
    }

    /// Shape someone into a particular kind of fighter before they are met:
    /// set their calling, raise the given skills to at least the given
    /// values, and give them a purse for kit. Only valid before details exist.
    pub fn specialize(&mut self, calling: Calling, skills: &[(super::stats::Skill, f32)], budget: f32) {
        debug_assert!(self.detail.is_none(), "can't respecialize someone already met");
        self.stats.calling = calling;
        for &(k, v) in skills {
            if self.stats.skill(k) < v {
                self.stats.set_skill(k, v);
            }
        }
        if calling == Calling::Mage {
            let int = self.stats.attr(super::stats::Attr::Intellect);
            self.stats.set_attr(super::stats::Attr::Intellect, int.max(48.0));
        }
        self.budget = budget;
        self.kit_stats = self.stats.clone();
        self.mana = self.max_mana();
        self.recompute_might();
    }

    pub fn name(&self) -> Option<&str> {
        self.detail.as_ref().map(|d| d.name.as_str())
    }

    /// Stats with gear enchantments applied, and weakened by hunger (squad only).
    pub fn effective_stats(&self) -> Stats {
        let mut s = inventory::effective(&self.stats, &self.kit());
        self.weaken(&mut s);
        s
    }

    /// Lower attributes for how badly they're holding up.
    pub fn weaken(&self, s: &mut Stats) {
        if let Some(c) = &self.cond {
            let f = c.attr_factor();
            if f < 1.0 {
                for a in super::stats::ATTRS {
                    let v = s.attr(a);
                    s.set_attr(a, v * f);
                }
            }
        }
    }

    pub fn max_mana(&self) -> f32 {
        let gear = self.kit();
        let bonus = gear.worn(super::effects::Does::MaxEnergy);
        inventory::effective(&self.stats, &gear).max_mana(self.race) + bonus
    }

    /// Mana per game minute, including enchantments.
    pub fn mana_regen(&self) -> f32 {
        MANA_REGEN + self.kit().worn(super::effects::Does::EnergyRegen)
    }

    /// Mana available at time `t`.
    pub fn mana_at(&self, t: f64) -> f32 {
        (self.mana + self.mana_regen() * ((t - self.mana_at).max(0.0) / 60.0) as f32).min(self.max_mana())
    }

    pub fn set_mana(&mut self, m: f32, t: f64) {
        self.mana = m.max(0.0);
        self.mana_at = t;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::ALL_RACES;

    #[test]
    fn details_match_the_score_they_were_built_from() {
        for i in 0..400u32 {
            let mut p = Person::summary(i, i as u64 * 7919, ALL_RACES[i as usize % 4], None);
            let before = p.might;
            p.ensure_detail();
            assert!((p.might - before).abs() < 1e-4, "might moved from {before} to {} when details were built", p.might);
        }
    }

    #[test]
    fn details_are_a_function_of_the_seed() {
        let mut a = Person::summary(1, 99, Race::Horaro, None);
        let mut b = Person::summary(1, 99, Race::Horaro, None);
        a.ensure_detail();
        b.ensure_detail();
        assert_eq!(a.detail, b.detail);
    }

    #[test]
    fn mana_refills_over_time_but_not_past_the_pool() {
        let mut p = Person::summary(2, 2024, Race::Tadoro, None);
        let full = p.max_mana();
        p.set_mana(0.0, 100.0);
        assert!(p.mana_at(100.0) < 0.01);
        assert!((p.mana_at(100.0 + 600.0) - 5.0).abs() < 0.01, "ten minutes should restore 5 mana");
        assert!((p.mana_at(1e9) - full).abs() < 0.01);
    }

    #[test]
    fn fighters_rate_higher_than_commoners() {
        let mut warriors = vec![];
        let mut commoners = vec![];
        for i in 0..2000u32 {
            let p = Person::summary(i, i as u64 * 31, ALL_RACES[i as usize % 4], None);
            match p.stats.calling {
                Calling::Warrior => warriors.push(p.might),
                Calling::Common => commoners.push(p.might),
                _ => {}
            }
        }
        let avg = |v: &Vec<f32>| v.iter().sum::<f32>() / v.len() as f32;
        assert!(avg(&warriors) > avg(&commoners) * 1.5, "warriors {} vs commoners {}", avg(&warriors), avg(&commoners));
    }
}
