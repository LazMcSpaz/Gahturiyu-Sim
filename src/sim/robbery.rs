//! Losing is a story (the playable-MVP list): bandits who beat the squad
//! don't just walk off. They go through the downed, take the coin and a
//! good share of what's worth having, and carry it home to their camp, where
//! it can be won back.
//!
//! Only the squad's own fights (the player exception). What's taken is
//! decided by rolls keyed to the fight, the person and the thing, never by
//! when the code runs.

use super::combat::{Battle, SQUAD_SIDE};
use super::inventory::Entry;
use super::items::{item, Kind, Slot, SLOTS};
use super::person::PersonId;
use super::rng::Rng;
use super::world::World;

/// Chance a robber takes each thing in a beaten member's pack (coin and
/// notes always go; food half as often).
pub const TAKE_PACK: f32 = 0.7;
/// Chance they take the weapon in hand, and each other worn piece.
pub const TAKE_WEAPON: f32 = 0.6;
pub const TAKE_WORN: f32 = 0.35;

impl World {
    /// After a fight the squad lost to bandits (or anyone hostile): the
    /// winners rob the downed. Returns how many things were taken.
    pub(super) fn rob_the_beaten(&mut self, b: &Battle) -> usize {
        let Some(won) = b.winner() else { return 0 };
        if won == SQUAD_SIDE {
            return 0;
        }
        // People on the winning side who'd rob: bandits, or a hostile band.
        let robbers: Vec<PersonId> = b
            .fighters
            .iter()
            .filter(|f| f.home == won && f.is_person() && f.active())
            .map(|f| f.pid)
            .filter(|&p| self.people[p as usize].bandit || self.group_of[p as usize].and_then(|g| self.group(g)).is_some_and(|g| g.hostile))
            .collect();
        if robbers.is_empty() {
            return 0;
        }
        let victims: Vec<PersonId> = b.fighters.iter().filter(|f| f.home == SQUAD_SIDE && f.is_person() && !f.dead && self.squad.index(f.pid).is_some()).map(|f| f.pid).collect();
        let t = b.time;
        let mut taken: Vec<Entry> = Vec::new();
        for &v in &victims {
            let roll = |what: u64| Rng::from_keys(&[self.seed, b.id as u64, v as u64, what, 0x524F_4242]).f32();
            let Some(d) = self.people[v as usize].detail.as_mut() else { continue };
            // Their pack.
            let mut k = d.gear.bag.len();
            while k > 0 {
                k -= 1;
                let e = d.gear.bag[k];
                let chance = match item(e.0).kind {
                    Kind::Coin => 1.0,
                    Kind::Food(_) => TAKE_PACK * 0.5,
                    _ => TAKE_PACK,
                };
                if roll(1000 + e.0 as u64 * 7 + k as u64) < chance {
                    taken.push(d.gear.bag.remove(k));
                }
            }
            // What they wear (not the pack on their back: the rest is in it).
            for (n, &s) in SLOTS.iter().enumerate() {
                if s == Slot::Back {
                    continue;
                }
                let Some(it) = d.gear.in_slot(s) else { continue };
                let chance = if s == Slot::MainHand { TAKE_WEAPON } else { TAKE_WORN };
                if roll(n as u64) < chance {
                    let piece = d.gear.piece(s).copied();
                    d.gear.discard(s);
                    taken.push(Entry(it, 1, piece));
                }
            }
            self.people[v as usize].recompute_might();
            self.settle_condition(v, t.max(self.people[v as usize].cond.as_ref().map(|c| c.at).unwrap_or(t)));
        }
        if taken.is_empty() {
            return 0;
        }
        // Shared out among the robbers (into their packs: they don't use it).
        let mut coin = 0u32;
        let mut things: Vec<String> = Vec::new();
        for (n, e) in taken.iter().enumerate() {
            let r = robbers[n % robbers.len()];
            if self.people[r as usize].ensure_detail() {
                self.stats.detailed += 1;
            }
            if let Some(d) = self.people[r as usize].detail.as_mut() {
                match e.2 {
                    Some(pc) => d.gear.add_piece(e.0, pc),
                    None => d.gear.add(e.0, e.1),
                }
            }
            match item(e.0).kind {
                Kind::Coin => coin += item(e.0).value as u32 * e.1 as u32,
                _ => things.push(if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() }),
            }
        }
        let mut what = Vec::new();
        if coin > 0 {
            what.push(format!("{coin} coin"));
        }
        let shown = things.len().min(4);
        what.extend(things.iter().take(shown).cloned());
        if things.len() > shown {
            what.push(format!("{} other things", things.len() - shown));
        }
        let camp = robbers.iter().find_map(|&r| self.group_of[r as usize]).is_some_and(|g| self.camps.iter().any(|c| c.group == g));
        let home = if camp { " They'll have it back at their camp." } else { "" };
        let line = format!("Beaten. They go through your things and take {}.{home}", what.join(", "));
        self.alerts.push(line.clone());
        self.log.push_front((t, line));
        self.log.truncate(14);
        taken.len()
    }
}
