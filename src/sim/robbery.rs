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
/// Chance someone who runs from the squad drops something on the way.
pub const DROP_ON_FLIGHT: f32 = 0.45;

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
        // The coin is shared out among the robbers' purses; the things go
        // back to their camp's stash (or into their packs, if they keep no
        // camp: they don't use it).
        let stash = robbers.iter().find_map(|&r| self.group_of[r as usize]).and_then(|g| self.camps.iter().find(|c| c.group == g)).and_then(|c| c.stash).filter(|s| self.containers.contains_key(s));
        let mut coin = 0u32;
        let mut things: Vec<String> = Vec::new();
        let mut stashed: Vec<Entry> = Vec::new();
        for (n, e) in taken.iter().enumerate() {
            let r = robbers[n % robbers.len()];
            if self.people[r as usize].ensure_detail() {
                self.stats.detailed += 1;
            }
            match (stash, item(e.0).kind) {
                (Some(_), k) if k != Kind::Coin => stashed.push(*e),
                _ => {
                    if let Some(d) = self.people[r as usize].detail.as_mut() {
                        match e.2 {
                            Some(pc) => d.gear.add_piece(e.0, pc),
                            None => d.gear.add(e.0, e.1),
                        }
                    }
                }
            }
            match item(e.0).kind {
                Kind::Coin => coin += item(e.0).value as u32 * e.1 as u32,
                _ => things.push(if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() }),
            }
        }
        if let Some(st) = stash {
            self.put_into(st, &stashed);
        }
        for &r in &robbers {
            self.people[r as usize].recompute_might();
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
        let home = if stash.is_some() && !stashed.is_empty() { " They'll have it back at their camp." } else { "" };
        let line = format!("Beaten. They go through your things and take {}.{home}", what.join(", "));
        self.alerts.push(line.clone());
        self.log.push_front((t, line));
        self.log.truncate(14);
        taken.len()
    }

    /// After a fight with the squad: some of those who ran dropped something
    /// as they went (their purse, or a thing from their pack), where they
    /// were when they got away. Keyed to the fight and the runner.
    pub(super) fn runners_drop(&mut self, b: &Battle) {
        let mut lines = Vec::new();
        for f in b.fighters.iter().filter(|f| f.home != SQUAD_SIDE && f.home != super::combat::GRAVE_SIDE && f.is_person() && f.fled && !f.dead) {
            let pid = f.pid;
            let hostile = self.people[pid as usize].bandit || self.group_of[pid as usize].and_then(|g| self.group(g)).is_some_and(|g| g.hostile);
            if !hostile {
                continue;
            }
            let mut r = Rng::from_keys(&[self.seed, b.id as u64, pid as u64, 0x4452_4F50]);
            if r.f32() >= DROP_ON_FLIGHT {
                continue;
            }
            let pick = r.f32();
            let Some(d) = self.people[pid as usize].detail.as_mut() else { continue };
            if d.gear.bag.is_empty() {
                continue;
            }
            // Their purse if they have one, else something from the pack.
            let k = d.gear.bag.iter().position(|e| item(e.0).kind == Kind::Coin).unwrap_or(((pick * d.gear.bag.len() as f32) as usize).min(d.gear.bag.len() - 1));
            let e = d.gear.bag.remove(k);
            self.people[pid as usize].recompute_might();
            let g = self.put_on_ground(e.0, e.1, f.pos);
            if let Some(x) = self.ground.iter_mut().find(|x| x.id == g) {
                x.piece = e.2;
            }
            let what = if e.1 > 1 { format!("{} × {}", e.1, item(e.0).name.to_lowercase()) } else { item(e.0).name.to_lowercase() };
            lines.push(format!("{} drops {what} as they run", self.name_of(pid)));
        }
        if !lines.is_empty() {
            let t = b.time;
            self.log.push_front((t, format!("{}. It lies where they dropped it.", lines.join("; "))));
            self.log.truncate(14);
        }
    }
}
