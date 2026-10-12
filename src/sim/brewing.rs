//! Brewing at the alchemy table (Part 4 of `docs/items-design.md`; Laz's
//! option (a): each bottle carries its own mix, `items::herbs::Mix`).
//!
//! - **Only at a table.** A mortar and pestle no longer brews from the pack;
//!   it's one of the table's tools. What's fitted is read where the table
//!   stands (`table_kit`): a town alchemist's has every tool, a healing
//!   house a mortar and a retort, any other a mortar; a base's table has the
//!   tools kept in the base's store.
//! - **A brew** is 2 to 4 ingredients of at least two kinds. The bottle
//!   carries every effect shared by two or more of them, as strong and as
//!   long as the brewer's Alchemy and the table's tools make it. It takes
//!   `BREW_SECS` a bottle, on the clock like any making (`do_brewing`).
//! - **What's known is per person** (`Lore`): brewing shows the brewer every
//!   effect the ingredients shared (an experiment), tasting a raw ingredient
//!   shows one unknown effect (a keyed roll) with a weak dose of it and a
//!   little toxicity, and a brewed mix is remembered by its name to be
//!   brewed again.

use serde::{Deserialize, Serialize};

use super::crafting::Station;
use super::effects::{Effect, Lasts};
use super::geo::V2;
use super::items::herbs::{self, Alch, Kit, Mix, MIX_INGREDIENTS};
use super::items::{self, item, ItemId};
use super::materials::{Craft, Grade, Piece};
use super::person::PersonId;
use super::rng::Rng;
use super::stats::Skill;
use super::world::World;

/// How long one bottle takes at the table.
pub const BREW_SECS: f64 = 20.0 * 60.0;
/// Toxicity a taste adds, and how strong its dose is beside a plain one.
pub const TASTE_TOX: f32 = 3.0;
pub const TASTE_SHARE: f32 = 0.3;
/// Skill gained by a bottle brewed, and by a brew that came to nothing.
const BREW_TRAIN: f32 = 2.0;
const EMPTY_TRAIN: f32 = 0.8;

/// What someone knows of the alchemist's art.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Lore {
    /// Each ingredient's known effects: bit k for its k-th effect.
    pub known: Vec<(ItemId, u8)>,
    /// Mixes remembered, to brew again.
    pub mixes: Vec<Remembered>,
}

/// A remembered mix: its name, and what goes in it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Remembered {
    pub name: String,
    pub ings: Vec<ItemId>,
}

/// A brew under way at a table.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BrewJob {
    pub who: PersonId,
    pub ings: Vec<ItemId>,
    pub mix: Mix,
    pub done_at: f64,
    /// Bottles still to brew after this one.
    pub left: u16,
    /// Where the table is.
    pub at: V2,
}

/// Why a brew can't go ahead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoBrew {
    /// Not at an alchemy table.
    NoTable,
    /// Hasn't taken up alchemy.
    Unknown,
    /// At other work.
    Busy,
    /// Fewer than two ingredients, or more than four.
    Count,
    /// All of one kind.
    OneKind,
    /// Not an ingredient.
    NotIngredient(ItemId),
    /// Short of this.
    Missing(ItemId),
}

impl NoBrew {
    pub fn say(&self) -> String {
        match *self {
            NoBrew::NoTable => "has to be done at an alchemy table".to_string(),
            NoBrew::Unknown => "hasn't taken up alchemy".to_string(),
            NoBrew::Busy => "is at other work".to_string(),
            NoBrew::Count => "takes two to four ingredients".to_string(),
            NoBrew::OneKind => "needs ingredients of two different kinds".to_string(),
            NoBrew::NotIngredient(it) => format!("{} isn't something to brew with", item(it).name),
            NoBrew::Missing(it) => format!("needs more {}", item(it).name.to_lowercase()),
        }
    }
}

/// How many of each ingredient a brew takes (one of each listed; the same
/// one listed twice takes two).
fn needs(ings: &[ItemId]) -> Vec<(ItemId, u16)> {
    let mut out: Vec<(ItemId, u16)> = Vec::new();
    for &i in ings {
        match out.iter_mut().find(|x| x.0 == i) {
            Some(x) => x.1 += 1,
            None => out.push((i, 1)),
        }
    }
    out
}

impl World {
    /// What someone knows of the alchemist's art.
    pub fn lore_of(&self, who: PersonId) -> Option<&Lore> {
        self.lore.get(&who)
    }

    /// An ingredient's four effects as `who` knows them (None: not yet).
    pub fn known_effects(&self, who: PersonId, ing: ItemId) -> [Option<Alch>; 4] {
        let Some(def) = items::ingredient(ing) else { return [None; 4] };
        let bits = self.lore.get(&who).and_then(|l| l.known.iter().find(|k| k.0 == ing)).map(|k| k.1).unwrap_or(0);
        std::array::from_fn(|k| (bits & (1 << k) != 0).then_some(def.effects[k]))
    }

    /// `who` now knows this ingredient carries `a` (true if it was new).
    pub fn learn_effect(&mut self, who: PersonId, ing: ItemId, a: Alch) -> bool {
        let Some(def) = items::ingredient(ing) else { return false };
        let mask = def.effects.iter().enumerate().filter(|(_, e)| **e == a).fold(0u8, |m, (k, _)| m | (1 << k));
        if mask == 0 {
            return false;
        }
        let lore = self.lore.entry(who).or_default();
        match lore.known.iter_mut().find(|k| k.0 == ing) {
            Some(k) if k.1 & mask == mask => false,
            Some(k) => {
                k.1 |= mask;
                true
            }
            None => {
                lore.known.push((ing, mask));
                true
            }
        }
    }

    /// The tools fitted to the alchemy table at `at`.
    pub fn table_kit(&self, at: V2) -> Kit {
        let c = Some(Grade::Common);
        // A base's table: the tools kept in the base's store.
        if let Some(b) = self.bases.iter().find(|b| b.buildings.iter().any(|bl| bl.standing() && bl.def().station == Some(Station::AlchemyTable) && bl.at.dist(at) < 1.0)) {
            let has = |k: &str| b.store.iter().any(|e| e.0 == items::id(k)).then_some(Grade::Common);
            return Kit { mortar: has("mortar_and_pestle"), retort: has("retort"), calcinator: has("calcinator"), alembic: has("alembic") };
        }
        // A town's: what its trade can afford.
        use super::jobs::PlaceKind as P;
        let near = self.society.towns.iter().flat_map(|tl| tl.places.iter()).filter(|p| p.pos.dist(at) <= p.kind.size()).min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))).map(|p| p.kind);
        match near {
            Some(P::AlchemyTable) => Kit { mortar: c, retort: c, calcinator: c, alembic: c },
            Some(P::HealingHouse) => Kit { mortar: c, retort: c, ..Kit::default() },
            _ => Kit { mortar: c, ..Kit::default() },
        }
    }

    /// The alchemy table `who` stands at, if any.
    pub fn table_for(&self, who: PersonId) -> Option<V2> {
        self.station_near(self.person_pos(who), Station::AlchemyTable)
    }

    /// What a brew of these by `who` at the table where they stand would
    /// come out as (the same numbers `brew` will use).
    pub fn brew_preview(&self, who: PersonId, ings: &[ItemId]) -> Mix {
        let skill = self.people[who as usize].effective_stats().skill(Skill::Alchemy);
        let kit = self.table_for(who).map(|at| self.table_kit(at)).unwrap_or_default();
        Mix::brew(&herbs::shared(ings), skill, kit)
    }

    /// Everything in the way of `who` brewing these where they stand.
    pub fn brew_blockers(&self, who: PersonId, ings: &[ItemId]) -> Vec<NoBrew> {
        let mut out = Vec::new();
        if self.table_for(who).is_none() {
            out.push(NoBrew::NoTable);
        }
        if !self.knows_craft(who, Craft::Alchemy) {
            out.push(NoBrew::Unknown);
        }
        if self.crafting.iter().any(|j| j.who == who) || self.brewing.iter().any(|j| j.who == who) || self.fighting.contains_key(&who) {
            out.push(NoBrew::Busy);
        }
        if ings.len() < 2 || ings.len() > MIX_INGREDIENTS {
            out.push(NoBrew::Count);
        }
        if let Some(&bad) = ings.iter().find(|&&i| items::ingredient(i).is_none()) {
            out.push(NoBrew::NotIngredient(bad));
        } else if ings.len() >= 2 && ings.iter().all(|&i| items::ingredient(i).map(|d| d.kind) == items::ingredient(ings[0]).map(|d| d.kind)) {
            out.push(NoBrew::OneKind);
        }
        for (i, n) in needs(ings) {
            if self.craft_stock(who, item(i).key) < n {
                out.push(NoBrew::Missing(i));
            }
        }
        out
    }

    /// Brew `count` bottles of these ingredients, one after another; the
    /// first starts now.
    pub fn brew(&mut self, who: PersonId, ings: &[ItemId], count: u16) -> Result<(), NoBrew> {
        self.start_brew(who, ings, count.max(1) - 1, self.time)
    }

    /// Start one bottle as of `t` (now, or the moment the one before ended).
    fn start_brew(&mut self, who: PersonId, ings: &[ItemId], left: u16, t: f64) -> Result<(), NoBrew> {
        if let Some(&why) = self.brew_blockers(who, ings).first() {
            return Err(why);
        }
        let at = self.table_for(who).unwrap();
        let mix = self.brew_preview(who, ings);
        // The brewer's own first, then from whoever stands with them.
        let givers = self.craft_givers(who);
        for (id, n) in needs(ings) {
            let mut need = n;
            for &g in &givers {
                let Some(d) = self.people[g as usize].detail.as_mut() else { continue };
                while need > 0 && d.gear.take(id) {
                    need -= 1;
                }
            }
        }
        for &g in &givers {
            self.people[g as usize].recompute_might();
        }
        self.brewing.push(BrewJob { who, ings: ings.to_vec(), mix, done_at: t + BREW_SECS, left, at });
        Ok(())
    }

    /// Each brew that has fallen due, earliest first: the bottle into the
    /// brewer's pack, what the ingredients shared learned, the mix
    /// remembered; the next of a run starts the moment this one ended.
    pub(super) fn do_brewing(&mut self) {
        for _ in 0..256 {
            let Some(i) = (0..self.brewing.len()).filter(|&i| self.time >= self.brewing[i].done_at).min_by(|&a, &b| self.brewing[a].done_at.total_cmp(&self.brewing[b].done_at).then(a.cmp(&b))) else { break };
            let j = self.brewing.remove(i);
            self.finish_brew(&j);
            if j.left > 0 {
                if let Err(why) = self.start_brew(j.who, &j.ings, j.left - 1, j.done_at) {
                    let name = self.name_of(j.who);
                    self.say(j.done_at, format!("{name} stops brewing: {}.", why.say()));
                }
            }
        }
    }

    fn finish_brew(&mut self, j: &BrewJob) {
        let t = j.done_at;
        let name = self.name_of(j.who);
        // An experiment: every effect they shared is learned, useful or not.
        let shared = herbs::shared(&j.ings);
        let mut learned = 0;
        for &ing in &j.ings {
            for &a in &shared {
                learned += self.learn_effect(j.who, ing, a) as u32;
            }
        }
        let p = &mut self.people[j.who as usize];
        p.stats.exercise(Skill::Alchemy, if j.mix.is_empty() { EMPTY_TRAIN } else { BREW_TRAIN });
        if j.mix.is_empty() {
            self.say(t, format!("{name}'s brew comes to nothing: murky water, poured away."));
            return;
        }
        if let Some(d) = p.detail.as_mut() {
            d.gear.add_piece(items::id("brewed_potion"), Piece::bottle(j.mix));
        }
        p.recompute_might();
        // Remembered, to brew again.
        let lore = self.lore.entry(j.who).or_default();
        let mut ings = j.ings.clone();
        ings.sort();
        if !lore.mixes.iter().any(|m| m.ings == ings) {
            lore.mixes.push(Remembered { name: j.mix.name(), ings });
        }
        let news = if learned > 0 { format!(" ({learned} effect{} learned)", if learned == 1 { "" } else { "s" }) } else { String::new() };
        self.say(t, format!("{name} brews a {}{news}.", j.mix.name().to_lowercase()));
    }

    /// Taste a raw ingredient: one of its unknown effects is learned (a keyed
    /// roll; it may be the harmful one), felt as a weak dose, and it adds a
    /// little toxicity. False if they have none or can't just now.
    pub fn taste(&mut self, who: PersonId, ing: ItemId) -> bool {
        let t = self.time;
        let Some(def) = items::ingredient(ing) else { return false };
        if self.is_down(who) || self.fighting.contains_key(&who) {
            return false;
        }
        let Some(d) = self.people[who as usize].detail.as_mut() else { return false };
        if !d.gear.take(ing) {
            return false;
        }
        let known = self.known_effects(who, ing);
        let unknown: Vec<usize> = (0..4).filter(|&k| known[k].is_none()).collect();
        let name = self.name_of(who);
        let what = item(ing).name.to_lowercase();
        let tasted = self.lore.get(&who).map(|l| l.known.len() as u64).unwrap_or(0);
        let Some(&k) = (!unknown.is_empty()).then(|| &unknown[Rng::from_keys(&[self.seed, who as u64, ing as u64, tasted, unknown.len() as u64, 0x5441_5354]).below(unknown.len())]) else {
            self.say(t, format!("{name} tastes the {what}: nothing new."));
            self.add_toxicity(who, TASTE_TOX, t);
            return true;
        };
        let a = def.effects[k];
        self.learn_effect(who, ing, a);
        self.say(t, format!("{name} tastes the {what}: {}.", a.name().to_lowercase()));
        for e in a.dose() {
            let mut e: Effect = *e;
            e.power *= TASTE_SHARE;
            if let Lasts::Secs(s) = e.lasts {
                e.lasts = Lasts::Secs(s * TASTE_SHARE);
            }
            self.take_dose(who, &e);
        }
        self.add_toxicity(who, TASTE_TOX, t);
        self.people[who as usize].recompute_might();
        true
    }

    /// How far along someone's brew is, 0..1.
    pub fn brew_progress(&self, who: PersonId) -> Option<f32> {
        let j = self.brewing.iter().find(|j| j.who == who)?;
        Some((1.0 - (j.done_at - self.time) / BREW_SECS).clamp(0.0, 1.0) as f32)
    }
}
