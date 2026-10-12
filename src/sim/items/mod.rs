//! Every kind of thing that can be carried, worn or wielded.
//!
//! Placeholders, but with real numbers behind them. Weapons do two kinds of
//! damage, Kenshi-style: **cut** (edges and points — deadly against flesh,
//! stopped well by good armour) and **blunt** (weight — gets through armour,
//! wears you down). Armour covers some body parts, catches a share of the blows
//! that land there, and takes a share off each kind of damage. Enchanted pieces
//! carry effects that change their wearer's numbers, Morrowind-style.
//!
//! The lines of data live in one file per kind (`weapons.rs`, `armour.rs`,
//! `clothing.rs`, `tools.rs`, `ingredients.rs`, `trinkets.rs`, `magic.rs`,
//! `food.rs`, `goods.rs`), each with its items, its made forms and its grid sizes; this
//! file holds what they share and puts them together (`ITEMS`, `FORMS`).
//! A new item is a line in its kind's file (Laz's items design,
//! `docs/items-design.md`).

use serde::{Deserialize, Serialize};

use super::body::Part;
use super::effects::{now, worn, Does, Effect, Reach};
use super::materials::{Craft, Grade, Material, GRADES};
use super::stats::{Attr, Skill};

pub type ItemId = u16;

mod armour;
mod clothing;
mod food;
pub mod goods;
mod ingredients;
mod magic;
mod tools;
mod trinkets;
mod weapons;

/// Where a thing is worn or held. Body and legs take two layers, an under
/// and an over (a shirt under a coat); a pack goes on the back and a cloak
/// over it. Rings go on either hand: an item says `Ring`, and `Ring2` is the
/// second one (`Gear::equip` fills whichever is free).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    MainHand,
    OffHand,
    Head,
    BodyUnder,
    BodyOver,
    Hands,
    LegsUnder,
    LegsOver,
    Feet,
    Cloak,
    Back,
    Belt,
    Brooch,
    Neck,
    Ring,
    Ring2,
}

pub const SLOTS: [Slot; 16] = [
    Slot::MainHand,
    Slot::OffHand,
    Slot::Head,
    Slot::BodyUnder,
    Slot::BodyOver,
    Slot::Hands,
    Slot::LegsUnder,
    Slot::LegsOver,
    Slot::Feet,
    Slot::Cloak,
    Slot::Back,
    Slot::Belt,
    Slot::Brooch,
    Slot::Neck,
    Slot::Ring,
    Slot::Ring2,
];

impl Slot {
    pub fn name(self) -> &'static str {
        match self {
            Slot::MainHand => "Main hand",
            Slot::OffHand => "Off hand",
            Slot::Head => "Head",
            Slot::BodyUnder => "Body, under",
            Slot::BodyOver => "Body, over",
            Slot::Hands => "Hands",
            Slot::LegsUnder => "Legs, under",
            Slot::LegsOver => "Legs, over",
            Slot::Feet => "Feet",
            Slot::Cloak => "Cloak",
            Slot::Back => "Back",
            Slot::Belt => "Belt",
            Slot::Brooch => "Brooch",
            Slot::Neck => "Neck",
            Slot::Ring => "Ring",
            Slot::Ring2 => "Ring",
        }
    }

    /// Worn for show and enchantment rather than to fight or carry in.
    pub fn accessory(self) -> bool {
        matches!(self, Slot::Belt | Slot::Brooch | Slot::Neck | Slot::Ring | Slot::Ring2)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct WeaponDef {
    pub skill: Skill,
    pub cut: f32,
    pub blunt: f32,
    /// Metres.
    pub reach: f32,
    /// Seconds from starting a swing to the blow landing.
    pub windup: f32,
    /// Seconds after a blow before the next can start.
    pub recover: f32,
    pub two_handed: bool,
    /// How well it parries, 0..1 (a shield does this better).
    pub parry: f32,
    /// Metres it shoots, for bows and the like (0 = a hand weapon).
    pub range: f32,
    /// What it shoots (an item key), for ranged weapons.
    #[serde(with = "super::save::opt_name")]
    pub ammo: Option<super::save::Name>,
    /// Share of the target's armour it gets through (Forgeiron's weight and
    /// point; Edgeglass has none).
    #[serde(default)]
    pub pierce: f32,
    /// Chance a hit tangles the target up for a few seconds (nets).
    #[serde(default)]
    pub entangle: f32,
    // ---- The fight rules of the items design, 0.2 (`combat.rs`) ----
    /// Chance a cutting hit opens a bleed, 0..1.
    #[serde(default)]
    pub bleed: f32,
    /// The room it needs to be swung well (tight places hurt `Long`).
    #[serde(default)]
    pub space: Space,
    /// How many it can catch in one swing (0 or 1: just the one aimed at).
    #[serde(default)]
    pub sweep: u8,
    /// A sneak attack's multiplier (0: the usual, `combat::SNEAK_ATTACK`).
    #[serde(default)]
    pub sneak: f32,
    /// A sneak attack with it knocks out instead of wounding (the sap).
    #[serde(default)]
    pub stuns: bool,
    /// How easily it's hidden on the body, 0..1 (`combat::notices`).
    #[serde(default)]
    pub conceal: f32,
    /// Can be thrown (it flies and lands where it stops).
    #[serde(default)]
    pub thrown: bool,
    /// Chance a thrown one breaks when it hits something, 0..1.
    #[serde(default)]
    pub breaks: f32,
    /// Can go in the off hand, to parry with.
    #[serde(default)]
    pub offhand: bool,
    /// The strength it wants (Part 1, B1): below it, slower and less sure.
    /// Read by the fight rules once they weigh it (the duel grid's step).
    #[serde(default)]
    pub strength: f32,
}

/// How much room a weapon needs to be swung well (the items design, 0.2
/// rule 5): long and sweeping weapons suffer indoors, in alleys and crowds.
/// Long ones are the reach weapons that keep foes at bay (rule 4);
/// sweeping ones are swung wide (the maul, the greatsword).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Space {
    Short,
    #[default]
    Medium,
    Long,
    Sweep,
}

/// The fight-rule numbers of a weapon line, laid over it with `fits`.
#[derive(Clone, Copy)]
pub(super) struct Fits {
    pub bleed: f32,
    pub space: Space,
    pub sweep: u8,
    pub sneak: f32,
    pub stuns: bool,
    pub conceal: f32,
    pub thrown: bool,
    pub breaks: f32,
    pub offhand: bool,
    pub strength: f32,
}

pub(super) const FIT: Fits = Fits { bleed: 0.0, space: Space::Medium, sweep: 0, sneak: 0.0, stuns: false, conceal: 0.0, thrown: false, breaks: 0.0, offhand: false, strength: 0.0 };

/// A weapon line with its fight-rule numbers.
pub(super) const fn fits(mut d: ItemDef, f: Fits) -> ItemDef {
    if let Kind::Weapon(mut w) = d.kind {
        w.bleed = f.bleed;
        w.space = f.space;
        w.sweep = f.sweep;
        w.sneak = f.sneak;
        w.stuns = f.stuns;
        w.conceal = f.conceal;
        w.thrown = f.thrown;
        w.breaks = f.breaks;
        w.offhand = f.offhand;
        w.strength = f.strength;
        d.kind = Kind::Weapon(w);
    }
    d
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct ArmorDef {
    #[serde(with = "super::save::covers")]
    pub covers: &'static [Part],
    /// Chance a blow on a covered part meets the armour.
    pub coverage: f32,
    /// Share of cut damage stopped.
    pub cut: f32,
    /// Share of blunt damage stopped.
    pub blunt: f32,
    /// Taken off the chance to dodge.
    pub dodge_penalty: f32,
    /// How well it's hidden when worn, 0..1: an under layer, gloves and
    /// boots go unseen; a helm or a coat of plates is plain to see.
    #[serde(default)]
    pub conceal: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Weapon(WeaponDef),
    Armor(ArmorDef),
    /// Off-hand blocking, 0..1.
    Shield(f32),
    /// Extra capacity, kg.
    Pack(f32),
    Trinket,
    /// Used for a job, not worn: lockpicks, tools of a trade.
    Tool,
    /// Something to make things from.
    Material,
    /// Drunk: its effects (`ItemDef::effects`) work on whoever drinks it.
    Potion,
    /// Read aloud: casts the spell with this key once, with no energy and no
    /// chance of failing.
    Scroll(&'static str),
    /// Notes on a structured spell (by key): read them to learn it.
    Notes(&'static str),
    /// A rare text on a ritual (by key): read it to learn it.
    Text(&'static str),
    /// Shot from a ranged weapon; stacks in the pack.
    Ammo,
    /// Money.
    Coin,
    /// Eaten: takes this much off hunger (0–100).
    Food(f32),
    /// Carried for someone else (a letter to deliver).
    Errand,
    /// A Ṭaḍoro manual on a craft: read it to take the craft up.
    Manual(Skill),
    /// Held in the off hand and lit: burns this many hours (see `torch`).
    Torch(f32),
    /// Set in the ground and lit: burns this many hours.
    StandingTorch(f32),
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    pub key: &'static str,
    pub name: &'static str,
    pub slot: Slot,
    pub kind: Kind,
    /// Kilograms.
    pub weight: f32,
    /// What it would fetch, in coin; also how NPC kit is budgeted.
    pub value: f32,
    /// What it does: while worn (enchantments), or when drunk (potions).
    pub effects: &'static [Effect],
}

const ARMS: &[Part] = &[Part::LeftArm, Part::RightArm];
const LEGS: &[Part] = &[Part::LeftLeg, Part::RightLeg];
const BODY: &[Part] = &[Part::Torso, Part::LeftArm, Part::RightArm];
const TORSO: &[Part] = &[Part::Torso];
const HEAD: &[Part] = &[Part::Head];

const fn weapon(key: &'static str, name: &'static str, skill: Skill, cut: f32, blunt: f32, reach: f32, windup: f32, recover: f32, two_handed: bool, parry: f32, weight: f32, value: f32) -> ItemDef {
    ItemDef {
        key,
        name,
        slot: Slot::MainHand,
        kind: Kind::Weapon(WeaponDef { skill, cut, blunt, reach, windup, recover, two_handed, parry, ..FISTS }),
        weight,
        value,
        effects: &[],
    }
}

#[allow(clippy::too_many_arguments)]
const fn armor(key: &'static str, name: &'static str, slot: Slot, covers: &'static [Part], coverage: f32, cut: f32, blunt: f32, dodge_penalty: f32, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot, kind: Kind::Armor(ArmorDef { covers, coverage, cut, blunt, dodge_penalty, conceal: worn_unseen(slot) }), weight, value, effects: &[] }
}

/// How well a worn piece is hidden, by where it's worn (the line can say
/// otherwise): under layers, gloves and boots go unseen, a helm never.
const fn worn_unseen(slot: Slot) -> f32 {
    match slot {
        Slot::BodyUnder | Slot::LegsUnder => 0.75,
        Slot::Hands | Slot::Feet => 0.4,
        Slot::BodyOver | Slot::LegsOver | Slot::Cloak => 0.1,
        _ => 0.0,
    }
}

const fn material(key: &'static str, name: &'static str, weight: f32, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Material, weight, value, effects: &[] }
}

/// A bow or the like: `windup` is drawing and aiming, `recover` reloading.
#[allow(clippy::too_many_arguments)]
const fn ranged(key: &'static str, name: &'static str, cut: f32, blunt: f32, range: f32, windup: f32, recover: f32, ammo: &'static str, weight: f32, value: f32) -> ItemDef {
    ItemDef {
        key,
        name,
        slot: Slot::MainHand,
        kind: Kind::Weapon(WeaponDef { skill: Skill::Marksman, cut, blunt, reach: 1.0, windup, recover, two_handed: true, range, ammo: Some(ammo), space: Space::Short, ..FISTS }),
        weight,
        value,
        effects: &[],
    }
}

const fn food(key: &'static str, name: &'static str, weight: f32, value: f32, nourishment: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Food(nourishment), weight, value, effects: &[] }
}

const fn scroll(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Scroll(spell), weight: 0.05, value, effects: &[] }
}

const fn notes(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Notes(spell), weight: 0.1, value, effects: &[] }
}

const fn text(key: &'static str, name: &'static str, spell: &'static str, value: f32) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Text(spell), weight: 0.5, value, effects: &[] }
}

const fn manual(key: &'static str, name: &'static str, skill: Skill) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Manual(skill), weight: 0.6, value: 120.0, effects: &[] }
}

const fn potion(key: &'static str, name: &'static str, value: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Potion, weight: 0.3, value, effects }
}

const fn trinket(key: &'static str, name: &'static str, slot: Slot, weight: f32, value: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot, kind: Kind::Trinket, weight, value, effects }
}

/// The whole catalogue, in the order of the files that hold it. An `ItemId`
/// is an index into this list (and the made versions after it, `Catalogue`).
pub static ITEMS: std::sync::LazyLock<Vec<ItemDef>> = std::sync::LazyLock::new(|| {
    [weapons::ITEMS, armour::ITEMS, clothing::ITEMS, tools::ITEMS, ingredients::ITEMS, trinkets::ITEMS, magic::ITEMS, food::ITEMS, goods::ITEMS].concat()
});

/// Can this be worn or held (rather than used up or crafted with)?
pub fn equippable(id: ItemId) -> bool {
    matches!(item(id).kind, Kind::Weapon(_) | Kind::Armor(_) | Kind::Shield(_) | Kind::Pack(_) | Kind::Trinket | Kind::Torch(_))
}

pub fn item(id: ItemId) -> &'static ItemDef {
    &catalogue().defs[id as usize]
}

/// Look an item up by its key. Panics on a typo, which is what tests want.
/// Is there an item by this key?
pub fn catalogue_has(key: &str) -> bool {
    catalogue().by_key.contains_key(key)
}

pub fn id(key: &str) -> ItemId {
    *catalogue().by_key.get(key).unwrap_or_else(|| panic!("no item called {key}"))
}

/// What a catalogue entry is made of: its form (the plain item it's a version
/// of), main and second material, and grade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemInfo {
    pub form: ItemId,
    pub main: Material,
    pub second: Material,
    pub grade: Grade,
}

pub fn info(id: ItemId) -> &'static ItemInfo {
    &catalogue().info[id as usize]
}

/// The version of `form` in these materials at this grade, if there is one.
pub fn variant(form: ItemId, main: Material, second: Material, grade: Grade) -> Option<ItemId> {
    catalogue().variants.get(&(form, main, second, grade)).copied()
}

/// How many blows it takes before it's worn out (0: it doesn't wear).
pub fn max_durability(id: ItemId) -> f32 {
    let i = info(id);
    if i.main == Material::None || !equippable(id) || matches!(item(id).kind, Kind::Trinket | Kind::Torch(_)) {
        return 0.0;
    }
    let d = i.main.def().durability;
    let d = if i.second != Material::None { (d + i.second.def().durability) * 0.5 * 1.2 } else { d };
    d * i.grade.durability()
}

/// Can it be mended at all?
pub fn repairable(id: ItemId) -> bool {
    let i = info(id);
    i.main.def().repairable && (i.second == Material::None || i.second.def().repairable)
}

/// The craft that mends it (the main material's).
pub fn craft_of(id: ItemId) -> Craft {
    info(id).main.def().craft
}

/// What a worn piece costs and gives beyond the blows it stops (Part 2 of
/// the items design, rules 4 to 7). Data for the rules to read; each rule
/// reads only its own number.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Wears {
    /// Takes the edge off stagger, knockdown and knockout from blunt blows
    /// to the parts it covers, 0..1.
    pub padding: f32,
    /// A helm's guard against being knocked out by a blow to the head, 0..1.
    pub ko_guard: f32,
    /// Sneaking made harder, 0..1 (metal and stone loudest; soft shoes below 0).
    pub noise: f32,
    /// Spotting and seeing at night made harder, 0..1 (full helms).
    pub sight: f32,
    /// Against cold, -1..1 (furs warm; metal chills in the cold).
    pub warmth: f32,
    /// Keeps the wet out, 0..1.
    pub dry: f32,
    /// Gets heavy when soaked (cloth and padding).
    pub soaks: bool,
    /// Slows in a fight and on the road, a share (set when a leg is written).
    pub slow: f32,
    /// Hides what's carried and worn beneath it from a look or a search, 0..1.
    pub hides: f32,
}

const NONE: Wears = Wears { padding: 0.0, ko_guard: 0.0, noise: 0.0, sight: 0.0, warmth: 0.0, dry: 0.0, soaks: false, slow: 0.0, hides: 0.0 };

static WEARS: std::sync::LazyLock<Vec<(&'static str, Wears)>> = std::sync::LazyLock::new(|| [armour::WEARS, clothing::WEARS].concat());

/// What wearing this costs and gives beyond protection (by its form).
pub fn wears(id: ItemId) -> Wears {
    let key = item(info(id).form).key;
    WEARS.iter().find(|w| w.0 == key).map(|w| w.1).unwrap_or(NONE)
}

/// Grid cells by form key, from the files above.
static SIZES: std::sync::LazyLock<Vec<(&'static str, u8, u8)>> = std::sync::LazyLock::new(|| {
    [weapons::SIZES, armour::SIZES, clothing::SIZES, tools::SIZES, ingredients::SIZES, trinkets::SIZES, magic::SIZES, food::SIZES, goods::SIZES].concat()
});

/// How many cells it takes in a pack's grid, (wide, tall), standing upright
/// (it can be turned). Its form's size from the tables, or one by its kind
/// and where it's worn. Every version of a form is the same size.
pub fn footprint(id: ItemId) -> (u8, u8) {
    let form = item(info(id).form);
    if let Some(&(_, w, h)) = SIZES.iter().find(|s| s.0 == form.key) {
        return (w, h);
    }
    match form.kind {
        Kind::Armor(_) => match form.slot {
            Slot::BodyUnder | Slot::BodyOver | Slot::LegsUnder | Slot::LegsOver | Slot::Cloak => (2, 3),
            Slot::Head | Slot::Feet => (2, 2),
            Slot::Hands => (2, 1),
            _ => (1, 1),
        },
        Kind::Weapon(w) => (1, if w.two_handed { 5 } else { 3 }),
        Kind::Shield(_) => (2, 2),
        Kind::Pack(_) => (3, 3),
        Kind::Text(_) | Kind::Manual(_) => (1, 2),
        _ => (1, 1),
    }
}

pub fn catalogue() -> &'static Catalogue {
    static CAT: std::sync::OnceLock<Catalogue> = std::sync::OnceLock::new();
    CAT.get_or_init(Catalogue::build)
}

/// Every item there is: the plain table above, then every version of each
/// form in each material and grade, worked out once.
pub struct Catalogue {
    pub defs: Vec<ItemDef>,
    pub info: Vec<ItemInfo>,
    by_key: std::collections::HashMap<&'static str, ItemId>,
    variants: std::collections::HashMap<(ItemId, Material, Material, Grade), ItemId>,
}

/// A form: what it's usually made of, and what else it can be made of
/// (main materials, and backing materials it can be made on).
#[derive(Clone, Copy)]
pub struct Form {
    pub key: &'static str,
    pub noun: &'static str,
    pub usual: Material,
    pub mains: &'static [Material],
    pub seconds: &'static [Material],
}

use Material as M;

const fn f(key: &'static str, noun: &'static str, usual: Material, mains: &'static [Material], seconds: &'static [Material]) -> Form {
    Form { key, noun, usual, mains, seconds }
}

/// Each made form and its materials (from the files above). Shared basics
/// are in plain iron, wood, leather and cloth; the traditions' materials are
/// the step up. A second material backs the main one (an Edgeglass edge on a
/// Forgeiron spine, Nacre scales on a Slatewing frame); any listed pairing
/// can turn up.
pub static FORMS: std::sync::LazyLock<Vec<Form>> = std::sync::LazyLock::new(|| {
    [weapons::FORMS, armour::FORMS, clothing::FORMS, tools::FORMS, ingredients::FORMS, trinkets::FORMS, magic::FORMS, food::FORMS, goods::FORMS].concat()
});

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl Catalogue {
    fn build() -> Catalogue {
        let mut defs: Vec<ItemDef> = ITEMS.to_vec();
        let mut info: Vec<ItemInfo> = (0..ITEMS.len())
            .map(|i| {
                let main = FORMS.iter().find(|f| f.key == ITEMS[i].key).map(|f| f.usual).unwrap_or(M::None);
                ItemInfo { form: i as ItemId, main, second: M::None, grade: Grade::Common }
            })
            .collect();
        let mut variants = std::collections::HashMap::new();
        for form in FORMS.iter() {
            let fid = ITEMS.iter().position(|d| d.key == form.key).unwrap_or_else(|| panic!("form {} has no item", form.key)) as ItemId;
            let base = ITEMS[fid as usize];
            // The plain version gets its usual material's point too.
            if let Kind::Weapon(w) = &mut defs[fid as usize].kind {
                w.pierce = form.usual.def().pierce;
            }
            for &main in form.mains {
                for second in std::iter::once(M::None).chain(form.seconds.iter().copied()) {
                    if second == main {
                        continue;
                    }
                    for grade in GRADES {
                        if main == form.usual && second == M::None && grade == Grade::Common {
                            variants.insert((fid, main, second, grade), fid);
                            continue;
                        }
                        let id = defs.len() as ItemId;
                        defs.push(made(&base, form, main, second, grade));
                        info.push(ItemInfo { form: fid, main, second, grade });
                        variants.insert((fid, main, second, grade), id);
                    }
                }
            }
        }
        let by_key = defs.iter().enumerate().map(|(i, d)| (d.key, i as ItemId)).collect();
        Catalogue { defs, info, by_key, variants }
    }
}

/// A form's numbers in other materials at another grade, scaled from its
/// usual version.
fn made(base: &ItemDef, form: &Form, main: Material, second: Material, grade: Grade) -> ItemDef {
    let (u, a) = (form.usual.def(), main.def());
    let b = if second == M::None { a } else { second.def() };
    let mass = (a.mass + b.mass) * 0.5;
    let density = (a.density + b.density) * 0.5;
    let turn_blunt = (a.turn_blunt + b.turn_blunt) * 0.5;
    let p = grade.power();
    let kind = match base.kind {
        Kind::Weapon(w) => Kind::Weapon(WeaponDef {
            cut: w.cut * a.edge / u.edge * p,
            blunt: w.blunt * mass / u.mass * p,
            pierce: a.pierce.max(b.pierce),
            ..w
        }),
        Kind::Armor(x) => Kind::Armor(ArmorDef {
            cut: (x.cut * a.turn_cut / u.turn_cut * p).min(0.92),
            blunt: (x.blunt * turn_blunt / u.turn_blunt * p).min(0.92),
            dodge_penalty: x.dodge_penalty * density / u.density,
            ..x
        }),
        Kind::Shield(s) => Kind::Shield((s * (a.turn_cut + turn_blunt) / (u.turn_cut + u.turn_blunt) * p).min(0.8)),
        k => k,
    };
    let backing = if second == M::None { String::new() } else { format!(" on {}", second.name()) };
    let grade_word = if grade == Grade::Common { String::new() } else { format!("{} ", grade.name()) };
    let worth = a.worth + if second == M::None { 0.0 } else { b.worth * 0.3 };
    ItemDef {
        key: leak(format!("{}~{:?}~{:?}~{:?}", form.key, main, second, grade)),
        name: leak(format!("{grade_word}{} {}{backing}", main.name(), form.noun)),
        slot: base.slot,
        kind,
        weight: base.weight * density / u.density,
        value: (base.value * worth / u.worth * grade.worth()).max(1.0),
        effects: base.effects,
    }
}

impl ItemDef {
    pub fn weapon(&self) -> Option<&WeaponDef> {
        match &self.kind {
            Kind::Weapon(w) => Some(w),
            _ => None,
        }
    }
    pub fn armor(&self) -> Option<&ArmorDef> {
        match &self.kind {
            Kind::Armor(a) => Some(a),
            _ => None,
        }
    }
}

/// Bare hands, when nothing is held.
pub const FISTS: WeaponDef = WeaponDef {
    skill: Skill::Unarmed,
    cut: 0.0,
    blunt: 4.0,
    reach: 0.8,
    windup: 0.35,
    recover: 0.35,
    two_handed: false,
    parry: 0.0,
    range: 0.0,
    ammo: None,
    pierce: 0.0,
    entangle: 0.0,
    bleed: 0.0,
    space: Space::Short,
    sweep: 0,
    sneak: 0.0,
    stuns: false,
    conceal: 0.0,
    thrown: false,
    breaks: 0.0,
    offhand: false,
    strength: 0.0,
};
