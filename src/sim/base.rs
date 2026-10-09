//! The squad's outposts (Part 7): a camp marker, buildings placed whole,
//! walls drawn as a chain of segments, and construction worked out from the
//! clock.
//!
//! A base starts when a camp marker (a fire ring) is laid. Buildings are
//! placed inside its reach, which grows as it's built up. Each placed
//! building is a construction site until it stands: it takes its materials
//! from the base's store, then labour. Labour is stored as hours done at one
//! moment plus a rate (the builders on it now); between changes it grows
//! steadily, and the moment the site will be finished is solved for, never
//! checked step by step. Anything that changes the rate (a builder arrives
//! or leaves, materials come in, another site finishes and its builders move
//! on) settles every site at that moment and solves again.
//!
//! Every building is a line of data (`BUILDINGS`), with a build method so
//! the peoples' own ways (grown stone, woven stilts, quarried steps, tents)
//! can be added later as more lines.

use serde::{Deserialize, Serialize};

use super::crafting::Station;
use super::geo::V2;
use super::inventory::Entry;
use super::items::{self, item, ItemId};
use super::person::PersonId;
use super::settlement::SettlementId;
use super::stats::Skill;
use super::baselife::{Job, Resident};
use super::world::{World, HOUR};

pub type BaseId = u32;

/// How a building goes up. Only the settlers' way exists so far.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Timber, rubble stone, turf and reed: anyone's, quick.
    Settler,
}

/// What a building is made of, as far as damage goes.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Armour {
    /// Burns.
    Wood,
    /// Doesn't burn; can be broken.
    Stone,
}

/// What a building is for.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The fire ring that founds a base.
    Marker,
    Shelter,
    Store,
    Kitchen,
    Well,
    Field,
    Pen,
    /// A shed housing a crafting station.
    Shed,
    Wall,
    Gate,
    Tower,
}

/// One kind of building, as data.
pub struct BuildingDef {
    pub key: &'static str,
    pub name: &'static str,
    /// The Build panel's heading.
    pub group: &'static str,
    pub kind: Kind,
    pub method: Method,
    /// The building skill it needs (none for the fire ring), and how good a
    /// builder must be to take it on.
    pub skill: Option<Skill>,
    pub min_skill: f32,
    /// Materials (item keys), all delivered before work starts.
    pub needs: &'static [(&'static str, u16)],
    /// Builder-hours at middling skill.
    pub labour: f32,
    /// Footprint: across and deep (metres; for a wall, `w` is the most a
    /// segment may be and `d` its thickness), and how tall it's drawn.
    pub w: f32,
    pub d: f32,
    pub h: f32,
    pub hp: f32,
    pub armour: Armour,
    /// What it's worth standing (coin): the base's wealth counts it.
    pub value: f32,
    pub beds: u8,
    /// Storage it adds, kg.
    pub storage: f32,
    pub station: Option<Station>,
    pub does: &'static str,
}

const fn b(key: &'static str, name: &'static str, group: &'static str, kind: Kind, skill: Option<Skill>, min_skill: f32, needs: &'static [(&'static str, u16)], labour: f32, wdh: (f32, f32, f32), hp: f32, armour: Armour, value: f32, does: &'static str) -> BuildingDef {
    BuildingDef { key, name, group, kind, method: Method::Settler, skill, min_skill, needs, labour, w: wdh.0, d: wdh.1, h: wdh.2, hp, armour, value, beds: 0, storage: 0.0, station: None, does }
}

const fn beds(mut d: BuildingDef, n: u8, storage: f32) -> BuildingDef {
    d.beds = n;
    d.storage = storage;
    d
}

const fn store(mut d: BuildingDef, kg: f32) -> BuildingDef {
    d.storage = kg;
    d
}

const fn shed(mut d: BuildingDef, s: Station) -> BuildingDef {
    d.station = Some(s);
    d
}

const C: Option<Skill> = Some(Skill::Carpentry);
const M: Option<Skill> = Some(Skill::Masonry);
use Armour::{Stone, Wood};

/// Every building the squad can put up.
pub const BUILDINGS: &[BuildingDef] = &[
    b("camp_marker", "Camp marker", "Founding", Kind::Marker, None, 0.0, &[], 0.5, (3.0, 3.0, 0.5), 60.0, Stone, 5.0, "Founds a base: a fire ring of stones from the spot, for cooking and light."),
    beds(b("lean_to", "Lean-to", "Shelter", Kind::Shelter, C, 0.0, &[("timber", 2), ("seareed", 2)], 2.0, (3.0, 2.5, 1.6), 40.0, Wood, 12.0, "One bed, poor rest."), 1, 0.0),
    beds(b("hut", "Hut", "Shelter", Kind::Shelter, C, 0.0, &[("timber", 6), ("seareed", 4)], 8.0, (5.0, 4.0, 3.0), 120.0, Wood, 45.0, "Two beds and a storage chest."), 2, 60.0),
    beds(b("longhouse", "Longhouse", "Shelter", Kind::Shelter, C, 20.0, &[("timber", 20), ("rock", 10), ("seareed", 10)], 40.0, (14.0, 7.0, 5.0), 400.0, Wood, 180.0, "Eight beds round a hearth."), 8, 0.0),
    store(b("storehouse", "Storehouse", "Stores and food", Kind::Store, M, 0.0, &[("timber", 12), ("rock", 12)], 24.0, (8.0, 6.0, 4.0), 360.0, Stone, 120.0, "Large storage; food keeps longer."), 600.0),
    b("hearth_kitchen", "Hearth kitchen", "Stores and food", Kind::Kitchen, M, 0.0, &[("rock", 10), ("clay", 6)], 16.0, (5.0, 4.0, 2.8), 300.0, Stone, 70.0, "Cooks meals from raw food (needs fuel)."),
    b("well", "Well", "Stores and food", Kind::Well, M, 0.0, &[("rock", 14)], 20.0, (2.5, 2.5, 1.2), 300.0, Stone, 60.0, "Water for people, fields and crafting."),
    b("field_plot", "Field plot", "Stores and food", Kind::Field, C, 0.0, &[("timber", 3)], 3.0, (10.0, 8.0, 0.8), 30.0, Wood, 15.0, "Grows food over time."),
    b("animal_pen", "Animal pen", "Stores and food", Kind::Pen, C, 0.0, &[("timber", 8)], 6.0, (10.0, 10.0, 1.2), 80.0, Wood, 30.0, "Holds the base's own livestock."),
    shed(b("shed_forge", "Work shed: forge", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 10), ("rock", 6), ("iron_ingot", 1)], 14.0, (6.0, 5.0, 3.2), 220.0, Wood, 90.0, "A forge under cover."), Station::Forge),
    shed(b("shed_bench", "Work shed: armourer's bench", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 10), ("rock", 6), ("iron_ingot", 1)], 14.0, (6.0, 5.0, 3.2), 220.0, Wood, 90.0, "An armourer's bench under cover."), Station::Bench),
    shed(b("shed_loom", "Work shed: weaver's frame", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 10), ("rock", 4)], 12.0, (6.0, 5.0, 3.2), 200.0, Wood, 75.0, "A weaver's frame and sealing pit."), Station::Loom),
    shed(b("shed_workbench", "Work shed: workbench", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 10), ("rock", 4)], 12.0, (6.0, 5.0, 3.2), 200.0, Wood, 75.0, "Leather, cloth and wood."), Station::Workbench),
    shed(b("shed_desk", "Work shed: scribe's desk", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 8), ("rock", 4)], 10.0, (5.0, 4.0, 3.0), 180.0, Wood, 70.0, "Paper, ink and scrolls."), Station::Desk),
    shed(b("shed_alchemy", "Work shed: alchemy table", "Work sheds", Kind::Shed, C, 0.0, &[("timber", 8), ("rock", 4), ("clay", 2)], 10.0, (5.0, 4.0, 3.0), 180.0, Wood, 70.0, "Potions."), Station::AlchemyTable),
    b("palisade", "Palisade", "Walls and defence", Kind::Wall, C, 0.0, &[("timber", 3)], 2.0, (4.0, 0.6, 3.0), 150.0, Wood, 6.0, "A wall of stakes; slows raiders. Drawn as a chain."),
    b("rubble_wall", "Rubble wall", "Walls and defence", Kind::Wall, M, 10.0, &[("rock", 6)], 5.0, (4.0, 1.0, 2.6), 450.0, Stone, 10.0, "Stronger, hard to burn. Drawn as a chain."),
    b("gate", "Gate", "Walls and defence", Kind::Gate, C, 20.0, &[("timber", 4), ("iron_ingot", 1)], 6.0, (3.5, 0.7, 3.2), 260.0, Wood, 30.0, "Opens and shuts; the weak point. Snaps onto a wall."),
    b("watchtower", "Watchtower", "Walls and defence", Kind::Tower, C, 20.0, &[("timber", 10)], 16.0, (3.0, 3.0, 8.0), 220.0, Wood, 55.0, "Spots raids early; archers shoot better. Snaps onto a wall's corner."),
];

pub fn def_index(key: &str) -> usize {
    BUILDINGS.iter().position(|d| d.key == key).unwrap_or_else(|| panic!("no building {key}"))
}

// ---- Placement numbers ---------------------------------------------------------

/// A base's reach to begin with, and what each standing building adds.
pub const REACH: f32 = 40.0;
pub const REACH_PER_BUILDING: f32 = 3.0;
pub const REACH_MAX: f32 = 120.0;
/// Camp markers keep at least this far apart.
pub const BASE_GAP: f32 = 250.0;
/// Clear of a road's middle by this much.
pub const ROAD_CLEAR: f32 = 4.0;
/// A gate or tower snaps to a wall within this distance of the cursor.
pub const SNAP: f32 = 4.0;
/// Deconstructing a standing building gives back this share of its materials.
pub const SALVAGE: f32 = 0.5;

/// Who owns a base.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Squad,
}

/// Whose land it's on.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Land {
    /// The open wilds: free to settle.
    Wilds,
    /// Within this town's reach, without a claim (the law comes in Stage 4).
    Unclaimed(SettlementId),
}

/// Labour on a construction site: hours done as of `since`, growing at
/// `rate` builder-hours an hour until something changes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Site {
    /// Materials brought to it so far.
    pub delivered: Vec<(ItemId, u16)>,
    pub labour: f32,
    pub since: f64,
    pub rate: f32,
    /// When it will stand, at this rate (None while nobody works it or
    /// materials are short).
    pub done_at: Option<f64>,
    /// Builder-hours each person has put in (they learn from it).
    pub hands: Vec<(PersonId, f32)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum State {
    Site(Site),
    /// Up, since `since`. Its health was `hp` at `hp_at` and moves at
    /// `rate` an hour from there (thatch rotting, builders mending).
    Standing { hp: f32, since: f64, hp_at: f64, rate: f32 },
    /// Burned or broken (Stage 4).
    Ruin,
}

/// A building placed at a base.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Built {
    pub id: u32,
    /// Index into `BUILDINGS`.
    pub def: u16,
    /// Middle of the footprint.
    pub at: V2,
    /// Facing (radians): the footprint's `w` runs across it. A wall
    /// segment's facing runs along it.
    pub rot: f32,
    /// Across, metres (a wall segment's own length).
    pub w: f32,
    pub state: State,
    pub placed: f64,
    /// Its thatch sealed with pitch: it doesn't rot.
    pub sealed: bool,
}

/// Unsealed reed thatch loses this share of a building's health a day.
pub const ROT_PER_DAY: f32 = 0.01;
/// Health a builder mends an hour (at middling pace).
pub const MEND_PER_HOUR: f32 = 25.0;
/// Builders with nothing to build mend what's fallen below this share.
pub const MEND_BELOW: f32 = 0.9;

impl Built {
    pub fn def(&self) -> &'static BuildingDef {
        &BUILDINGS[self.def as usize]
    }
    pub fn standing(&self) -> bool {
        matches!(self.state, State::Standing { .. })
    }
    /// When it was finished, if it stands.
    pub fn stood_at(&self) -> Option<f64> {
        match self.state {
            State::Standing { since, .. } => Some(since),
            _ => None,
        }
    }
    pub fn site(&self) -> Option<&Site> {
        match &self.state {
            State::Site(s) => Some(s),
            _ => None,
        }
    }
    /// Does its roof rot (reed thatch, unsealed)?
    pub fn rots(&self) -> bool {
        !self.sealed && self.def().needs.iter().any(|(k, _)| *k == "seareed")
    }
    /// Health lost an hour to rot.
    pub fn rot_rate(&self) -> f32 {
        if self.rots() { self.def().hp * ROT_PER_DAY / 24.0 } else { 0.0 }
    }
    /// Health at `t`.
    pub fn hp_at(&self, t: f64) -> f32 {
        match self.state {
            State::Standing { hp, hp_at, rate, .. } => (hp + rate * ((t - hp_at).max(0.0) / HOUR) as f32).clamp(0.0, self.def().hp),
            State::Site(_) => self.def().hp * self.progress(t),
            State::Ruin => 0.0,
        }
    }
    /// The footprint's four corners.
    pub fn corners(&self) -> [V2; 4] {
        corners(self.at, self.rot, self.w, self.def().d)
    }
    /// Materials still to come.
    pub fn missing(&self) -> Vec<(ItemId, u16)> {
        let Some(s) = self.site() else { return Vec::new() };
        self.def()
            .needs
            .iter()
            .filter_map(|&(k, n)| {
                let id = items::id(k);
                let got = s.delivered.iter().filter(|e| e.0 == id).map(|e| e.1).sum::<u16>();
                (got < n).then_some((id, n - got))
            })
            .collect()
    }
    /// How far along its labour is, 0..1, at `t`.
    pub fn progress(&self, t: f64) -> f32 {
        match &self.state {
            State::Site(s) => ((s.labour + s.rate * ((t - s.since).max(0.0) / HOUR) as f32) / self.def().labour).clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

/// A rectangle's corners: `w` across the facing, `d` along it.
pub fn corners(at: V2, rot: f32, w: f32, d: f32) -> [V2; 4] {
    let f = V2::new(rot.cos(), rot.sin());
    let s = V2::new(-f.y, f.x);
    let (hw, hd) = (w * 0.5, d * 0.5);
    [at.add(s.scale(hw)).add(f.scale(hd)), at.sub(s.scale(hw)).add(f.scale(hd)), at.sub(s.scale(hw)).sub(f.scale(hd)), at.add(s.scale(hw)).sub(f.scale(hd))]
}

/// Do two convex quads overlap (separating axes)?
pub fn overlap(a: &[V2; 4], b: &[V2; 4]) -> bool {
    for poly in [a, b] {
        for i in 0..4 {
            let (p, q) = (poly[i], poly[(i + 1) % 4]);
            let axis = V2::new(-(q.y - p.y), q.x - p.x);
            let proj = |c: &[V2; 4]| {
                let v: Vec<f32> = c.iter().map(|v| v.x * axis.x + v.y * axis.y).collect();
                (v.iter().cloned().fold(f32::INFINITY, f32::min), v.iter().cloned().fold(f32::NEG_INFINITY, f32::max))
            };
            let (a0, a1) = proj(a);
            let (b0, b1) = proj(b);
            if a1 <= b0 + 0.01 || b1 <= a0 + 0.01 {
                return false;
            }
        }
    }
    true
}

fn seg_dist(p: V2, a: V2, b: V2) -> f32 {
    let d = b.sub(a);
    let l2 = d.x * d.x + d.y * d.y;
    let t = if l2 > 0.0 { (((p.x - a.x) * d.x + (p.y - a.y) * d.y) / l2).clamp(0.0, 1.0) } else { 0.0 };
    p.dist(a.add(d.scale(t)))
}

/// An outpost.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Base {
    pub id: BaseId,
    pub name: String,
    pub owner: Owner,
    /// The camp marker.
    pub at: V2,
    pub founded: f64,
    pub land: Land,
    pub buildings: Vec<Built>,
    pub next_id: u32,
    /// What's kept here.
    pub store: Vec<Entry>,
    /// Who was at the base the last time it was looked at, and which site
    /// each is building (by building id).
    pub present: Vec<PersonId>,
    /// (who, which building, their pace): on a site or mending.
    pub builders: Vec<(PersonId, u32, f32)>,
    /// Squad members left here, and their work.
    pub residents: Vec<Resident>,
    /// What's happened here, oldest first (capped at `BASE_LOG`).
    pub log: Vec<(f64, String)>,
    /// The last day whose dawn has been tallied here (wages, hands' food).
    pub dawn_done: i64,
    /// Cached summary numbers, refreshed when something changes: what the
    /// base is worth (stored goods and buildings), and how well it's
    /// defended (walls, gates and towers standing).
    pub wealth: f32,
    pub defence: f32,
}

/// Why a building can't go here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bad {
    Steep,
    Water,
    Overlaps,
    Road,
    /// On a town's buildings.
    Town,
    /// Outside the base's reach.
    TooFar,
    /// No base here: lay a camp marker first.
    NoBase,
    /// Too near another camp.
    NearBase,
    /// A gate or tower needs a wall to sit on.
    NeedsWall,
    /// A wall chain needs two points.
    TooShort,
}

impl Bad {
    pub fn why(self) -> &'static str {
        match self {
            Bad::Steep => "too steep",
            Bad::Water => "in the water",
            Bad::Overlaps => "in the way of another building",
            Bad::Road => "on a road",
            Bad::Town => "on a town's buildings",
            Bad::TooFar => "too far from the camp marker",
            Bad::NoBase => "lay a camp marker first",
            Bad::NearBase => "too near another camp",
            Bad::NeedsWall => "must sit on a wall",
            Bad::TooShort => "too short",
        }
    }
}

/// A building ready to place: where, which way, how wide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub def: usize,
    pub at: V2,
    pub rot: f32,
    pub w: f32,
    /// A gate replacing this wall segment.
    pub replaces: Option<u32>,
}

impl Base {
    pub fn reach(&self) -> f32 {
        let n = self.buildings.iter().filter(|b| b.standing() && !matches!(b.def().kind, Kind::Wall | Kind::Gate | Kind::Tower | Kind::Marker)).count();
        (REACH + REACH_PER_BUILDING * n as f32).min(REACH_MAX)
    }

    pub fn building(&self, id: u32) -> Option<&Built> {
        self.buildings.iter().find(|b| b.id == id)
    }

    /// Has this hired hand got here yet (their arrival handled)?
    pub fn arrived(&self, who: PersonId) -> bool {
        self.residents.iter().find(|r| r.who == who).is_some_and(|r| r.hire.as_ref().is_none_or(|h| r.since >= h.arrives))
    }

    pub fn beds(&self) -> u32 {
        self.buildings.iter().filter(|b| b.standing()).map(|b| b.def().beds as u32).sum()
    }

    pub fn count_in_store(&self, id: ItemId) -> u16 {
        self.store.iter().filter(|e| e.0 == id).map(|e| e.1).sum()
    }

    fn take_from_store(&mut self, id: ItemId, n: u16) -> u16 {
        let mut left = n;
        for e in self.store.iter_mut().filter(|e| e.0 == id && e.2.is_none()) {
            let k = e.1.min(left);
            e.1 -= k;
            left -= k;
        }
        self.store.retain(|e| e.1 > 0);
        n - left
    }

    pub fn add_to_store(&mut self, id: ItemId, n: u16) {
        if n == 0 {
            return;
        }
        if let Some(e) = self.store.iter_mut().find(|e| e.0 == id && e.2.is_none()) {
            e.1 += n;
        } else {
            self.store.push(Entry(id, n, None));
        }
    }

    pub(super) fn settle_pub(&mut self, t: f64) {
        self.settle(t);
    }

    /// Settle every site's labour and every building's health at `t`
    /// (rates unchanged).
    fn settle(&mut self, t: f64) {
        for b in &mut self.buildings {
            let max = b.def().hp;
            match &mut b.state {
                State::Site(s) => {
                    if t > s.since {
                        let h = ((t - s.since) / HOUR) as f32;
                        s.labour += s.rate * h;
                        for hand in self.builders.iter().filter(|x| x.1 == b.id) {
                            match s.hands.iter_mut().find(|e| e.0 == hand.0) {
                                Some(e) => e.1 += hand.2 * h,
                                None => s.hands.push((hand.0, hand.2 * h)),
                            }
                        }
                    }
                    s.since = s.since.max(t);
                }
                State::Standing { hp, hp_at, rate, .. } => {
                    if t > *hp_at {
                        *hp = (*hp + *rate * ((t - *hp_at) / HOUR) as f32).clamp(0.0, max);
                        *hp_at = t;
                    }
                }
                State::Ruin => {}
            }
        }
    }

    /// Recompute the cached wealth and defence.
    fn summarise(&mut self) {
        let goods: f32 = self.store.iter().map(|e| item(e.0).value * e.1 as f32).sum();
        let built: f32 = self.buildings.iter().filter(|b| b.standing()).map(|b| b.def().value).sum();
        self.wealth = goods + built;
        self.defence = self
            .buildings
            .iter()
            .filter_map(|b| match b.state {
                State::Standing { hp, .. } if matches!(b.def().kind, Kind::Wall | Kind::Gate | Kind::Tower) => Some(hp * if b.def().kind == Kind::Tower { 0.3 } else { 0.05 }),
                _ => None,
            })
            .sum();
    }
}

/// A builder's pace: builder-hours of work an hour.
pub fn pace(skill: f32) -> f32 {
    0.5 + skill / 100.0
}

impl World {
    // ---- Asking ---------------------------------------------------------------

    /// The base whose reach covers `p`, if any.
    pub fn base_at(&self, p: V2) -> Option<BaseId> {
        self.bases.iter().filter(|b| b.at.dist(p) <= b.reach()).min_by(|a, b| a.at.dist(p).total_cmp(&b.at.dist(p))).map(|b| b.id)
    }

    pub fn base(&self, id: BaseId) -> Option<&Base> {
        self.bases.iter().find(|b| b.id == id)
    }

    fn base_index(&self, id: BaseId) -> Option<usize> {
        self.bases.iter().position(|b| b.id == id)
    }

    /// Does this person know the building skill, and well enough?
    pub fn can_build(&self, who: PersonId, def: usize) -> bool {
        let d = &BUILDINGS[def];
        match d.skill {
            None => true,
            Some(s) => {
                let p = &self.people[who as usize];
                let knows = if p.in_squad { p.detail.as_ref().is_some_and(|x| x.crafts.contains(&s)) } else { super::materials::Craft::of_skill(s).is_some_and(|c| self.knows_craft(who, c)) };
                knows && self.build_skill(who, s) >= d.min_skill
            }
        }
    }

    /// Who in the squad can put this up.
    pub fn squad_builders(&self, def: usize) -> Vec<PersonId> {
        self.squad.members.iter().copied().filter(|&m| !self.people[m as usize].dead && self.can_build(m, def)).collect()
    }

    // ---- Placing -------------------------------------------------------------------

    /// Can a building go here (and at which base)? `plan` may be adjusted:
    /// a gate or tower snaps onto the nearest wall.
    pub fn check_place(&self, plan: &mut Plan) -> Result<Option<BaseId>, Bad> {
        let def = &BUILDINGS[plan.def];
        if def.kind == Kind::Marker {
            if self.bases.iter().any(|b| b.at.dist(plan.at) < BASE_GAP) {
                return Err(Bad::NearBase);
            }
            self.check_ground(plan, None)?;
            return Ok(None);
        }
        let Some(bid) = self.base_at(plan.at) else { return Err(Bad::NoBase) };
        let base = self.base(bid).unwrap();
        // Gates and towers sit on a wall.
        if matches!(def.kind, Kind::Gate | Kind::Tower) {
            let walls = base.buildings.iter().filter(|b| b.def().kind == Kind::Wall);
            match def.kind {
                Kind::Gate => {
                    let near = walls.filter(|b| b.w >= def.w - 0.01).min_by(|a, b| a.at.dist(plan.at).total_cmp(&b.at.dist(plan.at)));
                    match near {
                        Some(wl) if wl.at.dist(plan.at) <= SNAP => {
                            plan.at = wl.at;
                            plan.rot = wl.rot;
                            plan.w = wl.w;
                            plan.replaces = Some(wl.id);
                        }
                        _ => return Err(Bad::NeedsWall),
                    }
                }
                _ => {
                    let mut best: Option<(f32, V2)> = None;
                    for wl in walls {
                        let f = V2::new(wl.rot.cos(), wl.rot.sin()).scale(wl.w * 0.5);
                        for end in [wl.at.add(f), wl.at.sub(f)] {
                            let d = end.dist(plan.at);
                            if d <= SNAP && best.is_none_or(|(bd, _)| d < bd) {
                                best = Some((d, end));
                            }
                        }
                    }
                    match best {
                        Some((_, p)) => plan.at = p,
                        None => return Err(Bad::NeedsWall),
                    }
                }
            }
        }
        let poly = corners(plan.at, plan.rot, plan.w, def.d);
        if poly.iter().chain(std::iter::once(&plan.at)).any(|c| c.dist(base.at) > base.reach()) {
            return Err(Bad::TooFar);
        }
        self.check_ground(plan, Some(bid))?;
        Ok(Some(bid))
    }

    /// The ground, the water, the roads, the towns and the other buildings.
    fn check_ground(&self, plan: &Plan, _base: Option<BaseId>) -> Result<(), Bad> {
        let def = &BUILDINGS[plan.def];
        let poly = corners(plan.at, plan.rot, plan.w, def.d);
        let mut pts: Vec<V2> = poly.to_vec();
        pts.push(plan.at);
        for i in 0..4 {
            pts.push(poly[i].lerp(poly[(i + 1) % 4], 0.5));
        }
        let t = &self.terrain;
        if pts.iter().any(|&p| t.is_sea(p)) {
            return Err(Bad::Water);
        }
        let hs: Vec<f32> = pts.iter().map(|&p| t.height(p)).collect();
        let (lo, hi) = (hs.iter().cloned().fold(f32::INFINITY, f32::min), hs.iter().cloned().fold(f32::NEG_INFINITY, f32::max));
        let span = plan.w.max(def.d);
        let tol = match def.kind {
            Kind::Wall | Kind::Gate => 0.45 * span + 0.4,
            _ => 0.14 * span + 0.5,
        };
        if hi - lo > tol {
            return Err(Bad::Steep);
        }
        // Roads.
        let (mn, mx) = pts.iter().fold((V2::new(f32::INFINITY, f32::INFINITY), V2::new(f32::NEG_INFINITY, f32::NEG_INFINITY)), |(a, b), p| (V2::new(a.x.min(p.x), a.y.min(p.y)), V2::new(b.x.max(p.x), b.y.max(p.y))));
        for road in &self.routes.roads {
            for s in road.windows(2) {
                let (a, c) = (s[0], s[1]);
                if a.x.max(c.x) < mn.x - ROAD_CLEAR || a.x.min(c.x) > mx.x + ROAD_CLEAR || a.y.max(c.y) < mn.y - ROAD_CLEAR || a.y.min(c.y) > mx.y + ROAD_CLEAR {
                    continue;
                }
                if pts.iter().any(|&p| seg_dist(p, a, c) < ROAD_CLEAR) {
                    return Err(Bad::Road);
                }
            }
        }
        // Towns' buildings, and the forged town's homes.
        let hits = |c: V2, r: f32| -> bool {
            let box_ = corners(c, 0.0, r * 2.0, r * 2.0);
            overlap(&poly, &box_)
        };
        for s in &self.settlements {
            if s.pos.dist(plan.at) > s.reach + 200.0 {
                continue;
            }
            if s.buildings.iter().any(|bd| hits(bd.pos, bd.size * 0.5 + 1.0)) {
                return Err(Bad::Town);
            }
        }
        if let Some(f) = &self.forge {
            if f.founding.homes.iter().any(|h| hits(h.at, 6.0)) {
                return Err(Bad::Town);
            }
        }
        // Other buildings at any base (a gate may sit on its own wall; a
        // tower on the walls it joins).
        for base in &self.bases {
            if base.at.dist(plan.at) > REACH_MAX + 30.0 {
                continue;
            }
            for bl in &base.buildings {
                if Some(bl.id) == plan.replaces && Some(base.id) == _base {
                    continue;
                }
                if def.kind == Kind::Tower && bl.def().kind == Kind::Wall {
                    continue;
                }
                if def.kind == Kind::Wall && bl.def().kind == Kind::Tower {
                    continue;
                }
                if overlap(&poly, &bl.corners()) {
                    return Err(Bad::Overlaps);
                }
            }
        }
        Ok(())
    }

    /// Lay a camp marker: a new base.
    pub fn found_base(&mut self, at: V2) -> Result<BaseId, Bad> {
        let mut plan = Plan { def: def_index("camp_marker"), at, rot: 0.0, w: BUILDINGS[def_index("camp_marker")].w, replaces: None };
        self.check_place(&mut plan)?;
        let id = self.next_base;
        self.next_base += 1;
        let land = match self.settlements.iter().filter(|s| s.pos.dist(at) <= s.reach).min_by(|a, b| a.pos.dist(at).total_cmp(&b.pos.dist(at))) {
            Some(s) => Land::Unclaimed(s.id),
            None => Land::Wilds,
        };
        let name = format!("Outpost {}", id + 1);
        let mut base = Base { id, name, owner: Owner::Squad, at, founded: self.time, land, buildings: Vec::new(), next_id: 0, store: Vec::new(), present: Vec::new(), builders: Vec::new(), residents: Vec::new(), log: Vec::new(), dawn_done: (self.time / super::world::DAY).floor() as i64, wealth: 0.0, defence: 0.0 };
        base.buildings.push(Built { id: 0, def: plan.def as u16, at, rot: 0.0, w: plan.w, state: State::Site(Site { delivered: Vec::new(), labour: 0.0, since: self.time, rate: 0.0, done_at: None, hands: Vec::new() }), placed: self.time, sealed: false });
        base.next_id = 1;
        self.bases.push(base);
        self.say_base(id, format!("A camp marker is laid: {}.", self.bases.last().unwrap().name));
        self.base_changed(id);
        Ok(id)
    }

    /// Place a building (a construction site) as planned.
    pub fn place_building(&mut self, mut plan: Plan) -> Result<u32, Bad> {
        if BUILDINGS[plan.def].kind == Kind::Marker {
            return self.found_base(plan.at).map(|_| 0);
        }
        let bid = self.check_place(&mut plan)?.ok_or(Bad::NoBase)?;
        let t = self.time;
        if let Some(old) = plan.replaces {
            self.remove_building(bid, old);
        }
        let i = self.base_index(bid).unwrap();
        let b = &mut self.bases[i];
        b.settle(t);
        let id = b.next_id;
        b.next_id += 1;
        b.buildings.push(Built { id, def: plan.def as u16, at: plan.at, rot: plan.rot, w: plan.w, state: State::Site(Site { delivered: Vec::new(), labour: 0.0, since: t, rate: 0.0, done_at: None, hands: Vec::new() }), placed: t, sealed: false });
        self.base_changed(bid);
        Ok(id)
    }

    /// The segments a wall drawn through these points would have (each no
    /// longer than the wall's `w`).
    pub fn wall_plans(def: usize, pts: &[V2]) -> Vec<Plan> {
        let d = &BUILDINGS[def];
        let mut out = Vec::new();
        for s in pts.windows(2) {
            let (a, c) = (s[0], s[1]);
            let len = a.dist(c);
            if len < 0.5 {
                continue;
            }
            let n = (len / d.w).ceil().max(1.0) as usize;
            let rot = (c.y - a.y).atan2(c.x - a.x);
            for k in 0..n {
                let p0 = a.lerp(c, k as f32 / n as f32);
                let p1 = a.lerp(c, (k + 1) as f32 / n as f32);
                // A wall segment's `w` runs along it: face across it.
                out.push(Plan { def, at: p0.lerp(p1, 0.5), rot: rot + std::f32::consts::FRAC_PI_2, w: len / n as f32, replaces: None });
            }
        }
        out
    }

    /// Lay a wall along a chain of points; segments that can't go are
    /// skipped. Returns the ids placed.
    pub fn place_wall(&mut self, def: usize, pts: &[V2]) -> Result<Vec<u32>, Bad> {
        if pts.len() < 2 {
            return Err(Bad::TooShort);
        }
        let mut placed = Vec::new();
        let mut last_err = Bad::TooShort;
        for plan in Self::wall_plans(def, pts) {
            match self.place_building(plan) {
                Ok(id) => placed.push(id),
                Err(e) => last_err = e,
            }
        }
        if placed.is_empty() {
            Err(last_err)
        } else {
            Ok(placed)
        }
    }

    /// Take a building down. A site gives back everything delivered to it; a
    /// standing building `SALVAGE` of its materials.
    pub fn deconstruct(&mut self, bid: BaseId, id: u32) -> bool {
        if self.base(bid).and_then(|b| b.building(id)).is_some_and(|bl| bl.def().kind == Kind::Marker) {
            return false;
        }
        let ok = self.remove_building(bid, id);
        if ok {
            self.base_changed(bid);
        }
        ok
    }

    fn remove_building(&mut self, bid: BaseId, id: u32) -> bool {
        let t = self.time;
        let Some(i) = self.base_index(bid) else { return false };
        let b = &mut self.bases[i];
        let Some(k) = b.buildings.iter().position(|x| x.id == id) else { return false };
        b.settle(t);
        let bl = b.buildings.remove(k);
        match &bl.state {
            State::Site(s) => {
                for &(it, n) in &s.delivered {
                    b.add_to_store(it, n);
                }
            }
            State::Standing { .. } => {
                for &(key, n) in bl.def().needs {
                    b.add_to_store(items::id(key), (n as f32 * SALVAGE).floor() as u16);
                }
                if let Some(st) = bl.def().station {
                    if let Some(j) = self.stations.iter().position(|(p, s)| *s == st && p.dist(bl.at) < 0.5) {
                        self.stations.remove(j);
                    }
                }
            }
            State::Ruin => {}
        }
        true
    }

    /// Building materials from the packs of everyone at the base into its
    /// store. How many items moved.
    pub fn store_materials(&mut self, bid: BaseId) -> u32 {
        let Some(i) = self.base_index(bid) else { return 0 };
        let (at, reach) = (self.bases[i].at, self.bases[i].reach());
        let keys: Vec<ItemId> = {
            let mut v: Vec<ItemId> = BUILDINGS.iter().flat_map(|d| d.needs.iter().map(|&(k, _)| items::id(k))).collect();
            v.sort();
            v.dedup();
            v
        };
        let mut moved = 0u32;
        for k in 0..self.squad.members.len() {
            let m = self.squad.members[k];
            if self.squad.at[k].dist(at) > reach || self.people[m as usize].dead {
                continue;
            }
            let Some(d) = self.people[m as usize].detail.as_mut() else { continue };
            for &id in &keys {
                // What the sites still need goes straight to them; beyond
                // that, as much as the store has room for.
                let want: u16 = self.bases[i].buildings.iter().flat_map(|bl| bl.missing()).filter(|e| e.0 == id).map(|e| e.1).sum::<u16>().saturating_sub(self.bases[i].count_in_store(id));
                let w = item(id).weight.max(0.01);
                let mut n = 0u16;
                while (n < want || self.bases[i].room_for(w * (n + 1 - want.min(n + 1)) as f32)) && d.gear.take(id) {
                    n += 1;
                }
                if n > 0 {
                    self.bases[i].add_to_store(id, n);
                    moved += n as u32;
                }
            }
            self.people[m as usize].recompute_might();
        }
        if moved > 0 {
            self.base_changed(bid);
        }
        moved
    }

    // ---- Construction, mending and work on the clock -----------------------------

    /// Something at a base changed at `self.time`: settle, deliver from the
    /// store, hand out the builders, start idle workers, and solve again.
    pub(super) fn base_changed(&mut self, bid: BaseId) {
        self.base_changed_at(bid, self.time);
    }

    pub(super) fn base_changed_at(&mut self, bid: BaseId, t: f64) {
        let Some(i) = self.base_index(bid) else { return };
        self.bases[i].settle(t);
        // Deliver: sites in the order they were placed take what they need.
        let n = self.bases[i].buildings.len();
        for k in 0..n {
            let missing = self.bases[i].buildings[k].missing();
            for (it, want) in missing {
                let got = self.bases[i].take_from_store(it, want);
                if got > 0 {
                    if let State::Site(s) = &mut self.bases[i].buildings[k].state {
                        match s.delivered.iter_mut().find(|e| e.0 == it) {
                            Some(e) => e.1 += got,
                            None => s.delivered.push((it, got)),
                        }
                    }
                }
            }
        }
        // Builders: the squad at the base, and residents set to build. Each
        // goes to the first site (in placing order) with its materials that
        // they can build; with none, to mending the worst-kept building.
        let mut hands: Vec<PersonId> = self.bases[i].present.clone();
        hands.extend(self.bases[i].residents.iter().filter(|r| r.job == Job::Builder && self.bases[i].arrived(r.who)).map(|r| r.who));
        let was_mending: Vec<u32> = self.bases[i].builders.iter().map(|h| h.1).filter(|&id| self.bases[i].building(id).is_some_and(|b| b.standing())).collect();
        let mut builders: Vec<(PersonId, u32, f32)> = Vec::new();
        for &m in &hands {
            let b = &self.bases[i];
            let site = b.buildings.iter().find(|bl| bl.site().is_some() && bl.missing().is_empty() && self.can_build(m, bl.def as usize)).map(|bl| bl.id);
            let target = site.or_else(|| {
                b.buildings
                    .iter()
                    .filter(|bl| bl.standing() && self.can_build(m, bl.def as usize))
                    .filter(|bl| {
                        let r = bl.hp_at(t) / bl.def().hp;
                        r < MEND_BELOW || (was_mending.contains(&bl.id) && r < 0.999)
                    })
                    .min_by(|a, c| (a.hp_at(t) / a.def().hp).total_cmp(&(c.hp_at(t) / c.def().hp)))
                    .map(|bl| bl.id)
            });
            if let Some(id) = target {
                let d = b.building(id).unwrap().def();
                let skill = d.skill.map(|s| self.build_skill(m, s)).unwrap_or(30.0);
                builders.push((m, id, pace(skill)));
            }
        }
        let b = &mut self.bases[i];
        b.builders = builders;
        for bl in &mut b.buildings {
            let total = bl.def().labour;
            let id = bl.id;
            let on: f32 = b.builders.iter().filter(|h| h.1 == id).map(|h| h.2).sum();
            let rot = bl.rot_rate();
            match &mut bl.state {
                State::Site(s) => {
                    s.rate = on;
                    s.since = t;
                    s.done_at = (s.rate > 0.0).then(|| t + ((total - s.labour).max(0.0) / s.rate) as f64 * HOUR);
                }
                State::Standing { rate, hp_at, .. } => {
                    *rate = on * MEND_PER_HOUR - rot;
                    *hp_at = t;
                }
                State::Ruin => {}
            }
        }
        // Workers with nothing on their hands start their next round.
        for r in 0..self.bases[i].residents.len() {
            if self.bases[i].residents[r].cycle.is_none() {
                self.start_cycle(i, r, t);
            }
        }
        self.bases[i].summarise();
        // Guards on watch add their strength.
        let guards: f32 = self.bases[i].residents.iter().filter(|r| r.job == Job::Guard).map(|r| self.people[r.who as usize].might).sum();
        self.bases[i].defence += guards;
    }

    /// The next thing due at any base: (when, base, what).
    fn next_base_event(&self) -> Option<(f64, BaseId, Due)> {
        let mut best: Option<(f64, BaseId, Due)> = None;
        let mut offer = |t: f64, b: BaseId, d: Due| {
            if best.is_none_or(|x| t < x.0) {
                best = Some((t, b, d));
            }
        };
        for b in &self.bases {
            for bl in &b.buildings {
                match bl.state {
                    State::Site(ref s) => {
                        if let Some(t) = s.done_at {
                            offer(t, b.id, Due::Stands(bl.id));
                        }
                    }
                    State::Standing { hp, hp_at, rate, .. } => {
                        let max = bl.def().hp;
                        if rate < 0.0 {
                            offer(hp_at + (hp / -rate) as f64 * HOUR, b.id, Due::Falls(bl.id));
                        } else if rate > 0.0 && hp < max {
                            offer(hp_at + ((max - hp) / rate) as f64 * HOUR, b.id, Due::Mended(bl.id));
                        }
                    }
                    State::Ruin => {}
                }
            }
            for r in &b.residents {
                if let Some(c) = &r.cycle {
                    offer(c.done_at, b.id, Due::Work(r.who));
                }
                if let Some(h) = &r.hire {
                    if !b.arrived(r.who) {
                        offer(h.arrives, b.id, Due::Arrives(r.who));
                    }
                }
            }
            // The day's tally, once there are hands to pay.
            if b.residents.iter().any(|r| r.hire.is_some()) {
                let dawn = ((b.dawn_done + 1) * 24 + super::society::DAWN) as f64 * HOUR;
                offer(dawn, b.id, Due::Dawn);
            }
        }
        best
    }

    /// One base event due before `before` (and by now), on the world's
    /// timeline. True if one was handled.
    pub(super) fn base_events(&mut self, before: f64) -> bool {
        let Some((t, bid, due)) = self.next_base_event().filter(|e| e.0 <= self.time && e.0 < before) else { return false };
        let i = self.base_index(bid).unwrap();
        self.bases[i].settle(t);
        match due {
            Due::Stands(id) => {
                let k = self.bases[i].buildings.iter().position(|x| x.id == id).unwrap();
                let bl = &mut self.bases[i].buildings[k];
                let d = bl.def();
                // Whoever put the work in learns from it.
                let hands = match &bl.state {
                    State::Site(s) => s.hands.clone(),
                    _ => Vec::new(),
                };
                bl.state = State::Standing { hp: d.hp, since: t, hp_at: t, rate: 0.0 };
                let (at, name) = (bl.at, d.name);
                if let (Some(sk), true) = (d.skill, d.labour > 0.0) {
                    for (who, hours) in hands {
                        self.people[who as usize].stats.exercise(sk, hours * PRACTICE);
                    }
                }
                if let Some(st) = d.station {
                    self.stations.push((at, st));
                }
                let bname = self.bases[i].name.clone();
                self.base_note(bid, t, format!("{name} stands at {bname}."), true);
            }
            Due::Mended(id) => {
                if let Some(bl) = self.bases[i].buildings.iter_mut().find(|x| x.id == id) {
                    let max = bl.def().hp;
                    if let State::Standing { hp, .. } = &mut bl.state {
                        *hp = max;
                    }
                }
            }
            Due::Falls(id) => {
                let Some(k) = self.bases[i].buildings.iter().position(|x| x.id == id) else { return true };
                let bl = &mut self.bases[i].buildings[k];
                bl.state = State::Ruin;
                let (at, d) = (bl.at, bl.def());
                if let Some(st) = d.station {
                    if let Some(j) = self.stations.iter().position(|(p, s)| *s == st && p.dist(at) < 0.5) {
                        self.stations.remove(j);
                    }
                }
                self.base_note(bid, t, format!("The {} has fallen in: its thatch rotted.", d.name.to_lowercase()), true);
            }
            Due::Work(who) => self.finish_cycle(i, who, t),
            Due::Arrives(who) => {
                if let Some(r) = self.bases[i].residents.iter_mut().find(|r| r.who == who) {
                    r.since = t;
                }
                let name = self.people[who as usize].name().unwrap_or("someone").to_string();
                let bname = self.bases[i].name.clone();
                self.base_note(bid, t, format!("{name} arrives at {bname}."), false);
            }
            Due::Dawn => self.base_dawn(i, t),
        }
        self.base_changed_at(bid, t);
        true
    }

    /// Who of the squad is at each base, now; anyone come or gone settles it.
    pub(super) fn refresh_bases(&mut self) {
        let t = self.time;
        for i in 0..self.bases.len() {
            let (at, reach) = (self.bases[i].at, self.bases[i].reach());
            let present: Vec<PersonId> = (0..self.squad.members.len())
                .filter(|&k| {
                    let m = self.squad.members[k];
                    let p = &self.people[m as usize];
                    self.squad.at[k].dist(at) <= reach && !p.dead && !self.fighting.contains_key(&m) && !super::body::knocked_out(&p.wounds.hp_at(&p.stats, t))
                })
                .map(|k| self.squad.members[k])
                .collect();
            if present != self.bases[i].present {
                self.bases[i].present = present;
                let id = self.bases[i].id;
                self.base_changed(id);
            }
        }
    }

    fn say_base(&mut self, bid: BaseId, line: String) {
        self.base_note(bid, self.time, line, true);
    }

    /// A line in the base's own log (and, if `loud`, the journal).
    pub(super) fn base_note(&mut self, bid: BaseId, t: f64, line: String, loud: bool) {
        if let Some(i) = self.base_index(bid) {
            let log = &mut self.bases[i].log;
            log.push((t, line.clone()));
            if log.len() > BASE_LOG {
                log.remove(0);
            }
        }
        if loud {
            self.log.push_front((t, line));
            self.log.truncate(14);
        }
    }
}

/// Something due at a base.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Due {
    Stands(u32),
    Mended(u32),
    Falls(u32),
    Work(PersonId),
    /// A hired hand reaches the base.
    Arrives(PersonId),
    /// The day's tally.
    Dawn,
}

/// How much of a builder's hours become practice in the skill.
pub const PRACTICE: f32 = 0.4;
/// How many lines a base's log keeps.
pub const BASE_LOG: usize = 40;
