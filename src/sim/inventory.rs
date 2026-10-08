//! What a person carries: gear in slots, the rest in their pack, and what all
//! of it weighs.
//!
//! Kenshi rules for weight: everything counts, worn or carried. Under your
//! capacity you move freely; over it you slow down, and well over it you can
//! barely move and fight badly. A pack raises the limit.

use super::items::{self, item, Effect, ItemId, Kind, Slot, WeaponDef, FISTS, SLOTS};
use super::race::Race;
use super::rng::Rng;
use super::stats::{Calling, Skill, Stats, ATTRS, SKILLS};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Gear {
    slots: [Option<ItemId>; 10],
    /// Everything not worn: (item, how many).
    pub bag: Vec<(ItemId, u16)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EquipError {
    NotInBag,
    WrongSlot,
    /// A two-handed weapon needs the off hand free, and vice versa.
    HandsFull,
}

fn slot_index(s: Slot) -> usize {
    SLOTS.iter().position(|&x| x == s).unwrap()
}

impl Gear {
    pub fn in_slot(&self, s: Slot) -> Option<ItemId> {
        self.slots[slot_index(s)]
    }

    pub fn equipped(&self) -> impl Iterator<Item = ItemId> + '_ {
        self.slots.iter().flatten().copied()
    }

    pub fn add(&mut self, id: ItemId, n: u16) {
        if let Some(e) = self.bag.iter_mut().find(|e| e.0 == id) {
            e.1 += n;
        } else {
            self.bag.push((id, n));
        }
    }

    pub fn take(&mut self, id: ItemId) -> bool {
        if let Some(i) = self.bag.iter().position(|e| e.0 == id) {
            self.bag[i].1 -= 1;
            if self.bag[i].1 == 0 {
                self.bag.remove(i);
            }
            true
        } else {
            false
        }
    }

    /// Put something on straight from nowhere (for building NPC kit).
    pub fn wear(&mut self, id: ItemId) {
        self.add(id, 1);
        let _ = self.equip(id);
    }

    /// Move an item from the pack into its slot. Whatever was there goes into
    /// the pack, and a two-handed weapon bumps the off hand (and the other way
    /// round).
    pub fn equip(&mut self, id: ItemId) -> Result<(), EquipError> {
        let def = item(id);
        if !self.bag.iter().any(|e| e.0 == id) {
            return Err(EquipError::NotInBag);
        }
        let two_handed = def.weapon().map(|w| w.two_handed).unwrap_or(false);
        if def.slot == Slot::OffHand {
            if let Some(m) = self.in_slot(Slot::MainHand) {
                if item(m).weapon().map(|w| w.two_handed).unwrap_or(false) {
                    self.unequip(Slot::MainHand);
                }
            }
        }
        if two_handed {
            self.unequip(Slot::OffHand);
        }
        self.take(id);
        self.unequip(def.slot);
        self.slots[slot_index(def.slot)] = Some(id);
        Ok(())
    }

    /// Whatever is in a slot is used up (not put back in the pack).
    pub fn discard(&mut self, s: Slot) -> Option<ItemId> {
        self.slots[slot_index(s)].take()
    }

    /// Take off whatever is in a slot and put it in the pack.
    pub fn unequip(&mut self, s: Slot) -> Option<ItemId> {
        let i = slot_index(s);
        let it = self.slots[i].take();
        if let Some(id) = it {
            self.add(id, 1);
        }
        it
    }

    /// The weapon in hand, or fists.
    pub fn weapon(&self) -> WeaponDef {
        self.in_slot(Slot::MainHand).and_then(|id| item(id).weapon().copied()).unwrap_or(FISTS)
    }

    pub fn weapon_name(&self) -> &'static str {
        self.in_slot(Slot::MainHand).map(|id| item(id).name).unwrap_or("bare hands")
    }

    /// Shield block value, if a shield is held.
    pub fn shield(&self) -> f32 {
        match self.in_slot(Slot::OffHand).map(|id| item(id).kind) {
            Some(Kind::Shield(b)) => b,
            _ => 0.0,
        }
    }

    /// Every effect from everything worn.
    pub fn effects(&self) -> impl Iterator<Item = &'static Effect> + '_ {
        self.equipped().flat_map(|id| item(id).effects.iter())
    }

    pub fn sum_effect(&self, f: impl Fn(&Effect) -> Option<f32>) -> f32 {
        self.effects().filter_map(|e| f(e)).sum()
    }

    /// Kilograms, worn and carried.
    pub fn weight(&self) -> f32 {
        let worn: f32 = self.equipped().map(|id| item(id).weight).sum();
        let packed: f32 = self.bag.iter().map(|(id, n)| item(*id).weight * *n as f32).sum();
        worn + packed
    }

    /// How much this person can carry with this gear on, kg.
    pub fn capacity(&self, stats: &Stats) -> f32 {
        let pack = match self.in_slot(Slot::Back).map(|id| item(id).kind) {
            Some(Kind::Pack(kg)) => kg,
            _ => 0.0,
        };
        let extra = self.sum_effect(|e| if let Effect::Carry(kg) = e { Some(*kg) } else { None });
        effective(stats, self).carry_capacity() + pack + extra
    }

    /// Load as a share of capacity (1.0 = exactly at the limit).
    pub fn load(&self, stats: &Stats) -> f32 {
        self.weight() / self.capacity(stats).max(1.0)
    }

    /// Total value, coin.
    pub fn value(&self) -> f32 {
        self.equipped().map(|id| item(id).value).sum::<f32>() + self.bag.iter().map(|(id, n)| item(*id).value * *n as f32).sum::<f32>()
    }
}

/// Stats as they stand with gear on: enchantments added on top of the base.
pub fn effective(base: &Stats, gear: &Gear) -> Stats {
    let mut s = base.clone();
    for e in gear.effects() {
        match *e {
            Effect::Attr(a, v) => {
                let cur = s.attr(a);
                s.set_attr(a, cur + v);
            }
            Effect::Skill(k, v) => {
                let cur = s.skill(k);
                s.set_skill(k, cur + v);
            }
            _ => {}
        }
    }
    let _ = (ATTRS, SKILLS);
    s
}

/// Speed multiplier from carrying too much.
pub fn encumbrance_factor(load: f32) -> f32 {
    if load <= 1.0 {
        1.0
    } else if load <= 2.0 {
        (1.0 - (load - 1.0) * 0.75).max(0.25)
    } else {
        0.1
    }
}

/// Starting kit for someone, spending roughly `budget` coin according to
/// their calling, best skill and people. Pure function of its inputs, so the
/// kit a far-off stranger is assumed to have is exactly the kit they turn up
/// with when you meet them.
pub fn starting_kit(race: Race, stats: &Stats, budget: f32, seed: u64) -> Gear {
    let mut rng = Rng::from_keys(&[seed, 0x4B49_5400]);
    let mut g = Gear::default();
    let mut left = budget;
    let buy = |g: &mut Gear, key: &str, left: &mut f32| {
        let id = items::id(key);
        g.wear(id);
        *left -= item(id).value;
    };
    // Everyone has clothes.
    buy(&mut g, "cloth_shirt", &mut left);
    buy(&mut g, "trousers", &mut left);

    // A weapon to suit their best skill, as good as the purse allows.
    let best = [Skill::Blade, Skill::Blunt, Skill::Spear, Skill::Marksman]
        .into_iter()
        .max_by(|a, b| stats.skill(*a).total_cmp(&stats.skill(*b)))
        .unwrap();
    let ladder: &[&str] = if stats.calling == Calling::Mage {
        &["staff"]
    } else {
        match (best, race) {
            (Skill::Blade, _) => &["knife", "short_sword", "longsword"],
            (Skill::Marksman, _) => &["short_bow", "crossbow"],
            (Skill::Blunt, Race::Roduro) => &["club", "stone_maul"],
            (Skill::Blunt, Race::Qotiro) => &["club", "war_pick"],
            (Skill::Blunt, _) => &["club", "war_pick"],
            (_, Race::Horaro) => &["spear", "harpoon", "glaive"],
            _ => &["spear", "glaive"],
        }
    };
    let weapon_share = if stats.calling == Calling::Common { 0.6 } else { 0.4 };
    let mut pick = None;
    for key in ladder {
        if item(items::id(key)).value <= (left * weapon_share).max(12.0) {
            pick = Some(*key);
        }
    }
    if let Some(k) = pick.or(if budget > 25.0 { Some(ladder[0]) } else { None }) {
        buy(&mut g, k, &mut left);
        // A bow needs something to shoot, and a knife for when they close in.
        if let Some(ammo) = g.weapon().ammo {
            g.add(items::id(ammo), 24);
            left -= 24.0 * item(items::id(ammo)).value;
            g.add(items::id("knife"), 1);
            left -= item(items::id("knife")).value;
        }
    }

    // Then armour, most protective first, while money lasts.
    let two_handed = g.weapon().two_handed;
    let mut wishlist: Vec<&str> = vec![];
    if stats.calling != Calling::Mage {
        wishlist.extend(["scale_hauberk", "iron_helm", "scale_greaves"]);
        if race == Race::Qotiro {
            wishlist.insert(0, "sandstone_lamellar");
        }
    }
    wishlist.extend(["hide_coat", "padded_jacket", "leather_cap", "hide_leggings", "boots", "leather_gloves"]);
    if !two_handed && stats.calling == Calling::Warrior {
        wishlist.insert(1, "kite_shield");
        wishlist.push("buckler");
    }
    if rng.chance(0.6) {
        wishlist.push("small_pack");
    }
    for key in wishlist {
        let id = items::id(key);
        let def = item(id);
        let filled = g.in_slot(def.slot).map(|cur| item(cur).value >= def.value).unwrap_or(false);
        if !filled && def.value <= left {
            // Replacing cheap clothes: the old piece goes in the pack.
            buy(&mut g, key, &mut left);
        }
    }
    // Spare clothes don't travel; drop anything replaced.
    g.bag.retain(|(id, _)| !matches!(item(*id).key, "cloth_shirt" | "trousers"));
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::race::Traits;

    fn stats() -> Stats {
        Stats::generate(Race::Roduro, &Traits { wanderlust: 0.5, boldness: 0.5, sociability: 0.5, patience: 0.5 }, 1)
    }

    #[test]
    fn two_handed_weapons_need_both_hands() {
        let mut g = Gear::default();
        g.wear(items::id("kite_shield"));
        g.add(items::id("glaive"), 1);
        g.equip(items::id("glaive")).unwrap();
        assert_eq!(g.in_slot(Slot::OffHand), None, "the shield should come off");
        assert!(g.bag.iter().any(|e| e.0 == items::id("kite_shield")));
        g.equip(items::id("kite_shield")).unwrap();
        assert_eq!(g.in_slot(Slot::MainHand), None, "the glaive should come off");
    }

    #[test]
    fn weight_counts_worn_and_carried_and_packs_help() {
        let s = stats();
        let mut g = Gear::default();
        let base_cap = g.capacity(&s);
        g.add(items::id("scale_hauberk"), 5);
        assert!((g.weight() - 70.0).abs() < 0.01);
        let heavy = g.load(&s);
        g.wear(items::id("large_pack"));
        assert!((g.capacity(&s) - base_cap - 45.0).abs() < 0.01);
        assert!(g.load(&s) < heavy);
        assert!(encumbrance_factor(heavy) < 1.0);
    }

    #[test]
    fn richer_people_are_better_equipped() {
        let s = stats();
        let poor = starting_kit(Race::Roduro, &s, 20.0, 5);
        let rich = starting_kit(Race::Roduro, &s, 600.0, 5);
        assert!(rich.value() > poor.value() * 4.0, "{} vs {}", rich.value(), poor.value());
        assert_eq!(starting_kit(Race::Roduro, &s, 300.0, 9), starting_kit(Race::Roduro, &s, 300.0, 9));
    }
}
