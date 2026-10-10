//! Reading danger before committing (play-test 3): how a fight with a band,
//! a camp, a ruin's wardens or a beast would likely go for the squad as it
//! stands, in a plain word ("far beyond you", "a fair fight", "easy").
//!
//! The reading is the fight itself: a few rehearsals with the full fight
//! rules, on copies, between the squad as it is now (wounds, mana, kit) and
//! whoever would come into it. Nothing in the world changes. Each rehearsal
//! has its own keyed seed (the target, the hour, the try), so the reading is
//! steady for the hour and is never the real fight's own roll. Rehearsing
//! rather than adding up `might` matters: a Cragmaw's might is a fraction of
//! a squad's, yet one puts most squads down.

use super::animals::{self, FrayPart, Who, ANIMAL_SIDE, CORNERED, MAX_FIGHTERS};
use super::combat::{Battle, Fighter, SQUAD_SIDE};
use super::geo::V2;
use super::group::GroupId;
use super::person::PersonId;
use super::rng;
use super::ruins::RuinKind;
use super::world::{World, HOUR};

/// How many rehearsals a reading takes.
pub const TRIES: u64 = 6;
/// The longest a rehearsal runs, seconds of fight.
pub const LONGEST: f64 = 900.0;
/// Rehearsals start with the other side this far off, metres.
pub const APART: f32 = 14.0;
/// A win counts as easy if the squad loses less than this share of its
/// health in it, on average, and nobody is put down.
pub const EASY_HURT: f32 = 0.3;

/// How a fight would likely go.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Odds {
    Easy,
    Fair,
    Risky,
    Beyond,
}

impl Odds {
    /// The plain word: "Cragmaw (huntable; far beyond you)".
    pub fn words(self) -> &'static str {
        match self {
            Odds::Easy => "easy",
            Odds::Fair => "a fair fight",
            Odds::Risky => "risky",
            Odds::Beyond => "far beyond you",
        }
    }

    /// As a sentence after a sighting.
    pub fn sentence(self) -> &'static str {
        match self {
            Odds::Easy => "Easy for you.",
            Odds::Fair => "A fair fight.",
            Odds::Risky => "Risky: you'd as likely lose.",
            Odds::Beyond => "Far beyond you.",
        }
    }
}

/// The rehearsals' tally.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub won: u8,
    pub of: u8,
    /// Share of the squad's health lost, on average, in the wins.
    pub hurt: f32,
    /// Some of the squad put down in a win.
    pub downed: bool,
}

impl Reading {
    pub fn odds(&self) -> Odds {
        if self.of == 0 || self.won == 0 {
            Odds::Beyond
        } else if self.won == self.of && self.hurt < EASY_HURT && !self.downed {
            Odds::Easy
        } else if self.won as u32 * 2 >= self.of as u32 {
            Odds::Fair
        } else {
            Odds::Risky
        }
    }
}

fn health(f: &Fighter) -> f32 {
    let (h, m): (f32, f32) = f.hp.iter().zip(f.max_hp.iter()).fold((0.0, 0.0), |a, (h, m)| (a.0 + h.max(0.0), a.1 + m));
    if m > 0.0 {
        h / m
    } else {
        0.0
    }
}

impl World {
    /// The squad's side for a rehearsal: the fit, as they are now.
    fn rehearsal_squad(&self) -> Vec<Fighter> {
        let t = self.time;
        self.squad_fit()
            .into_iter()
            .map(|pid| {
                let mut f = Fighter::from_person(&self.people[pid as usize], SQUAD_SIDE, self.person_pos(pid), t);
                f.torch = self.torch_lit(pid);
                f.has_torch = self.torch_in_hand(pid).is_some();
                f
            })
            .collect()
    }

    /// Run the rehearsals: `them` placed `APART` from the squad (keeping
    /// their shape), `parts` for any animals among them, `slow` the moment
    /// they take to answer when the squad goes for them (as in the real
    /// thing: `attack`, `hunt`).
    fn rehearse(&self, key: u64, them: Vec<(Fighter, String)>, parts: Vec<FrayPart>, slow: f64) -> Reading {
        let t = self.time;
        let ours = self.rehearsal_squad();
        let mut r = Reading { won: 0, of: 0, hurt: 0.0, downed: false };
        if ours.is_empty() || them.is_empty() {
            return r;
        }
        // Bring them close, keeping how they stand.
        let here = self.squad.pos;
        let centre = them.iter().fold(V2::default(), |a, (f, _)| a.add(f.pos)).scale(1.0 / them.len() as f32);
        let mut dir = centre.sub(here);
        let len = dir.dist(V2::default());
        dir = if len > 0.1 { dir.scale(1.0 / len) } else { V2::new(1.0, 0.0) };
        let to = here.add(dir.scale(APART));
        let before: f32 = ours.iter().map(health).sum();
        let hour = (t / HOUR).floor() as u64;
        let mut hurt = 0.0;
        for k in 0..TRIES {
            let seed = rng::key(&[self.seed, key, hour, k, 0x5245_4845]);
            let mut b = Battle::new(u32::MAX, seed, t, Vec::new(), Vec::new());
            b.lights = Some(self.fixed_lights());
            for f in &ours {
                b.fighters.push(f.clone());
                b.names.push(String::new());
            }
            for (f, name) in &them {
                let mut f = f.clone();
                f.pos = to.add(f.pos.sub(centre));
                f.aware_at = t + slow;
                b.fighters.push(f);
                b.names.push(name.clone());
            }
            let n = ours.len() as u16;
            let parts: Vec<FrayPart> = parts.iter().map(|p| FrayPart { fighter: p.fighter + n, ..*p }).collect();
            if parts.is_empty() {
                b.advance_to(t + LONGEST);
            } else {
                animals::advance_fray(&mut b, &parts, t + LONGEST);
            }
            r.of += 1;
            if b.over && b.winner() == Some(SQUAD_SIDE) {
                r.won += 1;
                let mine: Vec<&Fighter> = b.fighters.iter().filter(|f| f.side == SQUAD_SIDE && f.is_person()).collect();
                let after: f32 = mine.iter().map(|f| health(f)).sum();
                hurt += if before > 0.0 { (1.0 - after / before).max(0.0) } else { 0.0 };
                if mine.iter().any(|f| f.ko || f.dead) {
                    r.downed = true;
                }
            }
        }
        if r.won > 0 {
            r.hurt = hurt / r.won as f32;
        }
        r
    }

    /// How a fight with a band of people would go.
    pub fn reading_vs_group(&self, gid: GroupId) -> Reading {
        let t = self.time;
        let Some(g) = self.group(gid) else { return Reading { won: 0, of: 0, hurt: 0.0, downed: false } };
        let them: Vec<(Fighter, String)> = g
            .members
            .iter()
            .copied()
            .filter(|&m| self.fit_to_fight(m) || self.fighting.contains_key(&m))
            .map(|m: PersonId| {
                let p = &self.people[m as usize];
                (Fighter::from_person(p, 1, self.person_pos(m), t), p.name().unwrap_or("someone").to_string())
            })
            .collect();
        let slow = if self.has_noticed(gid) { 0.0 } else { 2.0 };
        self.rehearse(gid as u64 | 0x6000_0000_0000, them, Vec::new(), slow)
    }

    /// How going for a wild herd would go: whoever of it would stand and
    /// fight (the nearest few, as when a fight with it starts).
    pub fn reading_vs_herd(&self, herd: u32) -> Reading {
        let t = self.time;
        let Some(h) = self.animals.herds.get(herd as usize) else { return Reading { won: 0, of: 0, hurt: 0.0, downed: false } };
        let here = self.squad.pos;
        let centre = self.herd_pos(herd, t);
        let mut slots: Vec<(usize, V2)> = (0..h.alive(t)).filter(|&j| !h.down(j, t)).map(|j| (j, h.member_at(centre, j, t))).collect();
        slots.sort_by(|a, b| a.1.dist(here).total_cmp(&b.1.dist(here)).then(a.0.cmp(&b.0)));
        slots.truncate(if h.def().toward == animals::Toward::ChargesIfCornered { CORNERED } else { MAX_FIGHTERS });
        let mut them = Vec::new();
        let mut parts = Vec::new();
        for (j, pos) in slots {
            parts.push(FrayPart { fighter: them.len() as u16, who: Who::Wild { herd, ident: h.ident(j) }, sp: h.sp, size: h.size(j, t), end: None });
            them.push((h.fighter(j, t, ANIMAL_SIDE, pos), h.def().name.to_string()));
        }
        let slow = animals::reaction(h.def().toward);
        self.rehearse(herd as u64 | 0x7000_0000_0000, them, parts, slow)
    }

    /// The bandit camp's band, if this group keeps a camp.
    pub fn camp_of(&self, gid: GroupId) -> Option<usize> {
        self.camps.iter().position(|c| c.group == gid)
    }

    /// How taking on a ruin's keepers would go (`None` if nobody holds it).
    pub fn reading_vs_ruin(&self, id: u32) -> Option<Reading> {
        if !self.ruin_held(id) {
            return None;
        }
        let ru = self.ruins.get(id as usize)?;
        Some(match ru.kind {
            RuinKind::Ruin => self.reading_vs_group(ru.guards?),
            RuinKind::Lair(h) => self.reading_vs_herd(h),
        })
    }
}
