//! Magic out in the world: effects worked on people who aren't in a fight
//! (potions drunk on the road, scrolls read, spells cast from the spell
//! book). Fights have their own half of this in `combat`; both read the same
//! effect list (`effects`).

use super::effects::{Does, Effect, Lasts};
use super::person::PersonId;
use super::world::World;

impl World {
    /// Work one effect on `target` (out of a fight). `skill` scales it the
    /// way a caster's skill does (1 for potions and scrolls). Returns whether
    /// it did anything.
    pub fn apply_effect(&mut self, target: PersonId, e: &Effect, skill: f32) -> bool {
        let t = self.time;
        if e.lasts != Lasts::Now {
            return false;
        }
        let p = &mut self.people[target as usize];
        match e.does {
            Does::Heal => {
                let base = p.stats.clone();
                let mut hp = p.wounds.hp_at(&base, t);
                super::crafting::mend(&mut hp, &base, e.power * skill);
                p.wounds.set(&base, &hp, t);
                true
            }
            Does::Energy => {
                let m = (p.mana_at(t) + e.power).min(p.max_mana());
                p.set_mana(m, t);
                true
            }
            _ => false,
        }
    }
}
