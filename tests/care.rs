//! Paying to recover: a healer at work mends the squad's wounds for coin;
//! an innkeeper lets beds, where sleep mends faster than in the street.

use gahturiyu_sim::sim::{
    body::PARTS,
    condition::Shelter,
    dialogue::Topic,
    geo::V2,
    items,
    jobs::Job,
    person::PersonId,
    world::{DAY, HOUR},
    worldgen, World,
};

/// Run on until someone of this trade is at work; returns them.
fn at_work(w: &mut World, job: Job) -> PersonId {
    at_work_from(w, job, 0.0)
}

/// The same, at or after this hour of the day.
fn at_work_from(w: &mut World, job: Job, hour: f64) -> PersonId {
    for _ in 0..48 * 6 {
        let t = w.time;
        if (t.rem_euclid(DAY) / HOUR) < hour {
            w.step(600.0);
            continue;
        }
        if let Some(p) = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == job && w.at_work(p, t)) {
            return p;
        }
        w.step(600.0);
    }
    panic!("no {job:?} at work");
}

fn hurt(w: &mut World, m: PersonId, by: f32) {
    let t = w.time;
    let p = &mut w.people[m as usize];
    let hp: [f32; 6] = std::array::from_fn(|i| p.stats.max_hp(PARTS[i]) - if i == 0 { 0.0 } else { by });
    let stats = p.stats.clone();
    p.wounds.set(&stats, &hp, t);
}

fn talk_to(w: &mut World, npc: PersonId) {
    let at = w.person_pos(npc);
    w.teleport_squad(at.add(V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(V2::new(1.0, 0.0));
    assert!(w.order_talk(w.squad.members[0], npc));
    for _ in 0..20 {
        w.step(0.1);
    }
    assert!(w.talk.is_some(), "talking");
}

fn give_coin(w: &mut World, n: u16) {
    let m = w.squad.members[0];
    w.people[m as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), n);
}

#[test]
fn a_healer_mends_the_hurt_for_coin() {
    let mut w = worldgen::generate(1);
    let healer = at_work(&mut w, Job::Healer);
    talk_to(&mut w, healer);
    assert!(!w.topics().iter().any(|t| matches!(t, Topic::Treat(_))), "nothing to treat while no one's hurt");
    let (a, b) = (w.squad.members[1], w.squad.members[2]);
    hurt(&mut w, a, 20.0);
    hurt(&mut w, b, 10.0);
    let price = match w.topics().into_iter().find(|t| matches!(t, Topic::Treat(_))) {
        Some(Topic::Treat(p)) => p,
        _ => panic!("the healer offers to treat"),
    };
    assert!(price > 4);
    give_coin(&mut w, price);
    let coin = items::id("coin");
    let before = w.squad_count(coin);
    w.ask(Topic::Treat(price));
    assert_eq!(w.squad_count(coin), before - price, "paid");
    let t = w.time;
    for m in [a, b] {
        assert!(!w.people[m as usize].wounds.is_hurt(t), "mended");
    }
}

#[test]
fn a_rented_bed_mends_faster_than_the_street() {
    let mut w = worldgen::generate(1);
    // Taken of an evening, for the night.
    let keeper = at_work_from(&mut w, Job::Innkeeper, 20.0);
    talk_to(&mut w, keeper);
    let price = match w.topics().into_iter().find(|t| matches!(t, Topic::RentBeds(_))) {
        Some(Topic::RentBeds(p)) => p,
        _ => panic!("the innkeeper lets beds"),
    };
    assert_eq!(price as usize, 5 * w.squad.members.len());
    give_coin(&mut w, price);
    w.ask(Topic::RentBeds(price));
    w.ask(Topic::Goodbye);
    let m = w.squad.members[1];
    assert!(w.has_room(m));
    hurt(&mut w, m, 30.0);
    // Walk there and sleep.
    for _ in 0..(2.0 * HOUR / 2.0) as usize {
        w.step(2.0);
    }
    assert!(w.is_asleep(m), "lying down in the bed");
    let c = w.people[m as usize].cond.as_ref().unwrap();
    assert_eq!(c.shelter, Shelter::Bed, "in a bed");
    assert!(c.heal_rate() > gahturiyu_sim::sim::body::HEAL_PER_HOUR * gahturiyu_sim::sim::condition::HEAL_SLEEP_INDOORS);
    // Next day the room is given up.
    let until = w.rooms.iter().find(|r| r.who == m).unwrap().until;
    assert!(until > w.time && until < w.time + DAY + 12.0 * HOUR);
}
