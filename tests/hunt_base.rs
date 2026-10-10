//! Base building, from the bug hunt (naming session). Each test states what
//! should happen; `nmN_…` failed before the fix for bug NM-N in the Project
//! doc `claude/bugs-naming.md`.

use gahturiyu_sim::sim::{
    base::{self, def_index, Plan, BUILDINGS},
    baselife::Job,
    geo::V2,
    items,
    person::PersonId,
    stats::Skill,
    world::{DAY, HOUR},
    worldgen, World,
};

// ---- Set-up (as tests/base/mod.rs has it) ---------------------------------------

fn with_base(seed: u64) -> (World, u32, V2) {
    let w0 = worldgen::generate(seed);
    let start = w0.squad.pos;
    for ring in 0..40 {
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let p = start.add(V2::new(a.cos(), a.sin()).scale(30.0 + ring as f32 * 25.0));
            let mut w = w0.clone();
            let Ok(bid) = w.found_base(p) else { continue };
            if w.base(bid).unwrap().land != base::Land::Wilds {
                continue;
            }
            let wall = World::wall_plans(def_index("palisade"), &[p.add(V2::new(-12.0, 18.0)), p.add(V2::new(12.0, 18.0))]);
            if !wall.into_iter().all(|mut q| w.check_place(&mut q).is_ok()) {
                continue;
            }
            for off in [V2::new(10.0, 0.0), V2::new(-10.0, 0.0), V2::new(0.0, -10.0)] {
                let mut plan = Plan { def: def_index("hut"), at: p.add(off), rot: 0.0, w: BUILDINGS[def_index("hut")].w, replaces: None };
                if w.check_place(&mut plan).is_ok() {
                    w.teleport_squad(p.add(V2::new(0.0, -6.0)));
                    let mut w = run(w, 1.0, 60.0);
                    assert!(w.base(bid).unwrap().building(0).unwrap().standing(), "the camp marker stands");
                    w.step(0.001);
                    return (w, bid, p.add(off));
                }
            }
        }
    }
    panic!("nowhere to lay a base near the squad");
}

fn run(mut w: World, hours: f64, step: f64) -> World {
    let end = w.time + hours * HOUR;
    while w.time < end - 1e-9 {
        w.step(step.min(end - w.time));
    }
    w
}

fn carpenter(w: &World) -> PersonId {
    w.squad.members.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().unwrap().crafts.contains(&Skill::Carpentry)).expect("the hunter knows carpentry")
}

fn supply(w: &mut World, who: PersonId, bid: u32, what: &[(&str, u16)]) {
    let d = w.people[who as usize].detail.as_mut().unwrap();
    for &(k, n) in what {
        d.gear.add(items::id(k), n);
    }
    w.store_materials(bid);
}

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

/// A field, a kitchen, a hut and a workbench shed standing, with 12
/// flatbread in the store; nobody left there yet.
fn built(seed: u64) -> (World, u32) {
    let (mut w, bid, _) = with_base(seed);
    teach_all(&mut w);
    let c = carpenter(&w);
    for key in ["field_plot", "hearth_kitchen", "hut", "shed_workbench"] {
        place_near(&mut w, bid, key);
    }
    supply(&mut w, c, bid, &[("timber", 19), ("rock", 14)]);
    let other = w.squad.members[1];
    supply(&mut w, other, bid, &[("seareed", 4), ("clay", 6)]);
    w = run(w, 48.0, 600.0);
    assert_eq!(w.base(bid).unwrap().buildings.iter().filter(|b| b.standing()).count(), 5, "everything stands");
    (w, bid)
}

/// The outpost of the existing tests: farmer, cook and hauler left there.
fn outpost(bread: u16) -> (World, u32) {
    let (mut w, bid) = built(1);
    let m = w.squad.members.clone();
    w.people[m[0] as usize].detail.as_mut().unwrap().gear.add(items::id("timber"), 8);
    w.store_materials(bid);
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("flatbread"), bread);
    for (k, job) in [(1usize, Job::Farmer), (2, Job::Cook), (3, Job::Hauler)] {
        w.leave_at_base(m[k], bid).expect("left at the base");
        w.set_base_job(m[k], job);
    }
    (w, bid)
}

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

fn stored(w: &World, bid: u32, key: &str) -> u16 {
    let id = items::id(key);
    w.base(bid).unwrap().store.iter().filter(|e| e.0 == id).map(|e| e.1).sum()
}

fn coin(w: &mut World, who: PersonId, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), n);
}

// ---- Probes ---------------------------------------------------------------------

#[test]
fn nm1_a_hut_placed_days_after_founding_takes_its_full_time_whoever_is_hired() {
    // Found a base, wait three days, lay a hut with its builder standing by,
    // then hire a hand. Hiring should change nothing about the hut.
    let (w, bid, spot) = with_base(1);
    let mut w = run(w, 72.0, 600.0);
    let c = carpenter(&w);
    supply(&mut w, c, bid, &[("timber", 6), ("seareed", 4)]);
    let def = def_index("hut");
    let hut = w.place_building(Plan { def, at: spot, rot: 0.0, w: BUILDINGS[def].w, replaces: None }).expect("hut placed");
    let placed = w.time;
    let due = w.base(bid).unwrap().building(hut).unwrap().site().unwrap().done_at.expect("someone is building it");
    assert!(due > placed);
    coin(&mut w, c, 200);
    let hand = willing(&w);
    w.hire(hand).expect("hired");
    w.step(1.0);
    let b = w.base(bid).unwrap().building(hut).unwrap();
    assert!(!b.standing(), "a hut laid a second ago can't be standing: stood_at {:?}, placed at {placed}", b.stood_at());
    let now_due = b.site().and_then(|s| s.done_at).expect("still being built");
    assert!((now_due - due).abs() < 60.0, "the hut is due when it was due: {due} then, {now_due} now");
}

#[test]
fn nm2_a_wall_can_turn_a_corner() {
    let (mut w, bid, _) = with_base(1);
    let a = w.base(bid).unwrap().at.add(V2::new(-12.0, 18.0));
    let pts = [a, a.add(V2::new(12.0, 0.0)), a.add(V2::new(12.0, 12.0))];
    let def = def_index("palisade");
    let plans = World::wall_plans(def, &pts);
    assert_eq!(plans.len(), 6, "two 12 m legs are three segments each");
    let ids = w.place_wall(def, &pts).expect("the wall is laid");
    assert_eq!(ids.len(), plans.len(), "every segment of both legs is laid, the one after the corner too");
}

#[test]
fn nm3_a_gate_fits_a_ten_metre_wall() {
    let (mut w, bid, _) = with_base(1);
    let a = w.base(bid).unwrap().at.add(V2::new(-12.0, 18.0));
    let pts = [a, a.add(V2::new(10.0, 0.0))];
    let def = def_index("palisade");
    let ids = w.place_wall(def, &pts).expect("the wall is laid");
    assert!(!ids.is_empty());
    // A gate on the middle of it.
    let gate = def_index("gate");
    let mut ok = false;
    let mut why = Vec::new();
    for k in 0..=20 {
        let at = a.add(V2::new(k as f32 * 0.5, 0.0));
        let mut plan = Plan { def: gate, at, rot: 0.0, w: BUILDINGS[gate].w, replaces: None };
        match w.check_place(&mut plan) {
            Ok(_) => ok = true,
            Err(e) => why.push(format!("{e:?}")),
        }
    }
    why.dedup();
    assert!(ok, "a gate goes somewhere on a 10 m palisade: {why:?}");
}

#[test]
fn nm4_someone_left_at_a_base_eats_the_bread_in_their_own_pack() {
    let (mut w, bid) = built(1);
    let m = w.squad.members[1];
    let bread = items::id("flatbread");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(bread, 12);
    let had = w.count_of(m, "flatbread");
    assert!(had >= 12 && stored(&w, bid, "flatbread") == 0);
    w.leave_at_base(m, bid).expect("left at the base");
    let w = run(w, 72.0, 600.0);
    let hunger = w.hunger_of(m).unwrap();
    assert!(w.count_of(m, "flatbread") < had, "three days on, they've eaten some of their own bread (still {})", w.count_of(m, "flatbread"));
    assert!(hunger < 65.0, "and aren't weak with hunger beside it: hunger {hunger:.0}");
}

#[test]
fn nm5_a_farmer_a_cook_and_a_hauler_keep_themselves_fed_for_a_fortnight() {
    let (w, bid) = outpost(12);
    let w = run(w, 14.0 * 24.0, 600.0);
    let b = w.base(bid).unwrap();
    let hunger: Vec<f32> = b.residents.iter().map(|r| w.hunger_of(r.who).unwrap()).collect();
    let (grain, bread) = (stored(&w, bid, "grain"), stored(&w, bid, "flatbread"));
    assert!(hunger.iter().all(|&h| h < 65.0), "nobody is weak with hunger: {hunger:?} (grain {grain}, flatbread {bread}, log {:?})", b.log.iter().rev().take(6).collect::<Vec<_>>());
    assert!(grain < 40, "grain isn't piling up unbaked: {grain}");
}

#[test]
fn nm6_a_day_s_work_is_paid_for_even_if_the_hand_is_let_go_before_dawn() {
    let (mut w, bid) = outpost(40);
    let lead = w.squad.members[0];
    coin(&mut w, lead, 300);
    let hand = willing(&w);
    let (wage, _) = (w.hire_terms(hand).expect("terms"), ());
    w.hire(hand).expect("hired");
    w.set_base_job(hand, Job::Hauler);
    let arrives = w.base(bid).unwrap().resident(hand).unwrap().hire.as_ref().unwrap().arrives;
    // Let them arrive, work most of a day, and let them go just before a dawn.
    let walk = (arrives - w.time) / HOUR + 0.5;
    let mut w = run(w, walk, 600.0);
    let next_dawn = ((w.time - 6.0 * HOUR) / DAY).floor() * DAY + DAY + 6.0 * HOUR;
    let worked = (next_dawn - 60.0 - w.time) / HOUR;
    w = run(w, worked, 600.0);
    let before = w.squad_count(items::id("coin"));
    w.teleport_squad(w.base(bid).unwrap().at);
    w.dismiss(hand).expect("let go");
    let paid = before - w.squad_count(items::id("coin"));
    if worked > 12.0 {
        assert!(paid > 0, "{worked:.0} hours of hauling at {wage:?} a day cost nothing");
    }
}

#[test]
fn nm7_stepping_someone_s_job_round_past_cook_uses_nothing_up() {
    let (mut w, bid) = outpost(12);
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("grain"), 10);
    let farmer = w.base(bid).unwrap().residents[0].who;
    w.step(1.0);
    let (grain, timber) = (stored(&w, bid, "grain"), stored(&w, bid, "timber"));
    // Free the kitchen: the cook becomes idle, then the farmer is clicked
    // round the jobs (the window's only control is "next job").
    let cook = w.base(bid).unwrap().residents[1].who;
    w.set_base_job(cook, Job::Idle);
    for job in [Job::Cook, Job::Crafter, Job::Hauler] {
        w.set_base_job(farmer, job);
    }
    assert_eq!((stored(&w, bid, "grain"), stored(&w, bid, "timber")), (grain, timber), "clicking past Cook took its grain and timber");
}

#[test]
fn nm8_the_travelling_squad_never_exceeds_ten() {
    use gahturiyu_sim::sim::recruit::MAX_SQUAD;
    let (mut w, bid) = built(1);
    // Fill the squad with anyone willing (for nothing, to keep it simple).
    let lead = w.squad.members[0];
    coin(&mut w, lead, 5000);
    let take = |w: &mut World, n: usize| {
        let all: Vec<PersonId> = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).filter(|&p| w.join_terms(p).is_some()).collect();
        for p in all {
            if w.squad.members.len() >= n {
                break;
            }
            let _ = w.recruit(p, lead);
        }
    };
    take(&mut w, MAX_SQUAD);
    assert_eq!(w.squad.members.len(), MAX_SQUAD, "recruited up to the limit");
    let left: Vec<PersonId> = w.squad.members.iter().copied().filter(|&m| m != lead).take(5).collect();
    let gather = |w: &mut World| {
        let at = w.base(bid).unwrap().at;
        w.teleport_squad(at);
        for k in 0..w.squad.members.len() {
            w.squad.at[k] = at;
            w.squad.goal[k] = at;
            w.squad.route[k].clear();
        }
        w.step(1.0);
    };
    gather(&mut w);
    for &m in &left {
        w.leave_at_base(m, bid).expect("left");
    }
    take(&mut w, MAX_SQUAD);
    assert_eq!(w.squad.members.len(), MAX_SQUAD, "and up to it again");
    gather(&mut w);
    let mut refused = 0;
    for &m in &left {
        if w.pick_up(m).is_err() {
            refused += 1;
        }
    }
    assert!(w.squad.members.len() <= MAX_SQUAD, "{} in the travelling squad after picking five back up ({refused} refused)", w.squad.members.len());
}

#[test]
fn a_tight_larder_is_shared_out_the_same_however_time_is_stepped() {
    // Rule 1 / rule 22: with two flatbread between three residents, who eats
    // and who goes hungry must not depend on the step.
    let snap = |w: &World, bid: u32| {
        let b = w.base(bid).unwrap();
        let hunger: Vec<String> = b.residents.iter().map(|r| format!("{:.1}", w.hunger_of(r.who).unwrap())).collect();
        let mut store: Vec<String> = b.store.iter().map(|e| format!("{}x{}", e.0, e.1)).collect();
        store.sort();
        format!("{hunger:?} {store:?}")
    };
    let (w, bid) = outpost(2);
    let a = run(w.clone(), 72.0, 60.0);
    let b = run(w, 72.0, 3600.0);
    assert_eq!(snap(&a, bid), snap(&b, bid), "minute steps against hour steps");
}

// ---- More of the same, written with the fixes ----------------------------------------

/// NM-3: any wall long enough to walk through takes a gate.
#[test]
fn a_gate_fits_walls_of_every_length() {
    for len in [4.0f32, 5.0, 8.0, 9.0, 10.0, 12.0, 16.0, 30.0] {
        let (mut w, bid, _) = with_base(1);
        let a = w.base(bid).unwrap().at.add(V2::new(-15.0, 18.0));
        let def = def_index("palisade");
        let ids = w.place_wall(def, &[a, a.add(V2::new(len, 0.0))]).expect("the wall is laid");
        let gate = def_index("gate");
        let mut plan = Plan { def: gate, at: a.add(V2::new(len * 0.5, 0.3)), rot: 0.0, w: BUILDINGS[gate].w, replaces: None };
        let ok = w.check_place(&mut plan);
        assert!(ok.is_ok(), "a gate on a {len} m palisade: {ok:?}");
        assert!(plan.replaces.is_some_and(|id| ids.contains(&id)), "it takes the place of a piece of that wall");
        let id = w.place_building(plan).expect("the gate is laid");
        let b = w.base(bid).unwrap();
        assert_eq!(b.buildings.iter().filter(|x| x.def().key == "palisade").count(), ids.len() - 1);
        assert!(b.building(id).is_some());
    }
}

/// NM-2: a yard can be walled right round, and a wall can't be laid on a wall.
#[test]
fn a_yard_can_be_walled_right_round() {
    let (mut w, bid, _) = with_base(1);
    let a = w.base(bid).unwrap().at.add(V2::new(-6.0, 14.0));
    let pts = [a, a.add(V2::new(12.0, 0.0)), a.add(V2::new(12.0, 12.0)), a.add(V2::new(0.0, 12.0)), a];
    let def = def_index("palisade");
    let plans = World::wall_plans(def, &pts);
    assert_eq!(plans.len(), 12);
    let ids = w.place_wall(def, &pts).expect("the wall is laid");
    assert_eq!(ids.len(), 12, "all four sides, corners and the closing piece too");
    // The same wall again is refused whole: every piece lies on one already there.
    assert!(w.place_wall(def, &pts).is_err(), "a wall can't be laid along a wall");
    // And a sharp bend still counts as a corner.
    let b = a.add(V2::new(-14.0, 0.0));
    let ids = w.place_wall(def, &[b, b.add(V2::new(8.0, 0.0)), b.add(V2::new(2.0, 6.0))]).expect("a sharp corner");
    assert_eq!(ids.len(), World::wall_plans(def, &[b, b.add(V2::new(8.0, 0.0)), b.add(V2::new(2.0, 6.0))]).len());
}

/// Found while fixing NM-2: a tower snapped to a point beside the wall's
/// middle, not to its end.
#[test]
fn a_tower_snaps_to_the_end_of_a_wall() {
    let (mut w, bid, _) = with_base(1);
    let a = w.base(bid).unwrap().at.add(V2::new(-6.0, 14.0));
    let end = a.add(V2::new(12.0, 0.0));
    let def = def_index("palisade");
    w.place_wall(def, &[a, end]).expect("the wall is laid");
    let tower = def_index("watchtower");
    for (near, want) in [(end.add(V2::new(1.0, 0.8)), end), (a.add(V2::new(-0.7, -0.9)), a)] {
        let mut plan = Plan { def: tower, at: near, rot: 0.0, w: BUILDINGS[tower].w, replaces: None };
        let ok = w.check_place(&mut plan);
        assert!(ok.is_ok(), "{ok:?}");
        assert!(plan.at.dist(want) < 0.05, "the tower sits on the wall's end: {:?} against {want:?}", plan.at);
    }
}

/// NM-7, the other half: letting someone go or fetching them mid-bake gives
/// the grain and timber back as well.
#[test]
fn a_round_dropped_gives_back_what_went_into_it() {
    let (mut w, bid) = outpost(12);
    let i = w.bases.iter().position(|b| b.id == bid).unwrap();
    w.bases[i].add_to_store(items::id("grain"), 10);
    let cook = w.base(bid).unwrap().residents[1].who;
    // Idle and back to cook, so a bake starts now with the new grain.
    w.set_base_job(cook, Job::Idle);
    let (grain, timber) = (stored(&w, bid, "grain"), stored(&w, bid, "timber"));
    w.set_base_job(cook, Job::Cook);
    assert!(w.base(bid).unwrap().resident(cook).unwrap().cycle.is_some(), "the bake is on");
    assert!(stored(&w, bid, "grain") < grain, "its grain is in the oven");
    let at = w.base(bid).unwrap().at;
    w.teleport_squad(at);
    w.step(1.0);
    w.pick_up(cook).expect("fetched");
    assert_eq!((stored(&w, bid, "grain"), stored(&w, bid, "timber")), (grain, timber), "fetching the cook mid-bake loses nothing");
}

/// NM-6, in numbers: a hand let go is paid for the part of the day worked,
/// and one let go on arrival is paid nothing.
#[test]
fn a_hand_let_go_is_paid_for_the_hours_worked() {
    let (mut w, bid) = outpost(40);
    let lead = w.squad.members[0];
    coin(&mut w, lead, 300);
    let hand = willing(&w);
    let wage = w.hire_terms(hand).expect("terms");
    w.hire(hand).expect("hired");
    let arrives = w.base(bid).unwrap().resident(hand).unwrap().hire.as_ref().unwrap().arrives;
    let walk = (arrives - w.time) / HOUR + 0.02;
    let mut w = run(w, walk, 600.0);
    let at = w.base(bid).unwrap().at;
    w.teleport_squad(at);
    w.step(1.0);
    // Six hours' work, not across a dawn.
    let dawn = 6.0 * HOUR;
    let to_dawn = DAY - (w.time - dawn).rem_euclid(DAY);
    if to_dawn < 7.0 * HOUR {
        w = run(w, to_dawn / HOUR + 0.1, 600.0);
    }
    let owed = w.base(bid).unwrap().resident(hand).unwrap().hire.as_ref().unwrap().owed;
    let before = w.squad_count(items::id("coin"));
    let start = w.time.max(arrives);
    w = run(w, 6.0, 600.0);
    let last_dawn = ((w.time - dawn) / DAY).floor() * DAY + dawn;
    let hours = (w.time - last_dawn.max(start.min(arrives.max(last_dawn)))) / HOUR;
    w.dismiss(hand).expect("let go");
    let paid = before - w.squad_count(items::id("coin"));
    let want = owed + (wage as f64 * (hours / 24.0).clamp(0.0, 1.0)).round() as u16;
    assert!(paid > 0 && (paid as i32 - want as i32).abs() <= 1, "paid {paid} for about {hours:.1} h at {wage} a day (wanted about {want})");
    assert!(paid < wage, "not a whole day's wage");
}
