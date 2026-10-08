//! Society: households, jobs and workplaces, built from each community's
//! customs (`culture.rs`), and kept up as people die, leave and settle.
//!
//! A town is one or two **communities**: the people on land, and (on the
//! coast) the stilt village just offshore. Each has its own blend and its own
//! customs. Everything here reads those customs and people's own rolled
//! habits — never their race.
//!
//! What changes over time is changed at **dawn** (`DAWN`), on the world's one
//! timeline: a community whose people have shifted enough is re-blended (and
//! may take up new customs); posts left empty by the dead are filled from the
//! labourers; gardens left untended get a new gardener. Day plans
//! (`routine.rs`) and the economy (`economy.rs`) are looked up from all this
//! and the clock.

use serde::{Deserialize, Serialize};

use super::culture::{self, Belonging, Blend, Cooking, Customs, Habits, Layout, BLEND_SHIFT};
use super::geo::{self, V2};
use super::jobs::{Job, PlaceKind, Shelf, MEALS_PER_COOK_HOUR, N_GOODS, POTS_PER_RUN};
use super::person::PersonId;
use super::race::Race;
use super::rng::Rng;
use super::settlement::{BuildingKind, SettlementId};
use super::world::{World, DAY, HOUR};

/// The hour of the day when the dawn boats land and the day's food, taxes and
/// changes are settled.
pub const DAWN: i64 = 6;
/// A town where this many roads meet counts as on the roads (and keeps an inn).
pub const ROADS_MEET: usize = 5;
/// People per crafter of a craft their community fully leans toward.
pub const CRAFT_PER: f32 = 25.0;
/// Days in the week (rest days come round once a week).
pub const WEEK: i64 = 6;
/// Market day comes round every this many days.
pub const MARKET_EVERY: i64 = 4;
/// Hours a worker can be counted on for, on average over a week (rest days
/// and meals taken out), by the rhythm they keep. Used to size posts.
pub fn expected_hours(r: culture::Rhythm) -> f32 {
    match r {
        culture::Rhythm::Seasonal => 7.5,
        culture::Rhythm::Bells => 6.25,
        culture::Rhythm::Tides => 5.8,
        culture::Rhythm::Irregular => 4.5,
    }
}

/// Share of a community's food each path is meant to bring, by way of
/// cooking: gardens, kitchens (cooked from the town's stock), dawn boats.
pub const PATHS: [[f32; 3]; 3] = [
    // Each household cooks its own: gardens first, then home pots from the market.
    [0.35, 0.45, 0.20],
    // Hearth kitchen: cooks feed the workers.
    [0.20, 0.60, 0.20],
    // Deck: the catch, cooked together.
    [0.15, 0.35, 0.50],
];
pub const PATH_NAMES: [&str; 3] = ["Gardens", "Kitchens", "Dawn boats"];

/// Someone's place in their town.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Life {
    pub job: Job,
    /// Which of the town's workplaces.
    pub place: Option<u16>,
    pub household: Option<u32>,
    pub community: Option<u32>,
    /// Years; there's no ageing, this only settles who's eldest.
    pub age: u8,
    pub habits: Habits,
    /// Which day of the week they rest.
    pub rest_day: u8,
    /// 0 the day shift; 1 the night watch for guards, the second bell under bells.
    pub shift: u8,
}

impl Life {
    pub fn new(race: Race, seed: u64) -> Life {
        let mut r = Rng::from_keys(&[seed, 0x4C49_4645]);
        Life {
            job: Job::None,
            place: None,
            household: None,
            community: None,
            age: (18.0 + r.f32().powf(1.3) * 62.0) as u8,
            habits: Habits::roll(race, seed),
            rest_day: r.below(WEEK as usize) as u8,
            shift: 0,
        }
    }
}

/// What a community gets to eat, as worked out each dawn for the day before.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Food {
    /// Person-days needed.
    pub need: f32,
    /// Share each path is meant to bring: gardens, kitchens, boats.
    pub share: [f32; 3],
    /// What each path brought, person-days.
    pub supply: [f32; 3],
    /// How much of its share each path met, 0..1.
    pub suff: [f32; 3],
    /// All told, 0..1.
    pub overall: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Community {
    pub town: SettlementId,
    pub stilts: bool,
    /// Head-counts the blend was made from.
    pub counted: [u32; 4],
    pub blend: Blend,
    pub customs: Customs,
    pub food: Food,
    /// Share of people working out of town (they need the runners).
    pub away: f32,
    /// The stilt village has stopped bringing the catch ashore. (Nothing sets
    /// this yet: Part 3 gives the reasons.)
    pub withdrawn: bool,
    // What its kitchens cooked, its runners carried and (a stilt village) its
    // boats caught since dawn; each is a running total plus this hour's rate.
    pub cooked: Flow,
    pub carried: Flow,
    pub catch: Flow,
}

/// A running total that grows at a steady rate through the hour in progress.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Flow {
    pub base: f32,
    /// Per hour, for the hour that began at `Society::rates_from`.
    pub rate: f32,
}

impl Flow {
    pub fn at(&self, from: f64, t: f64) -> f32 {
        self.base + self.rate * ((t - from) / HOUR).clamp(0.0, 1.0) as f32
    }
    pub(super) fn settle(&mut self) {
        self.base += self.rate;
        self.rate = 0.0;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Workplace {
    pub kind: PlaceKind,
    pub pos: V2,
    pub rot: f32,
    pub seed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Household {
    pub community: u32,
    pub members: Vec<PersonId>,
    /// The building they share, if one.
    pub home: Option<u16>,
}

/// A home's garden and who tends it.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Garden {
    pub building: u16,
    pub gardener: Option<PersonId>,
}

/// A town's work and wealth.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TownLife {
    pub shore: u32,
    pub stilts: Option<u32>,
    pub places: Vec<Workplace>,
    pub gardens: Vec<Garden>,
    /// Several roads meet here.
    pub on_road: bool,
    /// How many roads end here.
    pub roads: u8,
    /// Which day of the market cycle is market day.
    pub market_day: u8,
    /// Goods in store, as a running total plus this hour's rate.
    pub stock: [Flow; N_GOODS],
    pub prosperity: f32,
    /// The merchants' coin: as of `purse_at`, refilling at `purse_rate` an hour up to `purse_cap`.
    pub purse: f32,
    pub purse_at: f64,
    pub purse_rate: f32,
    pub purse_cap: f32,
    pub treasury: f32,
    /// Guards' pay the treasury couldn't meet.
    pub owed: f32,
    /// The catch the dawn boats brought ashore last.
    pub landed: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Society {
    pub lives: Vec<Life>,
    pub communities: Vec<Community>,
    pub towns: Vec<TownLife>,
    pub households: Vec<Household>,
    /// When the hour now being worked began (the rates in every `Flow` are for it).
    pub rates_from: f64,
}

/// A strong minority's share, and the cooking its institution brings, for
/// each institution that cooks.
fn cooking_mix(c: &Community) -> Vec<(Cooking, f32)> {
    let mut parts = Vec::new();
    let mut rest = 1.0;
    for (r, inst) in c.customs.institutions() {
        if let Some(k) = inst.cooking() {
            let m = c.blend.share[r.index()];
            parts.push((k, m));
            rest -= m;
        }
    }
    parts.push((c.customs.cooking, rest.max(0.0)));
    parts
}

/// What a community's food paths are meant to bring, and how the kitchens'
/// share splits: (shares, fraction of the kitchen path cooked at home, fraction
/// carried by runners).
pub fn food_plan(c: &Community, has_gardens: bool, has_boats: bool) -> ([f32; 3], f32, f32) {
    let mut share = [0.0f32; 3];
    let mut kitchen_by = [0.0f32; 3];
    for (k, w) in cooking_mix(c) {
        let p = PATHS[k as usize];
        for i in 0..3 {
            share[i] += w * p[i];
        }
        kitchen_by[k as usize] += w * p[1];
    }
    if !has_gardens {
        share[0] = 0.0;
    }
    if !has_boats {
        share[2] = 0.0;
    }
    let total: f32 = share.iter().sum::<f32>().max(1e-6);
    let share = share.map(|s| s / total);
    let k: f32 = kitchen_by.iter().sum::<f32>().max(1e-6);
    (share, kitchen_by[0] / k, kitchen_by[1] / k)
}

impl World {
    /// The community someone belongs to.
    pub fn community_of(&self, pid: PersonId) -> Option<&Community> {
        self.society.lives.get(pid as usize)?.community.map(|c| &self.society.communities[c as usize])
    }

    pub fn life(&self, pid: PersonId) -> &Life {
        &self.society.lives[pid as usize]
    }

    /// A workplace someone goes to.
    pub fn workplace_of(&self, pid: PersonId) -> Option<&Workplace> {
        let l = self.society.lives.get(pid as usize)?;
        let town = self.people[pid as usize].home?;
        l.place.map(|i| &self.society.towns[town as usize].places[i as usize])
    }

    /// Living residents of a community.
    pub fn members_of(&self, ci: u32) -> impl Iterator<Item = PersonId> + '_ {
        let c = &self.society.communities[ci as usize];
        self.settlements[c.town as usize]
            .residents
            .iter()
            .copied()
            .filter(move |&p| self.society.lives[p as usize].community == Some(ci) && !self.people[p as usize].dead)
    }

    fn counts_of(&self, ci: u32) -> [u32; 4] {
        let mut n = [0u32; 4];
        for p in self.members_of(ci) {
            n[self.people[p as usize].race.index()] += 1;
        }
        n
    }

    fn lives_offshore(&self, pid: PersonId) -> bool {
        let p = &self.people[pid as usize];
        match (p.home, p.dwelling) {
            (Some(h), Some(d)) => self.settlements[h as usize].buildings.get(d as usize).map(|b| b.kind == BuildingKind::HoraroStilt).unwrap_or(false),
            _ => false,
        }
    }

    // ---- Founding -------------------------------------------------------------

    /// Give every town its communities, customs, households, workplaces and
    /// jobs. Called once, as the world is made.
    pub(super) fn found_society(&mut self) {
        let mut s = Society { lives: self.people.iter().map(|p| Life::new(p.race, p.seed)).collect(), rates_from: self.time, ..Default::default() };
        // Wanderers belong nowhere; they drift.
        for (i, p) in self.people.iter().enumerate() {
            if p.home.is_none() && !p.bandit && !p.in_squad {
                s.lives[i].job = Job::Drifter;
            }
        }
        for t in 0..self.settlements.len() {
            let town = &self.settlements[t];
            let mut r = Rng::from_keys(&[self.seed, t as u64, 0x544F_574C]);
            let roads = self.routes.roads.iter().filter(|rd| rd.first().into_iter().chain(rd.last()).any(|e| e.dist(town.pos) < town.reach + 80.0)).count();
            let on_road = roads >= ROADS_MEET;
            let shore = s.communities.len() as u32;
            let has_stilts = town.residents.iter().any(|&p| self.lives_offshore(p));
            let stilts = has_stilts.then_some(shore + 1);
            for (k, offshore) in [(shore, false), (shore + 1, true)] {
                if offshore && !has_stilts {
                    continue;
                }
                for &p in &town.residents {
                    if self.lives_offshore(p) == offshore {
                        s.lives[p as usize].community = Some(k);
                    }
                }
                s.communities.push(Community {
                    town: t as SettlementId,
                    stilts: offshore,
                    counted: [0; 4],
                    blend: Blend::default(),
                    customs: Customs { cooking: Cooking::Household, rhythm: culture::Rhythm::Seasonal, belonging: Belonging::Lineage, layout: Layout::Combined, own: [false; 4] },
                    food: Food::default(),
                    away: 0.0,
                    withdrawn: false,
                    cooked: Flow::default(),
                    carried: Flow::default(),
                    catch: Flow::default(),
                });
            }
            s.towns.push(TownLife {
                shore,
                stilts,
                places: Vec::new(),
                gardens: Vec::new(),
                on_road,
                roads: roads as u8,
                market_day: r.below(MARKET_EVERY as usize) as u8,
                stock: [Flow::default(); N_GOODS],
                prosperity: 0.6,
                purse: 0.0,
                purse_at: self.time,
                purse_rate: 0.0,
                purse_cap: 0.0,
                treasury: 0.0,
                owed: 0.0,
                landed: 0.0,
            });
        }
        self.society = s;
        for ci in 0..self.society.communities.len() as u32 {
            self.reblend(ci);
        }
        for t in 0..self.settlements.len() {
            self.lay_out(t as SettlementId);
        }
        self.form_all_households();
        for ci in 0..self.society.communities.len() as u32 {
            self.assign_jobs(ci, None);
        }
        for t in 0..self.settlements.len() {
            self.choose_gardeners(t as SettlementId);
            self.open_books(t as SettlementId);
        }
        self.place_stations();
        self.set_rates();
    }

    /// Work out a community's blend from who lives there now, and choose its
    /// customs from it. Returns whether any custom changed.
    pub(super) fn reblend(&mut self, ci: u32) -> bool {
        let counts = self.counts_of(ci);
        let c = &self.society.communities[ci as usize];
        let key = culture::community_key(c.town, c.stilts);
        let blend = Blend::of(counts, self.seed, key);
        let customs = blend.choose(self.seed, key);
        let c = &mut self.society.communities[ci as usize];
        let changed = c.customs != customs;
        c.counted = counts;
        c.blend = blend;
        c.customs = customs;
        changed
    }

    // ---- Households -----------------------------------------------------------

    /// Group every community into households by its custom (lodgers live
    /// with whoever's home they sleep in).
    pub(super) fn form_all_households(&mut self) {
        self.society.households.clear();
        for l in &mut self.society.lives {
            l.household = None;
        }
        for ci in 0..self.society.communities.len() as u32 {
            self.form_households(ci);
        }
    }

    fn form_households(&mut self, ci: u32) {
        let members: Vec<PersonId> = self.members_of(ci).collect();
        let belonging = self.society.communities[ci as usize].customs.belonging;
        // Homes in order, so a tier block is two neighbouring homes.
        let town = self.society.communities[ci as usize].town;
        let homes: Vec<u16> = self.settlements[town as usize].buildings.iter().enumerate().filter(|(_, b)| matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock | BuildingKind::HoraroStilt)).map(|(i, _)| i as u16).collect();
        let rank = |d: u16| homes.iter().position(|&h| h == d).unwrap_or(d as usize) as u64;
        let mut groups: Vec<(u64, Option<u16>, Vec<PersonId>)> = Vec::new();
        for &p in &members {
            let d = self.people[p as usize].dwelling;
            let key = match belonging {
                Belonging::Lineage => d.map(|d| d as u64).unwrap_or(u64::MAX - p as u64),
                Belonging::TierBlock => d.map(|d| rank(d) / 2).unwrap_or(u64::MAX - p as u64),
                Belonging::Village => 0,
                Belonging::Lodging => p as u64,
            };
            match groups.iter_mut().find(|g| g.0 == key) {
                Some(g) => g.2.push(p),
                None => groups.push((key, d, vec![p])),
            }
        }
        for (_, home, mem) in groups {
            let hi = self.society.households.len() as u32;
            for &p in &mem {
                self.society.lives[p as usize].household = Some(hi);
            }
            self.society.households.push(Household { community: ci, members: mem, home: if belonging == Belonging::Village { None } else { home } });
        }
    }

    // ---- Workplaces -----------------------------------------------------------

    /// How many living people each community of a town has, and whether a
    /// stilt village is there.
    fn sizes(&self, t: SettlementId) -> (usize, usize) {
        let tl = &self.society.towns[t as usize];
        let shore = self.members_of(tl.shore).count();
        let stilts = tl.stilts.map(|s| self.members_of(s).count()).unwrap_or(0);
        (shore, stilts)
    }

    /// The workplaces a town keeps, from its size and its customs.
    fn places_wanted(&self, t: SettlementId) -> Vec<PlaceKind> {
        use PlaceKind as P;
        let tl = &self.society.towns[t as usize];
        let shore = &self.society.communities[tl.shore as usize];
        let (n, v) = self.sizes(t);
        let combined = shore.customs.layout == Layout::Combined;
        let mut out = Vec::new();
        let farmers = self.posts_for(tl.shore, Job::Farmer, n);
        for _ in 0..farmers.div_ceil(4).max(1) {
            out.push(P::Fields);
        }
        out.push(P::Wilds);
        if n >= 40 {
            out.push(P::Woodlot);
        }
        if tl.stilts.is_some() {
            out.push(P::Dock);
        }
        match shore.customs.cooking {
            Cooking::Hearth => out.push(P::Kitchen),
            Cooking::Deck => out.push(P::Deck),
            Cooking::Household => {}
        }
        for (_, inst) in shore.customs.institutions() {
            let k = match inst {
                culture::Institution::TendersYard => P::TendersYard,
                culture::Institution::MessHall => P::MessHall,
                culture::Institution::Deck => P::Deck,
                culture::Institution::LettersHouse => P::LettersHouse,
            };
            if !out.contains(&k) {
                out.push(k);
            }
        }
        // Trade.
        if combined {
            out.push(P::Market);
        } else {
            out.push(P::Shop(Shelf::Food));
            out.push(P::Shop(Shelf::Goods));
            if n >= 60 {
                out.push(P::Shop(Shelf::Materials));
            }
        }
        if n >= 110 || (tl.on_road && n >= 50) {
            out.push(P::Inn);
        }
        out.extend([P::Shrine, P::GuardPost, P::Hall]);
        // Body, knowledge, money, crafts.
        let large = n >= 150;
        if combined {
            out.push(if n >= 80 { P::HealingHouse } else { P::AlchemyTable });
            if large && !out.contains(&P::LettersHouse) {
                out.push(P::LettersHouse);
            }
            out.push(P::Workyard);
        } else {
            if n >= 80 {
                out.push(P::HealersHouse);
            }
            out.push(P::AlchemyTable);
            if large {
                out.push(P::TeachingHouse);
                out.push(P::ExchangeHouse);
            }
            out.push(P::Forge);
            out.push(P::Bench);
            out.push(P::WeaversShed);
            out.push(P::Workshop);
            if !out.contains(&P::TendersYard) {
                out.push(P::TendersYard);
            }
        }
        if !out.iter().any(|k| k.stations().contains(&super::crafting::Station::Desk)) {
            out.push(P::Desk);
        }
        // The stilt village's own.
        if let Some(si) = tl.stilts {
            let st = &self.society.communities[si as usize];
            out.push(P::DivePlatform);
            out.push(P::KelpBeds);
            if v >= 10 {
                out.push(P::Boatyard);
            }
            if st.customs.cooking != Cooking::Household {
                // (Village kitchens sit on the deck whichever way they cook.)
                out.push(P::Deck);
            }
        }
        out
    }

    /// Lay out a town's workplaces (and move its crafting stations into them).
    pub(super) fn lay_out(&mut self, t: SettlementId) {
        let kinds = self.places_wanted(t);
        let town = &self.settlements[t as usize];
        let mut r = Rng::from_keys(&[self.seed, t as u64, 0x504C_4143]);
        let mut placed: Vec<Workplace> = Vec::new();
        let clear = |p: V2, size: f32, placed: &[Workplace]| {
            town.buildings.iter().all(|b| b.pos.dist(p) > (b.size + size) * 0.6 + 3.0) && placed.iter().all(|o| o.pos.dist(p) > (o.kind.size() + size) * 0.55 + 2.5)
        };
        // In-town places spiral out from the hearth, between the homes.
        let mut ring = 12.0f32;
        let mut ang = r.f32() * std::f32::consts::TAU;
        let stilts_at = town.stilts;
        let offshore_village = stilts_at.map(|st| {
            let homes: Vec<V2> = town.buildings.iter().filter(|b| b.kind == BuildingKind::HoraroStilt).map(|b| b.pos).collect();
            if homes.is_empty() {
                st
            } else {
                homes.iter().fold(V2::default(), |a, p| a.add(*p)).scale(1.0 / homes.len() as f32)
            }
        });
        let mut village_decks = 0;
        for kind in kinds {
            let size = kind.size();
            let seed = r.next_u64();
            let pos = match kind {
                PlaceKind::Fields | PlaceKind::Wilds | PlaceKind::Woodlot => {
                    let base = town.reach.max(town.radius()) + if kind == PlaceKind::Fields { 18.0 } else { 55.0 };
                    let mut at = town.pos;
                    for k in 0..300 {
                        let a = r.f32() * std::f32::consts::TAU;
                        let rad = base + size * 0.5 + (k / 30) as f32 * 12.0 + r.range(0.0, 20.0);
                        let c = town.pos.add(V2::new(a.cos(), a.sin()).scale(rad));
                        if geo::inland(c) > size + 10.0 && clear(c, size, &placed) {
                            at = c;
                            break;
                        }
                    }
                    at
                }
                PlaceKind::Dock => {
                    let y = stilts_at.map(|s| s.y).unwrap_or(town.pos.y) + r.range(-25.0, 25.0);
                    V2::new(geo::coast_x(y) + 4.0, y)
                }
                PlaceKind::DivePlatform | PlaceKind::KelpBeds | PlaceKind::Boatyard => {
                    let c = offshore_village.unwrap_or(town.pos);
                    let mut at = c;
                    for k in 0..200 {
                        let a = r.f32() * std::f32::consts::PI + std::f32::consts::FRAC_PI_2; // seaward half
                        let rad = 30.0 + k as f32 * 0.8 + r.range(0.0, 15.0);
                        let cand = c.add(V2::new(a.cos(), a.sin()).scale(rad));
                        let wet = -geo::inland(cand);
                        let ok = if kind == PlaceKind::Boatyard { wet > 6.0 && wet < 60.0 } else { wet > 40.0 && wet < 300.0 };
                        if ok && clear(cand, size, &placed) {
                            at = cand;
                            break;
                        }
                    }
                    at
                }
                PlaceKind::Deck if offshore_village.is_some() && village_decks == 0 && self.society.towns[t as usize].stilts.is_some() && placed.iter().any(|p| p.kind == PlaceKind::DivePlatform) => {
                    village_decks += 1;
                    offshore_village.unwrap()
                }
                _ => {
                    let mut at = town.pos;
                    for _ in 0..500 {
                        ang += 2.399_963 / (1.0 + ring / 50.0);
                        ring += size * 0.09;
                        let cand = town.pos.add(V2::new(ang.cos(), ang.sin()).scale(ring + size * 0.5 + r.range(-2.0, 2.0)));
                        if geo::inland(cand) > size && clear(cand, size, &placed) {
                            at = cand;
                            break;
                        }
                    }
                    at
                }
            };
            let rot = (town.pos.y - pos.y).atan2(town.pos.x - pos.x);
            placed.push(Workplace { kind, pos, rot, seed });
        }
        let reach = placed.iter().map(|p| p.pos.dist(town.pos) + p.kind.size()).fold(0.0, f32::max);
        let s = &mut self.settlements[t as usize];
        s.reach = s.reach.max(reach);
        self.society.towns[t as usize].places = placed;
    }

    /// Crafting stations stand in the workplaces that hold them.
    pub(super) fn place_stations(&mut self) {
        self.stations.clear();
        for tl in &self.society.towns {
            for p in &tl.places {
                let n = p.kind.stations().len();
                for (k, &st) in p.kind.stations().iter().enumerate() {
                    let off = if n > 1 { (k as f32 - (n - 1) as f32 * 0.5) * 4.0 } else { 0.0 };
                    let side = V2::new(-p.rot.sin(), p.rot.cos());
                    self.stations.push((p.pos.add(side.scale(off)), st));
                }
            }
        }
    }

    // ---- Jobs -----------------------------------------------------------------

    /// The workplaces in a town where a job is done, for a community.
    fn places_for(&self, ci: u32, job: Job) -> Vec<u16> {
        use PlaceKind as P;
        let c = &self.society.communities[ci as usize];
        let tl = &self.society.towns[c.town as usize];
        let fits = |k: PlaceKind| -> bool {
            match job {
                Job::Farmer => k == P::Fields,
                Job::Forager => k == P::Wilds,
                Job::Woodcutter => k == P::Woodlot,
                Job::Fisher => if c.stilts { k == P::DivePlatform } else { k == P::Dock },
                Job::KelpGatherer => c.stilts && k == P::KelpBeds,
                Job::Boatwright => c.stilts && k == P::Boatyard,
                Job::Cook => if c.stilts { k == P::Deck } else { matches!(k, P::Kitchen | P::MessHall) || (k == P::Deck) },
                Job::Runner => !c.stilts && matches!(k, P::Kitchen | P::MessHall),
                Job::Merchant | Job::Caravaner => matches!(k, P::Market | P::Shop(_)),
                Job::Innkeeper => k == P::Inn,
                Job::Guard => k == P::GuardPost,
                Job::Official | Job::Arbiter => k == P::Hall,
                Job::Priest => k == P::Shrine,
                Job::Healer => matches!(k, P::HealingHouse | P::HealersHouse),
                Job::Teacher => matches!(k, P::TeachingHouse | P::LettersHouse),
                Job::Exchanger => matches!(k, P::ExchangeHouse | P::LettersHouse),
                Job::Scribe => matches!(k, P::Desk | P::LettersHouse),
                Job::Weaver => if c.stilts { k == P::Boatyard } else { matches!(k, P::WeaversShed | P::Workyard) },
                Job::Tanner | Job::Leatherworker | Job::Tailor | Job::Woodworker => !c.stilts && matches!(k, P::Workshop | P::Workyard),
                Job::CharcoalBurner => !c.stilts && k == P::Woodlot,
                Job::Alchemist => matches!(k, P::AlchemyTable | P::HealingHouse),
                Job::Smith => matches!(k, P::Workyard | P::Forge),
                Job::Armourer => matches!(k, P::Workyard | P::Bench),
                Job::StoneTender => k == P::TendersYard,
                Job::Labourer => if c.stilts { k == P::DivePlatform } else { matches!(k, P::Market | P::Workyard | P::Shop(_) | P::Forge) },
                Job::Drifter | Job::None => false,
            }
        };
        // Village decks belong to the village; a shore deck to the shore.
        tl.places
            .iter()
            .enumerate()
            .filter(|(_, p)| fits(p.kind) && (p.kind != P::Deck || (geo::inland(p.pos) < 0.0) == c.stilts))
            .map(|(i, _)| i as u16)
            .collect()
    }

    /// How many of a job a community wants, given its size.
    fn posts_for(&self, ci: u32, job: Job, n: usize) -> usize {
        let c = &self.society.communities[ci as usize];
        let tl = &self.society.towns[c.town as usize];
        let nf = n as f32;
        let has_gardens = !c.stilts;
        let has_boats = c.stilts || tl.stilts.is_some();
        let (share, home_part, runner_part) = food_plan(c, has_gardens, has_boats);
        let kitchen_need = nf * share[1] * (1.0 - home_part);
        let day = expected_hours(c.customs.rhythm);
        let cook_day = MEALS_PER_COOK_HOUR * day;
        if n == 0 {
            return 0;
        }
        match job {
            Job::Farmer if !c.stilts => ((nf * share[1] + nf * 0.12) / (super::jobs::FARM_PER_HOUR * day)).ceil() as usize,
            Job::Fisher if c.stilts => {
                let shore_n = self.members_of(tl.shore).count() as f32;
                let shore_c = &self.society.communities[tl.shore as usize];
                let (ss, _, _) = food_plan(shore_c, true, true);
                ((nf * (share[1] + share[2]) + shore_n * ss[2]) * 1.25 / (super::jobs::FISH_PER_HOUR * day)).ceil() as usize
            }
            Job::Fisher => (nf * 0.02).ceil() as usize,
            Job::KelpGatherer if c.stilts => (nf * 0.1).ceil() as usize,
            Job::Boatwright if c.stilts && n >= 10 => 1,
            Job::Forager if !c.stilts => (nf * 0.03).ceil().max(1.0) as usize,
            Job::Woodcutter if !c.stilts && n >= 40 => (nf * 0.03).ceil() as usize,
            Job::Cook => (kitchen_need * 1.4 / cook_day).ceil() as usize,
            Job::Runner if !c.stilts => (kitchen_need * runner_part * c.away * 1.4 / POTS_PER_RUN).ceil() as usize,
            Job::Merchant if !c.stilts => 1 + n / 80,
            Job::Caravaner if !c.stilts && n >= 60 => (n / 90).max(1),
            Job::Innkeeper if !c.stilts => 1,
            Job::Guard if !c.stilts => (n / 28).max(2),
            Job::Official if !c.stilts => 1 + n / 220,
            Job::Arbiter if !c.stilts && n >= 90 && c.blend.split < 0.5 => 1,
            Job::Priest if !c.stilts => 1 + n / 250,
            Job::Healer if !c.stilts => 1 + n / 250,
            Job::Teacher | Job::Exchanger if !c.stilts => 1,
            Job::Alchemist if !c.stilts && n >= 30 => 1,
            // Crafts by what the community leans toward: a town with few
            // who know the forge may have no smith at all.
            Job::Weaver => self.craft_posts(c, Job::Weaver, nf),
            Job::Smith | Job::Armourer | Job::Scribe if !c.stilts => self.craft_posts(c, job, nf),
            Job::Tanner | Job::Woodworker if !c.stilts && n >= 40 => 1,
            Job::Leatherworker if !c.stilts => 1 + n / 150,
            Job::Tailor if !c.stilts && n >= 60 => 1,
            Job::CharcoalBurner if !c.stilts && n >= 60 && c.blend.crafts[super::materials::Craft::Smithing.index()] > 0.3 => 1,
            Job::StoneTender if !c.stilts => {
                let stone = self.settlements[c.town as usize].buildings.iter().filter(|b| matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::HoraroStilt)).count();
                stone.div_ceil(14)
            }
            _ => 0,
        }
    }

    fn craft_posts(&self, c: &Community, job: Job, n: f32) -> usize {
        let craft = job.craft().expect("a craft job");
        (n / CRAFT_PER * c.blend.crafts[craft.index()] + 0.5).floor() as usize
    }

    /// The order posts are filled in: what a town can least do without first.
    const POST_ORDER: [Job; 29] = [
        Job::Guard,
        Job::Cook,
        Job::Farmer,
        Job::Fisher,
        Job::Runner,
        Job::Healer,
        Job::Merchant,
        Job::Priest,
        Job::Official,
        Job::Innkeeper,
        Job::Smith,
        Job::Armourer,
        Job::Alchemist,
        Job::Scribe,
        Job::Weaver,
        Job::Leatherworker,
        Job::Tanner,
        Job::Woodworker,
        Job::Tailor,
        Job::CharcoalBurner,
        Job::Exchanger,
        Job::Teacher,
        Job::Arbiter,
        Job::StoneTender,
        Job::KelpGatherer,
        Job::Boatwright,
        Job::Forager,
        Job::Woodcutter,
        Job::Caravaner,
    ];

    /// Fill a community's open posts. At founding (`day` None) everyone is a
    /// candidate and whoever's left becomes a labourer or a drifter; later,
    /// posts are filled from the labourers and drifters. Returns who took
    /// which post.
    pub(super) fn assign_jobs(&mut self, ci: u32, day: Option<i64>) -> Vec<(PersonId, Job)> {
        let members: Vec<PersonId> = self.members_of(ci).collect();
        let n = members.len();
        let mut free: Vec<PersonId> = members.iter().copied().filter(|&p| if day.is_none() { true } else { matches!(self.society.lives[p as usize].job, Job::Labourer | Job::Drifter | Job::None) }).collect();
        // Away share is worked out once jobs exist; at founding, estimate it.
        let mut taken = Vec::new();
        for job in Self::POST_ORDER {
            let places = self.places_for(ci, job);
            // Stone Tenders make rounds of the homes; a yard is optional.
            if places.is_empty() && job != Job::StoneTender {
                continue;
            }
            if job == Job::Runner {
                // Who works out of town so far (runners carry to them).
                let tl = &self.society.towns[self.society.communities[ci as usize].town as usize];
                let away = members.iter().filter(|&&p| self.society.lives[p as usize].place.map(|i| tl.places[i as usize].kind.is_away()).unwrap_or(false)).count();
                self.society.communities[ci as usize].away = away as f32 / n.max(1) as f32;
            }
            let want = self.posts_for(ci, job, n);
            let have = members.iter().filter(|&&p| self.society.lives[p as usize].job == job && day.is_some()).count();
            let mut r = Rng::from_keys(&[self.seed, ci as u64, job as u64, day.unwrap_or(-1) as u64, 0x4A4F_4253]);
            for k in have..want {
                let w: Vec<f32> = free
                    .iter()
                    .map(|&p| {
                        let pp = &self.people[p as usize];
                        let lean = culture::profile(pp.race).jobs.iter().find(|j| j.0 == job).map(|j| j.1).unwrap_or(1.0);
                        job.fit(pp.stats.calling, &pp.traits) * lean
                    })
                    .collect();
                let Some(i) = r.weighted(&w) else { break };
                let p = free.swap_remove(i);
                let place = places.get(k % places.len().max(1)).copied();
                let l = &mut self.society.lives[p as usize];
                l.job = job;
                l.place = place;
                l.shift = match job {
                    Job::Guard => (k % 3 == 2) as u8,
                    Job::Fisher if k == 0 => 1, // the first fisher crews the dawn boat
                    _ => r.chance(0.25) as u8,
                };
                taken.push((p, job));
            }
        }
        if day.is_none() {
            let labour = self.places_for(ci, Job::Labourer);
            for (k, &p) in free.iter().enumerate() {
                let restless = self.people[p as usize].traits.wanderlust > 0.62;
                let l = &mut self.society.lives[p as usize];
                if restless || labour.is_empty() {
                    l.job = Job::Drifter;
                    l.place = None;
                } else {
                    l.job = Job::Labourer;
                    l.place = Some(labour[k % labour.len()]);
                }
            }
        }
        // A stilt village always has someone to crew the dawn boat.
        if self.society.communities[ci as usize].stilts {
            let fishers: Vec<PersonId> = members.iter().copied().filter(|&p| self.society.lives[p as usize].job == Job::Fisher).collect();
            if !fishers.iter().any(|&p| self.society.lives[p as usize].shift == 1) {
                if let Some(&p) = fishers.first() {
                    self.society.lives[p as usize].shift = 1;
                }
            }
        }
        // Who works out of town.
        let tl = &self.society.towns[self.society.communities[ci as usize].town as usize];
        let away = members.iter().filter(|&&p| self.society.lives[p as usize].place.map(|i| tl.places[i as usize].kind.is_away()).unwrap_or(false)).count();
        self.society.communities[ci as usize].away = away as f32 / n.max(1) as f32;
        taken
    }

    /// When a town is laid out anew, everyone's workplace is found again for
    /// the job they already have.
    fn rehouse_jobs(&mut self, t: SettlementId) {
        let tl = &self.society.towns[t as usize];
        let comms: Vec<u32> = std::iter::once(tl.shore).chain(tl.stilts).collect();
        for ci in comms {
            let members: Vec<PersonId> = self.members_of(ci).collect();
            let mut count: std::collections::BTreeMap<Job, usize> = Default::default();
            let mut demote = Vec::new();
            for p in members {
                let job = self.society.lives[p as usize].job;
                let places = self.places_for(ci, job);
                let k = count.entry(job).or_insert(0);
                if places.is_empty() {
                    if job.is_post() && job != Job::StoneTender {
                        demote.push(p);
                    }
                    self.society.lives[p as usize].place = None;
                } else {
                    self.society.lives[p as usize].place = Some(places[*k % places.len()]);
                }
                *k += 1;
            }
            // A post whose place is gone: back to labouring.
            let labour = self.places_for(ci, Job::Labourer);
            for (k, p) in demote.into_iter().enumerate() {
                let l = &mut self.society.lives[p as usize];
                l.job = Job::Labourer;
                l.place = labour.get(k % labour.len().max(1)).copied();
            }
        }
    }

    // ---- Gardens --------------------------------------------------------------

    /// Every home on land has a garden; the household member with the
    /// lightest work tends it — a lodger before anyone, then the eldest.
    pub(super) fn choose_gardeners(&mut self, t: SettlementId) {
        let town = &self.settlements[t as usize];
        let mut gardens = Vec::new();
        for (bi, b) in town.buildings.iter().enumerate() {
            if !matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock) {
                continue;
            }
            let best = town
                .residents
                .iter()
                .copied()
                .filter(|&p| self.people[p as usize].dwelling == Some(bi as u16) && !self.people[p as usize].dead)
                .max_by(|&a, &b| {
                    let score = |p: PersonId| {
                        let l = &self.society.lives[p as usize];
                        l.job.lightness() + if l.habits.lodger { 0.6 } else { 0.0 } + l.age as f32 / 200.0
                    };
                    score(a).total_cmp(&score(b)).then(b.cmp(&a))
                });
            gardens.push(Garden { building: bi as u16, gardener: best });
        }
        self.society.towns[t as usize].gardens = gardens;
    }

    // ---- At dawn: drift, vacancies, gardens -----------------------------------

    /// A community whose people have shifted enough re-blends, and may take up
    /// new customs; empty posts are filled; untended gardens get a gardener.
    pub(super) fn dawn_changes(&mut self, t: SettlementId, day: i64) {
        let tl = &self.society.towns[t as usize];
        let comms: Vec<u32> = std::iter::once(tl.shore).chain(tl.stilts).collect();
        let mut relay = false;
        let mut regroup = false;
        for &ci in &comms {
            let now = self.counts_of(ci);
            let c = &self.society.communities[ci as usize];
            if culture::shift(c.counted, now) >= BLEND_SHIFT {
                let before = c.customs;
                if self.reblend(ci) {
                    let after = self.society.communities[ci as usize].customs;
                    let name = self.settlements[t as usize].name.clone();
                    let place = if self.society.communities[ci as usize].stilts { format!("The stilt village off {name}") } else { name };
                    let what = if after.cooking != before.cooking {
                        format!("now cooks: {}", after.cooking.name().to_lowercase())
                    } else if after.rhythm != before.rhythm {
                        format!("keeps {} now", after.rhythm.name().to_lowercase())
                    } else if after.belonging != before.belonging {
                        format!("lives by {} now", after.belonging.name().to_lowercase())
                    } else if after.layout != before.layout {
                        format!("goes over to {}", after.layout.name().to_lowercase())
                    } else {
                        "takes up new ways".to_string()
                    };
                    self.log.push_front((day as f64 * DAY + DAWN as f64 * HOUR, format!("{place} {what}.")));
                    self.log.truncate(14);
                    if after.belonging != before.belonging {
                        regroup = true;
                    }
                    if after.layout != before.layout || after.cooking != before.cooking || after.own != before.own {
                        relay = true;
                    }
                }
            }
        }
        if regroup {
            self.form_all_households();
        }
        if relay {
            self.lay_out(t);
            self.rehouse_jobs(t);
            self.place_stations();
        }
        for &ci in &comms {
            // The dead hand on nothing: their posts are open.
            let dead: Vec<PersonId> = self.settlements[t as usize]
                .residents
                .iter()
                .copied()
                .filter(|&p| self.people[p as usize].dead && self.society.lives[p as usize].community == Some(ci) && self.society.lives[p as usize].job != Job::None)
                .collect();
            for p in dead {
                self.society.lives[p as usize].job = Job::None;
                self.society.lives[p as usize].place = None;
            }
            for (p, job) in self.assign_jobs(ci, Some(day)) {
                if self.bands.band_at(self.settlements[t as usize].pos) <= 2 {
                    let who = self.name_of(p);
                    self.log.push_front((day as f64 * DAY + DAWN as f64 * HOUR, format!("{who} takes up work as {} in {}.", job.name().to_lowercase(), self.settlements[t as usize].name)));
                    self.log.truncate(14);
                }
            }
        }
        if self.society.towns[t as usize].gardens.iter().any(|g| g.gardener.map(|p| self.people[p as usize].dead || self.people[p as usize].home != Some(t)).unwrap_or(true)) {
            self.choose_gardeners(t);
        }
    }

    // ---- Moving house ---------------------------------------------------------

    /// Someone settles in another town: they join its land community, take a
    /// home there and, until a post opens, labour. (Their old town and new
    /// one re-blend at the next dawn if the move tipped them far enough.)
    pub fn resettle(&mut self, pid: PersonId, to: SettlementId) {
        if let Some(from) = self.people[pid as usize].home {
            self.settlements[from as usize].residents.retain(|&p| p != pid);
        }
        let town = &self.settlements[to as usize];
        // The least crowded home on land.
        let homes: Vec<u16> = town.buildings.iter().enumerate().filter(|(_, b)| matches!(b.kind, BuildingKind::RoduroHome | BuildingKind::QotiroBlock)).map(|(i, _)| i as u16).collect();
        let dwelling = homes.iter().copied().min_by_key(|&h| (town.residents.iter().filter(|&&p| self.people[p as usize].dwelling == Some(h)).count(), h));
        self.settlements[to as usize].residents.push(pid);
        let p = &mut self.people[pid as usize];
        p.home = Some(to);
        p.dwelling = dwelling;
        let shore = self.society.towns[to as usize].shore;
        let labour = self.places_for(shore, Job::Labourer);
        let l = &mut self.society.lives[pid as usize];
        l.community = Some(shore);
        l.job = Job::Labourer;
        l.place = labour.first().copied();
        if let Some(old) = l.household.take() {
            self.society.households[old as usize].members.retain(|&m| m != pid);
        }
        // Lodge with the household of the home they've moved into (or the
        // one household, where everyone is one).
        let one = self.society.communities[shore as usize].customs.belonging == Belonging::Village;
        if let Some(h) = self.society.households.iter().position(|h| h.community == shore && ((dwelling.is_some() && h.home == dwelling) || one)) {
            self.society.households[h].members.push(pid);
            self.society.lives[pid as usize].household = Some(h as u32);
        } else {
            let hi = self.society.households.len() as u32;
            self.society.households.push(Household { community: shore, members: vec![pid], home: dwelling });
            self.society.lives[pid as usize].household = Some(hi);
        }
    }

    /// Hook for Part 3: the stilt village off a town stops (or starts again)
    /// bringing its catch ashore at dawn.
    pub fn set_withdrawn(&mut self, t: SettlementId, withdrawn: bool) {
        if let Some(si) = self.society.towns[t as usize].stilts {
            self.society.communities[si as usize].withdrawn = withdrawn;
        }
    }

    /// The day number of a time (day 0 begins at midnight).
    pub fn day_of(t: f64) -> i64 {
        (t / DAY).floor() as i64
    }
}

