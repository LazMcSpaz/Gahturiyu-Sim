//! Magic out in the world: spells cast from the spell book, rituals
//! performed and held, potions drunk on the road, and learning new spells.
//! Fights have their own half of this in `combat`; both read the same effect
//! list (`effects`).
//!
//! **Rituals** take minutes to hours of standing still (never mid-fight).
//! The components are used up when it begins, and any health it asks for is
//! given then too (a wound like any other). When the time is up the dice
//! decide: a success is **held ready** — one at a time — and released
//! whenever wanted, even mid-fight; a failure is **backlash**, which hurts.
//! Holding a ritual drains stamina, and one still held when its caster next
//! sleeps slips away. Walking off breaks a ritual off (the components are
//! still gone).
//!
//! **Learning**: felt spells come by use (see `fights`); structured spells
//! from notes or a teacher, rituals from rare texts or a teacher.

use serde::{Deserialize, Serialize};

use super::effects::{Does, Effect, Lasts, Reach, Who};
use super::geo::V2;
use super::items::{self, item, ItemId, Kind};
use super::magic::{self, Place, Spell, Style};
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::BuildingKind;
use super::stats::Skill;
use super::world::World;

/// Within this many metres of a hearth counts as "at the hearth".
pub const HEARTH_REACH: f32 = 15.0;
/// Within this many metres of a temple or hall counts as "at a shrine".
pub const SHRINE_REACH: f32 = 20.0;
/// Drawing a fresh circle adds this many minutes to a ritual.
pub const CIRCLE_MINUTES: f32 = 20.0;
/// A circle already drawn within this many metres can be used again.
pub const CIRCLE_REUSE: f32 = 3.0;
/// Stamina drained per hour while holding a ritual ready.
pub const STAMINA_HOLD: f32 = 15.0;
/// Backlash from a failed ritual: this much damage, plus a fifth of the
/// skill it asks for.
pub const BACKLASH: f32 = 14.0;
/// Learning from notes or texts: style skill needed, as a share of the
/// spell's own requirement. Teachers get you there from a little lower.
pub const READ_SKILL: f32 = 0.9;
pub const TAUGHT_SKILL: f32 = 0.75;

/// A ritual being performed.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct RitualJob {
    pub who: PersonId,
    pub spell: Spell,
    pub started: f64,
    pub done_at: f64,
    /// Where they stand (moving away breaks it off).
    pub at: V2,
    /// Which cast this is for them (keys the roll).
    pub n: u64,
}

/// Why something can't be done.
#[derive(Clone, Debug, PartialEq)]
pub struct Cannot(pub String);

fn cannot<T>(s: impl Into<String>) -> Result<T, Cannot> {
    Err(Cannot(s.into()))
}

impl World {
    // ---- Who knows what ---------------------------------------------------

    /// The spells this person knows (empty for people not yet met).
    pub fn known_spells(&self, pid: PersonId) -> Vec<Spell> {
        self.people[pid as usize].detail.as_ref().map(|d| d.spells.clone()).unwrap_or_default()
    }

    pub fn knows(&self, pid: PersonId, s: Spell) -> bool {
        self.people[pid as usize].detail.as_ref().map(|d| d.spells.contains(&s)).unwrap_or(false)
    }

    /// The ritual someone is holding ready, if any.
    pub fn held_ritual(&self, pid: PersonId) -> Option<Spell> {
        self.held.get(&pid).copied()
    }

    /// The ritual someone is in the middle of, and how far along (0..1).
    pub fn ritual_progress(&self, pid: PersonId) -> Option<(Spell, f32)> {
        let j = self.rituals.iter().find(|j| j.who == pid)?;
        Some((j.spell, ((self.time - j.started) / (j.done_at - j.started)).clamp(0.0, 1.0) as f32))
    }

    fn next_cast(&mut self, pid: PersonId) -> u64 {
        let n = self.cast_count.entry(pid).or_insert(0);
        *n += 1;
        *n
    }


    // ---- Casting outside a fight ------------------------------------------

    /// Cast a felt or structured spell outside a fight (from the spell book).
    /// `target` is who it's aimed at (a squad member), if anyone. Energy is
    /// spent whether or not it works.
    pub fn cast(&mut self, who: PersonId, s: Spell, target: Option<PersonId>) -> Result<(), Cannot> {
        let d = s.def();
        if !self.knows(who, s) {
            return cannot("doesn't know that spell");
        }
        if self.fighting.contains_key(&who) {
            return cannot("in a fight (it's cast there by itself)");
        }
        if d.style == Style::Ritual {
            return cannot("a ritual has to be performed");
        }
        if !d.works_outside_fights() {
            return cannot("only of use in a fight");
        }
        let t = self.time;
        let mana = self.people[who as usize].mana_at(t);
        if mana < d.cost {
            return cannot("not enough energy");
        }
        let p = &mut self.people[who as usize];
        p.set_mana(mana - d.cost, t);
        self.tire(who, d.tire);
        let n = self.next_cast(who);
        let roll = Rng::from_keys(&[self.seed, who as u64, n, 0x4341_5354]).f32();
        let p = &self.people[who as usize];
        let chance = magic::success_chance(&p.effective_stats(), s, 1.0);
        let name = self.name_of(who);
        if roll > chance {
            self.people[who as usize].stats.exercise(d.skill(), 0.4);
            self.say(t, format!("{name}'s {} fizzles.", d.name.to_lowercase()));
            return Ok(());
        }
        self.people[who as usize].stats.exercise(d.skill(), 1.5);
        let skill = magic::skill_power(&self.people[who as usize].effective_stats(), s);
        self.work_spell(who, s, target, skill);
        self.say(t, format!("{name} casts {}.", d.name.to_lowercase()));
        Ok(())
    }

    /// Tiredness from felt casting (squad members).
    pub(super) fn tire(&mut self, pid: PersonId, amount: f32) {
        if amount <= 0.0 || self.people[pid as usize].cond.is_none() {
            return;
        }
        let t = self.time;
        self.settle_condition(pid, t);
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.tired = (c.tired + amount).min(100.0);
        }
    }

    /// Carry out a spell's effects (it's already been paid for and worked).
    fn work_spell(&mut self, by: PersonId, s: Spell, target: Option<PersonId>, skill: f32) {
        let point = self.person_pos(by);
        for e in s.def().effects {
            for pid in self.reached(by, e, target, point) {
                self.apply_effect(pid, e, skill);
            }
        }
    }

    /// Who an effect reaches outside a fight. Areas reach the squad (it's the
    /// only company a caster keeps out here).
    fn reached(&self, by: PersonId, e: &Effect, target: Option<PersonId>, point: V2) -> Vec<PersonId> {
        match e.reach {
            Reach::Caster => vec![by],
            Reach::Target => vec![target.unwrap_or(by)],
            Reach::Area { radius, who: Who::Friends | Who::All } if self.squad.index(by).is_some() => {
                self.squad.members.iter().copied().filter(|&m| !self.people[m as usize].dead && self.person_pos(m).dist(point) <= radius).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Work one effect on `target` (out of a fight). `skill` scales it the
    /// way a caster's skill does (1 for potions and scrolls). Returns whether
    /// it did anything.
    pub fn apply_effect(&mut self, target: PersonId, e: &Effect, skill: f32) -> bool {
        let t = self.time;
        if e.lasts != Lasts::Now {
            return false;
        }
        match e.does {
            Does::Heal => {
                if self.people[target as usize].cond.is_some() {
                    self.settle_condition(target, t);
                }
                let p = &mut self.people[target as usize];
                let base = p.stats.clone();
                let mut hp = p.wounds.hp_at(&base, t);
                super::crafting::mend(&mut hp, &base, e.power * skill);
                p.wounds.set(&base, &hp, t);
                if p.cond.is_some() {
                    self.settle_condition(target, t);
                }
                true
            }
            Does::Energy => {
                let p = &mut self.people[target as usize];
                let m = (p.mana_at(t) + e.power).min(p.max_mana());
                p.set_mana(m, t);
                true
            }
            Does::Rest => {
                if self.people[target as usize].cond.is_none() {
                    return false;
                }
                self.settle_condition(target, t);
                if let Some(c) = self.people[target as usize].cond.as_mut() {
                    c.tired = (c.tired - e.power * skill).max(0.0);
                    c.stamina = c.max_stamina;
                }
                true
            }
            _ => false,
        }
    }

    // ---- Rituals ----------------------------------------------------------

    /// Is `at` a fit place for this ritual? Also says how many extra minutes
    /// drawing a circle would add.
    pub fn ritual_place(&self, place: Place, at: V2) -> Result<f32, Cannot> {
        let near = |kinds: &[BuildingKind], reach: f32| self.settlements.iter().flat_map(|s| s.buildings.iter()).any(|b| kinds.contains(&b.kind) && b.pos.dist(at) <= reach + b.size * 0.5);
        match place {
            Place::Anywhere => Ok(0.0),
            Place::Hearth if near(&[BuildingKind::Hearth], HEARTH_REACH) => Ok(0.0),
            Place::Hearth => cannot("has to be done at a hearth"),
            Place::Shrine if near(&[BuildingKind::QotiroTemple, BuildingKind::QotiroHall], SHRINE_REACH) => Ok(0.0),
            Place::Shrine => cannot("has to be done at a shrine"),
            Place::Circle if self.circles.iter().any(|c| c.dist(at) <= CIRCLE_REUSE) => Ok(0.0),
            Place::Circle => Ok(CIRCLE_MINUTES),
        }
    }

    /// Why someone can't begin a ritual right now, if they can't.
    pub fn can_perform(&self, who: PersonId, s: Spell) -> Result<f32, Cannot> {
        let d = s.def();
        if d.style != Style::Ritual {
            return cannot("not a ritual");
        }
        if !self.knows(who, s) {
            return cannot("doesn't know it");
        }
        let Some(k) = self.squad.index(who) else { return cannot("not in your squad") };
        if self.fighting.contains_key(&who) {
            return cannot("not in the middle of a fight");
        }
        if self.rituals.iter().any(|j| j.who == who) {
            return cannot("already performing one");
        }
        if self.held.contains_key(&who) {
            return cannot("already holding one ready");
        }
        let extra = self.ritual_place(d.rite.place, self.member_pos(k))?;
        let gear = &self.people[who as usize].detail.as_ref().unwrap().gear;
        for &(key, n) in d.rite.components {
            let have: u16 = gear.bag.iter().filter(|e| item(e.0).key == key).map(|e| e.1).sum();
            if have < n {
                return cannot(format!("needs {n} × {}", item(items::id(key)).name.to_lowercase()));
            }
        }
        // Giving more blood than you have is no ritual at all.
        if d.rite.health > 0.0 {
            let p = &self.people[who as usize];
            let hp = p.wounds.hp_at(&p.stats, self.time);
            if hp[1] - d.rite.health * super::body::Part::Torso.hit_weight() < p.stats.max_hp(super::body::Part::Torso) * 0.25 {
                return cannot("too badly hurt to give the blood it asks");
            }
        }
        Ok(extra)
    }

    /// Begin a ritual. The components (and any blood) are given now.
    pub fn perform(&mut self, who: PersonId, s: Spell) -> Result<(), Cannot> {
        let extra = self.can_perform(who, s)?;
        let d = s.def();
        let t = self.time;
        let k = self.squad.index(who).unwrap();
        let at = self.member_pos(k);
        if let Some(dd) = self.people[who as usize].detail.as_mut() {
            for &(key, n) in d.rite.components {
                for _ in 0..n {
                    dd.gear.take(items::id(key));
                }
            }
        }
        if d.rite.health > 0.0 {
            self.wound_whole(who, d.rite.health, t);
        }
        if d.rite.place == Place::Circle && extra > 0.0 {
            self.circles.push(at);
        }
        // Stand still for it.
        self.squad.goal[k] = self.squad.at[k];
        self.squad.route[k].clear();
        self.squad.resting[k] = false;
        let n = self.next_cast(who);
        let done_at = t + ((d.rite.minutes + extra) * 60.0) as f64;
        self.rituals.push(RitualJob { who, spell: s, started: t, done_at, at, n });
        self.people[who as usize].recompute_might();
        let name = self.name_of(who);
        let drawing = if extra > 0.0 { "draws a circle and " } else { "" };
        self.say(t, format!("{name} {drawing}begins the {} ritual.", d.name.to_lowercase()));
        Ok(())
    }

    /// Damage spread over the whole body (out of a fight).
    fn wound_whole(&mut self, pid: PersonId, dmg: f32, t: f64) {
        if self.people[pid as usize].cond.is_some() {
            self.settle_condition(pid, t);
        }
        let p = &mut self.people[pid as usize];
        let base = p.stats.clone();
        let mut hp = p.wounds.hp_at(&base, t);
        for (i, part) in super::body::PARTS.iter().enumerate() {
            hp[i] -= dmg * part.hit_weight();
        }
        p.wounds.set(&base, &hp, t);
        if p.cond.is_some() {
            self.settle_condition(pid, t);
        }
    }

    /// Break off rituals whose performer walked away, went down or was
    /// dragged into a fight. (Finishing happens on the condition timeline, at
    /// the exact moment each is done: see `condition`.)
    pub(super) fn update_rituals(&mut self) {
        let now = self.time;
        let jobs = std::mem::take(&mut self.rituals);
        let mut keep = Vec::new();
        for j in jobs {
            let k = self.squad.index(j.who);
            let moved = k.map(|k| self.squad.at[k].dist(j.at) > 1.0 || self.squad.goal[k].dist(j.at) > 1.0).unwrap_or(true);
            let down = self.people[j.who as usize].dead || self.fighting.contains_key(&j.who);
            if (moved || down) && j.done_at > now {
                let name = self.name_of(j.who);
                self.say(now, format!("{name} breaks off the {} ritual.", j.spell.def().name.to_lowercase()));
                continue;
            }
            keep.push(j);
        }
        self.rituals = keep;
    }

    /// When someone's ritual is done, if they're performing one.
    pub(super) fn ritual_due(&self, pid: PersonId) -> Option<f64> {
        self.rituals.iter().find(|j| j.who == pid).map(|j| j.done_at)
    }

    /// The ritual's moment has come (called at exactly `done_at`).
    pub(super) fn ritual_done(&mut self, pid: PersonId) {
        let Some(i) = self.rituals.iter().position(|j| j.who == pid) else { return };
        let j = self.rituals.remove(i);
        self.finish_ritual(j);
    }

    fn finish_ritual(&mut self, j: RitualJob) {
        let d = j.spell.def();
        let t = j.done_at;
        let p = &self.people[j.who as usize];
        let chance = magic::success_chance(&p.effective_stats(), j.spell, 1.0);
        let roll = Rng::from_keys(&[self.seed, j.who as u64, j.n, 0x5249_5445]).f32();
        let name = self.name_of(j.who);
        if roll <= chance {
            self.people[j.who as usize].stats.exercise(Skill::Ritual, 2.0);
            self.set_holding(j.who, t, Some(j.spell));
            self.say(t, format!("{name} holds the {} ready.", d.name.to_lowercase()));
        } else {
            self.people[j.who as usize].stats.exercise(Skill::Ritual, 0.8);
            self.wound_whole(j.who, BACKLASH + d.min_skill / 5.0, t);
            self.say(t, format!("The {} ritual turns on {name}!", d.name.to_lowercase()));
        }
        self.people[j.who as usize].recompute_might();
    }

    /// Start or stop holding a ritual (stamina drains while one is held).
    pub(super) fn set_holding(&mut self, pid: PersonId, t: f64, s: Option<Spell>) {
        if self.people[pid as usize].cond.is_some() {
            self.settle_condition(pid, t);
        }
        match s {
            Some(s) => {
                self.held.insert(pid, s);
            }
            None => {
                self.held.remove(&pid);
            }
        }
        if let Some(c) = self.people[pid as usize].cond.as_mut() {
            c.holding = s.is_some();
        }
    }

    /// Falling asleep lets a held ritual slip away.
    pub(super) fn drop_held_on_sleep(&mut self, pid: PersonId, t: f64) {
        if let Some(s) = self.held.get(&pid).copied() {
            self.set_holding(pid, t, None);
            let name = self.name_of(pid);
            self.say(t, format!("The held {} slips away as {name} sleeps.", s.def().name.to_lowercase()));
        }
    }

    /// Let a held ritual go (outside a fight; in one, the fighter does it).
    /// `target` is who it's aimed at, if it needs anyone.
    pub fn release(&mut self, who: PersonId, target: Option<PersonId>) -> Result<(), Cannot> {
        let Some(s) = self.held.get(&who).copied() else { return cannot("isn't holding a ritual") };
        if self.fighting.contains_key(&who) {
            return cannot("in a fight (it's released there)");
        }
        if !s.def().works_outside_fights() {
            return cannot("only of use in a fight");
        }
        let t = self.time;
        self.set_holding(who, t, None);
        let skill = magic::skill_power(&self.people[who as usize].effective_stats(), s);
        self.work_spell(who, s, target, skill);
        let name = self.name_of(who);
        self.say(t, format!("{name} releases the {}.", s.def().name.to_lowercase()));
        Ok(())
    }

    // ---- Learning ---------------------------------------------------------

    /// Learn a spell from notes (structured) or a rare text (ritual). The
    /// notes are kept. Returns what happened, for the log.
    pub(super) fn read_lore(&mut self, who: PersonId, it: ItemId) -> bool {
        let (key, style) = match item(it).kind {
            Kind::Notes(k) => (k, Style::Structured),
            Kind::Text(k) => (k, Style::Ritual),
            _ => return false,
        };
        let s = magic::spell(key);
        let t = self.time;
        let name = self.name_of(who);
        if s.def().style != style {
            return false;
        }
        if self.knows(who, s) {
            self.say(t, format!("{name} already knows {}.", s.def().name.to_lowercase()));
            return false;
        }
        let need = s.def().min_skill * READ_SKILL;
        let p = &mut self.people[who as usize];
        if p.effective_stats().skill(style.skill()) < need {
            self.say(t, format!("{name} can't make sense of it yet ({} {need:.0} needed).", style.skill().name()));
            return false;
        }
        p.detail.as_mut().unwrap().spells.push(s);
        p.recompute_might();
        self.say(t, format!("{name} learns {} from the {}.", s.def().name.to_lowercase(), item(it).name.to_lowercase()));
        true
    }

    /// What a teacher could teach someone: structured spells and rituals they
    /// know that the learner doesn't, and is skilled enough to take in.
    pub fn lessons(&self, teacher: PersonId, learner: PersonId) -> Vec<Spell> {
        let mine = self.known_spells(learner);
        let st = self.people[learner as usize].effective_stats();
        self.known_spells(teacher)
            .into_iter()
            .filter(|s| s.def().style != Style::Felt && !mine.contains(s))
            .filter(|s| st.skill(s.def().skill()) >= s.def().min_skill * TAUGHT_SKILL)
            .collect()
    }

    /// The price of a lesson, in coin.
    pub fn lesson_price(s: Spell) -> u16 {
        let d = s.def();
        let base = 30.0 + d.min_skill * 2.0;
        (if d.style == Style::Ritual { base * 2.0 } else { base }).round() as u16
    }

    /// Be taught a spell (the coin comes out of the squad's packs).
    pub(super) fn learn_from(&mut self, learner: PersonId, s: Spell) -> bool {
        let price = Self::lesson_price(s);
        if self.squad_count(items::id("coin")) < price || self.knows(learner, s) {
            return false;
        }
        self.take_from_squad(items::id("coin"), price);
        let p = &mut self.people[learner as usize];
        p.detail.as_mut().unwrap().spells.push(s);
        p.recompute_might();
        true
    }
}
