//! Paying to recover (playtest item 14): a town's healer patches the
//! squad's wounds for coin, and its innkeeper rents beds for the night,
//! where sleep mends faster than in the street.
//!
//! Both are services of people already in every town, offered in talk while
//! they're at work. A healer's work is done on the spot; a rented bed is a
//! `Room` (who, where, until when), read by `shelter_of` when they bed down.

use serde::{Deserialize, Serialize};

use super::geo::V2;
use super::items;
use super::jobs::Job;
use super::person::PersonId;
use super::settlement::SettlementId;
use super::world::{World, DAY, HOUR};

/// Coin per point of health the healer mends.
pub const TREAT_COIN_PER_HP: f32 = 0.3;
/// The least a healer asks.
pub const TREAT_MIN: u16 = 4;
/// Squad members this near the healer (or the innkeeper) are seen to.
pub const CARE_REACH: f32 = 25.0;
/// A night's bed, per head.
pub const BED_COIN: u16 = 5;
/// A rented bed is kept until this hour of the next morning.
pub const ROOM_UNTIL_HOUR: f64 = 9.0;
/// Asleep this near the bed counts as in it.
pub const IN_BED: f32 = 12.0;

/// A bed paid for at an inn.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Room {
    pub who: PersonId,
    pub town: SettlementId,
    /// Where the bed is (inside the inn).
    pub at: V2,
    pub until: f64,
}

impl World {
    /// Hurt squad members close by the healer `npc`, and what mending them
    /// costs. `None` if `npc` isn't a healer at work or no one near is hurt.
    pub fn treat_terms(&self, npc: PersonId) -> Option<(Vec<PersonId>, u16)> {
        if self.life(npc).job != Job::Healer || !self.at_work(npc, self.time) {
            return None;
        }
        let here = self.person_pos(npc);
        let t = self.time;
        let mut hurt = Vec::new();
        let mut hp = 0.0;
        for (k, &m) in self.squad.members.iter().enumerate() {
            let p = &self.people[m as usize];
            if self.member_pos(k).dist(here) > CARE_REACH || !p.wounds.is_hurt(t) {
                continue;
            }
            hp += p.wounds.lost_at(t).iter().enumerate().filter(|(i, _)| !p.wounds.missing[*i]).map(|(_, l)| l).sum::<f32>();
            hurt.push(m);
        }
        if hurt.is_empty() {
            return None;
        }
        Some((hurt, ((hp * TREAT_COIN_PER_HP).ceil() as u16).max(TREAT_MIN)))
    }

    /// The healer mends everyone hurt nearby, for the price. Lost limbs stay
    /// lost. Returns who was seen to.
    pub fn treat(&mut self, npc: PersonId) -> Result<Vec<PersonId>, &'static str> {
        let (hurt, price) = self.treat_terms(npc).ok_or("there's no one here I can help")?;
        let coin = items::id("coin");
        if self.squad_count(coin) < price {
            return Err("that's more coin than you have");
        }
        self.take_from_squad(coin, price);
        let t = self.time;
        for &m in &hurt {
            self.settle_condition(m, t);
            let p = &mut self.people[m as usize];
            let full: [f32; 6] = std::array::from_fn(|i| p.stats.max_hp(super::body::PARTS[i]));
            p.wounds.set(&p.stats.clone(), &full, t);
            self.settle_condition(m, t);
        }
        if let Some(home) = self.people[npc as usize].home {
            self.pay_town_household(npc, home, price as f32);
        }
        Ok(hurt)
    }

    /// Beds the innkeeper `npc` can let to the squad members in the inn
    /// (or close by): (who, price). `None` if they aren't an innkeeper at work.
    pub fn bed_terms(&self, npc: PersonId) -> Option<(Vec<PersonId>, u16)> {
        if self.life(npc).job != Job::Innkeeper || !self.at_work(npc, self.time) {
            return None;
        }
        let here = self.person_pos(npc);
        let who: Vec<PersonId> = self.squad.members.iter().enumerate().filter(|&(k, &m)| self.member_pos(k).dist(here) <= CARE_REACH && !self.has_room(m)).map(|(_, &m)| m).collect();
        if who.is_empty() {
            return None;
        }
        Some((who.clone(), BED_COIN * who.len() as u16))
    }

    /// Rent beds for the night: paid now, kept till the next morning. The
    /// squad members go to them and lie down.
    pub fn rent_beds(&mut self, npc: PersonId) -> Result<Vec<PersonId>, &'static str> {
        let (who, price) = self.bed_terms(npc).ok_or("I've nothing to let you")?;
        let coin = items::id("coin");
        if self.squad_count(coin) < price {
            return Err("that's more coin than you have");
        }
        let Some(town) = self.people[npc as usize].home else { return Err("I've nothing to let you") };
        self.take_from_squad(coin, price);
        self.pay_town_household(npc, town, price as f32);
        let t = self.time;
        // Kept till the morning after tonight's sleep.
        let day = World::day_of(t) + if (t.rem_euclid(DAY) / HOUR) < ROOM_UNTIL_HOUR - 3.0 { 0 } else { 1 };
        let until = day as f64 * DAY + ROOM_UNTIL_HOUR * HOUR;
        let bed = self.inn_bed(npc);
        self.rooms.retain(|r| r.until > t);
        for (n, &m) in who.iter().enumerate() {
            let at = bed.add(super::squad::formation(n).scale(0.25));
            self.rooms.push(Room { who: m, town, at, until });
            self.order_sleep_at(m, at);
        }
        Ok(who)
    }

    /// Does this member have a bed paid for now?
    pub fn has_room(&self, who: PersonId) -> bool {
        self.rooms.iter().any(|r| r.who == who && r.until > self.time)
    }

    /// Is this member at their rented bed (so sleeping there is a bed's sleep)?
    pub fn in_rented_bed(&self, who: PersonId, at: V2) -> bool {
        self.rooms.iter().any(|r| r.who == who && r.until > self.time && r.at.dist(at) <= IN_BED)
    }

    /// Where the inn's beds are: inside the building the innkeeper keeps or
    /// works in, else where they work.
    fn inn_bed(&self, npc: PersonId) -> V2 {
        let at = self.workplace_of(npc).map(|w| w.pos).unwrap_or_else(|| self.person_pos(npc));
        match self.building_at(at).or_else(|| self.doors_near(at, 8.0).into_iter().next()) {
            Some(d) => d.centre,
            None => at,
        }
    }

    /// Coin paid to a townsperson for a service goes to their household.
    fn pay_town_household(&mut self, npc: PersonId, _town: SettlementId, coin: f32) {
        if let Some(h) = self.society.lives.get(npc as usize).and_then(|l| l.household) {
            self.society.households[h as usize].purse.coin += coin;
        }
    }
}
