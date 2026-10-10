//! A crime seen must be told, and the thief caught (Laz, 2026-10-09): most
//! witnesses tell the watch, some don't; a guard runs the thief down, or
//! loses them and it becomes a bounty.

use gahturiyu_sim::sim::{
    containers::Owner,
    geo::V2,
    jobs::Job,
    law::Wrong,
    person::PersonId,
    worldgen,
    world::HOUR,
    World,
};

/// World 1 at mid-morning, with the squad stood in a town that has a guard on
/// watch, member 0 beside a townsperson out of doors (who isn't a guard).
fn in_town() -> (World, u16, PersonId, Vec<PersonId>) {
    let mut w = worldgen::generate(1);
    while w.time < 10.0 * HOUR {
        w.step(60.0);
    }
    let t = w.time;
    let town = (0..w.settlements.len() as u16)
        .find(|&s| {
            let guards = w.settlements[s as usize].residents.iter().filter(|&&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard && w.at_work(p, t)).count();
            guards > 0 && w.settlements[s as usize].residents.len() > 40
        })
        .expect("a town with its watch out");
    w.teleport_squad(w.settlements[town as usize].pos);
    w.step(0.5);
    let me = w.squad.members[0];
    let folk: Vec<PersonId> = w.residents_in_band1(town).into_iter().filter(|&p| w.building_at(w.person_pos(p)).is_none() && w.life(p).job != Job::Guard).collect();
    assert!(!folk.is_empty());
    let k = w.squad.index(me).unwrap();
    let at = w.person_pos(folk[0]).add(V2::new(1.5, 0.0));
    w.squad.at[k] = at;
    w.squad.goal[k] = at;
    w.squad.route[k].clear();
    (w, town, me, folk)
}

/// Report a theft from the witness's own household until one is told (each
/// witness gets their own roll).
fn told(w: &mut World, town: u16, me: PersonId, folk: &[PersonId]) -> bool {
    for &f in folk {
        let Some(h) = w.life(f).household else { continue };
        let at = w.person_pos(me);
        w.wrong_seen(me, town, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(f), Some(Owner::Household(h)), at);
        if !w.pursuits.is_empty() {
            return true;
        }
    }
    false
}

#[test]
fn most_tell_the_watch_but_not_everyone() {
    let (mut w, _, me, folk) = in_town();
    let mut odds: Vec<f32> = folk.iter().map(|&f| w.report_chance(f, me, None)).collect();
    odds.sort_by(|a, b| a.total_cmp(b));
    let mean = odds.iter().sum::<f32>() / odds.len() as f32;
    eprintln!("{} witnesses; odds {:.2}..{:.2}, mean {mean:.2}", odds.len(), odds[0], odds[odds.len() - 1]);
    assert!(mean > 0.6, "most would tell");
    // Their own household's things: they almost always tell.
    let f = folk[0];
    let h = w.life(f).household.unwrap();
    assert!(w.report_chance(f, me, Some(Owner::Household(h))) > 0.95);
    // A friend of the thief is less likely to.
    let before = w.report_chance(f, me, None);
    w.regard.insert(f, 60.0);
    assert!(w.report_chance(f, me, None) < before - 0.1, "a friend looks away");
    w.regard.remove(&f);
    // So is one of the town's thieves.
    if let Some(r) = w.society.rings.iter().position(|r| !r.members.is_empty()) {
        let thief = w.society.rings[r].members[0];
        let honest = w.report_chance(thief, me, None);
        w.society.rings[r].members.clear();
        assert!(w.report_chance(thief, me, None) > honest + 0.3, "thieves don't turn in thieves");
    }
}

#[test]
fn a_told_theft_sends_a_guard_who_catches_the_thief() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    let p = w.pursuits[0].clone();
    assert!(w.alerts.iter().any(|a| a.contains("fetch the watch")), "{:?}", w.alerts);
    assert!(w.bounty.is_empty(), "not a bounty while they're after you");
    let guard_was = w.person_pos(p.guard);
    // Stand still and wait.
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
        if w.chased_by(me).is_some() {
            assert_eq!(w.person_pos(p.guard), w.chase_pos(p.guard).unwrap(), "the guard is where the chase has them");
        }
    }
    eprintln!("caught after {:.0} s; the guard came {:.0} m", n as f64 * 0.25, guard_was.dist(w.person_pos(me)));
    assert!(w.pursuits.is_empty());
    assert!(w.log.iter().any(|l| l.1.contains("catches")), "{:?}", w.log);
    assert!(w.bounty.is_empty(), "caught, so no bounty");
}

#[test]
fn a_thief_who_gets_away_has_a_bounty_instead() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk));
    // Off and away before the watch arrives.
    let far = w.settlements[town as usize].pos.add(V2::new(w.settlements[town as usize].reach + 400.0, 0.0));
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = far;
    w.squad.goal[k] = far;
    w.squad.route[k].clear();
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.pursuits.is_empty());
    assert!(w.bounty.get(&town).copied().unwrap_or(0.0) > 0.0, "a bounty: {:?}", w.log);
    assert!(w.log.iter().any(|l| l.1.contains("slipped the watch")));
}

#[test]
fn keeping_quiet_means_nothing_follows() {
    let (mut w, town, me, folk) = in_town();
    // Someone who'd never tell: a close friend with no scruples.
    let mut quiet = 0;
    for &f in &folk {
        w.regard.insert(f, 100.0);
        let at = w.person_pos(me);
        let said = w.log.len() + w.alerts.len();
        let last = w.log.front().cloned();
        w.wrong_seen(me, town, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(f), None, at);
        if w.pursuits.is_empty() && w.bounty.is_empty() {
            quiet += 1;
            // Nothing comes of it, and nothing is said (BL-37).
            assert!(w.log.front().cloned() == last && w.log.len() + w.alerts.len() == said, "{:?}", w.log.front());
        }
        w.pursuits.clear();
        w.bounty.clear();
    }
    eprintln!("{quiet} of {} friends kept quiet", folk.len());
    assert!(quiet > 0, "friends look away sometimes");
}
