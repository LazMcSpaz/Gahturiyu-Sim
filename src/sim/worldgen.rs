//! Building a fresh world from a seed.

use super::geo::{self, V2, WORLD_SIZE};
use super::group::{Group, Kind, Leg};
use super::person::{Person, PersonId};
use super::race::{Race, ALL_RACES};
use super::rng::{self, Rng};
use super::settlement::{Settlement, SettlementId};
use super::world::{Squad, World, HOUR};
use super::names;
use super::routes::Routes;
use super::terrain::{self, Terrain};

/// How many people live in the world, split evenly between the four races.
pub const POPULATION: usize = 5_000;
/// Coastal towns, strung along the shore.
pub const COASTAL_TOWNS: usize = 10;
/// Upper limit on inland towns (fewer if they do not fit).
pub const INLAND_TOWNS: usize = 20;
/// Above this wanderlust a person belongs nowhere and roams.
pub const WANDERER_THRESHOLD: f32 = 0.85;

pub fn generate(seed: u64) -> World {
    let mut rng = Rng::from_keys(&[seed, 0x574F_524C]);
    let terrain = Terrain::generate(seed);
    let mut settlements = place_settlements(&mut rng, seed, &terrain);

    // --- People ---------------------------------------------------------
    let mut people: Vec<Person> = Vec::with_capacity(POPULATION + 4);
    let mut wanderers: Vec<PersonId> = Vec::new();
    let per_race = POPULATION / ALL_RACES.len();
    for (ri, &race) in ALL_RACES.iter().enumerate() {
        let count = if ri == 0 { POPULATION - per_race * 3 } else { per_race };
        for _ in 0..count {
            let id = people.len() as PersonId;
            let pseed = rng::key(&[seed, id as u64, 0x5045_4F50]);
            let mut p = Person::summary(id, pseed, race, None);
            if p.traits.wanderlust > WANDERER_THRESHOLD {
                wanderers.push(id);
            } else {
                let mut hr = Rng::from_keys(&[pseed, 0x484F_4D45]);
                let w: Vec<f32> = settlements.iter().map(|s| s.size * affinity(race, s)).collect();
                let h = hr.weighted(&w).unwrap_or(0) as SettlementId;
                p.home = Some(h);
                settlements[h as usize].residents.push(id);
            }
            people.push(p);
        }
    }

    // --- Lay out each town from who ended up living there ---------------
    for s in settlements.iter_mut() {
        let races: Vec<Race> = s.residents.iter().map(|&p| people[p as usize].race).collect();
        let homes = s.build(&races, seed);
        for (&p, h) in s.residents.iter().zip(homes) {
            people[p as usize].dwelling = Some(h);
        }
    }

    // --- Your squad: one of each, because they live side by side ---------
    let start_town = settlements
        .iter()
        .filter(|s| s.coastal && s.founders == Race::Roduro)
        .max_by(|a, b| a.residents.len().cmp(&b.residents.len()))
        .or(settlements.iter().max_by(|a, b| a.residents.len().cmp(&b.residents.len())))
        .map(|s| s.id)
        .unwrap_or(0);
    let mut squad_ids = Vec::new();
    // A grown-stone brawler, a coast hunter, a Qotiro shield-fighter and a
    // Ṭaḍoro mage: enough variety to see every part of a fight.
    use super::stats::{Calling, Skill};
    let roles: [(Race, Calling, &[(Skill, f32)], f32); 4] = [
        (Race::Roduro, Calling::Warrior, &[(Skill::Blunt, 58.0), (Skill::Block, 25.0), (Skill::Athletics, 25.0)], 420.0),
        (Race::Horaro, Calling::Hunter, &[(Skill::Spear, 42.0), (Skill::Dodge, 35.0), (Skill::Athletics, 30.0), (Skill::Sneak, 30.0), (Skill::Security, 35.0)], 260.0),
        (Race::Qotiro, Calling::Warrior, &[(Skill::Blunt, 55.0), (Skill::Block, 38.0)], 480.0),
        (Race::Tadoro, Calling::Mage, &[(Skill::Destruction, 48.0), (Skill::Alteration, 42.0), (Skill::Illusion, 44.0), (Skill::Restoration, 40.0), (Skill::Dodge, 25.0)], 120.0),
    ];
    for (i, &(race, calling, skills, budget)) in roles.iter().enumerate() {
        let id = people.len() as PersonId;
        let mut p = Person::summary(id, rng::key(&[seed, 0x5351_5544, i as u64]), race, Some(start_town));
        p.in_squad = true;
        p.cond = Some(super::condition::Condition::new(6.0 * HOUR));
        p.specialize(calling, skills, budget);
        p.ensure_detail();
        // A few lockpicks for the light-fingered.
        if calling == Calling::Hunter {
            let g = &mut p.detail.as_mut().unwrap().gear;
            g.add(super::items::id("tent"), 1);
            // A bow to try (in the pack; equip it to use it).
            g.add(super::items::id("short_bow"), 1);
            g.add(super::items::id("arrows"), 30);
            g.add(super::items::id("standing_torch"), 2);
        }
        let picks = match calling {
            Calling::Hunter => 6,
            Calling::Mage => 2,
            _ => 0,
        };
        if picks > 0 {
            p.detail.as_mut().unwrap().gear.add(super::items::id("lockpick"), picks);
        }
        // A few days' food.
        p.detail.as_mut().unwrap().gear.add(super::items::id("flatbread"), 3);
        p.detail.as_mut().unwrap().gear.add(super::items::id("dried_fish"), 2);
        // Light for the road.
        p.detail.as_mut().unwrap().gear.add(super::items::id("torch"), 2);
        // Something to make things with, to start.
        let starter: &[(&str, u16)] = match calling {
            Calling::Mage => &[("mortar_and_pestle", 1), ("kelp_frond", 4), ("ash_moss", 2), ("ghostcap", 2), ("salt_crystal", 1), ("reed_paper", 2), ("squid_ink", 2), ("healing_draught", 2)],
            Calling::Hunter => &[("hide", 4), ("healing_draught", 1)],
            _ if race == Race::Qotiro => &[("iron_ingot", 3), ("leather", 1), ("timber", 2), ("healing_draught", 1)],
            _ => &[("healing_draught", 1)],
        };
        for &(k, n) in starter {
            p.detail.as_mut().unwrap().gear.add(super::items::id(k), n);
        }
        people.push(p);
        squad_ids.push(id);
    }
    let st = &settlements[start_town as usize];
    // A clear spot near the hearth, not inside somebody's house.
    let mut squad_pos = st.pos.add(V2::new(14.0, 9.0));
    'search: for ring in 1..40 {
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU + ring as f32;
            let c = st.pos.add(V2::new(a.cos(), a.sin()).scale(6.0 + ring as f32 * 2.5));
            if st.buildings.iter().skip(1).all(|b| b.pos.dist(c) > b.size * 0.65 + 5.0) && geo::inland(c) > 5.0 {
                squad_pos = c;
                break 'search;
            }
        }
    }
    let squad = Squad::new(squad_ids, squad_pos);

    // --- Wanderers: each starts somewhere in the wild, resting ----------
    let mut groups = Vec::new();
    for (gi, &pid) in wanderers.iter().enumerate() {
        let p = &mut people[pid as usize];
        let mut wr = Rng::from_keys(&[p.seed, 0x5752_4E44]);
        let mut at = V2::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0);
        for _ in 0..80 {
            let c = V2::new(wr.range(800.0, WORLD_SIZE - 800.0), wr.range(800.0, WORLD_SIZE - 800.0));
            if geo::inland(c) > 400.0 && terrain.height(c) < 380.0 && terrain.slope(c) < 0.25 {
                at = c;
                break;
            }
        }
        let first_rest = wr.f64() * 8.0 * HOUR;
        let restless = p.traits.wanderlust as f64;
        groups.push(Group {
            id: gi as u32,
            seed: p.seed,
            members: vec![pid],
            kind: Kind::Wanderer { rest_min: HOUR * (1.0 + 4.0 * (1.0 - restless)), rest_max: HOUR * (6.0 + 14.0 * (1.0 - restless)) },
            legs: vec![Leg::wait(at, first_rest)],
            speed: p.race.walk_speed(),
            ends: f64::INFINITY,
            written: 1,
            hostile: false,
            pos: at,
            last_update: 0.0,
            band: 3,
        });
    }

    // --- Roads between the towns ----------------------------------------
    let (terrain, routes) = land(terrain, &settlements);

    // Start at 06:00 on day 1.
    let mut w = World::assemble(seed, people, settlements, groups, squad, 6.0 * HOUR, terrain, routes);
    // Bandits by the roads; workshops in town; things to gather.
    w.place_camps();
    w.place_crafting();
    w
}

/// The land and the roads between the towns: fixed by the seed and where
/// the towns are, so a save doesn't store them.
pub fn land(mut terrain: Terrain, settlements: &[Settlement]) -> (Terrain, Routes) {
    let mut routes = Routes::build(&terrain, settlements);
    terrain.set_roads(&routes.roads);
    routes.build_network(&terrain);
    (terrain, routes)
}

/// How strongly a settlement draws people of a race. Everyone lives
/// everywhere; this only tilts the odds.
fn affinity(race: Race, s: &Settlement) -> f32 {
    // Founders are the biggest single group in their own town, usually, but
    // every town is a mix (Laz, 2026-10-08: Horaro are very present on the
    // coast but rarely the majority).
    let founders = if race == s.founders { 2.5 } else { 1.0 };
    founders * match race {
        // The water people settle just off any coast, whoever owns the shore;
        // a few live inland in land homes.
        Race::Horaro => {
            if s.coastal {
                1.3
            } else {
                0.75
            }
        }
        // The air people lodge in other peoples' towns rather than build their own.
        Race::Tadoro => 1.3,
        _ => 1.0,
    }
}

fn place_settlements(rng: &mut Rng, seed: u64, t: &Terrain) -> Vec<Settlement> {
    let mut out: Vec<Settlement> = Vec::new();
    let push = |out: &mut Vec<Settlement>, pos: V2, founders: Race, coastal: bool, rng: &mut Rng| {
        let id = out.len() as SettlementId;
        out.push(Settlement {
            id,
            name: names::place_name(founders, rng::key(&[seed, id as u64, 0x544F_574E])),
            pos,
            founders,
            coastal,
            stilts: if coastal { Some(V2::new(geo::coast_x(pos.y) - 70.0, pos.y)) } else { None },
            size: rng.f32().powi(2) * 1.8 + 0.35,
            residents: Vec::new(),
            buildings: Vec::new(),
            reach: 0.0,
        });
    };

    // Coastal towns, spaced down the shore.
    for i in 0..COASTAL_TOWNS {
        let y0 = ((i as f32 + 0.5) / COASTAL_TOWNS as f32 * WORLD_SIZE + rng.range(-800.0, 800.0)).clamp(900.0, WORLD_SIZE - 900.0);
        let back = rng.range(170.0, 300.0);
        // Slide along the shore to the nearest stretch of low, gentle beach:
        // nobody builds a harbour town on a cliff or a mountainside.
        let site = |y: f32| V2::new(geo::coast_x(y) + back, y);
        let badness = |y: f32| {
            let p = site(y);
            terrain::cliff_mask(t.seed(), y) * 4.0 + t.slope(p) * 20.0 + (t.height(p) - 12.0).max(0.0) * 0.1 + t.mountains(p) * 10.0
        };
        // ...but stay well clear of the coastal towns already placed.
        let crowded = |y: f32, out: &Vec<Settlement>| out.iter().any(|s| s.pos.dist(site(y)) < 2400.0);
        let mut y = y0;
        for k in 1..50 {
            for dy in [k as f32 * 60.0, -(k as f32) * 60.0] {
                let c = (y0 + dy).clamp(900.0, WORLD_SIZE - 900.0);
                if !crowded(c, &out) && (crowded(y, &out) || badness(c) < badness(y)) {
                    y = c;
                }
            }
        }
        // No decent shore anywhere near: this stretch gets no harbour.
        if crowded(y, &out) || badness(y) > 2.0 {
            continue;
        }
        let x = site(y).x;
        let founders = match rng.weighted(&[0.55, 0.30, 0.15]) {
            Some(0) => Race::Roduro,
            Some(1) => Race::Horaro,
            _ => Race::Qotiro,
        };
        push(&mut out, V2::new(x, y), founders, true, rng);
    }

    // Inland towns. The south-east is Qotiro country; elsewhere mostly Roduro.
    let mut tries = 0;
    while out.len() < COASTAL_TOWNS + INLAND_TOWNS && tries < 5000 {
        tries += 1;
        let p = V2::new(rng.range(800.0, WORLD_SIZE - 800.0), rng.range(800.0, WORLD_SIZE - 800.0));
        if geo::inland(p) < 2200.0 || out.iter().any(|s| s.pos.dist(p) < 2700.0) {
            continue;
        }
        // Flat, habitable ground: not up a mountain or on a slope.
        if t.height(p) > 260.0 || t.slope(p) > 0.07 || t.mountains(p) > 0.25 || (t.plateau(p) > 0.05 && t.plateau(p) < 0.95) {
            continue;
        }
        let homeland = p.x > 11_500.0 && p.y > 10_500.0;
        let founders = if rng.chance(if homeland { 0.75 } else { 0.28 }) { Race::Qotiro } else { Race::Roduro };
        push(&mut out, p, founders, false, rng);
    }
    out
}
