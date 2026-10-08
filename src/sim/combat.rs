//! Fights, up close.
//!
//! A battle is a little simulation of its own: everyone in it has a position,
//! a body, fatigue, mana and whatever they're doing this instant (swinging,
//! recovering, casting). It runs in fixed ticks of `DT` game seconds, so it
//! comes out identical however coarsely the world around it is stepped, and
//! every random draw is keyed by the battle and the tick.
//!
//! A blow goes: can I reach? → does it land (attack skill vs dodge)? → is it
//! blocked (shield or parry)? → where does it land (body part)? → how much
//! gets through the armour there? Damage is cut and blunt, Kenshi-style;
//! hit chance and skills are Morrowind-style. Everyone learns from what they
//! do: swinging trains the weapon skill, dodging trains Dodge, being hit
//! toughens you.

use super::body::{self, Part, PARTS};
use super::geo::V2;
use super::inventory::{self, Gear};
use super::items::{item, ArmorDef, Effect, ItemId, Kind, WeaponDef, FISTS};
use super::magic::{self, Spell, Status, StatusKind, Target};
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng::Rng;
use super::stats::{Attr, Skill, Stats};

/// Length of one combat tick, game seconds.
pub const DT: f64 = 0.1;
/// Damage multiplier for hitting someone who hasn't noticed you (before the
/// attacker's Sneak adds to it).
pub const SNEAK_ATTACK: f32 = 2.0;
/// A fight that drags on longer than this simply ends (everyone disengages).
pub const MAX_LENGTH: f64 = 900.0;
/// Body radius, metres: added to weapon reach.
const BODY: f32 = 0.45;

pub type Side = u8;
pub const SQUAD_SIDE: Side = 0;

#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Idle,
    Swing { target: usize, lands: f64 },
    Recover { until: f64 },
    /// `scroll`: read from a scroll — no mana, can't fail.
    Cast { spell: Spell, target: Option<usize>, point: V2, done: f64, scroll: bool },
    /// Drinking a potion.
    Drink { item: ItemId, done: f64 },
}

/// What the player (or a leader) has told someone to do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Order {
    /// Go here and don't stop to fight on the way.
    MoveTo(V2),
    /// Go for this one.
    Attack(usize),
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub pid: PersonId,
    pub side: Side,
    pub race: Race,
    pub pos: V2,
    pub hp: [f32; 6],
    pub max_hp: [f32; 6],
    pub fatigue: f32,
    pub max_fatigue: f32,
    pub mana: f32,
    pub max_mana: f32,
    /// Mana per second.
    pub mana_regen: f32,
    /// With gear enchantments applied.
    pub stats: Stats,
    pub weapon: WeaponDef,
    pub weapon_name: &'static str,
    pub shield: f32,
    pub armor: Vec<ArmorDef>,
    pub dodge_penalty: f32,
    pub resist_paralysis: f32,
    pub resist_blind: f32,
    pub resist_elements: f32,
    /// Running speed with no injuries or spells, m/s.
    pub base_speed: f32,
    pub spells: Vec<Spell>,
    pub boldness: f32,
    pub might: f32,

    pub act: Act,
    pub target: Option<usize>,
    pub order: Option<Order>,
    pub statuses: Vec<Status>,
    pub ko: bool,
    pub dead: bool,
    pub fleeing: bool,
    pub fled: bool,
    pub think_at: f64,
    /// Potions and scrolls in their pack, and those used up this fight.
    pub potions: Vec<ItemId>,
    pub scrolls: Vec<ItemId>,
    pub used: Vec<ItemId>,
    /// Until this time they haven't realised they're under attack: they
    /// stand there, can't dodge or block, and take a sneak attack's damage.
    pub aware_at: f64,

    /// Skill uses this fight, written back to the person afterwards.
    pub trained: [f32; super::stats::N_SKILLS],
    pub damage_taken: f32,
}

impl Fighter {
    /// Bring a person into a fight at `pos`. Someone never met fights with
    /// exactly the kit and spells they'd have if met: the same ones their
    /// might was rated on.
    pub fn from_person(p: &Person, side: Side, pos: V2, t: f64) -> Fighter {
        let kit = p.kit();
        let gear = &kit;
        let spells = match &p.detail {
            Some(d) => d.spells.clone(),
            None => super::magic::starting_spells(&p.stats),
        };
        let stats = inventory::effective(&p.stats, gear);
        let mut max_hp = [0.0; 6];
        for (i, part) in PARTS.iter().enumerate() {
            max_hp[i] = p.stats.max_hp(*part);
        }
        let armor: Vec<ArmorDef> = gear.equipped().filter_map(|id| item(id).armor().copied()).collect();
        let dodge_penalty = armor.iter().map(|a| a.dodge_penalty).sum();
        let sum = |f: &dyn Fn(&Effect) -> Option<f32>| gear.sum_effect(f);
        let speed_bonus = sum(&|e| if let Effect::MoveSpeed(v) = e { Some(*v) } else { None });
        let load = gear.load(&p.stats);
        Fighter {
            pid: p.id,
            side,
            race: p.race,
            pos,
            hp: p.wounds.hp_at(&p.stats, t),
            max_hp,
            fatigue: stats.max_fatigue(),
            max_fatigue: stats.max_fatigue(),
            mana: p.mana_at(t),
            max_mana: p.max_mana(),
            mana_regen: p.mana_regen() / 60.0,
            weapon: gear.weapon(),
            weapon_name: gear.weapon_name(),
            shield: gear.shield(),
            armor,
            dodge_penalty,
            resist_paralysis: sum(&|e| if let Effect::ResistParalysis(v) = e { Some(*v) } else { None }).min(0.95),
            resist_blind: sum(&|e| if let Effect::ResistBlind(v) = e { Some(*v) } else { None }).min(0.95),
            resist_elements: sum(&|e| if let Effect::ResistElements(v) = e { Some(*v) } else { None }).min(0.9),
            base_speed: p.race.walk_speed() * 2.6 * stats.move_factor() * (1.0 + speed_bonus) * inventory::encumbrance_factor(load),
            spells,
            boldness: p.traits.boldness,
            might: p.might,
            stats,
            act: Act::Idle,
            target: None,
            order: None,
            statuses: Vec::new(),
            ko: false,
            dead: false,
            fleeing: false,
            fled: false,
            think_at: 0.0,
            aware_at: 0.0,
            potions: gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Potion(_))).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            scrolls: gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Scroll(_))).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            used: Vec::new(),
            trained: [0.0; super::stats::N_SKILLS],
            damage_taken: 0.0,
        }
    }

    /// Caught unawares at time `t`.
    pub fn unaware(&self, t: f64) -> bool {
        t < self.aware_at
    }

    /// Can act at all: not down, dead or gone.
    pub fn active(&self) -> bool {
        !self.ko && !self.dead && !self.fled
    }

    pub fn has(&self, k: StatusKind) -> Option<&Status> {
        self.statuses.iter().find(|s| s.kind == k)
    }

    pub fn paralyzed(&self) -> bool {
        self.has(StatusKind::Paralyzed).is_some()
    }

    /// 1 when fresh, down to 0.6 when spent.
    pub fn tired(&self) -> f32 {
        0.6 + 0.4 * (self.fatigue / self.max_fatigue.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn haste(&self) -> f32 {
        self.has(StatusKind::Hasted).map(|s| s.magnitude).unwrap_or(0.0)
    }

    pub fn speed(&self) -> f32 {
        if self.paralyzed() || matches!(self.act, Act::Cast { .. }) {
            return 0.0;
        }
        self.base_speed * body::leg_factor(&self.hp) * (1.0 + self.haste()) * if self.fleeing { 1.1 } else { 1.0 }
    }

    /// Multiplier on swing and recovery times.
    pub fn attack_time(&self) -> f32 {
        ((1.25 - self.stats.attr(Attr::Agility) * 0.005) * (1.0 - self.haste() * 0.6)).clamp(0.5, 1.3)
    }

    /// The weapon actually usable: a ruined sword arm means fists from the
    /// other hand; both arms ruined means no attack at all.
    pub fn usable_weapon(&self) -> Option<(WeaponDef, f32)> {
        let (l, r) = (self.hp[Part::LeftArm as usize] > 0.0, self.hp[Part::RightArm as usize] > 0.0);
        match (r, l) {
            (true, _) => Some((self.weapon, 1.0)),
            (false, true) => Some((FISTS, 0.6)),
            _ => None,
        }
    }

    pub fn reach(&self) -> f32 {
        self.usable_weapon().map(|w| w.0.reach).unwrap_or(0.8) + BODY * 2.0
    }

    /// Share of vital health left, 0..1.
    pub fn vitality(&self) -> f32 {
        let h = self.hp[Part::Head as usize].max(0.0) / self.max_hp[0];
        let t = self.hp[Part::Torso as usize].max(0.0) / self.max_hp[1];
        h.min(t)
    }

    fn train(&mut self, s: Skill, amount: f32) {
        self.trained[s as usize] += amount;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FxKind {
    Fireball { at: V2, radius: f32 },
    Bolt { from: V2, to: V2 },
    Fizzle { at: V2 },
}

/// A visual moment for the window to show.
#[derive(Clone, Debug, PartialEq)]
pub struct Fx {
    pub kind: FxKind,
    pub at: f64,
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub id: u32,
    pub seed: u64,
    pub start: f64,
    /// Battle clock; always a whole number of ticks past `start`.
    pub time: f64,
    pub ticks: u64,
    pub fighters: Vec<Fighter>,
    pub over: bool,
    pub log: Vec<(f64, String)>,
    pub fx: Vec<Fx>,
    pub names: Vec<String>,
}

impl Battle {
    pub fn new(id: u32, seed: u64, start: f64, fighters: Vec<Fighter>, names: Vec<String>) -> Battle {
        Battle { id, seed, start, time: start, ticks: 0, fighters, over: false, log: Vec::new(), fx: Vec::new(), names }
    }

    pub fn index_of(&self, pid: PersonId) -> Option<usize> {
        self.fighters.iter().position(|f| f.pid == pid)
    }

    fn say(&mut self, line: String) {
        self.log.push((self.time, line));
        if self.log.len() > 400 {
            self.log.remove(0);
        }
    }

    pub fn hostile(&self, a: usize, b: usize) -> bool {
        self.fighters[a].side != self.fighters[b].side
    }

    /// Run whole ticks up to time `t`.
    pub fn advance_to(&mut self, t: f64) {
        while !self.over && self.time + DT <= t + 1e-9 {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
        self.time = self.start + self.ticks as f64 * DT;
        let t = self.time;
        let mut rng = Rng::from_keys(&[self.seed, self.ticks, 0x4649_4748]);
        let n = self.fighters.len();

        // Spells wear off; breath and mana come back.
        for f in &mut self.fighters {
            f.statuses.retain(|s| s.until > t);
            if f.active() {
                let resting = matches!(f.act, Act::Idle | Act::Recover { .. });
                f.fatigue = (f.fatigue + if resting { 2.0 } else { 0.5 } * DT as f32).min(f.max_fatigue);
                f.mana = (f.mana + f.mana_regen * DT as f32).min(f.max_mana);
            }
        }

        // Decide.
        for i in 0..n {
            if self.fighters[i].active() && t >= self.fighters[i].think_at && !self.fighters[i].unaware(t) {
                super::ai::think(self, i, &mut rng);
                self.fighters[i].think_at = t + 0.3;
            }
        }

        // Act.
        for i in 0..n {
            if !self.fighters[i].active() {
                continue;
            }
            if self.fighters[i].paralyzed() {
                self.fighters[i].act = Act::Idle;
                continue;
            }
            match self.fighters[i].act.clone() {
                Act::Swing { target, lands } => {
                    if t >= lands {
                        self.resolve_attack(i, target, &mut rng);
                        let rec = self.fighters[i].weapon.recover * self.fighters[i].attack_time();
                        self.fighters[i].act = Act::Recover { until: t + rec as f64 };
                    }
                }
                Act::Recover { until } => {
                    if t >= until {
                        self.fighters[i].act = Act::Idle;
                    }
                }
                Act::Cast { spell, target, point, done, scroll } => {
                    if t >= done {
                        self.resolve_cast(i, spell, target, point, scroll, &mut rng);
                        self.fighters[i].act = Act::Recover { until: t + 0.3 };
                    }
                }
                Act::Drink { item: it, done } => {
                    if t >= done {
                        self.drink(i, it);
                        self.fighters[i].act = Act::Idle;
                    }
                }
                Act::Idle => self.idle(i),
            }
        }

        self.separate();

        // Over when no two sides still standing are enemies, or it's dragged on.
        // People running away don't keep a fight going.
        let mut sides: Vec<Side> = self.fighters.iter().filter(|f| f.active() && !f.fleeing).map(|f| f.side).collect();
        sides.sort();
        sides.dedup();
        if sides.len() <= 1 || t - self.start > MAX_LENGTH {
            for f in self.fighters.iter_mut().filter(|f| f.fleeing && f.active()) {
                f.fled = true;
            }
            self.over = true;
            let line = match sides.first() {
                Some(s) if sides.len() == 1 => format!("The fight is over. {} side holds the field.", if *s == SQUAD_SIDE { "Your" } else { "The other" }),
                _ => "The fight breaks off.".to_string(),
            };
            self.say(line);
        }
    }

    /// Doing nothing in particular: carry out orders, chase the target, or run.
    fn idle(&mut self, i: usize) {
        let me = &self.fighters[i];
        let speed = me.speed() * DT as f32;
        if me.fleeing {
            let threat = self.nearest_enemy(i);
            let away = match threat {
                Some(j) => {
                    let d = me.pos.sub(self.fighters[j].pos);
                    if d.len() > 45.0 {
                        let name = self.names[i].clone();
                        self.fighters[i].fled = true;
                        self.say(format!("{name} gets away."));
                        return;
                    }
                    d.scale(1.0 / d.len().max(0.01))
                }
                None => V2::new(1.0, 0.0),
            };
            self.fighters[i].pos = me.pos.add(away.scale(speed));
            return;
        }
        if let Some(Order::MoveTo(p)) = me.order {
            let d = p.sub(me.pos);
            if d.len() < 0.6 {
                self.fighters[i].order = None;
            } else {
                self.fighters[i].pos = me.pos.add(d.scale(speed.min(d.len()) / d.len()));
            }
            return;
        }
        let Some(j) = me.target else { return };
        let them = &self.fighters[j];
        let d = them.pos.sub(me.pos);
        let dist = d.len();
        let reach = me.reach();
        if dist > reach * 0.92 {
            let step = speed.min(dist - reach * 0.85).max(0.0);
            self.fighters[i].pos = me.pos.add(d.scale(step / dist.max(0.01)));
        } else if me.usable_weapon().is_some() && me.fatigue > 4.0 {
            let windup = me.weapon.windup * me.attack_time();
            self.fighters[i].act = Act::Swing { target: j, lands: self.time + windup as f64 };
        }
    }

    pub fn nearest_enemy(&self, i: usize) -> Option<usize> {
        let me = &self.fighters[i];
        (0..self.fighters.len())
            .filter(|&j| self.hostile(i, j) && self.fighters[j].active())
            .min_by(|&a, &b| me.pos.dist(self.fighters[a].pos).total_cmp(&me.pos.dist(self.fighters[b].pos)))
    }

    /// Keep bodies from piling onto one spot.
    fn separate(&mut self) {
        let n = self.fighters.len();
        for i in 0..n {
            if !self.fighters[i].active() {
                continue;
            }
            for j in i + 1..n {
                if !self.fighters[j].active() {
                    continue;
                }
                let d = self.fighters[j].pos.sub(self.fighters[i].pos);
                let l = d.len();
                let min = BODY * 2.0;
                if l < min {
                    let push = if l < 1e-4 { V2::new(0.05 * (j as f32 - i as f32), 0.03) } else { d.scale((min - l) * 0.5 / l) };
                    self.fighters[i].pos = self.fighters[i].pos.sub(push);
                    self.fighters[j].pos = self.fighters[j].pos.add(push);
                }
            }
        }
    }

    // ---- Blows ----------------------------------------------------------

    fn resolve_attack(&mut self, a: usize, d: usize, rng: &mut Rng) {
        // Every draw happens whatever the outcome, so one swing missing never
        // shifts the dice for everyone after it.
        let (r_hit, r_block, r_part, r_dmg, r_cover1, r_cover2) = (rng.f32(), rng.f32(), rng.f32(), rng.f32(), rng.f32(), rng.f32());
        let att = &self.fighters[a];
        let def = &self.fighters[d];
        let Some((weapon, arm)) = att.usable_weapon() else { return };
        if !def.active() || att.pos.dist(def.pos) > att.reach() * 1.3 {
            return;
        }
        // Blindness: a swing in roughly the right direction, mostly missing.
        let blinded = att.has(StatusKind::Blinded).map(|s| s.magnitude).unwrap_or(0.0);
        let blind = 1.0 - blinded * 0.3;
        let skill = att.stats.skill(weapon.skill);
        let atk = (skill + att.stats.attr(Attr::Agility) * 0.25 + 10.0) * att.tired() * arm * blind;
        let unaware = def.unaware(self.time);
        let helpless = def.paralyzed() || unaware;
        let dodge = if helpless {
            -40.0
        } else {
            (def.stats.skill(Skill::Dodge) * 0.6 + def.stats.attr(Attr::Agility) * 0.25 - def.dodge_penalty * 100.0) * def.tired()
        };
        let p_hit = (0.5 + (atk - dodge) * 0.012).clamp(0.08, 0.95) * (1.0 - blinded * 0.7);
        let (an, dn) = (self.names[a].clone(), self.names[d].clone());
        let strength = att.stats.attr(Attr::Strength);
        self.fighters[a].fatigue -= 3.0 + weapon.windup * 4.0;

        if r_hit > p_hit {
            self.fighters[d].train(Skill::Dodge, 1.0);
            self.fighters[a].train(weapon.skill, 0.3);
            if rng_line(r_dmg) {
                self.say(format!("{dn} dodges {an}."));
            }
            return;
        }
        let def = &self.fighters[d];
        let guard = if helpless { 0.0 } else { def.shield * 100.0 + def.weapon.parry * 60.0 };
        let p_block = if guard <= 0.0 { 0.0 } else { (guard * (0.3 + def.stats.skill(Skill::Block) / 100.0) / (guard + atk + 20.0)).clamp(0.0, 0.75) };
        let roll = 0.8 + r_dmg * 0.4;
        let skill_mult = 0.6 + skill * 0.006;
        let mut cut = weapon.cut * (1.0 + strength * 0.006) * skill_mult * roll;
        let mut blunt = weapon.blunt * (1.0 + strength * 0.010) * skill_mult * roll;

        if r_block < p_block {
            self.fighters[d].train(Skill::Block, 1.0);
            self.fighters[d].fatigue -= (cut + blunt) * 0.4;
            self.fighters[a].train(weapon.skill, 0.5);
            // A block still jars the arm a little.
            self.wound(d, Part::LeftArm, blunt * 0.15);
            if rng_line(r_dmg) {
                self.say(format!("{dn} blocks {an}'s {}.", self.fighters[a].weapon_name.to_lowercase()));
            }
            return;
        }

        // Where it lands.
        let mut acc = 0.0;
        let mut part = Part::Torso;
        for p in PARTS {
            acc += p.hit_weight();
            if r_part < acc {
                part = p;
                break;
            }
        }
        // Armour on that part: each layer that covers it may catch the blow.
        let mut layers: Vec<&ArmorDef> = self.fighters[d].armor.iter().filter(|a| a.covers.contains(&part)).collect();
        layers.sort_by(|x, y| y.cut.total_cmp(&x.cut));
        for (k, layer) in layers.iter().enumerate() {
            let r = if k == 0 { r_cover1 } else { r_cover2 };
            if r < layer.coverage {
                cut *= 1.0 - layer.cut;
                blunt *= 1.0 - layer.blunt;
            }
        }
        let shield = self.fighters[d].has(StatusKind::MageArmor).map(|s| 1.0 - s.magnitude).unwrap_or(1.0);
        let mut dmg = (cut + blunt) * shield;
        self.fighters[a].train(weapon.skill, 1.0);
        if unaware {
            // A sneak attack: the better the sneak, the worse the wound.
            let sneak = self.fighters[a].stats.skill(Skill::Sneak);
            dmg *= SNEAK_ATTACK + sneak / 50.0;
            self.fighters[a].train(Skill::Sneak, 3.0);
            self.say(format!("{an} catches {dn} unawares: the {} ({:.0}).", part.name(), dmg));
        } else {
            self.say(format!("{an} hits {dn} in the {} ({:.0}).", part.name(), dmg));
        }
        self.wound(d, part, dmg);
    }

    /// Everyone on a side realises they're under attack.
    pub fn wake(&mut self, side: Side) {
        let t = self.time;
        for f in self.fighters.iter_mut().filter(|f| f.side == side && f.aware_at > t) {
            f.aware_at = t;
        }
    }

    /// Apply damage to a body part and see what it does.
    fn wound(&mut self, d: usize, part: Part, dmg: f32) {
        if dmg <= 0.0 {
            return;
        }
        self.wake(self.fighters[d].side);
        let f = &mut self.fighters[d];
        let was_ko = f.ko;
        f.hp[part as usize] -= dmg;
        f.damage_taken += dmg;
        // A hit spoils whatever spell was being cast.
        if matches!(f.act, Act::Cast { .. }) && dmg > 6.0 {
            f.act = Act::Recover { until: self.time + 0.4 };
        }
        let name = self.names[d].clone();
        let stats = self.fighters[d].stats.clone();
        if body::dead(&self.fighters[d].hp, &stats) && !self.fighters[d].dead {
            self.fighters[d].dead = true;
            self.fighters[d].ko = true;
            self.say(format!("{name} is killed."));
        } else if body::knocked_out(&self.fighters[d].hp) && !was_ko {
            self.fighters[d].ko = true;
            self.fighters[d].act = Act::Idle;
            self.say(format!("{name} goes down."));
        } else if self.fighters[d].hp[part as usize] <= 0.0 && self.fighters[d].hp[part as usize] + dmg > 0.0 && !part.vital() {
            self.say(format!("{name}'s {} is ruined.", part.name()));
        }
    }

    // ---- Spells -----------------------------------------------------------

    /// Start casting. Mana is spent now, whether or not it works.
    pub fn begin_cast(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2) -> bool {
        let d = spell.def();
        let f = &mut self.fighters[i];
        if f.mana < d.cost || !f.spells.contains(&spell) {
            return false;
        }
        f.mana -= d.cost;
        let time = d.cast_time * f.attack_time();
        f.act = Act::Cast { spell, target, point, done: self.time + time as f64, scroll: false };
        true
    }

    /// Read a scroll: the spell goes off after a moment, no mana spent, no
    /// chance of fizzling. The scroll is used up.
    pub fn read_scroll(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2) -> bool {
        let f = &mut self.fighters[i];
        let Some(k) = f.scrolls.iter().position(|&s| matches!(item(s).kind, Kind::Scroll(x) if x == spell)) else { return false };
        let it = f.scrolls.remove(k);
        f.used.push(it);
        f.act = Act::Cast { spell, target, point, done: self.time + 0.6, scroll: true };
        let name = self.names[i].clone();
        self.say(format!("{name} reads a scroll of {}.", spell.def().name.to_lowercase()));
        true
    }

    /// Start drinking a potion from the pack.
    pub fn begin_drink(&mut self, i: usize, it: ItemId) -> bool {
        let f = &mut self.fighters[i];
        let Some(k) = f.potions.iter().position(|&p| p == it) else { return false };
        f.potions.remove(k);
        f.used.push(it);
        f.act = Act::Drink { item: it, done: self.time + 1.2 };
        true
    }

    fn drink(&mut self, i: usize, it: ItemId) {
        let Kind::Potion(pd) = item(it).kind else { return };
        let name = self.names[i].clone();
        let f = &mut self.fighters[i];
        if pd.heal > 0.0 {
            let stats = f.stats.clone();
            super::crafting::mend(&mut f.hp, &stats, pd.heal);
            if f.ko && !body::knocked_out(&f.hp) {
                f.ko = false;
            }
        }
        f.mana = (f.mana + pd.mana).min(f.max_mana);
        self.say(format!("{name} drinks a {}.", item(it).name.to_lowercase()));
    }

    fn resolve_cast(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2, scroll: bool, rng: &mut Rng) {
        let (r_ok, r_resist, r_dmg) = (rng.f32(), rng.f32(), rng.f32());
        let d = spell.def();
        let caster = &self.fighters[i];
        let name = self.names[i].clone();
        let chance = if scroll { 1.0 } else { magic::success_chance(&caster.stats, spell, caster.tired()) };
        if r_ok > chance {
            let at = caster.pos;
            self.fighters[i].train(d.school, 0.4);
            self.fx.push(Fx { kind: FxKind::Fizzle { at }, at: self.time });
            self.say(format!("{name}'s {} fizzles.", d.name.to_lowercase()));
            return;
        }
        self.fighters[i].train(d.school, 1.5);
        let t = self.time;
        match d.target {
            Target::Caster => {
                let kind = if spell == Spell::MageArmor { StatusKind::MageArmor } else { StatusKind::Hasted };
                let f = &mut self.fighters[i];
                f.statuses.retain(|s| s.kind != kind);
                f.statuses.push(Status { kind, until: t + d.duration as f64, magnitude: d.magnitude });
                self.say(format!("{name} casts {}.", d.name.to_lowercase()));
            }
            Target::Other => {
                let Some(j) = target else { return };
                self.wake(self.fighters[j].side);
                let tname = self.names[j].clone();
                // Blind casters can only reach what's right in front of them.
                let blind = self.fighters[i].has(StatusKind::Blinded).is_some();
                let range = if blind { 5.0 } else { d.range };
                if self.fighters[i].pos.dist(self.fighters[j].pos) > range + 1.0 || self.fighters[j].dead || self.fighters[j].fled {
                    self.say(format!("{name}'s {} goes wide.", d.name.to_lowercase()));
                    return;
                }
                match spell {
                    Spell::LightningBolt => {
                        let from = self.fighters[i].pos;
                        self.fx.push(Fx { kind: FxKind::Bolt { from, to: self.fighters[j].pos }, at: t });
                        // Armour is no help against lightning.
                        let resist = 1.0 - self.fighters[j].resist_elements;
                        let shield = self.fighters[j].has(StatusKind::MageArmor).map(|s| 1.0 - s.magnitude).unwrap_or(1.0);
                        let dmg = d.magnitude * (0.8 + r_dmg * 0.4) * resist * shield * (0.8 + self.fighters[i].stats.skill(Skill::Destruction) / 250.0);
                        self.say(format!("{name}'s lightning strikes {tname} ({dmg:.0})."));
                        self.wound(j, Part::Torso, dmg * 0.75);
                        self.wound(j, Part::Head, dmg * 0.25);
                    }
                    Spell::Paralyze | Spell::Blind => {
                        let (kind, item_resist) = if spell == Spell::Paralyze {
                            (StatusKind::Paralyzed, self.fighters[j].resist_paralysis)
                        } else {
                            (StatusKind::Blinded, self.fighters[j].resist_blind)
                        };
                        let resist = 1.0 - (1.0 - magic::willpower_resist(&self.fighters[j].stats)) * (1.0 - item_resist);
                        if r_resist < resist {
                            self.say(format!("{tname} shrugs off {name}'s {}.", d.name.to_lowercase()));
                            return;
                        }
                        let f = &mut self.fighters[j];
                        f.statuses.retain(|s| s.kind != kind);
                        f.statuses.push(Status { kind, until: t + d.duration as f64, magnitude: d.magnitude });
                        if kind == StatusKind::Paralyzed {
                            f.act = Act::Idle;
                        }
                        self.say(format!("{name} {} {tname}.", if kind == StatusKind::Paralyzed { "paralyzes" } else { "blinds" }));
                    }
                    _ => {}
                }
            }
            Target::Ally => {
                let j = target.unwrap_or(i);
                let tname = self.names[j].clone();
                if self.fighters[i].pos.dist(self.fighters[j].pos) > d.range + 1.0 || self.fighters[j].dead {
                    self.say(format!("{name}'s heal falls short."));
                    return;
                }
                let mut left = d.magnitude * (0.8 + self.fighters[i].stats.skill(Skill::Restoration) / 200.0) * (0.9 + r_dmg * 0.2);
                // Worst wounds first: head and torso when someone is down,
                // otherwise whatever is most hurt.
                while left > 0.5 {
                    let f = &self.fighters[j];
                    let worst = (0..6)
                        .filter(|&k| f.hp[k] < f.max_hp[k])
                        .max_by(|&a, &b| {
                            let need = |k: usize| (f.max_hp[k] - f.hp[k]) / f.max_hp[k] + if f.ko && k < 2 && f.hp[k] <= 0.0 { 10.0 } else { 0.0 };
                            need(a).total_cmp(&need(b))
                        });
                    let Some(k) = worst else { break };
                    let give = left.min(self.fighters[j].max_hp[k] - self.fighters[j].hp[k]).min(12.0);
                    self.fighters[j].hp[k] += give;
                    left -= give;
                }
                self.say(format!("{name} heals {tname}."));
                let f = &mut self.fighters[j];
                if f.ko && !f.dead && !body::knocked_out(&f.hp) {
                    f.ko = false;
                    f.act = Act::Idle;
                    self.say(format!("{tname} gets back up."));
                }
            }
            Target::Ground => {
                // Fireball: everyone in the blast, friend or foe.
                self.fx.push(Fx { kind: FxKind::Fireball { at: point, radius: d.radius }, at: t });
                self.say(format!("{name}'s fireball bursts."));
                let power = 0.8 + self.fighters[i].stats.skill(Skill::Destruction) / 250.0;
                for j in 0..self.fighters.len() {
                    let f = &self.fighters[j];
                    if f.dead || f.fled {
                        continue;
                    }
                    let dist = f.pos.dist(point);
                    if dist > d.radius {
                        continue;
                    }
                    // Armour helps a little against fire (it's mostly heat).
                    let armour = f.armor.iter().filter(|a| a.covers.contains(&Part::Torso)).map(|a| a.blunt * 0.3).fold(0.0, f32::max);
                    let shield = f.has(StatusKind::MageArmor).map(|s| 1.0 - s.magnitude).unwrap_or(1.0);
                    let dmg = d.magnitude * power * (1.0 - dist / d.radius * 0.5) * (0.8 + r_dmg * 0.4) * (1.0 - f.resist_elements) * (1.0 - armour) * shield;
                    self.wound(j, Part::Torso, dmg * 0.5);
                    self.wound(j, Part::LeftArm, dmg * 0.15);
                    self.wound(j, Part::RightArm, dmg * 0.15);
                    self.wound(j, Part::Head, dmg * 0.2);
                }
            }
        }
    }

    pub fn winner(&self) -> Option<Side> {
        let mut sides: Vec<Side> = self.fighters.iter().filter(|f| f.active()).map(|f| f.side).collect();
        sides.dedup();
        if sides.len() == 1 {
            Some(sides[0])
        } else {
            None
        }
    }
}

/// Only about half of misses and blocks are worth a line in the log.
fn rng_line(r: f32) -> bool {
    r < 0.5
}

/// One number for how dangerous someone is in a fight, used by the far bands
/// in place of a blow-by-blow fight. Roughly: expected damage dealt per
/// second against an average opponent, times how much punishment they can
/// take, square-rooted so it adds up sensibly across a group — plus a share
/// for magic.
pub fn rating(race: Race, base: &Stats, gear: &Gear, spells: &[Spell]) -> f32 {
    let s = inventory::effective(base, gear);
    let w = gear.weapon();
    let skill = s.skill(w.skill);
    let str_ = s.attr(Attr::Strength);
    let dmg = (w.cut * (1.0 + str_ * 0.006) * 0.7 + w.blunt * (1.0 + str_ * 0.010)) * (0.6 + skill * 0.006);
    let rate = 1.0 / ((w.windup + w.recover) * (1.25 - s.attr(Attr::Agility) * 0.005).clamp(0.5, 1.3));
    let hit = (0.5 + (skill + s.attr(Attr::Agility) * 0.25 + 10.0 - 26.0) * 0.012).clamp(0.08, 0.95);
    let offense = dmg * rate * hit;

    let armor: Vec<ArmorDef> = gear.equipped().filter_map(|id| item(id).armor().copied()).collect();
    let mut reduction = 0.0;
    for p in PARTS {
        let best = armor.iter().filter(|a| a.covers.contains(&p)).map(|a| a.coverage * (a.cut + a.blunt) * 0.5).fold(0.0, f32::max);
        reduction += p.hit_weight() * best;
    }
    let vital = s.max_hp(Part::Torso) + s.max_hp(Part::Head) * 0.5;
    let dodge = (s.skill(Skill::Dodge) * 0.6 + s.attr(Attr::Agility) * 0.25) * 0.008;
    let guard = gear.shield() * 0.6 + w.parry * 0.3;
    let ehp = vital / (1.0 - reduction.min(0.8)) * (1.0 + dodge) * (1.0 + guard);

    let magic = if spells.is_empty() {
        0.0
    } else {
        s.max_mana(race) * (s.best_magic() / 100.0) * 0.25 * (spells.len() as f32 / 6.0).sqrt()
    };
    (offense * ehp).sqrt() * 0.9 + magic
}
