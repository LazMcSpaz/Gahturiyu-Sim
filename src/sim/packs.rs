//! Putting a pack down and picking it up (the items design, 0.4): one
//! order before a fight or a sneak, and back on after. It lies on the ground
//! with what's in it, and can be stolen. A duffel carried in both hands is
//! put down by itself when a fight starts.

use super::inventory::Entry;
use super::items::{item, Kind, Slot};
use super::person::PersonId;
use super::world::World;

impl World {
    /// `who` puts down the pack on their back (or the duffel in their
    /// hands), with everything in it, where they stand. Returns the line.
    pub fn drop_pack(&mut self, who: PersonId) -> Result<String, String> {
        let name = self.name_of(who);
        let Some(d) = self.people[who as usize].detail.as_ref() else { return Err("Nobody there.".into()) };
        let slot = if d.gear.hands_full() { Slot::MainHand } else { Slot::Back };
        let Some(pack) = d.gear.in_slot(slot).filter(|&id| matches!(item(id).kind, Kind::Pack(_))) else {
            return Err(format!("{name} has no pack on."));
        };
        // What's in that pack's grid comes with it.
        let g = d.gear.holders().iter().position(|h| h.0 == Some(slot)).unwrap_or(usize::MAX);
        let (placed, _) = d.gear.arrange();
        let mut inside: Vec<Entry> = Vec::new();
        let mut taken: Vec<(usize, u16)> = Vec::new();
        for p in placed.iter().filter(|p| p.grid as usize == g) {
            taken.push((p.entry, p.n));
        }
        let at = self.person_pos(who);
        let d = self.people[who as usize].detail.as_mut().unwrap();
        // (Highest entries first, so the indices stay good.)
        taken.sort_by(|a, b| b.0.cmp(&a.0));
        let mut merged: Vec<(usize, u16)> = Vec::new();
        for (k, n) in taken {
            match merged.iter_mut().find(|m| m.0 == k) {
                Some(m) => m.1 += n,
                None => merged.push((k, n)),
            }
        }
        for (k, n) in merged {
            let e = d.gear.bag[k];
            inside.push(Entry(e.0, n, e.2));
            if e.1 > n {
                d.gear.bag[k].1 -= n;
            } else {
                d.gear.bag.remove(k);
            }
        }
        let piece = d.gear.piece(slot).copied();
        d.gear.discard(slot);
        self.people[who as usize].recompute_might();
        let id = self.put_on_ground(pack, 1, at);
        if let Some(gi) = self.ground.iter_mut().find(|g| g.id == id) {
            gi.piece = piece;
            gi.inside = inside;
        }
        let line = format!("{name} puts down the {}.", item(pack).name.to_lowercase());
        self.log.push_front((self.time, line.clone()));
        self.log.truncate(14);
        Ok(line)
    }

    /// Picking up a pack that was put down (ground item `gi`): it goes back
    /// on with what's in it, if their back (or hands) are free. False if it
    /// isn't such a pack, or they can't put it on.
    pub(super) fn take_up_pack(&mut self, who: PersonId, gi: usize) -> bool {
        let g = &self.ground[gi];
        let Kind::Pack(_) = item(g.item).kind else { return false };
        let Some(d) = self.people[who as usize].detail.as_ref() else { return false };
        if d.gear.in_slot(item(g.item).slot).is_some() {
            return false;
        }
        let g = self.ground.remove(gi);
        let d = self.people[who as usize].detail.as_mut().unwrap();
        match g.piece {
            Some(p) => d.gear.add_piece(g.item, p),
            None => d.gear.add(g.item, 1),
        }
        let _ = d.gear.equip(g.item);
        for e in g.inside {
            match e.2 {
                Some(p) => d.gear.add_piece(e.0, p),
                None => d.gear.add(e.0, e.1),
            }
        }
        self.people[who as usize].recompute_might();
        let name = self.name_of(who);
        self.log.push_front((self.time, format!("{name} shoulders the {} again.", item(g.item).name.to_lowercase())));
        self.log.truncate(14);
        true
    }
}
