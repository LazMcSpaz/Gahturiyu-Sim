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

/// Stand by until the chase is over; true if it ended with the thief caught.
fn caught_in_the_end(w: &mut World) -> bool {
    let mut n = 0;
    while !w.pursuits.is_empty() && n < 20_000 {
        w.step(0.25);
        n += 1;
    }
    assert!(w.pursuits.is_empty(), "the chase ended");
    w.log.iter().any(|l| l.1.contains("catches"))
}

/// NM-20: stepping into the house next door doesn't shake the watch off:
/// a door close by where the thief was last seen is tried.
#[test]
fn nm20_the_watch_follows_through_the_next_door() {
    use gahturiyu_sim::sim::buildings::door_of;
    let (mut w, town, me, folk) = in_town();
    // Seen at it in the street, 25 m from a house that can be walked into.
    let s = w.settlements[town as usize].clone();
    let mut doors: Vec<_> = (0..s.buildings.len() as u16).filter_map(|i| door_of(&s, i)).filter(|d| !w.is_locked(d.id)).collect();
    doors.sort_by(|a, b| a.outside.dist(s.pos).total_cmp(&b.outside.dist(s.pos)));
    let d = doors.first().expect("an open door in town").clone();
    let out = d.outside.sub(d.inside);
    let from = d.outside.add(out.scale(25.0 / out.len().max(0.01)));
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = from;
    w.squad.goal[k] = from;
    w.squad.route[k].clear();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    // And in they go.
    w.order_members(&[me], d.centre);
    for _ in 0..600 {
        if w.squad.inside[k] == Some(d.id) || w.pursuits.is_empty() {
            break;
        }
        w.step(0.25);
    }
    // (Caught on the way in is the watch doing its job too.)
    assert!(w.squad.inside[k] == Some(d.id) || w.log.iter().any(|l| l.1.contains("catches")), "got indoors");
    assert!(caught_in_the_end(&mut w), "indoors next door, the thief slipped the watch: {:?}", w.log.iter().take(4).collect::<Vec<_>>());
    assert!(w.bounty.is_empty());
}

/// NM-20: nor does walking a hundred metres down the street in broad day,
/// with people about to point the way.
#[test]
fn nm20_the_watch_follows_down_the_street() {
    let (mut w, town, me, folk) = in_town();
    assert!(told(&mut w, town, me, &folk), "somebody tells");
    let from = w.person_pos(me);
    let s = w.settlements[town as usize].clone();
    // 110 m off, still in town, out of doors.
    let to = (0..16)
        .map(|i| {
            let a = i as f32 / 16.0 * std::f32::consts::TAU;
            from.add(V2::new(a.cos(), a.sin()).scale(110.0))
        })
        .filter(|&p| p.dist(s.pos) < s.reach && w.building_at(p).is_none() && !w.terrain.is_sea(p))
        .min_by(|a, b| a.dist(s.pos).total_cmp(&b.dist(s.pos)))
        .expect("a spot down the street");
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = to;
    w.squad.goal[k] = to;
    w.squad.route[k].clear();
    assert!(caught_in_the_end(&mut w), "110 m down the street, the thief slipped the watch: {:?}", w.log.iter().take(4).collect::<Vec<_>>());
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

/// NM-28: a crime seen with nobody to come running still says something
/// follows, and what's owed can be looked up afterwards.
#[test]
fn nm28_a_bounty_posted_says_so_and_can_be_looked_up() {
    let mut w = worldgen::generate(1);
    assert!(w.wanted_lines().is_empty(), "nothing owed at the start");
    // A town with no guard on watch just now (some hour of the night).
    let mut found = None;
    'hours: for _ in 0..48 {
        w.step(HOUR / 2.0);
        let t = w.time;
        for s in 0..w.settlements.len() as u16 {
            let guards = w.settlements[s as usize].residents.iter().filter(|&&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard && w.at_work(p, t)).count();
            if guards == 0 && w.settlements[s as usize].residents.len() > 40 {
                found = Some(s);
                break 'hours;
            }
        }
    }
    let town = found.expect("a town with nobody on watch at some hour");
    w.teleport_squad(w.settlements[town as usize].pos);
    let me = w.squad.members[0];
    let at = w.person_pos(me);
    // (No witness named: the town saw it for itself, so it is told.)
    w.wrong_seen(me, town, Wrong::Theft, 40.0, "Seen stealing!".into(), None, None, at);
    assert!(w.pursuits.is_empty(), "nobody on watch to send");
    let owed = w.bounty.get(&town).copied().unwrap_or(0.0);
    assert!(owed >= 40.0, "a bounty instead: {owed}");
    let alert = w.alerts.last().cloned().unwrap_or_default();
    assert!(alert.contains("Seen stealing!") && alert.contains("watch will hear"), "the line the player sees says it isn't over: {alert:?}");
    let name = w.settlements[town as usize].name.clone();
    assert_eq!(w.wanted_lines(), vec![format!("Wanted in {name}: {owed:.0} coin owed.")]);
    // Paid off, it's gone from the list.
    w.bounty.insert(town, 0.0);
    assert!(w.wanted_lines().is_empty());
}

fn guards_of(w: &World, town: usize) -> Vec<PersonId> {
    w.settlements[town].residents.iter().copied().filter(|&p| !w.people[p as usize].dead && w.life(p).job == Job::Guard).collect()
}

/// NM-40: a town with two guards or more keeps a watch through the night
/// (it used to take three, and the whole watch sat down to eat at 02:00);
/// and the night watch walks the town instead of standing at the post.
#[test]
fn nm40_someone_keeps_watch_at_night_and_walks_the_town() {
    let mut w = worldgen::generate(3);
    let towns: Vec<usize> = (0..w.settlements.len()).filter(|&s| guards_of(&w, s).len() >= 2).collect();
    assert!(towns.len() > 10);
    // The second night, at one, two and half past three in the morning.
    let mut dark = Vec::new();
    for h in [25.0, 26.25, 27.5] {
        while w.time < 24.0 * HOUR + h * HOUR {
            w.step(300.0);
        }
        let t = w.time;
        for &s in &towns {
            if !guards_of(&w, s).iter().any(|&p| w.at_work(p, t)) {
                dark.push((w.settlements[s].name.clone(), h - 24.0));
            }
        }
    }
    // (A town whose one night guard has the night off is dark that night;
    // that's a rest day, not a town without a watch.)
    eprintln!("{} towns with two guards or more; unwatched: {dark:?}", towns.len());
    for h in [1.0, 2.25, 3.5] {
        let n = dark.iter().filter(|d| d.1 == h).count();
        assert!(n * 10 <= towns.len() * 3, "at {h} in the morning {n} of {} towns have nobody on watch: {dark:?}", towns.len());
    }
    // Someone on the night watch is out walking: they move, they stay in
    // town, and where they are is the same whenever it's asked.
    let t = w.time;
    let (town, walker) = towns.iter().find_map(|&s| guards_of(&w, s).into_iter().find(|&p| w.life(p).shift == 1 && w.at_work(p, t)).map(|p| (s, p))).expect("a night guard on watch");
    let here = w.person_pos(walker);
    assert_eq!(here, w.person_pos(walker));
    let (centre, reach) = (w.settlements[town].pos, w.settlements[town].reach);
    let mut far = 0.0f32;
    for _ in 0..8 {
        w.step(300.0);
        let at = w.person_pos(walker);
        if w.at_work(walker, w.time) {
            far = far.max(at.dist(here));
            assert!(at.dist(centre) <= reach + 60.0, "the watch keeps to the town: {:.0} m from its middle", at.dist(centre));
        }
    }
    assert!(far > 15.0, "in forty minutes the night watch moved {far:.0} m");
}

/// NM-40: with nobody at the post, a witness fetches a guard who is up
/// and close by, though off duty.
#[test]
fn nm40_a_guard_near_by_comes_though_off_duty() {
    let mut w = worldgen::generate(3);
    // Mid-afternoon: the night watch is up and at home, off duty.
    while w.time < 15.5 * HOUR {
        w.step(300.0);
    }
    let t = w.time;
    let found = (0..w.settlements.len()).find_map(|s| guards_of(&w, s).into_iter().find(|&p| w.life(p).shift == 1 && !w.at_work(p, t) && !w.is_indoors_asleep(p) && w.is_about(p, t)).map(|p| (s, p)));
    let (town, off) = found.expect("a night guard up and off duty in the afternoon");
    // The day watch is laid out cold: nobody at the post to fetch.
    for g in guards_of(&w, town) {
        if g != off {
            let p = &mut w.people[g as usize];
            let max = p.stats.max_hp(gahturiyu_sim::sim::body::Part::Torso);
            p.wounds.lost = p.wounds.lost_at(t);
            p.wounds.lost[1] = max + 20.0;
            p.wounds.at = t;
        }
    }
    assert!(!guards_of(&w, town).iter().any(|&p| w.at_work(p, t)), "nobody on watch");
    let near = w.person_pos(off).add(V2::new(20.0, 0.0));
    w.teleport_squad(near);
    let me = w.squad.members[0];
    w.wrong_seen(me, town as u16, Wrong::Theft, 30.0, "Seen stealing!".into(), None, None, near);
    assert_eq!(w.pursuits.iter().map(|p| p.guard).collect::<Vec<_>>(), vec![off], "the guard 20 m away is fetched: {:?}", w.log.iter().take(3).collect::<Vec<_>>());
    assert!(w.bounty.is_empty());
}
