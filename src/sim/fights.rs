//! Where fights meet the world: who starts one, who joins, and what is left
//! afterwards.
//!
//! Hostile groups (bandits) jump anyone who comes within `AGGRO` metres of
//! them — your squad included. A fight runs as a `Battle` until one side is
//! down, dead or gone; then everything that happened in it is written back to
//! the people involved: wounds (which heal over time), mana spent, skills
//! trained, and the dead.

use super::combat::{Battle, Fighter, Order, Side, SQUAD_SIDE};
use super::geo::V2;
use super::group::{Group, GroupId, Kind, Leg};
use super::person::{Person, PersonId};
use super::race::ALL_RACES;
use super::rng::{self, Rng};
use super::stats::{Calling, Skill, SKILLS};
use super::world::World;

/// Metres at which a hostile group notices someone and attacks.
pub const AGGRO: f32 = 35.0;
/// How long the fallen stay on the ground, game seconds.
pub const CORPSE_TIME: f64 = 2.0 * 3600.0;

impl World {
    pub fn battle(&self, id: u32) -> Option<&Battle> {
        self.battles.iter().find(|b| b.id == id)
    }

    /// The fight your squad is in, if any.
    pub fn squad_battle(&self) -> Option<&Battle> {
        self.squad.members.iter().find_map(|m| self.fighting.get(m)).and_then(|&id| self.battle(id))
    }

    pub fn fighter(&self, pid: PersonId) -> Option<&Fighter> {
        let b = self.battle(*self.fighting.get(&pid)?)?;
        b.fighters.iter().find(|f| f.pid == pid)
    }

    pub(super) fn fighter_pos(&self, pid: PersonId) -> Option<V2> {
        self.fighter(pid).map(|f| f.pos)
    }

    /// Squad members able to fight right now.
    pub fn squad_fit(&self) -> Vec<PersonId> {
        let t = self.time;
        self.squad
            .members
            .iter()
            .copied()
            .filter(|&m| {
                let p = &self.people[m as usize];
                !p.dead && !super::body::knocked_out(&p.wounds.hp_at(&p.stats, t))
            })
            .collect()
    }

    /// Tell some squad members to move somewhere mid-fight (they stop
    /// attacking until they get there).
    pub(super) fn order_in_battle(&mut self, who: &[PersonId], target: V2) {
        let mut k = 0usize;
        for &pid in who {
            let Some(&id) = self.fighting.get(&pid) else { continue };
            let Some(b) = self.battles.iter_mut().find(|b| b.id == id) else { continue };
            let Some(f) = b.fighters.iter_mut().find(|f| f.pid == pid && f.side == SQUAD_SIDE && f.active()) else { continue };
            f.order = Some(Order::MoveTo(target.add(super::squad::formation(k))));
            f.act = super::combat::Act::Idle;
            k += 1;
        }
    }

    /// Tell the whole squad to go for one enemy.
    pub fn order_attack(&mut self, enemy: PersonId) -> bool {
        let all = self.squad.members.clone();
        self.order_attack_with(&all, enemy)
    }

    /// Tell some squad members to go for one enemy.
    pub fn order_attack_with(&mut self, who: &[PersonId], enemy: PersonId) -> bool {
        let Some(&id) = self.fighting.get(&enemy) else { return false };
        let Some(b) = self.battles.iter_mut().find(|b| b.id == id) else { return false };
        let Some(j) = b.fighters.iter().position(|f| f.pid == enemy && f.side != SQUAD_SIDE) else { return false };
        let mut any = false;
        for f in b.fighters.iter_mut().filter(|f| f.side == SQUAD_SIDE && f.active() && who.contains(&f.pid)) {
            f.order = Some(Order::Attack(j));
            any = true;
        }
        any
    }

    /// Put a band of bandits at `at`. One of them is a mage if asked. Returns
    /// the new group. (For testing and for the world's own bandit camps.)
    pub fn spawn_bandits(&mut self, at: V2, count: usize, with_mage: bool) -> GroupId {
        let tag = self.people.len() as u64;
        let mut members = Vec::new();
        for k in 0..count {
            let id = self.people.len() as PersonId;
            let seed = rng::key(&[self.seed, 0xBAD, id as u64]);
            let mut r = Rng::from_keys(&[seed, 0x5241_4345]);
            let race = ALL_RACES[r.below(4)];
            let mut p = Person::summary(id, seed, race, None);
            p.bandit = true;
            if with_mage && k == 0 {
                p.specialize(Calling::Mage, &[(Skill::Destruction, 45.0), (Skill::Alteration, 35.0), (Skill::Illusion, 38.0)], 80.0);
            } else {
                let main = *r.pick(&[Skill::Blade, Skill::Blunt, Skill::Spear]);
                p.specialize(Calling::Warrior, &[(main, 30.0 + r.f32() * 18.0), (Skill::Dodge, 20.0), (Skill::Block, 18.0)], 90.0 + r.f32() * 220.0);
            }
            p.ensure_detail();
            p.wounds.at = self.time;
            p.mana_at = self.time;
            self.people.push(p);
            self.busy_until.push(f64::INFINITY);
            self.group_of.push(None);
            self.stats.detailed += 1;
            members.push(id);
        }
        let gid = self.next_group;
        self.next_group += 1;
        let speed = members.iter().map(|&m| self.people[m as usize].race.walk_speed()).fold(f32::MAX, f32::min);
        let g = Group {
            id: gid,
            seed: rng::key(&[self.seed, 0xBAD_6, tag]),
            members,
            kind: Kind::Wanderer { rest_min: f64::INFINITY, rest_max: f64::INFINITY },
            legs: vec![Leg::wait(at, f64::INFINITY)],
            speed,
            ends: f64::INFINITY,
            written: 1,
            hostile: true,
            pos: at,
            last_update: self.time,
            band: 3,
        };
        self.add_group(g);
        gid
    }

    /// Start fights that are due, run the ones in progress, settle the ones
    /// that are over.
    pub(super) fn update_battles(&mut self) {
        let t = self.time;
        self.corpses.retain(|c| t - c.2 < CORPSE_TIME);

        // Hostiles near the squad attack it, or join the fight it's in.
        let squad_fit = self.squad_fit();
        if !squad_fit.is_empty() {
            let current = self.squad.members.iter().find_map(|m| self.fighting.get(m)).copied();
            let squad_at: Vec<V2> = squad_fit.iter().map(|&m| self.person_pos(m)).collect();
            let spotted: Vec<GroupId> = self
                .groups
                .iter()
                .filter(|g| g.hostile && g.band == 1 && squad_at.iter().any(|p| g.pos.dist(*p) < AGGRO))
                .filter(|g| g.members.iter().any(|m| !self.people[*m as usize].dead && !self.fighting.contains_key(m)))
                .map(|g| g.id)
                .collect();
            for gid in spotted {
                let fresh: Vec<PersonId> = self.group(gid).unwrap().members.iter().copied().filter(|m| !self.people[*m as usize].dead && !self.fighting.contains_key(m)).collect();
                match current.or_else(|| self.squad.members.iter().find_map(|m| self.fighting.get(m)).copied()) {
                    Some(bid) => self.join_battle(bid, 1, &fresh),
                    None => {
                        let line = "You're attacked!".to_string();
                        self.alerts.push(line.clone());
                        self.log.push_front((t, line));
                        self.start_battle(vec![(SQUAD_SIDE, squad_fit.clone()), (1, fresh)]);
                    }
                }
            }
        }

        for b in &mut self.battles {
            b.advance_to(t);
        }
        let (done, live): (Vec<Battle>, Vec<Battle>) = std::mem::take(&mut self.battles).into_iter().partition(|b| b.over);
        self.battles = live;
        for b in done {
            self.conclude(b);
        }
    }

    fn start_battle(&mut self, sides: Vec<(Side, Vec<PersonId>)>) -> u32 {
        let id = self.next_battle;
        self.next_battle += 1;
        let seed = rng::key(&[self.seed, id as u64, 0xBA77]);
        let mut b = Battle::new(id, seed, self.time, Vec::new(), Vec::new());
        self.battles.push(b.clone());
        for (side, who) in sides {
            self.add_fighters(&mut b, side, &who);
        }
        let i = self.battles.iter().position(|x| x.id == id).unwrap();
        self.battles[i] = b;
        id
    }

    fn join_battle(&mut self, id: u32, side: Side, who: &[PersonId]) {
        let Some(i) = self.battles.iter().position(|b| b.id == id) else { return };
        let mut b = self.battles[i].clone();
        self.add_fighters(&mut b, side, who);
        self.battles[i] = b;
    }

    fn add_fighters(&mut self, b: &mut Battle, side: Side, who: &[PersonId]) {
        for &pid in who {
            self.people[pid as usize].ensure_detail();
            let pos = self.person_pos(pid);
            let f = Fighter::from_person(&self.people[pid as usize], side, pos, self.time);
            b.names.push(self.people[pid as usize].name().unwrap_or("someone").to_string());
            b.fighters.push(f);
            self.fighting.insert(pid, b.id);
        }
    }

    /// Write everything that happened in a finished fight back into the world.
    fn conclude(&mut self, b: Battle) {
        let t = b.time;
        let mut killed = 0;
        for f in &b.fighters {
            self.fighting.remove(&f.pid);
            let p = &mut self.people[f.pid as usize];
            let base = p.stats.clone();
            p.wounds.set(&base, &f.hp, t);
            p.set_mana(f.mana, t);
            for (k, amt) in f.trained.iter().enumerate() {
                if *amt > 0.0 {
                    p.stats.exercise(SKILLS[k], *amt);
                }
            }
            p.stats.harden(f.damage_taken);
            if f.dead {
                p.dead = true;
                killed += 1;
                self.busy_until[f.pid as usize] = f64::INFINITY;
                self.corpses.push((f.pos, p.race, t, f.pid));
            }
            p.recompute_might();
        }

        // Survivors' groups settle where the fight left them; wiped-out groups end.
        let touched: Vec<GroupId> = self.groups.iter().filter(|g| g.members.iter().any(|m| b.index_of(*m).is_some())).map(|g| g.id).collect();
        for gid in touched {
            let alive: Vec<PersonId> = self.group(gid).unwrap().members.iter().copied().filter(|m| !self.people[*m as usize].dead).collect();
            let gi = self.groups.iter().position(|g| g.id == gid).unwrap();
            if alive.is_empty() {
                self.groups[gi].ends = t;
                continue;
            }
            let c = alive.iter().filter_map(|m| b.index_of(*m)).map(|i| b.fighters[i].pos).fold(V2::default(), |a, p| a.add(p)).scale(1.0 / alive.len() as f32);
            let g = &mut self.groups[gi];
            g.members = alive;
            if g.hostile {
                g.legs = vec![Leg::wait(c, f64::INFINITY)];
                g.pos = c;
            }
        }

        // Squad members carry on from where the fight left them.
        for f in b.fighters.iter().filter(|f| f.side == SQUAD_SIDE && !f.dead) {
            if let Some(k) = self.squad.index(f.pid) {
                self.squad.at[k] = f.pos;
                self.squad.goal[k] = f.pos;
            }
        }
        // The dead drop everything they had, for whoever wants it.
        for f in b.fighters.iter().filter(|f| f.dead) {
            self.drop_everything(f.pid, f.pos);
        }
        let dead_squad: Vec<PersonId> = self.squad.members.iter().copied().filter(|m| self.people[*m as usize].dead).collect();
        let people = &self.people;
        self.squad.retain(|m| !people[m as usize].dead);
        self.pickups.retain(|p| !dead_squad.contains(&p.who));
        self.recentre_squad();
        for m in dead_squad {
            let name = self.people[m as usize].name().unwrap_or("someone").to_string();
            self.log.push_front((t, format!("{name} of your squad has died.")));
        }
        let line = if killed > 0 { format!("The fight is over. {killed} dead.") } else { "The fight is over.".to_string() };
        self.log.push_front((t, line));
        self.log.truncate(14);
    }
}
