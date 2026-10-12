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
use super::elements;
use super::items::{item, ArmorDef, ItemId, Kind, WeaponDef, FISTS};
use super::magic::{self, Aim, Spell, Status, Style};
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng::Rng;
use super::stats::{Attr, Skill, Stats};

/// Length of one combat tick, game seconds.
pub const DT: f64 = 0.1;
/// Breath back per second in a fight: standing off, and in the thick of it.
pub const FIGHT_BREATH_REST: f32 = 0.4;
pub const FIGHT_BREATH: f32 = 0.1;
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
/// Someone this far from everyone they're fighting, with nobody after them,
/// has got away, metres.
pub const ESCAPE_GAP: f32 = 45.0;
/// Seconds into a fight before anyone can be clear of it (`clear_of_it`).
pub const SETTLE: f64 = 3.0;
/// Body radius, metres: added to weapon reach.
const BODY: f32 = 0.45;
/// How far apart fighters on the same side keep, metres.
const ALLY_SPACE: f32 = 1.4;
/// Circling a target between blows, as a share of running pace.
const CIRCLE_PACE: f32 = 0.35;

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

// ---- Cuts, knocks and sneaking (the items design, 0.2 rules 1-3 and 7) ----

/// A bleed's strength: hit points a second, from the torso, for each point
/// of cut damage that got through.
pub const BLEED_PER_CUT: f32 = 0.008;
/// The most one body bleeds, hit points a second, however many cuts.
pub const BLEED_MOST: f32 = 1.2;
/// A cut this much stopped by armour opens no bleed (0.3: 60%).
pub const BLEED_STOPPED: f32 = 0.6;
/// Seconds to bind a bandage on, oneself or someone beside (a blow
/// spoils it).
pub const BIND_SECS: f64 = 4.0;
/// How near someone must be to bind them, metres (beyond body to body).
pub const BIND_REACH: f32 = 1.2;
/// The AI binds a bleed this strong or worse (hit points a second).
pub const BIND_AT: f32 = 0.08;
/// How heavy a blunt blow is against who it lands on: blunt damage that got
/// through over (`STAGGER_BASE` + their Strength × `STAGGER_STRENGTH`).
pub const STAGGER_BASE: f32 = 6.0;
pub const STAGGER_STRENGTH: f32 = 0.15;
/// Chance to stagger (a swing or a cast spoilt) and to knock down, by how
/// heavy the blow was: (weight - from) × per, capped.
pub const STAGGER: (f32, f32, f32) = (0.3, 0.5, 0.8);
pub const KNOCKDOWN: (f32, f32, f32) = (1.0, 0.35, 0.6);
/// A stagger loses them this long; on the ground this long.
pub const STAGGER_SECS: f64 = 0.6;
pub const KNOCKDOWN_SECS: f64 = 2.0;
/// Blunt damage to the head or torso counts this much more towards
/// knocking someone out, and only this much once they're out: clubs take
/// people alive (rule 3). Edged damage counts as it always has.
pub const BLUNT_TO_KO: f32 = 1.25;
/// How much of a blow's weight (for staggers and knockdowns) full padding
/// takes off, and how much of a head blow's blunt a full knockout guard
/// does (the items design, 0.3).
pub const PADDING_EASE: f32 = 0.6;
pub const HELM_KO: f32 = 0.6;
pub const BLUNT_PAST_KO: f32 = 0.4;

/// Chance a cutting blow from `w` opens a bleed (`WeaponDef::bleed`).
pub fn bleed_chance(w: &WeaponDef) -> f32 {
    if w.cut <= 0.0 {
        0.0
    } else {
        w.bleed
    }
}

/// A sneak attack's multiplier on the damage before the attacker's Sneak
/// adds to it (`WeaponDef::sneak`): the knife's is highest by far.
pub fn sneak_mult(w: &WeaponDef) -> f32 {
    if w.sneak > 0.0 {
        w.sneak
    } else {
        SNEAK_ATTACK
    }
}

/// A sneak attack with it knocks out instead of wounding (the sap).
pub fn stuns(w: &WeaponDef) -> bool {
    w.stuns
}

// ---- Room to swing, reach and the off hand (rules 4-6) -----------------------

pub use super::items::Space;

/// The room a weapon needs (bows and the like need none: they're drawn,
/// not swung).
pub fn space_of(w: &WeaponDef) -> Space {
    if w.range > 0.0 {
        Space::Short
    } else {
        w.space
    }
}

/// Swung in a wide arc that can catch two or three at once (rule 5).
pub fn sweeps(w: &WeaponDef) -> bool {
    w.sweep > 1 && w.range == 0.0
}

/// Can go in the off hand, to parry with (rule 6).
pub fn offhand_ok(w: &WeaponDef) -> bool {
    w.offhand && !w.two_handed && w.range == 0.0
}

/// Long and sweeping weapons in tight places (indoors, an alley, friends
/// pressing close): this much slower to swing and less likely to land at
/// worst. A space this narrow (metres any way, `solid::room_at`) is tight;
/// each friend within `CROWD_NEAR` adds `CROWD_EACH`.
pub const TIGHT_SLOW: f32 = 0.5;
pub const TIGHT_HIT: f32 = 0.3;
pub const TIGHT_ROOM: f32 = 2.0;
pub const CROWD_NEAR: f32 = 1.6;
pub const CROWD_EACH: f32 = 0.35;
/// A sweep catches the rest of its `WeaponDef::sweep` within reach and
/// this wide an arc (cosine of the half-angle), for this share of the blow;
/// friends too once it's this tight.
pub const SWEEP_COS: f32 = 0.5;
pub const SWEEP_SHARE: f32 = 0.6;
pub const SWEEP_ALLIES: f32 = 0.5;
/// Keeping at bay (rule 4): a weapon this much longer than the other's
/// strikes first at someone closing (its wind-up this much quicker), and a
/// hit may push them back out of their own reach; inside this share of its
/// reach a long weapon is crowded (blows this much softer, recovery this
/// much slower) until its wielder steps back.
pub const BAY_EDGE: f32 = 0.5;
pub const FIRST_STRIKE: f32 = 0.7;
pub const PUSH_CHANCE: f32 = 0.55;
pub const INSIDE: f32 = 0.55;
pub const CROWDED_HIT: f32 = 0.6;
pub const CROWDED_RECOVER: f32 = 1.4;
/// An off-hand blade's guard (a shield's is its whole block value, 0..1),
/// and the chance a parry with it comes back as a quick blow, at this share.
pub const OFFHAND_PARRY: f32 = 0.25;
pub const RIPOSTE: f32 = 0.3;
pub const RIPOSTE_SHARE: f32 = 0.7;

/// How a blow is struck: whole, or a share of one (a sweep's other foes, an
/// off-hand's quick return), with the hand weapon or another (`weapon`).
/// Only a whole blow follows through (pushes back, sweeps, is parried back).
#[derive(Clone, Copy)]
struct Blow {
    share: f32,
    weapon: Option<WeaponDef>,
    follow: bool,
    /// Thrown (`weapon` is the thing thrown): flies like a shot.
    throw: bool,
}

const WHOLE: Blow = Blow { share: 1.0, weapon: None, follow: true, throw: false };

// ---- Thrown weapons (rule 8) ---------------------------------------------------

/// How far a thrown weapon carries, metres (a net less far).
pub const THROW_RANGE: f32 = 12.0;
pub const NET_RANGE: f32 = 8.0;
/// Nobody throws at someone nearer than this (they'd rather strike).
pub const THROW_NEAREST: f32 = 3.0;
/// A thrown weapon's chance to break, times what it hit: a body, armour, a
/// shield (or a parrying blade), the ground past a miss.
pub const BREAK_ON: (f32, f32, f32, f32) = (1.0, 1.5, 2.0, 0.8);

/// How far `w` carries when thrown (0: it isn't thrown).
pub fn throw_range(w: &WeaponDef) -> f32 {
    if !w.thrown || w.range > 0.0 {
        0.0
    } else if w.entangle > 0.0 {
        NET_RANGE
    } else {
        THROW_RANGE
    }
}

/// A thrown weapon where it came down.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Thrown {
    pub item: ItemId,
    /// Who threw it (a fighter's index).
    pub by: usize,
    pub at: V2,
    /// It broke when it hit (and is gone).
    pub broke: bool,
    /// When it came down.
    pub when: f64,
    /// Thrown from the hand (rather than a spare from the pack).
    pub from_hand: bool,
}

/// What became of a blow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Landed {
    /// No blow at all (out of reach, nothing to strike with).
    Gone,
    Missed,
    /// Turned: a shield, a parry, a brace.
    Blocked,
    /// It landed; `armour` if armour met it.
    Hit { armour: bool },
}

/// Chance from (weight - from) × per, capped.
fn chance_of(weight: f32, (from, per, most): (f32, f32, f32)) -> f32 {
    ((weight - from) * per).clamp(0.0, most)
}

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
    /// Binding a bandage on someone (themselves, or a friend beside them).
    Bind { who: usize, item: ItemId, done: f64 },
    /// Throwing a weapon (from the hand, or a spare from the pack).
    Throw { target: usize, item: ItemId, lands: f64 },
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
    /// Seconds spent burning this fight, whether they were soaked, and
    /// whether they wear pitch-sealed gear (it catches).
    #[serde(default)]
    pub burned: f32,
    /// A body raised in this fight: which side raised it.
    #[serde(default)]
    pub raised_by: Option<Side>,
    #[serde(default)]
    pub soaked: bool,
    #[serde(default)]
    pub pitch: bool,
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
    /// Bandages in their pack (anything that staunches, `Does::Staunch`).
    #[serde(default)]
    pub bandages: Vec<ItemId>,
    /// What's in their belt's quick slots (quicker to use, 0.4).
    #[serde(default)]
    pub quick: Vec<ItemId>,
    /// A blade in the off hand to parry with (`offhand_ok`).
    #[serde(default)]
    pub offhand: Option<WeaponDef>,
    /// What's in the main hand (none once it's been thrown).
    #[serde(default)]
    pub weapon_item: Option<ItemId>,
    /// Throwing weapons in the pack, besides the one in the hand.
    #[serde(default)]
    pub throws: Vec<ItemId>,
    /// The best hand weapon in the pack that isn't for throwing: drawn
    /// once the one in the hand has been thrown.
    #[serde(default, with = "super::save::named_weapon")]
    pub backup: Option<(WeaponDef, &'static str)>,
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
    /// Each armour piece's padding and knockout guard (`items::wears`), in
    /// the order of `armor` (the items design, 0.3).
    #[serde(default)]
    pub armor_pad: Vec<[f32; 2]>,
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
        let armor_pad: Vec<[f32; 2]> = armor_slots.iter().map(|&s| gear.in_slot(s).map(super::items::wears).map(|w| [w.padding, w.ko_guard]).unwrap_or([0.0; 2])).collect();
        // A bulky pack costs dodging too, the fuller the more (0.4).
        let dodge_penalty = armor.iter().map(|a| a.dodge_penalty).sum::<f32>() + gear.bulk() * BULK_DODGE;
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
            base_speed: p.race.walk_speed() * 2.6 * stats.move_factor() * (1.0 + speed_bonus) * inventory::encumbrance_factor(load) * gear.pack_pace() * gear.wear_pace(stats.attr(Attr::Strength)),
            spells,
            boldness: p.traits.boldness,
            might: p.might,
            stats,
            act: Act::Idle,
            target: None,
            order: None,
            // Still bleeding from before (a squad member not yet bound).
            statuses: bleeding_in(p),
            ko: false,
            dead: false,
            fleeing: false,
            fled: false,
            think_at: 0.0,
            aware_at: 0.0,
            burdened: false,
            torch: false,
            has_torch: false,
            burned: 0.0,
            raised_by: None,
            soaked: false,
            pitch: gear.pieces_sealed(),
            missing: p.wounds.missing,
            ammo,
            shots: 0,
            sidearm: if held.range > 0.0 { sidearm } else { None },
            stowed: None,
            potions: gear.bag.iter().filter(|e| item(e.0).kind == Kind::Potion).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            scrolls: gear.bag.iter().filter(|e| matches!(item(e.0).kind, Kind::Scroll(_))).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            used: Vec::new(),
            bandages: gear.bag.iter().filter(|e| is_bandage(e.0)).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            quick: gear.quick(),
            offhand: gear.in_slot(super::items::Slot::OffHand).and_then(|id| item(id).weapon().copied()).filter(|w| offhand_ok(w) && !held.two_handed),
            weapon_item: gear.in_slot(super::items::Slot::MainHand),
            throws: gear.bag.iter().filter(|e| item(e.0).weapon().is_some_and(|w| throw_range(w) > 0.0)).flat_map(|e| std::iter::repeat(e.0).take(e.1 as usize)).collect(),
            backup: gear.bag.iter().filter_map(|e| item(e.0).weapon().filter(|w| w.range == 0.0 && !w.thrown).map(|w| (*w, item(e.0).name))).max_by(|a, b| (a.0.cut + a.0.blunt).total_cmp(&(b.0.cut + b.0.blunt))),
            trained: [0.0; super::stats::N_SKILLS],
            tire: 0.0,
            held: None,
            damage_taken: 0.0,
            summon: None,
            raised: false,
            broke: Vec::new(),
            armor_wear: vec![[0.0; 2]; armor_slots.len()],
            armor_slots: armor_slots,
            armor_pad,
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
        let weapon = WeaponDef { cut: c.cut * power, blunt: c.blunt * power, reach: c.reach, windup: 0.5, recover: 0.45, bleed: if c.cut > 0.0 { 0.3 } else { 0.0 }, ..FISTS };
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
            burned: 0.0,
            raised_by: None,
            soaked: false,
            pitch: false,
            missing: [false; 6],
            ammo: 0,
            shots: 0,
            sidearm: None,
            stowed: None,
            potions: Vec::new(),
            scrolls: Vec::new(),
            used: Vec::new(),
            bandages: Vec::new(),
            quick: Vec::new(),
            offhand: None,
            weapon_item: None,
            throws: Vec::new(),
            backup: None,
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
            armor_pad: Vec::new(),
            weapon_wear: 0.0,
            shield_wear: 0.0,
        }
    }

    /// How hard they're bleeding, hit points a second (0 if not).
    pub fn bleeding(&self) -> f32 {
        self.has(Does::Bleed).map(|s| s.power).unwrap_or(0.0)
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
        self.paralyzed() || self.has(Does::KnockDown).is_some() || self.has(Does::Frozen).is_some()
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

    /// The off-hand blade in use: a working left arm, no shield, nothing
    /// two-handed in the other.
    pub fn offhand_up(&self) -> Option<WeaponDef> {
        let w = self.offhand?;
        (self.hp[Part::LeftArm as usize] > 0.0 && self.shield_up() <= 0.0 && !self.weapon.two_handed && !self.burdened).then_some(w)
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
    /// Fighter `who` rocked by a heavy blow (a stagger, a knockdown, or
    /// knocked out by a blunt one).
    Jolt { who: usize, down: bool },
    /// A blow on someone who never saw it coming, at `at`.
    Sneak { at: V2 },
    /// Fighter `who` turned a blow with their off-hand blade.
    Parry { who: usize },
    /// Fighter `who` driven back by a long weapon, from `from` to `to`.
    Push { who: usize, from: V2, to: V2 },
    /// A weapon thrown from `from`, coming down at `to` (`hit`: on the one
    /// it was thrown at); `broke`: in pieces when it lands.
    Throw { item: ItemId, from: V2, to: V2, hit: bool, broke: bool },
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
    /// Where the fight started: the middle of everyone in it at the first
    /// tick. Those chasing someone off measure how far they've come from
    /// their own people against it when none of them are still fighting
    /// (`ai::anchor`).
    #[serde(default)]
    pub ground: Option<V2>,
    /// What's solid round the fight, for fighters to go round (`Around`).
    #[serde(skip)]
    pub around: Around,
    /// Thrown weapons where they came down (rule 8).
    #[serde(default)]
    pub thrown: Vec<Thrown>,
}

/// What's solid round a fight (`solid.rs`): fighters step round it, never
/// through it. Not saved: the world hands it over before a fight is advanced
/// (`World::around_fights`, and before a far fight's result is run), the
/// same lists for the copy near the squad as for the result, so both are
/// fought on the same ground (rules 1 and 7). None: open ground (a duel,
/// tests).
#[derive(Clone, Default)]
pub struct Around(pub Option<std::sync::Arc<Vec<std::sync::Arc<super::solid::Solids>>>>);

impl std::fmt::Debug for Around {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // (The same either way, so a fight reads the same saved and loaded.)
        f.write_str("Around")
    }
}

impl Around {
    /// Where someone at `a` stepping for `b` ends up (`walk::slide`).
    fn step(&self, a: V2, b: V2) -> V2 {
        match &self.0 {
            Some(sets) => super::walk::slide(sets, a, b, super::solid::BODY),
            None => b,
        }
    }

    /// Where someone at `a` making for `goal` is after `d` metres: straight
    /// at it if nothing solid stands between, else along the way round
    /// (`walk::find`, the same finder everyone walks by).
    fn toward(&self, a: V2, goal: V2, d: f32) -> V2 {
        let to = goal.sub(a);
        let straight = if to.len() <= d { goal } else { a.add(to.scale(d / to.len())) };
        let Some(sets) = &self.0 else { return straight };
        let body = super::solid::BODY;
        if sets.iter().all(|s| s.crossing(a, goal, body).is_none()) {
            return straight;
        }
        let (from, end) = (super::solid::free_spot_in(sets, a, body), super::solid::free_spot_in(sets, goal, body));
        let way = super::walk::find(sets, from, end, body).way;
        super::walk::along(a, &way, d)
    }
}

impl Battle {
    pub fn new(id: u32, seed: u64, start: f64, fighters: Vec<Fighter>, names: Vec<String>) -> Battle {
        Battle { id, seed, start, time: start, ticks: 0, fighters, over: false, log: Vec::new(), fx: Vec::new(), names, lights: None, zones: Vec::new(), ground: None, around: Around::default(), thrown: Vec::new() }
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
        if self.ground.is_none() {
            let at: Vec<V2> = self.fighters.iter().filter(|f| f.active()).map(|f| f.pos).collect();
            if !at.is_empty() {
                self.ground = Some(at.iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / at.len() as f32));
            }
        }

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

        // Whoever is on fire burns (and is counted, for what they carry).
        for j in 0..n {
            let f = &self.fighters[j];
            if f.dead || f.fled {
                continue;
            }
            let Some(burn) = f.has(Does::Burning).map(|s| s.power) else { continue };
            let dmg = burn * DT as f32 * f.warded();
            self.fighters[j].burned += DT as f32;
            self.hurt_whole(j, dmg);
        }

        // Whoever is bleeding loses blood from the torso, down or not, until
        // they're bound or healed (rule 1): left long enough, it kills.
        for j in 0..n {
            let f = &self.fighters[j];
            if f.dead || f.fled {
                continue;
            }
            let bleed = f.bleeding();
            if bleed > 0.0 {
                self.lose_blood(j, bleed * DT as f32);
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
                // Breath comes back slowly in a fight (B5): a few minutes'
                // standing off to get it all back.
                f.fatigue = (f.fatigue + (if resting { FIGHT_BREATH_REST } else { FIGHT_BREATH }) * DT as f32).min(f.max_fatigue);
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
                        self.sweep(i, target);
                        let crowded = if self.crowded(i, target) { CROWDED_RECOVER } else { 1.0 };
                        let rec = self.fighters[i].weapon.recover * self.fighters[i].attack_time() * crowded;
                        self.fighters[i].act = Act::Recover { until: t + rec as f64 };
                    }
                }
                Act::Recover { until } => {
                    if t >= until {
                        self.fighters[i].act = Act::Idle;
                    } else {
                        self.circle(i);
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
                Act::Bind { who, item: it, done } => {
                    if t >= done {
                        self.bind(i, who, it);
                        self.fighters[i].act = Act::Idle;
                    }
                }
                Act::Throw { target, item: it, lands } => {
                    if t >= lands {
                        self.throw_at(i, target, it, &mut rng);
                        let rec = item(it).weapon().map(|w| w.recover).unwrap_or(0.5) * self.fighters[i].attack_time();
                        self.fighters[i].act = Act::Recover { until: t + rec as f64 };
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
        // People running away don't keep a fight going. An animal that has
        // turned tail is still in it until it gets clear (`idle`): a hunt
        // isn't over the instant the prey starts to run (BL-24).
        // (A fighter turned by a spell still counts for the side they came in on.)
        // Nor does someone nobody is after any more, well clear of everyone
        // on the other side (P23: they've been let go).
        // (Only asked when the fight would otherwise go on: a side left alone
        // on the field holds it, as ever.)
        let standing = |f: &Fighter| f.active() && (!f.fleeing || f.home == super::animals::ANIMAL_SIDE) && !f.is_decoy();
        let mut plain: Vec<Side> = self.fighters.iter().filter(|f| standing(f)).map(|f| f.home).collect();
        plain.sort();
        plain.dedup();
        let clear: Vec<bool> = (0..self.fighters.len()).map(|i| plain.len() > 1 && self.clear_of_it(i)).collect();
        let mut sides: Vec<Side> = self.fighters.iter().enumerate().filter(|(i, f)| f.active() && (!f.fleeing || f.home == super::animals::ANIMAL_SIDE) && !f.is_decoy() && !clear[*i]).map(|(_, f)| f.home).collect();
        sides.sort();
        sides.dedup();
        if sides.len() <= 1 || t - self.start > MAX_LENGTH {
            // Those let go while walking off have gone; those who let them go
            // and stood are still there (they hold the field, they didn't run).
            for (i, f) in self.fighters.iter_mut().enumerate() {
                if f.active() && (f.fleeing || (clear[i] && matches!(f.order, Some(Order::MoveTo(_))))) {
                    f.fled = true;
                }
            }
            self.over = true;
            let line = match sides.first() {
                Some(s) if sides.len() == 1 => format!("The fight is over. {} side holds the field.", if *s == SQUAD_SIDE { "Your" } else { "The other" }),
                _ => "The fight breaks off.".to_string(),
            };
            self.say(line);
        }
    }

    /// Out of the fight without running: on their feet, aware, after nobody
    /// (or walking off under orders), nobody after them, and more than
    /// `ESCAPE_GAP` from everyone on the other side still in it. Not in the
    /// first few seconds, before anyone has picked a foe, and not with beasts
    /// in it.
    pub fn clear_of_it(&self, i: usize) -> bool {
        let f = &self.fighters[i];
        if !f.active() || f.fleeing || f.is_decoy() || f.home == super::animals::ANIMAL_SIDE || f.unaware(self.time) || self.time - self.start < SETTLE {
            return false;
        }
        if f.target.is_some_and(|j| self.fighters.get(j).is_some_and(|o| o.active())) {
            return false;
        }
        // (Not against beasts: a hunt goes on while the quarry is still in
        // it, and a beast that gives up runs by its own nerve, BL-24.)
        if self.fighters.iter().enumerate().any(|(j, o)| self.hostile(i, j) && o.active() && o.home == super::animals::ANIMAL_SIDE) {
            return false;
        }
        !(0..self.fighters.len()).any(|j| {
            let o = &self.fighters[j];
            self.hostile(i, j) && o.active() && !o.fleeing && !o.is_decoy() && (o.target == Some(i) || o.pos.dist(f.pos) <= ESCAPE_GAP)
        })
    }

    /// Move fighter `i` toward `to`, round anything solid on the way
    /// (`Around`), unless that would take them into a sanctuary held against
    /// them.
    fn step_to(&mut self, i: usize, to: V2) {
        let me = &self.fighters[i];
        let barred = self.zones.iter().any(|z| z.does == Does::Sanctuary && z.until > self.time && z.side != me.side && to.dist(z.pos) < z.radius && me.pos.dist(z.pos) >= z.radius);
        if !barred {
            self.fighters[i].pos = self.around.step(me.pos, to);
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
                    if d.len() > ESCAPE_GAP {
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
                let to = self.around.toward(me.pos, p, speed.min(d.len()));
                self.step_to(i, to);
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
        let tpos = them.pos;
        // A long weapon with a foe inside it: step back to where it works
        // (rule 4), unless there's nowhere to step.
        if self.crowded(i, j) {
            let from = self.fighters[i].pos;
            let back = from.sub(d.scale(speed * 0.7 / dist.max(0.01)));
            self.step_to(i, back);
            if self.fighters[i].pos.dist(from) > speed * 0.3 {
                return;
            }
        }
        let me = &self.fighters[i];
        // Out of reach but within a throw: throw something (rule 8).
        if dist > reach * 1.2 && dist >= THROW_NEAREST && !me.shooting() && me.fatigue > 4.0 {
            if let Some(it) = self.to_throw(i).filter(|&it| item(it).weapon().is_some_and(|w| dist <= throw_range(w))) {
                let quick = if me.quick.contains(&it) { QUICK_THROW } else { 1.0 };
                let windup = item(it).weapon().map(|w| w.windup).unwrap_or(0.5) * me.attack_time() * quick;
                self.fighters[i].act = Act::Throw { target: j, item: it, lands: self.time + windup as f64 };
                return;
            }
        }
        if dist > reach * 0.92 {
            let step = speed.min(dist - reach * 0.85).max(0.0);
            let to = self.around.toward(me.pos, tpos, step);
            self.step_to(i, to);
        } else if me.usable_weapon().is_some() && me.fatigue > 4.0 {
            let mut windup = me.weapon.windup * me.attack_time();
            // Meeting someone coming in, a longer weapon strikes first.
            if !me.shooting() && self.keeps_at_bay(i, j) {
                windup *= FIRST_STRIKE;
            }
            // Long and sweeping weapons are slow in tight places.
            if self.needs_room(i) {
                windup *= 1.0 + TIGHT_SLOW * self.tightness(i);
            }
            self.fighters[i].act = Act::Swing { target: j, lands: self.time + windup as f64 };
        }
    }

    /// Between blows, a fighter with a hand weapon steps round their target
    /// at striking distance (each turning their own way), so a fight spreads
    /// into rings rather than a pile.
    fn circle(&mut self, i: usize) {
        let me = &self.fighters[i];
        let Some(j) = me.target else { return };
        if me.shooting() || me.burdened || me.order.is_some_and(|o| matches!(o, Order::MoveTo(_))) || !self.fighters[j].active() {
            return;
        }
        let them = self.fighters[j].pos;
        let d = me.pos.sub(them);
        let dist = d.len();
        let reach = me.attack_range();
        if dist < 0.05 || dist > reach * 1.3 {
            return;
        }
        let way = if (me.pid as usize + i).is_multiple_of(2) { 1.0 } else { -1.0 };
        let step = me.speed() * DT as f32 * CIRCLE_PACE;
        let turn = way * step / dist.max(0.5);
        let (s, c) = turn.sin_cos();
        let keep = reach * 0.85;
        let rotated = V2::new(d.x * c - d.y * s, d.x * s + d.y * c).scale(keep / dist);
        self.step_to(i, them.add(rotated));
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
                // Friends keep a little more room, so a fight reads as
                // pairs and rings, not a crowd.
                let min = if self.fighters[i].side == self.fighters[j].side { ALLY_SPACE } else { BODY * 2.0 };
                if l < min {
                    let push = if l < 1e-4 { V2::new(0.05 * (j as f32 - i as f32), 0.03) } else { d.scale((min - l) * 0.5 / l) };
                    // (Not into a wall or a table.)
                    let (a, b) = (self.fighters[i].pos, self.fighters[j].pos);
                    self.fighters[i].pos = self.around.step(a, a.sub(push));
                    self.fighters[j].pos = self.around.step(b, b.add(push));
                }
            }
        }
    }

    // ---- Blows ----------------------------------------------------------

    fn resolve_attack(&mut self, a: usize, d: usize, rng: &mut Rng) {
        self.resolve_blow(a, d, rng, WHOLE);
    }

    fn resolve_blow(&mut self, a: usize, d: usize, rng: &mut Rng, blow: Blow) -> Landed {
        // Every draw happens whatever the outcome, so one swing missing never
        // shifts the dice for everyone after it.
        let (r_hit, r_block, r_part, r_dmg, r_cover1, r_cover2) = (rng.f32(), rng.f32(), rng.f32(), rng.f32(), rng.f32(), rng.f32());
        let att = &self.fighters[a];
        let def = &self.fighters[d];
        let held = if blow.throw { Some((FISTS, 1.0)) } else { att.usable_weapon() };
        let Some((weapon, arm)) = held else { return Landed::Gone };
        let (weapon, arm) = blow.weapon.map(|w| (w, 1.0)).unwrap_or((weapon, arm));
        let shot = (att.shooting() && blow.weapon.is_none()) || blow.throw;
        let dist = att.pos.dist(def.pos);
        if !def.active() || dist > if shot { weapon.range * 1.05 } else { att.reach() * 1.3 } {
            return Landed::Gone;
        }
        // A bow without arrows (or swung up close with no hand weapon) is a club, and a poor one.
        let weapon = if weapon.range > 0.0 && !shot { FISTS } else { weapon };
        if shot && !blow.throw {
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
        // Long and sweeping weapons in tight places land less often (rule 5).
        let tight = if !shot && needs_room(&weapon) { 1.0 - TIGHT_HIT * self.tightness(a) } else { 1.0 };
        let p_hit = (0.5 + (atk - dodge) * 0.012).clamp(0.08, 0.95) * (1.0 - blinded * 0.7) * far * dark * tight;
        let (an, dn) = (self.names[a].clone(), self.names[d].clone());
        if shot && !blow.throw {
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
            return Landed::Missed;
        }
        // A brace turns the blow that would have landed, then it's spent.
        if self.fighters[d].has(Does::Brace).is_some() {
            self.fighters[d].statuses.retain(|s| s.does != Does::Brace);
            self.say(format!("{dn}'s brace turns {an}'s blow."));
            return Landed::Blocked;
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
        // An off-hand blade turns blows too, less well than a shield (rule 6).
        let parrying = !helpless && !def.burdened && !shot && def.offhand_up().is_some();
        let guard = guard + if parrying { OFFHAND_PARRY * 100.0 } else { 0.0 };
        let p_block = if guard <= 0.0 { 0.0 } else { (guard * (0.3 + def.skill(Skill::Block) / 100.0) / (guard + atk + 20.0)).clamp(0.0, 0.75) };
        let roll = 0.8 + r_dmg * 0.4;
        let skill_mult = 0.6 + skill * 0.006;
        // Strength puts weight behind a blow (and a throw); a bowstring
        // doesn't care.
        let pull = if shot && !blow.throw { 0.0 } else { 1.0 };
        let size = if shot { 1.0 } else { self.fighters[a].size() };
        // A long weapon with its foe inside its reach hits soft (rule 4).
        let crowded = if blow.weapon.is_none() && self.crowded(a, d) { CROWDED_HIT } else { 1.0 };
        let mut cut = weapon.cut * (1.0 + strength * 0.006 * pull) * skill_mult * roll * size * blow.share * crowded;
        let mut blunt = weapon.blunt * (1.0 + strength * 0.010 * pull) * skill_mult * roll * size * blow.share * crowded;
        // The weapon in the hand slot wears with every blow that connects
        // (bowstrings less). An archer's pack sidearm doesn't count here.
        if self.fighters[a].stowed.is_none() && !blow.throw {
            self.fighters[a].weapon_wear += if shot { 0.3 } else { 1.0 };
        }

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
            // Turned with the off-hand blade: a flash of steel, and now and
            // then a quick blow straight back.
            if parrying && self.fighters[d].shield <= 0.0 {
                self.fx.push(Fx { kind: FxKind::Parry { who: d }, at: self.time });
                let mut back = Rng::from_keys(&[self.seed, self.ticks, d as u64, a as u64, 0x5249_504F]);
                if blow.follow && back.f32() < RIPOSTE && self.fighters[d].active() {
                    let off = self.fighters[d].offhand_up();
                    self.resolve_blow(d, a, &mut back, Blow { share: RIPOSTE_SHARE, weapon: off, follow: false, throw: false });
                }
            }
            return Landed::Blocked;
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
        let skin = ArmorDef { covers: &PARTS, coverage: 1.0, cut: skin, blunt: skin * 0.6, dodge_penalty: 0.0, conceal: 1.0 };
        // (Indices into the armour worn; None for the skin.)
        let mut layers: Vec<(Option<usize>, ArmorDef)> = self.fighters[d].armor.iter().enumerate().filter(|(_, a)| a.covers.contains(&part)).map(|(i, a)| (Some(i), *a)).collect();
        if skin.cut > 0.0 {
            layers.push((None, skin));
        }
        // A blow meets what's worn outermost first: the cloak, then the over
        // layer (and helm, gloves, boots), then the under layer, then the skin
        // (0.3, rule 2).
        let slots = &self.fighters[d].armor_slots;
        let rank = |w: &Option<usize>| match w.and_then(|i| slots.get(i).copied()) {
            None => 3,
            Some(super::items::Slot::Cloak) => 0,
            Some(super::items::Slot::BodyUnder | super::items::Slot::LegsUnder) => 2,
            Some(_) => 1,
        };
        layers.sort_by_key(|x| rank(&x.0));
        // Rusted armour stops less (skin doesn't rust); a heavy point gets
        // through some of it.
        let rust = 1.0 - self.fighters[d].power(Does::Rust).min(1.0);
        let through = 1.0 - weapon.pierce.clamp(0.0, 0.9);
        let cut_swung = cut;
        // Frozen stiff, even a light blow cracks brittle things.
        let heavy = blunt > cut || self.fighters[d].has(Does::Frozen).is_some();
        let mut met = false;
        // The most padding and the best guard among the layers that met it.
        let (mut pad, mut guard) = (0.0f32, 0.0f32);
        for (k, (which, layer)) in layers.iter().enumerate() {
            // (Each layer its own roll; past the second, drawn from the
            // second's, so the stream is as it was: rule 2.)
            let r = match k {
                0 => r_cover1,
                1 => r_cover2,
                _ => (r_cover2 * (7.31 * k as f32 + 1.0)).fract(),
            };
            let worn = if which.is_none() { 1.0 } else { rust * through };
            if r < layer.coverage {
                met |= which.is_some();
                if let Some(p) = which.and_then(|i| self.fighters[d].armor_pad.get(i)) {
                    pad = pad.max(p[0]);
                    guard = guard.max(p[1]);
                }
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
        // (Its own dice, keyed to this blow, so the stream everyone else
        // draws from is the same as ever: rule 2.)
        let mut dice = Rng::from_keys(&[self.seed, self.ticks, a as u64, d as u64, 0x424C_4F57]);
        let (r_bleed, r_stagger, r_down, r_push) = (dice.f32(), dice.f32(), dice.f32(), dice.f32());
        // A cut the armour mostly stopped opens no bleed (0.3).
        let stopped = cut_swung > 0.0 && cut < cut_swung * (1.0 - BLEED_STOPPED);
        let mut cut = cut * shield;
        let mut blunt = blunt * shield;
        self.fighters[a].train(weapon.skill, 1.0);
        if unaware {
            self.fx.push(Fx { kind: FxKind::Sneak { at: self.fighters[d].pos }, at: self.time });
            self.fighters[a].train(Skill::Sneak, 3.0);
            // A sap from behind: out cold, not hurt (rule 7), unless a helm
            // turns it (0.3, rule 4: a strong blow is needed then).
            let helmed = part == Part::Head && (r_bleed * 13.7).fract() < guard;
            if stuns(&weapon) && part.vital() && !helmed {
                self.say(format!("{an} catches {dn} unawares and knocks them senseless."));
                let head = self.fighters[d].hp[Part::Head as usize].max(0.0);
                self.wound(d, Part::Head, head + 0.5);
                self.fx.push(Fx { kind: FxKind::Jolt { who: d, down: true }, at: self.time });
                return Landed::Hit { armour: met };
            }
            // A sneak attack: the better the sneak, the worse the wound, and
            // a knife worst of all.
            let sneak = self.fighters[a].skill(Skill::Sneak);
            let k = sneak_mult(&weapon) + sneak / 50.0;
            cut *= k;
            blunt *= k;
            self.say(format!("{an} catches {dn} unawares: the {} ({:.0}).", part.name(), cut + blunt));
        } else if blow.throw {
            self.say(format!("{an}'s {} strikes {dn} in the {} ({:.0}).", self.names_thing(weapon), part.name(), cut + blunt));
        } else if shot {
            self.say(format!("{an} shoots {dn} in the {} ({:.0}).", part.name(), cut + blunt));
        } else {
            self.say(format!("{an} hits {dn} in the {} ({:.0}).", part.name(), cut + blunt));
        }
        // A helm takes the knockout out of a blunt blow to the head (0.3).
        let blunt = if part == Part::Head { blunt * (1.0 - HELM_KO * guard) } else { blunt };
        self.wound_blow(d, part, cut, blunt);
        // A cut the armour didn't mostly stop opens a bleed (rule 1).
        if cut > 0.5 && !stopped && r_bleed < bleed_chance(&weapon) && !self.fighters[d].dead && self.fighters[d].summon.is_none_or(|s| !s.mindless) {
            self.open_bleed(d, cut * BLEED_PER_CUT);
        }
        // A long weapon meeting someone on their way in can drive them back
        // out of their own reach (rule 4).
        if blow.follow && !shot && self.keeps_at_bay(a, d) && r_push < PUSH_CHANCE {
            self.push_back(a, d);
        }
        // A heavy blunt blow rocks them: a swing or a cast spoilt, or off
        // their feet (rule 2).
        let f = &self.fighters[d];
        if blunt > 0.0 && f.active() && !f.helpless() {
            // Padding takes the edge off (0.3, rule 4).
            let weight = blunt * (1.0 - PADDING_EASE * pad) / (STAGGER_BASE + f.attr(Attr::Strength) * STAGGER_STRENGTH);
            if r_down < chance_of(weight, KNOCKDOWN) {
                self.knock_down(d, KNOCKDOWN_SECS);
            } else if r_stagger < chance_of(weight, STAGGER) {
                self.stagger(d);
            }
        }
        // A net tangles them up for a few seconds.
        if weapon.entangle > 0.0 && (r_dmg * 7.31).fract() < weapon.entangle && !self.fighters[d].dead {
            let until = self.time + ENTANGLE_SECS;
            self.fighters[d].statuses.push(super::magic::Status { does: Does::Slow, power: 0.75, until });
            self.say(format!("{an}'s net tangles {dn}."));
        }
        Landed::Hit { armour: met }
    }

    /// A word for a weapon thrown ("hatchet").
    fn names_thing(&self, w: WeaponDef) -> &'static str {
        if w.entangle > 0.0 {
            "net"
        } else {
            "throw"
        }
    }

    /// What fighter `i` would throw now: a spare from the pack first, else
    /// what's in the hand if there's something else to draw after.
    fn to_throw(&self, i: usize) -> Option<ItemId> {
        let f = &self.fighters[i];
        if f.burdened || f.usable_weapon().is_none_or(|w| w.1 < 1.0) {
            return None;
        }
        if let Some(&it) = f.throws.first() {
            return Some(it);
        }
        let it = f.weapon_item?;
        let w = item(it).weapon()?;
        (throw_range(w) > 0.0 && f.backup.is_some()).then_some(it)
    }

    /// Fighter `a` throws `it` at `d`: it flies, lands where it stops, and
    /// may break on what it hit (rule 8).
    fn throw_at(&mut self, a: usize, d: usize, it: ItemId, rng: &mut Rng) {
        let Some(w) = item(it).weapon().copied() else { return };
        if d >= self.fighters.len() || !self.fighters[a].active() {
            return;
        }
        // Out of the pack, or out of the hand (and the next weapon drawn).
        let f = &mut self.fighters[a];
        if let Some(q) = f.quick.iter().position(|&x| x == it) {
            f.quick.remove(q);
        }
        let from_hand = if let Some(k) = f.throws.iter().position(|&x| x == it) {
            f.throws.remove(k);
            false
        } else if f.weapon_item == Some(it) {
            let (next, name) = f.backup.take().unwrap_or((FISTS, "bare hands"));
            f.weapon = next;
            f.weapon_name = name;
            f.weapon_item = None;
            true
        } else {
            return;
        };
        let (from, aim) = (self.fighters[a].pos, self.fighters[d].pos);
        let thrown = WeaponDef { skill: Skill::Marksman, range: throw_range(&w), ..w };
        let landed = self.resolve_blow(a, d, rng, Blow { share: 1.0, weapon: Some(thrown), follow: false, throw: true });
        let mut dice = Rng::from_keys(&[self.seed, self.ticks, a as u64, d as u64, 0x5448_5257]);
        let (r_break, r_far, r_side) = (dice.f32(), dice.f32(), dice.f32());
        let dir = aim.sub(from);
        let dir = dir.scale(1.0 / dir.len().max(0.01));
        let side = V2::new(-dir.y, dir.x);
        let (at, on) = match landed {
            Landed::Hit { armour } => (aim.add(side.scale((r_side - 0.5) * 0.8)), if armour { BREAK_ON.1 } else { BREAK_ON.0 }),
            Landed::Blocked => (aim.sub(dir.scale(0.6 + r_far)).add(side.scale((r_side - 0.5) * 1.5)), BREAK_ON.2),
            Landed::Missed | Landed::Gone => (aim.add(dir.scale(2.0 + r_far * 5.0)).add(side.scale((r_side - 0.5) * 3.0)), BREAK_ON.3),
        };
        let at = self.around.step(aim, at);
        let broke = r_break < (w.breaks * on).min(1.0);
        let hit = matches!(landed, Landed::Hit { .. });
        let flight = (from.dist(at) / THROWN_SPEED) as f64;
        self.fx.push(Fx { kind: FxKind::Throw { item: it, from, to: at, hit, broke }, at: self.time });
        self.thrown.push(Thrown { item: it, by: a, at, broke, when: self.time + flight, from_hand });
        if broke {
            let name = self.names[a].clone();
            self.say(format!("{name}'s {} breaks.", item(it).name.to_lowercase()));
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
        // A hit spoils whatever spell was being cast, and any bandage being
        // bound (by them, or on them).
        if matches!(f.act, Act::Cast { .. }) && dmg > 6.0 {
            f.act = Act::Recover { until: self.time + 0.4 };
        }
        if matches!(f.act, Act::Bind { .. }) {
            f.act = Act::Recover { until: self.time + 0.3 };
        }
        for o in &mut self.fighters {
            if matches!(o.act, Act::Bind { who, .. } if who == d) {
                o.act = Act::Recover { until: self.time + 0.3 };
            }
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

    /// How tight it is round fighter `i`, 0 (open ground) to 1: indoors, a
    /// narrow way between walls (`solid::room_at`), friends pressing close.
    pub fn tightness(&self, i: usize) -> f32 {
        let me = &self.fighters[i];
        let friends = (0..self.fighters.len()).filter(|&j| j != i && !self.hostile(i, j) && self.fighters[j].active() && self.fighters[j].pos.dist(me.pos) <= CROWD_NEAR).count();
        let crowd = friends as f32 * CROWD_EACH;
        let walls = match &self.around.0 {
            Some(sets) => {
                let indoors = super::solid::enclosure(sets, me.pos).is_some_and(|t| t.what == super::solid::What::Building);
                if indoors {
                    1.0
                } else {
                    ((TIGHT_ROOM - super::solid::room_at(sets, me.pos, super::solid::BODY)) / TIGHT_ROOM).clamp(0.0, 1.0)
                }
            }
            None => 0.0,
        };
        crowd.max(walls).min(1.0)
    }

    /// Fighter `i` holds a long or sweeping weapon (they suffer in tight
    /// places; short and middling ones don't).
    fn needs_room(&self, i: usize) -> bool {
        let f = &self.fighters[i];
        !f.shooting() && f.usable_weapon().is_some_and(|w| needs_room(&w.0))
    }

    /// Does `a`'s weapon outreach `d`'s, with `d` coming in but not yet
    /// near enough to strike (rule 4)?
    fn keeps_at_bay(&self, a: usize, d: usize) -> bool {
        let (f, o) = (&self.fighters[a], &self.fighters[d]);
        let Some((w, _)) = f.usable_weapon() else { return false };
        if space_of(&w) != Space::Long || f.shooting() || !o.active() || o.helpless() {
            return false;
        }
        let dist = f.pos.dist(o.pos);
        o.reach() + BAY_EDGE <= f.reach() && dist > o.reach() * 0.9 && o.target == Some(a)
    }

    /// Fighter `a`'s long weapon has `d` inside it.
    fn crowded(&self, a: usize, d: usize) -> bool {
        let (f, o) = (&self.fighters[a], &self.fighters[d]);
        f.usable_weapon().is_some_and(|w| space_of(&w.0) == Space::Long) && !f.shooting() && o.active() && f.pos.dist(o.pos) < f.reach() * INSIDE
    }

    /// `a` drives `d` back to just beyond `d`'s own reach.
    fn push_back(&mut self, a: usize, d: usize) {
        let (from, at) = (self.fighters[d].pos, self.fighters[a].pos);
        let dir = from.sub(at);
        let len = dir.len().max(0.01);
        let out = (self.fighters[d].reach() + 0.4).max(len + 0.4);
        let to = self.around.step(from, at.add(dir.scale(out / len)));
        self.fighters[d].pos = to;
        if matches!(self.fighters[d].act, Act::Swing { .. }) {
            self.fighters[d].act = Act::Recover { until: self.time + 0.3 };
        }
        let (an, dn) = (self.names[a].clone(), self.names[d].clone());
        self.fx.push(Fx { kind: FxKind::Push { who: d, from, to }, at: self.time });
        self.say(format!("{an} keeps {dn} at bay."));
    }

    /// A sweeping weapon's swing catches others in its arc as well as the
    /// one aimed at: foes, and friends too once it's tight (rule 5).
    fn sweep(&mut self, a: usize, target: usize) {
        let f = &self.fighters[a];
        let Some((w, _)) = f.usable_weapon() else { return };
        if !sweeps(&w) || f.shooting() || !f.active() || target >= self.fighters.len() {
            return;
        }
        let dir = self.fighters[target].pos.sub(f.pos);
        if dir.len() < 0.01 {
            return;
        }
        let dir = dir.scale(1.0 / dir.len());
        let allies_too = self.tightness(a) >= SWEEP_ALLIES;
        let reach = f.reach();
        let mut caught: Vec<(usize, f32)> = (0..self.fighters.len())
            .filter(|&j| j != a && j != target && self.fighters[j].active() && !self.fighters[j].is_decoy() && (self.hostile(a, j) || allies_too))
            .filter_map(|j| {
                let off = self.fighters[j].pos.sub(f.pos);
                let d = off.len();
                (d <= reach && d > 0.01 && (off.x * dir.x + off.y * dir.y) / d >= SWEEP_COS).then_some((j, d))
            })
            .collect();
        caught.sort_by(|x, y| x.1.total_cmp(&y.1).then(x.0.cmp(&y.0)));
        for (j, _) in caught.into_iter().take(w.sweep.saturating_sub(1) as usize) {
            let mut r = Rng::from_keys(&[self.seed, self.ticks, a as u64, j as u64, 0x5357_4545]);
            self.resolve_blow(a, j, &mut r, Blow { share: SWEEP_SHARE, weapon: None, follow: false, throw: false });
        }
    }

    /// A blow of `cut` and `blunt` (after armour) on a part. On the head and
    /// torso blunt counts more towards knocking out and less past it, so
    /// clubs take people alive (rule 3); edges count as ever.
    pub fn wound_blow(&mut self, d: usize, part: Part, cut: f32, blunt: f32) {
        let total = cut + blunt;
        if total <= 0.0 {
            return;
        }
        if !part.vital() || blunt <= 0.0 {
            self.wound(d, part, total);
            return;
        }
        let was_down = self.fighters[d].ko;
        let left = self.fighters[d].hp[part as usize].max(0.0);
        let to_line = cut + blunt * BLUNT_TO_KO;
        let dmg = if to_line <= left {
            to_line
        } else {
            // What's left of the blow past the line counts its blunt for less.
            left + (cut + blunt * BLUNT_PAST_KO) * (1.0 - left / to_line)
        };
        self.wound(d, part, dmg);
        let f = &self.fighters[d];
        if !was_down && f.ko && !f.dead {
            self.fx.push(Fx { kind: FxKind::Jolt { who: d, down: true }, at: self.time });
        }
    }

    /// Lose blood from the torso (a bleed): no one's woken or calmed out of
    /// anything by it, but it can put them down, and kill them.
    fn lose_blood(&mut self, d: usize, dmg: f32) {
        let torso = Part::Torso as usize;
        let was_ko = self.fighters[d].ko;
        self.fighters[d].hp[torso] -= dmg;
        let name = self.names[d].clone();
        let stats = self.fighters[d].stats.clone();
        if body::dead(&self.fighters[d].hp, &stats) {
            self.fighters[d].dead = true;
            self.fighters[d].ko = true;
            self.say(format!("{name} bleeds to death."));
        } else if body::knocked_out(&self.fighters[d].hp) && !was_ko {
            self.fighters[d].ko = true;
            self.fighters[d].act = Act::Idle;
            self.say(format!("{name} collapses from loss of blood."));
        }
    }

    /// A cut opens (or worsens) a bleed of `power` hit points a second.
    pub fn open_bleed(&mut self, d: usize, power: f32) {
        let f = &mut self.fighters[d];
        let was = f.bleeding();
        let now = (was + power).min(BLEED_MOST);
        f.statuses.retain(|s| s.does != Does::Bleed);
        f.statuses.push(Status { does: Does::Bleed, power: now, until: f64::INFINITY });
        if was <= 0.0 {
            let name = self.names[d].clone();
            self.say(format!("{name} is bleeding."));
        }
    }

    /// Every bleed on them stops (a bandage, or healing).
    pub fn staunch(&mut self, d: usize) -> bool {
        let f = &mut self.fighters[d];
        let had = f.bleeding() > 0.0;
        f.statuses.retain(|s| s.does != Does::Bleed);
        had
    }

    /// Rocked by a blow: whatever they were swinging or casting is spoilt.
    fn stagger(&mut self, d: usize) {
        let t = self.time;
        let f = &mut self.fighters[d];
        if matches!(f.act, Act::Swing { .. } | Act::Cast { .. } | Act::Bind { .. } | Act::Throw { .. }) {
            f.act = Act::Recover { until: t + STAGGER_SECS };
        }
        f.think_at = f.think_at.max(t + STAGGER_SECS);
        let name = self.names[d].clone();
        self.fx.push(Fx { kind: FxKind::Jolt { who: d, down: false }, at: t });
        self.say(format!("{name} staggers."));
    }

    /// Off their feet: no dodging or blocking, and getting up takes `secs`.
    pub fn knock_down(&mut self, d: usize, secs: f64) {
        let until = self.time + secs;
        let f = &mut self.fighters[d];
        f.statuses.retain(|s| s.does != Does::KnockDown);
        f.statuses.push(Status { does: Does::KnockDown, power: 1.0, until });
        f.act = Act::Idle;
        let name = self.names[d].clone();
        self.fx.push(Fx { kind: FxKind::Jolt { who: d, down: true }, at: self.time });
        self.say(format!("{name} is knocked off their feet."));
    }

    /// Start binding a bandage on `who` (themselves, or someone beside).
    pub fn begin_bind(&mut self, i: usize, who: usize) -> bool {
        if self.busy_with_hands(i) || !self.fighters[who].is_person() || self.fighters[who].bleeding() <= 0.0 || self.fighters[who].dead {
            return false;
        }
        if i != who && self.fighters[i].pos.dist(self.fighters[who].pos) > BIND_REACH + BODY * 2.0 {
            return false;
        }
        let Some(&it) = self.fighters[i].bandages.first() else { return false };
        let t = self.time;
        let f = &mut self.fighters[i];
        let quick = if f.quick.contains(&it) { QUICK_USE } else { 1.0 };
        f.act = Act::Bind { who, item: it, done: t + BIND_SECS * (f.attack_time().max(0.8) * quick) as f64 };
        true
    }

    fn bind(&mut self, i: usize, who: usize, it: ItemId) {
        let f = &self.fighters[i];
        let Some(k) = f.bandages.iter().position(|&b| b == it) else { return };
        if who != i && f.pos.dist(self.fighters[who].pos) > BIND_REACH + BODY * 2.0 + 0.5 {
            return;
        }
        let f = &mut self.fighters[i];
        f.bandages.remove(k);
        if let Some(q) = f.quick.iter().position(|&x| x == it) {
            f.quick.remove(q);
        }
        f.used.push(it);
        let name = self.names[i].clone();
        let line = if who == i { format!("{name} binds their wounds.") } else { format!("{name} binds {}'s wounds.", self.names[who]) };
        let src = Source { by: i, spell: None, target: Some(who), point: self.fighters[who].pos, skill: 1.0, r_dmg: 0.5, r_resist: 1.0 };
        for e in item(it).effects {
            self.apply(&src, e);
        }
        self.say(line);
    }

    // ---- Spells -----------------------------------------------------------

    /// Start casting. Energy is spent now, whether or not it works. Felt
    /// spells go off at once (this tick) and tire the caster a little;
    /// structured ones take a second or two, and a solid hit spoils them.
    /// Rituals can't be cast mid-fight at all.
    pub fn begin_cast(&mut self, i: usize, spell: Spell, target: Option<usize>, point: V2) -> bool {
        let d = spell.def();
        if self.busy_with_hands(i) {
            return false;
        }
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
        if self.busy_with_hands(i) {
            return false;
        }
        let f = &mut self.fighters[i];
        let Some(k) = f.scrolls.iter().position(|&s| matches!(item(s).kind, Kind::Scroll(x) if x == key)) else { return false };
        let it = f.scrolls.remove(k);
        f.used.push(it);
        let quick = if from_quick(f, it) { QUICK_USE } else { 1.0 };
        f.act = Act::Cast { spell, target, point, done: self.time + 0.6 * quick as f64, scroll: true };
        let name = self.names[i].clone();
        self.say(format!("{name} reads a scroll of {}.", spell.def().name.to_lowercase()));
        true
    }

    /// Mid-way through a cast or a drink: a second one waits rather than
    /// throwing the first away (the draught, the energy, the scroll).
    fn busy_with_hands(&self, i: usize) -> bool {
        match self.fighters[i].act {
            Act::Cast { done, .. } | Act::Drink { done, .. } | Act::Bind { done, .. } => done > self.time,
            _ => false,
        }
    }

    /// Start drinking a potion from the pack.
    pub fn begin_drink(&mut self, i: usize, it: ItemId) -> bool {
        if self.busy_with_hands(i) {
            return false;
        }
        let f = &mut self.fighters[i];
        let Some(k) = f.potions.iter().position(|&p| p == it) else { return false };
        f.potions.remove(k);
        f.used.push(it);
        let quick = if from_quick(f, it) { QUICK_USE } else { 1.0 };
        f.act = Act::Drink { item: it, done: self.time + DRINK_SECS * quick as f64 };
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

    /// Soak someone until `until`: any fire on them goes out.
    fn soak(&mut self, j: usize, until: f64) {
        let f = &mut self.fighters[j];
        let was_burning = f.has(Does::Burning).is_some();
        let until = f.has(Does::Wet).map(|s| s.until.max(until)).unwrap_or(until);
        f.statuses.retain(|s| !matches!(s.does, Does::Wet | Does::Burning));
        f.statuses.push(Status { does: Does::Wet, power: 1.0, until });
        f.soaked = true;
        if was_burning {
            let tname = self.names[j].clone();
            self.say(format!("The flames on {tname} hiss out."));
        }
    }

    /// Fire, cold or lightning meets whatever's already on someone: returns
    /// the hurt that lands, and leaves them burning, chilled or frozen
    /// (`elements.rs`). `resist` is the share their wards let through.
    fn element_hits(&mut self, j: usize, el: Element, dmg: f32, resist: f32) -> f32 {
        let t = self.time;
        let tname = self.names[j].clone();
        let f = &mut self.fighters[j];
        let wet = f.has(Does::Wet).is_some();
        match el {
            Element::Fire => {
                if wet {
                    f.statuses.retain(|s| s.does != Does::Wet);
                    self.say(format!("Steam bursts from {tname}."));
                    return dmg * elements::STEAM;
                }
                // Fire thaws the cold out of them, and sets them alight.
                f.statuses.retain(|s| !matches!(s.does, Does::Chilled | Does::Frozen | Does::Burning));
                f.statuses.push(Status { does: Does::Burning, power: elements::BURN_PER_SEC * resist, until: t + elements::BURN_SECS });
                let pitch = f.pitch;
                self.say(format!("{tname} catches fire!"));
                if pitch {
                    dmg * elements::PITCH_FIRE
                } else {
                    dmg
                }
            }
            Element::Frost => {
                let had_fire = f.has(Does::Burning).is_some();
                f.statuses.retain(|s| s.does != Does::Burning);
                let gain = dmg * if wet { elements::WET_CHILL } else { 1.0 };
                let chill = f.power(Does::Chilled) + gain;
                f.statuses.retain(|s| s.does != Does::Chilled);
                if had_fire {
                    self.say(format!("The flames on {tname} die in the cold."));
                }
                let f = &mut self.fighters[j];
                if chill >= elements::FREEZE_AT {
                    f.statuses.push(Status { does: Does::Frozen, power: 1.0, until: t + elements::FROZEN_SECS });
                    f.act = Act::Idle;
                    self.say(format!("{tname} freezes solid!"));
                } else {
                    f.statuses.push(Status { does: Does::Chilled, power: chill, until: t + elements::CHILL_SECS });
                }
                if wet {
                    dmg * elements::WET_COLD
                } else {
                    dmg
                }
            }
            Element::Lightning if wet => dmg * elements::WET_SHOCK,
            _ => dmg,
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
        if e.does.harmful() && !matches!(e.does, Does::Damage(_) | Does::Wet | Does::Burning | Does::Chilled | Does::Frozen) && self.hostile(src.by, j) && e.lasts != Lasts::Worn {
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
                    let dmg = self.element_hits(j, el, dmg, elemental * ward);
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
                    if f.has_torch && !f.torch && f.has(Does::Wet).is_none() {
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
                    self.soak(j, self.time + elements::WET_SECS);
                }
                Does::Heal => {
                    // Worst wounds first: head and torso when someone is down,
                    // otherwise whatever is most hurt. Healing closes cuts too.
                    if self.staunch(j) {
                        self.say(format!("{tname} stops bleeding."));
                    }
                    self.mend_fighter(j, e.power * src.skill * (0.9 + src.r_dmg * 0.2) * boost);
                }
                Does::Staunch => {
                    self.staunch(j);
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
                    Does::Wet => {
                        self.soak(j, until);
                    }
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
        self.fighters[j].raised_by = Some(side);
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

/// How fast a thrown weapon flies, metres a second (drawing, and when it
/// comes down).
pub const THROWN_SPEED: f32 = 18.0;

/// Long and sweeping weapons need room (rule 5); short and middling don't.
pub fn needs_room(w: &WeaponDef) -> bool {
    matches!(space_of(w), Space::Long | Space::Sweep) || sweeps(w)
}

/// Drinking a draught from the pack takes this long, seconds; from a quick
/// slot on the belt this share of it (and binding a bandage, or reading a
/// scroll); a throwing weapon from the belt winds up this share quicker.
pub const DRINK_SECS: f64 = 1.2;
pub const QUICK_USE: f32 = 0.5;
pub const QUICK_THROW: f32 = 0.8;
/// Each point of a pack's bulk (0.4) is this much off dodging.
pub const BULK_DODGE: f32 = 0.2;

/// Was `it` in their quick slots? (Then it's used from there.)
fn from_quick(f: &mut Fighter, it: ItemId) -> bool {
    match f.quick.iter().position(|&x| x == it) {
        Some(q) => {
            f.quick.remove(q);
            true
        }
        None => false,
    }
}

/// Anything that stops bleeding when used (a bandage): found by what it
/// does, never by name.
pub fn is_bandage(it: ItemId) -> bool {
    let d = item(it);
    !matches!(d.kind, Kind::Potion) && d.effects.iter().any(|e| e.does == Does::Staunch && e.lasts == Lasts::Now)
}

/// A squad member still bleeding from before (their wounds, `Wounds::bleed`,
/// per hour) comes into a fight with the bleed on them, per second.
fn bleeding_in(p: &Person) -> Vec<Status> {
    if p.wounds.bleed > 0.0 {
        vec![Status { does: Does::Bleed, power: p.wounds.bleed / 3600.0, until: f64::INFINITY }]
    } else {
        Vec::new()
    }
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
    let off = gear.in_slot(super::items::Slot::OffHand).and_then(|id| item(id).weapon().copied()).filter(|o| offhand_ok(o) && !w.two_handed);
    let guard = gear.shield() * 0.6 + w.parry * 0.3 + if off.is_some() { OFFHAND_PARRY * 0.6 } else { 0.0 };
    let ehp = vital / (1.0 - reduction.min(0.8)) * (1.0 + dodge) * (1.0 + guard);

    let magic = if spells.is_empty() {
        0.0
    } else {
        s.max_mana(race) * (s.best_magic() / 100.0) * 0.25 * (spells.len() as f32 / 6.0).sqrt()
    };
    (offense * ehp).sqrt() * 0.9 + magic
}
