//! Bases (Part 7): construction is worked out from the clock, so how finely
//! the world is stepped never changes when a building stands, and changing
//! the builders part-way gives the same answer as working it out in one go.

use gahturiyu_sim::sim::{
    base::{self, def_index, Plan, BUILDINGS},
    geo::V2,
    items,
    person::PersonId,
    stats::Skill,
    world::HOUR,
    worldgen, World,
};

/// A world with a base laid near the squad, the squad standing at it, and
/// the base's id plus a free spot beside the marker for a building.
pub fn with_base(seed: u64) -> (World, u32, V2) {
    let w0 = worldgen::generate(seed);
    let start = w0.squad.pos;
    for ring in 0..40 {
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let p = start.add(V2::new(a.cos(), a.sin()).scale(30.0 + ring as f32 * 25.0));
            let mut w = w0.clone();
            let Ok(bid) = w.found_base(p) else { continue };
            // Out in the wilds, with room for a wall north of the marker.
            if w.base(bid).unwrap().land != base::Land::Wilds {
                continue;
            }
            let wall = World::wall_plans(def_index("palisade"), &[p.add(V2::new(-12.0, 18.0)), p.add(V2::new(12.0, 18.0))]);
            if !wall.into_iter().all(|mut q| w.check_place(&mut q).is_ok()) {
                continue;
            }
            // A spot for a hut beside it.
            for off in [V2::new(10.0, 0.0), V2::new(-10.0, 0.0), V2::new(0.0, -10.0)] {
                let mut plan = Plan { def: def_index("hut"), at: p.add(off), rot: 0.0, w: BUILDINGS[def_index("hut")].w, replaces: None };
                if w.check_place(&mut plan).is_ok() {
                    w.teleport_squad(p.add(V2::new(0.0, -6.0)));
                    // The marker goes up first (half an hour).
                    let mut w = run_until(w, 1.0, 60.0);
                    assert!(w.base(bid).unwrap().building(0).unwrap().standing(), "the camp marker stands");
                    w.step(0.001);
                    return (w, bid, p.add(off));
                }
            }
        }
    }
    panic!("nowhere to lay a base near the squad");
}

/// The squad member who starts knowing carpentry (the hunter).
pub fn carpenter(w: &World) -> PersonId {
    w.squad.members.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().unwrap().crafts.contains(&Skill::Carpentry)).expect("the hunter knows carpentry")
}

/// Give someone materials and put them in the base's store.
pub fn supply(w: &mut World, who: PersonId, bid: u32, what: &[(&str, u16)]) {
    let d = w.people[who as usize].detail.as_mut().unwrap();
    for &(k, n) in what {
        d.gear.add(items::id(k), n);
    }
    w.store_materials(bid);
}

fn stood(w: &World, bid: u32, id: u32) -> Option<f64> {
    w.base(bid).unwrap().building(id).unwrap().stood_at()
}

fn run_until(mut w: World, hours: f64, step: f64) -> World {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-9 {
        w.step(step.min(end - w.time));
    }
    w
}

#[test]
fn construction_finishes_at_the_same_moment_whatever_the_step() {
    let (mut w, bid, spot) = with_base(1);
    let c = carpenter(&w);
    supply(&mut w, c, bid, &[("timber", 6), ("seareed", 4)]);
    let hut = w.place_building(Plan { def: def_index("hut"), at: spot, rot: 0.0, w: BUILDINGS[def_index("hut")].w, replaces: None }).expect("hut placed");
    let site = w.base(bid).unwrap().building(hut).unwrap().site().unwrap().clone();
    let expect = site.done_at.expect("someone is building it");
    // One builder at their own pace.
    let pace = base::pace(w.people[c as usize].stats.skill(Skill::Carpentry));
    let hours = BUILDINGS[def_index("hut")].labour / pace;
    assert!((expect - (w.time + hours as f64 * HOUR)).abs() < 1.0, "finish solved from the pace: {expect} vs {}", w.time + hours as f64 * HOUR);
    let a = run_until(w.clone(), hours as f64 + 1.0, 1.0);
    let b = run_until(w.clone(), hours as f64 + 1.0, 97.0);
    let c2 = run_until(w, hours as f64 + 1.0, HOUR);
    let (ta, tb, tc) = (stood(&a, bid, hut).expect("stands (1 s steps)"), stood(&b, bid, hut).expect("stands (97 s steps)"), stood(&c2, bid, hut).expect("stands (hour steps)"));
    assert!((ta - expect).abs() < 1e-3 && (tb - expect).abs() < 1e-3 && (tc - expect).abs() < 1e-3, "stood at {ta} / {tb} / {tc}, expected {expect}");
}

#[test]
fn changing_builders_midway_matches_working_it_out_in_one_go() {
    let (mut w, bid, spot) = with_base(1);
    let c = carpenter(&w);
    // A second builder: someone else in the squad takes up carpentry.
    let other = *w.squad.members.iter().find(|&&m| m != c).unwrap();
    w.people[other as usize].detail.as_mut().unwrap().crafts.push(Skill::Carpentry);
    supply(&mut w, c, bid, &[("timber", 6), ("seareed", 4)]);
    let def = def_index("hut");
    let hut = w.place_building(Plan { def, at: spot, rot: 0.0, w: BUILDINGS[def].w, replaces: None }).unwrap();
    let t0 = w.time;
    let (p1, p2) = (base::pace(w.people[c as usize].stats.skill(Skill::Carpentry)), base::pace(w.people[other as usize].stats.skill(Skill::Carpentry)));
    // Both build for two hours; then the second walks off.
    let leave = |w: &mut World| {
        let k = w.squad.index(other).unwrap();
        let away = w.base(bid).unwrap().at.add(V2::new(400.0, 0.0));
        w.squad.at[k] = away;
        w.squad.goal[k] = away;
        w.squad.route[k].clear();
    };
    let mut runs = Vec::new();
    for step in [1.0, 60.0, 600.0] {
        let mut x = run_until(w.clone(), 2.0, step);
        leave(&mut x);
        let x = run_until(x, 40.0, step);
        runs.push(stood(&x, bid, hut).expect("stands"));
    }
    // Worked out in one go: two hours at both paces, the rest at one.
    let total = BUILDINGS[def].labour;
    let after_two = 2.0 * (p1 + p2);
    let expect = t0 + 2.0 * HOUR + ((total - after_two) / p1) as f64 * HOUR;
    for t in &runs {
        assert!((t - expect).abs() < 2.0, "stood at {t}, expected {expect} (runs {runs:?})");
    }
    assert!((runs[0] - runs[1]).abs() < 1e-3 && (runs[1] - runs[2]).abs() < 1e-3, "{runs:?}");
}

#[test]
fn nothing_is_built_without_its_materials_or_a_builder_who_knows_how() {
    let (mut w, bid, spot) = with_base(1);
    let def = def_index("hut");
    let hut = w.place_building(Plan { def, at: spot, rot: 0.0, w: BUILDINGS[def].w, replaces: None }).unwrap();
    let w = run_until(w, 24.0, 600.0);
    assert!(stood(&w, bid, hut).is_none(), "a hut with no timber can't stand");
    // Masonry: nobody knows it.
    let (mut w, bid, _) = with_base(1);
    let c = carpenter(&w);
    assert!(w.squad_builders(def_index("well")).is_empty(), "nobody in the squad starts with masonry");
    supply(&mut w, c, bid, &[("rock", 14)]);
    assert_eq!(w.base(bid).unwrap().count_in_store(items::id("rock")), 14);
}

#[test]
fn placement_says_no_where_it_should() {
    let (mut w, bid, spot) = with_base(1);
    let at = w.base(bid).unwrap().at;
    let def = def_index("hut");
    // On top of the marker.
    let mut p = Plan { def, at, rot: 0.0, w: BUILDINGS[def].w, replaces: None };
    assert_eq!(w.check_place(&mut p), Err(base::Bad::Overlaps));
    // Too far out.
    let mut p = Plan { def, at: at.add(V2::new(300.0, 0.0)), rot: 0.0, w: BUILDINGS[def].w, replaces: None };
    assert!(matches!(w.check_place(&mut p), Err(base::Bad::NoBase) | Err(base::Bad::TooFar)));
    // A second camp too close.
    assert_eq!(w.found_base(at.add(V2::new(60.0, 0.0))), Err(base::Bad::NearBase));
    // A gate needs a wall; once the wall is drawn, it snaps on.
    let gdef = def_index("gate");
    let mut g = Plan { def: gdef, at: spot, rot: 0.0, w: BUILDINGS[gdef].w, replaces: None };
    assert_eq!(w.check_place(&mut g), Err(base::Bad::NeedsWall));
    let a = at.add(V2::new(-12.0, 18.0));
    let b2 = at.add(V2::new(12.0, 18.0));
    if let Ok(ids) = w.place_wall(def_index("palisade"), &[a, b2]) {
        assert!(ids.len() >= 6, "a 24 m wall is cut into segments of 4 m or less ({})", ids.len());
        let mut g = Plan { def: gdef, at: a.lerp(b2, 0.5).add(V2::new(0.5, 0.5)), rot: 0.0, w: BUILDINGS[gdef].w, replaces: None };
        assert!(w.check_place(&mut g).is_ok(), "gate snaps onto the wall");
        assert!(g.replaces.is_some());
    }
}

