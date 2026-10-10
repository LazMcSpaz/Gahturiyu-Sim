//! When animals and people meet: who attacks whom, when, and how it goes.
//!
//! **Travellers.** Once a game-hour, straight after that hour's departures
//! are planned, every dangerous herd's round for the hour is laid against
//! every traveller's schedule for the hour. The first moment a traveller
//! comes within the herd's senses becomes a `Notice` on the timeline. At
//! that moment the herd weighs them up — the pack's strength against
//! theirs with a keyed roll, the same way a bandit camp decides — and
//! either lets them pass or (a pack) falls in behind them and sets a
//! `Strike` for a few minutes later, or (an ambusher, a territorial beast)
//! strikes at once. The strike starts a fight that is fought to the end
//! there and then with the ordinary fight rules; it is filed with the
//! world's other far-off fights, so the travellers' side of its ending
//! (wounds, deaths, waiting for the fallen, turning for home) is the very
//! code that ends a bandit ambush.
//!
//! **Your squad.** The squad is the one thing in the world whose run-ins
//! depend on being looked at (as with bandits): the herds near it are
//! looked over every couple of seconds. Prey bolts. A pack sizes the squad
//! up once an hour and closes in or keeps its distance; ambushers,
//! territorial beasts and Briarbacks come for whoever steps in reach.
//!
//! **What the fight rules don't know about animals** is applied here, from
//! outside, as each fight is run (`advance_fray`):
//!
//! - *Nerve.* The rules give people nerve (they may break and run) but not
//!   creatures. Once a second of fight-time each animal still fighting
//!   checks whether it is badly hurt or badly outmatched and may turn tail.
//! - *Dying.* The rules judge death by a person's build. An animal is dead
//!   when its head or body is at minus its *own* most health, so a small
//!   thing dies of one good blow.
//! - *Dispel.* The rules send anything that isn't a person "back" when a
//!   Dispel lands on it. An animal is flesh and blood: it stays.
//!
//! Every copy of a fight (the one fought to its end, the one played out near
//! you) is advanced through that one function, so they agree.

use serde::{Deserialize, Serialize};

use super::super::body;
use super::super::combat::{Battle, Fighter, Zone, DT, SQUAD_SIDE};
use super::super::effects::Does;
use super::super::encounters::NpcFight;
use super::super::geo::{self, V2};
use super::super::group::{GroupId, Kind, Leg};
use super::super::magic::Status;
use super::super::person::PersonId;
use super::super::rng::{self, Rng};
use super::super::squad::formation;
use super::super::stealth;
use super::super::torch::TRAVEL_TORCH_DARK;
use super::super::world::{World, HOUR};
use super::herd::{fighter, Away, Hunt, Hurt};
use super::species::{Class, Sp, Toward};
use super::{AttackRecord, Due, What, ANIMAL_SIDE, ATTACK_LOG, NEAR, NO_CAMP, TRAVELLER_SIDE};

/// How finely an hour is looked through for a hunter and a traveller
/// coming together, game seconds: half the time a walker takes to cross the
/// hunter's senses, but never coarser than `SCAN_STEP` nor finer than
/// `SCAN_FINEST` (so a lurker that only notices what is on top of it isn't
/// walked straight past between two looks).
pub const SCAN_STEP: f64 = 30.0;
pub const SCAN_FINEST: f64 = 5.0;
/// How long a herd leaves people alone after a fight.
pub const PACK_REST: f64 = 10.0 * HOUR;
/// How long a hunter that thought better of it stays put off.
pub const LOSE_INTEREST: f64 = 2.0 * HOUR;
/// How much stronger than its quarry a pack wants to be (after its roll).
pub const PACK_CAUTION: f32 = 1.25;
/// Share of nights on which a pack is bold enough to go for people at all
/// (the rest it keeps to its own prey).
pub const PACK_BOLD_NIGHTS: f32 = 0.5;
/// An ambusher always goes for a party this small; a bigger one only
/// sometimes.
pub const AMBUSH_SMALL: usize = 2;
pub const AMBUSH_BOLD: f32 = 0.25;
/// Seconds before someone ambushed knows what has hold of them.
pub const AMBUSH_SURPRISE: f64 = 2.5;
/// How long a herd that holds the field stays on it.
pub const FEED_TIME: f64 = 40.0 * 60.0;
/// The most animals of one herd that come into a fight.
pub const MAX_FIGHTERS: usize = 10;
/// How many of a herd that fights only when cornered turn and fight.
pub const CORNERED: usize = 4;
/// Fight ticks between checks of animals' nerve (one second).
const NERVE_TICKS: u64 = 10;
/// Chance, per check, that an animal with reason to run does.
pub const FLEE_CHANCE: f32 = 0.35;
/// How far a routed or startled herd runs, metres, and how long it stays
/// wary afterwards.
pub const FLEE_DIST: f32 = 280.0;
pub const WARY_TIME: f64 = 15.0 * 60.0;
/// How close the squad has to get to a herd to set about it, metres.
pub const HUNT_REACH: f32 = 40.0;
/// A pack that means to attack the squad closes to this before it does.
pub const PACK_CLOSE: f32 = 45.0;
/// Going quietly, the squad is noticed at this share of the usual distance.
pub const SNEAK_HALVES: f32 = 0.5;
/// Seconds a herd takes to react when set upon, by how it behaves.
fn reaction(toward: Toward) -> f64 {
    match toward {
        Toward::Calm => 4.0,
        Toward::Outruns => 0.8,
        Toward::Bolts => 1.0,
        _ => 1.5,
    }
}

/// Who an animal in a fight is.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Who {
    /// One of a wild herd, by the number that fixes its looks.
    Wild { herd: u32, ident: u16 },
    /// A tamed hound.
    Hound(u32),
}

/// How a fight left one animal.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PartEnd {
    pub dead: bool,
    pub ko: bool,
    pub fled: bool,
    pub hp: [f32; 6],
    pub max: [f32; 6],
    pub pos: V2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct FrayPart {
    /// Its place among the fight's fighters.
    pub fighter: u16,
    pub who: Who,
    pub sp: Sp,
    pub size: f32,
    /// Known from the start for a far-off fight (it is fought at once).
    pub end: Option<PartEnd>,
}

/// A fight with animals in it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Fray {
    pub battle: u32,
    pub at: V2,
    /// A far-off fight is already fought: this is when it ends.
    pub ends: Option<f64>,
    pub parts: Vec<FrayPart>,
    /// Whether the animals held the field at the end.
    pub animals_won: bool,
    /// Whether your squad did.
    pub squad_won: bool,
    /// Your squad started it (a hunt).
    pub hunted: bool,
    /// What the travellers attacked were carrying, as it stood before the
    /// fight. (The world's ending of a far-off fight marks beaten travellers
    /// robbed; animals take nothing, so this is put back.)
    pub cargo: Option<super::super::economy::Cargo>,
}

/// An animal in a fight, for drawing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighting {
    pub pos: V2,
    pub sp: Sp,
    pub size: f32,
    pub down: bool,
    pub fleeing: bool,
    pub tame: bool,
    pub vitality: f32,
}

/// Is a pack in the mood for people on the night that holds time `t`? One
/// keyed answer per pack per night (nights run noon to noon).
pub fn bold_tonight(world_seed: u64, herd: u32, t: f64) -> bool {
    let night = ((t - 12.0 * HOUR) / (24.0 * HOUR)).floor() as i64;
    Rng::from_keys(&[world_seed, herd as u64, night as u64, 0xB01D]).f32() < PACK_BOLD_NIGHTS
}

/// Does a herd of this kind go for these people? `roll` is 0..1.
///
/// - A pack attacks only what it is clearly stronger than: the weak and
///   the lone. A strong group is left alone whatever the roll.
/// - An ambusher takes a small party, and a larger one only sometimes.
/// - Territorial beasts and Briarbacks attack whoever is there.
pub fn would_attack(sp: Sp, pack_might: f32, their_might: f32, their_count: usize, roll: f32) -> bool {
    if their_count == 0 || pack_might <= 0.0 {
        return false;
    }
    match sp.def().toward {
        Toward::PackHunter => pack_might * (0.6 + 0.5 * roll) > their_might * PACK_CAUTION,
        Toward::Ambusher => their_count <= AMBUSH_SMALL || roll < AMBUSH_BOLD,
        Toward::Territorial | Toward::Hostile => true,
        _ => false,
    }
}

/// Run a fight with animals in it forward to time `to`, with what the fight
/// rules don't know about animals put right after every tick, and their
/// nerve checked once a second of fight-time. Every copy of such a fight
/// goes through here and nowhere else.
pub(super) fn advance_fray(b: &mut Battle, parts: &[FrayPart], to: f64) {
    while !b.over && b.time + DT <= to + 1e-9 {
        b.tick();
        if b.over {
            break;
        }
        flesh_and_blood(b, parts);
        if b.ticks % NERVE_TICKS == 0 {
            nerve(b, parts);
        }
    }
}

/// An animal is not a called-up creature: a Dispel doesn't send it anywhere
/// (the rules mark it gone; nothing else marks an animal gone without it
/// first running), and it dies when its head or body is at minus its own
/// most health, not a person's.
fn flesh_and_blood(b: &mut Battle, parts: &[FrayPart]) {
    for p in parts {
        let i = p.fighter as usize;
        let Some(f) = b.fighters.get_mut(i) else { continue };
        if f.dead {
            continue;
        }
        if f.fled && !f.fleeing {
            f.fled = false;
        }
        if f.hp[0] <= -f.max_hp[0] || f.hp[1] <= -f.max_hp[1] {
            f.dead = true;
            f.ko = true;
            let name = b.names[i].clone();
            b.log.push((b.time, format!("{name} is killed.")));
        }
    }
}

/// Animals with reason to run may run: prey as soon as it knows it's set
/// upon; the rest when badly hurt or badly outmatched.
fn nerve(b: &mut Battle, parts: &[FrayPart]) {
    let sec = b.ticks / NERVE_TICKS;
    for p in parts {
        let i = p.fighter as usize;
        let Who::Wild { .. } = p.who else { continue };
        let Some(f) = b.fighters.get(i) else { continue };
        if !f.active() || f.fleeing || f.unaware(b.time) {
            continue;
        }
        let d = p.sp.def();
        let run = if d.toward.flees() {
            true
        } else {
            let mine: f32 = b.fighters.iter().filter(|x| x.side == f.side && x.active()).map(|x| x.might).sum();
            let theirs: f32 = b.fighters.iter().filter(|x| x.side != f.side && x.active()).map(|x| x.might).sum();
            let hurt = d.body.nerve > 0.0 && f.vitality() < d.body.nerve;
            let losing = d.body.odds > 0.0 && theirs > mine * d.body.odds;
            (hurt || losing) && Rng::from_keys(&[b.seed, i as u64, sec, 0x4E45_5256]).f32() < FLEE_CHANCE
        };
        if run {
            b.fighters[i].fleeing = true;
            b.fighters[i].order = None;
            let name = b.names[i].clone();
            b.log.push((b.time, format!("{name} turns tail.")));
        }
    }
}

fn part_end(f: &Fighter) -> PartEnd {
    PartEnd { dead: f.dead, ko: f.ko, fled: f.fled, hp: f.hp, max: f.max_hp, pos: f.pos }
}

impl World {
    // ---- Finding hunters and travellers coming together -----------------------

    /// Lay every dangerous herd's round for game-hour `h` against every
    /// traveller's schedule for it, and note the first moment each herd
    /// has travellers within its senses.
    pub(super) fn watch_the_roads(&mut self, h: i64) {
        let (t0, t1) = (h as f64 * HOUR, (h + 1) as f64 * HOUR);
        let hunters: Vec<u32> = self
            .animals
            .herds
            .iter()
            .filter(|x| x.def().toward.dangerous() && x.hunt.is_none() && x.ready_at < t1 && x.alive(t0) > 0)
            .filter(|x| (0..12).any(|k| x.active(t0 + k as f64 * 300.0)))
            .map(|x| x.id)
            .collect();
        if hunters.is_empty() {
            return;
        }
        // Each traveller's track through the hour, a point every five
        // minutes, and the box it fits in.
        struct Track {
            gid: GroupId,
            lo: V2,
            hi: V2,
        }
        let mut tracks: Vec<Track> = Vec::new();
        for g in self.groups.iter().filter(|g| !g.hostile && g.ends > t0 && !self.fighting_groups.contains(&g.id)) {
            let (mut lo, mut hi) = (V2::new(f32::MAX, f32::MAX), V2::new(f32::MIN, f32::MIN));
            for k in 0..=12 {
                let p = g.position_at(t0 + k as f64 * 300.0);
                lo = V2::new(lo.x.min(p.x), lo.y.min(p.y));
                hi = V2::new(hi.x.max(p.x), hi.y.max(p.y));
            }
            tracks.push(Track { gid: g.id, lo, hi });
        }
        let mut found: Vec<Due> = Vec::new();
        for hid in hunters {
            let herd = &self.animals.herds[hid as usize];
            let d = herd.def();
            // (A walker covers a few hundred metres between track points.)
            let reach = d.range + d.sense + 500.0;
            let step = (d.sense as f64 / 2.0).clamp(SCAN_FINEST, SCAN_STEP);
            let steps = (HOUR / step).ceil() as usize;
            let mut best: Option<(f64, GroupId)> = None;
            for tr in &tracks {
                let c = herd.home;
                if c.x < tr.lo.x - reach || c.x > tr.hi.x + reach || c.y < tr.lo.y - reach || c.y > tr.hi.y + reach {
                    continue;
                }
                let Some(g) = self.group(tr.gid) else { continue };
                for k in 0..steps {
                    let t = t0 + k as f64 * step;
                    if best.map(|b| t > b.0).unwrap_or(false) {
                        break;
                    }
                    if t < herd.ready_at || t >= g.ends || !herd.active(t) || herd.mother_out(t) {
                        continue;
                    }
                    if herd.round_pos(t).dist(g.position_at(t)) <= d.sense {
                        if best.map(|b| (t, tr.gid) < b).unwrap_or(true) {
                            best = Some((t, tr.gid));
                        }
                        break;
                    }
                }
            }
            if let Some((t, victim)) = best {
                found.push(Due { t, what: What::Notice, herd: hid, victim, battle: 0 });
            }
        }
        self.animals.pending.extend(found);
    }

    /// Travellers able to fight at time `t`.
    fn fit_travellers(&self, gid: GroupId, t: f64) -> Vec<PersonId> {
        let Some(g) = self.group(gid) else { return Vec::new() };
        g.members
            .iter()
            .copied()
            .filter(|&m| {
                let p = &self.people[m as usize];
                !p.dead && !self.fighting.contains_key(&m) && !body::knocked_out(&p.wounds.hp_at(&p.stats, t))
            })
            .collect()
    }

    /// Whether these travellers can be attacked at time `t`, and where they are.
    fn quarry(&self, gid: GroupId, t: f64) -> Option<V2> {
        let g = self.group(gid)?;
        if g.hostile || self.fighting_groups.contains(&gid) || t >= g.ends {
            return None;
        }
        let at = g.position_at(t);
        // Nothing wild comes into a town after anyone.
        if self.settlements.iter().any(|s| s.pos.dist(at) < s.radius() + 40.0) {
            return None;
        }
        Some(at)
    }

    /// Would this herd, as it stands now, go for these people? (`roll` 0..1.)
    pub fn herd_would_attack(&self, herd: u32, people: &[PersonId], roll: f32) -> bool {
        let h = &self.animals.herds[herd as usize];
        let theirs: f32 = people.iter().map(|&m| self.people[m as usize].might).sum();
        would_attack(h.sp, h.might(self.time), theirs, people.len(), roll)
    }

    /// Is any of this herd in a fight right now?
    pub fn herd_in_fray(&self, herd: u32) -> bool {
        self.animals.frays.iter().any(|f| f.parts.iter().any(|p| matches!(p.who, Who::Wild { herd: h, .. } if h == herd)))
    }

    /// A herd's round has brought it within reach of travellers: it decides
    /// what to do about them.
    pub(super) fn pack_notices(&mut self, herd: u32, victim: GroupId, t: f64) {
        let h = &self.animals.herds[herd as usize];
        let d = h.def();
        if h.hunt.is_some() || t < h.ready_at || h.alive(t) == 0 || self.herd_in_fray(herd) {
            return;
        }
        if d.toward == Toward::PackHunter && !bold_tonight(self.seed, herd, t) {
            // Not tonight: it keeps to its own prey until tomorrow's dusk.
            let noon = ((t - 12.0 * HOUR) / (24.0 * HOUR)).floor() * 24.0 * HOUR + 36.0 * HOUR;
            self.animals.herds[herd as usize].ready_at = noon;
            return;
        }
        let Some(at) = self.quarry(victim, t) else { return };
        let from = h.round_pos(t);
        if from.dist(at) > d.sense + 5.0 {
            return; // the plan this was found on has since changed
        }
        let people = self.fit_travellers(victim, t);
        let theirs: f32 = people.iter().map(|&m| self.people[m as usize].might).sum();
        let mut r = Rng::from_keys(&[self.seed, herd as u64, victim as u64, t.to_bits(), 0xA41A]);
        let (roll, wait) = (r.f32(), r.f64());
        if !would_attack(h.sp, h.might(t), theirs, people.len(), roll) {
            self.animals.herds[herd as usize].ready_at = t + LOSE_INTEREST;
            self.animals.stats.left_alone += 1;
            return;
        }
        let strike = t + (d.stalk.0 as f64 + (d.stalk.1 - d.stalk.0) as f64 * wait) * 60.0;
        self.animals.herds[herd as usize].hunt = Some(Hunt { victim, seen: t, off: from.sub(at), strike });
        self.animals.pending.push(Due { t: strike, what: What::Strike, herd, victim, battle: 0 });
    }

    /// The pack closes. The fight is fought here and now, to the end.
    pub(super) fn pack_strikes(&mut self, herd: u32, victim: GroupId, t: f64) {
        // Only the stalk this strike was set for: if the pack has since been
        // in a fight of its own (your squad went for it, say) the stalk was
        // called off, and this comes to nothing.
        let live = self.animals.herds[herd as usize].hunt.map(|hu| hu.victim == victim && hu.strike == t).unwrap_or(false);
        if !live {
            return;
        }
        let from = self.herd_pos(herd, t);
        self.animals.herds[herd as usize].hunt = None;
        let people = self.fit_travellers(victim, t);
        let h = &self.animals.herds[herd as usize];
        let d = h.def();
        let slots: Vec<usize> = (0..h.alive(t)).filter(|&j| !h.down(j, t)).take(MAX_FIGHTERS).collect();
        // Called off if the travellers have got to safety or are fighting
        // someone else, if the pack's hours are over, or if it is busy.
        let (Some(at), false, false, true, false) = (self.quarry(victim, t), people.is_empty(), slots.is_empty(), h.active(t), self.herd_in_fray(herd)) else {
            self.animals.herds[herd as usize].ready_at = t + LOSE_INTEREST;
            return;
        };
        let (pack_might, their_might) = (h.might(t), people.iter().map(|&m| self.people[m as usize].might).sum::<f32>());

        // The fight, with everyone where they are: the animals coming in
        // from the side they were on.
        let id = self.animals.next_battle;
        self.animals.next_battle += 1;
        let seed = rng::key(&[self.seed, herd as u64, victim as u64, t.to_bits(), 0xBA77]);
        let toward = from.sub(at);
        let gap = toward.len();
        let start = if gap > 12.0 { at.add(toward.scale(12.0 / gap)) } else { from };
        let mut fighters = Vec::new();
        let mut names = Vec::new();
        let mut parts = Vec::new();
        for (k, &j) in slots.iter().enumerate() {
            parts.push(FrayPart { fighter: fighters.len() as u16, who: Who::Wild { herd, ident: h.ident(j) }, sp: h.sp, size: h.size(j, t), end: None });
            fighters.push(h.fighter(j, t, ANIMAL_SIDE, start.add(formation(k))));
            names.push(d.name.to_string());
        }
        // Travellers caught at night have their torch lit (the leader carries it).
        let lit = stealth::daylight(t) < TRAVEL_TORCH_DARK;
        let ambushed = d.toward == Toward::Ambusher;
        for (k, &m) in people.iter().enumerate() {
            let mut f = Fighter::from_person(&self.people[m as usize], TRAVELLER_SIDE, at.add(formation(k)), t);
            f.torch = lit && k == 0;
            if ambushed {
                f.aware_at = t + AMBUSH_SURPRISE;
            }
            fighters.push(f);
            names.push(self.name_of(m));
        }
        let mut b = Battle::new(id, seed, t, fighters, names);
        b.lights = Some(self.fixed_lights_at(t));
        for &m in &people {
            self.fighting.insert(m, id);
        }
        let mut result = b.clone();
        advance_fray(&mut result, &parts, f64::INFINITY);
        for p in parts.iter_mut() {
            p.end = Some(part_end(&result.fighters[p.fighter as usize]));
        }
        let animals_won = result.winner() == Some(ANIMAL_SIDE);
        let downed = result.fighters.iter().filter(|f| f.side == TRAVELLER_SIDE && f.is_person() && (f.ko || f.dead)).count();
        let killed = result.fighters.iter().filter(|f| f.side == TRAVELLER_SIDE && f.is_person() && f.dead).count();

        // The travellers' plans stop here for now (as in a bandit ambush).
        let gi = self.groups.iter().position(|g| g.id == victim).unwrap();
        let g = &mut self.groups[gi];
        let cargo = g.cargo.clone();
        let li = g.legs.iter().rposition(|l| t >= l.depart).unwrap_or(0);
        let was: Vec<Leg> = g.legs[li..].to_vec();
        let (cut, k) = if t < g.legs[li].arrive { g.legs[li].cut_at(t) } else { (Leg::stay(at, g.legs[li].arrive, t), 0) };
        let dropped = if matches!(g.kind, Kind::Wanderer { .. }) { (g.legs.len() - li - 1) as u64 } else { 0 };
        g.legs.truncate(li);
        if t < was[0].arrive {
            g.legs.push(cut);
        } else {
            g.legs.push(was[0].clone());
        }
        g.legs.push(Leg::stay(at, t, f64::INFINITY));
        g.written -= dropped;
        g.ends = f64::INFINITY;
        for &m in &g.members.clone() {
            self.busy_until[m as usize] = f64::INFINITY;
        }
        self.encounters.retain(|x| x.victim != victim || x.t <= t);
        // (Anything else of the animals' still due on these travellers is
        // left to come up: it finds them fighting, or gone, and lets go.)
        self.fighting_groups.insert(victim);

        // The herd is at the fight until it's over.
        let ends = result.time;
        let hd = &mut self.animals.herds[herd as usize];
        hd.away = Some(Away { from: start, to: start, depart: t, arrive: t, until: ends });
        hd.ready_at = ends + PACK_REST;

        self.animals.stats.attacks += 1;
        self.animals.stats.people_downed += downed as u32;
        self.animals.stats.people_killed += killed as u32;
        self.animals.attacks.push(AttackRecord {
            t,
            herd,
            sp: d.sp,
            victim: Some(victim),
            at,
            pack: slots.len() as u8,
            people: people.len() as u8,
            pack_might,
            their_might,
            battle: id,
            over: false,
            animals_won,
            by_squad: false,
            people_killed: killed.min(255) as u8,
            animals_lost: 0,
        });
        if self.animals.attacks.len() > ATTACK_LOG {
            self.animals.attacks.remove(0);
        }
        if self.bands.band_at(at) <= 2 {
            let what = if slots.len() > 1 { format!("{}s fall", d.name) } else { format!("A {} falls", d.name) };
            self.log.push_front((t, format!("{what} on travellers {:.0} m away.", at.dist(self.squad.pos))));
            self.log.truncate(14);
        }

        // Close by, a copy plays out in real time; it ends exactly as the
        // result already says.
        let mut show = b;
        advance_fray(&mut show, &parts, self.time);
        self.battles.push(show);
        self.npc_fights.push(NpcFight { id, ends, result, camp: NO_CAMP, victim, was, dropped, cut: k });
        self.animals.frays.push(Fray { battle: id, at, ends: Some(ends), parts, animals_won, squad_won: false, hunted: false, cargo });
        self.animals.pending.push(Due { t: ends, what: What::FrayEnd, herd, victim, battle: id });
    }

    /// A far-off fight is over: what it did to the animals in it.
    ///
    /// This runs straight after the world's own ending of the fight (which
    /// gets the travellers up and on their way, and marks beaten ones
    /// robbed, as bandits would have left them). Animals take nothing, so
    /// what the travellers carried is put back as it was.
    pub(super) fn far_fray_ends(&mut self, battle: u32, t: f64) {
        let Some(k) = self.animals.frays.iter().position(|f| f.battle == battle) else { return };
        let fray = self.animals.frays.remove(k);
        let victim = self.animals.attacks.iter().find(|a| a.battle == battle).and_then(|a| a.victim);
        if let (Some(cargo), Some(v)) = (&fray.cargo, victim) {
            if let Some(g) = self.groups.iter_mut().find(|g| g.id == v) {
                // (If none of them lived, what they carried is lost with them.)
                if !g.members.is_empty() && g.cargo.as_ref().map(|c| c.robbed && !cargo.robbed).unwrap_or(false) {
                    g.cargo = Some(cargo.clone());
                }
            }
        }
        self.settle_fray(&fray, t);
    }

    /// Write what a finished fight did to its animals back onto them:
    /// wounds, the dead (and their carcasses), and where each herd goes now.
    fn settle_fray(&mut self, fray: &Fray, t: f64) {
        let mut herds: Vec<u32> = fray.parts.iter().filter_map(|p| if let Who::Wild { herd, .. } = p.who { Some(herd) } else { None }).collect();
        herds.sort();
        herds.dedup();
        // After a hunt your squad won, prey left lying on the field is
        // finished off (that is what the hunt was for). Anything else left
        // down is left to get up again: see `finish_off`.
        let finish = fray.hunted && fray.squad_won;
        let mut lost = 0u32;
        for hid in herds {
            let mine: Vec<(u16, FrayPart, PartEnd)> = fray
                .parts
                .iter()
                .filter_map(|p| match (p.who, p.end) {
                    (Who::Wild { herd, ident }, Some(mut e)) if herd == hid => {
                        if finish && e.ko && !e.fled && p.sp.def().class == Class::Prey {
                            e.dead = true;
                        }
                        Some((ident, *p, e))
                    }
                    _ => None,
                })
                .collect();
            // Wounds first, then the dead (so places in the herd still mean
            // who they meant when the fight began).
            for (ident, _, e) in mine.iter().filter(|x| !x.2.dead) {
                let h = &mut self.animals.herds[hid as usize];
                if let Some(slot) = h.slot_of(*ident, t) {
                    let mut gone = [0.0; 6];
                    for i in 0..6 {
                        gone[i] = (e.max[i] - e.hp[i]).max(0.0);
                    }
                    h.set_hurt(slot, gone, t);
                }
            }
            for (ident, p, e) in mine.iter().filter(|x| x.2.dead) {
                let h = &mut self.animals.herds[hid as usize];
                if let Some(slot) = h.slot_of(*ident, t) {
                    h.remove(slot, t);
                    lost += 1;
                    self.leave_carcass(p.sp, p.size, e.pos, t, rng::key(&[hid as u64, *ident as u64]));
                }
            }
            let h = &mut self.animals.herds[hid as usize];
            h.hunt = None;
            h.ready_at = t + PACK_REST;
            let stood: Vec<V2> = mine.iter().filter(|x| !x.2.dead && !x.2.fled).map(|x| x.2.pos).collect();
            if h.alive(t) == 0 {
                h.away = None;
            } else if !stood.is_empty() {
                // Holding the field, or waiting by its fallen.
                let site = stood.iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / stood.len() as f32);
                let until = (t + FEED_TIME).max(h.all_up_at(t) + 600.0);
                h.away = Some(Away { from: site, to: site, depart: t, arrive: t, until });
            } else {
                // Routed: off the way it came, and wary for a while.
                let off = h.home.sub(fray.at);
                let dir = if off.len() > 30.0 { off.scale(1.0 / off.len()) } else { V2::new(1.0, 0.0) };
                let mut to = fray.at.add(dir.scale(FLEE_DIST.min(off.len().max(FLEE_DIST * 0.5))));
                if !geo::is_land(to) {
                    to = h.home;
                }
                let arrive = t + (fray.at.dist(to) / h.def().speed.max(1.0)) as f64;
                h.away = Some(Away { from: fray.at, to, depart: t, arrive, until: arrive + WARY_TIME });
            }
        }
        for p in &fray.parts {
            let (Who::Hound(id), Some(e)) = (p.who, p.end) else { continue };
            if let Some(h) = self.animals.hounds.iter_mut().find(|h| h.id == id) {
                let mut gone = [0.0; 6];
                for i in 0..6 {
                    gone[i] = (e.max[i] - e.hp[i]).max(0.0);
                }
                h.hurt = Hurt { lost: gone, at: t };
                if e.dead {
                    h.gone = true;
                    self.log.push_front((t, "Your hound is dead.".to_string()));
                    self.log.truncate(14);
                }
            }
        }
        self.animals.stats.animals_killed += lost;
        if let Some(a) = self.animals.attacks.iter_mut().find(|a| a.battle == fray.battle) {
            a.over = true;
            a.animals_lost = lost.min(255) as u8;
            if fray.ends.is_none() {
                a.animals_won = fray.animals_won;
            }
        }
    }

    // ---- Round the squad ---------------------------------------------------------

    /// Once a step: fights with animals in them go on; tamed hounds join
    /// their owner's fight; and every couple of seconds the herds near the
    /// squad are looked over.
    pub(super) fn animals_by_the_squad(&mut self) {
        // (Hounds first: a fight that ends this step is then settled with
        // them in it, and they are never added to one that is over.)
        self.hounds_join();
        self.advance_frays();
        let t = self.time;
        if t >= self.animals.looked_at && t - self.animals.looked_at < super::LOOK_EVERY {
            return;
        }
        self.animals.looked_at = t;
        let here = self.squad.pos;
        if self.animals.near_at.dist(here) > 300.0 {
            self.animals.near_at = here;
            self.animals.near = self.animals.herds.iter().filter(|h| h.home.dist(here) <= NEAR + h.def().range).map(|h| h.id).collect();
        }
        let fit = self.squad_fit();
        let fighting = self.squad_battle().is_some();
        let in_town = self.settlements.iter().any(|s| s.pos.dist(here) < s.radius() + 40.0);
        // Animals go by nose and ear as much as by eye, so the dark is no
        // cover; going quietly is.
        let seen = if !fit.is_empty() && fit.iter().all(|&m| self.is_sneaking(m)) { SNEAK_HALVES } else { 1.0 };
        let sheltered = !fit.is_empty() && fit.iter().all(|&m| self.wards_at(Does::Sanctuary, self.person_pos(m)).next().is_some());
        let their_might: f32 = fit.iter().map(|&m| self.people[m as usize].might).sum();
        for id in self.animals.near.clone() {
            let h = &self.animals.herds[id as usize];
            let alive = h.alive(t);
            if alive == 0 {
                continue;
            }
            let d = h.def();
            let pos = self.herd_pos(id, t);
            if !h.met && self.bands.band_at(pos) == 1 {
                self.animals.herds[id as usize].met = true;
                self.animals.stats.met += alive as u32;
            }
            let h = &self.animals.herds[id as usize];
            let dist = fit.iter().map(|&m| self.person_pos(m).dist(pos)).fold(f32::MAX, f32::min);
            // (A pack on its way in to the squad isn't busy: it is looked at
            // again each time, and re-aimed.)
            let closing = h.away.map(|a| a.until <= a.arrive).unwrap_or(false);
            let held = h.away.map(|a| t < a.until).unwrap_or(false) && !closing;
            if h.hunt.is_some() || held || fit.is_empty() || self.herd_in_fray(id) {
                continue;
            }
            if !d.toward.dangerous() {
                // Prey bolts from anyone it notices coming.
                if d.flight > 0.0 && dist < d.flight * seen {
                    self.herd_runs(id, here, t);
                }
                continue;
            }
            if fighting || in_town || sheltered || t < h.ready_at || !h.active(t) || h.mother_out(t) {
                continue;
            }
            let hour = (t / HOUR).floor() as u64;
            let roll = Rng::from_keys(&[self.seed, id as u64, hour, 0x5C0A]).f32();
            let bold = d.toward != Toward::PackHunter || bold_tonight(self.seed, id, t);
            let attack = bold && would_attack(h.sp, h.might(t), their_might, fit.len(), roll);
            match d.toward {
                Toward::PackHunter => {
                    if dist > d.sense * seen {
                        continue;
                    }
                    if !attack {
                        // Too strong: the pack keeps its distance.
                        if dist < d.flight * 1.5 {
                            self.herd_runs(id, here, t);
                        }
                    } else if dist <= PACK_CLOSE {
                        self.herd_attacks_squad(id, 0.0);
                    } else {
                        // Closing in.
                        let to = here.add(pos.sub(here).scale((PACK_CLOSE * 0.6) / dist.max(1.0)));
                        let arrive = t + (pos.dist(to) / (d.speed * 0.6)) as f64;
                        self.animals.herds[id as usize].away = Some(Away { from: pos, to, depart: t, arrive, until: arrive });
                    }
                }
                Toward::Ambusher => {
                    if dist <= d.sense && attack {
                        self.herd_attacks_squad(id, AMBUSH_SURPRISE);
                    }
                }
                Toward::Territorial => {
                    // Whoever is inside its ground, wherever in it the beast is.
                    let inside = fit.iter().any(|&m| self.person_pos(m).dist(h.home) <= d.sense);
                    if inside {
                        self.herd_attacks_squad(id, 0.0);
                    }
                }
                _ => {
                    if dist <= d.sense * seen {
                        self.herd_attacks_squad(id, 0.0);
                    }
                }
            }
        }
    }

    /// A herd runs from a point: off the other way, wary for a while, then
    /// back into its round.
    fn herd_runs(&mut self, herd: u32, from: V2, t: f64) {
        let pos = self.herd_pos(herd, t);
        let h = &mut self.animals.herds[herd as usize];
        let off = pos.sub(from);
        let dir = if off.len() > 0.5 { off.scale(1.0 / off.len()) } else { V2::new(1.0, 0.0) };
        let mut to = pos.add(dir.scale(FLEE_DIST));
        if !geo::is_land(to) || geo::inland(to) < 10.0 {
            to = h.home;
        }
        let arrive = t + (pos.dist(to) / h.def().speed.max(1.0)) as f64;
        h.away = Some(Away { from: pos, to, depart: t, arrive, until: arrive + WARY_TIME });
    }

    /// A fight with the squad in it, set up the way `fights.rs` sets one up
    /// (the squad as it stands, the lights round it, the squad's wards, the
    /// dead lying near to be raised, a guardian waiting), but with no enemy
    /// yet: the caller adds the animals.
    ///
    /// (This repeats what `fights::start_battle` does, which is private to
    /// that file. If that function is opened up, this should call it.)
    fn squad_fray_shell(&mut self, id: u32, at: f64) -> Battle {
        let seed = rng::key(&[self.seed, id as u64, 0xBA77]);
        let mut b = Battle::new(id, seed, at, Vec::new(), Vec::new());
        b.lights = Some(self.fixed_lights());
        for pid in self.squad_fit() {
            self.people[pid as usize].ensure_detail();
            let pos = self.person_pos(pid);
            let mut f = Fighter::from_person(&self.people[pid as usize], SQUAD_SIDE, pos, self.time);
            if self.is_asleep(pid) {
                f.aware_at = b.start + 3.0;
            }
            if let Some(k) = self.squad.index(pid) {
                self.squad.resting[k] = false;
            }
            f.torch = self.torch_lit(pid);
            f.has_torch = self.torch_in_hand(pid).is_some();
            f.held = self.held.get(&pid).copied();
            for bn in self.boons_on(pid) {
                f.statuses.push(Status { does: bn.does, power: bn.power, until: bn.until });
            }
            if self.carrying(pid).is_some() {
                f.burdened = true;
                f.base_speed *= self.carry_pace(pid);
            }
            b.names.push(self.people[pid as usize].name().unwrap_or("someone").to_string());
            b.fighters.push(f);
            self.fighting.insert(pid, id);
        }
        let centre = self.squad.pos;
        for w in self.wards.iter().filter(|w| w.until > at && w.pos.dist(centre) <= w.radius + 80.0 && w.does != Does::Glow) {
            b.zones.push(Zone { does: w.does, pos: w.pos, radius: w.radius, power: w.power, until: w.until, side: SQUAD_SIDE, owner: w.owner, fresh: false });
        }
        let graves: Vec<(V2, PersonId)> = self.corpses.iter().filter(|c| c.0.dist(centre) <= 40.0 && !self.people[c.3 as usize].in_squad).map(|c| (c.0, c.3)).collect();
        for (pos, pid) in graves {
            let mut f = Fighter::from_person(&self.people[pid as usize], super::super::combat::GRAVE_SIDE, pos, at);
            f.dead = true;
            f.ko = true;
            b.names.push(self.people[pid as usize].name().unwrap_or("someone").to_string());
            b.fighters.push(f);
        }
        let waiting: Vec<super::super::casting::Ward> = self.wards.iter().copied().filter(|w| matches!(w.does, Does::Summon(_)) && w.until > at && w.pos.dist(centre) <= w.radius).collect();
        for w in waiting {
            if let Does::Summon(kind) = w.does {
                let owner = self.people.get(w.owner as usize).and_then(|p| p.name()).unwrap_or("Someone").to_string();
                let race = self.people.get(w.owner as usize).map(|p| p.race).unwrap_or(super::super::race::Race::Roduro);
                b.call_up_for(SQUAD_SIDE, race, &owner, kind, w.pos, w.until, w.power);
            }
            self.wards.retain(|x| *x != w);
        }
        b.zones.retain(|z| !matches!(z.does, Does::Summon(_)));
        b
    }

    /// Start a fight between the squad and a herd. `squad_surprised`: how
    /// long before the squad knows it's attacked; `herd_surprised`: how long
    /// before the herd knows it is; `hunted`: the squad started it. Returns
    /// the fight's id, or `None` if there can't be one (the squad or the
    /// herd is already fighting).
    fn squad_fray(&mut self, herd: u32, squad_surprised: f64, herd_surprised: f64, hunted: bool) -> Option<u32> {
        let t = self.time;
        if self.squad_battle().is_some() || self.squad_fit().is_empty() || self.herd_in_fray(herd) {
            return None;
        }
        let h = &self.animals.herds[herd as usize];
        let here = self.squad.pos;
        let centre = self.herd_pos(herd, t);
        // The nearest of them come into it.
        let mut slots: Vec<(usize, V2)> = (0..h.alive(t)).filter(|&j| !h.down(j, t)).map(|j| (j, h.member_at(centre, j, t))).collect();
        slots.sort_by(|a, b| a.1.dist(here).total_cmp(&b.1.dist(here)).then(a.0.cmp(&b.0)));
        // Of a herd that only fights when cornered, the cornered few fight.
        slots.truncate(if h.def().toward == Toward::ChargesIfCornered { CORNERED } else { MAX_FIGHTERS });
        if slots.is_empty() {
            return None;
        }
        let id = self.animals.next_battle;
        self.animals.next_battle += 1;
        let mut b = self.squad_fray_shell(id, t);
        let h = &self.animals.herds[herd as usize];
        let d = h.def();
        let mut parts = Vec::new();
        for (j, pos) in &slots {
            let mut f = h.fighter(*j, t, ANIMAL_SIDE, *pos);
            f.aware_at = t + herd_surprised;
            parts.push(FrayPart { fighter: b.fighters.len() as u16, who: Who::Wild { herd, ident: h.ident(*j) }, sp: h.sp, size: h.size(*j, t), end: None });
            b.fighters.push(f);
            b.names.push(d.name.to_string());
        }
        if squad_surprised > 0.0 {
            for f in b.fighters.iter_mut().filter(|f| f.side == SQUAD_SIDE) {
                f.aware_at = f.aware_at.max(t + squad_surprised);
            }
        }
        let pack = slots.len() as u8;
        let (pack_might, their_might) = (h.might(t), self.squad_fit().iter().map(|&m| self.people[m as usize].might).sum::<f32>());
        let (sp, people) = (h.sp, b.fighters.iter().filter(|f| f.side == SQUAD_SIDE).count() as u8);
        self.battles.push(b);
        self.animals.frays.push(Fray { battle: id, at: centre, ends: None, parts, animals_won: false, squad_won: false, hunted, cargo: None });
        let hd = &mut self.animals.herds[herd as usize];
        hd.away = Some(Away { from: centre, to: centre, depart: t, arrive: t, until: f64::INFINITY });
        // (If it was stalking travellers, that is off: the strike it had
        // set finds no stalk and does nothing.)
        hd.hunt = None;
        self.animals.attacks.push(AttackRecord { t, herd, sp, victim: None, at: centre, pack, people, pack_might, their_might, battle: id, over: false, animals_won: false, by_squad: hunted, people_killed: 0, animals_lost: 0 });
        if self.animals.attacks.len() > ATTACK_LOG {
            self.animals.attacks.remove(0);
        }
        Some(id)
    }

    /// A herd sets on the squad. Returns the fight's id, or `None` if it
    /// can't (the squad is already fighting, say).
    pub(super) fn herd_attacks_squad(&mut self, herd: u32, surprise: f64) -> Option<u32> {
        let t = self.time;
        let id = self.squad_fray(herd, surprise, 0.0, false)?;
        // A tripwire round them: sleepers are up at once.
        let tripped = self.squad.members.iter().any(|&m| self.wards_at(Does::Tripwire, self.person_pos(m)).next().is_some());
        if tripped {
            if let Some(b) = self.battles.iter_mut().find(|b| b.id == id) {
                for f in b.fighters.iter_mut().filter(|f| f.side == SQUAD_SIDE) {
                    f.aware_at = f.aware_at.min(t);
                }
            }
        }
        let d = self.animals.herds[herd as usize].def();
        let n = self.animals.frays.last().map(|f| f.parts.len()).unwrap_or(1);
        let line = if n > 1 { format!("{}s are on you!", d.name) } else { format!("A {} is on you!", d.name) };
        self.alerts.push(line.clone());
        self.log.push_front((t, line));
        self.log.truncate(14);
        self.animals.stats.attacks += 1;
        Some(id)
    }

    /// Some of the squad set about a wild herd (hunting). They have to be
    /// within `HUNT_REACH` of it. Prey takes a moment to react — longer if
    /// everyone going in is sneaking — and then runs; what stands and fights
    /// does. Returns false if there's nothing there to set about.
    pub fn hunt(&mut self, who: &[PersonId], herd: u32) -> bool {
        let t = self.time;
        let Some(h) = self.animals.herds.get(herd as usize) else { return false };
        if h.alive(t) == 0 || who.is_empty() {
            return false;
        }
        let pos = self.herd_pos(herd, t);
        let near = who.iter().map(|&m| self.person_pos(m).dist(pos)).fold(f32::MAX, f32::min);
        if near > HUNT_REACH + h.spread(t) {
            return false;
        }
        let running = h.away.map(|a| t < a.arrive && a.from.dist(a.to) > 1.0).unwrap_or(false);
        let surprise = if running {
            0.0
        } else if who.iter().all(|&m| self.is_sneaking(m)) {
            5.0
        } else {
            reaction(h.def().toward)
        };
        let name = h.def().name;
        if self.squad_fray(herd, 0.0, surprise, true).is_none() {
            return false;
        }
        let line = if surprise >= 5.0 { format!("You fall on the {name}s before they know it.") } else { format!("You go for the {name}s.") };
        self.log.push_front((t, line));
        self.log.truncate(14);
        true
    }

    /// The wild herd nearest a point, within `radius`.
    pub fn herd_near(&self, p: V2, radius: f32) -> Option<u32> {
        let t = self.time;
        self.animals
            .herds
            .iter()
            .filter(|h| h.alive(t) > 0 && h.home.dist(p) <= radius + h.def().range + FLEE_DIST)
            .map(|h| (h.id, self.herd_pos(h.id, t).dist(p)))
            .filter(|x| x.1 <= radius)
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
            .map(|x| x.0)
    }

    /// Run the fights with animals in them up to now, and settle the ones
    /// that are over. (Run here, before the world's own pass over its
    /// fights, so the animals' nerve is applied.)
    fn advance_frays(&mut self) {
        if self.animals.frays.is_empty() {
            return;
        }
        let t = self.time;
        let mut done: Vec<Fray> = Vec::new();
        let mut k = 0;
        while k < self.animals.frays.len() {
            let id = self.animals.frays[k].battle;
            let Some(bi) = self.battles.iter().position(|b| b.id == id) else {
                // A far-off fight's copy is taken down when it ends on the
                // timeline. A fight of the squad's gone missing some other
                // way is simply over: its herds are let go.
                if self.animals.frays[k].ends.is_none() {
                    let fray = self.animals.frays.remove(k);
                    for p in &fray.parts {
                        if let Who::Wild { herd, .. } = p.who {
                            let h = &mut self.animals.herds[herd as usize];
                            h.away = None;
                            h.ready_at = t + PACK_REST;
                        }
                    }
                } else {
                    k += 1;
                }
                continue;
            };
            let parts = self.animals.frays[k].parts.clone();
            advance_fray(&mut self.battles[bi], &parts, t);
            if self.animals.frays[k].ends.is_none() && self.battles[bi].over {
                let b = &self.battles[bi];
                let mut fray = self.animals.frays.remove(k);
                for p in fray.parts.iter_mut() {
                    p.end = b.fighters.get(p.fighter as usize).map(part_end);
                }
                fray.animals_won = b.winner() == Some(ANIMAL_SIDE);
                fray.squad_won = b.winner() == Some(SQUAD_SIDE);
                let downed = b.fighters.iter().filter(|f| f.side == SQUAD_SIDE && f.is_person() && (f.ko || f.dead)).count();
                let killed = b.fighters.iter().filter(|f| f.side == SQUAD_SIDE && f.is_person() && f.dead).count();
                if let Some(a) = self.animals.attacks.iter_mut().find(|a| a.battle == id) {
                    a.people_killed = killed.min(255) as u8;
                    if !a.by_squad {
                        self.animals.stats.people_downed += downed as u32;
                        self.animals.stats.people_killed += killed as u32;
                    }
                }
                done.push(fray);
                continue;
            }
            k += 1;
        }
        for fray in done {
            self.settle_fray(&fray, t);
        }
    }

    /// Tamed hounds go into any fight their owner is in, on the squad's side.
    fn hounds_join(&mut self) {
        if self.animals.hounds.iter().all(|h| h.gone) {
            return;
        }
        let t = self.time;
        for k in 0..self.animals.hounds.len() {
            let h = &self.animals.hounds[k];
            if h.gone {
                continue;
            }
            let Some(&bid) = self.fighting.get(&h.owner) else { continue };
            let (id, owner) = (h.id, h.owner);
            if self.animals.frays.iter().any(|f| f.parts.iter().any(|p| p.who == Who::Hound(id))) {
                continue;
            }
            let Some(pos) = self.hound_pos(id) else { continue };
            let lost = h.hurt.lost_at(t);
            let mut f = fighter(Sp::Ridgehound, h.size, &lost, SQUAD_SIDE, pos);
            if f.ko {
                continue;
            }
            let power = h.power(t);
            f.weapon.cut *= power;
            f.weapon.blunt *= power;
            let size = h.size;
            let Some(b) = self.battles.iter_mut().find(|b| b.id == bid) else { continue };
            if b.over || !b.fighters.iter().any(|x| x.pid == owner && x.side == SQUAD_SIDE) {
                continue;
            }
            let idx = b.fighters.len() as u16;
            b.fighters.push(f);
            b.names.push("Your hound".to_string());
            let part = FrayPart { fighter: idx, who: Who::Hound(id), sp: Sp::Ridgehound, size, end: None };
            match self.animals.frays.iter_mut().find(|f| f.battle == bid) {
                Some(fr) => fr.parts.push(part),
                None => self.animals.frays.push(Fray { battle: bid, at: pos, ends: None, parts: vec![part], animals_won: false, squad_won: false, hunted: false, cargo: None }),
            }
        }
    }

    pub(super) fn fray_pos_of_hound(&self, id: u32) -> Option<V2> {
        for fr in &self.animals.frays {
            for p in &fr.parts {
                if p.who == Who::Hound(id) {
                    return self.battle(fr.battle).and_then(|b| b.fighters.get(p.fighter as usize)).map(|f| f.pos);
                }
            }
        }
        None
    }

    /// Is this animal of this herd in a fight right now?
    pub fn animal_fighting(&self, herd: u32, ident: u16) -> bool {
        self.animals.frays.iter().any(|f| f.parts.iter().any(|p| p.who == Who::Wild { herd, ident }))
    }

    /// Every animal in a fight near enough to be on show, for drawing.
    pub fn animals_fighting(&self) -> Vec<Fighting> {
        let mut out = Vec::new();
        for fr in &self.animals.frays {
            let Some(b) = self.battle(fr.battle) else { continue };
            for p in &fr.parts {
                let Some(f) = b.fighters.get(p.fighter as usize) else { continue };
                if f.dead || f.fled {
                    continue;
                }
                out.push(Fighting { pos: f.pos, sp: p.sp, size: p.size, down: f.ko, fleeing: f.fleeing, tame: matches!(p.who, Who::Hound(_)), vitality: f.vitality() });
            }
        }
        out
    }
}

