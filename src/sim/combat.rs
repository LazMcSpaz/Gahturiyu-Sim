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

use serde::{Deserialize, Serialize};

use super::body::{self, Part, PARTS};
use super::geo::V2;
use super::inventory::{self, Gear};
use super::effects::{Does, Effect, Element, Lasts, Reach, Summon, Who};
use super::items::{item, ArmorDef, ItemId, Kind, WeaponDef, FISTS};
use super::magic::{self, Aim, Spell, Status, Style};
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng::Rng;
use super::stats::{Attr, Skill, Stats};

/// Length of one combat tick, game seconds.
pub const DT: f64 = 0.1;
/// Damage multiplier for hitting someone who hasn't noticed you (before the
/// attacker's Sneak adds to it).
pub const SNEAK_ATTACK: f32 = 2.0;
/// Chance to hit in pitch dark, as a share of in full light: shots, and
/// blows up close (where you can at least make out a shape).
pub const DARK_SHOT: f32 = 0.45;
pub const DARK_BLOW: f32 = 0.85;
/// An archer with no hand weapon backs away from anyone closer than this.
pub const ARCHER_SPACE: f32 = 5.0;
/// An archer with a hand weapon draws it when an enemy gets this close...
pub const ARCHER_DRAW: f32 = 3.5;
/// ...and goes back to the bow once they're this far off.
pub const ARCHER_STOW: f32 = 9.0;
/// A fight that drags on longer than this simply ends (everyone disengages).
pub const MAX_LENGTH: f64 = 900.0;
/// Body radius, metres: added to weapon reach.
const BODY: f32 = 0.45;

pub type Side = u8;
pub const SQUAD_SIDE: Side = 0;

/// The side the already-dead lie on: bodies brought into a fight to be
/// raised, belonging to no one.
pub const GRAVE_SIDE: Side = 250;

/// The "person" behind a fighter that isn't one: a summoned creature or a
/// raised corpse.
/// How long a net's tangle lasts, seconds.
pub const ENTANGLE_SECS: f64 = 4.0;
pub const NOBODY: PersonId = PersonId::MAX;

/// A fighter that a spell called up.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Summoned {
    pub kind: Summon,
    /// When the binding runs out and it fades.
    pub until: f64,
    /// Goes for the nearest enemy and nothing else.
    pub mindless: bool,
}

/// A lasting spell worked on a patch of ground in a fight.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Zone {
    pub does: Does,
    pub pos: V2,
    pub radius: f32,
    pub power: f32,
    pub until: f64,
    /// Whose it is (friends of this side are spared what it does to foes).
    pub side: Side,
    /// Who worked it (`NOBODY` if no one in particular).
    pub owner: PersonId,
    /// Worked during this fight (rather than brought in from the world).
    pub fresh: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Order {
    /// Go here and don't stop to fight on the way.
    MoveTo(V2),
    /// Go for this one.
    Attack(usize),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fighter {
    pub pid: PersonId,
    /// The side they're fighting for now (a spell can turn them).
    pub side: Side,
    /// The side they came in on.
    pub home: Side,
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
    #[serde(with = "super::save::name")]
    pub weapon_name: super::save::Name,
    pub shield: f32,
    pub armor: Vec<ArmorDef>,
    pub dodge_penalty: f32,
    /// Effects from what they wear: (what it does, how strong). Together with
    /// `statuses`, read through `power`.
    pub worn: Vec<(Does, f32)>,
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
    /// Carrying someone: can move, can't fight or block.
    pub burdened: bool,
    /// Holding a lit torch: lights the ground round them.
    pub torch: bool,
    /// Has a torch in hand at all (lit or not), for kindling.
    #[serde(default)]
    pub has_torch: bool,
    /// Limbs lost for good (before or during this fight).
    pub missing: [bool; 6],
    /// Shots left for a ranged weapon, and shots loosed this fight.
    pub ammo: u16,
    pub shots: u16,
    /// A hand weapon in the pack to draw when enemies close in on an archer,
    /// and the ranged weapon put away meanwhile.
    #[serde(with = "super::save::named_weapon")]
    pub sidearm: Option<(WeaponDef, &'static str)>,
    #[serde(with = "super::save::named_weapon")]
    pub stowed: Option<(WeaponDef, &'static str)>,
    /// Potions and scrolls in their pack, and those used up this fight.
    pub potions: Vec<ItemId>,
    pub scrolls: Vec<ItemId>,
    pub used: Vec<ItemId>,
    /// Until this time they haven't realised they're under attack: they
    /// stand there, can't dodge or block, and take a sneak attack's damage.
    pub aware_at: f64,

    /// Skill uses this fight, written back to the person afterwards.
    pub trained: [f32; super::stats::N_SKILLS],
    /// Tiredness from felt casting this fight, added to the squad member's
    /// own afterwards.
    pub tire: f32,
    /// A ritual held ready, to be released (once) when it's wanted.
    pub held: Option<Spell>,
    pub damage_taken: f32,
    /// Called up by a spell (not a person).
    #[serde(default)]
    pub summon: Option<Summoned>,
    /// A corpse already raised (it can't be raised twice).
    #[serde(default)]
    pub raised: bool,
    /// Gear shattered by a spell this fight (gone for good afterwards).
    #[serde(default)]
    pub broke: Vec<super::items::Slot>,
    /// Which slot each piece of `armor` is worn in.
    #[serde(default)]
    pub armor_slots: Vec<super::items::Slot>,
    /// Wear this fight, written back afterwards: blows struck with the
    /// weapon, blows taken by the shield, and for each piece of armour the
    /// blows it caught and how many of them were heavy blunt ones.
    #[serde(default)]
    pub weapon_wear: f32,
    #[serde(default)]
    pub shield_wear: f32,
    #[serde(default)]
    pub armor_wear: Vec<[f32; 2]>,
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
            None => super::magic::starting_spells(&p.kit_stats, p.seed),
        };
        let mut stats = inventory::effective(&p.stats, gear);
        p.weaken(&mut stats);
        let mut max_hp = [0.0; 6];
        for (i, part) in PARTS.iter().enumerate() {
            max_hp[i] = p.stats.max_hp(*part);
        }
        let worn_armor: Vec<(super::items::Slot, ArmorDef)> = super::items::SLOTS.iter().filter_map(|&s| gear.in_slot(s).and_then(|id| item(id).armor().copied()).map(|a| (s, a))).collect();
        let armor: Vec<ArmorDef> = worn_armor.iter().map(|x| x.1).collect();
        let armor_slots: Vec<super::items::Slot> = worn_armor.iter().map(|x| x.0).collect();
        let dodge_penalty = armor.iter().map(|a| a.dodge_penalty).sum();
        let speed_bonus = gear.worn(Does::MoveSpeed);
        let load = gear.load(&p.stats);
        let held = gear.weapon();
        let count = |key: &str| gear.bag.iter().filter(|e| item(e.0).key == key).map(|e| e.1).sum::<u16>();
        let ammo = held.ammo.map(count).unwrap_or(0);
        // The best hand weapon in the pack, for an archer pressed close.
        let sidearm = gear
            .bag
            .iter()
            .filter_map(|e| item(e.0).weapon().filter(|w| w.range == 0.0).map(|w| (*w, item(e.0).name)))
            .max_by(|a, b| (a.0.cut + a.0.blunt).total_cmp(&(b.0.cut + b.0.blunt)));
        Fighter {
            pid: p.id,
            side,
            home: side,
            race: p.race,
            pos,
            hp: p.wounds.hp_at(&p.stats, t),
            max_hp,
            // Squad members bring the stamina they have; everyone else is fresh.
            fatigue: p.cond.as_ref().map(|c| c.stamina_at(t).min(stats.max_fatigue())).unwrap_or(stats.max_fatigue()),
            max_fatigue: stats.max_fatigue(),
            mana: p.mana_at(t),
            max_mana: p.max_mana(),
            mana_regen: p.mana_regen() / 60.0,
            weapon: gear.weapon(),
            weapon_name: gear.weapon_name(),
            shield: gear.shield(),
            armor,
            dodge_penalty,
            worn: gear.worn_effects().collect(),
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
            burdened: false,
            torch: false,
            has_torch: false,
            missing: p.wounds.missing,
            ammo,
            shots: 0,
            sidearm: if held.range > 0.0 { sidearm } else { None },
            stowed: None,
            potions: gear.bag.iter().filter(|e| item(e.0).kind == Kind::Potion).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            scrolls: gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Scroll(_))).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            used: Vec::new(),
            trained: [0.0; super::stats::N_SKILLS],
            tire: 0.0,
            held: None,
            damage_taken: 0.0,
            summon: None,
            raised: false,
            broke: Vec::new(),
            armor_wear: vec![[0.0; 2]; armor_slots.len()],
            armor_slots: armor_slots,
            weapon_wear: 0.0,
            shield_wear: 0.0,
        }
    }

    /// A creature called up by a spell, `power` times as strong as made.
    pub fn creature(kind: Summon, side: Side, pos: V2, until: f64, power: f32, race: Race) -> Fighter {
        let c = creature(kind);
        let mut skills = [0.0; super::stats::N_SKILLS];
        skills[Skill::Unarmed as usize] = c.skill;
        skills[Skill::Dodge as usize] = c.dodge;
        skills[Skill::Block as usize] = c.skill * 0.5;
        let stats = Stats { attrs: [c.strength, 45.0, c.toughness, 10.0, 60.0], skills, calling: super::stats::Calling::Common };
        let mut max_hp = [0.0; 6];
        for (i, part) in PARTS.iter().enumerate() {
            max_hp[i] = stats.max_hp(*part) * c.hp * power;
        }
        let weapon = WeaponDef { skill: Skill::Unarmed, cut: c.cut * power, blunt: c.blunt * power, reach: c.reach, windup: 0.5, recover: 0.45, two_handed: false, parry: 0.0, range: 0.0, ammo: None, pierce: 0.0, entangle: 0.0 };
        Fighter {
            pid: NOBODY,
            side,
            home: side,
            race,
            pos,
            hp: max_hp,
            max_hp,
            fatigue: 1000.0,
            max_fatigue: 1000.0,
            mana: 0.0,
            max_mana: 0.0,
            mana_regen: 0.0,
            stats,
            weapon,
            weapon_name: c.attack,
            shield: c.shield,
            armor: Vec::new(),
            dodge_penalty: 0.0,
            worn: Vec::new(),
            base_speed: c.speed,
            spells: Vec::new(),
            boldness: 1.0,
            might: c.might * power,
            act: Act::Idle,
            target: None,
            order: None,
            statuses: Vec::new(),
            ko: false,
            dead: false,
            fleeing: false,
            fled: false,
            think_at: 0.0,
            burdened: false,
            torch: false,
            has_torch: false,
            missing: [false; 6],
            ammo: 0,
            shots: 0,
            sidearm: None,
            stowed: None,
            potions: Vec::new(),
            scrolls: Vec::new(),
            used: Vec::new(),
            aware_at: 0.0,
            trained: [0.0; super::stats::N_SKILLS],
            tire: 0.0,
            held: None,
            damage_taken: 0.0,
            summon: Some(Summoned { kind, until, mindless: false }),
            raised: false,
            broke: Vec::new(),
            armor_wear: Vec::new(),
            armor_slots: Vec::new(),
            weapon_wear: 0.0,
            shield_wear: 0.0,
        }
    }

    /// A real person (not a summoned creature or a raised corpse).
    pub fn is_person(&self) -> bool {
        self.summon.is_none()
    }

    /// An illusion that only draws attacks.
    pub fn is_decoy(&self) -> bool {
        self.summon.map(|s| s.kind == Summon::Decoy).unwrap_or(false)
    }

    /// Caught unawares at time `t`.
    pub fn unaware(&self, t: f64) -> bool {
        t < self.aware_at
    }

    /// Can act at all: not down, dead or gone.
    pub fn active(&self) -> bool {
        !self.ko && !self.dead && !self.fled
    }

    /// A lasting effect in force on them, if there is one.
    pub fn has(&self, does: Does) -> Option<&Status> {
        self.statuses.iter().find(|s| s.does == does)
    }

    /// How strongly something works on them: from what they wear plus any
    /// spell in force.
    pub fn power(&self, does: Does) -> f32 {
        super::effects::total(self.worn.iter().copied(), does) + self.statuses.iter().filter(|s| s.does == does).map(|s| s.power).sum::<f32>()
    }

    /// Share of damage that gets past wards (Barrier and the like).
    pub fn warded(&self) -> f32 {
        (1.0 - self.power(Does::Barrier)).clamp(0.1, 1.0)
    }

    pub fn paralyzed(&self) -> bool {
        self.has(Does::Paralyze).is_some()
    }

    /// Held still or knocked flat: can't move, act, dodge or block.
    pub fn helpless(&self) -> bool {
        self.paralyzed() || self.has(Does::KnockDown).is_some()
    }

    /// An attribute with spells in force added.
    pub fn attr(&self, a: Attr) -> f32 {
        self.stats.attr(a) + self.power(Does::Attr(a))
    }

    /// A skill with spells in force added.
    pub fn skill(&self, k: Skill) -> f32 {
        self.stats.skill(k) + self.power(Does::Skill(k))
    }

    /// 1 when fresh, down to 0.6 when spent.
    pub fn tired(&self) -> f32 {
        0.6 + 0.4 * (self.fatigue / self.max_fatigue.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn haste(&self) -> f32 {
        self.power(Does::Haste)
    }

    pub fn speed(&self) -> f32 {
        if self.helpless() || matches!(self.act, Act::Cast { .. }) {
            return 0.0;
        }
        let drag = (1.0 - self.power(Does::Slow) - self.power(Does::Burden) * 0.6 - self.power(Does::Enlarge) * 0.2).max(0.2);
        self.base_speed * body::leg_factor(&self.hp) * (1.0 + self.haste()) * drag * if self.fleeing { 1.1 } else { 1.0 }
    }

    /// Multiplier on swing and recovery times.
    pub fn attack_time(&self) -> f32 {
        ((1.25 - self.attr(Attr::Agility) * 0.005) * (1.0 - self.haste() * 0.6) * (1.0 + self.power(Does::Slow) * 0.5 + self.power(Does::Burden) * 0.3)).clamp(0.5, 1.6)
    }

    /// The weapon actually usable: a ruined (or lost) sword arm means fists
    /// from the other hand; a two-handed weapon needs both arms; both arms
    /// gone means no attack at all.
    pub fn usable_weapon(&self) -> Option<(WeaponDef, f32)> {
        if self.is_decoy() {
            return None;
        }
        let (l, r) = (self.hp[Part::LeftArm as usize] > 0.0, self.hp[Part::RightArm as usize] > 0.0);
        match (r, l) {
            (true, true) => Some((self.weapon, 1.0)),
            (true, false) if !self.weapon.two_handed => Some((self.weapon, 1.0)),
            (true, false) | (false, true) => Some((FISTS, 0.6)),
            _ => None,
        }
    }

    /// Shield in use: only with a working left arm.
    pub fn shield_up(&self) -> f32 {
        if self.hp[Part::LeftArm as usize] > 0.0 {
            self.shield
        } else {
            0.0
        }
    }

    pub fn reach(&self) -> f32 {
        (self.usable_weapon().map(|w| w.0.reach).unwrap_or(0.8) + BODY * 2.0) * (1.0 - self.power(Does::Shrink) * 0.5 + self.power(Does::Enlarge) * 0.5)
    }

    /// How hard their blows land for their size: shrunk softer, enlarged harder.
    pub fn size(&self) -> f32 {
        (1.0 - self.power(Does::Shrink) + self.power(Does::Enlarge)).max(0.2)
    }

    /// Holding a ranged weapon with something to shoot.
    pub fn shooting(&self) -> bool {
        self.weapon.range > 0.0 && self.ammo > 0 && self.usable_weapon().map(|w| w.1 >= 1.0).unwrap_or(false)
    }

    /// How far away they can attack from.
    pub fn attack_range(&self) -> f32 {
        if self.shooting() {
            self.weapon.range
        } else {
            self.reach()
        }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FxKind {
    Fireball { at: V2, radius: f32 },
    Bolt { from: V2, to: V2 },
    Fizzle { at: V2 },
    /// An arrow or bolt in flight: loosed at `from`, coming down at `to`.
    /// `hit` is whether it struck (or was blocked by) the target; a miss lands
    /// a little past them and lies there.
    Arrow { from: V2, to: V2, hit: bool },
}

/// A visual moment for the window to show.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fx {
    pub kind: FxKind,
    pub at: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    /// Lights that stay put round the fight (fires, town, standing torches).
    /// `None`: fought in broad daylight (tests, mostly). With lights, the
    /// sun follows the battle clock and fighters' torches are added.
    pub lights: Option<Vec<super::torch::Light>>,
    /// Lasting spells on patches of ground.
    #[serde(default)]
    pub zones: Vec<Zone>,
}

impl Battle {
    pub fn new(id: u32, seed: u64, start: f64, fighters: Vec<Fighter>, names: Vec<String>) -> Battle {
        Battle { id, seed, start, time: start, ticks: 0, fighters, over: false, log: Vec::new(), fx: Vec::new(), names, lights: None, zones: Vec::new() }
    }

    /// How well lit a spot in the fight is, 0..1.
    pub fn light_at(&self, p: V2) -> f32 {
        let Some(fixed) = &self.lights else { return 1.0 };
        let mut all = fixed.clone();
        for f in &self.fighters {
            if f.dead || f.fled {
                continue;
            }
            if f.torch {
                all.push(super::torch::Light { pos: f.pos, reach: super::torch::TORCH_REACH, power: super::torch::TORCH_POWER, flat: false });
            }
            // Glow and gloom go with whoever bears them.
            for s in &f.statuses {
                if let Some(l) = super::torch::spell_light(s.does, s.power, f.pos, 0.0) {
                    all.push(l);
                }
            }
        }
        for z in self.zones.iter().filter(|z| z.until > self.time) {
            if let Some(l) = super::torch::spell_light(z.does, z.power, z.pos, z.radius) {
                all.push(l);
            }
        }
        super::torch::light_from(self.time, &all, p)
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

        // Bindings run out: called-up creatures fade.
        for i in 0..n {
            let f = &self.fighters[i];
            if let Some(s) = f.summon {
                if s.until <= t && !f.fled && !f.dead {
                    self.fighters[i].fled = true;
                    let name = self.names[i].clone();
                    self.say(format!("{name} fades away."));
                }
            }
        }
        self.zones.retain(|z| z.until > t);
        // Blight eats at everyone standing in it.
        // (Not the downed: rot doesn't finish off the fallen. A ward against
        // necromancy helps.)
        let blights: Vec<Zone> = self.zones.iter().copied().filter(|z| z.does == Does::Blight).collect();
        for z in blights {
            for j in 0..n {
                let f = &self.fighters[j];
                if f.active() && f.pos.dist(z.pos) <= z.radius {
                    let ward = (1.0 - f.power(Does::DomainResist(super::magic::Domain::Necromancy))).max(0.05);
                    let dmg = z.power * DT as f32 * f.warded() * ward;
                    self.hurt_whole(j, dmg);
                }
            }
        }

        // Spells wear off; breath and mana come back.
        for f in &mut self.fighters {
            f.statuses.retain(|s| s.until > t);
            // A turned fighter goes back to their own side.
            if f.side != f.home && f.has(Does::Dominate).is_none() {
                f.side = f.home;
            }
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
            if self.fighters[i].helpless() {
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

        // Nothing mends a lost limb (potions and healing spells included).
        for f in &mut self.fighters {
            for k in 0..6 {
                if f.missing[k] {
                    f.hp[k] = f.hp[k].min(-body::LIMB_LOSS * f.max_hp[k]);
                }
            }
        }

        // Over when no two sides still standing are enemies, or it's dragged on.
        // People running away don't keep a fight going.
        // (A fighter turned by a spell still counts for the side they came in on.)
        let mut sides: Vec<Side> = self.fighters.iter().filter(|f| f.active() && !f.fleeing && !f.is_decoy()).map(|f| f.home).collect();
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

    /// Move fighter `i` to `to`, unless that would take them into a
    /// sanctuary held against them.
    fn step_to(&mut self, i: usize, to: V2) {
        let me = &self.fighters[i];
        let barred = self.zones.iter().any(|z| z.does == Does::Sanctuary && z.until > self.time && z.side != me.side && to.dist(z.pos) < z.radius && me.pos.dist(z.pos) >= z.radius);
        if !barred {
            self.fighters[i].pos = to;
        }
    }

    /// Doing nothing in particular: carry out orders, chase the target, or run.
    fn idle(&mut self, i: usize) {
        let me = &self.fighters[i];
        let speed = me.speed() * DT as f32;
        let afraid = me.has(Does::Fear).is_some();
        if me.fleeing || afraid {
            let threat = self.nearest_enemy(i);
            let away = match threat {
                Some(j) => {
                    let d = me.pos.sub(self.fighters[j].pos);
                    // Fear only drives them off a way; real flight gets them out.
                    if afraid && !me.fleeing && d.len() > 30.0 {
                        return;
                    }
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
            self.step_to(i, me.pos.add(away.scale(speed)));
            return;
        }
        if let Some(Order::MoveTo(p)) = me.order {
            let d = p.sub(me.pos);
            if d.len() < 0.6 {
                self.fighters[i].order = None;
            } else {
                self.step_to(i, me.pos.add(d.scale(speed.min(d.len()) / d.len())));
            }
            return;
        }
        let Some(j) = me.target else { return };
        // An archer with no hand weapon backs away from anyone too close.
        if me.shooting() && me.sidearm.is_none() {
            if let Some(n) = self.nearest_enemy(i) {
                let away = me.pos.sub(self.fighters[n].pos);
                if away.len() < ARCHER_SPACE {
                    let step = speed * 0.8;
                    self.step_to(i, me.pos.add(away.scale(step / away.len().max(0.01))));
                    return;
                }
            }
        }
        let them = &self.fighters[j];
        let d = them.pos.sub(me.pos);
        let dist = d.len();
        let reach = me.attack_range();
        if dist > reach * 0.92 {
            let step = speed.min(dist - reach * 0.85).max(0.0);
            self.step_to(i, me.pos.add(d.scale(step / dist.max(0.01))));
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
        let shot = att.shooting();
        let dist = att.pos.dist(def.pos);
        if !def.active() || dist > if shot { weapon.range * 1.05 } else { att.reach() * 1.3 } {
            return;
        }
        // A bow without arrows (or swung up close with no hand weapon) is a club, and a poor one.
        let weapon = if weapon.range > 0.0 && !shot { FISTS } else { weapon };
        if shot {
            self.fighters[a].ammo -= 1;
            self.fighters[a].shots += 1;
        }
        let att = &self.fighters[a];
        let def = &self.fighters[d];
        // Blindness: a swing in roughly the right direction, mostly missing.
        let blinded = att.power(Does::Blind).min(1.0);
        let blind = 1.0 - blinded * 0.3;
        let skill = att.skill(weapon.skill);
        let atk = (skill + att.attr(Attr::Agility) * 0.25 + 10.0) * att.tired() * arm * blind;
        let unaware = def.unaware(self.time);
        let helpless = def.helpless() || unaware;
        let dodge = if helpless {
            -40.0
        } else {
            (def.skill(Skill::Dodge) * 0.6 + def.attr(Attr::Agility) * 0.25 - def.dodge_penalty * 100.0 + def.power(Does::Shrink) * 30.0 - def.power(Does::Enlarge) * 20.0) * def.tired()
        };
        // Arrows are dodged less but lose accuracy with distance.
        let dodge = if shot { dodge * 0.5 } else { dodge };
        let far = if shot { 1.0 - 0.35 * dist / weapon.range.max(1.0) } else { 1.0 };
        // In the dark it's hard to hit what you can't see, an arrow most of all.
        let seen = self.light_at(def.pos).max(att.power(Does::Nightsight).min(1.0));
        let dark = if shot { DARK_SHOT + (1.0 - DARK_SHOT) * seen } else { DARK_BLOW + (1.0 - DARK_BLOW) * seen };
        let p_hit = (0.5 + (atk - dodge) * 0.012).clamp(0.08, 0.95) * (1.0 - blinded * 0.7) * far * dark;
        let (an, dn) = (self.names[a].clone(), self.names[d].clone());
        if shot {
            // Only for drawing. A miss comes down a few metres past the target,
            // placed from dice already rolled so nothing else shifts.
            let hit = r_hit <= p_hit;
            let (from, at) = (att.pos, def.pos);
            let dir = at.sub(from).scale(1.0 / dist.max(0.01));
            let to = if hit { at } else { at.add(dir.scale(2.0 + r_part * 6.0)).add(V2::new(-dir.y, dir.x).scale((r_dmg - 0.5) * 3.0)) };
            self.fx.push(Fx { kind: FxKind::Arrow { from, to, hit }, at: self.time });
        }
        let att = &self.fighters[a];
        let strength = att.attr(Attr::Strength);
        self.fighters[a].fatigue -= 3.0 + weapon.windup * 4.0;

        if r_hit > p_hit {
            self.fighters[d].train(Skill::Dodge, 1.0);
            self.fighters[a].train(weapon.skill, 0.3);
            if rng_line(r_dmg) {
                self.say(format!("{dn} dodges {an}."));
            }
            return;
        }
        // A brace turns the blow that would have landed, then it's spent.
        if self.fighters[d].has(Does::Brace).is_some() {
            self.fighters[d].statuses.retain(|s| s.does != Does::Brace);
            self.say(format!("{dn}'s brace turns {an}'s blow."));
            return;
        }
        let def = &self.fighters[d];
        // Only a shield stops an arrow.
        let guard = if helpless || def.burdened {
            0.0
        } else if shot {
            def.shield_up() * 100.0
        } else {
            def.shield_up() * 100.0 + def.weapon.parry * 60.0
        };
        let p_block = if guard <= 0.0 { 0.0 } else { (guard * (0.3 + def.skill(Skill::Block) / 100.0) / (guard + atk + 20.0)).clamp(0.0, 0.75) };
        let roll = 0.8 + r_dmg * 0.4;
        let skill_mult = 0.6 + skill * 0.006;
        // Strength puts weight behind a blow; a bowstring doesn't care.
        let pull = if shot { 0.0 } else { 1.0 };
        let size = if shot { 1.0 } else { self.fighters[a].size() };
        let mut cut = weapon.cut * (1.0 + strength * 0.006 * pull) * skill_mult * roll * size;
        let mut blunt = weapon.blunt * (1.0 + strength * 0.010 * pull) * skill_mult * roll * size;
        // The weapon wears with every blow that connects (bowstrings less).
        self.fighters[a].weapon_wear += if shot { 0.3 } else { 1.0 };

        if r_block < p_block {
            if self.fighters[d].shield > 0.0 {
                self.fighters[d].shield_wear += 1.0;
            }
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
        // Toughened skin is one more layer, everywhere.
        let skin = self.fighters[d].power(Does::Toughen).min(0.8);
        let skin = ArmorDef { covers: &PARTS, coverage: 1.0, cut: skin, blunt: skin * 0.6, dodge_penalty: 0.0 };
        // (Indices into the armour worn; None for the skin.)
        let mut layers: Vec<(Option<usize>, ArmorDef)> = self.fighters[d].armor.iter().enumerate().filter(|(_, a)| a.covers.contains(&part)).map(|(i, a)| (Some(i), *a)).collect();
        if skin.cut > 0.0 {
            layers.push((None, skin));
        }
        layers.sort_by(|x, y| y.1.cut.total_cmp(&x.1.cut));
        // Rusted armour stops less (skin doesn't rust); a heavy point gets
        // through some of it.
        let rust = 1.0 - self.fighters[d].power(Does::Rust).min(1.0);
        let through = 1.0 - weapon.pierce.clamp(0.0, 0.9);
        let heavy = blunt > cut;
        for (k, (which, layer)) in layers.iter().enumerate() {
            let r = if k == 0 { r_cover1 } else { r_cover2 };
            let worn = if which.is_none() { 1.0 } else { rust * through };
            if r < layer.coverage {
                cut *= 1.0 - layer.cut * worn;
                blunt *= 1.0 - layer.blunt * worn;
                if let Some(i) = which {
                    if let Some(w) = self.fighters[d].armor_wear.get_mut(*i) {
                        w[0] += 1.0;
                        if heavy {
                            w[1] += 1.0;
                        }
                    }
                }
            }
        }
        let shield = self.fighters[d].warded();
        let mut dmg = (cut + blunt) * shield;
        self.fighters[a].train(weapon.skill, 1.0);
        if unaware {
            // A sneak attack: the better the sneak, the worse the wound.
            let sneak = self.fighters[a].skill(Skill::Sneak);
            dmg *= SNEAK_ATTACK + sneak / 50.0;
            self.fighters[a].train(Skill::Sneak, 3.0);
            self.say(format!("{an} catches {dn} unawares: the {} ({:.0}).", part.name(), dmg));
        } else if shot {
            self.say(format!("{an} shoots {dn} in the {} ({:.0}).", part.name(), dmg));
        } else {
            self.say(format!("{an} hits {dn} in the {} ({:.0}).", part.name(), dmg));
        }
        self.wound(d, part, dmg);
        // A net tangles them up for a few seconds.
        if weapon.entangle > 0.0 && (r_dmg * 7.31).fract() < weapon.entangle && !self.fighters[d].dead {
            let until = self.time + ENTANGLE_SECS;
            self.fighters[d].statuses.push(super::magic::Status { does: Does::Slow, power: 0.75, until });
            self.say(format!("{an}'s net tangles {dn}."));
        }
    }

    /// Switch between bow and hand weapon (takes a moment).
    pub fn swap_weapon(&mut self, i: usize) {
        let t = self.time;
        let f = &mut self.fighters[i];
        let current = (f.weapon, f.weapon_name);
        let next = if let Some(st) = f.stowed.take() {
            f.sidearm = Some(current);
            st
        } else if let Some(sa) = f.sidearm.take() {
            f.stowed = Some(current);
            sa
        } else {
            return;
        };
        f.weapon = next.0;
        f.weapon_name = next.1;
        f.act = Act::Recover { until: t + 0.8 };
        let name = self.names[i].clone();
        self.say(format!("{name} switches to the {}.", next.1.to_lowercase()));
    }

    /// Everyone on a side realises they're under attack.
    pub fn wake(&mut self, side: Side) {
        let t = self.time;
        for f in self.fighters.iter_mut().filter(|f| f.side == side && f.aware_at > t) {
            f.aware_at = t;
        }
    }

    /// Apply damage to a body part and see what it does.
    pub fn wound(&mut self, d: usize, part: Part, dmg: f32) {
        if dmg <= 0.0 {
            return;
        }
        self.wake(self.fighters[d].side);
        let f = &mut self.fighters[d];
        // A blow breaks a calm.
        f.statuses.retain(|s| s.does != Does::Calm);
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
        // Battered far enough, a limb is gone for good.
        let i = part as usize;
        if part.is_limb() && !self.fighters[d].missing[i] && self.fighters[d].hp[i] <= -body::LIMB_LOSS * self.fighters[d].max_hp[i] {
            self.fighters[d].missing[i] = true;
            self.fighters[d].hp[i] = -body::LIMB_LOSS * self.fighters[d].max_hp[i];
            self.say(format!("{name} loses the {}!", part.name()));
        }
    }

    // ---- Spells -----------------------------------------------------------

    /// Start casting. Energy is spent now, whether or not it works. Felt
    /// spells go off at once (this tick) and tire the caster a little;
    /// structured ones take a second or two, and a solid hit spoils them.
    /// Rituals can't be cast mid-fight at all.
    pub fn begin_cast(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2) -> bool {
        let d = spell.def();
        let f = &mut self.fighters[i];
        if f.mana < d.cost || !f.spells.contains(&spell) || d.style == Style::Ritual {
            return false;
        }
        f.mana -= d.cost;
        f.tire += d.tire;
        // (Quickness shades a structured cast, but it always takes 1–2 s.)
        let time = if d.style == Style::Felt { 0.0 } else { (d.cast_time * f.attack_time()).clamp(1.0, 2.0) };
        f.act = Act::Cast { spell, target, point, done: self.time + time as f64, scroll: false };
        true
    }

    /// Let a held ritual go: it works at once, and can't fail (the gamble
    /// was in performing it).
    pub fn release(&mut self, i: usize, target: Option<usize>, point: V2) -> bool {
        let Some(spell) = self.fighters[i].held.take() else { return false };
        self.fighters[i].act = Act::Cast { spell, target, point, done: self.time, scroll: true };
        let name = self.names[i].clone();
        self.say(format!("{name} releases the {}!", spell.def().name.to_lowercase()));
        true
    }

    /// Read a scroll: the spell goes off after a moment, no energy spent, no
    /// chance of fizzling. The scroll is used up.
    pub fn read_scroll(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2) -> bool {
        let key = spell.def().key;
        let f = &mut self.fighters[i];
        let Some(k) = f.scrolls.iter().position(|&s| matches!(item(s).kind, Kind::Scroll(x) if x == key)) else { return false };
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
        if item(it).kind != Kind::Potion {
            return;
        }
        let name = self.names[i].clone();
        self.say(format!("{name} drinks a {}.", item(it).name.to_lowercase()));
        let src = Source { by: i, spell: None, target: Some(i), point: self.fighters[i].pos, skill: 1.0, r_dmg: 0.5, r_resist: 1.0 };
        for e in item(it).effects {
            self.apply(&src, e);
        }
    }

    fn resolve_cast(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2, scroll: bool, rng: &mut Rng) {
        let (r_ok, r_resist, r_dmg) = (rng.f32(), rng.f32(), rng.f32());
        let d = spell.def();
        let caster = &self.fighters[i];
        let name = self.names[i].clone();
        let chance = if scroll { 1.0 } else { magic::success_chance(&caster.stats, spell, caster.tired()) };
        if r_ok > chance {
            let at = caster.pos;
            self.fighters[i].train(d.skill(), 0.4);
            self.fx.push(Fx { kind: FxKind::Fizzle { at }, at: self.time });
            self.say(format!("{name}'s {} fizzles.", d.name.to_lowercase()));
            return;
        }
        self.fighters[i].train(d.skill(), 1.5);
        let t = self.time;
        let spell_name = d.name.to_lowercase();
        // Is what it's aimed at still there, and in reach?
        let (target, point) = match d.aim {
            Aim::Foe | Aim::Friend | Aim::Anyone => {
                let j = match (target, d.aim) {
                    (Some(j), _) => j,
                    (None, Aim::Friend) => i,
                    _ => return,
                };
                // Blind casters can only reach what's right in front of them.
                let blind = self.fighters[i].has(Does::Blind).is_some() && d.aim == Aim::Foe;
                let range = if blind { 5.0 } else { d.range };
                let gone = self.fighters[j].dead || self.fighters[j].fled;
                if self.fighters[i].pos.dist(self.fighters[j].pos) > range + 1.0 || gone {
                    self.say(format!("{name}'s {spell_name} goes wide."));
                    return;
                }
                (Some(j), self.fighters[j].pos)
            }
            Aim::Point | Aim::Door | Aim::Corpse => (target, point),
            Aim::Caster => (Some(i), self.fighters[i].pos),
        };
        if d.harmful() {
            if let Some(j) = target.filter(|&j| self.hostile(i, j)) {
                self.wake(self.fighters[j].side);
            }
        }
        // What it looks like.
        let from = self.fighters[i].pos;
        for e in d.effects {
            match (e.does, e.reach) {
                (Does::Damage(Element::Fire), Reach::Area { radius, .. }) => self.fx.push(Fx { kind: FxKind::Fireball { at: point, radius }, at: t }),
                (Does::Damage(_), Reach::Target) => self.fx.push(Fx { kind: FxKind::Bolt { from, to: point }, at: t }),
                _ => {}
            }
        }
        let area_damage = d.effects.iter().any(|e| matches!(e.reach, Reach::Area { .. }) && matches!(e.does, Does::Damage(_)));
        if area_damage {
            self.say(format!("{name}'s {spell_name} bursts."));
        } else if !d.harmful() {
            match target.filter(|&j| j != i) {
                Some(j) => {
                    let tname = self.names[j].clone();
                    self.say(format!("{name} casts {spell_name} on {tname}."));
                }
                None => self.say(format!("{name} casts {spell_name}.")),
            }
        }
        let src = Source { by: i, spell: Some(spell), target, point, skill: magic::skill_power(&self.fighters[i].stats, spell), r_dmg, r_resist };
        for e in d.effects {
            self.apply(&src, e);
        }
    }

    /// Carry out one effect from a spell or potion: find who it reaches,
    /// then do it to each of them.
    fn apply(&mut self, src: &Source, e: &Effect) {
        let secs = if let Lasts::Secs(s) = e.lasts { s as f64 } else { 30.0 };
        if let Does::Summon(kind) = e.does {
            let boost = src.spell.map(|s| 1.0 + self.fighters[src.by].power(Does::DomainPower(s.def().domain))).unwrap_or(1.0);
            self.call_up(src.by, kind, src.point, self.time + secs, e.power * src.skill * boost);
            return;
        }
        // The dead: the nearest body to the spot, or every body in the area.
        if e.does == Does::Raise {
            let (near, radius) = match e.reach {
                Reach::Area { radius, .. } => (false, radius),
                _ => (true, 4.0),
            };
            let mut bodies: Vec<usize> = (0..self.fighters.len()).filter(|&j| self.raisable(src.by, j) && self.fighters[j].pos.dist(src.point) <= radius).collect();
            bodies.sort_by(|&a, &b| self.fighters[a].pos.dist(src.point).total_cmp(&self.fighters[b].pos.dist(src.point)));
            if near {
                bodies.truncate(1);
            }
            for j in bodies {
                self.raise(src.by, j, self.time + secs);
            }
            return;
        }
        if let Reach::Ground { radius } = e.reach {
            let side = self.fighters[src.by].side;
            let owner = self.fighters[src.by].pid;
            self.zones.push(Zone { does: e.does, pos: src.point, radius, power: e.power, until: self.time + secs, side, owner, fresh: true });
            return;
        }
        let reached: Vec<(usize, f32)> = match e.reach {
            Reach::Caster => vec![(src.by, 1.0)],
            Reach::Target => src.target.map(|j| vec![(j, 1.0)]).unwrap_or_default(),
            Reach::Area { radius, who } => (0..self.fighters.len())
                .filter(|&j| !self.fighters[j].dead && !self.fighters[j].fled)
                .filter(|&j| match who {
                    Who::All => true,
                    Who::Foes => self.hostile(src.by, j),
                    Who::Friends => !self.hostile(src.by, j),
                })
                .filter_map(|j| {
                    let d = self.fighters[j].pos.dist(src.point);
                    (d <= radius).then_some((j, 1.0 - d / radius * 0.5))
                })
                .collect(),
            // A thing: whoever is aimed at, or nearest the spot (their torch).
            Reach::Object => src
                .target
                .or_else(|| (0..self.fighters.len()).filter(|&j| !self.fighters[j].dead && !self.fighters[j].fled && self.fighters[j].pos.dist(src.point) <= 3.0).min_by(|&a, &b| self.fighters[a].pos.dist(src.point).total_cmp(&self.fighters[b].pos.dist(src.point))))
                .map(|j| vec![(j, 1.0)])
                .unwrap_or_default(),
            Reach::Ground { .. } => Vec::new(),
        };
        for (j, near) in reached {
            self.affect(src, e, j, near);
        }
    }

    /// What one effect does to one fighter. `near` is 1 at the middle of an
    /// area, less towards its edge.
    fn affect(&mut self, src: &Source, e: &Effect, j: usize, near: f32) {
        let domain = src.spell.map(|s| s.def().domain);
        let boost = domain.map(|d| 1.0 + self.fighters[src.by].power(Does::DomainPower(d))).unwrap_or(1.0);
        let ward = domain.map(|d| (1.0 - self.fighters[j].power(Does::DomainResist(d))).max(0.05)).unwrap_or(1.0);
        let name = self.names[src.by].clone();
        let tname = self.names[j].clone();
        let what = src.spell.map(|s| s.def().name.to_lowercase()).unwrap_or_default();
        // Hostile spells other than plain damage can be thrown off: by will,
        // by what they wear, by a ward against the domain.
        if e.does.harmful() && !matches!(e.does, Does::Damage(_)) && self.hostile(src.by, j) && e.lasts != Lasts::Worn {
            let f = &self.fighters[j];
            let item_resist = e.does.resisted_by().map(|r| f.power(r).min(0.95)).unwrap_or(0.0);
            let resist = 1.0 - (1.0 - magic::willpower_resist(&f.stats)) * (1.0 - item_resist) * ward;
            if src.r_resist < resist {
                self.say(format!("{tname} shrugs off {name}'s {what}."));
                return;
            }
        }
        match e.lasts {
            Lasts::Worn => {}
            Lasts::Now => match e.does {
                Does::Damage(el) => {
                    let f = &self.fighters[j];
                    let best = |part: Part, k: f32| 1.0 - f.armor.iter().filter(|a| a.covers.contains(&part)).map(|a| a.blunt * k).fold(0.0, f32::max);
                    let elemental = if el.elemental() { 1.0 - f.power(Does::ResistElements).min(0.9) } else { 1.0 };
                    let resist = elemental
                        * match el {
                            // Armour helps a little against fire and frost (it's
                            // mostly heat and cold), properly against stone.
                            Element::Fire => best(Part::Torso, 0.3),
                            Element::Frost => best(Part::Torso, 0.2),
                            Element::Stone => best(Part::LeftLeg, 1.0),
                            Element::Lightning | Element::Rot => 1.0,
                        };
                    let dmg = e.power * src.skill * (0.8 + src.r_dmg * 0.4) * near * boost * ward * resist * f.warded();
                    if matches!(e.reach, Reach::Target) {
                        self.say(format!("{name}'s {what} strikes {tname} ({dmg:.0})."));
                    }
                    if el == Element::Stone {
                        self.hurt_parts(j, dmg, &[Part::LeftLeg, Part::RightLeg]);
                    } else {
                        self.hurt_whole(j, dmg);
                    }
                }
                Does::Kindle => {
                    let f = &mut self.fighters[j];
                    if f.has_torch && !f.torch {
                        f.torch = true;
                        self.say(format!("{tname}'s torch flares alight."));
                    }
                }
                Does::Daze => {
                    let t = self.time;
                    let f = &mut self.fighters[j];
                    f.act = Act::Recover { until: t + e.power as f64 };
                    f.think_at = f.think_at.max(t + e.power as f64);
                    self.say(format!("{tname} reels, dazed."));
                }
                Does::Douse => {
                    if self.fighters[j].torch {
                        self.fighters[j].torch = false;
                        self.say(format!("{tname}'s torch gutters out."));
                    }
                }
                Does::Heal => {
                    // Worst wounds first: head and torso when someone is down,
                    // otherwise whatever is most hurt.
                    self.mend_fighter(j, e.power * src.skill * (0.9 + src.r_dmg * 0.2) * boost);
                }
                Does::Drain => {
                    let dmg = e.power * src.skill * (0.8 + src.r_dmg * 0.4) * boost * ward * self.fighters[j].warded();
                    self.say(format!("{name} drains {tname} ({dmg:.0})."));
                    self.hurt_whole(j, dmg);
                    self.mend_fighter(src.by, dmg);
                }
                Does::Wither => {
                    let f = &self.fighters[j];
                    let limb = [Part::LeftArm, Part::RightArm, Part::LeftLeg, Part::RightLeg].into_iter().filter(|p| !f.missing[*p as usize]).max_by(|a, b| f.hp[*a as usize].total_cmp(&f.hp[*b as usize]));
                    if let Some(limb) = limb {
                        let dmg = e.power * src.skill * (0.8 + src.r_dmg * 0.4) * boost * ward;
                        self.say(format!("{tname}'s {} withers ({dmg:.0}).", limb.name()));
                        self.wound(j, limb, dmg);
                    }
                }
                Does::Energy => {
                    let f = &mut self.fighters[j];
                    f.mana = (f.mana + e.power).min(f.max_mana);
                }
                Does::Rest => {
                    let f = &mut self.fighters[j];
                    f.fatigue = f.max_fatigue;
                }
                Does::Stamina => {
                    let f = &mut self.fighters[j];
                    f.fatigue = (f.fatigue + e.power * src.skill).min(f.max_fatigue);
                }
                Does::Dispel => {
                    let f = &mut self.fighters[j];
                    if f.summon.is_some() {
                        f.fled = true;
                        self.say(format!("{tname} is sent back."));
                    } else if !f.statuses.is_empty() {
                        f.statuses.clear();
                        if f.side != f.home {
                            f.side = f.home;
                        }
                        self.say(format!("The spells on {tname} unravel."));
                    }
                }
                Does::Shatter => {
                    let f = &mut self.fighters[j];
                    if f.weapon != FISTS && f.weapon_name != "bare hands" {
                        let what = f.weapon_name.to_lowercase();
                        f.weapon = FISTS;
                        f.weapon_name = "bare hands";
                        f.broke.push(super::items::Slot::MainHand);
                        f.act = Act::Idle;
                        self.say(format!("{tname}'s {what} shatters!"));
                    } else if f.shield > 0.0 {
                        f.shield = 0.0;
                        f.broke.push(super::items::Slot::OffHand);
                        self.say(format!("{tname}'s shield shatters!"));
                    }
                }
                Does::Regrow => {
                    let f = &mut self.fighters[j];
                    if let Some(k) = (0..6).find(|&k| f.missing[k]) {
                        f.missing[k] = false;
                        f.hp[k] = 1.0;
                        self.say(format!("{tname}'s {} grows back!", PARTS[k].name()));
                    }
                }
                _ => {}
            },
            Lasts::Secs(secs) => {
                let until = self.time + secs as f64;
                let f = &mut self.fighters[j];
                f.statuses.retain(|s| s.does != e.does);
                f.statuses.push(Status { does: e.does, power: e.power, until });
                match e.does {
                    Does::Paralyze => {
                        f.act = Act::Idle;
                        self.say(format!("{tname} is held fast."));
                    }
                    Does::KnockDown => {
                        f.act = Act::Idle;
                        self.say(format!("{tname} is thrown to the ground."));
                    }
                    Does::Blind => self.say(format!("{tname} is blinded.")),
                    Does::Slow => self.say(format!("{tname} slows.")),
                    Does::Calm => {
                        f.target = None;
                        f.act = Act::Idle;
                        self.say(format!("{tname} lowers their guard, calm."));
                    }
                    Does::Fear => {
                        f.target = None;
                        f.act = Act::Idle;
                        self.say(format!("{tname} breaks in terror."));
                    }
                    Does::Dominate => {
                        let side = self.fighters[src.by].side;
                        let f = &mut self.fighters[j];
                        f.side = side;
                        f.target = None;
                        f.order = None;
                        f.act = Act::Idle;
                        self.say(format!("{tname} falls under {name}'s will."));
                    }
                    _ => {}
                }
            }
        }
    }

    /// Can fighter `by` raise the body of fighter `j`? Only the real dead,
    /// once, and never one of their own side: necromancy never brings back
    /// a fallen friend.
    pub fn raisable(&self, by: usize, j: usize) -> bool {
        let f = &self.fighters[j];
        // (Your squad's dead are never raised, by anyone.)
        f.dead && f.is_person() && !f.raised && f.home != self.fighters[by].home && f.home != SQUAD_SIDE
    }

    /// Raise a body as a mindless thrall on `by`'s side.
    pub fn raise(&mut self, by: usize, j: usize, until: f64) {
        let side = self.fighters[by].side;
        let t = self.time;
        let mut f = self.fighters[j].clone();
        self.fighters[j].raised = true;
        f.pid = NOBODY;
        f.side = side;
        f.home = side;
        f.dead = false;
        f.ko = false;
        f.fled = false;
        f.fleeing = false;
        for k in 0..6 {
            if !f.missing[k] {
                f.hp[k] = f.max_hp[k] * 0.6;
            }
        }
        f.statuses.clear();
        f.spells.clear();
        f.potions.clear();
        f.scrolls.clear();
        f.used.clear();
        f.held = None;
        f.act = Act::Idle;
        f.target = None;
        f.order = None;
        f.think_at = t;
        f.aware_at = 0.0;
        f.ammo = 0;
        f.torch = false;
        f.boldness = 1.0;
        f.trained = [0.0; super::stats::N_SKILLS];
        f.tire = 0.0;
        f.damage_taken = 0.0;
        f.broke.clear();
        f.summon = Some(Summoned { kind: Summon::Thrall, until, mindless: true });
        let name = format!("{} (raised)", self.names[j]);
        self.fighters.push(f);
        self.names.push(name.clone());
        self.say(format!("{name} rises, empty-eyed."));
    }

    /// Mend fighter `j` by `amount`, worst wounds first; the downed may get up.
    fn mend_fighter(&mut self, j: usize, amount: f32) {
        let mut left = amount;
        while left > 0.5 {
            let f = &self.fighters[j];
            let worst = (0..6).filter(|&k| f.hp[k] < f.max_hp[k] && !f.missing[k]).max_by(|&a, &b| {
                let need = |k: usize| (f.max_hp[k] - f.hp[k]) / f.max_hp[k] + if f.ko && k < 2 && f.hp[k] <= 0.0 { 10.0 } else { 0.0 };
                need(a).total_cmp(&need(b))
            });
            let Some(k) = worst else { break };
            let give = left.min(self.fighters[j].max_hp[k] - self.fighters[j].hp[k]).min(12.0);
            self.fighters[j].hp[k] += give;
            left -= give;
        }
        let f = &mut self.fighters[j];
        if f.ko && !f.dead && !body::knocked_out(&f.hp) {
            f.ko = false;
            f.act = Act::Idle;
            let name = self.names[j].clone();
            self.say(format!("{name} gets back up."));
        }
    }

    /// Call up creatures for the side of fighter `by`, round `at`.
    pub fn call_up(&mut self, by: usize, kind: Summon, at: V2, until: f64, power: f32) {
        let (side, race, owner) = (self.fighters[by].side, self.fighters[by].race, self.names[by].clone());
        self.call_up_for(side, race, &owner, kind, at, until, power);
    }

    /// Call up creatures for a side (named for their caster), round `at`.
    #[allow(clippy::too_many_arguments)]
    pub fn call_up_for(&mut self, side: Side, race: Race, owner: &str, kind: Summon, at: V2, until: f64, power: f32) {
        let count = creature(kind).count;
        for k in 0..count {
            let a = k as f32 / count as f32 * std::f32::consts::TAU;
            let off = if count > 1 { V2::new(a.cos(), a.sin()).scale(1.2) } else { V2::default() };
            let mut f = Fighter::creature(kind, side, at.add(off), until, power, race);
            f.think_at = self.time;
            self.fighters.push(f);
            self.names.push(format!("{owner}'s {}", kind.name().to_lowercase()));
        }
        self.say(format!("{owner} calls up a {}.", kind.name().to_lowercase()));
    }

    /// Damage spread over the whole body by where blows usually land. A big
    /// enough total spoils a spell being cast.
    pub fn hurt_whole(&mut self, j: usize, dmg: f32) {
        self.hurt_parts(j, dmg, &PARTS);
    }

    /// Damage spread over some parts, by where blows usually land.
    pub fn hurt_parts(&mut self, j: usize, dmg: f32, parts: &[Part]) {
        if dmg <= 0.0 {
            return;
        }
        if dmg > 6.0 && matches!(self.fighters[j].act, Act::Cast { .. }) {
            self.fighters[j].act = Act::Recover { until: self.time + 0.4 };
        }
        let total: f32 = parts.iter().map(|p| p.hit_weight()).sum();
        for &p in parts {
            self.wound(j, p, dmg * p.hit_weight() / total);
        }
    }

    pub fn winner(&self) -> Option<Side> {
        let mut sides: Vec<Side> = self.fighters.iter().filter(|f| f.active() && !f.fleeing && !f.is_decoy()).map(|f| f.home).collect();
        sides.sort();
        sides.dedup();
        if sides.len() == 1 {
            Some(sides[0])
        } else {
            None
        }
    }
}

/// What a called-up creature is like.
pub struct Creature {
    pub attack: &'static str,
    /// Toughness (sets health on every part) and a further multiplier on it.
    pub toughness: f32,
    pub hp: f32,
    pub strength: f32,
    pub cut: f32,
    pub blunt: f32,
    pub reach: f32,
    /// Running speed, m/s.
    pub speed: f32,
    pub skill: f32,
    pub dodge: f32,
    pub shield: f32,
    /// How many come.
    pub count: usize,
    /// For the AI's sense of how dangerous it is.
    pub might: f32,
}

/// The creatures spells call up. (Raised corpses are their own bodies.)
pub fn creature(kind: Summon) -> Creature {
    match kind {
        Summon::SpiritBeast => Creature { attack: "claws", toughness: 45.0, hp: 1.0, strength: 50.0, cut: 9.0, blunt: 3.0, reach: 1.0, speed: 5.5, skill: 50.0, dodge: 35.0, shield: 0.0, count: 1, might: 35.0 },
        Summon::Swarmling => Creature { attack: "bites", toughness: 5.0, hp: 0.35, strength: 15.0, cut: 3.0, blunt: 0.0, reach: 0.7, speed: 6.0, skill: 35.0, dodge: 40.0, shield: 0.0, count: 6, might: 8.0 },
        Summon::Decoy => Creature { attack: "nothing", toughness: 1.0, hp: 0.2, strength: 1.0, cut: 0.0, blunt: 0.0, reach: 0.0, speed: 0.0, skill: 0.0, dodge: 0.0, shield: 0.0, count: 1, might: 20.0 },
        Summon::Guardian => Creature { attack: "stone fists", toughness: 80.0, hp: 1.4, strength: 70.0, cut: 0.0, blunt: 14.0, reach: 1.6, speed: 3.0, skill: 55.0, dodge: 10.0, shield: 0.3, count: 1, might: 55.0 },
        Summon::Thrall => Creature { attack: "hands", toughness: 30.0, hp: 1.0, strength: 40.0, cut: 0.0, blunt: 5.0, reach: 0.9, speed: 3.0, skill: 30.0, dodge: 0.0, shield: 0.0, count: 1, might: 20.0 },
    }
}

/// Where an effect comes from: who, by which spell (none for a potion), aimed
/// at whom or where, how strongly their skill drives it, and the dice
/// already rolled for it.
struct Source {
    by: usize,
    spell: Option<Spell>,
    target: Option<usize>,
    point: V2,
    skill: f32,
    r_dmg: f32,
    r_resist: f32,
}

/// Only about half of misses and blocks are worth a line in the log.
fn rng_line(r: f32) -> bool {
    r < 0.5
}

/// One number for how dangerous someone is in a fight. Fights themselves are
/// always fought blow by blow; this is for decisions about them (do bandits
/// attack? does a side lose its nerve?) and for summaries. Roughly: expected damage dealt per
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
