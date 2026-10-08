//! Wear, breakage, rot and repair.
//!
//! Your squad's weapons and armour wear with every blow they strike or take
//! (counted in the fight, written back when it ends); heavy blunt blows crack
//! brittle things like Nacre and Slatewing faster. At zero a piece is gone.
//! Edgeglass wears slowly but can't be mended: it shatters when spent.
//! Unsealed reed rots with the days; pitch seals it.
//!
//! Mending needs someone who knows the material's craft: a town's crafter
//! at work (a smith for Forgeiron and Bronze, a weaver for reed and shell, a
//! Tender for grown stone, any leather- or woodworker for the basics), or a
//! squad member with the craft at the right station.
//!
//! Everyone outside the squad keeps their gear up at home, so wear is the
//! squad's alone (like hunger).

use super::combat::Fighter;
use super::items::{self, item, ItemId, Slot, SLOTS};
use super::materials::{Craft, Piece};
use super::person::PersonId;
use super::world::World;

/// Coin a town's crafter charges to mend a piece, per share of its worth
/// that's worn away.
pub const MEND_PRICE: f32 = 0.35;
/// Durability a squad member restores in one go at a station, per skill point.
pub const SELF_MEND: f32 = 1.2;

impl World {
    /// A finished fight's wear, onto a squad member's gear.
    pub(super) fn wear_gear(&mut self, f: &Fighter, t: f64) {
        let pid = f.pid;
        // (slot, blows, of which heavy blunt ones)
        let mut hits: Vec<(Slot, f32, f32)> = Vec::new();
        if f.weapon_wear > 0.0 {
            hits.push((Slot::MainHand, f.weapon_wear, 0.0));
        }
        if f.shield_wear > 0.0 {
            hits.push((Slot::OffHand, f.shield_wear, 0.0));
        }
        for (i, s) in f.armor_slots.iter().enumerate() {
            if let Some(w) = f.armor_wear.get(i).filter(|w| w[0] > 0.0) {
                hits.push((*s, w[0], w[1]));
            }
        }
        for (slot, blows, heavy) in hits {
            let Some(d) = self.people[pid as usize].detail.as_mut() else { return };
            let Some(id) = d.gear.in_slot(slot) else { continue };
            let m = items::info(id).main.def();
            // Heavy blows crack brittle things on top of the usual wear.
            let loss = blows * m.wear + heavy * m.brittle;
            let Some(pc) = d.gear.own_piece(slot, t) else { continue };
            pc.settle(m.rots, t);
            pc.left -= loss;
            if pc.left <= 0.0 {
                self.break_piece(pid, slot, t);
            }
        }
    }

    /// A piece worn to nothing is gone.
    fn break_piece(&mut self, pid: PersonId, slot: Slot, t: f64) {
        let p = &mut self.people[pid as usize];
        let Some(d) = p.detail.as_mut() else { return };
        let Some(id) = d.gear.discard(slot) else { return };
        let name = d.name.clone();
        let how = if items::repairable(id) { "breaks" } else { "shatters" };
        p.recompute_might();
        self.log.push_front((t, format!("{name}'s {} {how}.", item(id).name.to_lowercase())));
        self.log.truncate(14);
    }

    /// Unsealed reed rots: anything the squad carries that's rotted through
    /// is gone. Checked on the hour.
    pub(super) fn rot_gear(&mut self, t: f64) {
        for m in self.squad.members.clone() {
            let Some(d) = self.people[m as usize].detail.as_mut() else { continue };
            let gone: Vec<Slot> = SLOTS.iter().copied().filter(|&s| d.gear.in_slot(s).map(|id| items::info(id).main.def().rots).unwrap_or(false) && d.gear.piece(s).map(|pc| pc.left_at(true, t) <= 0.0).unwrap_or(false)).collect();
            let rotted_bag = d.gear.bag.len();
            d.gear.bag.retain(|e| !(items::info(e.0).main.def().rots && e.2.map(|pc| pc.left_at(true, t) <= 0.0).unwrap_or(false)));
            let lost_bag = rotted_bag != d.gear.bag.len();
            for s in gone {
                self.break_piece(m, s, t);
            }
            if lost_bag {
                let name = self.people[m as usize].name().unwrap_or("someone").to_string();
                self.log.push_front((t, format!("Something of {name}'s has rotted through.")));
                self.people[m as usize].recompute_might();
            }
        }
    }

    /// How worn a squad member's piece in a slot is: (left, most), if it wears.
    pub fn wear_of(&self, pid: PersonId, slot: Slot) -> Option<(f32, f32)> {
        let d = self.people[pid as usize].detail.as_ref()?;
        let id = d.gear.in_slot(slot)?;
        let most = items::max_durability(id);
        if most <= 0.0 {
            return None;
        }
        let left = d.gear.piece(slot).map(|pc| pc.left_at(items::info(id).main.def().rots, self.time)).unwrap_or(most);
        Some((left, most))
    }

    /// Does anyone in this town know how to mend this, and are they at work?
    pub fn mender_for(&self, town: u16, id: ItemId) -> Option<PersonId> {
        if !items::repairable(id) {
            return None;
        }
        let craft = items::craft_of(id);
        self.settlements[town as usize].residents.iter().copied().find(|&p| self.life(p).job.craft() == Some(craft) && self.at_work(p, self.time))
    }

    /// What a town's crafter asks to mend a piece worn in a slot.
    pub fn mend_price(&self, pid: PersonId, slot: Slot) -> Option<u16> {
        let (left, most) = self.wear_of(pid, slot)?;
        let id = self.people[pid as usize].detail.as_ref()?.gear.in_slot(slot)?;
        if left >= most || !items::repairable(id) {
            return None;
        }
        Some(((1.0 - left / most) * item(id).value * MEND_PRICE).ceil().max(1.0) as u16)
    }

    /// Pay a town's crafter (`mender`, at work) to mend a squad member's
    /// piece. Refused for what they don't know how to work, and for Edgeglass.
    pub fn pay_to_mend(&mut self, mender: PersonId, pid: PersonId, slot: Slot) -> Result<(), &'static str> {
        let id = self.people[pid as usize].detail.as_ref().and_then(|d| d.gear.in_slot(slot)).ok_or("nothing there")?;
        if !items::repairable(id) {
            return Err("that can't be mended");
        }
        if self.life(mender).job.craft() != Some(items::craft_of(id)) || !self.at_work(mender, self.time) {
            return Err("they don't work that");
        }
        let price = self.mend_price(pid, slot).ok_or("it isn't worn")?;
        if self.squad_count(items::id("coin")) < price {
            return Err("not enough coin");
        }
        self.take_from_squad(items::id("coin"), price);
        if let Some(town) = self.people[mender as usize].home {
            let t = self.time;
            let tl = &mut self.society.towns[town as usize];
            tl.purse = super::economy::purse_at(tl, t) + price as f32;
            tl.purse_at = t;
        }
        self.restore(pid, slot, f32::MAX);
        Ok(())
    }

    /// A squad member mends a piece themselves, at the station for its craft.
    pub fn mend_myself(&mut self, who: PersonId, owner: PersonId, slot: Slot) -> Result<(), &'static str> {
        let id = self.people[owner as usize].detail.as_ref().and_then(|d| d.gear.in_slot(slot)).ok_or("nothing there")?;
        if !items::repairable(id) {
            return Err("that can't be mended");
        }
        let craft = items::craft_of(id);
        if !self.knows_craft(who, craft) {
            return Err("they don't know that craft");
        }
        let at = self.person_pos(who);
        if self.station_near(at, craft.station()).is_none() {
            return Err("no station for it here");
        }
        let skill = self.people[who as usize].effective_stats().skill(craft.skill());
        self.restore(owner, slot, skill * SELF_MEND + 10.0);
        self.people[who as usize].stats.exercise(craft.skill(), 1.0);
        Ok(())
    }

    /// Has someone taken up this craft? (Your squad: from a teacher or a
    /// manual; anyone else: if it's their trade.)
    pub fn knows_craft(&self, who: PersonId, craft: Craft) -> bool {
        let p = &self.people[who as usize];
        if p.in_squad {
            p.detail.as_ref().map(|d| d.crafts.contains(&craft.skill())).unwrap_or(false)
        } else {
            self.society.lives.get(who as usize).map(|l| l.job.craft() == Some(craft)).unwrap_or(false)
        }
    }

    fn restore(&mut self, pid: PersonId, slot: Slot, amount: f32) {
        let t = self.time;
        let Some(d) = self.people[pid as usize].detail.as_mut() else { return };
        let Some(id) = d.gear.in_slot(slot) else { return };
        let most = items::max_durability(id);
        let rots = items::info(id).main.def().rots;
        if let Some(pc) = d.gear.own_piece(slot, t) {
            pc.settle(rots, t);
            pc.left = (pc.left + amount).min(most);
        }
    }

    /// Seal a squad member's reed piece in a slot with pitch from the packs.
    pub fn seal(&mut self, pid: PersonId, slot: Slot) -> bool {
        let t = self.time;
        let pitch = items::id("pitch");
        if self.squad_count(pitch) == 0 {
            return false;
        }
        let Some(d) = self.people[pid as usize].detail.as_mut() else { return false };
        let Some(id) = d.gear.in_slot(slot) else { return false };
        if !items::info(id).main.def().rots {
            return false;
        }
        let Some(pc) = d.gear.own_piece(slot, t) else { return false };
        if pc.sealed {
            return false;
        }
        pc.settle(true, t);
        pc.sealed = true;
        self.take_from_squad(pitch, 1);
        true
    }
}

/// A fresh made piece.
pub fn fresh(id: ItemId, t: f64) -> Piece {
    Piece::new(items::max_durability(id), t)
}

impl Craft {
    /// The station this craft is worked at.
    pub fn station(self) -> super::crafting::Station {
        use super::crafting::Station as S;
        match self {
            Craft::Handcraft => S::Workbench,
            Craft::Smithing => S::Forge,
            Craft::Armoring => S::Bench,
            Craft::Tending => S::GrowerBed,
            Craft::Weaving => S::Loom,
            Craft::Inscription => S::Desk,
            Craft::Alchemy => S::AlchemyTable,
        }
    }
}
