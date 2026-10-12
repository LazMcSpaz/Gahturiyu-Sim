//! Town alchemists brew for the shelf (Part 4 of `docs/items-design.md`,
//! rule 11: "Town alchemists brew from what their store holds; cheap
//! potions on sale often carry side effects; Fine and better don't").
//!
//! - **What they know** is data (`KNOWN_MIXES`): a mix's name and its
//!   ingredients, from the approved list and only ingredients a town's store
//!   can hold. Some are clean; some, made of the commonest things, carry a
//!   side effect (a cheap healing draught that dazes a little).
//! - **They work by the town's making rules** (rule 17): at the top of the
//!   hour an alchemist with nothing in hand picks a mix (one roll keyed to
//!   them and the hour) from those whose ingredients the store holds and the
//!   shelf lacks, the ingredients leave the store then, the hours settle on
//!   the hour, and after `BREW_HOURS` the bottles go on the shelf. A shelved
//!   bottle is a `brewed_potion` carrying its mix (`Piece::bottle`), as
//!   strong as the alchemist's `work_skill` and their table's tools
//!   (`table_kit`) make it.
//! - **Cheap and clean.** A clean mix is liked twice as well as one that
//!   harms. A side effect is only ever on a bottle that comes out below
//!   Fine: an alchemist whose hand would make a harmful mix Fine won't brew
//!   it (they'd put their name to nothing that harms), so Fine and better
//!   bottles on sale never do.
//!
//! The old fixed potions (`healing_draught`, `mana_tonic`) stay as they
//! were: the squad can still make them, and merchants sell them as wares.

use super::items::herbs::{self, Kit, Mix};
use super::items::{self, ItemId};
use super::jobs::{good_of, Job};
use super::making::{Making, Shelved, SHELF_CAP, SHELF_EACH};
use super::materials::{Craft, Grade, Piece};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::world::World;

/// A mix town alchemists know.
#[derive(Clone, Copy, Debug)]
pub struct KnownMix {
    /// Its key (kept with the work in hand).
    pub key: &'static str,
    /// One of each listed.
    pub ings: &'static [&'static str],
}

const fn known(key: &'static str, ings: &'static [&'static str]) -> KnownMix {
    KnownMix { key, ings }
}

/// The mixes town alchemists brew (Claude's choice from the approved
/// ingredients; what each carries is worked out from them, `herbs::shared`).
pub static KNOWN_MIXES: &[KnownMix] = &[
    // Healing: clean from bloodleaf and berries; cheap from sweetroot and
    // hearthleaf, which daze.
    known("healing", &["bloodleaf", "wild_berries"]),
    known("cheap_healing", &["sweetroot", "hearthleaf"]),
    // Stanching.
    known("stanching", &["bloodleaf", "bluestem"]),
    // Energy (a mage's draught): clean from ghostcap and stonebean; cheap
    // from peatroot and stonebean, which turn the stomach.
    known("energy", &["ghostcap", "stonebean"]),
    known("cheap_energy", &["peatroot", "stonebean"]),
    // Stamina: clean from salt and sweetroot; cheap from brinekale and salt,
    // which weaken.
    known("stamina", &["salt_crystal", "sweetroot"]),
    known("cheap_stamina", &["brinekale", "salt_crystal"]),
    // Night sight, from imported duskseed.
    known("night_sight", &["ghostcap", "duskseed"]),
    // Warmth.
    known("warmth", &["moorbell", "stonebean"]),
    // A clear head.
    known("clear_mind", &["stoneapple", "wild_berries"]),
    // Cleansing.
    known("cleansing", &["peatroot", "hearthleaf"]),
    // Fire's bite kept off (it burns a little going down).
    known("fireproof", &["firepod", "emberroot"]),
    // Swiftness (thins the blood).
    known("swiftness", &["weepmoss", "sunbark"]),
];

/// Hours a town alchemist takes over a brew, and the bottles it makes.
pub const BREW_HOURS: f32 = 2.0;
pub const BREW_BOTTLES: u16 = 2;
/// Most bottles on a town's shelf at once.
pub const SHELF_BOTTLES: usize = 12;
/// How much more a clean mix is liked than one that harms.
pub const CLEAN_WEIGHT: f32 = 2.0;

/// A known mix by key.
pub fn known_mix(key: &str) -> Option<&'static KnownMix> {
    KNOWN_MIXES.iter().find(|m| m.key == key)
}

impl KnownMix {
    pub fn items(&self) -> Vec<ItemId> {
        self.ings.iter().map(|k| items::id(k)).collect()
    }

    /// What it comes out as, by this hand at this table.
    pub fn brewed(&self, skill: f32, kit: Kit) -> Mix {
        Mix::brew(&herbs::shared(&self.items()), skill, kit)
    }
}

/// A brewed bottle's mix, if this is one.
fn bottle_mix(s: &Shelved) -> Option<Mix> {
    s.piece.and_then(|p| p.mix)
}

impl World {
    /// The tools at the table a town alchemist works at.
    pub fn alchemist_kit(&self, who: PersonId) -> Kit {
        match self.workplace_of(who) {
            Some(w) => self.table_kit(w.pos),
            None => Kit { mortar: Some(Grade::Common), ..Kit::default() },
        }
    }

    /// What this alchemist's bottle of a known mix would come out as.
    pub fn town_brew(&self, who: PersonId, m: &KnownMix) -> Mix {
        m.brewed(self.work_skill(who, Craft::Alchemy), self.alchemist_kit(who))
    }

    /// Has the town's store what a mix takes?
    fn store_holds(&self, town: SettlementId, m: &KnownMix) -> bool {
        let st = &self.society.towns[town as usize].stock;
        m.ings.iter().all(|&k| good_of(k).is_some_and(|g| st[g.index()].base >= Self::units_of(g, items::id(k))))
    }

    /// A mix for an alchemist to take up: from those the store holds and the
    /// shelf lacks, a clean one liked better, by one roll keyed to them and
    /// the hour. The ingredients leave the store now.
    pub(super) fn choose_brew(&mut self, town: SettlementId, who: PersonId, h: i64) -> Option<&'static str> {
        let tl = &self.society.towns[town as usize];
        let bottles = tl.shelf.iter().filter(|s| bottle_mix(s).is_some()).count() + tl.making.iter().filter(|m| m.brew.is_some()).count() * BREW_BOTTLES as usize;
        if bottles >= SHELF_BOTTLES || tl.shelf.len() + tl.making.len() >= SHELF_CAP {
            return None;
        }
        let mut picks: Vec<(&'static KnownMix, f32)> = Vec::new();
        for m in KNOWN_MIXES {
            if !self.store_holds(town, m) {
                continue;
            }
            let mix = self.town_brew(who, m);
            if mix.is_empty() {
                continue;
            }
            // Never a side effect on a bottle good enough to be Fine.
            if mix.harms() && mix.grade() >= Grade::Fine {
                continue;
            }
            let name = mix.name();
            let on_shelf = tl.shelf.iter().filter(|s| bottle_mix(s).is_some_and(|x| x.name() == name)).count();
            let in_hand = tl.making.iter().filter(|x| x.brew == Some(m.key)).count() * BREW_BOTTLES as usize;
            if on_shelf + in_hand >= SHELF_EACH {
                continue;
            }
            picks.push((m, if mix.harms() { 1.0 } else { CLEAN_WEIGHT }));
        }
        let mut r = Rng::from_keys(&[self.seed, who as u64, h as u64, 0x4252_4557]);
        let k = r.weighted(&picks.iter().map(|p| p.1).collect::<Vec<_>>())?;
        let m = picks[k].0;
        let st = &mut self.society.towns[town as usize].stock;
        for &key in m.ings {
            if let Some(g) = good_of(key) {
                st[g.index()].base -= Self::units_of(g, items::id(key));
            }
        }
        Some(m.key)
    }

    /// A town brew finished: its bottles go on the shelf.
    pub(super) fn finish_town_brew(&mut self, town: SettlementId, m: &Making) {
        let Some(km) = m.brew.and_then(known_mix) else { return };
        let mix = self.town_brew(m.who, km);
        let tl = &mut self.society.towns[town as usize];
        for _ in 0..BREW_BOTTLES {
            if tl.shelf.len() < SHELF_CAP {
                tl.shelf.push(Shelved { item: items::id("brewed_potion"), piece: Some(Piece::bottle(mix)) });
            }
        }
    }

    /// Is this the work of a town alchemist (their trade brews)?
    pub(super) fn brews(job: Job) -> bool {
        job == Job::Alchemist
    }
}
