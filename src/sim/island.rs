//! Salt Isle (`docs/tutorial-island.md`, Laz): one small island offshore
//! that is both the game's opening and the dev test quarter. A labour
//! island run round a main compound where bonded debtors work; the Binders
//! run it for a Roduro Free Lord.
//!
//! **Where it is** is fixed geometry like the coast itself
//! (`geo::isle_centre()`): the same spot in every world, whatever the seed,
//! so `geo::inland` and `Terrain` know it as land and nothing else has to.
//! Its shape and height are pure functions of the place (`geo::isle_inland`,
//! `height`); its spots (`Spot`) are fixed offsets from the centre.
//!
//! **The compound is a real settlement** (`Settlement::island`), added after
//! everyone else's dice have been rolled (after the squad and the wanderers,
//! before the roads), with its own extra people: Binder guards and
//! overseers (Qotiro), the lord's steward (Roduro), and the debtors (mixed),
//! who are in slave bonds to the steward from the start. No road or caravan
//! reaches it (`Routes::build`, `send_caravan` skip it), its customs always
//! allow slavery (`reblend`), and its workplaces are laid where the brief
//! puts them (`lay_out`), not spiralled from the hearth.
//!
//! Not here yet (step 2, `tutorial.rs`): the shipwreck start, the shelter
//! hut, the lessons, the ship off. The test quarter's contents are the
//! second session's (`testquarter.rs`); this only keeps its ground flat.

use serde::{Deserialize, Serialize};

use super::culture::Slavery;
use super::geo::{self, V2};
use super::jobs::{Good, Job, PlaceKind};
use super::society::Workplace;
use super::person::{Person, PersonId};
use super::race::Race;
use super::rng::{self, Rng};
use super::settlement::{Building, BuildingKind, Settlement, SettlementId};
use super::world::World;

/// The island's name as the game shows it.
pub const NAME: &str = "Salt Isle";

/// Binder guards on the island.
pub const GUARDS: usize = 10;
/// Overseers (Binders too).
pub const OVERSEERS: usize = 3;
/// Bonded debtors, by people: Roduro, Horaro, Qotiro, Ṭaḍoro.
pub const DEBTORS: [usize; 4] = [10, 6, 6, 2];

/// Flat ground from this far in from the shore.
const FLAT_FROM: f32 = 45.0;
/// The flat ground's height.
const FLAT: f32 = 8.0;
/// The north hill: how high, and where it starts rising (metres north of the centre).
const HILL: f32 = 12.0;
const HILL_FROM: f32 = 130.0;
const HILL_TO: f32 = 230.0;

/// The island's ground height at `p` (only asked inside its outline).
pub fn height(p: V2) -> f32 {
    let d = geo::isle_inland(p);
    let off = p.sub(geo::isle_centre());
    let north = -off.y;
    let base = 1.0 + (FLAT - 1.0) * smooth(0.0, FLAT_FROM, d);
    // The wild north edge rises; the compound and the test quarter stay flat.
    let hill = HILL * smooth(HILL_FROM, HILL_TO, north) * smooth(0.0, FLAT_FROM, d);
    base + hill
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The island's named places. North is -y (`quests::compass`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spot {
    /// The south-east shore where the player is wrecked.
    Shore,
    /// The wreck itself, in the shallows off the shore.
    Wreck,
    /// The shelter hut above the shore.
    Hut,
    /// The compound's gate, on its south side.
    Gate,
    /// The overseers' hall.
    Hall,
    /// The debtors' bunkhouses.
    Bunkhouses,
    /// The workyard.
    Workyard,
    /// The stores.
    Stores,
    /// The yard where fights are had.
    Yard,
    /// The dock, south-west.
    Dock,
    /// The quarry, north-east.
    Quarry,
    /// The woodlot, north.
    Woodlot,
    /// The fields, east.
    Fields,
    /// The pens, south of the compound.
    Pens,
    /// The fenced briar patch on the wild north edge.
    Briar,
    /// The middle of the walled test quarter, west.
    TestQuarter,
}

impl Spot {
    pub const ALL: [Spot; 16] = [
        Spot::Shore,
        Spot::Wreck,
        Spot::Hut,
        Spot::Gate,
        Spot::Hall,
        Spot::Bunkhouses,
        Spot::Workyard,
        Spot::Stores,
        Spot::Yard,
        Spot::Dock,
        Spot::Quarry,
        Spot::Woodlot,
        Spot::Fields,
        Spot::Pens,
        Spot::Briar,
        Spot::TestQuarter,
    ];

    /// Metres from the island's centre.
    pub fn offset(self) -> V2 {
        match self {
            Spot::Shore => V2::new(255.0, 165.0),
            Spot::Wreck => V2::new(300.0, 195.0),
            Spot::Hut => V2::new(200.0, 130.0),
            Spot::Gate => V2::new(0.0, 60.0),
            Spot::Hall => V2::new(0.0, -40.0),
            Spot::Bunkhouses => V2::new(-55.0, 2.0),
            Spot::Workyard => V2::new(45.0, 0.0),
            Spot::Stores => V2::new(25.0, -45.0),
            Spot::Yard => V2::new(-50.0, 55.0),
            Spot::Dock => V2::new(-230.0, 180.0),
            Spot::Quarry => V2::new(180.0, -150.0),
            Spot::Woodlot => V2::new(40.0, -170.0),
            Spot::Fields => V2::new(170.0, 0.0),
            Spot::Pens => V2::new(80.0, 120.0),
            Spot::Briar => V2::new(-80.0, -200.0),
            Spot::TestQuarter => V2::new(-260.0, 0.0),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Spot::Shore => "the shore",
            Spot::Wreck => "the wreck",
            Spot::Hut => "the shelter hut",
            Spot::Gate => "the compound gate",
            Spot::Hall => "the overseers' hall",
            Spot::Bunkhouses => "the bunkhouses",
            Spot::Workyard => "the workyard",
            Spot::Stores => "the stores",
            Spot::Yard => "the yard",
            Spot::Dock => "the dock",
            Spot::Quarry => "the quarry",
            Spot::Woodlot => "the woodlot",
            Spot::Fields => "the fields",
            Spot::Pens => "the pens",
            Spot::Briar => "the briar patch",
            Spot::TestQuarter => "the test quarter",
        }
    }
}

/// Half the test quarter's side: it's this far each way from `Spot::TestQuarter`.
pub const TEST_QUARTER_HALF: f32 = 80.0;
/// The fenced briar patch on the wild north edge (a first look at the
/// Overgrowth, Laz's brief): the fence's radius and height.
pub const BRIAR_RADIUS: f32 = 14.0;
pub const BRIAR_FENCE: f32 = 1.1;
/// The shelter hut's footprint, and its building's index in the compound.
pub const HUT_SIZE: f32 = 7.5;
pub const HUT_BUILDING: usize = 6;
/// The hut's fire burns this long from the opening (then Diʻub's to feed).
pub const HUT_FIRE_HOURS: f64 = 72.0;
/// What lies by the fire when the player comes: (item key, how many).
pub const HUT_FOOD: [(&str, u16); 3] = [("raw_meat", 2), ("fish", 2), ("wild_berries", 3)];
/// How chilled a wrecked sailor wakes (`condition::CHILLED` is 30).
pub const WRECKED_COLD: f32 = 45.0;
/// The crate of what the sea left, at the water's edge below the wreck.
pub const WRECK_CRATE: super::containers::ContainerId = (super::containers::WILD, u16::MAX - 1, 0);
/// What's in it: (item key, how many).
pub const WRECK_GOODS: [(&str, u16); 5] = [("coin", 8), ("bedroll", 1), ("waterskin", 1), ("torch", 2), ("flatbread", 2)];
/// A traveller's kit for skipping the island (`tutorial::skip_island`):
/// worn, and in the pack.
pub const BASIC_WEAR: [&str; 6] = ["club", "hide_coat", "leather_cap", "hide_leggings", "boots", "small_pack"];
pub const BASIC_PACK: [(&str, u16); 7] = [("bedroll", 1), ("bandage", 3), ("waterskin", 1), ("flatbread", 3), ("dried_fish", 2), ("torch", 2), ("coin", 20)];

/// The island as the world holds it: which settlement is the compound.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Island {
    pub settlement: SettlementId,
    pub centre: V2,
    /// The lord's steward, who holds the debtors' bonds.
    pub steward: PersonId,
}

impl Island {
    pub fn spot(&self, s: Spot) -> V2 {
        self.centre.add(s.offset())
    }

    /// On the island's land.
    pub fn contains(&self, p: V2) -> bool {
        geo::isle_inland(p) > 0.0
    }

    /// The walled test quarter: its south-west-most and north-east-most corners
    /// (min x and y, max x and y).
    pub fn test_quarter(&self) -> (V2, V2) {
        let c = self.spot(Spot::TestQuarter);
        (c.add(V2::new(-TEST_QUARTER_HALF, -TEST_QUARTER_HALF)), c.add(V2::new(TEST_QUARTER_HALF, TEST_QUARTER_HALF)))
    }

    /// Inside the test quarter's walls.
    pub fn in_test_quarter(&self, p: V2) -> bool {
        let (a, b) = self.test_quarter();
        p.x >= a.x && p.x <= b.x && p.y >= a.y && p.y <= b.y
    }
}

/// The compound's buildings, at fixed places round the hearth: the hall, two
/// bunkhouses, the overseers' block and the steward's house. Returns each
/// resident's home (in `residents` order): debtors fill the bunkhouses, the
/// Binders the overseers' block, the steward their house.
fn build_compound(s: &mut Settlement, roles: &[Role], seed: u64) -> Vec<u16> {
    let mut rng = Rng::from_keys(&[seed, s.id as u64, 0x4255_494C]);
    let c = s.pos;
    let at = |o: V2| c.add(o);
    let facing = |p: V2| (c.y - p.y).atan2(c.x - p.x);
    let mut b: Vec<Building> = vec![Building { pos: c, kind: BuildingKind::Hearth, size: 6.0, rot: 0.0, seed: rng.next_u64() }];
    let plan: [(BuildingKind, V2, f32); 6] = [
        (BuildingKind::QotiroHall, Spot::Hall.offset(), 16.0),
        (BuildingKind::QotiroBlock, V2::new(-55.0, -16.0), 15.0),
        (BuildingKind::QotiroBlock, V2::new(-55.0, 20.0), 15.0),
        (BuildingKind::QotiroBlock, V2::new(50.0, -40.0), 14.0),
        (BuildingKind::RoduroHome, V2::new(55.0, 35.0), 11.0),
        // The shelter hut above the shore (nobody's home until Diʻub).
        (BuildingKind::RoduroHome, Spot::Hut.offset(), HUT_SIZE),
    ];
    for (kind, off, size) in plan {
        let pos = at(off);
        b.push(Building { pos, kind, size, rot: facing(pos), seed: rng.next_u64() });
    }
    s.reach = b.iter().map(|o| o.pos.dist(s.pos) + o.size).fold(s.radius(), f32::max);
    s.buildings = b;
    // Homes: 1 the hall (nobody sleeps there), 2 and 3 the bunkhouses, 4 the
    // overseers' block, 5 the steward's house.
    let mut debtor = 0usize;
    roles
        .iter()
        .map(|r| match r {
            Role::Debtor => {
                debtor += 1;
                2 + ((debtor - 1) % 2) as u16
            }
            Role::Guard | Role::Overseer => 4,
            Role::Steward => 5,
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Guard,
    Overseer,
    Steward,
    Debtor,
}

/// Add the island's compound to a world being made: the settlement and its
/// people, all extra, so nobody else's rolls move. Called after the squad
/// and the wanderers exist and before the roads are found.
pub(super) fn add_compound(settlements: &mut Vec<Settlement>, people: &mut Vec<Person>, seed: u64) -> Island {
    let id = settlements.len() as SettlementId;
    let mut s = Settlement {
        id,
        name: NAME.to_string(),
        pos: geo::isle_centre(),
        // The lord's island: named in Roduro. The Binders who run it are
        // placed by hand below, whatever the founders say.
        founders: Race::Roduro,
        coastal: false,
        stilts: None,
        size: 0.0,
        residents: Vec::new(),
        buildings: Vec::new(),
        reach: 0.0,
        styles: Vec::new(),
        sleepers: Vec::new(),
        holder: Some(super::factions::id("free_lords")),
        island: true,
    };
    let mut roles: Vec<(Role, Race)> = Vec::new();
    roles.push((Role::Steward, Race::Roduro));
    for _ in 0..OVERSEERS {
        roles.push((Role::Overseer, Race::Qotiro));
    }
    for _ in 0..GUARDS {
        roles.push((Role::Guard, Race::Qotiro));
    }
    for (ri, &n) in DEBTORS.iter().enumerate() {
        for _ in 0..n {
            roles.push((Role::Debtor, super::race::ALL_RACES[ri]));
        }
    }
    let mut steward = 0;
    for (k, &(role, race)) in roles.iter().enumerate() {
        let pid = people.len() as PersonId;
        let pseed = rng::key(&[seed, 0x4953_4C45, k as u64]);
        let mut p = Person::summary(pid, pseed, race, Some(id));
        p.home = Some(id);
        if role == Role::Steward {
            steward = pid;
        }
        s.residents.push(pid);
        people.push(p);
    }
    let kinds: Vec<Role> = roles.iter().map(|r| r.0).collect();
    let homes = build_compound(&mut s, &kinds, seed);
    for (&p, h) in s.residents.iter().zip(homes) {
        people[p as usize].dwelling = Some(h);
    }
    settlements.push(s);
    Island { settlement: id, centre: geo::isle_centre(), steward }
}

impl World {
    /// Salt Isle, if this world has it (a world made before it, or a test
    /// world without it, has none).
    pub fn island(&self) -> Option<&Island> {
        self.island.as_ref()
    }

    /// Is this settlement the island's compound?
    pub fn is_island(&self, t: SettlementId) -> bool {
        self.settlements.get(t as usize).is_some_and(|s| s.island)
    }

    /// The compound's workplaces, where the brief puts them (instead of
    /// `lay_out`'s spiral).
    pub(super) fn island_places(&self, t: SettlementId) -> Vec<Workplace> {
        let c = self.settlements[t as usize].pos;
        let mut r = Rng::from_keys(&[self.seed, t as u64, 0x504C_4143]);
        let plan: [(PlaceKind, V2); 10] = [
            (PlaceKind::GuardPost, Spot::Gate.offset()),
            (PlaceKind::Hall, V2::new(-22.0, -48.0)),
            (PlaceKind::Workyard, Spot::Workyard.offset()),
            (PlaceKind::Shop(super::jobs::Shelf::Materials), Spot::Stores.offset()),
            (PlaceKind::MessHall, V2::new(-15.0, 42.0)),
            (PlaceKind::Fields, Spot::Fields.offset()),
            (PlaceKind::Woodlot, Spot::Woodlot.offset()),
            (PlaceKind::Quarry, Spot::Quarry.offset()),
            (PlaceKind::Wilds, V2::new(-120.0, -190.0)),
            (PlaceKind::Dock, Spot::Dock.offset()),
        ];
        plan.iter()
            .map(|&(kind, off)| {
                let pos = c.add(off);
                Workplace { kind, pos, rot: (c.y - pos.y).atan2(c.x - pos.x), seed: r.next_u64() }
            })
            .collect()
    }

    /// What the island's ground gives: stone and salt pans, a little wood
    /// and clay; nothing the sea round it would have said.
    pub(super) fn island_sources(&self, src: &mut [f32]) {
        let mut set = |g: Good, x: f32| {
            if let Some(v) = src.get_mut(g.index()) {
                *v = x;
            }
        };
        set(Good::Rock, 0.8);
        set(Good::Timber, 0.35);
        set(Good::Clay, 0.3);
        set(Good::Ore, 0.0);
        set(Good::Gold, 0.0);
    }

    /// The island's customs, whoever lives there: slavery allowed (it's what
    /// the place is for).
    pub(super) fn island_customs(&self, ci: u32, customs: &mut super::culture::Customs) {
        let c = &self.society.communities[ci as usize];
        if self.is_island(c.town) {
            customs.slavery = Slavery::Allowed;
        }
    }

    /// The hut's door side: toward the compound.
    pub fn hut_fire(&self) -> Option<V2> {
        let isle = self.island()?;
        let hut = self.settlements[isle.settlement as usize].buildings.get(HUT_BUILDING)?;
        let to = isle.centre.sub(hut.pos);
        let len = to.len().max(1.0);
        Some(hut.pos.add(to.scale((hut.size * 0.5 + 5.0) / len)))
    }

    /// Where the wreck's crate lies: the last dry ground between the shore
    /// and the wreck.
    pub fn wreck_edge(&self) -> Option<V2> {
        let isle = self.island()?;
        let (a, z) = (isle.spot(Spot::Shore), isle.spot(Spot::Wreck));
        let d = z.sub(a);
        let len = d.len().max(1e-3);
        let dir = d.scale(1.0 / len);
        let mut k = 0.0;
        while k < len && !self.terrain.is_sea(a.add(dir.scale(k + 1.0))) {
            k += 1.0;
        }
        Some(a.add(dir.scale((k - 2.5).max(0.0))))
    }

    /// The opening (`tutorial.rs`): the squad wakes on the shore, chilled;
    /// the hut's fire is lit and there's raw food beside it; what the sea
    /// left of their things lies in a crate at the water's edge.
    pub(super) fn start_opening(&mut self) {
        let Some(isle) = self.island.clone() else { return };
        let t = self.time;
        if let Some(edge) = self.wreck_edge() {
            use super::containers::{Container, Owner};
            use super::inventory::Entry;
            let items: Vec<Entry> = WRECK_GOODS.iter().map(|&(k, n)| Entry(super::items::id(k), n, None)).collect();
            let rot = (isle.centre.y - edge.y).atan2(isle.centre.x - edge.x);
            self.containers.insert(WRECK_CRATE, Container { id: WRECK_CRATE, what: super::layout::Holder::Crate, pos: edge, rot, items, lock: 0.0, picked: false, owner: Owner::Nobody, taken: 0, ours: Vec::new() });
        }
        let Some(fire) = self.hut_fire() else { return };
        self.campfires.push(super::campfire::Campfire { id: self.next_fire, pos: fire, by: isle.steward, since: t - super::world::HOUR, lit_at: t - super::world::HOUR, out_at: t + HUT_FIRE_HOURS * super::world::HOUR });
        self.next_fire += 1;
        for (k, &(key, n)) in HUT_FOOD.iter().enumerate() {
            let a = 1.2 + k as f32 * 0.9;
            let at = fire.add(V2::new(a.cos(), a.sin()).scale(2.2));
            self.put_on_ground(super::items::id(key), n, at);
        }
        for &m in &self.squad.members.clone() {
            if let Some(c) = self.people[m as usize].cond.as_mut() {
                c.cold = WRECKED_COLD;
            }
        }
        self.tutorial = Some(super::tutorial::Tutorial::new(t, self.squad.pos));
        self.solid_changed();
    }

    /// After the society is founded: the Binders are the compound's guards
    /// and the debtors are in slave bonds to the steward, for life.
    pub(super) fn bind_debtors(&mut self) {
        let Some(isle) = self.island.clone() else { return };
        let t = self.time;
        let town = isle.settlement;
        let residents = self.settlements[town as usize].residents.clone();
        for &p in &residents {
            // (Nor the test quarter's culture row: second's `testquarter.rs`.)
            if p == isle.steward || self.in_culture_row(p) {
                continue;
            }
            let k = residents.iter().position(|&q| q == p).unwrap_or(0);
            // The first after the steward are the Binders (`add_compound`'s order).
            if k <= OVERSEERS + GUARDS {
                self.society.lives[p as usize].job = Job::Guard;
                continue;
            }
            self.bond(p, town, Some(isle.steward), t, f64::INFINITY);
            if let Some(b) = self.bonds.iter_mut().find(|b| b.who == p) {
                b.slave = true;
            }
        }
    }
}
