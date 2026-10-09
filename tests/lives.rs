//! Lives with stakes: purses, work, needs.

use gahturiyu_sim::sim::{
    jobs::Job,
    lives::{Need, Work},
    world::{DAY, HOUR},
    worldgen, World,
};

fn run(w: &mut World, secs: f64) {
    let n = (secs / HOUR).round() as usize;
    for _ in 0..n {
        w.step(HOUR);
    }
}

/// Knock someone about badly enough that they can't work.
fn lay_up(w: &mut World, p: u32) {
    let t = w.time;
    let pp = &mut w.people[p as usize];
    let base = pp.stats.clone();
    let mut hp = pp.wounds.hp_at(&base, t);
    for (k, x) in hp.iter_mut().enumerate() {
        *x = base.max_hp(gahturiyu_sim::sim::body::PARTS[k]) * 0.2;
    }
    pp.wounds.set(&base, &hp, t);
}

#[test]
fn households_earn_spend_and_some_owe() {
    let mut w = worldgen::generate(1);
    run(&mut w, 4.0 * DAY);
    let hh = &w.society.households;
    assert!(hh.iter().any(|h| h.purse.earned > 0.0), "people earn");
    assert!(hh.iter().all(|h| h.purse.coin >= 0.0), "purses never go below nothing (they borrow)");
    let working = w.society.minds.iter().filter(|m| m.work == Work::Working).count();
    assert!(working > 3000, "most people work: {working}");
}

#[test]
fn a_household_whose_earner_is_laid_up_falls_into_debt() {
    let mut w = worldgen::generate(1);
    // Someone who keeps house alone, with a post and not much put by.
    let h = (0..w.society.households.len())
        .filter(|&h| {
            let hh = &w.society.households[h];
            hh.members.len() == 1 && w.society.lives[hh.members[0] as usize].job.is_post() && !w.people[hh.members[0] as usize].in_squad
        })
        .min_by(|&a, &b| w.society.households[a].purse.coin.total_cmp(&w.society.households[b].purse.coin))
        .expect("someone living alone");
    let p = w.society.households[h].members[0];
    let job = w.society.lives[p as usize].job;
    // Kept laid up (wounds heal within the day otherwise).
    for _ in 0..8 * 24 {
        lay_up(&mut w, p);
        w.step(HOUR);
    }
    assert_eq!(w.mind(p).work, Work::Injured);
    assert_eq!(w.mind(p).lost.map(|l| l.0), Some(job), "their post went to someone else");
    assert_ne!(w.society.lives[p as usize].job, job);
    assert!(w.why_not_working(p).unwrap().contains("laid up"));
    let purse = &w.society.households[h].purse;
    assert!(purse.debt() > 0.0, "and they're borrowing: {:?}", purse);
    assert!(w.mind(p).needs()[Need::Money as usize] > 0.1, "and worried about it");
    assert_eq!(w.household_need(h as u32).0, Need::Money);
}

#[test]
fn the_out_of_work_find_work_again() {
    let mut w = worldgen::generate(1);
    let p = (0..w.people.len() as u32).find(|&p| w.society.lives[p as usize].job == Job::Labourer && w.people[p as usize].home.is_some()).unwrap();
    w.society.lives[p as usize].job = Job::None;
    w.society.lives[p as usize].place = None;
    run(&mut w, DAY + HOUR);
    assert_ne!(w.society.lives[p as usize].job, Job::None, "taken on again (posts or labouring)");
}

#[test]
fn honour_is_rolled_per_person() {
    let w = worldgen::generate(1);
    let hs: Vec<f32> = w.society.lives.iter().map(|l| l.habits.honour).collect();
    let lo = hs.iter().filter(|&&h| h < 0.35).count();
    let hi = hs.iter().filter(|&&h| h > 0.8).count();
    assert!(lo > 100 && hi > 100, "a spread: {lo} low, {hi} high");
}
