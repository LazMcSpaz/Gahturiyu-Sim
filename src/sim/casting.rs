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

/// What Transmute turns into what: (material, how many, becomes, how many).
/// Each lot is one row's worth, worked down the list.
pub const TRANSMUTE: &[(&str, u16, &str, u16)] = &[("iron_ore", 1, "iron_ingot", 1), ("hide", 1, "leather", 1), ("salt_crystal", 3, "storm_glass", 1), ("ash_moss", 2, "ghostcap", 1)];

/// Preserve keeps bodies from rotting for this many hours.
pub const PRESERVE_HOURS: f64 = 24.0;

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

/// A lasting effect on someone outside a fight: a blessing (or curse) with
/// an end time. Carried into fights as a status, and back out again.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Boon {
    pub pid: PersonId,
    pub does: Does,
    pub power: f32,
    pub until: f64,
}

/// A lasting spell on a patch of ground outside a fight (a veil, a ward, a
/// light, a guardian waiting).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Ward {
    pub does: Does,
    pub pos: V2,
    pub radius: f32,
    pub power: f32,
    pub until: f64,
    /// Who worked it.
    pub owner: PersonId,
}

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

/// A spell the player ordered that waits on the caster: to get in range,
/// or for the fight it opens to begin.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PendingCast {
    pub who: PersonId,
    pub spell: Spell,
    pub target: Option<PersonId>,
    pub point: Option<V2>,
    /// Read from a scroll rather than cast.
    #[serde(default)]
    pub scroll: bool,
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


    // ---- Lasting effects ----------------------------------------------------

    /// How strongly something works on someone right now (boons only).
    pub fn boon(&self, pid: PersonId, does: Does) -> f32 {
        self.boon_at(pid, does, self.time)
    }

    /// The same at time `t` (a boon counts while `t` is before its end).
    pub fn boon_at(&self, pid: PersonId, does: Does, t: f64) -> f32 {
        self.boons.iter().filter(|b| b.pid == pid && b.does == does && t < b.until).map(|b| b.power).sum()
    }

    /// What someone's kit weighs at time `t`, with spells that lighten or
    /// burden it.
    pub fn kit_weight_at(&self, pid: PersonId, t: f64) -> f32 {
        let w = self.people[pid as usize].kit().weight();
        w * (1.0 - self.boon_at(pid, Does::Lighten, t)).max(0.2) * (1.0 + self.boon_at(pid, Does::Burden, t))
    }

    /// How much someone can carry at time `t`, with spells that strengthen
    /// them or carry for them.
    pub fn capacity_at(&self, pid: PersonId, t: f64) -> f32 {
        let p = &self.people[pid as usize];
        p.kit().capacity(&p.stats) + self.boon_at(pid, Does::Carry, t) + self.boon_at(pid, Does::Attr(super::stats::Attr::Strength), t) * 0.8
    }

    /// Every boon on someone right now.
    pub fn boons_on(&self, pid: PersonId) -> Vec<Boon> {
        self.boons.iter().copied().filter(|b| b.pid == pid && self.time < b.until).collect()
    }

    /// Put a lasting effect on someone (replacing the same kind). Their
    /// condition starts a new piece here, so what it changes (load, hunger)
    /// counts from exactly now.
    pub(super) fn add_boon(&mut self, pid: PersonId, does: Does, power: f32, until: f64) {
        let t = self.time;
        self.boons.retain(|b| !(b.pid == pid && b.does == does));
        self.boons.push(Boon { pid, does, power, until });
        if self.people[pid as usize].cond.is_some() {
            self.settle_condition(pid, t);
        }
    }

    /// When the next of someone's boons runs out (for the condition timeline).
    pub(super) fn boon_ends(&self, pid: PersonId) -> Vec<f64> {
        self.boons.iter().filter(|b| b.pid == pid).map(|b| b.until).collect()
    }

    /// Forget boons and wards that are over (anything that cares has seen
    /// them end).
    pub(super) fn expire_boons(&mut self) {
        let t = self.time;
        self.boons.retain(|b| b.until > t);
        self.wards.retain(|w| w.until > t);
    }

    /// The wards of a kind covering a spot right now.
    pub fn wards_at(&self, does: Does, p: V2) -> impl Iterator<Item = &Ward> + '_ {
        let t = self.time;
        self.wards.iter().filter(move |w| w.does == does && t < w.until && w.pos.dist(p) <= w.radius)
    }

    // ---- The spell book -----------------------------------------------------

    /// Use a spell from the spell book: in a fight the caster casts it (or
    /// lets a held ritual go) there; otherwise a felt or structured spell is
    /// cast, a ritual is begun, or one held ready is released. `target` is
    /// who it's aimed at, `point` where.
    pub fn use_spell(&mut self, who: PersonId, s: Spell, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
        if self.fighting.contains_key(&who) {
            return self.cast_in_fight(who, s, target, point);
        }
        match s.def().style {
            Style::Ritual if self.held.get(&who) == Some(&s) => self.release(who, target, point),
            Style::Ritual => self.perform(who, s),
            _ => self.cast(who, s, target, point),
        }
    }

    /// The player's order to cast, as in Baldur's Gate 3: a harmful spell
    /// aimed at an enemy starts the fight and is cast as it opens; anything
    /// else out of reach has the caster walk into range first, then cast.
    pub fn order_cast(&mut self, who: PersonId, s: Spell, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
        self.casts.retain(|c| c.who != who);
        let d = s.def();
        if self.fighting.contains_key(&who) || d.style == Style::Ritual {
            return self.use_spell(who, s, target, point);
        }
        if !self.free_to_order(who) {
            return cannot("bound to work: can't leave it");
        }
        if !self.knows(who, s) {
            return cannot("doesn't know that spell");
        }
        if self.people[who as usize].mana_at(self.time) < d.cost {
            return cannot("not enough energy");
        }
        if d.aim == magic::Aim::Foe {
            let Some(t) = target else { return cannot("aim it at an enemy") };
            if !self.is_enemy(t) {
                return cannot("not an enemy");
            }
            if !self.attack(&[who], t) {
                return cannot("can't get at them");
            }
            self.casts.push(PendingCast { who, spell: s, target, point, scroll: false });
            return Ok(());
        }
        if !d.works_outside_fights() {
            // A harmful spell at enemies opens the fight with it.
            let at = point.or(target.map(|p| self.person_pos(p))).unwrap_or(self.person_pos(who));
            let Some(foe) = self.enemy_near(target, at, d.radius().max(3.0) + 3.0) else { return cannot("only of use in a fight: aim it at enemies") };
            if !self.attack(&[who], foe) {
                return cannot("can't get at them");
            }
            self.casts.push(PendingCast { who, spell: s, target: Some(foe), point: Some(at), scroll: false });
            return Ok(());
        }
        let at = point.or(target.map(|p| self.person_pos(p))).unwrap_or(self.person_pos(who));
        if self.person_pos(who).dist(at) <= d.range.max(2.0) {
            return self.cast(who, s, target, point);
        }
        // Too far: walk over and cast when in reach.
        self.order_members(&[who], at);
        self.casts.push(PendingCast { who, spell: s, target, point, scroll: false });
        Ok(())
    }

    /// A potion drunk or a scroll read in the middle of a fight, on the
    /// player's order: the fighter does it now (a scroll aimed at a foe goes
    /// at `target`, or the nearest enemy).
    pub fn use_in_fight(&mut self, who: PersonId, it: super::items::ItemId, target: Option<PersonId>) -> Result<String, String> {
        use super::items::{item, Kind};
        let Some(&id) = self.fighting.get(&who) else { return Err("Not in a fight.".into()) };
        let name = self.name_of(who);
        let Some(b) = self.battles.iter_mut().find(|b| b.id == id) else { return Err("Not in a fight.".into()) };
        let Some(i) = b.index_of(who) else { return Err("Not in a fight.".into()) };
        if !b.fighters[i].active() {
            return Err(format!("{name} is down."));
        }
        match item(it).kind {
            Kind::Potion => {
                if b.begin_drink(i, it) {
                    Ok(format!("{name} drinks the {}.", item(it).name.to_lowercase()))
                } else {
                    Err(format!("{name} has no {} to hand in this fight.", item(it).name.to_lowercase()))
                }
            }
            Kind::Scroll(key) => {
                let sp = magic::spell(key);
                let j = target.and_then(|p| b.index_of(p)).or_else(|| if sp.def().aim == magic::Aim::Foe || !sp.def().works_outside_fights() { b.nearest_enemy(i) } else { Some(i) });
                let point = j.map(|j| b.fighters[j].pos).unwrap_or(b.fighters[i].pos);
                if b.read_scroll(i, sp, j, point) {
                    Ok(format!("{name} reads the scroll of {}.", sp.def().name.to_lowercase()))
                } else {
                    Err(format!("{name} has no such scroll to hand in this fight."))
                }
            }
            _ => Err(format!("The {} can't be used in a fight.", item(it).name.to_lowercase())),
        }
    }

    /// The scroll of this spell in someone's pack, if they carry one.
    pub fn scroll_item(&self, who: PersonId, s: Spell) -> Option<super::items::ItemId> {
        use super::items::{item, Kind};
        let key = s.def().key;
        self.people[who as usize].detail.as_ref()?.gear.bag.iter().map(|e| e.0).find(|&it| matches!(item(it).kind, Kind::Scroll(x) if x == key))
    }

    /// Read a scroll of a harmful spell at enemies: it starts the fight and
    /// is read as it opens (in a fight already, it's read now).
    pub fn order_read(&mut self, who: PersonId, it: super::items::ItemId, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
        use super::items::{item, Kind};
        let Kind::Scroll(key) = item(it).kind else { return cannot("that's not a scroll") };
        let s = magic::spell(key);
        if self.fighting.contains_key(&who) {
            return self.use_in_fight(who, it, target).map(|_| ()).map_err(Cannot);
        }
        if s.def().works_outside_fights() {
            return if self.use_item(who, it) { Ok(()) } else { cannot("can't read it now") };
        }
        if !self.free_to_order(who) {
            return cannot("bound to work: can't leave it");
        }
        let at = point.or(target.map(|p| self.person_pos(p))).unwrap_or(self.person_pos(who));
        let Some(foe) = self.enemy_near(target, at, s.def().radius().max(3.0) + 3.0) else { return cannot("read it at enemies") };
        if !self.attack(&[who], foe) {
            return cannot("can't get at them");
        }
        self.casts.retain(|c| c.who != who);
        self.casts.push(PendingCast { who, spell: s, target: Some(foe), point: Some(at), scroll: true });
        Ok(())
    }

    /// The enemy a harmful spell aimed at a spot would start a fight with:
    /// the one aimed at, or the nearest enemy close to the spot.
    pub fn enemy_near(&self, target: Option<PersonId>, at: V2, within: f32) -> Option<PersonId> {
        if let Some(t) = target.filter(|&t| self.is_enemy(t)) {
            return Some(t);
        }
        self.groups
            .iter()
            .filter(|g| g.band <= 1 && g.hostile)
            .flat_map(|g| g.members.iter().copied())
            .filter(|&p| self.is_enemy(p) && !self.is_down(p))
            .map(|p| (p, self.person_pos(p).dist(at)))
            .filter(|(_, d)| *d <= within)
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
            .map(|(p, _)| p)
    }

    /// Someone the squad may fight: a bandit, or one of a hostile band.
    pub fn is_enemy(&self, p: PersonId) -> bool {
        let pp = &self.people[p as usize];
        !pp.in_squad && !pp.dead && (pp.bandit || self.group_of[p as usize].and_then(|g| self.group(g)).is_some_and(|g| g.hostile))
    }

    /// Casts waiting on the caster getting there, or on the fight opening
    /// (the squad's step).
    pub(super) fn do_casts(&mut self) {
        let mut k = 0;
        while k < self.casts.len() {
            let c = self.casts[k];
            let Some(i) = self.squad.index(c.who) else {
                self.casts.remove(k);
                continue;
            };
            if self.fighting.contains_key(&c.who) {
                self.casts.remove(k);
                if c.scroll {
                    let it = self.scroll_item(c.who, c.spell);
                    match it.map(|it| self.use_in_fight(c.who, it, c.target)) {
                        Some(Ok(_)) => {}
                        Some(Err(e)) => self.cast_failed(c.who, c.spell, &e),
                        None => self.cast_failed(c.who, c.spell, "the scroll is gone"),
                    }
                } else if let Err(e) = self.cast_in_fight(c.who, c.spell, c.target, c.point) {
                    self.cast_failed(c.who, c.spell, &e.0);
                }
                continue;
            }
            if c.spell.def().aim == magic::Aim::Foe || !c.spell.def().works_outside_fights() {
                // Waiting for the fight to open; it never did.
                if self.squad_battle().is_none() && self.squad.at[i].dist(self.squad.goal[i]) < 0.5 {
                    self.casts.remove(k);
                    continue;
                }
                k += 1;
                continue;
            }
            let at = c.point.or(c.target.map(|p| self.person_pos(p))).unwrap_or(self.squad.at[i]);
            if self.squad.at[i].dist(at) <= c.spell.def().range.max(2.0) {
                self.casts.remove(k);
                self.squad.goal[i] = self.squad.at[i];
                self.squad.route[i].clear();
                if let Err(e) = self.cast(c.who, c.spell, c.target, c.point) {
                    self.cast_failed(c.who, c.spell, &e.0);
                }
                continue;
            }
            // A moving target: follow it.
            if self.squad.goal[i].dist(at) > 3.0 {
                let (path, _) = self.route(self.member_pos(i), at);
                self.squad.goal[i] = *path.last().unwrap_or(&at);
                self.squad.route[i] = path;
            }
            k += 1;
        }
    }

    fn cast_failed(&mut self, who: PersonId, s: Spell, why: &str) {
        let name = self.people[who as usize].name().unwrap_or("someone").to_string();
        let line = format!("{name} can't cast {}: {why}.", s.def().name.to_lowercase());
        self.log.push_front((self.time, line));
        self.log.truncate(14);
    }

    /// A squad member in a fight casts a spell (or releases the ritual they
    /// hold) on the player's order.
    pub fn cast_in_fight(&mut self, who: PersonId, s: Spell, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
        let Some(&id) = self.fighting.get(&who) else { return cannot("not in a fight") };
        let Some(b) = self.battles.iter_mut().find(|b| b.id == id) else { return cannot("not in a fight") };
        let Some(i) = b.index_of(who) else { return cannot("not in a fight") };
        if !b.fighters[i].active() {
            return cannot("down");
        }
        let j = target.and_then(|p| b.index_of(p));
        let point = point.unwrap_or(j.map(|j| b.fighters[j].pos).unwrap_or(b.fighters[i].pos));
        if b.fighters[i].held == Some(s) {
            b.release(i, j, point);
            return Ok(());
        }
        if s.def().style == Style::Ritual {
            return cannot("a ritual can't be performed mid-fight");
        }
        if b.fighters[i].mana < s.def().cost {
            return cannot("not enough energy");
        }
        if !b.begin_cast(i, s, j, point) {
            return cannot("can't cast that now");
        }
        Ok(())
    }

    // ---- Casting outside a fight ------------------------------------------

    /// Cast a felt or structured spell outside a fight (from the spell book).
    /// `target` is who it's aimed at (a squad member), if anyone; `point`
    /// where, for spells aimed at a spot. Energy is spent whether or not it
    /// works.
    pub fn cast(&mut self, who: PersonId, s: Spell, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
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
        let from = self.person_pos(who);
        let point = point.unwrap_or(target.map(|p| self.person_pos(p)).unwrap_or(from));
        if from.dist(point) > d.range.max(2.0) + 1.0 {
            return cannot("too far away");
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
            self.say(t, format!("{name}'s {} fizzles ({:.0}% chance at their skill; the energy is spent).", d.name.to_lowercase(), chance * 100.0));
            return Ok(());
        }
        self.people[who as usize].stats.exercise(d.skill(), 1.5);
        self.learn_felt(who, t);
        let skill = magic::skill_power(&self.people[who as usize].effective_stats(), s) * self.domain_boost(who, s);
        self.say(t, format!("{name} casts {}.", d.name.to_lowercase()));
        self.work_spell(who, s, target, point, skill);
        Ok(())
    }

    /// How much stronger someone's spells of this one's domain are (worn
    /// effects and blessings that boost a domain).
    pub fn domain_boost(&self, pid: PersonId, s: Spell) -> f32 {
        let d = Does::DomainPower(s.def().domain);
        1.0 + self.people[pid as usize].kit().worn(d) + self.boon(pid, d)
    }

    /// A squad member's feel for felt magic has grown: the felt spells it now
    /// reaches come to them. (Squad only; see `fights::write_back`.)
    pub(super) fn learn_felt(&mut self, pid: PersonId, t: f64) {
        let p = &mut self.people[pid as usize];
        if !p.in_squad {
            return;
        }
        let Some(d) = p.detail.as_mut() else { return };
        let new = magic::felt_reached(&p.stats, &d.spells);
        if new.is_empty() {
            return;
        }
        d.spells.extend(new.iter().copied());
        let name = d.name.clone();
        for sp in new {
            self.say(t, format!("{name} has a feel for {} now.", sp.def().name.to_lowercase()));
        }
        self.people[pid as usize].recompute_might();
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
    fn work_spell(&mut self, by: PersonId, s: Spell, target: Option<PersonId>, point: V2, skill: f32) {
        let point = if s.def().aim == magic::Aim::Caster { self.person_pos(by) } else { point };
        for e in s.def().effects {
            if e.reach == Reach::Object {
                self.affect_object(by, e, point);
                continue;
            }
            if let (Reach::Ground { radius }, Lasts::Secs(secs)) = (e.reach, e.lasts) {
                let t = self.time;
                self.wards.push(Ward { does: e.does, pos: point, radius, power: e.power, until: t + secs as f64, owner: by });
                continue;
            }
            for pid in self.reached(by, e, target, point) {
                self.apply_effect(pid, e, skill);
            }
        }
    }

    /// An effect on a thing near `point`: a torch, a campfire.
    fn affect_object(&mut self, by: PersonId, e: &Effect, point: V2) -> bool {
        let t = self.time;
        const NEAR: f32 = 6.0;
        match e.does {
            Does::Preserve => {
                let mut any = false;
                for c in self.corpses.iter_mut().filter(|c| c.0.dist(point) <= 4.0) {
                    c.2 = c.2.max(t + PRESERVE_HOURS * super::world::HOUR - super::fights::CORPSE_TIME);
                    any = true;
                }
                if any {
                    self.say(t, "The bodies are kept from rotting.".to_string());
                }
                any
            }
            Does::Unlock => {
                // The nearest lock to the spot: a door's, or a chest's or cupboard's.
                let door = self.doors_near(point, 3.0).into_iter().filter(|d| d.lock > 0.0 && self.is_locked(d.id)).map(|d| (d.outside.dist(point), d.id)).min_by(|a, b| a.0.total_cmp(&b.0));
                let chest = self.containers.values().filter(|c| c.lock > 0.0 && !c.picked && c.pos.dist(point) <= 3.0).map(|c| (c.pos.dist(point), c.id)).min_by(|a, b| a.0.total_cmp(&b.0));
                match (door, chest) {
                    (_, Some((dc, c))) if door.map(|d| dc <= d.0).unwrap_or(true) => {
                        if let Some(c) = self.containers.get_mut(&c) {
                            c.picked = true;
                        }
                        self.say(t, format!("The {}'s lock clicks open.", self.containers[&c].what.name()));
                        true
                    }
                    (Some((_, id)), _) => {
                        self.picked.insert(id, super::buildings::night_of(t));
                        self.say(t, "A lock clicks open.".to_string());
                        true
                    }
                    _ => {
                        self.say(t, "Unlock finds no lock there to open.".to_string());
                        false
                    }
                }
            }
            Does::Transmute => {
                let mut lots = e.power.round() as u32;
                let mut made = Vec::new();
                let Some(dd) = self.people[by as usize].detail.as_mut() else { return false };
                for &(from, n, to, m) in TRANSMUTE {
                    let (fi, ti) = (items::id(from), items::id(to));
                    while lots > 0 && dd.gear.bag.iter().filter(|x| x.0 == fi).map(|x| x.1).sum::<u16>() >= n {
                        for _ in 0..n {
                            dd.gear.take(fi);
                        }
                        dd.gear.add(ti, m);
                        lots -= 1;
                        made.push(item(ti).name.to_lowercase());
                    }
                }
                self.people[by as usize].recompute_might();
                if made.is_empty() {
                    return false;
                }
                made.dedup();
                let name = self.name_of(by);
                self.say(t, format!("Under {name}'s hands the materials change: {}.", made.join(", ")));
                true
            }
            Does::Douse => {
                // The nearest flame: a torch in a squad member's hand, a
                // standing torch, a campfire.
                let mut best: Option<(f32, u8, usize)> = None;
                let mut offer = |d: f32, kind: u8, i: usize| {
                    if d <= NEAR && best.map(|b| d < b.0).unwrap_or(true) {
                        best = Some((d, kind, i));
                    }
                };
                for (k, &m) in self.squad.members.iter().enumerate() {
                    if self.torch_lit(m) {
                        offer(self.member_pos(k).dist(point), 0, k);
                    }
                }
                for (i, st) in self.standing.iter().enumerate() {
                    if st.burning(t) {
                        offer(st.pos.dist(point), 1, i);
                    }
                }
                for (i, c) in self.camps.iter().enumerate() {
                    if t >= c.doused_until {
                        offer(c.pos.dist(point), 2, i);
                    }
                }
                match best {
                    Some((_, 0, k)) => self.toggle_torch(self.squad.members[k]),
                    Some((_, 1, i)) => {
                        self.standing[i].out_at = t;
                        self.say(t, "A standing torch hisses out.".to_string());
                        true
                    }
                    Some((_, _, i)) => {
                        self.camps[i].doused_until = t + super::torch::DOUSE_HOURS * super::world::HOUR;
                        self.say(t, "A campfire hisses out.".to_string());
                        true
                    }
                    None => false,
                }
            }
            Does::Kindle => {
                let member = self.squad.members.iter().enumerate().filter(|(k, &m)| !self.torch_lit(m) && self.member_pos(*k).dist(point) <= NEAR).min_by(|a, b| self.member_pos(a.0).dist(point).total_cmp(&self.member_pos(b.0).dist(point))).map(|(_, &m)| m);
                if let Some(m) = member {
                    return self.toggle_torch(m);
                }
                if let Some(i) = self.camps.iter().position(|c| t < c.doused_until && c.pos.dist(point) <= NEAR) {
                    self.camps[i].doused_until = t;
                    self.say(t, "A campfire roars back to life.".to_string());
                    return true;
                }
                false
            }
            _ => false,
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
        match e.lasts {
            Lasts::Now => {}
            Lasts::Secs(s) if e.does.works_outside_fights() => {
                self.add_boon(target, e.does, e.power, t + s as f64);
                return true;
            }
            _ => return false,
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
            Does::Dispel => {
                let had = self.boons.iter().any(|b| b.pid == target && b.until > t);
                self.boons.retain(|b| b.pid != target);
                if self.people[target as usize].cond.is_some() {
                    self.settle_condition(target, t);
                }
                had
            }
            Does::Stamina => {
                if self.people[target as usize].cond.is_none() {
                    return false;
                }
                self.settle_condition(target, t);
                if let Some(c) = self.people[target as usize].cond.as_mut() {
                    c.stamina = (c.stamina + e.power * skill).min(c.max_stamina);
                }
                true
            }
            Does::Regrow => {
                let p = &mut self.people[target as usize];
                let Some(k) = (0..6).find(|&k| p.wounds.missing[k]) else { return false };
                if p.cond.is_some() {
                    self.settle_condition(target, t);
                }
                let p = &mut self.people[target as usize];
                let base = p.stats.clone();
                let mut hp = p.wounds.hp_at(&base, t);
                p.wounds.missing[k] = false;
                hp[k] = 1.0;
                p.wounds.set(&base, &hp, t);
                p.recompute_might();
                if p.cond.is_some() {
                    self.settle_condition(target, t);
                }
                let name = self.name_of(target);
                self.say(t, format!("{name}'s {} grows back.", super::body::PARTS[k].name()));
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
    pub fn set_holding(&mut self, pid: PersonId, t: f64, s: Option<Spell>) {
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
    pub fn release(&mut self, who: PersonId, target: Option<PersonId>, point: Option<V2>) -> Result<(), Cannot> {
        let Some(s) = self.held.get(&who).copied() else { return cannot("isn't holding a ritual") };
        if self.fighting.contains_key(&who) {
            return cannot("in a fight (it's released there)");
        }
        if !s.def().works_outside_fights() {
            return cannot("only of use in a fight");
        }
        let t = self.time;
        if let Some(p) = target {
            if s.def().aim == magic::Aim::Friend && self.person_pos(p).dist(self.person_pos(who)) > s.def().range + 1.0 {
                return cannot("too far away");
            }
        }
        self.set_holding(who, t, None);
        let skill = magic::skill_power(&self.people[who as usize].effective_stats(), s) * self.domain_boost(who, s);
        let name = self.name_of(who);
        self.say(t, format!("{name} releases the {}.", s.def().name.to_lowercase()));
        let point = point.unwrap_or(target.map(|p| self.person_pos(p)).unwrap_or(self.person_pos(who)));
        self.work_spell(who, s, target, point, skill);
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
        let mut v: Vec<Spell> = self
            .known_spells(teacher)
            .into_iter()
            .filter(|s| s.def().style != Style::Felt && !mine.contains(s))
            .filter(|s| st.skill(s.def().skill()) >= s.def().min_skill * TAUGHT_SKILL)
            .collect();
        // The most advanced first: that's what's worth paying for.
        v.sort_by(|a, b| b.def().min_skill.total_cmp(&a.def().min_skill).then(a.cmp(b)));
        v
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
