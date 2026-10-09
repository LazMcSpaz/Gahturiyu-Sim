//! Actors and opportunities: the storyteller, townsfolk crime, the ring,
//! and guard contracts.

use gahturiyu_sim::sim::{
    chances::{self, Chance, OppState},
    history::Deed,
    jobs::Job,
    ring::Move,
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, secs: f64) {
    let n = (secs / HOUR).round() as usize;
    for _ in 0..n {
        w.step(HOUR);
    }
}

fn set_drama(w: &mut World, d: f32) {
    for tl in &mut w.society.towns {
        tl.drama = d;
    }
}

/// Run on to just after the next dawn (06:00).
fn to_dawn(w: &mut World) {
    let h = (w.time.rem_euclid(DAY) / HOUR).round() as i64;
    let to = match (6 - h).rem_euclid(24) {
        0 => 24,
        n => n,
    } + 2;
    run(w, to as f64 * HOUR);
}

#[test]
fn a_household_whose_earner_is_laid_up_ends_up_stealing() {
    let mut w = worldgen::generate(1);
    set_drama(&mut w, 0.8);
    // A pair keeping house together, one with a post and little put by.
    let h = (0..w.society.households.len())
        .filter(|&h| {
            let hh = &w.society.households[h];
            hh.members.len() == 2 && hh.members.iter().all(|&m| !w.people[m as usize].in_squad) && w.society.lives[hh.members[0] as usize].job.is_post()
        })
        .min_by(|&a, &b| w.society.households[a].purse.coin.total_cmp(&w.society.households[b].purse.coin))
        .unwrap();
    let (earner, other) = (w.society.households[h].members[0], w.society.households[h].members[1]);
    // The other is bold and not over-honest.
    w.society.lives[other as usize].habits.honour = 0.1;
    w.people[other as usize].traits.boldness = 0.9;
    w.people[other as usize].traits.sociability = 0.2;
    w.society.households[h].purse.coin = 0.0;
    let town = w.people[other as usize].home.unwrap();
    let mut stole = false;
    for _ in 0..24 * 25 {
        // Kept laid up.
        let t = w.time;
        let pp = &mut w.people[earner as usize];
        let base = pp.stats.clone();
        let hp: Vec<f32> = (0..gahturiyu_sim::sim::body::PARTS.len()).map(|k| base.max_hp(gahturiyu_sim::sim::body::PARTS[k]) * 0.2).collect();
        let mut arr = pp.wounds.hp_at(&base, t);
        arr.copy_from_slice(&hp);
        pp.wounds.set(&base, &arr, t);
        w.step(HOUR);
        if w.events(town).iter().any(|e| e.deed == Deed::Theft && e.actor == Some(other)) {
            stole = true;
            break;
        }
    }
    assert!(w.society.households[h].purse.debt() > 0.0 || w.society.households[h].purse.coin > 0.0, "they borrowed (or have stolen enough to get by)");
    assert!(stole, "and in the end, they stole");
}

#[test]
fn too_many_outside_thefts_make_the_ring_act_against_the_thief() {
    let mut w = worldgen::generate(1);
    set_drama(&mut w, 0.8);
    to_dawn(&mut w);
    let town = w.society.rings.first().expect("a ring").town;
    let members = w.ring(town).unwrap().members.clone();
    let folk: Vec<u32> = w.settlements[town as usize].residents.iter().copied().filter(|p| !members.contains(p) && !w.people[*p as usize].in_squad).collect();
    let thief = folk[7];
    for k in 0..4 {
        let t = w.time;
        w.note(Deed::Theft, Some(thief), Some(folk[20 + k]), town, t, true);
    }
    run(&mut w, 2.0 * DAY);
    let r = w.ring(town).unwrap();
    let acted = matches!(r.last, Some((Move::Example { thief: x, .. }, _)) if x == thief);
    assert!(acted, "the ring made an example of them: {:?}", r.last);
    let against = w.events(town).iter().any(|e| e.victim == Some(thief) && matches!(e.deed, Deed::Beating | Deed::TurnedIn | Deed::Arrest)) || w.society.opps.iter().any(|o| o.kind == Chance::Kill && o.target == Some(thief));
    assert!(against, "and it did something about them");
}

#[test]
fn a_robbed_merchant_hires_a_guard_who_stands_there_and_is_paid() {
    let mut w = worldgen::generate(1);
    set_drama(&mut w, 0.9);
    to_dawn(&mut w);
    // A merchant with a stall and money put by.
    let merchant = (0..w.people.len() as u32).find(|&p| w.society.lives[p as usize].job == Job::Merchant && w.society.lives[p as usize].place.is_some() && w.people[p as usize].home.is_some() && !w.people[p as usize].in_squad).unwrap();
    let town = w.people[merchant as usize].home.unwrap();
    let hh = w.society.lives[merchant as usize].household.unwrap();
    w.society.households[hh as usize].purse.coin = 800.0;
    let thief = *w.settlements[town as usize].residents.iter().find(|&&p| p != merchant && !w.people[p as usize].in_squad).unwrap();
    for _ in 0..2 {
        let t = w.time;
        w.note(Deed::Theft, Some(thief), Some(merchant), town, t, true);
    }
    run(&mut w, DAY);
    let opp = w.society.opps.iter().find(|o| o.kind == Chance::Guard && o.asker == merchant && o.state == OppState::Open).map(|o| o.id).expect("the merchant wants a guard");
    let m = w.squad.members[0];
    assert!(w.take_opportunity(opp, m).is_some());
    let c = w.contract_of(m).unwrap().clone();
    let at = w.contract_pos(&c);
    // Wait for the next day's shift, standing nearby.
    let hour = |w: &World| (w.time.rem_euclid(DAY) / HOUR) as f32;
    run(&mut w, HOUR);
    while !(hour(&w) + 1.0 >= c.hours.0 && hour(&w) < c.hours.0) || World::day_of(w.time) as f64 * DAY < c.until - chances::GUARD_DAYS as f64 * DAY {
        w.step(HOUR / 2.0);
    }
    w.teleport_squad(at.add(gahturiyu_sim::sim::geo::V2::new(30.0, 0.0)));
    let mut there = false;
    for _ in 0..(2.0 * HOUR / 1.0) as usize {
        w.step(1.0);
        if w.member_pos(0).dist(at) <= chances::AT_POST {
            there = true;
            break;
        }
    }
    assert!(there, "sent to the shop when the shift began");
    // A thief who tries it now is caught.
    let t = w.time;
    w.steal(thief, Some(merchant), None, town, t);
    assert!(w.events(town).iter().any(|e| e.deed == Deed::Caught && e.actor == Some(m) && e.victim == Some(thief)));
    // Stand the day out, and be paid at dawn.
    let coin = gahturiyu_sim::sim::items::id("coin");
    let before = w.squad_count(coin);
    for _ in 0..(24.0 * HOUR / 30.0) as usize {
        w.step(30.0);
        let h = (w.time.rem_euclid(DAY) / HOUR) as f32;
        if h >= c.hours.0 + 0.5 && h < c.hours.1 - 0.5 {
            assert!(w.member_pos(0).dist(at) <= chances::AT_POST + 2.0, "stands there in its hours");
        }
    }
    eprintln!("contract now {:?}, time h {}", w.contract_of(m), (w.time.rem_euclid(DAY) / HOUR));
    assert!(w.squad_count(coin) > before, "paid for the day");
    assert_eq!(w.contract_of(m).unwrap().days_paid, 1);
}

#[test]
fn the_storyteller_keeps_it_calm() {
    let mut w = worldgen::generate(1);
    let mut needy = std::collections::BTreeSet::new();
    let mut acted = std::collections::BTreeSet::new();
    for _ in 0..12 * 24 {
        w.step(HOUR);
        for t in 0..w.settlements.len() as u16 {
            assert!(w.stories_in(t).len() <= w.story_cap(t), "never past the cap");
        }
        for (p, m) in w.society.minds.iter().enumerate() {
            if m.needs.iter().take(4).any(|&x| x > 0.2) {
                needy.insert(p as u32);
            }
        }
        for s in &w.society.stories {
            if needy.contains(&s.who) && !matches!(s.plot, gahturiyu_sim::sim::stories::Plot::Ring { .. }) {
                acted.insert(s.who);
            }
        }
    }
    assert!(!acted.is_empty(), "some act");
    eprintln!("{} of {} with needs acted", acted.len(), needy.len());
    assert!(acted.len() * 4 < needy.len(), "most with needs never act: {} of {}", acted.len(), needy.len());
}

#[test]
fn less_drama_means_fewer_storylines() {
    let count = |d: f32| {
        let mut w = worldgen::generate(1);
        set_drama(&mut w, d);
        run(&mut w, 8.0 * DAY);
        w.society.next_story
    };
    let (calm, wild) = (count(0.2), count(0.7));
    assert!(calm < wild, "calm {calm}, eventful {wild}");
}
