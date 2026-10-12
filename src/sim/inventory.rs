//! What a person carries: gear in slots, the rest in their pack, and what all
//! of it weighs.
//!
//! Kenshi rules for weight: everything counts, worn or carried. Under your
//! capacity you move freely; over it you slow down, and well over it you can
//! barely move and fight badly. A pack raises the limit.

use serde::{Deserialize, Serialize};

use super::effects::{Does, Lasts};
use super::items::{self, item, ItemId, Kind, Slot, WeaponDef, FISTS, SLOTS};
use super::materials::Piece;
use super::race::Race;
use super::rng::Rng;
use super::stats::{Calling, Skill, Stats, ATTRS, SKILLS};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Gear {
    slots: [Option<ItemId>; SLOTS.len()],
    /// The made pieces being worn, slot by slot (wear and maker's mark).
    #[serde(default)]
    pieces: [Option<Piece>; SLOTS.len()],
    /// Everything not worn: (item, how many, and for a made piece its own
    /// state — such pieces never stack).
    pub bag: Vec<Entry>,
    /// Where things were put in the grids (`arrange`): kept beside the bag,
    /// so whatever adds to or takes from it needn't know. Anything without a
    /// spot here packs itself into the first free cells.
    #[serde(default)]
    pub layout: Vec<Spot>,
}

/// Where one thing (a stack of it) was put: which grid, the cell its top
/// left corner is in, and whether it's turned on its side.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Spot {
    pub item: ItemId,
    pub grid: u8,
    pub x: u8,
    pub y: u8,
    pub turned: bool,
}

/// One stack of a pack entry as it sits in a grid: the entry (an index into
/// the bag), how many of it are in this stack, and the cells it covers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub entry: usize,
    pub n: u16,
    pub grid: u8,
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
    pub turned: bool,
}

/// Pockets: what anyone carries without a pack, (wide, tall) cells.
pub const POCKETS: (u8, u8) = (6, 5);

/// The grid a pack holds, by how much it carries.
pub fn pack_grid(kg: f32) -> (u8, u8) {
    if kg <= 25.0 {
        (6, 5)
    } else if kg <= 50.0 {
        (8, 7)
    } else {
        (8, 9)
    }
}

/// How many of a thing go in one stack (in one footprint of cells).
pub fn stack_of(id: ItemId) -> u16 {
    match item(id).kind {
        Kind::Coin => 1000,
        Kind::Ammo => 30,
        Kind::Material => 10,
        Kind::Food(_) | Kind::Potion | Kind::Scroll(_) | Kind::Torch(_) => 5,
        Kind::Tool if item(id).key == "lockpick" => 10,
        // Thrown weapons carried a few at a time (items design, Part 1).
        Kind::Weapon(_) if items::info(id).form == items::id("throwing_knives") => 6,
        Kind::Weapon(_) if items::info(id).form == items::id("javelin") => 3,
        _ => 1,
    }
}

/// Some of one thing in the pack.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Entry(pub ItemId, pub u16, pub Option<Piece>);

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

    /// Where something meant for `s` goes on: a ring on whichever hand is
    /// free (the first if both are taken), anything else in its own slot.
    /// Where a thing goes when put on: its own slot, except that a blade
    /// that can go in the off hand (`WeaponDef::offhand`) goes there when
    /// the main hand already holds a one-handed weapon and the off hand is
    /// free (the items design, 0.2 rule 6).
    pub fn hand_for(&self, id: ItemId) -> Slot {
        let def = item(id);
        let off = def.weapon().is_some_and(|w| w.offhand);
        let main_one_handed = self.in_slot(Slot::MainHand).and_then(|m| item(m).weapon()).is_some_and(|w| !w.two_handed && w.range == 0.0);
        if def.slot == Slot::MainHand && off && main_one_handed && self.in_slot(Slot::OffHand).is_none() {
            Slot::OffHand
        } else {
            self.slot_for(def.slot)
        }
    }

    pub fn slot_for(&self, s: Slot) -> Slot {
        if s == Slot::Ring && self.in_slot(Slot::Ring).is_some() && self.in_slot(Slot::Ring2).is_none() {
            Slot::Ring2
        } else {
            s
        }
    }

    pub fn add(&mut self, id: ItemId, n: u16) {
        if n == 0 {
            return;
        }
        if let Some(e) = self.bag.iter_mut().find(|e| e.0 == id && e.2.is_none()) {
            e.1 += n;
        } else {
            self.bag.push(Entry(id, n, None));
        }
    }

    /// Put one made piece in the pack, with its own wear and mark.
    pub fn add_piece(&mut self, id: ItemId, piece: Piece) {
        self.bag.push(Entry(id, 1, Some(piece)));
    }

    pub fn take(&mut self, id: ItemId) -> bool {
        self.take_piece(id).is_some()
    }

    /// Take one of something from the pack; for a made piece, its own state
    /// comes with it (`Some(None)` for a plain one).
    pub fn take_piece(&mut self, id: ItemId) -> Option<Option<Piece>> {
        // Plain ones first, so a marked piece isn't spent by accident.
        let i = self.bag.iter().position(|e| e.0 == id && e.2.is_none()).or_else(|| self.bag.iter().position(|e| e.0 == id))?;
        let piece = self.bag[i].2;
        self.bag[i].1 -= 1;
        if self.bag[i].1 == 0 {
            self.bag.remove(i);
        }
        Some(piece)
    }

    /// Take the `k`th entry of the pack (as listed) whole: for picking a
    /// particular piece.
    pub fn take_entry(&mut self, k: usize) -> Option<(ItemId, Option<Piece>)> {
        let e = *self.bag.get(k)?;
        if e.1 > 1 {
            self.bag[k].1 -= 1;
        } else {
            self.bag.remove(k);
        }
        Some((e.0, e.2))
    }

    /// The made piece worn in a slot, if it has its own state.
    /// Is anything worn sealed with pitch (it catches fire)?
    pub fn pieces_sealed(&self) -> bool {
        self.pieces.iter().flatten().any(|p| p.sealed)
    }

    pub fn piece(&self, s: Slot) -> Option<&Piece> {
        self.pieces[slot_index(s)].as_ref()
    }

    pub fn piece_mut(&mut self, s: Slot) -> Option<&mut Piece> {
        self.pieces[slot_index(s)].as_mut()
    }

    /// Give whatever's worn in a slot its own state (full durability) if it
    /// has none yet, and return it.
    pub fn own_piece(&mut self, s: Slot, t: f64) -> Option<&mut Piece> {
        let i = slot_index(s);
        let id = self.slots[i]?;
        let most = items::max_durability(id);
        if most <= 0.0 {
            return None;
        }
        if self.pieces[i].is_none() {
            self.pieces[i] = Some(Piece::new(most, t));
        }
        self.pieces[i].as_mut()
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
        // Only gear is worn or held: coin, food, materials, a tent and the
        // like are carried (round 3: a coin in hand, and one short).
        if !wearable(def.kind) {
            return Err(EquipError::WrongSlot);
        }
        let two_handed = def.weapon().map(|w| w.two_handed).unwrap_or(false);
        let to = self.hand_for(id);
        if to == Slot::OffHand {
            if let Some(m) = self.in_slot(Slot::MainHand) {
                if item(m).weapon().map(|w| w.two_handed).unwrap_or(false) {
                    self.unequip(Slot::MainHand);
                }
            }
        }
        if two_handed {
            self.unequip(Slot::OffHand);
        }
        let piece = self.take_piece(id).flatten();
        self.unequip(to);
        self.slots[slot_index(to)] = Some(id);
        self.pieces[slot_index(to)] = piece;
        Ok(())
    }

    /// Put on the `k`th thing in the pack (a particular piece).
    pub fn equip_entry(&mut self, k: usize) -> Result<(), EquipError> {
        let Some(e) = self.bag.get(k).copied() else { return Err(EquipError::NotInBag) };
        if !wearable(item(e.0).kind) {
            return Err(EquipError::WrongSlot);
        }
        // Move it to the front of its kind so `equip` picks this one.
        self.bag.remove(k);
        self.bag.insert(0, e);
        if e.2.is_some() {
            // (take_piece prefers plain ones; take this piece directly.)
            let def = item(e.0);
            let two_handed = def.weapon().map(|w| w.two_handed).unwrap_or(false);
            let to = self.hand_for(e.0);
            if to == Slot::OffHand {
                if let Some(m) = self.in_slot(Slot::MainHand) {
                    if item(m).weapon().map(|w| w.two_handed).unwrap_or(false) {
                        self.unequip(Slot::MainHand);
                    }
                }
            }
            if two_handed {
                self.unequip(Slot::OffHand);
            }
            let (id, piece) = self.take_entry(0).unwrap();
            self.unequip(to);
            self.slots[slot_index(to)] = Some(id);
            self.pieces[slot_index(to)] = piece;
            return Ok(());
        }
        self.equip(e.0)
    }

    /// Whatever is in a slot is used up (not put back in the pack).
    pub fn discard(&mut self, s: Slot) -> Option<ItemId> {
        self.pieces[slot_index(s)] = None;
        self.slots[slot_index(s)].take()
    }

    /// Take off whatever is in a slot and put it in the pack.
    pub fn unequip(&mut self, s: Slot) -> Option<ItemId> {
        let i = slot_index(s);
        let it = self.slots[i].take();
        let piece = self.pieces[i].take();
        if let Some(id) = it {
            match piece {
                Some(p) => self.add_piece(id, p),
                None => self.add(id, 1),
            }
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

    /// Every effect from everything worn: (what it does, how strong).
    pub fn worn_effects(&self) -> impl Iterator<Item = (Does, f32)> + '_ {
        self.equipped().flat_map(|id| item(id).effects.iter()).filter(|e| e.lasts == Lasts::Worn).map(|e| (e.does, e.power))
    }

    /// The summed power of everything worn that does `does`.
    pub fn worn(&self, does: Does) -> f32 {
        super::effects::total(self.worn_effects(), does)
    }

    /// Kilograms, worn and carried.
    pub fn weight(&self) -> f32 {
        let worn: f32 = self.equipped().map(|id| item(id).weight).sum();
        let packed: f32 = self.bag.iter().map(|e| item(e.0).weight * e.1 as f32).sum();
        worn + packed
    }

    /// How much this person can carry with this gear on, kg.
    pub fn capacity(&self, stats: &Stats) -> f32 {
        let pack = match self.in_slot(Slot::Back).map(|id| item(id).kind) {
            Some(Kind::Pack(kg)) => kg,
            _ => 0.0,
        };
        let extra = self.worn(Does::Carry);
        effective(stats, self).carry_capacity() + pack + extra
    }

    /// Load as a share of capacity (1.0 = exactly at the limit).
    pub fn load(&self, stats: &Stats) -> f32 {
        self.weight() / self.capacity(stats).max(1.0)
    }

    /// Total value, coin.
    pub fn value(&self) -> f32 {
        self.equipped().map(|id| item(id).value).sum::<f32>() + self.bag.iter().map(|e| item(e.0).value * e.1 as f32).sum::<f32>()
    }
}

impl Gear {
    /// The grids things are carried in: pockets, then the pack's if one is
    /// worn. (wide, tall) cells.
    pub fn grids(&self) -> Vec<(u8, u8)> {
        let mut out = vec![POCKETS];
        if let Some(Kind::Pack(kg)) = self.in_slot(Slot::Back).map(|id| item(id).kind) {
            out.push(pack_grid(kg));
        }
        out
    }

    /// Where everything in the pack sits: the stacks placed, and those that
    /// don't fit anywhere (entry, how many), carried loose. A pure function of
    /// the gear: things with a spot in `layout` go there if it's still free,
    /// the rest into the first free cells, upright first, then turned.
    pub fn arrange(&self) -> (Vec<Placed>, Vec<(usize, u16)>) {
        let grids = self.grids();
        let mut used: Vec<Vec<bool>> = grids.iter().map(|&(w, h)| vec![false; w as usize * h as usize]).collect();
        // Every stack of every entry, in order.
        let mut stacks: Vec<(usize, u16)> = Vec::new();
        for (k, e) in self.bag.iter().enumerate() {
            let per = stack_of(e.0).max(1);
            let mut left = e.1;
            while left > 0 {
                let n = left.min(per);
                stacks.push((k, n));
                left -= n;
            }
        }
        let fits = |used: &Vec<Vec<bool>>, g: usize, x: u8, y: u8, w: u8, h: u8| {
            let (gw, gh) = grids[g];
            if x as u16 + w as u16 > gw as u16 || y as u16 + h as u16 > gh as u16 {
                return false;
            }
            (y..y + h).all(|yy| (x..x + w).all(|xx| !used[g][yy as usize * gw as usize + xx as usize]))
        };
        let take = |used: &mut Vec<Vec<bool>>, g: usize, x: u8, y: u8, w: u8, h: u8| {
            let gw = grids[g].0 as usize;
            for yy in y..y + h {
                for xx in x..x + w {
                    used[g][yy as usize * gw + xx as usize] = true;
                }
            }
        };
        let mut placed: Vec<Placed> = Vec::new();
        let mut done = vec![false; stacks.len()];
        // Things put somewhere stay there while it's free.
        for sp in &self.layout {
            let Some(i) = (0..stacks.len()).find(|&i| !done[i] && self.bag[stacks[i].0].0 == sp.item) else { continue };
            let (fw, fh) = items::footprint(sp.item);
            let (w, h) = if sp.turned { (fh, fw) } else { (fw, fh) };
            let g = sp.grid as usize;
            if g < grids.len() && fits(&used, g, sp.x, sp.y, w, h) {
                take(&mut used, g, sp.x, sp.y, w, h);
                placed.push(Placed { entry: stacks[i].0, n: stacks[i].1, grid: sp.grid, x: sp.x, y: sp.y, w, h, turned: sp.turned });
                done[i] = true;
            }
        }
        // The rest: the first free cells.
        let mut loose = Vec::new();
        for i in 0..stacks.len() {
            if done[i] {
                continue;
            }
            let (fw, fh) = items::footprint(self.bag[stacks[i].0].0);
            let mut at = None;
            'search: for g in 0..grids.len() {
                for turned in [false, true] {
                    let (w, h) = if turned { (fh, fw) } else { (fw, fh) };
                    for y in 0..grids[g].1 {
                        for x in 0..grids[g].0 {
                            if fits(&used, g, x, y, w, h) {
                                at = Some((g, x, y, w, h, turned));
                                break 'search;
                            }
                        }
                    }
                }
            }
            match at {
                Some((g, x, y, w, h, turned)) => {
                    take(&mut used, g, x, y, w, h);
                    placed.push(Placed { entry: stacks[i].0, n: stacks[i].1, grid: g as u8, x, y, w, h, turned });
                }
                None => loose.push(stacks[i]),
            }
        }
        (placed, loose)
    }

    /// How many things are carried loose (no room in the grids).
    pub fn loose(&self) -> u16 {
        self.arrange().1.iter().map(|l| l.1).sum()
    }

    /// Is there room in the grids for `n` more of this (without anything
    /// already packed having to come out)?
    pub fn room_for(&self, id: ItemId, n: u16) -> bool {
        self.room_for_all(&[(id, n)])
    }

    /// Is there room for all of these at once?
    pub fn room_for_all(&self, things: &[(ItemId, u16)]) -> bool {
        let before = self.loose();
        let mut g = self.clone();
        for &(id, n) in things {
            g.add(id, n);
        }
        g.loose() <= before
    }

    /// Put the stack placed `which` (an index into `arrange().0`) at another
    /// spot, turned or not. Everything else stays where it is. False (and
    /// nothing moves) if it wouldn't fit there.
    pub fn place(&mut self, which: usize, grid: u8, x: u8, y: u8, turned: bool) -> bool {
        let (placed, _) = self.arrange();
        let Some(&moved) = placed.get(which) else { return false };
        let id = self.bag[moved.entry].0;
        let (fw, fh) = items::footprint(id);
        let (w, h) = if turned { (fh, fw) } else { (fw, fh) };
        let grids = self.grids();
        let Some(&(gw, gh)) = grids.get(grid as usize) else { return false };
        if x as u16 + w as u16 > gw as u16 || y as u16 + h as u16 > gh as u16 {
            return false;
        }
        let overlaps = |p: &Placed| p.grid == grid && p.x < x + w && x < p.x + p.w && p.y < y + h && y < p.y + p.h;
        if placed.iter().enumerate().any(|(k, p)| k != which && overlaps(p)) {
            return false;
        }
        // Pin everything where it is now, the moved stack at its new spot (in
        // the same order, so stacks of one thing keep their places).
        self.layout = placed
            .iter()
            .enumerate()
            .map(|(k, p)| if k == which { Spot { item: id, grid, x, y, turned } } else { Spot { item: self.bag[p.entry].0, grid: p.grid, x: p.x, y: p.y, turned: p.turned } })
            .collect();
        true
    }

    /// Repack from scratch: biggest first, then by name.
    pub fn tidy(&mut self) {
        self.layout.clear();
        self.bag.sort_by(|a, b| {
            let (aw, ah) = items::footprint(a.0);
            let (bw, bh) = items::footprint(b.0);
            (bw as u16 * bh as u16).cmp(&(aw as u16 * ah as u16)).then(item(a.0).name.cmp(item(b.0).name))
        });
    }
}

/// Stats as they stand with gear on: enchantments added on top of the base.
pub fn effective(base: &Stats, gear: &Gear) -> Stats {
    let mut s = base.clone();
    for (does, v) in gear.worn_effects() {
        match does {
            Does::Attr(a) => {
                let cur = s.attr(a);
                s.set_attr(a, cur + v);
            }
            Does::Skill(k) => {
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
    // Full pace up to the limit, a quarter at twice it, and nothing at
    // three times it: past that they can't move at all.
    if load <= 1.0 {
        1.0
    } else if load <= 2.0 {
        1.0 - (load - 1.0) * 0.75
    } else {
        (0.25 * (OVERLOAD_STOP - load)).max(0.0)
    }
}

/// At this many times what they can carry, nobody can move.
pub const OVERLOAD_STOP: f32 = 3.0;

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
            (Skill::Blade, _) => &["knife", "short_sword", "cutlass", "longsword", "greatsword"],
            (Skill::Marksman, _) => &["short_bow", "longbow", "crossbow"],
            (Skill::Blunt, Race::Roduro) => &["club", "mace", "stone_maul"],
            (Skill::Blunt, Race::Qotiro) => &["club", "mace", "war_pick", "battle_axe"],
            (Skill::Blunt, _) => &["club", "mace", "war_pick"],
            (_, Race::Horaro) => &["spear", "trident", "glaive"],
            _ => &["short_spear", "spear", "glaive"],
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
        wishlist.extend(["scale_hauberk", "mail_shirt", "iron_helm", "scale_greaves", "brigandine"]);
        match race {
            Race::Qotiro => wishlist.insert(0, "plate_cuirass"),
            Race::Roduro if stats.calling == Calling::Warrior => wishlist.insert(0, "stone_cuirass"),
            _ => {}
        }
    }
    wishlist.extend(["hide_coat", "padded_jacket", "padded_vest", "leather_cap", "hide_leggings", "boots", "leather_gloves", "bracers"]);
    // Something for the weather, now and then (cloaks and hoods).
    let weather = rng.f32();
    if weather < 0.25 {
        wishlist.push("cloak");
    } else if weather < 0.35 {
        wishlist.push("hood");
    }
    if !two_handed && stats.calling == Calling::Warrior {
        wishlist.insert(1, "round_shield");
        wishlist.push("buckler");
    }
    if rng.chance(0.6) {
        wishlist.push("small_pack");
    }
    // Now and then something kept hidden, or something to throw (items
    // design, Part 1: every form should turn up somewhere).
    let extra = rng.f32();
    let tucked: &[(&str, u16)] = if extra < 0.08 {
        &[("sap", 1)]
    } else if extra < 0.14 {
        &[("throwing_knives", 6)]
    } else if extra < 0.19 {
        &[("dirk", 1)]
    } else if extra < 0.23 {
        &[("javelin", 3)]
    } else if extra < 0.26 {
        &[("knuckle_guards", 1)]
    } else if extra < 0.28 {
        &[("hand_crossbow", 1), ("bolts", 6)]
    } else {
        &[]
    };
    for &(key, n) in tucked {
        let id = items::id(key);
        if item(id).value * n as f32 <= left {
            g.add(id, n);
            left -= item(id).value * n as f32;
        }
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
    g.bag.retain(|e| !matches!(item(e.0).key, "cloth_shirt" | "trousers"));
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
        g.wear(items::id("round_shield"));
        g.add(items::id("glaive"), 1);
        g.equip(items::id("glaive")).unwrap();
        assert_eq!(g.in_slot(Slot::OffHand), None, "the shield should come off");
        assert!(g.bag.iter().any(|e| e.0 == items::id("round_shield")));
        g.equip(items::id("round_shield")).unwrap();
        assert_eq!(g.in_slot(Slot::MainHand), None, "the glaive should come off");
    }

    #[test]
    fn weight_counts_worn_and_carried_and_packs_help() {
        let s = stats();
        let mut g = Gear::default();
        let base_cap = g.capacity(&s);
        g.add(items::id("scale_hauberk"), 8);
        assert!((g.weight() - 8.0 * item(items::id("scale_hauberk")).weight).abs() < 0.01);
        let heavy = g.load(&s);
        g.wear(items::id("large_pack"));
        assert!((g.capacity(&s) - base_cap - 45.0).abs() < 0.01);
        assert!(g.load(&s) < heavy);
        assert!(encumbrance_factor(heavy) < 1.0);
    }

    #[test]
    fn things_pack_into_pockets_then_the_pack() {
        let mut g = Gear::default();
        assert_eq!(g.grids(), vec![POCKETS]);
        g.add(items::id("knife"), 1);
        g.add(items::id("flatbread"), 12);
        let (placed, loose) = g.arrange();
        assert!(loose.is_empty());
        // Twelve flatbreads are three stacks of five, four and three... of five at most.
        assert_eq!(placed.iter().filter(|p| g.bag[p.entry].0 == items::id("flatbread")).map(|p| p.n).sum::<u16>(), 12);
        assert!(placed.iter().all(|p| p.n <= stack_of(g.bag[p.entry].0)));
        // Pockets full of knives: no room for a spear (six long).
        g.add(items::id("knife"), 14);
        assert!(!g.room_for(items::id("knife"), 1));
        assert!(!g.room_for(items::id("spear"), 1));
        // With a large pack on, it fits.
        g.wear(items::id("large_pack"));
        assert_eq!(g.grids().len(), 2);
        assert!(g.room_for(items::id("spear"), 1));
    }

    #[test]
    fn moving_a_thing_keeps_the_rest_where_they_were() {
        let mut g = Gear::default();
        g.wear(items::id("small_pack"));
        for k in ["knife", "hide_coat", "healing_draught"] {
            g.add(items::id(k), 1);
        }
        let before = g.arrange().0;
        let knife = before.iter().position(|p| g.bag[p.entry].0 == items::id("knife")).unwrap();
        // Into the bottom corner of the pack.
        let (gw, gh) = g.grids()[1];
        assert!(g.place(knife, 1, gw - 1, gh - 2, false));
        let after = g.arrange().0;
        let moved = after.iter().find(|p| g.bag[p.entry].0 == items::id("knife")).unwrap();
        assert_eq!((moved.grid, moved.x, moved.y), (1, gw - 1, gh - 2));
        for p in before.iter().filter(|p| g.bag[p.entry].0 != items::id("knife")) {
            assert!(after.contains(p), "{} moved", item(g.bag[p.entry].0).name);
        }
        // Not on top of something else.
        let coat = after.iter().position(|p| g.bag[p.entry].0 == items::id("hide_coat")).unwrap();
        assert!(!g.place(coat, 1, gw - 1, gh - 2, false));
        // Turned, if it fits that way.
        let knife = after.iter().position(|p| g.bag[p.entry].0 == items::id("knife")).unwrap();
        assert!(g.place(knife, 1, 0, 0, true));
        assert!(g.arrange().0.iter().any(|p| g.bag[p.entry].0 == items::id("knife") && p.turned && p.w == 2 && p.h == 1));
    }

    #[test]
    fn what_will_not_fit_is_carried_loose() {
        let mut g = Gear::default();
        for _ in 0..8 {
            g.add_piece(items::id("hide_coat"), crate::sim::materials::Piece::new(100.0, 0.0));
        }
        let (_, loose) = g.arrange();
        assert!(!loose.is_empty(), "eight coats don't fit in pockets");
        assert_eq!(g.loose(), loose.iter().map(|l| l.1).sum::<u16>());
        // And tidying doesn't lose anything.
        let n = g.bag.len();
        g.tidy();
        assert_eq!(g.bag.len(), n);
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

/// What can be worn or held: weapons, tools, shields, torches and wear.
pub fn wearable(kind: super::items::Kind) -> bool {
    use super::items::Kind as K;
    matches!(kind, K::Weapon(_) | K::Armor(_) | K::Shield(_) | K::Pack(_) | K::Trinket | K::Tool | K::Torch(_))
}
