//! Animals: wildlife, livestock, and where they meet people.
//!
//! The first two tests are the animals' share of the promises in
//! `consistency.rs`: the same seed gives the same herds, attacks and
//! outcomes however the world is stepped and wherever the squad stands.
//! The rest check each piece does what it says.

use gahturiyu_sim::sim::{
    animals::*,
    fights::CORPSE_TIME,
    geo::V2,
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(mut w: World, hours: f64, step: f64) -> World {
    let steps = (hours * HOUR / step).round() as usize;
    for _ in 0..steps {
        w.step(step);
    }
    w
}

fn step_to(w: &mut World, t: f64) {
    while w.time < t {
        w.step(60.0f64.min(t - w.time).max(0.01));
    }
}

/// Everything about the animals that counts as "what happened". Left out
/// on purpose: which herds the squad has seen up close, the list of herds
/// near it, and herds running from it — those follow the squad, and decide
/// nothing.
fn history(w: &World) -> String {
    let t = w.time;
    let a = &w.animals;
    let mut s = String::new();
    for h in &a.herds {
        s += &format!("{} {:?} {} {:?} {:?} {:?} {:?} {:?} {:?} {:?}\n", h.id, h.sp, h.alive(t), h.n, h.at, h.ready_at, h.hunt, h.idents, h.hurt, h.round_pos(t));
    }
    s += &format!("{:?}\n{:?}\n", a.pending, a.frays);
    s += &format!("{:?}\n", a.attacks);
    let st = &a.stats;
    s += &format!("{} {} {} {} {} {}\n", st.attacks, st.left_alone, st.people_downed, st.people_killed, st.animals_killed, st.bodies_cleared);
    for p in &a.pens {
        s += &format!("{:?}\n", p.now(t));
    }
    s += &format!("{:?}\n{:?}\n{:?}\n{:?}\n", a.colonies, a.carcasses, a.cleared, a.fish);
    s
}

/// What became of everybody else: who is dead, how hurt each person is,
/// and where every party on the road is and is going.
fn everyone(w: &World) -> String {
    let t = w.time;
    let mut s = String::new();
    for p in &w.people {
        s += &format!("{} {} {:?}\n", p.id, p.dead, p.wounds.lost_at(t));
    }
    for g in &w.groups {
        s += &format!("{} {:?} {:?} {:?} {:?}\n", g.id, g.members, g.position_at(t), g.ends, g.cargo);
    }
    s
}

fn assert_same(a: &World, b: &World, what: &str) {
    let (ha, hb) = (history(a), history(b));
    if ha != hb {
        let line = ha.lines().zip(hb.lines()).position(|(x, y)| x != y).unwrap_or(0);
        panic!("{what}: the animals went their own way (first difference at line {line}):\n{}\n{}", ha.lines().nth(line).unwrap_or(""), hb.lines().nth(line).unwrap_or(""));
    }
}

#[test]
fn step_size_does_not_change_what_the_animals_do() {
    // (45 hours is a whole number of every one of these steps, so all three
    // worlds stop at the same moment.)
    let fine = run(worldgen::generate(7), 45.0, 2.0);
    let coarse = run(worldgen::generate(7), 45.0, HOUR);
    let lumpy = run(worldgen::generate(7), 45.0, 337.5);
    assert!(fine.time == coarse.time && fine.time == lumpy.time);
    assert_same(&fine, &coarse, "2 s steps against hour steps");
    assert_same(&fine, &lumpy, "2 s steps against lumpy steps");
    // The people they met fared the same, and are in the same places.
    assert!(everyone(&fine) == everyone(&coarse), "2 s steps against hour steps: the travellers fared differently");
    assert!(everyone(&fine) == everyone(&lumpy), "2 s steps against lumpy steps: the travellers fared differently");
    assert!(fine.animals.stats.attacks > 0, "the animals should actually be doing something");
    assert!(fine.animals.stats.left_alone > 0, "and thinking better of it sometimes");
    // Every attack is a fight that was really fought, and is over.
    for at in fine.animals.attacks.iter().filter(|a| a.t < fine.time - HOUR) {
        assert!(at.over, "a fight begun at {} never ended", at.t);
    }
}

#[test]
fn where_the_squad_stands_does_not_change_what_the_animals_do() {
    // Far from everything: nothing that happens is anywhere near the squad.
    let far = V2::new(18_000.0, 18_000.0);
    let mut elsewhere = worldgen::generate(11);
    elsewhere.teleport_squad(far);
    let elsewhere = run(elsewhere, 36.0, 2.0);
    // Now the same world with the squad stood a few hundred metres from
    // where a pack falls on travellers, watching: the whole thing (the
    // stalk, the strike, the fight) happens in the nearest band, drawn and
    // played out blow by blow, with the pack inside the squad's own
    // looking-over. (A full, fit squad: the pack doesn't fancy it.)
    let attack = elsewhere.animals.attacks.iter().find(|a| a.t > 2.0 * HOUR && a.sp == Sp::Ridgehound && a.pack >= 2).expect("a pack attack to go and watch").clone();
    let spot = [V2::new(280.0, 0.0), V2::new(-280.0, 0.0), V2::new(0.0, 280.0), V2::new(0.0, -280.0)]
        .iter()
        .map(|o| attack.at.add(*o))
        .find(|p| gahturiyu_sim::sim::geo::is_land(*p) && elsewhere.settlements.iter().all(|s| s.pos.dist(*p) > s.radius() + 200.0))
        .expect("somewhere to stand");
    let mut here = worldgen::generate(11);
    here.teleport_squad(spot);
    let mut seen_fighting = false;
    let steps = (36.0 * HOUR / 2.0).round() as usize;
    for _ in 0..steps {
        here.step(2.0);
        assert!(here.squad_battle().is_none(), "the squad was only meant to watch (at {})", here.time);
        seen_fighting |= here.animals_fighting().iter().any(|f| f.pos.dist(spot) < 500.0);
    }
    assert!(seen_fighting, "the attack never played out in front of the squad");
    assert!(here.animals.herds[attack.herd as usize].met, "the pack was never close enough to be seen");
    assert_same(&here, &elsewhere, "squad watching an attack against squad in the far corner");
    assert!(everyone(&here) == everyone(&elsewhere), "the travellers fared differently for being watched");
}

#[test]
fn a_loaded_world_keeps_its_animals_and_they_carry_on_the_same() {
    // Save, load, and run both on: they must stay the same world.
    fn check(w: &World, what: &str) {
        let mut a = w.clone_by_save();
        let mut b = w.clone_by_save();
        assert_eq!(format!("{:?}", w.animals), format!("{:?}", a.animals), "{what}: the animals didn't come back as they were saved");
        for _ in 0..(6.0 * HOUR / 60.0) as usize {
            a.step(60.0);
        }
        // (The second copy is saved and loaded once more on the way.)
        for _ in 0..(3.0 * HOUR / 60.0) as usize {
            b.step(60.0);
        }
        let mut b = b.clone_by_save();
        for _ in 0..(3.0 * HOUR / 60.0) as usize {
            b.step(60.0);
        }
        assert_eq!(format!("{:?}", a.animals), format!("{:?}", b.animals), "{what}: the loaded world's animals went their own way");
        assert!(everyone(&a) == everyone(&b), "{what}: the loaded world's people went their own way");
    }
    trait Resave {
        fn clone_by_save(&self) -> World;
    }
    impl Resave for World {
        fn clone_by_save(&self) -> World {
            World::load_bytes(&self.save_bytes()).expect("loads")
        }
    }
    let mut w = run(worldgen::generate(3), 20.0, 60.0);
    check(&w, "an ordinary moment");
    // In the middle of a stalk: a pack has picked travellers up and its
    // strike is still to come.
    let mut tries = 0;
    while !w.animals.herds.iter().any(|h| h.hunt.is_some()) {
        w.step(60.0);
        tries += 1;
        assert!(tries < 6 * 24 * 60, "six days and no pack ever stalked anyone");
    }
    assert!(w.animals.pending.iter().any(|d| d.what == What::Strike));
    check(&w, "mid-stalk");
    // In the middle of a fight far from the squad: fought already, its
    // ending still to come.
    let mut tries = 0;
    while !w.animals.frays.iter().any(|f| f.ends.is_some()) {
        w.step(2.0);
        tries += 1;
        assert!(tries < 6 * 24 * 1800, "six days and no pack ever struck");
    }
    assert!(w.animals.pending.iter().any(|d| d.what == What::FrayEnd) && !w.npc_fights.is_empty());
    check(&w, "mid-fight");
}

#[test]
fn a_new_world_is_stocked() {
    for seed in [1, 2, 3] {
        let w = worldgen::generate(seed);
        let census = w.wildlife_census();
        let count = |sp: Sp| census.iter().find(|c| c.0 == sp).map(|c| c.1).unwrap_or(0.0);
        for sp in [Sp::WildTuriyu, Sp::Brushleaper, Sp::Wallowback, Sp::CragGrazer, Sp::Dustrunner, Sp::Tidepicker, Sp::Silkcrawler, Sp::Ridgehound, Sp::Mirejaw, Sp::SilkMother] {
            assert!(count(sp) > 0.0, "seed {seed}: no {} anywhere", sp.name());
        }
        assert!(count(Sp::Cragmaw) <= 2.0, "Cragmaws are very rare");
        assert_eq!(count(Sp::SilkMother) as usize, w.colonies().len(), "one Silk Mother per colony");
        // Region numbers are the sums of the herds.
        for sp in [Sp::Brushleaper, Sp::Ridgehound] {
            let by_region: f32 = (0..w.animals.regions.len() as u16).map(|r| w.population(r, sp)).sum();
            assert_eq!(by_region, count(sp));
        }
        // Animals live where their country is.
        for h in &w.animals.herds {
            assert_eq!(region_of(h.home), h.region);
            match h.sp {
                Sp::Dustrunner => assert!(w.terrain.plateau(h.home) > 0.4, "a Dustrunner flock off the plateau"),
                Sp::Tidepicker => assert!(gahturiyu_sim::sim::geo::inland(h.home) < 120.0, "Tidepickers inland"),
                Sp::Cragmaw => assert!(h.home.dist(MASSIF) < MASSIF_RADIUS + 200.0, "a Cragmaw off the massif"),
                _ => {}
            }
            // Nothing dangerous makes its home on top of a town.
            if h.def().toward.dangerous() {
                assert!(w.settlements.iter().all(|s| s.pos.dist(h.home) > s.radius() + 300.0), "a {} lairs at a town's edge", h.def().name);
            }
        }
        // Every town keeps animals, and every pen is somebody's town's.
        for s in &w.settlements {
            let pens = w.pens_of_town(s.id);
            assert!(!pens.is_empty(), "{} keeps no animals", s.name);
            for id in pens {
                assert!(w.pen_now(id).unwrap().count >= 1.0);
            }
        }
        let (wild, kept) = w.animal_count();
        assert!(wild > 1000 && kept > 500, "seed {seed}: {wild} wild, {kept} kept");
    }
}

#[test]
fn killing_prey_thins_a_region_and_left_alone_it_recovers() {
    let mut w = worldgen::generate(5);
    let sp = Sp::Brushleaper;
    let r = (0..w.animals.regions.len() as u16).max_by(|&a, &b| w.population(a, sp).total_cmp(&w.population(b, sp))).unwrap();
    let before = w.population(r, sp);
    assert!(before >= 6.0, "the best Brushleaper country only holds {before}");
    let herds: Vec<u32> = w.animals.regions[r as usize].herds.iter().copied().filter(|&h| w.animals.herds[h as usize].sp == sp).collect();
    let mut taken = Vec::new();
    for &h in &herds {
        let n = w.animals.herds[h as usize].alive(w.time);
        taken.extend(w.take_animals(h, n.saturating_sub(2)));
    }
    let after = w.population(r, sp);
    assert!(after < before * 0.7, "hunting barely dented them: {before} -> {after}");
    assert!(taken.iter().any(|(k, a)| *k == "meat" && *a > 0.0), "the kills gave nothing: {taken:?}");
    // The game_near question sees the thinned herds.
    let home = w.animals.herds[herds[0] as usize].home;
    let near = w.game_near(home, 3000.0);
    assert!(near.iter().any(|g| g.herd == herds[0] && g.count == w.animals.herds[herds[0] as usize].alive(w.time)));
    // Left alone for a couple of months.
    let later = run(w, 60.0 * 24.0, HOUR);
    let healed = later.population(r, sp);
    assert!(healed > after + 1.0, "left alone they should recover: {after} -> {healed}");
    assert!(healed <= later.capacity(r, sp), "but never past what the country holds");
}

#[test]
fn a_pack_goes_for_the_lone_and_weak_and_leaves_the_strong_alone() {
    let w = worldgen::generate(7);
    let t = w.time;
    let pack = w.animals.herds.iter().filter(|h| h.sp == Sp::Ridgehound && h.alive(t) >= 4).min_by(|a, b| a.id.cmp(&b.id)).expect("a pack of four or more").id;
    let mut folk: Vec<u32> = w.people.iter().filter(|p| !p.in_squad && !p.bandit).map(|p| p.id).collect();
    folk.sort_by(|&a, &b| w.people[a as usize].might.total_cmp(&w.people[b as usize].might));
    let weak = [folk[folk.len() / 10]];
    let strong: Vec<u32> = folk.iter().rev().take(6).copied().collect();
    for k in 0..=20 {
        let roll = k as f32 / 20.0;
        assert!(w.herd_would_attack(pack, &weak, roll), "a lone weak traveller got past the pack at roll {roll}");
        assert!(!w.herd_would_attack(pack, &strong, roll), "the pack went for six strong people at roll {roll}");
    }
    // Nobody at all: nothing to attack.
    assert!(!w.herd_would_attack(pack, &[], 0.5));

    // And out in the world, over a few nights: packs do attack, they do
    // think better of it, and every attack was on someone they outmatched.
    let w = run(w, 4.0 * 24.0, 300.0);
    let st = &w.animals.stats;
    assert!(st.attacks > 3 && st.left_alone > 3, "{st:?}");
    let by_hounds: Vec<&AttackRecord> = w.animals.attacks.iter().filter(|a| a.sp == Sp::Ridgehound).collect();
    assert!(!by_hounds.is_empty());
    for a in by_hounds {
        assert!(a.pack_might * 1.1 > a.their_might * PACK_CAUTION, "a pack of might {} attacked people of might {}", a.pack_might, a.their_might);
        // They come out at dusk and hunt by night.
        assert!(Active::DuskNight.at(a.t), "a pack attacked in daylight at {}", a.t);
    }
}

#[test]
fn a_pack_leaves_a_strong_squad_alone_but_falls_on_a_lone_survivor() {
    let mut w = worldgen::generate(7);
    let pack = w.animals.herds.iter().filter(|h| h.sp == Sp::Ridgehound).max_by(|a, b| a.alive(0.0).cmp(&b.alive(0.0)).then(b.id.cmp(&a.id))).unwrap().id;
    // A night it's in the mood for people, well after dark.
    let mut night = 22.0 * HOUR;
    while !bold_tonight(w.seed, pack, night) {
        night += DAY;
    }
    step_to(&mut w, night);
    let stand_by = |w: &mut World| {
        let at = w.herd_pos(pack, w.time);
        w.teleport_squad(at.add(V2::new(30.0, 0.0)));
    };
    // The whole squad: the pack keeps away.
    stand_by(&mut w);
    for _ in 0..240 {
        w.step(1.0);
    }
    assert!(w.squad_battle().is_none(), "the pack went for four armed people");
    assert_eq!(w.animals.stats.attacks, w.animals.attacks.iter().filter(|a| a.victim.is_some()).count() as u32, "no attack on the squad should be on record");

    // Three of them down: the pack closes on the one left standing. (Hungry
    // again, whatever it has caught lately.)
    w.animals.herds[pack as usize].ready_at = 0.0;
    let t = w.time;
    for &m in &w.squad.members.clone()[1..] {
        let p = &mut w.people[m as usize];
        p.wounds.lost[1] = p.stats.max_hp(gahturiyu_sim::sim::body::Part::Torso) + 30.0;
        p.wounds.at = t;
    }
    assert_eq!(w.squad_fit().len(), 1);
    let mut attacked = false;
    'night: for _ in 0..5 {
        // (Its nerve is rolled afresh each hour.)
        let lone = [w.squad.members[0]];
        let roll_ok = (0..=10).any(|k| w.herd_would_attack(pack, &lone, k as f32 / 10.0));
        assert!(roll_ok, "this pack would never attack one person");
        stand_by(&mut w);
        for _ in 0..600 {
            w.step(1.0);
            if w.squad_battle().is_some() {
                attacked = true;
                break 'night;
            }
        }
        if !Active::DuskNight.at(w.time + HOUR) {
            break;
        }
        let next_hour = (w.time / HOUR).floor() * HOUR + HOUR + 1.0;
        step_to(&mut w, next_hour);
    }
    assert!(attacked, "the pack never came for a lone, wounded squad");
    assert!(w.animals_fighting().iter().any(|f| f.sp == Sp::Ridgehound && !f.tame));
    assert!(w.animals.attacks.iter().any(|a| a.victim.is_none() && a.herd == pack));
}

#[test]
fn turiyu_graze_and_briarbacks_follow_the_overgrowth() {
    let mut w = worldgen::generate(4);
    let r = (0..w.animals.regions.len() as u16).max_by(|&a, &b| w.grazing_pressure(a).total_cmp(&w.grazing_pressure(b))).unwrap();
    let before = w.grazing_pressure(r);
    assert!(before > 0.0, "nothing grazes anywhere");
    // Take every Turiyu out of the region, wild and kept.
    let wild: Vec<u32> = w.animals.regions[r as usize].herds.iter().copied().filter(|&h| w.animals.herds[h as usize].sp == Sp::WildTuriyu).collect();
    for (k, &h) in wild.iter().enumerate() {
        w.take_animals(h, 100);
        if k == 0 {
            let part = w.grazing_pressure(r);
            assert!(part < before, "pressure should fall as they go: {before} -> {part}");
        }
    }
    let pens: Vec<u32> = w.animals.pens.iter().filter(|p| p.sp == Sp::Turiyu && p.region == r).map(|p| p.id).collect();
    for p in pens {
        while !w.cull_pen(p).is_empty() {}
    }
    assert_eq!(w.grazing_pressure(r), 0.0, "with no Turiyu there is no grazing");

    // The Overgrowth: nothing by default, nothing below the threshold.
    assert_eq!(w.overgrowth(r), 0.0);
    assert_eq!(w.population(r, Sp::Briarback), 0.0);
    w.set_overgrowth(r, BRIAR_THRESHOLD - 0.05);
    assert_eq!(w.population(r, Sp::Briarback), 0.0, "Briarbacks below the threshold");
    // Across it: they appear, and more the higher it stands.
    w.set_overgrowth(r, BRIAR_THRESHOLD + 0.05);
    let some = w.population(r, Sp::Briarback);
    assert!(some >= 2.0, "no Briarbacks above the threshold");
    w.set_overgrowth(r, 1.0);
    let many = w.population(r, Sp::Briarback);
    assert!(many > some, "a higher Overgrowth should breed more: {some} -> {many}");
    let briars: Vec<&Herd> = w.animals.herds.iter().filter(|h| h.sp == Sp::Briarback).collect();
    assert!(briars.iter().all(|h| h.region == r && region_of(h.home) == r), "Briarbacks outside the overgrown region");
    // It recedes: they die back.
    w.set_overgrowth(r, 0.1);
    let w = run(w, 30.0 * 24.0, HOUR);
    assert_eq!(w.population(r, Sp::Briarback), 0.0, "Briarbacks outlived the Overgrowth");
}

#[test]
fn bonepickers_clear_a_body_by_day_and_not_by_night() {
    let mut w = worldgen::generate(9);
    step_to(&mut w, 10.0 * HOUR);
    // Someone dies out in the open.
    let pid = w.people.iter().find(|p| !p.in_squad && !p.bandit && p.home.is_some()).unwrap().id;
    let at = w.squad.pos.add(V2::new(900.0, 300.0));
    let died = w.time;
    w.people[pid as usize].dead = true;
    let race = w.people[pid as usize].race;
    w.corpses.push((at, race, died, pid));
    let cleared = w.body_cleared_at(pid).expect("by day the birds come");
    assert!(cleared > died + (BONE_ARRIVE.0 + BONE_PICK) * 60.0 - 1.0 && cleared < died + (BONE_ARRIVE.1 + BONE_PICK) * 60.0 + 1.0);
    assert!(cleared < died + CORPSE_TIME, "they should beat the rot to it");
    assert!(w.body_there(pid) && w.flocks().is_empty());
    // They arrive...
    let arrive = cleared - BONE_PICK * 60.0;
    step_to(&mut w, arrive + 60.0);
    let flocks = w.flocks();
    assert_eq!(flocks.len(), 1, "a flock should be down on the body");
    assert!(flocks[0].0.dist(at) < 0.1 && flocks[0].1 >= 5);
    assert!(w.body_there(pid), "the body went before they'd finished");
    // ...and when they've finished, there's nothing left to find (or raise).
    step_to(&mut w, cleared + 1.0);
    assert!(!w.body_there(pid), "the body is still there after the birds have done");
    assert!(w.flocks().is_empty());
    assert_eq!(w.animals.stats.bodies_cleared, 1);
    assert_eq!(w.body_cleared_at(pid), Some(cleared), "others can still ask when it was cleared");

    // After dark nothing comes: the body lies until it rots.
    step_to(&mut w, 23.0 * HOUR);
    let other = w.people.iter().find(|p| !p.in_squad && !p.bandit && !p.dead && p.home.is_some()).unwrap().id;
    w.people[other as usize].dead = true;
    let t = w.time;
    w.corpses.push((at, race, t, other));
    assert_eq!(w.body_cleared_at(other), None);
    step_to(&mut w, t + CORPSE_TIME - 120.0);
    assert!(w.body_there(other));
    assert_eq!(w.animals.stats.bodies_cleared, 1);
}

#[test]
fn taking_cocoons_under_the_silk_mothers_eye_starts_a_fight() {
    let mut w = worldgen::generate(6);
    let (id, pos, mother) = {
        let c = &w.colonies()[0];
        (c.id, c.pos, c.mother)
    };
    // A time she's at home.
    while !w.mother_guarding(id) {
        w.step(600.0);
    }
    let had = w.cocoons(id);
    assert!(had > 0, "a colony with nothing hanging");
    let who = w.squad.members[0];
    // From across the valley you can't reach them.
    assert_eq!(w.take_cocoons(who, id), Take::TooFar);
    // Stand the squad off, and send one of them in.
    w.teleport_squad(pos.add(V2::new(60.0, 0.0)));
    w.squad.at[0] = pos.add(V2::new(2.0, 0.0));
    let got = w.take_cocoons(who, id);
    let Take::Fight(battle) = got else { panic!("she let them be taken: {got:?}") };
    assert_eq!(w.squad_battle().map(|b| b.id), Some(battle));
    assert_eq!(w.cocoons(id), had, "nothing is taken when she attacks");
    assert!(w.animals_fighting().iter().any(|f| f.sp == Sp::SilkMother));
    // While she's fighting she is still very much there: a second pair of
    // hands gets nothing either (and doesn't start a second fight).
    let second = w.squad.members[1];
    let b = w.battles.iter_mut().find(|b| b.id == battle).unwrap();
    b.fighters.iter_mut().find(|f| f.pid == second).unwrap().pos = pos.add(V2::new(-2.0, 0.0));
    assert!(w.mother_guarding(id));
    assert_eq!(w.take_cocoons(second, id), Take::Guarded);
    assert_eq!(w.cocoons(id), had);
    // Someone who isn't there can't reach them at all.
    let stranger = w.people.iter().find(|p| !p.in_squad).unwrap().id;
    assert_eq!(w.take_cocoons(stranger, id), Take::TooFar);

    // With her dead, they're there for the taking; then they grow back.
    let mut w = worldgen::generate(6);
    while !w.mother_guarding(id) {
        w.step(600.0);
    }
    w.take_animals(mother, 1);
    assert!(!w.mother_guarding(id));
    w.teleport_squad(pos.add(V2::new(60.0, 0.0)));
    w.squad.at[0] = pos.add(V2::new(2.0, 0.0));
    let had = w.cocoons(id);
    assert_eq!(w.take_cocoons(who, id), Take::Taken(had));
    assert_eq!(w.cocoons(id), 0);
    assert_eq!(w.take_cocoons(who, id), Take::Empty);
    assert!(w.squad_battle().is_none());
    let t = w.time;
    w.teleport_squad(V2::new(9_000.0, 4_000.0));
    step_to(&mut w, t + 3.0 * DAY);
    let grown = w.cocoons(id);
    assert!(grown > 5 && grown as f32 <= COCOON_CAP, "three days of spinning gave {grown}");

    // While she's off hunting, too.
    let mut w = worldgen::generate(6);
    let mut t = w.time;
    while !w.animals.herds[mother as usize].mother_out(t) {
        t += 600.0;
    }
    step_to(&mut w, t + 60.0);
    assert!(!w.mother_guarding(id));
    w.teleport_squad(pos.add(V2::new(60.0, 0.0)));
    w.squad.at[0] = pos.add(V2::new(2.0, 0.0));
    assert!(matches!(w.take_cocoons(who, id), Take::Taken(n) if n > 0));
}

#[test]
fn a_hungry_pen_yields_less_than_a_fed_one() {
    let mut w = worldgen::generate(2);
    let at = w.squad.pos.add(V2::new(3_000.0, 0.0));
    let fed = w.add_pen(Sp::Shellhen, at, None, 12.0, 12.0);
    let hungry = w.add_pen(Sp::Shellhen, at.add(V2::new(40.0, 0.0)), None, 12.0, 12.0);
    assert!(w.feed_pen(fed, 500.0));
    let since = w.time;
    let w = run(w, 6.0 * 24.0, HOUR);
    let eggs = |id: u32| w.pen_yield_since(id, since).iter().find(|y| y.0 == "egg").map(|y| y.1).unwrap();
    let (a, b) = (eggs(fed), eggs(hungry));
    assert!((a - 72.0).abs() < 2.0, "twelve fed hens over six days laid {a}");
    assert!(b < a * 0.7, "the hungry pen laid {b} against the fed pen's {a}");
    let (fa, fb) = (w.pen_now(fed).unwrap(), w.pen_now(hungry).unwrap());
    assert!(fa.hunger < 0.05 && fb.hunger > 0.5, "hunger: fed {} hungry {}", fa.hunger, fb.hunger);
    assert!(fa.feed < 500.0, "the fed pen should have eaten into its store");
    // One animal's share is the pen's divided by its head.
    let one = w.animal_yield_since(fed, since)[0].1;
    assert!((one * fa.count.floor() - a).abs() < 0.5);
    // Yield since a later morning is less than since the start.
    assert!(w.pen_yield_since(fed, since + 3.0 * DAY)[0].1 < a - 20.0);
}

#[test]
fn pens_have_owners_grow_to_their_limit_and_can_be_culled() {
    let mut w = worldgen::generate(2);
    let town = w.settlements[0].id;
    let pens = w.pens_of_town(town);
    assert!(pens.len() >= 2);
    // Owners are whatever number society gives them.
    assert!(w.pens_of_owner(42).is_empty());
    w.set_pen_owner(pens[0], 42);
    assert_eq!(w.pens_of_owner(42), vec![pens[0]]);
    // A cull gives what the species gives and takes one from the pen.
    let turiyu = w.animals.pens.iter().find(|p| p.sp == Sp::Turiyu).unwrap().id;
    let before = w.pen_now(turiyu).unwrap().count.floor();
    let got = w.cull_pen(turiyu);
    assert!(got.iter().any(|y| y.0 == "meat"));
    assert_eq!(w.pen_now(turiyu).unwrap().count.floor(), before - 1.0);
    // Fed and given room, a herd grows — up to the limit it's given.
    w.set_pen_limit(turiyu, before + 3.0);
    w.feed_pen(turiyu, 10_000.0);
    // (A pen's state at any later moment is worked out, not counted up to,
    // so a year ahead can simply be asked for.)
    let year = w.pen(turiyu).unwrap().now(w.time + 400.0 * DAY).count.floor();
    assert_eq!(year, before + 3.0, "the pen should grow to its limit");
    // And living through some of it day by day comes to the same thing.
    let ahead = w.pen(turiyu).unwrap().now(w.time + 12.0 * DAY).count;
    let mut w = run(w, 12.0 * 24.0, HOUR);
    let lived = w.pen_now(turiyu).unwrap().count;
    assert!((lived - ahead).abs() < 0.02 && lived > before - 1.0, "worked out {ahead}, lived {lived}");
    // Pack animals: one can be led, and carries what the table says.
    let stable = w.animals.pens.iter().find(|p| p.sp == Sp::Plodder).unwrap().id;
    let leader = w.squad.members[0];
    assert!(w.plodder_capacity() > 50.0);
    assert!(w.lead_plodder(stable, leader));
    assert!(w.led_pos(0).unwrap().dist(w.person_pos(leader)) < 5.0, "it should be at its leader's shoulder");
    assert!(!w.lead_plodder(turiyu, leader), "a Turiyu is no pack animal");
    w.release_plodders(leader);
    assert!(w.animals.led.is_empty());
}

#[test]
fn a_hound_that_is_down_can_be_tamed_and_then_follows_fights_and_hungers() {
    let mut w = worldgen::generate(7);
    step_to(&mut w, 10.0 * HOUR);
    let t = w.time;
    let pack = w.animals.herds.iter().find(|h| h.sp == Sp::Ridgehound && h.alive(t) >= 3 && !h.young(0, t)).unwrap().id;
    let who = w.squad.members[0];
    // A grown hound on its feet won't be tamed.
    let at = w.animal_pos(pack, 0, t);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    assert!(!w.tameable(pack, 0));
    assert!(w.tame(who, pack, 0).is_err());
    // Knock it out (as a fight would leave it).
    let max = max_hp(Sp::Ridgehound, w.animals.herds[pack as usize].size(0, t));
    w.animals.herds[pack as usize].set_hurt(0, [0.0, max[1] + 5.0, 0.0, 0.0, 0.0, 0.0], t);
    assert!(w.animals.herds[pack as usize].down(0, t) && w.tameable(pack, 0));
    let before = w.animals.herds[pack as usize].alive(t);
    // The check is a keyed roll for now: try until it takes.
    let mut hound = None;
    for _ in 0..40 {
        if let Ok(id) = w.tame(who, pack, 0) {
            hound = Some(id);
            break;
        }
    }
    let hound = hound.expect("forty tries at better than even odds");
    assert_eq!(w.animals.herds[pack as usize].alive(t), before - 1, "it should have left its pack");
    assert_eq!(w.tamed().len(), 1);
    // It follows its owner.
    w.teleport_squad(V2::new(9_000.0, 9_000.0));
    assert!(w.hound_pos(hound).unwrap().dist(w.person_pos(who)) < 4.0);
    // It gets hungry; feeding helps.
    let h0 = w.hound_hunger(hound).unwrap();
    step_to(&mut w, t + 20.0 * HOUR);
    let h1 = w.hound_hunger(hound).unwrap();
    assert!(h1 > h0 + 0.3, "{h0} -> {h1}");
    assert!(w.feed_hound(hound, 1.0));
    assert!(w.hound_hunger(hound).unwrap() < h1 - 0.3);
    // Healed by now, it fights beside its owner.
    let at = w.squad.pos.add(V2::new(12.0, 4.0));
    w.spawn_bandits(at, 2, false);
    let mut joined = false;
    for _ in 0..2000 {
        w.step(0.25);
        if w.animals_fighting().iter().any(|f| f.tame) {
            joined = true;
            break;
        }
    }
    assert!(joined, "the hound stayed out of its owner's fight");
    let b = w.squad_battle().expect("the squad is fighting");
    assert!(b.fighters.iter().any(|f| !f.is_person() && f.side == gahturiyu_sim::sim::combat::SQUAD_SIDE), "the hound should be on the squad's side");
    // Left unfed for long enough, it goes back to the hills.
    for _ in 0..4000 {
        if w.squad_battle().is_none() {
            break;
        }
        w.step(0.25);
    }
    if !w.tamed().is_empty() {
        let w = run(w, 8.0 * 24.0, HOUR);
        assert!(w.tamed().is_empty(), "a starved hound should have left");
    }
}

#[test]
fn the_squad_can_hunt_and_prey_runs() {
    let mut w = worldgen::generate(8);
    step_to(&mut w, 9.0 * HOUR);
    let t = w.time;
    // Wallowbacks stand and fight; the squad's hunt is a real fight.
    let herd = w.animals.herds.iter().find(|h| h.sp == Sp::Wallowback && h.alive(t) >= 4).unwrap().id;
    let who = w.squad.members.clone();
    assert!(!w.hunt(&who, herd), "nothing to set about from the far side of the world");
    let at = w.herd_pos(herd, t);
    w.teleport_squad(at.add(V2::new(10.0, 0.0)));
    assert_eq!(w.herd_near(w.squad.pos, 60.0), Some(herd));
    assert!(w.hunt(&who, herd));
    let b = w.squad_battle().expect("a fight");
    let animals = b.fighters.iter().filter(|f| !f.is_person()).count();
    assert!(animals >= 1 && animals <= CORNERED, "{animals} Wallowbacks turned to fight");
    let before = w.animals.herds[herd as usize].alive(t);
    for _ in 0..6000 {
        w.step(0.2);
        if w.squad_battle().is_none() {
            break;
        }
    }
    assert!(w.squad_battle().is_none(), "the hunt never ended");
    let after = w.animals.herds[herd as usize].alive(w.time);
    let carcasses = w.carcasses_near(w.squad.pos, 300.0).len();
    assert_eq!(before - after, carcasses, "every animal killed should lie where it fell");
    if let Some(id) = w.carcasses_near(w.squad.pos, 300.0).first().map(|c| c.id) {
        let got = w.butcher(id);
        assert!(got.iter().any(|y| y.0 == "meat" && y.1 > 5.0), "{got:?}");
        assert_eq!(w.carcasses_near(w.squad.pos, 300.0).len(), carcasses - 1);
    }
    assert_eq!(w.animals.stats.animals_killed as usize, before - after);
    let rec = w.animals.attacks.last().unwrap();
    assert!(rec.by_squad && rec.over && rec.victim.is_none(), "{rec:?}");
    assert_eq!(w.animals.stats.attacks, 0, "a hunt is not an attack by animals");

    // Small game, crept up on: the squad actually brings some down, and
    // what it brings down lies there to be butchered.
    let mut w = worldgen::generate(8);
    step_to(&mut w, 9.0 * HOUR);
    let t = w.time;
    let herd = w.animals.herds.iter().filter(|h| h.sp == Sp::Dustrunner && h.alive(t) >= 6).min_by_key(|h| h.id).expect("a Dustrunner flock").id;
    let who = w.squad.members.clone();
    for &m in &who {
        w.set_sneaking(m, true);
    }
    let at = w.herd_pos(herd, t);
    w.teleport_squad(at.add(V2::new(3.0, 0.0)));
    let before = w.animals.herds[herd as usize].alive(t);
    assert!(w.hunt(&who, herd));
    assert!(!w.hunt(&who, herd), "a herd can't be set about twice at once");
    for _ in 0..6000 {
        w.step(0.2);
        if w.squad_battle().is_none() {
            break;
        }
    }
    assert!(w.squad_battle().is_none(), "the hunt never ended");
    let after = w.animals.herds[herd as usize].alive(w.time);
    let rec = w.animals.attacks.last().unwrap().clone();
    assert!(rec.by_squad && rec.over);
    assert!(before > after, "four hunters falling on a flock unawares killed nothing");
    assert_eq!((before - after) as u8, rec.animals_lost);
    let lying = w.carcasses_near(at, 400.0);
    assert_eq!(lying.len(), before - after, "every kill should leave a carcass");
    assert!(lying.iter().all(|c| c.sp == Sp::Dustrunner));
    assert_eq!(w.animals.stats.people_downed, 0, "nobody is brought down by Dustrunners");
    // Nothing of the flock that got away is left lying down for good:
    // whatever was dropped and not killed is up again within a day or so.
    let t = w.time;
    assert!(w.animals.herds[herd as usize].all_up_at(t) < t + 3.0 * DAY);

    // An animal left down can be finished off by someone standing over it.
    let mut w = worldgen::generate(8);
    step_to(&mut w, 9.0 * HOUR);
    let t = w.time;
    let pack = w.animals.herds.iter().find(|h| h.sp == Sp::Ridgehound && h.alive(t) >= 3).unwrap().id;
    let max = max_hp(Sp::Ridgehound, w.animals.herds[pack as usize].size(1, t));
    let who = w.squad.members[0];
    w.teleport_squad(w.animal_pos(pack, 1, t).add(V2::new(1.0, 0.0)));
    assert!(!w.finish_off(who, pack, 1), "it is on its feet");
    w.animals.herds[pack as usize].set_hurt(1, [0.0, max[1] + 5.0, 0.0, 0.0, 0.0, 0.0], t);
    let before = w.animals.herds[pack as usize].alive(t);
    w.teleport_squad(w.animal_pos(pack, 1, t).add(V2::new(1.0, 0.0)));
    assert!(w.finish_off(who, pack, 1));
    assert_eq!(w.animals.herds[pack as usize].alive(t), before - 1);
    assert_eq!(w.carcasses_near(w.squad.pos, 30.0).len(), 1);

    // Brushleapers bolt before the squad is anywhere near.
    let mut w = worldgen::generate(8);
    let herd = w.animals.herds.iter().find(|h| h.sp == Sp::Brushleaper && h.alive(0.0) >= 3).unwrap().id;
    let mut t = w.time;
    while !w.animals.herds[herd as usize].active(t) {
        t += 600.0;
    }
    step_to(&mut w, t + 60.0);
    let at = w.herd_pos(herd, w.time);
    w.teleport_squad(at.add(V2::new(50.0, 0.0)));
    for _ in 0..20 {
        w.step(1.0);
    }
    let now = w.herd_pos(herd, w.time);
    assert!(now.dist(w.squad.pos) > 100.0, "they let the squad walk up to {} m", now.dist(w.squad.pos));
    assert_eq!(w.herd_doing(herd), "running");
    // ...and come back to their round when it has gone.
    w.teleport_squad(V2::new(9_000.0, 4_000.0));
    let w = run(w, 3.0, 60.0);
    let h = &w.animals.herds[herd as usize];
    assert!(w.herd_pos(herd, w.time).dist(h.round_pos(w.time)) < 1.0, "they never settled");
}

#[test]
fn fish_territory_and_dive_danger_can_be_asked_for() {
    let mut w = worldgen::generate(1);
    // Fish: a stock per coastal region; a catch thins it and it grows back.
    let r = w.fishing_region(w.squad.pos).expect("the squad starts in a harbour town");
    let before = w.fish_stock(r, Sp::Silverling, w.time);
    assert!(before > 50.0);
    let got = w.catch_fish(r, Sp::Silverling, 1e9);
    assert!((got - before * 0.5).abs() < 0.01, "never more than half at one go");
    let after = w.fish_stock(r, Sp::Silverling, w.time);
    assert!((after - (before - got)).abs() < 0.01);
    assert_eq!(w.catch_fish(r, Sp::Ribbonback, 0.0), 0.0);
    let inland = region_of(V2::new(12_000.0, 9_000.0));
    assert_eq!(w.fish_stock(inland, Sp::Silverling, w.time), 0.0);
    assert_eq!(w.catch_fish(inland, Sp::Silverling, 10.0), 0.0);
    // The Deepcoil: nothing on land, something at sea, and a spot's danger is fixed.
    assert_eq!(w.dive_danger(w.squad.pos), 0.0);
    let sea: Vec<f32> = (0..40).map(|k| w.dive_danger(V2::new(400.0, 500.0 + k as f32 * 500.0))).collect();
    assert!(sea.iter().all(|d| (0.0..=1.0).contains(d)) && sea.iter().any(|d| *d > 0.3) && sea.iter().any(|d| *d < 0.3), "{sea:?}");
    assert_eq!(w.dive_danger(V2::new(400.0, 5_000.0)), worldgen::generate(1).dive_danger(V2::new(400.0, 5_000.0)));
    // Cragmaw: its territory covers its home ground and not the far coast.
    let maw = w.animals.herds.iter().find(|h| h.sp == Sp::Cragmaw).expect("a Cragmaw");
    assert_eq!(w.cragmaw_territory(maw.home.add(V2::new(50.0, 50.0))), Some(maw.id));
    assert_eq!(w.cragmaw_territory(w.squad.pos), None);
    // Wild Turiyu are flagged for whoever makes the law.
    assert!(Sp::WildTuriyu.def().protected && !Sp::Brushleaper.def().protected);
    let w = run(w, 30.0 * 24.0, HOUR);
    assert!(w.fish_stock(r, Sp::Silverling, w.time) > after * 1.5, "the stock should grow back");
}

#[test]
fn travellers_beaten_by_animals_are_not_robbed_and_a_called_off_stalk_comes_to_nothing() {
    let mut w = worldgen::generate(7);
    // By day, a caravan out on the open road.
    step_to(&mut w, 8.0 * HOUR);
    let on_the_road = |w: &World| {
        let t = w.time + 90.0;
        w.groups
            .iter()
            .filter(|g| g.cargo.as_ref().map(|c| !c.robbed && c.amount > 0.0).unwrap_or(false) && g.ends > t + HOUR && !g.hostile)
            .filter(|g| w.settlements.iter().all(|s| s.pos.dist(g.position_at(t)) > s.radius() + 300.0))
            .map(|g| g.id)
            .min()
    };
    let mut tries = 0;
    let caravan = loop {
        let day = phase(w.time + 90.0) == Phase::Day && phase(w.time + 600.0) == Phase::Day;
        if let (true, Some(g)) = (day, on_the_road(&w)) {
            break g;
        }
        w.step(300.0);
        tries += 1;
        assert!(tries < 12 * 24 * 10, "ten days and never a caravan on the open road by day");
    };
    let t = w.time;
    let carried = w.group(caravan).unwrap().cargo.clone().unwrap();
    // The Cragmaw comes down on them. (Put on their trail by hand: the test
    // is about what follows, not about how it found them.)
    let maw = w.animals.herds.iter().find(|h| h.sp == Sp::Cragmaw && h.alive(t) > 0).expect("a Cragmaw").id;
    let strike = t + 60.0;
    let attacks = w.animals.attacks.len();

    // First, a stalk that has been called off: the strike set for it finds
    // nothing to do.
    w.animals.pending.push(Due { t: strike, what: What::Strike, herd: maw, victim: caravan, battle: 0 });
    step_to(&mut w, strike + 30.0);
    assert_eq!(w.animals.attacks.len(), attacks, "a strike with no stalk behind it went ahead");
    assert!(w.animals.frays.is_empty());

    // Now a live one.
    let t = w.time;
    let strike = t + 60.0;
    w.animals.herds[maw as usize].ready_at = 0.0;
    w.animals.herds[maw as usize].hunt = Some(Hunt { victim: caravan, seen: t, off: V2::new(30.0, 0.0), strike });
    w.animals.pending.push(Due { t: strike, what: What::Strike, herd: maw, victim: caravan, battle: 0 });
    step_to(&mut w, strike + 1.0);
    let rec = w.animals.attacks.iter().find(|a| a.herd == maw && a.victim == Some(caravan)).expect("the Cragmaw's attack").clone();
    assert!(rec.animals_won, "a caravan beat a Cragmaw: {rec:?}");
    assert!(w.animals.herds[maw as usize].hunt.is_none());
    // While it's fighting, nobody can set about it a second time. (A lone
    // traveller can be finished in under a second: then it's already over.)
    if !rec.over {
        assert!(w.herd_in_fray(maw));
    }
    let mut tries = 0;
    while !w.animals.attacks.iter().any(|a| a.battle == rec.battle && a.over) {
        w.step(30.0);
        tries += 1;
        assert!(tries < 2000, "the fight never ended");
    }
    assert!(!w.herd_in_fray(maw) && w.npc_fights.iter().all(|f| f.id != rec.battle));
    // Beaten, but not robbed: animals take nothing.
    let g = w.group(caravan).expect("the caravan is still a party on the road");
    if !g.members.is_empty() {
        let now = g.cargo.clone().expect("the caravan's goods");
        assert!(!now.robbed, "a Cragmaw robbed a caravan");
        assert_eq!((now.amount, now.coin), (carried.amount, carried.coin), "the goods didn't survive the mauling");
    }
    assert_eq!(w.animals.stats.people_killed as usize, w.animals.attacks.iter().map(|a| a.people_killed as usize).sum::<usize>());
}
