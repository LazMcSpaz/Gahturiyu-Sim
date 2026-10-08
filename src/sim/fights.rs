//! Where fights meet the world: who starts one, who joins, and what is left
//! afterwards.
//!
//! Bandits attack your squad when they notice someone in it (see
//! `stealth.rs`), and you can attack them first. A fight runs as a `Battle` until one side is
//! down, dead or gone; then everything that happened in it is written back to
//! the people involved: wounds (which heal over time), mana spent, skills
//! trained, and the dead.

use super::combat::{Battle, Fighter, Order, Side, SQUAD_SIDE};
use super::effects::Does;
use super::geo::V2;
use super::group::{Group, GroupId, Kind, Leg};
use super::person::{Person, PersonId};
use super::race::ALL_RACES;
use super::rng::{self, Rng};
use super::stats::{Calling, Skill, SKILLS};
use super::world::World;

/// Share of arrows and bolts found again after a fight.
pub const AMMO_FOUND: f32 = 0.5;
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

    /// Not dead, not down, not already in a fight.
    pub(super) fn fit_to_fight(&self, pid: PersonId) -> bool {
        let p = &self.people[pid as usize];
        !p.dead && !self.fighting.contains_key(&pid) && !super::body::knocked_out(&p.wounds.hp_at(&p.stats, self.time))
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

    /// Some of the squad go for someone. Mid-fight that's an order; otherwise
    /// it starts a fight with them and their band (bandits only, for now). If
    /// their band hasn't noticed the squad, they're caught unawares for a
    /// moment — longer if everyone going in is sneaking.
    pub fn attack(&mut self, who: &[PersonId], enemy: PersonId) -> bool {
        if self.squad_battle().is_some() {
            return self.order_attack_with(who, enemy);
        }
        let Some(gid) = self.group_of[enemy as usize] else { return false };
        let Some(g) = self.group(gid) else { return false };
        if !g.hostile || self.fighting_groups.contains(&gid) || !self.fit_to_fight(enemy) {
            return false;
        }
        let them: Vec<PersonId> = g.members.iter().copied().filter(|&m| self.fit_to_fight(m)).collect();
        let fit = self.squad_fit();
        if fit.is_empty() || them.is_empty() {
            return false;
        }
        let surprise = if self.has_noticed(gid) {
            0.0
        } else if who.iter().all(|&m| self.is_sneaking(m)) {
            5.0
        } else {
            2.0
        };
        let t = self.time;
        let id = self.start_battle(vec![(SQUAD_SIDE, fit), (1, them)], t);
        self.fighting_groups.insert(gid);
        if let Some(b) = self.battles.iter_mut().find(|b| b.id == id) {
            for f in b.fighters.iter_mut().filter(|f| f.side != SQUAD_SIDE) {
                f.aware_at = t + surprise;
            }
        }
        let line = if surprise > 0.0 { "You fall on them before they know it." } else { "You attack." };
        self.log.push_front((t, line.to_string()));
        self.order_attack_with(who, enemy)
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

    /// Put a band of bandits at `at`, met already (named and kitted). One of
    /// them is a mage if asked. They make camp there. Returns the new group.
    pub fn spawn_bandits(&mut self, at: V2, count: usize, with_mage: bool) -> GroupId {
        self.add_bandits(at, count, with_mage, true)
    }

    /// A bandit camp at `at`. `met` builds their details now (otherwise
    /// that waits until someone comes close, like anyone else).
    pub(super) fn add_bandits(&mut self, at: V2, count: usize, with_mage: bool, met: bool) -> GroupId {
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
                p.specialize(Calling::Mage, &[(Skill::Structured, 45.0), (Skill::Felt, 30.0)], 80.0);
            } else if k % 3 == 1 {
                // An archer.
                p.specialize(Calling::Hunter, &[(Skill::Marksman, 40.0 + r.f32() * 15.0), (Skill::Dodge, 20.0)], 120.0 + r.f32() * 120.0);
            } else {
                let main = *r.pick(&[Skill::Blade, Skill::Blunt, Skill::Spear]);
                p.specialize(Calling::Warrior, &[(main, 30.0 + r.f32() * 18.0), (Skill::Dodge, 20.0), (Skill::Block, 18.0)], 90.0 + r.f32() * 220.0);
            }
            if met {
                p.ensure_detail();
                self.stats.detailed += 1;
            }
            p.wounds.at = self.time;
            p.mana_at = self.time;
            self.people.push(p);
            self.busy_until.push(f64::INFINITY);
            self.group_of.push(None);
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
        self.camps.push(super::encounters::Camp { group: gid, pos: at, ready_at: self.time, doused_until: 0.0 });
        self.scan_camp(self.camps.len() - 1);
        gid
    }

    /// Start fights that are due, run the ones in progress, settle the ones
    /// that are over.
    pub(super) fn update_battles(&mut self) {
        let t = self.time;
        self.corpses.retain(|c| t - c.2 < CORPSE_TIME);

        // Bandits who notice someone in the squad attack it, or join the
        // fight it's already in.
        let noticed = self.update_watchers();
        let squad_fit = self.squad_fit();
        if !squad_fit.is_empty() {
            for (gid, when) in noticed {
                let fresh: Vec<PersonId> = match self.group(gid) {
                    Some(g) => g.members.iter().copied().filter(|m| self.fit_to_fight(*m)).collect(),
                    None => continue,
                };
                if fresh.is_empty() {
                    continue;
                }
                // A sanctuary: they won't come for anyone sheltering in it.
                let open: Vec<PersonId> = squad_fit.iter().copied().filter(|&m| self.wards_at(Does::Sanctuary, self.person_pos(m)).next().is_none()).collect();
                if open.is_empty() {
                    // They hold off, but they'll be watching when you leave it.
                    for (k, v) in self.suspicion.iter_mut() {
                        if k.0 == gid {
                            *v = v.min(0.9);
                        }
                    }
                    continue;
                }
                match self.squad.members.iter().find_map(|m| self.fighting.get(m)).copied() {
                    Some(bid) => self.join_battle(bid, 1, &fresh),
                    None => {
                        let line = "You're attacked!".to_string();
                        self.alerts.push(line.clone());
                        self.log.push_front((when, line));
                        let fit = self.squad_fit();
                        let id = self.start_battle(vec![(SQUAD_SIDE, fit), (1, fresh)], when);
                        // A tripwire round them: sleepers are up at once.
                        let tripped = self.squad.members.iter().any(|&m| self.wards_at(Does::Tripwire, self.person_pos(m)).next().is_some());
                        if tripped {
                            if let Some(b) = self.battles.iter_mut().find(|b| b.id == id) {
                                for f in b.fighters.iter_mut().filter(|f| f.side == SQUAD_SIDE) {
                                    f.aware_at = f.aware_at.min(when);
                                }
                            }
                            self.log.push_front((when, "The tripwire sings — everyone's up!".to_string()));
                        }
                    }
                }
                self.fighting_groups.insert(gid);
            }
        }

        for b in &mut self.battles {
            b.advance_to(t);
        }
        // Fights away from the squad are only copies on show; their ends are
        // handled on the timeline.
        let npc: Vec<u32> = self.npc_fights.iter().map(|f| f.id).collect();
        let (done, live): (Vec<Battle>, Vec<Battle>) = std::mem::take(&mut self.battles).into_iter().partition(|b| b.over && !npc.contains(&b.id));
        self.battles = live;
        for b in done {
            self.conclude(b);
        }
    }

    fn start_battle(&mut self, sides: Vec<(Side, Vec<PersonId>)>, at: f64) -> u32 {
        let id = self.next_battle;
        self.next_battle += 1;
        let seed = rng::key(&[self.seed, id as u64, 0xBA77]);
        let mut b = Battle::new(id, seed, at, Vec::new(), Vec::new());
        b.lights = Some(self.fixed_lights());
        self.battles.push(b.clone());
        for (side, who) in sides {
            self.add_fighters(&mut b, side, &who);
        }
        // The squad's wards round the fight come into it.
        let centre = self.squad.pos;
        // (Their light is already among the fixed lights.)
        for w in self.wards.iter().filter(|w| w.until > at && w.pos.dist(centre) <= w.radius + 80.0 && w.does != Does::Glow) {
            b.zones.push(super::combat::Zone { does: w.does, pos: w.pos, radius: w.radius, power: w.power, until: w.until, side: SQUAD_SIDE, owner: w.owner, fresh: false });
        }
        // The dead lying nearby (not your own: they're never raised) are
        // there to be raised.
        let graves: Vec<(V2, PersonId)> = self.corpses.iter().filter(|c| c.0.dist(centre) <= 40.0 && !self.people[c.3 as usize].in_squad).map(|c| (c.0, c.3)).collect();
        for (pos, pid) in graves {
            let mut f = Fighter::from_person(&self.people[pid as usize], super::combat::GRAVE_SIDE, pos, at);
            f.dead = true;
            f.ko = true;
            b.names.push(self.people[pid as usize].name().unwrap_or("someone").to_string());
            b.fighters.push(f);
        }
        // A guardian left waiting near the fight comes into it (once).
        let waiting: Vec<super::casting::Ward> = self.wards.iter().copied().filter(|w| matches!(w.does, Does::Summon(_)) && w.until > at && w.pos.dist(centre) <= w.radius).collect();
        for w in waiting {
            if let Does::Summon(kind) = w.does {
                let owner = self.people.get(w.owner as usize).and_then(|p| p.name()).unwrap_or("Someone").to_string();
                let race = self.people.get(w.owner as usize).map(|p| p.race).unwrap_or(super::race::Race::Roduro);
                b.call_up_for(SQUAD_SIDE, race, &owner, kind, w.pos, w.until, w.power);
            }
            self.wards.retain(|x| *x != w);
        }
        b.zones.retain(|z| !matches!(z.does, Does::Summon(_)));
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
            let mut f = Fighter::from_person(&self.people[pid as usize], side, pos, self.time);
            // Woken by the attack: a few seconds before they're on their feet.
            if self.is_asleep(pid) {
                f.aware_at = b.start + 3.0;
            }
            if let Some(k) = self.squad.index(pid) {
                self.squad.resting[k] = false;
            }
            f.torch = self.torch_lit(pid);
            f.has_torch = self.torch_in_hand(pid).is_some();
            f.held = self.held.get(&pid).copied();
            // Spells on them come into the fight.
            for bn in self.boons_on(pid) {
                f.statuses.push(super::magic::Status { does: bn.does, power: bn.power, until: bn.until });
            }
            // Carrying someone: can't fight, and slow.
            if self.carrying(pid).is_some() {
                f.burdened = true;
                f.base_speed *= self.carry_pace(pid);
            }
            b.names.push(self.people[pid as usize].name().unwrap_or("someone").to_string());
            b.fighters.push(f);
            self.fighting.insert(pid, b.id);
        }
    }

    /// Write what a finished fight did to everyone in it back onto the people:
    /// wounds, mana, skills trained, toughening, deaths (and the dead's
    /// belongings onto the ground). Returns how many died.
    pub(super) fn write_back(&mut self, b: &Battle) -> usize {
        let t = b.time;
        let mut killed = 0;
        // Lasting spells the squad worked on the ground stay after the fight.
        for z in b.zones.iter().filter(|z| z.fresh && z.side == SQUAD_SIDE && z.until > t) {
            self.wards.push(super::casting::Ward { does: z.does, pos: z.pos, radius: z.radius, power: z.power, until: z.until, owner: z.owner });
        }
        // (Called-up creatures and raised dead are nobody: nothing to write.)
        // A body raised and spent crumbles away.
        for f in b.fighters.iter().filter(|f| f.home == super::combat::GRAVE_SIDE && f.raised) {
            self.corpses.retain(|c| c.3 != f.pid);
        }
        for f in b.fighters.iter().filter(|f| f.is_person() && f.home != super::combat::GRAVE_SIDE) {
            self.fighting.remove(&f.pid);
            let p = &mut self.people[f.pid as usize];
            let base = p.stats.clone();
            p.wounds.set(&base, &f.hp, t);
            for k in 0..6 {
                p.wounds.missing[k] |= f.missing[k];
            }
            p.set_mana(f.mana, t);
            for (k, amt) in f.trained.iter().enumerate() {
                if *amt > 0.0 {
                    p.stats.exercise(SKILLS[k], *amt);
                }
            }
            p.stats.harden(f.damage_taken);
            // Felt magic comes with use: new felt spells as the feel grows.
            // (Your squad only: everyone else's spells are fixed by their
            // stats when their kit was chosen, so meeting them or not never
            // changes what they can do.)
            if let Some(d) = p.detail.as_mut().filter(|_| p.in_squad) {
                let new = super::magic::felt_reached(&p.stats, &d.spells);
                d.spells.extend(new.iter().copied());
                if p.in_squad {
                    for sp in new {
                        let line = format!("{} has a feel for {} now.", d.name, sp.def().name.to_lowercase());
                        self.log.push_front((t, line));
                    }
                }
            }
            let p = &mut self.people[f.pid as usize];
            let (pid, fatigue, tire) = (f.pid, f.fatigue, f.tire);
            // Potions drunk and scrolls read are gone; arrows loosed are gone
            // too, except the ones found again afterwards.
            if let Some(d) = p.detail.as_mut() {
                for &it in &f.used {
                    d.gear.take(it);
                }
                // Shattered gear is gone.
                for &slot in &f.broke {
                    d.gear.discard(slot);
                }
                let ammo = f.stowed.map(|w| w.0.ammo).unwrap_or(f.weapon.ammo);
                if let (Some(key), true) = (ammo, f.shots > 0) {
                    let id = super::items::id(key);
                    for _ in 0..f.shots {
                        d.gear.take(id);
                    }
                    let found = (0..f.shots).filter(|&k| Rng::from_keys(&[b.seed, f.pid as u64, k as u64, 0xA770]).f32() < AMMO_FOUND).count() as u16;
                    if found > 0 {
                        d.gear.add(id, found);
                    }
                }
            }
            if f.dead {
                p.dead = true;
                killed += 1;
                self.busy_until[f.pid as usize] = f64::INFINITY;
                // A body raised and spent in the fight is gone.
                if !f.raised {
                    self.corpses.push((f.pos, p.race, t, f.pid));
                }
                // Their things stay where they fell, so they need to exist.
                if p.ensure_detail() {
                    self.stats.detailed += 1;
                }
                self.drop_everything(f.pid, f.pos);
            }
            self.people[f.pid as usize].recompute_might();
            // Spells on a squad member go back out with them (including ones
            // cast in the fight that haven't run out yet).
            if self.squad.index(pid).is_some() {
                self.boons.retain(|bn| bn.pid != pid);
                for s in f.statuses.iter().filter(|s| s.until > t) {
                    self.boons.push(super::casting::Boon { pid, does: s.does, power: s.power, until: s.until });
                }
            }
            // A held ritual let go in the fight is gone (and so is one held by
            // someone who died).
            if (f.held.is_none() || f.dead) && self.held.contains_key(&pid) {
                self.set_holding(pid, t, None);
            }
            // A torch lit or put out by magic in the fight stays that way.
            if self.squad.index(pid).is_some() && !f.dead && f.torch != self.torch_lit(pid) && self.torch_in_hand(pid).is_some() {
                self.toggle_torch(pid);
            }
            self.after_fight(pid, t, fatigue, tire);
        }
        killed
    }

    /// Write everything that happened in a finished fight back into the world.
    fn conclude(&mut self, b: Battle) {
        let t = b.time;
        let killed = self.write_back(&b);

        // Survivors' groups settle where the fight left them; wiped-out groups end.
        let touched: Vec<GroupId> = self.groups.iter().filter(|g| g.members.iter().any(|m| b.index_of(*m).is_some())).map(|g| g.id).collect();
        let squad_won = b.winner() == Some(SQUAD_SIDE);
        for gid in touched {
            self.fighting_groups.remove(&gid);
            if squad_won && self.camps.iter().any(|c| c.group == gid) {
                self.beaten_camps.insert(gid);
            }
            // They've had their fight; they'll need to spot you again.
            self.suspicion.retain(|(g, _), _| *g != gid);
            let alive: Vec<PersonId> = self.group(gid).unwrap().members.iter().copied().filter(|m| !self.people[*m as usize].dead).collect();
            let gi = self.groups.iter().position(|g| g.id == gid).unwrap();
            if alive.is_empty() {
                self.groups[gi].ends = t;
                self.camps.retain(|cp| cp.group != gid);
                continue;
            }
            let c = alive.iter().filter_map(|m| b.index_of(*m)).map(|i| b.fighters[i].pos).fold(V2::default(), |a, p| a.add(p)).scale(1.0 / alive.len() as f32);
            let camp = self.camps.iter().position(|cp| cp.group == gid);
            let g = &mut self.groups[gi];
            g.members = alive;
            if g.hostile {
                match camp {
                    // Back to camp to lick their wounds.
                    Some(ci) => {
                        let home = self.camps[ci].pos;
                        let walk = Leg::straight(c, home, t, g.speed, None, &self.terrain);
                        let at = walk.arrive;
                        g.legs = vec![walk, Leg::stay(home, at, f64::INFINITY)];
                        self.camps[ci].ready_at = at + super::encounters::CAMP_REST;
                    }
                    None => g.legs = vec![Leg::wait(c, f64::INFINITY)],
                }
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
