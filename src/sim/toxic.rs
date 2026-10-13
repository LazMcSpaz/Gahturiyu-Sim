//! The alchemist's toxicity meter (Part 4 of `docs/items-design.md`, rule 7)
//! and drinking out of a fight. Every potion adds toxicity, nausea adds more,
//! and it drains on the condition timeline (`condition.rs`: queasy, sick,
//! collapse; it never kills). Sick, a draught may come straight back up (one
//! keyed roll per draught): it does nothing then. A potion's harmful side
//! effects reach whoever drinks it, out of a fight too: poison runs its
//! course at once, nausea takes their breath at once, and the rest stay on
//! them as boons (they go into a fight as statuses).

use super::condition::{SICK, TOX_COLLAPSE};
use super::effects::{Does, Effect, Lasts};
use super::items::herbs::Mix;
use super::items::{item, ItemId, Kind};
use super::person::PersonId;
use super::rng::Rng;
use super::world::World;

/// Toxicity one plain draught adds (brewing will scale it by the mix's
/// strength and impurity).
pub const DRAUGHT_TOX: f32 = 12.0;
/// Toxicity per point of nausea's power.
pub const NAUSEA_TOX: f32 = 10.0;
/// The chance a draught comes straight back up, sick.
pub const VOMIT_CHANCE: f32 = 0.35;
/// Head and torso lifted to this share of their most by a rousing draught.
pub const ROUSED: f32 = 0.15;
const VOMIT_TAG: u64 = 0x564F_4D49;

/// Toxicity a potion adds when drunk (a brewed bottle: by its mix).
pub fn draught_tox(it: ItemId) -> f32 {
    let _ = it;
    DRAUGHT_TOX
}

/// What a potion does when drunk: a brewed bottle's mix, else the item's own
/// effects.
pub fn draught_effects(it: ItemId, mix: Option<&Mix>) -> Vec<Effect> {
    match mix {
        Some(m) => m.effects(),
        None => item(it).effects.to_vec(),
    }
}

/// Does a draught come straight back up? One roll per drinker, draught and
/// moment (`key`), at this much toxicity.
pub fn throws_up(seed: u64, who: u64, it: ItemId, key: u64, tox: f32) -> bool {
    tox >= SICK && Rng::from_keys(&[seed, who, it as u64, key, VOMIT_TAG]).f32() < VOMIT_CHANCE
}

impl World {
    /// Toxicity now (0..100), for a squad member.
    pub fn toxicity_of(&self, pid: PersonId) -> Option<f32> {
        self.people[pid as usize].cond.as_ref().map(|c| c.tox_at(self.time))
    }

    /// Add toxicity at `t`: settled first; at `TOX_COLLAPSE` they collapse
    /// (knocked out until it's down to `TOX_WAKE`). Returns the new level.
    pub(super) fn add_toxicity(&mut self, pid: PersonId, amount: f32, t: f64) -> f32 {
        if self.people[pid as usize].cond.is_none() || amount <= 0.0 || !self.rules.toxicity {
            return self.people[pid as usize].cond.as_ref().map(|c| c.tox_at(t)).unwrap_or(0.0);
        }
        self.settle_condition(pid, t);
        let c = self.people[pid as usize].cond.as_mut().unwrap();
        let was = c.tox;
        c.tox = (c.tox + amount).min(100.0);
        let now = c.tox;
        let line = if now >= TOX_COLLAPSE && was < TOX_COLLAPSE {
            c.tox_out = true;
            Some("collapses, sick from all they've drunk")
        } else if now >= SICK && was < SICK {
            Some("feels sick")
        } else if now >= super::condition::QUEASY && was < super::condition::QUEASY {
            Some("feels queasy")
        } else {
            None
        };
        // (Settled again so the stages read the new level from here on.)
        self.settle_condition(pid, t);
        if let (Some(how), Some(_)) = (line, self.squad.index(pid)) {
            let name = self.name_of(pid);
            self.say(t, format!("{name} {how}."));
        }
        now
    }

    /// Drink a potion out of a fight (for a brewed one, the first bottle in
    /// the pack): its toxicity first; sick, it may come straight back up;
    /// then each of its effects on the drinker.
    pub(super) fn drink_potion(&mut self, who: PersonId, it: ItemId) -> bool {
        if item(it).kind != Kind::Potion {
            return false;
        }
        let Some(d) = self.people[who as usize].detail.as_mut() else { return false };
        let Some(piece) = d.gear.take_piece(it) else { return false };
        self.drunk(who, it, piece.and_then(|p| p.mix))
    }

    /// Use the `k`th thing in someone's pack: a bottle is drunk, a recipe
    /// sheet read (it stays), anything else used as `use_item` would.
    pub fn use_entry(&mut self, who: PersonId, k: usize) -> bool {
        let Some(e) = self.people[who as usize].detail.as_ref().and_then(|d| d.gear.bag.get(k)).copied() else { return false };
        if let Some(s) = e.2.and_then(|p| p.sheet) {
            if self.is_down(who) {
                return false;
            }
            self.read_sheet(who, s);
            return true;
        }
        if item(e.0).kind == Kind::Potion {
            return self.drink_entry(who, k);
        }
        self.use_item(who, e.0)
    }

    /// Drink the `k`th thing in someone's pack (a particular bottle).
    pub fn drink_entry(&mut self, who: PersonId, k: usize) -> bool {
        if self.is_down(who) || self.fighting.contains_key(&who) {
            return false;
        }
        let Some(d) = self.people[who as usize].detail.as_mut() else { return false };
        let Some(e) = d.gear.bag.get(k) else { return false };
        if item(e.0).kind != Kind::Potion {
            return false;
        }
        let (it, mix) = (e.0, e.2.and_then(|p| p.mix));
        match mix {
            Some(m) => d.gear.take_mix(&m),
            None => d.gear.take(it),
        };
        self.drunk(who, it, mix)
    }

    /// What drinking it does, once it's out of the pack.
    fn drunk(&mut self, who: PersonId, it: ItemId, mix: Option<Mix>) -> bool {
        let t = self.time;
        let name = self.name_of(who);
        let what = mix.map(|m| m.name()).unwrap_or_else(|| item(it).name.to_string());
        self.say(t, format!("{name} drinks the {}.", what.to_lowercase()));
        let tox = self.add_toxicity(who, mix.map(|m| m.tox()).unwrap_or_else(|| draught_tox(it)), t);
        if throws_up(self.seed, who as u64, it, t.to_bits(), tox) {
            self.say(t, format!("{name} retches it straight back up."));
            self.people[who as usize].recompute_might();
            return true;
        }
        for e in draught_effects(it, mix.as_ref()) {
            self.take_dose(who, &e);
        }
        self.people[who as usize].recompute_might();
        true
    }

    /// One effect of something drunk, on the drinker, out of a fight. Side
    /// effects reach them too (`apply_effect` alone keeps attacks for fights).
    pub fn take_dose(&mut self, who: PersonId, e: &Effect) {
        self.take_dose_at(who, e, self.time);
    }

    /// The same, at `t` (something eaten on the condition timeline: rule 8).
    pub fn take_dose_at(&mut self, who: PersonId, e: &Effect, t: f64) {
        let Lasts::Secs(secs) = e.lasts else {
            self.apply_effect(who, e, 1.0);
            return;
        };
        match e.does {
            // Poison runs its course at once.
            Does::Poison => {
                let name = self.name_of(who);
                self.say(t, format!("{name} is poisoned."));
                self.wound_whole(who, e.power * secs, t);
            }
            // Nausea: breath gone at once, and the stomach's worse for it.
            Does::Nausea => {
                if self.people[who as usize].cond.is_some() {
                    self.settle_condition(who, t);
                    if let Some(c) = self.people[who as usize].cond.as_mut() {
                        c.stamina = (c.stamina - e.power * secs).max(0.0);
                    }
                    self.settle_condition(who, t);
                }
                self.add_toxicity(who, e.power * NAUSEA_TOX, t);
            }
            d if d.harmful() => self.add_boon_at(who, d, e.power, t + secs as f64, t),
            _ => {
                self.apply_effect(who, e, 1.0);
            }
        }
    }
}
