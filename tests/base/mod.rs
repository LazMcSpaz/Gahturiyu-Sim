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
    // The same moment whatever the step, to the last digit; and within a
    // blink of the one worked out when the site was laid (a change of shift
    // in between settles the labour so far, which rounds it by milliseconds).
    assert!(ta == tb && tb == tc, "stood at {ta} / {tb} / {tc}");
    assert!((ta - expect).abs() < 0.05, "stood at {ta}, expected {expect}");
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


// ---- Stage 2: life at the base ------------------------------------------------------

use gahturiyu_sim::sim::baselife::Job;

/// Everyone in the squad knows both building crafts, well.
fn teach_all(w: &mut World) {
    for m in w.squad.members.clone() {
        let p = &mut w.people[m as usize];
        for s in [Skill::Carpentry, Skill::Masonry] {
            if !p.detail.as_ref().unwrap().crafts.contains(&s) {
                p.detail.as_mut().unwrap().crafts.push(s);
            }
            p.stats.set_skill(s, 50.0);
        }
    }
}

/// Place a building somewhere free near the marker.
fn place_near(w: &mut World, bid: u32, key: &str) -> u32 {
    let at = w.base(bid).unwrap().at;
    let def = def_index(key);
    for r in [8.0f32, 13.0, 18.0, 23.0, 28.0] {
        for k in 0..16 {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let plan = Plan { def, at: at.add(V2::new(a.cos(), a.sin()).scale(r)), rot: a, w: BUILDINGS[def].w, replaces: None };
            if w.check_place(&mut plan.clone()).is_ok() {
                return w.place_building(plan).unwrap();
            }
        }
    }
    panic!("no room for a {key}");
}

/// An outpost with a field, a kitchen, a hut and a workbench shed up, three
/// of the squad living there (farmer, cook, hauler) with a little food, and
/// the fourth gone off.
fn outpost() -> (World, u32) {
    let (mut w, bid, _) = with_base(1);
    teach_all(&mut w);
    let c = carpenter(&w);
    for key in ["field_plot", "hearth_kitchen", "hut", "shed_workbench"] {
        place_near(&mut w, bid, key);
    }
    supply(&mut w, c, bid, &[("timber", 19), ("rock", 14)]);
    let other = w.squad.members[1];
    supply(&mut w, other, bid, &[("seareed", 4), ("clay", 6)]);
    w = run_until(w, 48.0, 600.0);
    assert_eq!(w.base(bid).unwrap().buildings.iter().filter(|b| b.standing()).count(), 5, "everything stands");
    let m = w.squad.members.clone();
    {
        let d = w.people[m[0] as usize].detail.as_mut().unwrap();
        d.gear.add(items::id("timber"), 8);
    }
    w.store_materials(bid);
    // Food for the residents (the store button takes building materials).
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("flatbread"), 12);
    for (k, job) in [(1usize, Job::Farmer), (2, Job::Cook), (3, Job::Hauler)] {
        w.leave_at_base(m[k], bid).expect("left at the base");
        w.set_base_job(m[k], job);
    }
    (w, bid)
}

fn snapshot(w: &World, bid: u32) -> String {
    let b = w.base(bid).unwrap();
    let mut store: Vec<String> = b.store.iter().map(|e| format!("{}x{}", e.0, e.1)).collect();
    store.sort();
    let hunger: Vec<String> = b.residents.iter().map(|r| format!("{:.3}", w.hunger_of(r.who).unwrap())).collect();
    let hp: Vec<String> = b.buildings.iter().map(|x| format!("{:.3}", x.hp_at(w.time))).collect();
    format!("{store:?}\n{hunger:?}\n{hp:?}\n{:?}", b.log)
}

#[test]
fn a_week_away_is_the_same_as_a_week_watched() {
    let (w, bid) = outpost();
    let at = w.base(bid).unwrap().at;
    let away = |mut w: World, d: f32| {
        w.teleport_squad(at.add(V2::new(d, 0.0)));
        w
    };
    let a = run_until(away(w.clone(), 3000.0), 7.0 * 24.0, 600.0);
    let b = run_until(away(w.clone(), 200.0), 7.0 * 24.0, 60.0);
    let c = run_until(away(w, 3000.0), 7.0 * 24.0, HOUR);
    let (sa, sb, sc) = (snapshot(&a, bid), snapshot(&b, bid), snapshot(&c, bid));
    assert_eq!(sa, sb, "away (10-minute steps) vs watched (1-minute steps)");
    assert_eq!(sa, sc, "away (10-minute steps) vs away (hour steps)");
    // And something happened: grain was grown and baked, wood fetched.
    let base = a.base(bid).unwrap();
    assert!(base.log.iter().any(|l| l.1.contains("grain")), "a harvest came in: {:?}", base.log);
    assert!(base.count_in_store(items::id("rock")) > 0, "the hauler brought stone");
}

#[test]
fn thatch_rots_unless_sealed_and_builders_mend_it() {
    let (mut w, bid) = outpost();
    let hut = w.base(bid).unwrap().buildings.iter().find(|b| b.def().key == "hut").unwrap().id;
    let hp0 = BUILDINGS[def_index("hut")].hp;
    // Ten days of rot, nobody mending (the builders are all away).
    let at = w.base(bid).unwrap().at;
    w.teleport_squad(at.add(V2::new(3000.0, 0.0)));
    let start = w.base(bid).unwrap().building(hut).unwrap().hp_at(w.time);
    let w10 = run_until(w.clone(), 240.0, 3600.0);
    let hp = w10.base(bid).unwrap().building(hut).unwrap().hp_at(w10.time);
    assert!(((start - hp) / hp0 - 0.1).abs() < 0.001, "a tenth gone in ten days: {start} -> {hp} of {hp0}");
    // A resident builder mends it back.
    let mut wm = w10.clone();
    let farmer = wm.base(bid).unwrap().residents[0].who;
    wm.set_base_job(farmer, Job::Builder);
    let wm = run_until(wm, 24.0, 600.0);
    let hp = wm.base(bid).unwrap().building(hut).unwrap().hp_at(wm.time);
    assert!(hp > hp0 * 0.99, "mended: {hp}");
    // Sealed with pitch, it holds.
    let mut ws = w.clone();
    let i = ws.bases.iter().position(|b| b.id == bid).unwrap();
    ws.bases[i].add_to_store(items::id("pitch"), 1);
    ws.seal_building(bid, hut).expect("sealed");
    let sealed_at = ws.base(bid).unwrap().building(hut).unwrap().hp_at(ws.time);
    let ws = run_until(ws, 240.0, 3600.0);
    assert!((ws.base(bid).unwrap().building(hut).unwrap().hp_at(ws.time) - sealed_at).abs() < 1e-3, "sealed thatch doesn't rot");
}

#[test]
fn leaving_and_picking_up() {
    let (mut w, bid) = outpost();
    assert_eq!(w.squad.members.len(), 1);
    let last = w.squad.members[0];
    assert!(w.leave_at_base(last, bid).is_err(), "someone has to travel");
    let who = w.base(bid).unwrap().residents[0].who;
    let at = w.base(bid).unwrap().at;
    w.teleport_squad(at.add(V2::new(500.0, 0.0)));
    w.step(1.0);
    assert!(w.pick_up(who).is_err(), "nobody there to collect them");
    w.teleport_squad(at);
    w.step(1.0);
    w.pick_up(who).expect("collected");
    assert_eq!(w.squad.members.len(), 2);
    assert!(w.resident_of(who).is_none());
}

#[test]
fn the_store_has_only_so_much_room() {
    let (mut w, bid, _) = with_base(1);
    let c = carpenter(&w);
    let cap = w.base(bid).unwrap().capacity();
    supply(&mut w, c, bid, &[("rock", 200)]);
    let b = w.base(bid).unwrap();
    assert!(b.load() <= cap + 1e-3 && b.count_in_store(items::id("rock")) < 200, "{} kg in a store of {cap}", b.load());
    assert!(w.count_of(c, "rock") > 0, "the rest stays in the pack");
}

// ---- Stage 3: hired hands -----------------------------------------------------------

/// Someone in a town who'd come and work at the base, nearest the base first.
fn willing(w: &World) -> PersonId {
    let mut best: Option<(f32, PersonId)> = None;
    for s in &w.settlements {
        for &p in &s.residents {
            if w.hire_terms(p).is_some() {
                let d = w.person_pos(p).dist(w.bases[0].at);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, p));
                }
            }
        }
    }
    best.expect("someone willing").1
}

fn coin(w: &mut World, who: PersonId, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), n);
}

#[test]
fn a_hired_hand_really_leaves_town_and_comes_to_work() {
    let (mut w, bid) = outpost();
    let hand = willing(&w);
    let town = w.people[hand as usize].home.unwrap();
    w.hire(hand).expect("hired");
    assert!(!w.settlements[town as usize].residents.contains(&hand), "gone from the town's roll");
    assert!(w.society.lives[hand as usize].community.is_none() && w.society.lives[hand as usize].household.is_none());
    w.set_base_job(hand, Job::Hauler);
    let arrives = w.base(bid).unwrap().resident(hand).unwrap().hire.as_ref().unwrap().arrives;
    assert!(arrives > w.time, "they have to walk there");
    let hours = ((arrives - w.time) / HOUR).ceil() + 4.0;
    let w = run_until(w, hours, 600.0);
    assert!(w.base(bid).unwrap().arrived(hand));
    assert!(w.base(bid).unwrap().log.iter().any(|l| l.1.contains("arrives")));
}

#[test]
fn unpaid_hands_quit_and_go_home_paid_ones_stay() {
    let (mut w, bid) = outpost();
    let hand = willing(&w);
    let town = w.people[hand as usize].home.unwrap();
    // No coin anywhere: they go unpaid.
    for m in w.squad.members.clone() {
        let c = w.count_of(m, "coin");
        let d = w.people[m as usize].detail.as_mut().unwrap();
        for _ in 0..c {
            d.gear.take(items::id("coin"));
        }
    }
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("flatbread"), 40);
    let mut paid = w.clone();
    w.hire(hand).unwrap();
    let w = run_until(w, 5.0 * 24.0, 3600.0);
    assert!(w.resident_of(hand).is_none(), "unpaid, they quit within five days");
    assert!(w.settlements[town as usize].residents.contains(&hand), "and went home");
    assert_eq!(w.society.lives[hand as usize].job, gahturiyu_sim::sim::jobs::Job::Labourer);
    assert!(w.base(bid).unwrap().log.iter().any(|l| l.1.contains("quits")));
    // The same hand, paid.
    let m = paid.squad.members[0];
    coin(&mut paid, m, 400);
    let hh = paid.society.lives[hand as usize].household;
    // Two of the squad's own come away, so there's a bed for the hand.
    for _ in 0..2 {
        let own = paid.base(bid).unwrap().residents[0].who;
        paid.pick_up(own).unwrap();
    }
    let _ = hh;
    paid.hire(hand).unwrap();
    let paid = run_until(paid, 5.0 * 24.0, 3600.0);
    let r = paid.base(bid).unwrap().resident(hand).expect("still there");
    assert!(r.hire.as_ref().unwrap().loyalty > 0.6, "content: {}", r.hire.as_ref().unwrap().loyalty);
    let h = r.hire.as_ref().unwrap();
    let spent = 400 - paid.squad_count(items::id("coin"));
    assert!(spent > 0 && spent % h.wage == 0 && h.owed == 0, "wages came out of the squad's purse, a day's at a time: {spent} at {} a day", h.wage);
}

#[test]
fn a_dishonest_hand_helps_themselves_on_the_way_out() {
    let (mut w, bid) = outpost();
    let hand = willing(&w);
    w.society.lives[hand as usize].habits.honour = 0.0;
    for m in w.squad.members.clone() {
        let c = w.count_of(m, "coin");
        let d = w.people[m as usize].detail.as_mut().unwrap();
        for _ in 0..c {
            d.gear.take(items::id("coin"));
        }
    }
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("iron_ingot"), 3);
    w.hire(hand).unwrap();
    let w = run_until(w, 5.0 * 24.0, 3600.0);
    assert!(w.resident_of(hand).is_none());
    assert!(w.count_of(hand, "iron_ingot") > 0, "the ingots went with them");
    assert!(w.base(bid).unwrap().log.iter().any(|l| l.1.contains("on the way out")));
}

#[test]
fn a_week_with_hired_hands_is_the_same_watched_or_away() {
    let (mut w, bid) = outpost();
    let hand = willing(&w);
    let m = w.squad.members[0];
    coin(&mut w, m, 30);
    w.hire(hand).unwrap();
    w.set_base_job(hand, Job::Hauler);
    let at = w.base(bid).unwrap().at;
    let away = |mut w: World, d: f32| {
        w.teleport_squad(at.add(V2::new(d, 0.0)));
        w
    };
    let a = run_until(away(w.clone(), 3000.0), 7.0 * 24.0, 600.0);
    let b = run_until(away(w.clone(), 200.0), 7.0 * 24.0, 60.0);
    let c = run_until(away(w, 3000.0), 7.0 * 24.0, HOUR);
    let loyal = |w: &World| format!("{:?} {:?} {}", w.base(bid).unwrap().residents.iter().map(|r| r.hire.as_ref().map(|h| (h.loyalty, h.owed))).collect::<Vec<_>>(), w.resident_of(hand), w.squad_count(items::id("coin")));
    assert_eq!(snapshot(&a, bid) + &loyal(&a), snapshot(&b, bid) + &loyal(&b));
    assert_eq!(snapshot(&a, bid) + &loyal(&a), snapshot(&c, bid) + &loyal(&c));
}
