//! Society: customs from culture blends, households and jobs, routines
//! looked up from the clock, food paths and money.

use gahturiyu_sim::sim::{
    culture::Cooking,
    jobs::{Job, Service},
    race::Race,
    routine::Doing,
    society::DAWN,
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, hours: f64, step: f64) {
    let n = (hours * HOUR / step).round() as usize;
    for _ in 0..n {
        w.step(step);
    }
}

/// Step until the clock reads `hour` (on the next day if it's past).
fn until_hour(w: &mut World, hour: f64, step: f64) {
    let now = w.time.rem_euclid(DAY) / HOUR;
    let wait = (hour - now).rem_euclid(24.0);
    run(w, wait, step);
}

#[test]
fn the_same_seed_gives_the_same_customs_and_routines_however_its_stepped() {
    let a = worldgen::generate(5);
    let b = worldgen::generate(5);
    for (ca, cb) in a.society.communities.iter().zip(&b.society.communities) {
        assert_eq!(ca.customs, cb.customs);
        assert_eq!(ca.blend, cb.blend);
    }
    assert_eq!(a.society.lives, b.society.lives);

    let (mut fine, mut coarse) = (a, b);
    run(&mut fine, 40.0, 3.0);
    run(&mut coarse, 40.0, HOUR);
    assert_eq!(fine.society.lives, coarse.society.lives);
    for (ca, cb) in fine.society.communities.iter().zip(&coarse.society.communities) {
        assert_eq!(ca.customs, cb.customs);
        assert_eq!(ca.food, cb.food);
    }
    // Everyone at home is in the same place doing the same thing.
    for s in &fine.settlements {
        for &p in &s.residents {
            assert_eq!(fine.doing_now(p), coarse.doing_now(p), "person {p} is doing something else");
            if fine.doing_now(p).is_some() {
                assert!(fine.person_pos(p).dist(coarse.person_pos(p)) < 0.01, "person {p} is somewhere else");
            }
        }
    }
    for (ta, tb) in fine.society.towns.iter().zip(&coarse.society.towns) {
        for (x, y) in ta.stock.iter().zip(&tb.stock) {
            assert!((x.base - y.base).abs() < 1e-3);
        }
    }
}

#[test]
fn customs_change_when_a_towns_mix_shifts() {
    let mut w = worldgen::generate(7);
    // A land community where the Qotiro are few.
    let (t, ci) = w
        .society
        .towns
        .iter()
        .enumerate()
        .map(|(t, tl)| (t as u16, tl.shore))
        .min_by(|a, b| w.society.communities[a.1 as usize].blend.share[1].total_cmp(&w.society.communities[b.1 as usize].blend.share[1]))
        .unwrap();
    let before = w.society.communities[ci as usize].customs;
    let share_before = w.society.communities[ci as usize].blend.share;
    // Hundreds of Qotiro settle there.
    let movers: Vec<u32> = w
        .people
        .iter()
        .filter(|p| p.race == Race::Qotiro && !p.in_squad && !p.bandit && p.home.is_some() && p.home != Some(t))
        .map(|p| p.id)
        .take(600)
        .collect();
    for p in movers {
        w.resettle(p, t);
    }
    // Nothing changes until the town takes stock at dawn...
    assert_eq!(w.society.communities[ci as usize].customs, before);
    run(&mut w, 26.0, HOUR);
    let c = &w.society.communities[ci as usize];
    assert!(c.blend.share[1] > share_before[1] + 0.3, "the blend should have moved: {:?} -> {:?}", share_before, c.blend.share);
    assert_ne!(c.customs, before, "a town gone mostly Qotiro should have taken up some new custom");
    // ...and the roll is the town's own: re-running the same history gives the same new customs.
}

#[test]
fn a_service_closes_when_its_worker_dies() {
    let mut w = worldgen::generate(3);
    // A town with exactly one healer.
    let (t, healer) = (0..w.settlements.len() as u16)
        .filter_map(|t| {
            let hs: Vec<u32> = w.settlements[t as usize].residents.iter().copied().filter(|&p| w.life(p).job == Job::Healer).collect();
            (hs.len() == 1).then(|| (t, hs[0]))
        })
        .next()
        .expect("some town has one healer");
    // Wait until they're at work.
    for _ in 0..(24 * 8) {
        if w.at_work(healer, w.time) {
            break;
        }
        run(&mut w, 0.25, 60.0);
    }
    assert!(w.at_work(healer, w.time), "the healer never went to work");
    assert!(w.service_open(t, Service::Healing));
    w.people[healer as usize].dead = true;
    assert!(!w.service_open(t, Service::Healing), "healing should close with the healer gone");
    // Until someone takes up the work at (the next) dawn.
    run(&mut w, 1.0, 60.0);
    assert!(!w.service_open(t, Service::Healing));
    until_hour(&mut w, DAWN as f64 + 0.5, 60.0);
    let new = w.settlements[t as usize].residents.iter().copied().find(|&p| !w.people[p as usize].dead && w.life(p).job == Job::Healer);
    assert!(new.is_some(), "a labourer should have taken the post");
    assert_eq!(w.service_workers(t, Service::Healing), 1);
}

#[test]
fn a_broken_food_path_leaves_the_town_short() {
    let mut w = worldgen::generate(1);
    let t = (0..w.settlements.len()).find(|&t| w.society.towns[t].stilts.is_some()).expect("a coastal town") as u16;
    let shore = w.society.towns[t as usize].shore;
    run(&mut w, 50.0, HOUR);
    let before = w.society.communities[shore as usize].food.clone();
    assert!(before.share[2] > 0.0 && before.suff[2] > 0.5, "the boats should be feeding the town: {before:?}");
    // The stilt village stops bringing the catch ashore.
    w.set_withdrawn(t, true);
    run(&mut w, 48.0, HOUR);
    let after = w.society.communities[shore as usize].food.clone();
    assert_eq!(after.suff[2], 0.0, "no boats, no catch");
    assert!(after.overall < before.overall - 0.1, "the town should go short: {} -> {}", before.overall, after.overall);
    assert_eq!(w.society.towns[t as usize].landed, 0.0);

    // A hearth town whose cooks all die.
    let mut w = worldgen::generate(1);
    let ci = (0..w.society.communities.len()).find(|&c| !w.society.communities[c].stilts && w.society.communities[c].customs.cooking == Cooking::Hearth).expect("a hearth town") as u32;
    let town = w.society.communities[ci as usize].town;
    run(&mut w, 26.0, HOUR);
    let fed = w.society.communities[ci as usize].food.suff[1];
    assert!(fed > 0.7, "the kitchens should be working: {fed}");
    // (Laid low just after dawn, before anyone can take their place.)
    until_hour(&mut w, DAWN as f64 + 0.5, HOUR / 4.0);
    for &p in &w.settlements[town as usize].residents.clone() {
        if w.life(p).job == Job::Cook {
            w.people[p as usize].dead = true;
        }
    }
    until_hour(&mut w, DAWN as f64 + 0.2, HOUR / 4.0);
    let starved = w.society.communities[ci as usize].food.suff[1];
    assert!(starved < fed - 0.4, "no cooks, no meals: {fed} -> {starved}");
}

#[test]
fn tide_keepers_start_later_each_day() {
    let w = worldgen::generate(1);
    // Someone keeping tide hours, working a normal day on two days running.
    let found = w.people.iter().filter(|p| !p.in_squad && p.home.is_some()).map(|p| p.id).find_map(|p| {
        if w.rhythm_of(p) != gahturiyu_sim::sim::culture::Rhythm::Tides || w.life(p).job != Job::Fisher {
            return None;
        }
        (3..30).find_map(|d| {
            let (a, b) = (w.day_plan(p, d), w.day_plan(p, d + 1));
            match (a.work, b.work) {
                (Some(x), Some(y)) if (4.5..12.0).contains(&x.0) && y.0 > x.0 => Some((x.0, y.0)),
                _ => None,
            }
        })
    });
    let (a, b) = found.expect("a fisher on tide hours");
    assert!((b - a - 0.84).abs() < 0.1, "tide hours should slide ~50 minutes a day: {a} -> {b}");
}

#[test]
fn people_sleep_at_night_and_work_by_day() {
    let mut w = worldgen::generate(2);
    until_hour(&mut w, 2.0, HOUR);
    let town = &w.settlements[0];
    let asleep = town.residents.iter().filter(|&&p| matches!(w.doing_now(p), Some((Doing::Asleep, _)))).count();
    let about = town.residents.iter().filter(|&&p| w.doing_now(p).is_some()).count();
    assert!(asleep as f32 > about as f32 * 0.6, "at 2 am most should be asleep: {asleep} of {about}");
    until_hour(&mut w, 10.0, HOUR);
    let town = &w.settlements[0];
    let working = town.residents.iter().filter(|&&p| matches!(w.doing_now(p), Some((Doing::Work | Doing::Rounds | Doing::Meal | Doing::Run, _)))).count();
    assert!(working > 10, "by mid-morning people should be at work: {working}");
}

#[test]
fn caravans_carry_real_cargo() {
    let mut w = worldgen::generate(7);
    let mut seen_arrival = false;
    for _ in 0..(24 * 4) {
        let before: Vec<(u32, bool)> = w.groups.iter().filter_map(|g| g.cargo.as_ref().map(|c| (g.id, c.delivered))).collect();
        run(&mut w, 1.0, HOUR / 2.0);
        for (gid, was) in before {
            if let Some(c) = w.group(gid).and_then(|g| g.cargo.as_ref()) {
                if c.delivered && !was {
                    seen_arrival = true;
                    assert_eq!(c.amount, 0.0, "the goods stay at market");
                }
                if c.robbed {
                    assert_eq!(c.amount + c.coin, 0.0, "robbed caravans keep nothing");
                }
            }
        }
    }
    assert!(w.stats.caravans > 5, "caravans should set out: {}", w.stats.caravans);
    assert!(seen_arrival, "some caravan should have reached its market");
}

#[test]
fn trade_and_exchange_need_someone_at_work() {
    let mut w = worldgen::generate(3);
    until_hour(&mut w, 11.0, HOUR / 2.0);
    let merchant = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Merchant && w.at_work(p, w.time)).expect("a merchant at work");
    let wares = w.for_sale(merchant);
    assert!(!wares.is_empty(), "an open stall has something on it");
    let (it, _, price) = wares[0];
    let have = w.squad_count(gahturiyu_sim::sim::items::id("coin"));
    let m0 = w.squad.members[0];
    w.people[m0 as usize].detail.as_mut().unwrap().gear.add(gahturiyu_sim::sim::items::id("coin"), 500);
    let town = w.people[merchant as usize].home.unwrap();
    let purse = w.purse_now(town);
    assert!(w.buy(merchant, it));
    assert_eq!(w.squad_count(gahturiyu_sim::sim::items::id("coin")), have + 500 - price);
    assert!((w.purse_now(town) - purse - price as f32).abs() < 1.0);
    // At 2 am the stall is shut.
    until_hour(&mut w, 2.0, HOUR / 2.0);
    if !w.at_work(merchant, w.time) {
        assert!(w.for_sale(merchant).is_empty());
        assert!(!w.buy(merchant, it));
    }
}
