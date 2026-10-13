//! The hidden test quarter (Laz's tutorial island brief, "The test quarter
//! (dev-only)"): a walled, locked square of flat ground on the island's west
//! where anything can be tried. Nothing here is part of the story; it's all
//! behind a debug key (Home in the window) and never in a release
//! screenshot.
//!
//! - **Where.** The island's walled west quarter (`island.rs`,
//!   `Island::test_quarter`) once the island stands; until then a stand-in:
//!   the nearest flat open ground to where the squad is when the key is
//!   first pressed (`quarter_spot`). Either way it's found once and kept
//!   (`World::test_quarter`, saved), so nothing else in the world depends on
//!   whether anyone ever opened it.
//! - **What stands there** is a pure function of its corners (`TestQuarter`'s
//!   layout): a wall all round with a gate in the east side, locked until
//!   the key opens it (`solid.rs` lists both); the combat yard (an arena
//!   ring and training dummies that never fight back); the culture row (a
//!   lot for each people with a pair of their folk standing at it, until
//!   the island gives the row real buildings); empty build plots.
//! - **The dev spawner** is a handful of plain calls the window's panel
//!   makes (`dev_*`): any item into a member's pack, a herd of any
//!   creature, a bandit or Binder band, a person of any people or of a
//!   faction, time skipped forward. They act like the world's own events
//!   (a band is a band, a herd a herd), so what's tried here is what
//!   happens out there.
//! - **The duel grid** (`duelgrid.rs`) runs a weapon's row from the yard.

use serde::{Deserialize, Serialize};

use super::factions::{self, FactionId};
use super::geo::V2;
use super::group::{Group, GroupId, Kind, Leg};
use super::items::ItemId;
use super::jobs::Job;
use super::layout::Rect;
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng::{self, Rng};
use super::stats::{Calling, Skill};
use super::world::{World, HOUR};

/// The quarter's side, metres, and its wall.
pub const SIDE: f32 = 160.0;
pub const WALL_H: f32 = 3.2;
pub const WALL_T: f32 = 0.8;
/// The gate's width.
pub const GATE_W: f32 = 5.0;
/// Training dummies in the combat yard, and the arena's reach.
pub const DUMMIES: usize = 4;
pub const ARENA_R: f32 = 14.0;
/// Build plots along the south side, and their size.
pub const PLOTS: usize = 5;
pub const PLOT: f32 = 22.0;
/// How flat the stand-in ground must be: the most the land may rise or fall
/// across it, metres.
const FLAT: f32 = 8.0;

/// The quarter: its corners, whether the gate is open, and who stands in
/// it (dummies, the culture row's folk).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TestQuarter {
    pub min: V2,
    pub max: V2,
    pub open: bool,
    /// Training dummies: people who never act in a fight.
    #[serde(default)]
    pub dummies: Vec<PersonId>,
    /// The culture row's folk, by lot: (people, who).
    #[serde(default)]
    pub row: Vec<(Race, PersonId)>,
}

impl TestQuarter {
    /// A quarter with these corners, nobody in it yet.
    pub fn new(min: V2, max: V2) -> TestQuarter {
        TestQuarter { min, max, open: false, dummies: Vec::new(), row: Vec::new() }
    }

    pub fn centre(&self) -> V2 {
        self.min.lerp(self.max, 0.5)
    }

    pub fn contains(&self, p: V2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    /// A point in the quarter by its share across and down (0..1).
    fn at(&self, u: f32, v: f32) -> V2 {
        V2::new(self.min.x + (self.max.x - self.min.x) * u, self.min.y + (self.max.y - self.min.y) * v)
    }

    /// The gate: the middle of the east wall.
    pub fn gate(&self) -> V2 {
        V2::new(self.max.x, (self.min.y + self.max.y) * 0.5)
    }

    /// Just inside the gate: where the squad is brought in, and where the
    /// spawner puts things.
    pub fn pad(&self) -> V2 {
        self.gate().add(V2::new(-12.0, 0.0))
    }

    /// The wall's stretches (the east side in two, round the gate), as boxes.
    pub fn walls(&self) -> Vec<Rect> {
        let (w, h) = (self.max.x - self.min.x, self.max.y - self.min.y);
        let (cx, cy) = ((self.min.x + self.max.x) * 0.5, (self.min.y + self.max.y) * 0.5);
        let half = (h - GATE_W) * 0.5;
        vec![
            Rect::new(V2::new(cx, self.min.y), 0.0, w + WALL_T, WALL_T),
            Rect::new(V2::new(cx, self.max.y), 0.0, w + WALL_T, WALL_T),
            Rect::new(V2::new(self.min.x, cy), std::f32::consts::FRAC_PI_2, h + WALL_T, WALL_T),
            Rect::new(V2::new(self.max.x, self.min.y + half * 0.5), std::f32::consts::FRAC_PI_2, half, WALL_T),
            Rect::new(V2::new(self.max.x, self.max.y - half * 0.5), std::f32::consts::FRAC_PI_2, half, WALL_T),
        ]
    }

    /// The gate's leaf, across the opening.
    pub fn gate_leaf(&self) -> Rect {
        Rect::new(self.gate(), std::f32::consts::FRAC_PI_2, GATE_W, WALL_T * 0.6)
    }

    /// The combat yard's arena: its middle and reach.
    pub fn arena(&self) -> (V2, f32) {
        (self.at(0.32, 0.5), ARENA_R)
    }

    /// The arena's two sides: the squad stands to fight on the west, and
    /// foes called up in the quarter come in on the east, across from them.
    pub fn squad_side(&self) -> V2 {
        let (c, r) = self.arena();
        c.add(V2::new(-r * 0.45, 0.0))
    }
    pub fn foe_side(&self) -> V2 {
        let (c, r) = self.arena();
        c.add(V2::new(r * 0.45, 0.0))
    }

    /// Where the training dummies stand: a row along the arena's north edge.
    pub fn dummy_spots(&self) -> Vec<V2> {
        let (c, r) = self.arena();
        (0..DUMMIES).map(|k| c.add(V2::new((k as f32 - (DUMMIES as f32 - 1.0) * 0.5) * 4.0, -r * 0.5))).collect()
    }

    /// The culture row: a lot for each people along the north wall.
    pub fn culture_lots(&self) -> Vec<(Race, Rect)> {
        super::race::ALL_RACES.iter().enumerate().map(|(k, &r)| (r, Rect::new(self.at(0.2 + 0.2 * k as f32, 0.15), 0.0, 24.0, 18.0))).collect()
    }

    /// Empty build plots along the south wall.
    pub fn plots(&self) -> Vec<Rect> {
        (0..PLOTS).map(|k| Rect::new(self.at(0.14 + 0.17 * k as f32, 0.84), 0.0, PLOT, PLOT)).collect()
    }
}

/// The culture row's homes on the island, added to the compound at world
/// creation (after `island::add_compound`, before the society is founded):
/// a Roduro home, a Qotiro block, a Horaro stilt home and a home where
/// Ṭaḍoro lodge (they don't build their own: canon), each at its lot in the
/// test quarter. Appended to the compound's buildings, so its own numbering
/// stands. Their folk come last of all (`people_the_culture_row`), so no
/// one else's dice move.
pub(super) fn add_culture_row(settlements: &mut [super::settlement::Settlement], isle: &super::island::Island, seed: u64) {
    use super::settlement::{Building, BuildingKind};
    let (min, max) = isle.test_quarter();
    let q = TestQuarter::new(min, max);
    let s = &mut settlements[isle.settlement as usize];
    let mut r = Rng::from_keys(&[seed, 0x7E57_C017]);
    for (race, lot) in q.culture_lots() {
        let (kind, size) = match race {
            Race::Qotiro => (BuildingKind::QotiroBlock, 15.0),
            Race::Horaro => (BuildingKind::HoraroStilt, 10.0),
            _ => (BuildingKind::RoduroHome, 11.0),
        };
        s.buildings.push(Building { pos: lot.c, kind, size, rot: std::f32::consts::FRAC_PI_2, seed: r.next_u64() });
    }
    s.reach = s.buildings.iter().map(|b| b.pos.dist(s.pos) + b.size).fold(s.reach, f32::max);
}

/// People in the culture row's homes: two of each home's people.
pub const ROW_FOLK: usize = 2;

impl World {
    /// The culture row's folk, last of all at world creation: two of each
    /// people in their home, each pair a household of the compound's
    /// community, with no trade (they keep to their homes:
    /// `in_culture_row`).
    pub(super) fn people_the_culture_row(&mut self) {
        let Some(isle) = self.island.clone() else { return };
        let town = isle.settlement;
        let homes: Vec<(u16, Race)> = self.settlements[town as usize]
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| isle.in_test_quarter(b.pos))
            .zip(super::race::ALL_RACES)
            .map(|((i, _), r)| (i as u16, r))
            .collect();
        let shore = self.society.towns[town as usize].shore;
        for (home, race) in homes {
            let hh = self.society.households.len() as u32;
            let mut members = Vec::new();
            for k in 0..ROW_FOLK as u64 {
                let pid = self.people.len() as PersonId;
                let mut p = Person::summary(pid, rng::key(&[self.seed, 0x7E57_C018, home as u64, k]), race, Some(town));
                p.home = Some(town);
                p.dwelling = Some(home);
                p.wounds.at = self.time;
                p.mana_at = self.time;
                self.people.push(p);
                self.busy_until.push(0.0);
                self.group_of.push(None);
                self.settlements[town as usize].residents.push(pid);
                let mut l = super::society::Life::new(race, self.people[pid as usize].seed);
                l.community = Some(shore);
                l.household = Some(hh);
                l.job = Job::None;
                l.place = None;
                self.society.lives.push(l);
                self.society.minds.push(Default::default());
                members.push(pid);
            }
            self.society.households.push(super::society::Household { community: shore, members, home: Some(home), purse: Default::default(), feelings: Vec::new() });
            let sl = self.current_sleepers((town, home));
            if let Some(x) = self.settlements[town as usize].sleepers.get_mut(home as usize) {
                *x = sl;
            }
        }
        self.refresh_container_owners();
        self.solid_changed();
    }

    /// Does this person live in the test quarter's culture row?
    pub fn in_culture_row(&self, pid: PersonId) -> bool {
        let p = &self.people[pid as usize];
        let (Some(isle), Some(home), Some(d)) = (self.island(), p.home, p.dwelling) else { return false };
        home == isle.settlement && self.settlements[home as usize].buildings.get(d as usize).is_some_and(|b| isle.in_test_quarter(b.pos))
    }

    /// The test quarter, if it has been found.
    pub fn test_quarter(&self) -> Option<&TestQuarter> {
        self.test_quarter.as_ref()
    }

    /// Where the quarter goes: the island's walled west quarter once the
    /// island stands; until then the nearest flat open ground to `near`.
    fn quarter_spot(&self, near: V2) -> Option<(V2, V2)> {
        if let Some(i) = self.island() {
            return Some(i.test_quarter());
        }
        // A world without the island: the nearest flat open ground.
        let half = SIDE * 0.5;
        for ring in 0..24 {
            let d = 400.0 + ring as f32 * 150.0;
            for k in 0..16 {
                let a = k as f32 / 16.0 * std::f32::consts::TAU + ring as f32 * 0.37;
                let c = near.add(V2::new(a.cos() * d, a.sin() * d));
                if self.quarter_fits(c, half) {
                    return Some((c.sub(V2::new(half, half)), c.add(V2::new(half, half))));
                }
            }
        }
        None
    }

    /// Is the square round `c` flat open land, clear of towns, camps, ruins,
    /// roads and bases?
    fn quarter_fits(&self, c: V2, half: f32) -> bool {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for j in 0..=4 {
            for i in 0..=4 {
                let p = c.add(V2::new((i as f32 / 4.0 - 0.5) * 2.0 * half, (j as f32 / 4.0 - 0.5) * 2.0 * half));
                if !super::geo::is_land(p) || self.terrain.is_sea(p) {
                    return false;
                }
                let h = self.terrain.height(p);
                lo = lo.min(h);
                hi = hi.max(h);
            }
        }
        let reach = half * 1.5;
        hi - lo <= FLAT
            && self.settlements.iter().all(|s| s.pos.dist(c) > s.radius() + reach + 200.0)
            && self.camps.iter().all(|k| k.pos.dist(c) > reach + 400.0)
            && self.ruins.iter().all(|r| r.pos.dist(c) > reach + 200.0)
            && self.bases.iter().all(|b| b.at.dist(c) > reach + 200.0)
            && self.routes.roads.iter().all(|road| road.iter().all(|p| p.dist(c) > reach + 40.0))
    }

    /// Open the quarter (finding it the first time) and bring the squad in,
    /// just inside the gate. False if there's nowhere for it.
    pub fn enter_test_quarter(&mut self) -> bool {
        if self.test_quarter.is_none() {
            let Some((min, max)) = self.quarter_spot(self.squad.pos) else { return false };
            self.test_quarter = Some(TestQuarter::new(min, max));
            self.stand_dummies();
            // On the island the row is real homes with residents
            // (`add_culture_row`); elsewhere, folk standing at each lot.
            if self.island().is_none() {
                self.stand_culture_row();
            }
        }
        if let Some(q) = self.test_quarter.as_mut() {
            q.open = true;
        }
        self.solid_changed();
        let pad = self.test_quarter.as_ref().map(|q| q.pad()).unwrap_or(self.squad.pos);
        self.teleport_squad(pad);
        true
    }

    /// Lock the gate again (the key toggles it from inside).
    pub fn lock_test_quarter(&mut self) {
        if let Some(q) = self.test_quarter.as_mut() {
            q.open = false;
        }
        self.solid_changed();
    }

    /// Is this someone the quarter keeps as a training dummy?
    pub fn is_dummy(&self, pid: PersonId) -> bool {
        self.test_quarter.as_ref().is_some_and(|q| q.dummies.contains(&pid))
    }

    /// A person of `race` standing at `at` on their own (a group of one that
    /// waits there), with a life and a mind like anyone. `hostile`: they can
    /// be attacked (dummies); nobody of the quarter ever goes looking for a fight.
    fn stand_person(&mut self, race: Race, at: V2, tag: u64, hostile: bool) -> PersonId {
        let id = self.people.len() as PersonId;
        let seed = rng::key(&[self.seed, 0x7E57, tag, id as u64]);
        let mut p = Person::summary(id, seed, race, None);
        p.bandit = hostile;
        p.specialize(Calling::Warrior, &[(Skill::Blade, 25.0), (Skill::Block, 15.0)], 60.0);
        p.ensure_detail();
        p.wounds.at = self.time;
        p.mana_at = self.time;
        self.people.push(p);
        self.busy_until.push(f64::INFINITY);
        self.group_of.push(None);
        while self.society.lives.len() < self.people.len() {
            let q = &self.people[self.society.lives.len()];
            let l = super::society::Life::new(q.race, q.seed);
            self.society.lives.push(l);
        }
        while self.society.minds.len() < self.people.len() {
            self.society.minds.push(Default::default());
        }
        let gid = self.next_group;
        self.next_group += 1;
        let g = Group {
            id: gid,
            seed: rng::key(&[seed, 0x6750]),
            members: vec![id],
            kind: Kind::Wanderer { rest_min: f64::INFINITY, rest_max: f64::INFINITY },
            legs: vec![Leg::wait(at, f64::INFINITY)],
            speed: race.walk_speed(),
            ends: f64::INFINITY,
            written: 1,
            hostile,
            cargo: None,
            pos: at,
            last_update: self.time,
            band: 3,
        };
        self.add_group(g);
        id
    }

    /// The combat yard's dummies.
    fn stand_dummies(&mut self) {
        let Some(spots) = self.test_quarter.as_ref().map(|q| q.dummy_spots()) else { return };
        let mut ids = Vec::new();
        for (k, at) in spots.into_iter().enumerate() {
            ids.push(self.stand_person(Race::Roduro, at, 0xD0 + k as u64, true));
        }
        if let Some(q) = self.test_quarter.as_mut() {
            q.dummies = ids;
        }
    }

    /// The culture row's folk: two of each people at their lot.
    fn stand_culture_row(&mut self) {
        let Some(lots) = self.test_quarter.as_ref().map(|q| q.culture_lots()) else { return };
        let mut row = Vec::new();
        for (k, (race, lot)) in lots.into_iter().enumerate() {
            for j in 0..2 {
                let at = lot.c.add(V2::new(j as f32 * 3.0 - 1.5, lot.hd + 2.0));
                row.push((race, self.stand_person(race, at, 0xC0 + (k * 2 + j) as u64, false)));
            }
        }
        if let Some(q) = self.test_quarter.as_mut() {
            q.row = row;
        }
    }

    /// The dummies stand up again, whole.
    pub fn dev_reset_dummies(&mut self) {
        let ids = self.test_quarter.as_ref().map(|q| q.dummies.clone()).unwrap_or_default();
        let t = self.time;
        for d in ids {
            let p = &mut self.people[d as usize];
            p.dead = false;
            p.wounds = Default::default();
            p.wounds.at = t;
        }
    }

    // ---- The dev spawner ---------------------------------------------------------

    /// `n` of an item into a member's pack.
    pub fn dev_give(&mut self, who: PersonId, it: ItemId, n: u16) {
        self.settle_condition(who, self.time);
        if let Some(d) = self.people[who as usize].detail.as_mut() {
            d.gear.add(it, n);
        }
        self.people[who as usize].recompute_might();
    }

    /// A herd of a species at `at`, `n` strong.
    pub fn dev_herd(&mut self, sp: super::animals::Sp, at: V2, n: u8) -> u32 {
        let seed = rng::key(&[self.seed, 0x7E57_4E7D, self.animals.herds.len() as u64]);
        self.add_herd(sp, at, n.max(1), n as f32, seed)
    }

    /// A bandit band (or the Binders) at `at`.
    pub fn dev_band(&mut self, at: V2, n: usize, slavers: bool) -> GroupId {
        if slavers {
            self.spawn_binders(at, n)
        } else {
            self.spawn_bandits(at, n, false)
        }
    }

    /// Someone of a people standing at `at`.
    pub fn dev_person(&mut self, race: Race, at: V2) -> PersonId {
        let tag = Rng::from_keys(&[self.seed, self.people.len() as u64]).next_u64();
        self.stand_person(race, at, tag, false)
    }

    /// Someone of a faction standing at `at`: their trade's group where the
    /// faction is one (a guard, a scribe, a priest...), else seated as its
    /// first leader (`leaders.rs`).
    pub fn dev_faction_member(&mut self, f: FactionId, at: V2) -> PersonId {
        let race = factions::def(f).people;
        let who = self.dev_person(race, at);
        let key = factions::def(f).key;
        let job = super::disguise::CIVIC.iter().find(|c| !c.elite && c.faction == Some(key)).and_then(|c| c.knows.first().copied()).or_else(|| super::membership::GUILD_TRADES.iter().find(|g| g.1 == key).map(|g| g.0));
        match job {
            Some(j) => self.society.lives[who as usize].job = j,
            None => {
                if let Some(l) = factions::LEADERS.iter().find(|l| l.faction == key) {
                    self.seat_leader(l.name, who);
                }
            }
        }
        who
    }

    /// The world runs on `hours` (in steps, as it would).
    pub fn dev_skip(&mut self, hours: f64) {
        let end = self.time + hours * HOUR;
        while self.time < end {
            self.step((end - self.time).min(600.0));
        }
    }

    /// The job a dev faction member was given (for the panel to say).
    pub fn dev_job_of(&self, who: PersonId) -> Job {
        self.life(who).job
    }

    // ---- The sticky dev panel's squad and place calls (Laz, 13 Oct) -------------

    /// Everyone of the squad whole again: wounds mended, bleeds stopped.
    pub fn dev_heal_all(&mut self) {
        let t = self.time;
        for m in self.squad.members.clone() {
            self.settle_condition(m, t);
            let p = &mut self.people[m as usize];
            let max: [f32; 6] = std::array::from_fn(|k| p.stats.max_hp(super::body::PARTS[k]));
            let stats = p.stats.clone();
            p.wounds.set(&stats, &max, t);
            p.wounds.bleed = 0.0;
            p.recompute_might();
        }
    }

    /// Knock one of the squad out cold.
    pub fn dev_knock_out(&mut self, m: PersonId) {
        let t = self.time;
        self.settle_condition(m, t);
        let p = &mut self.people[m as usize];
        let stats = p.stats.clone();
        let mut hp = p.wounds.hp_at(&stats, t);
        hp[super::body::Part::Head as usize] = -0.5;
        p.wounds.set(&stats, &hp, t);
    }

    /// One of the squad starving (hunger near the top).
    pub fn dev_starve(&mut self, m: PersonId) {
        let t = self.time;
        self.settle_condition(m, t);
        if let Some(c) = self.people[m as usize].cond.as_mut() {
            c.hunger = 90.0;
        }
        self.settle_condition(m, t);
    }

    /// One of the squad chilled to the bone.
    pub fn dev_chill(&mut self, m: PersonId) {
        let t = self.time;
        self.settle_condition(m, t);
        if let Some(c) = self.people[m as usize].cond.as_mut() {
            c.cold = 70.0;
        }
        self.settle_condition(m, t);
    }

    /// One of the squad's pack soaked through.
    pub fn dev_soak(&mut self, m: PersonId) {
        let t = self.time;
        match self.soaks.iter_mut().find(|s| s.pid == m) {
            Some(s) => {
                s.at = t;
                s.through = 1.0;
            }
            None => self.soaks.push(super::soak::Soak { pid: m, at: t, through: 1.0, inside: 0.0, rate: 0.0, inside_rate: 0.0, roofed: false, fire: false, warned: false, proof: 0, food_rate: 1.0 }),
        }
        self.update_soaking();
    }

    /// Every bleed in the squad stopped.
    pub fn dev_bind_all(&mut self) {
        let t = self.time;
        for m in self.squad.members.clone() {
            self.settle_condition(m, t);
            self.people[m as usize].wounds.bleed = 0.0;
        }
    }

    /// A standing torch lit beside one of the squad.
    pub fn dev_torch(&mut self, m: PersonId) -> bool {
        if let Some(d) = self.people[m as usize].detail.as_mut() {
            d.gear.add(super::items::id("standing_torch"), 1);
        }
        self.place_torch(m)
    }

    /// The squad to the nearest mainland town, beside its hearth.
    pub fn dev_go_town(&mut self) {
        let here = self.squad.pos;
        let Some(s) = self.settlements.iter().filter(|s| !s.island).min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))) else { return };
        let at = self.free_spot(s.pos.add(V2::new(12.0, 6.0)));
        self.teleport_squad(at);
    }

    /// The squad to a spot on Salt Isle.
    pub fn dev_go_island(&mut self, spot: super::island::Spot) {
        let Some(at) = self.island().map(|i| i.spot(spot)) else { return };
        let at = self.free_spot(at);
        self.teleport_squad(at);
    }

    /// Into the combat yard: the quarter opened (found first if need be)
    /// and the squad on the arena's west side. False where no quarter fits.
    pub fn dev_to_arena(&mut self) -> bool {
        if !self.enter_test_quarter() {
            return false;
        }
        let Some(at) = self.test_quarter().map(|q| q.squad_side()) else { return false };
        self.teleport_squad(at);
        true
    }

    /// The building arena's store: a storehouse put up at once beside the
    /// base (as a finished site stands, `base_events`), and the store filled
    /// with what building takes, and food, leaving room for the haulers.
    /// How many things went in.
    pub fn dev_stock_base(&mut self, bid: super::base::BaseId) -> u32 {
        use super::base::{def_index, Plan, State, BUILDINGS};
        let Some(i) = self.bases.iter().position(|b| b.id == bid) else { return 0 };
        let centre = self.bases[i].at;
        let def = def_index("storehouse");
        let spot = (0..48).find_map(|k| {
            let a = (k % 12) as f32 / 12.0 * std::f32::consts::TAU;
            let mut p = Plan { def, at: centre.add(V2::new(a.cos(), a.sin()).scale(16.0 + (k / 12) as f32 * 6.0)), rot: 0.0, w: BUILDINGS[def].w, replaces: None };
            self.check_place(&mut p).is_ok().then_some(p)
        });
        if let Some(plan) = spot {
            if let Ok(id) = self.place_building(plan) {
                let t = self.time;
                let b = &mut self.bases[i];
                b.settle_pub(t);
                if let Some(bl) = b.buildings.iter_mut().find(|x| x.id == id) {
                    bl.state = State::Standing { hp: bl.def().hp, since: t, hp_at: t, rate: 0.0 };
                }
                self.solid_changed();
            }
        }
        let mut put = 0u32;
        for &(key, want) in STOCK {
            let it = super::items::id(key);
            let b = &mut self.bases[i];
            let kg = super::items::item(it).weight.max(0.01);
            let room = (b.capacity() * (1.0 - super::baselife::HAUL_SPARE) - b.load()).max(0.0);
            let n = want.min((room / kg) as u16);
            b.add_to_store(it, n);
            put += n as u32;
        }
        self.base_changed(bid);
        put
    }
}

/// What the building arena's store is stocked with: enough for a longhouse,
/// a wall and a few huts, and food for the hands.
const STOCK: &[(&str, u16)] = &[("flatbread", 40), ("timber", 80), ("rock", 50), ("seareed", 30), ("clay", 20), ("iron_ingot", 6)];

