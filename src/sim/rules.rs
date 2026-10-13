//! Rules the player can switch off (Laz, 13 Oct): the systems that make the
//! game harder or more fiddly, each a toggle, so he can feel whether the
//! game is more or less fun without them. They're part of the world (saved
//! with it, so a save keeps its rules), read by each system at its decision
//! point: nothing is torn out, the rule just answers "no" there.
//!
//! The table `RULES` is the data the settings panel lists; a new toggle is
//! a field here, a line in the table, and the `if !self.rules.x` at the
//! system's decision. Everything is on unless switched off.

use serde::{Deserialize, Serialize};

fn yes() -> bool {
    true
}

/// What's switched on. Every field defaults to on (an older save has none
/// of them).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Cold, soaking, walking pace, sight and hearing, archery in the
    /// weather (`cold.rs`, `soak.rs`, `squad.rs`, `stealth.rs`, `fights.rs`).
    #[serde(default = "yes")]
    pub weather_on_squad: bool,
    /// Fires and torches drowned by rain (`torch.rs` `drowned_by`).
    #[serde(default = "yes")]
    pub weather_on_things: bool,
    /// Fire and water reaching what's carried: paper burns or soaks, pitch
    /// scorches, packs soak through (`elements.rs`, `soak.rs`).
    #[serde(default = "yes")]
    pub elements_on_items: bool,
    /// Bottles cracking in a fight or a fall (`combat.rs` `crack_bottle`).
    #[serde(default = "yes")]
    pub potions_break: bool,
    /// Gear wearing with use and rot (`wear.rs`).
    #[serde(default = "yes")]
    pub gear_wears: bool,
    /// Thirst (`thirst.rs`, `condition.rs`).
    #[serde(default = "yes")]
    pub thirst: bool,
    /// Food going off (`spoil.rs`).
    #[serde(default = "yes")]
    pub food_spoils: bool,
    /// Potions' toxicity (`toxic.rs`).
    #[serde(default = "yes")]
    pub toxicity: bool,
    /// Wounds that bleed until bound (`bleeding.rs`, `combat.rs`).
    #[serde(default = "yes")]
    pub bleeding: bool,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules { weather_on_squad: true, weather_on_things: true, elements_on_items: true, potions_break: true, gear_wears: true, thirst: true, food_spoils: true, toxicity: true, bleeding: true }
    }
}

/// A rule as the settings panel shows it.
pub struct Rule {
    pub key: &'static str,
    pub name: &'static str,
    pub about: &'static str,
}

const fn r(key: &'static str, name: &'static str, about: &'static str) -> Rule {
    Rule { key, name, about }
}

/// The rules, in the order the panel lists them.
pub const RULES: &[Rule] = &[
    r("weather_on_squad", "Weather on the squad", "Cold, soaking, slower walking, shorter sight and hearing, worse archery in bad weather"),
    r("weather_on_things", "Weather on fires", "Campfires and standing torches drowned by heavy rain"),
    r("elements_on_items", "Fire and water on carried things", "Paper burns or soaks, pitch scorches, packs soak through"),
    r("potions_break", "Potions break", "A heavy blow, a fall or fire may crack a bottle (a potion case keeps them whole)"),
    r("gear_wears", "Gear wears", "Weapons and armour wear with use and rot when neglected"),
    r("thirst", "Thirst", "The squad needs water as well as food"),
    r("food_spoils", "Food goes off", "Fresh food spoils; eating it makes them sick"),
    r("toxicity", "Potions have toxicity", "Each draught adds to a meter: queasy, sick, collapsed"),
    r("bleeding", "Wounds bleed", "A cut bleeds until it's bound; a bleed can kill"),
];

impl super::world::World {
    /// The squad's step: a rule switched since the squad's conditions were
    /// last settled settles them now (thirst, bleeds, food going off, packs
    /// soaking), so a switch takes hold at once. The window also calls it
    /// when a switch is flipped, so it shows at once with time paused.
    pub fn rules_step(&mut self) {
        if self.rules == self.rules_applied {
            return;
        }
        self.rules_applied = self.rules;
        let t = self.time;
        for m in self.squad.members.clone() {
            self.settle_condition(m, t);
            let rate = self.food_rate(m, t);
            self.settle_food(m, rate, t);
        }
        self.update_soaking();
    }
}

impl Rules {
    pub fn get(&self, key: &str) -> Option<bool> {
        Some(match key {
            "weather_on_squad" => self.weather_on_squad,
            "weather_on_things" => self.weather_on_things,
            "elements_on_items" => self.elements_on_items,
            "potions_break" => self.potions_break,
            "gear_wears" => self.gear_wears,
            "thirst" => self.thirst,
            "food_spoils" => self.food_spoils,
            "toxicity" => self.toxicity,
            "bleeding" => self.bleeding,
            _ => return None,
        })
    }

    /// Switch one on or off. False if there's no such rule.
    pub fn set(&mut self, key: &str, on: bool) -> bool {
        let slot = match key {
            "weather_on_squad" => &mut self.weather_on_squad,
            "weather_on_things" => &mut self.weather_on_things,
            "elements_on_items" => &mut self.elements_on_items,
            "potions_break" => &mut self.potions_break,
            "gear_wears" => &mut self.gear_wears,
            "thirst" => &mut self.thirst,
            "food_spoils" => &mut self.food_spoils,
            "toxicity" => &mut self.toxicity,
            "bleeding" => &mut self.bleeding,
            _ => return false,
        };
        *slot = on;
        true
    }

    /// Flip one.
    pub fn toggle(&mut self, key: &str) {
        if let Some(v) = self.get(key) {
            self.set(key, !v);
        }
    }

    /// Apply a list like `potions_break=off,thirst=off,bleeding=on`
    /// (`GAHT_RULES`). Unknown keys and values are ignored.
    pub fn apply(&mut self, list: &str) {
        for part in list.split(',') {
            let Some((k, v)) = part.split_once('=') else { continue };
            let on = match v.trim() {
                "on" | "1" | "true" | "yes" => true,
                "off" | "0" | "false" | "no" => false,
                _ => continue,
            };
            self.set(k.trim(), on);
        }
    }

    /// Everything on?
    pub fn all_on(&self) -> bool {
        RULES.iter().all(|r| self.get(r.key) == Some(true))
    }
}
