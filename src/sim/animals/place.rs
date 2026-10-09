//! Stocking a new world with animals: wild herds where the country suits
//! them, hunters' haunts, silk colonies, the Cragmaws, livestock round the
//! towns, and fish along the coast. Everything is drawn from the world seed
//! with its own keys, so adding animals never shifts anything else the seed
//! decides.

use super::super::geo::{self, V2};
use super::super::rng::{self, Rng};
use super::super::terrain::seg_dist;
use super::super::world::{World, DAY};
use super::herd::Herd;
use super::hooks::Colony;
use super::region::{region_of, survey, Region, ACROSS};
use super::species::{Class, Habitat, Sp, Toward, ALL_SPECIES};
use super::{Animals, FIRST_BATTLE};

/// How far wild herds keep from the edge of a town, metres: grazers at the
/// field edges, most things further, anything dangerous further still.
const CLEAR_GRAZERS: f32 = 160.0;
const CLEAR_WILD: f32 = 380.0;
const CLEAR_DANGER: f32 = 650.0;
/// How many Cragmaws the massif holds, and how far apart they keep.
const CRAGMAWS: usize = 2;
const CRAGMAW_APART: f32 = 1400.0;
/// About how many silk colonies the hill cliffs hold.
const COLONIES: usize = 10;
const COLONY_APART: f32 = 1800.0;
/// Share of ambushers that make their lair right beside a road, where one
/// passes close enough.
const ROADSIDE_LAIRS: f32 = 0.4;
/// How far from a road such a lair lies, metres.
const LAIR_FROM_ROAD: f32 = 9.0;

impl World {
    /// A new wild herd. Returns its id.
    pub(super) fn add_herd(&mut self, sp: Sp, home: V2, cap: u8, n: f32, seed: u64) -> u32 {
        let id = self.animals.herds.len() as u32;
        let region = region_of(home);
        let cap = cap.max(1);
        self.animals.herds.push(Herd {
            id,
            sp,
            region,
            seed,
            home,
            haunt: None,
            cap,
            n: n.min(cap as f32 + 0.4),
            at: self.time,
            rate: sp.def().growth as f64 / DAY,
            limit: cap as f32 + 0.5,
            restock_at: f64::INFINITY,
            idents: Vec::new(),
            next_ident: 0,
            hurt: Vec::new(),
            grown: (n.min(cap as f32 + 0.4) + 1e-3).floor() as u8,
            born: Vec::new(),
            away: None,
            hunt: None,
            ready_at: self.time,
            met: false,
            colony: None,
        });
        self.animals.regions[region as usize].herds.push(id);
        id
    }

    /// The nearest point of any road to `p`, if one runs within `within`.
    fn road_near(&self, p: V2, within: f32) -> Option<V2> {
        let mut best: Option<(f32, V2)> = None;
        for road in &self.routes.roads {
            for w in road.windows(2) {
                let d = seg_dist(w[0], w[1], p);
                if d <= within && best.map(|b| d < b.0).unwrap_or(true) {
                    let ab = w[1].sub(w[0]);
                    let l2 = (ab.x * ab.x + ab.y * ab.y).max(1e-6);
                    let u = (((p.x - w[0].x) * ab.x + (p.y - w[0].y) * ab.y) / l2).clamp(0.0, 1.0);
                    best = Some((d, w[0].lerp(w[1], u)));
                }
            }
        }
        best.map(|b| b.1)
    }

    fn clear_of_towns(&self, p: V2, margin: f32) -> bool {
        self.settlements.iter().all(|s| s.pos.dist(p) > s.radius() + margin)
    }

    /// Put animals into a freshly made world. Called once from `worldgen`.
    pub(in crate::sim) fn place_animals(&mut self) {
        let seed = self.seed;
        self.animals = Animals { hour_done: self.hour_done - 1, next_battle: FIRST_BATTLE, near_at: V2::new(-1e9, -1e9), ..Animals::default() };

        // ---- The country, region by region -----------------------------------
        let n = ACROSS * ACROSS;
        let mut surveys = Vec::with_capacity(n);
        for r in 0..n {
            let (points, coast) = survey(seed, &self.terrain, r as u16);
            self.animals.regions.push(Region::from_survey(&points, coast));
            surveys.push(points);
        }

        // ---- Wild herds, wherever the country suits them ---------------------
        for sp in ALL_SPECIES {
            let d = sp.def();
            if d.density <= 0.0 || d.habitats.is_empty() || matches!(d.class, Class::Fish | Class::Sea | Class::Domestic) {
                continue;
            }
            let (lo, hi) = d.group;
            let mean = (lo as f32 + hi as f32) * 0.5;
            let mask: u16 = d.habitats.iter().map(|h| 1u16 << (*h as u16)).sum();
            let margin = if d.toward.dangerous() {
                CLEAR_DANGER
            } else if sp == Sp::WildTuriyu {
                CLEAR_GRAZERS
            } else {
                CLEAR_WILD
            };
            for r in 0..n {
                let room = self.animals.regions[r].room_for(sp);
                if room <= 0.0 {
                    continue;
                }
                let mut rr = Rng::from_keys(&[seed, sp as u64, r as u64, 0x4845_5244]);
                let want = room * d.density / mean;
                let herds = want.floor() as usize + usize::from(rr.f32() < want.fract());
                let spots: Vec<V2> = surveys[r].iter().filter(|(_, c)| c & mask != 0).map(|(p, _)| *p).collect();
                if spots.is_empty() {
                    continue;
                }
                for k in 0..herds {
                    let mut home = None;
                    for _ in 0..10 {
                        let base = spots[rr.below(spots.len())];
                        let p = base.add(V2::new(rr.range(-70.0, 70.0), rr.range(-70.0, 70.0)));
                        let on_shore = d.habitats.contains(&Habitat::Shore);
                        let dry = geo::is_land(p) && geo::inland(p) > if on_shore { 2.0 } else { 30.0 };
                        let apart = self.animals.herds.iter().filter(|h| h.sp == sp).all(|h| h.home.dist(p) > d.range * 0.7);
                        let safe = !d.toward.dangerous() || p.dist(self.squad.pos) > 800.0;
                        if dry && apart && safe && self.clear_of_towns(p, margin) {
                            home = Some(p);
                            break;
                        }
                    }
                    let Some(home) = home else { continue };
                    let cap = lo + rr.below((hi - lo + 1) as usize) as u8;
                    let fill = 0.6 + 0.4 * rr.f32();
                    let id = self.add_herd(sp, home, cap, (cap as f32 * fill).max(1.0), rng::key(&[seed, sp as u64, r as u64, k as u64, 0x4845_5245]));
                    self.haunt_the_road(id, &mut rr);
                }
            }
        }

        // ---- Cragmaws: a couple, deep in the massif ----------------------------
        let massif: Vec<V2> = surveys.iter().flatten().filter(|(_, c)| c & (1 << Habitat::Massif as u16) != 0).map(|(p, _)| *p).collect();
        let mut rr = Rng::from_keys(&[seed, 0xC2A6_3A77]);
        let mut placed: Vec<V2> = Vec::new();
        for _ in 0..200 {
            if placed.len() >= CRAGMAWS || massif.is_empty() {
                break;
            }
            let p = massif[rr.below(massif.len())];
            if placed.iter().all(|q| q.dist(p) > CRAGMAW_APART) && self.clear_of_towns(p, 900.0) && self.terrain.slope(p) < 0.6 {
                let k = placed.len() as u64;
                self.add_herd(Sp::Cragmaw, p, 1, 1.0, rng::key(&[seed, k, 0xC2A6_3A78]));
                placed.push(p);
            }
        }

        // ---- Silk colonies on the hill cliffs ----------------------------------
        let cliffs: Vec<V2> = surveys.iter().flatten().filter(|(_, c)| c & (1 << Habitat::Cliff as u16) != 0).map(|(p, _)| *p).collect();
        let mut rr = Rng::from_keys(&[seed, 0x0051_1C01]);
        let mut placed: Vec<V2> = Vec::new();
        for _ in 0..600 {
            if placed.len() >= COLONIES || cliffs.is_empty() {
                break;
            }
            let p = cliffs[rr.below(cliffs.len())];
            if placed.iter().all(|q| q.dist(p) > COLONY_APART) && self.clear_of_towns(p, 500.0) && p.dist(self.squad.pos) > 800.0 {
                let k = placed.len() as u64;
                let (lo, hi) = Sp::Silkcrawler.def().group;
                let cap = lo + rr.below((hi - lo + 1) as usize) as u8;
                let crawlers = self.add_herd(Sp::Silkcrawler, p, cap, cap as f32 * (0.7 + 0.3 * rr.f32()), rng::key(&[seed, k, 0x0051_1C02]));
                let mother = self.add_herd(Sp::SilkMother, p, 1, 1.0, rng::key(&[seed, k, 0x0051_1C03]));
                let id = self.animals.colonies.len() as u32;
                self.animals.herds[crawlers as usize].colony = Some(id);
                self.animals.herds[mother as usize].colony = Some(id);
                let share = self.animals.herds[crawlers as usize].n / cap as f32;
                self.animals.colonies.push(Colony { id, pos: p, seed: rng::key(&[seed, k, 0x0051_1C04]), crawlers, mother, cocoons: super::hooks::COCOON_CAP * rr.range(0.2, 0.8), per_day: super::hooks::COCOONS_PER_DAY * share, at: self.time });
                placed.push(p);
            }
        }

        self.place_livestock();
        self.stock_the_sea();

        // How each herd grows from here, given who is hunting what.
        let t = self.time;
        self.settle_herds(t);
    }

    /// A hunter takes a stretch of road in its range to watch; an ambusher
    /// with a road close by may make its lair right beside it.
    fn haunt_the_road(&mut self, id: u32, rr: &mut Rng) {
        let h = &self.animals.herds[id as usize];
        let d = h.def();
        match d.toward {
            Toward::PackHunter => {
                let near = self.road_near(h.home, d.range);
                self.animals.herds[id as usize].haunt = near;
            }
            Toward::Ambusher => {
                let roll = rr.f32();
                let side = if rr.chance(0.5) { 1.0 } else { -1.0 };
                if roll >= ROADSIDE_LAIRS {
                    return;
                }
                let Some(road) = self.road_near(h.home, 260.0) else { return };
                let off = h.home.sub(road);
                let len = off.len();
                let dir = if len > 1.0 { off.scale(1.0 / len) } else { V2::new(side, 0.0) };
                let lair = road.add(dir.scale(LAIR_FROM_ROAD));
                if geo::is_land(lair) && geo::inland(lair) > 20.0 && self.clear_of_towns(lair, CLEAR_DANGER) && region_of(lair) == h.region {
                    let h = &mut self.animals.herds[id as usize];
                    h.home = lair;
                    h.haunt = Some(road);
                }
            }
            _ => {}
        }
    }

    /// A clear spot on the edge of a town for a pen `need` metres across.
    fn pen_site(&self, town: usize, rr: &mut Rng, need: f32) -> Option<V2> {
        let s = &self.settlements[town];
        let places: Vec<(V2, f32)> = self.society.towns.get(town).map(|tl| tl.places.iter().map(|p| (p.pos, p.kind.size())).collect()).unwrap_or_default();
        let start = rr.f32() * std::f32::consts::TAU;
        for ring in 0..4 {
            for k in 0..16 {
                let a = start + k as f32 / 16.0 * std::f32::consts::TAU;
                let p = s.pos.add(V2::new(a.cos(), a.sin()).scale(s.radius() + 28.0 + need + ring as f32 * 26.0));
                let ok = geo::is_land(p)
                    && geo::inland(p) > 30.0
                    && self.terrain.slope(p) < 0.22
                    && !self.terrain.on_road(p)
                    && self.road_near(p, need + 8.0).is_none()
                    && places.iter().all(|(q, size)| q.dist(p) > size * 0.75 + need + 3.0)
                    && s.buildings.iter().all(|b| b.pos.dist(p) > b.size * 0.6 + need + 4.0)
                    && self.animals.pens.iter().all(|pen| pen.home.dist(p) > pen.radius + need + 5.0);
                if ok {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Livestock round every town: a Turiyu pasture, a Shellhen coop, pack
    /// animals; Mossbacks for most; a Raftback or two moored off some stilt
    /// villages. How many follows how many people live there.
    fn place_livestock(&mut self) {
        let seed = self.seed;
        for town in 0..self.settlements.len() {
            let mut rr = Rng::from_keys(&[seed, town as u64, 0x4C49_5645]);
            let folk = self.settlements[town].residents.len() as f32;
            let id = self.settlements[town].id;
            let wants: [(Sp, f32, f32); 4] = [
                (Sp::Turiyu, (folk / 8.0).clamp(4.0, 14.0), 1.0),
                (Sp::Shellhen, (folk / 5.0).clamp(6.0, 30.0), 1.0),
                (Sp::Mossback, (folk / 25.0).clamp(2.0, 10.0), 0.65),
                (Sp::Plodder, (2.0 + folk / 50.0).clamp(2.0, 8.0), 1.0),
            ];
            for (sp, limit, chance) in wants {
                let keep = rr.f32() < chance;
                let fill = 0.6 + 0.35 * rr.f32();
                if !keep {
                    continue;
                }
                let limit = limit.round();
                let need = match sp {
                    Sp::Shellhen => 5.0,
                    _ => (4.0 + limit.sqrt() * sp.def().looks.len * 1.6).min(26.0),
                };
                if let Some(at) = self.pen_site(town, &mut rr, need) {
                    self.add_pen(sp, at, Some(id), limit, (limit * fill).round().max(1.0));
                }
            }
            if let Some(stilts) = self.settlements[town].stilts {
                let keep = rr.f32() < 0.6;
                let n = 1.0 + rr.below(2) as f32;
                let south = rr.range(-40.0, 40.0);
                if keep {
                    let at = V2::new(geo::coast_x(stilts.y + south) - 150.0, stilts.y + south);
                    self.add_pen(Sp::Raftback, at, Some(id), n + 1.0, n);
                }
            }
        }
    }
}
