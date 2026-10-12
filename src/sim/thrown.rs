//! Thrown weapons after a fight (the items design, 0.2 rule 8). In a fight a
//! thrown weapon flies, lands where it stops and may break (`combat.rs`,
//! `Battle::thrown`). When it's over, the squad's own lie on the ground
//! where they came down, for anyone to pick up; the squad is offered to
//! gather them (right-click one: "Gather the thrown weapons"). Everyone
//! else's are picked up by their own (they restock at home, rule 17, so
//! what they carry never depends on whether they'd been met).

use super::combat::Battle;
use super::person::PersonId;
use super::world::World;

impl World {
    /// After a fight: the squad's thrown weapons leave their packs and lie
    /// where they came down (those that broke are gone).
    pub(super) fn lay_thrown(&mut self, b: &Battle) {
        let mut any = false;
        for th in &b.thrown {
            let Some(f) = b.fighters.get(th.by) else { continue };
            if !f.is_person() || !self.people[f.pid as usize].in_squad {
                continue;
            }
            let Some(d) = self.people[f.pid as usize].detail.as_mut() else { continue };
            let main = super::items::Slot::MainHand;
            let piece = if th.from_hand && d.gear.in_slot(main) == Some(th.item) {
                let p = d.gear.piece(main).copied();
                d.gear.discard(main);
                p
            } else {
                match d.gear.take_piece(th.item) {
                    Some(p) => p,
                    None => continue,
                }
            };
            self.people[f.pid as usize].recompute_might();
            if th.broke {
                continue;
            }
            let g = self.put_on_ground(th.item, 1, th.at);
            if let Some(x) = self.ground.iter_mut().find(|x| x.id == g) {
                x.piece = piece;
            }
            self.thrown_lying.push(g);
            any = true;
        }
        if any {
            self.log.push_front((b.time, "Thrown weapons lie where they fell: right-click one to gather them.".to_string()));
            self.log.truncate(14);
        }
    }

    /// Is this thing on the ground one the squad threw?
    pub fn thrown_by_squad(&self, ground: u32) -> bool {
        self.thrown_lying.contains(&ground)
    }

    /// Send `who` to gather the squad's thrown weapons lying about: each
    /// goes for the nearest left, and on to the next once they have it.
    /// Returns how many there are to gather.
    pub fn gather_thrown(&mut self, who: &[PersonId]) -> usize {
        self.thrown_lying.retain(|id| self.ground.iter().any(|g| g.id == *id));
        let n = self.thrown_lying.len();
        for &m in who {
            if self.is_down(m) || self.squad.index(m).is_none() {
                continue;
            }
            if !self.gatherers.contains(&m) {
                self.gatherers.push(m);
            }
            self.gather_next(m);
        }
        n
    }

    /// A gatherer goes for the nearest thrown weapon nobody else is after.
    pub(super) fn gather_next(&mut self, m: PersonId) {
        let at = self.person_pos(m);
        let claimed: Vec<u32> = self.pickups.iter().filter(|p| p.who != m).map(|p| p.thing).collect();
        // (Only what they've room for.)
        let next = self.thrown_lying.iter().copied().filter(|id| !claimed.contains(id)).filter_map(|id| self.ground.iter().find(|g| g.id == id).filter(|g| self.room_in_pack(m, g.item, g.count)).map(|g| (id, g.pos.dist(at)))).min_by(|a, b| a.1.total_cmp(&b.1));
        match next {
            Some((id, _)) if self.order_pickup(m, id) => {}
            _ => self.gatherers.retain(|&g| g != m),
        }
    }
}
