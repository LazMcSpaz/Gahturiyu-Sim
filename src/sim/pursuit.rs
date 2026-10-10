//! A crime seen is not yet a crime punished (Laz, 2026-10-09).
//!
//! Someone has to see it (`World::spotter`: the townsperson sharing the space
//! most likely to notice), then choose to tell the watch, and then a guard has
//! to catch the one who did it.
//!
//! **Telling.** Most people do (`REPORT_BASE`). A witness who likes the thief,
//! who has few scruples, who runs with the town's ring, or who bears the
//! victim a grudge may keep quiet; the victim's own household almost never
//! does. One keyed roll per witness, thief and minute decides
//! (`report_chance` gives the odds). Kept quiet, nothing follows.
//!
//! **Catching.** The witness runs to the nearest guard on watch, and that
//! guard sets off for where the thief was last seen, running a little faster
//! than anyone walks (`GUARD_PACE`). While the guard can see the thief (the
//! same sight rules as a witness: light, sneaking, walls) they head for them;
//! out of sight they go to the last place they saw them and look about. Close
//! enough and the thief is caught and judged (`World::judge`). Lost for
//! `LOSE_AFTER`, out of town, or after `GIVE_UP`, the guard gives up and the
//! crime becomes a bounty, whose news travels as before. With no guard on
//! watch (or an unpaid watch that won't stir) it's a bounty straight away.
//!
//! The chase is the squad's own (the player exception): the guard is moved
//! step by step, like the squad. A guard on a chase stands where the chase
//! has them (`World::person_pos`).

use serde::{Deserialize, Serialize};

use super::containers::Owner;
use super::geo::V2;
use super::law::Wrong;
use super::person::PersonId;
use super::rng::Rng;
use super::settlement::SettlementId;
use super::jobs::Job;
use super::world::World;

/// A bounty is the fine and half again: running and paying later costs
/// more than standing to be judged.
pub const BOUNTY_MARKUP: f32 = 1.5;

/// How likely an ordinary witness is to tell the watch.
pub const REPORT_BASE: f32 = 0.85;
/// How fast a guard runs after a thief, m/s (the squad walks at 1.5).
pub const GUARD_PACE: f32 = 2.4;
/// How fast a witness runs to fetch the watch, m/s.
pub const WITNESS_PACE: f32 = 2.6;
/// The longest a witness takes to reach a guard, s.
pub const FETCH_MAX: f64 = 150.0;
/// Close enough to lay a hand on someone, m.
pub const CATCH: f32 = 1.6;
/// Out of sight this long, a guard gives up, s.
pub const LOSE_AFTER: f64 = 120.0;
/// No chase lasts longer than this, s.
pub const GIVE_UP: f64 = 20.0 * 60.0;
/// A thief this far beyond the town's edge has got away, m.
pub const BEYOND: f32 = 150.0;

/// A guard after a squad member.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Pursuit {
    pub guard: PersonId,
    pub culprit: PersonId,
    pub town: SettlementId,
    pub wrong: Wrong,
    pub fine: f32,
    /// Who told the watch (none for a fight the town saw for itself).
    pub witness: Option<PersonId>,
    /// When the guard sets off (once the witness has reached them).
    pub start: f64,
    /// Where the guard is, once they've set off.
    pub pos: Option<V2>,
    /// Where and when they last saw the thief.
    pub last_seen: V2,
    pub seen_t: f64,
}

impl World {
    /// Who, of the townsfolk sharing the space, is likeliest to see `who` at
    /// `at`, and how likely. Walls hide: only those in the same building (or
    /// both outside) count; sleepers notice only close by, and seldom.
    pub fn spotter(&self, who: PersonId, at: V2, town: SettlementId) -> (f32, Option<PersonId>) {
        // No town's eyes out in the wild.
        if town == super::containers::WILD {
            return (0.0, None);
        }
        let sight = super::stealth::SIGHT * 0.6 * self.visibility_of(who);
        let room = self.building_at(at).map(|d| d.id);
        let mut best = (0.0f32, None);
        for p in self.residents_in_band1(town) {
            // A guard already running someone down isn't looking about.
            if self.pursuits.iter().any(|x| x.guard == p) {
                continue;
            }
            let pos = self.person_pos(p);
            if self.building_at(pos).map(|d| d.id) != room {
                continue;
            }
            let d = pos.dist(at);
            let asleep = self.is_indoors_asleep(p);
            if d > sight || (asleep && d > 6.0) {
                continue;
            }
            let c = (0.6 * (1.0 - d / sight.max(0.1)) + 0.2) * if asleep { 0.35 } else { 1.0 };
            if c > best.0 {
                best = (c, Some(p));
            }
        }
        best
    }

    /// How likely `witness` is to tell the watch what `culprit` did to
    /// `owner`'s things (or to whoever, with no owner).
    pub fn report_chance(&self, witness: PersonId, culprit: PersonId, owner: Option<Owner>) -> f32 {
        if witness as usize >= self.society.lives.len() {
            return REPORT_BASE;
        }
        let life = self.life(witness);
        let mine = matches!((owner, life.household), (Some(Owner::Household(h)), Some(w)) if h == w);
        if mine {
            return 0.97;
        }
        let mut p = REPORT_BASE;
        // A friend of the thief looks away.
        let regard = self.regard_of(witness, culprit);
        if regard > 65.0 {
            p -= (regard - 65.0) / 35.0 * 0.55;
        }
        // Few scruples, little wish to help the watch.
        p -= (0.45 - life.habits.honour).max(0.0) * 0.6;
        // Thieves don't turn in thieves.
        if self.society.rings.iter().any(|r| r.members.contains(&witness)) {
            p -= 0.45;
        }
        // The guard is the law; a guard who sees it tells themselves.
        if life.job == Job::Guard {
            p += 0.1;
        }
        // Bad blood with the victim, or warmth toward them.
        if let (Some(Owner::Household(h)), Some(w)) = (owner, life.household) {
            if let Some(f) = self.feeling(w, h) {
                if f.warmth < 0.0 || f.stage >= 1 {
                    p -= 0.15 + 0.08 * f.stage as f32;
                } else if f.warmth > 0.3 {
                    p += 0.08;
                }
            }
        }
        p.clamp(0.05, 0.97)
    }

    /// A wrong a townsperson saw (or, with no witness, one the town saw for
    /// itself, as in a fight in the street): told or not, and if told, the
    /// watch is sent. `at` is where it happened.
    pub fn wrong_seen(&mut self, who: PersonId, town: SettlementId, wrong: Wrong, fine: f32, line: String, witness: Option<PersonId>, owner: Option<Owner>, at: V2) {
        let t = self.time;
        // In disguise, no one knows whose crime it was.
        if self.boon(who, super::effects::Does::Disguise) > 0.0 {
            self.say(t, format!("{line} No one knows who it was."));
            self.alerts.push(line);
            return;
        }
        if let Some(wi) = witness {
            let mut r = Rng::from_keys(&[self.seed, wi as u64, who as u64, (t / 60.0) as u64, 0x5445_4C4C]);
            if r.f32() >= self.report_chance(wi, who, owner) {
                let name = self.name_of(wi);
                self.say(t, format!("{line} {name} saw it, but says nothing."));
                self.alerts.push(format!("{line} {name} saw it, but says nothing."));
                return;
            }
        }
        // Told: the town knows.
        self.add_standing(who, town, -fine / 4.0);
        self.society.towns[town as usize].gov.wrongs += 1.0;
        if wrong.always_recorded() {
            *self.records.entry(who).or_insert(0.0) += wrong.gravity();
        }
        if self.pursuits.iter().any(|p| p.culprit == who && p.town == town) {
            // The watch is already after them: this goes on the charge.
            if let Some(p) = self.pursuits.iter_mut().find(|p| p.culprit == who && p.town == town) {
                p.fine += fine;
                if wrong.gravity() > p.wrong.gravity() {
                    p.wrong = wrong;
                }
            }
            self.say(t, line.clone());
            self.alerts.push(line);
            return;
        }
        let in_duel = self.duels.iter().any(|d| d.accused == who) || self.fighting.contains_key(&who);
        let guard = if self.is_bonded(who, t) || in_duel { None } else { self.guard_for(town, witness.map(|w| self.person_pos(w)).unwrap_or(at), who) };
        match guard {
            Some(g) => {
                let from = witness.map(|w| self.person_pos(w)).unwrap_or(at);
                let fetch = match witness {
                    Some(_) => (from.dist(self.person_pos(g)) / WITNESS_PACE) as f64 + 10.0,
                    None => 20.0,
                }
                .min(FETCH_MAX);
                self.pursuits.push(Pursuit { guard: g, culprit: who, town, wrong, fine, witness, start: t + fetch, pos: None, last_seen: at, seen_t: t + fetch });
                let told = match witness {
                    Some(w) => format!("{line} {} runs to fetch the watch!", self.name_of(w)),
                    None => format!("{line} Someone runs to fetch the watch!"),
                };
                self.say(t, told.clone());
                self.alerts.push(told);
            }
            None => self.post_bounty(who, town, fine, line),
        }
    }

    /// The guard on watch nearest `from` who'll come (an unpaid watch stirs
    /// only now and then).
    fn guard_for(&self, town: SettlementId, from: V2, who: PersonId) -> Option<PersonId> {
        let t = self.time;
        let paid = self.society.towns[town as usize].owed <= 0.0;
        let mut r = Rng::from_keys(&[self.seed, who as u64, (t / 60.0) as u64, 0x4152_5354]);
        if !paid && !r.chance(0.3) {
            return None;
        }
        self.settlements[town as usize]
            .residents
            .iter()
            .copied()
            .filter(|&p| !self.people[p as usize].dead && self.people[p as usize].home == Some(town) && self.life(p).job == Job::Guard && self.at_work(p, t))
            .filter(|&p| !self.pursuits.iter().any(|x| x.guard == p))
            .min_by(|&a, &b| self.person_pos(a).dist(from).total_cmp(&self.person_pos(b).dist(from)).then(a.cmp(&b)))
    }

    /// The crime is known but the one who did it isn't held: a bounty, and
    /// the news of it sets off.
    pub(super) fn post_bounty(&mut self, who: PersonId, town: SettlementId, fine: f32, line: String) {
        let _ = who;
        *self.bounty.entry(town).or_insert(0.0) += fine * BOUNTY_MARKUP;
        self.crime_known(town);
        let total = self.bounty[&town];
        let t = self.time;
        self.say(t, format!("{line} Bounty in {}: {total:.0}.", self.settlements[town as usize].name));
        self.alerts.push(line);
    }

    /// Where a guard on a chase is, if they're on one and have set off.
    pub fn chase_pos(&self, pid: PersonId) -> Option<V2> {
        self.pursuits.iter().find(|p| p.guard == pid).and_then(|p| p.pos)
    }

    /// Who this squad member has the watch after them, if anyone.
    pub fn chased_by(&self, culprit: PersonId) -> Option<PersonId> {
        self.pursuits.iter().find(|p| p.culprit == culprit && p.pos.is_some()).map(|p| p.guard)
    }

    /// Can a guard at `from` see this squad member now?
    fn guard_sees(&self, from: V2, culprit: PersonId) -> bool {
        let at = self.person_pos(culprit);
        let sight = super::stealth::SIGHT * self.visibility_of(culprit);
        at.dist(from) <= sight && self.building_at(at).map(|d| d.id) == self.building_at(from).map(|d| d.id)
    }

    /// Each step: guards run thieves down, catch them, or give up.
    pub(super) fn chase(&mut self, dt: f64) {
        let t = self.time;
        let mut i = 0;
        while i < self.pursuits.len() {
            let p = self.pursuits[i].clone();
            if t < p.start {
                i += 1;
                continue;
            }
            let gone = self.people[p.guard as usize].dead || self.people[p.culprit as usize].dead || self.squad.index(p.culprit).is_none();
            // A fight with the guard is its own matter (resisting the watch).
            if gone || self.fighting.contains_key(&p.guard) {
                self.pursuits.remove(i);
                continue;
            }
            let mut pos = p.pos.unwrap_or_else(|| self.routine_pos(p.guard, p.start));
            let at = self.person_pos(p.culprit);
            let mut seen = (p.last_seen, p.seen_t);
            if self.guard_sees(pos, p.culprit) {
                seen = (at, t);
            }
            let to = seen.0.sub(pos);
            let step = GUARD_PACE * dt as f32;
            pos = if to.len() <= step { seen.0 } else { pos.add(to.scale(step / to.len())) };
            let reach = self.settlements[p.town as usize].reach;
            let out_of_town = at.dist(self.settlements[p.town as usize].pos) > reach + BEYOND;
            if pos.dist(at) <= CATCH && seen.1 == t {
                self.pursuits.remove(i);
                self.caught(p);
                continue;
            }
            if t - seen.1 > LOSE_AFTER || t - p.start > GIVE_UP || out_of_town {
                self.pursuits.remove(i);
                let name = self.name_of(p.culprit);
                let line = format!("{name} has slipped the watch in {}.", self.settlements[p.town as usize].name);
                self.post_bounty(p.culprit, p.town, p.fine, line);
                continue;
            }
            let x = &mut self.pursuits[i];
            x.pos = Some(pos);
            x.last_seen = seen.0;
            x.seen_t = seen.1;
            i += 1;
        }
    }

    /// Caught: judged by the custom of those wronged (the town on land).
    fn caught(&mut self, p: Pursuit) {
        let t = self.time;
        let (guard, name) = (self.name_of(p.guard), self.name_of(p.culprit));
        let line = format!("{guard} of the watch catches {name}.");
        self.say(t, line.clone());
        self.alerts.push(line);
        if self.is_bonded(p.culprit, t) || self.duels.iter().any(|d| d.accused == p.culprit) {
            return;
        }
        let custom = {
            let ci = self.society.towns[p.town as usize].shore;
            self.society.communities[ci as usize].customs.justice
        };
        self.judge(p.culprit, p.town, p.wrong, p.fine, custom);
    }
}
