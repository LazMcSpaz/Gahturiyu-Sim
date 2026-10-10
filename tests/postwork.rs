//! Town work (a post going in town, taken by a squad member): the wage is
//! known before it's taken, the worker walks to the place at the shift's
//! start (even from their bedroll), and a day there pays like honest work.

use gahturiyu_sim::sim::{
    chances::AT_POST,
    world::{DAY, HOUR},
    worldgen, World,
};

fn world() -> World {
    let mut w = worldgen::generate(1);
    for _ in 0..60 {
        w.step(600.0);
    }
    w
}

fn take(w: &mut World) -> (u32, u16, u32) {
    let town = 0u16;
    let (job, place) = w.vacant_posts(town)[0];
    let wage = w.post_wage(job);
    assert!(wage >= 15, "a day's wage of {wage} is worth turning up for");
    let who = w.squad.members[0];
    let at = w.society.towns[town as usize].places[place as usize].pos;
    w.teleport_squad(at.add(gahturiyu_sim::sim::geo::V2::new(120.0, 40.0)));
    assert!(w.take_post_work(who, town, job, place));
    assert_eq!(w.contract_of(who).unwrap().pay.round() as u16, wage, "the wage named is the wage paid");
    (who, wage, place as u32)
}

fn run_to(w: &mut World, t: f64) {
    while w.time < t {
        w.step((t - w.time).min(5.0).max(0.01));
    }
}

#[test]
fn the_worker_walks_to_work_and_is_paid_the_wage_named() {
    let mut w = world();
    let (who, wage, _) = take(&mut w);
    let c = w.contract_of(who).unwrap().clone();
    // Asleep in the street the night before: the shift still gets them up.
    run_to(&mut w, c.first_day as f64 * DAY + 1.0 * HOUR);
    let k = w.squad.index(who).unwrap();
    if !w.squad.resting[k] {
        w.order_rest(&[who]);
    }
    assert!(w.squad.resting[k], "lying down");
    run_to(&mut w, c.first_day as f64 * DAY + 10.0 * HOUR);
    assert!(w.member_pos(k).dist(w.contract_pos(&c)) <= AT_POST, "at work by mid-morning");
    assert!(w.log.iter().any(|(_, l)| l.contains("sets off for")), "the walk to work is in the news");
    assert!(w.work_lines().iter().any(|l| l.contains(&w.name_of(who)) && l.contains("coin a day")), "the journal names the worker and the wage");
    let coin0 = w.count_of(who, "coin");
    run_to(&mut w, (c.first_day + 1) as f64 * DAY + 6.5 * HOUR);
    let got = w.count_of(who, "coin") - coin0;
    assert_eq!(got, wage, "paid the day's wage at dawn");
}

#[test]
fn a_woodcutter_says_where_the_woodlot_is() {
    use gahturiyu_sim::sim::{dialogue::Topic, jobs::{Job, PlaceKind}};
    let mut w = world();
    let wc = w
        .settlements
        .iter()
        .flat_map(|s| s.residents.iter().copied())
        .find(|&p| w.life(p).job == Job::Woodcutter && w.workplace_of(p).is_some_and(|wp| wp.kind == PlaceKind::Woodlot) && !w.is_indoors_asleep(p))
        .expect("a woodcutter up and about");
    let at = w.person_pos(wc);
    w.teleport_squad(at.add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0)));
    w.squad.at[0] = at.add(gahturiyu_sim::sim::geo::V2::new(1.0, 0.0));
    assert!(w.order_talk(w.squad.members[0], wc));
    for _ in 0..20 {
        w.step(0.1);
    }
    assert!(w.talk.is_some());
    w.ask(Topic::Background);
    let said = w.talk.as_ref().unwrap().lines.last().unwrap().1.clone();
    eprintln!("{said}");
    assert!(said.contains("woodlot") && said.contains(" m ") && said.contains("of here"), "{said}");
}
